//! # fruits_asset_loading
//!
//! Loads engine assets — materials, textures, meshes, audio clips, and prefabs — from disk on
//! demand and hands back cached handles so the rest of the engine can reference them by name.
//!
//! # How to use
//!
//! Every asset is identified by a **key**: a path, relative to the `assets/` directory, of a small
//! JSON *asset file* that declares an `asset_type` and points at the raw payload (image, `.obj`,
//! `.wav`). The loaders are the `get_or_load_*` family; each has a convenient `*_from_world`
//! variant that pulls the resources it needs straight out of the world. The asset storages they
//! cache into are registered by the engine's modules — `add_render_module_to` (part of
//! `add_defult_modules_to`) for materials, meshes, and textures, and `add_audio_module_to` for
//! audio — so enable those before loading.
//!
//! #### Loading a texture, material, or mesh
//!
//! Resolve a render asset to an [`AssetHandle`](fruits_asset_storage::AssetHandle) from inside the world. The handle can then be
//! stored on a component and rendered:
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! let texture = get_or_load_texture_from_world(
//!     app.ecs_mut().data_mut().resources_mut().as_mut(),
//!     "sprites/player.asset",
//! ).unwrap();
//!
//! let material = get_or_load_material_from_world(
//!     app.ecs_mut().data_mut().resources_mut().as_mut(),
//!     "materials/player.asset",
//! ).unwrap();
//!
//! let mesh = get_or_load_mesh_from_world(
//!     app.ecs_mut().data_mut().resources_mut().as_mut(),
//!     "meshes/player.asset",
//! ).unwrap();
//! ```
//!
//! Loading the same key twice returns the same cached handle rather than re-reading the file.
//!
//! #### Loading an audio clip
//!
//! Resolve an [`AssetHandle`](fruits_asset_storage::AssetHandle) for an audio clip so it can be attached to an `AudioSource`:
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! let clip = get_or_load_audio_clip_from_world(
//!     app.ecs_mut().data_mut().resources_mut().as_mut(),
//!     "Through space.asset",
//! ).unwrap();
//! ```
//!
//! #### Instantiating a prefab
//!
//! [`get_or_load_prefab_from_world`] loads a prefab and its referenced assets; [`instantiate_prefab`]
//! then spawns the prefab's entities and components into the world, returning the root entity:
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! let prefab = get_or_load_prefab_from_world(
//!     app.ecs_mut().data_mut().resources_mut().as_mut(),
//!     "prefabs/enemy.asset",
//! ).unwrap();
//!
//! let root = instantiate_prefab(app.ecs_mut().data_mut(), prefab).unwrap();
//! ```
//!
//! # How to maintain
//!
//! Every loader follows the same shape. `get_or_load_<asset>` first consults the asset's
//! [`AssetStorageResource`](fruits_asset_storage::AssetStorageResource) via `get_registered(key)`: if the key is registered *and* the handle
//! still resolves it returns the cached handle; if the key is registered but the handle was evicted
//! it unregisters the stale key and reloads. On a miss it reads `assets/<key>` from disk,
//! deserializes it, inserts the result into the storage, and registers the key against the new
//! handle. A failed read or parse returns `None` rather than panicking. The `<asset>_from_world`
//! wrappers exist only to fetch the required resources out of [`fruits_ecs::ResourcesHolderMut`]
//! through raw pointers and forward to the borrow-explicit `get_or_load_<asset>`.
//!
//! Asset files are JSON. Materials, textures, meshes, and audio clips are parsed with the
//! engine's own [`fruits_json`] parser; prefabs are parsed with `serde_json`. Each file carries an
//! `asset_type` discriminator that must match the loader (`"material"`, `"texture"`, `"mesh"`,
//! `"audio_clip"`, `"prefab"`), and most reference a raw payload by a further `assets/`-relative
//! path:
//!
//! - **Texture** — `raw_texture` names an image decoded by the `image` crate; the bytes are
//!   uploaded through `RenderApiResource::create_texture` with `FilterMode::Nearest`.
//! - **Material** — optional `is_lit`, `color`/`emission_color` (hex strings parsed by
//!   `parse_color_rgba_f32`), `space` (`"world"`/`"clip"`/`"window"`), `metallic`, `roughness`,
//!   and a `color_tex` key that is itself loaded as a texture. `alpha_threshold` defaults to `0.5`;
//!   when `is_transparent` is set the material's `alpha_threshold` is cleared to `None` instead.
//! - **Mesh** — `raw_mesh` names a Wavefront `.obj` parsed by [`fruits_wavefront`]. Faces are
//!   flattened into a non-indexed vertex list, then fixed up per the `coordinate_space`
//!   (`"right_hand_z_up"`/`"right_hand_z_back"`, otherwise left-hand Z-forward),
//!   `has_clockwise_winding`, `has_inverted_u`, and `has_inverted_v` flags before being uploaded
//!   with `RenderApiResource::create_mesh`.
//! - **Audio clip** — `raw_audio` names a `.wav` read by `hound`. Integer samples are normalized to
//!   `f32`, the buffer is forced to stereo, and it is resampled to the engine's sample rate when the
//!   file's rate differs.
//!
//! Prefabs are richer. `deserialize_prefab` turns the JSON into a [`fruits_prefab::Prefab`] of
//! entity ids mapped to component blobs. `load_prefab_dependencies` then walks every component and
//! eagerly loads the assets they reference: it builds a local [`fruits_serialization::SerializerRegistry`]
//! of *trans-serializers* — [`TextureLoadTransSerializer`], [`MaterialLoadTransSerializer`],
//! [`MeshLoadTransSerializer`], and [`PrefabLoadTransSerializer`] — each of which resolves an asset
//! key by calling the matching `get_or_load_*` loader. Because that registry runs across borrows of
//! all four storages at once, the storages are wrapped in [`std::sync::Mutex`] for the duration.
//! [`instantiate_prefab`] runs a *second* pass with a different registry — [`EntityTransSerializer`]
//! remaps stored entity ids onto freshly created entities and [`AssetGetTransSerializer`] looks up
//! the already-loaded handles — then spawns each component onto its entity and returns the first
//! (root) entity created.
//!
//! Loaders are intentionally side-effecting on the filesystem and never copy assets into the build
//! directory; see the `todo` notes in the source about wiring that into the build process and about
//! the asset formats still to be supported (font import, and `.fbx`/`.obj` import details).

mod material;
mod mesh;
mod texture;
mod prefab;
mod audio_clip;

pub use material::*;
pub use mesh::*;
pub use texture::*;
pub use prefab::*;
pub use audio_clip::*;

// todo: specify supported file formats.

// todo: asset types:
// - mesh (import details of the existing: .obj, .fbx)
// + texture (import details of the existing: .bmp, .png, .jpg)
// + material
// - font
// +/2 prefab
// + audio_clip

const _MATERIAL_FILE_EXAMPLE: &str = r##"
{
    "asset_type": "material",
    "material_type": "lit",
    "color": "#ffa641ff",
    "color_texture": "path/to/texture.asset",
    "metallic": 0.95,
    "roughness": 0.1,
    "space": "world"
}
"##;

const _TEXTURE_FILE_EXAMPLE: &str = r#"
{
    "asset_type": "texture",
    "raw_texture": "path/to/raw_texture.png",
    "format": "srgb"
}
"#;

const _MESH_FILE_EXAMPLE: &str = r#"
{
    "asset_type": "mesh",
    "raw_mesh": "path/to/raw_mesh.obj",
    "recalculate_normals": false,
    "force_shade_flat": false
}
"#;

const _FONT_FILE_EXAMPLE: &str = r##"
{
    "asset_type": "font",
    "texture": "path/to/texture.asset",
    "characters_uv": { "a": [[0.5, 0.7], [0.55, 0.75]] },
    "missing_character_uv": [[0.5, 0.7], [0.55, 0.75]],
    "character_ratio": 0.756
}
"##;

const _AUDIO_CLIP_FILE_EXAMPLE: &str = r##"
{
    "asset_type": "audio",
    "raw_audio": "path/to/audio.asset"
}
"##;

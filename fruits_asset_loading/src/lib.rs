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
mod serializers_load;

use std::{ffi::OsStr, path::{Path, PathBuf}};

use fruits_asset_storage::{AssetHandle, AssetStorageResource};
use fruits_audio::AudioClip;
use fruits_prefab::Prefab;
use fruits_render::StandardMaterial;
use fruits_render_core::{StandardMesh, StandardTexture};
use fruits_serialization::{SerializedPrimitive, SerializedValue, SerializerCtx};
pub use material::*;
pub use mesh::*;
pub use texture::*;
pub use prefab::*;
pub use audio_clip::*;
pub use serializers_load::*;

use fruits_ecs::{ResourcesHolderMut, Schedule, WorldBuilderMut};

// todo: specify supported file formats.

// todo: asset types:
// - mesh (import details of the existing: .obj, .fbx)
// + texture (import details of the existing: .bmp, .png, .jpg)
// + material
// - font
// +/2 prefab
// + audio_clip

pub const SYSTEM_GROUP_ASSETS: &'static str = "fruits_assets";

pub fn add_asset_module_to(mut world: WorldBuilderMut) {
    world.data_mut().resources_mut().insert(AssetStorageResource::<Prefab>::new());

    world.behavior_mut()
        .get_mut(Schedule::Start)
        .group(SYSTEM_GROUP_ASSETS)
        .insert_child_system(load_all_assets_system);
}

pub fn load_all_assets_system(res: ResourcesHolderMut) {
    let mut assets_dir_path = PathBuf::new();
    assets_dir_path.push("assets");

    load_all_assets(res, assets_dir_path);
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssetType {
    Texture,
    Material,
    Mesh,
    Font,
    AudioClip,
    Prefab,
}

impl AssetType {
    pub const fn serialized_str(&self) -> &'static str {
        match self {
            AssetType::Texture => "texture",
            AssetType::Material => "material",
            AssetType::Mesh => "mesh",
            AssetType::Font => "font",
            AssetType::AudioClip => "audio_clip",
            AssetType::Prefab => "prefab",
        }
    }

    pub fn from_serialized_str(serialized_str: &str) -> Option<Self> {
        match serialized_str {
            "texture" => Some(AssetType::Texture),
            "material" => Some(AssetType::Material),
            "mesh" => Some(AssetType::Mesh),
            "font" => Some(AssetType::Font),
            "audio_clip" => Some(AssetType::AudioClip),
            "prefab" => Some(AssetType::Prefab),
            _ => None
        }
    }
}

pub trait AssetLoader {
    type Asset: 'static + Send + Sync;
    type SelfWithAnotherLifetime<'r>: 'r + AssetLoader<Asset = Self::Asset>;

    fn create_loader<'r>(res: ResourcesHolderMut<'r>) -> Option<Self::SelfWithAnotherLifetime<'r>>;

    fn load_from_serialized(&mut self, ctx: SerializerCtx, value: &SerializedValue, assets_dir_path: impl AsRef<Path>) -> Option<Self::Asset>;
    fn get_related_asset_storage(&mut self) -> &mut AssetStorageResource<Self::Asset>;

    fn get_or_load_from_key(&mut self, ctx: SerializerCtx, key: &str, assets_dir_path: impl AsRef<Path>) -> Option<AssetHandle<Self::Asset>> {
        let storage = self.get_related_asset_storage();

        if let Some(stored_asset) = storage.get_registered(key) {
            if storage.get(stored_asset).is_some() {
                return Some(stored_asset.clone());
            }

            storage.unregister(key);
        }

        let mut path = assets_dir_path.as_ref().to_path_buf();
        path.push(key);

        let raw_asset = match std::fs::read_to_string(path) {
            Ok(data) => data,
            Err(_err) => return None,
        };

        let raw_asset = SerializedValue::from_json(&serde_json::from_str::<serde_json::Value>(&raw_asset).ok()?);
        let Some(texture) = self.load_from_serialized(ctx, &raw_asset, &assets_dir_path) else {
            return None;
        };

        let storage = self.get_related_asset_storage();
        let asset_handle = storage.insert(texture);

        storage.register(key.into(), asset_handle.clone());

        Some(asset_handle)
    }
}

pub fn load_all_assets(mut res: ResourcesHolderMut, assets_dir_path: impl AsRef<Path>) {
    load_asset_transitively_from_world(
        res.as_mut(),
        assets_dir_path.as_ref(),
        None, 
        |local_serializer, serializer| {
            let mut err_handler = |err| println!("[{}:{}] {err}", file!(), line!());
            let mut serializer_ctx = serializer.to_ctx(Some(&local_serializer), &mut err_handler);

            traverse_files_in_dir_deep(&assets_dir_path, &mut |file_path| {
                if file_path.extension() != Some(OsStr::new("asset")) {
                    return;
                }

                if file_path.to_str().is_none() {
                    println!("failed to load asset at path {file_path:?}, path is not utf-8 friendly");
                    return;
                };

                let asset_key = file_path.components().skip(assets_dir_path.as_ref().components().count()).map(|c| c.as_os_str().to_str().unwrap()).collect::<Vec<_>>().join("/");

                let asset_json = match std::fs::read_to_string(&file_path) {
                    Ok(data) => data,
                    Err(err) => {
                        println!("failed to load asset at path {file_path:?}, file read error: {err}");
                        return;
                    },
                };

                let asset_json = match serde_json::from_str::<serde_json::Value>(&asset_json) {
                    Ok(data) => data,
                    Err(err) => {
                        println!("failed to load asset at path {file_path:?}, invalid json: {err}");
                        return;
                    }
                };

                let serde_json::Value::Object(asset_json_obj) = &asset_json else {
                    println!("failed to load asset at path {file_path:?}, asset should be a json object");
                    return;
                };

                let Some(serde_json::Value::String(asset_type)) = asset_json_obj.get("asset_type") else {
                    println!("failed to load asset at path {file_path:?}, asset_type is missing");
                    return;
                };

                let Some(asset_type) = AssetType::from_serialized_str(&asset_type) else {
                    println!("failed to load asset at path {file_path:?}, unsupported asset_type: {asset_type}");
                    return;
                };

                let serialized_value = SerializedPrimitive::String(asset_key.into()).into();

                match asset_type {
                    AssetType::Texture => _ = serializer_ctx.deserialize::<AssetHandle<StandardTexture>>(&serialized_value),
                    AssetType::Material => _ = serializer_ctx.deserialize::<AssetHandle<StandardMaterial>>(&serialized_value),
                    AssetType::Mesh => _ = serializer_ctx.deserialize::<AssetHandle<StandardMesh>>(&serialized_value),
                    AssetType::AudioClip => _ = serializer_ctx.deserialize::<AssetHandle<AudioClip>>(&serialized_value),
                    // todo: font
                    AssetType::Font => todo!(),
                    AssetType::Prefab => _ = serializer_ctx.deserialize::<AssetHandle<Prefab>>(&serialized_value),
                };
            });
        },
    ).unwrap();
}

fn traverse_files_in_dir_deep(dir_path: impl AsRef<Path>, f: &mut impl FnMut(PathBuf)) {
    let Ok(dir) = std::fs::read_dir(&dir_path) else {
        return;
    };

    for entry in dir {
        let Ok(entry) = entry else {
            continue;
        };

        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_file() {
            f(entry.path());
        } else if file_type.is_dir() {
            traverse_files_in_dir_deep(entry.path(), f);
        }
    }
}

//

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

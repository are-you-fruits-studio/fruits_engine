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

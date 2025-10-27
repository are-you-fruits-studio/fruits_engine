use std::path::PathBuf;

use fruits_ecs::ResourcesHolderMut;
use fruits_ffi::FfiString;
use fruits_modules::{AssetHandle, AssetStorageResource, FilterMode, RenderApiResource, StandardTexture};
use fruits_serialization::JsonValue;
use image::GenericImageView;

pub fn get_or_load_texture_from_world(mut res: ResourcesHolderMut, key: &str) -> Option<AssetHandle<StandardTexture>> {
    let (render_api, textures) = unsafe { (
        &*res.get_ptr::<RenderApiResource>()?,
        &mut *res.get_ptr::<AssetStorageResource<StandardTexture>>()?,
    ) };

    get_or_load_texture(textures, render_api, key)
}


pub fn get_or_load_texture(textures: &mut AssetStorageResource<StandardTexture>, render_api: &RenderApiResource, key: &str) -> Option<AssetHandle<StandardTexture>> {
    if let Some(stored_texture) = textures.get_registered(key) {
        if textures.get(stored_texture).is_some() {
            return Some(stored_texture.clone());
        }

        textures.unregister(key);
    }

    let mut path = PathBuf::new();

    path.push("assets");
    path.push(key);

    // todo: make copying assets folder into the build directory a part of the build process
    let raw_texture = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => return None,
    };

    let Some(texture) = deserialize_texture(&raw_texture, render_api) else {
        return None;
    };
    
    let texture_handle = textures.insert(texture);

    textures.register(FfiString::from_string(key.to_string()), texture_handle.clone());

    Some(texture_handle)
}

fn deserialize_texture(data: &str, render_api: &RenderApiResource) -> Option<StandardTexture> {
    let Some(raw_texture) = JsonValue::parse(&mut data.chars()) else {
        return None
    };

    let JsonValue::Object(raw_texture) = raw_texture else {
        return None;
    };

    let Some(JsonValue::String(asset_type)) = raw_texture.get_value("asset_type") else {
        return None;
    };

    if asset_type != "texture" {
        return None;
    }

    let Some(JsonValue::String(raw_texture)) = raw_texture.get_value("raw_texture") else {
        return None;
    };

    let mut path = PathBuf::new();

    path.push("assets");
    path.push(raw_texture);

    let texture_data = match std::fs::read(path) {
        Ok(data) => data,
        Err(err) => return None,
    };

    let img = image::load_from_memory(&texture_data).ok()?;
    
    let texture = render_api.create_texture(FilterMode::Nearest, img.dimensions().into(), img.as_bytes());

    Some(texture)
}

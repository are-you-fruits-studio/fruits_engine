use std::path::PathBuf;

use fruits_asset_storage::{AssetHandle, AssetStorageResource};
use fruits_ecs::ResourcesHolderMut;
use fruits_ffi::FfiString;
use fruits_math::{Vec4, parse_color_rgba_f32};
use fruits_render::{RenderSpace, StandardMaterial};
use fruits_render_core::{RenderApiResource, StandardTexture};
use fruits_json::JsonValue;

use crate::get_or_load_texture;

pub fn get_or_load_material_from_world(res: ResourcesHolderMut, key: &str) -> Option<AssetHandle<StandardMaterial>> {
    let (render_api, materials, textures) = unsafe {
        (
            &*res.get_ptr::<RenderApiResource>()?,
            &mut *res.get_ptr::<AssetStorageResource<StandardMaterial>>()?,
            &mut *res.get_ptr::<AssetStorageResource<StandardTexture>>()?,
        )
    };

    get_or_load_material(render_api, materials, textures, key)
}

pub fn get_or_load_material(
    render_api: &RenderApiResource,
    materials: &mut AssetStorageResource<StandardMaterial>,
    textures: &mut AssetStorageResource<StandardTexture>,
    key: &str,
) -> Option<AssetHandle<StandardMaterial>> {
    if let Some(stored_material) = materials.get_registered(key) {
        if materials.get(stored_material).is_some() {
            return Some(stored_material.clone());
        }

        materials.unregister(key);
    }

    let mut path = PathBuf::new();

    path.push("assets");
    path.push(key);

    // todo: make copying assets folder into the build directory a part of the build process
    let raw_material = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(_err) => return None,
    };

    let Some(material) = deserialize_material(textures, render_api, &raw_material) else {
        return None;
    };

    let material_handle = materials.insert(material);

    materials.register(FfiString::from_string(key.to_string()), material_handle.clone());

    Some(material_handle)
}

//

fn deserialize_material(
    textures: &mut AssetStorageResource<StandardTexture>,
    render_api: &RenderApiResource,
    data: &str,
) -> Option<StandardMaterial> {
    let Some(raw_material) = JsonValue::parse(&mut data.chars()) else {
        return None;
    };

    let JsonValue::Object(raw_material) = raw_material else {
        return None;
    };

    let Some(JsonValue::String(asset_type)) = raw_material.get_value("asset_type") else {
        return None;
    };

    if asset_type != "material" {
        return None;
    }

    let mut material = StandardMaterial::default();

    if let Some(JsonValue::Bool(is_lit)) = raw_material.get_value("is_lit") {
        material.is_lit = *is_lit;
    }

    if let Some(JsonValue::String(color)) = raw_material.get_value("color") {
        material.color = Vec4::from_array(parse_color_rgba_f32(color)?);
    }

    if let Some(JsonValue::String(space)) = raw_material.get_value("space") {
        material.space = match space.as_str() {
            "world" => RenderSpace::World,
            "clip" => RenderSpace::Clip,
            "window" => RenderSpace::Window,
            _ => return None,
        };
    }

    if let Some(JsonValue::Number(metallic)) = raw_material.get_value("metallic") {
        material.metallic = metallic.to_f() as f32;
    }

    if let Some(JsonValue::Number(roughness)) = raw_material.get_value("roughness") {
        material.roughness = roughness.to_f() as f32;
    }

    if let Some(JsonValue::String(color_tex)) = raw_material.get_value("color_tex") {
        if let Some(color_tex) = get_or_load_texture(render_api, textures, color_tex) {
            material.color_tex = Some(color_tex);
        };
    }

    if let Some(JsonValue::String(emission_color)) = raw_material.get_value("emission_color") {
        material.emission_color = Vec4::from_array(parse_color_rgba_f32(emission_color)?);
    };

    let alpha_threshold = match raw_material.get_value("alpha_threshold") {    
        Some(JsonValue::Number(alpha_threshold)) => alpha_threshold.to_f() as f32,
        _ => 0.5,
    };

    let is_transparent = match raw_material.get_value("is_transparent") {
        Some(JsonValue::Bool(is_transparent)) => *is_transparent,
        _ => false,
    };

    material.alpha_threshold = if is_transparent {
        None
    } else {
        Some(alpha_threshold)
    };

    Some(material)
}

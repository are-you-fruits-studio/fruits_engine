use std::path::PathBuf;

use fruits_asset_storage::{AssetHandle, AssetStorageResource};
use fruits_ecs::ResourcesHolderMut;
use fruits_ffi::FfiString;
use fruits_render_core::{RenderApiResource, StandardMesh, StandardVertex};
use fruits_json::JsonValue;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CoordinateSpaceType {
    LeftHandZForward,
    RightHandZBack,
    RightHandZUp,
}

pub fn get_or_load_mesh_from_world(res: ResourcesHolderMut, key: &str) -> Option<AssetHandle<StandardMesh>> {
    let (render_api, meshes) = unsafe {
        (
            &*res.get_ptr::<RenderApiResource>()?,
            &mut *res.get_ptr::<AssetStorageResource<StandardMesh>>()?,
        )
    };

    get_or_load_mesh(render_api, meshes, key)
}

pub fn get_or_load_mesh(
    render_api: &RenderApiResource,
    meshes: &mut AssetStorageResource<StandardMesh>,
    key: &str,
) -> Option<AssetHandle<StandardMesh>> {
    if let Some(stored_mesh) = meshes.get_registered(key) {
        if meshes.get(stored_mesh).is_some() {
            return Some(stored_mesh.clone());
        }

        meshes.unregister(key);
    }

    let mut path = PathBuf::new();

    path.push("assets");
    path.push(key);

    // todo: make copying assets folder into the build directory a part of the build process
    let raw_mesh = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(_err) => return None,
    };

    let Some(mesh) = deserialize_mesh(&raw_mesh, render_api) else {
        return None;
    };

    let mesh_handle = meshes.insert(mesh);

    meshes.register(FfiString::from_string(key.to_string()), mesh_handle.clone());

    Some(mesh_handle)
}

fn deserialize_mesh(data: &str, render_api: &RenderApiResource) -> Option<StandardMesh> {
    let Some(mesh_asset) = JsonValue::parse(&mut data.chars()) else {
        return None;
    };

    let JsonValue::Object(mesh_asset) = mesh_asset else {
        return None;
    };

    let Some(JsonValue::String(asset_type)) = mesh_asset.get_value("asset_type") else {
        return None;
    };

    if asset_type != "mesh" {
        return None;
    }

    let Some(JsonValue::String(raw_mesh)) = mesh_asset.get_value("raw_mesh") else {
        return None;
    };

    let coordinate_space_type = match mesh_asset.get_value("coordinate_space") {
        Some(JsonValue::String(coordinate_space)) => {
            match coordinate_space.as_str() {
                "right_hand_z_up" => CoordinateSpaceType::RightHandZUp,
                "right_hand_z_back" => CoordinateSpaceType::RightHandZBack,
                _ => CoordinateSpaceType::LeftHandZForward,
            }
        },
        _ => CoordinateSpaceType::LeftHandZForward,
    };

    let has_clockwise_winding = match mesh_asset.get_value("has_clockwise_winding") {
        Some(JsonValue::Bool(true)) => true,
        _ => false,
    };

    let has_inverted_u = match mesh_asset.get_value("has_inverted_u") {
        Some(JsonValue::Bool(true)) => true,
        _ => false,
    };
    let has_inverted_v = match mesh_asset.get_value("has_inverted_v") {
        Some(JsonValue::Bool(true)) => true,
        _ => false,
    };

    dbg!(coordinate_space_type);

    let mut path = PathBuf::new();

    path.push("assets");
    path.push(raw_mesh);

    let mesh_data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(_err) => return None,
    };

    let obj = fruits_wavefront::parse_obj(&mesh_data)?;

    let mut vertices = Vec::new();

    for face in obj.faces {
        for vertex in face {
            vertices.push(StandardVertex {
                position: obj.positions.get(vertex.v).copied()?,
                color: [1.0; 4],
                normal: vertex.vn.map(|n| obj.normals.get(n).copied()).flatten()?,
                uv: vertex.vt.map(|n| obj.texcoords.get(n).copied()).flatten()?,
            });
        }
    }

    if coordinate_space_type == CoordinateSpaceType::RightHandZBack {
        for vertex in &mut vertices {
            vertex.position[2] *= -1.0;
            vertex.normal[2] *= -1.0;
        }

    } else if coordinate_space_type == CoordinateSpaceType::RightHandZUp {
        for vertex in &mut vertices {
            (vertex.position[1], vertex.position[2]) = (vertex.position[2], vertex.position[1]);
            (vertex.normal[1], vertex.normal[2]) = (vertex.normal[2], vertex.normal[1]);
        }
    }

    let mut indices = (0..vertices.len()).map(|i| i as u16).collect::<Vec<_>>();

    if has_clockwise_winding {
        if coordinate_space_type == CoordinateSpaceType::RightHandZBack || coordinate_space_type == CoordinateSpaceType::RightHandZUp {
            for i in 0..(indices.len() / 3) {
                let offset = i * 3;
                (indices[offset + 1], indices[offset + 2]) = (indices[offset + 2], indices[offset + 1])
            }
        }
    }

    if has_inverted_u {
        for vertex in &mut vertices {
            vertex.uv[0] = 1.0 - vertex.uv[0];
        }
    }
    if has_inverted_v {
        for vertex in &mut vertices {
            vertex.uv[1] = 1.0 - vertex.uv[1];
        }
    }

    let mesh = render_api.create_mesh(&vertices, &indices);

    Some(mesh)
}

fn _array_of_option_to_option_of_array<T, const N: usize>(array: [Option<T>; N]) -> Option<[T; N]> {
    for item in &array {
        if item.is_none() {
            return None;
        }
    }

    Some(array.map(|o| o.unwrap()))
}

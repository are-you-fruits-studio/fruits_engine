use std::path::Path;

use fruits_asset_storage::AssetStorageResource;
use fruits_ecs::{ResourcesHolderMut, ResourcesHolderRef};
use fruits_render_core::{CoordinateSpaceType, RenderApiResource, StandardMesh, StandardMeshAssetMetadata, StandardVertex};
use fruits_serialization::*;

use crate::AssetLoader;

pub struct MeshHandleLoader<'a> {
    pub render_api: &'a RenderApiResource,
    pub meshes: &'a mut AssetStorageResource<StandardMesh>,
}

impl<'a> MeshHandleLoader<'a> {
    pub fn from_world(res: ResourcesHolderMut<'a>) -> Option<Self> {
        Some(unsafe { Self {
            render_api: &*res.get_ptr::<RenderApiResource>()?,
            meshes: &mut *res.get_ptr::<AssetStorageResource<StandardMesh>>()?,
        }})
    }
}
impl<'a> AssetLoader for MeshHandleLoader<'a> {
    type Asset = StandardMesh;
    type SelfWithAnotherLifetime<'r> = MeshHandleLoader<'r>;

    fn create_loader<'r>(res: ResourcesHolderMut<'r>) -> Option<Self::SelfWithAnotherLifetime<'r>> {
        Self::SelfWithAnotherLifetime::from_world(res)
    }
    
    fn get_related_asset_storage(&mut self) -> &mut AssetStorageResource<Self::Asset> {
        self.meshes
    }
    
    fn load_from_serialized(&mut self, ctx: SerializerCtx, value: &SerializedValue, assets_dir_path: impl AsRef<Path>) -> Option<Self::Asset> {
        MeshLoader {
            render_api: self.render_api,
        }.load_from_serialized(value, assets_dir_path)
    }
}

//

pub struct MeshLoader<'a> {
    pub render_api: &'a RenderApiResource,
}

impl<'a> MeshLoader<'a> {
    pub fn from_world(res: ResourcesHolderRef<'a>) -> Option<Self> {
        Some(Self {
            render_api: res.get::<RenderApiResource>()?,
        })
    }
    
    pub fn load_from_serialized(&mut self, value: &SerializedValue, assets_dir_path: impl AsRef<Path>) -> Option<StandardMesh> {
        let SerializedValue::Composite(SerializedComposite { values: SerializedCompositeValues::Map(SerializedMap { values: value, .. }), .. }) = value else {
            return None;
        };
    
        let Some(SerializedValue::Primitive(SerializedPrimitive::String(raw_mesh))) = value.get("raw_mesh") else {
            return None;
        };

        let coordinate_space = match value.get("coordinate_space") {
            Some(SerializedValue::Primitive(SerializedPrimitive::String(coordinate_space))) => {
                match coordinate_space.as_str() {
                    "RightHandZUp" => CoordinateSpaceType::RightHandZUp,
                    "RightHandZBack" => CoordinateSpaceType::RightHandZBack,
                    _ => CoordinateSpaceType::LeftHandZForward,
                }
            },
            _ => CoordinateSpaceType::LeftHandZForward,
        };
    
        let has_clockwise_winding = match value.get("has_clockwise_winding") {
            Some(SerializedValue::Primitive(SerializedPrimitive::Bool(true))) => true,
            _ => false,
        };
    
        let has_inverted_u = match value.get("has_inverted_u") {
            Some(SerializedValue::Primitive(SerializedPrimitive::Bool(true))) => true,
            _ => false,
        };
        let has_inverted_v = match value.get("has_inverted_v") {
            Some(SerializedValue::Primitive(SerializedPrimitive::Bool(true))) => true,
            _ => false,
        };

        let value = StandardMeshAssetMetadata {
            raw_mesh: raw_mesh.clone(),
            coordinate_space,
            has_clockwise_winding,
            has_inverted_u,
            has_inverted_v,
        };

        self.load_from_deserialized(value, assets_dir_path)
    }
    pub fn load_from_deserialized(&mut self, value: StandardMeshAssetMetadata, assets_dir_path: impl AsRef<Path>) -> Option<StandardMesh> {
        let mut path = assets_dir_path.as_ref().to_path_buf();
        path.push(value.raw_mesh.as_str());
    
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
    
        if value.coordinate_space == CoordinateSpaceType::RightHandZBack {
            for vertex in &mut vertices {
                vertex.position[2] *= -1.0;
                vertex.normal[2] *= -1.0;
            }
        
        } else if value.coordinate_space == CoordinateSpaceType::RightHandZUp {
            for vertex in &mut vertices {
                (vertex.position[1], vertex.position[2]) = (vertex.position[2], vertex.position[1]);
                (vertex.normal[1], vertex.normal[2]) = (vertex.normal[2], vertex.normal[1]);
            }
        }
    
        let mut indices = (0..vertices.len()).map(|i| i as u16).collect::<Vec<_>>();
    
        if value.has_clockwise_winding {
            if value.coordinate_space == CoordinateSpaceType::RightHandZBack || value.coordinate_space == CoordinateSpaceType::RightHandZUp {
                for i in 0..(indices.len() / 3) {
                    let offset = i * 3;
                    (indices[offset + 1], indices[offset + 2]) = (indices[offset + 2], indices[offset + 1])
                }
            }
        }
    
        if value.has_inverted_u {
            for vertex in &mut vertices {
                vertex.uv[0] = 1.0 - vertex.uv[0];
            }
        }
        if value.has_inverted_v {
            for vertex in &mut vertices {
                vertex.uv[1] = 1.0 - vertex.uv[1];
            }
        }
    
        let mesh = self.render_api.create_mesh(&vertices, &indices, Some(value));
    
        Some(mesh)
    }
}
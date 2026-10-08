use fruits_asset_storage::AssetHandle;
use fruits_ecs::Component;
use fruits_ffi::FfiVec;
use fruits_math::Vec3;
use fruits_render_core::{StandardMaterial, StandardMesh, StandardVertex};
use fruits_serialization::*;

#[repr(C)]
#[derive(Component)]
pub struct StandardRenderComponent {
    pub mesh: AssetHandle<StandardMesh>,
    pub material: AssetHandle<StandardMaterial>,
}

#[repr(C)]
#[derive(Component, Clone, TransSerializable)]
pub struct StandardMeshComponent {
    pub mesh: AssetHandle<StandardMesh>,
}

#[repr(C)]
#[derive(Component, Default, Clone)]
pub struct BatchedMeshComponent {
    pub vertices: FfiVec<StandardVertex>,
    pub indices: FfiVec<u16>,
}

#[repr(C)]
#[derive(Component, Clone, TransSerializable)]
pub struct StandardMaterialComponent {
    pub material: AssetHandle<StandardMaterial>,
}

#[repr(C)]
#[derive(Component, Clone, Copy)]
pub enum StandardLightComponent {
    Point {
        color: Vec3<f32>,
        range: f32,
    },
    Spot {
        color: Vec3<f32>,
        range: f32,
        fov: f32,
    },
    Directional {
        color: Vec3<f32>,
    },
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum CameraProjection {
    Perspective { fov: f32 },
    Orthographic { size: f32 },
}

#[repr(C)]
#[derive(Component)]
pub struct CameraComponent {
    pub projection: CameraProjection,
    pub near: f32,
    pub far: f32,
}

impl CameraComponent {
    pub fn projection_matrix(&self, aspect: f32) -> fruits_math::Mat4<f32> {
        match self.projection {
            CameraProjection::Perspective { fov } => fruits_math::perspective_proj_matrix(fov, self.near, self.far, aspect),
            CameraProjection::Orthographic { size } => fruits_math::orthographic_proj_matrix(size, self.near, self.far, aspect),
        }
    }
}

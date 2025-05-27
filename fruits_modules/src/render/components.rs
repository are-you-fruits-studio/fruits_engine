use fruits_ecs::Component;

use crate::asset::AssetHandle;

use super::assets::{StandardMaterial, StandardMesh};

#[derive(Component)]
pub struct StandardMeshComponent {
    pub mesh: AssetHandle<StandardMesh>,
}

#[derive(Component)]
pub struct StandardMaterialComponent {
    pub material: AssetHandle<StandardMaterial>,
}

#[derive(Component)]
pub struct CameraComponent {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

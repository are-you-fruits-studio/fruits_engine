use fruits_ecs::Component;
use fruits_math::Vec4;

use crate::{asset::AssetHandle, render::{Font, StandardVertex}, transform::UiVal};

use super::assets::{StandardMaterial, StandardMesh};

// todo: support ffi
#[derive(Component)]
pub struct StandardRenderComponent {
    pub mesh: AssetHandle<StandardMesh>,
    pub material: AssetHandle<StandardMaterial>,
}

// todo: support ffi
#[derive(Component)]
pub struct StandardMeshComponent {
    pub mesh: AssetHandle<StandardMesh>,
}

// todo: support ffi
#[derive(Component, Default)]
pub struct BatchedMeshComponent {
    pub vertices: Vec<StandardVertex>,
    pub indices: Vec<u16>,
}

// todo: support ffi
#[derive(Component)]
pub struct StandardMaterialComponent {
    pub material: AssetHandle<StandardMaterial>,
}

// todo: support ffi
#[derive(Component)]
pub struct CameraComponent {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

// todo: support ffi
#[derive(Component)]
pub struct TextComponent {
    pub font: AssetHandle<Font>,
    pub text: String,
    pub font_size: UiVal,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
    pub is_y_inverted: bool,
    pub horizontal_spacing: f32,
    pub color: Vec4<f32>,
}

// todo: support ffi
#[derive(Component)]
pub struct ImageComponent {
    pub is_y_inverted: bool,
    pub color: Vec4<f32>,
    pub fill_amt: f32,
    pub fill_settings: Option<ImageFillSettings>,
}
impl Default for ImageComponent {
    fn default() -> Self {
        Self {
            is_y_inverted: true,
            color: Vec4::splat(1.0),
            fill_amt: 1.0,
            fill_settings: None,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ImageFillSettings {
    // todo: add more variants and details.
    RadialCenter,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum HorizontalAlign {
    Left,
    Middle,
    Right,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

// todo: support ffi
#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct GlobalDisableableComponent {
    pub is_disabled: bool,
}
impl Default for GlobalDisableableComponent {
    fn default() -> Self {
        Self {
            is_disabled: false,
        }
    }
}
// todo: support ffi
#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LocalDisableableComponent {
    pub is_disabled: bool,
}
impl Default for LocalDisableableComponent {
    fn default() -> Self {
        Self {
            is_disabled: false,
        }
    }
}

// todo: support ffi
#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct ChildrenRectMaskComponent;
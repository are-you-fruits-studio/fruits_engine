use fruits_math::{Mat4, Vec3, Vec4};
use fruits_utils::mem::{AllBitVariationsValid, AllBitsInit};

use crate::{
    asset::AssetHandle,
    render::{RenderSpace, StandardTexture},
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StandardUniform {
    pub world_to_clip: Mat4<f32>,
    pub color: Vec4<f32>,
    pub emission_color: Vec4<f32>,
    pub camera_position_world: Vec3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub alpha_threshold: f32,
    pub _padding: [f32; 2],
}

unsafe impl AllBitVariationsValid for StandardUniform {}
unsafe impl AllBitsInit for StandardUniform {}

impl Default for StandardUniform {
    fn default() -> Self {
        Self {
            world_to_clip: Mat4::IDENTITY,
            color: Vec4::splat(0.5),
            emission_color: Vec4::splat(0.0),
            camera_position_world: Vec3::splat(0.0),
            metallic: 0.5,
            roughness: 0.5,
            alpha_threshold: 0.5,
            _padding: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StandardMaterial {
    pub space: RenderSpace,
    pub color: Vec4<f32>,
    pub emission_color: Vec4<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub alpha_threshold: Option<f32>,
    pub color_tex: Option<AssetHandle<StandardTexture>>,
    pub is_lit: bool,
}

impl Default for StandardMaterial {
    fn default() -> Self {
        Self {
            space: RenderSpace::World,
            color: Vec4::splat(0.5),
            emission_color: Vec4::splat(0.0),
            metallic: 0.0,
            roughness: 0.5,
            alpha_threshold: Some(0.5),
            color_tex: None,
            is_lit: false,
        }
    }
}

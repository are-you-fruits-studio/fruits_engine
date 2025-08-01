use fruits_math::{Mat4, Vec3, Vec4};
use fruits_utils::mem::{AllBitVariationsValid, AllBitsInit};

use crate::{asset::AssetHandle, render::{RenderSpace, StandardTexture}};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LitUniform {
    pub world_to_clip: Mat4<f32>,
    pub albedo_color: Vec4<f32>,
    pub emission_color: Vec4<f32>,
    pub camera_position_world: Vec3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub alpha_threshold: f32,
    pub _padding: [f32; 2],
}

unsafe impl AllBitVariationsValid for LitUniform { }
unsafe impl AllBitsInit for LitUniform { }

impl Default for LitUniform {
    fn default() -> Self {
        Self {
            world_to_clip: Mat4::IDENTITY,
            albedo_color: Vec4::with_all(0.5),
            emission_color: Vec4::with_all(0.0),
            camera_position_world: Vec3::with_all(0.0),
            metallic: 0.5,
            roughness: 0.5,
            alpha_threshold: 0.5,
            _padding: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UnlitUniform {
    pub world_to_clip: Mat4<f32>,
    pub color: Vec4<f32>,
    pub alpha_threshold: f32,
    pub _padding: [f32; 3],
}

unsafe impl AllBitVariationsValid for UnlitUniform { }
unsafe impl AllBitsInit for UnlitUniform { }

impl Default for UnlitUniform {
    fn default() -> Self {
        Self {
            world_to_clip: Mat4::IDENTITY,
            color: Vec4::with_all(0.5),
            alpha_threshold: 0.5,
            _padding: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LitMaterial {
    pub space: RenderSpace,
    pub albedo_color: Vec4<f32>,
    pub emission_color: Vec4<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub alpha_threshold: f32,
    pub albedo_tex: Option<AssetHandle<StandardTexture>>,
}

impl Default for LitMaterial {
    fn default() -> Self {
        LitMaterial {
            space: RenderSpace::World,
            albedo_color: Vec4::with_all(0.5),
            emission_color: Vec4::with_all(0.0),
            metallic: 0.0,
            roughness: 0.5,
            alpha_threshold: 0.5,
            albedo_tex: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnlitMaterial {
    pub space: RenderSpace,
    pub color: Vec4<f32>,
    pub color_tex: Option<AssetHandle<StandardTexture>>,
    pub alpha_threshold: f32,
}

impl Default for UnlitMaterial {
    fn default() -> Self {
        Self {
            space: RenderSpace::World,
            color: Vec4::with_all(0.5),
            color_tex: None,
            alpha_threshold: 0.5,
        }
    }
}

#[derive(Clone)]
pub enum StandardMaterial {
    Lit(LitMaterial),
    Unlit(UnlitMaterial),
}

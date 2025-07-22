use fruits_math::{Mat4, Vec3, Vec4};
use fruits_utils::mem::{AllBitVariationsValid, AllBitsInit};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LitUniform {
    pub world_to_clip: Mat4<f32>,
    pub albedo_color: Vec4<f32>,
    pub emission_color: Vec4<f32>,
    pub camera_position_world: Vec3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub _padding: [f32; 3],
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
            _padding: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UnlitUniform {
    pub world_to_clip: Mat4<f32>,
    pub color: Vec4<f32>,
}

unsafe impl AllBitVariationsValid for UnlitUniform { }
unsafe impl AllBitsInit for UnlitUniform { }

impl Default for UnlitUniform {
    fn default() -> Self {
        Self {
            world_to_clip: Mat4::IDENTITY,
            color: Vec4::with_all(0.5),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LitMaterial {
    pub albedo_color: Vec4<f32>,
    pub emission_color: Vec4<f32>,
    pub metallic: f32,
    pub roughness: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct UnlitMaterial {
    pub color: Vec4<f32>,
}

#[derive(Copy, Clone)]
pub enum StandardMaterial {
    Lit(LitMaterial),
    Unlit(UnlitMaterial),
}

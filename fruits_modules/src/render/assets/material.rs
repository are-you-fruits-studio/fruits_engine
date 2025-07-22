
use fruits_app::RenderStateResource;
use fruits_ecs::WorldData;
use fruits_math::{Mat4, Vec3, Vec4};
use fruits_utils::mem::{AllBitVariationsValid, AllBitsInit};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, DepthStencilState, RenderPipeline};

use crate::render::{DepthTextureResource, StandardRenderResource};

use super::{mesh::StandardVertex, StandardInstance};

// todo
pub struct StandardMaterialNew {
    pub color: [f32; 3],
    pub texture: (),
    pub emission: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StandardGlobalUniform {
    pub world_to_clip: Mat4<f32>,
    pub camera_position_world: Vec3<f32>,
    pub _padding: f32,
}

unsafe impl AllBitVariationsValid for StandardGlobalUniform { }
unsafe impl AllBitsInit for StandardGlobalUniform { }

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StandardMaterialUniform {
    pub albedo_color: Vec4<f32>,
    pub emission_color: Vec4<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub _padding: [f32; 2],
}

unsafe impl AllBitVariationsValid for StandardMaterialUniform { }
unsafe impl AllBitsInit for StandardMaterialUniform { }

impl Default for StandardGlobalUniform {
    fn default() -> Self {
        Self {
            world_to_clip: Mat4::IDENTITY,
            camera_position_world: Vec3::with_all(0.0),
            _padding: 0.0,
        }
    }
}

impl Default for StandardMaterialUniform {
    fn default() -> Self {
        Self {
            albedo_color: Vec4::with_all(0.5),
            emission_color: Vec4::with_all(0.0),
            metallic: 0.5,
            roughness: 0.5,
            _padding: [0.0; 2],
        }
    }
}

pub struct StandardMaterial {
    uniform: StandardMaterialUniform,
}

impl StandardMaterial {
    // todo: redundant world
    pub fn from_world(world: &WorldData) -> Self {
        Self {
            uniform: StandardMaterialUniform::default(),
        }
    }

    pub fn uniform(&self) -> &StandardMaterialUniform {
        &self.uniform
    }

    pub fn uniform_mut(&mut self) -> &mut StandardMaterialUniform {
        &mut self.uniform
    }
}

use fruits_asset_storage::AssetHandle;
use fruits_ffi::{FfiDroppable, FfiOption};
use fruits_math::{Mat4, Vec3, Vec4};
use fruits_serialization::*;
use fruits_utils::mem::{AllBitVariationsValid, AllBitsInit};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, util::{BufferInitDescriptor, DeviceExt}};

use crate::{RenderState, StandardTexture};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, TransSerializable)]
pub enum RenderSpace {
    Clip,
    Window,
    World,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StandardGenericLight {
    pub color: Vec3<f32>,
    pub light_type: u32,
    pub center: Vec3<f32>,
    pub range: f32,
    pub direction_dst: Vec3<f32>,
    pub fov: f32,
}

unsafe impl AllBitVariationsValid for StandardGenericLight {}
unsafe impl AllBitsInit for StandardGenericLight {}

impl Default for StandardGenericLight {
    fn default() -> Self {
        Self {
            color: Vec3::splat(0.0),
            light_type: 0,
            center: Vec3::splat(0.0),
            range: 0.0,
            direction_dst: Vec3::new(0.0, -1.0, 0.0),
            fov: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StandardUniformGlobal {
    pub camera_position_world: Vec3<f32>,
    pub lights_count: u32,
}

unsafe impl AllBitVariationsValid for StandardUniformGlobal {}
unsafe impl AllBitsInit for StandardUniformGlobal {}

impl Default for StandardUniformGlobal {
    fn default() -> Self {
        Self {
            camera_position_world: Vec3::splat(0.0),
            lights_count: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StandardUniformMaterial {
    pub world_to_clip: Mat4<f32>,
    pub color: Vec4<f32>,
    pub emission_color: Vec4<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub alpha_threshold: f32,
    pub _padding: [f32; 1],
}

unsafe impl AllBitVariationsValid for StandardUniformMaterial {}
unsafe impl AllBitsInit for StandardUniformMaterial {}

impl Default for StandardUniformMaterial {
    fn default() -> Self {
        Self {
            world_to_clip: Mat4::IDENTITY,
            color: Vec4::splat(0.5),
            emission_color: Vec4::splat(0.0),
            metallic: 0.5,
            roughness: 0.5,
            alpha_threshold: 0.5,
            _padding: Default::default(),
        }
    }
}

impl From<StandardLight> for StandardGenericLight {
    fn from(value: StandardLight) -> Self {
        match value {
            StandardLight::Point { color, center, range } => Self {
                light_type: 0,
                color,
                center,
                range,
                ..Default::default()
            },
            StandardLight::Spot { color, center, range, direction_dst, fov } => Self {
                light_type: 1,
                color,
                center,
                range,
                direction_dst,
                fov,
            },
            StandardLight::Directional { color, direction_dst } => Self {
                light_type: 2,
                color,
                direction_dst,
                ..Default::default()
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, TransSerializable)]
pub enum StandardLight {
    Point {
        color: Vec3<f32>,
        center: Vec3<f32>,
        range: f32,
    },
    Spot {
        color: Vec3<f32>,
        center: Vec3<f32>,
        range: f32,
        direction_dst: Vec3<f32>,
        fov: f32,
    },
    Directional {
        color: Vec3<f32>,
        direction_dst: Vec3<f32>,
    },
}

pub struct StandardMaterialNative {
    pub buffer_uniform: Buffer,
    pub bind_group: BindGroup,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, PartialOrd, TransSerializable)]
pub struct StandardMaterialAssetMetadata {
    pub is_lit: bool,
    pub space: RenderSpace,
    pub color: Vec4<f32>,
    pub emission_color: Vec4<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub alpha_threshold: FfiOption<f32>,
    pub color_tex: AssetHandle<StandardTexture>,
}

impl Default for StandardMaterialAssetMetadata {
    fn default() -> Self {
        Self {
            space: RenderSpace::World,
            color: Vec4::splat(0.5),
            emission_color: Vec4::splat(0.0),
            metallic: 0.0,
            roughness: 0.5,
            alpha_threshold: Some(0.5).into(),
            color_tex: AssetHandle::EMPTY,
            is_lit: false,
        }
    }
}

#[repr(C)]
pub struct StandardMaterial {
    meta: StandardMaterialAssetMetadata,
    native: FfiDroppable,
}

impl StandardMaterial {
    pub(crate) fn new(
        render_state: &RenderState,
        color_texture: &StandardTexture,
        meta: StandardMaterialAssetMetadata,
    ) -> Self {
        let buffer_uniform = render_state.device().create_buffer_init(&BufferInitDescriptor {
            label: Some("Standard Material Uniform Buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: fruits_utils::mem::as_bytes(&[StandardUniformMaterial::default()]),
        });

        let texture_view_color = unsafe { color_texture.native().texture.create_view(&Default::default()) };
        let sampler_color = unsafe { &color_texture.native().sampler };

        let bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
            label: Some("Standard Material Bind Group"),
            layout: &render_state.render_data().bind_group_layout_material,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer_uniform.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view_color),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler_color),
                },
            ],
        });

        Self {
            native: FfiDroppable::new(StandardMaterialNative {
                buffer_uniform,
                bind_group,
            }),
            meta: meta.into(),
        }
    }

    pub fn meta(&self) -> &StandardMaterialAssetMetadata {
        &self.meta
    }

    pub unsafe fn native(&self) -> &StandardMaterialNative {
        unsafe { &*(self.native.get() as *const StandardMaterialNative) }
    }
}

unsafe impl Send for StandardMaterial where StandardMaterialNative: Send {}
unsafe impl Sync for StandardMaterial where StandardMaterialNative: Sync {}

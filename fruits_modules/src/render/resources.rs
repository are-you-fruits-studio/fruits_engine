use fruits_ecs_data::Resource;
use fruits_ecs_macros::Resource;
use fruits_math::Vec2;
use wgpu::{BindGroup, BindGroupLayout, Buffer, RenderPipeline, SurfaceTexture};

#[derive(Resource)]
pub struct SurfaceTextureResource {
    pub texture: Option<SurfaceTexture>,
}

#[derive(Resource)]
pub struct CameraUniformBufferResource {
    pub buffer: Buffer,
    pub group: BindGroup,
}

#[derive(Resource)]
pub struct CameraUniformBufferGroupLayoutResource {
    layout: BindGroupLayout,
}

impl CameraUniformBufferGroupLayoutResource {
    pub fn new(layout: BindGroupLayout) -> Self {
        Self {
            layout,
        }
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }
}

#[derive(Resource)]
pub struct InstanceBufferResource {
    pub buffer: Buffer,
}

#[derive(Resource)]
pub struct GizmosResource {
    pub lines: Vec<[Vec2<f32>; 2]>,
}

#[derive(Resource)]
pub struct GizmosRenderResource {
    pub vertex_buffer: Buffer,
    pub pipeline: RenderPipeline,
    pub bind_group: BindGroup,
}
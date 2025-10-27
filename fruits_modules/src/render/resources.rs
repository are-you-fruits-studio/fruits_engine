use std::collections::{hash_map::IterMut, HashMap};

use fruits_ecs::Resource;
use fruits_math::{Mat4, Vec3, Vec4};
use wgpu::{BindGroup, BindGroupLayout, Buffer, PipelineLayout, RenderPipeline, Sampler, SurfaceTexture, Texture, TextureView};

use crate::{asset::AssetHandle, render::{Font, StandardTexture, StandardVertex}};

use super::GizmoLine;

mod render_state;
pub use render_state::*;

// todo: support ffi
#[derive(Resource)]
pub struct SurfaceTextureResource {
    pub texture: Option<SurfaceTexture>,
}

// todo: support ffi
#[derive(Resource)]
pub struct GizmosResource {
    lines: HashMap<RenderSpace, Vec<GizmoLine>>,
}
impl GizmosResource {
    pub fn space(&mut self, space: RenderSpace) -> &mut Vec<GizmoLine> {
        self.lines.get_mut(&space).unwrap()
    }

    pub fn spaces(&mut self) -> IterMut<'_, RenderSpace, Vec<GizmoLine>> {
        self.lines.iter_mut()
    }
}
impl Default for GizmosResource {
    fn default() -> Self {
        let mut lines = HashMap::new();

        lines.insert(RenderSpace::Clip, Vec::new());
        lines.insert(RenderSpace::Window, Vec::new());
        lines.insert(RenderSpace::World, Vec::new());

        Self {
            lines,
        }
    }
}

// todo: support ffi
#[derive(Resource)]
pub struct GizmosRenderResource {
    pub vertex_buffer: Buffer,
    pub color_buffer: Buffer,
    pub transform_buffer: Buffer,
    pub pipeline: RenderPipeline,
    pub bind_group: BindGroup,
    pub vertices_cpu_buffer: Box<[[Vec4<f32>; 2]]>,
    pub colors_cpu_buffer: Box<[Vec4<f32>]>,
}

// todo: support ffi
#[derive(Resource)]
pub struct DepthTextureResource {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub sampler: Sampler,
}

// todo: support ffi
#[derive(Resource)]
pub struct StandardRenderResource {
    pub pipeline_layout: PipelineLayout,
    pub batched_vertex_buffer: Buffer,
    pub instance_buffer: Buffer,
    pub instance_cpu_buffer: Box<[[[f32; 4]; 4]]>,
    pub camera_pos: Vec3<f32>,
    pub camera_proj_matrix: Mat4<f32>,
    pub lit: MaterialStandardRenderResourceData,
    pub unlit: MaterialStandardRenderResourceData,
}

// todo: support ffi
#[derive(Resource)]
pub struct BatchedVertexCpuBufferResource(pub Box<[StandardVertex]>);

// todo: support ffi
#[derive(Resource)]
pub struct StandardRenderAssetsResource {
    pub texture_white: AssetHandle<StandardTexture>,
    pub texture_text_px_5_7: AssetHandle<StandardTexture>,
    pub font_px_5_7: AssetHandle<Font>,
    pub texture_text_px_8_8: AssetHandle<StandardTexture>,
    pub font_px_8_8: AssetHandle<Font>,
    pub texture_text_px_8_12: AssetHandle<StandardTexture>,
    pub font_px_8_12: AssetHandle<Font>,
}

pub struct MaterialStandardRenderResourceData {
    pub buffer_uniform: Buffer,
    pub bind_group_uniform: BindGroup,
    pub render_pipeline: RenderPipeline,
}

// todo: support ffi
#[derive(Resource)]
pub struct ScreenSpaceResource {
    pub near: f32,
    pub far: f32,
}

impl Default for ScreenSpaceResource {
    fn default() -> Self {
        Self {
            near: -1_000.0,
            far: 1_000.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RenderSpace {
    Clip,
    Window,
    World,
}

//
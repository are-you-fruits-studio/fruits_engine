use fruits_ecs::Resource;
use fruits_math::{Mat4, Vec3, Vec4};
use fruits_render_core::{RenderSpace, StandardGenericLight, StandardVertex};
use fruits_ffi::*;
use wgpu::{BindGroup, BindGroupLayout, Buffer, PipelineLayout, RenderPipeline, Sampler, Texture, TextureView};

use super::GizmoLine;

// todo: support ffi
#[derive(Resource)]
pub struct MainRenderTargetResource {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub sampler: Sampler,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
    pub render_pipeline: RenderPipeline,
}

#[repr(C)]
#[derive(Resource)]
pub struct GizmosResource {
    lines: FfiIndexMap<RenderSpace, FfiVec<GizmoLine>>,
}
impl GizmosResource {
    pub fn space(&mut self, space: RenderSpace) -> &mut FfiVec<GizmoLine> {
        self.lines.get_mut(&space).unwrap()
    }

    pub fn spaces(&mut self) -> FfiIndexMapPairsMutIter<'_, RenderSpace, FfiVec<GizmoLine>> {
        self.lines.iter_mut()
    }
}
impl Default for GizmosResource {
    fn default() -> Self {
        let mut lines = FfiIndexMap::new();

        lines.insert(RenderSpace::Clip, FfiVec::new());
        lines.insert(RenderSpace::Window, FfiVec::new());
        lines.insert(RenderSpace::World, FfiVec::new());

        Self { lines }
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
pub struct TransparentTargetTextureResource {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub sampler: Sampler,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

#[repr(C)]
#[derive(Resource)]
pub struct BloomResource {
    pub is_enabled: bool,
    pub threshold: f32,
    pub threshold_softening: f32,
    pub intensity: f32,
}

impl Default for BloomResource {
    fn default() -> Self {
        Self {
            is_enabled: false,
            threshold: 1.5,
            threshold_softening: 0.15,
            intensity: 2.8,
        }
    }
}

// todo: support ffi
#[derive(Resource)]
pub struct BloomRenderResource {
    pub buffer_uniform_blur_dir: Buffer,
    pub buffer_uniform_threshold: Buffer,
    pub buffer_uniform_uv_scale_offset: Buffer,
    pub buffer_uniform_intensity: Buffer,
    pub textures: [Texture; 3],
    pub sampler: Sampler,
    pub bind_group_gather: BindGroup,
    pub bind_group_apply_0: BindGroup,
    pub bind_group_apply_1: BindGroup,
    pub bind_group_blur_horiz_0: BindGroup,
    pub bind_group_blur_horiz_1: BindGroup,
    pub bind_group_blur_vert: BindGroup,
    pub bind_group_downscale_0: BindGroup,
    pub bind_group_downscale_1: BindGroup,
    pub render_pipeline_gather: RenderPipeline,
    pub render_pipeline_blur: RenderPipeline,
    pub render_pipeline_downscale: RenderPipeline,
    pub render_pipeline_apply: RenderPipeline,
}

// todo: support ffi
#[derive(Resource)]
pub struct StandardRenderResource {
    pub pipeline_layout: PipelineLayout,
    pub global_bind_group: BindGroup,
    pub global_uniform_buffer: Buffer,
    pub batched_vertex_buffer: Buffer,
    pub lights_buffer: Buffer,
    pub lights_count: u32,
    pub instance_buffer: Buffer,
    pub instance_cpu_buffer: Box<[[[f32; 4]; 4]]>,
    pub camera_pos: Vec3<f32>,
    pub camera_proj_matrix: Mat4<f32>,
    pub render_pipeline_opaque_lit: RenderPipeline,
    pub render_pipeline_opaque_unlit: RenderPipeline,
    pub render_pipeline_transparent_lit: RenderPipeline,
    pub render_pipeline_transparent_unlit: RenderPipeline,
    pub render_pipeline_transparent_final: RenderPipeline,
}

// todo: support ffi
#[derive(Resource)]
pub struct BatchedVertexCpuBufferResource(pub Box<[StandardVertex]>);

// todo: support ffi
#[derive(Resource)]
pub struct LightsCpuBufferResource(pub Box<[StandardGenericLight]>);

pub struct MaterialStandardRenderResourceData {
    pub render_pipeline: RenderPipeline,
}

#[repr(C)]
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

//

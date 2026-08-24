use fruits_asset_storage::AssetHandle;
use fruits_ecs::Resource;
use fruits_math::{Mat4, Vec3, Vec4};
use fruits_render_core::{RenderSpace, StandardGenericLight, StandardTexture, StandardVertex};
use fruits_ffi::*;
use wgpu::{BindGroup, BindGroupLayout, Buffer, PipelineLayout, RenderPipeline, Sampler, SurfaceTexture, Texture, TextureView};

use crate::Font;

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

// todo: support ffi
#[derive(Resource)]
pub struct BloomRenderResource {
    pub textures: [Texture; 2],
    pub sampler: Sampler,
    // todo
    // pub bind_group_layout: BindGroupLayout,
    // pub bind_group: BindGroup,
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

#[repr(C)]
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

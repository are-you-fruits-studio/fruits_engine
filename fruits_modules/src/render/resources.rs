use std::collections::{hash_map::IterMut, HashMap};

use fruits_ecs::Resource;
use fruits_math::{Mat4, Vec3, Vec4};
use wgpu::{BindGroup, Buffer, PipelineLayout, RenderPipeline, Sampler, SurfaceTexture, Texture, TextureView};


use super::{GizmoLine, GizmoSpace};

#[derive(Resource)]
pub struct SurfaceTextureResource {
    pub texture: Option<SurfaceTexture>,
}

#[derive(Resource)]
pub struct GizmosResource {
    lines: HashMap<GizmoSpace, Vec<GizmoLine>>,
}
impl GizmosResource {
    pub fn new() -> Self {
        let mut lines = HashMap::new();

        lines.insert(GizmoSpace::Clip, Vec::new());
        lines.insert(GizmoSpace::Window, Vec::new());
        lines.insert(GizmoSpace::World, Vec::new());

        Self {
            lines,
        }
    }

    pub fn space(&mut self, space: GizmoSpace) -> &mut Vec<GizmoLine> {
        self.lines.get_mut(&space).unwrap()
    }

    pub fn spaces(&mut self) -> IterMut<'_, GizmoSpace, Vec<GizmoLine>> {
        self.lines.iter_mut()
    }
}

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

#[derive(Resource)]
pub struct DepthTextureResource {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub sampler: Sampler,
}

#[derive(Resource)]
pub struct StandardRenderResource {
    pub pipeline_layout: PipelineLayout,
    pub instance_buffer: Buffer,
    pub instance_cpu_buffer: Box<[[[f32; 4]; 4]]>,
    pub camera_pos: Vec3<f32>,
    pub camera_proj_matrix: Mat4<f32>,
    pub lit: MaterialStandardRenderResourceData,
    pub unlit: MaterialStandardRenderResourceData,
}

pub struct MaterialStandardRenderResourceData {
    pub buffer_uniform: Buffer,
    pub bind_group_uniform: BindGroup,
    pub render_pipeline: RenderPipeline,
}

pub struct UnlitStandardRenderResourceData {

}
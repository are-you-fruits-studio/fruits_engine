use std::collections::{hash_map::IterMut, HashMap};

use fruits_ecs::Resource;
use wgpu::{BindGroup, BindGroupLayout, Buffer, PipelineLayout, RenderPipeline, Sampler, ShaderModule, SurfaceTexture, Texture, TextureView};

use super::{GizmoLine, GizmoSpace, StandardGlobalUniform};

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

        lines.insert(GizmoSpace::Viewport, Vec::new());
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
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub color_buffer: Buffer,
    pub transform_buffer: Buffer,
    pub pipeline: RenderPipeline,
    pub bind_group: BindGroup,
}

#[derive(Resource)]
pub struct DepthTextureResource {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub sampler: Sampler,
}

#[derive(Resource)]
pub struct StandardRenderResource {
    pub shader: ShaderModule,
    pub pipeline_layout: PipelineLayout,
    pub instance_buffer: Buffer,
    pub uniform: StandardGlobalUniform,
    pub uniform_buffer: Buffer,
    pub uniform_bind_group: BindGroup,
    pub material_uniform_bind_group_layout: BindGroupLayout,
}
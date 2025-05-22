use std::collections::{hash_map::IterMut, HashMap};

use fruits_ecs_data::Resource;
use fruits_ecs_macros::Resource;
use wgpu::{BindGroup, BindGroupLayout, Buffer, RenderPipeline, SurfaceTexture};

use super::{GizmoLine, GizmoSpace};

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
    lines: HashMap<GizmoSpace, Vec<GizmoLine>>,
}
impl GizmosResource {
    pub fn new() -> Self {
        let mut lines = HashMap::new();

        lines.insert(GizmoSpace::Viewport, Vec::new());
        lines.insert(GizmoSpace::Screen, Vec::new());
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
    pub pipeline: RenderPipeline,
    pub bind_group: BindGroup,
}

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
    render_pipeline: RenderPipeline,
    uniform: StandardMaterialUniform,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
}

impl StandardMaterial {
    pub fn from_world(world: &WorldData) -> Self {
        let render_state = &**world.resources().get::<RenderStateResource>().unwrap();
        let render_res = world.resources().get::<StandardRenderResource>().unwrap();
        let depth_tex = world.resources().get::<DepthTextureResource>().unwrap();

        let uniform_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
            label: Some("Standard Material Uniform Buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: fruits_utils::mem::as_bytes(&[StandardMaterialUniform::default()]),
        });

        let uniform_bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
            label: Some("Standard Material Uniform Bind Group"),
            layout: &render_res.material_uniform_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let render_pipeline = render_state.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Standard Material Render Pipeline"),
            layout: Some(&render_res.pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_res.shader,
                entry_point: "vs_main",
                buffers: &[
                    StandardVertex::desc(),
                    StandardInstance::desc(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_res.shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_state.surface_config().format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                bias: Default::default(),
                depth_compare: wgpu::CompareFunction::LessEqual,
                depth_write_enabled: true,
                format: depth_tex.texture.format(),
                stencil: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            uniform: StandardMaterialUniform::default(),
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }

    pub fn uniform(&self) -> &StandardMaterialUniform {
        &self.uniform
    }

    pub fn uniform_mut(&mut self) -> &mut StandardMaterialUniform {
        &mut self.uniform
    }

    pub fn uniform_buffer(&self) -> &Buffer {
        &self.uniform_buffer
    }

    pub fn uniform_bind_group(&self) -> &BindGroup {
        &self.uniform_bind_group
    }
}

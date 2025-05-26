
use wgpu::{BindGroupLayout, DepthStencilState, Device, RenderPipeline, SurfaceConfiguration};

use crate::render::DepthTextureResource;

use super::{mesh::StandardVertex, shader::Shader, StandardInstance};

pub struct StandardMaterialNew {
    pub color: [f32; 3],
    pub texture: (),
    pub emission: [f32; 3],
}

pub struct Material {
    render_pipeline: RenderPipeline,
}

impl Material {
    pub fn from_render_pipeline(render_pipeline: RenderPipeline) -> Self {
        Self {
            render_pipeline,
        }
    }

    pub fn new(
        device: &Device,
        surface_config: &SurfaceConfiguration,
        shader: &Shader,
        bind_group_layouts: &[&BindGroupLayout],
        depth_res: &DepthTextureResource,
    ) -> Self {
        let shader = shader.shader_module();

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    StandardVertex::desc(),
                    StandardInstance::desc(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
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
                format: depth_res.texture.format(),
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
            render_pipeline
        }
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }
}

use core::f32;
use std::collections::HashMap;

use fruits_ecs::*;
use fruits_math::*;
use fruits_render_core::*;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt}, wgt::TextureViewDescriptor, *,
};

use fruits_transform::*;
use crate::{utils::*, *};

use super::{
    DepthTextureResource, GizmosRenderResource, GizmosResource, StandardRenderResource,
    components::{CameraComponent, StandardMaterialComponent, StandardMeshComponent},
    resources::MainRenderTargetResource,
};

// todo: refactor this file's duplications

pub fn create_standard_render_resource(mut world: WorldDataMut) {
    let render_state = unsafe { world.as_ref().resources().get::<RenderApiResource>().unwrap().raw() };

    let depth_tex = world.as_ref().resources().get::<DepthTextureResource>().unwrap();
    let transparent_target_tex = world.as_ref().resources().get::<TransparentTargetTextureResource>().unwrap();
    let main_render_target_res = world.as_ref().resources().get::<MainRenderTargetResource>().unwrap();

    let pipeline_layout_standard = render_state.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Standard Pipeline Layout"),
        bind_group_layouts: &[
            &render_state.render_data().bind_group_layout_global,
            &render_state.render_data().bind_group_layout_material,
        ],
        push_constant_ranges: &[],
    });

    let pipeline_layout_transparent_final = render_state.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Transparent Final Pipeline Layout"),
        bind_group_layouts: &[&transparent_target_tex.bind_group_layout],
        push_constant_ranges: &[],
    });

    let create_render_pipeline_fn = |is_lit: bool, is_transparent: bool| -> RenderPipeline {
        let shader = render_state.device().create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(super::assets::shader_standard(is_lit, is_transparent).into()),
        });

        let color_target_state = match is_transparent {
            false => wgpu::ColorTargetState {
                format: main_render_target_res.texture.format(),
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            },
            true => wgpu::ColorTargetState {
                format: transparent_target_tex.texture.format(),
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            },
        };

        let render_pipeline = render_state.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Standard Render Pipeline"),
            layout: Some(&pipeline_layout_standard),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[StandardVertex::desc(), StandardInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(color_target_state)],
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
                depth_write_enabled: !is_transparent,
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

        render_pipeline
    };

    let render_pipeline_opaque_unlit = create_render_pipeline_fn(false, false);
    let render_pipeline_opaque_lit = create_render_pipeline_fn(true, false);
    let render_pipeline_transparent_unlit = create_render_pipeline_fn(false, true);
    let render_pipeline_transparent_lit = create_render_pipeline_fn(true, true);

    let render_pipeline_transparent_shader = render_state
        .device()
        .create_shader_module(include_wgsl!("./assets/shader_transparent_final.wgsl"));

    let render_pipeline_transparent_final = render_state.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Transparent Final Render Pipeline"),
        layout: Some(&pipeline_layout_transparent_final),
        vertex: wgpu::VertexState {
            module: &render_pipeline_transparent_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &render_pipeline_transparent_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: main_render_target_res.texture.format(),
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::COLOR,
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
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    let instance_cpu_buffer =
        vec![Mat4::<f32>::IDENTITY.into_array(); INSTANCES_PER_DRAW_MAX].into_boxed_slice();

    let instance_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Instance Buffer"),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes_slice(&instance_cpu_buffer),
    });

    let batched_vertex_cpu_buffer =
        vec![StandardVertex::default(); TRIANGLES_PER_BATCHED_DRAW_MAX * 3].into_boxed_slice();

    let batched_vertex_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Batch vertex buffer"),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes_slice(&batched_vertex_cpu_buffer),
    });

    let lights_cpu_buffer =
        vec![StandardGenericLight::default(); LIGHTS_COUNT_MAX].into_boxed_slice();

    let lights_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Lights buffer"),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes_slice(&lights_cpu_buffer),
    });

    let global_uniform_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Standard Global Uniform Buffer"),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(&[StandardUniformGlobal::default()]),
    });

    let global_bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
        label: Some("Standard Global Bind Group"),
        layout: &render_state.render_data().bind_group_layout_global,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: global_uniform_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: lights_buffer.as_entire_binding(),
            },
        ],
    });

    world.as_mut()
        .resources_mut()
        .insert(StandardRenderResource {
            pipeline_layout: pipeline_layout_standard,
            global_uniform_buffer,
            global_bind_group,
            instance_cpu_buffer,
            instance_buffer,
            batched_vertex_buffer,
            lights_buffer,
            lights_count: 0,
            render_pipeline_opaque_lit,
            render_pipeline_opaque_unlit,
            render_pipeline_transparent_lit,
            render_pipeline_transparent_unlit,
            render_pipeline_transparent_final,
            camera_pos: Vec3::default(),
            camera_proj_matrix: Mat4::IDENTITY,
        });

    world.as_mut()
        .resources_mut()
        .insert(BatchedVertexCpuBufferResource(batched_vertex_cpu_buffer));

    world.as_mut()
        .resources_mut()
        .insert(LightsCpuBufferResource(lights_cpu_buffer));
}

pub fn recreate_main_render_target_resource(mut world: WorldDataMut) {
    let render_api = world.as_ref().resources().get::<RenderApiResource>().unwrap();

    let screen_size = render_api.size();

    let render_state = unsafe { render_api.raw() };

    let mut contains_main_render_target = false;

    if let Some(main_render_target_res) = world.as_ref().resources().get::<MainRenderTargetResource>() {
        contains_main_render_target = true;

        let are_same_size = {
            main_render_target_res.texture.size().width == screen_size[0]
                && main_render_target_res.texture.size().height == screen_size[1]
        };

        if are_same_size {
            return;
        }
    }

    let texture = render_state.device().create_texture(&TextureDescriptor {
        label: Some("Main Render Target"),
        size: Extent3d {
            width: screen_size[0],
            height: screen_size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let texture_view = texture.create_view(&TextureViewDescriptor::default());

    let sampler = render_state.device().create_sampler(&SamplerDescriptor {
        label: Some("Render Surface Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        compare: None,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    let bind_group_layout = render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Render Surface Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                count: None,
                ty: BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                visibility: ShaderStages::VERTEX_FRAGMENT,
            },
            BindGroupLayoutEntry {
                binding: 1,
                count: None,
                ty: BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                visibility: ShaderStages::VERTEX_FRAGMENT,
            },
        ],
    });

    let bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
        label: Some("Render Surface Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let pipeline_layout = render_state.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Surface Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let shader = render_state
        .device()
        .create_shader_module(include_wgsl!("./assets/shader_render_surface.wgsl"));

    let render_pipeline = render_state.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Surface Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: render_state.surface_config_format(),
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
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    let main_render_target_res = MainRenderTargetResource {
        texture,
        texture_view,
        sampler,
        bind_group_layout,
        bind_group,
        render_pipeline,
    };

    if contains_main_render_target {
        *world.resources_mut().get_mut().unwrap() = main_render_target_res;
    } else {
        world.resources_mut().insert(main_render_target_res);
    }
}

pub fn recreate_depth_texture_resource(mut world: WorldDataMut) {
    let render_api = world.as_ref().resources().get::<RenderApiResource>().unwrap();

    let screen_size = render_api.size();

    let render_state = unsafe { render_api.raw() };

    let mut contains_depth = false;

    if let Some(depth_res) = world.as_ref().resources().get::<DepthTextureResource>() {
        contains_depth = true;

        let are_same_size =
            { depth_res.texture.size().width == screen_size[0] && depth_res.texture.size().height == screen_size[1] };

        if are_same_size {
            return;
        }
    }

    let texture = render_state.device().create_texture(&TextureDescriptor {
        label: Some("Depth Buffer"),
        size: Extent3d {
            width: screen_size[0],
            height: screen_size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let texture_view = texture.create_view(&TextureViewDescriptor::default());

    let sampler = render_state.device().create_sampler(&SamplerDescriptor {
        label: Some("Depth Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    let depth_res = DepthTextureResource {
        texture,
        texture_view,
        sampler,
    };

    if contains_depth {
        *world.resources_mut().get_mut().unwrap() = depth_res;
    } else {
        world.resources_mut().insert(depth_res);
    }
}

pub fn recreate_transparent_target_resource(mut world: WorldDataMut) {
    let render_api = world.as_ref().resources().get::<RenderApiResource>().unwrap();

    let screen_size = render_api.size();

    let render_state = unsafe { render_api.raw() };

    let mut contains_transparent_target = false;

    if let Some(transparent_target_res) = world.as_ref().resources().get::<TransparentTargetTextureResource>() {
        contains_transparent_target = true;

        let are_same_size = {
            transparent_target_res.texture.size().width == screen_size[0]
                && transparent_target_res.texture.size().height == screen_size[1]
        };

        if are_same_size {
            return;
        }
    }

    let texture = render_state.device().create_texture(&TextureDescriptor {
        label: Some("Transparent Target Buffer"),
        size: Extent3d {
            width: screen_size[0],
            height: screen_size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let texture_view = texture.create_view(&TextureViewDescriptor::default());

    let sampler = render_state.device().create_sampler(&SamplerDescriptor {
        label: Some("Transparent Target Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        compare: None,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    let bind_group_layout = render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Transparent Target Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                count: None,
                ty: BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                visibility: ShaderStages::VERTEX_FRAGMENT,
            },
            BindGroupLayoutEntry {
                binding: 1,
                count: None,
                ty: BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                visibility: ShaderStages::VERTEX_FRAGMENT,
            },
        ],
    });

    let bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
        label: Some("Transparent Target Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let transparent_target_res = TransparentTargetTextureResource {
        texture,
        texture_view,
        sampler,
        bind_group_layout,
        bind_group,
    };

    if contains_transparent_target {
        *world.resources_mut().get_mut().unwrap() = transparent_target_res;
    } else {
        world.resources_mut().insert(transparent_target_res);
    }
}

pub fn recreate_bloom_render_resource(mut world: WorldDataMut) {
    let render_api = world.as_ref().resources().get::<RenderApiResource>().unwrap();

    let screen_size = render_api.size();

    let render_state = unsafe { render_api.raw() };

    let mut contains_transparent_target = false;

    if let Some(render_res) = world.as_ref().resources().get::<BloomRenderResource>() {
        contains_transparent_target = true;

        let are_same_size = {
            render_res.textures[0].size().width == screen_size[0]
                && render_res.textures[0].size().height == screen_size[1]
                && render_res.textures[1].size().width == screen_size[0]
                && render_res.textures[1].size().height == screen_size[1]
        };

        if are_same_size {
            return;
        }
    }

    let textures = std::array::from_fn::<_, 2, _>(|i| render_state.device().create_texture(&TextureDescriptor {
        label: Some(&format!("Bloom Buffer {i}")),
        size: Extent3d {
            width: screen_size[0],
            height: screen_size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    }));

    let sampler = render_state.device().create_sampler(&SamplerDescriptor {
        label: Some("Transparent Target Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Nearest,
        compare: None,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    // todo
    // let bind_group_layout = render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
    //     label: Some("Transparent Target Bind Group Layout"),
    //     entries: &[
    //         BindGroupLayoutEntry {
    //             binding: 0,
    //             count: None,
    //             ty: BindingType::Texture {
    //                 sample_type: wgpu::TextureSampleType::Float { filterable: false },
    //                 view_dimension: wgpu::TextureViewDimension::D2,
    //                 multisampled: false,
    //             },
    //             visibility: ShaderStages::VERTEX_FRAGMENT,
    //         },
    //         BindGroupLayoutEntry {
    //             binding: 1,
    //             count: None,
    //             ty: BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
    //             visibility: ShaderStages::VERTEX_FRAGMENT,
    //         },
    //     ],
    // });

    // let bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
    //     label: Some("Transparent Target Bind Group"),
    //     layout: &bind_group_layout,
    //     entries: &[
    //         BindGroupEntry {
    //             binding: 0,
    //             resource: wgpu::BindingResource::TextureView(&texture_view),
    //         },
    //         BindGroupEntry {
    //             binding: 1,
    //             resource: wgpu::BindingResource::Sampler(&sampler),
    //         },
    //     ],
    // });

    let render_res = BloomRenderResource {
        textures,
        sampler,
    };

    if contains_transparent_target {
        *world.resources_mut().get_mut().unwrap() = render_res;
    } else {
        world.resources_mut().insert(render_res);
    }
}

pub fn create_gizmos_render_resource(mut world: WorldDataMut) {
    let render_state = unsafe { world.as_ref().resources().get::<RenderApiResource>().unwrap().raw() };
    let main_render_target_res = world.as_ref().resources().get::<MainRenderTargetResource>().unwrap();

    let vertices_cpu_buffer = vec![[Vec4::splat(0.0); 2]; GIZMO_LINES_PER_DRAW_MAX].into_boxed_slice();
    let colors_cpu_buffer = vec![Vec4::splat(0.0); GIZMO_LINES_PER_DRAW_MAX].into_boxed_slice();

    let vertex_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Gizmos Vertex Buffer"),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes_slice(&vertices_cpu_buffer),
    });

    let color_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Gizmos Color Buffer"),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes_slice(&colors_cpu_buffer),
    });

    let transform_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Gizmos Transform Buffer"),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(Mat4::<f32>::IDENTITY.as_array()),
    });

    let bind_group_layout = render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Gizmos Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                count: None,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                visibility: ShaderStages::VERTEX,
            },
            BindGroupLayoutEntry {
                binding: 1,
                count: None,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                visibility: ShaderStages::VERTEX,
            },
            BindGroupLayoutEntry {
                binding: 2,
                count: None,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                visibility: ShaderStages::VERTEX,
            },
        ],
    });

    let bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
        label: Some("Gizmos Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: vertex_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: color_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: transform_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = render_state.device().create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Gizmos Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let shader = render_state
        .device()
        .create_shader_module(include_wgsl!("./assets/shader_gizmo.wgsl"));

    let pipeline = render_state.device().create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Gizmos Render Pipeline"),
        cache: None,
        depth_stencil: None,
        layout: Some(&pipeline_layout),
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        primitive: wgpu::PrimitiveState {
            topology: PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fragment_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: main_render_target_res.texture.format(),
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vertex_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
    });

    world.as_mut()
        .resources_mut()
        .insert(GizmosRenderResource {
            vertex_buffer,
            color_buffer,
            transform_buffer,
            bind_group,
            pipeline,
            vertices_cpu_buffer,
            colors_cpu_buffer,
        });
}

pub fn update_camera_uniform(
    render_state: Res<RenderApiResource>,
    mut standard_render_res: ResMut<StandardRenderResource>,
    query: WorldQuery<(&GlobalTransform, &CameraComponent)>,
) {
    if query.len() == 0 {
        return;
    }

    if query.len() > 1 {
        panic!("There should be no more than one camera in the world.");
    }

    let (transform, camera) = query.iter().next().unwrap();

    let window_size = render_state.size();

    let aspect = window_size[0] as f32 / window_size[1] as f32;

    let projection_matrix = fruits_math::perspective_proj_matrix(camera.fov, camera.near, camera.far, aspect);

    let transform_matrix = transform
        .scale_rotation
        .into_4x4_with_offset(transform.position)
        .inverse()
        .unwrap();

    standard_render_res.camera_proj_matrix = projection_matrix * transform_matrix;
    standard_render_res.camera_pos = transform.position;
}

pub fn render_main_target_to_surface_system(
    render_api: Res<RenderApiResource>,
    main_render_target: Res<MainRenderTargetResource>
) {
    let surface_texture = unsafe { render_api.raw().surface().get_current_texture().ok() };

    let Some(surface_texture) = surface_texture else {
        return;
    };

    let render_state = unsafe { render_api.raw() };

    let view = &surface_texture.texture.create_view(&Default::default());

    let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Surface Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Surface Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        render_pass.set_pipeline(&main_render_target.render_pipeline);
        render_pass.set_bind_group(0, &main_render_target.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    render_state.queue().submit(std::iter::once(encoder.finish()));
    
    surface_texture.present();
}

pub fn clear_depth(render_api: Res<RenderApiResource>, depth_res: Res<DepthTextureResource>) {
    let render_state = unsafe { render_api.raw() };

    let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Clear Depth Encoder"),
    });

    {
        let mut _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Depth Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_res.texture_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
    }

    render_state.queue().submit(std::iter::once(encoder.finish()));
}

pub fn clear_main_render_target(
    render_api: Res<RenderApiResource>,
    main_render_res: Res<MainRenderTargetResource>,
) {
    let render_state = unsafe { render_api.raw() };

    let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Clear Main Render Target Encoder"),
    });

    {
        let mut _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Main Render Target Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &main_render_res.texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
    }

    render_state.queue().submit(std::iter::once(encoder.finish()));
}

pub fn clear_transparent_target(render_api: Res<RenderApiResource>, transparent_target_res: Res<TransparentTargetTextureResource>) {
    let render_state = unsafe { render_api.raw() };

    let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Clear Transparent Target Encoder"),
    });

    {
        let mut _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Transparent Target Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &transparent_target_res.texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
    }

    render_state.queue().submit(std::iter::once(encoder.finish()));
}

pub fn update_lights_buffer(
    light_q: WorldQuery<(
        Option<&GlobalTransform>,
        &StandardLightComponent,
    )>,
    render_api: Res<RenderApiResource>,
    mut standard_render_res: ResMut<StandardRenderResource>,
    mut lights_cpu_buffer: ResMut<LightsCpuBufferResource>,
) {
    let render_state = unsafe { render_api.raw() };

    standard_render_res.lights_count = light_q.len().min(LIGHTS_COUNT_MAX as u64) as u32;

    for (i, (light_transform, light_c)) in light_q.iter().take(LIGHTS_COUNT_MAX).enumerate() {
        let light_transform = light_transform.copied().unwrap_or_default();

        let light = utils::light_from_components(light_c, &light_transform).into();

        lights_cpu_buffer.0[i] = light;
    }

    let lights_bytes = fruits_utils::mem::as_bytes_slice(&lights_cpu_buffer.0);

    render_state
        .queue()
        .write_buffer(&standard_render_res.lights_buffer, 0, lights_bytes);

    render_state.queue().submit([]);
}

pub fn update_global_uniforms(
    render_api: Res<RenderApiResource>,
    standard_render_res: Res<StandardRenderResource>,
) {
    let render_state = unsafe { render_api.raw() };

    let uniform = StandardUniformGlobal {
        lights_count: standard_render_res.lights_count,
        camera_position_world: standard_render_res.camera_pos,
    };

    render_state.queue().write_buffer(
        &standard_render_res.global_uniform_buffer,
        0,
        fruits_utils::mem::as_bytes(&[uniform]),
    );
}

pub fn render_opaque_instanced(
    mesh_q: WorldQuery<(
        Option<&GlobalTransform>,
        &StandardMeshComponent,
        &StandardMaterialComponent,
        Option<&GlobalDisableableComponent>,
    )>,
    render_api: Res<RenderApiResource>,
    screen_space_res: Res<ScreenSpaceResource>,
    standard_render_res: Res<StandardRenderResource>,
    depth_res: Res<DepthTextureResource>,
    surface_texture: Res<MainRenderTargetResource>,
    meshes: Res<AssetStorageResource<StandardMesh>>,
    materials: Res<AssetStorageResource<StandardMaterial>>,
) {
    if mesh_q.len() == 0 {
        return;
    }

    let render_state = unsafe { render_api.raw() };

    let view = &surface_texture.texture_view;

    let window_size = render_state.size();

    let window_to_clip_mat = crate::utils::create_window_to_clip_matrix(
        window_size[0] as f32,
        window_size[1] as f32,
        screen_space_res.near,
        screen_space_res.far,
    );

    let mut instanced_matrices = HashMap::new();

    for (transform, render_mesh, render_material, disableable) in mesh_q.iter() {
        if disableable.copied().unwrap_or_default().is_disabled {
            continue;
        }

        let Some(material) = materials.get(&render_material.material) else {
            continue;
        };

        if material.meta().alpha_threshold.is_none() {
            continue;
        }

        let mat = match transform {
            Some(transform) => transform.scale_rotation.into_4x4_with_offset(transform.position),
            None => Mat4::IDENTITY,
        };
        instanced_matrices
            .entry((render_mesh.mesh.clone(), render_material.material.clone()))
            .or_insert_with(|| Vec::new())
            .push(mat);
    }

    for ((mesh, material), matrices) in instanced_matrices.iter() {
        let Some(mesh) = meshes.get(&mesh) else {
            continue;
        };
        let Some(material) = materials.get(&material) else {
            continue;
        };

        let mesh = unsafe { mesh.native() };

        let (render_pipeline, bind_group) = crate::utils::get_render_data(
            material,
            &standard_render_res,
            &render_state,
            window_to_clip_mat,
        );

        for matrices in matrices.chunks(INSTANCES_PER_DRAW_MAX) {
            let matrices_bytes = fruits_utils::mem::as_bytes_slice(matrices);

            render_state
                .queue()
                .write_buffer(&standard_render_res.instance_buffer, 0, matrices_bytes);

            let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_res.texture_view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });

                render_pass.set_pipeline(render_pipeline);
                render_pass.set_bind_group(0, &standard_render_res.global_bind_group, &[]);
                render_pass.set_bind_group(1, bind_group, &[]);
                render_pass.set_vertex_buffer(1, standard_render_res.instance_buffer.slice(..));
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..(mesh.indices_count as u32), 0, 0..(matrices.len() as u32));
            }

            render_state.queue().submit(std::iter::once(encoder.finish()));
        }
    }
}

pub fn render_opaque_batched(
    query: WorldQuery<(
        Option<&GlobalTransform>,
        &BatchedMeshComponent,
        &StandardMaterialComponent,
        Option<&GlobalDisableableComponent>,
    )>,
    render_api: Res<RenderApiResource>,
    screen_space_res: Res<ScreenSpaceResource>,
    standard_render_res: Res<StandardRenderResource>,
    depth_res: Res<DepthTextureResource>,
    surface_texture: Res<MainRenderTargetResource>,
    materials: Res<AssetStorageResource<StandardMaterial>>,
    mut batched_vertex_cpu_buffer: ResMut<BatchedVertexCpuBufferResource>,
) {
    if query.len() == 0 {
        return;
    }

    let render_state = unsafe { render_api.raw() };

    let view = &surface_texture.texture_view;

    let window_size = render_state.size();

    let window_to_clip_mat = crate::utils::create_window_to_clip_matrix(
        window_size[0] as f32,
        window_size[1] as f32,
        screen_space_res.near,
        screen_space_res.far,
    );

    let mut batched_meshes_by_material = HashMap::new();

    for (transform, batched_mesh, render_material, disableable) in query.iter() {
        if disableable.copied().unwrap_or_default().is_disabled {
            continue;
        }

        let Some(material) = materials.get(&render_material.material) else {
            continue;
        };

        if material.meta().alpha_threshold.is_none() {
            continue;
        }

        let mat = match transform {
            Some(transform) => transform.scale_rotation.into_4x4_with_offset(transform.position),
            None => Mat4::IDENTITY,
        };
        batched_meshes_by_material
            .entry(render_material.material.clone())
            .or_insert_with(|| Vec::new())
            .push((mat, batched_mesh));
    }

    render_state.queue().write_buffer(
        &standard_render_res.instance_buffer,
        0,
        fruits_utils::mem::as_bytes(&Mat4::<f32>::IDENTITY),
    );
    render_state.queue().submit([]);

    let batch_cpu_buffer = &mut batched_vertex_cpu_buffer.0;

    for (material, matrices_and_meshes) in &batched_meshes_by_material {
        let Some(material) = materials.get(&material) else {
            continue;
        };

        let (render_pipeline, bind_group) = crate::utils::get_render_data(
            material,
            &standard_render_res,
            &render_state,
            window_to_clip_mat,
        );

        let mut batch_buffer_i = 0;

        let srgb_to_linear = |x: f32| x.powf(2.2);
        for (mat, batched_mesh) in matrices_and_meshes {
            for &i in &batched_mesh.indices {
                let mut vertex = batched_mesh.vertices[i as u64];

                vertex.position = mat.mul_with_projection(Vec3::from_array(vertex.position)).into_array();
                vertex.normal = mat.mul_with_projection_as_dir(Vec3::from_array(vertex.normal)).into_array();
                vertex.color = [
                    srgb_to_linear(vertex.color[0]),
                    srgb_to_linear(vertex.color[1]),
                    srgb_to_linear(vertex.color[2]),
                    vertex.color[3],
                ];

                batch_cpu_buffer[batch_buffer_i] = vertex;

                batch_buffer_i += 1;

                if batch_buffer_i < batch_cpu_buffer.len() {
                    continue;
                }

                batch_buffer_i = 0;

                submit_render_batched(
                    &render_state,
                    &standard_render_res,
                    &batch_cpu_buffer[..],
                    view,
                    &depth_res,
                    render_pipeline,
                    bind_group,
                );
            }
        }

        if batch_buffer_i > 0 {
            submit_render_batched(
                &render_state,
                &standard_render_res,
                &batch_cpu_buffer[..batch_buffer_i],
                view,
                &depth_res,
                render_pipeline,
                bind_group,
            );
        }
    }
}

// todo: a lot of duplication
pub fn render_transparent_instanced(
    query: WorldQuery<(
        Option<&GlobalTransform>,
        &StandardMeshComponent,
        &StandardMaterialComponent,
        Option<&GlobalDisableableComponent>,
    )>,
    render_api: Res<RenderApiResource>,
    screen_space_res: Res<ScreenSpaceResource>,
    standard_render_res: Res<StandardRenderResource>,
    depth_res: Res<DepthTextureResource>,
    transparent_target_res: Res<TransparentTargetTextureResource>,
    meshes: Res<AssetStorageResource<StandardMesh>>,
    materials: Res<AssetStorageResource<StandardMaterial>>,
) {
    if query.len() == 0 {
        return;
    }

    let render_state = unsafe { render_api.raw() };

    let view = &transparent_target_res.texture_view;

    let window_size = render_state.size();

    let window_to_clip_mat = crate::utils::create_window_to_clip_matrix(
        window_size[0] as f32,
        window_size[1] as f32,
        screen_space_res.near,
        screen_space_res.far,
    );

    let mut instanced_matrices = HashMap::new();

    for (transform, render_mesh, render_material, disableable) in query.iter() {
        if disableable.copied().unwrap_or_default().is_disabled {
            continue;
        }

        let Some(material) = materials.get(&render_material.material) else {
            continue;
        };

        if material.meta().alpha_threshold.is_some() {
            continue;
        }

        let mat = match transform {
            Some(transform) => transform.scale_rotation.into_4x4_with_offset(transform.position),
            None => Mat4::IDENTITY,
        };
        instanced_matrices
            .entry((render_mesh.mesh.clone(), render_material.material.clone()))
            .or_insert_with(|| Vec::new())
            .push(mat);
    }

    for ((mesh, material), matrices) in instanced_matrices.iter() {
        let Some(mesh) = meshes.get(&mesh) else {
            continue;
        };
        let Some(material) = materials.get(&material) else {
            continue;
        };

        let mesh = unsafe { mesh.native() };

        let (render_pipeline, bind_group) = crate::utils::get_render_data(
            material,
            &standard_render_res,
            &render_state,
            window_to_clip_mat,
        );

        for matrices in matrices.chunks(INSTANCES_PER_DRAW_MAX) {
            let matrices_bytes = fruits_utils::mem::as_bytes_slice(matrices);

            render_state
                .queue()
                .write_buffer(&standard_render_res.instance_buffer, 0, matrices_bytes);

            let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_res.texture_view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });

                render_pass.set_pipeline(render_pipeline);
                render_pass.set_bind_group(0, &standard_render_res.global_bind_group, &[]);
                render_pass.set_bind_group(1, bind_group, &[]);
                render_pass.set_vertex_buffer(1, standard_render_res.instance_buffer.slice(..));
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..(mesh.indices_count as u32), 0, 0..(matrices.len() as u32));
            }

            render_state.queue().submit(std::iter::once(encoder.finish()));
        }
    }
}

// todo: a lot of duplication
pub fn render_transparent_batched(
    query: WorldQuery<(
        Option<&GlobalTransform>,
        &BatchedMeshComponent,
        &StandardMaterialComponent,
        Option<&GlobalDisableableComponent>,
    )>,
    render_api: Res<RenderApiResource>,
    screen_space_res: Res<ScreenSpaceResource>,
    standard_render_res: Res<StandardRenderResource>,
    depth_res: Res<DepthTextureResource>,
    transparent_target_res: Res<TransparentTargetTextureResource>,
    materials: Res<AssetStorageResource<StandardMaterial>>,
    mut batched_vertex_cpu_buffer: ResMut<BatchedVertexCpuBufferResource>,
) {
    if query.len() == 0 {
        return;
    }

    let render_state = unsafe { render_api.raw() };

    let view = &transparent_target_res.texture_view;

    let window_size = render_state.size();

    let window_to_clip_mat = crate::utils::create_window_to_clip_matrix(
        window_size[0] as f32,
        window_size[1] as f32,
        screen_space_res.near,
        screen_space_res.far,
    );

    let mut batched_meshes_by_material = HashMap::new();

    for (transform, batched_mesh, render_material, disableable) in query.iter() {
        if disableable.copied().unwrap_or_default().is_disabled {
            continue;
        }

        let Some(material) = materials.get(&render_material.material) else {
            continue;
        };

        if material.meta().alpha_threshold.is_some() {
            continue;
        }

        let mat = match transform {
            Some(transform) => transform.scale_rotation.into_4x4_with_offset(transform.position),
            None => Mat4::IDENTITY,
        };
        batched_meshes_by_material
            .entry(render_material.material.clone())
            .or_insert_with(|| Vec::new())
            .push((mat, batched_mesh));
    }

    render_state.queue().write_buffer(
        &standard_render_res.instance_buffer,
        0,
        fruits_utils::mem::as_bytes(&Mat4::<f32>::IDENTITY),
    );
    render_state.queue().submit([]);

    let batch_cpu_buffer = &mut batched_vertex_cpu_buffer.0;

    for (material, matrices_and_meshes) in &batched_meshes_by_material {
        let Some(material) = materials.get(&material) else {
            continue;
        };

        let (render_pipeline, bind_group) = crate::utils::get_render_data(
            material,
            &standard_render_res,
            &render_state,
            window_to_clip_mat,
        );

        let mut batch_buffer_i = 0;

        let srgb_to_linear = |x: f32| x.powf(2.2);
        for (mat, batched_mesh) in matrices_and_meshes {
            for &i in &batched_mesh.indices {
                let mut vertex = batched_mesh.vertices[i as u64];

                vertex.position = mat.mul_with_projection(Vec3::from_array(vertex.position)).into_array();
                vertex.normal = mat.mul_with_projection_as_dir(Vec3::from_array(vertex.normal)).into_array();
                vertex.color = [
                    srgb_to_linear(vertex.color[0]),
                    srgb_to_linear(vertex.color[1]),
                    srgb_to_linear(vertex.color[2]),
                    vertex.color[3],
                ];

                batch_cpu_buffer[batch_buffer_i] = vertex;

                batch_buffer_i += 1;

                if batch_buffer_i < batch_cpu_buffer.len() {
                    continue;
                }

                batch_buffer_i = 0;

                submit_render_batched(
                    &render_state,
                    &standard_render_res,
                    &batch_cpu_buffer[..],
                    &view,
                    &depth_res,
                    render_pipeline,
                    bind_group,
                );
            }
        }

        if batch_buffer_i > 0 {
            submit_render_batched(
                &render_state,
                &standard_render_res,
                &batch_cpu_buffer[..batch_buffer_i],
                &view,
                &depth_res,
                render_pipeline,
                bind_group,
            );
        }
    }
}

pub fn render_transparent_final(
    render_api: Res<RenderApiResource>,
    standard_render_res: Res<StandardRenderResource>,
    transparent_target_res: Res<TransparentTargetTextureResource>,
    surface_texture: Res<MainRenderTargetResource>,
) {
    let render_state = unsafe { render_api.raw() };

    let view = &surface_texture.texture_view;

    let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Transparent Final Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Transparent Final Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        render_pass.set_pipeline(&standard_render_res.render_pipeline_transparent_final);
        render_pass.set_bind_group(0, &transparent_target_res.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    render_state.queue().submit(std::iter::once(encoder.finish()));
}

pub fn render_gather_bloom_threshold(
    render_api: Res<RenderApiResource>,
    surface_texture: Res<MainRenderTargetResource>,
) {
    // todo
    return;

    let render_state = unsafe { render_api.raw() };

    let view = &surface_texture.texture_view;

    let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Gather Bloom Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Gather Bloom Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        // render_pass.set_pipeline(&standard_render_res.render_pipeline_transparent_final);
        // render_pass.set_bind_group(0, &transparent_target_res.bind_group, &[]);
        // render_pass.draw(0..3, 0..1);
    }

    render_state.queue().submit(std::iter::once(encoder.finish()));
}

pub fn render_gizmos(
    mut gizmos: ResMut<GizmosResource>,
    mut gizmos_render_res: ResMut<GizmosRenderResource>,
    surface_texture: Res<MainRenderTargetResource>,
    screen_space_res: Res<ScreenSpaceResource>,
    render_api: Res<RenderApiResource>,
    camera_query: WorldQuery<(&GlobalTransform, &CameraComponent)>,
) {
    let view = &surface_texture.texture_view;

    let render_state = unsafe { render_api.raw() };

    let window_size = render_state.size();

    for (space, lines) in gizmos.spaces() {
        if lines.len() == 0 {
            continue;
        }

        let transform = match space {
            RenderSpace::Clip => Mat4::<f32>::IDENTITY,
            RenderSpace::Window => crate::utils::create_window_to_clip_matrix(
                window_size[0] as f32,
                window_size[1] as f32,
                screen_space_res.near,
                screen_space_res.far,
            ),
            RenderSpace::World => {
                let Some((transform, camera)) = camera_query.iter().next() else {
                    continue;
                };

                let aspect = window_size[0] as f32 / window_size[1] as f32;

                let projection_matrix = fruits_math::perspective_proj_matrix(camera.fov, camera.near, camera.far, aspect);

                let transform_matrix = transform
                    .scale_rotation
                    .into_4x4_with_offset(transform.position)
                    .inverse()
                    .unwrap();

                projection_matrix * transform_matrix
            }
        };

        render_state.queue().write_buffer(
            &gizmos_render_res.transform_buffer,
            0,
            fruits_utils::mem::as_bytes(transform.as_array()),
        );

        loop {
            if lines.is_empty() {
                break;
            }

            let mut count = 0_usize;

            for i in 0..GIZMO_LINES_PER_DRAW_MAX {
                let Some(line) = lines.pop() else {
                    break;
                };

                gizmos_render_res.vertices_cpu_buffer[i] = [
                    Vec4::new(line.start.x, line.start.y, line.start.z, 1.0),
                    Vec4::new(line.end.x, line.end.y, line.end.z, 1.0),
                ];
                gizmos_render_res.colors_cpu_buffer[i] = line.color;
                count += 1;
            }

            render_state.queue().write_buffer(
                &gizmos_render_res.vertex_buffer,
                0,
                fruits_utils::mem::as_bytes_slice(&gizmos_render_res.vertices_cpu_buffer[..count]),
            );
            render_state.queue().write_buffer(
                &gizmos_render_res.color_buffer,
                0,
                fruits_utils::mem::as_bytes_slice(&gizmos_render_res.colors_cpu_buffer[..count]),
            );

            let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Gizmos Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Gizmos Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                });

                render_pass.set_pipeline(&gizmos_render_res.pipeline);
                render_pass.set_bind_group(0, &gizmos_render_res.bind_group, &[]);
                render_pass.draw(0..(count as u32 * 2), 0..1);
            }

            render_state.queue().submit(std::iter::once(encoder.finish()));
        }
    }
}

//

fn submit_render_batched(
    render_state: &RenderState,
    standard_render_res: &StandardRenderResource,
    batched_cpu_slice: &[StandardVertex],
    render_target_view: &TextureView,
    depth_res: &DepthTextureResource,
    render_pipeline: &RenderPipeline,
    bind_group: &BindGroup,
) {
    render_state.queue().write_buffer(
        &standard_render_res.batched_vertex_buffer,
        0,
        fruits_utils::mem::as_bytes_slice(batched_cpu_slice),
    );

    let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: render_target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_res.texture_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &standard_render_res.global_bind_group, &[]);
        render_pass.set_bind_group(1, bind_group, &[]);
        render_pass.set_vertex_buffer(1, standard_render_res.instance_buffer.slice(..));
        render_pass.set_vertex_buffer(0, standard_render_res.batched_vertex_buffer.slice(..));
        render_pass.draw(0..(batched_cpu_slice.len() as u32), 0..1);
    }

    render_state.queue().submit(std::iter::once(encoder.finish()));
}
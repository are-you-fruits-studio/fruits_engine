use core::f32;
use std::collections::HashMap;

use fruits_ecs::{Entity, ExclusiveWorldAccess, Res, ResMut, WithFilter, WorldDataMut, WorldQuery};
use fruits_math::{Mat4, Vec2, Vec3, Vec4};
use image::GenericImageView;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType,
    BufferUsages, CommandEncoderDescriptor, DepthStencilState, Extent3d, FilterMode, FragmentState, FrontFace, IndexFormat, LoadOp,
    MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveTopology, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StoreOp, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    VertexState, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    ChildComponent, ChildrenRectMaskComponent, ParentComponent, RenderApiResource, RenderState, TransparentTargetTextureResource,
    UiVal,
    asset::{AssetHandle, AssetStorageResource},
    render::{
        BatchedMeshComponent, BatchedVertexCpuBufferResource, Font, GlobalDisableableComponent, HorizontalAlign, ImageComponent,
        ScreenSpaceResource, StandardInstance, StandardRenderAssetsResource, StandardTexture, StandardUniform, StandardVertex,
        TextComponent, VerticalAlign,
        utils::{
            self, TRIANGLES_PER_BATCHED_DRAW_MAX, GIZMO_LINES_PER_DRAW_MAX,
            INSTANCES_PER_DRAW_MAX,
        },
    },
    transform::{GlobalRectComponent, GlobalTransform},
};

use super::{
    DepthTextureResource, GizmosRenderResource, GizmosResource, ImageFillSettings, RenderSpace, StandardRenderResource,
    assets::{StandardMaterial, StandardMesh},
    components::{CameraComponent, StandardMaterialComponent, StandardMeshComponent},
    resources::SurfaceTextureResource,
};

pub fn create_standard_render_resource(mut world: ExclusiveWorldAccess) {
    let render_state = world.resources().get::<RenderApiResource>().unwrap().raw();

    let depth_tex = world.resources().get::<DepthTextureResource>().unwrap();
    let transparent_target_tex = world.resources().get::<TransparentTargetTextureResource>().unwrap();

    let bind_group_layout = render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Standard Bind Group Layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let pipeline_layout_standard = render_state.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Standard Pipeline Layout"),
        bind_group_layouts: &[
            &bind_group_layout,
            &render_state.render_data().bind_group_layout_standard_texture,
        ],
        push_constant_ranges: &[],
    });

    let pipeline_layout_transparent_final = render_state.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Transparent Final Pipeline Layout"),
        bind_group_layouts: &[&transparent_target_tex.bind_group_layout],
        push_constant_ranges: &[],
    });

    let standard_uniform_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Lit Uniform Buffer"),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(&[StandardUniform::default()]),
    });

    let standard_bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
        label: Some("Lit Uniform Bind Group"),
        layout: &bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: standard_uniform_buffer.as_entire_binding(),
        }],
    });

    let create_render_pipeline_fn = |is_lit: bool, is_transparent: bool| -> RenderPipeline {
        let shader = render_state.device().create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(super::assets::shader_standard(is_lit, is_transparent).into()),
        });

        let color_target_state = match is_transparent {
            false => wgpu::ColorTargetState {
                format: render_state.surface_config().format,
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
                format: render_state.surface_config().format,
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

    world
        .resources_mut()
        .insert(StandardRenderResource {
            pipeline_layout: pipeline_layout_standard,
            instance_cpu_buffer,
            instance_buffer,
            batched_vertex_buffer,
            buffer_uniform: standard_uniform_buffer,
            bind_group_uniform: standard_bind_group,
            render_pipeline_opaque_lit,
            render_pipeline_opaque_unlit,
            render_pipeline_transparent_lit,
            render_pipeline_transparent_unlit,
            render_pipeline_transparent_final,
            camera_pos: Vec3::default(),
            camera_proj_matrix: Mat4::IDENTITY,
        })
        .ok()
        .unwrap();

    world
        .resources_mut()
        .insert(BatchedVertexCpuBufferResource(batched_vertex_cpu_buffer))
        .ok()
        .unwrap();

    let render_api = world.resources().get::<RenderApiResource>().unwrap();

    let texture_white = render_api.create_texture(FilterMode::Linear, [2, 2], &[255; 16]);

    let texture_white = world
        .resources_mut()
        .get_mut::<AssetStorageResource<StandardTexture>>()
        .unwrap()
        .insert(texture_white);

    let (texture_text_px_5_7, font_px_5_7) =
        create_ascii_monospace_font(world.as_mut(), include_bytes!("./assets/ascii_px_5x7.png"));
    let (texture_text_px_8_8, font_px_8_8) =
        create_ascii_monospace_font(world.as_mut(), include_bytes!("./assets/ascii_px_8x8.png"));
    let (texture_text_px_8_12, font_px_8_12) =
        create_ascii_monospace_font(world.as_mut(), include_bytes!("./assets/ascii_px_8x12.png"));

    world
        .resources_mut()
        .insert(StandardRenderAssetsResource {
            texture_white,
            texture_text_px_5_7,
            font_px_5_7,
            texture_text_px_8_8,
            font_px_8_8,
            texture_text_px_8_12,
            font_px_8_12,
        })
        .ok()
        .unwrap();
}

fn create_ascii_monospace_font(mut world: WorldDataMut, texture_bytes: &[u8]) -> (AssetHandle<StandardTexture>, AssetHandle<Font>) {
    let image = image::load_from_memory(texture_bytes).unwrap();

    let texture_dimensions: [u32; 2] = image.dimensions().into();

    let render_api = world.resources().get::<RenderApiResource>().unwrap();

    let texture = render_api.create_texture(FilterMode::Nearest, texture_dimensions, image.as_bytes());

    let text_chars_count = [16, 8];
    let single_char_uv_size = [1.0 / text_chars_count[0] as f32, 1.0 / text_chars_count[1] as f32];

    let characters_uv = (' '..='~')
        .map(|c| {
            let char_uv_index = [c as i32 % text_chars_count[0], c as i32 / text_chars_count[0]];
            let char_uv_min = fruits_math::zip(&char_uv_index, &text_chars_count, |a, b| *a as f32 / *b as f32);

            let char_uvs = [
                Vec2::from_array(char_uv_min),
                Vec2::from_array(fruits_math::zip(&char_uv_min, &single_char_uv_size, |a, b| a + b)),
            ];

            (c, char_uvs)
        })
        .collect::<HashMap<_, _>>();

    let texture = world
        .resources_mut()
        .get_mut::<AssetStorageResource<StandardTexture>>()
        .unwrap()
        .insert(texture);

    let font = Font {
        texture: texture.clone(),
        missing_character_uv: characters_uv[&'?'],
        characters_uv,
        character_ratio: (text_chars_count[1] as f32 / text_chars_count[0] as f32)
            * (texture_dimensions[0] as f32 / texture_dimensions[1] as f32),
    };

    let font = world
        .resources_mut()
        .get_mut::<AssetStorageResource<Font>>()
        .unwrap()
        .insert(font);

    (texture, font)
}

pub fn recreate_depth_texture_resource(mut world: ExclusiveWorldAccess) {
    let render_api = world.resources().get::<RenderApiResource>().unwrap();

    let screen_size = render_api.size();

    let render_state = render_api.raw();

    let mut contains_depth = false;

    if let Some(depth_res) = world.resources().get::<DepthTextureResource>() {
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
        world.resources_mut().insert(depth_res).ok().unwrap();
    }
}

pub fn recreate_transparent_target_resource(mut world: ExclusiveWorldAccess) {
    let render_api = world.resources().get::<RenderApiResource>().unwrap();

    let screen_size = render_api.size();

    let render_state = render_api.raw();

    let mut contains_transparent_target = false;

    if let Some(transparent_target_res) = world.resources().get::<TransparentTargetTextureResource>() {
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
        world.resources_mut().insert(transparent_target_res).ok().unwrap();
    }
}

pub fn create_gizmos_render_resource(mut world: ExclusiveWorldAccess) {
    let render_state = world.resources().get::<RenderApiResource>().unwrap().raw();

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
                format: render_state.surface_config().format,
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

    world
        .resources_mut()
        .insert(GizmosRenderResource {
            vertex_buffer,
            color_buffer,
            transform_buffer,
            bind_group,
            pipeline,
            vertices_cpu_buffer,
            colors_cpu_buffer,
        })
        .ok()
        .unwrap();
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

pub fn update_text_batched_mesh(
    mut q: WorldQuery<(&TextComponent, &mut BatchedMeshComponent, Option<&GlobalRectComponent>)>,
    render_res: Res<RenderApiResource>,
    font_assets: Res<AssetStorageResource<Font>>,
) {
    const VERTICES_PER_CHAR: usize = 4;
    const INDICES_PER_CHAR: usize = 6;

    let window_size = render_res.size();
    let window_size = Vec2::from_array(window_size.map(|v| v as f32));

    let normal = [0.0, 0.0, -1.0];

    for (text_c, mesh_c, rect_c) in q.iter_mut() {
        let color = text_c.color.into_array();
        let font = font_assets.get(&text_c.font).unwrap();

        let rect = rect_c.copied().unwrap_or(GlobalRectComponent {
            center: Vec2::splat(0.0),
            scale: Vec2::splat(0.0),
            z: 0.0,
        });

        let ui_val_to_px_fn = |ui_val: UiVal| ui_val.into_px(rect_c.map(|r| r.scale).unwrap_or(window_size), window_size);

        let font_size = ui_val_to_px_fn(text_c.font_size)[1];
        let horizontal_spacing = ui_val_to_px_fn(text_c.horizontal_spacing)[1];

        let mut quad_scale = Vec2::new(font_size * font.character_ratio, font_size);

        let chars_count = text_c.text.chars().count();

        let mut text_scale = quad_scale;
        text_scale.x *= chars_count as f32;
        text_scale.x += (usize::max(chars_count, 1) - 1) as f32 * horizontal_spacing;

        let center = Vec2::new(
            match text_c.horizontal_align {
                HorizontalAlign::Left => rect.center.x - rect.scale.x * 0.5 + text_scale.x * 0.5,
                HorizontalAlign::Middle => rect.center.x,
                HorizontalAlign::Right => rect.center.x + rect.scale.x * 0.5 - text_scale.x * 0.5,
            },
            match text_c.vertical_align {
                VerticalAlign::Top => rect.center.y - rect.scale.y * 0.5 + text_scale.y * 0.5,
                VerticalAlign::Middle => rect.center.y,
                VerticalAlign::Bottom => rect.center.y + rect.scale.y * 0.5 - text_scale.y * 0.5,
            },
        );

        if text_c.is_y_inverted {
            quad_scale.y *= -1.0;
            text_scale.y *= -1.0;
        }

        let start_pos = center - text_scale * 0.5;

        mesh_c
            .vertices
            .resize((chars_count * VERTICES_PER_CHAR) as u64, StandardVertex::default());
        mesh_c.indices.resize((chars_count * INDICES_PER_CHAR) as u64, 0);

        for (i, character) in text_c.text.chars().enumerate() {
            let char_uvs = font.characters_uv.get(&character).unwrap_or(&font.missing_character_uv);

            let pos = [
                start_pos + Vec2::new((i + 0) as f32, 0.0) * quad_scale + Vec2::X * horizontal_spacing * i as f32,
                start_pos + Vec2::new((i + 1) as f32, 1.0) * quad_scale + Vec2::X * horizontal_spacing * i as f32,
            ];

            mesh_c.vertices[i * VERTICES_PER_CHAR + 0] = StandardVertex {
                color,
                normal,
                uv: [char_uvs[0][0], char_uvs[0][1]],
                position: [pos[0][0], pos[1][1], rect.z],
            };
            mesh_c.vertices[i * VERTICES_PER_CHAR + 1] = StandardVertex {
                color,
                normal,
                uv: [char_uvs[1][0], char_uvs[0][1]],
                position: [pos[1][0], pos[1][1], rect.z],
            };
            mesh_c.vertices[i * VERTICES_PER_CHAR + 2] = StandardVertex {
                color,
                normal,
                uv: [char_uvs[0][0], char_uvs[1][1]],
                position: [pos[0][0], pos[0][1], rect.z],
            };
            mesh_c.vertices[i * VERTICES_PER_CHAR + 3] = StandardVertex {
                color,
                normal,
                uv: [char_uvs[1][0], char_uvs[1][1]],
                position: [pos[1][0], pos[0][1], rect.z],
            };

            mesh_c.indices[i * INDICES_PER_CHAR + 0] = (i * VERTICES_PER_CHAR + 0) as u16;
            mesh_c.indices[i * INDICES_PER_CHAR + 1] = (i * VERTICES_PER_CHAR + 3) as u16;
            mesh_c.indices[i * INDICES_PER_CHAR + 2] = (i * VERTICES_PER_CHAR + 1) as u16;
            mesh_c.indices[i * INDICES_PER_CHAR + 3] = (i * VERTICES_PER_CHAR + 0) as u16;
            mesh_c.indices[i * INDICES_PER_CHAR + 4] = (i * VERTICES_PER_CHAR + 2) as u16;
            mesh_c.indices[i * INDICES_PER_CHAR + 5] = (i * VERTICES_PER_CHAR + 3) as u16;
        }
    }
}

pub fn update_image_batched_mesh(mut q: WorldQuery<(&ImageComponent, &mut BatchedMeshComponent, Option<&GlobalRectComponent>)>) {
    let normal = [0.0, 0.0, -1.0];

    for (image_c, mesh_c, rect_c) in q.iter_mut() {
        let color = image_c.color.into_array();

        let mut rect = rect_c.copied().unwrap_or(GlobalRectComponent {
            center: Vec2::splat(0.5),
            scale: Vec2::splat(1.0),
            z: 0.0,
        });

        if image_c.is_y_inverted {
            rect.scale.y *= -1.0;
        }

        let center = rect.center;

        let pos = [center - rect.scale * 0.5, center, center + rect.scale * 0.5];

        let fill_amt = image_c.fill_amt.clamp(0.0, 1.0);

        let clear_fill_f = |mesh_c: &mut BatchedMeshComponent| {
            mesh_c.vertices.clear();
            mesh_c.indices.clear();
        };

        let standard_fill_f = |mesh_c: &mut BatchedMeshComponent| {
            mesh_c.vertices.resize(4, StandardVertex::default());
            mesh_c.indices.resize(6, 0);

            mesh_c.vertices[0] = StandardVertex {
                color,
                normal,
                uv: [0.0, 0.0],
                position: [pos[0][0], pos[2][1], rect.z],
            };
            mesh_c.vertices[1] = StandardVertex {
                color,
                normal,
                uv: [1.0, 0.0],
                position: [pos[2][0], pos[2][1], rect.z],
            };
            mesh_c.vertices[2] = StandardVertex {
                color,
                normal,
                uv: [0.0, 1.0],
                position: [pos[0][0], pos[0][1], rect.z],
            };
            mesh_c.vertices[3] = StandardVertex {
                color,
                normal,
                uv: [1.0, 1.0],
                position: [pos[2][0], pos[0][1], rect.z],
            };

            mesh_c.indices[0] = 0;
            mesh_c.indices[1] = 3;
            mesh_c.indices[2] = 1;
            mesh_c.indices[3] = 0;
            mesh_c.indices[4] = 2;
            mesh_c.indices[5] = 3;
        };

        match &image_c.fill_settings {
            _ if fill_amt == 1.0 => standard_fill_f(mesh_c),
            _ if fill_amt == 0.0 => clear_fill_f(mesh_c),
            ImageFillSettings::RadialCenter => {
                let uvs = [
                    [0.5, 0.5],
                    [0.5, 1.0],
                    [0.0, 1.0],
                    [0.0, 0.5],
                    [0.0, 0.0],
                    [0.5, 0.0],
                    [1.0, 0.0],
                    [1.0, 0.5],
                    [1.0, 1.0],
                    [0.5, 1.0],
                ];

                let poss = [
                    [pos[1][0], pos[1][1], rect.z],
                    [pos[1][0], pos[2][1], rect.z],
                    [pos[0][0], pos[2][1], rect.z],
                    [pos[0][0], pos[1][1], rect.z],
                    [pos[0][0], pos[0][1], rect.z],
                    [pos[1][0], pos[0][1], rect.z],
                    [pos[2][0], pos[0][1], rect.z],
                    [pos[2][0], pos[1][1], rect.z],
                    [pos[2][0], pos[2][1], rect.z],
                ];

                let fill_amt = image_c.fill_amt.clamp(0.0, 1.0);

                let slices = 1 + ((fill_amt * 8.0).floor() as usize).clamp(0, 7);

                mesh_c.vertices.resize(3 + slices as u64, StandardVertex::default());
                mesh_c.indices.resize(slices as u64 * 3, 0);

                mesh_c.vertices[0] = StandardVertex {
                    color,
                    normal,
                    uv: uvs[0],
                    position: poss[0],
                };
                mesh_c.vertices[1] = StandardVertex {
                    color,
                    normal,
                    uv: uvs[1],
                    position: poss[1],
                };

                for i in 0..slices {
                    if i + 1 == slices {
                        let (x, y) = (fill_amt * 2.0 * f32::consts::PI).sin_cos();

                        let t = Vec2::new(x, -y) / f32::max(x.abs(), y.abs());

                        let last_pos = pos[1].lerp_separately(pos[0], t);

                        mesh_c.vertices[i + 2] = StandardVertex {
                            color,
                            normal,
                            uv: uvs[i + 2],
                            position: [last_pos[0], last_pos[1], rect.z],
                        };
                    } else {
                        mesh_c.vertices[i + 2] = StandardVertex {
                            color,
                            normal,
                            uv: uvs[i + 2],
                            position: poss[i + 2],
                        };
                    }

                    mesh_c.indices[i * 3 + 0] = (i + 1) as u16;
                    mesh_c.indices[i * 3 + 1] = (i + 2) as u16;
                    mesh_c.indices[i * 3 + 2] = 0;
                }
            }
        }
    }
}

pub fn update_masked_batched_mesh(
    hierarchy_q: WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    mask_q: WorldQuery<&GlobalRectComponent, WithFilter<ChildrenRectMaskComponent>>,
    mut mesh_q: WorldQuery<&mut BatchedMeshComponent>,
) {
    let mut masked = HashMap::<Entity, GlobalRectComponent>::new();

    crate::transform::utils::hierarchy_iter_depth_first_parent_to_child(&hierarchy_q, |e, c| {
        let parent_mask = masked.remove(&e);

        for &child in c {
            let child_mask = mask_q.get(child);

            let rect = match (parent_mask, child_mask) {
                (None, None) => continue,
                (Some(m), None) => m,
                (None, Some(&m)) => m,
                (Some(p), Some(c)) => {
                    let min = (p.center - p.scale * 0.5).zip_copied(c.center - c.scale * 0.5, f32::max);
                    let max = (p.center + p.scale * 0.5).zip_copied(c.center + c.scale * 0.5, f32::min);

                    let center = (min + max) * 0.5;
                    let scale = max - min;

                    GlobalRectComponent { center, scale, z: 0.0 }
                }
            };

            masked.insert(child, rect);

            let min = rect.center - rect.scale * 0.5;
            let max = rect.center + rect.scale * 0.5;

            // todo: Use proper masking.
            if let Some(mesh) = mesh_q.get_mut(child) {
                for vertex in &mut mesh.vertices {
                    let mut pos = Vec3::from_array(vertex.position).xy();

                    pos = pos.zip_copied(min, f32::max);
                    pos = pos.zip_copied(max, f32::min);

                    vertex.position = [pos.x, pos.y, vertex.position[2]];
                }
            }
        }
    });
}

pub fn request_surface_texture(render_api: Res<RenderApiResource>, mut surface_texture: ResMut<SurfaceTextureResource>) {
    surface_texture.texture = render_api.raw().surface().get_current_texture().ok();
}

pub fn present_surface(mut surface_texture: ResMut<SurfaceTextureResource>) {
    if let Some(texture) = surface_texture.texture.take() {
        texture.present();
    }
}

pub fn clear_depth(render_api: Res<RenderApiResource>, depth_res: Res<DepthTextureResource>) {
    let render_state = render_api.raw();

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

pub fn clear_transparent_target(render_api: Res<RenderApiResource>, transparent_target_res: Res<TransparentTargetTextureResource>) {
    let render_state = render_api.raw();

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

pub fn render_opaque_instanced(
    query: WorldQuery<(
        Option<&GlobalTransform>,
        &StandardMeshComponent,
        &StandardMaterialComponent,
        Option<&GlobalDisableableComponent>,
    )>,
    render_api: Res<RenderApiResource>,
    screen_space_res: Res<ScreenSpaceResource>,
    standard_render_res: Res<StandardRenderResource>,
    standard_render_assets_res: Res<StandardRenderAssetsResource>,
    depth_res: Res<DepthTextureResource>,
    surface_texture: Res<SurfaceTextureResource>,
    meshes: Res<AssetStorageResource<StandardMesh>>,
    materials: Res<AssetStorageResource<StandardMaterial>>,
    textures: Res<AssetStorageResource<StandardTexture>>,
) {
    if query.len() == 0 {
        return;
    }

    let render_state = render_api.raw();

    let Some(surface_texture) = &surface_texture.texture else {
        return;
    };

    let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());

    let window_size = render_state.size();

    let window_to_clip_mat = utils::create_window_to_clip_matrix(
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

        if material.alpha_threshold.is_none() {
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

        let mesh = mesh.native();

        let (render_pipeline, bind_group, bind_group_tex) = utils::get_render_data(
            material,
            &standard_render_res,
            render_state,
            &textures,
            &standard_render_assets_res,
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
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_bind_group(1, bind_group_tex, &[]);
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
    standard_render_assets_res: Res<StandardRenderAssetsResource>,
    depth_res: Res<DepthTextureResource>,
    surface_texture: Res<SurfaceTextureResource>,
    materials: Res<AssetStorageResource<StandardMaterial>>,
    textures: Res<AssetStorageResource<StandardTexture>>,
    mut batched_vertex_cpu_buffer: ResMut<BatchedVertexCpuBufferResource>,
) {
    if query.len() == 0 {
        return;
    }

    let render_state = render_api.raw();

    let Some(surface_texture) = &surface_texture.texture else {
        return;
    };

    let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());

    let window_size = render_state.size();

    let window_to_clip_mat = utils::create_window_to_clip_matrix(
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

        if material.alpha_threshold.is_none() {
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

    for (material, matrices_and_meshes) in batched_meshes_by_material {
        let Some(material) = materials.get(&material) else {
            continue;
        };

        let (render_pipeline, bind_group, bind_group_tex) = utils::get_render_data(
            material,
            &standard_render_res,
            render_state,
            &textures,
            &standard_render_assets_res,
            window_to_clip_mat,
        );

        let mut batch_buffer_i = 0;

        for (mat, batched_mesh) in matrices_and_meshes {
            for &i in &batched_mesh.indices {
                let mut vertex = batched_mesh.vertices[i as usize];

                vertex.position = mat.mul_with_projection(Vec3::from_array(vertex.position)).into_array();
                vertex.normal = mat.mul_with_projection_as_dir(Vec3::from_array(vertex.normal)).into_array();
                vertex.color = vertex.color.map(|x| x.powf(2.2));

                batch_cpu_buffer[batch_buffer_i] = vertex;

                batch_buffer_i += 1;

                if batch_buffer_i < batch_cpu_buffer.len() {
                    continue;
                }

                batch_buffer_i = 0;

                submit_render(
                    &render_state,
                    &standard_render_res,
                    &batch_cpu_buffer[..],
                    &view,
                    &depth_res,
                    render_pipeline,
                    bind_group,
                    bind_group_tex,
                );
            }
        }

        if batch_buffer_i > 0 {
            submit_render(
                &render_state,
                &standard_render_res,
                &batch_cpu_buffer[..batch_buffer_i],
                &view,
                &depth_res,
                render_pipeline,
                bind_group,
                bind_group_tex,
            );
        }

        fn submit_render(
            render_state: &RenderState,
            standard_render_res: &StandardRenderResource,
            batched_cpu_slice: &[StandardVertex],
            render_target_view: &TextureView,
            depth_res: &DepthTextureResource,
            render_pipeline: &RenderPipeline,
            bind_group: &BindGroup,
            bind_group_tex: &BindGroup,
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
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_bind_group(1, bind_group_tex, &[]);
                render_pass.set_vertex_buffer(1, standard_render_res.instance_buffer.slice(..));
                render_pass.set_vertex_buffer(0, standard_render_res.batched_vertex_buffer.slice(..));
                render_pass.draw(0..(batched_cpu_slice.len() as u32), 0..1);
            }

            render_state.queue().submit(std::iter::once(encoder.finish()));
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
    standard_render_assets_res: Res<StandardRenderAssetsResource>,
    depth_res: Res<DepthTextureResource>,
    transparent_target_res: Res<TransparentTargetTextureResource>,
    meshes: Res<AssetStorageResource<StandardMesh>>,
    materials: Res<AssetStorageResource<StandardMaterial>>,
    textures: Res<AssetStorageResource<StandardTexture>>,
) {
    if query.len() == 0 {
        return;
    }

    let render_state = render_api.raw();

    let view = &transparent_target_res.texture_view;

    let window_size = render_state.size();

    let window_to_clip_mat = utils::create_window_to_clip_matrix(
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

        if material.alpha_threshold.is_some() {
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

        let mesh = mesh.native();

        let (render_pipeline, bind_group, bind_group_tex) = utils::get_render_data(
            material,
            &standard_render_res,
            render_state,
            &textures,
            &standard_render_assets_res,
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
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_bind_group(1, bind_group_tex, &[]);
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
    standard_render_assets_res: Res<StandardRenderAssetsResource>,
    depth_res: Res<DepthTextureResource>,
    transparent_target_res: Res<TransparentTargetTextureResource>,
    materials: Res<AssetStorageResource<StandardMaterial>>,
    textures: Res<AssetStorageResource<StandardTexture>>,
    mut batched_vertex_cpu_buffer: ResMut<BatchedVertexCpuBufferResource>,
) {
    if query.len() == 0 {
        return;
    }

    let render_state = render_api.raw();

    let view = &transparent_target_res.texture_view;

    let window_size = render_state.size();

    let window_to_clip_mat = utils::create_window_to_clip_matrix(
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

        if material.alpha_threshold.is_some() {
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

    for (material, matrices_and_meshes) in batched_meshes_by_material {
        let Some(material) = materials.get(&material) else {
            continue;
        };

        let (render_pipeline, bind_group, bind_group_tex) = utils::get_render_data(
            material,
            &standard_render_res,
            render_state,
            &textures,
            &standard_render_assets_res,
            window_to_clip_mat,
        );

        let mut batch_buffer_i = 0;

        for (mat, batched_mesh) in matrices_and_meshes {
            for &i in &batched_mesh.indices {
                let mut vertex = batched_mesh.vertices[i as usize];

                vertex.position = mat.mul_with_projection(Vec3::from_array(vertex.position)).into_array();
                vertex.normal = mat.mul_with_projection_as_dir(Vec3::from_array(vertex.normal)).into_array();

                batch_cpu_buffer[batch_buffer_i] = vertex;

                batch_buffer_i += 1;

                if batch_buffer_i < batch_cpu_buffer.len() {
                    continue;
                }

                batch_buffer_i = 0;

                submit_render(
                    &render_state,
                    &standard_render_res,
                    &batch_cpu_buffer[..],
                    &view,
                    &depth_res,
                    render_pipeline,
                    bind_group,
                    bind_group_tex,
                );
            }
        }

        if batch_buffer_i > 0 {
            submit_render(
                &render_state,
                &standard_render_res,
                &batch_cpu_buffer[..batch_buffer_i],
                &view,
                &depth_res,
                render_pipeline,
                bind_group,
                bind_group_tex,
            );
        }

        fn submit_render(
            render_state: &RenderState,
            standard_render_res: &StandardRenderResource,
            batched_cpu_slice: &[StandardVertex],
            render_target_view: &TextureView,
            depth_res: &DepthTextureResource,
            render_pipeline: &RenderPipeline,
            bind_group: &BindGroup,
            bind_group_tex: &BindGroup,
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
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_bind_group(1, bind_group_tex, &[]);
                render_pass.set_vertex_buffer(1, standard_render_res.instance_buffer.slice(..));
                render_pass.set_vertex_buffer(0, standard_render_res.batched_vertex_buffer.slice(..));
                render_pass.draw(0..(batched_cpu_slice.len() as u32), 0..1);
            }

            render_state.queue().submit(std::iter::once(encoder.finish()));
        }
    }
}

pub fn render_transparent_final(
    render_api: Res<RenderApiResource>,
    standard_render_res: Res<StandardRenderResource>,
    transparent_target_res: Res<TransparentTargetTextureResource>,
    surface_texture: Res<SurfaceTextureResource>,
) {
    let render_state = render_api.raw();

    let Some(surface_texture) = &surface_texture.texture else {
        return;
    };

    let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());

    let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Transparent Final Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Transparent Final Render Pass"),
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

        render_pass.set_pipeline(&standard_render_res.render_pipeline_transparent_final);
        render_pass.set_bind_group(0, &transparent_target_res.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    render_state.queue().submit(std::iter::once(encoder.finish()));
}

pub fn render_gizmos(
    mut gizmos: ResMut<GizmosResource>,
    mut gizmos_render_res: ResMut<GizmosRenderResource>,
    surface_texture: Res<SurfaceTextureResource>,
    screen_space_res: Res<ScreenSpaceResource>,
    render_api: Res<RenderApiResource>,
    camera_query: WorldQuery<(&GlobalTransform, &CameraComponent)>,
) {
    let Some(surface_texture) = &surface_texture.texture else {
        return;
    };

    let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());

    let render_state = render_api.raw();

    let window_size = render_state.size();

    for (space, lines) in gizmos.spaces() {
        if lines.len() == 0 {
            continue;
        }

        let transform = match space {
            RenderSpace::Clip => Mat4::<f32>::IDENTITY,
            RenderSpace::Window => utils::create_window_to_clip_matrix(
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

use std::collections::HashMap;

use fruits_app::RenderStateResource;
use fruits_ecs::{ExclusiveWorldAccess, Res, ResMut, WorldData, WorldQuery};
use fruits_math::{Mat4, Vec2, Vec3, Vec4};
use image::GenericImageView;
use wgpu::{include_wgsl, util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, CommandEncoderDescriptor, DepthStencilState, Extent3d, FilterMode, FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor, ShaderStages, StoreOp, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexState};

use crate::{asset::{AssetHandle, AssetStorageResource}, render::{utils::{self, BATCHED_MESH_MATERIAL_TRIANGLES_PER_DRAW_MAX, GIZMO_LINES_PER_DRAW_MAX, STANDARD_MESH_MATERIAL_INSTANCES_PER_DRAW_MAX}, BatchedMeshComponent, BatchedVertexCpuBufferResource, Font, HorizontalAlign, ImageComponent, LitUniform, MaterialStandardRenderResourceData, ScreenSpaceResource, StandardInstance, StandardRenderAssetsResource, StandardTexture, StandardVertex, TextComponent, UnlitUniform, VerticalAlign}, transform::{GlobalRectComponent, GlobalTransform}};

use super::{assets::{StandardMaterial, StandardMesh}, components::{CameraComponent, StandardMaterialComponent, StandardMeshComponent}, resources::SurfaceTextureResource, DepthTextureResource, RenderSpace, GizmosRenderResource, GizmosResource, StandardRenderResource};

pub fn create_standard_render_resource(
    mut world: ExclusiveWorldAccess,
) {
    let render_state = world.resources().get::<RenderStateResource>().unwrap();
    let depth_tex = world.resources().get::<DepthTextureResource>().unwrap();

    let bind_group_layout = render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Standard Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]
    });

    let bind_group_layout_standard_texture = render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Standard Texture Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
            },
            BindGroupLayoutEntry {
                binding: 1,
                count: None,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
            },
        ],
    });

    let pipeline_layout = render_state.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Standard Pipeline Layout"),
        bind_group_layouts: &[
            &bind_group_layout,
            &bind_group_layout_standard_texture,
        ],
        push_constant_ranges: &[],
    });

    let lit_uniform_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Lit Uniform Buffer"),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(&[LitUniform::default()]),
    });
    
    let lit_bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
        label: Some("Lit Uniform Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: lit_uniform_buffer.as_entire_binding(),
            },
        ],
    });
    
    let lit_shader = render_state.device().create_shader_module(include_wgsl!("./assets/shader_lit.wgsl"));

    let lit_render_pipeline = render_state.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Lit Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &lit_shader,
            entry_point: Some("vs_main"),
            buffers: &[
                StandardVertex::desc(),
                StandardInstance::desc(),
            ],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &lit_shader,
            entry_point: Some("fs_main"),
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

    let lit_data = MaterialStandardRenderResourceData {
        buffer_uniform: lit_uniform_buffer,
        render_pipeline: lit_render_pipeline,
        bind_group_uniform: lit_bind_group,
    };

    let unlit_uniform_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Unlit Uniform Buffer"),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(&[UnlitUniform::default()]),
    });
    
    let unlit_bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
        label: Some("Unlit Uniform Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: unlit_uniform_buffer.as_entire_binding(),
            },
        ],
    });
    
    let unlit_shader = render_state.device().create_shader_module(include_wgsl!("./assets/shader_unlit.wgsl"));

    let unlit_render_pipeline = render_state.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Unlit Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &unlit_shader,
            entry_point: Some("vs_main"),
            buffers: &[
                StandardVertex::desc(),
                StandardInstance::desc(),
            ],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &unlit_shader,
            entry_point: Some("fs_main"),
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

    let unlit_data = MaterialStandardRenderResourceData {
        buffer_uniform: unlit_uniform_buffer,
        render_pipeline: unlit_render_pipeline,
        bind_group_uniform: unlit_bind_group,
    };

    let instance_cpu_buffer = vec![Mat4::<f32>::IDENTITY.into_array(); STANDARD_MESH_MATERIAL_INSTANCES_PER_DRAW_MAX].into_boxed_slice();
    
    let instance_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Instance Buffer"),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes_slice(&instance_cpu_buffer),
    });

    let batched_vertex_cpu_buffer = vec![StandardVertex::default(); BATCHED_MESH_MATERIAL_TRIANGLES_PER_DRAW_MAX * 3].into_boxed_slice();

    let batched_vertex_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Batch vertex buffer"),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes_slice(&batched_vertex_cpu_buffer)
    });

    world.resources_mut().insert(StandardRenderResource {
        bind_group_layout_standard_texture,
        pipeline_layout,
        instance_cpu_buffer,
        instance_buffer,
        batched_vertex_buffer,
        lit: lit_data,
        unlit: unlit_data,
        camera_pos: Vec3::default(),
        camera_proj_matrix: Mat4::IDENTITY,
    }).ok().unwrap();

    world.resources_mut().insert(BatchedVertexCpuBufferResource(batched_vertex_cpu_buffer)).ok().unwrap();

    let texture_white = StandardTexture::from_world(
        &world,
        FilterMode::Linear,
        [2, 2],
        &[255; 16]
    );

    let texture_white = world.resources_mut().get_mut::<AssetStorageResource<StandardTexture>>().unwrap().insert(texture_white);

    let (texture_text_px_5_7, font_px_5_7) = create_ascii_monospace_font(&mut world, include_bytes!("./assets/ascii_px_5x7.png"));
    let (texture_text_px_8_8, font_px_8_8) = create_ascii_monospace_font(&mut world, include_bytes!("./assets/ascii_px_8x8.png"));
    let (texture_text_px_8_12, font_px_8_12) = create_ascii_monospace_font(&mut world, include_bytes!("./assets/ascii_px_8x12.png"));

    world.resources_mut().insert(StandardRenderAssetsResource {
        texture_white,
        texture_text_px_5_7,
        font_px_5_7,
        texture_text_px_8_8,
        font_px_8_8,
        texture_text_px_8_12,
        font_px_8_12,
    }).ok().unwrap();
}

fn create_ascii_monospace_font(
    world: &mut WorldData,
    texture_bytes: &[u8],
) -> (AssetHandle<StandardTexture>, AssetHandle<Font>) {
    let image = image::load_from_memory(texture_bytes).unwrap();

    let texture_dimensions: [u32; 2] = image.dimensions().into();

    let texture = StandardTexture::from_world(
        world,
        FilterMode::Nearest,
        texture_dimensions,
        image.as_bytes(),
    );

    let text_chars_count = [16, 8];
    let single_char_uv_size = [1.0 / text_chars_count[0] as f32, 1.0 / text_chars_count[1] as f32];

    let characters_uv = (' '..='~').map(|c| {
        let char_uv_index = [c as i32 % text_chars_count[0], c as i32 / text_chars_count[0]];
        let char_uv_min = fruits_math::zip(&char_uv_index, &text_chars_count, |a, b| a as f32 / b as f32);
        
        let char_uvs = [
            Vec2::from_array(char_uv_min),
            Vec2::from_array(fruits_math::zip(&char_uv_min, &single_char_uv_size, |a, b| a + b)),
        ];

        (c, char_uvs)
    }).collect::<HashMap<_, _>>();
    
    let texture = world.resources_mut().get_mut::<AssetStorageResource<StandardTexture>>().unwrap().insert(texture);
    
    let font = Font {
        texture: texture.clone(),
        mising_character_uv: characters_uv[&'?'],
        characters_uv: characters_uv,
        character_ratio: (text_chars_count[1] as f32 / text_chars_count[0] as f32) * (texture_dimensions[0] as f32 / texture_dimensions[1] as f32),
    };

    let font = world.resources_mut().get_mut::<AssetStorageResource<Font>>().unwrap().insert(font);

    (texture, font)
}

pub fn recreate_depth_texture_resource(
    mut world: ExclusiveWorldAccess,
) {
    let render_state = world.resources().get::<RenderStateResource>().unwrap();
    let surface_config = render_state.surface_config();

    let mut contains_depth = false;

    if let Some(depth_res) = world.resources().get::<DepthTextureResource>() {
        contains_depth = true;

        let are_same_size = {
            depth_res.texture.size().width == surface_config.width
            && depth_res.texture.size().height == surface_config.height
        };

        if are_same_size {
            return;
        }
    }

    let texture = render_state.device().create_texture(&TextureDescriptor {
        label: Some("Depth Buffer"),
        size: Extent3d {
            width: surface_config.width,
            height: surface_config.height,
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

pub fn create_gizmos_render_resource(
    mut world: ExclusiveWorldAccess,
) {
    let render_state = world.resources().get::<RenderStateResource>().unwrap();

    let vertices_cpu_buffer = vec![[Vec4::with_all(0.0); 2]; GIZMO_LINES_PER_DRAW_MAX].into_boxed_slice();
    let colors_cpu_buffer = vec![Vec4::with_all(0.0); GIZMO_LINES_PER_DRAW_MAX].into_boxed_slice();

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
        bind_group_layouts: &[
            &bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let shader = render_state.device().create_shader_module(include_wgsl!("./assets/shader_gizmo.wgsl"));

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
        }
    });

    world.resources_mut().insert(GizmosRenderResource {
        vertex_buffer,
        color_buffer,
        transform_buffer,
        bind_group,
        pipeline,
        vertices_cpu_buffer,
        colors_cpu_buffer,
    }).ok().unwrap();
}

pub fn update_camera_uniform(
    render_state: Res<RenderStateResource>,
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

    let aspect = window_size.width as f32 / window_size.height as f32;

    let projection_matrix = fruits_math::perspective_proj_matrix(camera.fov, camera.near, camera.far, aspect);

    let transform_matrix = transform.scale_rotation.into_4x4_with_offset(transform.position).inverse().unwrap();

    standard_render_res.camera_proj_matrix = projection_matrix * transform_matrix;
    standard_render_res.camera_pos = transform.position;
}

pub fn update_text_batched_mesh(
    mut q: WorldQuery<(&TextComponent, &mut BatchedMeshComponent, Option<&GlobalRectComponent>)>,
    render_res: Res<RenderStateResource>,
    font_assets: Res<AssetStorageResource<Font>>,
) {
    const VERTICES_PER_CHAR: usize = 4;
    const INDICES_PER_CHAR: usize = 6;

    let window_size: [u32; 2] = render_res.size().into();
    let window_size = Vec2::from_array(window_size.map(|v| v as f32));

    let normal = [0.0, 0.0, -1.0];

    for (text_c, mesh_c, rect_c) in q.iter_mut() {
        let color = text_c.color.into_array();
        let font = font_assets.get(&text_c.font).unwrap();

        let rect = rect_c.copied().unwrap_or(GlobalRectComponent { center: Vec2::with_all(0.0), scale: Vec2::with_all(0.0), });

        let font_size = text_c.font_size.into_px(rect_c.map(|r| r.scale).unwrap_or(window_size), window_size);
        
        let mut quad_scale = Vec2::new(font_size * font.character_ratio, font_size);

        let chars_count = text_c.text.chars().count();

        let mut text_scale = quad_scale;
        text_scale.x *= chars_count as f32;
        text_scale.x += (usize::max(chars_count, 1) - 1) as f32 * text_c.horizontal_spacing;

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

        mesh_c.vertices.resize(chars_count * VERTICES_PER_CHAR, StandardVertex::default());
        mesh_c.indices.resize(chars_count * INDICES_PER_CHAR, 0);

        for (i, character) in text_c.text.chars().enumerate() {
            let char_uvs = font.characters_uv.get(&character).unwrap_or(&font.mising_character_uv);
            
            let pos = [
                start_pos + Vec2::new((i + 0) as f32, 0.0) * quad_scale + Vec2::X * text_c.horizontal_spacing * i as f32,
                start_pos + Vec2::new((i + 1) as f32, 1.0) * quad_scale + Vec2::X * text_c.horizontal_spacing * i as f32,
            ];

            mesh_c.vertices[i * VERTICES_PER_CHAR + 0] = StandardVertex { color, normal, uv: [char_uvs[0][0], char_uvs[0][1]], position: [pos[0][0], pos[1][1], 0.0] };
            mesh_c.vertices[i * VERTICES_PER_CHAR + 1] = StandardVertex { color, normal, uv: [char_uvs[1][0], char_uvs[0][1]], position: [pos[1][0], pos[1][1], 0.0] };
            mesh_c.vertices[i * VERTICES_PER_CHAR + 2] = StandardVertex { color, normal, uv: [char_uvs[0][0], char_uvs[1][1]], position: [pos[0][0], pos[0][1], 0.0] };
            mesh_c.vertices[i * VERTICES_PER_CHAR + 3] = StandardVertex { color, normal, uv: [char_uvs[1][0], char_uvs[1][1]], position: [pos[1][0], pos[0][1], 0.0] };
            
            mesh_c.indices[i * INDICES_PER_CHAR + 0] = (i * VERTICES_PER_CHAR + 0) as u16;
            mesh_c.indices[i * INDICES_PER_CHAR + 1] = (i * VERTICES_PER_CHAR + 3) as u16;
            mesh_c.indices[i * INDICES_PER_CHAR + 2] = (i * VERTICES_PER_CHAR + 1) as u16;
            mesh_c.indices[i * INDICES_PER_CHAR + 3] = (i * VERTICES_PER_CHAR + 0) as u16;
            mesh_c.indices[i * INDICES_PER_CHAR + 4] = (i * VERTICES_PER_CHAR + 2) as u16;
            mesh_c.indices[i * INDICES_PER_CHAR + 5] = (i * VERTICES_PER_CHAR + 3) as u16;
        }
    }
}

pub fn update_image_batched_mesh(
    mut q: WorldQuery<(&ImageComponent, &mut BatchedMeshComponent, Option<&GlobalRectComponent>)>,
) {
    let normal = [0.0, 0.0, -1.0];

    for (image_c, mesh_c, rect_c) in q.iter_mut() {
        let color = image_c.color.into_array();

        mesh_c.vertices.resize(4, StandardVertex::default());
        mesh_c.indices.resize(6, 0);

        let mut rect = rect_c.copied().unwrap_or(GlobalRectComponent { center: Vec2::with_all(0.5), scale: Vec2::with_all(1.0), });

        if image_c.is_y_inverted {
            rect.scale.y *= -1.0;
        }

        let pos = [
            rect.center - rect.scale * 0.5,
            rect.center + rect.scale * 0.5,
        ];

        mesh_c.vertices[0] = StandardVertex { color, normal, uv: [0.0, 0.0], position: [pos[0][0], pos[1][1], 0.0] };
        mesh_c.vertices[1] = StandardVertex { color, normal, uv: [1.0, 0.0], position: [pos[1][0], pos[1][1], 0.0] };
        mesh_c.vertices[2] = StandardVertex { color, normal, uv: [0.0, 1.0], position: [pos[0][0], pos[0][1], 0.0] };
        mesh_c.vertices[3] = StandardVertex { color, normal, uv: [1.0, 1.0], position: [pos[1][0], pos[0][1], 0.0] };
        
        mesh_c.indices[0] = 0;
        mesh_c.indices[1] = 3;
        mesh_c.indices[2] = 1;
        mesh_c.indices[3] = 0;
        mesh_c.indices[4] = 2;
        mesh_c.indices[5] = 3;
    }
}

pub fn request_surface_texture(
    render_state: Res<RenderStateResource>,
    mut surface_texture: ResMut<SurfaceTextureResource>,
) {
    surface_texture.texture = render_state.surface().get_current_texture().ok();
}

pub fn present_surface(mut surface_texture: ResMut<SurfaceTextureResource>) {
    if let Some(texture) = surface_texture.texture.take() {
        texture.present();
    }
}

pub fn clear_depth(
    render_state: Res<RenderStateResource>,
    depth_res: Res<DepthTextureResource>,
) {
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

pub fn render_meshes_and_materials_instanced(
    query: WorldQuery<(&GlobalTransform, &StandardMeshComponent, &StandardMaterialComponent)>,
    render_state: Res<RenderStateResource>,
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

    let Some(surface_texture) = &surface_texture.texture else { return; }; 

    let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());

    let window_size = render_state.size();

    let window_to_clip_mat = utils::create_window_to_clip_matrix(
        window_size.width as f32,
        window_size.height as f32,
        screen_space_res.near,
        screen_space_res.far,
    );

    let mut instanced_matrices = HashMap::new();

    for (transform, render_mesh, render_material) in query.iter() {
        instanced_matrices.entry((render_mesh.mesh.clone(), render_material.material.clone()))
            .or_insert_with(|| Vec::new())
            .push(transform.scale_rotation.into_4x4_with_offset(transform.position));
    }

    for ((mesh, material), matrices) in instanced_matrices.iter() {
        let Some(mesh) = meshes.get(&mesh) else { continue; };
        let Some(material) = materials.get(&material) else { continue; };

        let (render_pipeline, bind_group, bind_group_tex) = match material {
            StandardMaterial::Lit(material) => {
                let lit_data = &standard_render_res.lit;
                
                let world_to_clip = match material.space {
                    RenderSpace::Clip => Mat4::IDENTITY,
                    RenderSpace::Window => window_to_clip_mat,
                    RenderSpace::World => standard_render_res.camera_proj_matrix,
                };

                let uniform = LitUniform {
                    albedo_color: material.albedo_color,
                    metallic: material.metallic,
                    emission_color: material.emission_color,
                    roughness: material.roughness,
                    alpha_threshold: material.alpha_threshold,
                    camera_position_world: standard_render_res.camera_pos,
                    world_to_clip,
                    _padding: Default::default(),
                };

                render_state.queue().write_buffer(&lit_data.buffer_uniform, 0, fruits_utils::mem::as_bytes(&[uniform]));

                let bind_group_tex = match &material.albedo_tex {
                    Some(albedo_tex) => textures.get(albedo_tex).unwrap().bind_group(),
                    None => textures.get(&standard_render_assets_res.texture_white).unwrap().bind_group(),
                };

                (&lit_data.render_pipeline, &lit_data.bind_group_uniform, bind_group_tex)
            },
            StandardMaterial::Unlit(material) => {
                let unlit_data = &standard_render_res.unlit;

                let world_to_clip = match material.space {
                    RenderSpace::Clip => Mat4::IDENTITY,
                    RenderSpace::Window => window_to_clip_mat,
                    RenderSpace::World => standard_render_res.camera_proj_matrix,
                };

                let uniform = UnlitUniform {
                    world_to_clip,
                    color: material.color,
                    alpha_threshold: material.alpha_threshold,
                    _padding: Default::default(),
                };

                render_state.queue().write_buffer(&unlit_data.buffer_uniform, 0, fruits_utils::mem::as_bytes(&[uniform]));

                let bind_group_tex = match &material.color_tex {
                    Some(color_tex) => textures.get(color_tex).unwrap().bind_group(),
                    None => textures.get(&standard_render_assets_res.texture_white).unwrap().bind_group(),
                };

                (&unlit_data.render_pipeline, &unlit_data.bind_group_uniform, bind_group_tex)
            },
        };
        
        for matrices in matrices.chunks(STANDARD_MESH_MATERIAL_INSTANCES_PER_DRAW_MAX) {
            let matrices_bytes = fruits_utils::mem::as_bytes_slice(matrices);
            
            render_state.queue().write_buffer(&standard_render_res.instance_buffer, 0, matrices_bytes);
            render_state.queue().submit([]);
            
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
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                render_pass.set_vertex_buffer(1, standard_render_res.instance_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..(mesh.indices_count() as u32), 0, 0..(matrices.len() as u32));
            }

            render_state.queue().submit(std::iter::once(encoder.finish()));
        }
    }
}

pub fn render_meshes_and_materials_batched(
    query: WorldQuery<(&GlobalTransform, &BatchedMeshComponent, &StandardMaterialComponent)>,
    render_state: Res<RenderStateResource>,
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

    let Some(surface_texture) = &surface_texture.texture else { return; }; 

    let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());

    let window_size = render_state.size();

    let window_to_clip_mat = utils::create_window_to_clip_matrix(
        window_size.width as f32,
        window_size.height as f32,
        screen_space_res.near,
        screen_space_res.far,
    );

    let mut batched_meshes_by_material = HashMap::new();

    for (transform, batched_mesh, render_material) in query.iter() {
        batched_meshes_by_material.entry(render_material.material.clone())
            .or_insert_with(|| Vec::new())
            .push((transform.scale_rotation.into_4x4_with_offset(transform.position), batched_mesh));
    }

    render_state.queue().write_buffer(&standard_render_res.instance_buffer, 0, fruits_utils::mem::as_bytes(&Mat4::<f32>::IDENTITY));
    render_state.queue().submit([]);

    let batch_cpu_buffer = &mut batched_vertex_cpu_buffer.0;
    
    for (material, matrices_and_meshes) in batched_meshes_by_material {
        let Some(material) = materials.get(&material) else { continue; };
        
        let (render_pipeline, bind_group, bind_group_tex) = match material {
            StandardMaterial::Lit(material) => {
                let lit_data = &standard_render_res.lit;
                
                let world_to_clip = match material.space {
                    RenderSpace::Clip => Mat4::IDENTITY,
                    RenderSpace::Window => window_to_clip_mat,
                    RenderSpace::World => standard_render_res.camera_proj_matrix,
                };

                let uniform = LitUniform {
                    albedo_color: material.albedo_color,
                    metallic: material.metallic,
                    emission_color: material.emission_color,
                    roughness: material.roughness,
                    alpha_threshold: material.alpha_threshold,
                    camera_position_world: standard_render_res.camera_pos,
                    world_to_clip,
                    _padding: Default::default(),
                };

                render_state.queue().write_buffer(&lit_data.buffer_uniform, 0, fruits_utils::mem::as_bytes(&[uniform]));

                let bind_group_tex = match &material.albedo_tex {
                    Some(albedo_tex) => textures.get(albedo_tex).unwrap().bind_group(),
                    None => textures.get(&standard_render_assets_res.texture_white).unwrap().bind_group(),
                };

                (&lit_data.render_pipeline, &lit_data.bind_group_uniform, bind_group_tex)
            },
            StandardMaterial::Unlit(material) => {
                let unlit_data = &standard_render_res.unlit;

                let world_to_clip = match material.space {
                    RenderSpace::Clip => Mat4::IDENTITY,
                    RenderSpace::Window => window_to_clip_mat,
                    RenderSpace::World => standard_render_res.camera_proj_matrix,
                };

                let uniform = UnlitUniform {
                    world_to_clip,
                    color: material.color,
                    alpha_threshold: material.alpha_threshold,
                    _padding: Default::default(),
                };

                render_state.queue().write_buffer(&unlit_data.buffer_uniform, 0, fruits_utils::mem::as_bytes(&[uniform]));

                let bind_group_tex = match &material.color_tex {
                    Some(color_tex) => textures.get(color_tex).unwrap().bind_group(),
                    None => textures.get(&standard_render_assets_res.texture_white).unwrap().bind_group(),
                };

                (&unlit_data.render_pipeline, &unlit_data.bind_group_uniform, bind_group_tex)
            },
        };
        
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
            render_state: &RenderStateResource,
            standard_render_res: &StandardRenderResource,
            batched_cpu_slice: &[StandardVertex],
            render_target_view: &TextureView, 
            depth_res: &DepthTextureResource,
            render_pipeline: &RenderPipeline,
            bind_group: &BindGroup,
            bind_group_tex: &BindGroup,
        ) {
            render_state.queue().write_buffer(&standard_render_res.batched_vertex_buffer, 0, fruits_utils::mem::as_bytes_slice(batched_cpu_slice));

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
                render_pass.set_vertex_buffer(0, standard_render_res.batched_vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, standard_render_res.instance_buffer.slice(..));
                render_pass.draw(0..(batched_cpu_slice.len() as u32), 0..1);
            }

            render_state.queue().submit(std::iter::once(encoder.finish()));
        }
    }
}

pub fn render_gizmos(
    mut gizmos: ResMut<GizmosResource>,
    mut gizmos_render_res: ResMut<GizmosRenderResource>,
    surface_texture: Res<SurfaceTextureResource>,
    screen_space_res: Res<ScreenSpaceResource>,
    render_state: Res<RenderStateResource>,
    camera_query: WorldQuery<(&GlobalTransform, &CameraComponent)>,
) {
    let Some(surface_texture) = &surface_texture.texture else { return; }; 

    let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());

    let render_state = &*render_state;

    let window_size = render_state.size();

    for (space, lines) in gizmos.spaces() {
        if lines.len() == 0 {
            continue;
        }

        let transform = match space {
            RenderSpace::Clip => Mat4::<f32>::IDENTITY,
            RenderSpace::Window => {
                utils::create_window_to_clip_matrix(window_size.width as f32, window_size.height as f32, screen_space_res.near, screen_space_res.far)
            },
            RenderSpace::World => {
                let Some((transform, camera)) = camera_query.iter().next() else {
                    continue;
                };

                let aspect = window_size.width as f32 / window_size.height as f32;

                let projection_matrix = fruits_math::perspective_proj_matrix(camera.fov, camera.near, camera.far, aspect);

                let transform_matrix = transform.scale_rotation.into_4x4_with_offset(transform.position).inverse().unwrap();

                projection_matrix * transform_matrix
            },
        };

        render_state.queue().write_buffer(&gizmos_render_res.transform_buffer, 0, fruits_utils::mem::as_bytes(transform.as_array()));

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

            render_state.queue().write_buffer(&gizmos_render_res.vertex_buffer, 0, fruits_utils::mem::as_bytes_slice(&gizmos_render_res.vertices_cpu_buffer[..count]));
            render_state.queue().write_buffer(&gizmos_render_res.color_buffer, 0, fruits_utils::mem::as_bytes_slice(&gizmos_render_res.colors_cpu_buffer[..count]));

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
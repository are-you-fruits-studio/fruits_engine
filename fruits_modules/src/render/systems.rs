use fruits_app::RenderStateResource;
use fruits_ecs::{ExclusiveWorldAccess, Res, ResMut, WorldQuery};
use fruits_math::{Mat3, Mat4, Vec3, Vec4};
use wgpu::{include_wgsl, util::{BufferInitDescriptor, DeviceExt}, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, CommandEncoderDescriptor, Extent3d, FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, SamplerDescriptor, ShaderStages, StoreOp, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState};

use crate::{asset::AssetStorageResource, transform::GlobalTransform};

use super::{assets::{StandardMaterial, StandardMesh}, components::{CameraComponent, StandardMaterialComponent, StandardMeshComponent}, resources::SurfaceTextureResource, DepthTextureResource, GizmoSpace, GizmosRenderResource, GizmosResource, StandardRenderResource, StandardGlobalUniform};

pub fn create_standard_render_resource(
    mut world: ExclusiveWorldAccess,
) {
    let render_state = world.resources().get::<RenderStateResource>().unwrap();

    let global_uniform_bind_group_layout = render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Standard Global Uniform Bind Group Layout"),
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
            }
        ]
    });
    
    let material_uniform_bind_group_layout = render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Standard Material Uniform Bind Group Layout"),
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
            }
        ]
    });

    let uniform_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Standard Global Uniform Buffer"),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(&[StandardGlobalUniform::default()]),
    });
    
    let uniform_bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
        label: Some("Standard Global Uniform Bind Group"),
        layout: &global_uniform_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    });
    
    let instance_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Instance Buffer"),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(Mat4::<f32>::IDENTITY.as_array()),
    });

    let pipeline_layout = render_state.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Standard Material Pipeline Layout"),
        bind_group_layouts: &[
            &global_uniform_bind_group_layout,
            &material_uniform_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });
    
    let shader = render_state.device().create_shader_module(include_wgsl!("./assets/standard_shader.wgsl"));

    world.resources_mut().insert(StandardRenderResource {
        shader,
        pipeline_layout,
        material_uniform_bind_group_layout,
        uniform: StandardGlobalUniform::default(),
        uniform_buffer,
        uniform_bind_group,
        instance_buffer,
    }).ok().unwrap();
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
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
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

    let index_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Gizmos Index Buffer"),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(&[0_u32; 2]),
    });

    let vertex_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Gizmos Vertex Buffer"),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(&[Vec4::<f32>::with_all(0.0); 2]),
    });

    let color_buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Gizmos Color Buffer"),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(Vec4::<f32>::with_all(0.0).as_array()),
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

    let shader = render_state.device().create_shader_module(include_wgsl!("./assets/gizmo_shader.wgsl"));

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
            entry_point: "fragment_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: render_state.surface_config().format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        vertex: VertexState {
            module: &shader,
            entry_point: "vertex_main",
            buffers: &[
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<u32>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Uint32,
                            offset: 0,
                            shader_location: 0,
                        }
                    ],
                }
            ],
            compilation_options: Default::default(),
        }
    });

    world.resources_mut().insert(GizmosRenderResource {
        index_buffer,
        vertex_buffer,
        color_buffer,
        transform_buffer,
        bind_group,
        pipeline,
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

    standard_render_res.uniform.world_to_clip = projection_matrix * transform_matrix;
    standard_render_res.uniform.camera_position_world = transform.position;
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

pub fn render_meshes_and_materials(
    query: WorldQuery<(&GlobalTransform, &StandardMeshComponent, &StandardMaterialComponent)>,
    render_state: Res<RenderStateResource>,
    standard_render_res: Res<StandardRenderResource>,
    depth_res: Res<DepthTextureResource>,
    surface_texture: Res<SurfaceTextureResource>,
    meshes: Res<AssetStorageResource<StandardMesh>>,
    mut materials: ResMut<AssetStorageResource<StandardMaterial>>,
) {
    if query.len() == 0 {
        return;
    }

    let Some(surface_texture) = &surface_texture.texture else { return; }; 

    let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());

    render_state.queue().write_buffer(&standard_render_res.uniform_buffer, 0, fruits_utils::mem::as_bytes(&[standard_render_res.uniform]));
    
    for (transform, render_mesh, render_material) in query.iter() {
        let Some(mesh) = meshes.get(&render_mesh.mesh) else { continue; };
        let Some(material) = materials.get_mut(&render_material.material) else { continue; };

        // todo: temp
        material.uniform_mut().albedo_color = Vec4::new(1.0, 1.0, 1.0, 1.0);
        material.uniform_mut().metallic = 0.0;
        material.uniform_mut().roughness = 0.2;

        render_state.queue().write_buffer(material.uniform_buffer(), 0, fruits_utils::mem::as_bytes(&[*material.uniform()]));
        
        let transform_matrix = transform.scale_rotation.into_4x4_with_offset(transform.position);
        let transform_matrix = transform_matrix.into_array();
        let transform_matrix = fruits_utils::mem::as_bytes(&transform_matrix);

        render_state.queue().write_buffer(&standard_render_res.instance_buffer, 0, transform_matrix);
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
    
            render_pass.set_pipeline(material.render_pipeline());
            render_pass.set_bind_group(0, &standard_render_res.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, material.uniform_bind_group(), &[]);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
            render_pass.set_vertex_buffer(1, standard_render_res.instance_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..(mesh.indices_count() as u32), 0, 0..1);
        }
        
        render_state.queue().submit(std::iter::once(encoder.finish()));
    }

}

pub fn render_gizmos(
    mut gizmos: ResMut<GizmosResource>,
    gizmos_render_res: Res<GizmosRenderResource>,
    surface_texture: Res<SurfaceTextureResource>,
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
            GizmoSpace::Viewport => Mat4::<f32>::IDENTITY,
            GizmoSpace::Window => {
                Mat4::<f32>::offset(Vec3::new(-1.0, 1.0, 0.0))
                * Mat3::<f32>::scale(Vec3::new(2.0 / window_size.width as f32, -2.0 / window_size.height as f32, 1.0)).into_4x4()
            },
            GizmoSpace::World => {
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

        while let Some(line) = lines.pop() {
            let vertex_data = [
                Vec4::new(line.start.x, line.start.y, line.start.z, 1.0),
                Vec4::new(line.end.x, line.end.y, line.end.z, 1.0),
            ];

            render_state.queue().write_buffer(&gizmos_render_res.index_buffer, 0, fruits_utils::mem::as_bytes(&[0_u32, 1_u32]));
            render_state.queue().write_buffer(&gizmos_render_res.vertex_buffer, 0, fruits_utils::mem::as_bytes(&vertex_data));
            render_state.queue().write_buffer(&gizmos_render_res.color_buffer, 0, fruits_utils::mem::as_bytes(line.color.as_array()));
            render_state.queue().submit([]);

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
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                });
            
                render_pass.set_pipeline(&gizmos_render_res.pipeline);
                render_pass.set_bind_group(0, &gizmos_render_res.bind_group, &[]);
                render_pass.set_vertex_buffer(0, gizmos_render_res.index_buffer.slice(..));
                render_pass.draw(0..2, 0..1);
            }

            render_state.queue().submit(std::iter::once(encoder.finish()));
        }
    }
}
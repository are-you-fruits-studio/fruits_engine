use std::collections::HashMap;

use fruits_app::RenderStateResource;
use fruits_ecs::{ExclusiveWorldAccess, Res, ResMut, WorldQuery};
use fruits_math::{Mat3, Mat4, Vec3, Vec4};
use wgpu::{include_wgsl, util::{BufferInitDescriptor, DeviceExt}, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, CommandEncoderDescriptor, DepthStencilState, Extent3d, FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, SamplerDescriptor, ShaderStages, StoreOp, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, VertexState};

use crate::{asset::AssetStorageResource, render::{utils::{GIZMO_LINES_PER_DRAW_MAX, STANDARD_MESH_MATERIAL_INSTANCES_PER_DRAW_MAX}, LitUniform, MaterialStandardRenderResourceData, StandardInstance, StandardVertex, UnlitUniform}, transform::GlobalTransform};

use super::{assets::{StandardMaterial, StandardMesh}, components::{CameraComponent, StandardMaterialComponent, StandardMeshComponent}, resources::SurfaceTextureResource, DepthTextureResource, GizmoSpace, GizmosRenderResource, GizmosResource, StandardRenderResource};

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

    let pipeline_layout = render_state.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Standard Pipeline Layout"),
        bind_group_layouts: &[
            &bind_group_layout,
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
            entry_point: "vs_main",
            buffers: &[
                StandardVertex::desc(),
                StandardInstance::desc(),
            ],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &lit_shader,
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
            entry_point: "vs_main",
            buffers: &[
                StandardVertex::desc(),
                StandardInstance::desc(),
            ],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &unlit_shader,
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

    world.resources_mut().insert(StandardRenderResource {
        pipeline_layout,
        instance_cpu_buffer,
        instance_buffer,
        lit: lit_data,
        unlit: unlit_data,
        camera_pos: Vec3::default(),
        camera_proj_matrix: Mat4::IDENTITY,
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

    let mut instanced_matrices = HashMap::new();

    for (transform, render_mesh, render_material) in query.iter() {
        instanced_matrices.entry((render_mesh.mesh.clone(), render_material.material.clone()))
            .or_insert_with(|| Vec::new())
            .push(transform.scale_rotation.into_4x4_with_offset(transform.position));
    }

    for ((mesh, material), matrices) in instanced_matrices.iter() {
        let Some(mesh) = meshes.get(&mesh) else { continue; };
        let Some(material) = materials.get_mut(&material) else { continue; };

        let (render_pipeline, bind_group) = match material {
            StandardMaterial::Lit(material) => {
                let lit_data = &standard_render_res.lit;

                let uniform = LitUniform {
                    albedo_color: material.albedo_color,
                    metallic: material.metallic,
                    emission_color: material.emission_color,
                    roughness: material.roughness,
                    camera_position_world: standard_render_res.camera_pos,
                    world_to_clip: standard_render_res.camera_proj_matrix,
                    _padding: Default::default(),
                };

                render_state.queue().write_buffer(&lit_data.buffer_uniform, 0, fruits_utils::mem::as_bytes(&[uniform]));

                (&lit_data.render_pipeline, &lit_data.bind_group_uniform)
            },
            StandardMaterial::Unlit(material) => {
                let unlit_data = &standard_render_res.unlit;

                let uniform = UnlitUniform {
                    world_to_clip: standard_render_res.camera_proj_matrix,
                    color: material.color,
                };

                render_state.queue().write_buffer(&unlit_data.buffer_uniform, 0, fruits_utils::mem::as_bytes(&[uniform]));

                (&unlit_data.render_pipeline, &unlit_data.bind_group_uniform)
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
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                render_pass.set_vertex_buffer(1, standard_render_res.instance_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..(mesh.indices_count() as u32), 0, 0..(matrices.len() as u32));
            }

            render_state.queue().submit(std::iter::once(encoder.finish()));
        }
    }

    // for (transform, render_mesh, render_material) in query.iter() {
    //     let Some(mesh) = meshes.get(&render_mesh.mesh) else { continue; };
    //     let Some(material) = materials.get_mut(&render_material.material) else { continue; };

    //     render_state.queue().write_buffer(material.uniform_buffer(), 0, fruits_utils::mem::as_bytes(&[*material.uniform()]));
        
    //     let transform_matrix = transform.scale_rotation.into_4x4_with_offset(transform.position);
    //     let transform_matrix = transform_matrix.into_array();
    //     let transform_matrix = fruits_utils::mem::as_bytes(&transform_matrix);

    //     render_state.queue().write_buffer(&standard_render_res.instance_buffer, 0, transform_matrix);
    //     render_state.queue().submit([]);

    //     let mut encoder = render_state.device().create_command_encoder(&CommandEncoderDescriptor {
    //         label: Some("Render Encoder"),
    //     });

    //     {
    //         let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
    //             label: Some("Render Pass"),
    //             color_attachments: &[Some(RenderPassColorAttachment {
    //                 view: &view,
    //                 resolve_target: None,
    //                 ops: Operations {
    //                     load: LoadOp::Load,
    //                     store: StoreOp::Store,
    //                 },
    //             })],
    //             depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
    //                 view: &depth_res.texture_view,
    //                 depth_ops: Some(Operations {
    //                     load: LoadOp::Load,
    //                     store: StoreOp::Store,
    //                 }),
    //                 stencil_ops: None,
    //             }),
    //             ..Default::default()
    //         });
    
    //         render_pass.set_pipeline(material.render_pipeline());
    //         render_pass.set_bind_group(0, &standard_render_res.uniform_bind_group, &[]);
    //         render_pass.set_bind_group(1, material.uniform_bind_group(), &[]);
    //         render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
    //         render_pass.set_vertex_buffer(1, standard_render_res.instance_buffer.slice(..));
    //         render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint16);
    //         render_pass.draw_indexed(0..(mesh.indices_count() as u32), 0, 0..1);
    //     }
        
    //     render_state.queue().submit(std::iter::once(encoder.finish()));
    // }

}

pub fn render_gizmos(
    mut gizmos: ResMut<GizmosResource>,
    mut gizmos_render_res: ResMut<GizmosRenderResource>,
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
            GizmoSpace::Clip => Mat4::<f32>::IDENTITY,
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
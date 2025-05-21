use fruits_app::RenderStateResource;
use fruits_ecs_system_params::{ExclusiveWorldAccess, Res, ResMut, WorldQuery};
use fruits_math::{Matrix, Matrix4x4, Vec2, Vec4};
use wgpu::{include_wgsl, util::{BufferInitDescriptor, DeviceExt}, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, CommandEncoderDescriptor, FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState};

use crate::{asset::AssetStorageResource, transform::GlobalTransform};

use super::{assets::{Material, Mesh}, components::{CameraComponent, RenderMaterialComponent, RenderMeshComponent}, resources::{CameraUniformBufferGroupLayoutResource, CameraUniformBufferResource, InstanceBufferResource, SurfaceTextureResource}, GizmosRenderResource, GizmosResource};

pub fn create_asset_resources(
    mut world: ExclusiveWorldAccess,
) {
    world.resources.insert(AssetStorageResource::<Material>::new()).ok().unwrap();
    world.resources.insert(AssetStorageResource::<Mesh>::new()).ok().unwrap();
}

pub fn create_camera_uniform_bind_group_layout(
    mut world: ExclusiveWorldAccess,
) {
    let layout = {
        let render_state = world.resources.get::<RenderStateResource>().unwrap();
        let render_state = &*render_state;

        render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        })
    };

    world.resources.insert(CameraUniformBufferGroupLayoutResource::new(layout)).ok().unwrap();
}

pub fn create_camera_uniform_buffer(
    mut world: ExclusiveWorldAccess,
) {
    let (buffer, group) = {
        let layout_resource = &*world.resources.get::<CameraUniformBufferGroupLayoutResource>().unwrap();

        let render_state = world.resources.get::<RenderStateResource>().unwrap();
        let render_state = &*render_state;

        let buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: fruits_utils::mem::as_bytes(Matrix4x4::<f32>::IDENTITY.as_array()),
        });

        let group = render_state.device().create_bind_group(&BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: layout_resource.layout(),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });

        (buffer, group)
    };

    world.resources.insert(CameraUniformBufferResource {
        buffer,
        group,
    }).ok().unwrap();
}

pub fn create_instance_buffer(
    mut world: ExclusiveWorldAccess,
) {
    let buffer = {
        let render_state = world.resources.get::<RenderStateResource>().unwrap();
        let render_state = &*render_state;

        let buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            contents: fruits_utils::mem::as_bytes(Matrix4x4::<f32>::IDENTITY.as_array()),
        });

        buffer
    };

    world.resources.insert(InstanceBufferResource {
        buffer,
    }).ok().unwrap();
}

pub fn create_gizmos_render_resource(
    mut world: ExclusiveWorldAccess,
) {
    let render_state = &*world.resources.get::<RenderStateResource>().unwrap();

    let buffer = render_state.device().create_buffer_init(&BufferInitDescriptor {
        label: Some("Gizmos Buffer"),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        contents: fruits_utils::mem::as_bytes(&[Vec2::<f32>::new(0.0, 0.0); 2]),
    });

    let bind_group_layout = render_state.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Gizmos Bind Group Layout"),
        entries: &[],
    });

    let bind_group = render_state.device().create_bind_group(&BindGroupDescriptor {
        label: Some("Gizmos Bind Group"),
        layout: &bind_group_layout,
        entries: &[],
    });

    let pipeline_layout = render_state.device().create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Gizmos Render Pipeline Layout"),
        bind_group_layouts: &[],
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
                format: render_state.surface_config().lock().unwrap().format,
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
                    array_stride: std::mem::size_of::<Vec2<f32>>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }
                    ],
                }
            ],
            compilation_options: Default::default(),
        }
    });

    world.resources.insert(GizmosRenderResource {
        vertex_buffer: buffer,
        bind_group,
        pipeline,
    }).ok().unwrap();
}

pub fn update_camera_uniform_buffer(
    render_state: Res<RenderStateResource>,
    buffer: ResMut<CameraUniformBufferResource>,
    query: WorldQuery<(&GlobalTransform, &CameraComponent)>,
) {
    if query.len() == 0 {
        return;
    }

    if query.len() > 1 {
        panic!("There should be no more than one camera in the world.");
    }

    let (transform, camera) = query.iter().next().unwrap();

    let window_size = render_state.size().lock().unwrap();

    let aspect = window_size.width as f32 / window_size.height as f32;

    let projection_matrix = fruits_math::perspective_proj_matrix(camera.fov, camera.near, camera.far, aspect);

    let transform_matrix = fruits_math::into_matrix4x4_with_pos(transform.scale_rotation, transform.position).inverse().unwrap();

    let matrix = projection_matrix * transform_matrix;

    let matrix = matrix.into_array();
    let matrix = fruits_utils::mem::as_bytes(&matrix);

    render_state.queue().write_buffer(&buffer.buffer, 0, matrix);
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

pub fn create_gizmos_resource(
    mut world: ExclusiveWorldAccess,
) {
    world.resources.insert(GizmosResource{ lines: Vec::new(), }).ok().unwrap();
}

pub fn render_meshes_and_materials(
    query: WorldQuery<(&GlobalTransform, &RenderMeshComponent, &RenderMaterialComponent)>,
    render_state: Res<RenderStateResource>,
    camera_buffer: Res<CameraUniformBufferResource>,
    instance_buffer: Res<InstanceBufferResource>,
    surface_texture: ResMut<SurfaceTextureResource>,
    meshes: Res<AssetStorageResource<Mesh>>,
    materials: Res<AssetStorageResource<Material>>,
) {
    if query.len() == 0 {
        return;
    }

    let Some(surface_texture) = &surface_texture.texture else { return; }; 

    let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());

    let render_state = &*render_state;

    for (transform, render_mesh, render_material) in query.iter() {
        let Some(mesh) = meshes.get(&render_mesh.mesh) else { continue; };
        let Some(material) = materials.get(&render_material.material) else { continue; };

        let transform_matrix = fruits_math::into_matrix4x4_with_pos(transform.scale_rotation, transform.position);
        let transform_matrix = transform_matrix.into_array();
        let transform_matrix = fruits_utils::mem::as_bytes(&transform_matrix);

        render_state.queue().write_buffer(&instance_buffer.buffer, 0, transform_matrix);
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
                depth_stencil_attachment: None,
                ..Default::default()
            });
    
            render_pass.set_pipeline(material.render_pipeline());
            render_pass.set_bind_group(0, &camera_buffer.group, &[]);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..(mesh.indices_count() as u32), 0, 0..1);
        }
        
        render_state.queue().submit(std::iter::once(encoder.finish()));
    }

}

pub fn render_gizmos(
    mut gizmos: ResMut<GizmosResource>,
    gizmos_render_res: ResMut<GizmosRenderResource>,
    surface_texture: ResMut<SurfaceTextureResource>,
    render_state: Res<RenderStateResource>,
) {
    if gizmos.lines.len() == 0 {
        return;
    }

    let Some(surface_texture) = &surface_texture.texture else { return; }; 

    let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());

    let render_state = &*render_state;

    while let Some(line) = gizmos.lines.pop() {
        let buffer_data = fruits_utils::mem::as_bytes(&line);

        render_state.queue().write_buffer(&gizmos_render_res.vertex_buffer, 0, buffer_data);
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
            render_pass.set_vertex_buffer(0, gizmos_render_res.vertex_buffer.slice(..));
            render_pass.draw(0..2, 0..1);
        }
        
        render_state.queue().submit(std::iter::once(encoder.finish()));
    }
}
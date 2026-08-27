use fruits_utils::stack_vec::StackVec;
use wgpu::*;

const MAX_ENTRIES: usize = 16;

pub enum CreateBindGroupLayoutEntry {
    BufferUniform,
    BufferStorage,
    BufferStorageMut,
    Texture,
    SamplerFiltering,
    SamplerNonFiltering,
}

pub enum CreateBindGroupEntry<'a> {
    Buffer(&'a Buffer),
    Texture(&'a TextureView),
    Sampler(&'a Sampler),
}

pub fn create_bind_group_layout(
    device: &Device,
    label: Option<&str>,
    entries: &[CreateBindGroupLayoutEntry],
) -> BindGroupLayout {
    fn create_biniding_type_buffer(ty: BufferBindingType) -> BindingType {
        BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    if entries.len() > MAX_ENTRIES {
        panic!("too many entries in a bind group. {} > {}", entries.len(), MAX_ENTRIES);
    }

    let mut descriptor_entries = StackVec::<BindGroupLayoutEntry, MAX_ENTRIES>::new();

    for (binding, entry) in entries.iter().enumerate() {
        descriptor_entries.push(BindGroupLayoutEntry {
            binding: binding as u32,
            count: None,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: match entry {
                CreateBindGroupLayoutEntry::BufferUniform => create_biniding_type_buffer(BufferBindingType::Uniform),
                CreateBindGroupLayoutEntry::BufferStorage => create_biniding_type_buffer(BufferBindingType::Storage { read_only: true }),
                CreateBindGroupLayoutEntry::BufferStorageMut => create_biniding_type_buffer(BufferBindingType::Storage { read_only: false }),
                CreateBindGroupLayoutEntry::Texture => BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                CreateBindGroupLayoutEntry::SamplerFiltering => BindingType::Sampler(SamplerBindingType::Filtering),
                CreateBindGroupLayoutEntry::SamplerNonFiltering => BindingType::Sampler(SamplerBindingType::NonFiltering),
            },
        }).unwrap();
    }

    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label,
        entries: &descriptor_entries,
    })
}

pub fn create_bind_group(
    device: &Device,
    label: Option<&str>,
    layout: &BindGroupLayout,
    entries: &[CreateBindGroupEntry],
) -> BindGroup {
    if entries.len() > MAX_ENTRIES {
        panic!("too many entries in a bind group. {} > {}", entries.len(), MAX_ENTRIES);
    }

    let mut descriptor_entries = StackVec::<BindGroupEntry, MAX_ENTRIES>::new();

    for (binding, entry) in entries.iter().enumerate() {
        descriptor_entries.push(BindGroupEntry {
            binding: binding as u32,
            resource: match entry {
                CreateBindGroupEntry::Buffer(buffer) => buffer.as_entire_binding(),
                CreateBindGroupEntry::Texture(texture) => BindingResource::TextureView(texture),
                CreateBindGroupEntry::Sampler(sampler) => BindingResource::Sampler(sampler),
            },
        }).unwrap();
    }

    device.create_bind_group(&BindGroupDescriptor {
        label,
        layout,
        entries: &descriptor_entries,
    })
}

pub fn create_render_pipeline(
    device: &Device,
    label: Option<&str>,
    layout: &PipelineLayout,
    shader: &ShaderModule,
    vertex_buffers: &[VertexBufferLayout],
    color_target_state: ColorTargetState,
    depth_stencil: Option<DepthStencilState>,
    topology: PrimitiveTopology,
    cull_mode: Option<Face>,
) -> RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_buffers,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(color_target_state)],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
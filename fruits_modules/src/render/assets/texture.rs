use fruits_app::RenderStateResource;
use fruits_ecs::WorldData;
use wgpu::{util::{DeviceExt, TextureDataOrder}, wgt::TextureViewDescriptor, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d, FilterMode, SamplerDescriptor, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

use crate::render::StandardRenderResource;

#[derive(Debug)]
pub struct StandardTexture {
    texture: Texture,
    bind_group: BindGroup,
}

impl StandardTexture {
    pub fn from_world(world: &WorldData, filter_mode: FilterMode, dimensions: [u32; 2], data: &[u8]) -> Self {
        let render_state = world.resources().get::<RenderStateResource>().unwrap();
        let standard_render_resource = world.resources().get::<StandardRenderResource>().unwrap();

        let texture = render_state.device().create_texture_with_data(render_state.queue(), &TextureDescriptor {
            label: None,
            size: Extent3d { width: dimensions[0], height: dimensions[1], depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[TextureFormat::Rgba8Unorm],
        }, TextureDataOrder::LayerMajor, data);

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = render_state.device().create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: FilterMode::Nearest,
            mipmap_filter: filter_mode,
            ..Default::default()
        });

        Self {
            texture: texture,
            bind_group: render_state.device().create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &standard_render_resource.bind_group_layout_standard_texture,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&sampler),
                    }
                ]
            })
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
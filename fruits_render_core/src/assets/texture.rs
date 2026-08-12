use std::fmt::Debug;

use fruits_ffi::{FfiDroppable, FfiOption, FfiString};
use fruits_serialization::*;
use wgpu::{
    AddressMode, Extent3d, Sampler, SamplerDescriptor, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, util::{DeviceExt, TextureDataOrder},
};

use crate::RenderApi;

pub use wgpu::FilterMode;

#[derive(Debug)]
pub struct StandardTextureNative {
    pub texture: Texture,
    pub sampler: Sampler,
}

#[repr(C)]
#[derive(TransSerializable, Clone)]
pub struct StandardTextureAssetMetadata {
    pub raw_texture: FfiString,
}

#[repr(C)]
pub struct StandardTexture {
    native: FfiDroppable,
    meta: FfiOption<StandardTextureAssetMetadata>,
}

impl StandardTexture {
    pub(crate) fn new(
        render_api: &RenderApi,
        filter_mode: FilterMode,
        dimensions: [u32; 2],
        data: &[u8],
        meta: Option<StandardTextureAssetMetadata>,
    ) -> Self {
        let px_count = (dimensions[0] * dimensions[1]) as usize;

        let bytes_per_pixel = data.len() / px_count;

        let mut data = data;
        let mut data_vec = Vec::new();

        if bytes_per_pixel < 4 {
            for i in 0..px_count {
                let mut px = [0, 0, 0, 255];

                for j in 0..bytes_per_pixel {
                    px[j] = data[bytes_per_pixel * i + j];
                }

                data_vec.extend_from_slice(&px);
            }

            data = data_vec.as_slice();
        }

        let texture = render_api.device.create_texture_with_data(
            &render_api.queue,
            &TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: dimensions[0],
                    height: dimensions[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[TextureFormat::Rgba8UnormSrgb],
            },
            TextureDataOrder::LayerMajor,
            data,
        );

        let sampler = render_api.device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: FilterMode::Nearest,
            mipmap_filter: filter_mode,
            ..Default::default()
        });

        Self {
            native: FfiDroppable::new(StandardTextureNative {
                texture,
                sampler,
            }),
            meta: meta.into(),
        }
    }

    pub unsafe fn native(&self) -> &StandardTextureNative {
        unsafe { &*(self.native.get() as *const StandardTextureNative) }
    }

    pub fn meta(&self) -> Option<&StandardTextureAssetMetadata> {
        self.meta.as_ref()
    }
}

impl Debug for StandardTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StandardTexture").field("native", &self.native.get()).finish()
    }
}

unsafe impl Send for StandardTexture where StandardTextureNative: Send {}
unsafe impl Sync for StandardTexture where StandardTextureNative: Sync {}

use std::{ffi::c_void, sync::{Arc, Mutex}};

use fruits_ecs::Resource;
use fruits_ffi::{FfiDroppable, FfiOption, FfiSliceRef, FfiStaticRef};
use wgpu::*;
use winit::window::Window;

use crate::{StandardMaterial, StandardMaterialAssetMetadata, StandardMaterialAssets, StandardMesh, StandardMeshAssetMetadata, StandardTexture, StandardTextureAssetMetadata, StandardVertex};

// todo: ffi?

pub struct SurfaceConfigCache {
    surface_config: SurfaceConfiguration,
    size: [u32; 2],
}

pub struct RenderApi {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub window: Arc<Window>,
    surface_config: Mutex<SurfaceConfigCache>,
}

impl RenderApi {
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let size = [size.width, size.height];

        // todo: move wgpu initialization into ecs Start handle?
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            required_features: Features::empty(),
            required_limits: Limits::default(),
            label: None,
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        }))
        .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size[0],
            height: size[1],
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let surface_config = Mutex::new(SurfaceConfigCache {
            size,
            surface_config,
        });

        Self {
            device,
            queue,
            surface,
            window,
            surface_config,
        }
    }

    pub fn create_texture(&self, filter_mode: FilterMode, dimensions: [u32; 2], data: &[u8], meta: Option<StandardTextureAssetMetadata>) -> StandardTexture {
        StandardTexture::new(self, filter_mode, dimensions, data, meta)
    }

    pub fn create_mesh(&self, vertices: &[StandardVertex], indices: &[u16], meta: Option<StandardMeshAssetMetadata>) -> StandardMesh {
        StandardMesh::new(&self.device, vertices, indices, meta)
    }
}

pub struct RenderData {
    pub bind_group_layout_global: BindGroupLayout,
    pub bind_group_layout_material: BindGroupLayout,
    pub texture_white: StandardTexture,
    pub texture_normal_default: StandardTexture,
}

impl RenderData {
    pub fn new(api: &RenderApi) -> Self {
        let bind_group_layout_global = api.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Standard Global Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
            ],
        });
        let bind_group_layout_material = api.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Standard Material Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 6,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                },
                BindGroupLayoutEntry {
                    binding: 7,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 8,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                },
                BindGroupLayoutEntry {
                    binding: 9,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 10,
                    count: None,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                },
            ],
        });
        
        let texture_white = api.create_texture(FilterMode::Linear, [2, 2], &[255; 16], None);
        let texture_normal_default = api.create_texture(
            FilterMode::Linear,
            [2, 2],
            &[
                128, 128, 255, 255,
                128, 128, 255, 255,
                128, 128, 255, 255,
                128, 128, 255, 255,
            ],
            Some(StandardTextureAssetMetadata {
                raw_texture: Default::default(),
                is_linear: true,
            }),
        );

        Self {
            bind_group_layout_global,
            bind_group_layout_material,
            texture_white,
            texture_normal_default,
        }
    }
}

pub struct RenderState {
    api: RenderApi,
    render_data: RenderData,
}

impl RenderState {
    pub fn new(window: Arc<Window>) -> Self {
        let api = RenderApi::new(window);
        let render_data = RenderData::new(&api);

        Self { api, render_data }
    }

    pub fn api(&self) -> &RenderApi {
        &self.api
    }

    // todo: expose api as a struct, not deconstruct it.
    pub fn device(&self) -> &Device {
        &self.api.device
    }

    pub fn queue(&self) -> &Queue {
        &self.api.queue
    }

    pub fn surface(&self) -> &Surface<'static> {
        &self.api.surface
    }

    pub fn surface_config_format(&self) -> TextureFormat {
        self.api.surface_config.lock().unwrap().surface_config.format
    }

    pub fn window(&self) -> &Window {
        &self.api.window
    }

    pub fn size(&self) -> [u32; 2] {
        self.api.surface_config.lock().unwrap().size
    }

    pub fn render_data(&self) -> &RenderData {
        &self.render_data
    }

    //

    pub fn resize(&self, new_size: [u32; 2]) {
        if new_size[0] <= 0 || new_size[1] <= 0 {
            return;
        }

        let mut surface_config_cache = self.api.surface_config.lock().unwrap();

        surface_config_cache.size = new_size;

        surface_config_cache.surface_config.width = new_size[0];
        surface_config_cache.surface_config.height = new_size[1];

        self.api.surface.configure(&self.api.device, &surface_config_cache.surface_config);
    }

    pub fn create_texture(&self, filter_mode: FilterMode, dimensions: [u32; 2], data: &[u8], meta: Option<StandardTextureAssetMetadata>) -> StandardTexture {
        self.api.create_texture(filter_mode, dimensions, data, meta)
    }

    pub fn create_mesh(&self, vertices: &[StandardVertex], indices: &[u16], meta: Option<StandardMeshAssetMetadata>) -> StandardMesh {
        self.api.create_mesh(vertices, indices, meta)
    }

    pub fn create_material(
        &self,
        assets: StandardMaterialAssets<Option<&StandardTexture>>,
        meta: StandardMaterialAssetMetadata,
    ) -> StandardMaterial {
        // todo: use asset-handles as a single source of truth - probably add an is_dirty flag to the material and update the gpu-side data every frame (if dirty)
        let assets = StandardMaterialAssets {
            color_texture: assets.color_texture.unwrap_or_else(|| &self.render_data.texture_white),
            roughness_texture: assets.roughness_texture.unwrap_or_else(|| &self.render_data.texture_white),
            metallic_texture: assets.metallic_texture.unwrap_or_else(|| &self.render_data.texture_white),
            normal_texture: assets.normal_texture.unwrap_or_else(|| &self.render_data.texture_normal_default),
            emission_texture: assets.emission_texture.unwrap_or_else(|| &self.render_data.texture_white),
        };
        StandardMaterial::new(
            self,
            assets,
            meta,
        )
    }
}

//

#[repr(C)]
struct RenderApiVTable {
    resize_fn: unsafe extern "C" fn(*const c_void, new_size: *const u32),
    size_fn: unsafe extern "C" fn(*const c_void, size_dst: *mut u32),
    create_texture_fn: unsafe extern "C" fn(*const c_void, filter_mode: FilterMode, dimensions: *const u32, data: FfiSliceRef<u8>, meta: FfiOption<StandardTextureAssetMetadata>) -> StandardTexture,
    create_mesh_fn: unsafe extern "C" fn(*const c_void, vertices: FfiSliceRef<StandardVertex>, indices: FfiSliceRef<u16>, meta: FfiOption<StandardMeshAssetMetadata>) -> StandardMesh,
    create_material_fn: unsafe extern "C" fn(*const c_void, StandardMaterialAssets<FfiOption<&StandardTexture>>, meta: StandardMaterialAssetMetadata) -> StandardMaterial,
    clone_fn: unsafe extern "C" fn(*const c_void) -> FfiDroppable,
}

#[derive(Resource)]
#[repr(C)]
pub struct RenderApiResource {
    data: FfiDroppable,
    vtable: FfiStaticRef<RenderApiVTable>,
}

impl RenderApiResource {
    pub fn new(window: Arc<Window>) -> Self {
        unsafe extern "C" fn ffi_resize(this: *const c_void, new_size: *const u32) {
            unsafe {
                let this = &*(this as *mut Arc<RenderState>);
                let new_size = (new_size as *const [u32; 2]).read();

                this.resize(new_size);
            }
        }
        unsafe extern "C" fn ffi_size(this: *const c_void, size_dst: *mut u32) {
            unsafe {
                let this = &*(this as *const Arc<RenderState>);

                let size = this.size();

                (size_dst as *mut [u32; 2]).write(size);
            }
        }
        unsafe extern "C" fn ffi_create_texture(
            this: *const c_void,
            filter_mode: FilterMode,
            dimensions: *const u32,
            data: FfiSliceRef<u8>,
            meta: FfiOption<StandardTextureAssetMetadata>,
        ) -> StandardTexture {
            unsafe {
                let this = &*(this as *const Arc<RenderState>);
                let dimensions = (dimensions as *const [u32; 2]).read();
                let data = data.into_slice();

                this.create_texture(filter_mode, dimensions, data, meta.into())
            }
        }
        unsafe extern "C" fn ffi_create_mesh(
            this: *const c_void,
            vertices: FfiSliceRef<StandardVertex>,
            indices: FfiSliceRef<u16>,
            meta: FfiOption<StandardMeshAssetMetadata>,
        ) -> StandardMesh {
            unsafe {
                let this = &*(this as *const Arc<RenderState>);
                let vertices = vertices.into_slice();
                let indices = indices.into_slice();

                this.create_mesh(vertices, indices, meta.into())
            }
        }
        unsafe extern "C" fn ffi_create_material(
            this: *const c_void,
            assets: StandardMaterialAssets<FfiOption<&StandardTexture>>,
            meta: StandardMaterialAssetMetadata,
        ) -> StandardMaterial {
            unsafe {
                let this = &*(this as *const Arc<RenderState>);

                this.create_material(assets.map(|o| o.into_option()), meta)
            }
        }
        unsafe extern "C" fn ffi_clone(this: *const c_void) -> FfiDroppable {
            unsafe {
                let this = &*(this as *const Arc<RenderState>);

                FfiDroppable::new(Arc::clone(this))
            }
        }

        Self {
            data: FfiDroppable::new(Arc::new(RenderState::new(window))),
            vtable: FfiStaticRef::new(&RenderApiVTable {
                resize_fn: ffi_resize,
                size_fn: ffi_size,
                create_texture_fn: ffi_create_texture,
                create_mesh_fn: ffi_create_mesh,
                create_material_fn: ffi_create_material,
                clone_fn: ffi_clone,
            }),
        }
    }

    pub fn resize(&self, new_size: [u32; 2]) {
        unsafe {
            let this = self.data.get();
            let new_size = &raw const new_size as *const u32;

            (self.vtable.resize_fn)(this, new_size)
        }
    }

    pub fn size(&self) -> [u32; 2] {
        unsafe {
            let this = self.data.get();
            let mut size = [0_u32; 2];
            let size_dst = &raw mut size as *mut u32;

            (self.vtable.size_fn)(this, size_dst);

            size
        }
    }

    pub fn create_texture(&self, filter_mode: FilterMode, dimensions: [u32; 2], data: &[u8], meta: Option<StandardTextureAssetMetadata>) -> StandardTexture {
        unsafe {
            let this = self.data.get();
            let dimensions = &raw const dimensions as *const u32;
            let data = FfiSliceRef::from_slice(data);

            let result = (self.vtable.create_texture_fn)(this, filter_mode, dimensions, data, meta.into());

            result
        }
    }

    pub fn create_mesh(&self, vertices: &[StandardVertex], indices: &[u16], meta: Option<StandardMeshAssetMetadata>) -> StandardMesh {
        unsafe {
            let this = self.data.get();
            let vertices = FfiSliceRef::from_slice(vertices);
            let indices = FfiSliceRef::from_slice(indices);

            (self.vtable.create_mesh_fn)(this, vertices, indices, meta.into())
        }
    }

    pub fn create_material(&self, assets: StandardMaterialAssets<Option<&StandardTexture>>, meta: StandardMaterialAssetMetadata) -> StandardMaterial {
        unsafe {
            let this = self.data.get();

            (self.vtable.create_material_fn)(this, assets.map(|t| t.into()), meta)
        }
    }

    pub fn clone(&self) -> Self {
        unsafe {
            Self {
                data: (self.vtable.clone_fn)(self.data.get()),
                vtable: self.vtable,
            }
        }
    }

    pub unsafe fn raw(&self) -> Arc<RenderState> {
        unsafe { &*(self.data.get() as *mut Arc<RenderState>) }.clone()
    }
}

unsafe impl Send for RenderApiResource {}
unsafe impl Sync for RenderApiResource {}

use std::{ffi::c_void, sync::Arc};

use fruits_ecs::Resource;
use fruits_ffi::{FfiDroppable, FfiSliceRef, FfiStaticRef};
use wgpu::*;
use winit::window::Window;

use crate::{StandardMesh, StandardTexture, StandardVertex};

struct RenderApi {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    window: Arc<Window>,
    size: [u32; 2],
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

        let adapter = pollster::block_on(instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )).unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            required_features: Features::empty(),
            required_limits: Limits::default(),
            label: None,
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        })).unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities.formats.iter()
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

        
        Self {
            device,
            queue,
            surface,
            surface_config,
            window,
            size,
        }
    }
}

pub(crate) struct RenderData {
    pub bind_group_layout_standard_texture: BindGroupLayout,
}

impl RenderData {
    pub fn new(api: &RenderApi) -> Self {
        let bind_group_layout_standard_texture = api.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        Self {
            bind_group_layout_standard_texture,
        }
    }
}

pub(crate) struct RenderState {
    api: RenderApi,
    render_data: RenderData,
}

impl RenderState {
    pub fn new(window: Arc<Window>) -> Self {
        let api = RenderApi::new(window);
        let render_data = RenderData::new(&api);

        Self {
            api,
            render_data
        }
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

    pub fn surface_config(&self) -> &SurfaceConfiguration {
        &self.api.surface_config
    }

    pub unsafe fn surface_config_mut(&mut self) -> &mut SurfaceConfiguration {
        &mut self.api.surface_config
    }

    pub fn window(&self) -> &Window {
        &self.api.window
    }

    pub fn size(&self) -> [u32; 2] {
        self.api.size
    }

    pub(crate) fn render_data(&self) -> &RenderData {
        &self.render_data
    }

    //

    pub fn resize(&mut self, new_size: [u32; 2]) {
        if new_size[0] <= 0 || new_size[1] <= 0 {
            return;
        }

        self.api.size = new_size;
        let surface_config = &mut self.api.surface_config;

        surface_config.width = new_size[0];
        surface_config.height = new_size[1];
        
        self.api.surface.configure(&self.api.device, &self.api.surface_config);
    }

    pub fn create_texture(&self, filter_mode: FilterMode, dimensions: [u32; 2], data: &[u8]) -> StandardTexture {
        StandardTexture::new(self, filter_mode, dimensions, data)
    }

    pub fn create_mesh(&self, vertices: &[StandardVertex], indices: &[u16]) -> StandardMesh {
        StandardMesh::new(&self.api.device, vertices, indices)
    }
}

//

struct RenderApiVTable {
    resize_fn: unsafe extern "C" fn(*mut c_void, new_size: *const u32),
    size_fn: unsafe extern "C" fn(*const c_void, size_dst: *mut u32),
    create_texture_fn: unsafe extern "C" fn(*const c_void, filter_mode: FilterMode, dimensions: *const u32, data: FfiSliceRef<u8>) -> StandardTexture,
    create_mesh_fn: unsafe extern "C" fn(*const c_void, vertices: FfiSliceRef<StandardVertex>, indices: FfiSliceRef<u16>) -> StandardMesh,
}

#[derive(Resource)]
pub struct RenderApiResource {
    data: FfiDroppable,
    vtable: FfiStaticRef<RenderApiVTable>
}

impl RenderApiResource {
    pub fn new(window: Arc<Window>) -> Self {
        unsafe extern "C" fn ffi_resize(this: *mut c_void, new_size: *const u32) {
            unsafe {
                let this = &mut *(this as *mut RenderState);
                let new_size = (new_size as *const [u32; 2]).read();

                this.resize(new_size);
            }
        }
        unsafe extern "C" fn ffi_size(this: *const c_void, size_dst: *mut u32) {
            unsafe {
                let this = &*(this as *const RenderState);
                
                let size = this.size();
                
                (size_dst as *mut [u32; 2]).write(size);
            }
        }
        unsafe extern "C" fn ffi_create_texture(this: *const c_void, filter_mode: FilterMode, dimensions: *const u32, data: FfiSliceRef<u8>) -> StandardTexture {
            unsafe {
                let this = &*(this as *const RenderState);
                let dimensions = (dimensions as *const [u32; 2]).read();
                let data = data.into_slice();
                
                this.create_texture(filter_mode, dimensions, data)
            }
        }
        unsafe extern "C" fn ffi_create_mesh(this: *const c_void, vertices: FfiSliceRef<StandardVertex>, indices: FfiSliceRef<u16>) -> StandardMesh {
            unsafe {
                let this = &*(this as *const RenderState);
                let vertices = vertices.into_slice();
                let indices = indices.into_slice();
                
                this.create_mesh(vertices, indices)
            }
        }

        Self {
            data: FfiDroppable::new(RenderState::new(window)),
            vtable: FfiStaticRef::new(&RenderApiVTable {
                resize_fn: ffi_resize,
                size_fn: ffi_size,
                create_texture_fn: ffi_create_texture,
                create_mesh_fn: ffi_create_mesh,
            })
        }
    }

    pub fn resize(&mut self, new_size: [u32; 2]) {
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

    pub fn create_texture(&self, filter_mode: FilterMode, dimensions: [u32; 2], data: &[u8]) -> StandardTexture {
        unsafe {
            let this = self.data.get();
            let dimensions = &raw const dimensions as *const u32;
            let data = FfiSliceRef::from_slice(data);

            let result = (self.vtable.create_texture_fn)(this, filter_mode, dimensions, data);

            result
        }
    }

    pub fn create_mesh(&self, vertices: &[StandardVertex], indices: &[u16]) -> StandardMesh {
        unsafe {
            let this = self.data.get();
            let vertices = FfiSliceRef::from_slice(vertices);
            let indices = FfiSliceRef::from_slice(indices);

            (self.vtable.create_mesh_fn)(this, vertices, indices)
        }
    }

    pub(crate) fn raw(&self) -> &RenderState {
        unsafe { &*(self.data.get() as *mut RenderState) }
    }
}

unsafe impl Send for RenderApiResource { }
unsafe impl Sync for RenderApiResource { }
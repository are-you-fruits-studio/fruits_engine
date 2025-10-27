use std::sync::Arc;

use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

// todo: remove
// pub struct RenderAppState {
//     device: Device,
//     queue: Queue,
//     surface: Surface<'static>,
//     surface_config: SurfaceConfiguration,
//     window: Arc<Window>,
//     size: PhysicalSize<u32>,
// }

// impl RenderAppState {
//     pub fn new(window: Arc<Window>) -> Self {
//         let size = window.inner_size();
        
//         // todo: move wgpu initialization into ecs Start handle?
//         let instance = Instance::new(&InstanceDescriptor {
//             backends: Backends::PRIMARY,
//             ..Default::default()
//         });

//         let surface = instance.create_surface(Arc::clone(&window)).unwrap();

//         let adapter = pollster::block_on(instance.request_adapter(
//             &RequestAdapterOptions {
//                 power_preference: PowerPreference::default(),
//                 compatible_surface: Some(&surface),
//                 force_fallback_adapter: false,
//             },
//         )).unwrap();

//         let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
//             required_features: Features::empty(),
//             required_limits: Limits::default(),
//             label: None,
//             memory_hints: wgpu::MemoryHints::Performance,
//             ..Default::default()
//         })).unwrap();

//         let surface_capabilities = surface.get_capabilities(&adapter);

//         let surface_format = surface_capabilities.formats.iter()
//             .copied()
//             .find(|f| f.is_srgb())
//             .unwrap_or(surface_capabilities.formats[0]);

//         let surface_config = SurfaceConfiguration {
//             usage: TextureUsages::RENDER_ATTACHMENT,
//             format: surface_format,
//             width: size.width,
//             height: size.height,
//             present_mode: surface_capabilities.present_modes[0],
//             alpha_mode: surface_capabilities.alpha_modes[0],
//             view_formats: vec![],
//             desired_maximum_frame_latency: 2,
//         };

//         surface.configure(&device, &surface_config);

        
//         Self {
//             device,
//             queue,
//             surface,
//             surface_config,
//             window,
//             size,
//         }
//     }

//     pub fn device(&self) -> &Device {
//         &self.device
//     }

//     pub fn queue(&self) -> &Queue {
//         &self.queue
//     }

//     pub fn surface(&self) -> &Surface<'static> {
//         &self.surface
//     }

//     pub fn surface_config(&self) -> &SurfaceConfiguration {
//         &self.surface_config
//     }

//     pub fn window(&self) -> &Window {
//         &self.window
//     }

//     pub fn size(&self) -> PhysicalSize<u32> {
//         self.size
//     }

//     //

//     pub fn resize(&mut self, new_size: [u32; 2]) {
//         if new_size[0] <= 0 || new_size[1] <= 0 {
//             return;
//         }

//         self.size = PhysicalSize { width: new_size[0], height: new_size[1] };
//         let surface_config = &mut self.surface_config;

//         surface_config.width = new_size[0];
//         surface_config.height = new_size[1];
        
//         self.surface.configure(&self.device, &self.surface_config);
//     }
// }
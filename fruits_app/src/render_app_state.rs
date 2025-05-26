use std::sync::Arc;

use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, window::Window};

pub struct RenderAppState {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    window: Arc<Window>,
    size: PhysicalSize<u32>,
}

impl RenderAppState {
    pub fn new(
        device: Device,
        queue: Queue,
        surface: Surface<'static>,
        surface_config: SurfaceConfiguration,
        window: Arc<Window>,
        size: PhysicalSize<u32>,
    ) -> Self {
        Self {
            device,
            queue,
            surface,
            surface_config,
            window,
            size,
        }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn surface_config(&self) -> &SurfaceConfiguration {
        &self.surface_config
    }

    pub unsafe fn surface_config_mut(&mut self) -> &mut SurfaceConfiguration {
        &mut self.surface_config
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub unsafe fn size_mut(&mut self) -> &mut PhysicalSize<u32> {
        &mut self.size
    }

}
use fruits_ffi::FfiDroppable;
use fruits_utils::mem::{AllBitVariationsValid, AllBitsInit};
use wgpu::{util::DeviceExt, Buffer, Device};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct StandardVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

unsafe impl AllBitVariationsValid for StandardVertex { }
unsafe impl AllBitsInit for StandardVertex { }

impl StandardVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x4,
            3 => Float32x2,
        ];
        
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<StandardVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub(crate) struct StandardMeshNative {
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) indices_count: usize,
}

#[repr(transparent)]
pub struct StandardMesh {
    native: FfiDroppable,
}

impl StandardMesh {
    pub(crate) fn new(device: &Device, vertices: &[StandardVertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: fruits_utils::mem::as_bytes_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: fruits_utils::mem::as_bytes_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        Self {
            native: FfiDroppable::new(StandardMeshNative {
                index_buffer,
                vertex_buffer,
                indices_count: indices.len(),
            }),
        }
    }

    pub(crate) fn native(&self) -> &StandardMeshNative {
        unsafe { &*(self.native.get() as *const StandardMeshNative) }
    }
}

unsafe impl Send for StandardMesh where StandardMeshNative: Send { }
unsafe impl Sync for StandardMesh where StandardMeshNative: Sync { }
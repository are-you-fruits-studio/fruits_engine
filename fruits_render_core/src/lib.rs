//! # fruits_render_core
//!
//! The low-level rendering foundation of the engine. It owns the connection to the GPU and the
//! window surface, and defines the standard geometry and texture formats that the rest of the
//! rendering stack draws with.
//!
//! # How to use
//!
//! The engine inserts a single [`RenderApiResource`] into the world when the window is created, so
//! application code never constructs one itself — it pulls the resource out of the world and asks
//! it to build GPU resources. The two things a user uploads are meshes and textures; the standard
//! vertex and instance formats describe the geometry those meshes hold.
//!
//! #### Uploading a mesh to the GPU
//!
//! Turn a vertex list and a `u16` index list into a [`StandardMesh`] that lives in GPU memory.
//! Most code reaches the resource through the world it is running in:
//!
//! ```ignore
//! use fruits_render_core::{RenderApiResource, StandardVertex};
//!
//! // `render_api` is the `&RenderApiResource` the engine inserted into the world.
//! let vertices = [
//!     StandardVertex { position: [-0.5, -0.5, 0.0], normal: [0.0, 0.0, 1.0], color: [1.0; 4], uv: [0.0, 1.0] },
//!     StandardVertex { position: [ 0.5, -0.5, 0.0], normal: [0.0, 0.0, 1.0], color: [1.0; 4], uv: [1.0, 1.0] },
//!     StandardVertex { position: [ 0.0,  0.5, 0.0], normal: [0.0, 0.0, 1.0], color: [1.0; 4], uv: [0.5, 0.0] },
//! ];
//! let indices = [0u16, 1, 2];
//!
//! let mesh = render_api.create_mesh(&vertices, &indices);
//! ```
//!
//! #### Uploading a texture to the GPU
//!
//! Upload raw pixel bytes as a [`StandardTexture`]. The [`FilterMode`] selects how the texture is
//! sampled when magnified; the dimensions are `[width, height]` in pixels:
//!
//! ```ignore
//! use fruits_render_core::{FilterMode, RenderApiResource};
//!
//! // `render_api` is the `&RenderApiResource` the engine inserted into the world.
//! // `rgba` holds `width * height * 4` bytes; sources with fewer channels are padded to RGBA.
//! let texture = render_api.create_texture(FilterMode::Nearest, [width, height], &rgba);
//! ```
//!
//! #### Describing the standard vertex and instance formats
//!
//! When building a render pipeline, declare the buffer layouts geometry is fed through.
//! [`StandardVertex::desc`] describes the per-vertex stream (position, normal, color, uv at shader
//! locations 0–3) and [`StandardInstance::desc`] the per-instance stream (a `local_to_world`
//! matrix at locations 5–8):
//!
//! ```
//! use fruits_render_core::{StandardInstance, StandardVertex};
//!
//! let vertex_layout = StandardVertex::desc();
//! let instance_layout = StandardInstance::desc();
//!
//! assert_eq!(vertex_layout.step_mode, wgpu::VertexStepMode::Vertex);
//! assert_eq!(instance_layout.step_mode, wgpu::VertexStepMode::Instance);
//! ```
//!
//! # How to maintain
//!
//! The crate's public surface is **FFI-stable**, so the engine can call into a dynamically linked
//! render module across a fixed ABI. [`RenderApiResource`] is the `#[repr(C)]` ECS resource that
//! actually lives in the world: it holds a type-erased [`fruits_ffi::FfiDroppable`] (the real
//! [`RenderState`]) plus a static vtable of `extern "C"` functions. Its inherent methods
//! ([`resize`](RenderApiResource::resize), [`size`](RenderApiResource::size),
//! [`create_texture`](RenderApiResource::create_texture),
//! [`create_mesh`](RenderApiResource::create_mesh)) only marshal their arguments through that
//! vtable into the underlying `RenderState`. Code running in the same binary skips the marshalling
//! with the unsafe [`raw`](RenderApiResource::raw) accessor, which reinterprets the erased pointer
//! back to `&RenderState` — `fruits_render` uses this to reach the device, queue, and surface
//! directly when building pipelines.
//!
//! [`RenderState`] composes a private `RenderApi` (the wgpu `Device`, `Queue`, `Surface`,
//! `SurfaceConfiguration`, owning `Window`, and cached size) with [`RenderData`] (long-lived bind
//! group layouts). `RenderApi::new` performs all wgpu initialization on construction: it requests
//! a `PRIMARY`-backend instance, creates the surface from the window, requests an adapter
//! compatible with that surface, and blocks on the device and queue with `pollster`. It prefers an
//! sRGB surface format when one is offered and configures the surface for `RENDER_ATTACHMENT`
//! usage. ([`resize`](RenderState::resize) ignores zero-sized requests and reconfigures the
//! surface; [`surface_config_mut`](RenderState::surface_config_mut) is `unsafe` because mutating
//! the config without reconfiguring the surface desyncs it.) Two `todo`s in `render_api.rs` flag
//! that wgpu init may later move into an ECS `Start` handler and that the per-field accessors
//! should be replaced by exposing the `api` as a struct.
//!
//! [`RenderData`] currently holds a single layout, `bind_group_layout_standard_texture`: a
//! filterable 2-D texture at binding 0 and a filtering sampler at binding 1, both visible to the
//! vertex and fragment stages. Every [`StandardTexture`] builds its bind group against this layout,
//! which is why textures are created from a `RenderState` rather than a bare `Device`.
//!
//! The GPU asset types wrap their native handles behind the same FFI boundary. [`StandardMesh`]
//! and [`StandardTexture`] are `#[repr(transparent)]` over an `FfiDroppable`; their `Send`/`Sync`
//! are forwarded from the native payloads (`StandardMeshNative`, `StandardTextureNative`), and the
//! native handles are reached only through the unsafe `native()` accessors. `StandardTexture::new`
//! derives bytes-per-pixel from the data length over the pixel count, pads any source with fewer
//! than four channels out to RGBA with opaque alpha, uploads as `Rgba8UnormSrgb`, and builds a
//! `ClampToEdge` sampler that uses the caller's filter mode for magnification and mipmapping but
//! `Nearest` for minification.
//!
//! [`StandardVertex`] is marked [`AllBitVariationsValid`](fruits_utils::mem::AllBitVariationsValid)
//! and [`AllBitsInit`](fruits_utils::mem::AllBitsInit) so it can be reinterpreted as raw bytes
//! (via `fruits_utils::mem::as_bytes_slice`) when its buffer is uploaded. Its four attributes sit
//! at locations 0–3; the per-instance `local_to_world` matrix occupies locations 5–8 as four
//! `Float32x4` rows. [`Shader`] is a thin WGSL module wrapper that the pipeline path does not use
//! (`fruits_render` creates its shader modules itself) and is marked `// todo: remove`.

mod assets;
pub use assets::*;

mod render_api;
pub use render_api::*;

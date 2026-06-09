//! # fruits_render
//!
//! Draws the contents of the world to the screen — meshes with materials, UI text and
//! images, and debug gizmo lines — making up the engine's standard rendering subsystem.
//!
//! # How to use
//!
//! Rendering is enabled by registering the module, which the engine's default-modules setup
//! (`fruits_modules::add_defult_modules_to`) already does. Once
//! registered, an entity is drawn by giving it the right components; the systems in this
//! crate pick them up automatically each frame. Nothing is drawn until exactly one entity
//! carries a [`CameraComponent`] (world-space content needs the camera to be visible).
//!
//! #### Drawing a mesh
//!
//! Attach a [`StandardMeshComponent`] and a [`StandardMaterialComponent`] (plus a transform)
//! to an entity. Entities sharing the same mesh and material are drawn together:
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! ec.add_component(entity, StandardMeshComponent { mesh: mesh.clone() }).ok().unwrap();
//! ec.add_component(entity, StandardMaterialComponent { material: material.clone() }).ok().unwrap();
//! ec.add_component(
//!     entity,
//!     GlobalTransform { scale_rotation: Mat3::IDENTITY, position: Vec3::new(0.0, 0.0, 0.0) },
//! ).ok().unwrap();
//! ```
//!
//! #### Placing the camera
//!
//! Give one entity a [`CameraComponent`] and a transform. The camera's transform is the eye
//! position; its `fov` is in radians:
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! let camera = ec.create_entity();
//! ec.add_component(camera, GlobalTransform {
//!     scale_rotation: Mat3::IDENTITY,
//!     position: Vec3::new(0.0, 0.0, -5.0),
//! }).ok().unwrap();
//! ec.add_component(camera, CameraComponent {
//!     near: 0.1,
//!     far: 1_000.0,
//!     fov: 90_f32.to_radians(),
//! }).ok().unwrap();
//! ```
//!
//! #### Defining a material
//!
//! A [`StandardMaterial`] describes how a surface is shaded. [`alpha_threshold`](StandardMaterial::alpha_threshold)
//! decides the draw path: `Some(_)` is an opaque, alpha-tested surface, `None` is a blended
//! transparent one. [`space`](StandardMaterial::space) selects the coordinate space the mesh
//! is interpreted in ([`RenderSpace::World`], [`RenderSpace::Window`], or [`RenderSpace::Clip`]):
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! let material = StandardMaterial {
//!     space: RenderSpace::World,
//!     color: Vec4::new(1.0, 0.5, 0.2, 1.0),
//!     is_lit: true,
//!     alpha_threshold: Some(0.5),
//!     ..Default::default()
//! };
//!
//! let handle = world
//!     .resources_mut()
//!     .get_mut::<AssetStorageResource<StandardMaterial>>()
//!     .unwrap()
//!     .insert(material);
//! ```
//!
//! #### Drawing UI text and images
//!
//! [`TextComponent`] and [`ImageComponent`] render in screen space. Pair them with a
//! [`StandardMaterialComponent`] whose material uses [`RenderSpace::Window`]; the crate builds
//! the geometry from the component each frame, so no mesh is needed:
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! ec.add_component(entity, TextComponent {
//!     font: font.clone(),
//!     text: "score: 0".into(),
//!     font_size: UiVal::px(32.0),
//!     horizontal_align: HorizontalAlign::Left,
//!     vertical_align: VerticalAlign::Top,
//!     is_y_inverted: true,
//!     horizontal_spacing: UiVal::px(0.0),
//!     color: Vec4::splat(1.0),
//! }).ok().unwrap();
//! ```
//!
//! #### Drawing debug gizmo lines
//!
//! [`GizmosResource`] collects lines to draw for the current frame. Pick a space with
//! [`space`](GizmosResource::space) and push a [`GizmoLine`]; the lines are drawn and cleared
//! each frame, so push them every frame they should appear:
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! fn draw(mut gizmos: ResMut<GizmosResource>) {
//!     gizmos.space(RenderSpace::World).push(GizmoLine {
//!         start: Vec3::splat(0.0),
//!         end: Vec3::new(1.0, 0.0, 0.0),
//!         color: Vec4::new(1.0, 0.0, 0.0, 1.0),
//!     });
//! }
//! ```
//!
//! # How to maintain
//!
//! #### Registration and frame order
//!
//! [`add_render_module_to`] inserts the subsystem's resources and registers its systems under
//! the [`SYSTEM_GROUP_RENDER`] group. [`Schedule::Start`] builds
//! the long-lived GPU resources once ([`create_standard_render_resource`],
//! [`create_gizmos_render_resource`], and the depth/transparent targets). Each
//! [`Schedule::Update`] acquires the surface texture
//! ([`request_surface_texture`]), runs the inner `fruits_render_stuff` group, then presents it
//! ([`present_surface`]). Inside that group the explicit ordering is: rebuild render targets and
//! the camera uniform, build text/image/masked mesh data, clear the depth and transparent
//! targets, draw opaque geometry, draw transparent geometry, composite the transparent target,
//! and finally draw gizmos.
//!
//! #### Two geometry paths: instanced and batched
//!
//! Geometry reaches the GPU two ways. The **instanced** path ([`render_opaque_instanced`],
//! [`render_transparent_instanced`]) groups entities by shared
//! [`StandardMeshComponent`]/[`StandardMaterialComponent`] and issues one instanced draw per
//! group, with per-instance model matrices written into a reused instance buffer in chunks of
//! [`INSTANCES_PER_DRAW_MAX`]. The **batched** path ([`render_opaque_batched`],
//! [`render_transparent_batched`]) is for [`BatchedMeshComponent`]: it transforms each vertex on
//! the CPU into world space, appends into a shared vertex buffer grouped by material, and flushes
//! whenever the buffer fills ([`TRIANGLES_PER_BATCHED_DRAW_MAX`] triangles). Text and image
//! entities have no mesh asset — [`update_text_batched_mesh`] and [`update_image_batched_mesh`]
//! regenerate their [`BatchedMeshComponent`] vertices each frame, so they always take the batched
//! path. The instanced and batched render functions are near-duplicates (see the `todo` notes).
//!
//! #### Opaque vs. transparent compositing
//!
//! A material's [`alpha_threshold`](StandardMaterial::alpha_threshold) splits the two: `Some`
//! materials are drawn opaque (depth-write on, alpha-tested with `discard` in the shader); `None`
//! materials are transparent. Transparent draws target an offscreen `Rgba16Float`
//! [`TransparentTargetTextureResource`] with additive blending and depth-test but no depth-write,
//! and [`render_transparent_final`] then composites that target over the surface with a
//! fullscreen triangle (`shader_transparent_final.wgsl`) using alpha blending. The depth buffer is
//! [`DepthTextureResource`] (`Depth32Float`); both targets are recreated on resize by
//! [`recreate_depth_texture_resource`] and [`recreate_transparent_target_resource`], which compare
//! against the current surface size and only rebuild when it changed.
//!
//! #### Shaders and coordinate spaces
//!
//! The standard shader is generated as WGSL source at pipeline-creation time by
//! [`shader_standard`], which concatenates code fragments and branches on `is_lit`/`is_transparent`;
//! the lit path applies a Cook-Torrance BRDF with a single hard-coded directional light, the unlit
//! path skips lighting. Vertex colors are gamma-corrected (raised to 2.2). `get_render_data` picks
//! the world-to-clip matrix per draw from the material's [`RenderSpace`]: `World` uses the camera
//! matrix, `Window` uses [`create_window_to_clip_matrix`] (pixel coordinates with the near/far from
//! [`ScreenSpaceResource`]), and `Clip` uses the identity. [`update_camera_uniform`] builds the
//! camera matrix from the sole [`CameraComponent`] and **panics if more than one camera exists**.
//!
//! #### Masking and gizmos
//!
//! [`ChildrenRectMaskComponent`] marks a subtree to clip; [`update_masked_batched_mesh`] walks the
//! hierarchy depth-first, intersects nested mask rects, and clamps each descendant's batched-mesh
//! vertices into the resulting rect. This is a positional clamp, not true scissor masking (see the
//! `todo`). [`GizmosResource`] keeps a line list per [`RenderSpace`]; [`render_gizmos`] drains each
//! list in chunks of [`GIZMO_LINES_PER_DRAW_MAX`] through a line-list pipeline backed by storage
//! buffers, popping the lines as it consumes them — which is why gizmos must be re-pushed every
//! frame.
//!
//! #### Built-in assets and FFI
//!
//! On startup the crate uploads a 2×2 white fallback texture and three embedded ASCII monospace
//! bitmap fonts (5×7, 8×8, 8×12), keeping their handles in [`StandardRenderAssetsResource`]. Every
//! resource in this crate is annotated `todo: support ffi`; FFI exposure of the render resources is
//! not yet implemented.

mod assets;
mod components;
mod resources;
mod systems;
mod utils;

pub use self::{assets::*, components::*, resources::*, systems::*, utils::*};

use fruits_asset_storage::AssetStorageResource;
use fruits_ecs::{Schedule, WorldBuilderMut};
use fruits_render_core::{StandardMesh, StandardTexture};

pub const SYSTEM_GROUP_RENDER: &'static str = "fruits_render";

pub fn add_render_module_to(mut world: WorldBuilderMut) {
    world
        .data_mut()
        .resources_mut()
        .insert(SurfaceTextureResource { texture: None })
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(AssetStorageResource::<StandardMaterial>::new())
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(AssetStorageResource::<StandardMesh>::new())
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(AssetStorageResource::<StandardTexture>::new())
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(AssetStorageResource::<Font>::new())
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(GizmosResource::default())
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(ScreenSpaceResource::default())
        .ok()
        .unwrap();

    let mut world_behavior = world.behavior_mut();

    let mut start = world_behavior.get_mut(Schedule::Start);

    start
        .group(SYSTEM_GROUP_RENDER)
        .insert_child_system(create_standard_render_resource)
        .insert_child_system(recreate_depth_texture_resource)
        .insert_child_system(recreate_transparent_target_resource)
        .insert_child_system(create_gizmos_render_resource);

    start
        .order_system(recreate_depth_texture_resource)
        .before_system(create_standard_render_resource);
    start
        .order_system(recreate_transparent_target_resource)
        .before_system(create_standard_render_resource);

    let mut update = world_behavior.get_mut(Schedule::Update);

    update
        .group(SYSTEM_GROUP_RENDER)
        .insert_child_system(request_surface_texture)
        .insert_child_group("fruits_render_stuff")
        .insert_child_system(present_surface);

    update
        .group("fruits_render_stuff")
        .insert_child_system(update_camera_uniform)
        .insert_child_system(update_text_batched_mesh)
        .insert_child_system(update_image_batched_mesh)
        .insert_child_system(update_masked_batched_mesh)
        .insert_child_system(recreate_depth_texture_resource)
        .insert_child_system(recreate_transparent_target_resource)
        .insert_child_system(clear_depth)
        .insert_child_system(clear_transparent_target)
        .insert_child_system(render_opaque_instanced)
        .insert_child_system(render_opaque_batched)
        .insert_child_system(render_transparent_instanced)
        .insert_child_system(render_transparent_batched)
        .insert_child_system(render_transparent_final)
        .insert_child_system(render_gizmos);

    update
        .order_system(recreate_depth_texture_resource)
        .before_system(recreate_transparent_target_resource)
        .before_system(clear_depth)
        .before_system(clear_transparent_target)
        .before_system(update_camera_uniform)
        .before_system(render_opaque_instanced)
        .before_system(render_opaque_batched)
        .before_system(render_transparent_instanced)
        .before_system(render_transparent_batched)
        .before_system(render_transparent_final)
        .before_system(render_gizmos);

    update
        .order_system(update_text_batched_mesh)
        .before_system(update_masked_batched_mesh);
    update
        .order_system(update_image_batched_mesh)
        .before_system(update_masked_batched_mesh);
    update
        .order_system(update_masked_batched_mesh)
        .before_system(render_opaque_batched);
    update
        .order_system(update_masked_batched_mesh)
        .before_system(render_transparent_batched);

    update
        .order_system(request_surface_texture)
        .before_group("fruits_render_stuff");
    update.order_group("fruits_render_stuff").before_system(present_surface);
}

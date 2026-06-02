//! # fruits_collision
//!
//! Collision shapes and broad-phase overlap queries for the Fruits engine.
//!
//! # How to use
//!
//! The collision subsystem is normally pulled into a world through the engine's default
//! modules, not by registering this crate by hand:
//!
//! ```ignore
//! use fruits_engine::*;
//!
//! let mut app = App::new();
//! add_defult_modules_to(app.ecs_mut().as_mut());
//! ```
//!
//! Make an entity collidable by attaching a [`ColliderComponent`]. The shape is given in the
//! entity's local space; each frame it is transformed by the entity's `GlobalTransform` and
//! re-indexed:
//!
//! ```ignore
//! ec.add_component(entity, ColliderComponent {
//!     shape: CollisionShape::Sphere(CollisionSphere { center: Vec3::splat(0.0), radius: 1.0 }),
//! }).ok().unwrap();
//! ```
//!
//! Query the world from a system by reading [`CollisionWorldResource`] and probing it with a
//! shape (here a mouse ray). [`overlaps`](CollisionWorldResource::overlaps) collects the
//! entities whose colliders the probe touches:
//!
//! ```ignore
//! fn pick(world: Res<CollisionWorldResource>) {
//!     let ray = CollisionLine { start, end, bounds: LineBoundType::UNRESTRICTED };
//!     let mut hits = Vec::new();
//!     world.overlaps(ray.into(), &mut hits);
//!     // `hits` now lists every entity under the ray
//! }
//! ```
//!
//! For a one-off geometric test without the ECS, call [`overlaps`] on two shapes directly:
//!
//! ```
//! use fruits_collision::{CollisionAabb, CollisionShape, overlaps};
//! use fruits_math::Vec3;
//!
//! let a = CollisionAabb { center: Vec3::splat(0.0), extents: Vec3::splat(1.0) };
//! let b = CollisionAabb { center: Vec3::new(1.5, 0.0, 0.0), extents: Vec3::splat(1.0) };
//! assert!(overlaps(a.into_shape(), b.into_shape()));
//! ```
//!
//! # How to maintain
//!
//! The crate has two layers.
//!
//! **Geometry** — [`overlaps`] and the [`CollisionShape`] variants. `overlaps` matches on the
//! ordered variant pair and forwards to a private primitive routine. Each *unordered* pair has
//! a single implementation that the swapped pair reuses by exchanging its arguments (e.g.
//! AABB-vs-box and box-vs-AABB call the same function). Oriented-box, triangle, and several
//! other tests are first moved into an origin-centered AABB's local frame, so one
//! separating-axis routine can back multiple shape pairs.
//!
//! **Broad phase** — [`CollisionWorldResource`], backed by [`Bvh`]. The BVH is a binary tree,
//! median-split on a cycling X/Y/Z axis chosen by node depth, and is **rebuilt from scratch
//! every frame** by the [`update_collision_world`] system (there is no incremental update — if
//! rebuild cost ever matters, that is the place to change). A query prunes by node AABB before
//! testing the exact leaf shapes.
//!
//! Caveats before changing anything:
//! - [`CollisionShape::to_aabb`] **panics** for a non-segment [`CollisionLine`]: an unbounded
//!   line has no finite box, so nothing that feeds the BVH may contain infinite lines.
//! - [`CollisionShape::apply_matrix_lossy`] is deliberately lossy — an AABB keeps its extents
//!   (only its center moves) and a sphere scales its radius by the *average* lossy scale.
//! - `shapes_collision.rs` (contact points: normal/penetration/point) is unfinished and is not
//!   yet part of the module tree.

mod components;
mod line_bound_type;
mod resources;
mod shapes;
mod shapes_overlap;
mod systems;
mod utils;
// mod shapes_collision;

pub use line_bound_type::*;
pub use shapes::*;
pub use shapes_overlap::*;
// todo
// pub use shapes_collision::*;
pub use components::*;
pub use resources::*;
pub use systems::*;
pub use utils::*;

use fruits_ecs::{Schedule, WorldBuilderMut};

/// System-group name for the collision systems; order your own systems against it via the
/// scheduler.
pub const SYSTEM_GROUP_COLLISION: &'static str = "fruits_collision";

/// Registers the collision subsystem into `world`: inserts a [`CollisionWorldResource`] and
/// schedules [`update_collision_world`] in [`Schedule::Update`].
///
/// Most users do not call this directly — it is pulled in by the engine's default-module
/// registration.
///
/// # Panics
///
/// Panics if a [`CollisionWorldResource`] was already inserted into `world`.
pub fn add_collision_module_to(mut world: WorldBuilderMut) {
    world
        .data_mut()
        .resources_mut()
        .insert(CollisionWorldResource::default())
        .ok()
        .unwrap();

    world
        .behavior_mut()
        .get_mut(Schedule::Update)
        .group(SYSTEM_GROUP_COLLISION)
        .insert_child_system(update_collision_world);
}

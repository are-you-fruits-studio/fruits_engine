//! # fruits_collision
//!
//! Collision shapes and broad-phase queries for the Fruits engine.
//!
//! The crate provides:
//! - [`CollisionShape`] and its concrete variants ([`CollisionAabb`], [`CollisionBox`],
//!   [`CollisionSphere`], [`CollisionLine`], and point/triangle shapes).
//! - [`overlaps`] for boolean intersection tests between any pair of shapes.
//! - [`CollisionWorldResource`], an ECS resource that indexes collider entities in a
//!   bounding-volume hierarchy ([`Bvh`]) for fast overlap queries.
//! - [`ColliderComponent`], the component that attaches a shape to an entity.
//!
//! Register the subsystem into a world with [`add_collision_module_to`]. It inserts the
//! collision world resource and schedules [`update_collision_world`] in [`Schedule::Update`].

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

/// Name of the system group that holds the collision systems in [`Schedule::Update`].
pub const SYSTEM_GROUP_COLLISION: &'static str = "fruits_collision";

/// Registers the collision subsystem into `world`.
///
/// Inserts a default [`CollisionWorldResource`] and schedules
/// [`update_collision_world`] inside the [`SYSTEM_GROUP_COLLISION`] group of
/// [`Schedule::Update`].
///
/// # Panics
///
/// Panics if a [`CollisionWorldResource`] has already been inserted into `world`.
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

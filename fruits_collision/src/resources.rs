use fruits_ecs::{EntityId, Resource};

use crate::*;

/// ECS resource holding the broad-phase collision index for the world.
///
/// Wraps a bounding-volume hierarchy ([`Bvh`]) keyed by [`Entity`], rebuilt every frame by
/// [`update_collision_world`](crate::update_collision_world). Query it with [`overlaps`](Self::overlaps).
#[repr(transparent)]
#[derive(Resource, Default)]
pub struct CollisionWorldResource {
    // todo: support full CollisionShape.
    //collision_shapes: Vec<(CollisionAabb, Entity)>,
    collision_shapes: Bvh<EntityId>,
}

impl CollisionWorldResource {
    pub fn new(values: impl Iterator<Item = (CollisionShape, EntityId)>) -> Self {
        Self {
            //collision_shapes: values,
            collision_shapes: Bvh::new(values),
        }
    }

    /// Returns `true` if no colliders are indexed.
    pub fn is_empty(&self) -> bool {
        self.collision_shapes.is_empty()
    }

    // pub fn refill(&mut self) -> &mut Vec<(CollisionAabb, Entity)> {
    //     self.collision_shapes.clear();

    //     &mut self.collision_shapes
    // }

    pub fn overlaps(&self, query: CollisionShape, results: &mut Vec<EntityId>) {
        // for &(shape, entity) in &self.collision_shapes {
        //     if overlaps(shape.into(), query) {
        //         results.push(entity);
        //     }
        // }
        self.collision_shapes.query(query, results);
    }
}

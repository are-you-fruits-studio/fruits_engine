use fruits_ecs::{Entity, Resource};

use crate::collision::*;

#[repr(transparent)]
#[derive(Resource, Default)]
pub struct CollisionWorldResource {
    // todo: support full CollisionShape.
    //collision_shapes: Vec<(CollisionAabb, Entity)>,
    collision_shapes: Bvh<Entity>,
}

impl CollisionWorldResource {
    pub fn new(values: Vec<(CollisionShape, Entity)>) -> Self {
        Self {
            //collision_shapes: values,
            collision_shapes: Bvh::new(values)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.collision_shapes.is_empty()
    }

    // pub fn refill(&mut self) -> &mut Vec<(CollisionAabb, Entity)> {
    //     self.collision_shapes.clear();

    //     &mut self.collision_shapes
    // }

    pub fn overlaps(&self, query: CollisionShape, results: &mut Vec<Entity>) {
        // for &(shape, entity) in &self.collision_shapes {
        //     if overlaps(shape.into(), query) {
        //         results.push(entity);
        //     }
        // }
        self.collision_shapes.query(query, results);
    }
}
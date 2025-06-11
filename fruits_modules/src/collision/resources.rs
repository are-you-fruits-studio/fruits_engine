use fruits_ecs::{Entity, Resource};

use crate::collision::*;

#[derive(Resource, Default)]
pub struct CollisionWorldResource {
    pub(crate) collision_shapes: Vec<(CollisionShape, Entity)>,
}

impl CollisionWorldResource {
    pub fn raycast(&self, line: CollisionLine) -> Entity {
        for (shape, entity) in &self.collision_shapes {
            if overlaps(*shape, line.into()) {
                return *entity;
            }
        }

        Entity::EMPTY
    }
}
use fruits_ecs::*;

use crate::collision::{ColliderComponent, CollisionWorldResource};

pub fn update_collision_world(
    mut collision_world: ResMut<CollisionWorldResource>,
    q: WorldQuery<(Entity, &ColliderComponent)>
) {
    let iter = q.iter().map(|(e, c)| (c.shape.to_aab(), e));

    *collision_world = CollisionWorldResource::new(iter.collect::<Vec<_>>());
    // collision_world.refill().extend(iter);
}
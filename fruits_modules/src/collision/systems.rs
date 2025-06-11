use fruits_ecs::*;

use crate::collision::{ColliderComponent, CollisionWorldResource};

pub fn update_collision_world(
    mut collision_world: ResMut<CollisionWorldResource>,
    q: WorldQuery<(Entity, &ColliderComponent)>
) {
    collision_world.collision_shapes.clear();
    collision_world.collision_shapes.extend(q.iter().map(|(e, c)| (c.shape, e)));
}
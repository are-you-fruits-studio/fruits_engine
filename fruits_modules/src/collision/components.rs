use fruits_ecs::Component;

use crate::collision::CollisionShape;

// todo: support ffi
#[derive(Component)]
pub struct ColliderComponent {
    pub shape: CollisionShape,
}
use fruits_ecs::Component;

use crate::collision::CollisionShape;

#[repr(C)]
#[derive(Component)]
pub struct ColliderComponent {
    pub shape: CollisionShape,
}
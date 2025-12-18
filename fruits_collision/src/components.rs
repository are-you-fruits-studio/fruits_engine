use fruits_ecs::Component;

use crate::CollisionShape;

#[repr(C)]
#[derive(Component)]
pub struct ColliderComponent {
    pub shape: CollisionShape,
}

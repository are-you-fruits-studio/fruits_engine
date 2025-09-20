use fruits_ecs::Component;

use crate::collision::CollisionShapeFfi;

#[repr(C)]
#[derive(Component)]
pub struct ColliderComponent {
    pub shape: CollisionShapeFfi,
}
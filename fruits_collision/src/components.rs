use fruits_ecs::Component;

use crate::CollisionShape;

/// A collision shape attached to an entity.
///
/// NOTE: the shape is interpreted in the entity's local space.
#[repr(C)]
#[derive(Component)]
pub struct ColliderComponent {
    pub shape: CollisionShape,
}

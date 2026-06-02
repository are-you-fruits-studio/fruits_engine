use fruits_ecs::Component;

use crate::CollisionShape;

/// Attaches a [`CollisionShape`] to an entity. The shape is interpreted in the entity's
/// local space.
#[repr(C)]
#[derive(Component)]
pub struct ColliderComponent {
    pub shape: CollisionShape,
}

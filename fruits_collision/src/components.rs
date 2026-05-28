use fruits_ecs::Component;

use crate::CollisionShape;

/// Component that gives an entity a collision shape.
///
/// [`update_collision_world`](crate::update_collision_world) reads this component,
/// applies the entity's global transform (when present), and inserts the resulting
/// world-space shape into the [`CollisionWorldResource`](crate::CollisionWorldResource).
#[repr(C)]
#[derive(Component)]
pub struct ColliderComponent {
    /// The collider's shape, expressed in the entity's local space.
    pub shape: CollisionShape,
}

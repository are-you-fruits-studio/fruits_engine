use fruits_ecs::Component;

use crate::collision::CollisionShape;

#[derive(Component)]
pub struct ColliderComponent {
    pub shape: CollisionShape,
}
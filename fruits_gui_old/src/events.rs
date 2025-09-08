use fruits_engine::ecs::{Entity, Event};

#[derive(Event)]
pub struct HierarchyUpdateEvent {
    pub entities: Vec<Entity>,
}
use fruits_prelude::{Entity, Event};

#[derive(Event)]
pub struct HierarchyUpdateEvent {
    pub entities: Vec<Entity>,
}
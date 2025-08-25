use crate::{Component, EntitiesComponentsHolder, Entity};

pub struct EntityBuilder<'ec> {
    ec: &'ec mut EntitiesComponentsHolder,
    queue: Vec<Box<dyn FnOnce(Entity, &mut EntitiesComponentsHolder)>>,
}

impl<'ec> EntityBuilder<'ec> {
    #[must_use]
    fn new(ec: &'ec mut EntitiesComponentsHolder) -> Self {
        Self {
            ec,
            queue: Vec::new(),
        }
    }

    pub fn build(mut self) -> Entity {
        let entity = self.ec.create_entity();

        for action in std::mem::replace(&mut self.queue, Vec::new()) {
            action(entity, self.ec);
        }

        entity
    }

    #[must_use]
    pub fn enqueue_raw(mut self, action: Box<dyn FnOnce(Entity, &mut EntitiesComponentsHolder)>) -> Self {
        self.queue.push(action);
        self
    }

    #[must_use]
    pub fn add_component<C: Component>(self, c: C) -> Self {
        self.enqueue_raw(Box::new(move |e, ec| ec.add_component(e, c).ok().unwrap()))
    }
}

pub trait EntitiesComponentsHolderEntityBuilderExt {
    #[must_use]
    fn create_entity_builder<'ec>(&'ec mut self) -> EntityBuilder<'ec>;
}

impl EntitiesComponentsHolderEntityBuilderExt for EntitiesComponentsHolder {
    fn create_entity_builder<'ec>(&'ec mut self) -> EntityBuilder<'ec> {
        EntityBuilder::new(self)
    }
}
pub use crate::*;

pub struct EntityComponentsBuilder<'ec> {
    pub(crate) ec: EntitiesHolderMut<'ec>,
    pub(crate) ent: EntityId,
}

impl<'ec> EntityComponentsBuilder<'ec> {
    pub fn new(ec: EntitiesHolderMut<'ec>, ent: EntityId) -> Self {
        Self { ec, ent }
    }

    pub fn add_component<C: Component>(&mut self, component: C) -> &mut Self {
        self.ec.add_component(self.ent, component).ok().unwrap();
        self
    }
}

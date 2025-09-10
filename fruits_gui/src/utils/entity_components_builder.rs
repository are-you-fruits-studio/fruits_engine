pub use crate::*;

pub struct EntityComponentsBuilder<'ec> {
    pub(crate) ec: &'ec mut EntitiesComponentsHolder,
    pub(crate) ent: Entity,
}

impl<'ec> EntityComponentsBuilder<'ec> {
    pub fn new(ec: &'ec mut EntitiesComponentsHolder, ent: Entity) -> Self {
        Self {
            ec,
            ent,
        }
    }

    pub fn add_component<C: Component>(&mut self, component: C) -> &mut Self {
        self.ec.add_component(self.ent, component).ok().unwrap();
        self
    }
}

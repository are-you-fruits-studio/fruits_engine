pub use crate::*;

pub struct EntityComponentsBuilder<'ec> {
    pub(crate) ec: EntitiesHolderMut<'ec>,
    pub(crate) ent: Entity,
}

impl<'ec> EntityComponentsBuilder<'ec> {
    pub fn new(ec: EntitiesHolderMut<'ec>, ent: Entity) -> Self {
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

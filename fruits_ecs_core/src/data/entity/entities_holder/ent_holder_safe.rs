use crate::*;

#[repr(transparent)]
pub struct EntitiesHolderMut<'e> {
    data: EntitiesHolderUnsafeRef<'e>,
}

impl<'e> EntitiesHolderMut<'e> {
    pub fn query<'a, A: ArchetypeIteratorItem>(&'a self) -> EntitiesHolderQuery<'a, A::Item<'a>> {
        // Safety. Managed with lifetimes.
        unsafe { self.data.query::<'a, A>() }
    }

    pub fn query_filtered<'a, A: ArchetypeIteratorItem, F: QueryFilter>(&'a self) -> EntitiesHolderQuery<'a, A::Item<'a>, F> {
        // Safety. Managed with lifetimes.
        unsafe { self.data.query::<'a, A, F>() }
    }

    pub fn query_mut<'a, A: ArchetypeIteratorItem>(&'a mut self) -> EntitiesHolderQuery<'a, A::ReadOnlyItem<'a>> {
        // Safety. Managed with lifetimes.
        unsafe { self.data.query::<'a, A>() }
    }

    pub fn query_filtered_mut<'a, A: ArchetypeIteratorItem, F: QueryFilter>(&'a mut self) -> EntitiesHolderQuery<'a, A::ReadOnlyItem<'a>, F> {
        // Safety. Managed with lifetimes.
        unsafe { self.data.query::<'a, A, F>() }
    }

    pub fn entities_count(&self) -> u64 {
        self.data.entities_count()
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.data.contains_entity(entity)
    }

    pub fn create_entity(&mut self) -> Entity {
        self.data.create_entity()
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        self.data.destroy_entity(entity)
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<(), C> {
        self.data.add_component(entity, component)
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        self.data.remove_component(entity)
    }

    pub fn get_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        // Safety. Managed with lifetimes.
        unsafe { self.data.get_component_ptr(entity).map(|p| &*p) }
    }

    pub fn get_component_mut<C: Component>(&mut self, entity: Entity) -> Option<&mut C> {
        // Safety. Managed with lifetimes.
        unsafe { self.data.get_component_ptr(entity).map(|p| &mut *p) }
    }

    pub fn as_mut<'r>(&'r mut self) -> EntitiesHolderMut<'r>
        where 'e: 'r
    {
        unsafe { EntitiesHolderUnsafeRef::from_safe_mut(self).as_mut().into_safe() }
    }

    pub fn as_ref<'r>(&'r self) -> EntitiesHolderRef<'r>
        where 'e: 'r
    {
        unsafe { EntitiesHolderUnsafeRef::from_safe_ref(self).as_ref().into_safe() }
    }
}

//

#[repr(transparent)]
pub struct EntitiesHolderRef<'e> {
    data: EntitiesHolderUnsafeRef<'e>,
}

impl<'e> EntitiesHolderRef<'e> {
    pub fn query<'a, A: ArchetypeIteratorItem>(&'a self) -> EntitiesHolderQuery<'a, A::Item<'a>> {
        // Safety. Managed with lifetimes.
        unsafe {
            EntitiesHolderQuery::new(self.data.as_ref())
        }
    }

    pub fn query_filtered<'a, A: ArchetypeIteratorItem, F: QueryFilter>(&'a self) -> EntitiesHolderQuery<'a, A::Item<'a>, F> {
        // Safety. Managed with lifetimes.
        unsafe {
            EntitiesHolderQuery::new(self.data.as_ref())
        }
    }

    pub fn entities_count(&self) -> u64 {
        self.data.entities_count()
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.data.contains_entity(entity)
    }

    pub fn get_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        // Safety. Managed with lifetimes.
        unsafe { self.data.get_component_ptr(entity).map(|p| &*p) }
    }

    pub fn as_ref<'r>(&'r self) -> EntitiesHolderRef<'r>
        where 'e: 'r
    {
        unsafe { EntitiesHolderUnsafeRef::from_safe_ref(self).as_ref().into_safe() }
    }
}

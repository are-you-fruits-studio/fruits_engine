use crate::*;

#[repr(transparent)]
pub struct EntitiesHolderMut<'e> {
    data: EntitiesHolderUnsafeRef<'e>,
}

impl<'e> EntitiesHolderMut<'e> {
    pub fn new(data: EntitiesHolderUnsafeRef<'e>) -> Self {
        Self {
            data,
        }
    }

    pub fn query<'r, A: ArchetypeIteratorItem>(&'r self) -> EntitiesHolderQuery<'r, A::ReadOnlyItem<'r>>
        where 'e: 'r
    {
        // Safety. Managed with lifetimes.
        unsafe { self.data.clone().into_query::<A::ReadOnlyItem<'r>, ()>() }
    }

    pub fn query_filtered<'r, A: ArchetypeIteratorItem, F: QueryFilter>(&'r self) -> EntitiesHolderQuery<'r, A::ReadOnlyItem<'r>, F>
        where 'e: 'r
    {
        // Safety. Managed with lifetimes.
        unsafe { self.data.clone().into_query::<A::ReadOnlyItem<'r>, F>() }
    }

    pub fn query_mut<'r, A: ArchetypeIteratorItem>(&'r mut self) -> EntitiesHolderQuery<'r, A::Item<'r>>
        where 'e: 'r
    {
        // Safety. Managed with lifetimes.
        unsafe { self.data.clone().into_query::<A::Item<'r>, ()>() }
    }

    pub fn query_filtered_mut<'r, A: ArchetypeIteratorItem, F: QueryFilter>(&'r mut self) -> EntitiesHolderQuery<'r, A::Item<'r>, F>
        where 'e: 'r
    {
        // Safety. Managed with lifetimes.
        unsafe { self.data.clone().into_query::<A::Item<'r>, F>() }
    }

    pub fn entities_count(&self) -> u64 {
        unsafe { self.data.entities_count() }
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        unsafe { self.data.contains_entity(entity) }
    }

    pub fn create_entity(&mut self) -> Entity {
        unsafe { self.data.create_entity() }
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        unsafe { self.data.destroy_entity(entity) }
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<(), C> {
        unsafe { self.data.add_component(entity, component) }
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        unsafe { self.data.remove_component(entity) }
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
        EntitiesHolderMut {
            data: self.data.clone(),
        }
    }

    pub fn as_ref<'r>(&'r self) -> EntitiesHolderRef<'r>
        where 'e: 'r
    {
        EntitiesHolderRef {
            data: self.data.clone(),
        }
    }
}

//

#[repr(transparent)]
pub struct EntitiesHolderRef<'e> {
    data: EntitiesHolderUnsafeRef<'e>,
}

impl<'e> EntitiesHolderRef<'e> {
    pub fn new(data: EntitiesHolderUnsafeRef<'e>) -> Self {
        Self {
            data,
        }
    }

    pub fn query<'r, A: ArchetypeIteratorItem>(&'r self) -> EntitiesHolderQuery<'r, A::ReadOnlyItem<'r>>
        where 'e: 'r
    {
        // Safety. Managed with lifetimes.
        unsafe { self.data.clone().into_query::<A::ReadOnlyItem<'r>, ()>() }
    }

    pub fn query_filtered<'r, A: ArchetypeIteratorItem, F: QueryFilter>(&'r self) -> EntitiesHolderQuery<'r, A::ReadOnlyItem<'r>, F>
        where 'e: 'r
    {
        // Safety. Managed with lifetimes.
        unsafe { self.data.clone().into_query::<A::ReadOnlyItem<'r>, F>() }
    }

    pub fn entities_count(&self) -> u64 {
        unsafe { self.data.entities_count() }
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        unsafe { self.data.contains_entity(entity) }
    }

    pub fn get_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        // Safety. Managed with lifetimes.
        unsafe { self.data.get_component_ptr(entity).map(|p| &*p) }
    }

    pub fn as_ref<'r>(&'r self) -> EntitiesHolderRef<'r>
        where 'e: 'r
    {
        EntitiesHolderRef {
            data: self.data.clone(),
        }
    }
}

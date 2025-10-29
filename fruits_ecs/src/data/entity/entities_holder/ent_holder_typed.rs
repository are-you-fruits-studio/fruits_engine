use std::mem::MaybeUninit;

use crate::*;

fn entities_holder_query<'e, A: ArchetypeIteratorItem, F: QueryFilter>(entities: &'e EntitiesHolderUnsafeFfi, types: &'e TypesRegistryCache) -> EntitiesHolderQuery<'e, A, F> {
    unsafe { EntitiesHolderQuery::<'e, A, F>::new(entities, types) }
}

fn entities_holder_add_component<C: 'static>(entities: &mut EntitiesHolderUnsafeFfi, types: &TypesRegistryCache, entity: Entity, component: C) -> Result<(), C> {
    unsafe {
        let mut component = MaybeUninit::new(component);

        if entities.add_component(
            entity,
            component.as_mut_ptr() as *mut u8,
            types.get_or_register::<C>(),
        ) {
            Ok(())
        } else {
            Err(component.assume_init())
        }
    }
}

fn entities_holder_remove_component<C: 'static>(entities: &mut EntitiesHolderUnsafeFfi, types: &TypesRegistryCache, entity: Entity) -> Option<C> {
    unsafe {
        let mut component = MaybeUninit::<C>::uninit();

        if entities.remove_component(
            entity,
            component.as_mut_ptr() as *mut u8,
            types.get_or_register::<C>(),
        ) {
            Some(component.assume_init())
        } else {
            None
        }
    }
}

fn entities_holder_get_component_ptr<C: 'static>(entities: &EntitiesHolderUnsafeFfi, types: &TypesRegistryCache, entity: Entity) -> Option<*mut C> {
    entities.get_component_ptr(entity, types.get_or_register::<C>()).map(|p| p as *mut C)
}

fn entities_holder_get_component<'r, C: 'static>(entities: &'r EntitiesHolderUnsafeFfi, types: &TypesRegistryCache, entity: Entity) -> Option<&'r C> {
    unsafe { entities_holder_get_component_ptr::<C>(entities, types, entity).map(|p| &*p) }
}

fn entities_holder_get_component_mut<'r, C: 'static>(entities: &'r mut EntitiesHolderUnsafeFfi, types: &TypesRegistryCache, entity: Entity) -> Option<&'r mut C> {
    unsafe { entities_holder_get_component_ptr::<C>(entities, types, entity).map(|p| &mut *p) }
}

//

pub struct EntitiesHolderMut<'e> {
    entities: &'e mut EntitiesHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
}

impl<'e> EntitiesHolderMut<'e> {
    pub fn new(entities: &'e mut EntitiesHolderUnsafeFfi, types: &'e TypesRegistryCache) -> Self {
        Self {
            entities,
            types,
        }
    }

    pub fn query<'r, A: ArchetypeIteratorItem>(&'r self) -> EntitiesHolderQuery<'r, A::ReadOnlyItem<'r>>
        where 'e: 'r
    {
        entities_holder_query::<A::ReadOnlyItem<'r>, ()>(self.entities, self.types)
    }

    pub fn query_filtered<'r, A: ArchetypeIteratorItem, F: QueryFilter>(&'r self) -> EntitiesHolderQuery<'r, A::ReadOnlyItem<'r>, F>
        where 'e: 'r
    {
        entities_holder_query::<A::ReadOnlyItem<'r>, F>(self.entities, self.types)
    }

    pub fn query_mut<'r, A: ArchetypeIteratorItem>(&'r mut self) -> EntitiesHolderQuery<'r, A::Item<'r>>
        where 'e: 'r
    {
        entities_holder_query::<A::Item<'r>, ()>(self.entities, self.types)
    }

    pub fn query_filtered_mut<'r, A: ArchetypeIteratorItem, F: QueryFilter>(&'r mut self) -> EntitiesHolderQuery<'r, A::Item<'r>, F>
        where 'e: 'r
    {
        entities_holder_query::<A::Item<'r>, F>(self.entities, self.types)
    }

    pub fn entities_count(&self) -> u64 {
        self.entities.entities_count()
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entities.contains_entity(entity)
    }

    pub fn create_entity(&mut self) -> Entity {
        self.entities.create_entity()
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        self.entities.destroy_entity(entity)
    }

    pub fn add_component<C: 'static>(&mut self, entity: Entity, component: C) -> Result<(), C> {
        entities_holder_add_component(self.entities, self.types, entity, component)
    }

    pub fn remove_component<C: 'static>(&mut self, entity: Entity) -> Option<C> {
        entities_holder_remove_component(self.entities, self.types, entity)
    }

    pub fn get_component_ptr<C: 'static>(&self, entity: Entity) -> Option<*mut C> {
        entities_holder_get_component_ptr(self.entities, self.types, entity)
    }

    pub fn get_component<'r, C: 'static>(&'r self, entity: Entity) -> Option<&'r C>
        where 'e: 'r
    {
        entities_holder_get_component(self.entities, self.types, entity)
    }

    pub fn get_component_mut<'r, C: 'static>(&'r mut self, entity: Entity) -> Option<&'r mut C>
        where 'e: 'r
    {
        entities_holder_get_component_mut(self.entities, self.types, entity)
    }

    pub fn as_mut<'r>(&'r mut self) -> EntitiesHolderMut<'r>
        where 'e: 'r
    {
        EntitiesHolderMut {
            entities: self.entities,
            types: self.types,
        }
    }

    pub fn as_ref<'r>(&'r self) -> EntitiesHolderRef<'r>
        where 'e: 'r
    {
        EntitiesHolderRef {
            entities: self.entities,
            types: self.types,
        }
    }
}

//

#[derive(Copy, Clone)]
pub struct EntitiesHolderRef<'e> {
    entities: &'e EntitiesHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
}

impl<'e> EntitiesHolderRef<'e> {
    pub fn new(entities: &'e EntitiesHolderUnsafeFfi, types: &'e TypesRegistryCache) -> Self {
        Self {
            entities,
            types,
        }
    }

    pub unsafe fn unsafe_query<A: ArchetypeIteratorItem, F: QueryFilter>(self) -> EntitiesHolderQuery<'e, A, F> {
        entities_holder_query::<A, F>(self.entities, self.types)
    }

    pub fn query<A: ArchetypeIteratorItem>(self) -> EntitiesHolderQuery<'e, A::ReadOnlyItem<'e>> {
        entities_holder_query::<A::ReadOnlyItem<'e>, ()>(self.entities, self.types)
    }

    pub fn query_filtered<A: ArchetypeIteratorItem, F: QueryFilter>(self) -> EntitiesHolderQuery<'e, A::ReadOnlyItem<'e>, F> {
        entities_holder_query::<A::ReadOnlyItem<'e>, F>(self.entities, self.types)
    }

    pub fn entities_count(self) -> u64 {
        self.entities.entities_count()
    }

    pub fn contains_entity(self, entity: Entity) -> bool {
        self.entities.contains_entity(entity)
    }

    pub fn get_component_ptr<C: 'static>(self, entity: Entity) -> Option<*mut C> {
        entities_holder_get_component_ptr(self.entities, self.types, entity)
    }

    pub fn get_component<C: 'static>(self, entity: Entity) -> Option<&'e C> {
        entities_holder_get_component(self.entities, self.types, entity)
    }
}

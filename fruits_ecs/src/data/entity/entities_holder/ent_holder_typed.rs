use std::{ffi::c_void, mem::MaybeUninit};

use fruits_ffi::{FfiAny, FfiAnyMut, FfiAnyRef, FfiExtendedTypeInfo, FfiFnMutMut, FfiOpaqueMut, FfiOpaqueRef};

use crate::*;

fn entities_holder_query<'e, A: ArchetypeIteratorItem, F: QueryFilter>(
    entities: &'e EntitiesHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
) -> EntitiesHolderQuery<'e, A, F> {
    unsafe { EntitiesHolderQuery::<'e, A, F>::new(entities, types) }
}

fn entities_holder_add_component<C: 'static>(
    entities: &mut EntitiesHolderUnsafeFfi,
    types: &TypesRegistryCache,
    entity: EntityId,
    component: C,
) -> Result<(), C> {
    unsafe {
        let mut component = MaybeUninit::new(component);

        if entities.add_component(entity, component.as_mut_ptr() as *mut u8, types.get_or_register::<C>()) {
            Ok(())
        } else {
            Err(component.assume_init())
        }
    }
}

fn entities_holder_add_component_any(
    entities: &mut EntitiesHolderUnsafeFfi,
    types: &TypesRegistryCache,
    entity: EntityId,
    component: FfiAny,
) -> Result<(), FfiAny> {
    unsafe {
        let types_registry = types.registry();

        let type_id = if let Some(type_info) = types_registry.get_by_name(component.type_info().short().name()) {
            type_info.id
        } else {
            types_registry.try_register(component.type_info()).unwrap()
        };

        let mut component = MaybeUninit::new(component);

        let component_ptr = { (&*component.as_mut_ptr()).ptr() };

        if entities.add_component(entity, component_ptr as *mut u8, type_id) {
            Ok(())
        } else {
            Err(component.assume_init())
        }
    }
}

fn entities_holder_remove_component<C: 'static>(
    entities: &mut EntitiesHolderUnsafeFfi,
    types: &TypesRegistryCache,
    entity: EntityId,
) -> Option<C> {
    unsafe {
        let mut component = Option::<C>::None;

        entities.remove_component(
            entity,
            |ptr, _info| component = Some((ptr as *const C).read()),
            types.get_or_register::<C>(),
        );

        component
    }
}

fn entities_holder_remove_component_any(
    entities: &mut EntitiesHolderUnsafeFfi,
    types: &TypesRegistryCache,
    entity: EntityId,
    type_name: &str,
) -> Option<FfiAny> {
    unsafe {
        let mut component = Option::<FfiAny>::None;

        entities.remove_component(
            entity,
            |ptr, info| component = Some(info.move_to_any(ptr as *const c_void)),
            types.registry().get_by_name(type_name).unwrap().id,
        );

        component
    }
}

fn entities_holder_get_component_ptr<C: 'static>(
    entities: &EntitiesHolderUnsafeFfi,
    types: &TypesRegistryCache,
    entity: EntityId,
) -> Option<*mut C> {
    entities
        .get_component_ptr(entity, types.get_or_register::<C>())
        .map(|p| p as *mut C)
}

fn entities_holder_get_component<'r, C: 'static>(
    entities: &'r EntitiesHolderUnsafeFfi,
    types: &TypesRegistryCache,
    entity: EntityId,
) -> Option<&'r C> {
    unsafe { entities_holder_get_component_ptr::<C>(entities, types, entity).map(|p| &*p) }
}

fn entities_holder_get_all_components<'r>(
    entities: &'r EntitiesHolderUnsafeFfi,
    entity: EntityId,
    mut handler: impl FnMut(FfiAnyRef<'r>),
) {
    unsafe {
        let mut handler = |i: (*mut u8, &'static FfiExtendedTypeInfo)| handler(i.1.as_any(FfiOpaqueRef::from_raw(i.0 as *const c_void)));
        let handler = FfiFnMutMut::new(&mut handler);
        entities.get_all_components_ptrs(entity, handler)
    }
}

fn entities_holder_get_all_components_mut<'r>(
    entities: &'r mut EntitiesHolderUnsafeFfi,
    entity: EntityId,
    mut handler: impl FnMut(FfiAnyMut<'r>),
) {
    unsafe {
        let mut handler = |i: (*mut u8, &'static FfiExtendedTypeInfo)| handler(i.1.as_any_mut(FfiOpaqueMut::from_raw(i.0 as *mut c_void)));
        let handler = FfiFnMutMut::new(&mut handler);
        entities.get_all_components_ptrs(entity, handler)
    }
}

fn entities_holder_get_component_mut<'r, C: 'static>(
    entities: &'r mut EntitiesHolderUnsafeFfi,
    types: &TypesRegistryCache,
    entity: EntityId,
) -> Option<&'r mut C> {
    unsafe { entities_holder_get_component_ptr::<C>(entities, types, entity).map(|p| &mut *p) }
}


fn entities_holder_entities_count(entities: &EntitiesHolderUnsafeFfi) -> u64 {
    entities.entities_count()
}

fn entities_holder_contains_entity(entities: &EntitiesHolderUnsafeFfi, entity: EntityId) -> bool {
    entities.contains_entity(entity)
}

fn entities_holder_create_entity(entities: &mut EntitiesHolderUnsafeFfi) -> EntityId {
    entities.create_entity()
}

fn entities_holder_destroy_entity(entities: &mut EntitiesHolderUnsafeFfi, entity: EntityId) -> bool {
    entities.destroy_entity(entity)
}


//

pub struct EntitiesHolderMut<'e> {
    entities: *mut EntitiesHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
}

impl<'e> EntitiesHolderMut<'e> {
    pub unsafe fn new(entities: *mut EntitiesHolderUnsafeFfi, types: &'e TypesRegistryCache) -> Self {
        Self { entities, types }
    }

    pub fn query<'r, A: ArchetypeIteratorItem>(&'r self) -> EntitiesHolderQuery<'r, A::ReadOnlyItem<'r>>
    where
        'e: 'r,
    {
        unsafe { entities_holder_query::<A::ReadOnlyItem<'r>, ()>(&*self.entities, self.types) }
    }

    pub fn query_filtered<'r, A: ArchetypeIteratorItem, F: QueryFilter>(&'r self) -> EntitiesHolderQuery<'r, A::ReadOnlyItem<'r>, F>
    where
        'e: 'r,
    {
        unsafe { entities_holder_query::<A::ReadOnlyItem<'r>, F>(&*self.entities, self.types) }
    }

    pub fn query_mut<'r, A: ArchetypeIteratorItem>(&'r mut self) -> EntitiesHolderQuery<'r, A::Item<'r>>
    where
        'e: 'r,
    {
        unsafe { entities_holder_query::<A::Item<'r>, ()>(&mut *self.entities, self.types) }
    }

    pub fn query_filtered_mut<'r, A: ArchetypeIteratorItem, F: QueryFilter>(&'r mut self) -> EntitiesHolderQuery<'r, A::Item<'r>, F>
    where
        'e: 'r,
    {
        unsafe { entities_holder_query::<A::Item<'r>, F>(&mut *self.entities, self.types) }
    }

    pub fn entities_count(&self) -> u64 {
        unsafe { entities_holder_entities_count(&*self.entities) }
    }

    pub fn contains_entity(&self, entity: EntityId) -> bool {
        unsafe { entities_holder_contains_entity(&*self.entities, entity) }
    }

    pub fn create_entity(&mut self) -> EntityId {
        unsafe { entities_holder_create_entity(&mut *self.entities) }
    }

    pub fn destroy_entity(&mut self, entity: EntityId) -> bool {
        unsafe { entities_holder_destroy_entity(&mut *self.entities, entity) }
    }

    pub fn add_component<C: 'static>(&mut self, entity: EntityId, component: C) -> Result<(), C> {
        unsafe { entities_holder_add_component(&mut *self.entities, self.types, entity, component) }
    }

    pub fn add_component_any(&mut self, entity: EntityId, component: FfiAny) -> Result<(), FfiAny> {
        unsafe { entities_holder_add_component_any(&mut *self.entities, self.types, entity, component) }
    }

    pub fn remove_component<C: 'static>(&mut self, entity: EntityId) -> Option<C> {
        unsafe { entities_holder_remove_component(&mut *self.entities, self.types, entity) }
    }

    pub fn remove_component_any(&mut self, entity: EntityId, type_name: &str) -> Option<FfiAny> {
        unsafe { entities_holder_remove_component_any(&mut *self.entities, self.types, entity, type_name) }
    }

    pub fn get_component_ptr<C: 'static>(&self, entity: EntityId) -> Option<*mut C> {
        unsafe { entities_holder_get_component_ptr(&*self.entities, self.types, entity) }
    }

    pub fn get_component<'r, C: 'static>(&'r self, entity: EntityId) -> Option<&'r C>
    where
        'e: 'r,
    {
        unsafe { entities_holder_get_component(&*self.entities, self.types, entity) }
    }

    pub fn get_component_mut<'r, C: 'static>(&'r mut self, entity: EntityId) -> Option<&'r mut C>
    where
        'e: 'r,
    {
        unsafe { entities_holder_get_component_mut(&mut *self.entities, self.types, entity) }
    }

    pub fn get_all_components<'r>(&'r self, entity: EntityId, handler: impl FnMut(FfiAnyRef<'r>))
    where
        'e: 'r,
    {
        unsafe { entities_holder_get_all_components(&*self.entities, entity, handler) }
    }

    pub fn as_mut<'r>(&'r mut self) -> EntitiesHolderMut<'r>
    where
        'e: 'r,
    {
        EntitiesHolderMut {
            entities: self.entities,
            types: self.types,
        }
    }

    pub fn as_ref<'r>(&'r self) -> EntitiesHolderRef<'r>
    where
        'e: 'r,
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
    entities: *const EntitiesHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
}

impl<'e> EntitiesHolderRef<'e> {
    pub unsafe fn new(entities: *const EntitiesHolderUnsafeFfi, types: &'e TypesRegistryCache) -> Self {
        Self { entities, types }
    }

    pub unsafe fn unsafe_query<A: ArchetypeIteratorItem, F: QueryFilter>(self) -> EntitiesHolderQuery<'e, A, F> {
        unsafe { entities_holder_query::<A, F>(&*self.entities, self.types) }
    }

    pub fn query<A: ArchetypeIteratorItem>(self) -> EntitiesHolderQuery<'e, A::ReadOnlyItem<'e>> {
        unsafe { entities_holder_query::<A::ReadOnlyItem<'e>, ()>(&*self.entities, self.types) }
    }

    pub fn query_filtered<A: ArchetypeIteratorItem, F: QueryFilter>(self) -> EntitiesHolderQuery<'e, A::ReadOnlyItem<'e>, F> {
        unsafe { entities_holder_query::<A::ReadOnlyItem<'e>, F>(&*self.entities, self.types) }
    }

    pub fn entities_count(self) -> u64 {
        unsafe { entities_holder_entities_count(&*self.entities) }
    }

    pub fn contains_entity(self, entity: EntityId) -> bool {
        unsafe { entities_holder_contains_entity(&*self.entities, entity) }
    }

    pub fn get_component_ptr<C: 'static>(self, entity: EntityId) -> Option<*mut C> {
        unsafe { entities_holder_get_component_ptr(&*self.entities, self.types, entity) }
    }

    pub fn get_component<C: 'static>(self, entity: EntityId) -> Option<&'e C> {
        unsafe { entities_holder_get_component(&*self.entities, self.types, entity) }
    }

    pub fn get_all_components<'r>(&'r self, entity: EntityId, handler: impl FnMut(FfiAnyRef<'r>))
    where
        'e: 'r,
    {
        unsafe { entities_holder_get_all_components(&*self.entities, entity, handler) }
    }
}

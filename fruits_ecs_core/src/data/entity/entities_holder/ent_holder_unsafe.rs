use std::mem::MaybeUninit;

use crate::*;

#[derive(Clone)]
pub struct EntitiesHolderUnsafeRef<'e> {
    entities: *mut EntitiesHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
}

impl<'e> EntitiesHolderUnsafeRef<'e> {
    pub fn new(entities: *mut EntitiesHolderUnsafeFfi, types: &'e TypesRegistryCache) -> Self {
        Self {
            entities,
            types,
        }
    }

    pub unsafe fn query<'r, A: ArchetypeIteratorItem, F: QueryFilter>(&'r self) -> EntitiesHolderQuery<'r, A, F>
        where 'e: 'r
    {
        unsafe { EntitiesHolderQuery::<'r, A, F>::new(self.clone()) }
    }

    pub unsafe fn entities_count(&self) -> u64 {
        unsafe { (&*self.entities).entities_count() }
    }

    pub unsafe fn contains_entity(&self, entity: Entity) -> bool {
        unsafe { (&*self.entities).contains_entity(entity) }
    }

    pub unsafe fn create_entity(&mut self) -> Entity {
        unsafe { (&mut *self.entities).create_entity() }
    }

    pub unsafe fn destroy_entity(&mut self, entity: Entity) -> bool {
        unsafe { (&mut *self.entities).destroy_entity(entity) }
    }

    pub unsafe fn add_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<(), C> {
        unsafe {
            let mut component = MaybeUninit::new(component);

            if (&mut *self.entities).add_component(
                entity,
                component.as_mut_ptr() as *mut u8,
                (&self.types).get_or_register::<C>(),
            ) {
                Ok(())
            } else {
                Err(component.assume_init())
            }
        }
    }

    pub unsafe fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        unsafe {
            let mut component = MaybeUninit::<C>::uninit();

            if (&mut *self.entities).remove_component(
                entity,
                component.as_mut_ptr() as *mut u8,
                (&self.types).get_or_register::<C>(),
            ) {
                Some(component.assume_init())
            } else {
                None
            }
        }
    }

    pub unsafe fn get_component_ptr<C: Component>(&self, entity: Entity) -> Option<*mut C> {
        unsafe { (&*self.entities).get_component_ptr(entity, (&self.types).get_or_register::<C>()).map(|p| p as *mut C) }
    }

    pub unsafe fn ffi(&self) -> *mut EntitiesHolderUnsafeFfi {
        self.entities
    }

    pub fn types(&self) -> &TypesRegistryCache {
        &self.types
    }

    pub unsafe fn into_safe(self) -> EntitiesHolderMut<'e> {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<EntitiesHolderUnsafeRef, EntitiesHolderMut>(self) }
    }

    pub unsafe fn as_safe<'r>(&'r self) -> &'r EntitiesHolderMut<'e> {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EntitiesHolderUnsafeRef, &EntitiesHolderMut>(self) }
    }

    pub fn as_safe_mut<'r>(&'r mut self) -> &'r mut EntitiesHolderMut<'e> {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut EntitiesHolderUnsafeRef, &mut EntitiesHolderMut>(self) }
    }

    pub unsafe fn from_safe(v: EntitiesHolderMut<'e>) -> Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<EntitiesHolderMut, EntitiesHolderUnsafeRef>(v) }
    }

    pub unsafe fn from_safe_ref<'r>(v: &'r EntitiesHolderMut<'e>) -> &'r Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EntitiesHolderMut, &EntitiesHolderUnsafeRef>(v) }
    }

    pub fn from_safe_mut<'r>(v: &'r mut EntitiesHolderMut<'e>) -> &'r mut Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut EntitiesHolderMut, &mut EntitiesHolderUnsafeRef>(v) }
    }
}
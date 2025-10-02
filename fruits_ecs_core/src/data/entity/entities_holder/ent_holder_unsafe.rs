use std::mem::MaybeUninit;

use crate::*;

fn entities_holder_unsafe_create_entity(
    entities: &mut EntitiesHolderUnsafeFfi,
) -> Entity {
    entities.create_entity()
}

fn entities_holder_unsafe_destroy_entity(
    entities: &mut EntitiesHolderUnsafeFfi,
    entity: Entity,
) -> bool {
    entities.destroy_entity(entity)
}

fn entities_holder_unsafe_add_component<C: Component>(
    entities: &mut EntitiesHolderUnsafeFfi,
    types: &TypesRegistryCache,
    entity: Entity,
    component: C,
) -> Result<(), C> {
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

fn entities_holder_unsafe_remove_component<C: Component>(
    entities: &mut EntitiesHolderUnsafeFfi,
    types: &TypesRegistryCache,
    entity: Entity,
) -> Option<C> {
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

fn entities_holder_unsafe_get_component_ptr<C: Component>(
    entities: &EntitiesHolderUnsafeFfi,
    types: &TypesRegistryCache,
    entity: Entity,
) -> Option<*mut C> {
    entities.get_component_ptr(entity, types.get_or_register::<C>()).map(|p| p as *mut C)
}

//

pub struct EntitiesHolderUnsafe {
    entities: EntitiesHolderUnsafeFfi,
    types: TypesRegistryCache,
}
impl EntitiesHolderUnsafe {
    pub fn new(types: TypesRegistryCache) -> Self {
        Self {
            entities: EntitiesHolderUnsafeFfi::new(unsafe { types.registry().clone() }),
            types: types,
        }
    }
    
    pub unsafe fn query<'a, A: ArchetypeIteratorItem, F: QueryFilter>(&'a self) -> EntitiesHolderQuery<'a, A, F> {
        unsafe { EntitiesHolderQuery::<'a, A, F>::new(self.as_ref()) }
    }

    pub fn entities_count(&self) -> u64 {
        self.entities.entities_count()
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entities.contains_entity(entity)
    }

    pub fn create_entity(&mut self) -> Entity {
        entities_holder_unsafe_create_entity(&mut self.entities)
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        entities_holder_unsafe_destroy_entity(&mut self.entities, entity)
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<(), C> {
        entities_holder_unsafe_add_component(&mut self.entities, &self.types, entity, component)
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        entities_holder_unsafe_remove_component(&mut self.entities, &self.types, entity)
    }

    pub fn get_component_ptr<C: Component>(&self, entity: Entity) -> Option<*mut C> {
        entities_holder_unsafe_get_component_ptr(&self.entities, &self.types, entity)
    }

    pub unsafe fn as_ffi(&self) -> &EntitiesHolderUnsafeFfi {
        &self.entities
    }

    pub fn types(&self) -> &TypesRegistryCache {
        &self.types
    }

    pub fn as_mut<'e>(&'e mut self) -> EntitiesHolderUnsafeMut<'e> {
        EntitiesHolderUnsafeMut {
            entities: &mut self.entities,
            types: &self.types,
        }
    }

    pub fn as_ref<'e>(&'e self) -> EntitiesHolderUnsafeRef<'e> {
        EntitiesHolderUnsafeRef {
            entities: &self.entities,
            types: &self.types,
        }
    }

    pub unsafe fn into_safe(self) -> EntitiesHolder {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<EntitiesHolderUnsafe, EntitiesHolder>(self) }
    }

    pub unsafe fn as_safe(&self) -> &EntitiesHolder {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EntitiesHolderUnsafe, &EntitiesHolder>(self) }
    }

    pub fn as_safe_mut(&mut self) -> &mut EntitiesHolder {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut EntitiesHolderUnsafe, &mut EntitiesHolder>(self) }
    }

    pub unsafe fn from_safe(v: EntitiesHolder) -> Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<EntitiesHolder, EntitiesHolderUnsafe>(v) }
    }

    pub unsafe fn from_safe_ref(v: &EntitiesHolder) -> &Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EntitiesHolder, &EntitiesHolderUnsafe>(v) }
    }

    pub fn from_safe_mut(v: &mut EntitiesHolder) -> &mut Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut EntitiesHolder, &mut EntitiesHolderUnsafe>(v) }
    }
}

//

pub struct EntitiesHolderUnsafeMut<'e> {
    entities: &'e mut EntitiesHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
}
impl<'e> EntitiesHolderUnsafeMut<'e> {
    pub fn new(entities: &'e mut EntitiesHolderUnsafeFfi, types: &'e TypesRegistryCache) -> Self {
        Self {
            entities,
            types,
        }
    }

    pub unsafe fn query<'a, A: ArchetypeIteratorItem, F: QueryFilter>(&'a self) -> EntitiesHolderQuery<'a, A, F> {
        unsafe { EntitiesHolderQuery::<'a, A, F>::new(self.as_ref()) }
    }

    pub fn entities_count(&self) -> u64 {
        self.entities.entities_count()
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entities.contains_entity(entity)
    }

    pub fn create_entity(&mut self) -> Entity {
        entities_holder_unsafe_create_entity(self.entities)
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        entities_holder_unsafe_destroy_entity(self.entities, entity)
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<(), C> {
        entities_holder_unsafe_add_component(self.entities, &self.types, entity, component)
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        entities_holder_unsafe_remove_component(self.entities, &self.types, entity)
    }

    pub fn get_component_ptr<C: Component>(&self, entity: Entity) -> Option<*mut C> {
        entities_holder_unsafe_get_component_ptr(self.entities, &self.types, entity)
    }

    pub unsafe fn as_ffi(&self) -> &EntitiesHolderUnsafeFfi {
        self.entities
    }

    pub fn types(&self) -> &TypesRegistryCache {
        &self.types
    }

    pub fn as_mut<'r>(&'r mut self) -> EntitiesHolderUnsafeMut<'r>
        where 'e: 'r
    {
        EntitiesHolderUnsafeMut {
            entities: self.entities,
            types: self.types,
        }
    }

    pub fn as_ref<'r>(&'r self) -> EntitiesHolderUnsafeRef<'r>
        where 'e: 'r
    {
        EntitiesHolderUnsafeRef {
            entities: self.entities,
            types: self.types,
        }
    }

    pub unsafe fn into_safe(self) -> EntitiesHolderMut<'e> {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<EntitiesHolderUnsafeMut, EntitiesHolderMut>(self) }
    }

    pub unsafe fn as_safe<'r>(&'r self) -> &'r EntitiesHolderMut<'e> {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EntitiesHolderUnsafeMut, &EntitiesHolderMut>(self) }
    }

    pub fn as_safe_mut<'r>(&'r mut self) -> &'r mut EntitiesHolderMut<'e> {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut EntitiesHolderUnsafeMut, &mut EntitiesHolderMut>(self) }
    }

    pub unsafe fn from_safe(v: EntitiesHolderMut<'e>) -> Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<EntitiesHolderMut, EntitiesHolderUnsafeMut>(v) }
    }

    pub unsafe fn from_safe_ref<'r>(v: &'r EntitiesHolderMut<'e>) -> &'r Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EntitiesHolderMut, &EntitiesHolderUnsafeMut>(v) }
    }

    pub fn from_safe_mut<'r>(v: &'r mut EntitiesHolderMut<'e>) -> &'r mut Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut EntitiesHolderMut, &mut EntitiesHolderUnsafeMut>(v) }
    }
}

//

pub struct EntitiesHolderUnsafeRef<'e> {
    entities: &'e EntitiesHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
}
impl<'e> EntitiesHolderUnsafeRef<'e> {
    pub fn new(entities: &'e EntitiesHolderUnsafeFfi, types: &'e TypesRegistryCache) -> Self {
        Self {
            entities,
            types,
        }
    }

    pub unsafe fn query<'a, A: ArchetypeIteratorItem, F: QueryFilter>(&'a self) -> EntitiesHolderQuery<'a, A, F> {
        unsafe { EntitiesHolderQuery::<'a, A, F>::new(self.as_ref()) }
    }

    pub fn entities_count(&self) -> u64 {
        self.entities.entities_count()
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entities.contains_entity(entity)
    }

    pub fn get_component_ptr<C: Component>(&self, entity: Entity) -> Option<*mut C> {
        entities_holder_unsafe_get_component_ptr(self.entities, &self.types, entity)
    }

    pub unsafe fn as_ffi(&self) -> &EntitiesHolderUnsafeFfi {
        self.entities
    }

    pub fn types(&self) -> &TypesRegistryCache {
        &self.types
    }

    pub fn as_ref<'r>(&'r self) -> EntitiesHolderUnsafeRef<'r>
        where 'e: 'r
    {
        EntitiesHolderUnsafeRef {
            entities: self.entities,
            types: self.types,
        }
    }

    pub unsafe fn into_safe(self) -> EntitiesHolderRef<'e> {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<EntitiesHolderUnsafeRef, EntitiesHolderRef>(self) }
    }

    pub unsafe fn as_safe<'r>(&'r self) -> &'r EntitiesHolderRef<'e> {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EntitiesHolderUnsafeRef, &EntitiesHolderRef>(self) }
    }

    pub unsafe fn from_safe(v: EntitiesHolderRef<'e>) -> Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<EntitiesHolderRef, EntitiesHolderUnsafeRef>(v) }
    }

    pub unsafe fn from_safe_ref<'r>(v: &'r EntitiesHolderRef<'e>) -> &'r Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EntitiesHolderRef, &EntitiesHolderUnsafeRef>(v) }
    }
}

use fruits_ffi::{FfiOpaqueVec, FfiVec};

use crate::*;

fn events_holder_get<'e, E: 'static>(evt: &'e EventsHolderUnsafeFfi, types: &TypesRegistryCache) -> &'e [E] {
    unsafe {
        let result = evt.get(types.get_or_register::<E>());

        if result.is_null() {
            &[]
        } else {
            &*FfiOpaqueVec::as_vec_ptr(result)
        }
    }
}

fn events_holder_get_ptr<'e, E: 'static>(evt: &'e EventsHolderUnsafeFfi, types: &TypesRegistryCache) -> *mut FfiVec<E> {
    unsafe {
        let result = evt.get_or_create(types.get_or_register::<E>());

        FfiOpaqueVec::as_vec_ptr(result)
    }
}

fn events_holder_get_mut<'e, E: 'static>(evt: &'e mut EventsHolderUnsafeFfi, types: &TypesRegistryCache) -> &'e mut FfiVec<E> {
    unsafe {
        &mut *events_holder_get_ptr(evt, types)
    }
}

//

// todo

pub struct EventsHolderMut<'e> {
    evt: &'e mut EventsHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
}

impl<'e> EventsHolderMut<'e> {
    pub fn new(evt: &'e mut EventsHolderUnsafeFfi, types: &'e TypesRegistryCache) -> Self {
        Self {
            evt,
            types,
        }
    }

    pub fn get<'r, E: 'static>(&'r self) -> &'r [E]
        where 'e: 'r
    {
        events_holder_get(self.evt, self.types)
    }

    pub fn get_ptr<'r, E: 'static>(&'r self) -> *mut FfiVec<E>
        where 'e: 'r
    {
        events_holder_get_ptr(self.evt, self.types)
    }

    pub fn get_mut<'r, E: 'static>(&'r mut self) -> &'r mut FfiVec<E>
        where 'e: 'r
    {
        events_holder_get_mut(self.evt, self.types)
    }

    pub fn clear(&mut self) {
        self.evt.clear();
    }
}


//

#[derive(Copy, Clone)]
pub struct EventsHolderRef<'e> {
    evt: &'e EventsHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
}

impl<'e> EventsHolderRef<'e> {
    pub fn new(evt: &'e EventsHolderUnsafeFfi, types: &'e TypesRegistryCache) -> Self {
        Self {
            evt,
            types,
        }
    }

    pub fn get<E: 'static>(self) -> &'e [E] {
        events_holder_get(self.evt, self.types)
    }

    pub fn get_ptr<E: 'static>(self) -> *mut FfiVec<E> {
        events_holder_get_ptr(self.evt, self.types)
    }
}
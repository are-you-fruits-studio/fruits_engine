use fruits_ffi::{FfiOpaqueVec, FfiVec};

use crate::{evt_ffi::EventsHolderUnsafeFfi, evt_safe::EventsHolderRef, TypesRegistryCache};

fn events_holder_unsafe_get<E: 'static>(evt: &EventsHolderUnsafeFfi, types: &TypesRegistryCache) -> Option<*mut FfiVec<E>> {
    let type_id = types.get_or_register::<E>();

    let result = evt.get(type_id);

    if result.is_null() {
        None
    } else {
        Some(unsafe { FfiOpaqueVec::as_vec_ptr(result) })
    }
}

fn events_holder_unsafe_get_or_create<E: 'static>(evt: &EventsHolderUnsafeFfi, types: &TypesRegistryCache) -> *mut FfiVec<E> {
    let type_id = types.get_or_register::<E>();
    
    let result = evt.get_or_create(type_id);

    unsafe { FfiOpaqueVec::as_vec_ptr(result) }
}

//

pub struct EventsHolderUnsafeRef<'e> {
    evt: &'e EventsHolderUnsafeFfi,
    types: TypesRegistryCache,
}

impl<'e> EventsHolderUnsafeRef<'e> {
    pub fn new(evt: &'e EventsHolderUnsafeFfi, types: TypesRegistryCache) -> Self {
        Self {
            evt,
            types,
        }
    }

    pub fn get<E: 'static>(&self) -> Option<*mut FfiVec<E>> {
        events_holder_unsafe_get(&self.evt, &self.types)
    }

    pub fn get_or_create<E: 'static>(&self) -> *mut FfiVec<E> {
        events_holder_unsafe_get_or_create(&self.evt, &self.types)
    }

    pub fn clear(&self) {
        self.evt.clear();
    }

    pub fn as_ref<'r>(&'r self) -> EventsHolderUnsafeRef<'r>
        where 'e: 'r
    {
        EventsHolderUnsafeRef {
            evt: self.evt,
            types: self.types.clone(),
        }
    }

    pub fn into_safe(self) -> EventsHolderRef<'e> {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<Self, EventsHolderRef<'e>>(self) }
    }

    pub fn from_safe(evt: EventsHolderRef<'e>) -> Self {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<EventsHolderRef<'e>, Self>(evt) }
    }
}
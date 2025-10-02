use fruits_ffi::{FfiOpaqueVec, FfiVec};

use crate::*;

//

#[derive(Clone)]
pub struct EventsHolderUnsafeRef<'e> {
    evt: *const EventsHolderUnsafeFfi,
    types: &'e TypesRegistryCache,
}

impl<'e> EventsHolderUnsafeRef<'e> {
    pub fn new(evt: *const EventsHolderUnsafeFfi, types: &'e TypesRegistryCache) -> Self {
        Self {
            evt,
            types,
        }
    }

    pub unsafe fn get<E: 'static>(&self) -> Option<*mut FfiVec<E>> {
        unsafe {
            let result = (&*self.evt).get((&self.types).get_or_register::<E>());
    
            if result.is_null() {
                None
            } else {
                Some(FfiOpaqueVec::as_vec_ptr(result))
            }
        }
    }

    pub unsafe fn get_or_create<E: 'static>(&self) -> *mut FfiVec<E> {
        unsafe {
            let result = (&*self.evt).get_or_create(self.types.get_or_register::<E>());
    
            FfiOpaqueVec::as_vec_ptr(result)
        }
    }

    pub unsafe fn clear(&self) {
        unsafe { (&*self.evt).clear(); }
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
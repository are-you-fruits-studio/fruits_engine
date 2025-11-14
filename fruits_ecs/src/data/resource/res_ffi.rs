use std::ffi::c_void;

use fruits_ffi::{FfiDroppable, FfiStaticRef};

use crate::*;

#[repr(C)]
struct ResourcesHolderUnsafeFfiVTable {
    insert_fn: unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void,
    get_fn: unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void,
}

#[repr(C)]
pub struct ResourcesHolderUnsafeFfi {
    data: FfiDroppable,
    vtable: FfiStaticRef<ResourcesHolderUnsafeFfiVTable>,
}

impl ResourcesHolderUnsafeFfi {
    pub fn new(types: TypesRegistryAccessFfi) -> Self {
        unsafe extern "C" fn ffi_insert(this_ref_mut: *mut c_void, id: u64) -> *mut c_void {
            // todo
            unsafe { (&mut *(this_ref_mut as *mut ResourcesHolderNative)).insert(id) as *mut c_void }
        }

        unsafe extern "C" fn ffi_get(this_ref: *mut c_void, id: u64) -> *mut c_void {
            // todo
            unsafe { (&*(this_ref as *mut ResourcesHolderNative)).get(id) as *mut c_void }
        }

        Self {
            data: FfiDroppable::new(ResourcesHolderNative::new(types)),
            vtable: FfiStaticRef::new(&ResourcesHolderUnsafeFfiVTable {
                get_fn: ffi_get,
                insert_fn: ffi_insert,
            }),
        }
    }

    pub unsafe fn insert(&mut self, id: u64) -> *mut u8 {
        // todo
        unsafe { (self.vtable.insert_fn)(self.data.get(), id) as *mut u8 }
    }

    pub unsafe fn get(&self, id: u64) -> *mut u8 {
        // todo
        unsafe { (self.vtable.get_fn)(self.data.get(), id) as *mut u8 }
    }
}

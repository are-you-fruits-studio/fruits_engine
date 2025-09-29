use std::ffi::c_void;

use fruits_ffi::{FfiDroppable, FfiTypedDroppable};

use crate::{res_native::ResourcesHolderNative, TypesRegistryAccessFfi};

#[repr(C)]
struct ResourcesHolderUnsafeFfiVTable {
    insert_fn: unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void,
    get_fn: unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void,
}

#[repr(C)]
pub struct ResourcesHolderUnsafeFfi {
    data: FfiDroppable,
    vtable: FfiTypedDroppable<ResourcesHolderUnsafeFfiVTable>,
}

impl ResourcesHolderUnsafeFfi {
    pub fn new(types: TypesRegistryAccessFfi) -> Self {
        let res = ResourcesHolderNative::new(types);

        Self {
            data: FfiDroppable::new(res),
            vtable: FfiTypedDroppable::new(ResourcesHolderUnsafeFfiVTable {
                get_fn: Self::ffi_get,
                insert_fn: Self::ffi_insert,
            }),
        }
    }

    pub unsafe fn insert(&mut self, id: u64) -> *mut u8 {
        // todo
        unsafe {
            (self.vtable.insert_fn)(self.data.get(), id) as *mut u8
        }
    }

    pub unsafe fn get(&self, id: u64) -> *mut u8  {
        // todo
        unsafe {
            (self.vtable.get_fn)(self.data.get(), id) as *mut u8
        }
    }

    pub unsafe extern "C" fn ffi_insert(this_ref_mut: *mut c_void, id: u64) -> *mut c_void {
        // todo
        unsafe {
            (&mut *(this_ref_mut as *mut ResourcesHolderNative)).insert(id) as *mut c_void
        }
    }

    pub unsafe extern "C" fn ffi_get(this_ref: *mut c_void, id: u64) -> *mut c_void {
        // todo
        unsafe {
            (&*(this_ref as *mut ResourcesHolderNative)).get(id) as *mut c_void
        }
    }
}
use std::ffi::c_void;

use fruits_ffi::{FfiDroppable, FfiTypedDroppable};

use crate::*;

#[repr(C)]
struct SystemResourcesHolderUnsafeFfiVTable {
    get_or_insert_fn: unsafe extern "C" fn(*const c_void, u64) -> SystemResourceGetOrInsertResult,
}

#[repr(C)]
pub struct SystemResourcesHolderUnsafeFfi {
    data: FfiDroppable,
    vtable: FfiTypedDroppable<SystemResourcesHolderUnsafeFfiVTable>,
}

impl SystemResourcesHolderUnsafeFfi {
    pub fn new(types: TypesRegistryAccessFfi) -> Self {
        let res = SystemResourcesHolderNative::new(types);

        Self {
            data: FfiDroppable::new(res),
            vtable: FfiTypedDroppable::new(SystemResourcesHolderUnsafeFfiVTable {
                get_or_insert_fn: Self::ffi_get_or_insert,
            }),
        }
    }

    pub unsafe fn get_or_insert(&self, id: u64) -> SystemResourceGetOrInsertResult  {
        // todo
        unsafe {
            (self.vtable.get_or_insert_fn)(self.data.get(), id)
        }
    }

    pub unsafe extern "C" fn ffi_get_or_insert(this_ref: *const c_void, id: u64) -> SystemResourceGetOrInsertResult {
        // todo
        unsafe {
            (&*(this_ref as *const SystemResourcesHolderNative)).get_or_insert(id)
        }
    }
}
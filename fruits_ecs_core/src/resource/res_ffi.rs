use std::ffi::c_void;

use crate::{res_native::ResourcesHolderNative, TypesRegistryAccessFfi};


#[repr(C)]
pub struct ResourcesHolderUnsafeFfi {
    data: *mut c_void,
    insert_fn: unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void,
    get_fn: unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void,
    drop_fn: unsafe extern "C" fn(*mut c_void),
}

impl ResourcesHolderUnsafeFfi {
    pub fn new(types: TypesRegistryAccessFfi) -> Self {
        let res = ResourcesHolderNative::new(types);

        Self {
            data: Box::into_raw(Box::new(res)) as *mut c_void,
            get_fn: Self::ffi_get,
            insert_fn: Self::ffi_insert,
            drop_fn: Self::ffi_drop,
        }
    }

    pub unsafe fn insert(&mut self, id: u64) -> *mut u8 {
        // todo
        unsafe {
            (self.insert_fn)(self.data, id) as *mut u8
        }
    }

    pub unsafe fn get(&self, id: u64) -> *mut u8  {
        // todo
        unsafe {
            (self.get_fn)(self.data, id) as *mut u8
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

    pub unsafe extern "C" fn ffi_drop(this_ref: *mut c_void) {
        // todo
        unsafe {
            drop(Box::from_raw(this_ref as *mut ResourcesHolderNative));
        }
    }
}

impl Drop for ResourcesHolderUnsafeFfi {
    fn drop(&mut self) {
        // todo
        unsafe {
            (self.drop_fn)(self.data);
        }
    }
}
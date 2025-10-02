use std::{collections::HashMap, ffi::c_void};

use crate::*;

pub struct ResourcesHolderNative {
    types: TypesRegistryAccessFfi,
    resources: HashMap<u64, *mut u8>,
}
impl ResourcesHolderNative {
    pub fn new(types: TypesRegistryAccessFfi) -> Self {
        Self {
            types,
            resources: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: u64) -> *mut u8 {
        let Some(type_data) = self.types.get(id) else {
            return std::ptr::null_mut();
        };

        if self.resources.contains_key(&id) {
            return std::ptr::null_mut();
        }

        // todo
        let mem = unsafe {
            let Some(layout) = std::alloc::Layout::from_size_align(
                type_data.size as usize,
                type_data.align as usize,
            ).ok() else {
                return std::ptr::null_mut();
            };

            std::alloc::alloc(layout)
        };

        self.resources.insert(id, mem);

        mem
    }

    pub fn get(&self, id: u64) -> *mut u8 {
        self.resources.get(&id).copied().unwrap_or(std::ptr::null_mut())
    }
}

impl Drop for ResourcesHolderNative {
    fn drop(&mut self) {
        for (&id, &mem) in self.resources.iter() {
            let type_data = self.types.get(id).unwrap();

            // todo
            unsafe {
                if let Some(drop_fn) = type_data.drop_fn {
                    drop_fn(mem as *mut c_void);
                }

                let layout = std::alloc::Layout::from_size_align(
                    type_data.size as usize,
                    type_data.align as usize,
                ).ok().unwrap();

                std::alloc::dealloc(mem, layout)
            }
        }
    }
}

// todo?
// Safety. It is safe itself. Ptr usage is managed by caller
unsafe impl Send for ResourcesHolderNative { }
unsafe impl Sync for ResourcesHolderNative { }

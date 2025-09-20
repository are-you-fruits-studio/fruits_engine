use std::{alloc::GlobalAlloc, collections::HashMap, ffi::c_void, ptr::NonNull};

use crate::{types_registry::TypesRegistryRef, TypesRegistryCache};

#[repr(C)]
pub struct ResourcesHolderUnsafeRefFfi {
    data: *mut c_void,
    insert_fn: unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void,
    get_fn: unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void,
}

impl ResourcesHolderUnsafeRefFfi {
    pub unsafe fn from_unsafe(resources: &mut ResourcesHolderUnsafe) -> Self {
        Self {
            data: &raw mut *resources as *mut c_void,
            get_fn: Self::ffi_get,
            insert_fn: Self::ffi_insert,
        }
    }

    pub unsafe fn insert(&mut self, id: u64) -> Option<NonNull<u8>>  {
        // todo
        unsafe {
            NonNull::new((self.insert_fn)(self.data, id) as *mut u8)
        }
    }

    pub unsafe fn get(&self, id: u64) -> Option<NonNull<u8>>  {
        // todo
        unsafe {
            NonNull::new((self.get_fn)(self.data, id) as *mut u8)
        }
    }

    pub unsafe extern "C" fn ffi_insert(this_ref_mut: *mut c_void, id: u64) -> *mut c_void {
        // todo
        unsafe {
            let this = &mut *(this_ref_mut as *mut ResourcesHolderUnsafe);

            let result = this.insert(id);

            result.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut()) as *mut c_void
        }
    }

    pub unsafe extern "C" fn ffi_get(this_ref: *mut c_void, id: u64) -> *mut c_void {
        // todo
        unsafe {
            let this = &*(this_ref as *mut ResourcesHolderUnsafe);

            let result = this.get(id);

            result.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut()) as *mut c_void
        }
    }
}

pub struct ResourcesHolderUnsafe {
    types: TypesRegistryRef,
    resources: HashMap<u64, NonNull<u8>>,
}
impl ResourcesHolderUnsafe {
    pub fn new(types: TypesRegistryRef) -> Self {
        Self {
            types,
            resources: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: u64) -> Option<NonNull<u8>> {
        let type_data = self.types.get(id)?;

        if self.resources.contains_key(&id) {
            return None;
        }

        // todo
        let mem = unsafe {
            std::alloc::System.alloc(std::alloc::Layout::from_size_align(
                type_data.data.size as usize,
                type_data.data.align as usize,
            ).ok()?)
        };

        let mem = NonNull::new(mem)?;

        self.resources.insert(id, mem);

        Some(mem)
    }

    pub fn get(&self, id: u64) -> Option<NonNull<u8>> {
        self.resources.get(&id).copied()
    }
}

impl Drop for ResourcesHolderUnsafe {
    fn drop(&mut self) {
        for (&id, &mem) in self.resources.iter() {
            let type_data = self.types.get(id).unwrap();

            // todo
            unsafe {
                if let Some(drop_fn) = type_data.data.drop_fn {
                    drop_fn(mem.as_ptr() as *mut c_void);
                }
            }
        }
    }
}

// todo?
// Safety. It is safe itself. Ptr usage is managed by caller
unsafe impl Send for ResourcesHolderUnsafe { }
unsafe impl Sync for ResourcesHolderUnsafe { }

//

// todo: depend on the ffi version instead
pub struct ResourcesHolderRef {
    types: TypesRegistryCache,
    res: ResourcesHolderUnsafeRefFfi,
}

impl ResourcesHolderRef {
    pub unsafe fn from_ffi(res: ResourcesHolderUnsafeRefFfi, types: TypesRegistryCache) -> Self {
        Self {
            res,
            types,
        }
    }

    pub fn insert<T: 'static>(&mut self, data: T) -> Result<(), T> {
        let type_id = self.types.get_or_register::<T>();

        unsafe {
            let Some(mem) = self.res.insert(type_id) else {
                return Err(data);
            };
            
            (mem.as_ptr() as *mut T).write(data);
        }

        Ok(())
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        let type_id = self.types.get_or_register::<T>();

        unsafe {
            let mem = self.res.get(type_id)?;

            Some(&*(mem.as_ptr() as *mut T))
        }
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = self.types.get_or_register::<T>();

        unsafe {
            let mem = self.res.get(type_id)?;

            Some(&mut *(mem.as_ptr() as *mut T))
        }
    }
}
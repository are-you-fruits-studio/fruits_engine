
use crate::{res_ffi::ResourcesHolderUnsafeFfi, res_safe::{ResourcesHolder, ResourcesHolderMut, ResourcesHolderRef}, TypesRegistryCache};

fn resources_holder_typed_unsafe_get<T: 'static>(types: &TypesRegistryCache, res: &ResourcesHolderUnsafeFfi) -> Option<*mut T> {
    let type_id = types.get_or_register::<T>();

    unsafe {
        let mem = res.get(type_id);

        if mem.is_null() {
            return None;
        }

        Some(mem as *mut T)
    }
}

fn resources_holder_typed_unsafe_insert<T: 'static>(types: &TypesRegistryCache, res: &mut ResourcesHolderUnsafeFfi, data: T) -> Result<(), T> {
    let type_id = types.get_or_register::<T>();

    unsafe {
        let mem = res.insert(type_id);
        
        if mem.is_null() {
            return Err(data);
        };
        
        (mem as *mut T).write(data);
    }

    Ok(())
}

// todo: refactor to use real references instead of lifetimed structs.

pub struct ResourcesHolderUnsafe {
    res: ResourcesHolderUnsafeFfi,
    types: TypesRegistryCache,
}

impl ResourcesHolderUnsafe {
    pub fn new(res: ResourcesHolderUnsafeFfi, types: TypesRegistryCache) -> Self {
        Self {
            res,
            types,
        }
    }

    pub fn get<T: 'static>(&self) -> Option<*mut T> {
        resources_holder_typed_unsafe_get(&self.types, &self.res)
    }
    
    pub fn insert<T: 'static>(&mut self, data: T) -> Result<(), T> {
        resources_holder_typed_unsafe_insert(&self.types, &mut self.res, data)
    }

    pub fn as_mut<'r>(&'r mut self) -> ResourcesHolderUnsafeMut<'r> {
        ResourcesHolderUnsafeMut {
            res: &mut self.res,
            types: &self.types,
        }
    }

    pub fn as_ref<'r>(&'r self) -> ResourcesHolderUnsafeRef<'r> {
        ResourcesHolderUnsafeRef {
            res: &self.res,
            types: &self.types,
        }
    }

    pub fn into_safe(self) -> ResourcesHolder {
        ResourcesHolder::new(self)
    }
}

pub struct ResourcesHolderUnsafeMut<'r> {
    res: &'r mut ResourcesHolderUnsafeFfi,
    types: &'r TypesRegistryCache,
}

impl<'r> ResourcesHolderUnsafeMut<'r> {
    pub fn new(res: &'r mut ResourcesHolderUnsafeFfi, types: &'r TypesRegistryCache) -> Self {
        Self {
            res,
            types,
        }
    }

    pub fn get<T: 'static>(&self) -> Option<*mut T> {
        resources_holder_typed_unsafe_get(&self.types, &self.res)
    }
    
    pub fn insert<T: 'static>(&mut self, data: T) -> Result<(), T> {
        resources_holder_typed_unsafe_insert(&self.types, self.res, data)
    }

    pub fn as_mut(&'r mut self) -> ResourcesHolderUnsafeMut<'r> {
        ResourcesHolderUnsafeMut {
            res: &mut self.res,
            types: self.types,
        }
    }

    pub fn as_ref(&'r self) -> ResourcesHolderUnsafeRef<'r> {
        ResourcesHolderUnsafeRef {
            res: &self.res,
            types: self.types,
        }
    }

    pub fn into_safe(self) -> ResourcesHolderMut<'r> {
        ResourcesHolderMut::new(self)
    }
}

pub struct ResourcesHolderUnsafeRef<'r> {
    res: &'r ResourcesHolderUnsafeFfi,
    types: &'r TypesRegistryCache,
}

impl<'r> ResourcesHolderUnsafeRef<'r> {
    pub fn new(res: &'r ResourcesHolderUnsafeFfi, types: &'r TypesRegistryCache) -> Self {
        Self {
            res,
            types,
        }
    }

    pub fn get<T: 'static>(&self) -> Option<*mut T> {
        resources_holder_typed_unsafe_get(&self.types, &self.res)
    }

    pub fn as_ref(&'r self) -> ResourcesHolderUnsafeRef<'r> {
        ResourcesHolderUnsafeRef {
            res: &self.res,
            types: self.types,
        }
    }

    pub fn into_safe(self) -> ResourcesHolderRef<'r> {
        ResourcesHolderRef::new(self)
    }
}

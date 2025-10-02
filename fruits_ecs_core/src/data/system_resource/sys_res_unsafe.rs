use crate::*;

fn resources_holder_typed_get_or_insert<'a, T: 'static + Default>(
    res: &'a SystemResourcesHolderUnsafeFfi,
    types: &TypesRegistryCache,
) -> *mut T {
    let type_id = types.get_or_register::<T>();

    unsafe {
        let result = res.get_or_insert(type_id);

        let ptr = result.ptr as *mut T;

        if result.is_new {
            ptr.write(T::default());
        }

        ptr
    }
}

// todo: refactor to use real references instead of lifetimed structs.

pub struct SystemResourcesUnsafeHolder {
    res: SystemResourcesHolderUnsafeFfi,
    types: TypesRegistryCache,
}

impl SystemResourcesUnsafeHolder {
    pub fn new(res: SystemResourcesHolderUnsafeFfi, types: TypesRegistryCache) -> Self {
        Self {
            res,
            types,
        }
    }

    pub fn get_or_insert<T: 'static + Default>(&self) -> *mut T {
        resources_holder_typed_get_or_insert(&self.res, &self.types)
    }
    
    pub fn as_mut<'r>(&'r mut self) -> SystemResourcesUnsafeHolderMut<'r> {
        SystemResourcesUnsafeHolderMut {
            res: &mut self.res,
            types: &self.types,
        }
    }

    pub fn as_ref<'r>(&'r self) -> SystemResourcesUnsafeHolderRef<'r> {
        SystemResourcesUnsafeHolderRef {
            res: &self.res,
            types: &self.types,
        }
    }
}

//

pub struct SystemResourcesUnsafeHolderMut<'r> {
    res: &'r mut SystemResourcesHolderUnsafeFfi,
    types: &'r TypesRegistryCache,
}

impl<'r> SystemResourcesUnsafeHolderMut<'r> {
    pub fn new(res: &'r mut SystemResourcesHolderUnsafeFfi, types: &'r TypesRegistryCache) -> Self {
        Self {
            res,
            types,
        }
    }

    pub fn get_or_insert<T: 'static + Default>(&self) -> *mut T {
        resources_holder_typed_get_or_insert(self.res, &self.types)
    }

    pub fn as_mut(&'r mut self) -> SystemResourcesUnsafeHolderMut<'r> {
        SystemResourcesUnsafeHolderMut {
            res: &mut self.res,
            types: self.types,
        }
    }

    pub fn as_ref(&'r self) -> SystemResourcesUnsafeHolderRef<'r> {
        SystemResourcesUnsafeHolderRef {
            res: self.res,
            types: self.types,
        }
    }
}

//

pub struct SystemResourcesUnsafeHolderRef<'r> {
    res: &'r SystemResourcesHolderUnsafeFfi,
    types: &'r TypesRegistryCache,
}

impl<'r> SystemResourcesUnsafeHolderRef<'r> {
    pub fn new(res: &'r SystemResourcesHolderUnsafeFfi, types: &'r TypesRegistryCache) -> Self {
        Self {
            res,
            types,
        }
    }

    pub fn get_or_insert<T: 'static + Default>(&self) -> *mut T {
        resources_holder_typed_get_or_insert(self.res, &self.types)
    }

    pub fn as_ref(&'r self) -> SystemResourcesUnsafeHolderRef<'r> {
        SystemResourcesUnsafeHolderRef {
            res: self.res,
            types: self.types,
        }
    }
}
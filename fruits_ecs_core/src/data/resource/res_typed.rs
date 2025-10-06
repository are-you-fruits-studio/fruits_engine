use crate::*;

fn resources_holder_get_ptr<T: 'static>(res: &ResourcesHolderUnsafeFfi, types: &TypesRegistryCache) -> Option<*mut T> {
    unsafe {
        let mem = res.get(types.get_or_register::<T>());

        if mem.is_null() {
            return None;
        }

        Some(mem as *mut T)
    }
}

fn resources_holder_get<'r, T: 'static>(res: &'r ResourcesHolderUnsafeFfi, types: &TypesRegistryCache) -> Option<&'r T> {
    unsafe { resources_holder_get_ptr(res, types).map(|p| &*p) }
}

fn resources_holder_get_mut<'r, T: 'static>(res: &'r mut ResourcesHolderUnsafeFfi, types: &TypesRegistryCache) -> Option<&'r mut T> {
    unsafe { resources_holder_get_ptr(res, types).map(|p| &mut *p) }
}

fn resources_holder_insert<T: 'static>(res: &mut ResourcesHolderUnsafeFfi, types: &TypesRegistryCache, data: T) -> Result<(), T> {
    unsafe {
        let mem = res.insert(types.get_or_register::<T>());

        if mem.is_null() {
            return Err(data);
        };

        (mem as *mut T).write(data);

        Ok(())
    }
}

//

pub struct ResourcesHolderMut<'r> {
    res: &'r mut ResourcesHolderUnsafeFfi,
    types: &'r TypesRegistryCache,
}

impl<'r> ResourcesHolderMut<'r> {
    pub fn new(res: &'r mut ResourcesHolderUnsafeFfi, types: &'r TypesRegistryCache) -> Self {
        Self {
            res,
            types,
        }
    }

    pub fn insert<T: 'static>(&mut self, data: T) -> Result<(), T> {
        resources_holder_insert::<T>(self.res, self.types, data)
    }

    pub fn get_ptr<T: 'static>(&self) -> Option<*mut T> {
        resources_holder_get_ptr::<T>(self.res, self.types)
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        resources_holder_get::<T>(self.res, self.types)
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        resources_holder_get_mut::<T>(self.res, self.types)
    }

    pub fn as_mut(&'r mut self) -> ResourcesHolderMut<'r> {
        ResourcesHolderMut {
            res: self.res,
            types: self.types,
        }
    }

    pub fn as_ref(&'r self) -> ResourcesHolderRef<'r> {
        ResourcesHolderRef {
            res: self.res,
            types: self.types,
        }
    }
}

//

#[derive(Copy, Clone)]
pub struct ResourcesHolderRef<'r> {
    res: &'r ResourcesHolderUnsafeFfi,
    types: &'r TypesRegistryCache,
}

impl<'r> ResourcesHolderRef<'r> {
    pub fn new(res: &'r ResourcesHolderUnsafeFfi, types: &'r TypesRegistryCache) -> Self {
        Self {
            res,
            types,
        }
    }

    pub fn get_ptr<T: 'static>(self) -> Option<*mut T> {
        resources_holder_get_ptr::<T>(self.res, self.types)
    }

    pub fn get<T: 'static>(self) -> Option<&'r T> {
        resources_holder_get::<T>(self.res, self.types)
    }
}
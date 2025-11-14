use crate::*;

pub trait Resource: 'static + Send + Sync {}

//

fn resources_holder_get_ptr<T: Resource>(res: &ResourcesHolderUnsafeFfi, types: &TypesRegistryCache) -> Option<*mut T> {
    unsafe {
        let mem = res.get(types.get_or_register::<T>());

        if mem.is_null() {
            return None;
        }

        Some(mem as *mut T)
    }
}

fn resources_holder_get<'r, T: Resource>(res: &'r ResourcesHolderUnsafeFfi, types: &TypesRegistryCache) -> Option<&'r T> {
    unsafe { resources_holder_get_ptr(res, types).map(|p| &*p) }
}

fn resources_holder_get_mut<'r, T: Resource>(
    res: &'r mut ResourcesHolderUnsafeFfi,
    types: &TypesRegistryCache,
) -> Option<&'r mut T> {
    unsafe { resources_holder_get_ptr(res, types).map(|p| &mut *p) }
}

fn resources_holder_insert<T: Resource>(res: &mut ResourcesHolderUnsafeFfi, types: &TypesRegistryCache, data: T) -> Result<(), T> {
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
        Self { res, types }
    }

    pub fn insert<T: Resource>(&mut self, data: T) -> Result<(), T> {
        resources_holder_insert::<T>(self.res, self.types, data)
    }

    pub fn get_ptr<T: Resource>(&self) -> Option<*mut T> {
        resources_holder_get_ptr::<T>(self.res, self.types)
    }

    pub fn get<'p, T: Resource>(&'p self) -> Option<&'p T>
    where
        'r: 'p,
    {
        resources_holder_get::<T>(self.res, self.types)
    }

    pub fn get_mut<'p, T: Resource>(&'p mut self) -> Option<&'p mut T>
    where
        'r: 'p,
    {
        resources_holder_get_mut::<T>(self.res, self.types)
    }

    pub fn as_mut<'p>(&'p mut self) -> ResourcesHolderMut<'p>
    where
        'r: 'p,
    {
        ResourcesHolderMut {
            res: self.res,
            types: self.types,
        }
    }

    pub fn as_ref<'p>(&'p self) -> ResourcesHolderRef<'p>
    where
        'r: 'p,
    {
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
        Self { res, types }
    }

    pub fn get_ptr<T: Resource>(self) -> Option<*mut T> {
        resources_holder_get_ptr::<T>(self.res, self.types)
    }

    pub fn get<T: Resource>(self) -> Option<&'r T> {
        resources_holder_get::<T>(self.res, self.types)
    }
}

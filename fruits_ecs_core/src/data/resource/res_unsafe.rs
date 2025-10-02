use crate::*;

// todo?: refactor to use real references instead of lifetimed structs.

#[derive(Clone)]
pub struct ResourcesHolderUnsafeRef<'r> {
    res: *mut ResourcesHolderUnsafeFfi,
    types: &'r TypesRegistryCache,
}

impl<'r> ResourcesHolderUnsafeRef<'r> {
    pub fn new(res: *mut ResourcesHolderUnsafeFfi, types: &'r TypesRegistryCache) -> Self {
        Self {
            res,
            types,
        }
    }

    pub unsafe fn get<T: 'static>(&self) -> Option<*mut T> {
        unsafe {
            let mem = (&*self.res).get((&self.types).get_or_register::<T>());

            if mem.is_null() {
                return None;
            }

            Some(mem as *mut T)
        }
    }
    
    pub unsafe fn insert<T: 'static>(&mut self, data: T) -> Result<(), T> {
        unsafe {
            let mem = (&mut *self.res).insert(self.types.get_or_register::<T>());
    
            if mem.is_null() {
                return Err(data);
            };
    
            (mem as *mut T).write(data);

            Ok(())
        }
    }

    pub fn into_safe(self) -> ResourcesHolderMut<'r> {
        ResourcesHolderMut::new(self)
    }
}

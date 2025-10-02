use crate::*;

#[repr(transparent)]
pub struct ResourcesHolder {
    res: ResourcesHolderUnsafe,
}

impl ResourcesHolder {
    pub fn new(res: ResourcesHolderUnsafe) -> Self {
        Self {
            res,
        }
    }

    pub fn insert<T: 'static>(&mut self, data: T) -> Result<(), T> {
        self.res.insert(data)
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        unsafe { self.res.get().map(|p| &*p) }
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        unsafe { self.res.get().map(|p| &mut *p) }
    }

    pub fn as_mut<'r>(&'r mut self) -> ResourcesHolderMut<'r> {
        ResourcesHolderMut {
            res: self.res.as_mut(),
        }
    }

    pub fn as_ref<'r>(&'r self) -> ResourcesHolderRef<'r> {
        ResourcesHolderRef {
            res: self.res.as_ref(),
        }
    }
}

#[repr(transparent)]
pub struct ResourcesHolderMut<'r> {
    res: ResourcesHolderUnsafeMut<'r>,
}

impl<'r> ResourcesHolderMut<'r> {
    pub fn new(res: ResourcesHolderUnsafeMut<'r>) -> Self {
        Self {
            res,
        }
    }

    pub fn insert<T: 'static>(&mut self, data: T) -> Result<(), T> {
        self.res.insert(data)
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        unsafe { self.res.get().map(|p| &*p) }
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        unsafe { self.res.get().map(|p| &mut *p) }
    }

    pub fn as_mut(&'r mut self) -> ResourcesHolderMut<'r> {
        ResourcesHolderMut {
            res: self.res.as_mut(),
        }
    }

    pub fn as_ref(&'r self) -> ResourcesHolderRef<'r> {
        ResourcesHolderRef {
            res: self.res.as_ref(),
        }
    }
}

#[repr(transparent)]
pub struct ResourcesHolderRef<'r> {
    res: ResourcesHolderUnsafeRef<'r>,
}

impl<'r> ResourcesHolderRef<'r> {
    pub fn new(res: ResourcesHolderUnsafeRef<'r>) -> Self {
        Self {
            res,
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        unsafe { self.res.get().map(|p| &*p) }
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        unsafe { self.res.get().map(|p| &mut *p) }
    }

    pub fn as_ref(&'r self) -> ResourcesHolderRef<'r> {
        ResourcesHolderRef {
            res: self.res.as_ref(),
        }
    }
}
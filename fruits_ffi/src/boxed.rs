use std::{
    fmt::{Debug, Display}, marker::PhantomData, ops::{Deref, DerefMut}
};

use crate::FfiAny;

#[repr(transparent)]
pub struct FfiBox<T> {
    data: FfiAny,
    _phantom: PhantomData<fn(T) -> T>,
}

impl<T> FfiBox<T> {
    pub fn new(value: T) -> Self {
        Self {
            data: FfiAny::new(value),
            _phantom: PhantomData,
        }
    }

    pub(crate) unsafe fn from_ffi_any(any: FfiAny) -> Self {
        Self {
            data: any,
            _phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> T {
        unsafe {
            let ptr = self.data.ptr();
            let ptr = ptr as *mut T;

            let val = ptr.read();
            
            self.data.dealloc_without_drop();

            val
        }
    }

    pub fn into_any(self) -> FfiAny {
        self.data
    }

    pub fn as_ref(&self) -> &T {
        unsafe { &*(self.data.ptr() as *mut T) }
    }

    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *(self.data.ptr() as *mut T) }
    }
}

impl<T> Deref for FfiBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for FfiBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: Debug> Debug for FfiBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: Display> Display for FfiBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&**self, f)
    }
}

unsafe impl<T: Send> Send for FfiBox<T> {}
unsafe impl<T: Sync> Sync for FfiBox<T> {}

use std::{ffi::c_void, fmt::Debug, mem::ManuallyDrop, ops::{Deref, DerefMut}};

use crate::{FfiOpaqueMemory, FfiTypedMemory};

#[repr(transparent)]
pub struct FfiBox<T> {
    mem: FfiTypedMemory<T>,
}

impl<T> FfiBox<T> {
    pub fn new(v: T) -> Self {
        unsafe {
            let data = FfiTypedMemory::<T>::new();
            data.as_ptr().write(v);
            Self {
                mem: data,
            }
        }
    }

    pub fn into_inner(self) -> T {
        unsafe {
            let this = ManuallyDrop::new(self);

            let mem = (&raw const this.mem).read();

            let val = mem.as_ptr().read();

            drop(mem);

            val
        }
    }

    pub fn into_opaque(self) -> FfiOpaqueBox {
        unsafe extern "C" fn ffi_drop<D>(data: *mut c_void) {
            unsafe { (data as *mut D).drop_in_place() };
        }
        
        unsafe {
            let this = ManuallyDrop::new(self);

            let mem = (&raw const this.mem).read();

            FfiOpaqueBox {
                mem: mem.into_opaque(),
                drop_fn: ffi_drop::<T>,
            }
        }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { &*self.mem.as_ptr() }
    }

    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.mem.as_ptr() }
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

impl<T> Drop for FfiBox<T> {
    fn drop(&mut self) {
        unsafe {
            self.mem.as_ptr().drop_in_place();
        }
    }
}

impl<T: Debug> Debug for FfiBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&**self, f)
    }
}

unsafe impl<T: Send> Send for FfiBox<T> { }
unsafe impl<T: Sync> Sync for FfiBox<T> { }

//

#[repr(C)]
pub struct FfiOpaqueBox {
    mem: FfiOpaqueMemory,
    drop_fn: unsafe extern "C" fn(*mut c_void),
}

impl FfiOpaqueBox {
    pub fn new<T>(v: T) -> Self {
        FfiBox::<T>::new(v).into_opaque()
    }

    pub unsafe fn into_typed<T>(self) -> FfiBox<T> {
        unsafe {
            let this = ManuallyDrop::new(self);

            let mem = (&raw const this.mem).read();

            FfiBox::<T> {
                mem: mem.into_typed::<T>(),
            }
        }
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.mem.as_ptr()
    }
}

impl Drop for FfiOpaqueBox {
    fn drop(&mut self) {
        unsafe {
            self.mem.as_ptr().drop_in_place();
        }
    }
}
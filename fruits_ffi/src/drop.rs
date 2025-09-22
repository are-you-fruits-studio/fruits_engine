use std::{alloc::Layout, ffi::c_void, mem::ManuallyDrop};

#[repr(C)]
pub struct FfiDroppableData {
    data: *mut c_void,
    drop_fn: unsafe extern "C" fn(*mut c_void),
    dealloc_fn: unsafe extern "C" fn(*mut c_void),
}

impl FfiDroppableData {
    pub fn new<T>(data: T) -> Self {
        unsafe extern "C" fn ffi_drop<D>(data: *mut c_void) {
            unsafe { (data as *mut D).drop_in_place() };
        }

        unsafe extern "C" fn ffi_dealloc<D>(data: *mut c_void) {
            unsafe { std::alloc::dealloc(data as *mut u8, Layout::new::<D>()) };
        }

        unsafe {
            let mem = std::alloc::alloc(Layout::new::<T>()) as *mut T;

            mem.write(data);
            
            Self {
                data: mem as *mut c_void,
                drop_fn: ffi_drop::<T>,
                dealloc_fn: ffi_dealloc::<T>,
            }
        }
    }

    pub fn into_unsafe<T>(self) -> T {
        unsafe {
            let this = ManuallyDrop::new(self);
            
            let data = (this.data as *mut T).read();
            // todo: no dealloc if drop panics.
            
            (this.dealloc_fn)(this.data);
            
            data
        }
    }

    pub fn as_ptr(&self) -> *const c_void {
        self.data
    }

    pub fn as_mut_ptr(&mut self) -> *mut c_void {
        self.data
    }
}

impl Drop for FfiDroppableData {
    fn drop(&mut self) {
        unsafe {
            (self.drop_fn)(self.data);
            (self.dealloc_fn)(self.data);
        }
    }
}
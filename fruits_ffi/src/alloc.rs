use std::{alloc::{GlobalAlloc, Layout}, ffi::c_void};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiAllocator {
    alloc_fn: unsafe extern "C" fn(u64, u64) -> *mut c_void,
    dealloc_fn: unsafe extern "C" fn(*mut c_void, u64, u64),
}

impl FfiAllocator {
    pub fn from_system() -> Self {
        Self {
            alloc_fn: Self::c_alloc_system,
            dealloc_fn: Self::c_dealloc_system,
        }
    }

    pub fn from_global() -> Self {
        Self {
            alloc_fn: Self::c_alloc_global,
            dealloc_fn: Self::c_dealloc_global,
        }
    }

    // todo
    pub unsafe fn alloc(&self, size: u64, align: u64) -> *mut u8 {
        // todo
        unsafe {
            (self.alloc_fn)(size, align) as *mut u8
        }
    }

    // todo
    pub unsafe fn dealloc(&self, ptr: *mut u8, size: u64, align: u64) {
        // todo
        unsafe {
            (self.dealloc_fn)(ptr as *mut c_void, size, align);
        }
    }

    unsafe extern "C" fn c_alloc_system(size: u64, align: u64) -> *mut c_void {
        unsafe {
            let layout = Layout::from_size_align_unchecked(size as usize, align as usize);

            std::alloc::System.alloc(layout) as *mut c_void
        }
    }

    unsafe extern "C" fn c_dealloc_system(ptr: *mut c_void, size: u64, align: u64) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(size as usize, align as usize);

            std::alloc::System.dealloc(ptr as *mut u8, layout);
        }
    }

    unsafe extern "C" fn c_alloc_global(size: u64, align: u64) -> *mut c_void {
        unsafe {
            let layout = Layout::from_size_align_unchecked(size as usize, align as usize);

            std::alloc::alloc(layout) as *mut c_void
        }
    }

    unsafe extern "C" fn c_dealloc_global(ptr: *mut c_void, size: u64, align: u64) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(size as usize, align as usize);

            std::alloc::dealloc(ptr as *mut u8, layout);
        }
    }
}

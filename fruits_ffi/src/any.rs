use std::{alloc::Layout, ffi::c_void, mem::ManuallyDrop};

use crate::{FfiBox, FfiStrSliceRef};

#[repr(C)]
pub struct FfiAny {
    ptr: *mut c_void,
    meta: &'static FfiAnyMetadata,
}

impl FfiAny {
    pub fn new<T>(value: T) -> Self {
        unsafe {
            let ptr = std::alloc::alloc(Layout::new::<T>());

            (ptr as *mut T).write(value);
            
            Self {
                ptr: ptr as *mut c_void,
                meta: const { &FfiAnyMetadata::new::<T>() },
            }
        }
    }

    pub fn ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub const fn size(&self) -> u64 {
        self.meta.size
    }
    pub const fn align(&self) -> u64 {
        self.meta.align
    }
    pub fn name(&self) -> &'static str {
        unsafe { (self.meta.name_fn)().into_slice::<'static>() }
    }

    pub unsafe fn into_box<T>(self) -> FfiBox<T> {
        unsafe { FfiBox::from_ffi_any(self) }
    }

    pub unsafe fn dealloc_without_drop(self) {
        let this = ManuallyDrop::new(self);

        unsafe { (this.meta.dealloc_fn)(this.ptr) };
    }
}

impl Drop for FfiAny {
    fn drop(&mut self) {
        struct FfiAnyMemoryToDealloc {
            ptr: *mut c_void,
            meta: &'static FfiAnyMetadata,
        }

        impl Drop for FfiAnyMemoryToDealloc {
            fn drop(&mut self) {
                unsafe { (self.meta.dealloc_fn)(self.ptr) };
            }
        }

        unsafe {
            let to_dealloc = FfiAnyMemoryToDealloc {
                ptr: self.ptr,
                meta: self.meta,
            };

            (self.meta.drop_in_place_fn)(self.ptr);

            drop(to_dealloc);
        }
    }
}

//

#[repr(C)]
pub struct FfiAnyMetadata {
    pub size: u64,
    pub align: u64,
    pub name_fn: unsafe extern "C" fn() -> FfiStrSliceRef,
    pub drop_in_place_fn: unsafe extern "C" fn(*mut c_void),
    pub dealloc_fn: unsafe extern "C" fn(*mut c_void),
}

impl FfiAnyMetadata {
    pub const fn new<T>() -> Self {
        unsafe extern "C" fn ffi_name<T>() -> FfiStrSliceRef {
            unsafe { FfiStrSliceRef::from_slice(std::any::type_name::<T>()) }
        }
        unsafe extern "C" fn ffi_drop_in_place<T>(ptr: *mut c_void) {
            unsafe { std::ptr::drop_in_place(ptr as *mut T) }
        }
        unsafe extern "C" fn ffi_dealloc<T>(ptr: *mut c_void) {
            unsafe { std::alloc::dealloc(ptr as *mut u8, Layout::new::<T>()) }
        }

        Self {
            size: std::mem::size_of::<T>() as u64,
            align: std::mem::align_of::<T>() as u64,
            name_fn: ffi_name::<T>,
            drop_in_place_fn: ffi_drop_in_place::<T>,
            dealloc_fn: ffi_dealloc::<T>,
        }
    }
}
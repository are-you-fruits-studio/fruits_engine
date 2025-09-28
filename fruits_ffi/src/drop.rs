use std::ffi::c_void;

pub struct FfiDroppable {
    data: *mut c_void,
    drop_fn: unsafe extern "C" fn(*mut c_void),
}

impl FfiDroppable {
    pub fn new<T>(data: T) -> Self {
        unsafe extern "C" fn ffi_drop<D>(data: *mut c_void) {
            unsafe { drop(Box::from_raw(data as *mut D)) }
        }

        Self {
            data: Box::into_raw(Box::new(data)) as *mut c_void,
            drop_fn: ffi_drop::<T>,
        }
    }

    pub fn get(&self) -> *mut c_void {
        self.data
    }
}

impl Drop for FfiDroppable {
    fn drop(&mut self) {
        unsafe { (self.drop_fn)(self.data) }
    }
}
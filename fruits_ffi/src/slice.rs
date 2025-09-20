

pub struct FfiSliceMut<T> {
    ptr: *mut T,
    len: u64,
}

impl<T> FfiSliceMut<T> {
    // todo
    pub unsafe fn into_slice_unsafe<'a>(self) -> &'a mut [T] {
        // todo
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, self.len as usize)
        }
    }

    pub fn as_slice(&self) -> &[T] {
        // todo
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.len as usize)
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        // todo
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, self.len as usize)
        }
    }
    
    // todo
    pub unsafe fn from_slice(slice: &mut [T]) -> Self {
        Self {
            len: slice.len() as u64,
            ptr: (&raw mut *slice) as *mut T,
        }
    }
}

pub struct FfiSliceRef<T> {
    ptr: *const T,
    len: u64,
}

impl<T> FfiSliceRef<T> {
    // todo
    pub unsafe fn into_slice_unsafe<'a>(self) -> &'a [T] {
        // todo
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.len as usize)
        }
    }

    pub fn as_slice(&self) -> &[T] {
        // todo
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.len as usize)
        }
    }
    
    // todo
    pub unsafe fn from_slice(slice: &[T]) -> Self {
        // todo
        unsafe {
            Self {
                len: slice.len() as u64,
                ptr: (&raw const *slice) as *const T,
            }
        }
    }
}
#[repr(C)]
pub struct FfiSliceMut<T> {
    ptr: *mut T,
    len: u64,
}

impl<T> FfiSliceMut<T> {
    // todo
    pub unsafe fn into_slice_mut<'a>(self) -> &'a mut [T] {
        // todo
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len as usize) }
    }

    // todo
    pub unsafe fn into_slice<'a>(self) -> &'a [T] {
        // todo
        unsafe { std::slice::from_raw_parts(self.ptr, self.len as usize) }
    }

    // todo
    pub unsafe fn from_slice(slice: &mut [T]) -> Self {
        Self {
            len: slice.len() as u64,
            ptr: (&raw mut *slice) as *mut T,
        }
    }
}

#[repr(C)]
pub struct FfiSliceRef<T> {
    ptr: *const T,
    len: u64,
}

impl<T> FfiSliceRef<T> {
    // todo
    pub unsafe fn into_slice<'a>(self) -> &'a [T] {
        // todo
        unsafe { std::slice::from_raw_parts(self.ptr, self.len as usize) }
    }

    // todo
    pub unsafe fn from_slice(slice: &[T]) -> Self {
        Self {
            len: slice.len() as u64,
            ptr: (&raw const *slice) as *const T,
        }
    }
}

//

#[repr(C)]
pub struct FfiStrSliceMut {
    bytes: FfiSliceMut<u8>,
}

impl FfiStrSliceMut {
    // todo
    pub unsafe fn into_slice_mut<'a>(self) -> &'a mut str {
        // todo
        unsafe { std::str::from_utf8_unchecked_mut(self.bytes.into_slice_mut()) }
    }

    // todo
    pub unsafe fn into_slice<'a>(self) -> &'a str {
        // todo
        unsafe { std::str::from_utf8_unchecked(self.bytes.into_slice()) }
    }

    // todo
    pub unsafe fn from_slice(slice: &mut str) -> Self {
        unsafe {
            Self {
                bytes: FfiSliceMut::from_slice(slice.as_bytes_mut()),
            }
        }
    }
}

#[repr(C)]
pub struct FfiStrSliceRef {
    bytes: FfiSliceRef<u8>,
}

impl FfiStrSliceRef {
    // todo
    pub unsafe fn into_slice<'a>(self) -> &'a str {
        // todo
        unsafe { std::str::from_utf8_unchecked(self.bytes.into_slice()) }
    }

    // todo
    pub unsafe fn from_slice(slice: &str) -> Self {
        unsafe {
            Self {
                bytes: FfiSliceRef::from_slice(slice.as_bytes()),
            }
        }
    }
}

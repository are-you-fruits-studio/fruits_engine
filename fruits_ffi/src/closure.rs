use std::{ffi::c_void, marker::PhantomData};

#[repr(C)]
pub struct FfiFnRef<'a, I, O> {
    data: *const c_void,
    fn_execute: unsafe extern "C" fn(this: *const c_void, input: I) -> O,
    _phantom: PhantomData<&'a ()>,
}
impl<'a, I, O> FfiFnRef<'a, I, O> {
    pub fn new<F: Fn(I) -> O>(f: &'a F) -> Self {
        unsafe extern "C" fn ffi_execute<I, O, F: Fn(I) -> O>(this: *const c_void, input: I) -> O {
            unsafe {
                let this = &*(this as *const F);

                this(input)
            }
        }

        Self {
            data: f as *const F as *const c_void,
            fn_execute: ffi_execute::<I, O, F>,
            _phantom: Default::default(),
        }
    }

    pub fn execute(&self, input: I) -> O {
        unsafe {
            (self.fn_execute)(self.data, input)
        }
    }
}

#[repr(C)]
pub struct FfiFnMutMut<'a, I, O> {
    data: *mut c_void,
    fn_execute: unsafe extern "C" fn(this: *mut c_void, input: I) -> O,
    _phantom: PhantomData<&'a mut ()>,
}
impl<'a, I, O> FfiFnMutMut<'a, I, O> {
    pub fn new<F: Fn(I) -> O>(f: &'a mut F) -> Self {
        unsafe extern "C" fn ffi_execute<I, O, F: Fn(I) -> O>(this: *mut c_void, input: I) -> O {
            unsafe {
                let this = &mut *(this as *mut F);

                this(input)
            }
        }

        Self {
            data: f as *mut F as *mut c_void,
            fn_execute: ffi_execute::<I, O, F>,
            _phantom: Default::default(),
        }
    }

    pub fn execute(&mut self, input: I) -> O {
        unsafe {
            (self.fn_execute)(self.data, input)
        }
    }
}
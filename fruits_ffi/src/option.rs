use std::{fmt::Debug, mem::MaybeUninit};

#[repr(C)]
pub struct FfiOption<T> {
    data: MaybeUninit<T>,
    is_init: bool,
}

impl<T> FfiOption<T> {
    pub fn from_option(v: Option<T>) -> Self {
        match v {
            None => Self {
                data: MaybeUninit::uninit(),
                is_init: false,
            },
            Some(v) => Self {
                data: MaybeUninit::new(v),
                is_init: true,
            },
        }
    }

    pub const fn into_option(self) -> Option<T> {
        unsafe {
            if self.is_init {
                Some(self.data.assume_init())
            } else {
                None
            }
        }
    }

    pub const fn as_ref(&self) -> Option<&T> {
        unsafe {
            if self.is_init {
                Some(self.data.assume_init_ref())
            } else {
                None
            }
        }
    }

    pub const fn as_mut(&mut self) -> Option<&mut T> {
        unsafe {
            if self.is_init {
                Some(self.data.assume_init_mut())
            } else {
                None
            }
        }
    }
}

impl<T: Debug> Debug for FfiOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = self.as_ref() {
            f.debug_tuple("FfiOption::Some")
                .field(v)
                .finish()
        } else {
            f.debug_tuple("FfiOption::None")
                .finish()
        }
    }
}

impl<T> From<Option<T>> for FfiOption<T> {
    fn from(value: Option<T>) -> Self {
        Self::from_option(value)
    }
}

impl<T> From<FfiOption<T>> for Option<T> {
    fn from(value: FfiOption<T>) -> Self {
        value.into_option()
    }
}

// todo: impl standard traits
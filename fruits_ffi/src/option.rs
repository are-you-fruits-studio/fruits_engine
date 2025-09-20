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

    pub const fn is_some(&self) -> bool {
        self.is_init
    }

    pub const fn is_none(&self) -> bool {
        !self.is_init
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

impl<T> From<T> for FfiOption<T> {
    fn from(value: T) -> Self {
        Some(value).into()
    }
}

impl<T: Copy> Copy for FfiOption<T> { }

impl<T: Clone> Clone for FfiOption<T> {
    fn clone(&self) -> Self {
        match self.as_ref() {
            Some(v) => Self::from_option(Some(v.clone())),
            None => Self::from_option(None),
        }
    }
}

impl<T: PartialEq> PartialEq for FfiOption<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self.as_ref(), other.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(l), Some(r)) => l == r,
        }
    }
}

impl<T: Eq> Eq for FfiOption<T> { }

// todo: impl standard traits
use std::{fmt::Debug, mem::MaybeUninit};

#[repr(C)]
pub struct FfiOption<T> {
    data: MaybeUninit<T>,
    is_init: bool,
}

#[repr(C)]
pub struct FfiOptionCopy<T: Copy> {
    data: MaybeUninit<T>,
    is_init: bool,
}

macro_rules! impl_option {
    ($I: ident$(: $B: ident)?) => {
        impl<T$(: $B)?> $I<T> {
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
        
            pub fn into_option(mut self) -> Option<T> {
                unsafe {
                    if self.is_init {
                        self.is_init = false;
                        Some(self.data.assume_init_read())
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

        impl<T: Debug$( + $B)?> Debug for $I<T> {
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
    
        impl<T$(: $B)?> From<Option<T>> for $I<T> {
            fn from(value: Option<T>) -> Self {
                Self::from_option(value)
            }
        }
        
        impl<T$(: $B)?> From<$I<T>> for Option<T> {
            fn from(value: $I<T>) -> Self {
                value.into_option()
            }
        }
        
        impl<T$(: $B)?> From<T> for $I<T> {
            fn from(value: T) -> Self {
                Some(value).into()
            }
        }
        
        impl<T: Clone$( + $B)?> Clone for $I<T> {
            fn clone(&self) -> Self {
                match self.as_ref() {
                    Some(v) => Self::from_option(Some(v.clone())),
                    None => Self::from_option(None),
                }
            }
        }
        
        impl<T: PartialEq$( + $B)?> PartialEq for $I<T> {
            fn eq(&self, other: &Self) -> bool {
                match (self.as_ref(), other.as_ref()) {
                    (None, None) => true,
                    (None, Some(_)) => false,
                    (Some(_), None) => false,
                    (Some(l), Some(r)) => l == r,
                }
            }
        }
        
        impl<T: Eq$( + $B)?> Eq for $I<T> { }
    };
}

impl_option!{ FfiOption }
impl_option!{ FfiOptionCopy: Copy }

impl<T: Copy> Copy for FfiOptionCopy<T> { }

impl<T> Drop for FfiOption<T> {
    fn drop(&mut self) {
        if self.is_init {
            unsafe {
                self.is_init = false;
                self.data.assume_init_drop();
            }
        }
    }
}

// todo: impl standard traits
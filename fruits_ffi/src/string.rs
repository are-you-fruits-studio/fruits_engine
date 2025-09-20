use std::ops::{Deref, DerefMut};

use crate::FfiVec;

pub struct FfiString {
    data: FfiVec<u8>,
}

impl FfiString {
    pub fn from_string(v: String) -> Self {
        Self {
            data: v.into_bytes().into(),
        }
    }

    pub const fn as_str(&self) -> &str {
        // todo
        unsafe {
            str::from_utf8_unchecked(self.data.as_slice())
        }
    }

    pub const fn as_mut_str(&mut self) -> &mut str {
        // todo
        unsafe {
            str::from_utf8_unchecked_mut(self.data.as_slice_mut())
        }
    }
}

impl Deref for FfiString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl DerefMut for FfiString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_str()
    }
}

impl From<String> for FfiString {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}
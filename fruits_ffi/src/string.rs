use std::{fmt::{Debug, Display, Write}, ops::{Deref, DerefMut}};

use crate::FfiVec;

// todo
#[repr(C)]
#[derive(Default)]
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

    pub fn clear(&mut self) {
        self.data.clear();
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

impl Debug for FfiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for FfiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl PartialEq for FfiString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Write for FfiString {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        // todo: optimize
        for &byte in s.as_bytes() {
            self.data.push(byte);
        };

        Ok(())
    }
}

impl Eq for FfiString { }

// todo: impl standard traits
use std::{fmt::Debug, ops::{Deref, DerefMut}};

use crate::FfiAllocator;

// todo
#[repr(C)]
pub struct FfiVec<T> {
    ptr: *mut T,
    cap: u64,
    len: u64,
    allocator: FfiAllocator,
}

impl<T> FfiVec<T> {
    pub const fn new() -> Self {
        let cap = if std::mem::size_of::<T>() == 0 {
            ZERO_SIZED_CAP
        } else {
            0
        };

        Self {
            allocator: FfiAllocator::from_global(),
            cap,
            len: 0,
            ptr: std::ptr::null_mut(),
        }
    }

    pub fn with_capacity(mut cap: u64) -> Self {
        let allocator = FfiAllocator::from_global();

        let ptr;

        if std::mem::size_of::<T>() == 0 {
            ptr = std::ptr::null_mut();
            cap = ZERO_SIZED_CAP;
        } else {
            ptr = unsafe {
                allocator.alloc(std::mem::size_of::<T>() as u64 * cap, std::mem::align_of::<T>() as u64) as *mut T
            };
        }

        Self {
            allocator,
            cap,
            len: 0,
            ptr,
        }
    }

    pub fn from_vec(mut vec: Vec<T>) -> Self {
        let ffi_vec = if std::mem::size_of::<T>() == 0 {
            Self {
                ptr: std::ptr::null_mut(),
                cap: ZERO_SIZED_CAP,
                len: vec.len() as u64,
                allocator: FfiAllocator::from_global(),
            }
        } else {
            Self {
                ptr: vec.as_mut_ptr(),
                cap: vec.capacity() as u64,
                len: vec.len() as u64,
                allocator: FfiAllocator::from_global(),
            }
        };

        std::mem::forget(vec);

        ffi_vec
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn capacity(&self) -> u64 {
        self.cap
    }

    pub fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            return &[];
        }

        let ptr = if std::mem::size_of::<T>() == 0 {
            std::ptr::NonNull::dangling().as_ptr()
        } else {
            self.ptr
        };
        
        // todo
        unsafe {
            std::slice::from_raw_parts(ptr, self.len as usize)
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        if self.len == 0 {
            return &mut [];
        }

        let ptr = if std::mem::size_of::<T>() == 0 {
            std::ptr::NonNull::dangling().as_ptr()
        } else {
            self.ptr
        };

        // todo
        unsafe {
            std::slice::from_raw_parts_mut(ptr, self.len as usize)
        }
    }

    pub fn push(&mut self, v: T) {
        if std::mem::size_of::<T>() == 0 {
            self.len += 1;
            return;
        }

        if self.len == self.cap {
            let new_cap = if self.cap == 0 { 4 } else { self.cap * 2 };

            let size_of_t = std::mem::size_of::<T>() as u64;
            let align_of_t = std::mem::align_of::<T>() as u64;

            unsafe {
                let new_ptr = self.allocator.alloc(size_of_t * new_cap, align_of_t);

                if std::mem::size_of::<T>() != 0 && self.cap != 0 {
                    std::ptr::copy_nonoverlapping(self.ptr as *const T, new_ptr as *mut T, self.len as usize);

                    self.allocator.dealloc(self.ptr as *mut u8, size_of_t * self.cap, align_of_t);
                }

                self.ptr = new_ptr as *mut T;
                self.cap = new_cap;
            }
        }

        unsafe {
            self.ptr.add(self.len as usize).write(v);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        if std::mem::size_of::<T>() == 0 {
            Some(unsafe {
                std::mem::zeroed::<T>()
            })
        } else {
            Some(unsafe {
                self.ptr.add(self.len as usize).read()
            })
        }
    }

    pub fn get(&self, idx: u64) -> Option<&T> {
        self.as_slice().get(idx as usize)
    }

    pub fn get_mut(&mut self, idx: u64) -> Option<&mut T> {
        self.as_slice_mut().get_mut(idx as usize)
    }

    pub fn swap_remove(&mut self, idx: u64) -> Option<T> {
        if idx >= self.len {
            return None;
        }

        unsafe {
            let item = self.ptr.add(idx as usize).read();
            
            self.len -= 1;

            std::ptr::copy(self.ptr.add(self.len as usize), self.ptr.add(idx as usize), 1);

            Some(item)
        }
    }

    pub fn remove(&mut self, idx: u64) -> Option<T> {
        if idx >= self.len {
            return None;
        }

        unsafe {
            let item = self.ptr.add(idx as usize).read();

            let moved_count = self.len - 1 - idx;

            std::ptr::copy(self.ptr.add(idx as usize + 1), self.ptr.add(idx as usize), moved_count as usize);

            self.len -= 1;

            Some(item)
        }
    }

    pub fn clear(&mut self) {
        if self.len == 0 {
            return;
        }

        let ptr = if std::mem::size_of::<T>() == 0 {
            std::ptr::NonNull::dangling().as_ptr()
        } else {
            self.ptr
        };

        unsafe {
            std::ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(ptr, self.len as usize));
        }

        self.len = 0;
    }
}

impl<T: Clone> FfiVec<T> {
    pub fn resize(&mut self, new_len: u64, value: T) {
        // todo: optimize
        while self.len > new_len {
            self.pop();
        }
        
        while self.len < new_len {
            self.push(value.clone());
        }
    }
}

impl<T> Default for FfiVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> FfiVec<T> {
    pub fn clone_to_vec(&self) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.len as usize);

        for item in self {
            vec.push(item.clone());
        }

        vec
    }
}

impl<T> Drop for FfiVec<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        let ptr = if std::mem::size_of::<T>() == 0 {
            std::ptr::NonNull::dangling().as_ptr()
        } else {
            self.ptr
        };
        
        // todo
        unsafe {
            std::ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(ptr, self.len as usize));
            
            // todo: exec on drop for guaranteed dealloc?
            if std::mem::size_of::<T>() != 0 {
                self.allocator.dealloc(self.ptr as *mut u8, std::mem::size_of::<T>() as u64 * self.cap, std::mem::align_of::<T>() as u64);
            }
        }
    }
}

impl<T> From<Vec<T>> for FfiVec<T> {
    fn from(value: Vec<T>) -> Self {
        FfiVec::from_vec(value)
    }
}

impl<'a, T> IntoIterator for &'a FfiVec<T> {
    type Item = <&'a [T] as IntoIterator>::Item;

    type IntoIter = <&'a [T] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut FfiVec<T> {
    type Item = <&'a mut [T] as IntoIterator>::Item;

    type IntoIter = <&'a mut [T] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().into_iter()
    }
}

impl<T: Debug> Debug for FfiVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_slice(), f)
    }
}

impl<T> Deref for FfiVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for FfiVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T: Clone> Clone for FfiVec<T> {
    fn clone(&self) -> Self {
        if std::mem::size_of::<T>() == 0 {
            unsafe {
                for _ in 0..self.len {
                    std::mem::forget((&*std::ptr::NonNull::<T>::dangling().as_ptr()).clone());
                }

                return Self {
                    ptr: self.ptr,
                    cap: self.len,
                    len: self.len,
                    allocator: self.allocator,
                };
            }
        }

        unsafe {
            let new_ptr = self.allocator.alloc(std::mem::size_of::<T>() as u64 * self.len, std::mem::align_of::<T>() as u64) as *mut T;

            for i in 0..self.len {
                let cloned = (&*self.ptr.add(i as usize)).clone();
                new_ptr.add(i as usize).write(cloned)
            }

            Self {
                ptr: new_ptr,
                cap: self.len,
                len: self.len,
                allocator: self.allocator,
            }
        }
    }
}

impl<T: PartialEq> PartialEq for FfiVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq> Eq for FfiVec<T> { }

impl<T> FromIterator<T> for FfiVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        // todo: optimize
        let mut vec = FfiVec::new();

        for item in iter {
            vec.push(item);    
        }

        vec
    }
}

impl<T> IntoIterator for FfiVec<T> {
    type Item = T;

    type IntoIter = FfiVecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        FfiVecIntoIter {
            vec: self,
            idx: 0,
        }
    }
}

pub struct FfiVecIntoIter<T> {
    vec: FfiVec<T>,
    idx: u64,
}

impl<T> Iterator for FfiVecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.vec.len {
            return None;
        }

        unsafe {
            let item = self.vec.ptr.add(self.idx as usize).read();

            self.idx += 1;

            Some(item)
        }
    }
}

impl<T> Drop for FfiVecIntoIter<T> {
    fn drop(&mut self) {
        let ptr = if std::mem::size_of::<T>() == 0 {
            std::ptr::NonNull::dangling().as_ptr()
        } else {
            self.vec.ptr
        };

        let count = self.vec.len - self.idx;

        // todo
        unsafe {
            let ptr = ptr.add(self.idx as usize);

            std::ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(ptr, count as usize));

            self.vec.len = 0;
        }
    }
}

const ZERO_SIZED_CAP: u64 = {
    let max_64 = u64::MAX as u64;
    let max_size = usize::MAX as u64;

    if max_64 < max_size {
        max_64
    } else {
        max_size
    }
};

// todo:
// - consuming IntoIterator
// - (reallocating) into_vec
// - docs
// - safe drops in case of exotic panics.
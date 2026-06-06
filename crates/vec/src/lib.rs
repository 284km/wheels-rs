//! Dynamic array (growable vector).
//!
//! Stored as a contiguous heap-allocated buffer, doubling in capacity on
//! demand. Zero-sized types are supported via a sentinel capacity of
//! `usize::MAX` and never trigger an allocation.

#![warn(unsafe_op_in_unsafe_fn)]

use std::alloc::{Layout, alloc, dealloc, handle_alloc_error, realloc};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Index, IndexMut};
use std::ptr::{self, NonNull};
use std::slice;

pub struct Vec<T> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
    _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Vec<T> {
    pub fn new() -> Self {
        let cap = if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            0
        };
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            cap,
            _marker: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if mem::size_of::<T>() == 0 || capacity == 0 {
            return Self::new();
        }
        let layout = Layout::array::<T>(capacity).expect("capacity overflow");
        // SAFETY: layout has non-zero size since T is non-ZST and capacity > 0.
        let raw = unsafe { alloc(layout) } as *mut T;
        let ptr = NonNull::new(raw).unwrap_or_else(|| handle_alloc_error(layout));
        Self {
            ptr,
            len: 0,
            cap: capacity,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        // SAFETY: len < cap after grow(); the slot at offset len is allocated and uninitialized.
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), value);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: the slot at offset len was initialized (len was len-1 < old len);
        // decrementing len first means the slot is now logically uninitialized.
        Some(unsafe { ptr::read(self.ptr.as_ptr().add(self.len)) })
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: index < len, so the slot at offset index is initialized.
        Some(unsafe { &*self.ptr.as_ptr().add(index) })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: same as get; &mut self prevents aliasing.
        Some(unsafe { &mut *self.ptr.as_ptr().add(index) })
    }

    pub fn set(&mut self, index: usize, value: T) -> Option<T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: index < len, so the slot is initialized; we replace its contents in place.
        unsafe {
            let slot = self.ptr.as_ptr().add(index);
            let old = ptr::read(slot);
            ptr::write(slot, value);
            Some(old)
        }
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: ptr is valid for `len` initialized elements of T.
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: same as as_slice; &mut self prevents aliasing.
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    fn grow(&mut self) {
        // ZSTs have cap == usize::MAX, so push() never reaches grow().
        assert!(mem::size_of::<T>() != 0, "capacity overflow");

        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).expect("capacity overflow"))
        } else {
            let new_cap = self.cap.checked_mul(2).expect("capacity overflow");
            let new_layout = Layout::array::<T>(new_cap).expect("capacity overflow");
            (new_cap, new_layout)
        };

        let new_raw = if self.cap == 0 {
            // SAFETY: new_layout has non-zero size.
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).expect("capacity overflow");
            let old_raw = self.ptr.as_ptr() as *mut u8;
            // SAFETY: old_raw was allocated by us with old_layout; new size is non-zero and fits in isize.
            unsafe { realloc(old_raw, old_layout, new_layout.size()) }
        };

        self.ptr = NonNull::new(new_raw as *mut T)
            .unwrap_or_else(|| handle_alloc_error(new_layout));
        self.cap = new_cap;
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<usize> for Vec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.get(index).expect("index out of bounds")
    }
}

impl<T> IndexMut<usize> for Vec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.get_mut(index).expect("index out of bounds")
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vec<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        // Drop all remaining elements. ptr::read for ZST is a no-op but still calls Drop.
        while self.pop().is_some() {}
        // Deallocate the raw buffer only when we actually allocated one.
        if self.cap != 0 && mem::size_of::<T>() != 0 {
            let layout = Layout::array::<T>(self.cap).expect("layout");
            // SAFETY: we allocated this buffer with this exact layout.
            unsafe {
                dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

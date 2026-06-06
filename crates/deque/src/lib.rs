//! Double-ended queue backed by a ring buffer.
//!
//! Elements live in a heap-allocated buffer of length `cap`. `head` is
//! the index of the front element; the back element sits at
//! `(head + len - 1) mod cap`. The data may wrap around the end of the
//! buffer, in which case iteration walks two halves.
//!
//! Capacity doubles on demand; growing reallocates and "unwraps" the
//! data into a contiguous run starting at index 0. Zero-sized types
//! are supported via a sentinel capacity of `usize::MAX` and never
//! allocate.

#![warn(unsafe_op_in_unsafe_fn)]

use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::iter::Chain;
use std::marker::PhantomData;
use std::mem;
use std::ptr::{self, NonNull};
use std::slice;

pub struct Deque<T> {
    ptr: NonNull<T>,
    head: usize,
    len: usize,
    cap: usize,
    _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for Deque<T> {}
unsafe impl<T: Sync> Sync for Deque<T> {}

impl<T> Deque<T> {
    pub fn new() -> Self {
        let cap = if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            0
        };
        Self {
            ptr: NonNull::dangling(),
            head: 0,
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
            head: 0,
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

    pub fn push_back(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        // wrapping_add handles ZST (cap == usize::MAX); for non-ZST no wrap happens.
        let index = self.head.wrapping_add(self.len) % self.cap;
        // SAFETY: index is within the buffer; the slot is uninitialized.
        unsafe {
            ptr::write(self.ptr.as_ptr().add(index), value);
        }
        self.len += 1;
    }

    pub fn push_front(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }
        // Decrement head with wrap-around. Written explicitly to avoid
        // overflow when cap == usize::MAX (ZST case).
        self.head = if self.head == 0 {
            self.cap - 1
        } else {
            self.head - 1
        };
        // SAFETY: head is within buffer; the new slot is uninitialized (it sat
        // outside the previous [old_head, old_head + len) range).
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.head), value);
        }
        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let index = self.head.wrapping_add(self.len) % self.cap;
        // SAFETY: the slot was initialized; decrementing len first makes it logically uninit.
        Some(unsafe { ptr::read(self.ptr.as_ptr().add(index)) })
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        // SAFETY: the slot at head was initialized.
        let value = unsafe { ptr::read(self.ptr.as_ptr().add(self.head)) };
        // head < cap, so head + 1 <= cap <= usize::MAX -> no overflow.
        self.head = (self.head + 1) % self.cap;
        self.len -= 1;
        Some(value)
    }

    pub fn front(&self) -> Option<&T> {
        self.get(0)
    }

    pub fn back(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            self.get(self.len - 1)
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        let pos = self.head.wrapping_add(index) % self.cap;
        // SAFETY: pos points at an initialized slot.
        Some(unsafe { &*self.ptr.as_ptr().add(pos) })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        let pos = self.head.wrapping_add(index) % self.cap;
        // SAFETY: pos points at an initialized slot; &mut self prevents aliasing.
        Some(unsafe { &mut *self.ptr.as_ptr().add(pos) })
    }

    /// Returns the two halves of the ring as slices: (front_half, back_half).
    /// When the data is contiguous, the second slice is empty.
    pub fn as_slices(&self) -> (&[T], &[T]) {
        if self.len == 0 {
            return (&[], &[]);
        }
        if mem::size_of::<T>() == 0 {
            // SAFETY: for ZST, a dangling pointer + any length is a valid slice
            // because no actual memory access occurs.
            let s = unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) };
            return (s, &[]);
        }
        if self.head + self.len <= self.cap {
            // SAFETY: contiguous initialized region [head, head + len).
            let s =
                unsafe { slice::from_raw_parts(self.ptr.as_ptr().add(self.head), self.len) };
            (s, &[])
        } else {
            let first = self.cap - self.head;
            let second = self.len - first;
            // SAFETY: two disjoint initialized regions (second <= head since len <= cap).
            unsafe {
                let a = slice::from_raw_parts(self.ptr.as_ptr().add(self.head), first);
                let b = slice::from_raw_parts(self.ptr.as_ptr(), second);
                (a, b)
            }
        }
    }

    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        if self.len == 0 {
            return (&mut [], &mut []);
        }
        if mem::size_of::<T>() == 0 {
            // SAFETY: same justification as as_slices for ZST.
            let s = unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) };
            return (s, &mut []);
        }
        if self.head + self.len <= self.cap {
            // SAFETY: contiguous initialized region; &mut self prevents aliasing.
            let s = unsafe {
                slice::from_raw_parts_mut(self.ptr.as_ptr().add(self.head), self.len)
            };
            (s, &mut [])
        } else {
            let first = self.cap - self.head;
            let second = self.len - first;
            // SAFETY: two disjoint initialized regions (second <= head since len <= cap).
            unsafe {
                let a = slice::from_raw_parts_mut(self.ptr.as_ptr().add(self.head), first);
                let b = slice::from_raw_parts_mut(self.ptr.as_ptr(), second);
                (a, b)
            }
        }
    }

    pub fn iter(&self) -> Chain<slice::Iter<'_, T>, slice::Iter<'_, T>> {
        let (a, b) = self.as_slices();
        a.iter().chain(b.iter())
    }

    pub fn iter_mut(&mut self) -> Chain<slice::IterMut<'_, T>, slice::IterMut<'_, T>> {
        let (a, b) = self.as_mut_slices();
        a.iter_mut().chain(b.iter_mut())
    }

    fn grow(&mut self) {
        // ZSTs have cap == usize::MAX so push() never reaches grow().
        assert!(mem::size_of::<T>() != 0, "capacity overflow");

        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).expect("capacity overflow"))
        } else {
            let new_cap = self.cap.checked_mul(2).expect("capacity overflow");
            let new_layout = Layout::array::<T>(new_cap).expect("capacity overflow");
            (new_cap, new_layout)
        };

        // We can't use realloc here because the data may wrap and must be
        // moved into a contiguous run. Allocate fresh, copy by halves, free old.
        // SAFETY: new_layout has non-zero size.
        let new_raw = unsafe { alloc(new_layout) } as *mut T;
        let new_ptr = NonNull::new(new_raw).unwrap_or_else(|| handle_alloc_error(new_layout));

        if self.len > 0 {
            if self.head + self.len <= self.cap {
                // SAFETY: copying len initialized values to a fresh non-overlapping buffer.
                unsafe {
                    ptr::copy_nonoverlapping(
                        self.ptr.as_ptr().add(self.head),
                        new_ptr.as_ptr(),
                        self.len,
                    );
                }
            } else {
                let first = self.cap - self.head;
                let second = self.len - first;
                // SAFETY: two contiguous initialized regions copied to disjoint destinations.
                unsafe {
                    ptr::copy_nonoverlapping(
                        self.ptr.as_ptr().add(self.head),
                        new_ptr.as_ptr(),
                        first,
                    );
                    ptr::copy_nonoverlapping(
                        self.ptr.as_ptr(),
                        new_ptr.as_ptr().add(first),
                        second,
                    );
                }
            }
        }

        if self.cap > 0 {
            let old_layout = Layout::array::<T>(self.cap).expect("layout");
            // SAFETY: we allocated this buffer with this exact layout; the elements
            // have been bitwise-moved out, so dealloc must not run their destructors.
            unsafe {
                dealloc(self.ptr.as_ptr() as *mut u8, old_layout);
            }
        }

        self.ptr = new_ptr;
        self.head = 0;
        self.cap = new_cap;
    }
}

impl<T> Default for Deque<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a Deque<T> {
    type Item = &'a T;
    type IntoIter = Chain<slice::Iter<'a, T>, slice::Iter<'a, T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Deque<T> {
    type Item = &'a mut T;
    type IntoIter = Chain<slice::IterMut<'a, T>, slice::IterMut<'a, T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Drop for Deque<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
        if self.cap != 0 && mem::size_of::<T>() != 0 {
            let layout = Layout::array::<T>(self.cap).expect("layout");
            // SAFETY: we allocated with this layout; all elements have been dropped.
            unsafe {
                dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

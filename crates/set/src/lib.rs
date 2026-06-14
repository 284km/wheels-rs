//! Hash set built on top of [`wheels_map::HashMap`].
//!
//! A set is a map whose values carry no information, so this crate is a
//! thin wrapper that renames the API around elements (`insert(x)` /
//! `contains(x)`) and adds the standard set-algebra operations. All the
//! probing, resize, and hashing logic lives in `wheels-map`.

#![forbid(unsafe_code)]

use std::hash::Hash;

use wheels_map::HashMap;

pub struct HashSet<T> {
    map: HashMap<T, ()>,
}

impl<T> HashSet<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.map.iter(),
        }
    }
}

impl<T: Hash + Eq> HashSet<T> {
    /// Adds `value` to the set. Returns `true` if it was newly inserted.
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    pub fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    /// Removes `value` from the set. Returns `true` if it was present.
    pub fn remove(&mut self, value: &T) -> bool {
        self.map.remove(value).is_some()
    }

    pub fn union<'a>(&'a self, other: &'a HashSet<T>) -> impl Iterator<Item = &'a T> + 'a {
        self.iter()
            .chain(other.iter().filter(move |&x| !self.contains(x)))
    }

    pub fn intersection<'a>(&'a self, other: &'a HashSet<T>) -> impl Iterator<Item = &'a T> + 'a {
        self.iter().filter(move |&x| other.contains(x))
    }

    pub fn difference<'a>(&'a self, other: &'a HashSet<T>) -> impl Iterator<Item = &'a T> + 'a {
        self.iter().filter(move |&x| !other.contains(x))
    }
}

impl<T> Default for HashSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Iter<'a, T> {
    inner: wheels_map::Iter<'a, T, ()>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, _)| k)
    }
}

impl<'a, T> IntoIterator for &'a HashSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

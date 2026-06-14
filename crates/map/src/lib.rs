//! Hash map with open addressing and linear probing.
//!
//! Slots are kept in a single `Vec<Slot<K, V>>` whose length is always a
//! power of two so that the probe index can be computed with a bitmask
//! instead of a modulo. Each occupied slot caches its hash to make
//! probing and resize cheap. Removed entries are marked as tombstones so
//! that probes for keys hashed past them keep walking the cluster.
//!
//! When the combined load (occupied + tombstones) reaches 3/4 of the
//! capacity, the table is rebuilt — doubling the backing storage when
//! live entries dominate, or rehashing in place when tombstones do.

#![forbid(unsafe_code)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;

const MIN_CAPACITY: usize = 8;

enum Slot<K, V> {
    Empty,
    Tombstone,
    Occupied { hash: u64, key: K, value: V },
}

pub struct HashMap<K, V> {
    slots: Vec<Slot<K, V>>,
    len: usize,
    tombstones: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            len: 0,
            tombstones: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }
        // Need cap * 4 / 3 slots so that `capacity` entries fit under
        // the 75% load threshold without immediately triggering a rebuild.
        let needed = capacity.saturating_mul(4) / 3 + 1;
        let cap = needed.next_power_of_two().max(MIN_CAPACITY);
        Self {
            slots: empty_slots(cap),
            len: 0,
            tombstones: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.slots.iter(),
        }
    }
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.needs_resize() {
            self.resize();
        }
        let hash = make_hash(&key);
        match self.probe(hash, &key) {
            ProbeResult::Found(idx) => match &mut self.slots[idx] {
                Slot::Occupied { value: v, .. } => Some(mem::replace(v, value)),
                _ => unreachable!("Found points to an Occupied slot"),
            },
            ProbeResult::Vacant(idx) => {
                let was_tombstone = matches!(self.slots[idx], Slot::Tombstone);
                self.slots[idx] = Slot::Occupied { hash, key, value };
                self.len += 1;
                if was_tombstone {
                    self.tombstones -= 1;
                }
                None
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if self.slots.is_empty() {
            return None;
        }
        let hash = make_hash(key);
        match self.probe(hash, key) {
            ProbeResult::Found(idx) => match &self.slots[idx] {
                Slot::Occupied { value, .. } => Some(value),
                _ => None,
            },
            ProbeResult::Vacant(_) => None,
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if self.slots.is_empty() {
            return None;
        }
        let hash = make_hash(key);
        match self.probe(hash, key) {
            ProbeResult::Found(idx) => match &mut self.slots[idx] {
                Slot::Occupied { value, .. } => Some(value),
                _ => None,
            },
            ProbeResult::Vacant(_) => None,
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if self.slots.is_empty() {
            return None;
        }
        let hash = make_hash(key);
        match self.probe(hash, key) {
            ProbeResult::Found(idx) => {
                let slot = mem::replace(&mut self.slots[idx], Slot::Tombstone);
                self.len -= 1;
                self.tombstones += 1;
                match slot {
                    Slot::Occupied { value, .. } => Some(value),
                    _ => None,
                }
            }
            ProbeResult::Vacant(_) => None,
        }
    }

    fn needs_resize(&self) -> bool {
        if self.slots.is_empty() {
            return true;
        }
        (self.len + self.tombstones) * 4 >= self.slots.len() * 3
    }

    fn resize(&mut self) {
        let new_cap = if self.slots.is_empty() {
            MIN_CAPACITY
        } else if self.len * 2 >= self.slots.len() {
            // Live entries dominate — double the table.
            self.slots.len() * 2
        } else {
            // Tombstones dominate — keep size, just rebuild to clear them.
            self.slots.len()
        };
        let old = mem::replace(&mut self.slots, empty_slots(new_cap));
        self.len = 0;
        self.tombstones = 0;
        for slot in old {
            if let Slot::Occupied { hash, key, value } = slot {
                let idx = self.probe_empty(hash);
                self.slots[idx] = Slot::Occupied { hash, key, value };
                self.len += 1;
            }
        }
    }

    fn probe(&self, hash: u64, key: &K) -> ProbeResult {
        let mask = self.slots.len() - 1;
        let mut idx = (hash as usize) & mask;
        let mut first_tombstone: Option<usize> = None;
        loop {
            match &self.slots[idx] {
                Slot::Empty => {
                    return ProbeResult::Vacant(first_tombstone.unwrap_or(idx));
                }
                Slot::Tombstone => {
                    if first_tombstone.is_none() {
                        first_tombstone = Some(idx);
                    }
                }
                Slot::Occupied { hash: h, key: k, .. } => {
                    if *h == hash && k == key {
                        return ProbeResult::Found(idx);
                    }
                }
            }
            idx = (idx + 1) & mask;
        }
    }

    // After `resize` allocates a fresh buffer, every destination slot is
    // Empty — there are no tombstones and no duplicate keys to consider —
    // so this fast path skips the bookkeeping that `probe` does.
    fn probe_empty(&self, hash: u64) -> usize {
        let mask = self.slots.len() - 1;
        let mut idx = (hash as usize) & mask;
        while !matches!(self.slots[idx], Slot::Empty) {
            idx = (idx + 1) & mask;
        }
        idx
    }
}

enum ProbeResult {
    Found(usize),
    Vacant(usize),
}

impl<K, V> Default for HashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Iter<'a, K, V> {
    inner: std::slice::Iter<'a, Slot<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        for slot in self.inner.by_ref() {
            if let Slot::Occupied { key, value, .. } = slot {
                return Some((key, value));
            }
        }
        None
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn empty_slots<K, V>(cap: usize) -> Vec<Slot<K, V>> {
    let mut v = Vec::with_capacity(cap);
    for _ in 0..cap {
        v.push(Slot::Empty);
    }
    v
}

fn make_hash<K: Hash + ?Sized>(key: &K) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

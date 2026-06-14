use wheels_map::HashMap;

#[test]
fn new_map_is_empty() {
    let map: HashMap<String, i32> = HashMap::new();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.get(&"missing".to_string()), None);
}

#[test]
fn insert_then_get() {
    let mut map = HashMap::new();
    map.insert("one".to_string(), 1);
    map.insert("two".to_string(), 2);
    map.insert("three".to_string(), 3);
    assert_eq!(map.len(), 3);
    assert_eq!(map.get(&"one".to_string()), Some(&1));
    assert_eq!(map.get(&"two".to_string()), Some(&2));
    assert_eq!(map.get(&"three".to_string()), Some(&3));
}

#[test]
fn insert_returns_previous_value_on_overwrite() {
    let mut map = HashMap::new();
    assert_eq!(map.insert("k".to_string(), 1), None);
    assert_eq!(map.insert("k".to_string(), 2), Some(1));
    assert_eq!(map.insert("k".to_string(), 3), Some(2));
    assert_eq!(map.len(), 1);
    assert_eq!(map.get(&"k".to_string()), Some(&3));
}

#[test]
fn get_returns_none_for_missing_key() {
    let mut map = HashMap::new();
    map.insert(1, "a");
    map.insert(2, "b");
    assert_eq!(map.get(&3), None);
}

#[test]
fn contains_key_reports_presence() {
    let mut map = HashMap::new();
    map.insert(1, "a");
    assert!(map.contains_key(&1));
    assert!(!map.contains_key(&2));
}

#[test]
fn remove_returns_value_and_decrements_len() {
    let mut map = HashMap::new();
    map.insert("a".to_string(), 1);
    map.insert("b".to_string(), 2);
    assert_eq!(map.remove(&"a".to_string()), Some(1));
    assert_eq!(map.len(), 1);
    assert_eq!(map.get(&"a".to_string()), None);
    assert_eq!(map.get(&"b".to_string()), Some(&2));
}

#[test]
fn remove_missing_returns_none() {
    let mut map: HashMap<i32, i32> = HashMap::new();
    assert_eq!(map.remove(&1), None);
    map.insert(1, 10);
    assert_eq!(map.remove(&2), None);
    assert_eq!(map.len(), 1);
}

#[test]
fn get_mut_allows_in_place_update() {
    let mut map = HashMap::new();
    map.insert("counter".to_string(), 0);
    if let Some(v) = map.get_mut(&"counter".to_string()) {
        *v += 5;
    }
    assert_eq!(map.get(&"counter".to_string()), Some(&5));
}

#[test]
fn with_capacity_preallocates() {
    let mut map = HashMap::with_capacity(100);
    let initial_cap = map.capacity();
    assert!(initial_cap >= 100);
    for i in 0..100 {
        map.insert(i, i * 2);
    }
    assert_eq!(map.len(), 100);
    assert_eq!(map.capacity(), initial_cap);
    for i in 0..100 {
        assert_eq!(map.get(&i), Some(&(i * 2)));
    }
}

#[test]
fn resize_preserves_all_entries() {
    let mut map = HashMap::new();
    for i in 0..1000 {
        map.insert(i, i * 3);
    }
    assert_eq!(map.len(), 1000);
    for i in 0..1000 {
        assert_eq!(map.get(&i), Some(&(i * 3)));
    }
}

#[test]
fn iter_visits_every_entry_exactly_once() {
    let mut map = HashMap::new();
    let entries: Vec<(i32, &str)> = vec![(1, "a"), (2, "b"), (3, "c"), (4, "d"), (5, "e")];
    for (k, v) in &entries {
        map.insert(*k, *v);
    }
    let mut seen: Vec<(i32, &str)> = map.iter().map(|(k, v)| (*k, *v)).collect();
    seen.sort_by_key(|(k, _)| *k);
    assert_eq!(seen, entries);
}

#[test]
fn iter_via_for_loop() {
    let mut map = HashMap::new();
    map.insert(1, 10);
    map.insert(2, 20);
    let mut sum = 0;
    for (k, v) in &map {
        sum += k + v;
    }
    assert_eq!(sum, 1 + 10 + 2 + 20);
}

#[test]
fn reinsert_after_remove_works() {
    let mut map = HashMap::new();
    map.insert("x".to_string(), 1);
    map.remove(&"x".to_string());
    assert_eq!(map.get(&"x".to_string()), None);
    map.insert("x".to_string(), 2);
    assert_eq!(map.get(&"x".to_string()), Some(&2));
    assert_eq!(map.len(), 1);
}

#[test]
fn churn_keeps_map_correct() {
    // Insert/remove the same keys many times. If tombstone bookkeeping
    // is wrong, probes eventually stop finding existing keys, or the
    // table fills up and rebuild never happens.
    let mut map = HashMap::new();
    for round in 0..200 {
        for i in 0..50 {
            map.insert(i, round * 1000 + i);
        }
        for i in 0..50 {
            assert_eq!(map.get(&i), Some(&(round * 1000 + i)));
        }
        for i in 0..50 {
            assert_eq!(map.remove(&i), Some(round * 1000 + i));
        }
        assert_eq!(map.len(), 0);
    }
}

#[test]
fn many_collisions_via_pseudo_random_inputs() {
    // Deterministic pseudo-random keys (no external deps), then verify
    // every key is retrievable. A broken probe would silently lose keys.
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut keys = Vec::with_capacity(2000);
    for _ in 0..2000 {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        keys.push(state);
    }
    let mut map = HashMap::new();
    for (i, k) in keys.iter().enumerate() {
        map.insert(*k, i);
    }
    assert_eq!(map.len(), keys.len());
    for (i, k) in keys.iter().enumerate() {
        assert_eq!(map.get(k), Some(&i));
    }
}

#[test]
fn works_with_unit_value_as_set() {
    let mut map: HashMap<i32, ()> = HashMap::new();
    map.insert(1, ());
    map.insert(2, ());
    map.insert(1, ());
    assert_eq!(map.len(), 2);
    assert!(map.contains_key(&1));
    assert!(map.contains_key(&2));
    assert!(!map.contains_key(&3));
}

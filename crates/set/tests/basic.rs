use wheels_set::HashSet;

fn sorted<T: Ord + Copy>(iter: impl Iterator<Item = T>) -> Vec<T> {
    let mut v: Vec<T> = iter.collect();
    v.sort();
    v
}

#[test]
fn new_set_is_empty() {
    let set: HashSet<i32> = HashSet::new();
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);
    assert!(!set.contains(&1));
}

#[test]
fn insert_returns_true_for_new_elements() {
    let mut set = HashSet::new();
    assert!(set.insert(1));
    assert!(set.insert(2));
    assert!(set.insert(3));
    assert_eq!(set.len(), 3);
}

#[test]
fn insert_returns_false_for_duplicate() {
    let mut set = HashSet::new();
    assert!(set.insert("a".to_string()));
    assert!(!set.insert("a".to_string()));
    assert_eq!(set.len(), 1);
}

#[test]
fn contains_reports_membership() {
    let mut set = HashSet::new();
    set.insert(10);
    set.insert(20);
    assert!(set.contains(&10));
    assert!(set.contains(&20));
    assert!(!set.contains(&30));
}

#[test]
fn remove_returns_true_when_present() {
    let mut set = HashSet::new();
    set.insert("x".to_string());
    assert!(set.remove(&"x".to_string()));
    assert_eq!(set.len(), 0);
    assert!(!set.contains(&"x".to_string()));
}

#[test]
fn remove_returns_false_when_missing() {
    let mut set: HashSet<i32> = HashSet::new();
    assert!(!set.remove(&1));
    set.insert(1);
    assert!(!set.remove(&2));
    assert_eq!(set.len(), 1);
}

#[test]
fn iter_yields_every_element_once() {
    let mut set = HashSet::new();
    for n in [1, 2, 3, 4, 5] {
        set.insert(n);
    }
    let collected = sorted(set.iter().copied());
    assert_eq!(collected, vec![1, 2, 3, 4, 5]);
}

#[test]
fn iter_via_for_loop() {
    let mut set = HashSet::new();
    set.insert(1);
    set.insert(2);
    set.insert(3);
    let mut sum = 0;
    for x in &set {
        sum += *x;
    }
    assert_eq!(sum, 6);
}

#[test]
fn union_yields_all_distinct_elements() {
    let mut a = HashSet::new();
    a.insert(1);
    a.insert(2);
    a.insert(3);
    let mut b = HashSet::new();
    b.insert(3);
    b.insert(4);
    b.insert(5);
    let collected = sorted(a.union(&b).copied());
    assert_eq!(collected, vec![1, 2, 3, 4, 5]);
}

#[test]
fn intersection_yields_common_elements() {
    let mut a = HashSet::new();
    for n in [1, 2, 3, 4] {
        a.insert(n);
    }
    let mut b = HashSet::new();
    for n in [3, 4, 5, 6] {
        b.insert(n);
    }
    let collected = sorted(a.intersection(&b).copied());
    assert_eq!(collected, vec![3, 4]);
}

#[test]
fn difference_yields_only_self_side() {
    let mut a = HashSet::new();
    for n in [1, 2, 3, 4] {
        a.insert(n);
    }
    let mut b = HashSet::new();
    for n in [3, 4, 5, 6] {
        b.insert(n);
    }
    let collected = sorted(a.difference(&b).copied());
    assert_eq!(collected, vec![1, 2]);
}

#[test]
fn set_operations_with_disjoint_inputs() {
    let mut a = HashSet::new();
    a.insert(1);
    a.insert(2);
    let mut b = HashSet::new();
    b.insert(3);
    b.insert(4);
    assert_eq!(sorted(a.union(&b).copied()), vec![1, 2, 3, 4]);
    assert!(a.intersection(&b).next().is_none());
    assert_eq!(sorted(a.difference(&b).copied()), vec![1, 2]);
}

#[test]
fn set_operations_with_empty_other() {
    let mut a = HashSet::new();
    a.insert(1);
    a.insert(2);
    let b: HashSet<i32> = HashSet::new();
    assert_eq!(sorted(a.union(&b).copied()), vec![1, 2]);
    assert!(a.intersection(&b).next().is_none());
    assert_eq!(sorted(a.difference(&b).copied()), vec![1, 2]);
}

#[test]
fn with_capacity_preallocates() {
    let mut set = HashSet::with_capacity(100);
    let initial_cap = set.capacity();
    assert!(initial_cap >= 100);
    for i in 0..100 {
        set.insert(i);
    }
    assert_eq!(set.len(), 100);
    assert_eq!(set.capacity(), initial_cap);
}

#[test]
fn many_elements_resize_correctly() {
    let mut set = HashSet::new();
    for i in 0..1000 {
        assert!(set.insert(i));
    }
    assert_eq!(set.len(), 1000);
    for i in 0..1000 {
        assert!(set.contains(&i));
    }
    assert!(!set.contains(&-1));
    assert!(!set.contains(&1000));
}

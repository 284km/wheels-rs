use std::cell::Cell;
use std::rc::Rc;
use wheels_vec::Vec;

#[test]
fn new_is_empty() {
    let v: Vec<i32> = Vec::new();
    assert!(v.is_empty());
    assert_eq!(v.len(), 0);
    assert_eq!(v.capacity(), 0);
    assert_eq!(v.get(0), None);
}

#[test]
fn push_and_len() {
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    assert_eq!(v.len(), 3);
    assert!(v.capacity() >= 3);
}

#[test]
fn pop_returns_lifo() {
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    assert_eq!(v.pop(), Some(3));
    assert_eq!(v.pop(), Some(2));
    assert_eq!(v.pop(), Some(1));
    assert_eq!(v.pop(), None);
}

#[test]
fn get_and_get_mut() {
    let mut v = Vec::new();
    v.push(10);
    v.push(20);
    v.push(30);
    assert_eq!(v.get(0), Some(&10));
    assert_eq!(v.get(2), Some(&30));
    assert_eq!(v.get(3), None);

    *v.get_mut(1).unwrap() = 200;
    assert_eq!(v.get(1), Some(&200));
}

#[test]
fn set_returns_old_value() {
    let mut v = Vec::new();
    v.push(String::from("a"));
    v.push(String::from("b"));
    let old = v.set(0, String::from("A"));
    assert_eq!(old.as_deref(), Some("a"));
    assert_eq!(v.get(0).map(|s| s.as_str()), Some("A"));
    assert_eq!(v.set(99, String::from("X")), None);
}

#[test]
fn indexing_works() {
    let mut v = Vec::new();
    v.push(7);
    v.push(8);
    assert_eq!(v[0], 7);
    v[1] = 80;
    assert_eq!(v[1], 80);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn indexing_out_of_bounds_panics() {
    let v: Vec<i32> = Vec::new();
    let _ = v[0];
}

#[test]
fn iteration_immutable() {
    let mut v = Vec::new();
    for n in 1..=5 {
        v.push(n);
    }
    let sum: i32 = v.iter().sum();
    assert_eq!(sum, 15);

    let collected: std::vec::Vec<i32> = (&v).into_iter().copied().collect();
    assert_eq!(collected, vec![1, 2, 3, 4, 5]);
}

#[test]
fn iteration_mutable() {
    let mut v = Vec::new();
    for n in 1..=4 {
        v.push(n);
    }
    for x in &mut v {
        *x *= 10;
    }
    let collected: std::vec::Vec<i32> = v.iter().copied().collect();
    assert_eq!(collected, vec![10, 20, 30, 40]);
}

#[test]
fn capacity_grows_on_push() {
    let mut v: Vec<i32> = Vec::new();
    let mut last_cap = v.capacity();
    let mut growth_count = 0usize;
    for n in 0..100 {
        v.push(n);
        if v.capacity() > last_cap {
            last_cap = v.capacity();
            growth_count += 1;
        }
    }
    assert!(growth_count > 0);
    assert!(v.capacity() >= 100);
}

#[test]
fn with_capacity_avoids_reallocation() {
    let mut v: Vec<i32> = Vec::with_capacity(100);
    assert_eq!(v.capacity(), 100);
    for n in 0..100 {
        v.push(n);
    }
    assert_eq!(v.capacity(), 100);
}

#[test]
fn drops_remaining_elements_when_vec_is_dropped() {
    struct Counter(Rc<Cell<usize>>);
    impl Drop for Counter {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let counter = Rc::new(Cell::new(0));
    {
        let mut v = Vec::new();
        for _ in 0..5 {
            v.push(Counter(counter.clone()));
        }
    }
    assert_eq!(counter.get(), 5);
}

#[test]
fn pop_runs_destructor_on_drop_of_returned_value() {
    struct Counter(Rc<Cell<usize>>);
    impl Drop for Counter {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let counter = Rc::new(Cell::new(0));
    let mut v = Vec::new();
    v.push(Counter(counter.clone()));
    drop(v.pop().unwrap());
    assert_eq!(counter.get(), 1);
}

#[test]
fn zst_supported() {
    let mut v: Vec<()> = Vec::new();
    assert_eq!(v.capacity(), usize::MAX);
    for _ in 0..1000 {
        v.push(());
    }
    assert_eq!(v.len(), 1000);
    assert_eq!(v.pop(), Some(()));
    assert_eq!(v.len(), 999);
}

#[test]
fn works_with_string_drop_type() {
    let mut v = Vec::new();
    v.push(String::from("hello"));
    v.push(String::from("world"));
    assert_eq!(v[0].as_str(), "hello");
    assert_eq!(v.pop().as_deref(), Some("world"));
    assert_eq!(v.len(), 1);
}

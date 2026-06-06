use std::cell::Cell;
use std::rc::Rc;
use wheels_deque::Deque;

#[test]
fn new_is_empty() {
    let d: Deque<i32> = Deque::new();
    assert!(d.is_empty());
    assert_eq!(d.len(), 0);
    assert_eq!(d.capacity(), 0);
    assert_eq!(d.front(), None);
    assert_eq!(d.back(), None);
}

#[test]
fn push_back_then_pop_front_is_fifo() {
    let mut d = Deque::new();
    d.push_back(1);
    d.push_back(2);
    d.push_back(3);
    assert_eq!(d.pop_front(), Some(1));
    assert_eq!(d.pop_front(), Some(2));
    assert_eq!(d.pop_front(), Some(3));
    assert_eq!(d.pop_front(), None);
}

#[test]
fn push_front_then_pop_back_is_fifo() {
    let mut d = Deque::new();
    d.push_front(1);
    d.push_front(2);
    d.push_front(3);
    assert_eq!(d.pop_back(), Some(1));
    assert_eq!(d.pop_back(), Some(2));
    assert_eq!(d.pop_back(), Some(3));
    assert_eq!(d.pop_back(), None);
}

#[test]
fn push_back_then_pop_back_is_lifo() {
    let mut d = Deque::new();
    d.push_back(1);
    d.push_back(2);
    d.push_back(3);
    assert_eq!(d.pop_back(), Some(3));
    assert_eq!(d.pop_back(), Some(2));
    assert_eq!(d.pop_back(), Some(1));
}

#[test]
fn push_front_then_pop_front_is_lifo() {
    let mut d = Deque::new();
    d.push_front(1);
    d.push_front(2);
    d.push_front(3);
    assert_eq!(d.pop_front(), Some(3));
    assert_eq!(d.pop_front(), Some(2));
    assert_eq!(d.pop_front(), Some(1));
}

#[test]
fn mixed_pushes_from_both_ends() {
    let mut d = Deque::new();
    d.push_back(2);
    d.push_front(1);
    d.push_back(3);
    d.push_front(0);
    // Logical order: [0, 1, 2, 3]
    assert_eq!(d.len(), 4);
    let collected: std::vec::Vec<i32> = d.iter().copied().collect();
    assert_eq!(collected, vec![0, 1, 2, 3]);
}

#[test]
fn front_and_back_peek() {
    let mut d = Deque::new();
    d.push_back(10);
    d.push_back(20);
    d.push_back(30);
    assert_eq!(d.front(), Some(&10));
    assert_eq!(d.back(), Some(&30));
}

#[test]
fn get_by_logical_index() {
    let mut d = Deque::new();
    d.push_back(10);
    d.push_back(20);
    d.push_back(30);
    assert_eq!(d.get(0), Some(&10));
    assert_eq!(d.get(1), Some(&20));
    assert_eq!(d.get(2), Some(&30));
    assert_eq!(d.get(3), None);
}

#[test]
fn iteration_when_data_wraps_around() {
    // Force the data to occupy a wrapped slice in the buffer.
    let mut d: Deque<i32> = Deque::with_capacity(4);
    d.push_back(1);
    d.push_back(2);
    d.push_back(3);
    d.push_back(4); // head=0, len=4
    d.pop_front(); // head=1, len=3
    d.pop_front(); // head=2, len=2
    d.push_back(5); // head=2, len=3, position 0 used
    d.push_back(6); // head=2, len=4, positions 0, 1 used. Data wraps.

    assert_eq!(d.capacity(), 4); // still cap=4, no grow yet
    let collected: std::vec::Vec<i32> = d.iter().copied().collect();
    assert_eq!(collected, vec![3, 4, 5, 6]);

    // as_slices should report two halves
    let (a, b) = d.as_slices();
    assert!(!a.is_empty() && !b.is_empty(), "data should be wrapped");
    assert_eq!(a, &[3, 4][..]);
    assert_eq!(b, &[5, 6][..]);
}

#[test]
fn iter_mut_when_data_wraps_around() {
    let mut d: Deque<i32> = Deque::with_capacity(4);
    d.push_back(1);
    d.push_back(2);
    d.push_back(3);
    d.push_back(4);
    d.pop_front();
    d.pop_front();
    d.push_back(5);
    d.push_back(6);
    // logical: [3, 4, 5, 6], physical wrap.

    for x in &mut d {
        *x *= 10;
    }
    let collected: std::vec::Vec<i32> = d.iter().copied().collect();
    assert_eq!(collected, vec![30, 40, 50, 60]);
}

#[test]
fn grow_correctly_unwraps_wrapped_data() {
    let mut d: Deque<i32> = Deque::with_capacity(4);
    d.push_back(1);
    d.push_back(2);
    d.push_back(3);
    d.push_back(4);
    d.pop_front(); // logical: [2, 3, 4]
    d.pop_front(); // logical: [3, 4]
    d.push_back(5);
    d.push_back(6); // logical: [3, 4, 5, 6], physical wrapped

    // Trigger growth.
    d.push_back(7);
    assert!(d.capacity() >= 5);

    let collected: std::vec::Vec<i32> = d.iter().copied().collect();
    assert_eq!(collected, vec![3, 4, 5, 6, 7]);

    // After grow, data must be contiguous from index 0.
    let (a, b) = d.as_slices();
    assert!(b.is_empty(), "data should be contiguous after grow");
    assert_eq!(a, &[3, 4, 5, 6, 7][..]);
}

#[test]
fn capacity_grows_from_zero() {
    let mut d: Deque<i32> = Deque::new();
    assert_eq!(d.capacity(), 0);
    for n in 0..10 {
        d.push_back(n);
    }
    assert_eq!(d.len(), 10);
    assert!(d.capacity() >= 10);
    let collected: std::vec::Vec<i32> = d.iter().copied().collect();
    assert_eq!(collected, (0..10).collect::<std::vec::Vec<_>>());
}

#[test]
fn zst_supported() {
    let mut d: Deque<()> = Deque::new();
    assert_eq!(d.capacity(), usize::MAX);
    for _ in 0..1000 {
        d.push_back(());
    }
    for _ in 0..500 {
        d.push_front(());
    }
    assert_eq!(d.len(), 1500);
    for _ in 0..1500 {
        assert_eq!(d.pop_front(), Some(()));
    }
    assert_eq!(d.pop_front(), None);
}

#[test]
fn drops_remaining_elements() {
    struct Counter(Rc<Cell<usize>>);
    impl Drop for Counter {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let counter = Rc::new(Cell::new(0));
    {
        let mut d = Deque::new();
        for _ in 0..5 {
            d.push_back(Counter(counter.clone()));
        }
        for _ in 0..3 {
            d.push_front(Counter(counter.clone()));
        }
        assert_eq!(d.len(), 8);
    }
    assert_eq!(counter.get(), 8);
}

#[test]
fn drop_after_pop_does_not_double_drop() {
    struct Counter(Rc<Cell<usize>>);
    impl Drop for Counter {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let counter = Rc::new(Cell::new(0));
    {
        let mut d = Deque::new();
        d.push_back(Counter(counter.clone()));
        d.push_back(Counter(counter.clone()));
        drop(d.pop_front()); // counter -> 1
        // remaining 1 element should drop when d goes out of scope
    }
    assert_eq!(counter.get(), 2);
}

#[test]
fn works_with_string_drop_type() {
    let mut d = Deque::new();
    d.push_back(String::from("b"));
    d.push_front(String::from("a"));
    d.push_back(String::from("c"));
    assert_eq!(d.front().map(|s| s.as_str()), Some("a"));
    assert_eq!(d.back().map(|s| s.as_str()), Some("c"));
    assert_eq!(d.pop_front().as_deref(), Some("a"));
    assert_eq!(d.pop_back().as_deref(), Some("c"));
    assert_eq!(d.pop_front().as_deref(), Some("b"));
}

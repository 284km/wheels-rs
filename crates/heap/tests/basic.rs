use wheels_heap::BinaryHeap;

#[test]
fn new_heap_is_empty() {
    let heap: BinaryHeap<i32> = BinaryHeap::new();
    assert!(heap.is_empty());
    assert_eq!(heap.len(), 0);
    assert_eq!(heap.peek(), None);
}

#[test]
fn pop_empty_returns_none() {
    let mut heap: BinaryHeap<i32> = BinaryHeap::new();
    assert_eq!(heap.pop(), None);
}

#[test]
fn push_then_peek_returns_max() {
    let mut heap = BinaryHeap::new();
    heap.push(3);
    heap.push(1);
    heap.push(4);
    heap.push(1);
    heap.push(5);
    assert_eq!(heap.peek(), Some(&5));
    assert_eq!(heap.len(), 5);
}

#[test]
fn pop_returns_descending_order() {
    let mut heap = BinaryHeap::new();
    for n in [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5] {
        heap.push(n);
    }
    let mut popped = Vec::new();
    while let Some(n) = heap.pop() {
        popped.push(n);
    }
    assert_eq!(popped, vec![9, 6, 5, 5, 5, 4, 3, 3, 2, 1, 1]);
}

#[test]
fn matches_sorted_descending_for_pseudo_random_inputs() {
    // Linear congruential generator (Numerical Recipes constants) for
    // deterministic pseudo-random values without external dependencies.
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut input = Vec::with_capacity(1000);
    for _ in 0..1000 {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        input.push((state >> 32) as i64);
    }

    let mut heap = BinaryHeap::new();
    for n in &input {
        heap.push(*n);
    }

    let mut from_heap = Vec::with_capacity(input.len());
    while let Some(n) = heap.pop() {
        from_heap.push(n);
    }

    let mut expected = input.clone();
    expected.sort_by(|a, b| b.cmp(a));
    assert_eq!(from_heap, expected);
}

#[test]
fn with_capacity_preserves_behavior() {
    let mut heap = BinaryHeap::with_capacity(100);
    for n in 0..100 {
        heap.push(n);
    }
    assert_eq!(heap.pop(), Some(99));
    assert_eq!(heap.len(), 99);
}

#[test]
fn works_with_string_type() {
    let mut heap = BinaryHeap::new();
    heap.push(String::from("alpha"));
    heap.push(String::from("gamma"));
    heap.push(String::from("beta"));
    assert_eq!(heap.pop().as_deref(), Some("gamma"));
    assert_eq!(heap.pop().as_deref(), Some("beta"));
    assert_eq!(heap.pop().as_deref(), Some("alpha"));
}

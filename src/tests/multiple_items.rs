use crate::queue::Queue;
use orx_concurrent_bag::*;
use std::fmt::Debug;
use test_case::test_matrix;

#[test_matrix([|x| x, |x| x.to_string()])]
fn con_just_extend<T: Send + Clone + Ord + Debug>(f: impl Fn(usize) -> T + Sync) {
    let f = &f;
    let num_extenders = 4;
    let num_ticks = 1000;

    let capacity = num_extenders * num_ticks * 3;

    let mut queue = Queue::new(capacity);
    let q = &queue;

    let mut expected = vec![];
    for t in 0..num_extenders {
        for i in 0..num_ticks {
            let value = t * num_ticks + i;
            let items = [f(value), f(1_000_000 + value), f(10_000_000 + value)]
                .into_iter()
                .map(|x| x.clone());
            expected.extend(items);
        }
    }
    expected.sort();

    std::thread::scope(|s| {
        for t in 0..num_extenders {
            s.spawn(move || {
                for i in 0..num_ticks {
                    let value = t * num_ticks + i;
                    let items = [f(value), f(1_000_000 + value), f(10_000_000 + value)]
                        .into_iter()
                        .map(|x| x.clone());
                    match t.is_multiple_of(2) {
                        true => q.extend(items),
                        false => unsafe { q.extend_n_items(items, 3) },
                    }
                }
            });
        }
    });

    let mut pushed: Vec<_> = queue.as_slice().iter().cloned().collect();
    pushed.sort();
    assert_eq!(pushed, expected);
}

#[test_matrix([|x| x, |x| x.to_string()])]
fn con_just_pull<T: Send + Clone + Ord + Debug>(f: impl Fn(usize) -> T + Sync) {
    let num_pullers = 4;
    let num_ticks = 1000;
    let chunk_size = 17;

    let capacity = num_pullers * num_ticks;

    let mut queue = Queue::new(capacity);
    let collected = ConcurrentBag::new();

    for i in 0..capacity {
        queue.push(f(i));
    }
    let mut expected: Vec<_> = queue.as_slice().iter().cloned().collect();
    expected.sort();

    let q = &queue;
    std::thread::scope(|s| {
        for _ in 0..num_pullers {
            s.spawn(|| {
                for _ in 0..(num_ticks + 20) {
                    if let Some(values) = q.pull(chunk_size) {
                        collected.extend(values);
                    }
                }
            });
        }
    });

    let mut collected = collected.into_inner().to_vec();
    collected.sort();

    assert_eq!(collected, expected);
}

#[test_matrix([|x| x, |x| x.to_string()])]
fn con_extend_and_pull<T: Send + Clone + Ord + Debug>(f: impl Fn(usize) -> T + Sync) {
    let f = &f;
    let num_extenders = 4;
    let num_pullers = 4;
    let num_ticks = 1000;
    let chunk_size = 17;

    let capacity = num_extenders * num_ticks * 3;

    let queue = Queue::new(capacity);
    let collected = ConcurrentBag::new();

    let q = &queue;
    std::thread::scope(|s| {
        for t in 0..num_extenders {
            s.spawn(move || {
                for i in 0..num_ticks {
                    let value = t * num_ticks + i;
                    let items = [f(value), f(1_000_000 + value), f(10_000_000 + value)]
                        .into_iter()
                        .map(|x| x.clone());
                    match t.is_multiple_of(2) {
                        true => q.extend(items),
                        false => unsafe { q.extend_n_items(items, 3) },
                    }
                }
            });
        }
    });

    let q = &queue;
    std::thread::scope(|s| {
        for _ in 0..num_pullers {
            s.spawn(|| {
                for _ in 0..(num_ticks + 20) {
                    if let Some(values) = q.pull(chunk_size) {
                        collected.extend(values);
                    }
                }
            });
        }
    });
}

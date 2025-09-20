use crate::queue::Queue;
use orx_concurrent_bag::*;
use std::fmt::Debug;
use test_case::test_matrix;

#[test_matrix([|x| x, |x| x.to_string()])]
fn con_just_push<T: Send + Clone + Ord + Debug>(f: impl Fn(usize) -> T + Sync) {
    let f = &f;
    let num_pushers = 4;
    let num_ticks = 20;

    let capacity = num_pushers * num_ticks;

    let mut queue = Queue::new(capacity);
    let q = &queue;

    std::thread::scope(|s| {
        for t in 0..num_pushers {
            s.spawn(move || {
                for i in 0..num_ticks {
                    q.push(f(t * num_ticks + i));
                }
            });
        }
    });

    let mut pushed: Vec<_> = queue.as_slice().iter().cloned().collect();
    pushed.sort();

    let mut expected: Vec<_> = (0..pushed.len()).map(f).collect();
    expected.sort();

    assert_eq!(pushed, expected);
}

#[test_matrix([|x| x, |x| x.to_string()])]
fn con_just_pop<T: Send + Clone + Ord + Debug>(f: impl Fn(usize) -> T + Sync) {
    let num_poppers = 4;
    let num_ticks = 1000;

    let capacity = num_poppers * num_ticks;

    let queue = Queue::new(capacity);
    let q = &queue;
    let collected = ConcurrentBag::new();

    for i in 0..capacity {
        queue.push(f(i));
    }

    std::thread::scope(|s| {
        for _ in 0..num_poppers {
            s.spawn(|| {
                for _ in 0..(num_ticks + 20) {
                    if let Some(value) = q.pop() {
                        collected.push(value);
                    }
                }
            });
        }
    });

    let mut collected = collected.into_inner();
    collected.sort();

    let mut expected: Vec<_> = (0..collected.len()).map(f).collect();
    expected.sort();

    assert_eq!(collected, expected);
}

#[test_matrix([|x| x, |x| x.to_string()])]
fn con_push_pop<T: Send + Clone + Ord + Debug>(f: impl Fn(usize) -> T + Sync) {
    let f = &f;
    let num_pushers = 4;
    let num_poppers = 4;
    let num_ticks = 1000;

    let capacity = num_pushers * num_ticks;
    let queue = Queue::new(capacity);
    let q = &queue;
    let collected = ConcurrentBag::new();

    std::thread::scope(|s| {
        for t in 0..num_pushers {
            s.spawn(move || {
                for i in 0..num_ticks {
                    q.push(f(t * num_ticks + i));
                }
            });
        }
    });

    std::thread::scope(|s| {
        for _ in 0..num_poppers {
            s.spawn(|| {
                for _ in 0..(num_ticks + 20) {
                    if let Some(value) = q.pop() {
                        collected.push(value);
                    }
                }
            });
        }
    });

    let mut collected = collected.into_inner();
    collected.sort();

    let mut expected: Vec<_> = (0..collected.len()).map(f).collect();
    expected.sort();

    assert_eq!(collected, expected);
}

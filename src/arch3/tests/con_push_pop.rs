use crate::Queue;
use orx_concurrent_bag::{ConcurrentBag, PinnedVec};
use orx_split_vec::Collection;
use std::{time::Duration, usize};

#[test]
fn con_push_pop() {
    let num_pushers = 3;
    let num_poppers = 2;
    let num_ticks = 100;

    let queue = Queue::new();
    let q = &queue;

    std::thread::scope(|s| {
        for t in 0..num_pushers {
            std::thread::sleep(Duration::from_millis(100));
            s.spawn(move || {
                for i in 0..num_ticks {
                    std::thread::sleep(Duration::from_millis(i % 5));
                    q.push(t * num_ticks + i);
                }
            });
        }

        for _ in 0..num_poppers {
            s.spawn(move || {
                std::thread::sleep(Duration::from_millis(10));
                for i in 0..num_ticks {
                    std::thread::sleep(Duration::from_millis(i % 5));
                    if let Some(value) = q.pop() {
                        dbg!(value);
                    }
                }
            });
        }
    });

    // assert_eq!(q.len(), 33);
}

#[test]
fn con_push_pop_collect() {
    let collected = ConcurrentBag::<usize>::new();

    let num_pushers = 1;
    let num_poppers = 8;
    let num_ticks = 10000;

    let queue = Queue::new();
    let q = &queue;
    let bag = &collected;

    std::thread::scope(|s| {
        for _ in 0..num_poppers {
            s.spawn(move || {
                for i in 0..num_ticks {
                    // std::thread::sleep(Duration::from_millis(i as u64 % 5));
                    if let Some(value) = q.pop() {
                        bag.push(value);
                    }
                }
            });
        }

        for t in 0..num_pushers {
            std::thread::sleep(Duration::from_millis(100));
            s.spawn(move || {
                for i in 0..num_ticks {
                    // std::thread::sleep(Duration::from_millis(i as u64 % 5));
                    q.push(t * num_ticks + i);
                }
            });
        }
    });

    let mut collected = collected.into_inner();
    collected.sort();

    let mut prev = usize::MAX;
    for x in collected.iter().copied() {
        assert_ne!(prev, x);
        prev = x;
    }

    dbg!(collected.len(), queue.len());

    // assert!(collected.len() == 33);
}

use crate::queue::Queue;
use orx_concurrent_bag::*;

#[test]
fn con_just_push() {
    let num_pushers = 4;
    let num_ticks = 1000;

    let capacity = num_pushers * num_ticks;

    let mut queue = Queue::new(capacity);
    let q = &queue;

    std::thread::scope(|s| {
        for t in 0..num_pushers {
            s.spawn(move || {
                for i in 0..num_ticks {
                    q.push(t * num_ticks + i);
                }
            });
        }
    });

    let mut pushed: Vec<_> = queue.as_slice().iter().copied().collect();
    pushed.sort();
    assert_eq!(pushed, (0..pushed.len()).collect::<Vec<_>>());
}

#[test]
fn con_just_pop() {
    let num_poppers = 4;
    let num_ticks = 1000;

    let capacity = num_poppers * num_ticks;

    let queue = Queue::new(capacity);
    let q = &queue;
    let collected = ConcurrentBag::new();

    for i in 0..capacity {
        queue.push(i);
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
    assert_eq!(collected, (0..collected.len()).collect::<Vec<_>>());
}

#[test]
fn con_push_pop() {
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
                    q.push(t * num_ticks + i);
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
    assert_eq!(collected, (0..collected.len()).collect::<Vec<_>>());
}

use crate::queue::Queue;
use orx_concurrent_bag::*;

#[test]
fn con_just_extend() {
    let num_extenders = 4;
    let num_ticks = 1000;

    let capacity = num_extenders * num_ticks * 3;

    let mut queue = Queue::new(capacity);
    let q = &queue;

    let mut expected = vec![];
    for t in 0..num_extenders {
        for i in 0..num_ticks {
            let value = t * num_ticks + i;
            let items = [value, 1_000_000 + value, 10_000_000 + value]
                .into_iter()
                .map(|x| x + 1);
            expected.extend(items);
        }
    }
    expected.sort();

    std::thread::scope(|s| {
        for t in 0..num_extenders {
            s.spawn(move || {
                for i in 0..num_ticks {
                    let value = t * num_ticks + i;
                    let items = [value, 1_000_000 + value, 10_000_000 + value]
                        .into_iter()
                        .map(|x| x + 1);
                    match t.is_multiple_of(2) {
                        true => q.extend(items),
                        false => unsafe { q.extend_n_items(items, 3) },
                    }
                }
            });
        }
    });

    let mut pushed: Vec<_> = queue.as_slice().iter().copied().collect();
    pushed.sort();
    assert_eq!(pushed, expected);
}

#[test]
fn con_just_pull() {
    let num_pullers = 4;
    let num_ticks = 1000;
    let chunk_size = 17;

    let capacity = num_pullers * num_ticks;

    let mut queue = Queue::new(capacity);
    let collected = ConcurrentBag::new();

    for i in 0..capacity {
        queue.push(i);
    }
    let expected: Vec<_> = queue.as_slice().iter().copied().collect();

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

#[test]
fn con_extend_and_pull() {
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
                    let items = [value, 1_000_000 + value, 10_000_000 + value]
                        .into_iter()
                        .map(|x| x + 1);
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

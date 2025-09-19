use crate::queue::Queue;

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
fn con_push_pop() {
    let queue = Queue::<usize>::new(100);
}

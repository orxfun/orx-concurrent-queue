use crate::queue::Queue;
use orx_concurrent_bag::*;

#[test]
fn con_just_extend() {
    let num_pushers = 4;
    let num_ticks = 1000;

    let capacity = num_pushers * num_ticks * 3;

    let mut queue = Queue::new(capacity);
    let q = &queue;

    let mut expected = vec![];
    for t in 0..num_pushers {
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
        for t in 0..num_pushers {
            s.spawn(move || {
                for i in 0..num_ticks {
                    let value = t * num_ticks + i;
                    let items = [value, 1_000_000 + value, 10_000_000 + value]
                        .into_iter()
                        .map(|x| x + 1);
                    q.extend(items);
                }
            });
        }
    });

    let mut pushed: Vec<_> = queue.as_slice().iter().copied().collect();
    pushed.sort();
    assert_eq!(pushed, expected);
}

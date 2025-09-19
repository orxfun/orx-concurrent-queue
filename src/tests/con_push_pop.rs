use crate::Queue;
use std::time::Duration;

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

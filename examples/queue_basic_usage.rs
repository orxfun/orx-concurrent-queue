use orx_concurrent_bag::*;
use orx_concurrent_queue::*;
use orx_split_vec::Collection;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    // queue to represents tasks or data shared
    // among senders (pushers/extenders) and receivers (poppers/pullers)
    let queue = ConcurrentQueue::<usize>::new();

    // a flag to signal to inform threads completion of the process
    let completed = AtomicBool::new(false);

    let num_senders = 4;
    let num_receivers = 4;

    let collected = ConcurrentBag::new();
    std::thread::scope(|s| {
        // receivers
        for _ in 0..num_receivers {
            s.spawn(|| {
                while !completed.load(Ordering::Relaxed) {
                    if let Some(value) = queue.pop() {
                        collected.push(value.to_string());
                    }
                }
            });
        }

        // senders
        for _ in 0..num_senders {
            s.spawn(|| {
                let mut rng = rand::rng();
                while !completed.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    let value = rng.random_range(0..100);
                    match value == 42 {
                        true => completed.store(true, Ordering::Relaxed),
                        false => queue.push(value),
                    }
                }
            });
        }
    });

    let collected_vec = collected.into_inner();
    assert!(collected_vec.iter().all(|x| x != "42"));
    println!(
        "Collected {} items before hitting 42:)",
        collected_vec.len()
    );
}

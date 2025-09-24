use orx_concurrent_bag::*;
use orx_concurrent_queue::*;

fn main() {
    let num_poppers = 4;
    let num_ticks = 1000;

    let capacity = num_poppers * num_ticks;

    // let queue = ConcurrentQueue::new();
    // let collected = ConcurrentBag::new();

    // for i in 0..capacity {
    //     queue.push(i);
    // }

    // std::thread::scope(|s| {
    //     for _ in 0..num_poppers {
    //         s.spawn(|| {
    //             for _ in 0..(num_ticks + 10) {
    //                 let popped = queue.pop();
    //                 // println!("{popped:?}");
    //                 if let Some(value) = popped {
    //                     collected.push(value);
    //                 }
    //             }
    //         });
    //     }
    // });

    // let mut collected = collected.into_inner();
    // collected.sort();
    // assert_eq!(collected, (0..collected.len()).collect::<Vec<_>>());
}

# orx-concurrent-queue

[![orx-concurrent-queue crate](https://img.shields.io/crates/v/orx-concurrent-queue.svg)](https://crates.io/crates/orx-concurrent-queue)
[![orx-concurrent-queue crate](https://img.shields.io/crates/d/orx-concurrent-queue.svg)](https://crates.io/crates/orx-concurrent-queue)
[![orx-concurrent-queue documentation](https://docs.rs/orx-concurrent-queue/badge.svg)](https://docs.rs/orx-concurrent-queue)

A high performance and convenient thread safe queue with push, extend, pop and pull capabilities.

## ConcurrentQueue Examples

The following example demonstrates a basic usage of [`ConcurrentQueue`](https://docs.rs/orx-concurrent-queue/latest/orx_concurrent_queue/struct.ConcurrentQueue.html) within a synchronous program. Note that `push`, `extend`, `pop` and `pull` methods can be called with a shared reference `&self`. This allows to use the queue conveniently in a concurrent program.

```rust
use orx_concurrent_queue::ConcurrentQueue;

let queue = ConcurrentQueue::new();

queue.push(0); // [0]
queue.push(1); // [0, 1]

let x = queue.pop(); // [1]
assert_eq!(x, Some(0));

queue.extend(2..7); // [1, 2, 3, 4, 5, 6]

let x: Vec<_> = queue.pull(4).unwrap().collect(); // [5, 6]
assert_eq!(x, vec![1, 2, 3, 4]);

assert_eq!(queue.len(), 2);

let vec = queue.into_inner();
assert_eq!(vec, vec![5, 6]);
```

The following example demonstrates the main purpose of the concurrent queue, which is to simultaneously push to and pop from the queue. This enables a dynamic iterator that can be traversed by multiple threads, which can also dynamically grow during the iteration.

In the following example, the queue is created with three pre-populated tasks. Every task might potentially lead to new tasks. These new tasks are also added to the back of the queue, to be popped later and potentially add new tasks to the queue.

```rust
use orx_concurrent_queue::ConcurrentQueue;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Task {
    micros: usize,
}

impl Task {
    fn perform(&self) {
        use std::{thread::sleep, time::Duration};
        sleep(Duration::from_micros(self.micros as u64));
    }
    fn child_tasks(&self) -> impl ExactSizeIterator<Item = Task> {
        let range = match self.micros < 5 {
            true => 0..0,
            false => 0..self.micros,
        };
        range.rev().take(5).map(|micros| Self { micros })
    }
}

let queue = ConcurrentQueue::new();

// pre-populate with 3 tasks
for micros in [10, 15, 10] {
    queue.push(Task { micros });
}

// count total number of performed tasks
let num_performed_tasks = AtomicUsize::new(queue.len());

let num_threads = 8;
std::thread::scope(|s| {
    for _ in 0..num_threads {
        s.spawn(|| {
            // keep popping a task from front of the queue
            // as long as the queue is not empty
            while let Some(task) = queue.pop() {
                // create children tasks, add to back
                queue.extend(task.child_tasks());

                // perform the popped task
                task.perform();

                _ = num_performed_tasks.fetch_add(1, Ordering::Relaxed);
            }
        });
    }
});

assert_eq!(num_performed_tasks.load(Ordering::Relaxed), 5046);
```
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_queue::{ConcurrentQueue, DynamicConcurrentIter};
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Node {
    value: u64,
    children: Vec<Node>,
}

fn fibonacci(n: u64) -> u64 {
    let n = n % 42; // let's not overflow
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

impl Node {
    fn new(rng: &mut impl Rng, value: u64) -> Self {
        let num_children = match value {
            0 => 0,
            n => rng.random_range(0..(n as usize)),
        };
        let children = (0..num_children)
            .map(|i| Self::new(rng, i as u64))
            .collect();
        Self { value, children }
    }

    fn num_nodes(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|node| node.num_nodes())
            .sum::<usize>()
    }
}

fn compute_with_rec_iter(root: &Node, num_threads: usize) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }
    let queue = ConcurrentQueue::new();
    queue.push(root);
    let iter = DynamicConcurrentIter::new(queue, extend);

    let num_spawned = AtomicUsize::new(0);
    std::thread::scope(|s| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(|| {
                // allow all threads to be spawned
                _ = num_spawned.fetch_add(1, Ordering::Relaxed);
                while num_spawned.load(Ordering::Relaxed) < num_threads {}

                // computation: parallel reduction
                let mut thread_sum = 0;
                while let Some(node) = iter.next() {
                    thread_sum += fibonacci(node.value % 42);
                }
                thread_sum
            }));
        }

        handles.into_iter().map(|x| x.join().unwrap()).sum()
    })
}

fn main() {
    let num_threads = 16;

    let mut rng = rand::rng();
    let root = Node::new(&mut rng, 100);

    println!("Tree has {} nodes", root.num_nodes());

    let total_fibonacci = compute_with_rec_iter(&root, num_threads);
    println!("Sum of Fibonacci numbers of all node values = {total_fibonacci}");
}

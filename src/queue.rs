use core::sync::atomic::{AtomicUsize, Ordering};
use orx_concurrent_bag::{ConcurrentBag, Doubling, SplitVec};

pub struct Queue<T> {
    bag: ConcurrentBag<T, SplitVec<T, Doubling>>,
    pushed: AtomicUsize,
    popped: AtomicUsize,
    len: AtomicUsize,
}

impl<T> Queue<T> {
    pub fn pop(&self) -> Option<T> {
        let prior = self.len.fetch_sub(1, Ordering::SeqCst);
        match prior {
            0 => {
                self.len.store(0, Ordering::SeqCst);
                None
            }
            _ => {
                let idx = self.popped.fetch_add(1, Ordering::SeqCst);
                Some(unsafe { self.bag.take(idx) })
            }
        }
    }
}

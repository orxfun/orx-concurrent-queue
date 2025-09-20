use core::sync::atomic::{AtomicIsize, AtomicUsize};
use std::sync::atomic::Ordering;

pub struct ConcurrentQueueState {
    pub(super) len: AtomicIsize,    // written_len
    pub(super) pushed: AtomicUsize, // len
    pub(super) popped: AtomicUsize,
}

impl ConcurrentQueueState {
    pub fn new_for_vec(len: usize) -> Self {
        Self {
            len: (len as isize).into(),
            pushed: len.into(),
            popped: 0.into(),
        }
    }

    pub fn pop_idx(&self) -> Option<usize> {
        let previous = self.len.fetch_sub(1, Ordering::Acquire);
        match previous {
            p if p <= 0 => {
                // no item to pop
                let current = p - 1;
                while self
                    .len
                    .compare_exchange_weak(current, p, Ordering::Acquire, Ordering::Relaxed)
                    .is_err()
                {}
                None
            }
            _ => {
                let idx = self.popped.fetch_add(1, Ordering::Acquire);
                Some(idx)
            }
        }
    }
}

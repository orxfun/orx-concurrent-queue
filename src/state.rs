use core::sync::atomic::{AtomicIsize, AtomicUsize};

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
}

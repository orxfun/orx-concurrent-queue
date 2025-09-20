use core::{
    cmp::Ordering,
    sync::atomic,
    sync::atomic::{AtomicIsize, AtomicUsize},
};
use orx_pinned_concurrent_col::{
    ConcurrentState, PinnedConcurrentCol, WritePermit, prelude::ConcurrentPinnedVec,
    prelude::PinnedVec,
};

pub struct ConcurrentQueueState {
    pub(super) len: AtomicIsize,    // written_len
    pub(super) pushed: AtomicUsize, // len
    pub(super) popped: AtomicUsize,
}

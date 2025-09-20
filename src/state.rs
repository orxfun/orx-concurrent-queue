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
    len: AtomicIsize,    // written_len
    pushed: AtomicUsize, // len
    popped: AtomicUsize,
}

impl<T> ConcurrentState<T> for ConcurrentQueueState {
    fn fill_memory_with(&self) -> Option<fn() -> T> {
        None
    }

    fn new_for_pinned_vec<P: PinnedVec<T>>(pinned_vec: &P) -> Self {
        Self {
            len: (pinned_vec.len() as isize).into(),
            pushed: pinned_vec.len().into(),
            popped: 0.into(),
        }
    }

    fn new_for_con_pinned_vec<P: ConcurrentPinnedVec<T>>(_: &P, len: usize) -> Self {
        Self {
            len: (len as isize).into(),
            pushed: len.into(),
            popped: 0.into(),
        }
    }

    fn write_permit<P>(&self, col: &PinnedConcurrentCol<T, P, Self>, idx: usize) -> WritePermit
    where
        P: ConcurrentPinnedVec<T>,
    {
        let capacity = col.capacity();

        match idx.cmp(&capacity) {
            Ordering::Less => WritePermit::JustWrite,
            Ordering::Equal => WritePermit::GrowThenWrite,
            Ordering::Greater => WritePermit::Spin,
        }
    }

    fn write_permit_n_items<P>(
        &self,
        col: &PinnedConcurrentCol<T, P, Self>,
        begin_idx: usize,
        num_items: usize,
    ) -> WritePermit
    where
        P: ConcurrentPinnedVec<T>,
    {
        let capacity = col.capacity();
        let last_idx = begin_idx + num_items - 1;

        match (begin_idx.cmp(&capacity), last_idx.cmp(&capacity)) {
            (_, core::cmp::Ordering::Less) => WritePermit::JustWrite,
            (core::cmp::Ordering::Greater, _) => WritePermit::Spin,
            _ => WritePermit::GrowThenWrite,
        }
    }

    fn release_growth_handle(&self) {}

    fn update_after_write(&self, _: usize, end_idx: usize) {
        self.len.store(end_idx as isize, atomic::Ordering::Release);
    }

    fn try_get_no_gap_len(&self) -> Option<usize> {
        Some(self.pushed.load(atomic::Ordering::Acquire))
    }
}

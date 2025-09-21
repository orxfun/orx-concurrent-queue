use core::sync::atomic::{AtomicIsize, AtomicUsize};
use std::sync::atomic::Ordering;

pub struct State {
    pub(super) len: AtomicIsize,    // written_len
    pub(super) pushed: AtomicUsize, // len
    pub(super) popped: AtomicUsize,
}

impl State {
    pub fn new_for_vec(len: usize) -> Self {
        Self {
            len: (len as isize).into(),
            pushed: len.into(),
            popped: 0.into(),
        }
    }

    // shrink

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

    pub fn pull_idx_and_len(&self, chunk_size: usize) -> Option<(usize, usize)> {
        let chunk_size_i = chunk_size as isize;

        let previous = self.len.fetch_sub(chunk_size_i, Ordering::Acquire);
        match previous {
            p if p <= 0 => {
                // there are no items
                let current = p - chunk_size_i;
                while self
                    .len
                    .compare_exchange_weak(current, p, Ordering::Acquire, Ordering::Relaxed)
                    .is_err()
                {}
                None
            }
            p if p < chunk_size_i => {
                // there are items, but fewer than chunk_size
                let current = p - chunk_size_i;
                while self
                    .len
                    .compare_exchange_weak(current, 0, Ordering::Acquire, Ordering::Relaxed)
                    .is_err()
                {}

                let chunk_size = p as usize;
                let idx = self.popped.fetch_add(chunk_size, Ordering::Acquire);
                Some((idx, chunk_size))
            }
            _ => {
                // there are at least chunk_size items
                let idx = self.popped.fetch_add(chunk_size, Ordering::Acquire);
                Some((idx, chunk_size))
            }
        }
    }

    // grow

    pub fn grow_handle(&self, num_items: usize) -> (GrowHandle<'_>, usize) {
        GrowHandle::create(self, num_items)
    }
}

// grow handle

pub struct GrowHandle<'a> {
    state: &'a State,
    num_items: usize,
}

impl<'a> GrowHandle<'a> {
    fn create(state: &'a State, num_items: usize) -> (Self, usize) {
        let idx = state.pushed.fetch_add(num_items, Ordering::Acquire);
        let handle = Self { state, num_items };
        (handle, idx)
    }

    pub fn release(self) {
        self.state
            .len
            .fetch_add(self.num_items as isize, Ordering::Release);
    }
}

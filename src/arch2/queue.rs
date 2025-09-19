use crate::con_split_vec::ConSplitVec;
use core::sync::atomic::{AtomicUsize, Ordering};
use orx_split_vec::GrowthWithConstantTimeAccess;

pub struct Queue<T, G>
where
    G: GrowthWithConstantTimeAccess,
{
    vec: ConSplitVec<T, G>,
    pushed: AtomicUsize,
    popped: AtomicUsize,
    len: AtomicUsize,
}

impl<T, G> Queue<T, G>
where
    G: GrowthWithConstantTimeAccess,
{
    pub fn push(&self, value: T) {
        let idx = self.pushed.fetch_add(1, Ordering::SeqCst);
    }

    pub fn pop(&self) -> Option<T> {
        let prior = self.len.fetch_sub(1, Ordering::SeqCst);
        match prior {
            0 => {
                self.len.store(0, Ordering::SeqCst);
                None
            }
            _ => {
                let idx = self.popped.fetch_add(1, Ordering::SeqCst);
                Some(unsafe { self.vec.take(idx) })
            }
        }
    }
}

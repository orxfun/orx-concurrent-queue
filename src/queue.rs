use crate::state::ConcurrentQueueState;
use orx_pinned_vec::{ConcurrentPinnedVec, IntoConcurrentPinnedVec};
use std::{marker::PhantomData, sync::atomic::Ordering};

pub struct ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    vec: P,
    state: ConcurrentQueueState,
    phantom: PhantomData<T>,
}

unsafe impl<T, P> Sync for ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
}

impl<T, P> Drop for ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    fn drop(&mut self) {
        if core::mem::needs_drop::<T>() {
            let s = &self.state;
            let popped = s.popped.load(Ordering::Relaxed);
            let pushed = s.pushed.load(Ordering::Relaxed);
            for i in popped..pushed {
                let ptr = unsafe { self.ptr(i) };
                unsafe { ptr.drop_in_place() };
            }
        }
        unsafe { self.vec.set_pinned_vec_len(0) };
    }
}

impl<T, P> From<P> for ConcurrentQueue<T, P::ConPinnedVec>
where
    T: Send,
    P: IntoConcurrentPinnedVec<T>,
{
    fn from(vec: P) -> Self {
        let state = ConcurrentQueueState::new_for_vec(vec.len());
        let vec = vec.into_concurrent();
        Self {
            vec,
            state,
            phantom: PhantomData,
        }
    }
}

impl<T, P> ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    #[inline(always)]
    unsafe fn ptr(&self, idx: usize) -> *mut T {
        unsafe { self.vec.get_ptr_mut(idx) }
    }

    // shrink

    pub fn pop(&self) -> Option<T> {
        self.state
            .pop_idx()
            .map(|idx| unsafe { self.ptr(idx).read() })
    }
}

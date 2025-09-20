use crate::state::ConcurrentQueueState;
use orx_pinned_vec::ConcurrentPinnedVec;
use std::{marker::PhantomData, mem::ManuallyDrop, sync::atomic::Ordering};

pub struct ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    phantom: PhantomData<T>,
    vec: P,
    state: ConcurrentQueueState,
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
                let ptr = unsafe { self.vec.get_ptr_mut(i) };
                unsafe { ptr.drop_in_place() };
            }
        }
        // let _vec = unsafe { Vec::from_raw_parts(self.data, 0, self.capacity) };
    }
}

impl<T, P> ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
}

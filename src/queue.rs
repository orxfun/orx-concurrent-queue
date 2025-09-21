use crate::{convec_state::WritePermit, queue_state::ConcurrentQueueState};
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
    pub fn len(&self, order: Ordering) -> usize {
        self.state.len.load(order).max(0) as usize
    }

    // shrink

    pub fn pop(&self) -> Option<T> {
        self.state
            .pop_idx()
            .map(|idx| unsafe { self.ptr(idx).read() })
    }

    pub fn pull(&self, chunk_size: usize) -> Option<impl ExactSizeIterator<Item = T>> {
        self.state
            .pull_idx_and_len(chunk_size)
            .map(|(idx, chunk_size)| {
                let range = idx..(idx + chunk_size);
                unsafe { self.vec.ptr_iter_unchecked(range) }.map(|ptr| unsafe { ptr.read() })
            })
    }

    // grow

    pub fn push(&self, value: T) {
        let (h, idx) = self.state.grow_handle(1);
        self.assert_has_capacity_for(idx);

        // TODO: this loop is not required for FixedVec. It can be avoided by abstracting it into PinnedConVec.
        loop {
            match WritePermit::new(self.vec.capacity(), idx) {
                WritePermit::JustWrite => {
                    unsafe { self.ptr(idx).write(value) };
                    break;
                }
                WritePermit::GrowThenWrite => {
                    self.grow_to(idx + 1);
                    unsafe { self.ptr(idx).write(value) };
                    break;
                }
                WritePermit::Spin => {}
            }
        }

        h.release();
    }

    pub fn extend<I, Iter>(&self, values: I)
    where
        I: IntoIterator<Item = T, IntoIter = Iter>,
        Iter: ExactSizeIterator<Item = T>,
    {
        let values = values.into_iter();
        let num_items = values.len();

        if num_items > 0 {
            let (h, idx) = self.state.grow_handle(num_items);
            let last_idx = idx + num_items - 1;
            self.assert_has_capacity_for(last_idx);

            loop {
                match WritePermit::new(self.vec.capacity(), last_idx) {
                    WritePermit::JustWrite => {
                        // unsafe { self.ptr(idx).write(value) };
                        break;
                    }
                    WritePermit::GrowThenWrite => {
                        self.grow_to(last_idx + 1);
                        // unsafe { self.ptr(idx).write(value) };
                        break;
                    }
                    WritePermit::Spin => {}
                }
            }

            h.release();
        }
    }

    // helpers

    #[inline(always)]
    unsafe fn ptr(&self, idx: usize) -> *mut T {
        unsafe { self.vec.get_ptr_mut(idx) }
    }

    fn assert_has_capacity_for(&self, idx: usize) {
        assert!(
            idx < self.vec.max_capacity(),
            "Out of capacity. Underlying pinned vector cannot grow any further while being concurrently safe."
        );
    }

    fn grow_to(&self, new_capacity: usize) {
        _ = self
            .vec
            .grow_to(new_capacity)
            .expect("The underlying pinned vector reached its capacity and failed to grow");
    }
}

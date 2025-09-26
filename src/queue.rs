use crate::{
    atomic_utils::{comp_exch, comp_exch_weak},
    common_traits::iter::{QueueIterOfMut, QueueIterOfRef},
    write_permit::WritePermit,
};
use core::{
    marker::PhantomData,
    ops::Range,
    sync::atomic::{AtomicUsize, Ordering},
};
use orx_pinned_vec::{ConcurrentPinnedVec, IntoConcurrentPinnedVec};
use orx_split_vec::{Doubling, SplitVec, prelude::PseudoDefault};

type DefaultPinnedVec<T> = SplitVec<T, Doubling>;
pub type DefaultConVec<T> = <DefaultPinnedVec<T> as IntoConcurrentPinnedVec<T>>::ConPinnedVec;

impl<T> Default for ConcurrentQueue<T, DefaultConVec<T>>
where
    T: Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ConcurrentQueue<T, DefaultConVec<T>>
where
    T: Send,
{
    pub fn new() -> Self {
        SplitVec::with_doubling_growth_and_max_concurrent_capacity().into()
    }
}

/// A thread safe queue that allows [`push`] and [`extend`] to concurrently add items to the back of the queue
/// while at the same time allowing [`pop`] and [`pull`] to remove items from the front.
///
/// [`push`]: crate::ConcurrentQueue::push
/// [`extend`]: crate::ConcurrentQueue::extend
/// [`pop`]: crate::ConcurrentQueue::pop
/// [`pull`]: crate::ConcurrentQueue::pull
///
/// # Examples
pub struct ConcurrentQueue<T, P = DefaultConVec<T>>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    vec: P,
    phantom: PhantomData<T>,
    written: AtomicUsize,
    write_reserved: AtomicUsize,
    popped: AtomicUsize,
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
            let popped = self.popped.load(Ordering::Relaxed);
            let reserved = self.write_reserved.load(Ordering::Relaxed);
            let written = self.written.load(Ordering::Relaxed);
            assert_eq!(reserved, written);
            for i in popped..written {
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
        Self {
            phantom: PhantomData,
            written: vec.len().into(),
            write_reserved: vec.len().into(),
            popped: 0.into(),
            vec: vec.into_concurrent(),
        }
    }
}

impl<T, P> ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    pub fn into_inner(mut self) -> <P as ConcurrentPinnedVec<T>>::P
    where
        <P as ConcurrentPinnedVec<T>>::P:
            PseudoDefault + IntoConcurrentPinnedVec<T, ConPinnedVec = P>,
    {
        let vec: <P as ConcurrentPinnedVec<T>>::P = PseudoDefault::pseudo_default();
        let mut vec = vec.into_concurrent();
        core::mem::swap(&mut self.vec, &mut vec);

        let a = self.popped.load(Ordering::Relaxed);
        let b = self.written.load(Ordering::Relaxed);
        let len = b - a;
        if a > 0 {
            let src = unsafe { vec.ptr_iter_unchecked(a..b) };
            let dst = unsafe { vec.ptr_iter_unchecked(0..len) };
            for (s, d) in src.zip(dst) {
                unsafe { d.write(s.read()) };
            }
        }

        for x in [&self.written, &self.write_reserved, &self.popped] {
            x.store(0, Ordering::Relaxed);
        }

        unsafe { vec.into_inner(len) }
    }

    // shrink

    pub fn pop(&self) -> Option<T> {
        let idx = self.popped.fetch_add(1, Ordering::Relaxed);

        loop {
            let written = self.written.load(Ordering::Acquire);
            match idx < written {
                true => return Some(unsafe { self.ptr(idx).read() }),
                false => {
                    if comp_exch(&self.popped, idx + 1, idx).is_ok() {
                        return None;
                    }
                }
            }
        }
    }

    pub fn pull(&self, chunk_size: usize) -> Option<impl ExactSizeIterator<Item = T>> {
        let begin_idx = self.popped.fetch_add(chunk_size, Ordering::Relaxed);
        let end_idx = begin_idx + chunk_size;

        loop {
            let written = self.written.load(Ordering::Acquire);

            let has_none = begin_idx >= written;
            let has_some = !has_none;
            let has_all = end_idx <= written;

            let range = match (has_some, has_all) {
                (false, _) => match comp_exch(&self.popped, end_idx, begin_idx).is_ok() {
                    true => return None,
                    false => None,
                },
                (true, true) => Some(begin_idx..end_idx),
                (true, false) => Some(begin_idx..written),
            };

            if let Some(range) = range {
                let ok = match has_all {
                    true => true,
                    false => comp_exch(&self.popped, end_idx, range.end).is_ok(),
                };

                if ok {
                    let iter = unsafe { self.vec.ptr_iter_unchecked(range) };
                    return Some(iter.map(|ptr| unsafe { ptr.read() }));
                }
            }
        }
    }

    // grow

    pub fn push(&self, value: T) {
        let idx = self.write_reserved.fetch_add(1, Ordering::Relaxed);
        self.assert_has_capacity_for(idx);

        loop {
            match WritePermit::for_one(self.vec.capacity(), idx) {
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

        let num_written = idx + 1;
        while comp_exch_weak(&self.written, idx, num_written).is_err() {}
    }

    pub fn extend<I, Iter>(&self, values: I)
    where
        I: IntoIterator<Item = T, IntoIter = Iter>,
        Iter: ExactSizeIterator<Item = T>,
    {
        let values = values.into_iter();
        let num_items = values.len();

        if num_items > 0 {
            let begin_idx = self.write_reserved.fetch_add(num_items, Ordering::Relaxed);
            let end_idx = begin_idx + num_items;
            let last_idx = begin_idx + num_items - 1;
            self.assert_has_capacity_for(last_idx);

            loop {
                match WritePermit::for_many(self.vec.capacity(), begin_idx, last_idx) {
                    WritePermit::JustWrite => {
                        let iter = unsafe { self.vec.ptr_iter_unchecked(begin_idx..end_idx) };
                        for (p, value) in iter.zip(values) {
                            unsafe { p.write(value) };
                        }
                        break;
                    }
                    WritePermit::GrowThenWrite => {
                        self.grow_to(end_idx);
                        let iter = unsafe { self.vec.ptr_iter_unchecked(begin_idx..end_idx) };
                        for (p, value) in iter.zip(values) {
                            unsafe { p.write(value) };
                        }
                        break;
                    }
                    WritePermit::Spin => {}
                }
            }

            while comp_exch_weak(&self.written, begin_idx, end_idx).is_err() {}
        }
    }

    // get

    pub fn len(&self) -> usize {
        self.written.load(Ordering::Relaxed) - self.popped.load(Ordering::Relaxed)
    }

    pub fn iter(&mut self) -> impl ExactSizeIterator<Item = &T> {
        QueueIterOfRef::<T, P>::new(self.ptr_iter())
    }

    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut T> {
        QueueIterOfMut::<T, P>::new(self.ptr_iter())
    }

    // helpers

    #[inline(always)]
    unsafe fn ptr(&self, idx: usize) -> *mut T {
        unsafe { self.vec.get_ptr_mut(idx) }
    }

    #[inline(always)]
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

    fn valid_range(&mut self) -> Range<usize> {
        self.popped.load(Ordering::Relaxed)..self.written.load(Ordering::Relaxed)
    }

    pub(crate) fn ptr_iter(&mut self) -> P::PtrIter<'_> {
        let range = self.valid_range();
        // SAFETY: with a mut ref, we ensure that the range contains all and only valid values
        unsafe { self.vec.ptr_iter_unchecked(range) }
    }
}

#[cfg(test)]
mod tsts {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;
    // use crate::*;

    #[test]
    fn abc() {
        let queue = ConcurrentQueue::new();

        queue.push(0); // [0]
        queue.push(1); // [0, 1]

        let x = queue.pop(); // [1]
        assert_eq!(x, Some(0));

        queue.extend(2..5); // [1, 2, 3, 4]

        let x: Vec<_> = queue.pull(3).unwrap().collect(); // [4]
        assert_eq!(x, vec![1, 2, 3]);

        assert_eq!(queue.len(), 1);
    }
}

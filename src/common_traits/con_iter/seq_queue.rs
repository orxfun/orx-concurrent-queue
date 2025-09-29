use core::{iter::FusedIterator, marker::PhantomData};
use orx_pinned_vec::ConcurrentPinnedVec;

pub struct DynSeqQueue<T, P, E, I>
where
    P: ConcurrentPinnedVec<T>,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    vec: P,
    written: usize,
    popped: usize,
    extend: E,
    phantom: PhantomData<T>,
}

impl<T, P, E, I> DynSeqQueue<T, P, E, I>
where
    P: ConcurrentPinnedVec<T>,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    pub(super) fn new(vec: P, written: usize, popped: usize, extend: E) -> Self {
        Self {
            vec,
            written,
            popped,
            extend,
            phantom: PhantomData,
        }
    }

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

    fn extend(&mut self, values: I) {
        let values = values.into_iter();
        let num_items = values.len();

        if num_items > 0 {
            let begin_idx = self.written;
            self.written += num_items;
            let end_idx = begin_idx + num_items;
            let last_idx = begin_idx + num_items - 1;
            self.assert_has_capacity_for(last_idx);

            let capacity = self.vec.capacity();

            if last_idx >= capacity {
                self.grow_to(end_idx);
            }

            let iter = unsafe { self.vec.ptr_iter_unchecked(begin_idx..end_idx) };
            for (p, value) in iter.zip(values) {
                unsafe { p.write(value) };
            }
        }
    }
}

impl<T, P, E, I> Iterator for DynSeqQueue<T, P, E, I>
where
    P: ConcurrentPinnedVec<T>,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.popped;
        self.popped += 1;

        match idx < self.written {
            true => {
                let n = unsafe { self.ptr(idx).read() };
                let children = (self.extend)(&n);
                self.extend(children);
                Some(n)
            }
            false => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let min = self.written - self.popped;
        (min, None)
    }
}

impl<T, P, E, I> FusedIterator for DynSeqQueue<T, P, E, I>
where
    P: ConcurrentPinnedVec<T>,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
}

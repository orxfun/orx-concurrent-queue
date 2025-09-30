use orx_pinned_vec::ConcurrentPinnedVec;

/// An iterator over owned elements of the concurrent queue created over a given range.
pub struct QueueIterOwned<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    iter: P::PtrIter<'a>,
}

impl<'a, T, P> Default for QueueIterOwned<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    fn default() -> Self {
        Self {
            iter: Default::default(),
        }
    }
}

impl<'a, T, P> QueueIterOwned<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    pub(crate) fn new(iter: P::PtrIter<'a>) -> Self {
        Self { iter }
    }
}

impl<'a, T, P> Iterator for QueueIterOwned<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ptr| unsafe { ptr.read() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, P> ExactSizeIterator for QueueIterOwned<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T, P> Drop for QueueIterOwned<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    fn drop(&mut self) {
        for ptr in self.iter.by_ref() {
            unsafe { ptr.drop_in_place() };
        }
    }
}

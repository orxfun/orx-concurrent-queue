use orx_pinned_vec::ConcurrentPinnedVec;

pub struct QueueIntoIter<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    iter: P::IntoIter,
}

impl<T, P> QueueIntoIter<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    pub(crate) fn new(iter: P::IntoIter) -> Self {
        Self { iter }
    }
}

impl<T, P> Iterator for QueueIntoIter<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, P> ExactSizeIterator for QueueIntoIter<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

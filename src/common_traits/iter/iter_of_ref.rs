use orx_pinned_vec::ConcurrentPinnedVec;

pub struct QueueIterOfRef<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    iter: P::PtrIter<'a>,
}

impl<'a, T, P> QueueIterOfRef<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    pub(crate) fn new(iter: P::PtrIter<'a>) -> Self {
        Self { iter }
    }
}

impl<'a, T, P> Iterator for QueueIterOfRef<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ptr| unsafe { &*ptr })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, P> ExactSizeIterator for QueueIterOfRef<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

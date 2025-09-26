use orx_pinned_vec::ConcurrentPinnedVec;

pub struct QueueIterOfMut<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    iter: P::PtrIter<'a>,
}

impl<'a, T, P> QueueIterOfMut<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    pub(crate) fn new(iter: P::PtrIter<'a>) -> Self {
        Self { iter }
    }
}

impl<'a, T, P> Iterator for QueueIterOfMut<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        #[allow(unused_mut)]
        self.iter.next().map(|mut ptr| unsafe { &mut *ptr })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, P> ExactSizeIterator for QueueIterOfMut<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

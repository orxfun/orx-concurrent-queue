use crate::ConcurrentQueue;
use orx_pinned_vec::{ConcurrentPinnedVec, IntoConcurrentPinnedVec};

// impl<'a, T, P> IntoIterator for &'a ConcurrentQueue<T, P>
// where
//     T: Send,
//     P: ConcurrentPinnedVec<T>,
// {
//     type Item = &'a T;

//     type IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         todo!()
//     }
// }

pub struct QueueIterOfRefs<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    iter: P::PtrIter<'a>,
}

impl<'a, T, P> QueueIterOfRefs<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    pub(crate) fn new(iter: P::PtrIter<'a>) -> Self {
        Self { iter }
    }
}

impl<'a, T, P> Iterator for QueueIterOfRefs<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ptr| unsafe { &*ptr })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, P> ExactSizeIterator for QueueIterOfRefs<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

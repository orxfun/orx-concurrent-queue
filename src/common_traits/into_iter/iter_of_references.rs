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

impl<'a, T, P> Iterator for QueueIterOfRefs<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ptr| unsafe { &*ptr })
    }
}

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
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    queue: &'a mut ConcurrentQueue<T, P>,
}

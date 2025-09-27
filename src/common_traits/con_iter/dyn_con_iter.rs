use crate::{ConcurrentQueue, queue::DefaultConVec};
use orx_pinned_vec::ConcurrentPinnedVec;

pub struct DynamicConcurrentIter<T, E, I, P = DefaultConVec<T>>
where
    T: Send,
    E: Fn(&T) -> I,
    I: IntoIterator<Item = T>,
    P: ConcurrentPinnedVec<T>,
{
    queue: ConcurrentQueue<T, P>,
    extend: E,
}

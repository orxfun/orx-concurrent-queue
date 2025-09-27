use orx_pinned_vec::ConcurrentPinnedVec;

pub struct QueueIntoIter<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    iter: P::IntoIter,
}

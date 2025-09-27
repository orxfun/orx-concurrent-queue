use orx_pinned_vec::ConcurrentPinnedVec;

pub struct QueueIterOwned<'a, T, P>
where
    T: Send + 'a,
    P: ConcurrentPinnedVec<T> + 'a,
{
    iter: P::PtrIter<'a>,
}

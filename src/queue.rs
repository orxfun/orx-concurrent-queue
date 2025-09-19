use crate::push_vec::PushVec;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_split_vec::SplitVec;

pub struct ConcurrentQueue<T, P = SplitVec<T>>
where
    T: Send,
    P: IntoConcurrentPinnedVec<T>,
{
    push_vec: PushVec<T, P>,
}

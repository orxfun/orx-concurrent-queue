use crate::state::ConcurrentQueueState;
use orx_pinned_concurrent_col::{PinnedConcurrentCol, prelude::IntoConcurrentPinnedVec};

pub struct ConcurrentQueue<T, P>
where
    T: Send,
    P: IntoConcurrentPinnedVec<T>,
{
    core: PinnedConcurrentCol<T, P::ConPinnedVec, ConcurrentQueueState>,
}

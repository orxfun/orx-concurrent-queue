use crate::ConcurrentQueue;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_split_vec::{Doubling, SplitVec};

type DefaultPinnedVec<T> = SplitVec<T, Doubling>;
pub type DefaultConVec<T> = <DefaultPinnedVec<T> as IntoConcurrentPinnedVec<T>>::ConPinnedVec;

impl<T> ConcurrentQueue<T, DefaultConVec<T>>
where
    T: Send,
{
    pub fn new() -> Self {
        SplitVec::with_doubling_growth_and_max_concurrent_capacity().into()
    }
}

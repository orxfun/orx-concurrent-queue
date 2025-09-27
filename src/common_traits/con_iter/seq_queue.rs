use core::marker::PhantomData;
use orx_pinned_vec::{ConcurrentPinnedVec, PinnedVec};

pub struct DynSeqQueue<T, P, E, I>
where
    P: ConcurrentPinnedVec<T>,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    vec: P,
    written: usize,
    popped: usize,
    extend: E,
    phantom: PhantomData<T>,
}

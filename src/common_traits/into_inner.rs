use crate::ConcurrentQueue;
use orx_fixed_vec::{ConcurrentFixedVec, FixedVec};
use orx_pinned_vec::PinnedVec;
use orx_pinned_vec::{ConcurrentPinnedVec, IntoConcurrentPinnedVec};
use orx_split_vec::prelude::PseudoDefault;
use orx_split_vec::{ConcurrentSplitVec, Growth, GrowthWithConstantTimeAccess, SplitVec};

// impl<T, G> ConcurrentQueue<T, ConcurrentSplitVec<T, G>>
// where
//     T: Send,
//     G: GrowthWithConstantTimeAccess,
// {
//     pub fn into_inner(self) -> SplitVec<T, G> {
//         self.into_inner_core(SplitVec::pseudo_default())
//     }
// }

// impl<T> ConcurrentQueue<T, ConcurrentFixedVec<T>>
// where
//     T: Send,
// {
//     pub fn into_inner(self) -> FixedVec<T> {
//         self.into_inner_core(FixedVec::pseudo_default())
//     }
// }

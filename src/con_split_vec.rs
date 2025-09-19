use orx_split_vec::GrowthWithConstantTimeAccess;
use std::{cell::UnsafeCell, sync::atomic::AtomicUsize};

pub struct ConSplitVec<T, G>
where
    G: GrowthWithConstantTimeAccess,
{
    growth: G,
    data: Vec<UnsafeCell<*mut T>>,
    fragments_capacity: usize,
    taken: AtomicUsize,
    pushed: AtomicUsize,
}

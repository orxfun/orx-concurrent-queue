use alloc::vec::Vec;
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};
use orx_split_vec::GrowthWithConstantTimeAccess;

const fn is_neg(number: usize) -> bool {
    const NEG_BOUND: usize = usize::MAX - 10;
    number > NEG_BOUND
}

pub struct ConSplitVec<T, G>
where
    G: GrowthWithConstantTimeAccess,
{
    growth: G,
    data: Vec<UnsafeCell<*mut T>>,
    fragments_capacity: usize,
}

impl<T, G> ConSplitVec<T, G>
where
    G: GrowthWithConstantTimeAccess,
{
    pub fn new(growth: G, fragments_capacity: usize) -> Self {
        let data = (0..fragments_capacity)
            .map(|_| UnsafeCell::new(core::ptr::null_mut()))
            .collect();

        Self {
            growth,
            data,
            fragments_capacity,
        }
    }

    unsafe fn get_raw_mut_unchecked_fi(&self, f: usize, i: usize) -> *mut T {
        let p = unsafe { *self.data[f].get() };
        unsafe { p.add(i) }
    }

    unsafe fn get_raw_mut_unchecked_idx(&self, idx: usize) -> *mut T {
        let (f, i) = self.growth.get_fragment_and_inner_indices_unchecked(idx);
        unsafe { self.get_raw_mut_unchecked_fi(f, i) }
    }

    pub unsafe fn take(&self, idx: usize) -> T {
        let p = unsafe { self.get_raw_mut_unchecked_idx(idx) };
        let mut value = unsafe { MaybeUninit::uninit().assume_init() };
        core::mem::swap(unsafe { &mut *p }, &mut value);
        value
    }
}

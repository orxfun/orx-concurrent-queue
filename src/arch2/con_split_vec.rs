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
    capacity: AtomicUsize,
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
            capacity: 0.into(),
        }
    }

    pub unsafe fn take(&self, idx: usize) -> T {
        let p = unsafe { self.get_raw_mut_unchecked_idx(idx) };
        let mut value = unsafe { MaybeUninit::uninit().assume_init() };
        core::mem::swap(unsafe { &mut *p }, &mut value);
        value
    }

    pub unsafe fn put(&self, idx: usize, value: T) {
        let p = unsafe { self.get_raw_mut_unchecked_idx(idx) };
        unsafe { p.write(value) };
    }

    unsafe fn get_raw_mut_unchecked_fi(&self, f: usize, i: usize) -> *mut T {
        let p = unsafe { *self.data[f].get() };
        unsafe { p.add(i) }
    }

    unsafe fn get_raw_mut_unchecked_idx(&self, idx: usize) -> *mut T {
        let (f, i) = self.growth.get_fragment_and_inner_indices_unchecked(idx);
        unsafe { self.get_raw_mut_unchecked_fi(f, i) }
    }

    fn num_fragments_for_capacity(&self, capacity: usize) -> usize {
        match capacity {
            0 => 0,
            _ => {
                self.growth
                    .get_fragment_and_inner_indices_unchecked(capacity - 1)
                    .0
                    + 1
            }
        }
    }

    fn capacity_of(&self, f: usize) -> usize {
        self.growth.fragment_capacity_of(f)
    }

    fn layout(len: usize) -> alloc::alloc::Layout {
        alloc::alloc::Layout::array::<T>(len).expect("len must not overflow")
    }

    fn grow_to(&self, new_capacity: usize) -> Result<usize, orx_pinned_vec::PinnedVecGrowthError> {
        let capacity = self.capacity.load(Ordering::Acquire);
        match new_capacity <= capacity {
            true => Ok(capacity),
            false => {
                let mut f = self.num_fragments_for_capacity(capacity);
                let mut current_capacity = capacity;

                while new_capacity > current_capacity {
                    let new_fragment_capacity = self.capacity_of(f);
                    let layout = Self::layout(new_fragment_capacity);
                    let ptr = unsafe { alloc::alloc::alloc(layout) } as *mut T;
                    unsafe { *self.data[f].get() = ptr };

                    f += 1;
                    current_capacity += new_fragment_capacity;
                }

                self.capacity.store(current_capacity, Ordering::Release);

                Ok(current_capacity)
            }
        }
    }
}

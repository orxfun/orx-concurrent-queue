use core::sync::atomic::{AtomicUsize, Ordering};

#[inline(always)]
pub fn comp_exch_weak(atom: &AtomicUsize, current: usize, new: usize) -> Result<usize, usize> {
    atom.compare_exchange_weak(current, new, Ordering::Release, Ordering::Relaxed)
}

#[inline(always)]
pub fn comp_exch(atom: &AtomicUsize, current: usize, new: usize) -> Result<usize, usize> {
    atom.compare_exchange(current, new, Ordering::Release, Ordering::Relaxed)
}

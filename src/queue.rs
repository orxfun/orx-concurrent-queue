use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};

const MAX_NUM_POPPERS: usize = 124;
const OVERFLOWN_THRESHOLD: usize = usize::MAX - MAX_NUM_POPPERS;
const fn has_overflown(prev: usize) -> bool {
    prev == 0 || prev > OVERFLOWN_THRESHOLD
}

pub struct Queue<T: Send> {
    capacity: usize,
    data: *mut T,
    len: AtomicIsize,
    pushed: AtomicUsize,
    popped: AtomicUsize,
}

unsafe impl<T: Send> Sync for Queue<T> {}

impl<T: Send> Drop for Queue<T> {
    fn drop(&mut self) {
        let _vec = unsafe { Vec::from_raw_parts(self.data, 0, self.capacity) };
    }
}

impl<T: Send> Queue<T> {
    pub fn new(capacity: usize) -> Self {
        let mut v = Vec::with_capacity(capacity);
        let data = v.as_mut_ptr();
        let _ = ManuallyDrop::new(v);
        Self {
            capacity,
            data,
            pushed: 0.into(),
            len: 0.into(),
            popped: 0.into(),
        }
    }

    pub fn as_slice(&mut self) -> &[T] {
        let reserved = self.pushed.load(Ordering::Relaxed);
        let pushed = self.len.load(Ordering::Relaxed);
        debug_assert_eq!(reserved as isize, pushed);
        let pushed = pushed as usize;
        let popped = self.popped.load(Ordering::Relaxed);

        let begin = unsafe { self.ptr(popped) };
        let len = pushed - popped;
        unsafe { std::slice::from_raw_parts(begin, len) }
    }

    unsafe fn ptr(&self, idx: usize) -> *mut T {
        unsafe { self.data.add(idx) }
    }

    // shrink

    pub fn pop(&self) -> Option<T> {
        let prev = self.len.fetch_sub(1, Ordering::Acquire);
        match prev {
            p if p <= 0 => {
                let current = p - 1;
                while self
                    .len
                    .compare_exchange_weak(current, prev, Ordering::Acquire, Ordering::Relaxed)
                    .is_err()
                {}
                None
            }
            _ => {
                let idx = self.popped.fetch_add(1, Ordering::Acquire);
                while self.pushed.load(Ordering::Relaxed) <= idx {}
                let value = unsafe { self.ptr(idx).read() };
                Some(value)
            }
        }
    }

    pub fn pull(&self, chunk_size: usize) {
        let prev = self.len.fetch_sub(chunk_size as isize, Ordering::Acquire);
        match prev {
            _ => todo!(),
        }
        todo!()
    }

    // grow

    pub fn push(&self, value: T) {
        let idx = self.pushed.fetch_add(1, Ordering::Acquire);
        unsafe { self.ptr(idx).write(value) };
        self.len.fetch_add(1, Ordering::Release);
    }

    pub fn extend<I, Iter>(&self, values: I)
    where
        Iter: ExactSizeIterator<Item = T>,
        I: IntoIterator<IntoIter = Iter, Item = T>,
    {
        let values = values.into_iter();
        let num_items = values.len();
        unsafe { self.extend_n_items(values, num_items) };
    }

    pub unsafe fn extend_n_items(&self, values: impl IntoIterator<Item = T>, num_items: usize) {
        let values = values.into_iter();
        let idx = self.pushed.fetch_add(num_items, Ordering::Acquire);

        let mut ptr = unsafe { self.ptr(idx) };
        for value in values {
            unsafe { ptr.write(value) };
            unsafe { ptr = ptr.add(1) };
        }

        self.len.fetch_add(num_items as isize, Ordering::Release);
    }
}

use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Queue<T: Send> {
    capacity: usize,
    data: *mut T,
    reserved: AtomicUsize,
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
            reserved: 0.into(),
            pushed: 0.into(),
            popped: 0.into(),
        }
    }

    unsafe fn ptr(&self, idx: usize) -> *mut T {
        unsafe { self.data.add(idx) }
    }

    pub fn push(&self, value: T) {
        let idx = self.reserved.fetch_add(1, Ordering::Acquire);
        unsafe { self.ptr(idx).write(value) };
        self.pushed.fetch_add(1, Ordering::Release);
    }

    pub fn as_slice(&mut self) -> &[T] {
        let reserved = self.reserved.load(Ordering::Relaxed);
        let pushed = self.pushed.load(Ordering::Relaxed);
        let popped = self.popped.load(Ordering::Relaxed);
        assert_eq!(reserved, pushed);
        let begin = unsafe { self.ptr(popped) };
        let len = pushed - popped;
        unsafe { std::slice::from_raw_parts(begin, len) }
    }
}

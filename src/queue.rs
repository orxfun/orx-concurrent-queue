use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Queue<T: Send> {
    capacity: usize,
    data: *mut T,
    push_reserved: AtomicUsize,
    pushed: AtomicUsize,
    pop_reserved: AtomicUsize,
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
            push_reserved: 0.into(),
            pushed: 0.into(),
            pop_reserved: 0.into(),
        }
    }

    pub fn as_slice(&mut self) -> &[T] {
        let reserved = self.push_reserved.load(Ordering::Relaxed);
        let pushed = self.pushed.load(Ordering::Relaxed);
        assert_eq!(reserved, pushed);
        let popped = self.pop_reserved.load(Ordering::Relaxed);

        let begin = unsafe { self.ptr(popped) };
        let len = pushed - popped;
        unsafe { std::slice::from_raw_parts(begin, len) }
    }

    unsafe fn ptr(&self, idx: usize) -> *mut T {
        unsafe { self.data.add(idx) }
    }

    pub fn push(&self, value: T) {
        let idx = self.push_reserved.fetch_add(1, Ordering::Acquire);
        unsafe { self.ptr(idx).write(value) };
        self.pushed.fetch_add(1, Ordering::Release);
    }

    pub fn pop(&self) -> Option<T> {
        let idx = self.pop_reserved.fetch_add(1, Ordering::Acquire);
        while self.pushed.load(Ordering::Relaxed) <= idx {}
        let value = unsafe { self.ptr(idx).read() };
        Some(value)
    }
}

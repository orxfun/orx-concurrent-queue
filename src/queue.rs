use std::mem::ManuallyDrop;

pub struct Queue<T: Send> {
    capacity: usize,
    data: *mut T,
}

impl<T: Send> Queue<T> {
    pub fn new(capacity: usize) -> Self {
        let mut v = Vec::with_capacity(capacity);
        let data = v.as_mut_ptr();
        let _ = ManuallyDrop::new(v);
        Self { capacity, data }
    }
}

impl<T: Send> Drop for Queue<T> {
    fn drop(&mut self) {
        let _vec = unsafe { Vec::from_raw_parts(self.data, 0, self.capacity) };
    }
}

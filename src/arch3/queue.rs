use core::sync::atomic::{AtomicUsize, Ordering};
use orx_concurrent_bag::{ConcurrentBag, Doubling, SplitVec};

pub struct Queue<T>
where
    T: Send,
{
    bag: ConcurrentBag<T, SplitVec<T, Doubling>>,
    popped: AtomicUsize,
    len: AtomicUsize,
}

impl<T> Queue<T>
where
    T: Send,
{
    pub fn new() -> Self {
        Self {
            bag: ConcurrentBag::new(),
            popped: 0.into(),
            len: 0.into(),
        }
    }

    pub fn pop(&self) -> Option<T> {
        let prior = self.len.fetch_sub(1, Ordering::Acquire);
        match prior {
            0 => {
                self.len.store(0, Ordering::Release);
                None
            }
            _ => {
                let idx = self.popped.fetch_add(1, Ordering::SeqCst);
                loop {
                    if idx == self.bag.written_len() {
                        return Some(unsafe { self.bag.take(idx) });
                    }
                }
            }
        }
    }

    pub fn push(&self, value: T) {
        self.bag.push(value);
        self.len.fetch_add(1, Ordering::SeqCst);
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abc() {
        let q = Queue::new();

        for i in 0..10_000 {
            q.push(i);
            dbg!(q.pop());
        }

        // q.push(1);
        // q.push(2);
        // q.push(3);

        // dbg!(q.pop());
        // dbg!(q.pop());
        // dbg!(q.pop());
        // dbg!(q.pop());

        assert_eq!(q.bag.len(), 33);
    }
}

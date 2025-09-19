use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{pop_vec::PopVec, push_vec::PushVec};
use orx_concurrent_iter::ExactSizeConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_split_vec::SplitVec;

pub struct ConcurrentQueue<I, T, P = SplitVec<T>>
where
    T: Send,
    P: IntoConcurrentPinnedVec<T>,
    I: ExactSizeConcurrentIter<Item = T>,
{
    push_vec: PushVec<T, P>,
    pop_vec: PopVec<I>,
    pop_len: AtomicUsize,
}

impl<I, T, P> ConcurrentQueue<I, T, P>
where
    T: Send,
    P: IntoConcurrentPinnedVec<T>,
    I: ExactSizeConcurrentIter<Item = T>,
{
    pub fn pop(&self) -> Option<T> {
        let before = self.pop_len.fetch_sub(1, Ordering::SeqCst);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abc() {
        let a = AtomicUsize::new(2);

        dbg!(&a);
        println!("\n");

        let x = a.fetch_sub(1, Ordering::SeqCst);
        dbg!(x, &a);
        println!("\n");

        let x = a.fetch_sub(1, Ordering::SeqCst);
        dbg!(x, &a);
        println!("\n");

        let x = a.fetch_sub(1, Ordering::SeqCst);
        dbg!(x, &a);
        println!("\n");

        let x = a.fetch_sub(1, Ordering::SeqCst);
        dbg!(x, &a);
        println!("\n");

        let x = a.fetch_sub(1, Ordering::SeqCst);
        dbg!(x, &a);
        println!("\n");

        let x = a.fetch_sub(1, Ordering::SeqCst);
        dbg!(x, &a);
        println!("\n");

        assert_eq!(x, 33);
    }
}

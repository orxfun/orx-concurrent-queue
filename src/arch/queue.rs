use crate::{pop_vec::PopVec, push_vec::PushVec, state::State};
use core::sync::atomic::{AtomicUsize, Ordering};
use orx_concurrent_iter::{ExactSizeConcurrentIter, IntoConcurrentIter};
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_split_vec::SplitVec;

pub struct ConcurrentQueue<T>
where
    T: Send + Sync,
{
    push_vec: PushVec<T>,
    pop_vec: PopVec<T>,
    state: State,
}

impl<T> ConcurrentQueue<T>
where
    T: Send + Sync,
{
    fn switch(&mut self) {
        let iter = self.push_vec.take_out_as_con_iter();
        let mut new_pop_vec = PopVec::from(iter);
        let mut new_push_vec = PushVec::new();

        core::mem::swap(&mut self.pop_vec, &mut new_pop_vec);
        core::mem::swap(&mut self.push_vec, &mut new_push_vec);
    }

    // pub

    pub fn pop(&self) -> Option<T> {
        let popped = self.pop_vec.pop();

        match popped.is_some() {
            true => self.pop(),
            false => {
                //
                todo!()
            }
        }
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

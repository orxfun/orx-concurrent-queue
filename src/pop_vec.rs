use orx_concurrent_iter::{ConcurrentIter, implementations::jagged_arrays::ConIterJaggedOwned};
use orx_split_vec::Doubling;

pub struct PopVec<T>
where
    T: Send + Sync,
{
    con_iter: ConIterJaggedOwned<T, Doubling>,
}

impl<T> PopVec<T>
where
    T: Send + Sync,
{
    pub fn pop(&self) -> Option<T> {
        self.con_iter.next()
    }

    pub fn pop_with_idx(&self) -> Option<(usize, T)> {
        self.con_iter.next_with_idx()
    }
}

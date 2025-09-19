use orx_concurrent_iter::ConcurrentIter;

pub struct PopVec<I>
where
    I: ConcurrentIter,
{
    con_iter: I,
}

impl<I> PopVec<I>
where
    I: ConcurrentIter,
{
    pub fn pop(&self) -> Option<I::Item> {
        self.con_iter.next()
    }

    pub fn pop_with_idx(&self) -> Option<(usize, I::Item)> {
        self.con_iter.next_with_idx()
    }
}

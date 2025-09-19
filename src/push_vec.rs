use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{IntoConcurrentIter, implementations::jagged_arrays::ConIterJaggedOwned};
use orx_split_vec::Doubling;

pub struct PushVec<T>
where
    // TODO: Sync requirement must be dropped from SplitVec
    T: Send + Sync,
{
    bag: ConcurrentBag<T>,
}

impl<T> PushVec<T>
where
    T: Send + Sync,
{
    pub fn push(&self, value: T) -> usize {
        self.bag.push(value)
    }

    pub fn extend<IntoIter, Iter>(&self, values: IntoIter) -> usize
    where
        IntoIter: IntoIterator<Item = T, IntoIter = Iter>,
        Iter: Iterator<Item = T> + ExactSizeIterator,
    {
        self.bag.extend(values)
    }

    pub unsafe fn extend_n_items(
        &self,
        values: impl IntoIterator<Item = T>,
        num_items: usize,
    ) -> usize {
        unsafe { self.bag.extend_n_items(values, num_items) }
    }

    pub fn into_con_iter(self) -> ConIterJaggedOwned<T, Doubling> {
        let vec = self.bag.into_inner();
        vec.into_con_iter()
    }
}

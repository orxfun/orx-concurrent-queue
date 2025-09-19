use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub struct PushVec<T, P>
where
    T: Send,
    P: IntoConcurrentPinnedVec<T>,
{
    bag: ConcurrentBag<T, P>,
}

impl<T, P> PushVec<T, P>
where
    T: Send,
    P: IntoConcurrentPinnedVec<T>,
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
}

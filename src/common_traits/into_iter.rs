use crate::ConcurrentQueue;
use orx_pinned_vec::{ConcurrentPinnedVec, IntoConcurrentPinnedVec};

impl<T, P> IntoIterator for ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
    <P as ConcurrentPinnedVec<T>>::P: IntoConcurrentPinnedVec<T, ConPinnedVec = P>,
{
    type Item = T;

    type IntoIter = P::IntoIter;

    fn into_iter(mut self) -> Self::IntoIter {
        let range = self.valid_range();
        let convec = self.destruct().0;
        // SAFETY: range is the only place with valid elements; positions on other sections
        // are either not initialized or popped.
        unsafe { convec.into_iter(range) }
    }
}

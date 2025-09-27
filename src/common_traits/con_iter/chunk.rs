use crate::{ConcurrentQueue, common_traits::iter::QueueIterOwned};
use orx_pinned_vec::ConcurrentPinnedVec;

pub struct DynChunk<'a, T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
{
    chunk: QueueIterOwned<'a, T, P>,
    extend: &'a E,
    queue: &'a ConcurrentQueue<T, P>,
}

impl<'a, T, E, I, P> Default for DynChunk<'a, T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
{
    fn default() -> Self {
        let x = 12;
        todo!()
    }
}

impl<'a, T, E, I, P> Iterator for DynChunk<'a, T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.chunk.next()?;
        let children = (self.extend)(&n);
        self.queue.extend(children);
        Some(n)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.chunk.len();
        (len, Some(len))
    }
}

impl<'a, T, E, I, P> ExactSizeIterator for DynChunk<'a, T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.chunk.len()
    }
}

use crate::{
    ConcurrentQueue, common_traits::con_iter::chunk_puller::DynChunkPuller, queue::DefaultConVec,
};
use core::sync::atomic::Ordering;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::ConcurrentPinnedVec;

pub struct DynamicConcurrentIter<T, E, I, P = DefaultConVec<T>>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
{
    queue: ConcurrentQueue<T, P>,
    extend: E,
}

impl<T, E, I, P> ConcurrentIter for DynamicConcurrentIter<T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
{
    type Item = T;

    type SequentialIter = core::iter::Empty<T>;

    type ChunkPuller<'i>
        = DynChunkPuller<'i, T, E, I, P>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        todo!()
    }

    fn skip_to_end(&self) {
        let len = self.queue.write_reserved(Ordering::Acquire);
        let _remaining_to_drop = self.queue.pull(len);
    }

    fn next(&self) -> Option<Self::Item> {
        let n = self.queue.pop()?;
        let children = (self.extend)(&n);
        self.queue.extend(children);
        Some(n)
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let min = self.queue.len();
        (min, None)
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        DynChunkPuller::new(&self.extend, &self.queue, chunk_size)
    }
}

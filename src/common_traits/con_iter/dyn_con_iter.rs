use crate::{
    ConcurrentQueue,
    common_traits::con_iter::{chunk_puller::DynChunkPuller, seq_queue::DynSeqQueue},
    queue::DefaultConVec,
};
use core::sync::atomic::Ordering;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::{ConcurrentPinnedVec, IntoConcurrentPinnedVec};

pub struct DynamicConcurrentIter<T, E, I, P = DefaultConVec<T>>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
    <P as ConcurrentPinnedVec<T>>::P: IntoConcurrentPinnedVec<T, ConPinnedVec = P>,
{
    queue: ConcurrentQueue<T, P>,
    extend: E,
}

impl<T, E, I, P> DynamicConcurrentIter<T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
    <P as ConcurrentPinnedVec<T>>::P: IntoConcurrentPinnedVec<T, ConPinnedVec = P>,
{
    pub fn new(queue: ConcurrentQueue<T, P>, extend: E) -> Self {
        Self { queue, extend }
    }
}

impl<T, E, I, P> ConcurrentIter for DynamicConcurrentIter<T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
    <P as ConcurrentPinnedVec<T>>::P: IntoConcurrentPinnedVec<T, ConPinnedVec = P>,
{
    type Item = T;

    type SequentialIter = DynSeqQueue<T, P, E, I>;

    type ChunkPuller<'i>
        = DynChunkPuller<'i, T, E, I, P>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        let (vec, written, popped) = self.queue.destruct();
        DynSeqQueue::new(vec, written, popped, self.extend)
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
        let (idx, n) = self.queue.pop_with_idx()?;
        let children = (self.extend)(&n);
        self.queue.extend(children);
        Some((idx, n))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let min = self.queue.len();
        match min {
            0 => (0, Some(0)),
            n => (n, None),
        }
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        DynChunkPuller::new(&self.extend, &self.queue, chunk_size)
    }
}

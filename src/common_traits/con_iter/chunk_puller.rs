use crate::{ConcurrentQueue, common_traits::con_iter::chunk::DynChunk};
use orx_concurrent_iter::ChunkPuller;
use orx_pinned_vec::ConcurrentPinnedVec;

pub struct DynChunkPuller<'a, T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
{
    extend: &'a E,
    queue: &'a ConcurrentQueue<T, P>,
    chunk_size: usize,
}

impl<'a, T, E, I, P> DynChunkPuller<'a, T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
{
    pub(super) fn new(extend: &'a E, queue: &'a ConcurrentQueue<T, P>, chunk_size: usize) -> Self {
        Self {
            extend,
            queue,
            chunk_size,
        }
    }
}

impl<'a, T, E, I, P> ChunkPuller for DynChunkPuller<'a, T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
{
    type ChunkItem = T;

    type Chunk<'c>
        = DynChunk<'c, T, E, I, P>
    where
        Self: 'c;

    #[inline(always)]
    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        let chunk = self.queue.pull(self.chunk_size)?;
        Some(DynChunk::new(chunk, self.extend, self.queue))
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        let (begin_idx, chunk) = self.queue.pull_with_idx(self.chunk_size)?;
        Some((begin_idx, DynChunk::new(chunk, self.extend, self.queue)))
    }
}

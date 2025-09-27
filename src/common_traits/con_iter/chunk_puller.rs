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
}

impl<'a, T, E, I, P> DynChunkPuller<'a, T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
{
    pub(super) fn new(extend: &'a E, queue: &'a ConcurrentQueue<T, P>) -> Self {
        Self { extend, queue }
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

    fn chunk_size(&self) -> usize {
        todo!()
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        todo!()
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        todo!()
    }
}

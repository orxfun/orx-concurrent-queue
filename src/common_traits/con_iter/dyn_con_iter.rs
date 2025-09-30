use crate::{
    ConcurrentQueue,
    common_traits::con_iter::{chunk_puller::DynChunkPuller, seq_queue::DynSeqQueue},
    queue::DefaultConVec,
};
use core::sync::atomic::Ordering;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::{ConcurrentPinnedVec, IntoConcurrentPinnedVec};
use orx_split_vec::SplitVec;

/// A dynamic [`ConcurrentIter`] which:
/// * naturally shrinks as we iterate,
/// * but can also grow as it allows to add new items to the iterator, during iteration.
///
/// The growth part is managed by the `extend: E` function with the signature `Fn(&T) -> I`,
/// where `I: IntoIterator<Item = T>`.
///
/// In other words, for each element `e` drawn from the iterator, we call `extend(&e)` before
/// returning it to the caller. All elements included in the iterator that `extend` returned
/// are added to the end of the concurrent iterator, to be yield later on.
///
///
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

impl<T, E, I, P> From<(E, ConcurrentQueue<T, P>)> for DynamicConcurrentIter<T, E, I, P>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
    P: ConcurrentPinnedVec<T>,
    <P as ConcurrentPinnedVec<T>>::P: IntoConcurrentPinnedVec<T, ConPinnedVec = P>,
{
    fn from((extend, queue): (E, ConcurrentQueue<T, P>)) -> Self {
        Self { queue, extend }
    }
}

impl<T, E, I> DynamicConcurrentIter<T, E, I, DefaultConVec<T>>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    /// TODO: PLACEHOLDER
    pub fn new(extend: E, initial_elements: impl IntoIterator<Item = T>) -> Self {
        let mut vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        vec.extend(initial_elements);
        let queue = vec.into();
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

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::vec::Vec;
    use rand::Rng;

    #[test]
    fn abc() {
        struct Node {
            value: u64,
            children: Vec<Node>,
        }

        impl Node {
            fn new(rng: &mut impl Rng, value: u64) -> Self {
                let num_children = match value {
                    0 => 0,
                    n => rng.random_range(0..(n as usize)),
                };
                let children = (0..num_children)
                    .map(|i| Self::new(rng, i as u64))
                    .collect();
                Self { value, children }
            }
        }

        fn compute(node_value: u64) {
            // fake computation
            std::thread::sleep(std::time::Duration::from_millis(node_value));
        }

        fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
            &node.children
        }

        // let queue = ConcurrentQueue::new();
        // queue.push(root);
        // let iter = DynamicConcurrentIter::new(queue, extend);
    }
}

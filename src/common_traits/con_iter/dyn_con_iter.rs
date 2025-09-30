use crate::{
    ConcurrentQueue,
    common_traits::con_iter::{chunk_puller::DynChunkPuller, seq_queue::DynSeqQueue},
    queue::DefaultConPinnedVec,
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
/// *The dynamic concurrent iterator internally uses a [`ConcurrentQueue`] which allows for concurrent
/// push / extend simultaneously with pop / pull operations.*
///
/// # Example
///
/// The following example demonstrates a use case for the dynamic concurrent iterator.
/// Notice that the iterator is instantiated with:
/// * a single element which is the root node,
/// * and the extend method which defines how to extend the iterator from each node.
///
/// Including the root, there exist 177 nodes in the tree. We observe that all these
/// nodes are concurrently added to the iterator, popped and processed.
///
/// ```
/// use orx_concurrent_queue::DynamicConcurrentIter;
/// use orx_concurrent_iter::ConcurrentIter;
/// use std::sync::atomic::{AtomicUsize, Ordering};
/// use rand::{Rng, SeedableRng};
/// use rand_chacha::ChaCha8Rng;
///
/// struct Node {
///     value: u64,
///     children: Vec<Node>,
/// }
///
/// impl Node {
///     fn new(rng: &mut impl Rng, value: u64) -> Self {
///         let num_children = match value {
///             0 => 0,
///             n => rng.random_range(0..(n as usize)),
///         };
///         let children = (0..num_children)
///             .map(|i| Self::new(rng, i as u64))
///             .collect();
///         Self { value, children }
///     }
/// }
///
/// fn process(node_value: u64) {
///     // fake computation
///     std::thread::sleep(std::time::Duration::from_millis(node_value));
/// }
///
/// // this defines how the iterator must extend:
/// // each node drawn from the iterator adds its children to the end of the iterator
/// fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
///     &node.children
/// }
///
/// // initiate iter with a single element, `root`
/// // however, the iterator will `extend` on the fly as we keep drawing its elements
/// let root = Node::new(&mut ChaCha8Rng::seed_from_u64(42), 70);
/// let iter = DynamicConcurrentIter::new(extend, [&root]);
///
/// let num_threads = 8;
/// let num_spawned = AtomicUsize::new(0);
/// let num_processed_nodes = AtomicUsize::new(0);
///
/// std::thread::scope(|s| {
///     let mut handles = vec![];
///     for _ in 0..num_threads {
///         handles.push(s.spawn(|| {
///             // allow all threads to be spawned
///             _ = num_spawned.fetch_add(1, Ordering::Relaxed);
///             while num_spawned.load(Ordering::Relaxed) < num_threads {}
///
///             // `next` will first extend `iter` with children of `node,
///             // and only then yield the `node`
///             while let Some(node) = iter.next() {
///                 process(node.value);
///                 _ = num_processed_nodes.fetch_add(1, Ordering::Relaxed);
///             }
///         }));
///     }
/// });
///
/// assert_eq!(num_processed_nodes.into_inner(), 177);
/// ```
pub struct DynamicConcurrentIter<T, E, I, P = DefaultConPinnedVec<T>>
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

impl<T, E, I> DynamicConcurrentIter<T, E, I, DefaultConPinnedVec<T>>
where
    T: Send,
    E: Fn(&T) -> I + Sync,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    /// Creates a new dynamic concurrent iterator:
    ///
    /// * The iterator will initially contain `initial_elements`.
    /// * Before yielding each element, say `e`, to the caller, the elements returned
    ///   by `extend(&e)` will be added to the concurrent iterator, to be yield later.
    ///
    /// This constructor uses a [`ConcurrentQueue`] with the default pinned concurrent
    /// collection under the hood. In order to crate the iterator using a different queue
    /// use the `From`/`Into` traits, as demonstrated below.
    ///
    /// # Examples
    ///
    /// The following is a simple example to demonstrate how the dynamic iterator works.
    ///
    /// ```
    /// use orx_concurrent_queue::DynamicConcurrentIter;
    /// use orx_concurrent_iter::ConcurrentIter;
    ///
    /// let extend = |x: &usize| (*x < 5).then_some(x + 1);
    /// let initial_elements = [1];
    ///
    /// let iter = DynamicConcurrentIter::new(extend, initial_elements);
    /// let all: Vec<_> = iter.item_puller().collect();
    ///
    /// assert_eq!(all, [1, 2, 3, 4, 5]);
    /// ```
    ///
    /// ```
    /// use orx_concurrent_queue::*;
    /// use orx_concurrent_iter::ConcurrentIter;
    ///
    /// let extend = |x: &usize| (*x < 5).then_some(x + 1);
    /// let initial_elements = [1];
    ///
    /// let iter = DynamicConcurrentIter::new(extend, initial_elements);
    /// let all: Vec<_> = iter.item_puller().collect();
    ///
    /// assert_eq!(all, [1, 2, 3, 4, 5]);
    /// ```
    ///
    /// # Examples - From
    ///
    /// In the above example, the underlying pinned vector of the dynamic iterator created
    /// with `new` is a [`SplitVec`] with a [`Doubling`] growth strategy.
    ///
    /// Alternatively, we can use a `SplitVec` with a [`Linear`] growth strategy, or a
    /// pre-allocated [`FixedVec`] as the underlying storage. In order to do so, we can
    /// use the `From` trait.
    ///
    /// [`SplitVec`]: orx_split_vec::SplitVec
    /// [`FixedVec`]: orx_fixed_vec::FixedVec
    /// [`Doubling`]: orx_split_vec::Doubling
    /// [`Linear`]: orx_split_vec::Linear
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
        // SAFETY: we destruct the queue and immediately convert it into a sequential
        // queue together with `popped..written` valid range information.
        let (vec, written, popped) = unsafe { self.queue.destruct() };
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

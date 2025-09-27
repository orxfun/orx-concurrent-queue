use crate::{
    atomic_utils::{comp_exch, comp_exch_weak},
    common_traits::iter::{QueueIterOfMut, QueueIterOfRef, QueueIterOwned},
    write_permit::WritePermit,
};
use core::{
    marker::PhantomData,
    ops::Range,
    sync::atomic::{AtomicUsize, Ordering},
};
use orx_pinned_vec::{ConcurrentPinnedVec, IntoConcurrentPinnedVec};
use orx_split_vec::{Doubling, SplitVec, prelude::PseudoDefault};

type DefaultPinnedVec<T> = SplitVec<T, Doubling>;
pub type DefaultConVec<T> = <DefaultPinnedVec<T> as IntoConcurrentPinnedVec<T>>::ConPinnedVec;

impl<T> Default for ConcurrentQueue<T, DefaultConVec<T>>
where
    T: Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ConcurrentQueue<T, DefaultConVec<T>>
where
    T: Send,
{
    /// Creates a new empty concurrent queue.
    ///
    /// This queue is backed with default concurrent pinned vec, which is the concurrent version of [`SplitVec`] with [`Doubling`] growth.
    ///
    /// In order to create a concurrent queue backed with a particular [`PinnedVec`], you may use the `From` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_queue::ConcurrentQueue;
    /// use orx_split_vec::{SplitVec, ConcurrentSplitVec, Doubling, Linear};
    /// use orx_fixed_vec::{FixedVec, ConcurrentFixedVec};
    ///
    /// let bag: ConcurrentQueue<usize> = ConcurrentQueue::new();
    /// // equivalent to:
    /// let bag: ConcurrentQueue<usize> = SplitVec::new().into();
    /// // equivalent to:
    /// let bag: ConcurrentQueue<usize, ConcurrentSplitVec<_, Doubling>> = SplitVec::with_doubling_growth_and_max_concurrent_capacity().into();
    ///
    /// // in order to create a queue from a different pinned vec, use into, rather than new:
    /// let bag: ConcurrentQueue<usize, _> = SplitVec::with_linear_growth_and_fragments_capacity(10, 64).into();
    /// let bag: ConcurrentQueue<usize, ConcurrentSplitVec<_, Linear>> = SplitVec::with_linear_growth_and_fragments_capacity(10, 64).into();
    ///
    /// let bag: ConcurrentQueue<usize, _> = FixedVec::new(1000).into();
    /// let bag: ConcurrentQueue<usize, ConcurrentFixedVec<usize>> = FixedVec::new(1000).into();
    /// ```
    ///
    /// [`SplitVec`]: orx_split_vec::SplitVec
    /// [`Doubling`]: orx_split_vec::Doubling
    /// [`PinnedVec`]: orx_pinned_vec::PinnedVec
    pub fn new() -> Self {
        SplitVec::with_doubling_growth_and_max_concurrent_capacity().into()
    }
}

/// A high performance and convenient thread safe queue that can concurrently
/// grow and shrink with [`push`], [`extend`], [`pop`] and [`pull`] capabilities.
///
/// [`push`]: crate::ConcurrentQueue::push
/// [`extend`]: crate::ConcurrentQueue::extend
/// [`pop`]: crate::ConcurrentQueue::pop
/// [`pull`]: crate::ConcurrentQueue::pull
///
/// # Examples
///
/// The following example demonstrates a basic usage of the queue within a synchronous program.
/// Note that push, extend, pop and pull methods can be called with a shared reference `&self`.
/// This allows to use the queue conveniently in a concurrent program.
///
/// ```
/// use orx_concurrent_queue::ConcurrentQueue;
///
/// let queue = ConcurrentQueue::new();
///
/// queue.push(0); // [0]
/// queue.push(1); // [0, 1]
///
/// let x = queue.pop(); // [1]
/// assert_eq!(x, Some(0));
///
/// queue.extend(2..7); // [1, 2, 3, 4, 5, 6]
///
/// let x: Vec<_> = queue.pull(4).unwrap().collect(); // [5, 6]
/// assert_eq!(x, vec![1, 2, 3, 4]);
///
/// assert_eq!(queue.len(), 2);
///
/// let vec = queue.into_inner();
/// assert_eq!(vec, vec![5, 6]);
/// ```
/// The following example demonstrates the main purpose of the concurrent queue:
/// to simultaneously push to and pop from the queue.
/// This enables a parallel program where tasks can be handled by multiple threads,
/// while at the same time, new tasks can be created and dynamically added to the queue.
///
/// In the following example, the queue is created with three pre-populated tasks.
/// Every task might potentially lead to new tasks.
/// These new tasks are also added to the back of the queue,
/// to be popped later and potentially add new tasks to the queue.
///
/// ```
/// use orx_concurrent_queue::ConcurrentQueue;
/// use std::sync::atomic::{AtomicUsize, Ordering};
///
/// struct Task {
///     micros: usize,
/// }
///
/// impl Task {
///     fn perform(&self) {
///         use std::{thread::sleep, time::Duration};
///         sleep(Duration::from_micros(self.micros as u64));
///     }
///
///     fn child_tasks(&self) -> impl ExactSizeIterator<Item = Task> {
///         let range = match self.micros < 5 {
///             true => 0..0,
///             false => 0..self.micros,
///         };
///
///         range.rev().take(5).map(|micros| Self { micros })
///     }
/// }
///
/// let queue = ConcurrentQueue::new();
/// for micros in [10, 15, 10] {
///     queue.push(Task { micros });
/// }
///
/// let num_performed_tasks = AtomicUsize::new(queue.len());
///
/// let num_threads = 8;
/// std::thread::scope(|s| {
///     for _ in 0..num_threads {
///         s.spawn(|| {
///             // keep popping a task from front of the queue
///             // as long as the queue is not empty
///             while let Some(task) = queue.pop() {
///                 // create children tasks, add to back
///                 queue.extend(task.child_tasks());
///
///                 // perform the popped task
///                 task.perform();
///
///                 _ = num_performed_tasks.fetch_add(1, Ordering::Relaxed);
///             }
///         });
///     }
/// });
///
/// assert_eq!(num_performed_tasks.load(Ordering::Relaxed), 5046);
/// ```
pub struct ConcurrentQueue<T, P = DefaultConVec<T>>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    vec: P,
    phantom: PhantomData<T>,
    written: AtomicUsize,
    write_reserved: AtomicUsize,
    popped: AtomicUsize,
}

unsafe impl<T, P> Sync for ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
}

impl<T, P> Drop for ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    fn drop(&mut self) {
        if core::mem::needs_drop::<T>() {
            let popped = self.popped.load(Ordering::Relaxed);
            let reserved = self.write_reserved.load(Ordering::Relaxed);
            let written = self.written.load(Ordering::Relaxed);
            assert_eq!(reserved, written);
            for i in popped..written {
                let ptr = unsafe { self.ptr(i) };
                unsafe { ptr.drop_in_place() };
            }
        }
        unsafe { self.vec.set_pinned_vec_len(0) };
    }
}

impl<T, P> From<P> for ConcurrentQueue<T, P::ConPinnedVec>
where
    T: Send,
    P: IntoConcurrentPinnedVec<T>,
{
    fn from(vec: P) -> Self {
        Self {
            phantom: PhantomData,
            written: vec.len().into(),
            write_reserved: vec.len().into(),
            popped: 0.into(),
            vec: vec.into_concurrent(),
        }
    }
}

impl<T, P> ConcurrentQueue<T, P>
where
    T: Send,
    P: ConcurrentPinnedVec<T>,
{
    /// Converts the bag into the underlying pinned vector.
    ///
    /// Whenever the second generic parameter is omitted, the underlying pinned vector is [`SplitVec`] with [`Doubling`] growth.
    ///
    /// [`SplitVec`]: orx_split_vec::SplitVec
    /// [`Doubling`]: orx_split_vec::Doubling
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_queue::ConcurrentQueue;
    /// use orx_split_vec::SplitVec;
    ///
    /// let queue = ConcurrentQueue::new();
    ///
    /// queue.push(0); // [0]
    /// queue.push(1); // [0, 1]
    /// _ = queue.pop(); // [1]
    /// queue.extend(2..7); // [1, 2, 3, 4, 5, 6]
    /// _ = queue.pull(4).unwrap(); // [5, 6]
    ///
    /// let vec: SplitVec<i32> = queue.into_inner();
    /// assert_eq!(vec, vec![5, 6]);
    ///
    /// let vec: Vec<i32> = vec.to_vec();
    /// assert_eq!(vec, vec![5, 6]);
    /// ```
    pub fn into_inner(mut self) -> <P as ConcurrentPinnedVec<T>>::P
    where
        <P as ConcurrentPinnedVec<T>>::P:
            PseudoDefault + IntoConcurrentPinnedVec<T, ConPinnedVec = P>,
    {
        let vec: <P as ConcurrentPinnedVec<T>>::P = PseudoDefault::pseudo_default();
        let mut vec = vec.into_concurrent();
        core::mem::swap(&mut self.vec, &mut vec);

        let a = self.popped.load(Ordering::Relaxed);
        let b = self.written.load(Ordering::Relaxed);
        let len = b - a;
        if a > 0 {
            let src = unsafe { vec.ptr_iter_unchecked(a..b) };
            let dst = unsafe { vec.ptr_iter_unchecked(0..len) };
            for (s, d) in src.zip(dst) {
                unsafe { d.write(s.read()) };
            }
        }

        for x in [&self.written, &self.write_reserved, &self.popped] {
            x.store(0, Ordering::Relaxed);
        }

        unsafe { vec.into_inner(len) }
    }

    // shrink

    /// Pops and returns the element in the front of the queue; returns None if the queue is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_queue::*;
    ///
    /// let queue = ConcurrentQueue::new();
    ///
    /// queue.extend(1..4);
    /// assert_eq!(queue.pop(), Some(1));
    /// assert_eq!(queue.pop(), Some(2));
    /// assert_eq!(queue.pop(), Some(3));
    /// assert_eq!(queue.pop(), None);
    /// ```
    pub fn pop(&self) -> Option<T> {
        let idx = self.popped.fetch_add(1, Ordering::Relaxed);

        loop {
            let written = self.written.load(Ordering::Acquire);
            match idx < written {
                true => return Some(unsafe { self.ptr(idx).read() }),
                false => {
                    if comp_exch(&self.popped, idx + 1, idx).is_ok() {
                        return None;
                    }
                }
            }
        }
    }

    /// Pulls `chunk_size` elements from the front of the queue:
    ///
    /// * returns None if `chunk_size` is zero,
    /// * returns Some of an ExactSizeIterator with `len = chunk_size` if the queue has at least `chunk_size` items,
    /// * returns Some of a non-empty ExactSizeIterator with `len` such that `0 < len < chunk_size` if the queue
    ///   has `len` elements,
    /// * returns None if the queue is empty.
    ///
    /// Therefore, if the method returns a Some variant, the exact size iterator is not empty.
    ///
    /// Pulled elements are guaranteed to be consecutive elements in the queue.
    ///
    /// In order to reduce the number of concurrent state updates, `pull` with a large enough chunk size might be preferred over `pop` whenever possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_queue::*;
    ///
    /// let queue = ConcurrentQueue::new();
    ///
    /// queue.extend(1..6);
    /// assert_eq!(
    ///     queue.pull(2).map(|x| x.collect::<Vec<_>>()),
    ///     Some(vec![1, 2])
    /// );
    /// assert_eq!(
    ///     queue.pull(7).map(|x| x.collect::<Vec<_>>()),
    ///     Some(vec![3, 4, 5])
    /// );
    /// assert_eq!(queue.pull(1).map(|x| x.collect::<Vec<_>>()), None);
    /// ```
    pub fn pull(&self, chunk_size: usize) -> Option<QueueIterOwned<'_, T, P>> {
        match chunk_size > 0 {
            true => {
                let begin_idx = self.popped.fetch_add(chunk_size, Ordering::Relaxed);
                let end_idx = begin_idx + chunk_size;

                loop {
                    let written = self.written.load(Ordering::Acquire);

                    let has_none = begin_idx >= written;
                    let has_some = !has_none;
                    let has_all = end_idx <= written;

                    let range = match (has_some, has_all) {
                        (false, _) => match comp_exch(&self.popped, end_idx, begin_idx).is_ok() {
                            true => return None,
                            false => None,
                        },
                        (true, true) => Some(begin_idx..end_idx),
                        (true, false) => Some(begin_idx..written),
                    };

                    if let Some(range) = range {
                        let ok = match has_all {
                            true => true,
                            false => comp_exch(&self.popped, end_idx, range.end).is_ok(),
                        };

                        if ok {
                            let iter = unsafe { self.vec.ptr_iter_unchecked(range) };
                            return Some(QueueIterOwned::new(iter));
                        }
                    }
                }
            }
            false => None,
        }
    }

    // shrink with idx

    pub(super) fn pop_with_idx(&self) -> Option<(usize, T)> {
        let idx = self.popped.fetch_add(1, Ordering::Relaxed);

        loop {
            let written = self.written.load(Ordering::Acquire);
            match idx < written {
                true => return Some((idx, unsafe { self.ptr(idx).read() })),
                false => {
                    if comp_exch(&self.popped, idx + 1, idx).is_ok() {
                        return None;
                    }
                }
            }
        }
    }

    // grow

    /// Pushes the `value` to the back of the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_queue::*;
    ///
    /// let queue = ConcurrentQueue::new();
    ///
    /// queue.push(1);
    /// queue.push(2);
    /// queue.push(3);
    /// assert_eq!(queue.into_inner(), vec![1, 2, 3]);
    /// ```
    pub fn push(&self, value: T) {
        let idx = self.write_reserved.fetch_add(1, Ordering::Relaxed);
        self.assert_has_capacity_for(idx);

        loop {
            match WritePermit::for_one(self.vec.capacity(), idx) {
                WritePermit::JustWrite => {
                    unsafe { self.ptr(idx).write(value) };
                    break;
                }
                WritePermit::GrowThenWrite => {
                    self.grow_to(idx + 1);
                    unsafe { self.ptr(idx).write(value) };
                    break;
                }
                WritePermit::Spin => {}
            }
        }

        let num_written = idx + 1;
        while comp_exch_weak(&self.written, idx, num_written).is_err() {}
    }

    /// Extends the queue by pushing `values` elements to the back of the queue.
    ///
    /// In order to reduce the number of concurrent state updates, `extend` might be preferred over `push` whenever possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_queue::ConcurrentQueue;
    ///
    /// let queue = ConcurrentQueue::new();
    ///
    /// queue.extend(1..3);
    /// queue.extend(vec![3, 4, 5, 6]);
    ///
    /// assert_eq!(queue.into_inner(), vec![1, 2, 3, 4, 5, 6]);
    /// ```
    pub fn extend<I, Iter>(&self, values: I)
    where
        I: IntoIterator<Item = T, IntoIter = Iter>,
        Iter: ExactSizeIterator<Item = T>,
    {
        let values = values.into_iter();
        let num_items = values.len();

        if num_items > 0 {
            let begin_idx = self.write_reserved.fetch_add(num_items, Ordering::Relaxed);
            let end_idx = begin_idx + num_items;
            let last_idx = begin_idx + num_items - 1;
            self.assert_has_capacity_for(last_idx);

            loop {
                match WritePermit::for_many(self.vec.capacity(), begin_idx, last_idx) {
                    WritePermit::JustWrite => {
                        let iter = unsafe { self.vec.ptr_iter_unchecked(begin_idx..end_idx) };
                        for (p, value) in iter.zip(values) {
                            unsafe { p.write(value) };
                        }
                        break;
                    }
                    WritePermit::GrowThenWrite => {
                        self.grow_to(end_idx);
                        let iter = unsafe { self.vec.ptr_iter_unchecked(begin_idx..end_idx) };
                        for (p, value) in iter.zip(values) {
                            unsafe { p.write(value) };
                        }
                        break;
                    }
                    WritePermit::Spin => {}
                }
            }

            while comp_exch_weak(&self.written, begin_idx, end_idx).is_err() {}
        }
    }

    // get

    /// Returns the number of elements in the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_queue::ConcurrentQueue;
    ///
    /// let queue = ConcurrentQueue::new();
    ///
    /// queue.push(1);
    /// queue.push(2);
    /// assert_eq!(queue.len(), 2);
    ///
    /// queue.extend(vec![3, 4, 5, 6]);
    /// assert_eq!(queue.len(), 6);
    ///
    /// _ = queue.pop();
    /// assert_eq!(queue.len(), 5);
    ///
    /// _ = queue.pull(4);
    /// assert_eq!(queue.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.written.load(Ordering::Relaxed) - self.popped.load(Ordering::Relaxed)
    }

    /// Returns true if the queue is empty, false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_queue::ConcurrentQueue;
    ///
    /// let queue = ConcurrentQueue::new();
    ///
    /// assert!(queue.is_empty());
    ///
    /// queue.push(1);
    /// queue.push(2);
    /// assert!(!queue.is_empty());
    ///
    /// _ = queue.pull(4);
    /// assert!(queue.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.written.load(Ordering::Relaxed) == self.popped.load(Ordering::Relaxed)
    }

    /// Returns an iterator of references to items in the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_queue::ConcurrentQueue;
    ///
    /// let mut queue = ConcurrentQueue::new();
    ///
    /// queue.push(1);
    /// queue.push(2);
    /// queue.push(3);
    ///
    /// let sum: i32 = queue.iter().sum();
    /// assert_eq!(sum, 6);
    /// ```
    ///
    /// # Safety
    ///
    /// Notice that this call requires a mutually exclusive `&mut self` reference.
    /// This is due to the fact that iterators are lazy and they are not necessarily consumed immediately.
    /// On the other hand, concurrent queue allows for popping elements from the queue with a shared reference.
    /// This could've led to the following undefined behavior.
    ///
    /// To prevent this, `iter` requires a mutually exclusive reference, and hence, the following code does not compile.
    ///
    /// ```compile_fail
    /// use orx_concurrent_queue::ConcurrentQueue;
    ///
    /// let queue = ConcurrentQueue::new();
    ///
    /// queue.push(1);
    /// queue.push(2);
    /// queue.push(3);
    ///
    /// let iter = queue.iter(); // iterator over elements 1, 2 and 3
    ///
    /// _ = queue.pop(); // 1 is removed
    ///
    /// let sum = iter.sum(); // UB
    /// ```
    pub fn iter(&mut self) -> impl ExactSizeIterator<Item = &T> {
        QueueIterOfRef::<T, P>::new(self.ptr_iter())
    }

    /// Returns an iterator of mutable references to items in the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_queue::ConcurrentQueue;
    ///
    /// let mut queue = ConcurrentQueue::new();
    ///
    /// queue.push(1);
    /// queue.push(2);
    /// queue.push(3);
    ///
    /// for x in queue.iter_mut() {
    ///     *x += 10;
    /// }
    ///
    /// assert_eq!(queue.into_inner(), vec![11, 12, 13]);
    /// ```
    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = &mut T> {
        QueueIterOfMut::<T, P>::new(self.ptr_iter())
    }

    // helpers

    #[inline(always)]
    unsafe fn ptr(&self, idx: usize) -> *mut T {
        unsafe { self.vec.get_ptr_mut(idx) }
    }

    #[inline(always)]
    fn assert_has_capacity_for(&self, idx: usize) {
        assert!(
            idx < self.vec.max_capacity(),
            "Out of capacity. Underlying pinned vector cannot grow any further while being concurrently safe."
        );
    }

    fn grow_to(&self, new_capacity: usize) {
        _ = self
            .vec
            .grow_to(new_capacity)
            .expect("The underlying pinned vector reached its capacity and failed to grow");
    }

    pub(super) fn valid_range(&mut self) -> Range<usize> {
        self.popped.load(Ordering::Relaxed)..self.written.load(Ordering::Relaxed)
    }

    pub(super) fn ptr_iter(&mut self) -> P::PtrIter<'_> {
        let range = self.valid_range();
        // SAFETY: with a mut ref, we ensure that the range contains all and only valid values
        unsafe { self.vec.ptr_iter_unchecked(range) }
    }

    pub(super) fn into_con_pinvec(mut self) -> P
    where
        <P as ConcurrentPinnedVec<T>>::P: IntoConcurrentPinnedVec<T, ConPinnedVec = P>,
    {
        let vec: <P as ConcurrentPinnedVec<T>>::P = PseudoDefault::pseudo_default();
        let mut vec = vec.into_concurrent();
        core::mem::swap(&mut self.vec, &mut vec);

        self.popped.store(0, Ordering::Relaxed);
        self.write_reserved.store(0, Ordering::Relaxed);
        self.written.store(0, Ordering::Relaxed);

        vec
    }

    pub(super) fn write_reserved(&self, order: Ordering) -> usize {
        self.write_reserved.load(order)
    }
}

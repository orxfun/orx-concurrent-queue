use crate::queue::ConcurrentQueue;
use alloc::string::ToString;
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};
use orx_split_vec::{SplitVec, prelude::PseudoDefault};
use std::{fmt::Debug, hash::Hash};
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 51;
#[cfg(not(miri))]
const N: usize = 4735;

const NUM_PUSHERS_POPPERS: usize = 8;

#[test_matrix(
    [FixedVec::new(N * NUM_PUSHERS_POPPERS), SplitVec::with_doubling_growth_and_max_concurrent_capacity(), SplitVec::with_linear_growth_and_fragments_capacity(10, 64)],
    [|x| x, |x| x.to_string()],
    [0, 3, N]
)]
fn into_inner<P, T>(vec: P, f: impl Fn(usize) -> T + Sync, num_popped: usize)
where
    P: IntoConcurrentPinnedVec<T>
        + PinnedVec<T>
        + Clone
        + PseudoDefault
        + Debug
        + PartialEq<Vec<T>>,
    T: Send + Clone + Ord + Debug + Hash,
{
    let vec = vec.clone();
    assert!(vec.is_empty());

    let f = &f;
    let queue = ConcurrentQueue::from(vec);
    let q = &queue;

    for i in 0..N * NUM_PUSHERS_POPPERS {
        q.push(f(i));
    }

    std::thread::scope(|s| {
        for _ in 0..NUM_PUSHERS_POPPERS {
            s.spawn(move || {
                for _ in 0..num_popped {
                    _ = q.pop();
                }
            });
        }
    });

    let mut queue = queue.into_inner();
    queue.sort();

    let total_num_popped = NUM_PUSHERS_POPPERS * num_popped;
    let expected_len = NUM_PUSHERS_POPPERS * N - total_num_popped;

    let mut expected: Vec<T> = (0..expected_len).map(|i| f(total_num_popped + i)).collect();
    expected.sort();

    assert_eq!(queue, expected);
}

use crate::queue::ConcurrentQueue;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_split_vec::SplitVec;
use std::fmt::Debug;
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 51;
#[cfg(not(miri))]
const N: usize = 4735;

const NUM_PUSHERS: usize = 4;

#[test_matrix(
    [FixedVec::new(N * NUM_PUSHERS), SplitVec::with_doubling_growth_and_max_concurrent_capacity(), SplitVec::with_linear_growth_and_fragments_capacity(10, 64)],
    [|x| x, |x| x.to_string()])
]
fn push<P, T>(vec: P, f: impl Fn(usize) -> T + Sync)
where
    P: IntoConcurrentPinnedVec<T>,
    T: Send + Clone + Ord + Debug,
{
    assert!(vec.is_empty());

    let mut expected = vec![];
    for t in 0..NUM_PUSHERS {
        for i in 0..N {
            expected.push(f(t * N + i));
        }
    }
    expected.sort();

    let queue = ConcurrentQueue::from(vec);
    let q = &queue;
    let f = &f;

    std::thread::scope(|s| {
        for t in 0..NUM_PUSHERS {
            s.spawn(move || {
                for i in 0..N {
                    q.push(f(t * N + i));
                }
            });
        }
    });

    let mut collected = vec![];
    while let Some(values) = queue.pull(queue.len()) {
        collected.extend(values);
    }
    collected.sort();

    assert_eq!(collected, expected);
}

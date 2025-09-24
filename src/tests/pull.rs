use crate::queue::ConcurrentQueue;
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_split_vec::SplitVec;
use std::fmt::Debug;
use test_case::test_matrix;

enum TryPull {
    Fewer,
    More,
}

#[cfg(miri)]
const N: usize = 51;
#[cfg(not(miri))]
const N: usize = 4735;

const NUM_PULLERS: usize = 4;

#[test_matrix(
    [TryPull::Fewer, TryPull::More],
    [FixedVec::new(N * NUM_PULLERS), SplitVec::with_doubling_growth_and_max_concurrent_capacity(), SplitVec::with_linear_growth_and_fragments_capacity(10, 64)],
    [|x| x, |x| x.to_string()],
    [1, 14, 10000])
]
fn pull<P, T>(p: TryPull, mut vec: P, f: impl Fn(usize) -> T + Sync, chunk_size: usize)
where
    P: IntoConcurrentPinnedVec<T>,
    T: Send + Clone + Ord + Debug,
{
    assert!(vec.is_empty());

    let capacity = NUM_PULLERS * N;
    for i in 0..capacity {
        vec.push(f(i));
    }

    let queue = ConcurrentQueue::from(vec);
    let q = &queue;
    let collected = ConcurrentBag::new();

    let num_pop = match p {
        TryPull::Fewer => N.saturating_sub(25) / chunk_size,
        TryPull::More => (N / chunk_size) + 25,
    };

    std::thread::scope(|s| {
        for _ in 0..NUM_PULLERS {
            s.spawn(|| {
                for _ in 0..num_pop {
                    if let Some(value) = q.pull(chunk_size) {
                        collected.extend(value);
                    }
                }
            });
        }
    });

    let mut collected = collected.into_inner().to_vec();
    collected.sort();

    let mut expected: Vec<_> = (0..collected.len()).map(f).collect();
    expected.sort();

    assert_eq!(collected, expected);
}

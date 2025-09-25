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

const NUM_EXTENDERS: usize = 4;

#[test_matrix(
    [FixedVec::new(17 * N * NUM_EXTENDERS)],
    [|x| x],
    [ 17])
]
fn xyz<P, T>(vec: P, f: impl Fn(usize) -> T + Sync, chunk_size: usize)
where
    P: IntoConcurrentPinnedVec<T>,
    T: Send + Clone + Ord + Debug,
{
    assert!(vec.is_empty());

    let mut expected = vec![];
    for t in 0..NUM_EXTENDERS {
        for i in 0..N {
            match usize::is_multiple_of(i, 2) {
                true => expected.extend((0..chunk_size).map(|j| f(t * N + i + j))),
                false => expected.extend(
                    (0..chunk_size)
                        .map(|j| f(t * N + i + j))
                        .collect::<Vec<_>>(),
                ),
            }
        }
    }
    expected.sort();

    let queue = ConcurrentQueue::from(vec);
    let q = &queue;
    let f = &f;

    std::thread::scope(|s| {
        for t in 0..NUM_EXTENDERS {
            s.spawn(move || {
                for i in 0..N {
                    match usize::is_multiple_of(i, 2) {
                        true => q.extend((0..chunk_size).map(|j| f(t * N + i + j))),
                        false => q.extend(
                            (0..chunk_size)
                                .map(|j| f(t * N + i + j))
                                .collect::<Vec<_>>(),
                        ),
                    }
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

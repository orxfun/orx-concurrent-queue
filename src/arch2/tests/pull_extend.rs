use crate::queue::ConcurrentQueue;
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_split_vec::SplitVec;
use std::{collections::HashSet, fmt::Debug, hash::Hash};
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 51;
#[cfg(not(miri))]
const N: usize = 1234;

const NUM_PULLERS_EXTENDERS: usize = 8;

#[test_matrix(
    [
        FixedVec::new(15 * N * NUM_PULLERS_EXTENDERS),
        SplitVec::with_doubling_growth_and_max_concurrent_capacity(),
        SplitVec::with_linear_growth_and_fragments_capacity(10, 1024)
    ],
    [|x| x.to_string()],
    [2, 14],
    [2, 14, 1000])
]
fn pull_extend<P, T>(vec: P, f: impl Fn(usize) -> T + Sync, extend_size: usize, chunk_size: usize)
where
    P: IntoConcurrentPinnedVec<T> + Clone,
    T: Send + Clone + Ord + Debug + Hash,
{
    let vec = vec.clone();
    assert!(vec.is_empty());

    let f = &f;
    let queue = ConcurrentQueue::from(vec);
    let q = &queue;
    let collected = ConcurrentBag::new();

    let mut potential = HashSet::new();
    for t in 0..NUM_PULLERS_EXTENDERS {
        match usize::is_multiple_of(t, 2) {
            true => {}
            false => {
                for i in 0..N {
                    let values = (0..extend_size).map(|j| f(t * N + i * j));
                    potential.extend(values);
                }
            }
        }
    }

    std::thread::scope(|s| {
        for t in 0..NUM_PULLERS_EXTENDERS {
            match usize::is_multiple_of(t, 2) {
                true => {
                    s.spawn(|| {
                        for _ in 0..N {
                            if let Some(values) = q.pull(chunk_size) {
                                collected.extend(values);
                            }
                        }
                    });
                }
                false => {
                    s.spawn(move || {
                        for i in 0..N {
                            let values = (0..extend_size).map(|j| f(t * N + i * j));
                            q.extend(values);
                        }
                    });
                }
            }
        }
    });

    // let mut collected = collected.into_inner().to_vec();
    // collected.sort();

    // let mut iter = collected.iter().cloned();
    // if let Some(mut prev) = iter.next() {
    //     for c in iter {
    //         potential.contains(&c);
    //         if prev == c {
    //             let v = collected.iter().cloned().take(10).collect::<Vec<_>>();
    //             dbg!(v);
    //         }
    //         assert_ne!(prev, c);
    //         prev = c;
    //     }
    // }
}

#[test]
fn abc() {
    use orx_pinned_vec::PinnedVec;

    for _ in 0..10 {
        // let vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let vec = FixedVec::new(10_000);
        let f = |x: usize| x.to_string();
        let extend_size = 1;
        let chunk_size = 1;

        let vec = vec.clone();
        assert!(vec.is_empty());

        let f = &f;
        let queue = ConcurrentQueue::from(vec);
        let q = &queue;
        let collected = ConcurrentBag::new();

        let mut potential = HashSet::new();
        for t in 0..NUM_PULLERS_EXTENDERS {
            match usize::is_multiple_of(t, 2) {
                true => {}
                false => {
                    for i in 0..N {
                        let values = (0..extend_size).map(|j| f(t * N + i * j));
                        potential.extend(values);
                    }
                }
            }
        }

        std::thread::scope(|s| {
            for t in 0..NUM_PULLERS_EXTENDERS {
                match usize::is_multiple_of(t, 2) {
                    true => {
                        s.spawn(|| {
                            for _ in 0..N {
                                if let Some(values) = q.pull(chunk_size) {
                                    collected.extend(values);
                                }
                            }
                        });
                    }
                    false => {
                        s.spawn(move || {
                            for i in 0..N {
                                let values = (0..extend_size).map(|j| f(t * N + i * j));
                                q.extend(values);
                            }
                        });
                    }
                }
            }
        });

        // let mut collected = collected.into_inner().to_vec();
        // collected.sort();

        // let mut iter = collected.iter().cloned();
        // if let Some(mut prev) = iter.next() {
        //     for c in iter {
        //         potential.contains(&c);
        //         if prev == c {
        //             let v = collected.iter().cloned().take(10).collect::<Vec<_>>();
        //             dbg!(v);
        //         }
        //         assert_ne!(prev, c);
        //         prev = c;
        //     }
        // }
    }
}

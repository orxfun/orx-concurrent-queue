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
const N: usize = 100; // 4735;

const NUM_PUSHERS_POPPERS: usize = 8;

#[test_matrix(
    // [FixedVec::new(N * NUM_PUSHERS_POPPERS), SplitVec::with_doubling_growth_and_max_concurrent_capacity(), SplitVec::with_linear_growth_and_fragments_capacity(10, 64)],
    // [|x| x, |x| x.to_string()])
    [FixedVec::new(N * NUM_PUSHERS_POPPERS)],
    [|x| x.to_string()])
]
fn push_pop<P, T>(vec: P, f: impl Fn(usize) -> T + Sync)
where
    P: IntoConcurrentPinnedVec<T> + Clone,
    T: Send + Clone + Ord + Debug + Hash,
{
    for _ in 0..100 {
        let vec = vec.clone();
        assert!(vec.is_empty());

        let f = &f;
        let queue = ConcurrentQueue::from(vec);
        let q = &queue;
        let collected = ConcurrentBag::new();

        let mut potential = HashSet::new();
        for t in 0..NUM_PUSHERS_POPPERS {
            match usize::is_multiple_of(t, 2) {
                true => {}
                false => {
                    for i in 0..N {
                        potential.insert(f(t * N + i));
                    }
                }
            }
        }

        std::thread::scope(|s| {
            for t in 0..NUM_PUSHERS_POPPERS {
                match usize::is_multiple_of(t, 2) {
                    true => {
                        s.spawn(|| {
                            for _ in 0..N {
                                if let Some(value) = q.pop() {
                                    collected.push(value);
                                }
                            }
                        });
                    }
                    false => {
                        s.spawn(move || {
                            for i in 0..N {
                                q.push(f(t * N + i));
                            }
                        });
                    }
                }
            }
        });

        let mut collected = collected.into_inner().to_vec();
        collected.sort();

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

// #[test]
fn abc() {
    let vec = FixedVec::new(1_000_000);
    let f = |x| x;

    let f = &f;
    let queue = ConcurrentQueue::from(vec);
    let q = &queue;
    let collected = ConcurrentBag::new();

    let mut potential = HashSet::new();
    for t in 0..NUM_PUSHERS_POPPERS {
        match usize::is_multiple_of(t, 2) {
            true => {}
            false => {
                for i in 0..N {
                    potential.insert(f(t * N + i));
                }
            }
        }
    }

    std::thread::scope(|s| {
        for t in 0..NUM_PUSHERS_POPPERS {
            match usize::is_multiple_of(t, 2) {
                true => {
                    s.spawn(|| {
                        for _ in 0..N {
                            let mut local = vec![];
                            if let Some(value) = q.pop() {
                                local.push(value);
                            }
                            collected.extend(local);
                        }
                    });
                }
                false => {
                    s.spawn(move || {
                        for i in 0..N {
                            q.push(f(t * N + i));
                        }
                    });
                }
            }
        }
    });

    let mut collected = collected.into_inner().to_vec();
    collected.sort();

    let mut prev = Default::default();
    for c in collected.iter().cloned() {
        // potential.contains(&c);
        assert_ne!(prev, c);
        prev = c;
    }

    dbg!(&q.state);

    // assert_eq!(q.len(std::sync::atomic::Ordering::AcqRel), 33);
}

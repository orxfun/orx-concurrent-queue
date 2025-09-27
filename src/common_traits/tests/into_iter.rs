use crate::ConcurrentQueue;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_split_vec::SplitVec;
use std::string::{String, ToString};
use test_case::test_matrix;

#[test_matrix([
    FixedVec::new(20),
    SplitVec::with_doubling_growth_and_max_concurrent_capacity(),
    SplitVec::with_linear_growth_and_fragments_capacity(2, 64),
])]
fn into_iter_empty<P>(vec: P)
where
    P: IntoConcurrentPinnedVec<String> + Clone,
{
    let queue = ConcurrentQueue::from(vec);
    let iter = queue.into_iter();
    assert_eq!(iter.count(), 0);
}

#[test_matrix([
    FixedVec::new(30),
    SplitVec::with_doubling_growth_and_max_concurrent_capacity(),
    SplitVec::with_linear_growth_and_fragments_capacity(2, 64),
])]
fn into_iter_none_used<P>(vec: P)
where
    P: IntoConcurrentPinnedVec<String> + Clone,
{
    let queue = ConcurrentQueue::from(vec);
    for i in 0..20 {
        queue.push(i.to_string());
    }

    let iter = queue.into_iter();
    assert_eq!(iter.count(), 20);
}

#[test_matrix([
    FixedVec::new(30),
    SplitVec::with_doubling_growth_and_max_concurrent_capacity(),
    SplitVec::with_linear_growth_and_fragments_capacity(2, 64),
])]
fn into_iter_fully_used<P>(vec: P)
where
    P: IntoConcurrentPinnedVec<String> + Clone,
{
    let queue = ConcurrentQueue::from(vec);
    for i in 0..20 {
        queue.push(i.to_string());
    }
    while let Some(_) = queue.pop() {}

    let iter = queue.into_iter();
    assert_eq!(iter.count(), 0);
}

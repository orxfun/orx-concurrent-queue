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
    let iter = || {
        let queue = ConcurrentQueue::from(vec.clone());
        queue.into_iter()
    };

    let consume_all = iter().count();
    assert_eq!(consume_all, 0);

    let mut consume_half = iter();
    for _ in 0..10 {
        _ = consume_half.next();
    }

    let _consume_none = iter();
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
    let iter = || {
        let queue = ConcurrentQueue::from(vec.clone());
        for i in 0..20 {
            queue.push(i.to_string());
        }
        queue.into_iter()
    };

    let consume_all = iter().count();
    assert_eq!(consume_all, 20);

    let mut consume_half = iter();
    for _ in 0..10 {
        _ = consume_half.next();
    }

    let _consume_none = iter();
}

#[test_matrix([
    FixedVec::new(30),
    SplitVec::with_doubling_growth_and_max_concurrent_capacity(),
    SplitVec::with_linear_growth_and_fragments_capacity(2, 64),
])]
fn into_iter_half_used<P>(vec: P)
where
    P: IntoConcurrentPinnedVec<String> + Clone,
{
    let iter = || {
        let queue = ConcurrentQueue::from(vec.clone());
        for i in 0..20 {
            queue.push(i.to_string());
        }
        for _ in 0..7 {
            _ = queue.pop();
        }
        queue.into_iter()
    };

    let consume_all = iter().count();
    assert_eq!(consume_all, 13);

    let mut consume_half = iter();
    for _ in 0..10 {
        _ = consume_half.next();
    }

    let _consume_none = iter();
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
    let iter = || {
        let queue = ConcurrentQueue::from(vec.clone());
        for i in 0..20 {
            queue.push(i.to_string());
        }
        while let Some(_) = queue.pop() {}
        queue.into_iter()
    };

    let consume_all = iter().count();
    assert_eq!(consume_all, 0);

    let mut consume_half = iter();
    for _ in 0..10 {
        _ = consume_half.next();
    }

    let _consume_none = iter();
}

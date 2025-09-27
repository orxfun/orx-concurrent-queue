use std::dbg;

use crate::{ConcurrentQueue, common_traits::dyn_con_iter::dyn_con_iter::DynamicConcurrentIter};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use orx_split_vec::{Doubling, Linear, SplitVec};
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 125;
#[cfg(not(miri))]
const N: usize = 4735;

fn new_vec_fixed(n: usize) -> FixedVec<String> {
    let mut vec = Vec::with_capacity(n + 10);
    vec.extend((0..n).map(|i| (i + 1).to_string()));
    vec.into()
}

fn new_vec_doubling(n: usize) -> SplitVec<String, Doubling> {
    let mut vec = SplitVec::new();
    vec.extend((0..n).map(|i| (i + 1).to_string()));
    vec
}

fn new_vec_linear(n: usize) -> SplitVec<String, Linear> {
    let mut vec = SplitVec::with_linear_growth_and_fragments_capacity(10, 1024);
    vec.extend((0..n).map(|i| (i + 1).to_string()));
    vec
}

#[test_matrix([
    new_vec_fixed,
    //  new_vec_doubling, new_vec_linear
     ])]
fn basic_iter<P>(vec: impl Fn(usize) -> P)
where
    P: IntoConcurrentPinnedVec<String>,
{
    // 1 2 0 0 1 0
    let queue = ConcurrentQueue::from(vec(2));
    let extend = |s: &String| {
        let i: usize = s.parse().unwrap();
        (0..i).map(|x| x.to_string())
    };
    // let iter = DynamicConcurrentIter::new(queue, extend);

    // assert_eq!(iter.next(), Some(1.to_string()));
    // assert_eq!(iter.next(), Some(2.to_string()));
    // assert_eq!(iter.next(), Some(0.to_string()));
    // assert_eq!(iter.next(), Some(0.to_string()));
    // assert_eq!(iter.next(), Some(1.to_string()));
    // assert_eq!(iter.next(), Some(0.to_string()));
    // assert_eq!(iter.next(), None);
    // assert_eq!(iter.next(), None);
    // assert_eq!(iter.next(), None);

    // dbg!(iter.queue.len());
    // assert_eq!(iter.queue.len(), 33);

    // 1 2 3 0 0 1 0 1 2 0 0 0 1 0
    let queue = ConcurrentQueue::from(vec(3));
    let extend = |s: &String| {
        let i: usize = s.parse().unwrap();
        (0..i).map(|x| x.to_string())
    };
    let iter = DynamicConcurrentIter::new(queue, extend);

    assert_eq!(iter.next(), Some(1.to_string()));
    assert_eq!(iter.next(), Some(2.to_string()));
    assert_eq!(iter.next(), Some(3.to_string()));
    assert_eq!(iter.next(), Some(0.to_string()));
    assert_eq!(iter.next(), Some(0.to_string()));
    assert_eq!(iter.next(), Some(1.to_string()));
    assert_eq!(iter.next(), Some(0.to_string()));
    assert_eq!(iter.next(), Some(1.to_string()));
    assert_eq!(iter.next(), Some(2.to_string()));
    assert_eq!(iter.next(), Some(0.to_string()));
    assert_eq!(iter.next(), Some(0.to_string()));
    assert_eq!(iter.next(), Some(0.to_string()));
    assert_eq!(iter.next(), Some(1.to_string()));
    // assert_eq!(iter.next(), Some(0.to_string()));

    // dbg!(iter.queue.len());
    // assert_eq!(iter.queue.len(), 33);

    // assert_eq!(iter.next(), Some(1.to_string()));
    // assert_eq!(iter.next(), Some(0.to_string()));

    // assert_eq!(iter.next(), Some(1.to_string()));
    // assert_eq!(iter.next(), Some(0.to_string()));
}

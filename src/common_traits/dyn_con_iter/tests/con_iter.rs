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

fn new_vec_fixed(n: usize, capacity: usize) -> FixedVec<String> {
    let mut vec = Vec::with_capacity(capacity + 10);
    vec.extend((0..n).map(|i| (i + 1).to_string()));
    vec.into()
}

fn new_vec_doubling(n: usize, _capacity: usize) -> SplitVec<String, Doubling> {
    let mut vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
    vec.extend((0..n).map(|i| (i + 1).to_string()));
    vec
}

fn new_vec_linear(n: usize, _capacity: usize) -> SplitVec<String, Linear> {
    let mut vec = SplitVec::with_linear_growth_and_fragments_capacity(10, 1024);
    vec.extend((0..n).map(|i| (i + 1).to_string()));
    vec
}

#[test_matrix([new_vec_fixed, new_vec_doubling, new_vec_linear])]
fn basic_iter<P>(vec: impl Fn(usize, usize) -> P)
where
    P: IntoConcurrentPinnedVec<String>,
{
    // 1 2 3 0 0 1 0 1 2 0 0 0 1 0
    let queue = ConcurrentQueue::from(vec(3, 20));
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
    assert_eq!(iter.next(), Some(0.to_string()));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

#[test_matrix([new_vec_fixed, new_vec_doubling, new_vec_linear])]
fn basic_iter_with_idx<P>(vec: impl Fn(usize, usize) -> P)
where
    P: IntoConcurrentPinnedVec<String>,
{
    // 1 2 3 0 0 1 0 1 2 0 0 0 1 0
    let queue = ConcurrentQueue::from(vec(3, 20));
    let extend = |s: &String| {
        let i: usize = s.parse().unwrap();
        (0..i).map(|x| x.to_string())
    };
    let iter = DynamicConcurrentIter::new(queue, extend);

    assert_eq!(iter.next_with_idx(), Some((0, 1.to_string())));
    assert_eq!(iter.next_with_idx(), Some((1, 2.to_string())));
    assert_eq!(iter.next_with_idx(), Some((2, 3.to_string())));
    assert_eq!(iter.next_with_idx(), Some((3, 0.to_string())));
    assert_eq!(iter.next_with_idx(), Some((4, 0.to_string())));
    assert_eq!(iter.next_with_idx(), Some((5, 1.to_string())));
    assert_eq!(iter.next_with_idx(), Some((6, 0.to_string())));
    assert_eq!(iter.next_with_idx(), Some((7, 1.to_string())));
    assert_eq!(iter.next_with_idx(), Some((8, 2.to_string())));
    assert_eq!(iter.next_with_idx(), Some((9, 0.to_string())));
    assert_eq!(iter.next_with_idx(), Some((10, 0.to_string())));
    assert_eq!(iter.next_with_idx(), Some((11, 0.to_string())));
    assert_eq!(iter.next_with_idx(), Some((12, 1.to_string())));
    assert_eq!(iter.next_with_idx(), Some((13, 0.to_string())));
    assert_eq!(iter.next_with_idx(), None);
    assert_eq!(iter.next_with_idx(), None);
    assert_eq!(iter.next_with_idx(), None);
    assert_eq!(iter.next_with_idx(), None);
}

#[test_matrix([new_vec_fixed, new_vec_doubling, new_vec_linear])]
fn size_hint<P>(vec: impl Fn(usize, usize) -> P)
where
    P: IntoConcurrentPinnedVec<String>,
{
    // 1 2 3 0 0 1 0 1 2 0 0 0 1 0
    let queue = ConcurrentQueue::from(vec(3, 20));
    let extend = |s: &String| {
        let i: usize = s.parse().unwrap();
        (0..i).map(|x| x.to_string())
    };
    let iter = DynamicConcurrentIter::new(queue, extend);

    // 1 2 3
    assert_eq!(iter.size_hint(), (3, None));

    _ = iter.next(); // 2 3 0
    assert_eq!(iter.size_hint(), (3, None));

    _ = iter.next(); // 3 0 0 1
    assert_eq!(iter.size_hint(), (4, None));

    _ = iter.next(); // 0 0 1 0 1 2
    assert_eq!(iter.size_hint(), (6, None));

    _ = iter.next(); // 0 1 0 1 2
    assert_eq!(iter.size_hint(), (5, None));

    _ = iter.next(); // 1 0 1 2
    assert_eq!(iter.size_hint(), (4, None));

    _ = iter.next(); // 0 1 2 0
    assert_eq!(iter.size_hint(), (4, None));

    _ = iter.next(); // 1 2 0
    assert_eq!(iter.size_hint(), (3, None));

    _ = iter.next(); // 2 0 0
    assert_eq!(iter.size_hint(), (3, None));

    _ = iter.next(); // 0 0 0 1
    assert_eq!(iter.size_hint(), (4, None));

    _ = iter.next(); // 0 0 1
    assert_eq!(iter.size_hint(), (3, None));

    _ = iter.next(); // 0 1
    assert_eq!(iter.size_hint(), (2, None));

    _ = iter.next(); // 1
    assert_eq!(iter.size_hint(), (1, None));

    _ = iter.next(); // 0
    assert_eq!(iter.size_hint(), (1, None));

    _ = iter.next(); // []
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test_matrix([new_vec_fixed, new_vec_doubling, new_vec_linear])]
fn size_hint_skip_to_end<P>(vec: impl Fn(usize, usize) -> P)
where
    P: IntoConcurrentPinnedVec<String>,
{
    // 1 2 3 0 0 1 0 1 2 0 0 0 1 0
    let queue = ConcurrentQueue::from(vec(3, 20));
    let extend = |s: &String| {
        let i: usize = s.parse().unwrap();
        (0..i).map(|x| x.to_string())
    };
    let iter = DynamicConcurrentIter::new(queue, extend);

    // 1 2 3
    assert_eq!(iter.size_hint(), (3, None));

    _ = iter.next(); // 2 3 0
    assert_eq!(iter.size_hint(), (3, None));

    _ = iter.next(); // 3 0 0 1
    assert_eq!(iter.size_hint(), (4, None));

    _ = iter.next(); // 0 0 1 0 1 2
    assert_eq!(iter.size_hint(), (6, None));

    iter.skip_to_end();
    assert_eq!(iter.size_hint(), (0, Some(0)));

    assert_eq!(iter.next(), None);
}

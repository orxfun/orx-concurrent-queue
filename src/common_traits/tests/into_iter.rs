use crate::ConcurrentQueue;
use std::string::String;

#[test]
fn empty() {
    let queue = ConcurrentQueue::<String>::new();
    let iter = queue.into_iter();
    assert_eq!(iter.count(), 0);
}

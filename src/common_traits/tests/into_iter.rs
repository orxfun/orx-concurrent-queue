use crate::ConcurrentQueue;
use std::string::{String, ToString};

#[test]
fn into_iter_empty() {
    let queue = ConcurrentQueue::<String>::new();
    queue.push("a".to_string());
    let iter = queue.into_iter();
    assert_eq!(iter.count(), 1);
}

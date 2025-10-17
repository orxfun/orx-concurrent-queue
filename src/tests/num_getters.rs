use crate::ConcurrentQueue;
use core::sync::atomic::Ordering;

#[test]
fn num_getters() {
    let queue = ConcurrentQueue::new();
    let order = Ordering::Relaxed;

    let mut wr = 0;
    let mut p = 0;

    // write
    for _ in 0..10 {
        assert_eq!(queue.num_write_reserved(order), wr);
        assert_eq!(queue.num_popped(order), p);

        queue.push('x');
        wr += 1;
    }

    // pop
    for _ in 0..6 {
        assert_eq!(queue.num_write_reserved(order), wr);
        assert_eq!(queue.num_popped(order), p);

        _ = queue.pop();
        p += 1;
    }

    // write
    for _ in 4..20 {
        assert_eq!(queue.num_write_reserved(order), wr);
        assert_eq!(queue.num_popped(order), p);

        queue.push('x');
        wr += 1;
    }

    // pop
    for _ in 0..20 {
        assert_eq!(queue.num_write_reserved(order), wr);
        assert_eq!(queue.num_popped(order), p);

        _ = queue.pop();
        p += 1;
    }

    assert!(queue.is_empty());
    assert_eq!(wr, 26);
    assert_eq!(p, 26);
}

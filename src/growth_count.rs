use std::sync::atomic::{AtomicUsize, Ordering};

const SWITCHING: usize = usize::MAX;

pub struct GrowthCount {
    count: AtomicUsize,
}

impl GrowthCount {
    pub fn get_switching_handle(&self) {
        //
    }

    pub fn switched(&self) {
        debug_assert_eq!(self.count.load(Ordering::Relaxed), SWITCHING);
        self.count.store(0, Ordering::Release);
    }
}

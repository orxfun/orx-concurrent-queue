use std::sync::atomic::AtomicUsize;

const SWITCHING: usize = usize::MAX;

pub struct GrowthCount {
    count: AtomicUsize,
}

impl GrowthCount {
    pub fn get_switching_handle(&self) {
        //
    }
}

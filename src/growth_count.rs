use crate::handle_switch::HandleSwitch;
use std::sync::atomic::{AtomicUsize, Ordering};

const SWITCHING: usize = usize::MAX;

pub struct GrowthCount {
    count: AtomicUsize,
}

impl GrowthCount {
    pub fn get_switch_handle(&self) -> HandleSwitch<'_> {
        loop {
            match self
                .count
                .compare_exchange(0, SWITCHING, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                true => return HandleSwitch::new(self),
                false => {}
            }
        }
    }

    pub fn switched(&self) {
        debug_assert_eq!(self.count.load(Ordering::Relaxed), SWITCHING);
        self.count.store(0, Ordering::Release);
    }

    pub fn grew_once(&self) {
        let before = self.count.fetch_sub(1, Ordering::Release);
        debug_assert!(before > 0);
    }
}

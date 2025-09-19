use crate::{handle_growth::HandleGrowth, handle_switch::HandleSwitch};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const SWITCHING: usize = usize::MAX;

pub struct GrowthCount {
    switch_requested: AtomicBool,
    count: AtomicUsize,
}

impl GrowthCount {
    pub fn get_switch_handle(&self) -> HandleSwitch<'_> {
        let prior = self.switch_requested.fetch_or(true, Ordering::SeqCst);
        debug_assert!(!prior);

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

    pub fn get_growth_handle(&self) -> HandleGrowth<'_> {
        loop {
            if self.switch_requested.load(Ordering::Relaxed) {}
            _ = self.count.fetch_add(1, Ordering::Release);
            return HandleGrowth::new(self);
        }
    }

    pub fn switched(&self) {
        debug_assert_eq!(self.count.load(Ordering::Relaxed), SWITCHING);
        self.count.store(0, Ordering::Release);
    }

    pub fn grew_once(&self) {
        let prior = self.switch_requested.fetch_and(false, Ordering::SeqCst);
        debug_assert!(prior);

        let prior = self.count.fetch_sub(1, Ordering::Release);
        debug_assert!(prior > 0);
    }
}

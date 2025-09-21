use std::cmp::Ordering;

pub enum WritePermit {
    JustWrite,
    GrowThenWrite,
    Spin,
}

impl WritePermit {
    pub fn new(capacity: usize, idx: usize) -> Self {
        match idx.cmp(&capacity) {
            Ordering::Less => Self::JustWrite,
            Ordering::Equal => Self::GrowThenWrite,
            Ordering::Greater => Self::Spin,
        }
    }
}

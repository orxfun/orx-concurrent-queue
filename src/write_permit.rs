use core::cmp::Ordering;

pub enum WritePermit {
    JustWrite,
    GrowThenWrite,
    Spin,
}

impl WritePermit {
    pub fn for_one(capacity: usize, idx: usize) -> Self {
        match idx.cmp(&capacity) {
            Ordering::Less => Self::JustWrite,
            Ordering::Equal => Self::GrowThenWrite,
            Ordering::Greater => Self::Spin,
        }
    }

    pub fn for_many(capacity: usize, begin_idx: usize, last_idx: usize) -> Self {
        match (begin_idx.cmp(&capacity), last_idx.cmp(&capacity)) {
            (_, core::cmp::Ordering::Less) => Self::JustWrite,
            (core::cmp::Ordering::Greater, _) => Self::Spin,
            _ => Self::GrowThenWrite,
        }
    }
}

#![doc = include_str!("../README.md")]
#![warn(
    missing_docs,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::missing_panics_doc,
    clippy::todo
)]
#![no_std]

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests;

mod atomic_utils;
mod queue;
mod write_permit;

pub use queue::ConcurrentQueue;

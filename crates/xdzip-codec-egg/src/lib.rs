//! Independent sequential EGG framing and verified members over host-owned streams.
#![allow(missing_docs)]
mod adapter;
mod content;
mod header;
pub use adapter::EggCodec;
pub use content::Reader;
pub use header::{Index, Record, inspect, read_comment};

#[cfg(test)]
mod tests;

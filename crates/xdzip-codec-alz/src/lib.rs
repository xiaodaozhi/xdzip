//! Independent sequential ALZ framing and verified members over host-owned streams.
#![allow(missing_docs)]
mod adapter;
mod content;
mod header;
pub use adapter::AlzCodec;
pub use content::Reader;
pub use header::{Index, Record, inspect};

#[cfg(test)]
mod tests;

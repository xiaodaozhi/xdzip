//! Independent sequential BlakHole framing and verified members over host-owned streams.
mod adapter;
mod content;
mod header;
pub use adapter::BhCodec;
pub use content::Reader;
pub use header::{Index, Record, inspect};

#[cfg(test)]
mod tests;

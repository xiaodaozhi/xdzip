//! Independent bounded LHA containers over host-owned input capabilities.
mod adapter;
mod content;
mod creation;
mod header;
mod names;
pub use adapter::{LhaCodec, PmaCodec};
pub use content::Reader;
pub use header::{Index, Record, inspect, inspect_pma};
#[cfg(test)]
mod tests;

pub use names::{ResolvedName, resolve_names};

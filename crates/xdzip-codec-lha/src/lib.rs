//! Independent bounded LHA containers over host-owned input capabilities.
mod adapter;
mod content;
mod header;
mod names;
pub use adapter::LhaCodec;
pub use content::Reader;
pub use header::{Index, Record, inspect};
#[cfg(test)]
mod tests;

pub use names::{ResolvedName, resolve_names};

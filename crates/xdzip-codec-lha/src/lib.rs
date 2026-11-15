//! Independent bounded LHA containers over host-owned input capabilities.
mod content;
mod header;
pub use content::Reader;
pub use header::{Index, Record, inspect};
#[cfg(test)]
mod tests;

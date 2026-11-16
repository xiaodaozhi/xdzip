//! Independent sequential ARC framing and verified members over host-owned streams.
mod adapter;
mod content;
mod header;
pub use adapter::ArcCodec;
pub use content::Reader;
pub use header::{Index, Record, inspect};
#[cfg(test)]
#[path = "../../xdzip-compression/src/arc/fixture_cases.rs"]
mod test_cases;
#[cfg(test)]
#[path = "../../../tests/fixtures/arc/known.rs"]
mod test_known;

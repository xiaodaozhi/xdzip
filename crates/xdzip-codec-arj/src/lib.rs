//! Independent ARJ framing and member decoding over host-owned input capabilities.
mod adapter;
mod content;
mod header;
pub use adapter::ArjCodec;
pub use content::Reader;
pub use header::{Index, Record, inspect};
#[cfg(test)]
#[path = "../../xdzip-compression/src/arj/fixture_cases.rs"]
mod test_cases;
#[cfg(test)]
#[path = "../../../tests/fixtures/arj/known.rs"]
mod test_known;

//! Independent RFC 7932 stream reader. Product capability admission is separate.
mod bits;
mod data;
mod dictionary;
mod meta;
mod prefix;
mod reader;
pub use reader::{BrotliReader, brotli_memory_required};
#[cfg(test)]
mod fixture_cases;
#[cfg(test)]
mod tests;

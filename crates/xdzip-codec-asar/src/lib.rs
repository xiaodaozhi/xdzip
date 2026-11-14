//! Bounded ASAR metadata primitives, independent of paths and application execution.
/// Incremental stored content and optional SHA-256 verification before host publication.
pub mod content;
/// Complete Pickle/schema admission and private stored-range descriptions.
pub mod index;
/// Ordered, duplicate-rejecting JSON admission for the ASAR header schema.
pub mod json;

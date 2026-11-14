//! Bounded ASAR metadata primitives, independent of paths and application execution.
/// Complete Pickle/schema admission and private stored-range descriptions.
pub mod index;
/// Ordered, duplicate-rejecting JSON admission for the ASAR header schema.
pub mod json;

//! Framework-independent identities and bounded query contracts.
//!
//! IDs are assigned by the runtime, never derived from paths or view row numbers.
//! This crate performs no I/O and has no dependency on Qt or archive backends.
pub mod identity;
pub mod query;

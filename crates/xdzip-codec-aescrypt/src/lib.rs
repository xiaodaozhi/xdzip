//! Independent AES Crypt framing; admitted general crypto primitives own no file authority.
mod adapter;
pub use adapter::AesCryptCodec;
/// Bounded CBC export with full authentication before host publication.
pub mod content;
mod framing;
/// Complete authentication and exact-size metadata admission.
pub mod index;
mod keys;
mod work;
pub use work::Limits;

//! Format-independent, fixed-storage hashes with erasable secret state.
//!
//! SHA-256 uses the already admitted SHA-2 compression primitive. HMAC follows
//! RFC 2104, and BLAKE2s/sp follow RFC 7693 and the BLAKE2 tree specification.
//! Format-specific key derivation, credentials and authentication policy belong
//! to callers. No filesystem, process or archive authority is exposed here.

mod blake;
mod sha256;

pub use blake::{Blake2s, Blake2sp};
pub use sha256::{HmacSha256, SecretSha256};

#[cfg(test)]
mod tests;

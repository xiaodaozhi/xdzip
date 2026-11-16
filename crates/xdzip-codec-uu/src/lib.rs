//! Bounded UU/XX/Base64 text envelopes, without header pathname or mode authority.
mod adapter;
mod line;
mod text;
pub use adapter::UuCodec;
pub use text::candidate;
#[cfg(test)]
mod tests;

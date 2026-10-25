use zeroize::{Zeroize, Zeroizing};

/// Incremental SHA-256 with erasable chaining words and input buffer.
///
/// Clones contain secret intermediate state and are erased independently.
/// Callers must bound total input below SHA-256's 2^64-bit message limit.
#[derive(Clone)]
pub struct SecretSha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    filled: usize,
    bytes: u64,
}
impl Default for SecretSha256 {
    fn default() -> Self {
        Self::new()
    }
}
impl SecretSha256 {
    /// Starts an empty SHA-256 message.
    pub fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buffer: [0; 64],
            filled: 0,
            bytes: 0,
        }
    }
    /// Appends message bytes without retaining an unbounded input copy.
    pub fn update(&mut self, mut bytes: &[u8]) {
        self.bytes = self.bytes.wrapping_add(bytes.len() as u64);
        while !bytes.is_empty() {
            let count = bytes.len().min(64 - self.filled);
            self.buffer[self.filled..self.filled + count].copy_from_slice(&bytes[..count]);
            self.filled += count;
            bytes = &bytes[count..];
            if self.filled == 64 {
                self.compress();
                self.buffer.zeroize();
                self.filled = 0;
            }
        }
    }
    #[allow(deprecated)] // Admitted SHA-2 0.10 uses generic-array 0.14.
    fn compress(&mut self) {
        let block = sha2::digest::generic_array::GenericArray::from_slice(&self.buffer);
        sha2::compress256(&mut self.state, std::slice::from_ref(block));
    }
    /// Finalizes the digest and erases this message's intermediate state.
    pub fn finish(mut self) -> Zeroizing<[u8; 32]> {
        let bits = self.bytes.wrapping_mul(8);
        self.update(&[0x80]);
        let zeros = (120 - self.filled) % 64;
        self.update(&[0; 64][..zeros]);
        self.update(&bits.to_be_bytes());
        let mut result = Zeroizing::new([0; 32]);
        for (part, value) in result.as_chunks_mut::<4>().0.iter_mut().zip(&self.state) {
            part.copy_from_slice(&value.to_be_bytes());
        }
        result
    }
}
impl Drop for SecretSha256 {
    fn drop(&mut self) {
        self.state.zeroize();
        self.buffer.zeroize();
        self.filled.zeroize();
        self.bytes.zeroize();
    }
}

/// Streaming HMAC-SHA-256, including erasable precomputed pad states.
///
/// An untouched instance can also authenticate many messages with the same key
/// via [`Self::authenticate`], without recompressing either 64-byte pad.
#[derive(Clone)]
pub struct HmacSha256 {
    inner: SecretSha256,
    outer: SecretSha256,
}
impl HmacSha256 {
    /// Normalizes the key as specified in RFC 2104 and starts an empty message.
    pub fn new(key: &[u8]) -> Self {
        let mut block = Zeroizing::new([0u8; 64]);
        if key.len() > 64 {
            let mut hash = SecretSha256::new();
            hash.update(key);
            block[..32].copy_from_slice(hash.finish().as_ref());
        } else {
            block[..key.len()].copy_from_slice(key);
        }
        let mut inner = SecretSha256::new();
        let mut outer = SecretSha256::new();
        for byte in block.iter_mut() {
            *byte ^= 0x36;
        }
        inner.update(block.as_ref());
        for byte in block.iter_mut() {
            *byte ^= 0x36 ^ 0x5c;
        }
        outer.update(block.as_ref());
        Self { inner, outer }
    }
    /// Appends bytes to the current authenticated message.
    pub fn update(&mut self, bytes: &[u8]) {
        self.inner.update(bytes);
    }
    /// Returns the full 32-byte MAC and erases both hash states.
    pub fn finish(mut self) -> Zeroizing<[u8; 32]> {
        self.outer.update(self.inner.finish().as_ref());
        self.outer.finish()
    }
    /// Authenticates a suffix of the current message using an independent clone.
    /// For a newly constructed instance this authenticates exactly `bytes`.
    pub fn authenticate(&self, bytes: &[u8]) -> Zeroizing<[u8; 32]> {
        let mut copy = self.clone();
        copy.update(bytes);
        copy.finish()
    }
}

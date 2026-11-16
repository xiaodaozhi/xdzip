//! Erasable SHA-512 state over the admitted SHA-2 compression primitive.
use zeroize::{Zeroize, Zeroizing};
/// Incremental SHA-512. Each clone owns independently erased secret state.
#[derive(Clone)]
pub struct SecretSha512 {
    state: [u64; 8],
    buffer: [u8; 128],
    filled: usize,
    bytes: u128,
}
impl Default for SecretSha512 {
    fn default() -> Self {
        Self::new()
    }
}
impl SecretSha512 {
    /// Initialize an empty SHA-512 message. Callers bound input below 2^125 bytes.
    pub fn new() -> Self {
        Self {
            state: [
                0x6a09e667f3bcc908,
                0xbb67ae8584caa73b,
                0x3c6ef372fe94f82b,
                0xa54ff53a5f1d36f1,
                0x510e527fade682d1,
                0x9b05688c2b3e6c1f,
                0x1f83d9abfb41bd6b,
                0x5be0cd19137e2179,
            ],
            buffer: [0; 128],
            filled: 0,
            bytes: 0,
        }
    }
    /// Append bytes without retaining an unbounded copy.
    pub fn update(&mut self, mut bytes: &[u8]) {
        self.bytes = self.bytes.wrapping_add(bytes.len() as u128);
        while !bytes.is_empty() {
            let count = bytes.len().min(128 - self.filled);
            self.buffer[self.filled..self.filled + count].copy_from_slice(&bytes[..count]);
            self.filled += count;
            bytes = &bytes[count..];
            if self.filled == 128 {
                self.compress();
                self.buffer.zeroize();
                self.filled = 0;
            }
        }
    }
    #[allow(deprecated)] // Admitted SHA-2 0.10 uses generic-array 0.14.
    fn compress(&mut self) {
        let block = sha2::digest::generic_array::GenericArray::from_slice(&self.buffer);
        sha2::compress512(&mut self.state, std::slice::from_ref(block));
    }
    /// Finalize the digest and erase intermediate chaining and buffered state.
    pub fn finish(mut self) -> Zeroizing<[u8; 64]> {
        let bits = self.bytes.wrapping_mul(8);
        self.update(&[0x80]);
        let zeros = (240 - self.filled) % 128;
        self.update(&[0; 128][..zeros]);
        self.update(&bits.to_be_bytes());
        let mut result = Zeroizing::new([0; 64]);
        for (part, value) in result.as_chunks_mut::<8>().0.iter_mut().zip(&self.state) {
            part.copy_from_slice(&value.to_be_bytes());
        }
        result
    }
}
impl Drop for SecretSha512 {
    fn drop(&mut self) {
        self.state.zeroize();
        self.buffer.zeroize();
        self.filled.zeroize();
        self.bytes.zeroize();
    }
}
/// HMAC-SHA-512 with independently erasable precomputed inner/outer pad states.
#[derive(Clone)]
pub struct HmacSha512 {
    inner: SecretSha512,
    outer: SecretSha512,
}
impl HmacSha512 {
    /// Normalize a key under RFC 2104 and precompute the two 128-byte pads.
    pub fn new(key: &[u8]) -> Self {
        let mut block = Zeroizing::new([0u8; 128]);
        if key.len() > 128 {
            let mut hash = SecretSha512::new();
            hash.update(key);
            block[..64].copy_from_slice(hash.finish().as_ref());
        } else {
            block[..key.len()].copy_from_slice(key);
        }
        let mut inner = SecretSha512::new();
        let mut outer = SecretSha512::new();
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
    /// Append the current message.
    pub fn update(&mut self, bytes: &[u8]) {
        self.inner.update(bytes);
    }
    /// Finish the full 64-byte MAC, erasing both hash states.
    pub fn finish(mut self) -> Zeroizing<[u8; 64]> {
        self.outer.update(self.inner.finish().as_ref());
        self.outer.finish()
    }
    /// Authenticate a suffix using an independently erased clone of the current state.
    pub fn authenticate(&self, bytes: &[u8]) -> Zeroizing<[u8; 64]> {
        let mut copy = self.clone();
        copy.update(bytes);
        copy.finish()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
    #[test]
    fn independent_hashlib_vectors_cross_both_padding_and_block_boundaries() {
        for (count, expected) in [
            (
                111,
                "deb28b32caf354adc13cbbc78e556d5ae22dd9a55b3e01b0ce03f750355da74520e575288a25069afc9ee4b9c660a5a8a9a3368ff3d4173e4080422dbe3f2258",
            ),
            (
                112,
                "1537886092d843d1fdbb9812f1383d9a80926605514c8d732fbee61640538693cbc139061bb2c3f25cb71fd62cda0f8116ea3e0a41ea0a8c0633e630558d1348",
            ),
            (
                127,
                "ea33141b87214fb3898c3a47968582a96a6473ab9f4ba4d2d3de7aa786b197a4adb7a095f31c6ba125f487552bd6f8d7ef23c07288045ff5f9ff67275f110f88",
            ),
            (
                128,
                "18cb20bc77b9193b59f321e9d32b4cd953fc0e5a7192d52f6641fdc5605d95f8b2562a861d0de76c590082cce05ff7db372067201fb8ae1c5d9ff7c8b7919a6c",
            ),
            (
                129,
                "0a090ec394a660a80d9421be36193e7e99961b8805f59fe8781b332e0203b7847a46cbe34837b55949bb9247530e6a992a3f5105525d139b9a3d2e87f5b63194",
            ),
            (
                257,
                "8c3c28b9f5fb1c44f0abdd963d41235d3a4669dff0a8ea5a05f17a226e8f198c844741a085026244ac8848ead33ed03aca496e1fe274473a2e5d500f2e854ba1",
            ),
            (
                1024,
                "c5fd9c79b0458e1f8d21ffcde0c6931ad804e02d645346cd5134d77e1c43d6cf86131047812119d44d1a9b7078ed865c8dd3f9462d6e1950200d9e024296d67d",
            ),
        ] {
            let data: Vec<u8> = (0..count).map(|at| (at * 29 + 7) as u8).collect();
            for quantum in [1, 17, 127, 128, 257] {
                let mut hash = SecretSha512::new();
                for chunk in data.chunks(quantum) {
                    hash.update(chunk);
                }
                assert_eq!(hex(hash.finish().as_ref()), expected, "{count}/{quantum}");
            }
        }
        let mut hash = SecretSha512::new();
        hash.update(b"abc");
        assert_eq!(
            hex(hash.clone().finish().as_ref()),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
        assert_eq!(
            hex(hash.finish().as_ref()),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }
    #[test]
    fn rfc4231_and_independent_long_key_message_mac_vectors() {
        let mac = HmacSha512::new(&[0x0b; 20]);
        assert_eq!(
            hex(mac.authenticate(b"Hi There").as_ref()),
            "87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cdedaa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854"
        );
        let key: Vec<u8> = (0..256).map(|at| at as u8).collect();
        let data: Vec<u8> = (0..700).map(|at| (at * 37 + 9) as u8).collect();
        let mut mac = HmacSha512::new(&key);
        mac.update(&data[..103]);
        let expected = "6875b1c063dfeccaee9aa95439422a6a60947eae6aacb1a2e50ca9e0939bd552e78ac0178eccc70a8d0da8f47687efc151887691d67ddb5e639822af27a4e594";
        assert_eq!(hex(mac.authenticate(&data[103..]).as_ref()), expected);
        mac.update(&data[103..]);
        assert_eq!(hex(mac.finish().as_ref()), expected);
        assert!(std::mem::size_of::<SecretSha512>() < 256);
        assert!(std::mem::size_of::<HmacSha512>() < 512);
    }
}

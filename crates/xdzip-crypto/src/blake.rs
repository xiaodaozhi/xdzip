use zeroize::{Zeroize, Zeroizing};

// Fixed permutations and IV are normative constants, not implementation code.
const IV: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];
const PERMUTATIONS: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

/// Fixed-output, unkeyed BLAKE2s-256 with erasable working storage.
/// Callers must bound input below 2^64 bytes.
pub struct Blake2s {
    state: [u32; 8],
    buffer: [u8; 64],
    filled: usize,
    bytes: u64,
    last_node: bool,
}
impl Default for Blake2s {
    fn default() -> Self {
        Self::new()
    }
}
impl Blake2s {
    /// Starts the standard sequential, unkeyed 32-byte digest.
    pub fn new() -> Self {
        Self::node(1, 1, 0, 0, 0, false)
    }
    fn node(fanout: u8, depth: u8, offset: u8, node_depth: u8, inner: u8, last: bool) -> Self {
        let mut parameters = [0u8; 32];
        parameters[0] = 32;
        parameters[2] = fanout;
        parameters[3] = depth;
        parameters[8] = offset;
        parameters[14] = node_depth;
        parameters[15] = inner;
        let mut state = IV;
        for (word, bytes) in state.iter_mut().zip(parameters.as_chunks::<4>().0) {
            *word ^= u32::from_le_bytes(*bytes);
        }
        Self {
            state,
            buffer: [0; 64],
            filled: 0,
            bytes: 0,
            last_node: last,
        }
    }
    /// Appends bytes, retaining one possible final block for its final flag.
    pub fn update(&mut self, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            if self.filled == 64 {
                self.compress(false);
                self.buffer.zeroize();
                self.filled = 0;
            }
            let count = bytes.len().min(64 - self.filled);
            self.buffer[self.filled..self.filled + count].copy_from_slice(&bytes[..count]);
            self.filled += count;
            self.bytes = self.bytes.wrapping_add(count as u64);
            bytes = &bytes[count..];
        }
    }
    fn compress(&mut self, final_block: bool) {
        let mut message = Zeroizing::new([0u32; 16]);
        for (word, bytes) in message.iter_mut().zip(self.buffer.as_chunks::<4>().0) {
            *word = u32::from_le_bytes(*bytes);
        }
        let mut work = Zeroizing::new([0u32; 16]);
        work[..8].copy_from_slice(&self.state);
        work[8..].copy_from_slice(&IV);
        work[12] ^= self.bytes as u32;
        work[13] ^= (self.bytes >> 32) as u32;
        if final_block {
            work[14] = !work[14];
            if self.last_node {
                work[15] = !work[15];
            }
        }
        const POSITIONS: [[usize; 4]; 8] = [
            [0, 4, 8, 12],
            [1, 5, 9, 13],
            [2, 6, 10, 14],
            [3, 7, 11, 15],
            [0, 5, 10, 15],
            [1, 6, 11, 12],
            [2, 7, 8, 13],
            [3, 4, 9, 14],
        ];
        for order in PERMUTATIONS {
            for (indices, pair) in POSITIONS.iter().zip(order.as_chunks::<2>().0) {
                let [a, b, c, d] = *indices;
                work[a] = work[a].wrapping_add(work[b]).wrapping_add(message[pair[0]]);
                work[d] = (work[d] ^ work[a]).rotate_right(16);
                work[c] = work[c].wrapping_add(work[d]);
                work[b] = (work[b] ^ work[c]).rotate_right(12);
                work[a] = work[a].wrapping_add(work[b]).wrapping_add(message[pair[1]]);
                work[d] = (work[d] ^ work[a]).rotate_right(8);
                work[c] = work[c].wrapping_add(work[d]);
                work[b] = (work[b] ^ work[c]).rotate_right(7);
            }
        }
        for (i, word) in self.state.iter_mut().enumerate() {
            *word ^= work[i] ^ work[i + 8];
        }
    }
    /// Finalizes this node, including an empty node, and erases its state.
    pub fn finish(mut self) -> [u8; 32] {
        self.compress(true);
        let mut result = [0; 32];
        for (part, word) in result.as_chunks_mut::<4>().0.iter_mut().zip(&self.state) {
            part.copy_from_slice(&word.to_le_bytes());
        }
        result
    }
}
impl Drop for Blake2s {
    fn drop(&mut self) {
        self.state.zeroize();
        self.buffer.zeroize();
        self.filled.zeroize();
        self.bytes.zeroize();
    }
}

/// BLAKE2sp-256, distributing successive 64-byte blocks over eight leaves.
///
/// The fixed tree uses fanout eight, depth two and 32-byte inner digests. Update
/// boundaries do not affect leaf routing. No worker threads or heap are used.
pub struct Blake2sp {
    leaves: [Blake2s; 8],
    lane: usize,
    in_block: usize,
}
impl Default for Blake2sp {
    fn default() -> Self {
        Self::new()
    }
}
impl Blake2sp {
    /// Starts an empty eight-leaf tree.
    pub fn new() -> Self {
        Self {
            leaves: std::array::from_fn(|lane| Blake2s::node(8, 2, lane as u8, 0, 32, lane == 7)),
            lane: 0,
            in_block: 0,
        }
    }
    /// Appends bytes to the interleaved leaf streams.
    pub fn update(&mut self, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            let count = bytes.len().min(64 - self.in_block);
            self.leaves[self.lane].update(&bytes[..count]);
            self.in_block += count;
            bytes = &bytes[count..];
            if self.in_block == 64 {
                self.in_block = 0;
                self.lane = (self.lane + 1) % 8;
            }
        }
    }
    /// Finalizes all eight leaves, then the last root node.
    pub fn finish(self) -> [u8; 32] {
        let mut root = Blake2s::node(8, 2, 0, 1, 32, true);
        for leaf in self.leaves {
            let digest = Zeroizing::new(leaf.finish());
            root.update(digest.as_ref());
        }
        root.finish()
    }
}

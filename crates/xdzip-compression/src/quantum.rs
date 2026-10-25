//! CAB Quantum frames, implemented from the separately prepared mathematical
//! state specification. The CAB container owns checksums and fragment assembly.
use crate::invalid;
use std::io;

const FRAME_LIMIT: usize = 32768;
const PACKED_LIMIT: usize = 1024 * 1024;
const RESCALE_LIMIT: u32 = 3800;

/// Bounded Quantum folder state. Models and actual history survive frame ends.
/// A frame error poisons the folder; a new folder requires a new decoder.
pub struct QuantumDecoder {
    selector: Model,
    literals: [Model; 4],
    distances: [Model; 3],
    lengths: Model,
    history: Vec<u8>,
    position: usize,
    available: usize,
    failed: bool,
    finished: bool,
}

impl QuantumDecoder {
    /// Admit the CAB dictionary exponent and compression level before allocating.
    /// Levels affect encoding decisions, rather than decoder probability models.
    pub fn new(window_bits: u8, level: u8) -> io::Result<Self> {
        if !(10..=21).contains(&window_bits) || !(1..=7).contains(&level) {
            return Err(invalid());
        }
        let slots = usize::from(window_bits) * 2;
        Ok(Self {
            selector: Model::new(0, 7),
            literals: std::array::from_fn(|group| Model::new((group * 64) as u8, 64)),
            distances: [
                Model::new(0, slots.min(24)),
                Model::new(0, slots.min(36)),
                Model::new(0, slots.min(42)),
            ],
            lengths: Model::new(5, 27),
            history: vec![0; 1usize << window_bits],
            position: 0,
            available: 0,
            failed: false,
            finished: false,
        })
    }

    /// Decode one complete logical CFDATA frame with its exact declared output.
    /// The 1 MiB packed limit is an implementation resource policy, not a
    /// format maximum. It applies after any physical fragments are assembled.
    /// Arithmetic framing resets here; folder models and history do not reset.
    pub fn decode_frame(&mut self, packed: &[u8], expected: usize) -> io::Result<Vec<u8>> {
        if self.failed || self.finished {
            self.failed = true;
            return Err(invalid());
        }
        let result = self.frame(packed, expected);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    /// Seal the folder after its final complete frame. Each frame has already
    /// validated arithmetic termination and has no deferred match or output.
    pub fn finish(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        self.finished = true;
        Ok(())
    }

    fn frame(&mut self, packed: &[u8], expected: usize) -> io::Result<Vec<u8>> {
        if expected == 0 || expected > FRAME_LIMIT {
            return Err(invalid());
        }
        if packed.len() > PACKED_LIMIT {
            return Err(io::Error::new(io::ErrorKind::OutOfMemory, "resource_limit"));
        }
        let mut arithmetic = Arithmetic::new(packed)?;
        let mut output = Vec::with_capacity(expected);
        while output.len() < expected {
            let selector = arithmetic.symbol(&mut self.selector)?;
            if selector < 4 {
                let byte = arithmetic.symbol(&mut self.literals[usize::from(selector)])?;
                self.emit(byte, &mut output);
                continue;
            }
            let length = match selector {
                4 => 3,
                5 => 4,
                6 => {
                    let symbol = arithmetic.symbol(&mut self.lengths)?;
                    length(symbol, &mut arithmetic.input)?
                }
                _ => return Err(invalid()),
            };
            let slot = arithmetic.symbol(&mut self.distances[usize::from(selector - 4)])?;
            let distance = distance(slot, &mut arithmetic.input)?;
            if length > expected - output.len() || distance > self.available {
                return Err(invalid());
            }
            for _ in 0..length {
                let at = (self.position + self.history.len() - distance) & (self.history.len() - 1);
                self.emit(self.history[at], &mut output);
            }
        }
        arithmetic.input.finish()?;
        Ok(output)
    }

    fn emit(&mut self, byte: u8, output: &mut Vec<u8>) {
        output.push(byte);
        self.history[self.position] = byte;
        self.position = (self.position + 1) & (self.history.len() - 1);
        self.available = (self.available + 1).min(self.history.len());
    }
}

struct Model {
    symbols: Vec<u8>,
    frequencies: Vec<u32>,
    rescalings: u8,
}

impl Model {
    fn new(first: u8, count: usize) -> Self {
        Self {
            symbols: (0..count).map(|offset| first + offset as u8).collect(),
            frequencies: vec![1; count],
            rescalings: 4,
        }
    }

    fn prepare(&mut self) -> u32 {
        let total: u32 = self.frequencies.iter().sum();
        if total <= RESCALE_LIMIT {
            return total;
        }
        self.rescalings -= 1;
        if self.rescalings == 0 {
            self.rescalings = 50;
            for frequency in &mut self.frequencies {
                *frequency = frequency.div_ceil(2);
            }
            // Equal frequencies preserve each comparator's input order. The
            // full comparator sequence determines ties after earlier swaps.
            for left in 0..self.frequencies.len() {
                for right in left + 1..self.frequencies.len() {
                    if self.frequencies[left] < self.frequencies[right] {
                        self.frequencies.swap(left, right);
                        self.symbols.swap(left, right);
                    }
                }
            }
        } else {
            // Halve descending cumulative counts, maintaining one count per
            // symbol. Halving the separate frequencies would change the wire.
            let mut old_cumulative = 0;
            let mut new_cumulative = 0;
            for frequency in self.frequencies.iter_mut().rev() {
                old_cumulative += *frequency;
                let cumulative = (old_cumulative / 2).max(new_cumulative + 1);
                *frequency = cumulative - new_cumulative;
                new_cumulative = cumulative;
            }
        }
        self.frequencies.iter().sum()
    }
}

struct Bits<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Bits<'_> {
    fn read(&mut self, count: u32) -> io::Result<u32> {
        let count = count as usize;
        if count > 21 || count > self.bytes.len() * 8 - self.position {
            return Err(invalid());
        }
        let mut value = 0;
        for _ in 0..count {
            value = (value << 1)
                | u32::from((self.bytes[self.position / 8] >> (7 - self.position % 8)) & 1);
            self.position += 1;
        }
        Ok(value)
    }

    fn finish(&mut self) -> io::Result<()> {
        let mut padding = (8 - self.position % 8) % 8;
        if padding < 2 {
            padding += 8;
        }
        if self.read(padding as u32)? != 0 || self.position != self.bytes.len() * 8 {
            return Err(invalid());
        }
        Ok(())
    }
}

struct Arithmetic<'a> {
    input: Bits<'a>,
    low: u32,
    high: u32,
    value: u32,
}

impl<'a> Arithmetic<'a> {
    fn new(packed: &'a [u8]) -> io::Result<Self> {
        let mut input = Bits {
            bytes: packed,
            position: 0,
        };
        let value = input.read(16)?;
        Ok(Self {
            input,
            low: 0,
            high: 65535,
            value,
        })
    }

    fn symbol(&mut self, model: &mut Model) -> io::Result<u8> {
        let total = model.prepare();
        let width = self.high - self.low + 1;
        let scaled = ((self.value - self.low + 1) * total - 1) / width;
        let mut upper = total;
        for index in 0..model.frequencies.len() {
            let lower = upper - model.frequencies[index];
            if scaled >= lower {
                let origin = self.low;
                self.low = origin + width * lower / total;
                self.high = origin + width * upper / total - 1;
                self.normalize()?;
                model.frequencies[index] += 8;
                return Ok(model.symbols[index]);
            }
            upper = lower;
        }
        Err(invalid())
    }

    fn normalize(&mut self) -> io::Result<()> {
        loop {
            let translation = if self.high < 32768 {
                0
            } else if self.low >= 32768 {
                32768
            } else if self.low >= 16384 && self.high < 49152 {
                16384
            } else {
                break;
            };
            self.low = (self.low - translation) * 2;
            self.high = (self.high - translation) * 2 + 1;
            self.value = (self.value - translation) * 2 + self.input.read(1)?;
            if !(self.low..=self.high).contains(&self.value) {
                return Err(invalid());
            }
        }
        Ok(())
    }
}

fn length(symbol: u8, input: &mut Bits<'_>) -> io::Result<usize> {
    if symbol <= 10 {
        return Ok(usize::from(symbol));
    }
    let shifted = u32::from(symbol - 7);
    let extra = shifted / 4;
    let base = ((4 + shifted % 4) << extra) + 3;
    Ok((base + if extra == 6 { 0 } else { input.read(extra)? }) as usize)
}

fn distance(slot: u8, input: &mut Bits<'_>) -> io::Result<usize> {
    if slot <= 3 {
        return Ok(usize::from(slot) + 1);
    }
    let slot = u32::from(slot);
    let extra = slot / 2 - 1;
    Ok((((2 + slot % 2) << extra) + input.read(extra)? + 1) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_frame() -> &'static [u8] {
        // Fixed LGPL-2.1-or-later test input, not decoder source. Provenance and
        // complete CAB hash are retained in tests/fixtures/cab-iso/sources.json.
        let cab = include_bytes!("../../../tests/fixtures/cab-iso/mszip_lzx_qtm.cab");
        &cab[331..379]
    }

    const FIXED_PLAIN: &[u8] = b"If you can read this, the Quantum decompressor is working!\n";

    #[test]
    fn independent_fixed_cab_frame_and_all_levels() {
        for level in 1..=7 {
            let mut decoder = QuantumDecoder::new(18, level).unwrap();
            assert_eq!(
                decoder.decode_frame(fixed_frame(), 59).unwrap(),
                FIXED_PLAIN
            );
        }
    }

    #[test]
    fn declared_size_sixty_matches_independent_official_output() {
        // With no end token, these bytes also terminate at 60 declared bytes.
        let mut longer = FIXED_PLAIN.to_vec();
        longer.push(b'm');
        assert_eq!(
            QuantumDecoder::new(18, 2)
                .unwrap()
                .decode_frame(fixed_frame(), 60)
                .unwrap(),
            longer
        );
    }

    #[test]
    fn independently_read_two_frame_vector_retains_models_and_wraps_history() {
        // Self-authored mathematical encoder, both complete frames independently
        // read by official 7-Zip 26.05. The second uses the maximum 1 KiB distance.
        // Reproducible recipe and identities: fixtures/cab/quantum-sources.json.
        fn hex(text: &str) -> Vec<u8> {
            (0..text.len())
                .step_by(2)
                .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
                .collect()
        }
        let first = hex("d1241826567d590f3bac000100");
        let second = hex(concat!(
            "0000003fc07f81fe0ff07f83fc3fc1fe1fe1fe1fe3fc3fc3fc7f8ff1fe1fe3fc",
            "7f8ff1fe7f8ff1fe3fcff1fe7f8ff3fc7f9fe7f8ff3fcff1fe7f9fe7f9fe7f9f",
            "e7f9fe7f9fe7f9fe7f9fe7fbfcff3fcff3fdfe7f9feff3fcff7f9fe7fbfcff7f",
            "9feff3fcff7fbfcff7f9feff3fdfe7fbfdfe7fbfdfe7fbfdfe7fbfdfe7fbfdfe",
            "ff3fdfeff7f9feff7fbfdfe7fbfdfeff7ffffff7fbfdffff7fbfffeff7f9feff",
            "7f801bfc"
        ));
        let mut decoder = QuantumDecoder::new(10, 2).unwrap();
        assert_eq!(
            decoder.decode_frame(&first, 32768).unwrap(),
            vec![b'Q'; 32768]
        );
        assert_eq!(
            decoder.decode_frame(&second, 32768).unwrap(),
            vec![b'Q'; 32768]
        );
        decoder.finish().unwrap();
        assert!(
            QuantumDecoder::new(10, 2)
                .unwrap()
                .decode_frame(&second, 32768)
                .is_err()
        );
        assert!(
            QuantumDecoder::new(10, 2)
                .unwrap()
                .decode_frame(&first, 32767)
                .is_err()
        );
    }

    #[test]
    fn invalid_parameters_and_frame_errors_poison() {
        for bits in [0, 9, 22, 255] {
            assert!(QuantumDecoder::new(bits, 2).is_err());
        }
        for level in [0, 8, 255] {
            assert!(QuantumDecoder::new(18, level).is_err());
        }
        for expected in [0, 58, 61, FRAME_LIMIT + 1] {
            let mut decoder = QuantumDecoder::new(18, 2).unwrap();
            assert!(
                decoder.decode_frame(fixed_frame(), expected).is_err(),
                "accepted output length {expected}"
            );
            assert!(decoder.decode_frame(fixed_frame(), 59).is_err());
        }
        for cut in 0..fixed_frame().len() {
            assert!(
                QuantumDecoder::new(18, 2)
                    .unwrap()
                    .decode_frame(&fixed_frame()[..cut], 59)
                    .is_err()
            );
        }
        for suffix in [0, 255] {
            let mut bytes = fixed_frame().to_vec();
            bytes.push(suffix);
            assert!(
                QuantumDecoder::new(18, 2)
                    .unwrap()
                    .decode_frame(&bytes, 59)
                    .is_err()
            );
        }
        let mut bad_tail = fixed_frame().to_vec();
        *bad_tail.last_mut().unwrap() |= 1;
        assert!(
            QuantumDecoder::new(18, 2)
                .unwrap()
                .decode_frame(&bad_tail, 59)
                .is_err()
        );
        // Long suffixes remain invalid entropy frames, including lengths beyond
        // a physical CFDATA u16 and exactly at the logical resource ceiling.
        // They are deliberately not presented as legal large Quantum samples.
        for count in [65536, PACKED_LIMIT] {
            let mut bytes = fixed_frame().to_vec();
            bytes.resize(count, 0);
            let mut decoder = QuantumDecoder::new(18, 2).unwrap();
            assert_eq!(
                decoder.decode_frame(&bytes, 59).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
            assert!(decoder.finish().is_err());
        }
        let mut decoder = QuantumDecoder::new(18, 2).unwrap();
        assert_eq!(
            decoder
                .decode_frame(&vec![0; PACKED_LIMIT + 1], 59)
                .unwrap_err()
                .kind(),
            io::ErrorKind::OutOfMemory
        );
        assert_eq!(decoder.available, 0);
        assert!(decoder.decode_frame(fixed_frame(), 59).is_err());
    }

    #[test]
    fn allocated_zero_window_is_not_valid_history() {
        // Self-authored arithmetic frame: selector4, distance slot0. Its first
        // token asks for three bytes at distance1 before any byte was produced.
        let mut decoder = QuantumDecoder::new(10, 2).unwrap();
        assert!(decoder.decode_frame(&[0x6c, 0, 0], 3).is_err());
        assert_eq!(decoder.available, 0);
        assert!(decoder.decode_frame(fixed_frame(), 59).is_err());
        assert!(decoder.finish().is_err());
    }

    #[test]
    fn descending_cdf_and_middle_half_normalization() {
        for (value, expected) in [(0, 6), (65535, 0)] {
            let bytes = [(value >> 8) as u8, value as u8, 0, 0];
            let mut arithmetic = Arithmetic::new(&bytes).unwrap();
            assert_eq!(arithmetic.symbol(&mut Model::new(0, 7)).unwrap(), expected);
        }
        let mut arithmetic = Arithmetic {
            input: Bits {
                bytes: &[0x80],
                position: 0,
            },
            low: 20000,
            high: 45000,
            value: 30000,
        };
        arithmetic.normalize().unwrap();
        assert_eq!(
            (arithmetic.low, arithmetic.high, arithmetic.value),
            (7232, 57233, 27233)
        );
        assert_eq!(arithmetic.input.position, 1);
    }

    #[test]
    fn cumulative_rescale_threshold_and_deterministic_reordering() {
        let mut model = Model::new(0, 3);
        model.frequencies = vec![1266, 1266, 1268];
        assert_eq!(model.prepare(), 3800);
        assert_eq!(model.rescalings, 4);
        model.frequencies = vec![1, 2, 3805];
        assert_eq!(model.prepare(), 1904);
        assert_eq!(model.frequencies, [1, 1, 1902]);
        assert_eq!(model.rescalings, 3);

        let mut model = Model::new(b'A', 5);
        model.frequencies = vec![600, 600, 1200, 1800, 600];
        model.rescalings = 1;
        assert_eq!(model.prepare(), 2400);
        assert_eq!(model.symbols, b"DCABE");
        assert_eq!(model.frequencies, [900, 600, 300, 300, 300]);
        assert_eq!(model.rescalings, 50);
    }

    #[test]
    fn length_distance_boundaries_and_exact_zero_padding() {
        let mut bits = Bits {
            bytes: &[255, 255, 255],
            position: 0,
        };
        assert_eq!(length(31, &mut bits).unwrap(), 259);
        assert_eq!(bits.position, 0);
        assert_eq!(length(30, &mut bits).unwrap(), 258);
        assert_eq!(distance(41, &mut bits).unwrap(), 2097152);
        for position in 0..8 {
            let count = if position == 7 { 2 } else { 1 };
            let bytes = [0, 0];
            let mut bits = Bits {
                bytes: &bytes[..count],
                position,
            };
            bits.finish().unwrap();
        }
        assert!(
            Bits {
                bytes: &[1],
                position: 6
            }
            .finish()
            .is_err()
        );
        assert!(
            Bits {
                bytes: &[0, 0],
                position: 6
            }
            .finish()
            .is_err()
        );
        assert!(
            Bits {
                bytes: &[0],
                position: 7
            }
            .finish()
            .is_err()
        );
    }

    #[test]
    fn finish_seals_successful_folder_and_rejects_failed_folder() {
        let mut decoder = QuantumDecoder::new(18, 2).unwrap();
        decoder.decode_frame(fixed_frame(), 59).unwrap();
        decoder.finish().unwrap();
        decoder.finish().unwrap();
        assert!(decoder.decode_frame(fixed_frame(), 59).is_err());
        assert!(decoder.finish().is_err());
        let mut decoder = QuantumDecoder::new(18, 2).unwrap();
        assert!(decoder.decode_frame(&[], 1).is_err());
        assert!(decoder.finish().is_err());
    }
}

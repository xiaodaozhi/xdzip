//! Unix compress wire profile: 9–16-bit LSB codes in eight-code groups.
//! CLEAR and width changes end a group; its unused alignment bits are unspecified.
use crate::{
    invalid,
    stream_io::{Input, READ_QUANTUM, filled, resource},
};
use std::io::{self, Read};
const WORK_MAX: u64 = 96 * 1024 * 1024 * 1024;
/// Fixed dictionary, reverse expansion and input-buffer admission for a declared width.
pub fn compress_memory_required(bits: u8) -> Option<u64> {
    (9..=16)
        .contains(&bits)
        .then(|| (4u64 << bits) + READ_QUANTUM as u64 + 512)
}
struct Codes<R> {
    input: Input<R>,
    group: [u8; 18],
    width: u8,
    position: usize,
    bits: usize,
    final_group: bool,
}
impl<R: Read> Codes<R> {
    fn align(&mut self, width: u8) {
        self.width = width;
        self.position = 0;
        self.bits = 0;
        self.final_group = false;
    }
    fn next(&mut self) -> io::Result<Option<u16>> {
        let width = usize::from(self.width);
        if self.position + width > self.bits {
            if self.final_group {
                if self.bits - self.position >= 8 {
                    return Err(invalid());
                }
                return Ok(None);
            }
            let count = (self.input.packed - self.input.consumed).min(width as u64) as usize;
            if count == 0 {
                return Ok(None);
            }
            self.group.fill(0);
            for byte in &mut self.group[..count] {
                *byte = self.input.byte(self.input.packed)?;
            }
            self.bits = count * 8;
            self.position = 0;
            self.final_group = count < width;
            if self.bits < width {
                return Err(invalid());
            }
        }
        let at = self.position / 8;
        let word = u32::from(self.group[at])
            | (u32::from(self.group[at + 1]) << 8)
            | (u32::from(self.group[at + 2]) << 16);
        let code = ((word >> (self.position % 8)) & ((1u32 << width) - 1)) as u16;
        self.position += width;
        Ok(Some(code))
    }
}
/// Independent bounded LZW reader. Framing has no checksum or decoded-size field;
/// the caller owns byte quotas, source authority, cancellation and publication.
pub struct CompressReader<R> {
    codes: Codes<R>,
    prefixes: Vec<u16>,
    suffixes: Vec<u8>,
    stack: Vec<u8>,
    next: usize,
    maximum: u8,
    block: bool,
    previous: Option<u16>,
    first: u8,
    after_clear: bool,
    work: u64,
    ended: bool,
    failed: bool,
    #[cfg(test)]
    resets: usize,
    #[cfg(test)]
    widths: u32,
}
impl<R: Read> CompressReader<R> {
    /// Validate the header and admit all storage before allocating the dictionary.
    /// Work counts codes, prefix expansion, dictionary insertions and output bytes.
    pub fn new(source: R, packed: u64, memory_limit: u64, work_limit: u64) -> io::Result<Self> {
        let mut input = Input::new(source, packed);
        if input.byte(packed)? != 0x1f || input.byte(packed)? != 0x9d {
            return Err(invalid());
        }
        let flags = input.byte(packed)?;
        let maximum = flags & 31;
        let memory = compress_memory_required(maximum).ok_or_else(|| {
            io::Error::new(io::ErrorKind::Unsupported, "unsupported_compress_width")
        })?;
        if flags & 0x60 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "unsupported_compress_flags",
            ));
        }
        if memory_limit < memory {
            return Err(resource());
        }
        let capacity = 1usize << maximum;
        let prefixes = filled(capacity, 0)?;
        let suffixes = filled(capacity, 0)?;
        let mut stack = Vec::new();
        stack
            .try_reserve_exact(capacity + 1)
            .map_err(|_| resource())?;
        let block = flags & 0x80 != 0;
        Ok(Self {
            codes: Codes {
                input,
                group: [0; 18],
                width: 9,
                position: 0,
                bits: 0,
                final_group: false,
            },
            prefixes,
            suffixes,
            stack,
            next: if block { 257 } else { 256 },
            maximum,
            block,
            previous: None,
            first: 0,
            after_clear: false,
            work: work_limit.min(WORK_MAX),
            ended: false,
            failed: false,
            #[cfg(test)]
            resets: 0,
            #[cfg(test)]
            widths: 1 << 9,
        })
    }
    fn charge(&mut self, work: u64) -> io::Result<()> {
        self.work = self.work.checked_sub(work).ok_or_else(resource)?;
        Ok(())
    }
    fn expand(&mut self) -> io::Result<()> {
        loop {
            if self.next == 1usize << self.codes.width && self.codes.width < self.maximum {
                self.codes.align(self.codes.width + 1);
                #[cfg(test)]
                {
                    self.widths |= 1 << self.codes.width;
                }
            }
            let Some(code) = self.codes.next()? else {
                if self.after_clear {
                    return Err(invalid());
                }
                self.ended = true;
                return Ok(());
            };
            self.charge(1)?;
            if self.block && code == 256 {
                if self.previous.is_none() {
                    return Err(invalid());
                }
                self.codes.align(9);
                self.next = 257;
                self.previous = None;
                self.after_clear = true;
                #[cfg(test)]
                {
                    self.resets += 1;
                }
                continue;
            }
            if self.previous.is_none() && code >= 256 {
                return Err(invalid());
            }
            let mut cursor = usize::from(code);
            if cursor == self.next && self.previous.is_some() && self.next < self.prefixes.len() {
                self.charge(1)?;
                self.stack.push(self.first);
                cursor = usize::from(self.previous.expect("checked previous code"));
            } else if cursor >= self.next {
                return Err(invalid());
            }
            while cursor >= 256 {
                self.charge(1)?;
                if cursor >= self.next || self.stack.len() >= self.prefixes.len() {
                    return Err(invalid());
                }
                self.stack.push(self.suffixes[cursor]);
                let parent = usize::from(self.prefixes[cursor]);
                if parent >= cursor {
                    return Err(invalid());
                }
                cursor = parent;
            }
            self.charge(1)?;
            let first = cursor as u8;
            self.stack.push(first);
            if let Some(previous) = self.previous
                && self.next < self.prefixes.len()
            {
                self.charge(1)?;
                self.prefixes[self.next] = previous;
                self.suffixes[self.next] = first;
                self.next += 1;
            }
            self.previous = Some(code);
            self.first = first;
            self.after_clear = false;
            return Ok(());
        }
    }
    fn read_inner(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let mut written = 0;
        let limit = bytes.len().min(READ_QUANTUM);
        while written != limit {
            if self.stack.is_empty() {
                if self.ended {
                    break;
                }
                self.expand()?;
                if self.ended {
                    break;
                }
            }
            let count = (limit - written).min(self.stack.len());
            self.charge(count as u64)?;
            let end = self.stack.len();
            for (at, byte) in bytes[written..written + count].iter_mut().enumerate() {
                *byte = self.stack[end - 1 - at];
            }
            self.stack.truncate(end - count);
            written += count;
        }
        Ok(written)
    }
}
impl<R: Read> Read for CompressReader<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        let result = self.read_inner(bytes);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/compress")
                .join(name),
        )
        .unwrap()
    }
    fn pattern(count: usize) -> Vec<u8> {
        let mut state = 0x6d2b79f5u32;
        (0..count)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                (state >> 24) as u8
            })
            .collect()
    }
    type Decoded<'a> = (Vec<u8>, CompressReader<Cursor<&'a [u8]>>);
    fn decode(bytes: &[u8]) -> io::Result<Decoded<'_>> {
        let mut reader =
            CompressReader::new(Cursor::new(bytes), bytes.len() as u64, 1 << 20, WORK_MAX)?;
        let mut output = Vec::new();
        reader.read_to_end(&mut output)?;
        Ok((output, reader))
    }
    #[test]
    fn independent_compressed_inputs_cover_every_width_full_dictionary_and_kwkwk() {
        for bits in 9..=16 {
            let bytes = fixture(&format!("width-{bits}.Z"));
            let (actual, reader) = decode(&bytes).unwrap();
            assert_eq!(actual, pattern((2 << bits) + 2048), "{bits}");
            assert_eq!(reader.codes.width, bits);
            assert_eq!(reader.widths, ((1u32 << (bits + 1)) - 1) & !511);
            assert_eq!(reader.next, 1usize << bits);
        }
        for (name, expected) in [
            ("nonblock.Z", b"ABABABA".to_vec()),
            ("clear.Z", b"ABABCDCD".to_vec()),
            ("nonblock-width.Z", pattern(5000)),
        ] {
            assert_eq!(decode(&fixture(name)).unwrap().0, expected, "{name}");
        }
    }
    #[test]
    fn actual_author_streams_include_natural_resets_empty_zeros_and_real_matches() {
        let phrase = b"abracadabra abracadabra\n".repeat(6);
        for (name, expected) in [
            ("phrase.Z", phrase.clone()),
            ("random.Z", pattern(90000)),
            ("empty.Z", vec![]),
            ("zeros.Z", vec![0; 4096]),
            ("empty-tar.Z", vec![0; 1024]),
        ] {
            assert_eq!(decode(&fixture(name)).unwrap().0, expected, "{name}");
        }
        let mut expected = phrase.repeat(4000);
        expected.extend(pattern(250000));
        let bytes = fixture("natural-clear.Z");
        let (actual, reader) = decode(&bytes).unwrap();
        assert_eq!(actual, expected);
        assert!(reader.resets > 0);
        let tar = decode(&fixture("files.taz")).unwrap().0;
        assert_eq!(tar.len(), 10240);
        assert_eq!(&tar[..9], b"hello.txt");
        assert_eq!(&tar[512..512 + phrase.len()], phrase);
        assert!(tar[1024..].iter().all(|b| *b == 0));
    }
    #[test]
    fn memory_and_actual_work_admission_are_exact_and_failure_is_terminal() {
        let bytes = [0x1f, 0x9d, 0x90, 0x41, 0];
        let memory = compress_memory_required(16).unwrap();
        assert_eq!(
            CompressReader::new(Cursor::new(bytes), 5, memory - 1, 3)
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::OutOfMemory
        );
        let mut reader = CompressReader::new(Cursor::new(bytes), 5, memory, 3).unwrap();
        let mut actual = vec![];
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, b"A");
        assert_eq!(reader.work, 0);
        let mut reader = CompressReader::new(Cursor::new(bytes), 5, memory, 2).unwrap();
        assert_eq!(
            reader.read(&mut [0; 8]).unwrap_err().kind(),
            io::ErrorKind::OutOfMemory
        );
        assert!(reader.read(&mut [0; 8]).is_err());
        for bits in [0, 8, 17, 31, 255] {
            assert_eq!(compress_memory_required(bits), None);
        }
    }
    #[test]
    fn malformed_codes_incomplete_groups_and_terminal_clear_are_rejected() {
        for bytes in [
            b"\x1f\x9d\x90\x41".as_slice(),
            b"\x1f\x9d\x90\xff\x01",
            b"\x1f\x9d\x90\x00\x01",
            b"\x1f\x9d\x90\x41\x00\xff",
        ] {
            assert!(decode(bytes).is_err(), "{bytes:?}");
        }
        let clear = fixture("clear.Z");
        assert!(decode(&clear[..12]).is_err());
        for flags in [0x88, 0x91, 0xb0, 0xd0] {
            assert_eq!(
                decode(&[0x1f, 0x9d, flags]).err().unwrap().kind(),
                io::ErrorKind::Unsupported
            );
        }
        let mut bytes = vec![0x1f, 0x9d, 0x90, 0x41, 0xfe];
        assert_eq!(decode(&bytes).unwrap().0, b"A"); // Unused final bits are unspecified.
        bytes = clear.clone();
        bytes[9] = 0xff;
        bytes[10] = 0x55;
        bytes[11] = 0xaa;
        assert_eq!(decode(&bytes).unwrap().0, b"ABABCDCD"); // Unused CLEAR alignment is unspecified.
    }
    #[test]
    fn short_reads_bare_interruptions_and_typed_terminal_errors_keep_their_boundaries() {
        #[derive(Debug)]
        struct Marker;
        impl std::fmt::Display for Marker {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("terminal")
            }
        }
        impl std::error::Error for Marker {}
        struct Source {
            bytes: Cursor<Vec<u8>>,
            calls: usize,
            typed: bool,
        }
        impl Read for Source {
            fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
                self.calls += 1;
                if self.typed {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, Marker));
                }
                if self.calls == 1 {
                    return Err(io::ErrorKind::Interrupted.into());
                }
                self.bytes.read(&mut bytes[..1])
            }
        }
        let data = fixture("phrase.Z");
        let packed = data.len() as u64;
        let source = Source {
            bytes: Cursor::new(data),
            calls: 0,
            typed: false,
        };
        let mut reader = CompressReader::new(source, packed, 1 << 20, WORK_MAX).unwrap();
        let mut out = vec![];
        reader.read_to_end(&mut out).unwrap();
        assert_eq!(out, b"abracadabra abracadabra\n".repeat(6));
        let source = Source {
            bytes: Cursor::new(vec![]),
            calls: 0,
            typed: true,
        };
        let error = CompressReader::new(source, 5, 1 << 20, WORK_MAX)
            .err()
            .unwrap();
        assert!(error.get_ref().unwrap().is::<Marker>());
        assert_ne!(error.kind(), io::ErrorKind::Interrupted);
    }
}

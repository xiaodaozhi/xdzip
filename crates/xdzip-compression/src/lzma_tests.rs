use crate::{
    Lzma2Reader, Lzma2Writer, LzmaReader, lzma_memory_required, lzma2_memory_required,
    lzma2_writer_memory_required,
};
use std::io::{self, Read, Write};

// Test-only Python 3 stdlib/liblzma compressed our own phrase. These bytes are
// data, not implementation. The raw LZMA1 stream includes EOS; LZMA2 does not.
const PROPS: [u8; 5] = [0x5d, 0, 0x10, 0, 0];
const RAW1: &[u8] = &[
    0, 48, 152, 138, 170, 154, 89, 245, 17, 216, 62, 177, 21, 90, 80, 62, 73, 129, 72, 143, 255,
    255, 157, 142, 0, 0,
];
const RAW2: &[u8] = &[
    224, 0, 143, 0, 19, 93, 0, 48, 152, 138, 170, 154, 89, 245, 17, 216, 62, 177, 21, 90, 80, 62,
    72, 175, 233, 128, 0,
];
fn expected() -> Vec<u8> {
    b"abracadabra abracadabra\n".repeat(6)
}
struct Singles<'a>(&'a [u8]);
impl Read for Singles<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() || self.0.is_empty() {
            return Ok(0);
        }
        output[0] = self.0[0];
        self.0 = &self.0[1..];
        Ok(1)
    }
}

#[test]
fn lzma_python_eos_known_unknown_and_single_byte_io() {
    for size in [Some(144), None] {
        let mut reader =
            LzmaReader::new(Singles(RAW1), RAW1.len() as u64, size, &PROPS, 1 << 20).unwrap();
        let mut output = Vec::new();
        let mut byte = [0];
        while reader.read(&mut byte).unwrap() != 0 {
            output.push(byte[0]);
        }
        assert_eq!(output, expected());
        assert_eq!(reader.consumed_bytes(), RAW1.len() as u64);
        assert_eq!(reader.decoded_bytes(), 144);
        reader.finish().unwrap();
        reader.finish().unwrap();
    }
    let mut reader = LzmaReader::new(RAW1, RAW1.len() as u64, Some(144), &PROPS, 1 << 20).unwrap();
    let mut output = [0; 144];
    reader.read_exact(&mut output).unwrap();
    reader.finish().unwrap();
    assert_eq!(output.as_slice(), expected());
}

#[test]
fn lzma_empty_no_eos_and_precise_span() {
    let zeros = [0; 5];
    let mut reader = LzmaReader::new(zeros.as_slice(), 5, Some(0), &PROPS, 1 << 20).unwrap();
    reader.finish().unwrap();
    assert_eq!(reader.consumed_bytes(), 5);
    for bytes in [&[0; 4][..], &[0; 6][..], &[1, 0, 0, 0, 0][..]] {
        let mut reader =
            LzmaReader::new(bytes, bytes.len() as u64, Some(0), &PROPS, 1 << 20).unwrap();
        assert!(reader.finish().is_err());
        assert!(reader.read(&mut [0]).is_err());
    }
    let mut source = RAW1.to_vec();
    source.extend_from_slice(b"outside admitted span");
    let mut reader = LzmaReader::new(
        source.as_slice(),
        RAW1.len() as u64,
        Some(144),
        &PROPS,
        1 << 20,
    )
    .unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, expected());
}

#[test]
fn lzma_truncation_size_tail_and_poison() {
    for size in [0, 143, 145] {
        let mut reader =
            LzmaReader::new(RAW1, RAW1.len() as u64, Some(size), &PROPS, 1 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
        assert!(reader.finish().is_err());
    }
    for cut in 0..RAW1.len() {
        let mut reader =
            LzmaReader::new(&RAW1[..cut], cut as u64, Some(144), &PROPS, 1 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err(), "cut={cut}");
        assert!(reader.read(&mut [0]).is_err());
    }
    for suffix in [&[0][..], RAW1] {
        let mut bytes = RAW1.to_vec();
        bytes.extend_from_slice(suffix);
        let mut reader = LzmaReader::new(
            bytes.as_slice(),
            bytes.len() as u64,
            Some(144),
            &PROPS,
            1 << 20,
        )
        .unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
    }
    let mut bad = RAW1.to_vec();
    *bad.last_mut().unwrap() ^= 1;
    let mut reader =
        LzmaReader::new(bad.as_slice(), bad.len() as u64, Some(144), &PROPS, 1 << 20).unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
    let mut early = LzmaReader::new(RAW1, RAW1.len() as u64, Some(144), &PROPS, 1 << 20).unwrap();
    assert!(early.finish().is_err());
    assert!(early.read(&mut [0]).is_err());
}

struct FaultingSource<'a> {
    bytes: &'a [u8],
    until_error: usize,
    interrupt: bool,
}
impl Read for FaultingSource<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.interrupt {
            self.interrupt = false;
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        if self.until_error == 0 {
            return Err(io::Error::other("injected_input_error"));
        }
        let count = self.bytes.len().min(output.len()).min(self.until_error);
        output[..count].copy_from_slice(&self.bytes[..count]);
        self.bytes = &self.bytes[count..];
        self.until_error -= count;
        Ok(count)
    }
}

#[test]
fn lzma_input_interruption_retry_and_io_error_poison() {
    let source = FaultingSource {
        bytes: RAW1,
        until_error: usize::MAX,
        interrupt: true,
    };
    let mut reader =
        LzmaReader::new(source, RAW1.len() as u64, Some(144), &PROPS, 1 << 20).unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, expected());
    let source = FaultingSource {
        bytes: RAW1,
        until_error: 10,
        interrupt: true,
    };
    let mut reader =
        LzmaReader::new(source, RAW1.len() as u64, Some(144), &PROPS, 1 << 20).unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
    assert!(reader.read(&mut [0]).is_err());
    assert!(reader.finish().is_err());
    let source = FaultingSource {
        bytes: RAW2,
        until_error: 12,
        interrupt: false,
    };
    let mut reader = Lzma2Reader::new(source, RAW2.len() as u64, 144, 0, 1 << 20).unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
    assert!(reader.read(&mut [0]).is_err());
    assert!(reader.finish().is_err());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoundarySignal {
    SourceChanged,
    Cancelled,
    Limit,
}
impl std::fmt::Display for BoundarySignal {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(output, "{self:?}")
    }
}
impl std::error::Error for BoundarySignal {}
fn signal_error(signal: Option<BoundarySignal>) -> io::Error {
    match signal {
        Some(signal) => io::Error::new(io::ErrorKind::Interrupted, signal),
        None => io::Error::from(io::ErrorKind::Interrupted),
    }
}
fn assert_signal(error: io::Error, signal: BoundarySignal) {
    assert_eq!(
        error.kind(),
        io::ErrorKind::Other,
        "std helpers must not retry this error"
    );
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<BoundarySignal>(),
        Some(&signal)
    );
}

struct SignallingSource<'a> {
    bytes: &'a [u8],
    before_error: usize,
    signal: BoundarySignal,
    raised: bool,
    calls: std::rc::Rc<std::cell::Cell<usize>>,
}
impl Read for SignallingSource<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.calls.set(self.calls.get() + 1);
        if self.before_error == 0 && !self.raised {
            self.raised = true;
            return Err(signal_error(Some(self.signal)));
        }
        let limit = if self.raised {
            usize::MAX
        } else {
            self.before_error
        };
        let count = output.len().min(self.bytes.len()).min(limit);
        output[..count].copy_from_slice(&self.bytes[..count]);
        self.bytes = &self.bytes[count..];
        self.before_error = self.before_error.saturating_sub(count);
        Ok(count)
    }
}
fn check_signalling_reader(
    reader: &mut impl Read,
    calls: &std::cell::Cell<usize>,
    signal: BoundarySignal,
    partial: bool,
    method: u8,
) {
    if partial {
        let mut first = [0];
        assert_eq!(reader.read(&mut first).unwrap(), 1);
        assert_eq!(first[0], b'a');
    }
    let error = match method {
        0 => loop {
            match reader.read(&mut [0; 16]) {
                Ok(count) => assert!(count != 0, "typed input failure must not become EOF"),
                Err(error) => break error,
            }
        },
        1 => reader.read_to_end(&mut Vec::new()).unwrap_err(),
        _ => reader.read_exact(&mut [0; 144]).unwrap_err(),
    };
    assert_signal(error, signal);
    let before = calls.get();
    assert_eq!(
        reader.read(&mut [0]).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        calls.get(),
        before,
        "poisoned reader must not touch the source again"
    );
}

#[test]
fn lzma_typed_interrupted_source_errors_survive_read_helpers() {
    for signal in [
        BoundarySignal::SourceChanged,
        BoundarySignal::Cancelled,
        BoundarySignal::Limit,
    ] {
        for partial in [false, true] {
            for method in 0..3 {
                for lzma2 in [false, true] {
                    let bytes = if lzma2 { RAW2 } else { RAW1 };
                    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
                    let source = SignallingSource {
                        bytes,
                        before_error: if partial {
                            if lzma2 { 17 } else { 10 }
                        } else {
                            0
                        },
                        signal,
                        raised: false,
                        calls: calls.clone(),
                    };
                    if lzma2 {
                        let mut reader =
                            Lzma2Reader::new(source, bytes.len() as u64, 144, 0, 1 << 20).unwrap();
                        check_signalling_reader(&mut reader, &calls, signal, partial, method);
                        assert!(reader.finish().is_err());
                    } else {
                        let mut reader =
                            LzmaReader::new(source, bytes.len() as u64, Some(144), &PROPS, 1 << 20)
                                .unwrap();
                        check_signalling_reader(&mut reader, &calls, signal, partial, method);
                        assert!(reader.finish().is_err());
                    }
                }
            }
        }
    }
}

struct SignallingSink {
    bytes: Vec<u8>,
    before_error: usize,
    signal: Option<BoundarySignal>,
    write_fault: bool,
    flush_fault: bool,
    calls: std::rc::Rc<std::cell::Cell<usize>>,
}
impl Write for SignallingSink {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        self.calls.set(self.calls.get() + 1);
        if self.write_fault && self.before_error == 0 {
            self.write_fault = false;
            return Err(signal_error(self.signal));
        }
        let count = input.len().min(if self.write_fault {
            self.before_error
        } else {
            usize::MAX
        });
        self.bytes.extend_from_slice(&input[..count]);
        self.before_error = self.before_error.saturating_sub(count);
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.calls.set(self.calls.get() + 1);
        if self.flush_fault {
            self.flush_fault = false;
            return Err(signal_error(self.signal));
        }
        Ok(())
    }
}

#[test]
fn lzma2_typed_interrupted_output_survives_write_helpers_and_poison() {
    let random = noise(65536);
    for signal in [
        BoundarySignal::SourceChanged,
        BoundarySignal::Cancelled,
        BoundarySignal::Limit,
    ] {
        for partial in [false, true] {
            for compressed in [false, true] {
                for method in 0..4 {
                    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
                    let sink = SignallingSink {
                        bytes: Vec::new(),
                        before_error: if partial { 8 } else { 0 },
                        signal: Some(signal),
                        write_fault: true,
                        flush_fault: false,
                        calls: calls.clone(),
                    };
                    let mut writer = Lzma2Writer::new(sink, 1, 64 << 20).unwrap();
                    let input = if compressed {
                        &vec![b'A'; 65536]
                    } else {
                        &random
                    };
                    writer.write_all(input).unwrap();
                    if method == 3 {
                        match writer.finish() {
                            Ok(_) => panic!("finish must preserve typed output failure"),
                            Err(error) => assert_signal(error, signal),
                        }
                        continue;
                    }
                    let error = match method {
                        0 => writer.write(&[0]).unwrap_err(),
                        1 => writer.write_all(&[0]).unwrap_err(),
                        _ => writer.flush().unwrap_err(),
                    };
                    assert_signal(error, signal);
                    assert_eq!(writer.encoded_bytes(), if partial { 8 } else { 0 });
                    let before = calls.get();
                    assert!(writer.write_all(&[0]).is_err());
                    assert!(writer.flush().is_err());
                    assert!(writer.finish().is_err());
                    assert_eq!(
                        calls.get(),
                        before,
                        "poisoned writer must not replay output"
                    );
                }
            }
        }
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let sink = SignallingSink {
            bytes: Vec::new(),
            before_error: usize::MAX,
            signal: Some(signal),
            write_fault: false,
            flush_fault: true,
            calls: calls.clone(),
        };
        let mut writer = Lzma2Writer::new(sink, 1, 64 << 20).unwrap();
        writer
            .write_all(b"fully written chunk followed by typed flush failure")
            .unwrap();
        assert_signal(writer.flush().unwrap_err(), signal);
        assert!(writer.encoded_bytes() > 0);
        let before = calls.get();
        assert!(writer.write(&[0]).is_err());
        assert!(writer.finish().is_err());
        assert_eq!(calls.get(), before);
    }
}

#[test]
fn lzma2_bare_interrupted_writes_and_flush_retry() {
    for compressed in [false, true] {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let sink = SignallingSink {
            bytes: Vec::new(),
            before_error: 8,
            signal: None,
            write_fault: true,
            flush_fault: true,
            calls,
        };
        let mut writer = Lzma2Writer::new(sink, 1, 64 << 20).unwrap();
        let input = if compressed {
            vec![b'A'; 65536]
        } else {
            noise(65536)
        };
        writer.write_all(&input).unwrap();
        writer.flush().unwrap();
        let sink = writer.finish().unwrap();
        let mut reader = Lzma2Reader::new(
            sink.bytes.as_slice(),
            sink.bytes.len() as u64,
            input.len() as u64,
            22,
            16 << 20,
        )
        .unwrap();
        let mut decoded = Vec::new();
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, input);
    }
}

#[test]
fn lzma_memory_and_property_admission_precedes_allocation() {
    for value in [225, 255] {
        let mut props = PROPS;
        props[0] = value;
        assert!(lzma_memory_required(&props).is_err());
    }
    assert!(lzma_memory_required(&PROPS[..4]).is_err());
    let budget = lzma_memory_required(&PROPS).unwrap();
    assert!(LzmaReader::new(&[][..], 0, Some(0), &PROPS, budget - 1).is_err());
    assert!(LzmaReader::new(&[][..], 0, Some(0), &PROPS, budget).is_ok());
    // LZMA1 admits the full lc+lp range; LZMA2 restricts it per chunk.
    let large_context = [224, 0, 0x10, 0, 0];
    assert!(lzma_memory_required(&large_context).unwrap() > 6 << 20);
    assert!(lzma2_memory_required(41).is_err());
    let enormous = lzma2_memory_required(40).unwrap();
    assert!(enormous > u32::MAX as u64);
    assert_eq!(
        Lzma2Reader::new(&[][..], 0, 0, 40, 1 << 20)
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::OutOfMemory
    );
}

#[test]
fn lzma2_python_compressed_and_raw_controls() {
    let mut reader = Lzma2Reader::new(Singles(RAW2), RAW2.len() as u64, 144, 0, 1 << 20).unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, expected());
    reader.finish().unwrap();
    assert_eq!(reader.consumed_bytes(), RAW2.len() as u64);
    for (raw, plain) in [
        (&[1, 0, 0, 65, 0][..], &b"A"[..]),
        (&[1, 0, 0, 65, 2, 0, 0, 66, 0][..], &b"AB"[..]),
        (&[0][..], &b""[..]),
    ] {
        let mut reader =
            Lzma2Reader::new(raw, raw.len() as u64, plain.len() as u64, 0, 1 << 20).unwrap();
        let mut output = vec![0; plain.len()];
        reader.read_exact(&mut output).unwrap();
        reader.finish().unwrap();
        assert_eq!(output, plain);
    }
}

#[test]
fn lzma2_bad_controls_chunks_and_exact_eof() {
    for bytes in [
        &[2, 0, 0, 65, 0][..],
        &[1, 0, 0, 65][..],
        &[1, 0, 0, 65, 0, 0][..],
        &[3][..],
        &[0x80, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0][..],
    ] {
        let mut reader = Lzma2Reader::new(bytes, bytes.len() as u64, 1, 0, 1 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
        assert!(reader.finish().is_err());
    }
    for cut in 0..RAW2.len() {
        let mut reader = Lzma2Reader::new(&RAW2[..cut], cut as u64, 144, 0, 1 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err(), "cut={cut}");
    }
    for size in [143, 145] {
        let mut reader = Lzma2Reader::new(RAW2, RAW2.len() as u64, size, 0, 1 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
    }
    let mut bad = RAW2.to_vec();
    bad[5] = 224; // legal LZMA1 properties with lc+lp=12, forbidden in LZMA2.
    let mut reader = Lzma2Reader::new(bad.as_slice(), bad.len() as u64, 144, 0, 1 << 20).unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
    let mut missing_properties = vec![1, 0, 0, 65];
    missing_properties.extend_from_slice(&[0x80, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0]);
    let mut reader = Lzma2Reader::new(
        missing_properties.as_slice(),
        missing_properties.len() as u64,
        2,
        0,
        1 << 20,
    )
    .unwrap();
    assert!(
        reader.read_to_end(&mut Vec::new()).is_err(),
        "raw01 invalidates properties for later compressed80"
    );
}

fn noise(size: usize) -> Vec<u8> {
    let mut state = 0x52c1_f087u32;
    (0..size)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        })
        .collect()
}

#[test]
fn lzma2_writer_real_compression_all_levels_and_mixed_raw_chunks() {
    for level in [1, 6, 9] {
        let mut writer = Lzma2Writer::new(Vec::new(), level, 64 << 20).unwrap();
        assert_eq!(writer.property(), 22);
        let first = b"project-owned LZMA2 payload, including repeated history\n".repeat(1800);
        let random = noise(131072);
        let mut expected = first.clone();
        expected.extend_from_slice(&random);
        expected.extend_from_slice(&first);
        for part in expected.chunks(137) {
            writer.write_all(part).unwrap();
        }
        assert_eq!(writer.input_bytes(), expected.len() as u64);
        let encoded = writer.finish().unwrap();
        assert!(
            encoded.len() < expected.len() * 3 / 4,
            "level {level} must actually compress"
        );
        let mut reader = Lzma2Reader::new(
            encoded.as_slice(),
            encoded.len() as u64,
            expected.len() as u64,
            22,
            16 << 20,
        )
        .unwrap();
        let mut decoded = Vec::new();
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, expected);
        reader.finish().unwrap();
    }
    let random = noise(150000);
    let repeated = b"same history\n".repeat(8000);
    let mut writer = Lzma2Writer::new(Vec::new(), 6, 64 << 20).unwrap();
    writer.write_all(&random).unwrap();
    writer.flush().unwrap();
    writer.write_all(&repeated).unwrap();
    writer.flush().unwrap();
    assert!(writer.encoded_bytes() >= random.len() as u64);
    let encoded = writer.finish().unwrap();
    assert_eq!(
        encoded[0], 1,
        "incompressible first chunk uses raw dictionary reset"
    );
    let expected = [random, repeated].concat();
    let mut reader = Lzma2Reader::new(
        encoded.as_slice(),
        encoded.len() as u64,
        expected.len() as u64,
        22,
        16 << 20,
    )
    .unwrap();
    let mut decoded = Vec::new();
    reader.read_to_end(&mut decoded).unwrap();
    assert_eq!(decoded, expected);
}

struct BrokenSink {
    remaining: usize,
    written: usize,
}
impl Write for BrokenSink {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::other("injected_write_failure"));
        }
        let count = self.remaining.min(input.len());
        self.remaining -= count;
        self.written += count;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn lzma2_writer_admission_partial_write_and_poison() {
    for level in [0, 2, 10] {
        assert!(Lzma2Writer::new(Vec::new(), level, u64::MAX).is_err());
    }
    let memory = lzma2_writer_memory_required(6).unwrap();
    assert!(Lzma2Writer::new(Vec::new(), 6, memory - 1).is_err());
    let mut writer = Lzma2Writer::new(
        BrokenSink {
            remaining: 8,
            written: 0,
        },
        6,
        memory,
    )
    .unwrap();
    writer
        .write_all(&b"meaningful compressed content\n".repeat(1000))
        .unwrap();
    assert!(writer.flush().is_err());
    assert_eq!(writer.encoded_bytes(), 8);
    assert!(writer.write(&[1]).is_err());
    assert!(writer.flush().is_err());
    assert!(writer.finish().is_err());
    let writer = Lzma2Writer::new(Vec::new(), 1, memory).unwrap();
    assert_eq!(writer.finish().unwrap(), [0]);
    let mut sink = Vec::new();
    {
        let mut writer = Lzma2Writer::new(&mut sink, 1, memory).unwrap();
        writer
            .write_all(b"not implicitly finished by Drop")
            .unwrap();
    }
    assert!(sink.is_empty());
}

#[test]
fn lzma2_writer_levels_search_different_match_depths() {
    let source = noise(70000);
    let mut payload = Vec::new();
    for group in 0..20 {
        let start = group * 2500;
        let reference = [&b"ABC"[..], &source[start..start + 160]].concat();
        payload.extend_from_slice(&reference);
        for decoy in 0..45 {
            payload.extend_from_slice(b"ABC");
            let offset = start + 200 + decoy * 40;
            payload.extend_from_slice(&source[offset..offset + 37]);
        }
        payload.extend_from_slice(&reference);
    }
    let mut sizes = Vec::new();
    for level in [1, 6, 9] {
        let mut writer = Lzma2Writer::new(Vec::new(), level, 64 << 20).unwrap();
        writer.write_all(&payload).unwrap();
        let packed = writer.finish().unwrap();
        sizes.push(packed.len());
        let mut reader = Lzma2Reader::new(
            packed.as_slice(),
            packed.len() as u64,
            payload.len() as u64,
            22,
            16 << 20,
        )
        .unwrap();
        let mut decoded = Vec::new();
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, payload);
    }
    assert!(
        sizes[2] < sizes[0],
        "deeper search must find the distant ambiguous-prefix matches: {sizes:?}"
    );
}

#[test]
#[ignore = "requires separately prepared independent Python/liblzma vectors and export directory"]
fn lzma_independent_matrix_and_export_writer() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("XDZIP_LZMA_VECTOR_DIR").expect("vector directory"),
    );
    let mut paths: Vec<_> = std::fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    paths.sort();
    let mut cases = 0;
    for path in paths {
        if path.extension().is_none_or(|extension| extension != "raw") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        let expected = std::fs::read(path.with_extension("plain")).unwrap();
        let props = std::fs::read(path.with_extension("props")).unwrap();
        let mut output = Vec::new();
        if props.len() == 5 {
            let mut reader = LzmaReader::new(
                Singles(&bytes),
                bytes.len() as u64,
                Some(expected.len() as u64),
                &props,
                64 << 20,
            )
            .unwrap();
            reader.read_to_end(&mut output).unwrap();
            reader.finish().unwrap();
            assert_eq!(reader.consumed_bytes(), bytes.len() as u64);
        } else {
            assert_eq!(props.len(), 1);
            let mut reader = Lzma2Reader::new(
                Singles(&bytes),
                bytes.len() as u64,
                expected.len() as u64,
                props[0],
                64 << 20,
            )
            .unwrap();
            reader.read_to_end(&mut output).unwrap();
            reader.finish().unwrap();
            assert_eq!(reader.consumed_bytes(), bytes.len() as u64);
        }
        assert_eq!(output, expected, "{}", path.display());
        cases += 1;
    }
    assert!(cases >= 10);
    let export = directory.join("rust-writer");
    std::fs::create_dir_all(&export).unwrap();
    let mut payload = noise(262144);
    payload.extend_from_slice(&b"LZMA2 independent output verification\n".repeat(25000));
    let prefix = payload[..131072].to_vec();
    payload.extend_from_slice(&prefix);
    std::fs::write(export.join("payload.plain"), &payload).unwrap();
    for level in [1, 6, 9] {
        let mut writer = Lzma2Writer::new(Vec::new(), level, 64 << 20).unwrap();
        for chunk in payload.chunks(173) {
            writer.write_all(chunk).unwrap();
        }
        let packed = writer.finish().unwrap();
        assert!(packed.len() < payload.len() / 2);
        std::fs::write(export.join(format!("level{level}.raw")), packed).unwrap();
    }
}

#[test]
#[ignore = "requires separately prepared official format-draft data vectors"]
fn lzma_official_draft_vectors() {
    let directory = std::env::var_os("XDZIP_LZMA_DRAFT_VECTORS").expect("vector directory");
    let mut entries: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    let mut good = 0;
    let mut bad = 0;
    for path in entries {
        if path.extension().is_none_or(|value| value != "lzma") {
            continue;
        }
        let data = std::fs::read(&path).unwrap();
        assert!(data.len() >= 13);
        let size = u64::from_le_bytes(data[5..13].try_into().unwrap());
        let expected_size = (size != u64::MAX).then_some(size);
        let mut reader = LzmaReader::new(
            &data[13..],
            (data.len() - 13) as u64,
            expected_size,
            &data[..5],
            64 << 20,
        )
        .unwrap();
        let mut output = Vec::new();
        let result = reader.read_to_end(&mut output);
        if path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("bad_")
        {
            assert!(result.is_err(), "{}", path.display());
            bad += 1;
        } else {
            result.unwrap();
            assert_eq!(output, b"LZMA decoder test example\r\n=========================\r\n! LZMA ! Decoder ! TEST !\r\n=========================\r\n! TEST ! LZMA ! Decoder !\r\n=========================\r\n---- Test Line 1 -------- \r\n=========================\r\n---- Test Line 2 -------- \r\n=========================\r\n=== End of test file ==== \r\n=========================\r\n");
            reader.finish().unwrap();
            good += 1;
        }
    }
    assert_eq!((good, bad), (4, 3));
}

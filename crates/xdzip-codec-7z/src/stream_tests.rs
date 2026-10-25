use super::*;
use std::cell::Cell;
use std::rc::Rc;

// Apply the reflected IEEE polynomial to a linear basis, then exponentiate a
// zero-byte transform. This computes test expectations without a huge Vec.
fn zero_crc(mut length: u64) -> u32 {
    fn apply(matrix: &[u32; 32], mut value: u32) -> u32 {
        let mut out = 0;
        let mut at = 0;
        while value != 0 {
            if value & 1 != 0 {
                out ^= matrix[at];
            }
            value >>= 1;
            at += 1;
        }
        out
    }
    let mut matrix = std::array::from_fn(|bit| {
        let mut value = 1u32 << bit;
        for _ in 0..8 {
            value = (value >> 1) ^ if value & 1 != 0 { 0xedb88320 } else { 0 };
        }
        value
    });
    let mut state = u32::MAX;
    while length != 0 {
        if length & 1 != 0 {
            state = apply(&matrix, state);
        }
        matrix = std::array::from_fn(|bit| apply(&matrix, matrix[bit]));
        length >>= 1;
    }
    !state
}
#[derive(Default)]
struct Observed {
    payload: Cell<u64>,
    maximum: Cell<usize>,
}
struct Sparse {
    position: u64,
    data: u64,
    signature: Vec<u8>,
    header: Vec<u8>,
    observed: Rc<Observed>,
}
impl Read for Sparse {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let length = 32 + self.data + self.header.len() as u64;
        let n = out
            .len()
            .min(length.saturating_sub(self.position).min(out.len() as u64) as usize);
        let end = self.position + n as u64;
        out[..n].fill(0);
        for (start, bytes) in [
            (0, self.signature.as_slice()),
            (32 + self.data, self.header.as_slice()),
        ] {
            let from = self.position.max(start);
            let to = end.min(start + bytes.len() as u64);
            if from < to {
                out[(from - self.position) as usize..(to - self.position) as usize]
                    .copy_from_slice(&bytes[(from - start) as usize..(to - start) as usize]);
            }
        }
        let amount = end
            .min(32 + self.data)
            .saturating_sub(self.position.max(32));
        self.observed
            .payload
            .set(self.observed.payload.get() + amount);
        self.observed
            .maximum
            .set(self.observed.maximum.get().max(n));
        self.position = end;
        Ok(n)
    }
}
impl Seek for Sparse {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let n = match from {
            SeekFrom::Start(n) => i128::from(n),
            SeekFrom::End(n) => {
                i128::from(32 + self.data + self.header.len() as u64) + i128::from(n)
            }
            SeekFrom::Current(n) => i128::from(self.position) + i128::from(n),
        };
        self.position = u64::try_from(n).map_err(|_| io::ErrorKind::InvalidInput)?;
        Ok(self.position)
    }
}
impl ReadSource for Sparse {}
fn sparse_copy(sizes: &[u64]) -> Sparse {
    let total = sizes.iter().sum::<u64>();
    let mut h = vec![1, 4, 6, 0, 1, 9];
    h.extend(number(total));
    h.extend([0, 7, 11, 1, 0, 1, 1, 0, 12]);
    h.extend(number(total));
    h.extend([0, 8, 13]);
    h.extend(number(sizes.len() as u64));
    if sizes.len() > 1 {
        h.push(9);
        for size in &sizes[..sizes.len() - 1] {
            h.extend(number(*size));
        }
    }
    h.extend([10, 1]);
    for size in sizes {
        h.extend(zero_crc(*size).to_le_bytes());
    }
    h.extend([0, 0, 5]);
    h.extend(number(sizes.len() as u64));
    let names_owned = (0..sizes.len())
        .map(|n| format!("large-{n}"))
        .collect::<Vec<_>>();
    h.extend(names(
        &names_owned.iter().map(String::as_str).collect::<Vec<_>>(),
    ));
    h.extend([0, 0]);
    let mut tail = total.to_le_bytes().to_vec();
    tail.extend((h.len() as u64).to_le_bytes());
    tail.extend(crc32(&h).to_le_bytes());
    let mut signature = b"7z\xbc\xaf\x27\x1c\x00\x04".to_vec();
    signature.extend(crc32(&tail).to_le_bytes());
    signature.extend(tail);
    Sparse {
        position: 0,
        data: total,
        signature,
        header: h,
        observed: Rc::new(Observed::default()),
    }
}
#[test]
fn zero_crc_expectations_match_small_vectors_and_independent_python_large_values() {
    for n in 0..256 {
        assert_eq!(zero_crc(n), crc32(&vec![0; n as usize]));
    }
    // Python stdlib zlib repeatedly consumed an actual 1MiB zero buffer.
    assert_eq!(zero_crc(4 << 30), 0xd202ef8d);
    assert_eq!(zero_crc(8 << 30), 0x41d912ff);
}
#[test]
fn unread_huge_folders_have_no_cache_cap_and_file_quota_precedes_data_io() {
    let mut source = sparse_copy(&[3, 32 << 30]);
    let observed = source.observed.clone();
    let control = Cancellation::default();
    let rows = read_index(&mut source, &control, ScanLimits::default(), None)
        .unwrap()
        .rows;
    let mut payload = SevenZipCodec
        .open_payload(&mut source, &rows, &control)
        .unwrap();
    let mut sink = Meter::default();
    payload.decode(&rows[0], &mut sink, &control).unwrap();
    assert_eq!(sink.output, [0; 3]);
    assert_eq!(observed.payload.get(), 3);
    assert_eq!(
        payload.decode(&rows[1], &mut sink, &control),
        Err(DecodeError::Limit)
    );
    assert_eq!(observed.payload.get(), 3);
    assert!(observed.maximum.get() <= 8192);
    let mut source = sparse_copy(&[(8 << 30) + 1]);
    let observed = source.observed.clone();
    let rows = read_index(&mut source, &control, ScanLimits::default(), None)
        .unwrap()
        .rows;
    let mut payload = SevenZipCodec
        .open_payload(&mut source, &rows, &control)
        .unwrap();
    assert_eq!(
        payload.decode(&rows[0], &mut Meter::default(), &control),
        Err(DecodeError::Limit)
    );
    assert_eq!(observed.payload.get(), 0);
}
#[test]
fn payload_custom_memory_budget_is_shared_with_header_and_graph_admission() {
    let mut source = sparse_copy(&[3]);
    let observed = source.observed.clone();
    let control = Cancellation::default();
    let rows = read_index(&mut source, &control, ScanLimits::default(), None)
        .unwrap()
        .rows;
    let mut payload = crate::payload::Payload::open_with_limits(
        &mut source,
        &rows,
        &control,
        None,
        DecodeLimits {
            memory_bytes: 64 << 20,
            ..DecodeLimits::default()
        },
    )
    .unwrap();
    assert_eq!(
        payload.decode(&rows[0], &mut Meter::default(), &control),
        Err(DecodeError::Limit)
    );
    assert_eq!(observed.payload.get(), 0);
    assert_eq!(
        payload.decode(&rows[0], &mut Meter::default(), &control),
        Err(DecodeError::Integrity)
    );
}
#[test]
fn full_graph_memory_sum_and_reentrant_source_borrows_fail_before_unbounded_work() {
    let bytes = archive(
        b"123",
        &[(&[0x21], &[34], 3), (&[0x21], &[34], 3)],
        b"123",
        &[3],
        CHECKS,
    );
    let index = scan(&bytes).unwrap();
    let mut source = Input { bytes, reads: 0 };
    assert_eq!(
        MethodGraphDecoder::default()
            .folder(
                &mut source,
                &index.streams,
                0,
                0,
                &Cancellation::default(),
                None,
                &mut |_| Ok(())
            )
            .unwrap_err(),
        DecodeError::Limit
    );
    assert_eq!(source.reads, 0);
    let mut source = sparse_copy(&[3]);
    let control = Cancellation::default();
    let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
    let mut reader = crate::source::Reader::new(&mut source, &control, 0).unwrap();
    reader.set_limit(20 << 30);
    let source: crate::decode::SharedSource<'_> =
        Rc::new(std::cell::RefCell::new(Box::new(reader)));
    let mut graph = MethodGraphDecoder::default()
        .stream(source.clone(), &index.streams, 0, &control, None)
        .unwrap();
    let _held = source.try_borrow_mut().unwrap();
    assert_eq!(graph.read(&mut [0; 3]), Err(DecodeError::Integrity));
}
#[test]
fn initial_graph_source_checks_retain_the_scan_error_and_poison_metadata_decoder() {
    struct Changed;
    impl HeaderSource for Changed {
        fn length(&self) -> u64 {
            1000
        }
        fn check(&self) -> Result<(), ScanError> {
            Err(ScanError::SourceChanged)
        }
        fn read_exact_at(&mut self, _: u64, _: &mut [u8]) -> Result<(), ScanError> {
            panic!("failed initial source checks must precede reads")
        }
    }
    struct NoOutput;
    impl MetadataSink for NoOutput {
        fn write_folder(&mut self, _: usize, _: &[u8]) -> Result<(), ScanError> {
            panic!("failed metadata must not reach the parser collector")
        }
    }
    let bytes = archive(b"abc", &[(&[0], &[], 3)], b"abc", &[3], CHECKS);
    let index = scan(&bytes).unwrap();
    let mut decoder = MethodGraphDecoder::default();
    for expected in [ScanError::SourceChanged, ScanError::Invalid] {
        assert_eq!(
            decoder.decode_metadata(
                HeaderPurpose::EncodedHeader,
                &mut Changed,
                &index.streams,
                &mut NoOutput,
                &Cancellation::default(),
                None
            ),
            Err(expected)
        );
    }
}
#[test]
fn terminal_and_input_quotas_are_independent_and_eof_at_the_exact_limit_is_allowed() {
    for (total, limits, want) in [
        (
            20,
            DecodeLimits {
                input_bytes: 20,
                work_bytes: 20,
                ..DecodeLimits::default()
            },
            Ok(()),
        ),
        (
            21,
            DecodeLimits {
                input_bytes: 20,
                work_bytes: 21,
                ..DecodeLimits::default()
            },
            Err(DecodeError::Limit),
        ),
        (
            21,
            DecodeLimits {
                input_bytes: 21,
                work_bytes: 20,
                ..DecodeLimits::default()
            },
            Err(DecodeError::Limit),
        ),
    ] {
        let mut source = sparse_copy(&[total]);
        let observed = source.observed.clone();
        let control = Cancellation::default();
        let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
        let mut reader = crate::source::Reader::new(&mut source, &control, 0).unwrap();
        reader.set_limit(limits.input_bytes);
        let source: crate::decode::SharedSource<'_> =
            Rc::new(std::cell::RefCell::new(Box::new(reader)));
        let mut decoder = MethodGraphDecoder::new(limits);
        let mut graph = decoder
            .stream(source, &index.streams, 0, &control, None)
            .unwrap();
        let result = (|| {
            let mut block = [0; 8];
            while graph.read(&mut block)? != 0 {}
            graph.finish()
        })();
        assert_eq!(result, want);
        assert_eq!(observed.payload.get(), 20);
        assert_eq!(decoder.counters(), (20, 20));
    }
}
#[test]
fn recursive_node_work_is_reserved_before_upstream_can_consume_its_credit() {
    let bytes = archive(
        b"abcdef",
        &[(&[0], &[], 6), (&[0], &[], 6)],
        b"abcdef",
        &[6],
        CHECKS,
    );
    let control = Cancellation::default();
    let index = scan(&bytes).unwrap();
    for budget in [11, 12] {
        let source = Input {
            bytes: bytes.clone(),
            reads: 0,
        };
        let shared: crate::decode::SharedSource<'_> =
            Rc::new(std::cell::RefCell::new(Box::new(source)));
        let mut decoder = MethodGraphDecoder::new(DecodeLimits {
            node_work_bytes: budget,
            ..DecodeLimits::default()
        });
        let mut graph = decoder
            .stream(shared, &index.streams, 0, &control, None)
            .unwrap();
        let result = (|| {
            while graph.read(&mut [0])? != 0 {}
            graph.finish()
        })();
        if budget == 11 {
            assert_eq!(result, Err(DecodeError::Limit));
            assert_eq!(decoder.node_work(), 10);
            assert_eq!(decoder.counters(), (5, 5));
        } else {
            result.unwrap();
            assert_eq!(decoder.node_work(), 12);
            assert_eq!(decoder.counters(), (6, 6));
        }
    }
}
struct Count {
    output: u64,
    work: u64,
    credit: u64,
    file: u64,
    declared: u64,
}
impl Count {
    fn new() -> Self {
        Self {
            output: 0,
            work: 0,
            credit: 0,
            file: 0,
            declared: 0,
        }
    }
    fn begin(&mut self, n: u64) {
        self.file = 0;
        self.declared = n;
    }
}
impl DecodedSink for Count {
    fn charge_skipped(&mut self, n: u64) -> Result<(), DecodeError> {
        self.work = self
            .work
            .checked_add(n)
            .filter(|n| *n <= 20 << 30)
            .ok_or(DecodeError::Limit)?;
        self.credit += n;
        Ok(())
    }
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.charge_skipped(bytes.len() as u64)?;
        self.write_precharged_chunk(bytes)
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        let n = bytes.len() as u64;
        self.credit = self.credit.checked_sub(n).ok_or(DecodeError::Limit)?;
        if bytes.iter().any(|byte| *byte != 0) {
            return Err(DecodeError::Integrity);
        }
        self.file = self
            .file
            .checked_add(n)
            .filter(|n| *n <= self.declared && *n <= 8 << 30)
            .ok_or(DecodeError::Limit)?;
        self.output = self
            .output
            .checked_add(n)
            .filter(|n| *n <= 20 << 30)
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
}
#[test]
#[ignore = "explicit release run, actually consumes more than 60GiB of owned zero bytes"]
fn actual_eight_gib_files_twenty_gib_tasks_and_extra_byte_limits_are_streamed() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "large validation uses release"
    );
    assert_eq!(std::env::var("XDZIP_7Z_LARGE").as_deref(), Ok("1"));
    eprintln!("large_validation_pid={}", std::process::id());
    for extra in [false, true] {
        let mut sizes = vec![8 << 30, 8 << 30, 4 << 30];
        if extra {
            sizes.push(1);
        }
        let mut source = sparse_copy(&sizes);
        let observed = source.observed.clone();
        let control = Cancellation::default();
        let rows = read_index(&mut source, &control, ScanLimits::default(), None)
            .unwrap()
            .rows;
        let mut payload = SevenZipCodec
            .open_payload(&mut source, &rows, &control)
            .unwrap();
        let mut count = Count::new();
        for (row, size) in rows.iter().zip(&sizes) {
            count.begin(*size);
            let result = payload.decode(row, &mut count, &control);
            if extra && *size == 1 {
                assert_eq!(result, Err(DecodeError::Limit));
            } else {
                result.unwrap();
                assert_eq!(count.file, *size);
            }
            eprintln!(
                "large_copy extra={extra} file={size} work={} reads={}",
                count.work,
                observed.payload.get()
            );
        }
        assert_eq!(count.work, 20 << 30);
        assert_eq!(count.output, 20 << 30);
        assert_eq!(observed.payload.get(), 20 << 30);
        assert!(observed.maximum.get() <= 8192);
    }
    // Work has an extra byte of capacity: the final failure is specifically
    // physical packed-input admission, rather than terminal WORK or file quota.
    let mut source = sparse_copy(&[(20 << 30) + 1]);
    let observed = source.observed.clone();
    let control = Cancellation::default();
    let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
    let mut reader = crate::source::Reader::new(&mut source, &control, 0).unwrap();
    reader.set_limit(20 << 30);
    let source: crate::decode::SharedSource<'_> =
        Rc::new(std::cell::RefCell::new(Box::new(reader)));
    let mut decoder = MethodGraphDecoder::new(DecodeLimits {
        work_bytes: (20 << 30) + 1,
        ..DecodeLimits::default()
    });
    let mut graph = decoder
        .stream(source, &index.streams, 0, &control, None)
        .unwrap();
    let result = (|| {
        let mut bytes = [0; 8192];
        loop {
            let n = graph.read(&mut bytes)?;
            if n == 0 {
                break;
            }
            if bytes[..n].iter().any(|b| *b != 0) {
                return Err(DecodeError::Integrity);
            }
        }
        Ok(())
    })();
    assert_eq!(result, Err(DecodeError::Limit));
    assert_eq!(observed.payload.get(), 20 << 30);
    assert_eq!(decoder.counters(), (20 << 30, 20 << 30));
    eprintln!(
        "large_input_limit actual_reads={} terminal={}",
        observed.payload.get(),
        decoder.counters().1
    );
}

use crate::{Index, Record, inspect, inspect_pma};
use std::io::{self, Read, SeekFrom, Take};
use xdzip_codec_api::{
    Cancellation, ReadSource, ScanError, ScanLimits,
    payload::{DecodeError, DecodedSink},
};
use xdzip_compression::{
    Crc16, LarcMethod, LarcReader, Lh2Variant, LhDynamicReader, LzhReader, Pm2Reader,
};
use xdzip_crypto::SecretSha256;

const INPUT_LIMIT: u64 = 24 * 1024 * 1024 * 1024;
const WORK_LIMIT: u64 = 96 * 1024 * 1024 * 1024;
#[cfg(test)]
#[path = "pma_tests.rs"]
mod pma_tests;
struct HashingSink<'a> {
    sink: &'a mut dyn DecodedSink,
    work: &'a mut u64,
    hash: SecretSha256,
    bytes: u64,
    emit: bool,
}
impl HashingSink<'_> {
    fn charge(&mut self, amount: u64) -> Result<(), DecodeError> {
        *self.work = self.work.checked_sub(amount).ok_or(DecodeError::Limit)?;
        self.sink.charge_auxiliary(amount)
    }
    fn finish(mut self) -> Result<[u8; 32], DecodeError> {
        self.charge(if self.bytes % 64 < 56 { 64 } else { 128 })?;
        Ok(*self.hash.finish())
    }
}
impl DecodedSink for HashingSink<'_> {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.charge(((self.bytes % 64 + bytes.len() as u64) / 64) * 64)?;
        self.hash.update(bytes);
        self.bytes += bytes.len() as u64;
        if self.emit {
            self.sink.write_chunk(bytes)
        } else {
            self.sink.charge_skipped(bytes.len() as u64)
        }
    }
    fn charge_auxiliary(&mut self, amount: u64) -> Result<(), DecodeError> {
        self.sink.charge_auxiliary(amount)
    }
}
fn scan_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::Integrity,
    }
}
fn io_error(error: io::Error) -> DecodeError {
    if let Some(typed) = error.get_ref().and_then(|e| e.downcast_ref::<ScanError>()) {
        return scan_error(*typed);
    }
    match error.kind() {
        io::ErrorKind::InvalidData | io::ErrorKind::UnexpectedEof => DecodeError::Integrity,
        io::ErrorKind::OutOfMemory => DecodeError::Limit,
        io::ErrorKind::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::SourceIo,
    }
}
fn source_failure(error: &io::Error) -> ScanError {
    match ScanError::from_source_error(error) {
        typed @ (ScanError::Cancelled
        | ScanError::Limit
        | ScanError::Unsupported
        | ScanError::SourceChanged) => typed,
        _ => ScanError::Io,
    }
}
struct Source<'a, 's> {
    source: Take<&'a mut (dyn ReadSource + 's)>,
    remaining: &'a mut u64,
    open_cancel: &'a Cancellation,
    task_cancel: &'a Cancellation,
}
impl Read for Source<'_, '_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.open_cancel
            .check()
            .and_then(|()| self.task_cancel.check())
            .map_err(io::Error::other)?;
        if output.is_empty() || self.source.limit() == 0 {
            return Ok(0);
        }
        let amount = output.len().min((*self.remaining).min(8192) as usize);
        if amount == 0 {
            return Err(io::Error::other(ScanError::Limit));
        }
        let n = match self.source.read(&mut output[..amount]) {
            Err(e) if e.kind() == io::ErrorKind::Interrupted && e.get_ref().is_none() => {
                return Err(e);
            }
            Err(e) => return Err(io::Error::other(source_failure(&e))),
            Ok(n) => n,
        };
        if n == 0 {
            return Err(io::Error::other(ScanError::SourceChanged));
        }
        *self.remaining -= n as u64;
        Ok(n)
    }
}
trait Algorithm: Read {
    fn work_left(&self) -> u64;
}
impl<R: Read, P: FnMut() -> io::Result<()>> Algorithm for Pm2Reader<R, P> {
    fn work_left(&self) -> u64 {
        self.work_remaining()
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Algorithm for LzhReader<R, P> {
    fn work_left(&self) -> u64 {
        self.work_remaining()
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Algorithm for LarcReader<R, P> {
    fn work_left(&self) -> u64 {
        self.work_remaining()
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Algorithm for LhDynamicReader<R, P> {
    fn work_left(&self) -> u64 {
        self.work_remaining()
    }
}
struct Stored<R> {
    source: R,
    work: u64,
    remaining: u64,
}
impl<R: Read> Read for Stored<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() || self.remaining == 0 {
            return Ok(0);
        }
        let amount = output
            .len()
            .min(self.work.min(self.remaining).min(8192) as usize);
        if amount == 0 {
            return Err(io::Error::new(io::ErrorKind::OutOfMemory, "resource_limit"));
        }
        let n = loop {
            match self.source.read(&mut output[..amount]) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted && e.get_ref().is_none() => {
                    continue;
                }
                other => break other?,
            }
        };
        self.work = self
            .work
            .checked_sub(n as u64)
            .ok_or_else(|| io::Error::new(io::ErrorKind::OutOfMemory, "resource_limit"))?;
        self.remaining -= n as u64;
        Ok(n)
    }
}
impl<R: Read> Algorithm for Stored<R> {
    fn work_left(&self) -> u64 {
        self.work
    }
}
fn drain(
    reader: &mut dyn Algorithm,
    record: &Record,
    decoded: &mut u64,
    work: &mut u64,
    sink: &mut dyn DecodedSink,
    cancel: &Cancellation,
) -> Result<(), DecodeError> {
    let mut crc = Crc16::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 8192];
    loop {
        cancel.check().map_err(scan_error)?;
        let before = reader.work_left();
        let result = reader.read(&mut buffer);
        let after = reader.work_left();
        *work = after;
        sink.charge_auxiliary(before.checked_sub(after).ok_or(DecodeError::Limit)?)?;
        let n = result.map_err(io_error)?;
        if n == 0 {
            break;
        }
        total = total.checked_add(n as u64).ok_or(DecodeError::Limit)?;
        *decoded = decoded.checked_sub(n as u64).ok_or(DecodeError::Limit)?;
        if total > record.size {
            return Err(DecodeError::Integrity);
        }
        crc.update(&buffer[..n]);
        sink.write_chunk(&buffer[..n])?;
    }
    if total != record.size
        || record
            .crc16
            .is_some_and(|expected| expected != crc.checksum())
    {
        return Err(DecodeError::Integrity);
    }
    Ok(())
}
/// One bounded payload instance; no output paths, publication or historical repair modes.
pub struct Reader<'a> {
    source: &'a mut dyn ReadSource,
    cancel: &'a Cancellation,
    index: Index,
    input: u64,
    work: u64,
    decoded: u64,
    hash_work: u64,
    failure: Option<DecodeError>,
}
impl<'a> Reader<'a> {
    /// Rescan and bind the complete physical snapshot before allowing any payload.
    pub fn open(
        source: &'a mut dyn ReadSource,
        snapshot: &Index,
        cancel: &'a Cancellation,
    ) -> Result<Self, DecodeError> {
        let result = Self::fresh(source, cancel)?;
        if &result.index != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(result)
    }
    pub(crate) fn fresh(
        source: &'a mut dyn ReadSource,
        cancel: &'a Cancellation,
    ) -> Result<Self, DecodeError> {
        Self::fresh_profile(source, cancel, false)
    }
    pub(crate) fn fresh_pma(
        source: &'a mut dyn ReadSource,
        cancel: &'a Cancellation,
    ) -> Result<Self, DecodeError> {
        Self::fresh_profile(source, cancel, true)
    }
    /// Bind a PMA snapshot, including its physical CP/M record padding.
    pub fn open_pma(
        source: &'a mut dyn ReadSource,
        snapshot: &Index,
        cancel: &'a Cancellation,
    ) -> Result<Self, DecodeError> {
        let result = Self::fresh_profile(source, cancel, true)?;
        if &result.index != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(result)
    }
    fn fresh_profile(
        source: &'a mut dyn ReadSource,
        cancel: &'a Cancellation,
        cpm_padding: bool,
    ) -> Result<Self, DecodeError> {
        let index = if cpm_padding {
            inspect_pma(source, cancel, ScanLimits::default())
        } else {
            inspect(source, cancel, ScanLimits::default())
        }
        .map_err(scan_error)?;
        let input = INPUT_LIMIT
            .checked_sub(index.input_used)
            .ok_or(DecodeError::Limit)?;
        Ok(Self {
            source,
            cancel,
            index,
            input,
            work: WORK_LIMIT,
            decoded: INPUT_LIMIT,
            hash_work: INPUT_LIMIT,
            failure: None,
        })
    }
    /// Frozen validated locators, not payload authentication or output authority.
    pub fn index(&self) -> &Index {
        &self.index
    }
    /// Decode a physical index, drain its exact stream and verify any declared CRC.
    pub fn decode(
        &mut self,
        at: usize,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = self.deliver(at, sink, cancel);
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
    fn deliver(
        &mut self,
        at: usize,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        if self
            .index
            .records
            .get(at)
            .is_some_and(|row| row.method == *b"-lh2-")
        {
            if self.index.records[at].crc16.is_none() {
                return Err(DecodeError::Unsupported);
            }
            let mut selected = None;
            for variant in [Lh2Variant::LhaUnix, Lh2Variant::Jlha] {
                match self.hash_profile(at, variant, sink, cancel, false) {
                    Ok(digest) => match selected {
                        Some((_, previous)) if digest != previous => {
                            return Err(DecodeError::Unsupported);
                        }
                        Some(_) => {}
                        None => selected = Some((variant, digest)),
                    },
                    Err(DecodeError::Integrity) => {}
                    Err(error) => return Err(error),
                }
            }
            let (variant, expected) = selected.ok_or(DecodeError::Integrity)?;
            let actual = self.hash_profile(at, variant, sink, cancel, true)?;
            if actual != expected {
                return Err(DecodeError::Integrity);
            }
            return Ok(());
        }
        self.decode_profile(at, Lh2Variant::Jlha, sink, cancel)
    }
    fn hash_profile(
        &mut self,
        at: usize,
        variant: Lh2Variant,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
        emit: bool,
    ) -> Result<[u8; 32], DecodeError> {
        let mut work = self.hash_work;
        let result = {
            let mut hashing = HashingSink {
                sink,
                work: &mut work,
                hash: SecretSha256::new(),
                bytes: 0,
                emit,
            };
            self.decode_profile(at, variant, &mut hashing, cancel)
                .and_then(|()| hashing.finish())
        };
        self.hash_work = work;
        result
    }
    fn decode_profile(
        &mut self,
        at: usize,
        variant: Lh2Variant,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.cancel
            .check()
            .and_then(|()| cancel.check())
            .map_err(scan_error)?;
        let record = self.index.records.get(at).ok_or(DecodeError::Integrity)?;
        if record.directory {
            return if matches!(record.unix_mode & 0xf000, 0 | 0x4000) {
                Ok(())
            } else {
                Err(DecodeError::Unsupported)
            };
        }
        if !matches!(record.unix_mode & 0xf000, 0 | 0x8000) {
            return Err(DecodeError::Unsupported);
        }
        if record.packed_size > self.input || record.size > self.decoded {
            return Err(DecodeError::Limit);
        }
        if self
            .source
            .seek(SeekFrom::Start(record.payload_offset))
            .map_err(|e| scan_error(source_failure(&e)))?
            != record.payload_offset
        {
            return Err(DecodeError::SourceIo);
        }
        let source = Source {
            source: (&mut *self.source).take(record.packed_size),
            remaining: &mut self.input,
            open_cancel: self.cancel,
            task_cancel: cancel,
        };
        let open_cancel = self.cancel;
        let poll = move || {
            open_cancel
                .check()
                .and_then(|()| cancel.check())
                .map_err(io::Error::other)
        };
        let mut reader: Box<dyn Algorithm + '_> = match &record.method {
            b"-pm2-" => Box::new(
                Pm2Reader::with_poll(
                    source,
                    record.packed_size,
                    record.size,
                    256 * 1024,
                    self.work,
                    poll,
                )
                .map_err(io_error)?,
            ),
            b"-lh0-" | b"-lz4-" | b"-pm0-" => Box::new(Stored {
                source,
                work: self.work,
                remaining: record.packed_size,
            }),
            b"-lh1-" => Box::new(
                LhDynamicReader::with_poll(
                    source,
                    record.method[3] - b'0',
                    record.packed_size,
                    record.size,
                    256 * 1024,
                    self.work,
                    poll,
                )
                .map_err(io_error)?,
            ),
            b"-lh2-" => Box::new(
                LhDynamicReader::with_lh2_variant(
                    source,
                    variant,
                    record.packed_size,
                    record.size,
                    256 * 1024,
                    self.work,
                    poll,
                )
                .map_err(io_error)?,
            ),
            b"-lh3-" | b"-lh4-" | b"-lh5-" | b"-lh6-" | b"-lh7-" => Box::new(
                LzhReader::with_poll(
                    source,
                    record.method[3] - b'0',
                    record.packed_size,
                    record.size,
                    256 * 1024,
                    self.work,
                    poll,
                )
                .map_err(io_error)?,
            ),
            b"-lzs-" | b"-lz5-" => Box::new(
                LarcReader::with_poll(
                    source,
                    if record.method == *b"-lzs-" {
                        LarcMethod::Lzs
                    } else {
                        LarcMethod::Lz5
                    },
                    record.packed_size,
                    record.size,
                    256 * 1024,
                    self.work,
                    poll,
                )
                .map_err(io_error)?,
            ),
            _ => return Err(DecodeError::Unsupported),
        };
        // Initialization already consumed private work. Report it before any
        // output and retain the reduced allowance even when the host rejects it.
        let initialization = self
            .work
            .checked_sub(reader.work_left())
            .ok_or(DecodeError::Limit)?;
        self.work = reader.work_left();
        sink.charge_auxiliary(initialization)?;
        drain(
            &mut *reader,
            record,
            &mut self.decoded,
            &mut self.work,
            sink,
            cancel,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixture_cases;
    use crate::tests::{fixture, refresh, scan};
    use std::io::Cursor;
    struct Sink {
        bytes: Vec<u8>,
        aux: u64,
        fail: Option<DecodeError>,
    }
    impl Sink {
        fn new() -> Self {
            Self {
                bytes: vec![],
                aux: 0,
                fail: None,
            }
        }
    }
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            if let Some(error) = self.fail {
                return Err(error);
            }
            self.bytes.extend_from_slice(bytes);
            Ok(())
        }
        fn charge_auxiliary(&mut self, amount: u64) -> Result<(), DecodeError> {
            self.aux += amount;
            Ok(())
        }
    }
    fn expected(name: &str) -> Vec<u8> {
        if name.ends_with("-short.lzh") {
            let mut result: Vec<u8> = (0..64).collect();
            let mut state = 0x6d2b79f5u32;
            for _ in 0..33000 {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                let distance = state as usize % result.len().min(8192) + 1;
                for _ in 0..3 {
                    let byte = result[result.len() - distance];
                    result.push(byte);
                }
            }
            return result;
        }
        if name == "lh5-initial.lzh" {
            return b"   ".to_vec();
        }
        if name == "lzs-initial.lzh" {
            return vec![b' '; 17];
        }
        if name == "lz5-initial.lzh" {
            return [vec![0; 13], vec![1; 5]].concat();
        }
        if name.contains("empty") {
            vec![]
        } else if name.contains("tail1") {
            b"A".to_vec()
        } else if name.contains("spaces") {
            vec![b' '; 4096]
        } else if name.contains("long") {
            b"abracadabra abracadabra\n".repeat(6 * 4096)
        } else if name.contains("random") {
            let mut state = 0x6d2b79f5u32;
            (0..65000)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    (state >> 24) as u8
                })
                .collect()
        } else {
            b"abracadabra abracadabra\n".repeat(6)
        }
    }
    #[test]
    fn complete_independent_payloads_use_validated_ranges_sizes_and_declared_crc() {
        let cancel = Cancellation::default();
        let mut supported = 0;
        for &(name, method, _, _, _, _) in fixture_cases::CASES {
            let bytes = fixture(name);
            let index = scan(&bytes).unwrap();
            let mut source = Cursor::new(bytes);
            let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
            let mut sink = Sink::new();
            let result = reader.decode(0, &mut sink, &cancel);
            result.unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(sink.bytes, expected(name), "{name}");
            if method != "lh2" {
                assert_eq!(
                    reader.input,
                    INPUT_LIMIT - index.input_used - index.records[0].packed_size
                );
            }
            if method != "lh2" {
                assert_eq!(reader.decoded, INPUT_LIMIT - index.records[0].size);
            }
            assert_eq!(
                reader.work + reader.hash_work,
                WORK_LIMIT + INPUT_LIMIT - sink.aux
            );
            supported += 1;
        }
        assert_eq!(supported, 98);
    }
    #[test]
    fn changed_headers_crc_and_later_packed_damage_are_terminal_even_after_output() {
        let cancel = Cancellation::default();
        let original = fixture("lh5-h0.lzh");
        let index = scan(&original).unwrap();
        let mut altered = original.clone();
        altered[22] = b'j';
        refresh(&mut altered, 33);
        assert!(matches!(
            Reader::open(&mut Cursor::new(altered), &index, &cancel),
            Err(DecodeError::Integrity)
        ));
        let mut altered = original.clone();
        altered[31] ^= 1;
        refresh(&mut altered, 33);
        let frozen = scan(&altered).unwrap();
        let mut source = Cursor::new(altered);
        let mut reader = Reader::open(&mut source, &frozen, &cancel).unwrap();
        let mut sink = Sink::new();
        assert_eq!(
            reader.decode(0, &mut sink, &cancel),
            Err(DecodeError::Integrity)
        );
        assert_eq!(sink.bytes, expected("lh5-h0.lzh"));
        assert_eq!(
            reader.decode(0, &mut Sink::new(), &cancel),
            Err(DecodeError::Integrity)
        );
        let mut altered = original;
        altered[40] ^= 0x80;
        let frozen = scan(&altered).unwrap();
        let mut source = Cursor::new(altered);
        let mut reader = Reader::open(&mut source, &frozen, &cancel).unwrap();
        assert_eq!(
            reader.decode(0, &mut Sink::new(), &cancel),
            Err(DecodeError::Integrity)
        );
        assert_eq!(
            reader.decode(0, &mut Sink::new(), &cancel),
            Err(DecodeError::Integrity)
        );
    }
    #[test]
    fn exact_private_ledgers_sink_failures_cancellation_and_repeated_members_do_not_refund() {
        let cancel = Cancellation::default();
        let bytes = fixture("lh5-h0.lzh");
        let index = scan(&bytes).unwrap();
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        let mut sink = Sink::new();
        reader.decode(0, &mut sink, &cancel).unwrap();
        let cost = sink.aux;
        reader.decode(0, &mut sink, &cancel).unwrap();
        assert_eq!(reader.work, WORK_LIMIT - cost * 2);
        assert_eq!(sink.bytes.len(), 288);
        for limit in [0, 1, 2] {
            let mut source = Cursor::new(&bytes);
            let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
            reader.input = 27;
            reader.decoded = 144;
            reader.work = cost;
            match limit {
                0 => reader.input -= 1,
                1 => reader.decoded -= 1,
                _ => reader.work -= 1,
            };
            assert_eq!(
                reader.decode(0, &mut Sink::new(), &cancel),
                Err(DecodeError::Limit)
            );
            assert_eq!(
                reader.decode(0, &mut Sink::new(), &cancel),
                Err(DecodeError::Limit)
            );
        }
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        reader.input = 27;
        reader.decoded = 144;
        reader.work = cost;
        reader.decode(0, &mut Sink::new(), &cancel).unwrap();
        assert_eq!((reader.input, reader.decoded, reader.work), (0, 0, 0));
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        let mut sink = Sink::new();
        sink.fail = Some(DecodeError::Output);
        assert_eq!(
            reader.decode(0, &mut sink, &cancel),
            Err(DecodeError::Output)
        );
        assert!(reader.input < INPUT_LIMIT - index.input_used);
        assert!(reader.work < WORK_LIMIT);
        assert_eq!(
            reader.decode(0, &mut Sink::new(), &cancel),
            Err(DecodeError::Output)
        );
        let task = Cancellation::default();
        task.cancel();
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        assert_eq!(
            reader.decode(0, &mut Sink::new(), &task),
            Err(DecodeError::Cancelled)
        );
        assert_eq!(
            reader.decode(0, &mut Sink::new(), &cancel),
            Err(DecodeError::Cancelled)
        );
    }
    #[test]
    fn lh2_profile_validation_never_emits_failed_candidate_bytes_or_refunds_probe_work() {
        let cancel = Cancellation::default();
        let mut bytes = fixture("lh2-h0.lzh");
        bytes[31] ^= 1;
        refresh(&mut bytes, 33);
        let index = scan(&bytes).unwrap();
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        let mut sink = Sink::new();
        assert_eq!(
            reader.decode(0, &mut sink, &cancel),
            Err(DecodeError::Integrity)
        );
        assert!(sink.bytes.is_empty());
        assert!(reader.input < INPUT_LIMIT - index.input_used);
        assert!(reader.work < WORK_LIMIT);
        assert_eq!(
            reader.decode(0, &mut Sink::new(), &cancel),
            Err(DecodeError::Integrity)
        );
        let bytes = fixture("lh2-unix-spaces.lzh");
        let index = scan(&bytes).unwrap();
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        reader.hash_work = 4095;
        let mut sink = Sink::new();
        assert_eq!(
            reader.decode(0, &mut sink, &cancel),
            Err(DecodeError::Limit)
        );
        assert!(sink.bytes.is_empty());
        assert_eq!(
            reader.decode(0, &mut sink, &cancel),
            Err(DecodeError::Limit)
        );
        struct Reject;
        impl DecodedSink for Reject {
            fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
                panic!("probe must not publish")
            }
            fn charge_skipped(&mut self, _: u64) -> Result<(), DecodeError> {
                Err(DecodeError::Cancelled)
            }
        }
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        assert_eq!(
            reader.decode(0, &mut Reject, &cancel),
            Err(DecodeError::Cancelled)
        );
        assert_eq!(
            reader.decode(0, &mut Sink::new(), &cancel),
            Err(DecodeError::Cancelled)
        );
    }
    #[test]
    fn typed_source_failure_during_profile_probe_is_terminal_without_trying_another_variant() {
        use std::{
            cell::Cell,
            io::{Seek, SeekFrom},
            rc::Rc,
        };
        struct FaultStream {
            source: Cursor<Vec<u8>>,
            failure: Rc<Cell<Option<ScanError>>>,
            attempts: Rc<Cell<u32>>,
        }
        impl Read for FaultStream {
            fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
                if let Some(error) = self.failure.get() {
                    self.attempts.set(self.attempts.get() + 1);
                    return Err(io::Error::new(io::ErrorKind::Interrupted, error));
                }
                self.source.read(bytes)
            }
        }
        impl Seek for FaultStream {
            fn seek(&mut self, at: SeekFrom) -> io::Result<u64> {
                self.source.seek(at)
            }
        }
        impl ReadSource for FaultStream {}
        for (error, expected) in [
            (ScanError::Io, DecodeError::SourceIo),
            (ScanError::SourceChanged, DecodeError::SourceIo),
            (ScanError::Invalid, DecodeError::SourceIo),
            (ScanError::Cancelled, DecodeError::Cancelled),
            (ScanError::Limit, DecodeError::Limit),
            (ScanError::Unsupported, DecodeError::Unsupported),
        ] {
            let bytes = fixture("lh2-h0.lzh");
            let index = scan(&bytes).unwrap();
            let cancel = Cancellation::default();
            let failure = Rc::new(Cell::new(None));
            let attempts = Rc::new(Cell::new(0));
            let mut source = FaultStream {
                source: Cursor::new(bytes),
                failure: failure.clone(),
                attempts: attempts.clone(),
            };
            let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
            failure.set(Some(error));
            let mut sink = Sink::new();
            assert_eq!(reader.decode(0, &mut sink, &cancel), Err(expected));
            assert_eq!(attempts.get(), 1);
            assert!(sink.bytes.is_empty());
            assert_eq!(reader.decode(0, &mut sink, &cancel), Err(expected));
            assert_eq!(attempts.get(), 1);
        }
    }
    #[test]
    fn unsupported_methods_special_records_and_crc_less_lh2_cannot_be_output() {
        let cancel = Cancellation::default();
        let mut bytes = fixture("lh0-h0.lzh");
        bytes[2..7].copy_from_slice(b"-lhx-");
        refresh(&mut bytes, 33);
        let index = scan(&bytes).unwrap();
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        let mut sink = Sink::new();
        assert_eq!(
            reader.decode(0, &mut sink, &cancel),
            Err(DecodeError::Unsupported)
        );
        assert!(sink.bytes.is_empty());
        let mut bytes = fixture("lh2-h0.lzh");
        bytes.drain(31..33);
        bytes[0] -= 2;
        refresh(&mut bytes, 31);
        let index = scan(&bytes).unwrap();
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        assert_eq!(
            reader.decode(0, &mut sink, &cancel),
            Err(DecodeError::Unsupported)
        );
        assert!(sink.bytes.is_empty());
        let bytes = crate::tests::with_extension(0x50, &0xa1ffu16.to_le_bytes());
        let index = scan(&bytes).unwrap();
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        assert_eq!(
            reader.decode(0, &mut sink, &cancel),
            Err(DecodeError::Unsupported)
        );
        assert!(sink.bytes.is_empty());
        let mut bytes = fixture("lh0-h0.lzh");
        bytes.truncate(33);
        bytes.push(0);
        bytes[2..7].copy_from_slice(b"-lhd-");
        bytes[7..15].fill(0);
        bytes[31..33].fill(0);
        refresh(&mut bytes, 33);
        let index = scan(&bytes).unwrap();
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        assert_eq!(reader.decode(0, &mut sink, &cancel), Ok(()));
        assert!(sink.bytes.is_empty());
    }
}

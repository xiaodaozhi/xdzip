use super::*;
use crate::{Index, read_index};
use std::{cell::Cell, io::Cursor};
use xdzip_codec_api::payload::{DecodedSink, PayloadCodec};
use xdzip_codec_api::{MetadataCodec, ScanLimits};
pub(crate) struct Owned {
    pub(crate) entries: Vec<EncodeEntry>,
    pub(crate) bytes: Vec<Vec<u8>>,
    pub(crate) opens: Cell<usize>,
}
impl EncodeInput for Owned {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        self.opens.set(self.opens.get() + 1);
        Ok(Box::new(Cursor::new(
            self.bytes.get(index).ok_or(EncodeError::Source)?.as_slice(),
        )))
    }
}
pub(crate) fn owned(records: &[(&str, bool, &[u8])]) -> Owned {
    Owned {
        entries: records
            .iter()
            .map(|(name, directory, bytes)| EncodeEntry {
                name: (*name).to_owned(),
                directory: *directory,
                size: bytes.len() as u64,
                modified_unix: 1_700_000_001,
            })
            .collect(),
        bytes: records.iter().map(|record| record.2.to_vec()).collect(),
        opens: Cell::new(0),
    }
}
pub(crate) fn settings(level: u8) -> EncodeSettings<'static> {
    EncodeSettings {
        method: if level == 0 { "copy" } else { "lzma2" },
        level,
        solid: false,
        encrypt_headers: false,
        password: None,
    }
}
pub(crate) fn create(input: &dyn EncodeInput, settings: EncodeSettings<'_>) -> Vec<u8> {
    let mut output = Vec::new();
    SevenZipCodec
        .encode(
            input,
            &mut output,
            settings,
            &Cancellation::default(),
            &mut |_| {},
        )
        .unwrap();
    output
}
struct Collect {
    bytes: Vec<u8>,
    credit: u64,
}
impl DecodedSink for Collect {
    fn charge_skipped(&mut self, count: u64) -> Result<(), DecodeError> {
        self.credit = self.credit.checked_add(count).ok_or(DecodeError::Limit)?;
        Ok(())
    }
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.credit = self
            .credit
            .checked_sub(bytes.len() as u64)
            .ok_or(DecodeError::Integrity)?;
        self.write_chunk(bytes)
    }
}
pub(crate) fn unpack(archive: &[u8], password: Option<&str>) -> (Index, Vec<Vec<u8>>) {
    let control = Cancellation::default();
    let mut source = Cursor::new(archive);
    let index = read_index(&mut source, &control, ScanLimits::default(), password).unwrap();
    let mut payload = SevenZipCodec
        .open_payload_with_password(&mut source, &index.rows, &control, password)
        .unwrap();
    let mut values = Vec::new();
    for row in &index.rows {
        let mut sink = Collect {
            bytes: Vec::new(),
            credit: 0,
        };
        payload.decode(row, &mut sink, &control).unwrap();
        values.push(sink.bytes);
    }
    (index, values)
}
#[test]
fn encode_four_levels_solid_crc_names_times_directories_and_empty() {
    let first = b"independent original sevenzip writer payload\n".repeat(2000);
    let last: Vec<_> = (0..=255).cycle().take(17000).collect();
    let input = owned(&[
        ("nested", true, b""),
        ("empty.txt", false, b""),
        ("nested/中文🙂.txt", false, &first),
        ("日本語.bin", false, &last),
    ]);
    for level in [0, 1, 6, 9] {
        for solid in [false, true] {
            let mut options = settings(level);
            options.solid = solid;
            let mut output = Vec::new();
            let mut progress = Vec::new();
            SevenZipCodec
                .encode(
                    &input,
                    &mut output,
                    options,
                    &Cancellation::default(),
                    &mut |event| progress.push(event.input_bytes),
                )
                .unwrap();
            assert_eq!(
                progress.last().copied(),
                Some(first.len() as u64 + last.len() as u64)
            );
            assert!(progress.windows(2).all(|pair| pair[0] < pair[1]));
            let (index, values) = unpack(&output, None);
            assert_eq!(values, input.bytes);
            assert_eq!(
                index
                    .rows
                    .iter()
                    .map(|row| row.name_bytes())
                    .collect::<Vec<_>>(),
                [
                    "nested/".as_bytes(),
                    "empty.txt".as_bytes(),
                    "nested/中文🙂.txt".as_bytes(),
                    "日本語.bin".as_bytes()
                ]
            );
            assert_eq!(index.streams.folders.len(), if solid { 1 } else { 2 });
            assert!(index.streams.packs.iter().all(|pack| pack.crc.is_some()));
            assert!(
                index
                    .streams
                    .folders
                    .iter()
                    .all(|folder| folder.crc.is_some()
                        && folder.substreams.iter().all(|stream| stream.crc.is_some()))
            );
            assert!(
                index
                    .files
                    .iter()
                    .all(|file| file.modified_time == Some(133_444_736_010_000_000))
            );
            if level != 0 {
                assert!(
                    output.len() < first.len() / 4,
                    "real LZMA2 compression required"
                );
            }
        }
    }
}
#[test]
fn encode_empty_archive_and_only_empty_records() {
    for input in [
        owned(&[]),
        owned(&[("dir", true, b""), ("empty", false, b"")]),
    ] {
        let output = create(&input, settings(0));
        let (index, values) = unpack(&output, None);
        assert_eq!(values, input.bytes);
        assert!(index.streams.folders.is_empty());
        assert_eq!(
            input.opens.get(),
            input
                .entries
                .iter()
                .filter(|entry| !entry.directory)
                .count()
        );
    }
}
#[test]
fn encode_aes_header_hides_names_and_fresh_ivs_reuse_kdf() {
    let input = owned(&[
        ("秘密.txt", false, b"classified owned test bytes"),
        ("second", false, b"other bytes"),
        ("empty", false, b""),
    ]);
    let mut options = settings(0);
    options.password = Some("自有测试密码🙂");
    options.encrypt_headers = true;
    let first = create(&input, options);
    let second = create(&input, options);
    assert_ne!(first, second);
    assert_eq!(
        SevenZipCodec.scan(
            &mut Cursor::new(&first),
            &Cancellation::default(),
            ScanLimits::default()
        ),
        Err(ScanError::PasswordRequired)
    );
    let (index, values) = unpack(&first, options.password);
    assert_eq!(values, input.bytes);
    assert!(index.rows[..2].iter().all(|row| row.encrypted));
    let ivs: Vec<_> = index
        .streams
        .folders
        .iter()
        .map(|folder| folder.coders[0].properties.clone())
        .collect();
    assert_ne!(ivs[0], ivs[1]);
    let offset = u64::from_le_bytes(first[12..20].try_into().unwrap()) as usize + 32;
    let raw_name: Vec<_> = "秘密.txt"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    assert!(
        !first[offset..]
            .windows(raw_name.len())
            .any(|window| window == raw_name)
    );
}
struct FaultInput {
    entries: Vec<EncodeEntry>,
    data: Vec<u8>,
    fail_at: Option<usize>,
    signal: ScanError,
    opened: Cell<usize>,
    reads: std::rc::Rc<Cell<usize>>,
}
struct FaultReader<'a> {
    owner: &'a FaultInput,
    position: usize,
}
impl Read for FaultReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.owner.reads.set(self.owner.reads.get() + 1);
        if self
            .owner
            .fail_at
            .is_some_and(|offset| self.position >= offset)
        {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                self.owner.signal,
            ));
        }
        let count = output.len().min(self.owner.data.len() - self.position).min(
            self.owner
                .fail_at
                .unwrap_or(usize::MAX)
                .saturating_sub(self.position),
        );
        output[..count].copy_from_slice(&self.owner.data[self.position..self.position + count]);
        self.position += count;
        Ok(count)
    }
}
impl EncodeInput for FaultInput {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        self.opened.set(self.opened.get() + 1);
        Ok(Box::new(FaultReader {
            owner: self,
            position: 0,
        }))
    }
}
#[test]
fn encode_strict_frozen_size_probe_and_typed_source_errors() {
    for (data, size) in [(vec![1; 9], 10), (vec![1; 11], 10), (vec![1], 0)] {
        let mut input = owned(&[("known", false, &data)]);
        input.entries[0].size = size;
        let mut output = Vec::new();
        assert_eq!(
            SevenZipCodec.encode(
                &input,
                &mut output,
                settings(0),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Source)
        );
        assert_eq!(
            &output[..6],
            [0; 6],
            "signature must only finalize on success"
        );
    }
    for signal in [
        ScanError::SourceChanged,
        ScanError::Cancelled,
        ScanError::Limit,
    ] {
        for partial in [false, true] {
            let input = FaultInput {
                entries: owned(&[("known", false, &[0; 9000])]).entries,
                data: vec![0; 9000],
                fail_at: Some(if partial { 8192 } else { 0 }),
                signal,
                opened: Cell::new(0),
                reads: std::rc::Rc::new(Cell::new(0)),
            };
            let mut output = Vec::new();
            let result = SevenZipCodec.encode(
                &input,
                &mut output,
                settings(6),
                &Cancellation::default(),
                &mut |_| {},
            );
            assert_eq!(result, Err(scan_error(signal)));
            assert!(input.reads.get() <= 2, "typed Interrupted must never retry");
            assert_eq!(&output[..6], [0; 6]);
        }
    }
}
struct RejectOutput {
    bytes: Vec<u8>,
    reject_after: usize,
    failure: EncodeError,
    calls: usize,
}
impl Write for RejectOutput {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        panic!("writer must use host random staging writes")
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl EncodeOutput for RejectOutput {
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<(), EncodeError> {
        self.calls += 1;
        if self.bytes.len() >= self.reject_after {
            return Err(self.failure);
        }
        self.bytes.write_at(offset, bytes)
    }
}
#[test]
fn encode_cancel_and_partial_output_rejection_never_finalize_signature() {
    let input = owned(&[("known", false, &vec![1; 32000])]);
    for failure in [
        EncodeError::Cancelled,
        EncodeError::Limit,
        EncodeError::Output,
    ] {
        let mut output = RejectOutput {
            bytes: Vec::new(),
            reject_after: 8192,
            failure,
            calls: 0,
        };
        assert_eq!(
            SevenZipCodec.encode(
                &input,
                &mut output,
                settings(0),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(failure)
        );
        assert!(output.bytes.len() > 32);
        assert_eq!(&output.bytes[..6], [0; 6]);
    }
    let control = Cancellation::default();
    let mut output = Vec::new();
    assert_eq!(
        SevenZipCodec.encode(&input, &mut output, settings(0), &control, &mut |_| control
            .cancel()),
        Err(EncodeError::Cancelled)
    );
    assert_eq!(output, [0; 32]);
}
#[test]
fn encode_preflight_limits_and_unsupported_settings_touch_no_input_or_output() {
    let mut input = owned(&[("known", false, b"x")]);
    {
        let (size, expected) = (FILE_LIMIT + 1, EncodeError::Limit);
        input.entries[0].size = size;
        let mut output = Vec::new();
        assert_eq!(
            SevenZipCodec.encode(
                &input,
                &mut output,
                settings(0),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(expected)
        );
        assert!(output.is_empty());
        assert_eq!(input.opens.get(), 0);
    }
    input.entries[0].size = 1;
    for options in [
        EncodeSettings {
            method: "ppmd",
            ..settings(0)
        },
        EncodeSettings {
            level: 2,
            ..settings(6)
        },
        EncodeSettings {
            encrypt_headers: true,
            ..settings(0)
        },
    ] {
        let mut output = Vec::new();
        assert_eq!(
            SevenZipCodec.encode(
                &input,
                &mut output,
                options,
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Unsupported)
        );
        assert!(output.is_empty());
    }
    for name in ["../bad", "bad/", "C:device", "NUL", "link\\target"] {
        input.entries[0].name = name.to_owned();
        let mut output = Vec::new();
        assert_eq!(
            SevenZipCodec.encode(
                &input,
                &mut output,
                settings(0),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Invalid)
        );
        assert!(output.is_empty());
    }
}
#[test]
#[ignore = "writes owned archives and verifies all payloads with a pinned independent official 7-Zip tool"]
fn encode_official_four_levels_solid_aes_headers_knownbytes_matrix() {
    use sha2::{Digest, Sha256};
    let reference = std::path::PathBuf::from(std::env::var_os("XDZIP_7Z_REFERENCE").unwrap());
    let expected_sha = std::env::var("XDZIP_7Z_REFERENCE_SHA256").unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(std::fs::read(&reference).unwrap())),
        expected_sha
    );
    let directory = std::path::PathBuf::from(std::env::var_os("XDZIP_7Z_WRITER_OUTPUT").unwrap())
        .canonicalize()
        .unwrap();
    let build = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build")
        .canonicalize()
        .unwrap();
    assert!(directory.starts_with(build));
    let plain = b"self-authored original 7z writer interoperability contents\n".repeat(2200);
    let binary: Vec<_> = (0..=255).cycle().take(65539).collect();
    let input = owned(&[
        ("nested", true, b""),
        ("empty", false, b""),
        ("nested/中文🙂.txt", false, &plain),
        ("日本語.bin", false, &binary),
    ]);
    let mut cases = 0;
    for level in [0, 1, 6, 9] {
        for solid in [false, true] {
            for encryption in [0, 1, 2] {
                let mut options = settings(level);
                options.solid = solid;
                options.password = if encryption == 0 {
                    None
                } else {
                    Some("owned-password-中文🙂")
                };
                options.encrypt_headers = encryption == 2;
                let name = format!("level{level}-solid{}-crypto{encryption}", u8::from(solid));
                let output = create(&input, options);
                let path = directory.join(format!("{name}.7z"));
                std::fs::write(&path, &output).unwrap();
                let (index, values) = unpack(&output, options.password);
                assert_eq!(values, input.bytes);
                let mut test = std::process::Command::new(&reference);
                test.arg("t").arg(&path).args(["-y", "-bsp0"]);
                if let Some(password) = options.password {
                    test.arg(format!("-p{password}"));
                }
                let tested = test.output().unwrap();
                std::fs::write(
                    directory.join(format!("{name}.log")),
                    [&tested.stdout[..], &tested.stderr[..]].concat(),
                )
                .unwrap();
                assert!(tested.status.success(), "{name} official test");
                for (entry, expected) in input
                    .entries
                    .iter()
                    .zip(&input.bytes)
                    .filter(|(entry, _)| !entry.directory)
                {
                    let mut extract = std::process::Command::new(&reference);
                    extract
                        .arg("x")
                        .arg(&path)
                        .arg("-so")
                        .arg(&entry.name)
                        .args(["-y", "-bso0", "-bse2", "-bsp0"]);
                    if let Some(password) = options.password {
                        extract.arg(format!("-p{password}"));
                    }
                    let extracted = extract.output().unwrap();
                    assert!(extracted.status.success(), "{name} extract");
                    assert_eq!(&extracted.stdout, expected, "{name} {}", entry.name);
                }
                eprintln!(
                    "writer_case {name} archive={} raw={} folders={} sha256={:x}",
                    output.len(),
                    plain.len() + binary.len(),
                    index.streams.folders.len(),
                    Sha256::digest(&output)
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 24);
}

#[test]
fn encode_many_distinct_iv_folders_do_not_repeat_kdf() {
    let count = 129usize;
    let input = Owned {
        entries: (0..count)
            .map(|index| EncodeEntry {
                name: format!("row-{index}"),
                directory: false,
                size: 1,
                modified_unix: 0,
            })
            .collect(),
        bytes: (0..count).map(|index| vec![index as u8]).collect(),
        opens: Cell::new(0),
    };
    let mut options = settings(0);
    options.password = Some("owned-many-folder-password");
    let archive = create(&input, options);
    let (index, bytes) = unpack(&archive, options.password);
    assert_eq!(bytes, input.bytes);
    assert_eq!(index.streams.folders.len(), count);
    let mut properties = std::collections::BTreeSet::new();
    for folder in &index.streams.folders {
        assert!(properties.insert(folder.coders[0].properties.clone()));
    }
}
struct VirtualInput {
    entries: [EncodeEntry; 1],
    reads: std::rc::Rc<Cell<usize>>,
}
struct VirtualReader {
    remaining: u64,
    reads: std::rc::Rc<Cell<usize>>,
    interrupt: bool,
}
impl Read for VirtualReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        if self.interrupt {
            self.interrupt = false;
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        assert!(
            output.len() <= 8192,
            "source read is bounded, including EOF probe"
        );
        let count = self.remaining.min(output.len() as u64) as usize;
        output[..count].fill(90);
        self.remaining -= count as u64;
        Ok(count)
    }
}
impl EncodeInput for VirtualInput {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        Ok(Box::new(VirtualReader {
            remaining: self.entries[0].size,
            reads: self.reads.clone(),
            interrupt: true,
        }))
    }
}
struct DiscardStage {
    end: u64,
    written: u64,
    max_chunk: usize,
    signature: [u8; 32],
    flush_interrupt: bool,
    writes: usize,
}
impl Write for DiscardStage {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        panic!("only bounded random staging writes")
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.flush_interrupt {
            self.flush_interrupt = false;
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        Ok(())
    }
}
impl EncodeOutput for DiscardStage {
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<(), EncodeError> {
        assert!(bytes.len() <= 8192);
        self.writes += 1;
        self.max_chunk = self.max_chunk.max(bytes.len());
        self.written += bytes.len() as u64;
        if offset == 0 {
            assert_eq!(bytes.len(), 32);
            self.signature.copy_from_slice(bytes);
        } else {
            assert_eq!(offset, self.end);
        }
        self.end = self.end.max(offset + bytes.len() as u64);
        Ok(())
    }
}
#[test]
fn encode_streams_large_source_into_bounded_discard_stage_and_retries_bare_io() {
    let input = VirtualInput {
        entries: [EncodeEntry {
            name: "owned-stream".to_owned(),
            directory: false,
            size: 16 << 20,
            modified_unix: 0,
        }],
        reads: std::rc::Rc::new(Cell::new(0)),
    };
    let mut output = DiscardStage {
        end: 0,
        written: 0,
        max_chunk: 0,
        signature: [0; 32],
        flush_interrupt: true,
        writes: 0,
    };
    let mut actual = 0;
    SevenZipCodec
        .encode(
            &input,
            &mut output,
            settings(0),
            &Cancellation::default(),
            &mut |event| actual = event.input_bytes,
        )
        .unwrap();
    assert_eq!(actual, 16 << 20);
    assert!(input.reads.get() > 2048);
    assert_eq!(output.max_chunk, 8192);
    assert!(output.writes < 2200);
    assert!(output.end > 16 << 20);
    assert_eq!(output.written, output.end + 32);
    assert_eq!(&output.signature[..6], b"7z\xbc\xaf\x27\x1c");
    let mut encrypted = DiscardStage {
        end: 0,
        written: 0,
        max_chunk: 0,
        signature: [0; 32],
        flush_interrupt: false,
        writes: 0,
    };
    let mut options = settings(0);
    options.password = Some("owned-buffered-aes");
    SevenZipCodec
        .encode(
            &input,
            &mut encrypted,
            options,
            &Cancellation::default(),
            &mut |_| {},
        )
        .unwrap();
    assert_eq!(encrypted.max_chunk, 8192);
    assert!(
        encrypted.writes < 2200,
        "AES blocks must be batched before host IPC"
    );
}
#[test]
fn encode_aggregate_and_record_limits_are_admitted_before_streams() {
    let mut input = owned(&[
        ("first", false, b""),
        ("second", false, b""),
        ("third", false, b""),
    ]);
    for entry in &mut input.entries {
        entry.size = FILE_LIMIT;
    }
    let mut output = Vec::new();
    assert_eq!(
        SevenZipCodec.encode(
            &input,
            &mut output,
            settings(0),
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Limit)
    );
    assert!(output.is_empty());
    assert_eq!(input.opens.get(), 0);
    let name = vec!["x".repeat(255); 16].join("/");
    let entry = EncodeEntry {
        name,
        directory: false,
        size: 0,
        modified_unix: 0,
    };
    let count = NAMES_LIMIT / entry.name.len() + 1;
    let input = Owned {
        entries: vec![entry; count],
        bytes: Vec::new(),
        opens: Cell::new(0),
    };
    assert_eq!(
        SevenZipCodec.encode(
            &input,
            &mut output,
            settings(0),
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Limit)
    );
    assert!(output.is_empty());
    assert_eq!(input.opens.get(), 0);
    let input = Owned {
        entries: vec![
            EncodeEntry {
                name: "x".to_owned(),
                directory: false,
                size: 0,
                modified_unix: 0
            };
            MAX_ENTRIES + 1
        ],
        bytes: Vec::new(),
        opens: Cell::new(0),
    };
    assert_eq!(
        SevenZipCodec.encode(
            &input,
            &mut output,
            settings(0),
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Limit)
    );
    assert!(output.is_empty());
}
#[test]
#[ignore = "requires pinned official tool; validates empty archives and many distinct-IV encrypted folders"]
fn encode_official_empty_and_many_folder_profiles() {
    use sha2::{Digest, Sha256};
    let reference = std::path::PathBuf::from(std::env::var_os("XDZIP_7Z_REFERENCE").unwrap());
    assert_eq!(
        format!("{:x}", Sha256::digest(std::fs::read(&reference).unwrap())),
        std::env::var("XDZIP_7Z_REFERENCE_SHA256").unwrap()
    );
    let directory = std::path::PathBuf::from(std::env::var_os("XDZIP_7Z_WRITER_OUTPUT").unwrap())
        .canonicalize()
        .unwrap();
    let build = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build")
        .canonicalize()
        .unwrap();
    assert!(directory.starts_with(build));
    let many = Owned {
        entries: (0..129)
            .map(|index| EncodeEntry {
                name: format!("row-{index:03}"),
                directory: false,
                size: 1,
                modified_unix: 1_700_000_001,
            })
            .collect(),
        bytes: (0..129).map(|index| vec![index as u8]).collect(),
        opens: Cell::new(0),
    };
    let mut encrypted = settings(0);
    encrypted.password = Some("owned-profile-password");
    for (name, input, options) in [
        ("empty-archive", owned(&[]), settings(0)),
        (
            "only-empty",
            owned(&[("folder", true, b""), ("empty", false, b"")]),
            settings(6),
        ),
        (
            "empty-header-aes",
            owned(&[]),
            EncodeSettings {
                encrypt_headers: true,
                ..encrypted
            },
        ),
        ("many-folder-aes", many, encrypted),
    ] {
        let archive = create(&input, options);
        assert_eq!(unpack(&archive, options.password).1, input.bytes);
        let path = directory.join(format!("{name}.7z"));
        std::fs::write(&path, &archive).unwrap();
        let mut test = std::process::Command::new(&reference);
        test.arg("t").arg(&path).args(["-y", "-bsp0"]);
        if let Some(password) = options.password {
            test.arg(format!("-p{password}"));
        }
        let checked = test.output().unwrap();
        std::fs::write(
            directory.join(format!("{name}.log")),
            [&checked.stdout[..], &checked.stderr[..]].concat(),
        )
        .unwrap();
        assert!(checked.status.success(), "{name}");
        let mut extract = std::process::Command::new(&reference);
        extract
            .arg("x")
            .arg(&path)
            .args(["-so", "-y", "-bso0", "-bse2", "-bsp0"]);
        if let Some(password) = options.password {
            extract.arg(format!("-p{password}"));
        }
        let decoded = extract.output().unwrap();
        assert!(decoded.status.success(), "{name}");
        assert_eq!(decoded.stdout, input.bytes.concat());
        eprintln!(
            "writer_extra {name} records={} archive={} sha256={:x}",
            input.entries.len(),
            archive.len(),
            Sha256::digest(&archive)
        );
    }
}
struct FinalOutput {
    bytes: Vec<u8>,
    reject_patch: bool,
    flush_signal: Option<DecodeError>,
}
impl Write for FinalOutput {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        panic!("host staging only")
    }
    fn flush(&mut self) -> io::Result<()> {
        self.flush_signal.map_or(Ok(()), |error| {
            Err(io::Error::new(io::ErrorKind::Interrupted, error))
        })
    }
}
impl EncodeOutput for FinalOutput {
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<(), EncodeError> {
        if self.reject_patch && offset == 0 && bytes.starts_with(b"7z") {
            return Err(EncodeError::Output);
        }
        self.bytes.write_at(offset, bytes)
    }
}
#[test]
fn encode_final_patch_and_typed_flush_failures_remain_failures() {
    let input = owned(&[("owned", false, b"finalized payload")]);
    let mut output = FinalOutput {
        bytes: Vec::new(),
        reject_patch: true,
        flush_signal: None,
    };
    assert_eq!(
        SevenZipCodec.encode(
            &input,
            &mut output,
            settings(0),
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Output)
    );
    assert_eq!(&output.bytes[..6], &[0; 6]);
    for (signal, expected) in [
        (DecodeError::Cancelled, EncodeError::Cancelled),
        (DecodeError::Limit, EncodeError::Limit),
        (DecodeError::Output, EncodeError::Output),
    ] {
        let mut output = FinalOutput {
            bytes: Vec::new(),
            reject_patch: false,
            flush_signal: Some(signal),
        };
        assert_eq!(
            SevenZipCodec.encode(
                &input,
                &mut output,
                settings(0),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(expected)
        );
        assert!(
            output.bytes.starts_with(b"7z"),
            "even a valid-looking stage is not publication permission"
        );
    }
    struct StreamingOnly;
    impl Write for StreamingOnly {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            panic!("missing random-write capability")
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    impl EncodeOutput for StreamingOnly {}
    assert_eq!(
        SevenZipCodec.encode(
            &input,
            &mut StreamingOnly,
            settings(0),
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Unsupported)
    );
}

// crates/xdzip-codec-rar/src/rar29_store_owned_tests.rs

//! Unchanged official-writer data, independently defined mathematical plaintext.
use crate::*;
use std::io::Cursor;
use xdzip_codec_api::payload::DecodedSink;
use xdzip_compression::{Crc32, crc32};

const ARCHIVE: &[u8] = include_bytes!("../tests/fixtures/rar29-solid-store-owned/archive.rar");
const LENGTHS: [usize; 3] = [131_071, 196_613, 131_071];
const PACKED: [u64; 3] = [461, 196_613, 220];
const NAMES: [&[u8]; 3] = [b"first.bin", b"middle.rnd", b"third.bin"];
const HASHES: [&str; 3] = [
    "f8a288569f0d4fbdedaf991c55bb5185a87e25ff4d4f2e9a82984454b9703f82",
    "1b306bf0818fd2d9821ac3ffee8b45eb0a2e5998a52e4f8a92c40423fd2f4ff1",
    "3c67e66908e089a85ad9bb7238c7a63522da2812609463892254646be32b237a",
];
const CRCS: [u32; 3] = [0x31aa_ebb4, 0xa7ce_4143, 0x9b26_f188];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn sha256(bytes: &[u8]) -> String {
    let mut hash = xdzip_crypto::SecretSha256::new();
    hash.update(bytes);
    hex(hash.finish().as_ref())
}
fn known_members() -> [Vec<u8>; 3] {
    let first: Vec<u8> = (0..LENGTHS[0])
        .map(|index| (31 * index + 7 * (index >> 5) + 13) as u8)
        .collect();
    let mut middle = Vec::with_capacity(LENGTHS[1] + 31);
    for counter in 0u32.. {
        let mut hash = xdzip_crypto::SecretSha256::new();
        hash.update(b"xdzip-rar20-store-middle-v1");
        hash.update(&counter.to_le_bytes());
        middle.extend_from_slice(hash.finish().as_ref());
        if middle.len() >= LENGTHS[1] {
            middle.truncate(LENGTHS[1]);
            break;
        }
    }
    let third = [&first[173..], &first[..173]].concat();
    let members = [first, middle, third];
    for (index, bytes) in members.iter().enumerate() {
        assert_eq!(bytes.len(), LENGTHS[index]);
        assert_eq!(sha256(bytes), HASHES[index]);
        assert_eq!(crc32(bytes), CRCS[index]);
    }
    members
}

struct Known<'a> {
    expected: &'a [u8],
    position: usize,
    hash: xdzip_crypto::SecretSha256,
    crc: Crc32,
    work: u64,
    aux: u64,
    work_limit: u64,
    reject_output: bool,
    cancel: Option<(&'a Cancellation, u64)>,
}
impl<'a> Known<'a> {
    fn new(expected: &'a [u8]) -> Self {
        Self {
            expected,
            position: 0,
            hash: xdzip_crypto::SecretSha256::new(),
            crc: Crc32::new(),
            work: 0,
            aux: 0,
            work_limit: u64::MAX,
            reject_output: false,
            cancel: None,
        }
    }
    fn complete(self, member: usize) {
        assert_eq!(self.position, LENGTHS[member]);
        assert_eq!(hex(self.hash.finish().as_ref()), HASHES[member]);
        assert_eq!(self.crc.checksum(), CRCS[member]);
    }
}
impl DecodedSink for Known<'_> {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        assert!(!bytes.is_empty() && bytes.len() <= 8192);
        let end = self.position.checked_add(bytes.len()).unwrap();
        assert_eq!(bytes, self.expected.get(self.position..end).unwrap());
        if self.reject_output {
            return Err(DecodeError::Output);
        }
        self.hash.update(bytes);
        self.crc.update(bytes);
        self.position = end;
        Ok(())
    }
    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        let next = self.work.checked_add(bytes).ok_or(DecodeError::Limit)?;
        if next > self.work_limit {
            return Err(DecodeError::Limit);
        }
        self.work = next;
        if let Some((control, after)) = self.cancel
            && self.work >= after
        {
            control.cancel();
        }
        Ok(())
    }
    fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.aux = self.aux.checked_add(bytes).ok_or(DecodeError::Limit)?;
        Ok(())
    }
}

#[test]
fn official_624_rar29_store_fixture_identity_and_physical_profiles() {
    assert_eq!(ARCHIVE.len(), 197_445);
    assert_eq!(
        sha256(ARCHIVE),
        "7d055696558ef69fa83372ea0d929e042a11ecdbe7415cb7efb3d1418e6b0d38"
    );
    let members = known_members();
    assert_eq!(members.iter().map(Vec::len).sum::<usize>(), 458_755);
    let control = Cancellation::default();
    let index = read_index(
        &mut Cursor::new(ARCHIVE),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert_eq!(
        (
            index.entries().len(),
            index.0.files.len(),
            index.0.parts.len()
        ),
        (3, 3, 3)
    );
    for (member, row) in index.entries().iter().enumerate() {
        assert_eq!(row.id.get(), member as u64 + 1);
        assert_eq!(row.name_bytes(), NAMES[member]);
        assert_eq!(
            (row.size, row.packed_size),
            (LENGTHS[member] as u64, PACKED[member])
        );
        assert_eq!(row.method, if member == 1 { 20 } else { 29 });
        assert!(row.size_known && row.payload_supported());
        assert!(!row.encrypted && !row.directory && !row.link && !row.warning);
        let file = &index.0.files[member];
        assert_eq!(file.parts.len(), 1);
        let part = &index.0.parts[file.parts[0]];
        assert_eq!((part.volume, part.before, part.after), (0, false, false));
        assert_eq!(
            part.description.compression,
            if member == 1 { 0 } else { 3 }
        );
        assert_eq!(part.description.solid, member != 0);
        assert_eq!(part.description.size, Some(LENGTHS[member] as u64));
        assert_eq!(
            u16::from_le_bytes([part.header[3], part.header[4]]),
            if member == 0 { 0x8080 } else { 0x8090 }
        );
        assert_eq!(
            u32::from_le_bytes(part.header[16..20].try_into().unwrap()),
            CRCS[member]
        );
    }
}

#[test]
fn official_624_rar29_store_sequential_reverse_and_repeated_selection_match_every_byte() {
    let members = known_members();
    // These costs are complete decoded mathematical members, including a Store
    // prefix when replay is needed. They do not prescribe compression internals.
    let cases: &[(&[usize], &[u64])] = &[
        (&[0, 1, 2], &[131_071, 0, 131_071]),
        (&[2, 1, 0], &[458_755, 0, 131_071]),
        (&[2, 0, 1, 2, 2], &[458_755, 131_071, 0, 131_071, 458_755]),
    ];
    for (order, expected_work) in cases {
        let control = Cancellation::default();
        let mut source = Cursor::new(ARCHIVE);
        let rows = RarCodec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
        for (&member, &work) in order.iter().zip(*expected_work) {
            let mut sink = Known::new(&members[member]);
            payload.decode(&rows[member], &mut sink, &control).unwrap();
            assert_eq!(sink.work, work, "selection {member} in {order:?}");
            sink.complete(member);
        }
    }
    // Also open a fresh reader for each reversed selection, matching the frozen
    // independent-worker observations instead of relying on one cached instance.
    for member in [2, 1, 0] {
        let control = Cancellation::default();
        let mut source = Cursor::new(ARCHIVE);
        let rows = RarCodec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
        let mut sink = Known::new(&members[member]);
        payload.decode(&rows[member], &mut sink, &control).unwrap();
        assert_eq!(sink.work, [131_071, 0, 458_755][member]);
        sink.complete(member);
    }
}

#[test]
fn official_624_rar29_corrupt_store_prefix_prevents_target_output_and_poison_is_terminal() {
    let members = known_members();
    let control = Cancellation::default();
    let index = read_index(
        &mut Cursor::new(ARCHIVE),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    let store = &index.0.parts[index.0.files[1].parts[0]];
    let mut corrupt = ARCHIVE.to_vec();
    corrupt[usize::try_from(store.data_at).unwrap() + 19] ^= 1;
    let mut source = Cursor::new(corrupt);
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Known::new(&members[2]);
    assert_eq!(
        payload.decode(&rows[2], &mut sink, &control),
        Err(DecodeError::Integrity)
    );
    assert_eq!(sink.position, 0);
    assert_eq!(sink.work, (LENGTHS[0] + LENGTHS[1]) as u64);
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &control),
        Err(DecodeError::Integrity)
    );
    assert_eq!(sink.position, 0);
}

#[test]
fn official_624_rar29_store_replay_respects_output_work_and_cancellation_failures() {
    let members = known_members();
    for mode in 0..3 {
        let control = Cancellation::default();
        let mut source = Cursor::new(ARCHIVE);
        let rows = RarCodec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
        let mut sink = Known::new(&members[2]);
        let expected = match mode {
            0 => {
                sink.reject_output = true;
                DecodeError::Output
            }
            1 => {
                sink.work_limit = 1;
                DecodeError::Limit
            }
            _ => {
                sink.cancel = Some((&control, LENGTHS[0] as u64 + 8192));
                DecodeError::Cancelled
            }
        };
        assert_eq!(payload.decode(&rows[2], &mut sink, &control), Err(expected));
        assert_eq!(sink.position, 0);
        if mode == 0 {
            assert!(sink.work >= (LENGTHS[0] + LENGTHS[1]) as u64);
        } else if mode == 2 {
            assert_eq!(sink.work, LENGTHS[0] as u64 + 8192);
        }
        assert_eq!(
            payload.decode(&rows[0], &mut sink, &control),
            Err(DecodeError::Integrity)
        );
        assert_eq!(sink.position, 0);
    }
}

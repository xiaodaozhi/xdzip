use super::*;
use std::io::Cursor;
use xdzip_codec_api::Format;
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/deb")
            .join(name),
    )
    .unwrap()
}
#[derive(Default)]
struct Sink(Vec<u8>);
impl xdzip_codec_api::payload::DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}
#[test]
fn all_allowed_combinations_export_three_views_with_exact_payloads() {
    let cancel = Cancellation::default();
    for control in ["none", "gz", "xz", "zst"] {
        for data in ["none", "gz", "xz", "zst", "bz2", "lzma"] {
            let bytes = fixture(&format!("control-{control}-data-{data}.deb"));
            let mut source = Cursor::new(&bytes);
            let rows = DebCodec
                .scan(&mut source, &cancel, ScanLimits::default())
                .unwrap();
            assert_eq!(rows.len(), 9);
            assert!(rows.iter().all(|row| row.format == Format::Deb
                && row.payload_supported()
                && row.unix_mode == 0));
            let mut reader = DebCodec.open_payload(&mut source, &rows, &cancel).unwrap();
            for (at, row) in rows.iter().enumerate() {
                assert_eq!(row.id.get(), at as u64 + 1);
                let mut sink = Sink::default();
                reader.decode(row, &mut sink, &cancel).unwrap();
                assert_eq!(sink.0.len() as u64, row.size);
                match row.display_name.as_str() {
                    "members/debian-binary" => assert_eq!(sink.0, b"2.0\n"),
                    "data/usr/share/xdzip/hello.txt" => {
                        assert_eq!(sink.0, b"abracadabra abracadabra\n".repeat(6))
                    }
                    "data/usr/share/xdzip/资料.txt" => {
                        assert_eq!(sink.0, b"owned UTF-8 content\n")
                    }
                    "data/usr/share/xdzip/empty.bin" => assert!(sink.0.is_empty()),
                    name if name.starts_with("members/") => {
                        let envelope = envelope::read_envelope(
                            &mut Cursor::new(&bytes),
                            &cancel,
                            ScanLimits::default(),
                        )
                        .unwrap();
                        let member = envelope
                            .members
                            .iter()
                            .find(|m| format!("members/{}", m.name) == name)
                            .unwrap();
                        assert_eq!(
                            sink.0,
                            bytes[member.offset as usize..(member.offset + member.size) as usize]
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}
#[test]
fn extension_and_duplicate_identities_remain_physical_and_cannot_be_substituted() {
    let cancel = Cancellation::default();
    for (name, count) in [("extensions.deb", 12), ("duplicates.deb", 10)] {
        let mut source = Cursor::new(fixture(name));
        let rows = DebCodec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), count);
        let mut reader = DebCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut hello = Vec::new();
        for row in &rows {
            let mut sink = Sink::default();
            reader.decode(row, &mut sink, &cancel).unwrap();
            if row.display_name.ends_with("hello.txt") {
                hello.push(sink.0);
            }
        }
        if name == "duplicates.deb" {
            assert_eq!(
                hello,
                [
                    b"abracadabra abracadabra\n".repeat(6),
                    b"second physical record\n".to_vec()
                ]
            );
        }
        drop(reader);
        let mut altered = rows.clone();
        altered[0].size += 1;
        assert_eq!(
            DebCodec.open_payload(&mut source, &altered, &cancel).err(),
            Some(DecodeError::Integrity)
        );
        let mut reader = DebCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut forged = rows[0].clone();
        forged.raw_name = b"data/forged".to_vec();
        assert_eq!(
            reader.decode(&forged, &mut Sink::default(), &cancel),
            Err(DecodeError::Integrity)
        );
        assert_eq!(
            reader.decode(&rows[0], &mut Sink::default(), &cancel),
            Err(DecodeError::Integrity)
        );
    }
}

fn ar_member(output: &mut Vec<u8>, name: &str, bytes: &[u8]) {
    output.extend_from_slice(
        format!(
            "{name:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
            1700000000,
            0,
            0,
            "100644",
            bytes.len()
        )
        .as_bytes(),
    );
    output.extend_from_slice(bytes);
    if bytes.len() & 1 != 0 {
        output.push(b'\n');
    }
}
fn package(control: (&str, &[u8]), data: (&str, &[u8])) -> Vec<u8> {
    let mut bytes = b"!<arch>\n".to_vec();
    ar_member(&mut bytes, "debian-binary", b"2.0\n");
    ar_member(&mut bytes, control.0, control.1);
    ar_member(&mut bytes, data.0, data.1);
    bytes
}
fn inner_bytes(bytes: &[u8], control: bool) -> Vec<u8> {
    let envelope = envelope::read_envelope(
        &mut Cursor::new(bytes),
        &Cancellation::default(),
        ScanLimits::default(),
    )
    .unwrap();
    let member = &envelope.members[if control {
        envelope.control
    } else {
        envelope.data
    }];
    bytes[member.offset as usize..(member.offset + member.size) as usize].to_vec()
}
fn checksum(header: &mut [u8]) {
    header[148..156].fill(b' ');
    let sum: u32 = header.iter().map(|b| u32::from(*b)).sum();
    header[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
}
#[test]
fn full_inner_checks_damaged_tar_and_required_member_profiles_cannot_publish_rows() {
    let cancel = Cancellation::default();
    for (name, group) in [
        ("control-gz-data-gz.deb", 0),
        ("control-gz-data-gz.deb", 1),
        ("control-xz-data-xz.deb", 1),
        ("control-zst-data-zst.deb", 1),
    ] {
        let mut bytes = fixture(name);
        let envelope =
            envelope::read_envelope(&mut Cursor::new(&bytes), &cancel, ScanLimits::default())
                .unwrap();
        let member = &envelope.members[if group == 0 {
            envelope.control
        } else {
            envelope.data
        }];
        bytes[(member.offset + member.size - 1) as usize] ^= 0x80;
        assert!(
            DebCodec
                .scan(&mut Cursor::new(bytes), &cancel, ScanLimits::default())
                .is_err(),
            "{name} group {group}"
        );
    }
    let valid = fixture("control-none-data-none.deb");
    let control = inner_bytes(&valid, true);
    let mut data = inner_bytes(&valid, false);
    data[512] ^= 1;
    assert_eq!(
        DebCodec
            .scan(
                &mut Cursor::new(package(("control.tar", &control), ("data.tar", &data))),
                &cancel,
                ScanLimits::default()
            )
            .err(),
        Some(ScanError::Invalid)
    );
    for (c, d) in [
        ("control.tar.bz2", "data.tar"),
        ("control.tar", "data.tar.lz"),
        ("data.tar", "control.tar"),
        ("control.tar", "control.tar"),
    ] {
        assert_eq!(
            DebCodec
                .scan(
                    &mut Cursor::new(package((c, &control), (d, &data))),
                    &cancel,
                    ScanLimits::default()
                )
                .err(),
            Some(ScanError::Unsupported)
        );
    }
    let mut thin = valid;
    thin[..8].copy_from_slice(b"!<thin>\n");
    assert_eq!(
        DebCodec
            .scan(&mut Cursor::new(thin), &cancel, ScanLimits::default())
            .err(),
        Some(ScanError::Unsupported)
    );
}
#[test]
fn exact_combined_decoded_work_entry_and_name_limits_apply_to_the_whole_index() {
    let cancel = Cancellation::default();
    let bytes = fixture("control-gz-data-zst.deb");
    let mut limits = ScanLimits {
        read_bytes: 20480,
        entries: 9,
        ..ScanLimits::default()
    };
    let rows = DebCodec
        .scan(&mut Cursor::new(&bytes), &cancel, limits)
        .unwrap();
    limits.total_name_bytes = rows.iter().map(|row| row.raw_name.len()).sum();
    limits.display_name_bytes = rows
        .iter()
        .map(|row| row.display_name.len() + row.decoded_name.as_ref().map_or(0, String::len))
        .sum();
    DebCodec
        .scan(&mut Cursor::new(&bytes), &cancel, limits)
        .unwrap();
    for limited in [
        ScanLimits {
            read_bytes: 20479,
            ..limits
        },
        ScanLimits {
            entries: 8,
            ..limits
        },
        ScanLimits {
            total_name_bytes: limits.total_name_bytes - 1,
            ..limits
        },
        ScanLimits {
            display_name_bytes: limits.display_name_bytes - 1,
            ..limits
        },
        ScanLimits {
            name_bytes: 8,
            ..limits
        },
    ] {
        assert_eq!(
            DebCodec
                .scan(&mut Cursor::new(&bytes), &cancel, limited)
                .err(),
            Some(ScanError::Limit)
        );
    }
}
#[test]
fn unsafe_original_names_links_and_special_types_remain_unsupported_after_prefixing() {
    let cancel = Cancellation::default();
    let valid = fixture("control-none-data-none.deb");
    let control = inner_bytes(&valid, true);
    let original = inner_bytes(&valid, false);
    for (name, kind) in [
        ("/absolute.txt", b'0'),
        ("../escape.txt", b'0'),
        ("safe/link", b'2'),
        ("safe/device", b'3'),
    ] {
        let mut data = original.clone();
        let header = &mut data[1024..1536];
        header[..100].fill(0);
        header[..name.len()].copy_from_slice(name.as_bytes());
        header[156] = kind;
        checksum(header);
        let mut source = Cursor::new(package(("control.tar", &control), ("data.tar", &data)));
        let rows = DebCodec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        let row = rows
            .iter()
            .find(|row| row.display_name == format!("data/{name}"))
            .unwrap();
        assert!(row.warning && !row.payload_supported());
        let mut reader = DebCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        assert_eq!(
            reader.decode(row, &mut Sink::default(), &cancel),
            Err(DecodeError::Unsupported)
        );
    }
}
#[test]
fn failed_output_cancellation_and_changed_inner_snapshot_poison_the_reader() {
    struct Refuse;
    impl xdzip_codec_api::payload::DecodedSink for Refuse {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            Err(DecodeError::Output)
        }
    }
    let cancel = Cancellation::default();
    let mut source = Cursor::new(fixture("control-gz-data-zst.deb"));
    let rows = DebCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut reader = DebCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    assert_eq!(
        reader.decode(&rows[0], &mut Refuse, &cancel),
        Err(DecodeError::Output)
    );
    assert_eq!(
        reader.decode(&rows[0], &mut Sink::default(), &cancel),
        Err(DecodeError::Output)
    );
    drop(reader);
    let mut altered = rows.clone();
    altered.last_mut().unwrap().size += 1;
    assert_eq!(
        DebCodec.open_payload(&mut source, &altered, &cancel).err(),
        Some(DecodeError::Integrity)
    );
    let mut reader = DebCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    cancel.cancel();
    assert_eq!(
        reader.decode(&rows[0], &mut Sink::default(), &cancel),
        Err(DecodeError::Cancelled)
    );
    assert_eq!(
        reader.decode(&rows[0], &mut Sink::default(), &Cancellation::default()),
        Err(DecodeError::Cancelled)
    );
}
#[test]
fn many_inner_records_use_retained_readers_with_linear_source_work() {
    use std::io::{Read, Seek, SeekFrom};
    struct Counted {
        cursor: Cursor<Vec<u8>>,
        read: Rc<Cell<u64>>,
    }
    impl Read for Counted {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            let n = self.cursor.read(bytes)?;
            self.read.set(self.read.get() + n as u64);
            Ok(n)
        }
    }
    impl Seek for Counted {
        fn seek(&mut self, offset: SeekFrom) -> std::io::Result<u64> {
            self.cursor.seek(offset)
        }
    }
    impl ReadSource for Counted {}
    let cancel = Cancellation::default();
    let valid = fixture("control-none-data-none.deb");
    let control = inner_bytes(&valid, true);
    let original = inner_bytes(&valid, false);
    let mut data = Vec::new();
    for at in 0..1000 {
        let mut record = original[1024..2048].to_vec();
        let header = &mut record[..512];
        header[..100].fill(0);
        let name = format!("many/{at:04}.txt");
        header[..name.len()].copy_from_slice(name.as_bytes());
        checksum(header);
        data.extend(record);
    }
    data.extend([0; 1024]);
    let bytes = package(("control.tar", &control), ("data.tar", &data));
    let length = bytes.len() as u64;
    let read = Rc::new(Cell::new(0));
    let mut source = Counted {
        cursor: Cursor::new(bytes),
        read: read.clone(),
    };
    let rows = DebCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    read.set(0);
    let mut reader = DebCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut count = 0;
    for row in rows
        .iter()
        .filter(|row| row.display_name.starts_with("data/"))
    {
        let mut sink = Sink::default();
        reader.decode(row, &mut sink, &cancel).unwrap();
        assert_eq!(sink.0, b"abracadabra abracadabra\n".repeat(6));
        count += 1;
    }
    assert_eq!(count, 1000);
    assert!(
        read.get() < length * 3,
        "{} actual reads for {length} input bytes",
        read.get()
    );
}

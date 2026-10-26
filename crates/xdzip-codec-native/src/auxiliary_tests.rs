use super::*;

#[derive(Default)]
struct Sink {
    auxiliary: u64,
    decoded: u64,
    output: u64,
    reject_auxiliary: bool,
}
impl DecodedSink for Sink {
    fn charge_auxiliary(&mut self, count: u64) -> Result<(), DecodeError> {
        if self.reject_auxiliary {
            return Err(DecodeError::Limit);
        }
        self.auxiliary += count;
        Ok(())
    }
    fn charge_skipped(&mut self, count: u64) -> Result<(), DecodeError> {
        self.decoded += count;
        Ok(())
    }
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.decoded += bytes.len() as u64;
        self.output += bytes.len() as u64;
        Ok(())
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.output += bytes.len() as u64;
        Ok(())
    }
}

#[test]
fn auxiliary_packets_are_rar_payload_only_bounded_and_do_not_fund_output() {
    let mut work = AuxiliaryWork::default();
    let mut sink = Sink::default();
    let packet = (protocol::CHUNK as u64).to_le_bytes();
    work.packet(Format::Rar, &mut Fields(&packet), &mut sink)
        .unwrap();
    assert_eq!(
        (work.0, sink.auxiliary, sink.decoded, sink.output),
        (65536, 65536, 0, 0)
    );
    let mut credit = WorkerCredit::default();
    let mut written = 0;
    assert_eq!(
        credit.data(Format::Cab, &mut sink, b"x", true, &mut written, 1),
        Err(DecodeError::Integrity)
    );
    assert_eq!((credit.0, sink.output), (0, 0));
    for format in [
        Format::Zip,
        Format::Tar,
        Format::TarGzip,
        Format::SevenZip,
        Format::Cab,
        Format::Iso,
        Format::Udf,
    ] {
        assert_eq!(
            work.packet(format, &mut Fields(&packet), &mut sink),
            Err(DecodeError::Integrity)
        );
    }
    for bytes in [
        &[][..],
        &[0; 7][..],
        &[0; 8][..],
        &[1, 0, 0, 0, 0, 0, 0, 0, 99][..],
    ] {
        assert_eq!(
            work.packet(Format::Rar, &mut Fields(bytes), &mut sink),
            Err(DecodeError::Integrity)
        );
    }
    let excessive = (protocol::CHUNK as u64 + 1).to_le_bytes();
    assert_eq!(
        work.packet(Format::Rar, &mut Fields(&excessive), &mut sink),
        Err(DecodeError::Limit)
    );
    assert_eq!((work.0, sink.auxiliary), (65536, 65536));
}

#[test]
fn auxiliary_actual_repeated_packets_hit_separate_host_limit_without_output() {
    let mut work = AuxiliaryWork::default();
    let mut sink = Sink::default();
    let packet = (protocol::CHUNK as u64).to_le_bytes();
    for _ in 0..AUXILIARY_LIMIT / protocol::CHUNK as u64 {
        work.packet(Format::Rar, &mut Fields(&packet), &mut sink)
            .unwrap();
    }
    assert_eq!(
        work.packet(Format::Rar, &mut Fields(&packet), &mut sink),
        Err(DecodeError::Limit)
    );
    assert_eq!(
        (work.0, sink.auxiliary, sink.decoded, sink.output),
        (AUXILIARY_LIMIT, AUXILIARY_LIMIT, 0, 0)
    );
}

#[test]
fn auxiliary_actual_work_survives_sink_failure_and_worker_credit_reset() {
    let mut work = AuxiliaryWork::default();
    let mut sink = Sink {
        reject_auxiliary: true,
        ..Sink::default()
    };
    let packet = 7u64.to_le_bytes();
    assert_eq!(
        work.packet(Format::Rar, &mut Fields(&packet), &mut sink),
        Err(DecodeError::Limit)
    );
    assert_eq!(work.0, 7);
    let mut credit = WorkerCredit::default();
    credit.reset(); // restarting or changing a password resets only delivery credit
    sink.reject_auxiliary = false;
    work.packet(Format::Rar, &mut Fields(&packet), &mut sink)
        .unwrap();
    assert_eq!((work.0, sink.auxiliary, credit.0), (14, 7, 0));
}

#[test]
fn rar_precharged_data_requires_decoded_work_and_cannot_spend_auxiliary_work() {
    let mut credit = WorkerCredit::default();
    let mut work = AuxiliaryWork::default();
    let mut sink = Sink::default();
    let mut written = 0;
    work.packet(Format::Rar, &mut Fields(&4u64.to_le_bytes()), &mut sink)
        .unwrap();
    assert_eq!(
        credit.data(Format::Rar, &mut sink, b"data", true, &mut written, 4),
        Err(DecodeError::Integrity)
    );
    credit.work(Format::Rar, &mut sink, 4).unwrap();
    credit
        .data(Format::Rar, &mut sink, b"data", true, &mut written, 4)
        .unwrap();
    assert_eq!((credit.0, work.0, written), (0, 4, 4));
    assert_eq!(
        credit.data(Format::Rar, &mut sink, b"data", true, &mut written, 8),
        Err(DecodeError::Integrity)
    );
    credit.work(Format::Rar, &mut sink, 4).unwrap();
    credit.reset();
    assert_eq!(
        credit.data(Format::Rar, &mut sink, b"data", true, &mut written, 8),
        Err(DecodeError::Integrity)
    );
    assert_eq!(
        work.0, 4,
        "worker restart clears data credit, never task auxiliary work"
    );
}

fn protocol_source(mode: u8) -> Cursor<Vec<u8>> {
    let mut bytes = b"Rar!\x1a\x07\x00".to_vec();
    bytes.push(mode);
    Cursor::new(bytes)
}

#[test]
#[ignore = "requires the authored auxiliary protocol worker, not an archive decoder"]
fn auxiliary_protocol_worker_scope_frames_credit_and_password_restart() {
    assert_eq!(
        std::env::var("XDZIP_AUXILIARY_PROTOCOL_FIXTURE").as_deref(),
        Ok("1")
    );
    let codec = NativeCodec {
        format: Format::Rar,
    };
    let control = Cancellation::default();
    for (mode, expected) in [
        (b'v', Ok(())),
        (b'e', Err(DecodeError::Integrity)),
        (b'z', Err(DecodeError::Integrity)),
        (b'o', Err(DecodeError::Limit)),
        (b't', Err(DecodeError::Integrity)),
        (b'p', Err(DecodeError::Integrity)),
    ] {
        let mut source = protocol_source(mode);
        let rows = codec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        let mut reader = codec.open_payload(&mut source, &rows, &control).unwrap();
        let mut sink = Sink::default();
        assert_eq!(
            reader.decode(&rows[0], &mut sink, &control),
            expected,
            "mode {mode}"
        );
        if mode == b'v' {
            assert_eq!(
                (sink.auxiliary, sink.decoded, sink.output),
                (81 * 65536, 87, 87)
            );
        } else {
            assert_eq!(sink.output, 0);
        }
    }
    let mut source = protocol_source(b's');
    assert_eq!(
        codec.scan(&mut source, &control, ScanLimits::default()),
        Err(ScanError::Invalid)
    );
    for format in [Format::SevenZip, Format::Iso] {
        let other = NativeCodec { format };
        let mut source = protocol_source(b'f');
        let rows = other
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        let mut reader = other.open_payload(&mut source, &rows, &control).unwrap();
        assert_eq!(
            reader.decode(&rows[0], &mut Sink::default(), &control),
            Err(DecodeError::Integrity)
        );
    }
    let mut source = protocol_source(b'r');
    let rows = codec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut reader = codec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        reader.decode(&rows[0], &mut sink, &control),
        Err(DecodeError::WrongPassword)
    );
    assert_eq!(
        (sink.auxiliary, sink.decoded, sink.output),
        (81 * 65536, 0, 0)
    );
    reader
        .decode_with_password(&rows[0], &mut sink, &control, Some("retry"))
        .unwrap();
    assert_eq!(
        (sink.auxiliary, sink.decoded, sink.output),
        (162 * 65536, 87, 87)
    );
}

#[test]
#[ignore = "327681 real AUX_WORK packets through the authored worker and ACK channel"]
fn auxiliary_protocol_worker_repeated_packets_hit_host_limit() {
    assert_eq!(
        std::env::var("XDZIP_AUXILIARY_PROTOCOL_FIXTURE").as_deref(),
        Ok("1")
    );
    let codec = NativeCodec {
        format: Format::Rar,
    };
    let control = Cancellation::default();
    let mut source = protocol_source(b'q');
    let rows = codec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut reader = codec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        reader.decode(&rows[0], &mut sink, &control),
        Err(DecodeError::Limit)
    );
    assert_eq!(
        (sink.auxiliary, sink.decoded, sink.output),
        (AUXILIARY_LIMIT, 0, 0)
    );
}

#[test]
#[ignore = "real authored metadata worker checks the RAR-only unknown-size flag and early refusal"]
fn unknown_size_protocol_never_becomes_an_empty_decodable_file() {
    assert_eq!(
        std::env::var("XDZIP_AUXILIARY_PROTOCOL_FIXTURE").as_deref(),
        Ok("1")
    );
    let control = Cancellation::default();
    let codec = NativeCodec {
        format: Format::Rar,
    };
    let mut source = protocol_source(b'u');
    let rows = codec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert_eq!((rows[0].size_known, rows[0].size), (false, 0));
    let mut reader = codec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        reader.decode(&rows[0], &mut sink, &control),
        Err(DecodeError::Unsupported)
    );
    assert_eq!((sink.auxiliary, sink.decoded, sink.output), (0, 0, 0));
    for mode in *b"nw" {
        assert_eq!(
            codec.scan(&mut protocol_source(mode), &control, ScanLimits::default()),
            Err(ScanError::Invalid)
        );
    }
    for format in [Format::SevenZip, Format::Iso, Format::Udf] {
        assert_eq!(
            NativeCodec { format }.scan(
                &mut protocol_source(b'u'),
                &control,
                ScanLimits::default()
            ),
            Err(ScanError::Invalid)
        );
    }
}

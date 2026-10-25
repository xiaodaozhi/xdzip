use super::*;
use std::io::Cursor;

#[test]
fn independent_four_port_math_vectors_and_one_byte_delivery() {
    let cases: [([&[u8]; 4], &[u8]); 5] = [
        ([b"", b"", b"", &[0; 5]], b""),
        ([&[0xe8], b"", b"", &[0; 5]], &[0xe8]),
        (
            [&[0xe8], &[0, 0, 0, 5], b"", &[0, 0x7f, 0xff, 0xfc, 0]],
            &[0xe8, 0, 0, 0, 0],
        ),
        (
            [&[0xe9], b"", &[0, 0, 0, 5], &[0, 0x7f, 0xff, 0xfc, 0]],
            &[0xe9, 0, 0, 0, 0],
        ),
        (
            [&[0x0f, 0x80], b"", &[0, 0, 0, 6], &[0, 0x7f, 0xff, 0xfc, 0]],
            &[0x0f, 0x80, 0, 0, 0, 0],
        ),
    ];
    for (inputs, expected) in cases {
        for chunk in 1..=11 {
            let control = Cancellation::default();
            let mut reader = Bcj2Reader::new(
                inputs.map(Cursor::new),
                inputs.map(|n| n.len() as u64),
                expected.len() as u64,
                &control,
            )
            .unwrap();
            let mut actual = Vec::new();
            let mut bytes = vec![0; chunk];
            loop {
                let n = reader.read(&mut bytes).unwrap();
                if n == 0 {
                    break;
                }
                actual.extend_from_slice(&bytes[..n]);
            }
            reader.finish().unwrap();
            assert_eq!(actual, expected);
            assert_eq!(reader.consumed_bytes(), inputs.map(|n| n.len() as u64));
        }
    }
}

#[test]
fn exact_streams_sizes_and_early_finish_are_independent_requirements() {
    let control = Cancellation::default();
    for rc in [
        &[1, 0, 0, 0, 0][..],
        &[0, 255, 255, 255, 255],
        &[0, 0, 0, 0, 0, 0],
    ] {
        let inputs = [&b""[..], &b""[..], &b""[..], rc];
        let mut r = Bcj2Reader::new(
            inputs.map(Cursor::new),
            inputs.map(|n| n.len() as u64),
            0,
            &control,
        )
        .unwrap();
        assert!(r.finish().is_err());
        assert!(r.read(&mut [0]).is_err());
    }
    let inputs: [&[u8]; 4] = [b"abc", b"", b"", &[0; 5]];
    let mut r = Bcj2Reader::new(
        inputs.map(Cursor::new),
        inputs.map(|n| n.len() as u64),
        3,
        &control,
    )
    .unwrap();
    assert!(r.finish().is_err());
    assert!(
        Bcj2Reader::new(
            inputs.map(Cursor::new),
            inputs.map(|n| n.len() as u64),
            2,
            &control
        )
        .is_err()
    );
    let inputs: [&[u8]; 4] = [&[0xe8], b"", b"", &[0, 0x7f, 0xff, 0xfc, 0]];
    let mut r = Bcj2Reader::new(
        inputs.map(Cursor::new),
        inputs.map(|n| n.len() as u64),
        1,
        &control,
    )
    .unwrap();
    assert!(r.read(&mut [0; 4]).is_err());
}

struct FaultInput {
    data: Cursor<Vec<u8>>,
    error_after: u64,
    calls: std::rc::Rc<std::cell::Cell<usize>>,
    kind: io::ErrorKind,
    bare: bool,
    emitted: bool,
}
impl Read for FaultInput {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.calls.set(self.calls.get() + 1);
        if !self.emitted && self.data.position() >= self.error_after {
            self.emitted = true;
            return if self.bare {
                Err(io::Error::from(io::ErrorKind::Interrupted))
            } else {
                Err(io::Error::new(
                    self.kind,
                    xdzip_codec_api::ScanError::SourceChanged,
                ))
            };
        }
        let available = self.error_after.saturating_sub(self.data.position());
        let count = if !self.emitted {
            bytes.len().min(available as usize)
        } else {
            bytes.len()
        };
        self.data.read(&mut bytes[..count])
    }
}
#[test]
fn typed_source_errors_on_each_port_survive_standard_io_and_poison() {
    let call_inputs: [&[u8]; 4] = [&[0xe8], &[0, 0, 0, 5], &[], &[0, 0x7f, 0xff, 0xfc, 0]];
    let jump_inputs: [&[u8]; 4] = [&[0xe9], &[], &[0, 0, 0, 5], &[0, 0x7f, 0xff, 0xfc, 0]];
    for port in 0..4 {
        for kind in [io::ErrorKind::Other, io::ErrorKind::Interrupted] {
            let inputs = if port == 2 { jump_inputs } else { call_inputs };
            let calls = std::rc::Rc::new(std::cell::Cell::new(0));
            let sources = std::array::from_fn(|index| FaultInput {
                data: Cursor::new(inputs[index].to_vec()),
                error_after: if index == port { 0 } else { u64::MAX },
                calls: if index == port {
                    calls.clone()
                } else {
                    Default::default()
                },
                kind,
                bare: false,
                emitted: false,
            });
            let control = Cancellation::default();
            let mut reader =
                Bcj2Reader::new(sources, inputs.map(|n| n.len() as u64), 5, &control).unwrap();
            let error = reader.read_to_end(&mut Vec::new()).unwrap_err();
            assert!(matches!(
                error
                    .get_ref()
                    .and_then(|e| e.downcast_ref::<xdzip_codec_api::ScanError>()),
                Some(xdzip_codec_api::ScanError::SourceChanged)
            ));
            assert_eq!(calls.get(), 1);
            assert!(reader.read(&mut [0]).is_err());
            assert_eq!(calls.get(), 1);
        }
    }
    let control = Cancellation::default();
    let inputs: [&[u8]; 4] = [b"abc", &[], &[], &[0; 5]];
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    let sources = std::array::from_fn(|index| FaultInput {
        data: Cursor::new(inputs[index].to_vec()),
        error_after: if index == 0 { 0 } else { u64::MAX },
        calls: if index == 0 {
            calls.clone()
        } else {
            Default::default()
        },
        kind: io::ErrorKind::Interrupted,
        bare: true,
        emitted: false,
    });
    let mut reader = Bcj2Reader::new(sources, inputs.map(|n| n.len() as u64), 3, &control).unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    reader.finish().unwrap();
    assert_eq!(output, b"abc");
    assert_eq!(calls.get(), 2);
}

#[test]
fn partial_source_failure_and_cancellation_cannot_be_completed() {
    let control = Cancellation::default();
    let inputs = [vec![42; 5000], vec![], vec![], vec![0; 5]];
    let sources = std::array::from_fn(|index| FaultInput {
        data: Cursor::new(inputs[index].clone()),
        error_after: if index == 0 { 4096 } else { u64::MAX },
        calls: Default::default(),
        kind: io::ErrorKind::Interrupted,
        bare: false,
        emitted: false,
    });
    let mut reader = Bcj2Reader::new(
        sources,
        inputs.each_ref().map(|n| n.len() as u64),
        5000,
        &control,
    )
    .unwrap();
    let error = reader.read_to_end(&mut Vec::new()).unwrap_err();
    assert!(matches!(
        error
            .get_ref()
            .and_then(|e| e.downcast_ref::<xdzip_codec_api::ScanError>()),
        Some(xdzip_codec_api::ScanError::SourceChanged)
    ));
    assert!(reader.finish().is_err());
    let inputs: [&[u8]; 4] = [b"abc", &[], &[], &[0; 5]];
    let mut reader = Bcj2Reader::new(
        inputs.map(Cursor::new),
        inputs.map(|n| n.len() as u64),
        3,
        &control,
    )
    .unwrap();
    let mut bytes = [0; 3];
    assert_eq!(reader.read(&mut bytes).unwrap(), 3);
    control.cancel();
    let error = reader.finish().unwrap_err();
    assert!(matches!(
        error
            .get_ref()
            .and_then(|e| e.downcast_ref::<DecodeError>()),
        Some(DecodeError::Cancelled)
    ));
}

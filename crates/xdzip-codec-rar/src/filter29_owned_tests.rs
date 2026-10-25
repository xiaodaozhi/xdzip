use crate::*;
use std::io::Cursor;
use std::path::Path;
use xdzip_codec_api::payload::DecodedSink;

fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut hash = xdzip_crypto::SecretSha256::new();
    hash.update(bytes);
    let mut out = String::new();
    for byte in hash.finish().iter() {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}
fn bounded_read(path: &Path) -> Vec<u8> {
    assert!(path.metadata().unwrap().len() <= 1024 * 1024);
    std::fs::read(path).unwrap()
}
#[derive(Default)]
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if self.0.len() + bytes.len() > 1024 * 1024 {
            return Err(DecodeError::Limit);
        }
        self.0.extend(bytes);
        Ok(())
    }
}

#[test]
#[ignore = "explicit frozen official-author archives and independently known original bytes; no compression encoder"]
fn rar29_official_authored_filter_execution_and_original_bytes() {
    let base = std::env::var_os("XDZIP_RAR29_FILTER_FIXTURES").expect("explicit fixture directory");
    let base = Path::new(&base);
    let cases = [
        (
            "e8",
            "d94857774090d22388d4743c9865ac2dbb60ceacd75cb30f97666e7f4b8d4a03",
            "9ba0077a9b764a2291b2c6e79394cafc505e54200624ee5e062be156668d1f49",
        ),
        (
            "e8e9",
            "79292db7cc870e2a6e0c1d7345331499a11f103aae61a5ba423bb8bec1c5161a",
            "58831d72ef82c4dc3f08e47f17c39affb801004a0feab4c53d2c248145b80f9a",
        ),
        (
            "delta4",
            "151dd99299e9bc3505119b71cca6fdf0c9bba2d93eefc07570e10a665502cc22",
            "b8ecd227124f8d48f2b8309969c4ba16a2e9d5eb54b629beed907a8433528f0c",
        ),
        (
            "audio1",
            "9b8e4472e80f742d5032ae107cc152e8b73569dc5fad6d6fe350e7f80622fbf3",
            "8d91246a25547bd859dd0ebec7624975a613995b9a3237c7854f6077d356db2d",
        ),
        (
            "rgb",
            "6a51e5271faf976749e381694b6a891a91d864149a008437eac1052ab408a0a8",
            "d7acec2bacdc0d0ab8e96f250a5ecd59ee8a0f07f8d514447b9fa600c10da3f4",
        ),
        (
            "itanium-switch-probe",
            "7725f127e2c99e6ce9787ffd5c8fb325c8f5c7c23c7c47ba82d673802ec052e6",
            "6d78cc49d60e5684f020720cc46c29364e8b56e1c453630f66d9c3b1c7aa416d",
        ),
    ];
    for (label, archive_hash, original_hash) in cases {
        let archive = bounded_read(&base.join(format!("{label}.rar")));
        let expected = bounded_read(&base.join(format!("{label}.bin")));
        assert_eq!(digest(&archive), archive_hash, "archive identity {label}");
        assert_eq!(
            digest(&expected),
            original_hash,
            "original identity {label}"
        );
        let control = Cancellation::default();
        let mut source = Cursor::new(archive);
        let rows = RarCodec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].method, 29);
        crate::filter29_transform::take_executed();
        let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
        let mut sink = Sink::default();
        payload.decode(&rows[0], &mut sink, &control).unwrap();
        assert_eq!(sink.0, expected, "complete original bytes {label}");
        let counts = crate::filter29_transform::take_executed();
        println!(
            "RAR29_FILTER_RESULT\t{label}\t{}\t{original_hash}\t{counts:?}",
            expected.len()
        );
    }
}

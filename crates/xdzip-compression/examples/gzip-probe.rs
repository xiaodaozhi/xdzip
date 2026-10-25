//! Synthetic interoperability and timing probe. Output is JSON for build/ artifacts.
//! Reference dependencies are development-only and never part of the application.
use std::io::{Read, Write};
use std::time::Instant;
use xdzip_compression::{GzipReader, GzipWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut seed = 0x517cc1b727220a95u64;
    let random: Vec<u8> = (0..4 * 1024 * 1024)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed as u8
        })
        .collect();
    let samples = [
        ("zeros", vec![0; 16 * 1024 * 1024]),
        (
            "text",
            b"owned compression probe, UTF-8: \xe4\xb8\xad\xe6\x96\x87\n".repeat(100000),
        ),
        ("random", random),
    ];
    let mut records = Vec::new();
    for (label, data) in samples {
        for level in [0, 1, 6, 9] {
            let start = Instant::now();
            let mut encoder = GzipWriter::new(Vec::new(), level)?;
            encoder.write_all(&data)?;
            let compressed = encoder.finish()?;
            let encode_ms = start.elapsed().as_secs_f64() * 1000.0;
            let start = Instant::now();
            let mut decoded = Vec::new();
            GzipReader::new(compressed.as_slice()).read_to_end(&mut decoded)?;
            let decode_ms = start.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(decoded, data);
            let mut oracle = Vec::new();
            flate2::read::MultiGzDecoder::new(compressed.as_slice()).read_to_end(&mut oracle)?;
            assert_eq!(oracle, data);
            let start = Instant::now();
            let mut reference = flate2::write::GzEncoder::new(
                Vec::new(),
                flate2::Compression::new(u32::from(level)),
            );
            reference.write_all(&data)?;
            let reference = reference.finish()?;
            let reference_encode_ms = start.elapsed().as_secs_f64() * 1000.0;
            let start = Instant::now();
            let mut oracle = Vec::new();
            flate2::read::MultiGzDecoder::new(reference.as_slice()).read_to_end(&mut oracle)?;
            let reference_decode_ms = start.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(oracle, data);
            let mut decoded = Vec::new();
            GzipReader::new(reference.as_slice()).read_to_end(&mut decoded)?;
            assert_eq!(decoded, data);
            records.push(format!(
                "{{\"case\":\"{label}\",\"level\":{level},\"input_bytes\":{},\"encoded_bytes\":{},\"reference_bytes\":{},\"encode_ms\":{encode_ms:.3},\"decode_ms\":{decode_ms:.3},\"reference_encode_ms\":{reference_encode_ms:.3},\"reference_decode_ms\":{reference_decode_ms:.3},\"verified\":true}}",
                data.len(), compressed.len(), reference.len()));
        }
    }
    println!(
        "{{\"arch\":\"{}\",\"os\":\"{}\",\"peak_rss\":null,\"cases\":[{}]}}",
        std::env::consts::ARCH,
        std::env::consts::OS,
        records.join(",")
    );
    Ok(())
}

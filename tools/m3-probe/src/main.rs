//! Actual extraction probe into an explicitly supplied empty owned test directory.
use std::io::Read;
use std::path::Path;
use std::time::Instant;
use xdzip_runtime::{
    extraction::{ConflictPolicy, Options},
    service,
};
fn submit(
    client: &service::Client,
    command: impl Fn() -> service::Command,
) -> Result<u64, &'static str> {
    let start = Instant::now();
    loop {
        match client.submit(command()) {
            Ok(id) => return Ok(id),
            Err(service::Rejection::Busy) if start.elapsed().as_secs() < 5 => {
                std::thread::yield_now()
            }
            _ => return Err("admission failed"),
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--native-temp-base"] {
        // Measurement fixtures use the same platform location as Rust/Qt
        // filesystem regressions; Python may choose a different shared CI TMP.
        let base = std::env::temp_dir();
        println!("{}", base.to_str().ok_or("non-UTF-8 temporary base")?);
        return Ok(());
    }
    if args.len() != 2 {
        return Err("provide owned ZIP and empty output directory".into());
    }
    // Preserve the selected object for the nofollow grant; canonicalize would
    // erase a directory link before the output layer can reject it.
    let target = std::path::absolute(Path::new(&args[1]))?;
    if target.read_dir()?.next().is_some() {
        return Err("output must be empty".into());
    }
    let (client, mut receiver) = service::start()?;
    let total_start = Instant::now();
    let open = submit(&client, || service::Command::Open(args[0].clone()))?;
    let mut session = 0;
    loop {
        let event = receiver.receive().ok_or("early disconnect")?;
        if let service::Payload::Page { session: id, .. } = event.payload {
            session = id;
        }
        if event.request == open
            && let service::Payload::Finished(error) = event.payload
        {
            if !error.is_empty() {
                return Err(error.into());
            }
            break;
        }
    }
    let open_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    submit(&client, || service::Command::Extract {
        session,
        options: Options {
            target: target.to_string_lossy().into(),
            all: true,
            selected: vec![],
            directory: String::new(),
            create_folder: false,
            conflict: ConflictPolicy::Ask,
            allow_high_ratio: false,
        },
    })?;
    let mut events = 0;
    let result = loop {
        let event = receiver.receive().ok_or("early disconnect")?;
        if let service::Payload::Task(snapshot) = event.payload {
            events += 1;
            if snapshot.status.terminal() {
                break snapshot;
            }
        } else if let service::Payload::Finished(error) = event.payload
            && !error.is_empty()
        {
            return Err(error.into());
        }
    };
    let extract_ms = start.elapsed().as_secs_f64() * 1000.0;
    if result.status != xdzip_runtime::extraction::Status::Succeeded || result.cleanup_warning {
        // Stable task codes and counters diagnose native failures without paths.
        return Err(format!(
            "extraction: status={:?} error={} cleanup_warning={} committed_files={} committed_bytes={} created_directories={}",
            result.status, result.error, result.cleanup_warning, result.committed_files,
            result.committed_bytes, result.created_directories
        ).into());
    }
    let shutdown = client.shutdown();
    while let Some(event) = receiver.receive() {
        if matches!(event.payload, service::Payload::ShutdownReady) {
            assert_eq!(event.request, shutdown);
        }
    }
    // Verify actual files through a separate streaming reader, after all worker handles exit.
    let verify_start = Instant::now();
    let mut verified = 0u64;
    for index in 0..64 {
        let mut file = std::fs::File::open(
            target.join(format!("batch-{:02}/payload-{index:03}.bin", index % 8)),
        )?;
        let mut offset = 0usize;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            if buffer[..n]
                .iter()
                .enumerate()
                .any(|(at, byte)| *byte != ((offset + at) % 256) as u8)
            {
                return Err("output mismatch".into());
            }
            offset += n;
        }
        if offset != 512 * 1024 {
            return Err("output length mismatch".into());
        }
        verified += offset as u64;
    }
    if verified != result.committed_bytes || result.committed_files != 64 {
        return Err("counter mismatch".into());
    }
    let verification_ms = verify_start.elapsed().as_secs_f64() * 1000.0;
    let peak = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|line| line.starts_with("VmHWM:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|n| n.parse::<u64>().ok())
        });
    println!(
        "{{\"files\":64,\"output_bytes\":{verified},\"decoded_bytes\":{},\"written_bytes\":{},\"committed_bytes\":{},\"created_directories\":{},\"open_ms\":{open_ms:.3},\"extract_ms\":{extract_ms:.3},\"verification_ms\":{verification_ms:.3},\"delivered_task_snapshots\":{events},\"peak_rss_kib\":{},\"independent_byte_check\":true}}",
        result.decoded_bytes,
        result.written_bytes,
        result.committed_bytes,
        result.created_directories,
        peak.map_or("null".into(), |n| n.to_string())
    );
    Ok(())
}

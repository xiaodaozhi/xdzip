//! Actual whole-archive creation into an explicitly owned verification directory.
use std::io::{self, Read};
use std::path::Path;
use std::time::Instant;
use xdzip_runtime::{creation::Options, extraction::ConflictPolicy, service};
use zeroize::Zeroizing;

fn read_fixture_password(
    mut input: impl Read,
) -> Result<xdzip_runtime::creation::Password, &'static str> {
    // Test-only stdin is bounded before allocation and cleared on every return,
    // including partial I/O failure and invalid UTF-8. No secret enters argv or reports.
    let mut bytes = Zeroizing::new([0u8; 129]);
    let mut length = 0;
    loop {
        match input.read(&mut bytes[length..]) {
            Ok(0) => break,
            Ok(count) => {
                length += count;
                if length > 128 {
                    return Err("invalid_password");
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return Err("invalid_fixture_password_input"),
        }
    }
    let value =
        std::str::from_utf8(&bytes[..length]).map_err(|_| "invalid_fixture_password_input")?;
    xdzip_runtime::creation::Password::new(value.to_owned())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    let fixture_password = args
        .last()
        .is_some_and(|arg| arg == "--fixture-password-stdin");
    if fixture_password {
        args.pop();
    }
    if args == ["--native-temp-base"] {
        println!(
            "{}",
            std::env::temp_dir()
                .to_str()
                .ok_or("non-UTF-8 temporary base")?
        );
        return Ok(());
    }
    if !(5..=6).contains(&args.len()) || Path::new(&args[1]).join(&args[2]).exists() {
        return Err("provide owned input, output directory, new ZIP name, method and level".into());
    }
    let options = Options {
        inputs: vec![args[0].clone()],
        target: args[1].clone(),
        name: args[2].clone(),
        method: args[3].parse()?,
        level: args[4].parse()?,
        conflict: ConflictPolicy::Ask,
        volume_bytes: args.get(5).map_or(Ok(0), |v| v.parse())?,
        solid: false,
        encrypt_headers: false,
        password: fixture_password
            .then(|| read_fixture_password(io::stdin().lock()))
            .transpose()?,
    };
    let (client, mut receiver) = service::start()?;
    let start = Instant::now();
    // The product mailbox is deliberately nonblocking; startup contention is
    // retryable and does not mean the requested archive parameters are invalid.
    loop {
        match client.submit(service::Command::Create(options.clone())) {
            Ok(_) => break,
            Err(service::Rejection::Busy) if start.elapsed().as_secs() < 2 => {
                std::thread::yield_now()
            }
            Err(error) => return Err(format!("admission failed: {error:?}").into()),
        }
    }
    let mut events = 0;
    let mut verification_start = None;
    let result = loop {
        let event = receiver.receive().ok_or("early disconnect")?;
        match event.payload {
            service::Payload::Task(s) => {
                events += 1;
                if s.status == xdzip_runtime::extraction::Status::Verifying {
                    verification_start.get_or_insert_with(Instant::now);
                }
                if s.status.terminal() {
                    break s;
                }
            }
            service::Payload::Finished(error) if !error.is_empty() => return Err(error.into()),
            _ => {}
        }
    };
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    let verification_ms = verification_start.map(|t| t.elapsed().as_secs_f64() * 1000.0);
    client.shutdown();
    while receiver.receive().is_some() {}
    if result.status != xdzip_runtime::extraction::Status::Succeeded || result.cleanup_warning {
        return Err(format!(
            "creation: status={:?} error={} cleanup_warning={}",
            result.status, result.error, result.cleanup_warning
        )
        .into());
    }
    let peak = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmHWM:"))?
                .split_whitespace()
                .nth(1)?
                .parse::<u64>()
                .ok()
        });
    println!(
        "{{\"files\":{},\"directories\":{},\"input_bytes\":{},\"output_bytes\":{},\"create_ms\":{ms:.3},\"verification_and_commit_ms\":{},\"delivered_task_snapshots\":{events},\"peak_rss_kib\":{}}}",
        result.committed_files,
        result.created_directories,
        result.committed_bytes,
        result.written_bytes,
        verification_ms.map_or("null".into(), |n| format!("{n:.3}")),
        peak.map_or("null".into(), |n| n.to_string())
    );
    Ok(())
}

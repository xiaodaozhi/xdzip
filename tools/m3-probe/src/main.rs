//! Actual extraction probe into an explicitly supplied empty owned test directory.
use std::io::Read;
use std::path::Path;
use std::time::Instant;
use xdzip_runtime::{
    extraction::{ConflictPolicy, MAX_PASSWORD_BYTES, Options, Password},
    service,
};
use zeroize::Zeroizing;

fn read_fixture_password(mut input: impl Read) -> Result<Password, &'static str> {
    // A fixed zeroizing buffer exists before the first read, so malformed UTF-8,
    // partial I/O failures and overlong input never free an uncleared prefix.
    // One extra byte detects oversize without buffering the remaining stream.
    let mut bytes = Zeroizing::new([0u8; MAX_PASSWORD_BYTES + 1]);
    let mut length = 0;
    loop {
        match input.read(&mut bytes[length..]) {
            Ok(0) => break,
            Ok(count) => {
                length += count;
                if length > MAX_PASSWORD_BYTES {
                    return Err("invalid_password");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err("invalid_fixture_password_input"),
        }
    }
    let value =
        std::str::from_utf8(&bytes[..length]).map_err(|_| "invalid_fixture_password_input")?;
    // Password takes ownership of this bounded string and zeroizes its copy;
    // the original fixed buffer is independently zeroized on every return.
    Password::new(value.to_owned())
}

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
    let flags = args.get(3..).unwrap_or_default();
    let generic = args.get(2).is_some_and(|arg| arg == "--generic")
        && flags.len() <= 2
        && flags.iter().all(|flag| {
            matches!(
                flag.as_str(),
                "--fixture-password-stdin" | "--fixture-allow-high-ratio"
            )
        })
        && flags
            .iter()
            .filter(|flag| flag.as_str() == "--fixture-password-stdin")
            .count()
            <= 1
        && flags
            .iter()
            .filter(|flag| flag.as_str() == "--fixture-allow-high-ratio")
            .count()
            <= 1;
    let fixture_password_mode =
        generic && flags.iter().any(|flag| flag == "--fixture-password-stdin");
    // Explicit consent for bounded, self-authored verification inputs only.
    // The default probe and all product admission remain unchanged.
    let fixture_allow_high_ratio = generic
        && flags
            .iter()
            .any(|flag| flag == "--fixture-allow-high-ratio");
    if args.len() != 2 && !generic {
        return Err("provide owned archive and empty output directory".into());
    }
    // This non-shipping verification probe accepts only an explicit owned-fixture
    // mode. The product has no password command-line/environment input channel.
    // Read bounded zeroizing stdin storage, never argv or a report.
    let fixture_password = if fixture_password_mode {
        Some(read_fixture_password(std::io::stdin().lock())?)
    } else {
        None
    };
    // Preserve the selected object for the nofollow grant; canonicalize would
    // erase a directory link before the output layer can reject it.
    let target = std::path::absolute(Path::new(&args[1]))?;
    if target.read_dir()?.next().is_some() {
        return Err("output must be empty".into());
    }
    let (client, mut receiver) = service::start()?;
    let total_start = Instant::now();
    let mut open = submit(&client, || service::Command::Open(args[0].clone()))?;
    let mut open_password_retries = 0;
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
                if error == "password_required"
                    && open_password_retries == 0
                    && let Some(password) = &fixture_password
                {
                    open = submit(&client, || service::Command::OpenWithPassword {
                        path: args[0].clone(),
                        password: password.clone(),
                    })?;
                    open_password_retries += 1;
                    continue;
                }
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
            allow_high_ratio: fixture_allow_high_ratio,
        },
    })?;
    let mut events = 0;
    let mut password_prompts = 0;
    let result = loop {
        let event = receiver.receive().ok_or("early disconnect")?;
        if let service::Payload::Task(snapshot) = event.payload {
            events += 1;
            if let Some(prompt) = &snapshot.password_prompt {
                let password = fixture_password
                    .as_ref()
                    .ok_or("password_fixture_mode_required")?;
                if prompt.retry || password_prompts != 0 {
                    return Err("fixture_password_rejected".into());
                }
                client
                    .respond_password(
                        snapshot.id,
                        prompt.id,
                        xdzip_runtime::extraction::Password::new(password.expose().to_owned())?,
                    )
                    .map_err(|_| "fixture_password_response_rejected")?;
                password_prompts += 1;
            }
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
    if generic {
        // Independent caller owns the fixture manifest and verifies every output byte.
        println!(
            "{{\"files\":{},\"bytes\":{},\"directories\":{},\"password_prompts\":{password_prompts},\"open_password_retries\":{open_password_retries}}}",
            result.committed_files, result.committed_bytes, result.created_directories
        );
        return Ok(());
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    #[test]
    fn fixture_password_reader_preserves_bounded_utf8_including_empty_and_controls() {
        for value in ["", "\0\t\n", "fixture 密码"] {
            let password = read_fixture_password(Cursor::new(value.as_bytes())).unwrap();
            assert_eq!(password.expose(), value);
            assert_eq!(format!("{password:?}"), "[REDACTED]");
        }
        let maximum = "x".repeat(MAX_PASSWORD_BYTES);
        assert_eq!(
            read_fixture_password(Cursor::new(maximum.as_bytes()))
                .unwrap()
                .expose()
                .len(),
            MAX_PASSWORD_BYTES
        );
    }

    #[test]
    fn malformed_and_overlong_fixture_password_input_is_rejected_with_stable_codes() {
        assert_eq!(
            read_fixture_password(Cursor::new(b"owned-prefix\xff")).unwrap_err(),
            "invalid_fixture_password_input"
        );
        let mut overlong = Cursor::new(vec![b'x'; MAX_PASSWORD_BYTES * 4]);
        assert_eq!(
            read_fixture_password(&mut overlong).unwrap_err(),
            "invalid_password"
        );
        assert_eq!(overlong.position(), (MAX_PASSWORD_BYTES + 1) as u64);
    }

    #[test]
    fn partial_fixture_password_read_failure_never_returns_a_secret() {
        struct FailAfterPrefix(bool);
        impl Read for FailAfterPrefix {
            fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
                if self.0 {
                    return Err(io::Error::other("owned_fixture_failure"));
                }
                self.0 = true;
                bytes[..6].copy_from_slice(b"prefix");
                Ok(6)
            }
        }
        assert_eq!(
            read_fixture_password(FailAfterPrefix(false)).unwrap_err(),
            "invalid_fixture_password_input"
        );
    }
}

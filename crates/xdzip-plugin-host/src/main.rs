//! Explicit SDK trial entry point; never discovers or automatically enables plugins.
use std::{path::Path, sync::atomic::AtomicBool};

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 1 && args[0] == "gate" {
        println!("{}", xdzip_plugin_host::platform_gate());
        return;
    }
    let result = match args.as_slice() {
        [mode, manifest, source] if mode == "scan" => xdzip_plugin_host::run(
            Path::new(manifest),
            Path::new(source),
            None,
            &AtomicBool::new(false),
        ),
        [mode, manifest, source, entry, target] if mode == "read" => entry
            .to_str()
            .and_then(|n| n.parse::<usize>().ok())
            .ok_or(xdzip_plugin_host::Error::Package)
            .and_then(|n| {
                xdzip_plugin_host::run(
                    Path::new(manifest),
                    Path::new(source),
                    Some((n, Path::new(target))),
                    &AtomicBool::new(false),
                )
            }),
        _ => {
            eprintln!(
                "usage: xdzip-plugin-host gate | scan MANIFEST SOURCE | read MANIFEST SOURCE ENTRY_INDEX TARGET"
            );
            std::process::exit(2);
        }
    };
    match result {
        Ok(report) => println!("{}", serde_json::to_string(&report).expect("fixed report")),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

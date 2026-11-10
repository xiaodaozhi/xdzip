//! Qt-independent command-line host for the shared archive task runtime.
mod args;
mod desktop;
mod installation;
mod output;
mod runtime;
mod system;
mod terminal;
mod update;
use clap::ArgMatches;
use output::{Failure, Response, Result};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use xdzip_runtime::{
    creation,
    extraction::{self, ConflictPolicy},
    service::Command,
};

fn strings(matches: &ArgMatches, name: &str) -> Vec<String> {
    matches
        .get_many::<String>(name)
        .map(|v| v.cloned().collect())
        .unwrap_or_default()
}
pub(crate) fn absolute(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = std::path::absolute(path)?;
    if path.to_str().is_none() {
        return Err(Failure::usage("non_utf8_path"));
    }
    Ok(path)
}
fn path_text(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| Failure::usage("non_utf8_path"))
}
fn policy(matches: &ArgMatches) -> ConflictPolicy {
    match matches.get_one::<String>("conflict").map(String::as_str) {
        Some("skip") => ConflictPolicy::Skip,
        Some("rename") => ConflictPolicy::Rename,
        Some("overwrite") => ConflictPolicy::Overwrite,
        _ => ConflictPolicy::Ask,
    }
}
fn volume(value: Option<&String>) -> Result<u64> {
    let Some(value) = value else {
        return Ok(0);
    };
    let (digits, multiplier) = if let Some(n) = value.strip_suffix("GiB") {
        (n, 1024u64.pow(3))
    } else if let Some(n) = value.strip_suffix("MiB") {
        (n, 1024u64.pow(2))
    } else if let Some(n) = value.strip_suffix("KiB") {
        (n, 1024)
    } else {
        (value.as_str(), 1)
    };
    digits
        .parse::<u64>()
        .ok()
        .and_then(|n| n.checked_mul(multiplier))
        .filter(|n| *n > 0 && *n <= 24 * 1024u64.pow(3))
        .ok_or_else(|| Failure::usage("invalid_volume_size"))
}
/// Create only user-requested missing directories through the existing safe grant.
pub(crate) fn ensure_directory(path: &Path) -> Result<PathBuf> {
    let path = absolute(path)?;
    let mut base = path.clone();
    let mut missing = vec![];
    loop {
        match std::fs::symlink_metadata(&base) {
            Ok(metadata) if metadata.is_dir() && !metadata.is_symlink() => break,
            Ok(_) => return Err("unsafe_target".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(base.file_name().ok_or("unsafe_target")?.to_owned());
                if !base.pop() {
                    return Err("unsafe_target".into());
                }
            }
            Err(_) => return Err("output_io".into()),
        }
    }
    // Parent aliases are handled by Grant's native authorization. Missing child
    // components never use create_dir_all or follow pre-existing links.
    let mut grant =
        xdzip_io::Grant::for_creation(&base).map_err(|_| Failure::new("unsafe_target"))?;
    for component in missing.iter().rev() {
        let component = component.to_str().ok_or("non_utf8_path")?;
        let name = xdzip_domain::path::PortablePath::parse(component).ok_or("unsafe_target")?;
        grant = grant
            .subdirectory(&name)
            .map_err(|_| Failure::new("unsafe_target"))?;
    }
    Ok(path)
}

fn execute(name: &str, matches: &ArgMatches, interactive: bool, chinese: bool) -> Result<Response> {
    // Keep the installed payload occupied even for setup/gui handoffs. Update
    // manages its own guard so installation can begin after explicit handoff.
    let _usage = if matches!(name, "version" | "update") {
        None
    } else {
        Some(installation::Usage::acquire()?)
    };
    match name {
        "version" => {
            return Ok(Response::success(
                name,
                output::identity(),
                output::version_text(),
            ));
        }
        "setup" => return installation::setup(matches),
        "gui" => return desktop::launch(&strings(matches, "archives")),
        "update" => return update::run(matches, interactive, chinese),
        _ => {}
    }
    let archive = absolute(
        matches
            .get_one::<String>("archive")
            .ok_or("invalid_options")?,
    )?;
    let password_stdin = matches
        .try_get_one::<bool>("password-stdin")
        .ok()
        .flatten()
        .copied()
        .unwrap_or(false);
    let mut runtime = runtime::Runtime::new(interactive, chinese, password_stdin)?;
    if name != "create" {
        runtime.open(path_text(&archive)?)?;
    }
    match name {
        "list" => runtime.list(),
        "test" => runtime.task(
            name,
            Command::Check {
                session: runtime.session,
            },
        ),
        "extract" => {
            let members = strings(matches, "members");
            let ids = matches
                .get_many::<u64>("entry-id")
                .map(|v| v.copied().collect())
                .unwrap_or_default();
            // Resolution completes before creating even the requested output directory.
            let (selected, _) = runtime.select(members, ids, String::new())?;
            let target =
                ensure_directory(Path::new(matches.get_one::<String>("directory").unwrap()))?;
            runtime.task(
                name,
                Command::Extract {
                    session: runtime.session,
                    options: extraction::Options {
                        target: path_text(&target)?,
                        all: selected.is_empty(),
                        selected,
                        directory: String::new(),
                        create_folder: false,
                        conflict: policy(matches),
                        allow_high_ratio: matches.get_flag("allow-high-ratio"),
                    },
                },
            )
        }
        "add" => {
            let (_, directory) = runtime.select(
                vec![],
                vec![],
                matches.get_one::<String>("into").unwrap().clone(),
            )?;
            let inputs = strings(matches, "inputs")
                .into_iter()
                .map(|p| absolute(p).and_then(|p| path_text(&p)))
                .collect::<Result<Vec<_>>>()?;
            runtime.task(
                name,
                Command::Update {
                    session: runtime.session,
                    options: xdzip_runtime::updating::Options {
                        inputs,
                        directory,
                        conflict: policy(matches),
                    },
                },
            )
        }
        "create" => {
            let name_lower = archive
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or("invalid_archive_name")?
                .to_lowercase();
            let seven = name_lower.ends_with(".7z");
            let tar = name_lower.ends_with(".tar");
            let gzip = name_lower.ends_with(".tar.gz") || name_lower.ends_with(".tgz");
            if !seven && !tar && !gzip && !name_lower.ends_with(".zip") {
                return Err(Failure::usage("unsupported_format"));
            }
            if (matches.get_flag("solid") || matches.get_flag("encrypt-names")) && !seven {
                return Err(Failure::usage("unsupported_options"));
            }
            if (tar || gzip)
                && (matches.get_flag("encrypt")
                    || matches.get_one::<String>("volume-size").is_some())
            {
                return Err(Failure::usage("unsupported_options"));
            }
            let mut password = None;
            if matches.get_flag("encrypt") {
                let mut secret = terminal::password(password_stdin, interactive, chinese)?;
                if !password_stdin {
                    let second = terminal::line(
                        if chinese {
                            "再次输入密码："
                        } else {
                            "Confirm password: "
                        },
                        true,
                        128,
                    )?;
                    if *second != *secret {
                        return Err("password_mismatch".into());
                    }
                }
                password = Some(
                    creation::Password::new(std::mem::take(&mut *secret)).map_err(Failure::from)?,
                );
            }
            let level = matches.get_one::<String>("level").unwrap().as_str();
            let method = if tar || level == "store" && !gzip {
                0
            } else if seven {
                32
            } else {
                8
            };
            let numeric_level = match level {
                "fast" => 1,
                "best" => 9,
                "store" if gzip => 0,
                _ => 6,
            };
            let inputs = strings(matches, "inputs")
                .into_iter()
                .map(|p| absolute(p).and_then(|p| path_text(&p)))
                .collect::<Result<Vec<_>>>()?;
            // Creation's existing-parent requirement is intentional; extraction
            // alone creates its explicitly requested missing destination.
            let target = archive.parent().ok_or("invalid_archive_name")?;
            let filename = archive
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or("invalid_archive_name")?
                .to_owned();
            runtime.task(
                "create",
                Command::Create(creation::Options {
                    inputs,
                    target: path_text(target)?,
                    name: filename,
                    method,
                    level: numeric_level,
                    conflict: policy(matches),
                    volume_bytes: volume(matches.get_one::<String>("volume-size"))?,
                    solid: matches.get_flag("solid"),
                    encrypt_headers: matches.get_flag("encrypt-names"),
                    password,
                }),
            )
        }
        _ => Err(Failure::usage("unknown_command")),
    }
}

/// Run one CLI invocation and return its documented process exit code.
pub fn run() -> i32 {
    let arguments: Vec<OsString> = std::env::args_os().collect();
    let boundary = arguments
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(arguments.len());
    let options = &arguments[..boundary];
    let explicit_language = options.iter().find_map(|arg| {
        arg.to_str()
            .and_then(|value| value.strip_prefix("--language="))
            .map(|value| value == "zh-CN")
    });
    let chinese = explicit_language
        .or_else(|| {
            options
                .windows(2)
                .find(|p| p[0] == "--language")
                .map(|p| p[1] == "zh-CN")
        })
        .unwrap_or_else(|| {
            ["LC_ALL", "LC_MESSAGES", "LANG"]
                .into_iter()
                .find_map(|name| std::env::var(name).ok().filter(|v| !v.is_empty()))
                .is_some_and(|v| v.starts_with("zh"))
        });
    let json_mode = options.iter().any(|arg| arg == "--json");
    let mut command = args::command(chinese);
    let matches = match command.clone().try_get_matches_from(arguments) {
        Ok(matches) => matches,
        Err(error) => {
            if error.kind() == clap::error::ErrorKind::DisplayHelp {
                print!("{error}");
                return 0;
            }
            if json_mode {
                return Response::failure("parse", Failure::usage("invalid_arguments"))
                    .emit(true, chinese);
            }
            eprint!("{error}");
            return 2;
        }
    };
    if matches.get_flag("version-json") {
        println!("{}", output::identity());
        return 0;
    }
    if matches.get_flag("version") {
        if matches.get_flag("json") {
            return Response::success("version", output::identity(), output::version_text())
                .emit(true, chinese);
        }
        println!("{}", output::version_text());
        return 0;
    }
    let Some((name, sub)) = matches.subcommand() else {
        let _ = command.print_long_help();
        println!();
        return 0;
    };
    if let Err(error) = terminal::install() {
        return Response::failure(name, error).emit(matches.get_flag("json"), chinese);
    }
    let interactive = terminal::interactive()
        && !matches.get_flag("json")
        && !matches.get_flag("non-interactive");
    let result = execute(name, sub, interactive, chinese)
        .unwrap_or_else(|error| Response::failure(name, error));
    result.emit(matches.get_flag("json"), chinese)
}

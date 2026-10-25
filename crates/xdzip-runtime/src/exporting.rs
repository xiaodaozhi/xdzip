// crates/xdzip-runtime/src/exporting.rs

//! Native file-promise publication, serialized with other heavy tasks and shutdown.
use crate::{
    creation,
    extraction::{Control, Snapshot, Status, TaskKind},
};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use xdzip_domain::path::PortablePath;
use xdzip_io::{Grant, InputTree};
pub(crate) struct Job {
    pub source: String,
    pub target: String,
}
impl Job {
    pub fn retained_bytes(&self) -> usize {
        self.source.capacity() + self.target.capacity()
    }
}
pub(crate) fn initial(job: &Job, id: u64) -> Snapshot {
    let target = Path::new(&job.target);
    let mut s = creation::initial(
        &creation::Job {
            options: creation::Options {
                inputs: vec![],
                target: target.parent().unwrap_or(target).to_string_lossy().into(),
                name: target
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                method: 8,
                level: 6,
                conflict: crate::extraction::ConflictPolicy::Ask,
                volume_bytes: 0,
                solid: false,
                encrypt_headers: false,
                password: None,
            },
        },
        id,
    );
    s.kind = TaskKind::ExportCopy;
    s.creating = false;
    s.source = job.source.clone();
    s
}
pub(crate) fn run(job: Job, id: u64, control: &Control, publish: &mut dyn FnMut(Snapshot)) {
    let start = Instant::now();
    let mut s = initial(&job, id);
    let mut workspace = None;
    let result = (|| -> Result<(), &'static str> {
        s.status = Status::Preparing;
        publish(s.clone());
        let stopped = || control.cancel.check().is_err();
        let source = PathBuf::from(&job.source);
        let target = PathBuf::from(&job.target);
        let basename = target
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("unsafe_name")?;
        let root_name = PortablePath::parse(basename)
            .filter(|p| p.parts().len() == 1)
            .ok_or("unsafe_name")?;
        let tree = InputTree::scan(&[source], vec![], &stopped).map_err(|e| e.code())?;
        let source_name = tree
            .entries()
            .first()
            .ok_or("invalid_selection")?
            .name
            .split('/')
            .next()
            .ok_or("unsafe_name")?
            .to_string();
        let grant = Grant::for_creation(target.parent().ok_or("unsafe_target")?)
            .map_err(|_| "unsafe_target")?;
        workspace = Some(grant.workspace(id).map_err(|_| "output_io")?);
        let w = workspace.as_mut().expect("created workspace");
        s.total_known = true;
        s.files = tree.entries().iter().filter(|e| !e.directory).count() as u64;
        s.declared_bytes = tree.total_bytes();
        let mut buffer = [0u8; 64 * 1024];
        for (index, entry) in tree.entries().iter().enumerate() {
            control.cancel.check().map_err(|_| "cancelled")?;
            let suffix = entry.name.strip_prefix(&source_name).ok_or("unsafe_name")?;
            let path = PortablePath::parse(&format!("{}{suffix}", root_name.display()))
                .ok_or("unsafe_name")?;
            s.current = path.display();
            s.status = Status::Running;
            publish(s.clone());
            if entry.directory {
                grant.ensure_directory(&path).map_err(|_| "unsafe_target")?;
                continue;
            }
            if let Some(old) = grant.existing(&path).map_err(|_| "unsafe_target")? {
                if tree.contains_identity(old.version.identity) {
                    return Err("protected_source");
                }
                return Err("output_conflict");
            }
            let mut stage = w.stage().map_err(|_| "output_io")?;
            let mut reader = tree.open(index).map_err(|e| e.code())?;
            let mut bytes = 0u64;
            loop {
                control.cancel.check().map_err(|_| "cancelled")?;
                let n = reader.read(&mut buffer).map_err(|_| "source_changed")?;
                if n == 0 {
                    break;
                }
                bytes += n as u64;
                if bytes > entry.size {
                    return Err("source_changed");
                }
                stage.write_all(&buffer[..n]).map_err(|_| "output_io")?;
                s.decoded_bytes += n as u64;
                s.written_bytes += n as u64;
            }
            if bytes != entry.size {
                return Err("source_changed");
            }
            let _ = tree.open(index).map_err(|e| e.code())?;
            #[cfg(unix)]
            stage
                .restore_mode(reader.portable_mode().map_err(|e| e.code())?)
                .map_err(|_| "output_io")?;
            control.begin_commit()?;
            let receipt = w.commit(stage, &path, None).map_err(|_| "output_io")?;
            s.cleanup_warning |= receipt.cleanup_warning;
            s.committed_files += 1;
            s.committed_bytes += bytes;
            s.record(&path.display(), bytes, "completed", "");
            control.end_commit(false)?;
        }
        s.created_directories = tree.entries().iter().filter(|e| e.directory).count() as u64;
        tree.validate(&stopped).map_err(|e| e.code())?;
        control.complete()?;
        Ok(())
    })();
    s.status = Status::Cleaning;
    publish(s.clone());
    if let Some(w) = workspace {
        s.cleanup_warning |= w.cleanup().is_err();
    }
    s.status = if result.is_ok() {
        Status::Succeeded
    } else if control.cancel.check().is_err() {
        Status::Cancelled
    } else {
        Status::Failed
    };
    s.error = result.err().unwrap_or_default().into();
    s.elapsed_ms = start.elapsed().as_millis() as u64;
    control.mark_terminal();
    publish(s);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "xdzip-promise-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(p.join("source/root/empty")).unwrap();
            fs::create_dir(p.join("target")).unwrap();
            fs::write(p.join("source/root/file.txt"), b"verified contents").unwrap();
            Self(p)
        }
        fn job(&self) -> Job {
            Job {
                source: self.0.join("source/root").to_string_lossy().into(),
                target: self.0.join("target/renamed").to_string_lossy().into(),
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn file_promise_preserves_tree_and_empty_directory_under_destination_name() {
        let f = Fixture::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                f.0.join("source/root/file.txt"),
                fs::Permissions::from_mode(0o751),
            )
            .unwrap();
        }
        let mut last = None;
        run(f.job(), 1, &Control::default(), &mut |s| {
            if s.status.terminal() {
                last = Some(s)
            }
        });
        let result = last.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        assert_eq!(
            fs::read(f.0.join("target/renamed/file.txt")).unwrap(),
            b"verified contents"
        );
        assert!(f.0.join("target/renamed/empty").is_dir());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(f.0.join("target/renamed/file.txt"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o751
            );
        }
    }
    #[test]
    fn file_promise_refuses_overwrite_and_cancel_keeps_existing_bytes() {
        let f = Fixture::new();
        fs::create_dir(f.0.join("target/renamed")).unwrap();
        fs::write(f.0.join("target/renamed/file.txt"), b"user bytes").unwrap();
        let mut last = None;
        run(f.job(), 1, &Control::default(), &mut |s| {
            if s.status.terminal() {
                last = Some(s)
            }
        });
        assert_eq!(last.unwrap().status, Status::Failed);
        assert_eq!(
            fs::read(f.0.join("target/renamed/file.txt")).unwrap(),
            b"user bytes"
        );
        let control = Control::default();
        control.cancel();
        let mut last = None;
        run(f.job(), 2, &control, &mut |s| {
            if s.status.terminal() {
                last = Some(s)
            }
        });
        assert_eq!(last.unwrap().status, Status::Cancelled);
    }
}

//! Archive comment presentation and ZIP-only tail replacement.
use super::{emit, initial_for, output};
use crate::extraction::{Control, Snapshot, Status, TaskKind};
use crate::{Archive, source::Source};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};
use xdzip_codec_api::{Cancellation, Format, ScanError, names};
use xdzip_codec_zip::ArchiveComment as ZipComment;
use xdzip_io::ArchiveUpdate;

/// Maximum encoded size of an ordinary ZIP archive comment.
pub const MAX_ARCHIVE_COMMENT_BYTES: usize = u16::MAX as usize;

/// Session-level comment shown in the desktop editor.
#[derive(Debug)]
pub struct Comment {
    /// Escaped plain text suitable for display or editing when editable.
    pub text: String,
    /// The source and decoded text admit a ZIP comment replacement.
    pub editable: bool,
    /// The format has a comment profile this reader cannot interpret.
    pub unavailable: bool,
}

fn forbidden(ch: char) -> bool {
    (ch.is_control() && !matches!(ch, '\t' | '\n' | '\r'))
        || matches!(ch, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// New ZIP comments are bounded plain text with LF line endings.
pub fn valid_archive_comment(text: &str) -> bool {
    text.len() <= MAX_ARCHIVE_COMMENT_BYTES && !text.chars().any(forbidden) && !text.contains('\r')
}

fn zip_text(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    let mut sample = names::NameSample::default();
    sample.observe_bytes(bytes);
    sample
        .finish(true)
        .decode_unmarked(bytes)
        .map(|(text, _)| text)
        .or_else(|| names::decode_legacy(bytes, "CP437"))
        .expect("CP437 is total")
}

fn display(text: String, writable: bool) -> Comment {
    let clean = !text.chars().any(forbidden);
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let text = if clean {
        text
    } else {
        text.chars()
            .map(|ch| if forbidden(ch) { '\u{fffd}' } else { ch })
            .collect()
    };
    Comment {
        text,
        editable: writable && clean,
        unavailable: false,
    }
}

fn verify_source_comment(
    source: &mut Source,
    comment: &ZipComment,
    control: &Cancellation,
) -> Result<(), ScanError> {
    // A same-size rewrite can retain the filesystem's modification timestamp.
    // Recheck the bounded original EOCD before trusting the session's comment.
    comment
        .verify(source, control, &comment.bytes)
        .map_err(|error| {
            if error == ScanError::Cancelled {
                error
            } else {
                ScanError::SourceChanged
            }
        })
}

impl Archive {
    fn can_replace_zip_comment(&self, comment: &ZipComment) -> Result<bool, ScanError> {
        let ordinary_suffix = self
            .source
            .display_path()
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("zip") || s.eq_ignore_ascii_case("zipx"));
        let eligible = comment.ordinary_layout
            && ordinary_suffix
            && !self.source.is_split()
            && self.disk.is_none()
            && self.password.is_none()
            && !self
                .rows
                .iter()
                .any(|entry| entry.encrypted || entry.method == 99)
            && !self.source.is_read_only()?;
        if !eligible {
            return Ok(false);
        }
        let id = self.source.identities()?[0];
        match ArchiveUpdate::open(self.source.display_path(), id) {
            Ok(_) => Ok(true),
            Err(xdzip_io::OutputError::TargetChanged) => Err(ScanError::SourceChanged),
            Err(_) => Ok(false),
        }
    }

    pub(crate) fn archive_comment(&self, control: &Cancellation) -> Result<Comment, ScanError> {
        control.check()?;
        self.source.validate()?;
        let comment = if let Some(comment) = &self.zip_comment {
            if !self.source.is_split() {
                verify_source_comment(&mut self.source.fork()?, comment, control)?;
            }
            display(
                zip_text(&comment.bytes),
                self.can_replace_zip_comment(comment)?,
            )
        } else if self.format == Format::Rar {
            let mut source = self.source.fork()?;
            let worker = xdzip_codec_native::NativeCodec {
                format: Format::Rar,
            };
            match worker.rar_archive_comment(
                &mut source,
                control,
                self.password
                    .as_ref()
                    .map(crate::extraction::Password::expose),
            ) {
                Ok(Some(text)) => display(text, false),
                Ok(None) => display(String::new(), false),
                Err(ScanError::Unsupported | ScanError::Limit) => Comment {
                    text: String::new(),
                    editable: false,
                    unavailable: true,
                },
                Err(error) => return Err(error),
            }
        } else {
            return Err(ScanError::Unsupported);
        };
        self.source.validate()?;
        Ok(comment)
    }

    pub(crate) fn comment_job(
        &self,
        text: String,
        cancel: &Cancellation,
    ) -> Result<Job, &'static str> {
        cancel.check().map_err(|_| "cancelled")?;
        let comment = self.zip_comment.as_ref().ok_or("unsupported_feature")?;
        if !self
            .can_replace_zip_comment(comment)
            .map_err(|_| "source_changed")?
        {
            return Err("unsupported_feature");
        }
        let original = display(zip_text(&comment.bytes), true);
        if !original.editable {
            return Err("unsupported_feature");
        }
        if !valid_archive_comment(&text) {
            return Err("comment_invalid");
        }
        if text == original.text {
            return Err("comment_unchanged");
        }
        let mut source = self.source.fork().map_err(|_| "source_changed")?;
        verify_source_comment(&mut source, comment, cancel).map_err(|error| {
            if error == ScanError::Cancelled {
                "cancelled"
            } else {
                "source_changed"
            }
        })?;
        Ok(Job {
            source,
            comment: Arc::clone(comment),
            text,
            session: self.key.session.get(),
        })
    }
}

pub(crate) struct Job {
    source: Source,
    comment: Arc<ZipComment>,
    text: String,
    session: u64,
}
impl Job {
    pub fn retained_bytes(&self) -> usize {
        self.source.retained_bytes() + self.comment.bytes.capacity() + self.text.capacity()
    }
}

pub(crate) fn initial(job: &Job, id: u64) -> Snapshot {
    let mut snapshot = initial_for(
        job.source.display_path(),
        Format::Zip,
        job.session,
        TaskKind::Comment,
        id,
    );
    snapshot.declared_bytes = job.comment.length_offset + 2 + job.text.len() as u64;
    snapshot.total_known = true;
    snapshot
}

fn write_error(error: std::io::Error) -> &'static str {
    if error.kind() == std::io::ErrorKind::StorageFull {
        "disk_full"
    } else {
        "output_io"
    }
}

pub(crate) fn run(mut job: Job, id: u64, control: &Control, publish: &mut dyn FnMut(Snapshot)) {
    let start = Instant::now();
    let mut s = initial(&job, id);
    let mut workspace = None;
    let result = (|| {
        s.status = Status::Preparing;
        emit(&mut s, start, publish);
        control.cancel.check().map_err(|_| "cancelled")?;
        job.source.validate().map_err(|_| "source_changed")?;
        if job.source.is_read_only().map_err(|_| "source_changed")? {
            return Err("unsupported_feature");
        }
        let identity = job.source.identities().map_err(|_| "source_changed")?;
        let authority =
            ArchiveUpdate::open(job.source.display_path(), identity[0]).map_err(output)?;
        verify_source_comment(&mut job.source, &job.comment, &control.cancel)
            .map_err(|_| "source_changed")?;
        job.source.validate().map_err(|_| "source_changed")?;
        let w = workspace.insert(authority.workspace(id).map_err(output)?);
        let mut stage = w.stage().map_err(output)?;
        let mut buffer = vec![0; 64 * 1024];
        let mut verify = vec![0; buffer.len()];
        job.source
            .seek(SeekFrom::Start(0))
            .map_err(|_| "source_changed")?;
        s.status = Status::Running;
        emit(&mut s, start, publish);
        let mut last_progress = Instant::now();
        let mut remaining = job.comment.length_offset;
        while remaining > 0 {
            control.cancel.check().map_err(|_| "cancelled")?;
            let n = remaining.min(buffer.len() as u64) as usize;
            job.source
                .read_exact(&mut buffer[..n])
                .map_err(|_| "source_changed")?;
            stage.write_all(&buffer[..n]).map_err(write_error)?;
            remaining -= n as u64;
            s.written_bytes += n as u64;
            if last_progress.elapsed() >= Duration::from_millis(100) {
                emit(&mut s, start, publish);
                last_progress = Instant::now();
            }
        }
        stage
            .write_all(&(job.text.len() as u16).to_le_bytes())
            .map_err(write_error)?;
        stage.write_all(job.text.as_bytes()).map_err(write_error)?;
        s.written_bytes = s.declared_bytes;
        s.status = Status::Verifying;
        emit(&mut s, start, publish);
        let mut reader = stage.verification_reader().map_err(output)?;
        job.source
            .seek(SeekFrom::Start(0))
            .map_err(|_| "source_changed")?;
        remaining = job.comment.length_offset;
        while remaining > 0 {
            control.cancel.check().map_err(|_| "cancelled")?;
            let n = remaining.min(buffer.len() as u64) as usize;
            job.source
                .read_exact(&mut buffer[..n])
                .map_err(|_| "source_changed")?;
            reader
                .read_exact(&mut verify[..n])
                .map_err(|_| "integrity_error")?;
            if buffer[..n] != verify[..n] {
                return Err("integrity_error");
            }
            remaining -= n as u64;
        }
        job.comment
            .verify(&mut reader, &control.cancel, job.text.as_bytes())
            .map_err(|_| "integrity_error")?;
        drop(reader);
        job.source.validate().map_err(|_| "source_changed")?;
        #[cfg(windows)]
        if let Some(zone) = job.source.origin_zone().map_err(|_| "source_changed")? {
            stage.preserve_origin(zone).map_err(output)?;
        }
        control.begin_commit()?;
        s.status = Status::Committing;
        emit(&mut s, start, publish);
        let receipt = authority.commit(w, stage).map_err(output)?;
        control.mark_terminal();
        s.cleanup_warning |= receipt.cleanup_warning;
        s.committed_bytes = s.written_bytes;
        s.output_parts = 1;
        Ok(())
    })();
    s.status = Status::Cleaning;
    emit(&mut s, start, publish);
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
    s.error = if s.status == Status::Cancelled {
        "cancelled"
    } else {
        result.err().unwrap_or_default()
    }
    .into();
    control.mark_terminal();
    emit(&mut s, start, publish);
}

//! Stateful solid-folder streams over host-authorized input and output sinks.
use crate::decode::{DecodeLimits, FolderStream, MethodGraphDecoder, SharedSource, decode_error};
use crate::source::Reader;
use crate::{Folder, Index, read_index_with_decoder};
use std::cell::RefCell;
use std::rc::Rc;
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadReader};
use xdzip_codec_api::{Cancellation, Entry, ReadSource, ScanLimits};
use xdzip_compression::Crc32;
use zeroize::Zeroizing;
const FILE_LIMIT: u64 = 8 << 30;
const CHUNK: usize = 8192;
struct Active<'a> {
    folder: usize,
    graph: FolderStream<'a>,
    next: usize,
    password: Option<Zeroizing<String>>,
}
pub(crate) struct Payload<'a> {
    source: SharedSource<'a>,
    control: &'a Cancellation,
    index: Index,
    decoder: MethodGraphDecoder,
    active: Option<Active<'a>>,
    password: Option<Zeroizing<String>>,
    failed: bool,
}
fn secret(value: Option<&str>) -> Result<Option<Zeroizing<String>>, DecodeError> {
    value
        .map(|value| {
            if value.len() > 4096 {
                return Err(DecodeError::Limit);
            }
            let mut copy = Zeroizing::new(String::new());
            copy.try_reserve_exact(value.len())
                .map_err(|_| DecodeError::Limit)?;
            copy.push_str(value);
            Ok(copy)
        })
        .transpose()
}
impl<'a> Payload<'a> {
    pub(crate) fn open(
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
        password: Option<&str>,
    ) -> Result<Self, DecodeError> {
        Self::open_with_limits(source, snapshot, control, password, DecodeLimits::default())
    }
    pub(crate) fn open_with_limits(
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
        password: Option<&str>,
        limits: DecodeLimits,
    ) -> Result<Self, DecodeError> {
        let mut decoder = MethodGraphDecoder::new(limits);
        let index = read_index_with_decoder(
            source,
            control,
            ScanLimits::default(),
            password,
            &mut decoder,
        )
        .map_err(decode_error)?;
        if index.rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        decoder.begin_payload()?;
        let mut source = Reader::new(source, control, 0).map_err(decode_error)?;
        source.set_limit(decoder.limits.input_bytes);
        let source: SharedSource<'a> = Rc::new(RefCell::new(Box::new(source)));
        Ok(Self {
            source,
            control,
            index,
            decoder,
            active: None,
            password: secret(password)?,
            failed: false,
        })
    }
    fn check(&self, control: &Cancellation) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        self.control.check().map_err(decode_error)?;
        self.source
            .try_borrow()
            .map_err(|_| DecodeError::Integrity)?
            .check()
            .map_err(decode_error)
    }
    fn record(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<(), DecodeError> {
        self.check(control)?;
        let record = entry
            .id
            .get()
            .checked_sub(1)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or(DecodeError::Integrity)?;
        if self.index.rows.get(record) != Some(entry) {
            return Err(DecodeError::Integrity);
        }
        if entry.link || !entry.payload_supported() {
            return Err(DecodeError::Unsupported);
        }
        if entry.size > FILE_LIMIT {
            return Err(DecodeError::Limit);
        }
        let file = self.index.files.get(record).ok_or(DecodeError::Integrity)?;
        let Some(location) = &file.data else {
            return if entry.size == 0 {
                self.check(control)
            } else {
                Err(DecodeError::Integrity)
            };
        };
        if entry.directory {
            return Err(DecodeError::Unsupported);
        }
        let folder_index = location.folder;
        let stream_index = location.stream;
        let folder = self
            .index
            .streams
            .folders
            .get(folder_index)
            .ok_or(DecodeError::Integrity)?;
        let stream = folder
            .substreams
            .get(stream_index)
            .ok_or(DecodeError::Integrity)?;
        if stream.length != entry.size {
            return Err(DecodeError::Integrity);
        }
        let encrypted = folder.encrypted();
        let result = (|| {
            let password = password.or(self.password.as_ref().map(|value| value.as_str()));
            let restart = self.active.as_ref().is_none_or(|active| {
                active.folder != folder_index
                    || active.next > stream_index
                    || folder.encrypted()
                        && active.password.as_ref().map(|p| p.as_str()) != password
            });
            if restart {
                // Drop dictionary/model state before admitting another graph.
                self.active = None;
                let graph = self.decoder.stream(
                    self.source.clone(),
                    &self.index.streams,
                    folder_index,
                    self.control,
                    password,
                )?;
                self.active = Some(Active {
                    folder: folder_index,
                    graph,
                    next: 0,
                    password: if folder.encrypted() {
                        secret(password)?
                    } else {
                        None
                    },
                });
            }
            let active = self.active.as_mut().ok_or(DecodeError::Integrity)?;
            while active.next < stream_index {
                active.consume(folder, false, sink, control)?;
            }
            active.consume(folder, true, sink, control)?;
            if stream.crc.is_none() {
                while active.next < folder.substreams.len() {
                    active.consume(folder, false, sink, control)?;
                }
                active.graph.finish()?;
            } else if active.graph.position() == folder.unpacked_size() {
                active.graph.finish()?;
            }
            Ok(())
        })();
        result.map_err(|error| crate::encrypted_decode_error(error, encrypted))?;
        self.check(control)
    }
}
impl Active<'_> {
    fn consume(
        &mut self,
        folder: &Folder,
        selected: bool,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        let stream = folder
            .substreams
            .get(self.next)
            .ok_or(DecodeError::Integrity)?;
        if self.graph.position() != stream.offset {
            return Err(DecodeError::Integrity);
        }
        let end = stream
            .offset
            .checked_add(stream.length)
            .ok_or(DecodeError::Integrity)?;
        let mut crc = Crc32::new();
        let mut block = Zeroizing::new([0; CHUNK]);
        while self.graph.position() < end {
            control.check().map_err(decode_error)?;
            let width = (end - self.graph.position()).min(CHUNK as u64) as usize;
            let n = self.graph.read(&mut block[..width])?;
            if n == 0 {
                return Err(DecodeError::Integrity);
            }
            // Only this actual terminal chunk funds its own output delivery.
            sink.charge_skipped(n as u64)?;
            crc.update(&block[..n]);
            if selected {
                control.check().map_err(decode_error)?;
                sink.write_precharged_chunk(&block[..n])?;
            }
        }
        if stream
            .crc
            .is_some_and(|expected| expected != crc.checksum())
        {
            return Err(DecodeError::Integrity);
        }
        self.next += 1;
        control.check().map_err(decode_error)
    }
}
impl PayloadReader for Payload<'_> {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.decode_with_password(entry, sink, control, None)
    }
    fn decode_with_password(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<(), DecodeError> {
        if self.failed {
            return Err(DecodeError::Integrity);
        }
        let result = self.record(entry, sink, control, password);
        if result.is_err() {
            self.failed = true;
            self.active = None;
        }
        result
    }
}

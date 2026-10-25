//! Complete-folder candidate cache, separate from the method graph executor.
//! No entry succeeds until its entire folder has passed EOF and all CRCs.
use crate::decode::{MethodGraphDecoder, decode_error};
use crate::source::Reader;
use crate::{HeaderSource, Index, read_index_with_decoder};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadReader};
use xdzip_codec_api::{Cancellation, Entry, ReadSource, ScanLimits};
use zeroize::Zeroizing;

struct Cache {
    bytes: Zeroizing<Vec<u8>>,
    credit: usize,
    password: Option<Zeroizing<String>>,
}
pub(crate) struct Payload<'a> {
    source: Reader<'a>,
    index: Index,
    decoder: MethodGraphDecoder,
    // Verified folders are never evicted or replayed within this session.
    cache: Vec<Option<Cache>>,
    retained: u64,
    cache_overhead: u64,
    failed: bool,
}
impl<'a> Payload<'a> {
    pub(crate) fn open(
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
        password: Option<&str>,
    ) -> Result<Self, DecodeError> {
        let mut decoder = MethodGraphDecoder::default();
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
        let cache_overhead = index
            .streams
            .folders
            .len()
            .checked_mul(std::mem::size_of::<Option<Cache>>())
            .ok_or(DecodeError::Limit)? as u64;
        if cache_overhead > decoder.limits.memory_bytes {
            return Err(DecodeError::Limit);
        }
        let mut cache = Vec::new();
        cache
            .try_reserve_exact(index.streams.folders.len())
            .map_err(|_| DecodeError::Limit)?;
        cache.resize_with(index.streams.folders.len(), || None);
        let mut source = Reader::new(source, control, 0).map_err(decode_error)?;
        source.set_limit(decoder.limits.input_bytes);
        Ok(Self {
            source,
            index,
            decoder,
            cache,
            retained: 0,
            cache_overhead,
            failed: false,
        })
    }
    fn record(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        self.source.check().map_err(decode_error)?;
        let record = entry
            .id
            .get()
            .checked_sub(1)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or(DecodeError::Integrity)?;
        if self.index.rows.get(record) != Some(entry) {
            return Err(DecodeError::Integrity);
        }
        if entry.directory || entry.link || !entry.payload_supported() {
            return Err(DecodeError::Unsupported);
        }
        let file = self.index.files.get(record).ok_or(DecodeError::Integrity)?;
        let Some(location) = &file.data else {
            return if entry.size == 0 {
                control.check().map_err(decode_error)
            } else {
                Err(DecodeError::Integrity)
            };
        };
        let folder_index = location.folder;
        let folder = self
            .index
            .streams
            .folders
            .get(folder_index)
            .ok_or(DecodeError::Integrity)?;
        let stream = folder
            .substreams
            .get(location.stream)
            .ok_or(DecodeError::Integrity)?;
        let start = usize::try_from(stream.offset).map_err(|_| DecodeError::Limit)?;
        let length = usize::try_from(stream.length).map_err(|_| DecodeError::Limit)?;
        let end = start.checked_add(length).ok_or(DecodeError::Integrity)?;
        if stream.length != entry.size {
            return Err(DecodeError::Integrity);
        }
        if self.cache[folder_index].is_none() {
            let secret_bytes = if folder.encrypted() {
                password.map_or(0, str::len) as u64
            } else {
                0
            };
            let next = self
                .retained
                .checked_add(folder.unpacked_size())
                .filter(|n| *n <= self.decoder.limits.cache_bytes)
                .ok_or(DecodeError::Limit)?;
            let bytes = self.decoder.folder(
                &mut self.source,
                &self.index.streams,
                folder_index,
                self.retained
                    .checked_add(self.cache_overhead)
                    .and_then(|n| n.checked_add(secret_bytes))
                    .ok_or(DecodeError::Limit)?,
                control,
                password,
                &mut |count| sink.charge_skipped(count),
            )?;
            let secret = if folder.encrypted() {
                let password = password.ok_or(DecodeError::PasswordRequired)?;
                let mut copy = Zeroizing::new(String::new());
                copy.try_reserve_exact(password.len())
                    .map_err(|_| DecodeError::Limit)?;
                copy.push_str(password);
                Some(copy)
            } else {
                None
            };
            self.cache_overhead = self
                .cache_overhead
                .checked_add(secret.as_ref().map_or(0, |value| value.capacity()) as u64)
                .ok_or(DecodeError::Limit)?;
            self.retained = next;
            self.cache[folder_index] = Some(Cache {
                credit: bytes.len(),
                bytes,
                password: secret,
            });
        }
        let cache = self.cache[folder_index]
            .as_mut()
            .ok_or(DecodeError::Integrity)?;
        if let Some(secret) = &cache.password {
            let password = password.ok_or(DecodeError::PasswordRequired)?;
            if password != secret.as_str() {
                return Err(DecodeError::Integrity);
            }
        }
        let bytes = cache.bytes.get(start..end).ok_or(DecodeError::Integrity)?;
        for block in bytes.chunks(8192) {
            control.check().map_err(decode_error)?;
            self.source.check().map_err(decode_error)?;
            let count = block.len().min(cache.credit);
            if count != 0 {
                sink.write_precharged_chunk(&block[..count])?;
                cache.credit -= count;
            }
            if count < block.len() {
                control.check().map_err(decode_error)?;
                sink.write_chunk(&block[count..])?;
            }
        }
        self.source.check().map_err(decode_error)?;
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
        }
        result
    }
}

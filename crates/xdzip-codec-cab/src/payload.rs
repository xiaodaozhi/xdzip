use crate::index::{Frame, Index, Method, u16_at, u32_at};
use crate::source::Reader;
use crate::{FILE_LIMIT, INPUT_LIMIT, decode_error};
use std::io;
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadReader};
use xdzip_codec_api::{Cancellation, Entry};
use xdzip_compression::{LzxDecoder, MszipDecoder, QuantumDecoder};

enum Decoder {
    Store,
    Mszip(Box<MszipDecoder>),
    Lzx(Box<LzxDecoder>),
    Quantum(Box<QuantumDecoder>),
}
fn algorithm_error(error: io::Error) -> DecodeError {
    if error.kind() == io::ErrorKind::OutOfMemory {
        DecodeError::Limit
    } else {
        DecodeError::Integrity
    }
}
impl Decoder {
    fn new(method: Method) -> Result<Self, DecodeError> {
        Ok(match method {
            Method::Store => Self::Store,
            Method::Mszip => Self::Mszip(Box::default()),
            Method::Lzx(bits) => {
                Self::Lzx(Box::new(LzxDecoder::new(bits).map_err(algorithm_error)?))
            }
            Method::Quantum(bits, level) => Self::Quantum(Box::new(
                QuantumDecoder::new(bits, level).map_err(algorithm_error)?,
            )),
        })
    }
    fn frame(&mut self, packed: Vec<u8>, expected: usize) -> Result<Vec<u8>, DecodeError> {
        match self {
            Self::Store if packed.len() == expected => Ok(packed),
            Self::Store => Err(DecodeError::Integrity),
            Self::Mszip(decoder) => decoder
                .decode_frame(&packed, expected)
                .map_err(algorithm_error),
            Self::Lzx(decoder) => decoder
                .decode_frame(&packed, expected)
                .map_err(algorithm_error),
            Self::Quantum(decoder) => decoder
                .decode_frame(&packed, expected)
                .map_err(algorithm_error),
        }
    }
    fn finish(&mut self) -> Result<(), DecodeError> {
        match self {
            Self::Store => Ok(()),
            Self::Mszip(decoder) => decoder.finish().map_err(algorithm_error),
            Self::Lzx(decoder) => decoder.finish().map_err(algorithm_error),
            Self::Quantum(decoder) => decoder.finish().map_err(algorithm_error),
        }
    }
}

struct Cache {
    folder: usize,
    decoder: Decoder,
    next_frame: usize,
    start: u64,
    output: Vec<u8>,
    remaining_precharged: usize,
    finished: bool,
}
pub(crate) struct Payload<'a> {
    reader: Reader<'a>,
    index: Index,
    cache: Option<Cache>,
    work: u64,
    failed: bool,
}

// MS-CAB 3.1 defines distinct word parity for payload and header, including
// big-endian placement of each call's 1..3-byte tail. It is not a CRC.
fn parity(bytes: &[u8]) -> u32 {
    let (words, remainder) = bytes.as_chunks::<4>();
    let full = words
        .iter()
        .fold(0, |sum, word| sum ^ u32::from_le_bytes(*word));
    full ^ remainder
        .iter()
        .fold(0u32, |tail, byte| (tail << 8) | u32::from(*byte))
}
fn packed_frame(reader: &mut Reader<'_>, frame: &Frame) -> Result<Vec<u8>, DecodeError> {
    let size: usize = frame
        .fragments
        .iter()
        .map(|fragment| usize::from(fragment.packed))
        .sum();
    let mut packed = Vec::new();
    packed
        .try_reserve_exact(size)
        .map_err(|_| DecodeError::Limit)?;
    for fragment in &frame.fragments {
        reader.check().map_err(decode_error)?;
        let header = reader
            .bytes(
                fragment.volume,
                fragment.offset,
                8 + usize::from(fragment.reserve),
            )
            .map_err(decode_error)?;
        if u32_at(&header, 0) != fragment.checksum
            || u16_at(&header, 4) != fragment.packed
            || u16_at(&header, 6) != fragment.unpacked
        {
            return Err(DecodeError::Integrity);
        }
        let bytes = reader
            .bytes(
                fragment.volume,
                fragment.offset + header.len() as u64,
                usize::from(fragment.packed),
            )
            .map_err(decode_error)?;
        if fragment.checksum != 0 && parity(&bytes) ^ parity(&header[4..]) != fragment.checksum {
            return Err(DecodeError::Integrity);
        }
        packed.extend_from_slice(&bytes);
    }
    Ok(packed)
}
impl<'a> Payload<'a> {
    pub(crate) fn new(reader: Reader<'a>, index: Index) -> Self {
        Self {
            reader,
            index,
            cache: None,
            work: 0,
            failed: false,
        }
    }
    fn select_folder(&mut self, folder: usize, offset: u64) -> Result<(), DecodeError> {
        if self
            .cache
            .as_ref()
            .is_none_or(|cache| cache.folder != folder || offset < cache.start)
        {
            self.cache = Some(Cache {
                folder,
                decoder: Decoder::new(self.index.folders[folder].method)?,
                next_frame: 0,
                start: 0,
                output: Vec::new(),
                remaining_precharged: 0,
                finished: false,
            });
        }
        Ok(())
    }
    fn next_frame(
        &mut self,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<bool, DecodeError> {
        control.check().map_err(decode_error)?;
        let cache = self.cache.as_mut().ok_or(DecodeError::Integrity)?;
        let folder = &self.index.folders[cache.folder];
        let Some(frame) = folder.frames.get(cache.next_frame) else {
            return Ok(false);
        };
        let count = u64::from(frame.unpacked);
        self.work = self
            .work
            .checked_add(count)
            .filter(|work| *work <= INPUT_LIMIT)
            .ok_or(DecodeError::Limit)?;
        let packed = packed_frame(&mut self.reader, frame)?;
        let output = cache.decoder.frame(packed, usize::from(frame.unpacked))?;
        if output.len() != usize::from(frame.unpacked) {
            return Err(DecodeError::Integrity);
        }
        sink.charge_skipped(count)?;
        cache.start = frame.start;
        cache.output = output;
        // Unpublished bytes were decoded too. Their work was charged with this
        // frame, but only this frame's remaining allowance can cover delivery.
        cache.remaining_precharged = usize::from(frame.unpacked);
        cache.next_frame += 1;
        Ok(true)
    }
    fn position(
        &mut self,
        position: u64,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        loop {
            let cache = self.cache.as_ref().ok_or(DecodeError::Integrity)?;
            if position >= cache.start && position < cache.start + cache.output.len() as u64 {
                return Ok(());
            }
            if !self.next_frame(sink, control)? {
                return Err(DecodeError::Integrity);
            }
        }
    }
    fn finish_folder(
        &mut self,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        while self.next_frame(sink, control)? {}
        let cache = self.cache.as_mut().ok_or(DecodeError::Integrity)?;
        if !cache.finished {
            cache.decoder.finish()?;
            cache.finished = true;
        }
        Ok(())
    }
    fn decode_record(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
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
        if entry.size > FILE_LIMIT {
            return Err(DecodeError::Limit);
        }
        let file = &self.index.files[record];
        let (folder, start) = (file.folder, file.offset);
        let end = start
            .checked_add(entry.size)
            .ok_or(DecodeError::Integrity)?;
        self.select_folder(folder, start)?;
        let mut cursor = start;
        if entry.size == 0 && self.index.folders[folder].size != 0 {
            let position = start.min(self.index.folders[folder].size - 1);
            // An empty record still validates its containing frame and all
            // necessary history; it is not an integrity-check escape hatch.
            self.position(position, sink, control)?;
        }
        while cursor < end {
            control.check().map_err(decode_error)?;
            self.position(cursor, sink, control)?;
            let cache = self.cache.as_mut().ok_or(DecodeError::Integrity)?;
            let offset =
                usize::try_from(cursor - cache.start).map_err(|_| DecodeError::Integrity)?;
            let count = usize::try_from((end - cursor).min((cache.output.len() - offset) as u64))
                .map_err(|_| DecodeError::Integrity)?;
            let precharged = count.min(cache.remaining_precharged);
            if precharged != 0 {
                sink.write_precharged_chunk(&cache.output[offset..offset + precharged])?;
                cache.remaining_precharged -= precharged;
            }
            if precharged < count {
                control.check().map_err(decode_error)?;
                sink.write_chunk(&cache.output[offset + precharged..offset + count])?;
            }
            cursor += count as u64;
        }
        if self.index.folders[folder].last_record == Some(record) {
            self.finish_folder(sink, control)?;
        }
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
        if self.failed {
            return Err(DecodeError::Integrity);
        }
        let result = self.decode_record(entry, sink, control);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}

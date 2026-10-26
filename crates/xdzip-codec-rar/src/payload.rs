use crate::algorithm::{Algorithm, Engine, WorkOutput};
use crate::crypto::{CbcReader, KeyMaterial};
use crate::index::{Descriptor, Index, Part};
use crate::legacy_crypto::{LegacyCipher, LegacyReader};
use crate::source::Reader;
use crate::{FILE_LIMIT, INPUT_LIMIT, MEMORY_LIMIT, decode_error};
use std::io::{self, Read};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadReader};
use xdzip_codec_api::{Cancellation, Entry, ScanError};
use xdzip_compression::Crc32;
use xdzip_crypto::Blake2sp;

enum Decrypted<'a, R> {
    Modern(CbcReader<'a, R>),
    Legacy(LegacyReader<'a, R>),
}
impl<R: Read> Read for Decrypted<'_, R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Modern(r) => r.read(bytes),
            Self::Legacy(r) => r.read(bytes),
        }
    }
}
impl<R: Read> Decrypted<'_, R> {
    fn finish(&mut self) -> io::Result<()> {
        match self {
            Self::Modern(r) => r.finish(),
            Self::Legacy(r) => r.finish(),
        }
    }
    fn finish_padding(&mut self, n: u8) -> io::Result<()> {
        match self {
            Self::Modern(r) => r.finish_padding(n),
            Self::Legacy(r) => r.finish_padding(n),
        }
    }
    fn counters(&self) -> (u64, u64) {
        match self {
            Self::Modern(r) => (r.consumed_bytes(), r.decoded_bytes()),
            Self::Legacy(r) => (r.consumed_bytes(), r.decoded_bytes()),
        }
    }
}

fn verify_hash(
    description: &Descriptor,
    crc: u32,
    blake: Option<Blake2sp>,
    key: Option<&KeyMaterial>,
    old: Option<u16>,
) -> Result<(), DecodeError> {
    if description
        .old_checksum
        .is_some_and(|expected| Some(expected) != old)
    {
        return Err(DecodeError::Integrity);
    }
    let keyed = description.crypt.as_ref().is_some_and(|c| c.flags & 2 != 0);
    let key = if keyed {
        Some(key.ok_or(DecodeError::Integrity)?)
    } else {
        None
    };
    // The known file-hash extra selects the full-width comparison, matching
    // the reference profile. A basic CRC field remains preserved metadata.
    if let Some(expected) = description.hash {
        let actual = blake.ok_or(DecodeError::Integrity)?.finish();
        let actual = key.map_or(actual, |k| k.tweak_blake(actual));
        if actual != expected {
            return Err(DecodeError::Integrity);
        }
    } else if let Some(expected) = description.crc {
        let actual = key.map_or(crc, |k| k.tweak_crc(crc));
        if actual != expected {
            return Err(DecodeError::Integrity);
        }
    }
    Ok(())
}

struct PackedInput<'a, 's, 'k> {
    reader: &'a mut Reader<'s>,
    parts: &'a [Part],
    selected: &'a [usize],
    part: usize,
    offset: u64,
    active: bool,
    crc: Crc32,
    blake: Option<Blake2sp>,
    key: Option<&'k KeyMaterial>,
    failed: bool,
}
impl PackedInput<'_, '_, '_> {
    fn current(&mut self) -> Result<bool, ScanError> {
        while let Some(index) = self.selected.get(self.part) {
            let part = &self.parts[*index];
            if !self.active {
                if self
                    .reader
                    .bytes(part.volume, part.header_at, part.header.len())?
                    != part.header
                {
                    return Err(ScanError::SourceChanged);
                }
                self.active = true;
                self.crc = Crc32::new();
                self.blake = part.description.hash.map(|_| Blake2sp::new());
            }
            if self.offset < part.packed {
                return Ok(true);
            }
            if part.after {
                let meaningful = part.description.method >= 50
                    || (part.description.method >= 20 && part.description.crc != Some(u32::MAX));
                if meaningful {
                    verify_hash(
                        &part.description,
                        self.crc.checksum(),
                        self.blake.take(),
                        self.key,
                        None,
                    )
                    .map_err(crate::protection::scan_error)?;
                }
            }
            self.part += 1;
            self.offset = 0;
            self.active = false;
        }
        Ok(false)
    }
    fn finish(&mut self) -> Result<(), DecodeError> {
        if self.failed {
            return Err(DecodeError::Integrity);
        }
        if self.current().map_err(decode_error)? {
            return Err(DecodeError::Integrity);
        }
        Ok(())
    }
    fn read_checked(&mut self, bytes: &mut [u8]) -> Result<usize, ScanError> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if !self.current()? {
            return Ok(0);
        }
        let part = &self.parts[self.selected[self.part]];
        let n = usize::try_from((part.packed - self.offset).min(bytes.len().min(8192) as u64))
            .map_err(|_| ScanError::Limit)?;
        self.reader
            .read_into(part.volume, part.data_at + self.offset, &mut bytes[..n])?;
        self.crc.update(&bytes[..n]);
        if let Some(hash) = &mut self.blake {
            hash.update(&bytes[..n]);
        }
        self.offset += n as u64;
        Ok(n)
    }
}
impl Read for PackedInput<'_, '_, '_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.failed {
            return Err(io::Error::other(ScanError::Invalid));
        }
        match self.read_checked(bytes) {
            Ok(n) => Ok(n),
            Err(error) => {
                self.failed = true;
                Err(io::Error::other(error))
            }
        }
    }
}
struct VerifiedSink<'a> {
    sink: &'a mut dyn DecodedSink,
    expected: u64,
    written: u64,
    crc: Crc32,
    blake: Option<Blake2sp>,
    old: Option<u16>,
    work: &'a mut u64,
    reserved: u64,
    auxiliary: u64,
    credit: u64,
}
impl VerifiedSink<'_> {
    fn update_integrity(&mut self, bytes: &[u8]) {
        self.crc.update(bytes);
        if let Some(hash) = &mut self.blake {
            hash.update(bytes);
        }
        if let Some(sum) = &mut self.old {
            for &byte in bytes {
                *sum = sum.wrapping_add(u16::from(byte)).rotate_left(1);
            }
        }
    }
}
impl DecodedSink for VerifiedSink<'_> {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        let next = self
            .written
            .checked_add(bytes.len() as u64)
            .filter(|n| *n <= self.expected && *n <= FILE_LIMIT)
            .ok_or(DecodeError::Integrity)?;
        *self.work = self
            .work
            .checked_add(bytes.len() as u64)
            .filter(|n| *n <= INPUT_LIMIT)
            .ok_or(DecodeError::Limit)?;
        self.update_integrity(bytes);
        self.sink.write_chunk(bytes)?;
        self.written = next;
        Ok(())
    }
    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        *self.work = self
            .work
            .checked_add(bytes)
            .filter(|n| *n <= INPUT_LIMIT)
            .ok_or(DecodeError::Limit)?;
        self.sink.charge_skipped(bytes)
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        let n = bytes.len() as u64;
        self.credit = self.credit.checked_sub(n).ok_or(DecodeError::Integrity)?;
        let next = self
            .written
            .checked_add(n)
            .filter(|n| *n <= self.expected && *n <= FILE_LIMIT)
            .ok_or(DecodeError::Integrity)?;
        self.update_integrity(bytes);
        self.sink.write_precharged_chunk(bytes)?;
        self.written = next;
        Ok(())
    }
}
impl WorkOutput for VerifiedSink<'_> {
    fn reserve_work(&mut self, bytes: u64) -> Result<(), DecodeError> {
        if bytes > 8192 {
            return Err(DecodeError::Limit);
        }
        if self
            .work
            .checked_add(self.reserved)
            .and_then(|n| n.checked_add(self.auxiliary))
            .and_then(|n| n.checked_add(bytes))
            .is_none_or(|n| n > INPUT_LIMIT)
        {
            return Err(DecodeError::Limit);
        }
        self.reserved = self
            .reserved
            .checked_add(bytes)
            .filter(|n| *n <= 8192)
            .ok_or(DecodeError::Integrity)?;
        Ok(())
    }
    fn produced_work(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.reserved = self
            .reserved
            .checked_sub(bytes)
            .ok_or(DecodeError::Integrity)?;
        *self.work = self
            .work
            .checked_add(bytes)
            .filter(|n| *n <= INPUT_LIMIT)
            .ok_or(DecodeError::Limit)?;
        if bytes != 0 {
            self.sink.charge_skipped(bytes)?;
        }
        self.credit = self
            .credit
            .checked_add(bytes)
            .filter(|n| *n <= self.expected)
            .ok_or(DecodeError::Integrity)?;
        Ok(())
    }
    fn reserve_aux(&mut self, bytes: u64) -> Result<(), DecodeError> {
        if bytes > INPUT_LIMIT
            || self
                .work
                .checked_add(self.reserved)
                .and_then(|n| n.checked_add(self.auxiliary))
                .and_then(|n| n.checked_add(bytes))
                .is_none_or(|n| n > INPUT_LIMIT)
        {
            return Err(DecodeError::Limit);
        }
        self.auxiliary = self
            .auxiliary
            .checked_add(bytes)
            .filter(|n| *n <= INPUT_LIMIT)
            .ok_or(DecodeError::Integrity)?;
        Ok(())
    }
    fn produced_aux(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.auxiliary = self
            .auxiliary
            .checked_sub(bytes)
            .ok_or(DecodeError::Integrity)?;
        *self.work = self
            .work
            .checked_add(bytes)
            .filter(|n| *n <= INPUT_LIMIT)
            .ok_or(DecodeError::Limit)?;
        let mut remaining = bytes;
        while remaining != 0 {
            let count = remaining.min(65536);
            self.sink.charge_auxiliary(count)?;
            remaining -= count;
        }
        Ok(())
    }
    fn remaining_aux_work(&self) -> u64 {
        self.work
            .checked_add(self.reserved)
            .and_then(|n| n.checked_add(self.auxiliary))
            .and_then(|n| INPUT_LIMIT.checked_sub(n))
            .unwrap_or(0)
    }
}
pub(crate) struct Payload<'a> {
    reader: Reader<'a>,
    index: Index,
    algorithm: Box<dyn Algorithm>,
    work: u64,
    failed: bool,
    next_file: usize,
}
impl<'a> Payload<'a> {
    pub fn new(reader: Reader<'a>, index: Index) -> Self {
        Self::with_algorithm(reader, index, Box::<Engine>::default())
    }
    fn with_algorithm(reader: Reader<'a>, index: Index, algorithm: Box<dyn Algorithm>) -> Self {
        Self {
            reader,
            index,
            algorithm,
            work: 0,
            failed: false,
            next_file: 0,
        }
    }
    fn run_one(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        let n = usize::try_from(entry.id.get() - 1).map_err(|_| DecodeError::Integrity)?;
        if self.index.rows.get(n) != Some(entry) {
            return Err(DecodeError::Integrity);
        }
        let file = &self.index.files[n];
        let mut first = self.index.parts[file.parts[0]].description.clone();
        first.size = Some(entry.size);
        if !entry.size_known || entry.warning || entry.link || first.special || first.redirection {
            return Err(DecodeError::Unsupported);
        }
        let (key, legacy): (Option<&KeyMaterial>, Option<LegacyCipher>) = if first.encrypted {
            if password.is_none() {
                return Err(DecodeError::PasswordRequired);
            }
            if first.legacy {
                (
                    None,
                    Some(self.index.protection.legacy_content(
                        &first,
                        password,
                        control,
                        &self.reader,
                    )?),
                )
            } else if matches!(first.method, 50 | 70) {
                let crypt = first.crypt.as_ref().ok_or(DecodeError::Unsupported)?;
                (
                    Some(self.index.protection.key(crypt, password, false, control)?),
                    None,
                )
            } else {
                return Err(DecodeError::Unsupported);
            }
        } else {
            (None, None)
        };
        let unverified = first.encrypted
            && !key.is_some_and(|key| {
                first
                    .crypt
                    .as_ref()
                    .is_some_and(|crypt| crate::protection::trusted_check(crypt, key, false))
            });
        let result = (|| {
            let padding = if first.encrypted && !first.ancient && !matches!(first.method, 13 | 15) {
                15
            } else {
                0
            };
            if entry.directory {
                if entry.size != 0 || file.parts.iter().any(|p| self.index.parts[*p].packed != 0) {
                    return Err(DecodeError::Integrity);
                }
                for p in &file.parts {
                    let part = &self.index.parts[*p];
                    if self
                        .reader
                        .bytes(part.volume, part.header_at, part.header.len())
                        .map_err(decode_error)?
                        != part.header
                    {
                        return Err(DecodeError::SourceIo);
                    }
                    verify_hash(
                        &part.description,
                        0,
                        part.description.hash.map(|_| Blake2sp::new()),
                        key,
                        Some(0),
                    )?;
                }
                return control.check().map_err(decode_error);
            }
            let mut coding = first.clone();
            // Encryption and final integrity belong to the bounded container
            // adapter; the compression engine receives admitted plaintext only.
            coding.encrypted = false;
            coding.crypt = None;
            coding.hash = None;
            coding.old_checksum = None;
            let memory = self
                .algorithm
                .memory_required(&coding)?
                .checked_add(crate::protection::Protection::MEMORY_BYTES)
                .ok_or(DecodeError::Limit)?;
            if memory > MEMORY_LIMIT {
                return Err(DecodeError::Limit);
            }
            if entry.size > FILE_LIMIT {
                return Err(DecodeError::Limit);
            }
            let total = file
                .parts
                .iter()
                .try_fold(0u64, |sum, p| sum.checked_add(self.index.parts[*p].packed))
                .ok_or(DecodeError::Integrity)?;
            if first.compression == 0
                && ((!first.encrypted && total != entry.size)
                    || (first.encrypted
                        && (total < entry.size || total - entry.size > u64::from(padding))))
            {
                return Err(DecodeError::Integrity);
            }
            let mut input = PackedInput {
                reader: &mut self.reader,
                parts: &self.index.parts,
                selected: &file.parts,
                part: 0,
                offset: 0,
                active: false,
                crc: Crc32::new(),
                blake: None,
                key,
                failed: false,
            };
            let mut verified = VerifiedSink {
                sink,
                expected: entry.size,
                written: 0,
                crc: Crc32::new(),
                blake: first.hash.map(|_| Blake2sp::new()),
                old: first.old_checksum.map(|_| 0),
                work: &mut self.work,
                reserved: 0,
                auxiliary: 0,
                credit: 0,
            };
            if first.encrypted {
                let mut plain = if let Some(key) = key {
                    let iv = first
                        .crypt
                        .as_ref()
                        .ok_or(DecodeError::Integrity)?
                        .iv
                        .as_slice()
                        .try_into()
                        .map_err(|_| DecodeError::Integrity)?;
                    Decrypted::Modern(
                        CbcReader::new(&mut input, total, key, iv, control)
                            .map_err(crate::algorithm::input_error)?,
                    )
                } else {
                    Decrypted::Legacy(
                        LegacyReader::new(
                            &mut input,
                            total,
                            legacy.ok_or(DecodeError::Integrity)?,
                            control,
                        )
                        .map_err(crate::algorithm::input_error)?,
                    )
                };
                if coding.compression == 0 {
                    let mut content = plain.by_ref().take(entry.size);
                    self.algorithm
                        .decode(&coding, &mut content, &mut verified, control)?;
                    if content.limit() != 0 {
                        return Err(DecodeError::Integrity);
                    }
                    plain
                        .finish_padding(padding)
                        .map_err(crate::algorithm::input_error)?;
                } else {
                    self.algorithm.decode_padded(
                        &coding,
                        &mut plain,
                        &mut verified,
                        control,
                        padding,
                    )?;
                    plain.finish().map_err(crate::algorithm::input_error)?;
                }
                if plain.counters() != (total, total) {
                    return Err(DecodeError::Integrity);
                }
            } else {
                self.algorithm
                    .decode_padded(&coding, &mut input, &mut verified, control, 0)?;
            }
            input.finish()?;
            control.check().map_err(decode_error)?;
            let last =
                &self.index.parts[*file.parts.last().ok_or(DecodeError::Integrity)?].description;
            if verified.written != entry.size
                || verified.reserved != 0
                || verified.auxiliary != 0
                || verified.credit != 0
            {
                return Err(DecodeError::Integrity);
            }
            verify_hash(
                last,
                verified.crc.checksum(),
                verified.blake.take(),
                key,
                verified.old,
            )?;
            Ok(())
        })();
        result.map_err(|error| crate::protection::encrypted_decode_error(error, unverified))
    }
    fn run(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<(), DecodeError> {
        let n = usize::try_from(entry.id.get() - 1).map_err(|_| DecodeError::Integrity)?;
        if self.index.rows.get(n) != Some(entry) {
            return Err(DecodeError::Integrity);
        }
        if !entry.size_known || entry.warning || entry.link {
            return Err(DecodeError::Unsupported);
        }
        if entry.directory
            || self.index.parts[self.index.files[n].parts[0]]
                .description
                .compression
                == 0
        {
            self.run_one(entry, sink, control, password)?;
            if n == self.next_file {
                self.next_file = n + 1;
            }
            return Ok(());
        }
        let description = &self.index.parts[self.index.files[n].parts[0]].description;
        let start = if n == self.next_file {
            n
        } else {
            self.algorithm.reset();
            if !description.solid {
                n
            } else {
                let mut start = n;
                while start > 0 {
                    start -= 1;
                    let d = &self.index.parts[self.index.files[start].parts[0]].description;
                    if !d.directory && d.compression != 0 && !d.solid {
                        break;
                    }
                }
                start
            }
        };
        struct Skipped<'a>(&'a mut dyn DecodedSink);
        impl DecodedSink for Skipped<'_> {
            fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
                self.0.charge_skipped(bytes.len() as u64)
            }
            fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
                self.0.charge_skipped(bytes)
            }
            fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
                self.0.charge_auxiliary(bytes)
            }
            fn write_precharged_chunk(&mut self, _bytes: &[u8]) -> Result<(), DecodeError> {
                Ok(())
            }
        }
        for prefix in start..n {
            let row = self.index.rows[prefix].clone();
            self.run_one(&row, &mut Skipped(sink), control, password)?;
        }
        self.run_one(entry, sink, control, password)?;
        self.next_file = n + 1;
        Ok(())
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
        let result = self.run(entry, sink, control, password);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Descriptor;
    use crate::tests::{new_archive, new_file};
    use xdzip_codec_api::ScanLimits;
    use xdzip_compression::crc32;
    struct Sink(Vec<u8>);
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, b: &[u8]) -> Result<(), DecodeError> {
            self.0.extend(b);
            Ok(())
        }
    }
    struct Premature;
    impl Algorithm for Premature {
        fn memory_required(&self, _: &Descriptor) -> Result<u64, DecodeError> {
            Ok(0)
        }
        fn decode(
            &mut self,
            _: &Descriptor,
            _: &mut dyn Read,
            _: &mut dyn WorkOutput,
            _: &Cancellation,
        ) -> Result<(), DecodeError> {
            Ok(())
        }
    }
    struct Oversized;
    impl Algorithm for Oversized {
        fn memory_required(&self, _: &Descriptor) -> Result<u64, DecodeError> {
            Ok(MEMORY_LIMIT + 1)
        }
        fn decode(
            &mut self,
            _: &Descriptor,
            _: &mut dyn Read,
            _: &mut dyn WorkOutput,
            _: &Cancellation,
        ) -> Result<(), DecodeError> {
            panic!("memory admission must precede decoder execution")
        }
    }
    #[test]
    fn injected_algorithm_cannot_skip_input_or_memory_admission() {
        let cancel = Cancellation::default();
        for (algorithm, expected) in [
            (
                Box::new(Premature) as Box<dyn Algorithm>,
                DecodeError::Integrity,
            ),
            (Box::new(Oversized), DecodeError::Limit),
        ] {
            let mut source = std::io::Cursor::new(new_archive(&[new_file(
                b"a",
                b"abc",
                3,
                0,
                0,
                false,
                crc32(b"abc"),
                &[],
            )]));
            let mut reader = Reader::new(&mut source, &cancel, crate::METADATA_LIMIT).unwrap();
            let index = crate::index::parse(&mut reader, ScanLimits::default(), None).unwrap();
            let row = index.rows[0].clone();
            let mut payload = Payload::with_algorithm(reader, index, algorithm);
            let mut sink = Sink(Vec::new());
            assert_eq!(payload.decode(&row, &mut sink, &cancel), Err(expected));
            assert!(sink.0.is_empty());
        }
    }
    #[test]
    fn private_work_and_input_boundaries_refuse_before_output() {
        let cancel = Cancellation::default();
        for input in [false, true] {
            let mut source = std::io::Cursor::new(new_archive(&[new_file(
                b"a",
                b"abc",
                3,
                0,
                0,
                false,
                crc32(b"abc"),
                &[],
            )]));
            let mut reader = Reader::new(&mut source, &cancel, crate::METADATA_LIMIT).unwrap();
            let index = crate::index::parse(&mut reader, ScanLimits::default(), None).unwrap();
            let row = index.rows[0].clone();
            if input {
                reader.set_limit(0);
            }
            let mut payload = Payload::new(reader, index);
            if !input {
                payload.work = INPUT_LIMIT - 1;
            }
            let mut sink = Sink(Vec::new());
            assert_eq!(
                payload.decode(&row, &mut sink, &cancel),
                Err(DecodeError::Limit)
            );
            assert!(sink.0.is_empty());
        }
    }

    #[test]
    fn reservation_is_not_external_work_or_delivery_credit() {
        #[derive(Default)]
        struct Meter {
            work: u64,
            bytes: Vec<u8>,
        }
        impl DecodedSink for Meter {
            fn write_chunk(&mut self, b: &[u8]) -> Result<(), DecodeError> {
                self.bytes.extend(b);
                Ok(())
            }
            fn charge_skipped(&mut self, n: u64) -> Result<(), DecodeError> {
                self.work += n;
                Ok(())
            }
        }
        let mut sink = Meter::default();
        let mut work = 0;
        {
            let mut verified = VerifiedSink {
                sink: &mut sink,
                expected: 3,
                written: 0,
                crc: Crc32::new(),
                blake: None,
                old: None,
                work: &mut work,
                reserved: 0,
                auxiliary: 0,
                credit: 0,
            };
            verified.reserve_work(3).unwrap();
            assert_eq!(verified.credit, 0);
            assert_eq!(*verified.work, 0);
            assert_eq!(
                verified.write_precharged_chunk(b"x"),
                Err(DecodeError::Integrity)
            );
            verified.produced_work(1).unwrap();
            verified.write_precharged_chunk(b"a").unwrap();
            assert_eq!(verified.reserved, 2);
            assert_eq!(verified.credit, 0);
            assert_eq!(
                verified.write_precharged_chunk(b"bc"),
                Err(DecodeError::Integrity)
            );
        }
        assert_eq!(work, 1);
        assert_eq!(sink.work, 1);
        assert_eq!(sink.bytes, b"a");
    }

    #[test]
    fn rejected_work_and_partial_output_cannot_fund_replay() {
        struct Reject;
        impl DecodedSink for Reject {
            fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
                panic!("rejected work cannot output")
            }
            fn charge_skipped(&mut self, _: u64) -> Result<(), DecodeError> {
                Err(DecodeError::Limit)
            }
        }
        let mut sink = Reject;
        let mut work = 0;
        let mut verified = VerifiedSink {
            sink: &mut sink,
            expected: 3,
            written: 0,
            crc: Crc32::new(),
            blake: None,
            old: None,
            work: &mut work,
            reserved: 0,
            auxiliary: 0,
            credit: 0,
        };
        verified.reserve_work(3).unwrap();
        assert_eq!(verified.produced_work(3), Err(DecodeError::Limit));
        assert_eq!(verified.credit, 0);
        assert_eq!(*verified.work, 3);
        struct Partial(Vec<u8>);
        impl DecodedSink for Partial {
            fn write_chunk(&mut self, b: &[u8]) -> Result<(), DecodeError> {
                self.0.extend(&b[..1]);
                Err(DecodeError::Output)
            }
        }
        let mut sink = Partial(Vec::new());
        let mut work = 0;
        let mut verified = VerifiedSink {
            sink: &mut sink,
            expected: 3,
            written: 0,
            crc: Crc32::new(),
            blake: None,
            old: None,
            work: &mut work,
            reserved: 0,
            auxiliary: 0,
            credit: 0,
        };
        verified.reserve_work(3).unwrap();
        verified.produced_work(3).unwrap();
        assert_eq!(
            verified.write_precharged_chunk(b"ab"),
            Err(DecodeError::Output)
        );
        assert_eq!(verified.credit, 1);
        assert_eq!(
            verified.write_precharged_chunk(b"bc"),
            Err(DecodeError::Integrity)
        );
    }
    #[test]
    fn auxiliary_work_shares_the_task_limit_and_never_mints_delivery_credit() {
        let mut sink = Sink(Vec::new());
        let mut work = INPUT_LIMIT - 10;
        let mut verified = VerifiedSink {
            sink: &mut sink,
            expected: 4,
            written: 0,
            crc: Crc32::new(),
            blake: None,
            old: None,
            work: &mut work,
            reserved: 0,
            auxiliary: 0,
            credit: 0,
        };
        verified.reserve_aux(4).unwrap();
        assert_eq!(verified.credit, 0);
        assert_eq!(*verified.work, INPUT_LIMIT - 10);
        verified.produced_aux(2).unwrap();
        assert_eq!(verified.credit, 0);
        assert_eq!(verified.auxiliary, 2);
        assert_eq!(*verified.work, INPUT_LIMIT - 8);
        assert_eq!(
            verified.write_precharged_chunk(b"a"),
            Err(DecodeError::Integrity)
        );
        assert_eq!(verified.reserve_work(7), Err(DecodeError::Limit));
        verified.produced_aux(2).unwrap();
        verified.reserve_work(4).unwrap();
        verified.produced_work(4).unwrap();
        assert_eq!(verified.credit, 4);
        assert_eq!(verified.reserve_aux(3), Err(DecodeError::Limit));
        assert_eq!(*verified.work, INPUT_LIMIT - 2);
    }
    #[test]
    fn auxiliary_host_messages_are_bounded_separate_and_rejection_keeps_actual_cost() {
        #[derive(Default)]
        struct Meter {
            decoded: u64,
            auxiliary: Vec<u64>,
            reject: bool,
        }
        impl DecodedSink for Meter {
            fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
                panic!("auxiliary work has no deliverable output")
            }
            fn charge_skipped(&mut self, n: u64) -> Result<(), DecodeError> {
                self.decoded += n;
                Ok(())
            }
            fn charge_auxiliary(&mut self, n: u64) -> Result<(), DecodeError> {
                assert!((1..=65536).contains(&n));
                self.auxiliary.push(n);
                if self.reject {
                    Err(DecodeError::Limit)
                } else {
                    Ok(())
                }
            }
        }
        let mut sink = Meter::default();
        let mut work = 0;
        for reject in [false, true] {
            sink.reject = reject;
            let mut verified = VerifiedSink {
                sink: &mut sink,
                expected: 1,
                written: 0,
                crc: Crc32::new(),
                blake: None,
                old: None,
                work: &mut work,
                reserved: 0,
                auxiliary: 0,
                credit: 0,
            };
            let n = if reject { 1 } else { 65537 };
            verified.reserve_aux(n).unwrap();
            assert_eq!(
                verified.produced_aux(n),
                if reject {
                    Err(DecodeError::Limit)
                } else {
                    Ok(())
                }
            );
            assert_eq!(
                (verified.credit, verified.written, verified.auxiliary),
                (0, 0, 0)
            );
            assert_eq!(
                verified.write_precharged_chunk(b"x"),
                Err(DecodeError::Integrity)
            );
        }
        assert_eq!(work, 65538);
        assert_eq!(sink.decoded, 0);
        assert_eq!(sink.auxiliary, [65536, 1, 1]);
    }
}

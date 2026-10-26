use crate::decode_error;
use crate::index::Descriptor;
use std::io::{self, Read};
use xdzip_codec_api::payload::{DecodeError, DecodedSink};
use xdzip_codec_api::{Cancellation, ScanError};

// Generation-specific implementations receive only the admitted packed stream.
// They own their solid state, termination and memory preflight, not file paths.
pub(crate) trait WorkOutput: DecodedSink {
    fn reserve_work(&mut self, _bytes: u64) -> Result<(), DecodeError> {
        Ok(())
    }
    fn produced_work(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.charge_skipped(bytes)
    }
    fn reserve_aux(&mut self, _bytes: u64) -> Result<(), DecodeError> {
        Ok(())
    }
    fn produced_aux(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.charge_auxiliary(bytes)
    }
    fn remaining_aux_work(&self) -> u64 {
        crate::INPUT_LIMIT
    }
}
pub(crate) trait Algorithm {
    fn reset(&mut self) {}
    #[cfg(test)]
    fn paged_history_stats(&self) -> Option<(u64, u64, u64)> {
        None
    }
    fn memory_required(&self, description: &Descriptor) -> Result<u64, DecodeError>;
    fn decode(
        &mut self,
        description: &Descriptor,
        input: &mut dyn Read,
        output: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError>;
    fn decode_padded(
        &mut self,
        description: &Descriptor,
        input: &mut dyn Read,
        output: &mut dyn WorkOutput,
        control: &Cancellation,
        padding: u8,
    ) -> Result<(), DecodeError> {
        if padding != 0 {
            return Err(DecodeError::Unsupported);
        }
        self.decode(description, input, output, control)
    }
}
pub(crate) struct Store;
pub(crate) fn input_error(error: io::Error) -> DecodeError {
    if let Some(error) = error.get_ref().and_then(|e| e.downcast_ref::<ScanError>()) {
        return decode_error(*error);
    }
    if let Some(error) = error
        .get_ref()
        .and_then(|e| e.downcast_ref::<DecodeError>())
    {
        return *error;
    }
    DecodeError::SourceIo
}

#[derive(Default)]
pub(crate) struct Engine {
    lz: crate::lz::Lz,
    lz29: crate::lz29::Lz29,
    lz15: crate::legacy_lz15::Lz15,
    lz20: crate::legacy_lz20::Lz20,
}
impl Algorithm for Engine {
    #[cfg(test)]
    fn paged_history_stats(&self) -> Option<(u64, u64, u64)> {
        self.lz.paged_history_stats()
    }
    fn reset(&mut self) {
        self.lz.reset();
        self.lz29.reset();
        self.lz15.reset();
        self.lz20.reset();
    }
    fn memory_required(&self, d: &Descriptor) -> Result<u64, DecodeError> {
        if d.compression == 0 {
            Store.memory_required(d)
        } else if matches!(d.method, 13 | 15) {
            self.lz15.memory_required(d)
        } else if matches!(d.method, 20 | 26) {
            self.lz20.memory_required(d)
        } else if d.method == 29 {
            self.lz29.memory_required(d)
        } else {
            self.lz.memory_required(d)
        }
    }
    fn decode(
        &mut self,
        d: &Descriptor,
        input: &mut dyn Read,
        output: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        if d.compression == 0 {
            Store.decode(d, input, output, control)
        } else if matches!(d.method, 13 | 15) {
            self.lz15.decode(d, input, output, control, 0)
        } else if matches!(d.method, 20 | 26) {
            self.lz20.decode(d, input, output, control, 0)
        } else if d.method == 29 {
            self.lz29.decode(d, input, output, control, 0)
        } else {
            self.lz.decode(d, input, output, control)
        }
    }
    fn decode_padded(
        &mut self,
        d: &Descriptor,
        input: &mut dyn Read,
        output: &mut dyn WorkOutput,
        control: &Cancellation,
        padding: u8,
    ) -> Result<(), DecodeError> {
        if padding > 15 || (padding != 0 && d.compression == 0) {
            return Err(DecodeError::Unsupported);
        }
        if padding == 0 {
            self.decode(d, input, output, control)
        } else if matches!(d.method, 13 | 15) {
            self.lz15.decode(d, input, output, control, padding)
        } else if matches!(d.method, 20 | 26) {
            self.lz20.decode(d, input, output, control, padding)
        } else if d.method == 29 {
            self.lz29.decode(d, input, output, control, padding)
        } else {
            self.lz.decode_padded(d, input, output, control, padding)
        }
    }
}
impl Algorithm for Store {
    fn memory_required(&self, d: &Descriptor) -> Result<u64, DecodeError> {
        if d.compression != 0 || d.encrypted || d.old_checksum.is_some() || d.hash.is_some() {
            return Err(DecodeError::Unsupported);
        }
        Ok(8192)
    }
    fn decode(
        &mut self,
        _: &Descriptor,
        input: &mut dyn Read,
        output: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        let mut bytes = [0u8; 8192];
        loop {
            control.check().map_err(decode_error)?;
            let n = match input.read(&mut bytes) {
                Err(error)
                    if error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none() =>
                {
                    continue;
                }
                result => result.map_err(input_error)?,
            };
            if n == 0 {
                return Ok(());
            }
            output.write_chunk(&bytes[..n])?;
        }
    }
}

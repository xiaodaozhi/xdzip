use crate::{Index, inspect};
use xdzip_codec_api::{
    Cancellation, ReadSource, ScanLimits,
    payload::{DecodeError, DecodedSink},
};
use xdzip_codec_deflate_member::{Member, MemberReader, Method, scan_error};

/// One frozen EGG container instance with cumulative input, work and output budgets.
pub struct Reader<'a> {
    index: Index,
    payload: MemberReader<'a>,
}
impl<'a> Reader<'a> {
    /// Rescan complete metadata and bind it to the caller's exact snapshot before output.
    pub fn open(
        source: &'a mut dyn ReadSource,
        snapshot: &Index,
        cancel: &'a Cancellation,
    ) -> Result<Self, DecodeError> {
        let result = Self::fresh(source, cancel)?;
        if &result.index != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(result)
    }
    pub(crate) fn fresh(
        source: &'a mut dyn ReadSource,
        cancel: &'a Cancellation,
    ) -> Result<Self, DecodeError> {
        let index = inspect(source, cancel, ScanLimits::default()).map_err(scan_error)?;
        let payload = MemberReader::new(source, cancel, index.input_used)?;
        Ok(Self { index, payload })
    }
    /// Return the frozen physical index.
    pub fn index(&self) -> &Index {
        &self.index
    }
    /// Decode one physical member and validate its size, termination and CRC-32.
    pub fn decode(
        &mut self,
        at: usize,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        let member = self
            .index
            .records
            .get(at)
            .ok_or(DecodeError::Integrity)
            .and_then(|record| {
                if !record.payload_supported() {
                    return Err(DecodeError::Unsupported);
                }
                Ok(Member {
                    offset: record.payload_offset,
                    packed_size: record.packed_size,
                    size: record.size,
                    crc32: record.crc32,
                    method: if record.method == 0 {
                        Method::Store
                    } else {
                        Method::Deflate
                    },
                })
            });
        self.payload.decode(member, sink, cancel)
    }
}

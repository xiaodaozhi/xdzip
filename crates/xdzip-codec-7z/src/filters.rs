//! 7z property admission for the shared container-independent byte filters.
use xdzip_codec_api::{Cancellation, payload::DecodeError};
use xdzip_compression::{ByteFilter as SharedFilter, ByteFilterKind as Kind};

/// The unchanged 7z property and cancellation contract over shared transforms.
pub struct ByteFilter(SharedFilter);
impl ByteFilter {
    /// Validate the 7z method's own property grammar before processing bytes.
    pub fn new(method: &[u8], properties: &[u8]) -> Result<Self, DecodeError> {
        let kind = match method {
            [3, 3, 1, 3] => Kind::X86,
            [3, 3, 2, 5] => Kind::Ppc,
            [3, 3, 4, 1] => Kind::Ia64,
            [3, 3, 5, 1] => Kind::Arm,
            [3, 3, 7, 1] => Kind::Thumb,
            [3, 3, 8, 5] => Kind::Sparc,
            [10] => Kind::Arm64,
            [11] => Kind::Riscv,
            [3] if properties.len() == 1 => Kind::Delta(usize::from(properties[0]) + 1),
            [2, 3, 2] => Kind::Swap(2),
            [2, 3, 4] => Kind::Swap(4),
            _ => return Err(DecodeError::Unsupported),
        };
        let position = match kind {
            Kind::Delta(_) => 0,
            Kind::Arm64 | Kind::Riscv if properties.len() == 4 => {
                u32::from_le_bytes(properties.try_into().expect("fixed property"))
            }
            _ if properties.is_empty() => 0,
            _ => return Err(DecodeError::Unsupported),
        };
        SharedFilter::new(kind, position)
            .map(Self)
            .map_err(|_| DecodeError::Unsupported)
    }
    /// Transform a prefix, preserving suffix, cancellation and terminal errors.
    pub fn transform(
        &mut self,
        bytes: &mut [u8],
        eof: bool,
        control: &Cancellation,
    ) -> Result<usize, DecodeError> {
        self.0
            .transform(bytes, eof, &mut || {
                control.check().map_err(std::io::Error::other)
            })
            .map_err(|error| {
                if error
                    .get_ref()
                    .and_then(|value| value.downcast_ref::<xdzip_codec_api::ScanError>())
                    == Some(&xdzip_codec_api::ScanError::Cancelled)
                {
                    DecodeError::Cancelled
                } else {
                    DecodeError::Integrity
                }
            })
    }
}
#[cfg(test)]
#[path = "filters_tests.rs"]
mod tests;

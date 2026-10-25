//! Task-private algorithm work. It never changes file/progress counters or
//! funds output credit, and its lifetime spans entries and password attempts.
use xdzip_codec_api::payload::DecodeError;

pub(crate) const MAX_AUXILIARY_BYTES: u64 = 20 * 1024 * 1024 * 1024;

#[derive(Default)]
pub(crate) struct AuxiliaryBudget(u64);
impl AuxiliaryBudget {
    pub(crate) fn charge(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.0 = self
            .0
            .checked_add(bytes)
            .filter(|n| *n <= MAX_AUXILIARY_BYTES)
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn used(&self) -> u64 {
        self.0
    }
}

//! Shared PPMd H statistics and deterministic nominal-capacity accounting.
//! Format wrappers supply their own range arithmetic, input and termination.
use crate::stream_io::resource;
use std::io;

#[path = "ppmd_arena.rs"]
pub(crate) mod arena;
#[path = "ppmd_model.rs"]
pub(crate) mod model;
pub use model::Model as PpmdModel;

/// Container-specific deterministic splitting rules for the nominal ledger.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PpmdCapacity {
    /// PPMd7z's 12-byte allocation and split ordering.
    SevenZip,
    /// RAR29's split order; coalescing still releases high tails first.
    Rar,
}

/// One symbol's probability selections, independent of the range variant.
/// Implementations must bound input and preserve terminal source errors.
pub trait PpmdRange: PpmdWork {
    /// Return the current count and per-frequency interval width.
    fn count(&mut self, total: u32) -> io::Result<(u32, u32)>;
    /// Commit a nonempty cumulative interval with the width from `count`.
    fn select(&mut self, unit: u32, start: u32, frequency: u32) -> io::Result<()>;
    /// Select the single-symbol interval on the fixed 16384 scale.
    fn binary(&mut self, probability: u16) -> io::Result<bool>;
}

/// Actual bounded initialization work, separate from nominal capacity restart.
/// Reservations admit future work; only `initialized` reports completed work.
pub trait PpmdWork {
    /// Check cooperative cancellation, including allocation and suffix walks.
    fn checkpoint(&mut self) -> io::Result<()> {
        Ok(())
    }
    /// Admit a future initialization without reporting it as completed work.
    fn reserve_initialization(&mut self, _bytes: u64) -> io::Result<()> {
        self.checkpoint()
    }
    /// Record bytes of storage actually initialized since the last callback.
    fn initialized(&mut self, _bytes: u64) -> io::Result<()> {
        Ok(())
    }
}
impl PpmdWork for () {}

#[cfg(test)]
#[path = "ppmd_tests.rs"]
mod tests;

/// Conservative safe-storage admission, separate from nominal model capacity.
/// Includes typed cells, coalescing scratch, history and fixed model/stack work.
pub fn ppmd_memory_required(memory: u32) -> io::Result<u64> {
    let slot =
        std::mem::size_of::<arena::Cell>() as u64 + std::mem::size_of::<arena::Address>() as u64;
    u64::from(memory / 12)
        .checked_mul(slot)
        .and_then(|value| value.checked_add(u64::from(memory)))
        .and_then(|value| value.checked_add(65536))
        .ok_or_else(resource)
}

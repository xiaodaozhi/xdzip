// crates/xdzip-codec-rar/src/paged_history.rs

//! Exact constant/owned-page history. Logical span never grants allocation credit.
use crate::algorithm::WorkOutput;
use std::mem::size_of;
use xdzip_codec_api::Cancellation;
use xdzip_codec_api::payload::DecodeError;

pub(crate) const MAX_LOGICAL_LEN: u64 = 1 << 40;
pub(crate) const MEMORY_BYTES: u64 = 512 * 1024 * 1024;
const PAGE_BYTES: usize = 64 * 1024;
const ALLOCATOR_ALLOWANCE: u64 = 16 * 1024 * 1024;
const AUX_CHUNK: usize = 8 * 1024;
const MAX_SLOTS: u64 = 8192;
// Values 0..=255 are bytes; every larger tag is one unique owned slot.
const RAW_BASE: u32 = 256;

#[derive(Default)]
struct Allocations {
    #[cfg(test)]
    remaining: Option<usize>,
}
impl Allocations {
    fn reserve<T>(&mut self, bytes: &mut Vec<T>, count: usize) -> Result<(), DecodeError> {
        #[cfg(test)]
        if let Some(remaining) = &mut self.remaining {
            *remaining = remaining.checked_sub(1).ok_or(DecodeError::Limit)?;
        }
        bytes
            .try_reserve_exact(count)
            .map_err(|_| DecodeError::Limit)
    }
}

/// All retained capacities, the inline state and allocator allowance share one cap.
pub(crate) struct PagedHistory {
    logical_len: u64,
    page_bytes: usize,
    position: u64,
    pages: Vec<u32>,
    slots: Vec<Vec<u8>>,
    free: Vec<u32>,
    next_slot: usize,
    allocated: u64,
    raw_allocated: u64,
    limit: u64,
    first: u8,
    same: bool,
    allocations: Allocations,
}

fn amount<T>(count: usize) -> Result<u64, DecodeError> {
    u64::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(size_of::<T>() as u64))
        .ok_or(DecodeError::Limit)
}
fn checked_control(control: &Cancellation) -> Result<(), DecodeError> {
    control.check().map_err(|_| DecodeError::Cancelled)
}
fn admission(
    logical_len: u64,
    page_bytes: usize,
    limit: u64,
    allowance: u64,
) -> Result<(usize, u64), DecodeError> {
    if logical_len == 0 || logical_len > MAX_LOGICAL_LEN || page_bytes == 0 {
        return Err(DecodeError::Limit);
    }
    let page_size = u64::try_from(page_bytes).map_err(|_| DecodeError::Limit)?;
    let page_count = logical_len
        .checked_sub(1)
        .and_then(|length| length.checked_div(page_size))
        .and_then(|pages| pages.checked_add(1))
        .and_then(|pages| usize::try_from(pages).ok())
        .ok_or(DecodeError::Limit)?;
    let baseline = allowance
        .checked_add(size_of::<PagedHistory>() as u64)
        .and_then(|bytes| bytes.checked_add(size_of::<PreparedWrite<'_>>() as u64))
        .filter(|bytes| *bytes <= limit)
        .ok_or(DecodeError::Limit)?;
    baseline
        .checked_add(amount::<u32>(page_count)?)
        .filter(|bytes| *bytes <= limit)
        .ok_or(DecodeError::Limit)?;
    Ok((page_count, baseline))
}
fn initialize(
    sink: &mut dyn WorkOutput,
    control: &Cancellation,
    bytes: u64,
    operation: impl FnOnce(),
) -> Result<(), DecodeError> {
    checked_control(control)?;
    if bytes > sink.remaining_aux_work() {
        return Err(DecodeError::Limit);
    }
    sink.reserve_aux(bytes)?;
    checked_control(control)?;
    operation();
    // Completed work is reported even when this callback or the next check fails.
    sink.produced_aux(bytes)?;
    checked_control(control)
}
fn reserve_empty<T>(
    vector: &mut Vec<T>,
    count: usize,
    allocated: &mut u64,
    limit: u64,
    allocations: &mut Allocations,
) -> Result<(), DecodeError> {
    if vector.capacity() != 0 || !vector.is_empty() {
        return Err(DecodeError::Integrity);
    }
    let requested = amount::<T>(count)?;
    allocated
        .checked_add(requested)
        .filter(|total| *total <= limit)
        .ok_or(DecodeError::Limit)?;
    allocations.reserve(vector, count)?;
    let actual = amount::<T>(vector.capacity())?;
    let Some(total) = allocated
        .checked_add(actual)
        .filter(|total| *total <= limit)
    else {
        // Exact reserve can expose extra capacity on an allocator. Never retain it
        // outside admission; the separately counted allowance covers allocator slack.
        *vector = Vec::new();
        return Err(DecodeError::Limit);
    };
    *allocated = total;
    Ok(())
}

impl PagedHistory {
    pub(crate) fn memory_required(logical_len: u64) -> Result<u64, DecodeError> {
        admission(logical_len, PAGE_BYTES, MEMORY_BYTES, ALLOCATOR_ALLOWANCE)?;
        // Slot metadata is selected only from the remainder after the directory;
        // every future Raw allocation is still admitted against this full cap.
        Ok(MEMORY_BYTES)
    }

    pub(crate) fn new(
        logical_len: u64,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<Self, DecodeError> {
        Self::with_limits(
            logical_len,
            PAGE_BYTES,
            MEMORY_BYTES,
            ALLOCATOR_ALLOWANCE,
            Allocations::default(),
            sink,
            control,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn with_limits(
        logical_len: u64,
        page_bytes: usize,
        limit: u64,
        allowance: u64,
        allocations: Allocations,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<Self, DecodeError> {
        checked_control(control)?;
        let (page_count, baseline) = admission(logical_len, page_bytes, limit, allowance)?;
        let page_size = u64::try_from(page_bytes).map_err(|_| DecodeError::Limit)?;
        let mut history = Self {
            logical_len,
            page_bytes,
            position: 0,
            pages: Vec::new(),
            slots: Vec::new(),
            free: Vec::new(),
            next_slot: 0,
            allocated: baseline,
            raw_allocated: 0,
            limit,
            first: 0,
            same: true,
            allocations,
        };
        reserve_empty(
            &mut history.pages,
            page_count,
            &mut history.allocated,
            limit,
            &mut history.allocations,
        )?;
        let slot_cost = page_size
            .checked_add(size_of::<Vec<u8>>() as u64)
            .and_then(|bytes| bytes.checked_add(size_of::<u32>() as u64))
            .ok_or(DecodeError::Limit)?;
        let slot_count = limit
            .checked_sub(history.allocated)
            .and_then(|bytes| bytes.checked_div(slot_cost))
            .map(|count| count.min(MAX_SLOTS).min(page_count as u64))
            .and_then(|count| usize::try_from(count).ok())
            .ok_or(DecodeError::Limit)?;
        u32::try_from(slot_count)
            .ok()
            .and_then(|count| RAW_BASE.checked_add(count))
            .ok_or(DecodeError::Limit)?;
        reserve_empty(
            &mut history.slots,
            slot_count,
            &mut history.allocated,
            limit,
            &mut history.allocations,
        )?;
        reserve_empty(
            &mut history.free,
            slot_count,
            &mut history.allocated,
            limit,
            &mut history.allocations,
        )?;
        while history.pages.len() < page_count {
            let count = (page_count - history.pages.len()).min(AUX_CHUNK / size_of::<u32>());
            let end = history
                .pages
                .len()
                .checked_add(count)
                .ok_or(DecodeError::Limit)?;
            initialize(sink, control, amount::<u32>(count)?, || {
                history.pages.resize(end, 0)
            })?;
        }
        while history.slots.len() < slot_count {
            let count = (slot_count - history.slots.len()).min(AUX_CHUNK / size_of::<Vec<u8>>());
            let end = history
                .slots
                .len()
                .checked_add(count)
                .ok_or(DecodeError::Limit)?;
            initialize(sink, control, amount::<Vec<u8>>(count)?, || {
                history.slots.resize_with(end, Vec::new);
            })?;
        }
        debug_assert!(history.allocated_bytes() <= limit);
        Ok(history)
    }

    pub(crate) fn logical_len(&self) -> u64 {
        self.logical_len
    }

    pub(crate) fn allocated_bytes(&self) -> u64 {
        self.allocated
    }

    #[cfg(test)]
    pub(crate) fn raw_allocated_bytes(&self) -> u64 {
        self.raw_allocated
    }

    fn locate(&self, index: u64) -> Result<(usize, usize), DecodeError> {
        if index >= self.logical_len {
            return Err(DecodeError::Integrity);
        }
        let page_size = u64::try_from(self.page_bytes).map_err(|_| DecodeError::Limit)?;
        let page = usize::try_from(index / page_size).map_err(|_| DecodeError::Limit)?;
        let offset = usize::try_from(index % page_size).map_err(|_| DecodeError::Limit)?;
        Ok((page, offset))
    }

    pub(crate) fn read_at(&self, index: u64) -> Result<u8, DecodeError> {
        let (page, offset) = self.locate(index)?;
        let tag = *self.pages.get(page).ok_or(DecodeError::Integrity)?;
        if tag < RAW_BASE {
            return Ok(tag as u8);
        }
        let slot = usize::try_from(tag - RAW_BASE).map_err(|_| DecodeError::Integrity)?;
        self.slots
            .get(slot)
            .and_then(|bytes| bytes.get(offset))
            .copied()
            .ok_or(DecodeError::Integrity)
    }

    fn prepared_slot(
        &mut self,
        value: u8,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<u32, DecodeError> {
        let slot = if let Some(slot) = self.free.pop() {
            slot
        } else {
            if self.next_slot >= self.slots.len() {
                return Err(DecodeError::Limit);
            }
            let slot = self.next_slot;
            let next = slot.checked_add(1).ok_or(DecodeError::Limit)?;
            let before = self.allocated;
            reserve_empty(
                &mut self.slots[slot],
                self.page_bytes,
                &mut self.allocated,
                self.limit,
                &mut self.allocations,
            )?;
            self.raw_allocated = self
                .raw_allocated
                .checked_add(
                    self.allocated
                        .checked_sub(before)
                        .ok_or(DecodeError::Limit)?,
                )
                .ok_or(DecodeError::Limit)?;
            self.next_slot = next;
            u32::try_from(slot).map_err(|_| DecodeError::Limit)?
        };
        let index = usize::try_from(slot).map_err(|_| DecodeError::Integrity)?;
        let bytes = &mut self.slots[index];
        let mut start = 0usize;
        let result = (|| {
            while start < self.page_bytes {
                let count = (self.page_bytes - start).min(AUX_CHUNK);
                let end = start.checked_add(count).ok_or(DecodeError::Limit)?;
                initialize(sink, control, count as u64, || {
                    let old_end = end.min(bytes.len());
                    bytes[start..old_end].fill(value);
                    if bytes.len() < end {
                        bytes.resize(end, value);
                    }
                })?;
                start = end;
            }
            Ok(())
        })();
        if let Err(error) = result {
            // Partial initialization is not history, but its capacity and work
            // remain retained/charged. A later reservation refills every byte.
            self.free.push(slot);
            return Err(error);
        }
        Ok(slot)
    }

    pub(crate) fn prepare_write_at<'a>(
        &'a mut self,
        index: u64,
        byte: u8,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<PreparedWrite<'a>, DecodeError> {
        checked_control(control)?;
        if index != self.position {
            return Err(DecodeError::Integrity);
        }
        let (page, offset) = self.locate(index)?;
        let page_start = index
            .checked_sub(offset as u64)
            .ok_or(DecodeError::Integrity)?;
        let valid = self
            .logical_len
            .checked_sub(page_start)
            .ok_or(DecodeError::Integrity)?
            .min(self.page_bytes as u64);
        let next = index.checked_add(1).ok_or(DecodeError::Integrity)?;
        let next = if next == self.logical_len { 0 } else { next };
        let first = if offset == 0 { byte } else { self.first };
        let same = offset == 0 || (self.same && first == byte);
        let fold = (offset as u64).checked_add(1) == Some(valid) && same;
        let tag = self.pages[page];
        let reserved = if tag < RAW_BASE && tag as u8 != byte {
            Some(self.prepared_slot(tag as u8, sink, control)?)
        } else {
            None
        };
        let slot = if tag >= RAW_BASE {
            Some(tag - RAW_BASE)
        } else {
            reserved
        };
        // The bound was checked for the entire fixed slot table before allocation.
        let raw_tag = slot.map(|slot| RAW_BASE + slot);
        debug_assert!(self.allocated_bytes() <= self.limit);
        Ok(PreparedWrite {
            history: self,
            page,
            offset,
            byte,
            slot,
            reserved,
            raw_tag,
            next,
            first,
            same,
            fold,
        })
    }
}

/// Exclusive reservation: output may fail before commit without changing history.
pub(crate) struct PreparedWrite<'a> {
    history: &'a mut PagedHistory,
    page: usize,
    offset: usize,
    byte: u8,
    slot: Option<u32>,
    reserved: Option<u32>,
    raw_tag: Option<u32>,
    next: u64,
    first: u8,
    same: bool,
    fold: bool,
}
impl PreparedWrite<'_> {
    /// All indexing and free-list capacity are fixed by the exclusive reservation.
    pub(crate) fn commit(mut self) {
        if let Some(slot) = self.slot {
            self.history.slots[slot as usize][self.offset] = self.byte;
            if self.fold {
                self.history.pages[self.page] = u32::from(self.first);
                self.history.free.push(slot);
            } else if let Some(tag) = self.raw_tag {
                self.history.pages[self.page] = tag;
            }
        }
        self.reserved = None;
        self.history.position = self.next;
        self.history.first = self.first;
        self.history.same = self.same;
    }
}
impl Drop for PreparedWrite<'_> {
    fn drop(&mut self) {
        if let Some(slot) = self.reserved.take() {
            self.history.free.push(slot);
        }
    }
}

#[cfg(test)]
#[path = "paged_history_tests.rs"]
mod tests;

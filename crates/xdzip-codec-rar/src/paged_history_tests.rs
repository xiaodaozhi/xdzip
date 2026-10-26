// crates/xdzip-codec-rar/src/paged_history_tests.rs

use super::*;
use xdzip_codec_api::payload::DecodedSink;

struct Meter<'a> {
    aux: u64,
    limit: u64,
    reserves: u64,
    produced: u64,
    reject_reserve: Option<u64>,
    reject_produced: Option<u64>,
    cancel: Option<(&'a Cancellation, u64)>,
    maximum_chunk: u64,
    work: u64,
    files: u64,
}
impl Default for Meter<'_> {
    fn default() -> Self {
        Self {
            aux: 0,
            limit: u64::MAX,
            reserves: 0,
            produced: 0,
            reject_reserve: None,
            reject_produced: None,
            cancel: None,
            maximum_chunk: 0,
            work: 0,
            files: 0,
        }
    }
}
impl DecodedSink for Meter<'_> {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.files += bytes.len() as u64;
        Ok(())
    }
    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.work += bytes;
        Ok(())
    }
}
impl WorkOutput for Meter<'_> {
    fn reserve_aux(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.reserves += 1;
        self.maximum_chunk = self.maximum_chunk.max(bytes);
        if self.reject_reserve == Some(self.reserves) || bytes > self.remaining_aux_work() {
            return Err(DecodeError::Limit);
        }
        Ok(())
    }
    fn produced_aux(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.aux += bytes;
        self.produced += 1;
        if let Some((control, after)) = self.cancel
            && after == self.produced
        {
            control.cancel();
        }
        if self.reject_produced == Some(self.produced) {
            return Err(DecodeError::Output);
        }
        Ok(())
    }
    fn remaining_aux_work(&self) -> u64 {
        self.limit.saturating_sub(self.aux)
    }
}
fn small(length: u64, page: usize, sink: &mut Meter<'_>) -> PagedHistory {
    PagedHistory::with_limits(
        length,
        page,
        2 * 1024 * 1024,
        0,
        Allocations::default(),
        sink,
        &Cancellation::default(),
    )
    .unwrap()
}
fn put(history: &mut PagedHistory, byte: u8, sink: &mut Meter<'_>) {
    let position = history.position;
    history
        .prepare_write_at(position, byte, sink, &Cancellation::default())
        .unwrap()
        .commit();
}
fn assert_no_file_credit(meter: &Meter<'_>) {
    assert_eq!((meter.work, meter.files), (0, 0));
    assert!(meter.maximum_chunk <= AUX_CHUNK as u64);
}
fn actual_ledger(history: &PagedHistory, allowance: u64) -> u64 {
    allowance
        + size_of::<PagedHistory>() as u64
        + size_of::<PreparedWrite<'_>>() as u64
        + (history.pages.capacity() * size_of::<u32>()) as u64
        + (history.slots.capacity() * size_of::<Vec<u8>>()) as u64
        + (history.free.capacity() * size_of::<u32>()) as u64
        + history
            .slots
            .iter()
            .map(|bytes| bytes.capacity() as u64)
            .sum::<u64>()
}

#[test]
fn random_sequential_literals_and_overlapping_copies_match_an_independent_vec() {
    for length in [1usize, 7, 8, 9, 23, 31, 64, 81] {
        let mut meter = Meter::default();
        let mut history = small(length as u64, 8, &mut meter);
        let mut oracle = vec![0u8; length];
        let mut position = 0usize;
        let mut available = 0usize;
        let mut random = 0xd3a2_9074_d619_8b5cu64 ^ length as u64;
        for _ in 0..1500 {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            let literal = available == 0 || random.is_multiple_of(3);
            let distance = if literal {
                0
            } else {
                1 + random as usize % available
            };
            let count = if literal {
                1
            } else {
                1 + ((random >> 32) as usize % 37)
            };
            for _ in 0..count {
                let byte = if literal {
                    (random >> 21) as u8
                } else {
                    let source = (position + length - distance) % length;
                    assert_eq!(history.read_at(source as u64).unwrap(), oracle[source]);
                    oracle[source]
                };
                put(&mut history, byte, &mut meter);
                oracle[position] = byte;
                position = (position + 1) % length;
                available = (available + 1).min(length);
            }
            assert_eq!(history.position, position as u64);
            for (index, &byte) in oracle.iter().enumerate() {
                assert_eq!(history.read_at(index as u64), Ok(byte));
            }
            assert_eq!(history.allocated_bytes(), actual_ledger(&history, 0));
        }
        assert_no_file_credit(&meter);
    }
}

#[test]
fn constants_promote_and_fold_only_after_the_complete_valid_page() {
    let mut meter = Meter::default();
    let mut history = small(21, 8, &mut meter);
    let initial = history.allocated_bytes();
    for _ in 0..8 {
        put(&mut history, 0, &mut meter);
    }
    assert_eq!(history.raw_allocated_bytes(), 0);
    for _ in 0..4 {
        put(&mut history, 255, &mut meter);
    }
    assert!(history.pages[1] >= RAW_BASE);
    assert_eq!(history.free.len(), 0);
    for index in 8..12 {
        assert_eq!(history.read_at(index), Ok(255));
    }
    for index in 12..16 {
        assert_eq!(history.read_at(index), Ok(0));
    }
    // A solid boundary changes neither this partial rewrite nor the old suffix.
    let retained = history.allocated_bytes();
    for _ in 0..4 {
        put(&mut history, 255, &mut meter);
    }
    assert_eq!(history.pages[1], 255);
    assert_eq!(history.free.len(), 1);
    assert_eq!(history.allocated_bytes(), retained);
    for _ in 0..5 {
        put(&mut history, 1, &mut meter);
    }
    assert_eq!(history.pages[2], 1);
    assert_eq!(history.position, 0);
    assert_eq!(history.allocated_bytes(), retained);
    assert_eq!(history.raw_allocated_bytes(), 8);
    assert_eq!(retained - initial, 8);
    assert_eq!(history.read_at(21), Err(DecodeError::Integrity));
    for _ in 0..8 {
        put(&mut history, 7, &mut meter);
    }
    assert_eq!(history.pages[0], 7);
    assert_eq!(history.allocated_bytes(), retained);
    assert_eq!(history.allocated_bytes(), actual_ledger(&history, 0));
    assert_no_file_credit(&meter);
}

#[test]
fn mixed_page_prefix_cannot_fold_or_erase_the_previous_suffix() {
    let mut meter = Meter::default();
    let mut history = small(11, 8, &mut meter);
    for byte in 1..=11 {
        put(&mut history, byte, &mut meter);
    }
    assert!(history.pages.iter().all(|tag| *tag >= RAW_BASE));
    for _ in 0..4 {
        put(&mut history, 91, &mut meter);
    }
    assert!(history.pages[0] >= RAW_BASE);
    for index in 4..11 {
        assert_eq!(history.read_at(index), Ok(index as u8 + 1));
    }
    for _ in 0..4 {
        put(&mut history, 91, &mut meter);
    }
    assert_eq!(history.pages[0], 91);
    assert!(history.pages[1] >= RAW_BASE);
    for _ in 0..3 {
        put(&mut history, 255, &mut meter);
    }
    assert_eq!(history.pages[1], 255);
    assert_no_file_credit(&meter);
}

#[test]
fn uncommitted_tokens_preserve_bytes_cursor_and_rewrite_candidate() {
    let mut meter = Meter::default();
    let mut history = small(16, 8, &mut meter);
    let initial = history.allocated_bytes();
    let control = Cancellation::default();
    drop(
        history
            .prepare_write_at(0, 7, &mut meter, &control)
            .unwrap(),
    );
    assert_eq!(
        (history.position, history.first, history.same),
        (0, 0, true)
    );
    assert_eq!(history.pages[0], 0);
    assert_eq!(history.read_at(0), Ok(0));
    assert_eq!(history.free.len(), 1);
    assert_eq!(history.allocated_bytes(), initial + 8);
    let retained = history.allocated_bytes();
    put(&mut history, 8, &mut meter);
    assert_eq!(history.allocated_bytes(), retained);
    drop(
        history
            .prepare_write_at(1, 9, &mut meter, &control)
            .unwrap(),
    );
    assert_eq!(history.position, 1);
    assert_eq!(history.read_at(1), Ok(0));
    assert_eq!((history.first, history.same), (8, true));
    for _ in 1..8 {
        put(&mut history, 8, &mut meter);
    }
    assert_eq!(history.pages[0], 8);
    assert_eq!(history.free.len(), 1);
    assert_eq!(history.allocated_bytes(), retained);
    assert_no_file_credit(&meter);
}

#[test]
fn commit_and_free_slot_reuse_do_not_allocate() {
    let mut meter = Meter::default();
    let mut history = small(16, 8, &mut meter);
    for _ in 0..7 {
        put(&mut history, 5, &mut meter);
    }
    let token = history
        .prepare_write_at(7, 5, &mut meter, &Cancellation::default())
        .unwrap();
    token.history.allocations.remaining = Some(0);
    token.commit();
    assert_eq!(history.pages[0], 5);
    let memory = history.allocated_bytes();
    for _ in 0..8 {
        put(&mut history, 91, &mut meter);
    }
    assert_eq!(history.pages[1], 91);
    assert_eq!(history.allocated_bytes(), memory);
    assert_eq!(history.allocations.remaining, Some(0));
    assert_no_file_credit(&meter);
}

#[test]
fn non_power_window_and_distance_equal_to_window_keep_old_current_bytes() {
    let mut meter = Meter::default();
    let mut history = small(19, 8, &mut meter);
    let original: Vec<_> = (1..=19).collect();
    for &byte in &original {
        put(&mut history, byte, &mut meter);
    }
    for &byte in original.iter().cycle().take(73) {
        let source = history.position;
        assert_eq!(history.read_at(source), Ok(byte));
        put(&mut history, byte, &mut meter);
    }
    // distance=1 and length>distance must immediately see the just-written byte.
    for _ in 0..61 {
        let source = if history.position == 0 {
            18
        } else {
            history.position - 1
        };
        let byte = history.read_at(source).unwrap();
        assert_eq!(byte, original[(73 - 1) % original.len()]);
        put(&mut history, byte, &mut meter);
    }
    assert_no_file_credit(&meter);
}

#[test]
fn all_capacities_and_the_inline_token_are_counted_at_the_exact_admission_edge() {
    let baseline = (size_of::<PagedHistory>() + size_of::<PreparedWrite<'_>>()) as u64;
    let page = 16u64;
    let directory = 4 * size_of::<u32>() as u64;
    let slot = page + size_of::<Vec<u8>>() as u64 + size_of::<u32>() as u64;
    let limit = baseline + directory + slot * 2;
    let mut meter = Meter::default();
    let mut history = PagedHistory::with_limits(
        64,
        page as usize,
        limit,
        0,
        Allocations::default(),
        &mut meter,
        &Cancellation::default(),
    )
    .unwrap();
    assert_eq!(history.slots.len(), 2);
    for index in 0..32 {
        put(&mut history, (index % 2 + 1) as u8, &mut meter);
    }
    assert_eq!(history.allocated_bytes(), limit);
    assert_eq!(history.allocated_bytes(), actual_ledger(&history, 0));
    assert_eq!(history.raw_allocated_bytes(), 32);
    let aux = meter.aux;
    assert!(matches!(
        history.prepare_write_at(32, 9, &mut meter, &Cancellation::default()),
        Err(DecodeError::Limit)
    ));
    assert_eq!(history.position, 32);
    assert_eq!(history.read_at(32), Ok(0));
    assert_eq!(history.allocated_bytes(), limit);
    assert_eq!(meter.aux, aux);
    assert_no_file_credit(&meter);
    assert!(matches!(
        PagedHistory::with_limits(
            64,
            page as usize,
            baseline + directory - 1,
            0,
            Allocations::default(),
            &mut meter,
            &Cancellation::default(),
        ),
        Err(DecodeError::Limit)
    ));
}

#[test]
fn fallible_allocation_injection_never_changes_logical_history() {
    for remaining in 0..3 {
        let mut meter = Meter::default();
        let result = PagedHistory::with_limits(
            64,
            8,
            4096,
            0,
            Allocations {
                remaining: Some(remaining),
            },
            &mut meter,
            &Cancellation::default(),
        );
        assert!(matches!(result, Err(DecodeError::Limit)));
        assert_eq!(meter.aux, 0);
        assert_no_file_credit(&meter);
    }
    let mut meter = Meter::default();
    let mut history = PagedHistory::with_limits(
        64,
        8,
        4096,
        0,
        Allocations { remaining: Some(3) },
        &mut meter,
        &Cancellation::default(),
    )
    .unwrap();
    let allocated = history.allocated_bytes();
    let aux = meter.aux;
    assert!(matches!(
        history.prepare_write_at(0, 9, &mut meter, &Cancellation::default()),
        Err(DecodeError::Limit)
    ));
    assert_eq!(history.read_at(0), Ok(0));
    assert_eq!(history.position, 0);
    assert_eq!(history.allocated_bytes(), allocated);
    assert_eq!(meter.aux, aux);
    assert_no_file_credit(&meter);
}

#[test]
fn initial_aux_cancellation_and_rejection_charge_only_completed_chunks() {
    let control = Cancellation::default();
    let mut meter = Meter {
        cancel: Some((&control, 1)),
        ..Default::default()
    };
    assert!(matches!(
        PagedHistory::with_limits(
            3000,
            1,
            2 * 1024 * 1024,
            0,
            Allocations::default(),
            &mut meter,
            &control,
        ),
        Err(DecodeError::Cancelled)
    ));
    assert_eq!(meter.aux, AUX_CHUNK as u64);
    assert_no_file_credit(&meter);
    let mut meter = Meter {
        reject_reserve: Some(2),
        ..Default::default()
    };
    assert!(matches!(
        PagedHistory::with_limits(
            3000,
            1,
            2 * 1024 * 1024,
            0,
            Allocations::default(),
            &mut meter,
            &Cancellation::default(),
        ),
        Err(DecodeError::Limit)
    ));
    assert_eq!(meter.aux, AUX_CHUNK as u64);
    assert_no_file_credit(&meter);
}

#[test]
fn cancelled_or_rejected_raw_initialization_keeps_real_work_and_capacity() {
    let page = AUX_CHUNK * 3;
    for mode in 0..4 {
        let mut initial = Meter::default();
        let mut history = small((page * 2) as u64, page, &mut initial);
        let control = Cancellation::default();
        let mut meter = match mode {
            0 => Meter {
                cancel: Some((&control, 1)),
                ..Default::default()
            },
            1 => Meter {
                reject_reserve: Some(2),
                ..Default::default()
            },
            2 => Meter {
                reject_produced: Some(1),
                ..Default::default()
            },
            _ => Meter {
                limit: (AUX_CHUNK - 1) as u64,
                ..Default::default()
            },
        };
        let before = history.allocated_bytes();
        let expected = match mode {
            0 => DecodeError::Cancelled,
            2 => DecodeError::Output,
            _ => DecodeError::Limit,
        };
        assert!(
            matches!(history.prepare_write_at(0, 33, &mut meter, &control), Err(error) if error == expected)
        );
        assert_eq!(history.allocated_bytes(), before + page as u64);
        assert_eq!(history.raw_allocated_bytes(), page as u64);
        assert_eq!(history.allocated_bytes(), actual_ledger(&history, 0));
        assert_eq!(history.position, 0);
        assert_eq!(history.pages[0], 0);
        assert_eq!(history.read_at(0), Ok(0));
        assert_eq!(history.free.len(), 1);
        assert_eq!(meter.aux, if mode == 3 { 0 } else { AUX_CHUNK as u64 });
        assert_no_file_credit(&meter);
    }
}

#[test]
fn no_op_constant_writes_still_check_cancel_and_sequential_indices() {
    let mut meter = Meter::default();
    let mut history = small(19, 8, &mut meter);
    assert!(matches!(
        history.prepare_write_at(1, 0, &mut meter, &Cancellation::default()),
        Err(DecodeError::Integrity)
    ));
    let control = Cancellation::default();
    control.cancel();
    let aux = meter.aux;
    assert!(matches!(
        history.prepare_write_at(0, 0, &mut meter, &control),
        Err(DecodeError::Cancelled)
    ));
    assert_eq!(history.position, 0);
    assert_eq!(meter.aux, aux);
    assert_eq!(history.raw_allocated_bytes(), 0);
    assert_no_file_credit(&meter);
}

#[test]
fn u64_constant_state_crosses_two_to_the_32_without_allocating_the_span() {
    let length = (1u64 << 32) + 19;
    let mut meter = Meter::default();
    let mut history = PagedHistory::new(length, &mut meter, &Cancellation::default()).unwrap();
    // This is an exact constructed mathematical state, not an archive decode.
    history.pages.fill(0xa5);
    history.position = length - 7;
    history.first = 0xa5;
    history.same = true;
    let allocated = history.allocated_bytes();
    for index in [0, (1u64 << 32) - 1, 1u64 << 32, length - 1] {
        assert_eq!(history.read_at(index), Ok(0xa5));
    }
    for distance in [1, 19, (1u64 << 32) + 1, length] {
        let position = history.position;
        let source = if position >= distance {
            position - distance
        } else {
            length - (distance - position)
        };
        assert!(source < length);
        assert_eq!(history.read_at(source), Ok(0xa5));
    }
    for _ in 0..7 {
        put(&mut history, 0xa5, &mut meter);
    }
    assert_eq!(history.position, 0);
    assert_eq!(history.logical_len(), length);
    assert_eq!(history.allocated_bytes(), allocated);
    assert_eq!(history.raw_allocated_bytes(), 0);
    assert_eq!(history.read_at(length), Err(DecodeError::Integrity));
    assert_eq!(
        history.allocated_bytes(),
        actual_ledger(&history, ALLOCATOR_ALLOWANCE)
    );
    assert_no_file_credit(&meter);
}

#[test]
fn logical_and_directory_math_reject_overflow_without_allocation() {
    assert_eq!(
        PagedHistory::memory_required(MAX_LOGICAL_LEN),
        Ok(MEMORY_BYTES)
    );
    let (pages, _) = admission(
        MAX_LOGICAL_LEN,
        PAGE_BYTES,
        MEMORY_BYTES,
        ALLOCATOR_ALLOWANCE,
    )
    .unwrap();
    assert_eq!(pages as u64, 1 << 24);
    assert_eq!(amount::<u32>(pages), Ok(64 * 1024 * 1024));
    for length in [0, MAX_LOGICAL_LEN + 1, u64::MAX] {
        assert_eq!(
            PagedHistory::memory_required(length),
            Err(DecodeError::Limit)
        );
    }
    assert_eq!(admission(1, 0, MEMORY_BYTES, 0), Err(DecodeError::Limit));
    assert_eq!(
        admission(1, 1, MEMORY_BYTES, u64::MAX),
        Err(DecodeError::Limit)
    );
    assert_eq!(amount::<u32>(usize::MAX), Err(DecodeError::Limit));
}

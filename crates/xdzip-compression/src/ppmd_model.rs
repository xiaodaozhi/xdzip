//! Ordered adaptive contexts and delayed successor growth, separate from the
//! range coder and virtual-capacity ledger. All handles are bounds checked.
use super::{
    PpmdCapacity, PpmdRange, PpmdWork,
    arena::{self, Address, Arena, Context, Link, State, Storage},
};
use crate::invalid;
use std::io;

#[derive(Clone, Copy)]
struct See {
    sum: u16,
    shift: u8,
    count: u8,
}
const BINARY_SEEDS: [u16; 8] = [
    0x3cdd, 0x1f3f, 0x59bf, 0x48f3, 0x64a1, 0x5abc, 0x6632, 0x6051,
];
const ESCAPES: [u8; 16] = [25, 14, 9, 7, 5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2, 2];

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Diagnostics {
    pub symbols: u64,
    pub restarts: u64,
    pub first_restarts: [u64; 16],
    pub last_restart: u64,
    pub arena: arena::Diagnostics,
}

/// Adaptive PPMd H context model, without format range or EOF policy.
pub struct Model {
    arena: Arena,
    binary: [[u16; 64]; 128],
    see: [[See; 16]; 25],
    order: u8,
    fall: u16,
    run: i32,
    initial_run: i32,
    previous_success: u8,
    previous_symbol: u8,
    initial_escape: u8,
    root: Address,
    current: Address,
    #[cfg(test)]
    symbols: u64,
    #[cfg(test)]
    restarts: u64,
    #[cfg(test)]
    first_restarts: [u64; 16],
    #[cfg(test)]
    last_restart: u64,
}
impl Model {
    /// Create a model with the 7z nominal-capacity profile.
    pub fn new(order: u8, memory: u32) -> io::Result<Self> {
        Self::with_profile(order, memory, PpmdCapacity::SevenZip)
    }
    /// Create an admitted model using an explicit format capacity profile.
    /// Callers must first admit `ppmd_memory_required(memory)` against their
    /// live task budget; allocation failures remain terminal host failures.
    pub fn with_profile(order: u8, memory: u32, profile: PpmdCapacity) -> io::Result<Self> {
        Self::with_work(order, memory, profile, &mut ())
    }
    /// Create a model with per-chunk admission, actual initialization work and
    /// cancellation. A failed allocation still reports completed prior chunks.
    pub fn with_work<W: PpmdWork + ?Sized>(
        order: u8,
        memory: u32,
        profile: PpmdCapacity,
        work: &mut W,
    ) -> io::Result<Self> {
        if !(2..=64).contains(&order) || memory < 2048 {
            return Err(invalid());
        }
        let cells = u64::from(memory / 12) * std::mem::size_of::<arena::Cell>() as u64;
        let scratch = u64::from(memory / 12) * std::mem::size_of::<Address>() as u64;
        work.reserve_initialization(
            2 * cells + scratch + u64::from(memory) + 2 * Self::table_work() + Self::root_work(),
        )?;
        let mut model = Self {
            arena: Arena::with_work(memory, profile, work)?,
            binary: [[0; 64]; 128],
            see: [[See {
                sum: 0,
                shift: 3,
                count: 4,
            }; 16]; 25],
            order,
            fall: u16::from(order),
            run: 0,
            initial_run: -i32::from(order.min(12)) - 1,
            previous_success: 0,
            previous_symbol: 0,
            initial_escape: 0,
            root: 0,
            current: 0,
            #[cfg(test)]
            symbols: 0,
            #[cfg(test)]
            restarts: 0,
            #[cfg(test)]
            first_restarts: [0; 16],
            #[cfg(test)]
            last_restart: 0,
        };
        work.initialized(Self::table_work())?;
        model.restart(work)?;
        Ok(model)
    }
    fn table_work() -> u64 {
        (std::mem::size_of::<[[u16; 64]; 128]>() + std::mem::size_of::<[[See; 16]; 25]>()) as u64
    }
    fn root_work() -> u64 {
        // Each state write updates its safe two-state cell; include root too.
        (257 * std::mem::size_of::<arena::Cell>()) as u64
    }
    fn restart<W: PpmdWork + ?Sized>(&mut self, work: &mut W) -> io::Result<()> {
        work.reserve_initialization(
            self.arena.reset_work() + Self::table_work() + Self::root_work(),
        )?;
        #[cfg(test)]
        if self.symbols != 0 {
            if (self.restarts as usize) < self.first_restarts.len() {
                self.first_restarts[self.restarts as usize] = self.symbols;
            }
            self.last_restart = self.symbols;
            self.restarts += 1;
        }
        self.arena.reset(work)?;
        for (frequency, row) in self.binary.iter_mut().enumerate() {
            work.reserve_initialization(std::mem::size_of_val(row) as u64)?;
            for (column, probability) in row.iter_mut().enumerate() {
                *probability = 16384 - BINARY_SEEDS[column % 8] / (frequency as u16 + 2);
            }
            work.initialized(std::mem::size_of_val(row) as u64)?;
        }
        for (row, values) in self.see.iter_mut().enumerate() {
            work.reserve_initialization(std::mem::size_of_val(values) as u64)?;
            values.fill(See {
                sum: (5 * row as u16 + 10) * 8,
                shift: 3,
                count: 4,
            });
            work.initialized(std::mem::size_of_val(values) as u64)?;
        }
        work.reserve_initialization(Self::root_work())?;
        let root = self.arena.context_slot()?.ok_or_else(invalid)?;
        let base = self.arena.vector(37)?.ok_or_else(invalid)?;
        self.arena.set_context(
            root,
            Context {
                suffix: 0,
                count: 256,
                total: 257,
                storage: Storage::Multiple { base, class: 37 },
            },
        )?;
        for symbol in 0..256 {
            self.arena.set_state(
                root,
                symbol,
                State {
                    symbol: symbol as u8,
                    frequency: 1,
                    next: Link::Absent,
                },
            )?;
        }
        work.initialized(Self::root_work())?;
        self.fall = u16::from(self.order);
        self.run = self.initial_run;
        self.previous_success = 0;
        self.previous_symbol = 0;
        self.initial_escape = 0;
        self.root = root;
        self.current = root;
        Ok(())
    }
    fn selected(&self, context: Address, symbol: u8) -> io::Result<(usize, State)> {
        let index = self.arena.find(context, symbol)?;
        Ok((index, self.arena.state(context, index)?))
    }
    fn successor(&mut self, context: Address, symbol: u8, next: Link) -> io::Result<()> {
        let (index, mut state) = self.selected(context, symbol)?;
        state.next = next;
        self.arena.set_state(context, index, state)
    }
    fn total_add(&mut self, address: Address, amount: u16) -> io::Result<()> {
        let mut context = self.arena.context(address)?;
        context.total = context.total.wrapping_add(amount);
        self.arena.set_context(address, context)
    }
    fn swap(&mut self, address: Address, a: usize, b: usize) -> io::Result<()> {
        let first = self.arena.state(address, a)?;
        let second = self.arena.state(address, b)?;
        self.arena.set_state(address, a, second)?;
        self.arena.set_state(address, b, first)
    }
    fn rescale(&mut self, address: Address, symbol: u8) -> io::Result<()> {
        let mut context = self.arena.context(address)?;
        let Storage::Multiple {
            base,
            class: old_class,
        } = context.storage
        else {
            return Err(invalid());
        };
        let mut states = [State::EMPTY; 256];
        let count = usize::from(context.count);
        let selected = self.arena.find(address, symbol)?;
        let mut sum = 0u32;
        for (index, target) in states[..count].iter_mut().enumerate() {
            *target = self.arena.state(address, index)?;
            sum += u32::from(target.frequency);
        }
        let mut escape = u32::from(context.total)
            .checked_sub(sum)
            .ok_or_else(invalid)?;
        let chosen = states[selected];
        states.copy_within(0..selected, 1);
        states[0] = chosen;
        let adjustment = u16::from(self.fall != 0);
        for (index, state) in states[..count].iter_mut().enumerate() {
            state.frequency =
                ((u16::from(state.frequency) + adjustment + if index == 0 { 4 } else { 0 }) / 2)
                    as u8;
        }
        // Stable descending insertion preserves equal-frequency identities.
        for index in 1..count {
            let mut target = index;
            while target != 0 && states[target].frequency > states[target - 1].frequency {
                states.swap(target, target - 1);
                target -= 1;
            }
        }
        let retained = states[..count]
            .iter()
            .position(|state| state.frequency == 0)
            .unwrap_or(count);
        if retained == 0 {
            return Err(invalid());
        }
        escape += (count - retained) as u32;
        if retained == 1 {
            let mut frequency = u32::from(states[0].frequency);
            loop {
                escape /= 2;
                frequency = frequency.div_ceil(2);
                if escape <= 1 {
                    break;
                }
            }
            states[0].frequency = frequency as u8;
            context.count = 1;
            context.total = 0;
            context.storage = Storage::Single(states[0]);
            self.arena.set_context(address, context)?;
            self.arena.release(base, old_class)?;
            return Ok(());
        }
        context.count = retained as u16;
        context.total = (states[..retained]
            .iter()
            .map(|state| u32::from(state.frequency))
            .sum::<u32>()
            + escape.div_ceil(2)) as u16;
        self.arena.set_context(address, context)?;
        for (index, &state) in states[..retained].iter().enumerate() {
            self.arena.set_state(address, index, state)?;
        }
        let new_class = arena::class(retained.div_ceil(2))?;
        let new_base = self.arena.shrink(base, old_class, new_class)?;
        context.storage = Storage::Multiple {
            base: new_base,
            class: new_class,
        };
        self.arena.set_context(address, context)
    }
    fn hit(&mut self, address: Address, index: usize, escaped: bool) -> io::Result<u8> {
        let context = self.arena.context(address)?;
        let mut state = self.arena.state(address, index)?;
        let symbol = state.symbol;
        if context.count == 1 {
            self.previous_success = 1;
            self.run = self.run.wrapping_add(1);
            if state.frequency < 128 {
                state.frequency += 1;
            }
            self.arena.set_state(address, 0, state)?;
            return Ok(symbol);
        }
        if escaped {
            self.run = self.initial_run;
        } else if index == 0 {
            self.previous_success =
                u8::from(2 * u32::from(state.frequency) > u32::from(context.total));
            self.run = self.run.wrapping_add(i32::from(self.previous_success));
        } else {
            self.previous_success = 0;
        }
        state.frequency = state.frequency.wrapping_add(4);
        self.arena.set_state(address, index, state)?;
        self.total_add(address, 4)?;
        let mut should_rescale = escaped || index == 0;
        if !escaped
            && index != 0
            && state.frequency > self.arena.state(address, index - 1)?.frequency
        {
            self.swap(address, index, index - 1)?;
            should_rescale = true;
        }
        if should_rescale && state.frequency > 124 {
            self.rescale(address, symbol)?;
        }
        Ok(symbol)
    }
    fn suffix_nudge(&mut self, address: Address, symbol: u8) -> io::Result<()> {
        let (_, selected) = self.selected(address, symbol)?;
        let suffix = self.arena.context(address)?.suffix;
        if selected.frequency >= 31 || suffix == 0 {
            return Ok(());
        }
        let context = self.arena.context(suffix)?;
        let (mut index, mut state) = self.selected(suffix, symbol)?;
        if context.count == 1 {
            if state.frequency < 32 {
                state.frequency += 1;
                self.arena.set_state(suffix, index, state)?;
            }
            return Ok(());
        }
        if index != 0 && state.frequency >= self.arena.state(suffix, index - 1)?.frequency {
            self.swap(suffix, index, index - 1)?;
            index -= 1;
        }
        if state.frequency < 115 {
            state.frequency += 2;
            self.arena.set_state(suffix, index, state)?;
            self.total_add(suffix, 2)?;
        }
        Ok(())
    }
    fn materialize(
        &mut self,
        address: Address,
        symbol: u8,
        raw: u32,
    ) -> io::Result<Option<Address>> {
        let mut chain = [0u32; 65];
        let mut length = 0;
        if self.fall != 0 {
            chain[length] = address;
            length += 1;
        }
        let mut cursor = self.arena.context(address)?.suffix;
        let mut base = self.root;
        while cursor != 0 {
            let (_, state) = self.selected(cursor, symbol)?;
            if state.next != Link::Raw(raw) {
                let Link::Context(context) = state.next else {
                    return Err(invalid());
                };
                base = context;
                break;
            }
            if length == chain.len() {
                return Err(invalid());
            }
            chain[length] = cursor;
            length += 1;
            cursor = self.arena.context(cursor)?.suffix;
        }
        if length == 0 {
            return Ok(Some(base));
        }
        let byte = self.arena.history(raw)?;
        let context = self.arena.context(base)?;
        let frequency = if context.count == 1 {
            self.arena.state(base, 0)?.frequency
        } else {
            let (_, state) = self.selected(base, byte)?;
            let c = u32::from(state.frequency)
                .checked_sub(1)
                .ok_or_else(invalid)?;
            let s = u32::from(context.total)
                .checked_sub(u32::from(context.count) + c)
                .ok_or_else(invalid)?;
            if s == 0 {
                return Err(invalid());
            }
            (1 + if 2 * c <= s {
                u32::from(5 * c > s)
            } else {
                (2 * c + s - 1) / (2 * s) + 1
            }) as u8
        };
        for &parent in chain[..length].iter().rev() {
            let Some(created) = self.arena.context_slot()? else {
                return Ok(None);
            };
            self.arena.set_context(
                created,
                Context {
                    suffix: base,
                    count: 1,
                    total: 0,
                    storage: Storage::Single(State {
                        symbol: byte,
                        frequency,
                        next: Link::Raw(raw.checked_add(1).ok_or_else(invalid)?),
                    }),
                },
            )?;
            self.successor(parent, symbol, Link::Context(created))?;
            base = created;
        }
        Ok(Some(base))
    }
    fn grow(
        &mut self,
        address: Address,
        symbol: u8,
        next: Link,
        n: u16,
        frequency: u8,
        escape: u32,
    ) -> io::Result<bool> {
        let mut context = self.arena.context(address)?;
        let count = usize::from(context.count);
        if count >= 256 {
            return Err(invalid());
        }
        let base_total = match context.storage {
            Storage::Single(mut state) => {
                let Some(base) = self.arena.vector(0)? else {
                    return Ok(false);
                };
                state.frequency = if state.frequency < 30 {
                    state.frequency * 2
                } else {
                    120
                };
                let total =
                    u32::from(state.frequency) + u32::from(self.initial_escape) + u32::from(n > 3);
                context.storage = Storage::Multiple { base, class: 0 };
                context.count = 2;
                self.arena.set_context(address, context)?;
                self.arena.set_state(address, 0, state)?;
                total
            }
            Storage::Multiple { base, class } => {
                let grade = arena::class((count + 1).div_ceil(2))?;
                let Some(base) = self.arena.expand(base, class, grade)? else {
                    return Ok(false);
                };
                let total = u32::from(context.total)
                    + u32::from(2 * context.count < n)
                    + 2 * u32::from(
                        4 * context.count <= n
                            && u32::from(context.total) <= 8 * u32::from(context.count),
                    );
                context.storage = Storage::Multiple { base, class: grade };
                context.count += 1;
                self.arena.set_context(address, context)?;
                total
            }
        };
        let c = 2 * (u64::from(base_total) + 6) * u64::from(frequency);
        let s = u64::from(escape) + u64::from(base_total);
        if s == 0 {
            return Err(invalid());
        }
        let (new_frequency, total) = if c < 6 * s {
            (1 + u8::from(c > s) + u8::from(c >= 4 * s), base_total + 3)
        } else {
            let new_frequency =
                4 + u8::from(c >= 9 * s) + u8::from(c >= 12 * s) + u8::from(c >= 15 * s);
            (new_frequency, base_total + u32::from(new_frequency))
        };
        self.arena.set_state(
            address,
            count,
            State {
                symbol,
                frequency: new_frequency,
                next,
            },
        )?;
        let mut context = self.arena.context(address)?;
        context.total = total as u16;
        self.arena.set_context(address, context)?;
        Ok(true)
    }
    fn update<R: PpmdRange>(
        &mut self,
        highest: Address,
        selected: Address,
        symbol: u8,
        range: &mut R,
    ) -> io::Result<()> {
        self.suffix_nudge(selected, symbol)?;
        let (_, state) = self.selected(selected, symbol)?;
        if self.fall == 0 {
            let Link::Raw(raw) = state.next else {
                return Err(invalid());
            };
            let Some(context) = self.materialize(selected, symbol, raw)? else {
                return self.restart(range);
            };
            self.successor(selected, symbol, Link::Context(context))?;
            self.current = context;
            return Ok(());
        }
        let Some(raw) = self.arena.push_history(symbol)? else {
            return self.restart(range);
        };
        let mut high_successor = Link::Raw(raw);
        let next = match state.next {
            Link::Absent => {
                self.successor(selected, symbol, Link::Raw(raw))?;
                selected
            }
            Link::Raw(position) => {
                let Some(context) = self.materialize(selected, symbol, position)? else {
                    return self.restart(range);
                };
                self.fall = self.fall.checked_sub(1).ok_or_else(invalid)?;
                context
            }
            Link::Context(context) => {
                self.fall = self.fall.checked_sub(1).ok_or_else(invalid)?;
                context
            }
        };
        if state.next != Link::Absent && self.fall == 0 {
            high_successor = Link::Context(next);
            if highest != selected {
                self.arena.undo_history()?;
            }
        }
        if highest != selected {
            let context = self.arena.context(selected)?;
            if context.count < 2 {
                return Err(invalid());
            }
            let (_, found) = self.selected(selected, symbol)?;
            let escape = u32::from(context.total)
                .checked_sub(u32::from(context.count) + u32::from(found.frequency) - 1)
                .ok_or_else(invalid)?;
            let mut cursor = highest;
            let mut depth = 0;
            while cursor != selected {
                if cursor == 0 || depth == 65 {
                    return Err(invalid());
                }
                depth += 1;
                let suffix = self.arena.context(cursor)?.suffix;
                if !self.grow(
                    cursor,
                    symbol,
                    high_successor,
                    context.count,
                    found.frequency,
                    escape,
                )? {
                    return self.restart(range);
                }
                cursor = suffix;
            }
        }
        self.current = next;
        Ok(())
    }
    fn complete<R: PpmdRange>(
        &mut self,
        highest: Address,
        selected: Address,
        index: usize,
        escaped: bool,
        range: &mut R,
    ) -> io::Result<u8> {
        let symbol = self.hit(selected, index, escaped)?;
        #[cfg(test)]
        {
            self.symbols += 1;
        }
        let (_, state) = self.selected(selected, symbol)?;
        // A format restart resets the previous-state identity to root byte 0.
        // Assign before growth so restart can deliberately replace this value.
        self.previous_symbol = symbol;
        if !escaped
            && self.fall == 0
            && let Link::Context(next) = state.next
        {
            self.current = next;
        } else {
            self.update(highest, selected, symbol, range)?;
        }
        Ok(symbol)
    }
    fn mask(&self, address: Address, mask: &mut [bool; 256], count: &mut u16) -> io::Result<()> {
        for index in 0..usize::from(self.arena.context(address)?.count) {
            let symbol = usize::from(self.arena.state(address, index)?.symbol);
            if !mask[symbol] {
                mask[symbol] = true;
                *count += 1;
            }
        }
        Ok(())
    }
    fn see_row(value: u16) -> io::Result<usize> {
        if value < 3 {
            return Ok(usize::from(value));
        }
        for row in 3..25 {
            let start = 3 + (row - 3) * (row - 2) / 2;
            if usize::from(value) < start + row - 2 {
                return Ok(row);
            }
        }
        Err(invalid())
    }
    /// Decode one model symbol. `None` means an escape past the root;
    /// its validity is decided by the format wrapper, never by this model.
    pub fn decode<R: PpmdRange>(&mut self, range: &mut R) -> io::Result<Option<u8>> {
        range.checkpoint()?;
        let highest = self.current;
        let mut current = highest;
        let context = self.arena.context(current)?;
        let high_flag = usize::from(self.previous_symbol >= 64) * 8;
        if context.count == 1 {
            let state = self.arena.state(current, 0)?;
            if !(1..=128).contains(&state.frequency) || context.suffix == 0 {
                return Err(invalid());
            }
            let suffix = self.arena.context(context.suffix)?.count;
            let group = match suffix {
                1 => 0,
                2 => 2,
                3..=11 => 4,
                _ => 6,
            };
            let column = usize::from(self.previous_success)
                + usize::from(self.run < 0) * 32
                + group
                + usize::from(state.symbol >= 64) * 16
                + high_flag;
            let probability = &mut self.binary[usize::from(state.frequency - 1)][column];
            let hit = range.binary(*probability)?;
            let mean = (*probability + 32) / 128;
            *probability -= mean;
            if hit {
                *probability += 128;
                return Ok(Some(self.complete(highest, current, 0, false, range)?));
            }
            self.initial_escape = ESCAPES[usize::from(*probability / 1024)];
        } else {
            let (count, unit) = range.count(u32::from(context.total))?;
            let mut start = 0u32;
            for index in 0..usize::from(context.count) {
                let frequency = u32::from(self.arena.state(current, index)?.frequency);
                if frequency == 0 {
                    return Err(invalid());
                }
                let end = start.checked_add(frequency).ok_or_else(invalid)?;
                if count < end {
                    range.select(unit, start, frequency)?;
                    return Ok(Some(self.complete(highest, current, index, false, range)?));
                }
                start = end;
            }
            let escape = u32::from(context.total)
                .checked_sub(start)
                .ok_or_else(invalid)?;
            range.select(unit, start, escape)?;
        }
        self.previous_success = 0;
        let mut excluded = [false; 256];
        let mut masked = 0u16;
        self.mask(current, &mut excluded, &mut masked)?;
        let mut depth = 0;
        loop {
            let suffix = self.arena.context(current)?.suffix;
            if suffix == 0 {
                return Ok(None);
            }
            range.checkpoint()?;
            depth += 1;
            if depth > 65 {
                return Err(invalid());
            }
            self.fall = self.fall.checked_add(1).ok_or_else(invalid)?;
            current = suffix;
            let context = self.arena.context(current)?;
            if context.count == masked {
                continue;
            }
            let unmasked = context.count.checked_sub(masked).ok_or_else(invalid)?;
            let mut frequency = 0u32;
            for index in 0..usize::from(context.count) {
                let state = self.arena.state(current, index)?;
                if !excluded[usize::from(state.symbol)] {
                    frequency += u32::from(state.frequency);
                }
            }
            let see_index = if context.count == 256 {
                None
            } else {
                if context.suffix == 0 {
                    return Err(invalid());
                }
                let suffix_count = self.arena.context(context.suffix)?.count;
                let column = usize::from(
                    unmasked
                        < suffix_count
                            .checked_sub(context.count)
                            .ok_or_else(invalid)?,
                ) + 2 * usize::from(
                    u32::from(context.total) < 11 * u32::from(context.count),
                ) + 4 * usize::from(masked > unmasked)
                    + high_flag;
                Some((
                    Self::see_row(unmasked.checked_sub(1).ok_or_else(invalid)?)?,
                    column,
                ))
            };
            let escape = if let Some((row, column)) = see_index {
                let see = &mut self.see[row][column];
                let quotient = see.sum >> see.shift;
                see.sum = see.sum.wrapping_sub(quotient);
                u32::from(quotient.max(1))
            } else {
                1
            };
            let total = frequency.checked_add(escape).ok_or_else(invalid)?;
            let (count, unit) = range.count(total)?;
            if count < frequency {
                let mut start = 0;
                for index in 0..usize::from(context.count) {
                    let state = self.arena.state(current, index)?;
                    if excluded[usize::from(state.symbol)] {
                        continue;
                    }
                    let width = u32::from(state.frequency);
                    if count < start + width {
                        range.select(unit, start, width)?;
                        if let Some((row, column)) = see_index {
                            let see = &mut self.see[row][column];
                            if see.shift < 7 {
                                see.count = see.count.checked_sub(1).ok_or_else(invalid)?;
                                if see.count == 0 {
                                    see.sum = see.sum.wrapping_mul(2);
                                    see.count = 3 << see.shift;
                                    see.shift += 1;
                                }
                            }
                        }
                        return Ok(Some(self.complete(highest, current, index, true, range)?));
                    }
                    start += width;
                }
                return Err(invalid());
            }
            range.select(unit, frequency, escape)?;
            if let Some((row, column)) = see_index {
                self.see[row][column].sum = self.see[row][column].sum.wrapping_add(total as u16);
            }
            self.mask(current, &mut excluded, &mut masked)?;
        }
    }
    /// Live heap buffers and model value, excluding the caller's range/input.
    pub fn allocated_bytes(&self) -> u64 {
        self.arena.allocated_bytes() + std::mem::size_of::<Self>() as u64
    }
    #[cfg(test)]
    pub(crate) fn diagnostics(&self) -> Diagnostics {
        Diagnostics {
            symbols: self.symbols,
            restarts: self.restarts,
            first_restarts: self.first_restarts,
            last_restart: self.last_restart,
            arena: self.arena.diagnostics(),
        }
    }
}

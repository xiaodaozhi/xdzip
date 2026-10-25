//! Safe object storage plus a separate deterministic virtual-capacity ledger.
//! Virtual addresses count format units; they are never host pointers.
use super::{PpmdCapacity, PpmdWork};
use crate::{invalid, stream_io::resource};
use std::io;

fn filled<T: Clone, W: PpmdWork + ?Sized>(
    count: usize,
    value: T,
    work: &mut W,
) -> io::Result<Vec<T>> {
    let mut result = Vec::new();
    result.try_reserve_exact(count).map_err(|_| resource())?;
    let quantum = (65536 / std::mem::size_of::<T>().max(1)).max(1);
    while result.len() < count {
        let end = (result.len() + quantum).min(count);
        let bytes = ((end - result.len()) * std::mem::size_of::<T>()) as u64;
        work.reserve_initialization(bytes)?;
        result.resize(end, value.clone());
        work.initialized(bytes)?;
    }
    Ok(result)
}

pub(crate) type Address = u32;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Link {
    Absent,
    Raw(u32),
    Context(Address),
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct State {
    pub symbol: u8,
    pub frequency: u8,
    pub next: Link,
}
impl State {
    pub const EMPTY: Self = Self {
        symbol: 0,
        frequency: 0,
        next: Link::Absent,
    };
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Storage {
    Single(State),
    Multiple { base: Address, class: u8 },
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Context {
    pub suffix: Address,
    pub count: u16,
    pub total: u16,
    pub storage: Storage,
}
#[derive(Clone, Copy)]
pub(crate) struct Free {
    units: u16,
    class: u8,
    older: Address,
    newer: Address,
}
#[derive(Clone, Copy)]
pub(crate) enum Cell {
    Empty,
    Reserved,
    Context(Context),
    States([State; 2]),
    Free(Free),
}
const _: () = assert!(std::mem::size_of::<Cell>() <= 32);

pub(crate) const GRADES: [u16; 38] = [
    1, 2, 3, 4, 6, 8, 10, 12, 15, 18, 21, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60, 64, 68, 72, 76,
    80, 84, 88, 92, 96, 100, 104, 108, 112, 116, 120, 124, 128,
];
pub(crate) fn class(units: usize) -> io::Result<u8> {
    GRADES
        .iter()
        .position(|&capacity| usize::from(capacity) >= units)
        .map(|value| value as u8)
        .ok_or_else(invalid)
}

#[cfg(test)]
#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Diagnostics {
    pub glues: u64,
    pub merges: u64,
    pub borrowed: u64,
    pub failed: u64,
}

pub(crate) struct Arena {
    cells: Vec<Cell>,
    traversal: Vec<Address>,
    heads: [Address; 38],
    tails: [Address; 38],
    history: Vec<u8>,
    capacity: u32,
    profile: PpmdCapacity,
    low: u32,
    high: u32,
    boundary: u32,
    text: u32,
    glue: u8,
    #[cfg(test)]
    diagnostics: Diagnostics,
}
impl Arena {
    #[cfg(test)]
    pub fn new(capacity: u32) -> io::Result<Self> {
        Self::with_work(capacity, PpmdCapacity::SevenZip, &mut ())
    }
    pub fn with_work<W: PpmdWork + ?Sized>(
        capacity: u32,
        profile: PpmdCapacity,
        work: &mut W,
    ) -> io::Result<Self> {
        let count = (capacity / 12) as usize;
        Ok(Self {
            cells: filled(count, Cell::Empty, work)?,
            traversal: filled(count, 0, work)?,
            heads: [0; 38],
            tails: [0; 38],
            history: filled(capacity as usize, 0, work)?,
            capacity,
            profile,
            low: capacity - 84 * (capacity / 96),
            high: capacity,
            boundary: capacity - 84 * (capacity / 96),
            text: 0,
            glue: 0,
            #[cfg(test)]
            diagnostics: Diagnostics::default(),
        })
    }
    pub fn reset<W: PpmdWork + ?Sized>(&mut self, work: &mut W) -> io::Result<()> {
        for chunk in self.cells.chunks_mut(65536 / std::mem::size_of::<Cell>()) {
            let bytes = std::mem::size_of_val(chunk) as u64;
            work.reserve_initialization(bytes)?;
            chunk.fill(Cell::Empty);
            work.initialized(bytes)?;
        }
        self.heads.fill(0);
        self.tails.fill(0);
        self.text = 0;
        self.high = self.capacity;
        self.low = self.capacity - 84 * (self.capacity / 96);
        self.boundary = self.low;
        self.glue = 0;
        Ok(())
    }
    fn index(&self, address: Address) -> io::Result<usize> {
        let offset = self.capacity.checked_sub(address).ok_or_else(invalid)?;
        if offset < 12 || !offset.is_multiple_of(12) {
            return Err(invalid());
        }
        let index = (offset / 12 - 1) as usize;
        if index >= self.cells.len() {
            return Err(invalid());
        }
        Ok(index)
    }
    fn cell(&self, address: Address) -> io::Result<Cell> {
        Ok(self.cells[self.index(address)?])
    }
    fn set(&mut self, address: Address, cell: Cell) -> io::Result<()> {
        let index = self.index(address)?;
        self.cells[index] = cell;
        Ok(())
    }
    fn address(base: Address, units: usize) -> io::Result<Address> {
        base.checked_add((units as u32).checked_mul(12).ok_or_else(invalid)?)
            .ok_or_else(invalid)
    }
    fn clear(&mut self, base: Address, units: u16) -> io::Result<()> {
        for index in 0..usize::from(units) {
            self.set(Self::address(base, index)?, Cell::Empty)?;
        }
        Ok(())
    }
    fn reserve(&mut self, base: Address, units: u16) -> io::Result<()> {
        for index in 0..usize::from(units) {
            let address = Self::address(base, index)?;
            if !matches!(self.cell(address)?, Cell::Empty) {
                return Err(invalid());
            }
            self.set(address, Cell::Reserved)?;
        }
        Ok(())
    }
    fn free_info(&self, address: Address) -> io::Result<Free> {
        match self.cell(address)? {
            Cell::Free(info) => Ok(info),
            _ => Err(invalid()),
        }
    }
    fn insert(&mut self, base: Address, grade: u8) -> io::Result<()> {
        let class = usize::from(grade);
        let older = self.heads[class];
        if older != 0 {
            let mut info = self.free_info(older)?;
            info.newer = base;
            self.set(older, Cell::Free(info))?;
        } else {
            self.tails[class] = base;
        }
        self.set(
            base,
            Cell::Free(Free {
                units: GRADES[class],
                class: grade,
                older,
                newer: 0,
            }),
        )?;
        self.heads[class] = base;
        Ok(())
    }
    fn unlink(&mut self, base: Address) -> io::Result<Free> {
        let info = self.free_info(base)?;
        let class = usize::from(info.class);
        if info.newer != 0 {
            let mut above = self.free_info(info.newer)?;
            above.older = info.older;
            self.set(info.newer, Cell::Free(above))?;
        } else {
            self.heads[class] = info.older;
        }
        if info.older != 0 {
            let mut below = self.free_info(info.older)?;
            below.newer = info.newer;
            self.set(info.older, Cell::Free(below))?;
        } else {
            self.tails[class] = info.newer;
        }
        Ok(info)
    }
    fn pop(&mut self, grade: u8) -> io::Result<Option<Address>> {
        let base = self.heads[usize::from(grade)];
        if base == 0 {
            return Ok(None);
        }
        let info = self.unlink(base)?;
        if info.units != GRADES[usize::from(grade)] {
            return Err(invalid());
        }
        self.clear(base, info.units)?;
        Ok(Some(base))
    }
    fn parts(&mut self, base: Address, units: u16, coalescing: bool) -> io::Result<()> {
        if units == 0 {
            return Ok(());
        }
        let grade = GRADES
            .iter()
            .rposition(|&value| value <= units)
            .ok_or_else(invalid)?;
        let whole = GRADES[grade];
        let tail = units - whole;
        // RAR's ordinary split releases the low large part before its
        // high tail. Coalescing and the 7z profile release the tail first.
        let low_first = self.profile == PpmdCapacity::Rar && !coalescing;
        if low_first {
            self.insert(base, grade as u8)?;
        }
        if tail != 0 {
            self.insert(
                Self::address(base, usize::from(whole))?,
                class(usize::from(tail))?,
            )?;
        }
        if !low_first {
            self.insert(base, grade as u8)?;
        }
        Ok(())
    }
    fn coalesce(&mut self) -> io::Result<()> {
        #[cfg(test)]
        {
            self.diagnostics.glues += 1;
        }
        let mut count = 0;
        for grade in (0..GRADES.len()).rev() {
            let mut address = self.tails[grade];
            while address != 0 {
                if count == self.traversal.len() {
                    return Err(invalid());
                }
                self.traversal[count] = address;
                count += 1;
                address = self.free_info(address)?.newer;
            }
        }
        for index in 0..count {
            let address = self.traversal[index];
            let Cell::Free(mut current) = self.cell(address)? else {
                continue;
            };
            loop {
                let upper = Self::address(address, usize::from(current.units))?;
                if upper >= self.capacity {
                    break;
                }
                let Cell::Free(next) = self.cell(upper)? else {
                    break;
                };
                let total = u32::from(current.units) + u32::from(next.units);
                if total >= 65536 {
                    break;
                }
                #[cfg(test)]
                {
                    self.diagnostics.merges += 1;
                }
                self.unlink(upper)?;
                self.set(upper, Cell::Empty)?;
                current = self.free_info(address)?;
                current.units = total as u16;
                self.set(address, Cell::Free(current))?;
            }
        }
        // Compact surviving identities before rebuilding. A new split may
        // reuse an absorbed address; it must not revive an old traversal item.
        let mut survivors = 0;
        for index in 0..count {
            let address = self.traversal[index];
            if matches!(self.cell(address)?, Cell::Free(_)) {
                self.traversal[survivors] = address;
                survivors += 1;
            }
        }
        self.heads.fill(0);
        self.tails.fill(0);
        for index in 0..survivors {
            let mut address = self.traversal[index];
            let mut units = self.free_info(address)?.units;
            self.clear(address, units)?;
            while units >= 128 {
                self.insert(address, 37)?;
                address = Self::address(address, 128)?;
                units -= 128;
            }
            self.parts(address, units, true)?;
        }
        Ok(())
    }
    fn scarce(&mut self, grade: u8) -> io::Result<Option<Address>> {
        let units = GRADES[usize::from(grade)];
        if self.glue == 0 {
            self.coalesce()?;
            self.glue = 255;
            if let Some(base) = self.pop(grade)? {
                self.reserve(base, units)?;
                return Ok(Some(base));
            }
        }
        for (larger, &larger_units) in GRADES.iter().enumerate().skip(usize::from(grade) + 1) {
            if let Some(base) = self.pop(larger as u8)? {
                self.parts(
                    Self::address(base, usize::from(units))?,
                    larger_units - units,
                    false,
                )?;
                self.reserve(base, units)?;
                return Ok(Some(base));
            }
        }
        self.glue = self.glue.wrapping_sub(1);
        let bytes = u32::from(units) * 12;
        if self.boundary.checked_sub(self.text).ok_or_else(invalid)? > bytes {
            #[cfg(test)]
            {
                self.diagnostics.borrowed += 1;
            }
            self.boundary -= bytes;
            self.reserve(self.boundary, units)?;
            return Ok(Some(self.boundary));
        }
        #[cfg(test)]
        {
            self.diagnostics.failed += 1;
        }
        Ok(None)
    }
    pub fn vector(&mut self, grade: u8) -> io::Result<Option<Address>> {
        let units = GRADES[usize::from(grade)];
        if let Some(base) = self.pop(grade)? {
            self.reserve(base, units)?;
            return Ok(Some(base));
        }
        let bytes = u32::from(units) * 12;
        if self.high.checked_sub(self.low).ok_or_else(invalid)? >= bytes {
            let base = self.low;
            self.low += bytes;
            self.reserve(base, units)?;
            return Ok(Some(base));
        }
        self.scarce(grade)
    }
    pub fn context_slot(&mut self) -> io::Result<Option<Address>> {
        if self.high != self.low {
            if self.high.checked_sub(self.low).ok_or_else(invalid)? < 12 {
                return Err(invalid());
            }
            self.high -= 12;
            self.reserve(self.high, 1)?;
            return Ok(Some(self.high));
        }
        if let Some(base) = self.pop(0)? {
            self.reserve(base, 1)?;
            return Ok(Some(base));
        }
        self.scarce(0)
    }
    pub fn release(&mut self, base: Address, grade: u8) -> io::Result<()> {
        self.clear(base, GRADES[usize::from(grade)])?;
        self.insert(base, grade)
    }
    pub fn expand(&mut self, base: Address, old: u8, new: u8) -> io::Result<Option<Address>> {
        if old == new {
            return Ok(Some(base));
        }
        let Some(target) = self.vector(new)? else {
            return Ok(None);
        };
        for unit in 0..usize::from(GRADES[usize::from(old)]) {
            self.set(
                Self::address(target, unit)?,
                self.cell(Self::address(base, unit)?)?,
            )?;
        }
        self.release(base, old)?;
        Ok(Some(target))
    }
    pub fn shrink(&mut self, base: Address, old: u8, new: u8) -> io::Result<Address> {
        if old == new {
            return Ok(base);
        }
        if let Some(target) = self.pop(new)? {
            self.reserve(target, GRADES[usize::from(new)])?;
            let retained = GRADES[usize::from(new)];
            for unit in 0..usize::from(retained) {
                self.set(
                    Self::address(target, unit)?,
                    self.cell(Self::address(base, unit)?)?,
                )?;
            }
            self.release(base, old)?;
            return Ok(target);
        }
        let retained = GRADES[usize::from(new)];
        let tail = GRADES[usize::from(old)]
            .checked_sub(retained)
            .ok_or_else(invalid)?;
        let address = Self::address(base, usize::from(retained))?;
        self.clear(address, tail)?;
        self.parts(address, tail, false)?;
        Ok(base)
    }
    pub fn context(&self, address: Address) -> io::Result<Context> {
        match self.cell(address)? {
            Cell::Context(context) => Ok(context),
            _ => Err(invalid()),
        }
    }
    pub fn set_context(&mut self, address: Address, context: Context) -> io::Result<()> {
        self.set(address, Cell::Context(context))
    }
    pub fn state(&self, address: Address, index: usize) -> io::Result<State> {
        let context = self.context(address)?;
        if index >= usize::from(context.count) {
            return Err(invalid());
        }
        match context.storage {
            Storage::Single(state) => {
                if index == 0 {
                    Ok(state)
                } else {
                    Err(invalid())
                }
            }
            Storage::Multiple { base, .. } => match self.cell(Self::address(base, index / 2)?)? {
                Cell::States(pair) => Ok(pair[index % 2]),
                _ => Err(invalid()),
            },
        }
    }
    pub fn set_state(&mut self, address: Address, index: usize, state: State) -> io::Result<()> {
        let mut context = self.context(address)?;
        if index >= usize::from(context.count) {
            return Err(invalid());
        }
        match context.storage {
            Storage::Single(_) => {
                if index != 0 {
                    return Err(invalid());
                }
                context.storage = Storage::Single(state);
                self.set_context(address, context)
            }
            Storage::Multiple { base, .. } => {
                let target = Self::address(base, index / 2)?;
                let mut pair = match self.cell(target)? {
                    Cell::States(pair) => pair,
                    Cell::Reserved => [State::EMPTY; 2],
                    _ => return Err(invalid()),
                };
                pair[index % 2] = state;
                self.set(target, Cell::States(pair))
            }
        }
    }
    pub fn find(&self, address: Address, symbol: u8) -> io::Result<usize> {
        let count = usize::from(self.context(address)?.count);
        for index in 0..count {
            if self.state(address, index)?.symbol == symbol {
                return Ok(index);
            }
        }
        Err(invalid())
    }
    pub fn push_history(&mut self, symbol: u8) -> io::Result<Option<u32>> {
        let target = self
            .history
            .get_mut(self.text as usize)
            .ok_or_else(invalid)?;
        *target = symbol;
        self.text = self.text.checked_add(1).ok_or_else(invalid)?;
        Ok((self.text < self.boundary).then_some(self.text))
    }
    pub fn undo_history(&mut self) -> io::Result<()> {
        self.text = self.text.checked_sub(1).ok_or_else(invalid)?;
        Ok(())
    }
    pub fn history(&self, position: u32) -> io::Result<u8> {
        if position >= self.text {
            return Err(invalid());
        }
        self.history
            .get(position as usize)
            .copied()
            .ok_or_else(invalid)
    }
    pub fn reset_work(&self) -> u64 {
        std::mem::size_of_val(self.cells.as_slice()) as u64
    }
    pub fn allocated_bytes(&self) -> u64 {
        (self.cells.capacity() as u64) * (std::mem::size_of::<Cell>() as u64)
            + (self.traversal.capacity() as u64) * (std::mem::size_of::<Address>() as u64)
            + self.history.capacity() as u64
    }
    #[cfg(test)]
    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn allocated() -> Arena {
        let mut arena = Arena::new(2048).unwrap();
        assert_eq!(arena.context_slot().unwrap(), Some(2036));
        assert_eq!(arena.vector(37).unwrap(), Some(284));
        arena
    }
    #[test]
    fn ppmd7_virtual_lifo_and_strict_history_boundary() {
        let mut arena = allocated();
        let a = arena.vector(0).unwrap().unwrap();
        let b = arena.vector(0).unwrap().unwrap();
        arena.release(a, 0).unwrap();
        arena.release(b, 0).unwrap();
        assert_eq!(arena.vector(0).unwrap(), Some(b));
        assert_eq!(arena.vector(0).unwrap(), Some(a));
        let mut arena = Arena::new(2048).unwrap();
        arena.low = arena.high;
        arena.boundary = 32;
        arena.text = 20;
        assert_eq!(arena.context_slot().unwrap(), None);
        arena.text = 19;
        assert_eq!(arena.context_slot().unwrap(), Some(20));
        assert_eq!(arena.boundary, 20);
    }
    #[test]
    fn ppmd7_virtual_coalesce_order_and_split_retains_low_address() {
        let mut arena = allocated();
        let x = arena.vector(1).unwrap().unwrap();
        let y = arena.vector(1).unwrap().unwrap();
        let z = arena.vector(1).unwrap().unwrap();
        assert_eq!(y, x + 24);
        assert_eq!(z, y + 24);
        arena.release(x, 1).unwrap();
        arena.release(z, 1).unwrap();
        arena.release(y, 1).unwrap();
        arena.low = arena.high;
        assert_eq!(arena.vector(3).unwrap(), Some(x));
        assert_eq!(arena.vector(1).unwrap(), Some(z));
    }
    #[test]
    fn ppmd7_virtual_shrink_relocates_or_splits_and_expand_failure_keeps_old() {
        let mut arena = allocated();
        let old = arena.vector(4).unwrap().unwrap();
        let target = arena.vector(1).unwrap().unwrap();
        arena.release(target, 1).unwrap();
        assert_eq!(arena.shrink(old, 4, 1).unwrap(), target);
        assert_eq!(arena.vector(4).unwrap(), Some(old));
        assert_eq!(arena.shrink(old, 4, 1).unwrap(), old);
        assert_eq!(arena.vector(3).unwrap(), Some(old + 24));
        let old = arena.vector(0).unwrap().unwrap();
        arena.low = arena.high;
        arena.boundary = 32;
        arena.text = 32;
        arena.heads.fill(0);
        arena.tails.fill(0);
        assert_eq!(arena.expand(old, 0, 1).unwrap(), None);
        assert!(matches!(arena.cell(old).unwrap(), Cell::Reserved));
    }
    #[test]
    fn ppmd7_virtual_glue_does_not_merge_65536_units() {
        let mut arena = Arena::new(1024 * 1024).unwrap();
        let lower = arena.capacity - 65536 * 12;
        let upper = lower + 32768 * 12;
        arena
            .set(
                upper,
                Cell::Free(Free {
                    units: 32768,
                    class: 37,
                    older: 0,
                    newer: lower,
                }),
            )
            .unwrap();
        arena
            .set(
                lower,
                Cell::Free(Free {
                    units: 32768,
                    class: 37,
                    older: upper,
                    newer: 0,
                }),
            )
            .unwrap();
        arena.heads[37] = lower;
        arena.tails[37] = upper;
        arena.coalesce().unwrap();
        assert_eq!(arena.heads[37], lower + (32768 - 128) * 12);
        assert_eq!(arena.tails[37], upper);
    }
}

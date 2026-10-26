//! Host calendar projection. Container timestamps retain their original precision.

/// Absolute timestamp presented to a host calendar service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Timestamp {
    /// Unsigned 100ns ticks since 1601-01-01 UTC.
    Filetime(u64),
    /// Whole unsigned seconds since 1970-01-01 UTC.
    UnixSeconds(u64),
}

/// Calendar fields after an explicit UTC or host-local projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Calendar {
    /// Gregorian year.
    pub year: i32,
    /// Month, 1 through 12.
    pub month: u8,
    /// Day of month, starting at 1.
    pub day: u8,
    /// Hour, 0 through 23.
    pub hour: u8,
    /// Minute, 0 through 59.
    pub minute: u8,
    /// Second, 0 through 60 if the host exposes a leap second.
    pub second: u8,
}
fn leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
fn months(year: i32) -> [u8; 12] {
    [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ]
}
impl Calendar {
    /// Validate host fields before using them as a packed display timestamp.
    pub fn is_valid(self) -> bool {
        self.year > 0
            && (1..=12).contains(&self.month)
            && self.day != 0
            && self.day <= months(self.year)[usize::from(self.month - 1)]
            && self.hour < 24
            && self.minute < 60
            && self.second <= 60
    }
    /// Pack valid fields with a wrapping 7-bit year, as historical native RAR did.
    /// This does not assert that an out-of-range year is a legal DOS timestamp.
    pub fn wrapping_dos(self) -> Option<u32> {
        self.is_valid().then(|| {
            (self.year as u32).wrapping_sub(1980).wrapping_shl(25)
                | (u32::from(self.month) << 21)
                | (u32::from(self.day) << 16)
                | (u32::from(self.hour) << 11)
                | (u32::from(self.minute) << 5)
                | u32::from(self.second / 2)
        })
    }
}

/// Deterministic UTC default for sources without an operating-system calendar.
/// Local-time hosts override `ReadSource::local_calendar`; this never guesses a zone.
pub fn utc_calendar(timestamp: Timestamp) -> Option<Calendar> {
    let (mut days, clock) = match timestamp {
        Timestamp::Filetime(ticks) => (ticks / 864_000_000_000, (ticks / 10_000_000) % 86400),
        Timestamp::UnixSeconds(seconds) => (seconds / 86400 + 134_774, seconds % 86400),
    };
    // 1601 starts a complete Gregorian 400-year cycle. Skip complete cycles,
    // then examine at most 399 individual years and twelve months.
    let cycles = days / 146_097;
    let mut year = i32::try_from(1601u64.checked_add(cycles.checked_mul(400)?)?).ok()?;
    days %= 146_097;
    loop {
        let length = if leap(year) { 366 } else { 365 };
        if days < length {
            break;
        }
        days -= length;
        year = year.checked_add(1)?;
    }
    let mut month = 1;
    for length in months(year) {
        if days < u64::from(length) {
            break;
        }
        days -= u64::from(length);
        month += 1;
    }
    Some(Calendar {
        year,
        month,
        day: days as u8 + 1,
        hour: (clock / 3600) as u8,
        minute: ((clock / 60) % 60) as u8,
        second: (clock % 60) as u8,
    })
}

/// Preserve 7z's existing UTC display policy: unavailable outside DOS years.
pub fn filetime_dos_utc(ticks: u64) -> u32 {
    utc_calendar(Timestamp::Filetime(ticks))
        .filter(|date| (1980..=2107).contains(&date.year))
        .and_then(Calendar::wrapping_dos)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_epochs_leap_centuries_and_extreme_inputs_are_bounded() {
        let epoch = utc_calendar(Timestamp::Filetime(0)).unwrap();
        assert_eq!((epoch.year, epoch.month, epoch.day), (1601, 1, 1));
        let unix = utc_calendar(Timestamp::UnixSeconds(0)).unwrap();
        assert_eq!((unix.year, unix.month, unix.day), (1970, 1, 1));
        let leap = utc_calendar(Timestamp::UnixSeconds(951_827_697)).unwrap();
        assert_eq!(
            (
                leap.year,
                leap.month,
                leap.day,
                leap.hour,
                leap.minute,
                leap.second
            ),
            (2000, 2, 29, 12, 34, 57)
        );
        let century = utc_calendar(Timestamp::UnixSeconds(4_107_542_400)).unwrap();
        assert_eq!((century.year, century.month, century.day), (2100, 3, 1));
        assert!(
            utc_calendar(Timestamp::Filetime(u64::MAX))
                .unwrap()
                .is_valid()
        );
        assert_eq!(utc_calendar(Timestamp::UnixSeconds(u64::MAX)), None);
    }

    #[test]
    fn native_rar_year_wrap_and_sevenzip_display_range_stay_distinct() {
        let before = Calendar {
            year: 1979,
            month: 12,
            day: 31,
            hour: 23,
            minute: 59,
            second: 59,
        };
        assert_eq!(before.wrapping_dos(), Some(0xff9f_bf7d));
        assert_eq!(
            Calendar {
                year: 2108,
                month: 1,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0
            }
            .wrapping_dos(),
            Some(0x0021_0000)
        );
        assert_eq!(filetime_dos_utc(0), 0);
        assert_eq!(
            filetime_dos_utc((315_532_800 + 11_644_473_600) * 10_000_000),
            0x0021_0000
        );
        assert_eq!(filetime_dos_utc(u64::MAX), 0);
        assert_eq!(Calendar { month: 0, ..before }.wrapping_dos(), None);
        assert_eq!(
            Calendar {
                year: 2100,
                month: 2,
                day: 29,
                ..before
            }
            .wrapping_dos(),
            None
        );
    }
}

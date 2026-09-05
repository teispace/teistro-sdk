//! The shipped offset histories, each row cited to tzdb's source files.
//! The embedded database replaces this list; until then the calendars
//! that need a clock take these.

use teistro_core::quantity::JulianDay;
use teistro_core::time::UtcOffset;

use crate::history::{OffsetHistory, OffsetRow};

/// Nepal, tzdb `asia`, zone `Asia/Kathmandu`: local mean time +05:41:16
/// until 1920, +05:30 until 1986, +05:45 since. tzdb's `until` instants
/// are local wall-clock midnights, converted here to UTC.
static NEPAL_ROWS: [OffsetRow; 3] = [
    OffsetRow {
        // Before 1920: Kathmandu's local mean time; the row applies to
        // every earlier instant.
        from: JulianDay::literal(0.0),
        offset: UtcOffset::literal(5, 41, 16),
        abbreviation: "LMT",
    },
    OffsetRow {
        // 1920-01-01 00:00 LMT: 0h UT of that day is JD 2 422 324.5.
        from: JulianDay::literal(2_422_324.5 - 20_476.0 / 86_400.0),
        offset: UtcOffset::literal(5, 30, 0),
        abbreviation: "+0530",
    },
    OffsetRow {
        // 1986-01-01 00:00 +05:30: 0h UT of that day is JD 2 446 431.5.
        from: JulianDay::literal(2_446_431.5 - 19_800.0 / 86_400.0),
        offset: UtcOffset::literal(5, 45, 0),
        abbreviation: "+0545",
    },
];

static NEPAL: OffsetHistory = OffsetHistory {
    zone: "Asia/Kathmandu",
    source: "tzdb asia: LMT +05:41:16 to 1920, +05:30 to 1986, +05:45",
    rows: &NEPAL_ROWS,
};

/// Nepal's clock history.
#[must_use]
pub fn nepal() -> &'static OffsetHistory {
    &NEPAL
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::Utc;
    use teistro_core::time::LocalClock;

    use super::*;

    #[test]
    fn nepal_changes_clock_at_the_local_midnights_tzdb_names() {
        let nepal = nepal();
        let at = |jd: f64| nepal.offset_at(JulianDay::<Utc>::try_new(jd).unwrap());
        assert_eq!(at(2_415_020.5).to_string(), "+05:41:16"); // 1900
        // A minute before and after the 1920 change, in UTC.
        let change_1920 = 2_422_324.5 - 20_476.0 / 86_400.0;
        assert_eq!(at(change_1920 - 1.0 / 1440.0).to_string(), "+05:41:16");
        assert_eq!(at(change_1920).to_string(), "+05:30");
        let change_1986 = 2_446_431.5 - 19_800.0 / 86_400.0;
        assert_eq!(at(change_1986 - 1.0 / 1440.0).to_string(), "+05:30");
        assert_eq!(at(change_1986).to_string(), "+05:45");
        assert_eq!(at(2_460_000.5).to_string(), "+05:45");
        assert_eq!(nepal.transitions().count(), 2);
        assert!(nepal.describe().starts_with("Asia/Kathmandu"));
        // The local midnight that begins 1986 is where the clock jumps:
        // 1985-12-31 23:59 +05:30 is followed by 1986-01-01 00:15 +05:45.
        let local_before = nepal.local_jd(JulianDay::try_new(change_1986 - 1e-9).unwrap());
        let local_after = nepal.local_jd(JulianDay::try_new(change_1986).unwrap());
        assert!((local_after - local_before - 900.0 / 86_400.0).abs() < 1e-7);
    }
}

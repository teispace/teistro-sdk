//! The zones the SDK's own calendars reckon in, as clocks from the
//! embedded database.

use std::sync::OnceLock;

use crate::zone::embedded::{EmbeddedTzdb, ZoneClock};

static NEPAL: OnceLock<ZoneClock> = OnceLock::new();

/// Nepal's clock, `Asia/Kathmandu`: local mean time +05:41:16 until
/// 1920, +05:30 until 1986, +05:45 since (tzdb `asia`).
///
/// # Panics
///
/// Never with the bundled database, which carries the zone; a build
/// whose database lacks it is broken and stops here.
#[allow(
    clippy::expect_used,
    reason = "the bundled database carries Asia/Kathmandu; its absence is a broken build, not a runtime state"
)]
pub fn nepal() -> &'static ZoneClock {
    NEPAL.get_or_init(|| {
        EmbeddedTzdb::shared()
            .clock("Asia/Kathmandu")
            .expect("the bundled tzdb carries Asia/Kathmandu")
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::{JulianDay, Utc};
    use teistro_core::time::LocalClock;

    use super::*;

    #[test]
    fn nepal_changes_clock_at_the_local_midnights_tzdb_names() {
        let nepal = nepal();
        let at = |jd: f64| nepal.offset_at(JulianDay::<Utc>::try_new(jd).unwrap());
        assert_eq!(at(2_415_020.5).to_string(), "+05:41:16"); // 1900
        // A minute before and after the 1920 change, in UTC: 1920-01-01
        // 00:00 LMT.
        let change_1920 = 2_422_324.5 - 20_476.0 / 86_400.0;
        assert_eq!(at(change_1920 - 1.0 / 1440.0).to_string(), "+05:41:16");
        assert_eq!(at(change_1920 + 1.0 / 1440.0).to_string(), "+05:30");
        // 1986-01-01 00:00 +05:30.
        let change_1986 = 2_446_431.5 - 19_800.0 / 86_400.0;
        assert_eq!(at(change_1986 - 1.0 / 1440.0).to_string(), "+05:30");
        assert_eq!(at(change_1986 + 1.0 / 1440.0).to_string(), "+05:45");
        assert_eq!(at(2_460_000.5).to_string(), "+05:45");
        assert!(nepal.describe().starts_with("Asia/Kathmandu"));
    }
}

//! The embedded zone database: tzdb bundled as `TZif` data, read through
//! the `jiff` engine and never from the host, so a resolution is the
//! same on every machine (ADR-0022). Implements the time-zone port and
//! offers any of its zones as a `LocalClock`. A signed pack replaces it
//! on a context (`zone::pack`, `docs/03-design/zone-packs.md`), and this
//! stays the fallback.

use std::sync::OnceLock;

use jiff::tz::{TimeZone, TimeZoneDatabase};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_port_timezone::{
    LocalCandidates, LocalSeconds, OffsetInfo, TimeZoneProvider, unknown_zone,
};

use super::engine::{self, ZoneClock};

/// The embedded database.
///
/// ```
/// use teistro_time::{EmbeddedTzdb, TimeZoneProvider};
///
/// let tzdb = EmbeddedTzdb::shared();
/// assert!(tzdb.has_zone("Asia/Kathmandu"));
/// assert!(!tzdb.has_zone("Asia/Katmandoo"));
/// ```
#[derive(Debug)]
pub struct EmbeddedTzdb {
    db: TimeZoneDatabase,
}

static SHARED: OnceLock<EmbeddedTzdb> = OnceLock::new();

impl EmbeddedTzdb {
    /// The process-wide database, built on first use.
    ///
    /// ```
    /// use teistro_time::EmbeddedTzdb;
    ///
    /// // Every call answers the same database.
    /// assert!(std::ptr::eq(EmbeddedTzdb::shared(), EmbeddedTzdb::shared()));
    /// ```
    pub fn shared() -> &'static EmbeddedTzdb {
        SHARED.get_or_init(EmbeddedTzdb::new)
    }

    /// A database over the bundled data.
    ///
    /// ```
    /// use teistro_time::{EmbeddedTzdb, TimeZoneProvider};
    ///
    /// let tzdb = EmbeddedTzdb::new();
    /// assert_eq!(tzdb.zones().len(), EmbeddedTzdb::shared().zones().len());
    /// ```
    #[must_use]
    pub fn new() -> EmbeddedTzdb {
        EmbeddedTzdb {
            db: TimeZoneDatabase::bundled(),
        }
    }

    /// The bundled tzdb's version (`2026a`), or `unknown`.
    ///
    /// ```
    /// use teistro_time::{EmbeddedTzdb, TimeZoneProvider};
    ///
    /// let version = EmbeddedTzdb::bundled_version();
    /// assert!(version.starts_with("20"), "{version}");
    /// assert_eq!(EmbeddedTzdb::shared().version(), version);
    /// ```
    #[must_use]
    pub fn bundled_version() -> &'static str {
        jiff_tzdb::VERSION.unwrap_or("unknown")
    }

    fn zone(&self, name: &str) -> Result<TimeZone, Error> {
        self.db
            .get(name)
            .map_err(|_| unknown_zone(name, &self.suggest(name)))
    }

    /// The nearest known names to an unknown one, within two edits on the
    /// lower-cased names.
    ///
    /// ```
    /// use teistro_time::EmbeddedTzdb;
    ///
    /// let near = EmbeddedTzdb::shared().suggest("asia/kathmandoo");
    /// assert_eq!(near.first().map(String::as_str), Some("Asia/Kathmandu"));
    /// ```
    #[must_use]
    pub fn suggest(&self, name: &str) -> Vec<String> {
        engine::suggest(self.db.available().map(|known| known.to_string()), name)
    }

    /// A zone of the database as a clock.
    ///
    /// ```
    /// use teistro_core::quantity::{JulianDay, Utc};
    /// use teistro_time::{EmbeddedTzdb, LocalClock, UtcOffset};
    ///
    /// let kathmandu = EmbeddedTzdb::shared().clock("Asia/Kathmandu")?;
    /// // +05:30 in 1985, +05:45 since the start of 1986.
    /// let (in_1985, in_2024) = (JulianDay::<Utc>::try_new(2_446_247.5)?, JulianDay::<Utc>::try_new(2_460_310.5)?);
    /// assert_eq!(kathmandu.offset_at(in_1985), UtcOffset::literal(5, 30, 0));
    /// assert_eq!(kathmandu.offset_at(in_2024), UtcOffset::literal(5, 45, 0));
    ///
    /// let unknown = EmbeddedTzdb::shared().clock("Asia/Kathmandoo").unwrap_err();
    /// assert!(unknown.hint().is_some_and(|hint| hint.contains("Asia/Kathmandu")));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An unknown zone, with the nearest names as the hint.
    pub fn clock(&self, name: &str) -> Result<ZoneClock, Error> {
        Ok(ZoneClock::new(
            self.zone(name)?,
            name,
            EmbeddedTzdb::bundled_version(),
        ))
    }
}

impl Default for EmbeddedTzdb {
    fn default() -> EmbeddedTzdb {
        EmbeddedTzdb::new()
    }
}

impl TimeZoneProvider for EmbeddedTzdb {
    fn version(&self) -> &str {
        EmbeddedTzdb::bundled_version()
    }

    fn has_zone(&self, zone: &str) -> bool {
        self.db.get(zone).is_ok()
    }

    fn zones(&self) -> Vec<String> {
        self.db.available().map(|name| name.to_string()).collect()
    }

    fn offset_at(&self, zone: &str, instant: JulianDay<Utc>) -> Result<OffsetInfo, Error> {
        engine::offset_info(&self.zone(zone)?, instant)
    }

    fn candidates(&self, zone: &str, local: LocalSeconds) -> Result<LocalCandidates, Error> {
        engine::candidates(&self.zone(zone)?, local)
    }

    fn current_offsets(&self, zone: &str) -> Result<Vec<UtcOffset>, Error> {
        engine::current_offsets(&self.zone(zone)?, self.version())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use teistro_core::time::LocalClock;

    fn at(jd: f64) -> JulianDay<Utc> {
        JulianDay::try_new(jd).unwrap()
    }

    #[test]
    fn the_bundled_database_has_a_version_and_knows_kathmandu() {
        let db = EmbeddedTzdb::shared();
        assert!(db.version().starts_with("20"), "{}", db.version());
        assert!(db.has_zone("Asia/Kathmandu") && !db.has_zone("Asia/Nowhere"));
        assert!(db.zones().len() > 300);
        let error = db
            .offset_at("Asia/Kathmandoo", at(2_460_000.5))
            .unwrap_err();
        assert_eq!(error.hint(), Some("did you mean Asia/Kathmandu?"));
        assert!(
            db.suggest("Europe/Londn")
                .contains(&String::from("Europe/London"))
        );
        assert!(db.suggest("zzzz").is_empty(), "{:?}", db.suggest("zzzz"));
    }

    #[test]
    fn nepal_before_1920_is_the_local_mean_time_stub() {
        let db = EmbeddedTzdb::shared();
        // 1910-03-21 0h UT.
        let info = db.offset_at("Asia/Kathmandu", at(2_418_751.5)).unwrap();
        assert_eq!(info.offset.to_string(), "+05:41:16");
        assert_eq!(info.abbreviation, "LMT");
        assert!(info.before_rules && !info.is_dst);
        // 1985-12-25: +05:30, not the stub; 1986-01-02: +05:45.
        let old = db.offset_at("Asia/Kathmandu", at(2_446_424.5)).unwrap();
        assert_eq!(
            (old.offset.to_string().as_str(), old.before_rules),
            ("+05:30", false)
        );
        let new = db.offset_at("Asia/Kathmandu", at(2_446_432.5)).unwrap();
        assert_eq!(new.offset.to_string(), "+05:45");
        assert_eq!(
            db.current_offsets("Asia/Kathmandu").unwrap(),
            vec![UtcOffset::literal(5, 45, 0)]
        );
        let york = db.current_offsets("America/New_York").unwrap();
        assert_eq!(
            york,
            vec![UtcOffset::literal(-5, 0, 0), UtcOffset::literal(-4, 0, 0)]
        );
    }

    #[test]
    fn candidates_cover_the_gap_and_the_fold() {
        let db = EmbeddedTzdb::shared();
        // 1986-01-01 00:05 local in Nepal: inside the fifteen-minute gap.
        let local = LocalSeconds {
            seconds: (2_446_431 - 2_440_587) * 86_400 - 43_200 + 300,
            nanos: 0,
        };
        // (JD 2446431.5 is 1986-01-01 0h; the local seconds count civil
        // time as if UTC: 1986-01-01 00:05:00.)
        let candidates = db
            .candidates(
                "Asia/Kathmandu",
                LocalSeconds {
                    seconds: local.seconds + 43_200,
                    nanos: 0,
                },
            )
            .unwrap();
        assert_eq!(
            candidates,
            LocalCandidates::Gap {
                before: UtcOffset::literal(5, 30, 0),
                after: UtcOffset::literal(5, 45, 0)
            }
        );
        // 2021-11-07 01:30 in New York: the fold.
        let fold_local = LocalSeconds {
            seconds: 1_636_248_600,
            nanos: 0,
        };
        assert_eq!(
            db.candidates("America/New_York", fold_local).unwrap(),
            LocalCandidates::Overlap {
                earlier: UtcOffset::literal(-4, 0, 0),
                later: UtcOffset::literal(-5, 0, 0)
            }
        );
        // An ordinary time.
        assert!(matches!(
            db.candidates(
                "Asia/Kathmandu",
                LocalSeconds {
                    seconds: 1_700_000_000,
                    nanos: 0
                }
            )
            .unwrap(),
            LocalCandidates::Unambiguous { .. }
        ));
    }

    #[test]
    fn a_zone_is_a_clock_and_the_range_is_refused_beyond_the_engine() {
        let clock = EmbeddedTzdb::shared().clock("Asia/Kathmandu").unwrap();
        assert_eq!(clock.name(), "Asia/Kathmandu");
        assert!(clock.describe().starts_with("Asia/Kathmandu (tzdb "));
        assert_eq!(clock.offset_at(at(2_460_000.5)).to_string(), "+05:45");
        assert_eq!(clock.offset_at(at(2_418_751.5)).to_string(), "+05:41:16");
        // Far outside the engine's range the clock still answers.
        assert_eq!(clock.offset_at(at(-1_000_000.0)).to_string(), "+05:41:16");
        assert!(
            EmbeddedTzdb::shared()
                .offset_at("Asia/Kathmandu", at(9_000_000.0))
                .is_err()
        );
        assert!(EmbeddedTzdb::shared().clock("Nowhere/Land").is_err());
        assert_eq!(
            EmbeddedTzdb::default().version(),
            EmbeddedTzdb::bundled_version()
        );
    }
}

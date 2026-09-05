//! The embedded zone database: tzdb bundled as `TZif` data, read through
//! the `jiff` engine and never from the host, so a resolution is the
//! same on every machine (ADR-0022). Implements the time-zone port and
//! offers any of its zones as a [`LocalClock`].

use std::sync::OnceLock;

use jiff::Timestamp;
use jiff::civil::DateTime;
use jiff::tz::{AmbiguousOffset, TimeZone, TimeZoneDatabase};
use teistro_core::catalogue::distance;
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::{LocalClock, UtcOffset};
use teistro_port_timezone::{
    LocalCandidates, LocalSeconds, OffsetInfo, TimeZoneProvider, unknown_zone,
};

/// Seconds in a day.
const SECONDS_PER_DAY: f64 = 86_400.0;
/// The year the bundled data is taken to describe when its version
/// cannot be read.
const FALLBACK_VERSION_YEAR: i16 = 2026;
/// How many suggestions an unknown zone gets.
const SUGGESTIONS: usize = 3;
/// The edit distance a suggestion is worth: the core's own threshold,
/// beyond which its distance answers with a lower bound.
const SUGGESTION_DISTANCE: usize = 2;

/// The embedded database.
#[derive(Debug)]
pub struct EmbeddedTzdb {
    db: TimeZoneDatabase,
}

static SHARED: OnceLock<EmbeddedTzdb> = OnceLock::new();

impl EmbeddedTzdb {
    /// The process-wide database, built on first use.
    pub fn shared() -> &'static EmbeddedTzdb {
        SHARED.get_or_init(EmbeddedTzdb::new)
    }

    /// A database over the bundled data.
    #[must_use]
    pub fn new() -> EmbeddedTzdb {
        EmbeddedTzdb {
            db: TimeZoneDatabase::bundled(),
        }
    }

    /// The bundled tzdb's version (`2026a`), or `unknown`.
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
    #[must_use]
    pub fn suggest(&self, name: &str) -> Vec<String> {
        let wanted = name.to_ascii_lowercase();
        let mut scored: Vec<(usize, String)> = self
            .db
            .available()
            .map(|known| {
                let name = known.to_string();
                (distance(&wanted, &name.to_ascii_lowercase()), name)
            })
            .filter(|(d, _)| *d <= SUGGESTION_DISTANCE)
            .collect();
        scored.sort();
        scored
            .into_iter()
            .take(SUGGESTIONS)
            .map(|(_, name)| name)
            .collect()
    }

    /// A zone of the database as a clock.
    ///
    /// # Errors
    ///
    /// An unknown zone, with the nearest names as the hint.
    pub fn clock(&self, name: &str) -> Result<ZoneClock, Error> {
        Ok(ZoneClock {
            tz: self.zone(name)?,
            name: name.to_string(),
            version: EmbeddedTzdb::bundled_version(),
        })
    }

    fn version_year(&self) -> i16 {
        self.version()
            .get(..4)
            .and_then(|digits| digits.parse().ok())
            .unwrap_or(FALLBACK_VERSION_YEAR)
    }
}

impl Default for EmbeddedTzdb {
    fn default() -> EmbeddedTzdb {
        EmbeddedTzdb::new()
    }
}

/// A Julian day as the engine's timestamp.
fn timestamp(instant: JulianDay<Utc>) -> Result<Timestamp, Error> {
    let seconds = (instant.get() - JulianDay::<Utc>::UNIX_EPOCH.get()) * SECONDS_PER_DAY;
    let whole = seconds.floor();
    let nanos = ((seconds - whole) * 1e9).round();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a floored second count and a rounded sub-second"
    )]
    let (mut second, mut nano) = (whole as i64, nanos as i32);
    if nano >= 1_000_000_000 {
        second = second.saturating_add(1);
        nano = 0;
    }
    Timestamp::new(second, nano).map_err(|_| {
        Error::new(
            Status::OutOfRange,
            format!("{instant} is outside the zone database's range (years -9999 to 9999)"),
        )
    })
}

fn offset_of(offset: jiff::tz::Offset) -> Result<UtcOffset, Error> {
    Ok(UtcOffset::try_from_seconds(offset.seconds())?)
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
        let tz = self.zone(zone)?;
        let ts = timestamp(instant)?;
        let info = tz.to_offset_info(ts);
        Ok(OffsetInfo {
            offset: offset_of(info.offset())?,
            abbreviation: info.abbreviation().to_string(),
            is_dst: info.dst().is_dst(),
            before_rules: tz.preceding(ts).next().is_none(),
        })
    }

    fn candidates(&self, zone: &str, local: LocalSeconds) -> Result<LocalCandidates, Error> {
        let tz = self.zone(zone)?;
        let nanos = i32::try_from(local.nanos).unwrap_or(0);
        let as_utc = Timestamp::new(local.seconds, nanos).map_err(|_| {
            Error::new(
                Status::OutOfRange,
                format!("{local} is outside the zone database's range"),
            )
        })?;
        let civil: DateTime = TimeZone::UTC.to_datetime(as_utc);
        Ok(match tz.to_ambiguous_timestamp(civil).offset() {
            AmbiguousOffset::Unambiguous { offset } => LocalCandidates::Unambiguous {
                offset: offset_of(offset)?,
            },
            AmbiguousOffset::Gap { before, after } => LocalCandidates::Gap {
                before: offset_of(before)?,
                after: offset_of(after)?,
            },
            AmbiguousOffset::Fold { before, after } => LocalCandidates::Overlap {
                earlier: offset_of(before)?,
                later: offset_of(after)?,
            },
        })
    }

    fn current_offsets(&self, zone: &str) -> Result<Vec<UtcOffset>, Error> {
        let tz = self.zone(zone)?;
        let year = self.version_year();
        let mut offsets = Vec::with_capacity(2);
        for month in [1i8, 4, 7, 10] {
            let Ok(civil) = DateTime::new(year, month, 1, 12, 0, 0, 0) else {
                continue;
            };
            let Ok(ts) = TimeZone::UTC.to_timestamp(civil) else {
                continue;
            };
            let offset = offset_of(tz.to_offset(ts))?;
            if !offsets.contains(&offset) {
                offsets.push(offset);
            }
        }
        offsets.sort();
        Ok(offsets)
    }
}

/// A zone of the embedded database as a clock.
#[derive(Clone, Debug)]
pub struct ZoneClock {
    tz: TimeZone,
    name: String,
    version: &'static str,
}

impl ZoneClock {
    /// The zone's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl LocalClock for ZoneClock {
    fn offset_at(&self, instant: JulianDay<Utc>) -> UtcOffset {
        let ts =
            timestamp(instant).unwrap_or(if instant.get() < JulianDay::<Utc>::UNIX_EPOCH.get() {
                Timestamp::MIN
            } else {
                Timestamp::MAX
            });
        UtcOffset::try_from_seconds(self.tz.to_offset(ts).seconds()).unwrap_or(UtcOffset::UTC)
    }

    fn describe(&self) -> String {
        format!("{} (tzdb {})", self.name, self.version)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

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
        assert!(db.suggest("zzzz").is_empty());
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

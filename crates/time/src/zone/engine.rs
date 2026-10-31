//! What the embedded database and a zone pack share: both are `TZif` read
//! by `jiff`, and differ only in where a zone's `TimeZone` comes from. A
//! row means one thing whichever gave it
//! (`docs/03-design/zone-packs.md` §2).

use jiff::Timestamp;
use jiff::civil::DateTime;
use jiff::tz::{AmbiguousOffset, TimeZone};
use teistro_core::catalogue::distance;
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::{LocalClock, UtcOffset};
use teistro_port_timezone::{LocalCandidates, LocalSeconds, OffsetInfo};

/// Seconds in a day.
const SECONDS_PER_DAY: f64 = 86_400.0;
/// The year a database is taken to describe when its version cannot be
/// read.
const FALLBACK_VERSION_YEAR: i16 = 2026;
/// How many suggestions an unknown zone gets.
const SUGGESTIONS: usize = 3;
/// The edit distance a suggestion is worth: the core's own threshold,
/// beyond which its distance answers with a lower bound.
const SUGGESTION_DISTANCE: usize = 2;

/// The nearest of `names` to an unknown name, within two edits on the
/// lower-cased names.
pub(crate) fn suggest(names: impl Iterator<Item = String>, name: &str) -> Vec<String> {
    let wanted = name.to_ascii_lowercase();
    let mut scored: Vec<(usize, String)> = names
        .map(|known| (distance(&wanted, &known.to_ascii_lowercase()), known))
        .filter(|(d, _)| *d <= SUGGESTION_DISTANCE)
        .collect();
    scored.sort();
    scored
        .into_iter()
        .take(SUGGESTIONS)
        .map(|(_, name)| name)
        .collect()
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

/// The offset in force at an instant in a zone.
pub(crate) fn offset_info(tz: &TimeZone, instant: JulianDay<Utc>) -> Result<OffsetInfo, Error> {
    let ts = timestamp(instant)?;
    let info = tz.to_offset_info(ts);
    Ok(OffsetInfo {
        offset: offset_of(info.offset())?,
        abbreviation: info.abbreviation().to_string(),
        is_dst: info.dst().is_dst(),
        before_rules: tz.preceding(ts).next().is_none(),
    })
}

/// The offsets a civil time could take in a zone.
pub(crate) fn candidates(tz: &TimeZone, local: LocalSeconds) -> Result<LocalCandidates, Error> {
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

/// The offsets a zone applies in the year its database's version names.
pub(crate) fn current_offsets(tz: &TimeZone, version: &str) -> Result<Vec<UtcOffset>, Error> {
    let year = version
        .get(..4)
        .and_then(|digits| digits.parse().ok())
        .unwrap_or(FALLBACK_VERSION_YEAR);
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

/// A zone of a database as a clock.
///
/// ```
/// use teistro_core::quantity::{JulianDay, Utc};
/// use teistro_time::{EmbeddedTzdb, LocalClock, UtcOffset};
///
/// let new_york = EmbeddedTzdb::shared().clock("America/New_York")?;
/// let (january, july) = (JulianDay::<Utc>::try_new(2_460_311.0)?, JulianDay::<Utc>::try_new(2_460_492.0)?);
/// assert_eq!(new_york.offset_at(january), UtcOffset::literal(-5, 0, 0));
/// assert_eq!(new_york.offset_at(july), UtcOffset::literal(-4, 0, 0));
/// assert!(new_york.describe().starts_with("America/New_York (tzdb "));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct ZoneClock {
    tz: TimeZone,
    name: String,
    version: String,
}

impl ZoneClock {
    pub(crate) fn new(tz: TimeZone, name: &str, version: &str) -> ZoneClock {
        ZoneClock {
            tz,
            name: name.to_string(),
            version: version.to_string(),
        }
    }

    /// The zone's name.
    ///
    /// ```
    /// use teistro_time::EmbeddedTzdb;
    ///
    /// assert_eq!(EmbeddedTzdb::shared().clock("Europe/London")?.name(), "Europe/London");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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

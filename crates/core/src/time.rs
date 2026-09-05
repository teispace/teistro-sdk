//! Clock offsets and the local-clock abstraction the calendars and the
//! time layer share: an offset from UTC as a validated quantity, and a
//! clock that answers "which offset applied at this instant", which a
//! fixed offset, local mean time and a zone's rule history all implement
//! (`docs/03-design/time-and-timezone.md`). The calendars consume the
//! trait (a sankranti falls on a civil day only under a clock); the time
//! layer supplies the zone histories.
//!
//! ```
//! use teistro_core::quantity::{JulianDay, Longitude, Utc};
//! use teistro_core::time::{LocalClock, LocalMeanTime, UtcOffset};
//!
//! let nepal = UtcOffset::try_from_seconds(5 * 3600 + 45 * 60).expect("in range");
//! assert_eq!(nepal.to_string(), "+05:45");
//!
//! let kathmandu = LocalMeanTime::new(Longitude::try_new(85.324).expect("in range"));
//! let noon_utc = JulianDay::<Utc>::try_new(2_460_413.0).expect("finite");
//! assert_eq!(kathmandu.offset_at(noon_utc).to_string(), "+05:41:18");
//! ```

use core::fmt;

use crate::quantity::{InvalidValue, JulianDay, Longitude, Utc};

/// An offset from UTC in whole seconds, east positive, within ±18 hours
/// (the bound `java.time` and RFC 3339 implementations share; tzdb's
/// widest offsets are inside it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct UtcOffset(i32);

impl UtcOffset {
    /// The widest offset either side of UTC, in seconds.
    pub const MAX_SECONDS: i32 = 18 * 3600;
    /// UTC itself.
    pub const UTC: UtcOffset = UtcOffset(0);
    /// What the quantity is called in messages.
    pub const NAME: &'static str = "UTC offset";

    /// Accepts an offset inside ±18 hours.
    ///
    /// # Errors
    ///
    /// An offset outside the range.
    pub fn try_from_seconds(seconds: i32) -> Result<UtcOffset, InvalidValue> {
        if (-UtcOffset::MAX_SECONDS..=UtcOffset::MAX_SECONDS).contains(&seconds) {
            Ok(UtcOffset(seconds))
        } else {
            Err(InvalidValue {
                quantity: UtcOffset::NAME,
                value: seconds.to_string(),
                accepted: "-64800 to 64800 seconds",
                field: None,
            })
        }
    }

    /// An offset written as a literal in a constant: hours, minutes and
    /// seconds east of UTC, negated for a western offset.
    ///
    /// # Panics
    ///
    /// When the parts do not describe an offset inside ±18 hours; in a
    /// constant that is a compile error, which is what the constructor is
    /// for. Runtime values go through [`UtcOffset::try_from_seconds`].
    #[must_use]
    pub const fn literal(hours: i32, minutes: i32, seconds: i32) -> UtcOffset {
        assert!(
            minutes >= 0 && minutes < 60 && seconds >= 0 && seconds < 60,
            "minutes and seconds are 0 to 59; the sign is on the hours"
        );
        let magnitude = hours.abs() * 3600 + minutes * 60 + seconds;
        let total = if hours < 0 { -magnitude } else { magnitude };
        assert!(
            total >= -UtcOffset::MAX_SECONDS && total <= UtcOffset::MAX_SECONDS,
            "a UTC offset is inside 18 hours either side"
        );
        UtcOffset(total)
    }

    /// The offset in seconds, east positive.
    #[must_use]
    pub const fn seconds(self) -> i32 {
        self.0
    }

    /// The offset in days, for Julian-day arithmetic.
    #[must_use]
    pub fn days(self) -> f64 {
        f64::from(self.0) / 86_400.0
    }
}

impl fmt::Display for UtcOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { '-' } else { '+' };
        let magnitude = self.0.abs();
        let (hours, minutes, seconds) = (magnitude / 3600, magnitude / 60 % 60, magnitude % 60);
        write!(f, "{sign}{hours:02}:{minutes:02}")?;
        if seconds != 0 {
            write!(f, ":{seconds:02}")?;
        }
        Ok(())
    }
}

impl<'de> serde::Deserialize<'de> for UtcOffset {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<UtcOffset, D::Error> {
        let seconds = i32::deserialize(deserializer)?;
        UtcOffset::try_from_seconds(seconds).map_err(serde::de::Error::custom)
    }
}

/// A clock: which offset from UTC applied at an instant. A fixed offset,
/// local mean time and a zone's history all answer the same question, so
/// a calendar that places an instant on a civil day takes any of them.
pub trait LocalClock: Send + Sync {
    /// The offset in force at the instant.
    fn offset_at(&self, instant: JulianDay<Utc>) -> UtcOffset;

    /// The clock's name for provenance stamps (`+05:45`, `LMT 85.324°E`,
    /// `Asia/Kathmandu tzdb 2026a`).
    fn describe(&self) -> String;

    /// The local Julian day of the instant: the UTC value plus the offset,
    /// so that its integer part and fraction are the local civil day and
    /// the time of day.
    fn local_jd(&self, instant: JulianDay<Utc>) -> f64 {
        instant.get() + self.offset_at(instant).days()
    }
}

impl LocalClock for UtcOffset {
    fn offset_at(&self, _instant: JulianDay<Utc>) -> UtcOffset {
        *self
    }

    fn describe(&self) -> String {
        self.to_string()
    }
}

impl<C: LocalClock + ?Sized> LocalClock for &C {
    fn offset_at(&self, instant: JulianDay<Utc>) -> UtcOffset {
        (**self).offset_at(instant)
    }

    fn describe(&self) -> String {
        (**self).describe()
    }
}

/// Local mean time at a longitude: four minutes of clock per degree east,
/// rounded to the second (`docs/03-design/time-and-timezone.md`, §4), which
/// is the time a classical almanac reckons in and the basis tzdb's own
/// pre-zone stubs use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalMeanTime {
    longitude: Longitude,
}

impl LocalMeanTime {
    /// The clock of a longitude.
    #[must_use]
    pub const fn new(longitude: Longitude) -> LocalMeanTime {
        LocalMeanTime { longitude }
    }

    /// The longitude.
    #[must_use]
    pub const fn longitude(self) -> Longitude {
        self.longitude
    }

    /// The offset: the longitude in degrees times 240 seconds, rounded.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a rounded value inside ±43200"
    )]
    pub fn offset(self) -> UtcOffset {
        UtcOffset((self.longitude.get() * 240.0).round() as i32)
    }
}

impl LocalClock for LocalMeanTime {
    fn offset_at(&self, _instant: JulianDay<Utc>) -> UtcOffset {
        self.offset()
    }

    fn describe(&self) -> String {
        format!("LMT {}", self.longitude)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn offsets_validate_render_and_round_trip() {
        assert_eq!(UtcOffset::try_from_seconds(0).unwrap(), UtcOffset::UTC);
        assert_eq!(UtcOffset::literal(5, 45, 0).seconds(), 20_700);
        assert_eq!(UtcOffset::literal(-3, 30, 0).seconds(), -12_600);
        assert_eq!(UtcOffset::literal(-3, 30, 0).to_string(), "-03:30");
        assert_eq!(UtcOffset::literal(5, 41, 16).to_string(), "+05:41:16");
        assert_eq!(UtcOffset::UTC.to_string(), "+00:00");
        let wrong = UtcOffset::try_from_seconds(19 * 3600).unwrap_err();
        assert_eq!(
            wrong.to_string(),
            "UTC offset 68400 is outside -64800 to 64800 seconds"
        );
        let json = serde_json::to_string(&UtcOffset::literal(5, 30, 0)).unwrap();
        assert_eq!(json, "19800");
        let back: UtcOffset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, UtcOffset::literal(5, 30, 0));
        assert!(serde_json::from_str::<UtcOffset>("100000").is_err());
        assert!((UtcOffset::literal(6, 0, 0).days() - 0.25).abs() < 1e-15);
    }

    #[test]
    fn local_mean_time_is_four_minutes_per_degree() {
        let lmt = LocalMeanTime::new(Longitude::try_new(85.324).unwrap());
        assert_eq!(lmt.offset().seconds(), 20_478);
        assert_eq!(lmt.describe(), "LMT 85.324°");
        let west = LocalMeanTime::new(Longitude::try_new(-74.0).unwrap());
        assert_eq!(west.offset().to_string(), "-04:56");
        let instant = JulianDay::<Utc>::try_new(2_460_000.0).unwrap();
        assert!((lmt.local_jd(instant) - (2_460_000.0 + 20_478.0 / 86_400.0)).abs() < 1e-12);
        let fixed: &dyn LocalClock = &UtcOffset::literal(5, 45, 0);
        assert_eq!(fixed.describe(), "+05:45");
        assert_eq!((&fixed).offset_at(instant).seconds(), 20_700);
    }
}

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
///
/// ```
/// use teistro_core::time::UtcOffset;
///
/// // Seconds east, as JSON carries it.
/// let india: UtcOffset = serde_json::from_str("19800").unwrap();
/// assert_eq!(india.to_string(), "+05:30");
/// assert!(serde_json::from_str::<UtcOffset>("100000").is_err());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct UtcOffset(i32);

impl UtcOffset {
    /// The widest offset either side of UTC, in seconds.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    ///
    /// assert!(UtcOffset::try_from_seconds(UtcOffset::MAX_SECONDS).is_ok());
    /// assert!(UtcOffset::try_from_seconds(-UtcOffset::MAX_SECONDS - 1).is_err());
    /// ```
    pub const MAX_SECONDS: i32 = 18 * 3600;
    /// UTC itself.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    ///
    /// assert_eq!(UtcOffset::UTC.to_string(), "+00:00");
    /// ```
    pub const UTC: UtcOffset = UtcOffset(0);
    /// What the quantity is called in messages.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    ///
    /// assert!(UtcOffset::try_from_seconds(i32::MAX).unwrap_err().to_string().starts_with(UtcOffset::NAME));
    /// ```
    pub const NAME: &'static str = "UTC offset";

    /// Accepts an offset inside ±18 hours.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    ///
    /// assert_eq!(UtcOffset::try_from_seconds(-12_600)?.to_string(), "-03:30");
    /// let refused = UtcOffset::try_from_seconds(19 * 3600).unwrap_err();
    /// assert_eq!(refused.to_string(), "UTC offset 68400 is outside -64800 to 64800 seconds");
    /// # Ok::<(), teistro_core::quantity::InvalidValue>(())
    /// ```
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
    /// ```
    /// use teistro_core::time::UtcOffset;
    ///
    /// const NEPAL: UtcOffset = UtcOffset::literal(5, 45, 0);
    /// const NEWFOUNDLAND: UtcOffset = UtcOffset::literal(-3, 30, 0);
    /// assert_eq!((NEPAL.seconds(), NEWFOUNDLAND.seconds()), (20_700, -12_600));
    /// ```
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
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    ///
    /// assert_eq!(UtcOffset::literal(5, 41, 16).seconds(), 20_476);
    /// ```
    #[must_use]
    pub const fn seconds(self) -> i32 {
        self.0
    }

    /// The offset in days, for Julian-day arithmetic.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    ///
    /// assert_eq!(UtcOffset::literal(6, 0, 0).days(), 0.25);
    /// ```
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
///
/// ```
/// use teistro_core::quantity::{JulianDay, Utc};
/// use teistro_core::time::{LocalClock, UtcOffset};
///
/// /// A clock that moved from +05:30 to +05:45 at the start of 1986.
/// struct Nepal;
///
/// impl LocalClock for Nepal {
///     fn offset_at(&self, instant: JulianDay<Utc>) -> UtcOffset {
///         if instant.get() < 2_446_431.5 - 5.5 / 24.0 { UtcOffset::literal(5, 30, 0) } else { UtcOffset::literal(5, 45, 0) }
///     }
///     fn describe(&self) -> String {
///         String::from("Nepal, by hand")
///     }
/// }
///
/// assert_eq!(Nepal.offset_at(JulianDay::literal(2_446_000.5)).to_string(), "+05:30");
/// assert_eq!(Nepal.offset_at(JulianDay::literal(2_460_310.5)).to_string(), "+05:45");
/// ```
pub trait LocalClock: Send + Sync {
    /// The offset in force at the instant.
    ///
    /// ```
    /// use teistro_core::quantity::{JulianDay, Utc};
    /// use teistro_core::time::{LocalClock, UtcOffset};
    ///
    /// // A fixed offset is a clock that never changes.
    /// let nepal = UtcOffset::literal(5, 45, 0);
    /// assert_eq!(nepal.offset_at(JulianDay::<Utc>::J2000), nepal);
    /// ```
    fn offset_at(&self, instant: JulianDay<Utc>) -> UtcOffset;

    /// The clock's name for provenance stamps (`+05:45`, `LMT 85.324°E`,
    /// `Asia/Kathmandu tzdb 2026a`).
    ///
    /// ```
    /// use teistro_core::quantity::Longitude;
    /// use teistro_core::time::{LocalClock, LocalMeanTime, UtcOffset};
    ///
    /// assert_eq!(UtcOffset::literal(5, 45, 0).describe(), "+05:45");
    /// assert_eq!(LocalMeanTime::new(Longitude::literal(85.324)).describe(), "LMT 85.324°");
    /// ```
    fn describe(&self) -> String;

    /// The local Julian day of the instant: the UTC value plus the offset,
    /// so that its integer part and fraction are the local civil day and
    /// the time of day.
    ///
    /// ```
    /// use teistro_core::quantity::{JulianDay, Utc};
    /// use teistro_core::time::{LocalClock, UtcOffset};
    ///
    /// // 18:00 UTC on 31 December 2023 is a quarter to midnight in Kathmandu.
    /// let local = UtcOffset::literal(5, 45, 0).local_jd(JulianDay::<Utc>::literal(2_460_310.25));
    /// assert!((local - 2_460_310.489_583_333).abs() < 1e-8);
    /// ```
    fn local_jd(&self, instant: JulianDay<Utc>) -> f64 {
        instant.get() + self.offset_at(instant).days()
    }

    /// The offset at the instant with the source that gave it, as an
    /// answer states it (`docs/03-design/time-and-timezone.md` §3.4). A
    /// clock that says nothing else is one the caller stated.
    ///
    /// ```
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_core::time::{LocalClock, UtcOffset, ZoneSource};
    ///
    /// let read = UtcOffset::literal(5, 45, 0).reading(JulianDay::J2000);
    /// assert_eq!((read.source, read.tzdb_version.as_str()), (ZoneSource::Manual, "manual"));
    /// ```
    fn reading(&self, instant: JulianDay<Utc>) -> ClockReading {
        ClockReading {
            offset: self.offset_at(instant),
            source: ZoneSource::Manual,
            era: ZoneEra::Current,
            tzdb_version: String::from("manual"),
            abbreviation: None,
        }
    }
}

/// Where an offset came from.
///
/// ```
/// use teistro_core::time::ZoneSource;
///
/// assert_eq!(serde_json::to_string(&ZoneSource::LocalMean).unwrap(), r#""LOCAL_MEAN""#);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ClockSource"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZoneSource {
    /// The zone database.
    Iana,
    /// Local mean time from the longitude.
    LocalMean,
    /// A fixed offset the consumer stated.
    Manual,
}

impl ZoneSource {
    /// The key it is serialised as: `IANA`, `LOCAL_MEAN`, `MANUAL`.
    ///
    /// ```
    /// use teistro_core::time::ZoneSource;
    ///
    /// assert_eq!(ZoneSource::LocalMean.key(), "LOCAL_MEAN");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            ZoneSource::Iana => "IANA",
            ZoneSource::LocalMean => "LOCAL_MEAN",
            ZoneSource::Manual => "MANUAL",
        }
    }
}

/// Which rules produced an offset.
///
/// ```
/// use teistro_core::time::ZoneEra;
///
/// assert_eq!(serde_json::to_string(&ZoneEra::BeforeRules).unwrap(), r#""BEFORE_RULES""#);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ClockEra"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZoneEra {
    /// An offset the zone applies in the database's own year.
    Current,
    /// An offset from the zone's earlier rules.
    Historical,
    /// Before the zone's first rule: the database's local-mean-time stub.
    BeforeRules,
}

/// A clock's offset at an instant and what gave it: what an answer carries
/// so a caller does not resolve its zone again.
///
/// ```
/// use teistro_core::quantity::{JulianDay, Longitude};
/// use teistro_core::time::{LocalClock, LocalMeanTime, ZoneSource};
///
/// let read = LocalMeanTime::new(Longitude::literal(85.324)).reading(JulianDay::J2000);
/// assert_eq!((read.offset.seconds(), read.source), (20_478, ZoneSource::LocalMean));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ClockReading {
    /// The offset.
    pub offset: UtcOffset,
    /// Where it came from.
    pub source: ZoneSource,
    /// Which rules produced it.
    pub era: ZoneEra,
    /// The database's release, `lmt` or `manual`.
    pub tzdb_version: String,
    /// The zone's abbreviation for the row, when the database has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,
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

    fn reading(&self, instant: JulianDay<Utc>) -> ClockReading {
        (**self).reading(instant)
    }
}

/// Local mean time at a longitude: four minutes of clock per degree east,
/// rounded to the second (`docs/03-design/time-and-timezone.md`, §4), which
/// is the time a classical almanac reckons in and the basis tzdb's own
/// pre-zone stubs use.
///
/// ```
/// use teistro_core::quantity::{JulianDay, Utc};
/// use teistro_core::time::LocalClock;
/// use teistro_core::quantity::Longitude;
/// use teistro_core::time::LocalMeanTime;
///
/// let kathmandu = LocalMeanTime::new(Longitude::literal(85.324));
/// // Four minutes a degree, whatever the date.
/// assert_eq!(kathmandu.offset_at(JulianDay::<Utc>::J2000), kathmandu.offset());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalMeanTime {
    longitude: Longitude,
}

impl LocalMeanTime {
    /// The clock of a longitude.
    ///
    /// ```
    /// use teistro_core::quantity::Longitude;
    /// use teistro_core::time::LocalMeanTime;
    ///
    /// let kathmandu = LocalMeanTime::new(Longitude::literal(85.324));
    /// assert_eq!(kathmandu.offset().to_string(), "+05:41:18");
    /// ```
    #[must_use]
    pub const fn new(longitude: Longitude) -> LocalMeanTime {
        LocalMeanTime { longitude }
    }

    /// The longitude.
    ///
    /// ```
    /// use teistro_core::quantity::Longitude;
    /// use teistro_core::time::LocalMeanTime;
    ///
    /// let kathmandu = LocalMeanTime::new(Longitude::literal(85.324));
    /// assert_eq!(kathmandu.longitude().get(), 85.324);
    /// ```
    #[must_use]
    pub const fn longitude(self) -> Longitude {
        self.longitude
    }

    /// The offset: the longitude in degrees times 240 seconds, rounded.
    ///
    /// ```
    /// use teistro_core::quantity::Longitude;
    /// use teistro_core::time::LocalMeanTime;
    ///
    /// let kathmandu = LocalMeanTime::new(Longitude::literal(85.324));
    /// // 85.324 x 240 s is 20477.76 s, rounded to the second.
    /// assert_eq!(kathmandu.offset().seconds(), 20_478);
    /// ```
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

    fn reading(&self, _instant: JulianDay<Utc>) -> ClockReading {
        ClockReading {
            offset: self.offset(),
            source: ZoneSource::LocalMean,
            era: ZoneEra::Current,
            tzdb_version: String::from("lmt"),
            abbreviation: None,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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

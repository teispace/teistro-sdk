//! The time-zone port: what the SDK needs from a zone database and
//! nothing else (`docs/03-design/time-and-timezone.md`, §2 and §4).
//!
//! A provider answers three questions: which offset was in force at an
//! instant (with the abbreviation and whether the instant precedes the
//! zone's first rule, tzdb's local-mean-time stub); which offsets could
//! apply to a civil time (one, none across a gap, or two across an
//! overlap); and which offsets the zone applies in the database's own
//! year, so a resolution can say whether it used today's rules. The
//! policies that choose among candidates, and the metadata a chart
//! keeps, live in the time layer, not here.

use core::fmt;

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;

/// The offset in force at an instant, with what a replay stamps.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OffsetInfo {
    /// The offset.
    pub offset: UtcOffset,
    /// The zone's abbreviation for the row (`+0545`, `EDT`, `LMT`).
    pub abbreviation: String,
    /// Whether the row is daylight-saving time.
    pub is_dst: bool,
    /// Whether the instant precedes the zone's first rule, so the offset
    /// is the database's local-mean-time stub.
    pub before_rules: bool,
}

/// The offsets a civil time could take in a zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocalCandidates {
    /// One offset applies.
    Unambiguous {
        /// The offset.
        offset: UtcOffset,
    },
    /// The civil time does not exist: the clocks jumped over it.
    Gap {
        /// The offset before the jump.
        before: UtcOffset,
        /// The offset after the jump.
        after: UtcOffset,
    },
    /// The civil time occurred twice: the clocks were set back over it.
    Overlap {
        /// The offset of the first occurrence, the earlier instant.
        earlier: UtcOffset,
        /// The offset of the second occurrence, the later instant.
        later: UtcOffset,
    },
}

/// A civil time as a provider takes it: the seconds since 1970-01-01
/// 00:00 read as if the civil clock were UTC, with the nanoseconds.
/// Calendar-free, so a provider needs no calendar to place it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalSeconds {
    /// Whole seconds.
    pub seconds: i64,
    /// Nanoseconds within the second, below one thousand million.
    pub nanos: u32,
}

impl fmt::Display for LocalSeconds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:09} local seconds", self.seconds, self.nanos)
    }
}

/// A zone database.
pub trait TimeZoneProvider: Send + Sync {
    /// The database's version (`2026a`), stamped on every resolution.
    fn version(&self) -> &str;

    /// Whether the database knows a zone.
    fn has_zone(&self, zone: &str) -> bool;

    /// Every zone the database knows, for suggestions and listings.
    fn zones(&self) -> Vec<String>;

    /// The offset in force at an instant.
    ///
    /// # Errors
    ///
    /// An unknown zone, or an instant the database cannot place.
    fn offset_at(&self, zone: &str, instant: JulianDay<Utc>) -> Result<OffsetInfo, Error>;

    /// The offsets a civil time could take.
    ///
    /// # Errors
    ///
    /// An unknown zone, or a civil time the database cannot place.
    fn candidates(&self, zone: &str, local: LocalSeconds) -> Result<LocalCandidates, Error>;

    /// The offsets the zone applies in the database's own year: one for
    /// a zone without daylight saving, two with it. A resolution whose
    /// offset is not among them used earlier rules.
    ///
    /// # Errors
    ///
    /// An unknown zone.
    fn current_offsets(&self, zone: &str) -> Result<Vec<UtcOffset>, Error>;
}

impl<P: TimeZoneProvider + ?Sized> TimeZoneProvider for &P {
    fn version(&self) -> &str {
        (**self).version()
    }

    fn has_zone(&self, zone: &str) -> bool {
        (**self).has_zone(zone)
    }

    fn zones(&self) -> Vec<String> {
        (**self).zones()
    }

    fn offset_at(&self, zone: &str, instant: JulianDay<Utc>) -> Result<OffsetInfo, Error> {
        (**self).offset_at(zone, instant)
    }

    fn candidates(&self, zone: &str, local: LocalSeconds) -> Result<LocalCandidates, Error> {
        (**self).candidates(zone, local)
    }

    fn current_offsets(&self, zone: &str) -> Result<Vec<UtcOffset>, Error> {
        (**self).current_offsets(zone)
    }
}

/// The error a provider raises for a zone it does not know, with the
/// nearest names it does know as the hint.
#[must_use]
pub fn unknown_zone(zone: &str, suggestions: &[String]) -> Error {
    let error = Error::invalid_arg(format!("unknown time zone `{zone}`")).with_field("zone");
    if suggestions.is_empty() {
        error.with_hint("an IANA zone name such as `Asia/Kathmandu`")
    } else {
        error.with_hint(format!("did you mean {}?", suggestions.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn the_unknown_zone_error_names_the_field_and_the_nearest_names() {
        let error = unknown_zone("Asia/Kathmandoo", &[String::from("Asia/Kathmandu")]);
        assert_eq!(error.field(), Some("zone"));
        assert_eq!(error.hint(), Some("did you mean Asia/Kathmandu?"));
        assert!(
            unknown_zone("Nowhere", &[])
                .hint()
                .unwrap()
                .contains("Asia/Kathmandu")
        );
        let local = LocalSeconds {
            seconds: 12,
            nanos: 5,
        };
        assert_eq!(local.to_string(), "12.000000005 local seconds");
        let json = serde_json::to_string(&LocalCandidates::Unambiguous {
            offset: UtcOffset::literal(5, 45, 0),
        })
        .unwrap();
        assert_eq!(json, "{\"kind\":\"UNAMBIGUOUS\",\"offset\":20700}");
    }
}

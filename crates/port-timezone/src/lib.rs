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

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use core::fmt;

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;

/// The offset in force at an instant, with what a replay stamps.
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_core::time::UtcOffset;
/// use teistro_port_timezone::{FixedZone, TimeZoneProvider};
///
/// let nepal = FixedZone::new("Asia/Kathmandu", UtcOffset::literal(5, 45, 0));
/// let info = nepal.offset_at("Asia/Kathmandu", JulianDay::literal(2_460_482.5))?;
/// assert_eq!(info.offset.seconds(), 20_700);
/// assert!(!info.is_dst && !info.before_rules);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
///
/// ```
/// use teistro_core::time::UtcOffset;
/// use teistro_port_timezone::LocalCandidates;
///
/// // New York's clocks jumped from 02:00 to 03:00 on 10 March 2024.
/// let gap = LocalCandidates::Gap {
///     before: UtcOffset::literal(-5, 0, 0),
///     after: UtcOffset::literal(-4, 0, 0),
/// };
/// assert_eq!(serde_json::to_value(gap).unwrap()["kind"], "GAP");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
///
/// ```
/// use teistro_port_timezone::LocalSeconds;
///
/// // 1 January 2024, 06:00 on the civil clock, and a half second.
/// let local = LocalSeconds { seconds: 1_704_088_800, nanos: 500_000_000 };
/// assert_eq!(local.to_string(), "1704088800.500000000 local seconds");
/// assert!(local < LocalSeconds { seconds: 1_704_088_801, nanos: 0 });
/// ```
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
///
/// ```
/// use teistro_core::time::UtcOffset;
/// use teistro_port_timezone::{FixedZone, TimeZoneProvider};
///
/// let nepal = FixedZone::new("Asia/Kathmandu", UtcOffset::literal(5, 45, 0));
/// // A reference to a provider is a provider.
/// fn version_of(zones: impl TimeZoneProvider) -> String {
///     zones.version().to_string()
/// }
/// assert_eq!(version_of(&nepal), "fixed");
/// assert!(nepal.has_zone("Asia/Kathmandu"));
/// ```
pub trait TimeZoneProvider: Send + Sync {
    /// The database's version (`2026a`), stamped on every resolution.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    /// use teistro_port_timezone::{FixedZone, TimeZoneProvider};
    ///
    /// let nepal = FixedZone::new("Asia/Kathmandu", UtcOffset::literal(5, 45, 0));
    /// assert_eq!(nepal.version(), "fixed");
    /// ```
    fn version(&self) -> &str;

    /// Whether the database knows a zone.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    /// use teistro_port_timezone::{FixedZone, TimeZoneProvider};
    ///
    /// let nepal = FixedZone::new("Asia/Kathmandu", UtcOffset::literal(5, 45, 0));
    /// assert!(nepal.has_zone("Asia/Kathmandu"));
    /// assert!(!nepal.has_zone("Asia/Katmandu"));
    /// ```
    fn has_zone(&self, zone: &str) -> bool;

    /// Every zone the database knows, for suggestions and listings.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    /// use teistro_port_timezone::{FixedZone, TimeZoneProvider};
    ///
    /// let nepal = FixedZone::new("Asia/Kathmandu", UtcOffset::literal(5, 45, 0));
    /// assert_eq!(nepal.zones(), ["Asia/Kathmandu"]);
    /// ```
    fn zones(&self) -> Vec<String>;

    /// The offset in force at an instant.
    ///
    /// ```
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_core::time::UtcOffset;
    /// use teistro_port_timezone::{FixedZone, TimeZoneProvider};
    ///
    /// let nepal = FixedZone::new("Asia/Kathmandu", UtcOffset::literal(5, 45, 0));
    /// let instant = JulianDay::literal(2_451_545.0);
    /// assert_eq!(nepal.offset_at("Asia/Kathmandu", instant)?.offset, UtcOffset::literal(5, 45, 0));
    /// assert_eq!(nepal.offset_at("Mars/Olympus", instant).unwrap_err().field(), Some("zone"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An unknown zone, or an instant the database cannot place.
    fn offset_at(&self, zone: &str, instant: JulianDay<Utc>) -> Result<OffsetInfo, Error>;

    /// The offsets a civil time could take.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    /// use teistro_port_timezone::{FixedZone, LocalCandidates, LocalSeconds, TimeZoneProvider};
    ///
    /// let nepal = FixedZone::new("Asia/Kathmandu", UtcOffset::literal(5, 45, 0));
    /// let local = LocalSeconds { seconds: 1_704_088_800, nanos: 0 };
    /// assert_eq!(
    ///     nepal.candidates("Asia/Kathmandu", local)?,
    ///     LocalCandidates::Unambiguous { offset: UtcOffset::literal(5, 45, 0) }
    /// );
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An unknown zone, or a civil time the database cannot place.
    fn candidates(&self, zone: &str, local: LocalSeconds) -> Result<LocalCandidates, Error>;

    /// The offsets the zone applies in the database's own year: one for
    /// a zone without daylight saving, two with it. A resolution whose
    /// offset is not among them used earlier rules.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    /// use teistro_port_timezone::{FixedZone, TimeZoneProvider};
    ///
    /// let nepal = FixedZone::new("Asia/Kathmandu", UtcOffset::literal(5, 45, 0));
    /// // One offset: Nepal keeps no daylight saving.
    /// assert_eq!(nepal.current_offsets("Asia/Kathmandu")?.len(), 1);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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

/// A shared database answers as the database does.
impl<P: TimeZoneProvider + ?Sized> TimeZoneProvider for std::sync::Arc<P> {
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
///
/// ```
/// use teistro_port_timezone::unknown_zone;
///
/// let error = unknown_zone("Asia/Katmandu", &[String::from("Asia/Kathmandu")]);
/// assert_eq!(error.field(), Some("zone"));
/// assert_eq!(error.hint(), Some("did you mean Asia/Kathmandu?"));
/// ```
#[must_use]
pub fn unknown_zone(zone: &str, suggestions: &[String]) -> Error {
    let error = Error::invalid_arg(format!("unknown time zone `{zone}`")).with_field("zone");
    if suggestions.is_empty() {
        error.with_hint("an IANA zone name such as `Asia/Kathmandu`")
    } else {
        error.with_hint(format!("did you mean {}?", suggestions.join(", ")))
    }
}

/// One zone at one fixed offset all year, refusing every other: a zone
/// with no daylight saving and no history, such as Nepal's since 1986, or
/// a provider for a test.
///
/// Its abbreviation is the offset written as tzdb writes a numbered one
/// (`+0545`), and no instant precedes its rules.
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_core::time::UtcOffset;
/// use teistro_port_timezone::{FixedZone, TimeZoneProvider};
///
/// let nepal = FixedZone::new("Asia/Kathmandu", UtcOffset::literal(5, 45, 0));
/// let info = nepal.offset_at("Asia/Kathmandu", JulianDay::literal(2_460_482.5))?;
/// assert_eq!(info.abbreviation, "+0545");
/// assert!(nepal.offset_at("Asia/Katmandu", JulianDay::literal(2_460_482.5)).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedZone {
    zone: String,
    offset: UtcOffset,
}

impl FixedZone {
    /// The zone `zone`, at `offset`.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    /// use teistro_port_timezone::{FixedZone, TimeZoneProvider};
    ///
    /// let zone = FixedZone::new("Etc/GMT+5", UtcOffset::literal(-5, 0, 0));
    /// assert_eq!(zone.zones(), ["Etc/GMT+5"]);
    /// ```
    #[must_use]
    pub fn new(zone: impl Into<String>, offset: UtcOffset) -> FixedZone {
        FixedZone {
            zone: zone.into(),
            offset,
        }
    }

    /// The offset, for a zone it knows.
    fn known(&self, zone: &str) -> Result<UtcOffset, Error> {
        if self.has_zone(zone) {
            Ok(self.offset)
        } else {
            Err(unknown_zone(zone, &self.zones()))
        }
    }

    /// The offset as tzdb abbreviates a numbered one: `+0545`, `-0500`,
    /// and the seconds only when there are some.
    fn abbreviation(&self) -> String {
        let total = self.offset.seconds();
        let sign = if total < 0 { '-' } else { '+' };
        let magnitude = total.unsigned_abs();
        let (hours, minutes, seconds) = (magnitude / 3600, magnitude / 60 % 60, magnitude % 60);
        if seconds == 0 {
            format!("{sign}{hours:02}{minutes:02}")
        } else {
            format!("{sign}{hours:02}{minutes:02}{seconds:02}")
        }
    }
}

impl TimeZoneProvider for FixedZone {
    #[allow(
        clippy::unnecessary_literal_bound,
        reason = "the trait's signature, which a real provider borrows from itself"
    )]
    fn version(&self) -> &str {
        "fixed"
    }

    fn has_zone(&self, zone: &str) -> bool {
        zone == self.zone
    }

    fn zones(&self) -> Vec<String> {
        vec![self.zone.clone()]
    }

    fn offset_at(&self, zone: &str, _: JulianDay<Utc>) -> Result<OffsetInfo, Error> {
        self.known(zone).map(|offset| OffsetInfo {
            offset,
            abbreviation: self.abbreviation(),
            is_dst: false,
            before_rules: false,
        })
    }

    fn candidates(&self, zone: &str, _: LocalSeconds) -> Result<LocalCandidates, Error> {
        self.known(zone)
            .map(|offset| LocalCandidates::Unambiguous { offset })
    }

    fn current_offsets(&self, zone: &str) -> Result<Vec<UtcOffset>, Error> {
        self.known(zone).map(|offset| vec![offset])
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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

    fn kathmandu() -> FixedZone {
        FixedZone::new("Asia/Kathmandu", UtcOffset::literal(5, 45, 0))
    }

    #[test]
    fn a_fixed_zone_abbreviates_its_offset_as_tzdb_numbers_one() {
        let at = |seconds| {
            FixedZone::new("Etc/Test", UtcOffset::try_from_seconds(seconds).unwrap()).abbreviation()
        };
        assert_eq!(kathmandu().abbreviation(), "+0545");
        assert_eq!(at(-5 * 3600), "-0500");
        assert_eq!(at(0), "+0000");
        assert_eq!(at(-(3600 + 60 + 5)), "-010105");
    }

    /// A borrowed or shared provider answers every question as the
    /// provider does, refusals included, so the blanket impls forward
    /// each method; one given a default later and left out of an impl
    /// would answer the default through it and fail here.
    #[test]
    fn a_borrowed_provider_answers_as_the_provider() {
        fn answers<P: TimeZoneProvider>(provider: &P, zone: &str) -> String {
            let instant = JulianDay::<Utc>::try_new(2_451_544.5).unwrap();
            let local = LocalSeconds {
                seconds: 946_684_800,
                nanos: 0,
            };
            format!(
                "{} {} {:?} {:?} {:?} {:?}",
                provider.version(),
                provider.has_zone(zone),
                provider.zones(),
                provider.offset_at(zone, instant),
                provider.candidates(zone, local),
                provider.current_offsets(zone)
            )
        }
        for zone in ["Asia/Kathmandu", "Asia/Kathmandoo"] {
            assert_eq!(answers(&&kathmandu(), zone), answers(&kathmandu(), zone));
            let shared = std::sync::Arc::new(kathmandu());
            assert_eq!(answers(&shared, zone), answers(&kathmandu(), zone));
        }
        assert!(answers(&&kathmandu(), "Asia/Kathmandoo").contains("did you mean"));
    }
}

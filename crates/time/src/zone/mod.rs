//! Zone resolution: a civil date-time in a zone to an instant under the
//! daylight-saving policies, with the metadata a stored chart keeps to
//! replay the resolution under a newer database, and the way back from
//! an instant to civil time (`docs/03-design/time-and-timezone.md`, §3.2
//! and §4).

pub mod embedded;
mod engine;
pub mod pack;

pub use engine::ZoneClock;

use core::fmt;

use teistro_calendar::solar::SolarModel;
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay, shipped};
use teistro_core::catalogue::Calendar;
use teistro_core::error::{Detail, Error};
use teistro_core::quantity::{JulianDay, Longitude, Place, Utc};
use teistro_core::settings::{DstGap, DstOverlap, PolarDayPolicy, Settings, UnknownTime};
use teistro_core::time::{ClockReading, LocalClock, LocalMeanTime, UtcOffset};
pub use teistro_core::time::{ZoneEra, ZoneSource};
use teistro_port_timezone::{LocalCandidates, LocalSeconds, TimeZoneProvider};

use crate::civil::{CivilDateTime, CivilTime, NANOS_PER_SECOND, SECONDS_PER_DAY};
use crate::leap;
use crate::local_day::local_day;

/// How a zone is given.
///
/// ```
/// use teistro_core::quantity::Longitude;
/// use teistro_core::time::UtcOffset;
/// use teistro_time::ZoneSpec;
///
/// assert_eq!(ZoneSpec::iana("Asia/Kathmandu").to_string(), "Asia/Kathmandu");
/// assert_eq!(ZoneSpec::fixed(UtcOffset::literal(5, 45, 0)).to_string(), "+05:45");
/// assert!(ZoneSpec::local_mean(Longitude::literal(85.324)).to_string().starts_with("LMT "));
///
/// // Serialised with its kind, as a stored chart keeps it.
/// let json = serde_json::to_string(&ZoneSpec::iana("Asia/Kathmandu")).unwrap();
/// assert_eq!(json, r#"{"kind":"IANA","zone":"Asia/Kathmandu"}"#);
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZoneSpec {
    /// A zone of the database, by its IANA name.
    Iana {
        /// The name (`Asia/Kathmandu`).
        zone: String,
    },
    /// Local mean time at a longitude: four minutes a degree.
    LocalMean {
        /// The longitude, east positive.
        longitude: Longitude,
    },
    /// A fixed offset the consumer states.
    Fixed {
        /// The offset.
        offset: UtcOffset,
    },
}

impl ZoneSpec {
    /// A zone of the database.
    ///
    /// ```
    /// use teistro_time::ZoneSpec;
    ///
    /// assert_eq!(ZoneSpec::iana("Europe/London"), ZoneSpec::Iana { zone: "Europe/London".into() });
    /// ```
    pub fn iana(zone: impl Into<String>) -> ZoneSpec {
        ZoneSpec::Iana { zone: zone.into() }
    }

    /// Local mean time at a longitude.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_core::catalogue::Calendar;
    /// # let noon = |y, m, d| CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, y, m, d), CivilTime::NOON);
    /// use teistro_core::quantity::Longitude;
    /// use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve};
    ///
    /// // Four minutes a degree: 85.324° east is 5 h 41 min 18 s ahead.
    /// let lmt = ZoneSpec::local_mean(Longitude::literal(85.324));
    /// let read = resolve(&noon(2024, 1, 1), &lmt, &Policy::default(), EmbeddedTzdb::shared())?;
    /// assert_eq!(read.zone.offset.seconds(), 20_478);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn local_mean(longitude: Longitude) -> ZoneSpec {
        ZoneSpec::LocalMean { longitude }
    }

    /// A fixed offset.
    ///
    /// ```
    /// use teistro_core::time::UtcOffset;
    /// use teistro_time::ZoneSpec;
    ///
    /// let offset = UtcOffset::literal(5, 45, 0);
    /// assert_eq!(ZoneSpec::fixed(offset), ZoneSpec::Fixed { offset });
    /// ```
    #[must_use]
    pub const fn fixed(offset: UtcOffset) -> ZoneSpec {
        ZoneSpec::Fixed { offset }
    }
}

impl ZoneSpec {
    /// The zone as a clock over a database.
    ///
    /// ```
    /// use teistro_time::{EmbeddedTzdb, ZoneSpec};
    ///
    /// let refused = ZoneSpec::iana("Asia/Kathmandoo").clock(EmbeddedTzdb::shared());
    /// assert!(refused.is_err_and(|why| why.field() == Some("zone")));
    /// ```
    ///
    /// # Errors
    ///
    /// A zone the database does not have, with the nearest names.
    pub fn clock<P: TimeZoneProvider>(&self, provider: P) -> Result<ProviderClock<P>, Error> {
        if let ZoneSpec::Iana { zone } = self {
            provider.offset_at(zone, JulianDay::<Utc>::J2000)?;
        }
        Ok(ProviderClock {
            provider,
            zone: self.clone(),
        })
    }
}

impl fmt::Display for ZoneSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoneSpec::Iana { zone } => f.write_str(zone),
            ZoneSpec::LocalMean { longitude } => write!(f, "LMT {longitude}"),
            ZoneSpec::Fixed { offset } => write!(f, "{offset}"),
        }
    }
}

/// The policies a resolution applies: the settings' time knobs.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::Calendar;
/// # let noon = |y, m, d| CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, y, m, d), CivilTime::NOON);
/// use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve};
/// use teistro_core::settings::DstOverlap;
/// let new_york = ZoneSpec::iana("America/New_York");
/// let twice = CivilDateTime::at(
///     CalendarDate::defined(Calendar::Gregorian, 2024, 11, 3),
///     CivilTime::new(1, 30, 0)?,
/// );
/// let later = Policy { overlap: DstOverlap::Later, ..Policy::default() };
///
/// let earlier = resolve(&twice, &new_york, &Policy::default(), EmbeddedTzdb::shared())?;
/// let second = resolve(&twice, &new_york, &later, EmbeddedTzdb::shared())?;
/// // The two 01:30s are an hour apart.
/// let apart = (second.instant.get() - earlier.instant.get()) * 86_400.0;
/// assert!((apart - 3600.0).abs() < 1e-3, "{apart}");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Policy {
    /// A civil time inside a gap.
    pub gap: DstGap,
    /// A civil time repeated by an overlap.
    pub overlap: DstOverlap,
    /// A civil time not given.
    pub unknown_time: UnknownTime,
}

impl Policy {
    /// The policies of a settings value.
    ///
    /// ```
    /// use teistro_core::settings;
    /// use teistro_time::Policy;
    ///
    /// assert_eq!(Policy::of(&settings::root()), Policy::default());
    /// ```
    #[must_use]
    pub const fn of(settings: &Settings) -> Policy {
        Policy {
            gap: settings.time.dst_gap,
            overlap: settings.time.dst_overlap,
            unknown_time: settings.time.unknown_time,
        }
    }
}

impl Default for Policy {
    /// The root profile's values: refuse a gap, take the earlier offset
    /// of an overlap, refuse a missing time.
    fn default() -> Policy {
        Policy {
            gap: DstGap::Error,
            overlap: DstOverlap::Earlier,
            unknown_time: UnknownTime::Refuse,
        }
    }
}

/// Which occurrence an overlap resolved to.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::Calendar;
/// # let noon = |y, m, d| CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, y, m, d), CivilTime::NOON);
/// use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve};
/// use teistro_time::{Chosen, DstOutcome};
///
/// use teistro_core::settings::DstOverlap;
/// let new_york = ZoneSpec::iana("America/New_York");
/// let twice = CivilDateTime::at(
///     CalendarDate::defined(Calendar::Gregorian, 2024, 11, 3),
///     CivilTime::new(1, 30, 0)?,
/// );
/// let later = Policy { overlap: DstOverlap::Later, ..Policy::default() };
/// let read = resolve(&twice, &new_york, &later, EmbeddedTzdb::shared())?;
/// assert_eq!(read.zone.dst, DstOutcome::Overlap { chosen: Chosen::Later });
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Chosen {
    /// The first occurrence, the earlier instant.
    Earlier,
    /// The second occurrence, the later instant.
    Later,
}

/// What the daylight-saving policy did.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::Calendar;
/// # let noon = |y, m, d| CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, y, m, d), CivilTime::NOON);
/// use teistro_core::settings::DstGap;
/// use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve};
/// use teistro_time::DstOutcome;
///
/// // 02:30 on 10 March 2024 never happened in New York.
/// let lost = CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, 2024, 3, 10), CivilTime::new(2, 30, 0)?);
/// let shift = Policy { gap: DstGap::ShiftForward, ..Policy::default() };
/// let read = resolve(&lost, &ZoneSpec::iana("America/New_York"), &shift, EmbeddedTzdb::shared())?;
/// assert_eq!(read.zone.dst, DstOutcome::Gap { shifted_by_seconds: 3600 });
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DstOutcome {
    /// The civil time was unambiguous.
    None,
    /// The civil time fell in a gap and was shifted forward past it.
    Gap {
        /// The gap's length, seconds.
        shifted_by_seconds: i32,
    },
    /// The civil time fell in an overlap and one occurrence was chosen.
    Overlap {
        /// Which.
        chosen: Chosen,
    },
}

/// What a resolution wants the consumer to know.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::Calendar;
/// # let noon = |y, m, d| CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, y, m, d), CivilTime::NOON);
/// use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve};
/// use teistro_time::Warning;
///
/// let kathmandu = ZoneSpec::iana("Asia/Kathmandu");
/// // 1985's +05:30 is not the offset Kathmandu keeps today.
/// let read = resolve(&noon(1985, 1, 1), &kathmandu, &Policy::default(), EmbeddedTzdb::shared())?;
/// assert_eq!(read.zone.warnings, [Warning::OffsetDiffersFromCurrentRules]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Warning {
    /// The offset is not one the zone applies today.
    OffsetDiffersFromCurrentRules,
    /// The civil time occurred twice; the policy chose.
    DstAmbiguous,
    /// The civil time did not exist; the policy shifted it forward.
    DstGapShifted,
    /// A leap second (23:59:60) was folded onto the following midnight.
    LeapSecondFolded,
    /// The instant lies beyond the leap-second table's word.
    LeapTableExpired,
    /// The time was not given; the policy supplied one.
    TimeUnknownFallback,
}

impl Warning {
    /// The key the locale packs and the fixtures use.
    ///
    /// ```
    /// use teistro_time::Warning;
    ///
    /// assert_eq!(Warning::DstGapShifted.key(), "DST_GAP_SHIFTED");
    /// assert_eq!(Warning::DstGapShifted.to_string(), "DST_GAP_SHIFTED");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Warning::OffsetDiffersFromCurrentRules => "OFFSET_DIFFERS_FROM_CURRENT_RULES",
            Warning::DstAmbiguous => "DST_AMBIGUOUS",
            Warning::DstGapShifted => "DST_GAP_SHIFTED",
            Warning::LeapSecondFolded => "LEAP_SECOND_FOLDED",
            Warning::LeapTableExpired => "LEAP_TABLE_EXPIRED",
            Warning::TimeUnknownFallback => "TIME_UNKNOWN_FALLBACK",
        }
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

/// What a stored chart keeps beside its instant: enough to replay the
/// resolution under a newer database and to report the difference.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::Calendar;
/// # let noon = |y, m, d| CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, y, m, d), CivilTime::NOON);
/// use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve};
/// use teistro_time::LocalClock;
///
/// let kathmandu = ZoneSpec::iana("Asia/Kathmandu");
/// let read = resolve(&noon(2024, 1, 1), &kathmandu, &Policy::default(), EmbeddedTzdb::shared())?;
/// assert_eq!(read.zone.tzdb_version, EmbeddedTzdb::bundled_version());
/// assert_eq!(read.zone.abbreviation.as_deref(), Some("+0545"));
/// // Stored, it replays as a clock whatever a newer database says.
/// assert_eq!(read.zone.offset_at(read.instant), read.zone.offset);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ZoneResolution {
    /// The offset applied.
    pub offset: UtcOffset,
    /// Where it came from.
    pub source: ZoneSource,
    /// Which rules produced it.
    pub era: ZoneEra,
    /// The database's version, `lmt` or `manual`.
    pub tzdb_version: String,
    /// The zone's abbreviation for the row, when the database has one.
    pub abbreviation: Option<String>,
    /// What the daylight-saving policy did.
    pub dst: DstOutcome,
    /// Whether the time was given rather than supplied by a policy.
    pub time_known: bool,
    /// What the consumer should know.
    pub warnings: Vec<Warning>,
}

impl ZoneResolution {
    /// Whether a warning was raised.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_core::catalogue::Calendar;
    /// # let noon = |y, m, d| CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, y, m, d), CivilTime::NOON);
    /// use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve};
    /// use teistro_time::Warning;
    ///
    /// let read = resolve(&noon(2024, 1, 1), &ZoneSpec::iana("Asia/Kathmandu"), &Policy::default(), EmbeddedTzdb::shared())?;
    /// assert!(!read.zone.has(Warning::DstAmbiguous));
    /// assert!(read.zone.time_known);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn has(&self, warning: Warning) -> bool {
        self.warnings.contains(&warning)
    }
}

/// A stored resolution is a clock: replaying under it reproduces the
/// instant whatever the database says today.
impl LocalClock for ZoneResolution {
    fn offset_at(&self, _instant: JulianDay<Utc>) -> UtcOffset {
        self.offset
    }

    fn describe(&self) -> String {
        format!(
            "{} (replayed, {:?}, {})",
            self.offset, self.source, self.tzdb_version
        )
    }
}

/// A resolved civil date-time.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::Calendar;
/// # let noon = |y, m, d| CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, y, m, d), CivilTime::NOON);
/// use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve};
///
/// let read = resolve(&noon(2024, 1, 1), &ZoneSpec::iana("Asia/Kathmandu"), &Policy::default(), EmbeddedTzdb::shared())?;
/// // Noon at +05:45 is 06:15 UTC.
/// assert!((read.instant.get() - (2_460_310.5 + 6.25 / 24.0)).abs() < 1e-9);
/// assert_eq!(read.civil, noon(2024, 1, 1));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Resolved {
    /// The instant.
    pub instant: JulianDay<Utc>,
    /// The resolution.
    pub zone: ZoneResolution,
    /// The civil date-time, with the time the policy supplied when it
    /// was not given.
    pub civil: CivilDateTime,
}

/// The seconds of a civil day at a fixed day, read as if the clock were
/// UTC, in the port's calendar-free form.
fn local_seconds(day: FixedDay, time: CivilTime) -> LocalSeconds {
    LocalSeconds {
        seconds: FixedDay::UNIX_EPOCH.days_until(day) * i64::from(SECONDS_PER_DAY)
            + i64::from(time.seconds_of_day()),
        nanos: time.nanos(),
    }
}

/// The instant of local seconds under an offset.
fn instant_of(local: LocalSeconds, offset: UtcOffset) -> Result<JulianDay<Utc>, Error> {
    let unix = local.seconds - i64::from(offset.seconds());
    #[allow(clippy::cast_precision_loss, reason = "seconds far below 2^53")]
    let days = (unix as f64 + f64::from(local.nanos) / f64::from(NANOS_PER_SECOND))
        / f64::from(SECONDS_PER_DAY);
    Ok(JulianDay::try_new(
        JulianDay::<Utc>::UNIX_EPOCH.get() + days,
    )?)
}

/// The shipped calendar a date names, or the error for one that needs a
/// context.
fn calendar_of(calendar: Calendar) -> Result<&'static dyn CalendarSystem, Error> {
    shipped(calendar).ok_or_else(|| {
        Error::unsupported(format!(
            "the {} calendar needs a context with an ephemeris; resolve through `resolve_with`",
            calendar.key()
        ))
        .with_field("date.calendar")
    })
}

/// Resolves a civil date-time in a zone to an instant, through the
/// shipped calendar the date names.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::Calendar;
/// # let noon = |y, m, d| CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, y, m, d), CivilTime::NOON);
/// use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve};
///
/// let tzdb = EmbeddedTzdb::shared();
/// let read = resolve(&noon(2024, 1, 1), &ZoneSpec::iana("Asia/Kathmandu"), &Policy::default(), tzdb)?;
/// assert_eq!(read.zone.offset.seconds(), 5 * 3600 + 45 * 60);
///
/// // An unknown zone is refused with the nearest names.
/// let refused = resolve(&noon(2024, 1, 1), &ZoneSpec::iana("Asia/Kathmandoo"), &Policy::default(), tzdb).unwrap_err();
/// assert!(refused.hint().is_some_and(|hint| hint.contains("Asia/Kathmandu")));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A date the calendar does not have; a missing time under `REFUSE`
/// (`INVALID_ARG`, detail `TIME_UNKNOWN`); a civil time in a gap under
/// `ERROR` (`INVALID_ARG`, detail `DST_GAP`, both offsets named) or in an
/// overlap under `ERROR`; a leap second on a day without one; an unknown
/// zone, with the nearest names as the hint.
pub fn resolve(
    civil: &CivilDateTime,
    zone: &ZoneSpec,
    policy: &Policy,
    provider: &dyn TimeZoneProvider,
) -> Result<Resolved, Error> {
    resolve_with(
        calendar_of(civil.date.calendar)?,
        civil,
        zone,
        policy,
        provider,
    )
}

/// Resolves through a given calendar (one that needs a context, say).
///
/// ```
/// use teistro_calendar::{BikramSambat, CalendarDate};
/// use teistro_core::catalogue::Calendar;
/// use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve, resolve_with};
///
/// // 16 Poush 2080 is 1 January 2024.
/// let bs = CivilDateTime::at(CalendarDate::defined(Calendar::BikramSambat, 2080, 9, 16), CivilTime::NOON);
/// let ad = CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1), CivilTime::NOON);
/// let (zone, policy, tzdb) = (ZoneSpec::iana("Asia/Kathmandu"), Policy::default(), EmbeddedTzdb::shared());
/// let read = resolve_with(BikramSambat::shipped(), &bs, &zone, &policy, tzdb)?;
/// assert_eq!(read.instant, resolve(&ad, &zone, &policy, tzdb)?.instant);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`resolve`].
pub fn resolve_with(
    calendar: &dyn CalendarSystem,
    civil: &CivilDateTime,
    zone: &ZoneSpec,
    policy: &Policy,
    provider: &dyn TimeZoneProvider,
) -> Result<Resolved, Error> {
    let day = calendar.fixed_of(&civil.date)?;
    let mut warnings = Vec::new();
    let (time, time_known) = time_of(civil, *policy, &mut warnings)?;
    let local = local_seconds(day, time);
    let (offset, source, era, abbreviation, dst, version) = match zone {
        ZoneSpec::Fixed { offset } => (
            *offset,
            ZoneSource::Manual,
            ZoneEra::Current,
            None,
            DstOutcome::None,
            String::from("manual"),
        ),
        ZoneSpec::LocalMean { longitude } => (
            LocalMeanTime::new(*longitude).offset(),
            ZoneSource::LocalMean,
            ZoneEra::Current,
            Some(String::from("LMT")),
            DstOutcome::None,
            String::from("lmt"),
        ),
        ZoneSpec::Iana { zone: name } => {
            let candidates = provider.candidates(name, local)?;
            let (offset, dst) = choose_offset(civil, name, candidates, *policy, &mut warnings)?;
            let instant = instant_of(local, offset)?;
            let (era, abbreviation) = era_of(provider, name, offset, instant, &mut warnings)?;
            (
                offset,
                ZoneSource::Iana,
                era,
                Some(abbreviation),
                dst,
                provider.version().to_string(),
            )
        }
    };
    let mut instant = instant_of(local, offset)?;
    if time.is_leap_second() {
        instant = fold_leap_second(civil, instant)?;
        warnings.push(Warning::LeapSecondFolded);
    }
    if leap::is_expired_at(instant) {
        warnings.push(Warning::LeapTableExpired);
    }
    Ok(Resolved {
        instant,
        zone: ZoneResolution {
            offset,
            source,
            era,
            tzdb_version: version,
            abbreviation,
            dst,
            time_known,
            warnings,
        },
        civil: CivilDateTime::at(civil.date.clone(), time),
    })
}

/// What the `SUNRISE` unknown-time fallback needs: a solar model, the
/// place, and the policy for a day without a sunrise.
///
/// ```
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// use teistro_core::settings::{PolarDayPolicy, UnknownTime};
/// use teistro_siddhanta::SuryaSiddhanta;
/// use teistro_time::{CivilDateTime, DayContext, EmbeddedTzdb, Policy, Warning, ZoneSpec, resolve_at_place};
///
/// let text = SuryaSiddhanta::text();
/// let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// let day = DayContext { model: &text, place: kathmandu, polar_day_policy: PolarDayPolicy::Undefined };
/// let sunrise = Policy { unknown_time: UnknownTime::Sunrise, ..Policy::default() };
/// let zone = ZoneSpec::iana("Asia/Kathmandu");
///
/// // A birth with a date and no time, read at that day's sunrise.
/// let birth = CivilDateTime::date_only(CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1));
/// let read = resolve_at_place(&birth, &zone, &sunrise, EmbeddedTzdb::shared(), day)?;
/// assert!(read.zone.has(Warning::TimeUnknownFallback) && !read.zone.time_known);
/// // Winter sunrise at Kathmandu is a little before seven.
/// let time = read.civil.time.unwrap();
/// assert!((6..8).contains(&time.hour()), "{time}");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy)]
pub struct DayContext<'a> {
    /// The solar model the sunrise comes from.
    pub model: &'a dyn SolarModel,
    /// The place.
    pub place: Place,
    /// What a day without a sunrise is.
    pub polar_day_policy: PolarDayPolicy,
}

impl fmt::Debug for DayContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DayContext")
            .field("model", &self.model.describe())
            .field("place", &self.place)
            .field("polar_day_policy", &self.polar_day_policy)
            .finish()
    }
}

/// A zone as a clock, read through a database: what an almanac asked by
/// zone reckons on (`docs/03-design/time-and-timezone.md` §3.4), and what
/// the local day needs to place a civil midnight when the polar policy
/// asks for one. Built by [`ZoneSpec::clock`], which has looked the zone
/// up.
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_time::{EmbeddedTzdb, LocalClock, ZoneSource, ZoneSpec};
///
/// let zone = ZoneSpec::iana("Asia/Kathmandu");
/// let clock = zone.clock(EmbeddedTzdb::shared())?;
/// let read = clock.reading(JulianDay::J2000);
/// assert_eq!((read.offset.seconds(), read.source), (20_700, ZoneSource::Iana));
/// assert_eq!(read.tzdb_version, EmbeddedTzdb::bundled_version());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct ProviderClock<P> {
    provider: P,
    zone: ZoneSpec,
}

impl<P: TimeZoneProvider> LocalClock for ProviderClock<P> {
    fn offset_at(&self, instant: JulianDay<Utc>) -> UtcOffset {
        match &self.zone {
            ZoneSpec::Fixed { offset } => *offset,
            ZoneSpec::LocalMean { longitude } => LocalMeanTime::new(*longitude).offset(),
            // The zone was looked up before this clock was built, so the
            // only refusal left is an instant beyond the database's range,
            // where no offset exists and UTC stands in.
            ZoneSpec::Iana { zone } => self
                .provider
                .offset_at(zone, instant)
                .map_or(UtcOffset::UTC, |info| info.offset),
        }
    }

    fn describe(&self) -> String {
        format!("{} ({})", self.zone, self.provider.version())
    }

    fn reading(&self, instant: JulianDay<Utc>) -> ClockReading {
        match &self.zone {
            ZoneSpec::Fixed { offset } => offset.reading(instant),
            ZoneSpec::LocalMean { longitude } => LocalMeanTime::new(*longitude).reading(instant),
            ZoneSpec::Iana { zone } => {
                let offset = self.offset_at(instant);
                let found = era_of(&self.provider, zone, offset, instant, &mut Vec::new());
                let (era, abbreviation) = found.unwrap_or((ZoneEra::Historical, String::new()));
                ClockReading {
                    offset,
                    source: ZoneSource::Iana,
                    era,
                    tzdb_version: self.provider.version().to_string(),
                    abbreviation: (!abbreviation.is_empty()).then_some(abbreviation),
                }
            }
        }
    }
}

/// Resolves a civil date-time whose time may be absent, supplying the
/// place's sunrise under the `SUNRISE` unknown-time policy: the local day
/// of the date at the place is computed from the model, its sunrise is
/// the instant, and the civil time reported is that instant's in the
/// zone, with `time_known` false and the fallback warning raised. With a
/// time given, or another policy, this is [`resolve`].
///
/// ```
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// use teistro_core::settings::{PolarDayPolicy, UnknownTime};
/// use teistro_siddhanta::SuryaSiddhanta;
/// use teistro_time::{CivilDateTime, DayContext, EmbeddedTzdb, Policy, Warning, ZoneSpec, resolve_at_place};
///
/// let text = SuryaSiddhanta::text();
/// let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// let day = DayContext { model: &text, place: kathmandu, polar_day_policy: PolarDayPolicy::Undefined };
/// let sunrise = Policy { unknown_time: UnknownTime::Sunrise, ..Policy::default() };
/// let zone = ZoneSpec::iana("Asia/Kathmandu");
/// use teistro_time::{CivilTime, resolve};
///
/// // A time given is read as given: this is `resolve`.
/// let noon = CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1), CivilTime::NOON);
/// let tzdb = EmbeddedTzdb::shared();
/// assert_eq!(resolve_at_place(&noon, &zone, &sunrise, tzdb, day)?, resolve(&noon, &zone, &sunrise, tzdb)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`resolve`], plus the model's refusal and a polar day under the
/// `UNDEFINED` policy.
pub fn resolve_at_place(
    civil: &CivilDateTime,
    zone: &ZoneSpec,
    policy: &Policy,
    provider: &dyn TimeZoneProvider,
    day: DayContext<'_>,
) -> Result<Resolved, Error> {
    resolve_at_place_with(
        calendar_of(civil.date.calendar)?,
        civil,
        zone,
        policy,
        provider,
        day,
    )
}

/// [`resolve_at_place`] through a given calendar.
///
/// ```
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// use teistro_core::settings::{PolarDayPolicy, UnknownTime};
/// use teistro_siddhanta::SuryaSiddhanta;
/// use teistro_time::{CivilDateTime, DayContext, EmbeddedTzdb, Policy, Warning, ZoneSpec, resolve_at_place};
///
/// let text = SuryaSiddhanta::text();
/// let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// let day = DayContext { model: &text, place: kathmandu, polar_day_policy: PolarDayPolicy::Undefined };
/// let sunrise = Policy { unknown_time: UnknownTime::Sunrise, ..Policy::default() };
/// let zone = ZoneSpec::iana("Asia/Kathmandu");
/// use teistro_calendar::BikramSambat;
/// use teistro_time::resolve_at_place_with;
///
/// // 16 Poush 2080 is 1 January 2024: the same sunrise either way.
/// let bs = CivilDateTime::date_only(CalendarDate::defined(Calendar::BikramSambat, 2080, 9, 16));
/// let ad = CivilDateTime::date_only(CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1));
/// let tzdb = EmbeddedTzdb::shared();
/// let read = resolve_at_place_with(BikramSambat::shipped(), &bs, &zone, &sunrise, tzdb, day)?;
/// assert_eq!(read.instant, resolve_at_place(&ad, &zone, &sunrise, tzdb, day)?.instant);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`resolve_at_place`].
pub fn resolve_at_place_with(
    calendar: &dyn CalendarSystem,
    civil: &CivilDateTime,
    zone: &ZoneSpec,
    policy: &Policy,
    provider: &dyn TimeZoneProvider,
    day: DayContext<'_>,
) -> Result<Resolved, Error> {
    if civil.time.is_some() || policy.unknown_time != UnknownTime::Sunrise {
        return resolve_with(calendar, civil, zone, policy, provider);
    }
    // An unknown zone is refused here, with the provider's suggestions.
    let clock = zone.clock(provider)?;
    let local = local_day(
        day.model,
        calendar,
        &clock,
        &day.place,
        &civil.date,
        day.polar_day_policy,
    )?;
    let (at_sunrise, mut resolution) = civil_of_with(calendar, local.sunrise, zone, provider)?;
    resolution.time_known = false;
    resolution.warnings.push(Warning::TimeUnknownFallback);
    Ok(Resolved {
        instant: local.sunrise,
        zone: resolution,
        civil: at_sunrise,
    })
}

/// The time of day a resolution uses: the one given, or the one the
/// unknown-time policy supplies.
fn time_of(
    civil: &CivilDateTime,
    policy: Policy,
    warnings: &mut Vec<Warning>,
) -> Result<(CivilTime, bool), Error> {
    if let Some(time) = civil.time {
        return Ok((time, true));
    }
    warnings.push(Warning::TimeUnknownFallback);
    match policy.unknown_time {
        UnknownTime::Noon => Ok((CivilTime::NOON, false)),
        UnknownTime::Midnight => Ok((CivilTime::MIDNIGHT, false)),
        UnknownTime::Refuse => Err(Error::invalid_arg(format!("{civil} has no time of day"))
            .with_detail(Detail::TimeUnknown)
            .with_field("time")
            .with_hint(
                "give a time, or choose NOON, MIDNIGHT or SUNRISE as the unknown-time policy",
            )),
        UnknownTime::Sunrise => Err(Error::invalid_arg(format!(
            "{civil} has no time of day and the SUNRISE fallback needs the place and a solar model"
        ))
        .with_detail(Detail::TimeUnknown)
        .with_field("time")
        .with_hint("resolve through `resolve_at_place` with the place and the model")),
        other => Err(crate::ghati::unknown_knob(
            "time.unknown_time",
            &format!("{other:?}"),
        )),
    }
}

/// The offset a civil time takes among its candidates, under the
/// daylight-saving policies.
fn choose_offset(
    civil: &CivilDateTime,
    name: &str,
    candidates: LocalCandidates,
    policy: Policy,
    warnings: &mut Vec<Warning>,
) -> Result<(UtcOffset, DstOutcome), Error> {
    match candidates {
        LocalCandidates::Unambiguous { offset } => Ok((offset, DstOutcome::None)),
        LocalCandidates::Gap { before, after } => match policy.gap {
            DstGap::Error => Err(Error::invalid_arg(format!(
                "{civil} does not exist in {name}: the clocks jumped from {before} to {after}"
            ))
            .with_detail(Detail::DstGap)
            .with_field("time")
            .with_hint("choose SHIFT_FORWARD as the gap policy to move the time past the jump")),
            DstGap::ShiftForward => {
                warnings.push(Warning::DstGapShifted);
                Ok((
                    before,
                    DstOutcome::Gap {
                        shifted_by_seconds: after.seconds() - before.seconds(),
                    },
                ))
            }
            other => Err(crate::ghati::unknown_knob(
                "time.dst_gap",
                &format!("{other:?}"),
            )),
        },
        LocalCandidates::Overlap { earlier, later } => {
            warnings.push(Warning::DstAmbiguous);
            match policy.overlap {
                DstOverlap::Earlier => Ok((
                    earlier,
                    DstOutcome::Overlap {
                        chosen: Chosen::Earlier,
                    },
                )),
                DstOverlap::Later => Ok((
                    later,
                    DstOutcome::Overlap {
                        chosen: Chosen::Later,
                    },
                )),
                DstOverlap::Error => Err(Error::invalid_arg(format!(
                    "{civil} occurred twice in {name}: at {earlier} and again at {later}"
                ))
                .with_field("time")
                .with_hint("choose EARLIER or LATER as the overlap policy")),
                other => Err(crate::ghati::unknown_knob(
                    "time.dst_overlap",
                    &format!("{other:?}"),
                )),
            }
        }
    }
}

/// Which rules produced an offset, and the zone's abbreviation for it.
fn era_of(
    provider: &dyn TimeZoneProvider,
    name: &str,
    offset: UtcOffset,
    instant: JulianDay<Utc>,
    warnings: &mut Vec<Warning>,
) -> Result<(ZoneEra, String), Error> {
    let info = provider.offset_at(name, instant)?;
    let current = provider.current_offsets(name)?;
    let era = if info.before_rules {
        ZoneEra::BeforeRules
    } else if current.contains(&offset) {
        ZoneEra::Current
    } else {
        ZoneEra::Historical
    };
    if !current.contains(&offset) {
        warnings.push(Warning::OffsetDiffersFromCurrentRules);
    }
    Ok((era, info.abbreviation))
}

/// A civil 23:59:60 is a leap second only if it lands on 23:59:60 UTC of
/// a day the table ends with one; it then folds onto the following
/// midnight.
fn fold_leap_second(
    civil: &CivilDateTime,
    instant: JulianDay<Utc>,
) -> Result<JulianDay<Utc>, Error> {
    let (utc_day, fraction) = FixedDay::from_jd(instant);
    let lands_on_midnight = fraction < 1e-9 || fraction > 1.0 - 1e-9;
    let leap_day = if fraction < 0.5 {
        utc_day.plus_days(-1)
    } else {
        utc_day
    };
    if !lands_on_midnight || !leap::is_leap_second_day(leap_day) {
        return Err(Error::invalid_arg(format!(
            "{civil} names a leap second, but no leap second ended that UTC day"
        ))
        .with_detail(Detail::NonexistentDate)
        .with_field("time")
        .with_hint(
            "a leap second is 23:59:60 UTC at the end of a day the table ends with one; give 23:59:59 or the next day's first second",
        ));
    }
    Ok(leap_day.plus_days(1).jd_at_midnight()?)
}

/// The civil date-time of an instant in a zone, in the Gregorian
/// calendar, with the resolution that applied.
///
/// ```
/// use teistro_core::quantity::{JulianDay, Utc};
/// use teistro_time::{CivilTime, EmbeddedTzdb, ZoneSpec, civil_of};
///
/// // 06:15 UTC on 1 January 2024.
/// let instant = JulianDay::<Utc>::try_new(2_460_310.5 + 6.25 / 24.0)?;
/// let (civil, zone) = civil_of(instant, &ZoneSpec::iana("Asia/Kathmandu"), EmbeddedTzdb::shared())?;
/// assert_eq!((civil.date.year, civil.date.month, civil.date.day), (2024, 1, 1));
/// assert_eq!(civil.time, Some(CivilTime::NOON));
/// assert_eq!(zone.offset.seconds(), 5 * 3600 + 45 * 60);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// An unknown zone, or an instant outside the calendar.
pub fn civil_of(
    instant: JulianDay<Utc>,
    zone: &ZoneSpec,
    provider: &dyn TimeZoneProvider,
) -> Result<(CivilDateTime, ZoneResolution), Error> {
    civil_of_with(calendar_of(Calendar::Gregorian)?, instant, zone, provider)
}

/// The civil date-time of an instant in a zone, in a given calendar.
///
/// ```
/// use teistro_core::quantity::{JulianDay, Utc};
/// use teistro_time::{CivilTime, EmbeddedTzdb, ZoneSpec, civil_of_with};
/// use teistro_calendar::BikramSambat;
///
/// // 06:15 UTC on 1 January 2024.
/// let instant = JulianDay::<Utc>::try_new(2_460_310.5 + 6.25 / 24.0)?;
/// let (civil, _) = civil_of_with(BikramSambat::shipped(), instant, &ZoneSpec::iana("Asia/Kathmandu"), EmbeddedTzdb::shared())?;
/// assert_eq!((civil.date.year, civil.date.month, civil.date.day), (2080, 9, 16));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`civil_of`].
pub fn civil_of_with(
    calendar: &dyn CalendarSystem,
    instant: JulianDay<Utc>,
    zone: &ZoneSpec,
    provider: &dyn TimeZoneProvider,
) -> Result<(CivilDateTime, ZoneResolution), Error> {
    let mut warnings = Vec::new();
    let (offset, source, era, abbreviation, version) = match zone {
        ZoneSpec::Fixed { offset } => (
            *offset,
            ZoneSource::Manual,
            ZoneEra::Current,
            None,
            String::from("manual"),
        ),
        ZoneSpec::LocalMean { longitude } => (
            LocalMeanTime::new(*longitude).offset(),
            ZoneSource::LocalMean,
            ZoneEra::Current,
            Some(String::from("LMT")),
            String::from("lmt"),
        ),
        ZoneSpec::Iana { zone: name } => {
            let offset = provider.offset_at(name, instant)?.offset;
            let (era, abbreviation) = era_of(provider, name, offset, instant, &mut warnings)?;
            (
                offset,
                ZoneSource::Iana,
                era,
                Some(abbreviation),
                provider.version().to_string(),
            )
        }
    };
    if leap::is_expired_at(instant) {
        warnings.push(Warning::LeapTableExpired);
    }
    let local = instant.get() + offset.days();
    let (mut day, fraction) = FixedDay::from_local_jd(local);
    // The day's fraction to the nearest microsecond, which is what an
    // `f64` Julian day resolves in the present era.
    let micros = (fraction * f64::from(SECONDS_PER_DAY) * 1e6).round();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a rounded fraction of a day in microseconds"
    )]
    let mut micros = micros as u64;
    if micros >= u64::from(SECONDS_PER_DAY) * 1_000_000 {
        micros = 0;
        day = day.plus_days(1);
    }
    let seconds = u32::try_from(micros / 1_000_000).unwrap_or(0);
    let nanos = u32::try_from(micros % 1_000_000).unwrap_or(0) * 1000;
    let time = CivilTime::from_seconds_of_day(seconds, nanos)?;
    let date = calendar.date_of(day)?;
    Ok((
        CivilDateTime::at(date, time),
        ZoneResolution {
            offset,
            source,
            era,
            tzdb_version: version,
            abbreviation,
            dst: DstOutcome::None,
            time_known: true,
            warnings,
        },
    ))
}

/// The date of an instant under a clock, in a calendar: the civil day
/// the instant falls in.
///
/// ```
/// use teistro_calendar::Gregorian;
/// use teistro_core::quantity::{JulianDay, Utc};
/// use teistro_core::time::UtcOffset;
/// use teistro_time::zone::date_of;
///
/// // 20:00 UTC on 31 December 2023 is already 1 January at Kathmandu.
/// let instant = JulianDay::<Utc>::try_new(2_460_310.5 - 4.0 / 24.0)?;
/// let ymd = |offset| date_of(instant, &offset, &Gregorian).map(|date| (date.year, date.month, date.day));
/// assert_eq!(ymd(UtcOffset::literal(5, 45, 0))?, (2024, 1, 1));
/// assert_eq!(ymd(UtcOffset::UTC)?, (2023, 12, 31));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// An instant outside the calendar.
pub fn date_of(
    instant: JulianDay<Utc>,
    clock: &dyn LocalClock,
    calendar: &dyn CalendarSystem,
) -> Result<CalendarDate, Error> {
    let (day, _) = FixedDay::from_local_jd(clock.local_jd(instant));
    calendar.date_of(day)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::ZoneSource;

    /// A source's key is its serialised spelling, so a program matching on
    /// either reads the same word.
    #[test]
    fn a_source_is_keyed_as_it_serialises() {
        for source in [ZoneSource::Iana, ZoneSource::LocalMean, ZoneSource::Manual] {
            assert_eq!(
                serde_json::to_string(&source).unwrap(),
                format!("\"{}\"", source.key())
            );
        }
    }
}

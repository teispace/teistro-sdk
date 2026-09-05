//! Zone resolution: a civil date-time in a zone to an instant under the
//! daylight-saving policies, with the metadata a stored chart keeps to
//! replay the resolution under a newer database, and the way back from
//! an instant to civil time (`docs/03-design/time-and-timezone.md`, §3.2
//! and §4).

pub mod embedded;

use core::fmt;

use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay, shipped};
use teistro_core::catalogue::Calendar;
use teistro_core::error::{Detail, Error};
use teistro_core::quantity::{JulianDay, Longitude, Utc};
use teistro_core::settings::{DstGap, DstOverlap, Settings, UnknownTime};
use teistro_core::time::{LocalClock, LocalMeanTime, UtcOffset};
use teistro_port_timezone::{LocalCandidates, LocalSeconds, TimeZoneProvider};

use crate::civil::{CivilDateTime, CivilTime, NANOS_PER_SECOND, SECONDS_PER_DAY};
use crate::leap;

/// How a zone is given.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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
    pub fn iana(zone: impl Into<String>) -> ZoneSpec {
        ZoneSpec::Iana { zone: zone.into() }
    }

    /// Local mean time at a longitude.
    #[must_use]
    pub const fn local_mean(longitude: Longitude) -> ZoneSpec {
        ZoneSpec::LocalMean { longitude }
    }

    /// A fixed offset.
    #[must_use]
    pub const fn fixed(offset: UtcOffset) -> ZoneSpec {
        ZoneSpec::Fixed { offset }
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

/// Where the offset came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZoneSource {
    /// The zone database.
    Iana,
    /// Local mean time from the longitude.
    LocalMean,
    /// A fixed offset the consumer stated.
    Manual,
}

/// Which rules produced the offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZoneEra {
    /// An offset the zone applies in the database's own year.
    Current,
    /// An offset from the zone's earlier rules.
    Historical,
    /// Before the zone's first rule: the database's local-mean-time stub.
    BeforeRules,
}

/// Which occurrence an overlap resolved to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Chosen {
    /// The first occurrence, the earlier instant.
    Earlier,
    /// The second occurrence, the later instant.
    Later,
}

/// What the daylight-saving policy did.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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
            "{civil} has no time of day and the SUNRISE fallback needs the local day"
        ))
        .with_detail(Detail::TimeUnknown)
        .with_field("time")
        .with_hint(
            "compute the day with `local_day` and resolve its sunrise instant with `civil_of`",
        )),
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
        .with_field("time"));
    }
    Ok(leap_day.plus_days(1).jd_at_midnight()?)
}

/// The civil date-time of an instant in a zone, in the Gregorian
/// calendar, with the resolution that applied.
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

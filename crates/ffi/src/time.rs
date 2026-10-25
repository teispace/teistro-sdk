//! Civil times to instants and back with the zone metadata a stored
//! chart keeps, and the conversions between the time scales
//! (`docs/03-design/time-and-timezone.md`).

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;
use core::ptr;

use teistro_astro::delta_t::{DeltaT, DeltaTSource, delta_t};
use teistro_core::Status;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Longitude, Tt, Ut1, Utc};
use teistro_core::time::UtcOffset;
use teistro_time::scale::{
    tt_from_utc, tt_of, ut1_from_tt, ut1_from_utc, utc_from_tt, utc_from_ut1,
};
use teistro_time::{
    Chosen, CivilDateTime, CivilTime, DstOutcome, EmbeddedTzdb, Policy, Warning, ZoneEra,
    ZoneResolution, ZoneSource, ZoneSpec, civil_of_with, resolve,
};

use crate::calendar::{TsCalendarDate, system_of};
use crate::context::TsContext;
use crate::support::{c_struct, optional_text, read_in, with_context, write_out};

/// A time scale of the conversions; the first two ids are the port's.
///
/// ```
/// use teistro_ffi::time::TsScale;
///
/// // A member crosses the boundary as its number: `Ut1` is 0, `Utc` 2.
/// assert_eq!((TsScale::Ut1 as u32, TsScale::Utc as u32), (0, 2));
/// ```
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsScale {
    /// Universal Time (UT1).
    Ut1 = 0,
    /// Terrestrial Time.
    Tt = 1,
    /// Coordinated Universal Time, through the leap-second table from
    /// 1972 and read as UT1 before it.
    Utc = 2,
}

impl TsScale {
    fn from_id(id: u32, field: &str) -> Result<TsScale, Error> {
        match id {
            0 => Ok(TsScale::Ut1),
            1 => Ok(TsScale::Tt),
            2 => Ok(TsScale::Utc),
            _ => Err(Error::invalid_arg(format!(
                "`{field}` is {id}; the scales are UT1=0, TT=1, UTC=2"
            ))
            .with_field(field)),
        }
    }
}

/// What a zone specification names.
///
/// ```
/// use teistro_ffi::time::TsZoneKind;
///
/// // A member crosses the boundary as its number: `Iana` is 0, `LocalMean` 2.
/// assert_eq!((TsZoneKind::Iana as u8, TsZoneKind::LocalMean as u8), (0, 2));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsZoneKind {
    /// A zone of the embedded database, by IANA name.
    Iana = 0,
    /// A fixed offset from UTC.
    Fixed = 1,
    /// Local mean time at a longitude.
    LocalMean = 2,
}

/// Where a resolution's offset came from.
///
/// ```
/// use teistro_ffi::time::TsZoneSource;
///
/// // A member crosses the boundary as its number: `Iana` is 0, `Manual` 2.
/// assert_eq!((TsZoneSource::Iana as u8, TsZoneSource::Manual as u8), (0, 2));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsZoneSource {
    /// The zone database.
    Iana = 0,
    /// Local mean time from the longitude.
    LocalMean = 1,
    /// A fixed offset the consumer stated.
    Manual = 2,
}

impl From<ZoneSource> for TsZoneSource {
    fn from(source: ZoneSource) -> TsZoneSource {
        match source {
            ZoneSource::Iana => TsZoneSource::Iana,
            ZoneSource::LocalMean => TsZoneSource::LocalMean,
            ZoneSource::Manual => TsZoneSource::Manual,
        }
    }
}

/// Which rules produced the offset.
///
/// ```
/// use teistro_ffi::time::TsZoneEra;
///
/// // A member crosses the boundary as its number: `Current` is 0, `BeforeRules` 2.
/// assert_eq!((TsZoneEra::Current as u8, TsZoneEra::BeforeRules as u8), (0, 2));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsZoneEra {
    /// An offset the zone applies in the database's own year.
    Current = 0,
    /// An offset from the zone's earlier rules.
    Historical = 1,
    /// Before the zone's first rule: the database's local-mean-time stub.
    BeforeRules = 2,
}

impl From<ZoneEra> for TsZoneEra {
    fn from(era: ZoneEra) -> TsZoneEra {
        match era {
            ZoneEra::Current => TsZoneEra::Current,
            ZoneEra::Historical => TsZoneEra::Historical,
            ZoneEra::BeforeRules => TsZoneEra::BeforeRules,
        }
    }
}

/// What the daylight-saving policy did.
///
/// ```
/// use teistro_ffi::time::TsDst;
///
/// // A member crosses the boundary as its number: `None` is 0, `Overlap` 2.
/// assert_eq!((TsDst::None as u8, TsDst::Overlap as u8), (0, 2));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsDst {
    /// The civil time was unambiguous.
    None = 0,
    /// The civil time fell in a gap and was shifted forward past it.
    Gap = 1,
    /// The civil time fell in an overlap and one occurrence was chosen.
    Overlap = 2,
}

impl From<DstOutcome> for TsDst {
    fn from(outcome: DstOutcome) -> TsDst {
        match outcome {
            DstOutcome::None => TsDst::None,
            DstOutcome::Gap { .. } => TsDst::Gap,
            DstOutcome::Overlap { .. } => TsDst::Overlap,
        }
    }
}

/// Which occurrence an overlap resolved to.
///
/// ```
/// use teistro_ffi::time::TsChosen;
///
/// // A member crosses the boundary as its number: `Earlier` is 0, `Later` 1.
/// assert_eq!((TsChosen::Earlier as u8, TsChosen::Later as u8), (0, 1));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsChosen {
    /// The first occurrence, the earlier instant.
    Earlier = 0,
    /// The second occurrence, the later instant.
    Later = 1,
}

impl From<Chosen> for TsChosen {
    fn from(chosen: Chosen) -> TsChosen {
        match chosen {
            Chosen::Earlier => TsChosen::Earlier,
            Chosen::Later => TsChosen::Later,
        }
    }
}

/// A warning of a resolution; `warnings` in the resolution is a bit set,
/// bit `n` for the warning with value `n`.
///
/// ```
/// use teistro_ffi::time::TsZoneWarning;
///
/// // A member crosses the boundary as its number: `OffsetDiffersFromCurrentRules` is 0, `TimeUnknownFallback` 5.
/// assert_eq!((TsZoneWarning::OffsetDiffersFromCurrentRules as u8, TsZoneWarning::TimeUnknownFallback as u8), (0, 5));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsZoneWarning {
    /// The offset is not one the zone applies today.
    OffsetDiffersFromCurrentRules = 0,
    /// The civil time occurred twice; the policy chose.
    DstAmbiguous = 1,
    /// The civil time did not exist; the policy shifted it forward.
    DstGapShifted = 2,
    /// A leap second (23:59:60) was folded onto the following midnight.
    LeapSecondFolded = 3,
    /// The instant lies beyond the leap-second table's word.
    LeapTableExpired = 4,
    /// The time was not given; the policy supplied one.
    TimeUnknownFallback = 5,
}

impl From<Warning> for TsZoneWarning {
    fn from(warning: Warning) -> TsZoneWarning {
        match warning {
            Warning::OffsetDiffersFromCurrentRules => TsZoneWarning::OffsetDiffersFromCurrentRules,
            Warning::DstAmbiguous => TsZoneWarning::DstAmbiguous,
            Warning::DstGapShifted => TsZoneWarning::DstGapShifted,
            Warning::LeapSecondFolded => TsZoneWarning::LeapSecondFolded,
            Warning::LeapTableExpired => TsZoneWarning::LeapTableExpired,
            Warning::TimeUnknownFallback => TsZoneWarning::TimeUnknownFallback,
        }
    }
}

/// What produced a Delta T value.
///
/// ```
/// use teistro_ffi::time::TsDeltaTSource;
///
/// // A member crosses the boundary as its number: `Table` is 0, `Custom` 3.
/// assert_eq!((TsDeltaTSource::Table as u8, TsDeltaTSource::Custom as u8), (0, 3));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsDeltaTSource {
    /// Interpolated in the IERS table.
    Table = 0,
    /// From a model, outside the table.
    Model = 1,
    /// Through the leap-second table: TT less UTC, exact.
    LeapSeconds = 2,
    /// Supplied by the consumer.
    Custom = 3,
}

impl From<DeltaTSource> for TsDeltaTSource {
    fn from(source: DeltaTSource) -> TsDeltaTSource {
        match source {
            DeltaTSource::Table => TsDeltaTSource::Table,
            DeltaTSource::Model => TsDeltaTSource::Model,
            DeltaTSource::LeapSeconds => TsDeltaTSource::LeapSeconds,
            DeltaTSource::Custom => TsDeltaTSource::Custom,
        }
    }
}

/// A time of day, or none when the birth time is unknown.
///
/// ```
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_ffi::time::TsCivilDateTime;
/// use teistro_time::{CivilDateTime, CivilTime};
///
/// // A date alone crosses with no time, which the context's policy then supplies.
/// let date_only = TsCivilDateTime::of(&CivilDateTime::date_only(CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6)));
/// assert_eq!(date_only.time.has_time, 0);
/// let dawn = TsCivilDateTime::of(&CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6), CivilTime::new(6, 15, 0)?));
/// assert_eq!((dawn.time.has_time, dawn.time.hour, dawn.time.minute), (1, 6, 15));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TsCivilTime {
    /// `sizeof(ts_civil_time)` as the caller compiled it.
    pub struct_size: u32,
    /// The hour.
    /// `api: range=[0,23] example=6`
    pub hour: u8,
    /// The minute.
    /// `api: range=[0,59] example=15`
    pub minute: u8,
    /// The second; `60` only for a leap second the table has.
    /// `api: range=[0,60] example=0`
    pub second: u8,
    /// Whether the time is given; a date-only civil time has none, and the
    /// context's unknown-time policy decides what to do.
    /// `api: flag example=1`
    pub has_time: u8,
    /// Nanoseconds within the second.
    /// `api: range=[0,999999999] example=0`
    pub nanos: u32,
}

/// A civil date and time in a calendar.
///
/// ```
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_ffi::time::TsCivilDateTime;
/// use teistro_time::{CivilDateTime, CivilTime};
///
/// let civil = CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6), CivilTime::new(6, 15, 0)?);
/// let crossed = TsCivilDateTime::of(&civil);
/// assert_eq!((crossed.date.year, crossed.date.month, crossed.date.day), (2026, 9, 6));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TsCivilDateTime {
    /// `sizeof(ts_civil_date_time)` as the caller compiled it.
    pub struct_size: u32,
    /// Reserved, zero.
    pub reserved: u32,
    /// The date.
    pub date: TsCalendarDate,
    /// The time of day.
    pub time: TsCivilTime,
}

/// A zone: an IANA name, a fixed offset, or local mean time at a longitude.
///
/// ```
/// use teistro_ffi::time::{TsZoneKind, TsZoneSpec};
///
/// // Nepal's clock as a fixed offset: kind and offset, the rest zero.
/// let nepal = TsZoneSpec { kind: TsZoneKind::Fixed as u8, offset_seconds: 5 * 3600 + 45 * 60, ..TsZoneSpec::default() };
/// assert!(nepal.zone.is_null());
/// assert_eq!(nepal.offset_seconds, 20_700);
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsZoneSpec {
    /// `sizeof(ts_zone_spec)` as the caller compiled it.
    pub struct_size: u32,
    /// Which field applies.
    /// `api: enum=TsZoneKind example=0`
    pub kind: u8,
    /// Reserved, zero.
    pub reserved: [u8; 3],
    /// The offset east of UTC in seconds, for a fixed offset.
    /// `api: unit=s range=[-64800,64800] example=20700`
    pub offset_seconds: i32,
    /// Reserved, zero.
    pub reserved2: u32,
    /// The longitude east-positive in degrees, for local mean time.
    /// `api: unit=deg range=[-180,180] brand=longitude example=85.3240`
    pub longitude_deg: f64,
    /// The IANA name, for a database zone.
    /// `api: nullable example=Asia/Kathmandu`
    pub zone: *const c_char,
}

/// What a resolution applied, enough to reproduce it after the zone
/// database changes.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// # use teistro_ffi::time::TsZoneResolution;
/// #
/// # let mut resolution = TsZoneResolution {
/// #     struct_size: u32::try_from(size_of::<TsZoneResolution>())?,
/// #     offset_seconds: 0, dst_shift_seconds: 0, warnings: 0, instant_jd_utc: 0.0,
/// #     tzdb_version: core::ptr::null(), abbreviation: core::ptr::null(),
/// #     source: 0, era: 0, dst: 0, chosen: 0, time_known: 0, reserved: [0; 3],
/// # };
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_ffi::time::TsCivilDateTime;
/// use teistro_time::{CivilDateTime, CivilTime};
///
/// use teistro_ffi::time::{TsZoneSpec, ts_time_resolve};
///
/// // 00:20 on 1 January 1986 in Kathmandu: the first night of +05:45, which
/// // the database applies from that midnight.
/// let name = c"Asia/Kathmandu";
/// let zone = TsZoneSpec { struct_size: u32::try_from(size_of::<TsZoneSpec>())?, zone: name.as_ptr(), ..TsZoneSpec::default() };
/// let mut birth = TsCivilDateTime::of(&CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, 1986, 1, 1), CivilTime::new(0, 20, 0)?));
/// // `of` leaves every size for the caller, who knows what it compiled.
/// birth.struct_size = u32::try_from(size_of::<TsCivilDateTime>())?;
/// birth.date.struct_size = u32::try_from(size_of::<teistro_ffi::calendar::TsCalendarDate>())?;
/// birth.time.struct_size = u32::try_from(size_of::<teistro_ffi::time::TsCivilTime>())?;
/// // SAFETY: a live context, and records sized for this build.
/// let status = unsafe { ts_time_resolve(&raw const ctx, &raw const birth, &raw const zone, &raw mut resolution) };
/// assert_eq!(status, teistro_core::Status::Ok);
/// assert_eq!(resolution.offset_seconds, 20_700);
/// assert!((resolution.instant_jd_utc - 2_446_431.274_305_6).abs() < 1e-6);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsZoneResolution {
    /// `sizeof(ts_zone_resolution)` as the caller compiled it.
    pub struct_size: u32,
    /// The offset east of UTC applied, seconds.
    /// `api: unit=s example=20700`
    pub offset_seconds: i32,
    /// How far a civil time inside a gap was shifted forward, seconds.
    /// `api: unit=s example=0`
    pub dst_shift_seconds: i32,
    /// What the resolution wants the consumer to know.
    /// `api: bitset=TsZoneWarning example=0`
    pub warnings: u32,
    /// The instant on UTC.
    /// `api: unit=jd example=2446431.2743`
    pub instant_jd_utc: f64,
    /// The zone database version (`2026c`), `manual` or `lmt`; lent until
    /// the next call on the context.
    pub tzdb_version: *const c_char,
    /// The zone's abbreviation at the instant (`NPT`), or null; lent until
    /// the next call on the context.
    /// `api: nullable`
    pub abbreviation: *const c_char,
    /// Where the offset came from.
    /// `api: enum=TsZoneSource example=0`
    pub source: u8,
    /// Which rules produced it.
    /// `api: enum=TsZoneEra example=0`
    pub era: u8,
    /// What the daylight-saving policy did.
    /// `api: enum=TsDst example=0`
    pub dst: u8,
    /// Which occurrence an overlap resolved to; meaningful when `dst` is
    /// an overlap.
    /// `api: enum=TsChosen example=0`
    pub chosen: u8,
    /// Whether the time was given rather than supplied by a policy.
    /// `api: flag`
    pub time_known: u8,
    /// Reserved, zero.
    pub reserved: [u8; 3],
}

/// A conversion between time scales with what it applied.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_ffi::time::{TsDeltaTSource, TsScale, TsTimeConversion, ts_time_convert};
///
/// let mut conversion = TsTimeConversion {
///     struct_size: u32::try_from(size_of::<TsTimeConversion>())?,
///     from: 0, to: 0, delta_t_source: 0, proleptic_utc: 0, has_uncertainty: 0, reserved: 0,
///     jd: 0.0, delta_t_seconds: 0.0, uncertainty_seconds: 0.0, dut1_seconds: 0.0,
///     delta_t_model: core::ptr::null(),
/// };
/// // 1 January 2000, 00:00 UTC, on TT: thirty-two leap seconds and 32.184 s.
/// // SAFETY: a live context, and a record sized for this build.
/// let status = unsafe { ts_time_convert(&raw const ctx, 2_451_544.5, TsScale::Utc as u32, TsScale::Tt as u32, &raw mut conversion) };
/// assert_eq!(status, teistro_core::Status::Ok);
/// assert!((conversion.delta_t_seconds - 64.184).abs() < 1e-9);
/// assert_eq!(conversion.delta_t_source, TsDeltaTSource::LeapSeconds as u8);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsTimeConversion {
    /// `sizeof(ts_time_conversion)` as the caller compiled it.
    pub struct_size: u32,
    /// The scale converted from.
    /// `api: enum=TsScale example=2`
    pub from: u32,
    /// The scale converted to.
    /// `api: enum=TsScale example=1`
    pub to: u32,
    /// What produced the Delta T applied.
    /// `api: enum=TsDeltaTSource example=0`
    pub delta_t_source: u8,
    /// Whether UTC before 1972 was read as UT1.
    /// `api: flag`
    pub proleptic_utc: u8,
    /// Whether `uncertainty_seconds` is meaningful.
    /// `api: flag`
    pub has_uncertainty: u8,
    /// Reserved, zero.
    pub reserved: u8,
    /// The instant on the target scale.
    /// `api: unit=jd example=2451545.0007`
    pub jd: f64,
    /// The Delta T applied, seconds; zero for a conversion that needs none.
    /// `api: unit=s example=63.83`
    pub delta_t_seconds: f64,
    /// Delta T's one-sigma uncertainty, seconds, when `has_uncertainty`.
    /// `api: unit=s example=0.0`
    pub uncertainty_seconds: f64,
    /// UT1 less UTC applied, seconds: zero until a provider supplies the
    /// IERS bulletins.
    /// `api: unit=s example=0.0`
    pub dut1_seconds: f64,
    /// The Delta T model's key (`TABLE_THEN_MODEL`); lent until the next
    /// call on the context.
    pub delta_t_model: *const c_char,
}

/// Delta T at an instant with what produced it.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::None)?;
/// use teistro_ffi::time::{TsDeltaT, TsDeltaTSource, ts_time_delta_t};
///
/// let mut delta = TsDeltaT {
///     struct_size: u32::try_from(size_of::<TsDeltaT>())?,
///     source: 0, has_uncertainty: 0, reserved: [0; 2],
///     seconds: 0.0, uncertainty_seconds: 0.0, model: core::ptr::null(),
/// };
/// // SAFETY: a live context, and a record sized for this build.
/// let status = unsafe { ts_time_delta_t(&raw const ctx, 2_451_544.5, &raw mut delta) };
/// assert_eq!(status, teistro_core::Status::Ok);
/// assert!((delta.seconds - 63.83).abs() < 0.02);
/// assert_eq!(delta.source, TsDeltaTSource::Table as u8);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsDeltaT {
    /// `sizeof(ts_delta_t)` as the caller compiled it.
    pub struct_size: u32,
    /// What produced it.
    /// `api: enum=TsDeltaTSource example=0`
    pub source: u8,
    /// Whether `uncertainty_seconds` is meaningful.
    /// `api: flag`
    pub has_uncertainty: u8,
    /// Reserved, zero.
    pub reserved: [u8; 2],
    /// TT less UT1, seconds.
    /// `api: unit=s example=63.83`
    pub seconds: f64,
    /// The one-sigma uncertainty, seconds, when `has_uncertainty`.
    /// `api: unit=s example=0.0`
    pub uncertainty_seconds: f64,
    /// The model's key; lent until the next call on the context.
    pub model: *const c_char,
}

c_struct!(
    TsCivilTime,
    TsCivilDateTime,
    TsZoneSpec,
    TsZoneResolution,
    TsTimeConversion,
    TsDeltaT
);

impl TsCivilDateTime {
    /// The boundary form of a civil date-time.
    ///
    /// ```
    /// use teistro_calendar::CalendarDate;
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_ffi::time::TsCivilDateTime;
    /// use teistro_time::{CivilDateTime, CivilTime};
    ///
    /// let civil = CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6), CivilTime::new(23, 59, 60)?);
    /// assert_eq!(TsCivilDateTime::of(&civil).time.second, 60);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn of(civil: &CivilDateTime) -> TsCivilDateTime {
        TsCivilDateTime {
            struct_size: 0,
            reserved: 0,
            date: TsCalendarDate::of(&civil.date),
            time: civil.time.map_or(
                TsCivilTime {
                    struct_size: 0,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    has_time: 0,
                    nanos: 0,
                },
                |t| TsCivilTime {
                    struct_size: 0,
                    hour: t.hour(),
                    minute: t.minute(),
                    second: t.second(),
                    has_time: 1,
                    nanos: t.nanos(),
                },
            ),
        }
    }

    /// The civil date-time this struct names.
    ///
    /// ```
    /// use teistro_calendar::CalendarDate;
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_ffi::time::TsCivilDateTime;
    /// use teistro_time::{CivilDateTime, CivilTime};
    ///
    /// let civil = CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6), CivilTime::new(6, 15, 0)?);
    /// assert_eq!(TsCivilDateTime::of(&civil).to_civil()?, civil);
    /// // An hour out of range is refused.
    /// let mut bad = TsCivilDateTime::of(&civil);
    /// bad.time.hour = 24;
    /// assert!(bad.to_civil().is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A date or time field outside its range.
    pub fn to_civil(&self) -> Result<CivilDateTime, Error> {
        let date = self.date.to_date()?;
        if self.time.has_time == 0 {
            return Ok(CivilDateTime::date_only(date));
        }
        let time = CivilTime::new(self.time.hour, self.time.minute, self.time.second)?
            .with_nanos(self.time.nanos)?;
        Ok(CivilDateTime::at(date, time))
    }
}

impl TsZoneSpec {
    /// The zone this struct names.
    ///
    /// ```
    /// use teistro_ffi::time::{TsZoneKind, TsZoneSpec};
    ///
    /// let name = c"Asia/Kathmandu";
    /// let iana = TsZoneSpec { kind: TsZoneKind::Iana as u8, zone: name.as_ptr(), ..TsZoneSpec::default() };
    /// // SAFETY: `zone` is a NUL-terminated string.
    /// assert!(unsafe { iana.to_spec() }.is_ok());
    /// // An IANA zone without its name is refused, naming the field.
    /// let nameless = TsZoneSpec { zone: core::ptr::null(), ..iana };
    /// // SAFETY: a null name is allowed, and refused.
    /// assert_eq!(unsafe { nameless.to_spec() }.unwrap_err().field(), Some("zone.zone"));
    /// ```
    ///
    /// # Safety
    ///
    /// `zone` must be null or a NUL-terminated string.
    ///
    /// # Errors
    ///
    /// An unknown kind, a missing name, an offset or longitude out of range.
    pub unsafe fn to_spec(&self) -> Result<ZoneSpec, Error> {
        match self.kind {
            0 => {
                // SAFETY: the caller's contract.
                let name = unsafe { optional_text(self.zone, "zone.zone") }?.ok_or_else(|| {
                    Error::invalid_arg("an IANA zone needs its name").with_field("zone.zone")
                })?;
                Ok(ZoneSpec::iana(name))
            }
            1 => Ok(ZoneSpec::fixed(
                UtcOffset::try_from_seconds(self.offset_seconds)
                    .map_err(|e| Error::from(e).with_field("zone.offset_seconds"))?,
            )),
            2 => Ok(ZoneSpec::local_mean(
                Longitude::try_new(self.longitude_deg)
                    .map_err(|e| Error::from(e).with_field("zone.longitude_deg"))?,
            )),
            other => Err(Error::invalid_arg(format!(
                "`zone.kind` is {other}; the kinds are IANA=0, FIXED=1, LOCAL_MEAN=2"
            ))
            .with_field("zone.kind")),
        }
    }
}

impl TsZoneResolution {
    fn of(
        ctx: &TsContext,
        resolution: &ZoneResolution,
        instant: JulianDay<Utc>,
    ) -> TsZoneResolution {
        let dst = TsDst::from(resolution.dst);
        let (chosen, shift) = match resolution.dst {
            DstOutcome::None => (TsChosen::Earlier, 0),
            DstOutcome::Gap { shifted_by_seconds } => (TsChosen::Earlier, shifted_by_seconds),
            DstOutcome::Overlap { chosen } => (TsChosen::from(chosen), 0),
        };
        let warnings = resolution
            .warnings
            .iter()
            .fold(0u32, |bits, w| bits | (1 << TsZoneWarning::from(*w) as u32));
        TsZoneResolution {
            struct_size: 0,
            offset_seconds: resolution.offset.seconds(),
            dst_shift_seconds: shift,
            warnings,
            instant_jd_utc: instant.get(),
            tzdb_version: ctx.lend(&resolution.tzdb_version).data,
            abbreviation: ctx.lend_ptr(resolution.abbreviation.as_deref()),
            source: TsZoneSource::from(resolution.source) as u8,
            era: TsZoneEra::from(resolution.era) as u8,
            dst: dst as u8,
            chosen: chosen as u8,
            time_known: u8::from(resolution.time_known),
            reserved: [0; 3],
        }
    }
}

/// Resolves a civil date-time in a zone to a UTC instant under the
/// context's daylight-saving and unknown-time policies, with the metadata
/// a stored chart keeps. An unknown zone is `UNSUPPORTED` with the nearest
/// names as the hint; a civil time inside a gap under the `error` policy
/// is `INVALID_ARG` with the `DST_GAP` detail.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// use teistro_core::Status;
/// # use teistro_ffi::time::TsZoneResolution;
/// #
/// # let mut resolution = TsZoneResolution {
/// #     struct_size: u32::try_from(size_of::<TsZoneResolution>())?,
/// #     offset_seconds: 0, dst_shift_seconds: 0, warnings: 0, instant_jd_utc: 0.0,
/// #     tzdb_version: core::ptr::null(), abbreviation: core::ptr::null(),
/// #     source: 0, era: 0, dst: 0, chosen: 0, time_known: 0, reserved: [0; 3],
/// # };
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_ffi::time::{TsCivilDateTime, TsZoneSpec, ts_time_resolve};
/// use teistro_time::{CivilDateTime, CivilTime};
///
/// // 06:15 on 6 September 2026 in Kathmandu.
/// let zone = TsZoneSpec { struct_size: u32::try_from(size_of::<TsZoneSpec>())?, zone: c"Asia/Kathmandu".as_ptr(), ..TsZoneSpec::default() };
/// let mut civil = TsCivilDateTime::of(&CivilDateTime::at(CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6), CivilTime::new(6, 15, 0)?));
/// civil.struct_size = u32::try_from(size_of::<TsCivilDateTime>())?;
/// civil.date.struct_size = u32::try_from(size_of::<teistro_ffi::calendar::TsCalendarDate>())?;
/// civil.time.struct_size = u32::try_from(size_of::<teistro_ffi::time::TsCivilTime>())?;
/// // SAFETY: a live context and sized records.
/// assert_eq!(unsafe { ts_time_resolve(&raw const ctx, &raw const civil, &raw const zone, &raw mut resolution) }, Status::Ok);
/// assert_eq!(resolution.offset_seconds, 20_700);
/// // 00:30 UTC: 06:15 less 5 h 45 min.
/// let utc_day_start = 2_461_289.5;
/// assert!((resolution.instant_jd_utc - (utc_day_start + 0.5 / 24.0)).abs() < 1e-6);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `civil` and `zone` valid for reads;
/// `out_resolution` valid for a read of its `struct_size` and a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_time_resolve(
    context: *const TsContext,
    civil: *const TsCivilDateTime,
    zone: *const TsZoneSpec,
    out_resolution: *mut TsZoneResolution,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let (civil, zone) = unsafe {
            (
                read_in(civil, "civil")?.to_civil()?,
                read_in(zone, "zone")?.to_spec()?,
            )
        };
        let policy = Policy::of(ctx.settings());
        let resolved = resolve(&civil, &zone, &policy, EmbeddedTzdb::shared())?;
        let out = TsZoneResolution::of(ctx, &resolved.zone, resolved.instant);
        // SAFETY: the entry point's contract.
        unsafe { write_out(out_resolution, "out_resolution", out) }
    })
}

/// The civil date-time of a UTC instant in a zone, in a calendar, with
/// the resolution that applied.
///
/// `api: calendar: enum=Calendar`
/// `api: jd_utc: unit=jd`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// use teistro_core::Status;
/// # use teistro_ffi::time::{TsCivilDateTime, TsZoneResolution};
/// # use teistro_calendar::CalendarDate;
/// #
/// # let mut resolution = TsZoneResolution {
/// #     struct_size: u32::try_from(size_of::<TsZoneResolution>())?,
/// #     offset_seconds: 0, dst_shift_seconds: 0, warnings: 0, instant_jd_utc: 0.0,
/// #     tzdb_version: core::ptr::null(), abbreviation: core::ptr::null(),
/// #     source: 0, era: 0, dst: 0, chosen: 0, time_known: 0, reserved: [0; 3],
/// # };
/// # let mut civil = TsCivilDateTime::of(&teistro_time::CivilDateTime::date_only(CalendarDate::defined(teistro_core::catalogue::Calendar::Gregorian, 1, 1, 1)));
/// # civil.struct_size = u32::try_from(size_of::<TsCivilDateTime>())?;
/// # civil.date.struct_size = u32::try_from(size_of::<teistro_ffi::calendar::TsCalendarDate>())?;
/// # civil.time.struct_size = u32::try_from(size_of::<teistro_ffi::time::TsCivilTime>())?;
/// use teistro_core::catalogue::{Calendar, Catalogued};
/// use teistro_ffi::time::{TsZoneKind, TsZoneSpec, ts_time_civil};
///
/// // 00:30 UTC on 6 September 2026, read on Nepal's clock.
/// let nepal = TsZoneSpec { struct_size: u32::try_from(size_of::<TsZoneSpec>())?, kind: TsZoneKind::Fixed as u8, offset_seconds: 20_700, ..TsZoneSpec::default() };
/// let instant = 2_461_289.5 + 0.5 / 24.0;
/// // SAFETY: a live context and sized records.
/// let status = unsafe { ts_time_civil(&raw const ctx, instant, &raw const nepal, Calendar::Gregorian.id(), &raw mut civil, &raw mut resolution) };
/// assert_eq!(status, Status::Ok);
/// assert_eq!((civil.date.day, civil.time.hour, civil.time.minute), (6, 6, 15));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `zone` valid for a read; `out_civil`
/// and `out_resolution` valid for reads of their `struct_size` and writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_time_civil(
    context: *const TsContext,
    jd_utc: f64,
    zone: *const TsZoneSpec,
    calendar: u16,
    out_civil: *mut TsCivilDateTime,
    out_resolution: *mut TsZoneResolution,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let zone = unsafe { read_in(zone, "zone")?.to_spec()? };
        let instant =
            JulianDay::<Utc>::try_new(jd_utc).map_err(|e| Error::from(e).with_field("jd_utc"))?;
        let (civil, resolution) =
            civil_of_with(system_of(calendar)?, instant, &zone, EmbeddedTzdb::shared())?;
        let out = TsZoneResolution::of(ctx, &resolution, instant);
        // SAFETY: the entry point's contract.
        unsafe {
            write_out(out_civil, "out_civil", TsCivilDateTime::of(&civil))?;
            write_out(out_resolution, "out_resolution", out)
        }
    })
}

/// What a conversion applied, before the target instant is known.
struct Applied {
    delta_t: Option<DeltaT>,
    proleptic_utc: bool,
    dut1_seconds: f64,
}

impl Applied {
    const NONE: Applied = Applied {
        delta_t: None,
        proleptic_utc: false,
        dut1_seconds: 0.0,
    };
}

/// One conversion between scales.
fn convert(jd: f64, from: TsScale, to: TsScale, ctx: &TsContext) -> Result<(f64, Applied), Error> {
    let model = ctx.delta_t();
    let jd_in = |field| JulianDay::<Ut1>::try_new(jd).map_err(|e| Error::from(e).with_field(field));
    Ok(match (from, to) {
        (TsScale::Ut1, TsScale::Tt) => {
            let (tt, applied) = tt_of(jd_in("jd")?, model)?;
            (
                tt.get(),
                Applied {
                    delta_t: Some(applied),
                    ..Applied::NONE
                },
            )
        }
        (TsScale::Tt, TsScale::Ut1) => {
            let (ut1, applied) = ut1_from_tt(jd_in("jd")?.relabel::<Tt>(), model)?;
            (
                ut1.get(),
                Applied {
                    delta_t: Some(applied),
                    ..Applied::NONE
                },
            )
        }
        (TsScale::Utc, TsScale::Tt) => {
            let conversion = tt_from_utc(jd_in("jd")?.relabel::<Utc>(), model)?;
            (
                conversion.tt.get(),
                Applied {
                    delta_t: Some(conversion.delta_t),
                    proleptic_utc: conversion.basis.proleptic_utc,
                    dut1_seconds: conversion.basis.dut1_applied_seconds,
                },
            )
        }
        (TsScale::Tt, TsScale::Utc) => {
            let (utc, conversion) = utc_from_tt(jd_in("jd")?.relabel::<Tt>(), model)?;
            (
                utc.get(),
                Applied {
                    delta_t: Some(conversion.delta_t),
                    proleptic_utc: conversion.basis.proleptic_utc,
                    dut1_seconds: conversion.basis.dut1_applied_seconds,
                },
            )
        }
        (TsScale::Utc, TsScale::Ut1) => {
            let (ut1, basis) = ut1_from_utc(jd_in("jd")?.relabel::<Utc>());
            (
                ut1.get(),
                Applied {
                    proleptic_utc: basis.proleptic_utc,
                    dut1_seconds: basis.dut1_applied_seconds,
                    delta_t: None,
                },
            )
        }
        (TsScale::Ut1, TsScale::Utc) => {
            let (utc, basis) = utc_from_ut1(jd_in("jd")?);
            (
                utc.get(),
                Applied {
                    proleptic_utc: basis.proleptic_utc,
                    dut1_seconds: basis.dut1_applied_seconds,
                    delta_t: None,
                },
            )
        }
        (TsScale::Ut1, TsScale::Ut1)
        | (TsScale::Tt, TsScale::Tt)
        | (TsScale::Utc, TsScale::Utc) => (jd_in("jd")?.get(), Applied::NONE),
    })
}

/// Converts an instant between UT1, TT and UTC under the context's Delta T
/// model, reporting what was applied. A model that cannot answer for the
/// instant is `OUT_OF_RANGE`.
///
/// `api: from: enum=TsScale`
/// `api: to: enum=TsScale`
/// `api: jd: unit=jd`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// use teistro_core::Status;
/// use teistro_ffi::time::{TsScale, TsTimeConversion, ts_time_convert};
///
/// let mut conversion = TsTimeConversion {
///     struct_size: u32::try_from(size_of::<TsTimeConversion>())?,
///     from: 0, to: 0, delta_t_source: 0, proleptic_utc: 0, has_uncertainty: 0, reserved: 0,
///     jd: 0.0, delta_t_seconds: 0.0, uncertainty_seconds: 0.0, dut1_seconds: 0.0,
///     delta_t_model: core::ptr::null(),
/// };
/// // TT back to UTC at J2000.0: the 64.184 s taken off again.
/// // SAFETY: a live context and a sized record.
/// assert_eq!(unsafe { ts_time_convert(&raw const ctx, 2_451_545.0, TsScale::Tt as u32, TsScale::Utc as u32, &raw mut conversion) }, Status::Ok);
/// assert!((conversion.jd - (2_451_545.0 - 64.184 / 86_400.0)).abs() < 1e-9);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `out_conversion` valid for a read of
/// its `struct_size` and a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_time_convert(
    context: *const TsContext,
    jd: f64,
    from: u32,
    to: u32,
    out_conversion: *mut TsTimeConversion,
) -> Status {
    with_context(context, |ctx| {
        let (from_scale, to_scale) = (TsScale::from_id(from, "from")?, TsScale::from_id(to, "to")?);
        let (converted, applied) = convert(jd, from_scale, to_scale, ctx)?;
        let out = TsTimeConversion {
            struct_size: 0,
            from,
            to,
            delta_t_source: applied
                .delta_t
                .map_or(TsDeltaTSource::Custom, |d| TsDeltaTSource::from(d.source))
                as u8,
            proleptic_utc: u8::from(applied.proleptic_utc),
            has_uncertainty: u8::from(
                applied
                    .delta_t
                    .is_some_and(|d| d.uncertainty_seconds.is_some()),
            ),
            reserved: 0,
            jd: converted,
            delta_t_seconds: applied.delta_t.map_or(0.0, |d| d.seconds),
            uncertainty_seconds: applied
                .delta_t
                .and_then(|d| d.uncertainty_seconds)
                .unwrap_or(0.0),
            dut1_seconds: applied.dut1_seconds,
            delta_t_model: ctx.lend(ctx.delta_t().key()).data,
        };
        // SAFETY: the entry point's contract.
        unsafe { write_out(out_conversion, "out_conversion", out) }
    })
}

/// Delta T (TT less UT1) at a UT1 instant under the context's model, with
/// what produced it and its uncertainty where the source has one.
///
/// `api: jd_ut1: unit=jd`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// use teistro_core::Status;
/// use teistro_ffi::time::{TsDeltaT, ts_time_delta_t};
///
/// let mut delta = TsDeltaT {
///     struct_size: u32::try_from(size_of::<TsDeltaT>())?,
///     source: 0, has_uncertainty: 0, reserved: [0; 2],
///     seconds: 0.0, uncertainty_seconds: 0.0, model: core::ptr::null(),
/// };
/// // SAFETY: a live context and a sized record.
/// assert_eq!(unsafe { ts_time_delta_t(&raw const ctx, 2_451_544.5, &raw mut delta) }, Status::Ok);
/// assert!((delta.seconds - 63.83).abs() < 0.02);
/// // The model's key, lent until the next call.
/// // SAFETY: a NUL-terminated string the context lent.
/// assert_eq!(unsafe { core::ffi::CStr::from_ptr(delta.model) }.to_str()?, "TABLE_THEN_MODEL");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `out_delta_t` valid for a read of its
/// `struct_size` and a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_time_delta_t(
    context: *const TsContext,
    jd_ut1: f64,
    out_delta_t: *mut TsDeltaT,
) -> Status {
    with_context(context, |ctx| {
        let instant =
            JulianDay::<Ut1>::try_new(jd_ut1).map_err(|e| Error::from(e).with_field("jd_ut1"))?;
        let value = delta_t(instant, ctx.delta_t())?;
        let out = TsDeltaT {
            struct_size: 0,
            source: TsDeltaTSource::from(value.source) as u8,
            has_uncertainty: u8::from(value.uncertainty_seconds.is_some()),
            reserved: [0; 2],
            seconds: value.seconds,
            uncertainty_seconds: value.uncertainty_seconds.unwrap_or(0.0),
            model: ctx.lend(value.model.key()).data,
        };
        // SAFETY: the entry point's contract.
        unsafe { write_out(out_delta_t, "out_delta_t", out) }
    })
}

/// A null string pointer, for tests and builders of zone specifications.
impl Default for TsZoneSpec {
    fn default() -> TsZoneSpec {
        TsZoneSpec {
            struct_size: 0,
            kind: 0,
            reserved: [0; 3],
            offset_seconds: 0,
            reserved2: 0,
            longitude_deg: 0.0,
            zone: ptr::null(),
        }
    }
}

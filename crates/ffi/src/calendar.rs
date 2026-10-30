//! Dates in every shipped calendar at the boundary: a date struct with the
//! era view and the resolution stamp, conversions through the fixed day,
//! month lengths, leap years, weekdays, and the fixed day's relation to
//! the Julian day.

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use teistro_calendar::arithmetic::{DateSpan, Overflow};
use teistro_calendar::{CalendarDate, CalendarSystem, EraNumber, FixedDay, Leap, shipped};
use teistro_core::Status;
use teistro_core::catalogue::{Calendar, Catalogued, Era};
use teistro_core::envelope::CalendarResolution;
use teistro_core::error::{Detail, Error};
use teistro_core::key::KeyId;

use crate::context::TsContext;
use crate::support::{c_struct, read_in, with_context, write_out, write_plain};

/// How a date was resolved (`docs/03-design/calendar-bikram-sambat.md`).
///
/// ```
/// use teistro_ffi::calendar::TsResolution;
///
/// // A member crosses the boundary as its number: `Defined` is 0, `Divergent` 3.
/// assert_eq!((TsResolution::Defined as u8, TsResolution::Divergent as u8), (0, 3));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsResolution {
    /// A mathematical definition; exact by construction.
    Defined = 0,
    /// From the authority's published table.
    Tabular = 1,
    /// Computed by the SDK's engine outside the table's range.
    Computed = 2,
    /// Inside the range and the table and the engine disagree; the table
    /// was followed and the engine's month and day are reported beside it.
    Divergent = 3,
}

/// Which part of a date is the first of a repeated pair, in a calendar
/// that repeats them (the Tibetan). A date marked neither holds
/// [`TsLeap::NONE`].
///
/// ```
/// use teistro_ffi::calendar::TsLeap;
///
/// // A member crosses the boundary as its number: `Month` is 1, `MonthAndDay` 3.
/// assert_eq!((TsLeap::Month as u8, TsLeap::MonthAndDay as u8), (1, 3));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsLeap {
    /// The leap month, before the month whose number it takes.
    Month = 1,
    /// The leap day, before the day whose number it repeats.
    Day = 2,
    /// The leap day of the leap month.
    MonthAndDay = 3,
}

impl TsLeap {
    /// What a date marked neither holds, as a nullable catalogue field
    /// holds `0xFFFF`; `0`, what a caller compiled before the field
    /// existed left in its reserved byte, is read as none too.
    ///
    /// ```
    /// use teistro_ffi::calendar::TsLeap;
    ///
    /// // Read as neither, as a zero is.
    /// assert_eq!(TsLeap::read(TsLeap::NONE)?, None);
    /// assert_eq!(TsLeap::read(0)?, None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    pub const NONE: u8 = 0xFF;

    /// The boundary form of a date's mark.
    ///
    /// ```
    /// use teistro_calendar::Leap;
    /// use teistro_ffi::calendar::TsLeap;
    ///
    /// assert_eq!(TsLeap::of(Some(Leap::Day)), 2);
    /// assert_eq!(TsLeap::of(None), TsLeap::NONE);
    /// ```
    #[must_use]
    pub const fn of(leap: Option<Leap>) -> u8 {
        match leap {
            None => TsLeap::NONE,
            Some(Leap::Month) => TsLeap::Month as u8,
            Some(Leap::Day) => TsLeap::Day as u8,
            Some(Leap::MonthAndDay) => TsLeap::MonthAndDay as u8,
        }
    }

    /// The mark a byte names.
    ///
    /// ```
    /// use teistro_calendar::Leap;
    /// use teistro_core::Status;
    /// use teistro_ffi::calendar::TsLeap;
    ///
    /// assert_eq!(TsLeap::read(3)?, Some(Leap::MonthAndDay));
    /// assert_eq!(TsLeap::read(4).map_err(|error| error.status), Err(Status::InvalidArg));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a byte that is no mark.
    pub fn read(byte: u8) -> Result<Option<Leap>, Error> {
        match byte {
            0 | TsLeap::NONE => Ok(None),
            1 => Ok(Some(Leap::Month)),
            2 => Ok(Some(Leap::Day)),
            3 => Ok(Some(Leap::MonthAndDay)),
            other => Err(Error::invalid_arg(format!(
                "a date's leap is 1 (the month), 2 (the day), 3 (both) or 0xFF (neither), not {other}"
            ))
            .with_field("leap")),
        }
    }
}

/// What a day number the target month lacks resolves to, when a date is
/// moved by months or a difference counts from such a day
/// (`03-design/calendar-bikram-sambat.md` §11): Temporal's `constrain`
/// and `reject`, and java.time's lenient carry.
///
/// ```
/// use teistro_ffi::calendar::TsOverflow;
///
/// // A member crosses the boundary as its number: `Constrain` is 0, `Reject` 2.
/// assert_eq!((TsOverflow::Constrain as u8, TsOverflow::Reject as u8), (0, 2));
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsOverflow {
    /// The month's last day: the 32nd of Jestha plus a month is the 31st
    /// of a 31-day Ashadh.
    Constrain = 0,
    /// The days past the month's end carried into the next month: the
    /// 1st of Shrawan.
    Carry = 1,
    /// Refused, `INVALID_ARG` with `NONEXISTENT_DATE` naming `day`.
    Reject = 2,
}

impl TsOverflow {
    /// The rule a byte names.
    ///
    /// ```
    /// use teistro_calendar::arithmetic::Overflow;
    /// use teistro_core::Status;
    /// use teistro_ffi::calendar::TsOverflow;
    ///
    /// assert_eq!(TsOverflow::read(1)?, Overflow::Carry);
    /// assert_eq!(TsOverflow::read(3).map_err(|error| error.status), Err(Status::InvalidArg));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `overflow` for a byte that is no rule.
    pub fn read(byte: u8) -> Result<Overflow, Error> {
        Overflow::ALL
            .get(usize::from(byte))
            .copied()
            .ok_or_else(|| {
                Error::invalid_arg(format!(
                    "`overflow` is {byte}; the rules are CONSTRAIN=0, CARRY=1, REJECT=2"
                ))
                .with_field("overflow")
            })
    }
}

/// Years, months and days, each signed: the difference of two dates.
///
/// ```
/// use teistro_calendar::arithmetic::DateSpan;
/// use teistro_ffi::calendar::TsDateSpan;
///
/// let span = TsDateSpan::of(DateSpan::new(30, 1, -2));
/// assert_eq!((span.years, span.months, span.days), (30, 1, -2));
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TsDateSpan {
    /// `sizeof(ts_date_span)` as the caller compiled it.
    pub struct_size: u32,
    /// Whole years.
    /// `api: example=30`
    pub years: i32,
    /// Whole months, -11 to 11.
    /// `api: example=0`
    pub months: i32,
    /// Days.
    /// `api: example=1`
    pub days: i32,
}

c_struct!(TsDateSpan);

impl TsDateSpan {
    /// The boundary form of a span.
    ///
    /// ```
    /// use teistro_calendar::arithmetic::DateSpan;
    /// use teistro_ffi::calendar::TsDateSpan;
    ///
    /// assert_eq!(TsDateSpan::of(DateSpan::days(3)).struct_size, 0);
    /// ```
    #[must_use]
    pub const fn of(span: DateSpan) -> TsDateSpan {
        TsDateSpan {
            struct_size: 0,
            years: span.years,
            months: span.months,
            days: span.days,
        }
    }
}

impl From<&CalendarResolution> for TsResolution {
    fn from(resolution: &CalendarResolution) -> TsResolution {
        match resolution {
            CalendarResolution::Defined => TsResolution::Defined,
            CalendarResolution::Tabular { .. } => TsResolution::Tabular,
            CalendarResolution::Computed { .. } => TsResolution::Computed,
            CalendarResolution::Divergent { .. } => TsResolution::Divergent,
        }
    }
}

/// A date in a calendar. Years are astronomical (1 BCE is 0); the era view
/// is presentation only and is filled by the library, ignored on input.
///
/// ```
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_ffi::calendar::TsCalendarDate;
///
/// let date = TsCalendarDate::of(&CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6));
/// assert_eq!((date.year, date.month, date.day), (2026, 9, 6));
/// assert_eq!(date.calendar, Calendar::Gregorian.id());
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TsCalendarDate {
    /// `sizeof(ts_calendar_date)` as the caller compiled it.
    pub struct_size: u32,
    /// The calendar's id.
    /// `api: enum=Calendar example=0`
    pub calendar: u16,
    /// The era the calendar attaches, `0xFFFF` for none; filled by the
    /// library and ignored on input.
    /// `api: enum=Era nullable example=7`
    pub era: u16,
    /// The astronomical year.
    /// `api: example=2026`
    pub year: i32,
    /// The year in the era, 1-based; meaningful when `era` is set.
    /// `api: example=2026`
    pub era_year: i32,
    /// The month, 1-based; for the ISO week date, the week.
    /// `api: range=[1,53] example=9`
    pub month: u8,
    /// The day, 1-based; for the ISO week date, the weekday 1 to 7.
    /// `api: range=[1,32] example=6`
    pub day: u8,
    /// How the date was resolved.
    /// `api: enum=TsResolution example=0`
    pub resolution: u8,
    /// The engine's month when the resolution is divergent, else zero.
    pub computed_month: u8,
    /// The engine's day when the resolution is divergent, else zero.
    pub computed_day: u8,
    /// Whether the month or the day is the first of a repeated pair, in a
    /// calendar that repeats them; `0xFF` for neither, which is every date
    /// of every other calendar (`0` is read as neither too).
    /// `api: enum=TsLeap nullable example=1`
    pub leap: u8,
    /// Reserved, zero.
    pub reserved: [u8; 2],
}

c_struct!(TsCalendarDate);

impl TsCalendarDate {
    /// The boundary form of a date.
    ///
    /// ```
    /// use teistro_calendar::CalendarDate;
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_ffi::calendar::TsCalendarDate;
    ///
    /// use teistro_ffi::calendar::{TsLeap, TsResolution};
    ///
    /// let date = TsCalendarDate::of(&CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6));
    /// assert_eq!(date.resolution, TsResolution::Defined as u8);
    /// assert_eq!(date.leap, TsLeap::NONE);
    /// ```
    #[must_use]
    pub fn of(date: &CalendarDate) -> TsCalendarDate {
        let resolution = TsResolution::from(&date.resolution);
        let computed = match &date.resolution {
            CalendarResolution::Divergent { computed, .. } => Some(*computed),
            _ => None,
        };
        TsCalendarDate {
            struct_size: 0,
            calendar: date.calendar.id(),
            era: date.era.map_or(KeyId::NONE_ID, |e| e.era.id()),
            year: date.year,
            era_year: date.era.map_or(0, |e| e.year),
            month: date.month,
            day: date.day,
            resolution: resolution as u8,
            computed_month: computed.map_or(0, |c| c.month),
            computed_day: computed.map_or(0, |c| c.day),
            leap: TsLeap::of(date.leap),
            reserved: [0; 2],
        }
    }

    /// The date this struct names, its resolution taken as defined.
    ///
    /// ```
    /// use teistro_calendar::CalendarDate;
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_ffi::calendar::TsCalendarDate;
    ///
    /// let date = CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6);
    /// assert_eq!(TsCalendarDate::of(&date).to_date()?, date);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A calendar or era id the catalogue does not have.
    pub fn to_date(&self) -> Result<CalendarDate, Error> {
        let calendar = calendar_of(self.calendar)?;
        let mut date = CalendarDate::defined(calendar, self.year, self.month, self.day)
            .with_leap(TsLeap::read(self.leap)?);
        if self.era != KeyId::NONE_ID {
            let era =
                Era::from_id(self.era).ok_or_else(|| unknown_member::<Era>(self.era, "era"))?;
            date.era = Some(EraNumber {
                era,
                year: self.era_year,
            });
        }
        Ok(date)
    }
}

/// `UNSUPPORTED` naming the kind and the id.
fn unknown_member<T: Catalogued>(id: u16, field: &str) -> Error {
    Error::unsupported(format!("no {} has id {id}", T::KIND.name()))
        .with_detail(Detail::UnknownKey)
        .with_field(field)
        .with_hint(format!(
            "the {} ids are {}",
            T::KIND.name(),
            T::all()
                .iter()
                .map(|m| format!("{}={}", m.key(), m.id()))
                .collect::<Vec<_>>()
                .join(", ")
        ))
}

fn calendar_of(id: u16) -> Result<Calendar, Error> {
    Calendar::from_id(id).ok_or_else(|| unknown_member::<Calendar>(id, "calendar"))
}

/// The shipped calendar with an id.
pub(crate) fn system_of(id: u16) -> Result<&'static dyn CalendarSystem, Error> {
    let calendar = calendar_of(id)?;
    shipped(calendar).ok_or_else(|| {
        Error::unsupported(format!(
            "the {} calendar needs a context's ephemeris and is not available yet",
            calendar.key()
        ))
        .with_field("calendar")
    })
}

/// The date a fixed day falls on in a calendar.
///
/// `api: calendar: enum=Calendar`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::{Calendar, Catalogued};
/// # use teistro_ffi::calendar::TsCalendarDate;
/// #
/// # let sized = |date: CalendarDate| -> Result<TsCalendarDate, core::num::TryFromIntError> {
/// #     Ok(TsCalendarDate { struct_size: u32::try_from(size_of::<TsCalendarDate>())?, ..TsCalendarDate::of(&date) })
/// # };
/// use teistro_core::Status;
/// use teistro_calendar::{CalendarSystem, Gregorian};
/// use teistro_ffi::calendar::ts_calendar_from_fixed;
///
/// let fixed = Gregorian.to_fixed_ymd(2026, 9, 6)?.get();
/// let mut date = sized(CalendarDate::defined(Calendar::Gregorian, 1, 1, 1))?;
/// // SAFETY: a live context and a sized, writable date.
/// let status = unsafe { ts_calendar_from_fixed(&raw const ctx, Calendar::Gregorian.id(), fixed, &raw mut date) };
/// assert_eq!(status, Status::Ok);
/// assert_eq!((date.year, date.month, date.day), (2026, 9, 6));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `out_date` valid for a read of its
/// `struct_size` and a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_calendar_from_fixed(
    context: *const TsContext,
    calendar: u16,
    fixed: i64,
    out_date: *mut TsCalendarDate,
) -> Status {
    with_context(context, |_| {
        let date = system_of(calendar)?.date_of(FixedDay::new(fixed))?;
        // SAFETY: the entry point's contract.
        unsafe { write_out(out_date, "out_date", TsCalendarDate::of(&date)) }
    })
}

/// The fixed day of a date, validated: a date the calendar does not have
/// is `INVALID_ARG` with the `NONEXISTENT_DATE` detail.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::{Calendar, Catalogued};
/// # use teistro_ffi::calendar::TsCalendarDate;
/// #
/// # let sized = |date: CalendarDate| -> Result<TsCalendarDate, core::num::TryFromIntError> {
/// #     Ok(TsCalendarDate { struct_size: u32::try_from(size_of::<TsCalendarDate>())?, ..TsCalendarDate::of(&date) })
/// # };
/// use teistro_core::Status;
/// use teistro_calendar::{CalendarSystem, Gregorian};
/// use teistro_ffi::calendar::ts_calendar_to_fixed;
///
/// let date = sized(CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6))?;
/// let mut fixed = 0_i64;
/// // SAFETY: a live context, a sized date and a writable slot.
/// assert_eq!(unsafe { ts_calendar_to_fixed(&raw const ctx, &raw const date, &raw mut fixed) }, Status::Ok);
/// assert_eq!(fixed, Gregorian.to_fixed_ymd(2026, 9, 6)?.get());
/// // The 30th of February is no date.
/// let none = sized(CalendarDate::defined(Calendar::Gregorian, 2026, 2, 30))?;
/// // SAFETY: as above.
/// assert_eq!(unsafe { ts_calendar_to_fixed(&raw const ctx, &raw const none, &raw mut fixed) }, Status::InvalidArg);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `date` valid for a read; `out_fixed`
/// valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_calendar_to_fixed(
    context: *const TsContext,
    date: *const TsCalendarDate,
    out_fixed: *mut i64,
) -> Status {
    with_context(context, |_| {
        // SAFETY: the entry point's contract.
        let date = unsafe { read_in(date, "date") }?.to_date()?;
        let fixed = system_of(date.calendar.id())?.fixed_of(&date)?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_fixed, "out_fixed", fixed.get()) }
    })
}

/// Converts a date into another calendar.
///
/// `api: into: enum=Calendar`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::{Calendar, Catalogued};
/// # use teistro_ffi::calendar::TsCalendarDate;
/// #
/// # let sized = |date: CalendarDate| -> Result<TsCalendarDate, core::num::TryFromIntError> {
/// #     Ok(TsCalendarDate { struct_size: u32::try_from(size_of::<TsCalendarDate>())?, ..TsCalendarDate::of(&date) })
/// # };
/// use teistro_core::Status;
/// use teistro_ffi::calendar::ts_calendar_convert;
///
/// // 14 April 2026 is the first of Baisakh, 2083, the Bikram Sambat's new year.
/// let april = sized(CalendarDate::defined(Calendar::Gregorian, 2026, 4, 14))?;
/// let mut baisakh = sized(CalendarDate::defined(Calendar::Gregorian, 1, 1, 1))?;
/// // SAFETY: a live context and two sized dates.
/// let status = unsafe { ts_calendar_convert(&raw const ctx, &raw const april, Calendar::BikramSambat.id(), &raw mut baisakh) };
/// assert_eq!(status, Status::Ok);
/// assert_eq!((baisakh.year, baisakh.month, baisakh.day), (2083, 1, 1));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `date` valid for a read; `out_date`
/// valid for a read of its `struct_size` and a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_calendar_convert(
    context: *const TsContext,
    date: *const TsCalendarDate,
    into: u16,
    out_date: *mut TsCalendarDate,
) -> Status {
    with_context(context, |_| {
        // SAFETY: the entry point's contract.
        let date = unsafe { read_in(date, "date") }?.to_date()?;
        let converted = system_of(date.calendar.id())?.convert(&date, system_of(into)?)?;
        // SAFETY: the entry point's contract.
        unsafe { write_out(out_date, "out_date", TsCalendarDate::of(&converted)) }
    })
}

/// The length of a month in a calendar.
///
/// `api: calendar: enum=Calendar`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::{Calendar, Catalogued};
/// # use teistro_ffi::calendar::TsCalendarDate;
/// #
/// # let sized = |date: CalendarDate| -> Result<TsCalendarDate, core::num::TryFromIntError> {
/// #     Ok(TsCalendarDate { struct_size: u32::try_from(size_of::<TsCalendarDate>())?, ..TsCalendarDate::of(&date) })
/// # };
/// use teistro_core::Status;
/// use teistro_ffi::calendar::ts_calendar_month_length;
///
/// let mut length = 0_u8;
/// // SAFETY: a live context and a writable slot.
/// assert_eq!(unsafe { ts_calendar_month_length(&raw const ctx, Calendar::Gregorian.id(), 2024, 2, &raw mut length) }, Status::Ok);
/// assert_eq!(length, 29);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `out_length` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_calendar_month_length(
    context: *const TsContext,
    calendar: u16,
    year: i32,
    month: u8,
    out_length: *mut u8,
) -> Status {
    with_context(context, |_| {
        let length = system_of(calendar)?.month_length(year, month)?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_length, "out_length", length) }
    })
}

/// Whether a year is a leap year in a calendar: `1` or `0`.
///
/// `api: calendar: enum=Calendar`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::{Calendar, Catalogued};
/// # use teistro_ffi::calendar::TsCalendarDate;
/// #
/// # let sized = |date: CalendarDate| -> Result<TsCalendarDate, core::num::TryFromIntError> {
/// #     Ok(TsCalendarDate { struct_size: u32::try_from(size_of::<TsCalendarDate>())?, ..TsCalendarDate::of(&date) })
/// # };
/// use teistro_core::Status;
/// use teistro_ffi::calendar::ts_calendar_is_leap;
///
/// let mut leap = 9_u8;
/// // SAFETY: a live context and a writable slot.
/// assert_eq!(unsafe { ts_calendar_is_leap(&raw const ctx, Calendar::Gregorian.id(), 1900, &raw mut leap) }, Status::Ok);
/// assert_eq!(leap, 0, "a century not divisible by 400");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `out_leap` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_calendar_is_leap(
    context: *const TsContext,
    calendar: u16,
    year: i32,
    out_leap: *mut u8,
) -> Status {
    with_context(context, |_| {
        let leap = system_of(calendar)?.is_leap(year);
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_leap, "out_leap", u8::from(leap)) }
    })
}

/// The weekday of a date as its ISO number: Monday is `1`, Sunday `7`.
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::{Calendar, Catalogued};
/// # use teistro_ffi::calendar::TsCalendarDate;
/// #
/// # let sized = |date: CalendarDate| -> Result<TsCalendarDate, core::num::TryFromIntError> {
/// #     Ok(TsCalendarDate { struct_size: u32::try_from(size_of::<TsCalendarDate>())?, ..TsCalendarDate::of(&date) })
/// # };
/// use teistro_core::Status;
/// use teistro_ffi::calendar::ts_calendar_weekday;
///
/// // 6 September 2026 is a Sunday: ISO's 7.
/// let date = sized(CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6))?;
/// let mut weekday = 0_u8;
/// // SAFETY: a live context, a sized date and a writable slot.
/// assert_eq!(unsafe { ts_calendar_weekday(&raw const ctx, &raw const date, &raw mut weekday) }, Status::Ok);
/// assert_eq!(weekday, 7);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `date` valid for a read; `out_weekday`
/// valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_calendar_weekday(
    context: *const TsContext,
    date: *const TsCalendarDate,
    out_weekday: *mut u8,
) -> Status {
    with_context(context, |_| {
        // SAFETY: the entry point's contract.
        let date = unsafe { read_in(date, "date") }?.to_date()?;
        let weekday = system_of(date.calendar.id())?.weekday(&date)?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_weekday, "out_weekday", weekday.iso_number()) }
    })
}

/// The date `years`, `months` and `days` after `date`, in its own
/// calendar (`03-design/calendar-bikram-sambat.md` §11): the years and
/// months first, a day number the month lacks resolved by `overflow`,
/// then the days, each count signed. A Bikram Sambat month runs 29 to 32
/// days, so the 32nd of Jestha plus a month is the 31st of a 31-day
/// Ashadh under `CONSTRAIN`, the 1st of Shrawan under `CARRY`, and
/// `INVALID_ARG` with `NONEXISTENT_DATE` naming `day` under `REJECT`. The
/// ISO week date and the Tibetan calendar, which number no twelve months,
/// are `UNSUPPORTED`; a date past the calendar's span `OUT_OF_RANGE`.
///
/// `api: overflow: enum=TsOverflow`
/// `api: months: example=1`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::{Calendar, Catalogued};
/// # use teistro_ffi::calendar::TsCalendarDate;
/// #
/// # let sized = |date: CalendarDate| -> Result<TsCalendarDate, core::num::TryFromIntError> {
/// #     Ok(TsCalendarDate { struct_size: u32::try_from(size_of::<TsCalendarDate>())?, ..TsCalendarDate::of(&date) })
/// # };
/// use teistro_core::Status;
/// use teistro_ffi::calendar::{TsOverflow, ts_calendar_add};
///
/// // Jestha 2081 has 32 days and Ashadh 31.
/// let jestha_32 = sized(CalendarDate::defined(Calendar::BikramSambat, 2081, 2, 32))?;
/// let mut on = sized(CalendarDate::defined(Calendar::Gregorian, 1, 1, 1))?;
/// // SAFETY: a live context and two sized dates.
/// let status = unsafe { ts_calendar_add(&raw const ctx, &raw const jestha_32, 0, 1, 0, TsOverflow::Constrain as u8, &raw mut on) };
/// assert_eq!(status, Status::Ok);
/// assert_eq!((on.year, on.month, on.day), (2081, 3, 31));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `date` valid for a read; `out_date`
/// valid for a read of its `struct_size` and a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_calendar_add(
    context: *const TsContext,
    date: *const TsCalendarDate,
    years: i32,
    months: i32,
    days: i32,
    overflow: u8,
    out_date: *mut TsCalendarDate,
) -> Status {
    with_context(context, |ctx| {
        let overflow = TsOverflow::read(overflow)?;
        // SAFETY: the entry point's contract.
        let date = unsafe { read_in(date, "date") }?.to_date()?;
        let span = DateSpan::new(years, months, days);
        let moved = ctx.sdk().calendar().add(&date, span, overflow)?;
        // SAFETY: the entry point's contract.
        unsafe { write_out(out_date, "out_date", TsCalendarDate::of(&moved)) }
    })
}

/// The years, months and days from `from` to `to`, both in one calendar,
/// counted as an age is (`03-design/calendar-bikram-sambat.md` §11): the
/// most whole months whose anniversary of `from` is not past `to`, then
/// the days from it, so `ts_calendar_add` of the span to `from` under the
/// same `overflow` gives `to`. An anniversary on a day its month lacks is
/// resolved by `overflow`: one born on the 32nd of Jestha completes a year
/// on the 31st of a 31-day Jestha under `CONSTRAIN`, on the 1st of Ashadh
/// under `CARRY`, and `REJECT` refuses a count from such a day (C806).
/// Before `from`, each part is negative or zero. Dates in two calendars
/// are `INVALID_ARG`, naming `calendar`.
///
/// `api: overflow: enum=TsOverflow`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # use teistro_calendar::CalendarDate;
/// # use teistro_core::catalogue::{Calendar, Catalogued};
/// # use teistro_ffi::calendar::TsCalendarDate;
/// #
/// # let sized = |date: CalendarDate| -> Result<TsCalendarDate, core::num::TryFromIntError> {
/// #     Ok(TsCalendarDate { struct_size: u32::try_from(size_of::<TsCalendarDate>())?, ..TsCalendarDate::of(&date) })
/// # };
/// use teistro_core::Status;
/// use teistro_ffi::calendar::{TsDateSpan, TsOverflow, ts_calendar_between};
///
/// let born = sized(CalendarDate::defined(Calendar::BikramSambat, 2050, 2, 32))?;
/// let birthday = sized(CalendarDate::defined(Calendar::BikramSambat, 2082, 2, 31))?;
/// let mut age = TsDateSpan { struct_size: u32::try_from(size_of::<TsDateSpan>())?, years: 0, months: 0, days: 0 };
/// // SAFETY: a live context, two sized dates and a sized span.
/// let status = unsafe { ts_calendar_between(&raw const ctx, &raw const born, &raw const birthday, TsOverflow::Constrain as u8, &raw mut age) };
/// assert_eq!(status, Status::Ok);
/// // Jestha 2082 has 31 days, so its last is the birthday.
/// assert_eq!((age.years, age.months, age.days), (32, 0, 0));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `from` and `to` valid for a read;
/// `out_span` valid for a read of its `struct_size` and a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_calendar_between(
    context: *const TsContext,
    from: *const TsCalendarDate,
    to: *const TsCalendarDate,
    overflow: u8,
    out_span: *mut TsDateSpan,
) -> Status {
    with_context(context, |ctx| {
        let overflow = TsOverflow::read(overflow)?;
        // SAFETY: the entry point's contract.
        let from = unsafe { read_in(from, "from") }?.to_date()?;
        // SAFETY: the entry point's contract.
        let to = unsafe { read_in(to, "to") }?.to_date()?;
        let span = ctx.sdk().calendar().between(&from, &to, overflow)?;
        // SAFETY: the entry point's contract.
        unsafe { write_out(out_span, "out_span", TsDateSpan::of(span)) }
    })
}

/// A Tibetan year's names (`calendars/tibetan.md`), as canonical JSON:
/// `{year, rabjung, yearInRabjung, samvatsara, element, gender, animal}`,
/// the rabjung cycle (the first began in 1027) and the year's place in
/// it, 1 to 60, with the Indian cycle's name for that place, and the
/// element, gender and animal it is called by, every member a full key:
/// 2026 is the 40th of the 17th, `samvatsara.PARABHAVA`, Fire–male–Horse.
/// `year` is the one a `TIBETAN_PHUGPA` date carries. A year outside 1027
/// to 9998 is `OUT_OF_RANGE`, naming `year`.
///
/// `api: year: example=2026`
///
/// ```
/// # use teistro_ffi::context::{OptionTexts, TsContext};
/// #
/// # let ctx = TsContext::build(&OptionTexts::default(), teistro::Ephemeris::Test)?;
/// # fn owned(mut out: teistro_ffi::string::TsString) -> String {
/// #     // SAFETY: the library wrote `len` bytes of UTF-8 at `data`.
/// #     let text = String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(out.data, out.len) }).into_owned();
/// #     // SAFETY: a descriptor the library wrote, freed once.
/// #     unsafe { teistro_ffi::string::ts_string_free(&raw mut out) };
/// #     text
/// # }
/// use teistro_core::Status;
/// use teistro_ffi::calendar::ts_calendar_tibetan_year;
/// use teistro_ffi::string::TsString;
///
/// let mut json = TsString::empty();
/// // SAFETY: a live context and a writable slot.
/// assert_eq!(unsafe { ts_calendar_tibetan_year(&raw const ctx, 2026, &raw mut json) }, Status::Ok);
/// let year: serde_json::Value = serde_json::from_str(&owned(json))?;
/// assert_eq!((year["rabjung"].as_u64(), year["yearInRabjung"].as_u64()), (Some(17), Some(40)));
/// assert_eq!(year["samvatsara"], "samvatsara.PARABHAVA");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Safety
///
/// `context` must be a live handle; `out_json` valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_calendar_tibetan_year(
    context: *const TsContext,
    year: i32,
    out_json: *mut crate::string::TsString,
) -> Status {
    with_context(context, |ctx| {
        let value = ctx.sdk().calendar().tibetan_year_in_full(year)?;
        let json =
            crate::string::TsString::from_string(teistro_core::envelope::canonical_json(&value));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// The Julian day at the UTC midnight that begins a fixed day (fixed day
/// 1 is Monday, 1 January 1 CE; the relation is `fixed + 1721424.5`).
///
/// `api: fixed: example=735702`
///
/// ```
/// use teistro_ffi::calendar::ts_calendar_jd_of_fixed;
///
/// // Fixed day 1, Monday 1 January 1 CE, begins at this Julian day.
/// assert_eq!(ts_calendar_jd_of_fixed(1), 1_721_425.5);
/// ```
#[cfg_attr(not(test), unsafe(no_mangle))]
#[allow(clippy::cast_precision_loss, reason = "days are far below 2^53")]
pub extern "C" fn ts_calendar_jd_of_fixed(fixed: i64) -> f64 {
    fixed as f64 + FixedDay::JD_EPOCH
}

/// The fixed day a Julian day falls in, and, when `out_fraction` is not
/// null, the fraction of that day elapsed since its midnight.
///
/// ```
/// use teistro_ffi::calendar::{ts_calendar_fixed_of_jd, ts_calendar_jd_of_fixed};
///
/// // Noon is half a day in.
/// let noon = ts_calendar_jd_of_fixed(735_702) + 0.5;
/// let mut fraction = 0.0;
/// // SAFETY: a writable slot.
/// assert_eq!(unsafe { ts_calendar_fixed_of_jd(noon, &raw mut fraction) }, 735_702);
/// assert_eq!(fraction, 0.5);
/// // SAFETY: a null fraction is allowed.
/// assert_eq!(unsafe { ts_calendar_fixed_of_jd(noon, core::ptr::null_mut()) }, 735_702);
/// ```
///
/// # Safety
///
/// `out_fraction` must be null or valid for a write.
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe extern "C" fn ts_calendar_fixed_of_jd(jd: f64, out_fraction: *mut f64) -> i64 {
    let (day, fraction) = FixedDay::from_local_jd(jd);
    if !out_fraction.is_null() {
        // SAFETY: non-null; the caller promises a writable slot.
        unsafe { out_fraction.write(fraction) };
    }
    day.get()
}

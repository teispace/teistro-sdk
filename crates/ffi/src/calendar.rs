//! Dates in every shipped calendar at the boundary: a date struct with the
//! era view and the resolution stamp, conversions through the fixed day,
//! month lengths, leap years, weekdays, and the fixed day's relation to
//! the Julian day.

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use teistro_calendar::{CalendarDate, CalendarSystem, EraNumber, FixedDay, shipped};
use teistro_core::Status;
use teistro_core::catalogue::{Calendar, Catalogued, Era};
use teistro_core::envelope::CalendarResolution;
use teistro_core::error::{Detail, Error};
use teistro_core::key::KeyId;

use crate::context::TsContext;
use crate::support::{c_struct, read_in, with_context, write_out, write_plain};

/// How a date was resolved (`docs/03-design/calendar-bikram-sambat.md`).
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

/// A date in a calendar. Years are astronomical (1 BCE is 0); the era view
/// is presentation only and is filled by the library, ignored on input.
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
    /// Reserved, zero.
    pub reserved: [u8; 3],
}

c_struct!(TsCalendarDate);

impl TsCalendarDate {
    /// The boundary form of a date.
    #[must_use]
    pub fn of(date: &CalendarDate) -> TsCalendarDate {
        let (resolution, computed) = match &date.resolution {
            CalendarResolution::Defined => (TsResolution::Defined, None),
            CalendarResolution::Tabular { .. } => (TsResolution::Tabular, None),
            CalendarResolution::Computed { .. } => (TsResolution::Computed, None),
            CalendarResolution::Divergent { computed, .. } => {
                (TsResolution::Divergent, Some(*computed))
            }
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
            reserved: [0; 3],
        }
    }

    /// The date this struct names, its resolution taken as defined.
    ///
    /// # Errors
    ///
    /// A calendar or era id the catalogue does not have.
    pub fn to_date(&self) -> Result<CalendarDate, Error> {
        let calendar = calendar_of(self.calendar)?;
        let mut date = CalendarDate::defined(calendar, self.year, self.month, self.day);
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
/// # Safety
///
/// `context` must be a live handle; `out_date` valid for a read of its
/// `struct_size` and a write.
#[unsafe(no_mangle)]
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
/// # Safety
///
/// `context` must be a live handle; `date` valid for a read; `out_fixed`
/// valid for a write.
#[unsafe(no_mangle)]
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
/// # Safety
///
/// `context` must be a live handle; `date` valid for a read; `out_date`
/// valid for a read of its `struct_size` and a write.
#[unsafe(no_mangle)]
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
/// # Safety
///
/// `context` must be a live handle; `out_length` valid for a write.
#[unsafe(no_mangle)]
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
/// # Safety
///
/// `context` must be a live handle; `out_leap` valid for a write.
#[unsafe(no_mangle)]
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
/// # Safety
///
/// `context` must be a live handle; `date` valid for a read; `out_weekday`
/// valid for a write.
#[unsafe(no_mangle)]
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

/// The Julian day at the UTC midnight that begins a fixed day (fixed day
/// 1 is Monday, 1 January 1 CE; the relation is `fixed + 1721424.5`).
///
/// `api: fixed: example=735702`
#[unsafe(no_mangle)]
#[allow(clippy::cast_precision_loss, reason = "days are far below 2^53")]
pub extern "C" fn ts_calendar_jd_of_fixed(fixed: i64) -> f64 {
    fixed as f64 + FixedDay::JD_EPOCH
}

/// The fixed day a Julian day falls in, and, when `out_fraction` is not
/// null, the fraction of that day elapsed since its midnight.
///
/// # Safety
///
/// `out_fraction` must be null or valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_calendar_fixed_of_jd(jd: f64, out_fraction: *mut f64) -> i64 {
    let (day, fraction) = FixedDay::from_local_jd(jd);
    if !out_fraction.is_null() {
        // SAFETY: non-null; the caller promises a writable slot.
        unsafe { out_fraction.write(fraction) };
    }
    day.get()
}

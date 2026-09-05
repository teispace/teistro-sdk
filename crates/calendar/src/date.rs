//! A date in any calendar, the trait every calendar implements, and the
//! shipped calendars by key.

use core::fmt;

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::envelope::CalendarResolution;
use teistro_core::error::{Detail, Error, Status};

use crate::fixed::{FixedDay, Weekday};

/// A year in an era, for presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EraNumber {
    /// The era.
    pub era: Era,
    /// The year in it, 1-based.
    pub year: i32,
}

/// A date in a calendar. Years are astronomical (1 BCE is 0); an era is
/// attached for presentation only.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CalendarDate {
    /// The calendar.
    pub calendar: Calendar,
    /// The astronomical year.
    pub year: i32,
    /// The month, 1-based; for the ISO week date, the week.
    pub month: u8,
    /// The day, 1-based; for the ISO week date, the weekday 1 to 7.
    pub day: u8,
    /// The era view, when the calendar has one.
    pub era: Option<EraNumber>,
    /// How the date was resolved.
    pub resolution: CalendarResolution,
}

impl CalendarDate {
    /// A date in a calendar that is a mathematical definition.
    #[must_use]
    pub const fn defined(calendar: Calendar, year: i32, month: u8, day: u8) -> CalendarDate {
        CalendarDate {
            calendar,
            year,
            month,
            day,
            era: None,
            resolution: CalendarResolution::Defined,
        }
    }

    /// The same date with an era attached.
    #[must_use]
    pub const fn with_era(mut self, era: Era, year: i32) -> CalendarDate {
        self.era = Some(EraNumber { era, year });
        self
    }
}

impl fmt::Display for CalendarDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}-{:02}-{:02}",
            self.calendar.key(),
            self.year,
            self.month,
            self.day
        )
    }
}

/// What a calendar declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarCapabilities {
    /// The first and last fixed day the calendar converts.
    pub range: (FixedDay, FixedDay),
    /// Whether a place is needed (sunrise-anchored calendars).
    pub needs_place: bool,
    /// Whether the ephemeris port is needed (lunisolar calendars).
    pub needs_ephemeris: bool,
    /// The eras a date may carry.
    pub eras: &'static [Era],
}

/// A calendar: conversions to and from the fixed day, month lengths and
/// leap years.
pub trait CalendarSystem: Send + Sync {
    /// The calendar's key.
    fn id(&self) -> Calendar;

    /// What the calendar declares.
    fn capabilities(&self) -> CalendarCapabilities;

    /// The date a fixed day falls on.
    ///
    /// # Errors
    ///
    /// `OUT_OF_RANGE` beyond the calendar's coverage.
    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error>;

    /// The fixed day of a date, validated.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` with `NONEXISTENT_DATE` for a date the calendar does
    /// not have; `OUT_OF_RANGE` beyond the coverage.
    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error>;

    /// The length of a month.
    ///
    /// # Errors
    ///
    /// A month the calendar does not have, or a year outside its coverage.
    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error>;

    /// Whether a year is a leap year.
    fn is_leap(&self, year: i32) -> bool;

    /// The fixed day of a year, month and day in this calendar.
    ///
    /// # Errors
    ///
    /// As [`CalendarSystem::fixed_of`].
    fn to_fixed_ymd(&self, year: i32, month: u8, day: u8) -> Result<FixedDay, Error> {
        self.fixed_of(&CalendarDate::defined(self.id(), year, month, day))
    }

    /// The weekday of a date.
    ///
    /// # Errors
    ///
    /// As [`CalendarSystem::fixed_of`].
    fn weekday(&self, date: &CalendarDate) -> Result<Weekday, Error> {
        self.fixed_of(date).map(FixedDay::weekday)
    }

    /// Converts a date into another calendar.
    ///
    /// # Errors
    ///
    /// Either calendar's refusal.
    fn convert(
        &self,
        date: &CalendarDate,
        into: &dyn CalendarSystem,
    ) -> Result<CalendarDate, Error> {
        into.date_of(self.fixed_of(date)?)
    }
}

/// `INVALID_ARG` with `NONEXISTENT_DATE`.
pub(crate) fn nonexistent(message: String) -> Error {
    Error::invalid_arg(message).with_detail(Detail::NonexistentDate)
}

/// `OUT_OF_RANGE`.
pub(crate) fn out_of_range(message: String) -> Error {
    Error::new(Status::OutOfRange, message)
}

/// Checks a month and a day against a month length.
pub(crate) fn check_day(
    calendar: Calendar,
    year: i32,
    month: u8,
    day: u8,
    months: u8,
    length: u8,
) -> Result<(), Error> {
    if month == 0 || month > months {
        return Err(nonexistent(format!(
            "{} has months 1 to {months}, not {month}",
            calendar.key()
        ))
        .with_field("month"));
    }
    if day == 0 || day > length {
        return Err(nonexistent(format!(
            "{} {year}-{month:02} has days 1 to {length}, not {day}",
            calendar.key()
        ))
        .with_field("day"));
    }
    Ok(())
}

/// The shipped calendar with a key, or `None` for one that needs a
/// context (the lunisolar calendar takes the ephemeris port).
#[must_use]
pub fn shipped(id: Calendar) -> Option<&'static dyn CalendarSystem> {
    match id {
        Calendar::Gregorian => Some(&crate::gregorian::Gregorian),
        Calendar::Julian => Some(&crate::julian::Julian),
        Calendar::Mixed => Some(&crate::mixed::GREGORIAN_1582),
        Calendar::IsoWeek => Some(&crate::iso_week::IsoWeek),
        Calendar::BikramSambat => Some(crate::bikram_sambat::BikramSambat::shipped()),
        _ => None,
    }
}

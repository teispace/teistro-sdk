//! The areas: what a consumer reads operations off.
//!
//! An area is a **borrowing view** of a context, which is the Rust
//! equivalent of what it is in every other binding — a frozen instance
//! in Node, a `late final` field in Dart, a `cached_property` in Python.
//! It allocates nothing, it cannot outlive the context it came from, and
//! `sdk.calendar()` costs a pointer copy, so a consumer who wants to
//! keep one writes `let cal = sdk.calendar();` exactly as a Node
//! consumer writes `const cal = ctx.calendar`.
//!
//! The operations are the ones `03-design/surface-areas.md` puts in each
//! area and no others. That is not tidiness: `check-areas` holds every
//! binding's list to the same canonical paths, so an operation invented
//! here would be an operation the other three lack.

use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay, Weekday, shipped};
use teistro_core::catalogue::Calendar;
use teistro_core::error::Error;

use crate::context::Context;

/// The calendar the SDK ships for an id, or the refusal that says why it
/// does not.
///
/// One reading, because six operations need it and each would otherwise
/// spell the refusal itself.
fn system_of(id: Calendar) -> Result<&'static dyn CalendarSystem, Error> {
    shipped(id).ok_or_else(|| {
        Error::unsupported(format!("the SDK does not ship the `{}` calendar", id.key()))
            .with_field("calendar")
    })
}

/// `sdk.calendar`: dates in the calendars the SDK carries — Bikram
/// Sambat, Gregorian, Julian, the 1582 mixed calendar and the ISO week
/// date — and the arithmetic over them.
#[derive(Clone, Copy, Debug)]
pub struct CalendarArea<'a> {
    context: &'a Context,
}

impl<'a> CalendarArea<'a> {
    pub(crate) fn of(context: &'a Context) -> CalendarArea<'a> {
        CalendarArea { context }
    }

    /// The context this area was read off, so a consumer who held the
    /// area still has everything.
    #[must_use]
    pub fn context(&self) -> &'a Context {
        self.context
    }

    /// The date a fixed day is, in a calendar.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship, or one that refuses the day.
    pub fn date_of(&self, calendar: Calendar, fixed: FixedDay) -> Result<CalendarDate, Error> {
        system_of(calendar)?.date_of(fixed)
    }

    /// The fixed day a date is.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship, or a date it refuses.
    pub fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        system_of(date.calendar)?.fixed_of(date)
    }

    /// The same day in another calendar.
    ///
    /// # Errors
    ///
    /// Either calendar's refusal.
    pub fn convert(&self, date: &CalendarDate, into: Calendar) -> Result<CalendarDate, Error> {
        system_of(date.calendar)?.convert(date, system_of(into)?)
    }

    /// The weekday a date falls on.
    ///
    /// # Errors
    ///
    /// As [`CalendarArea::fixed_of`].
    pub fn weekday_of(&self, date: &CalendarDate) -> Result<Weekday, Error> {
        system_of(date.calendar)?.weekday(date)
    }

    /// How many days a month has.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship, or a month it refuses.
    pub fn month_length(&self, calendar: Calendar, year: i32, month: u8) -> Result<u8, Error> {
        system_of(calendar)?.month_length(year, month)
    }

    /// Whether a year is a leap year.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship.
    pub fn is_leap(&self, calendar: Calendar, year: i32) -> Result<bool, Error> {
        Ok(system_of(calendar)?.is_leap(year))
    }
}

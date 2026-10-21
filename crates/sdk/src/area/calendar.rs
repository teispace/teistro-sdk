//! `sdk.calendar`: dates in the calendars the SDK carries, and the
//! arithmetic over them.

use teistro_calendar::tibetan::{TibetanYear, year_name};
use teistro_calendar::{CalendarDate, FixedDay, Weekday};
use teistro_core::catalogue::{Calendar, write_in_full};
use teistro_core::error::Error;

use crate::area::system_of;
use crate::context::Context;

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
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// The date a fixed day is, in a calendar.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship, or one that refuses the day.
    pub fn date_of(self, calendar: Calendar, fixed: FixedDay) -> Result<CalendarDate, Error> {
        system_of(calendar)?.date_of(fixed)
    }

    /// The fixed day a date is.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship, or a date it refuses.
    pub fn fixed_of(self, date: &CalendarDate) -> Result<FixedDay, Error> {
        system_of(date.calendar)?.fixed_of(date)
    }

    /// The same day in another calendar.
    ///
    /// # Errors
    ///
    /// Either calendar's refusal.
    pub fn convert(self, date: &CalendarDate, into: Calendar) -> Result<CalendarDate, Error> {
        system_of(date.calendar)?.convert(date, system_of(into)?)
    }

    /// The weekday a date falls on.
    ///
    /// # Errors
    ///
    /// As [`CalendarArea::fixed_of`].
    pub fn weekday_of(self, date: &CalendarDate) -> Result<Weekday, Error> {
        system_of(date.calendar)?.weekday(date)
    }

    /// How many days a month has.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship, or a month it refuses.
    pub fn month_length(self, calendar: Calendar, year: i32, month: u8) -> Result<u8, Error> {
        system_of(calendar)?.month_length(year, month)
    }

    /// Whether a year is a leap year.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship.
    pub fn is_leap(self, calendar: Calendar, year: i32) -> Result<bool, Error> {
        Ok(system_of(calendar)?.is_leap(year))
    }

    /// A Tibetan year's names (`calendars/tibetan.md`): its rabjung cycle
    /// and place in it with the Indian cycle's name, and the element,
    /// gender and animal it is called by, "Fire–male–Horse" for 2026. The
    /// year is the one a `TIBETAN_PHUGPA` date carries: the Gregorian year
    /// in which the Tibetan year begins.
    ///
    /// ```
    /// use teistro::Context;
    /// use teistro::catalogue::{Samvatsara, TibetanAnimal};
    ///
    /// let sdk = Context::builder().build()?;
    /// let year = sdk.calendar().tibetan_year(2026)?;
    /// assert_eq!((year.rabjung, year.year_in_rabjung), (17, 40));
    /// assert_eq!((year.samvatsara, year.animal), (Samvatsara::Parabhava, TibetanAnimal::Horse));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `OUT_OF_RANGE` for a year outside 1027 to 9998, named `year`.
    pub fn tibetan_year(self, year: i32) -> Result<TibetanYear, Error> {
        year_name(year)
    }

    /// As [`CalendarArea::tibetan_year`], every catalogue member written as
    /// its full key: what the boundary carries.
    ///
    /// # Errors
    ///
    /// As [`CalendarArea::tibetan_year`].
    pub fn tibetan_year_in_full(self, year: i32) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(year_name(year)?)
            .map_err(|why| Error::internal(format!("the Tibetan year did not serialise: {why}")))?;
        for (path, kind) in TibetanYear::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        Ok(value)
    }
}

//! `sdk.calendar`: dates in the calendars the SDK carries, and the
//! arithmetic over them.

use teistro_calendar::arithmetic::{self, DateSpan, Overflow};
use teistro_calendar::kirat_yele;
use teistro_calendar::tibetan::{TibetanYear, year_name};
use teistro_calendar::{CalendarDate, FixedDay, Weekday};
use teistro_core::catalogue::{Calendar, Era, write_in_full};
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

    /// The date viewed in a named era: its own calendar's reading, with
    /// the year that era gives it.
    ///
    /// An era the date's calendar attaches itself (`CalendarCapabilities::eras`)
    /// is the calendar's own reading of the day, as [`CalendarArea::convert`]
    /// into the same calendar gives it, when that reading is in the era
    /// asked: a Gregorian date before year 1 reads in `BEFORE_COMMON_ERA`
    /// and is refused `COMMON_ERA`. `KIRAT_YELE` views a Bikram Sambat date
    /// in the Kirat Yele Sambat (`calendars/bikram-sambat.md` §Kirat Yele
    /// Sambat): the same year, month and day, with the Yele year counted
    /// from the epoch `calendars.kirat_yele` names, turning on Magh 1.
    ///
    /// ```
    /// use teistro::{CalendarDate, Context};
    /// use teistro::catalogue::{Calendar, Era};
    ///
    /// let sdk = Context::builder().build()?;
    /// // 15 January 2026 was Magh 1, 2082 BS: Kirati Sambat 5086 began.
    /// let magh = sdk.calendar().convert(&CalendarDate::defined(Calendar::Gregorian, 2026, 1, 15), Calendar::BikramSambat)?;
    /// let yele = sdk.calendar().in_era(&magh, Era::KiratYele)?;
    /// assert_eq!((yele.year, yele.month, yele.day), (2082, 10, 1));
    /// assert_eq!(yele.era.map(|era| (era.era, era.year)), Some((Era::KiratYele, 5086)));
    /// // Under Regmi's epoch from Yalambar's accession, the same day opened 3805.
    /// let regmi = Context::builder().settings_json(r#"{"calendars": {"kirat_yele": "YALAMBAR"}}"#).build()?;
    /// assert_eq!(regmi.calendar().in_era(&magh, Era::KiratYele)?.era.map(|era| era.year), Some(3805));
    /// // An era the Bikram Sambat neither attaches nor is viewed in is refused by its field.
    /// assert_eq!(sdk.calendar().in_era(&magh, Era::Hijri).unwrap_err().field(), Some("era"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship, or a date it refuses, as
    /// [`CalendarArea::fixed_of`]; `UNSUPPORTED` on `era` for an era the
    /// date's calendar cannot give it, with a hint naming those it can.
    pub fn in_era(self, date: &CalendarDate, era: Era) -> Result<CalendarDate, Error> {
        let system = system_of(date.calendar)?;
        let reading = system.date_of(system.fixed_of(date)?)?;
        if era == Era::KiratYele && date.calendar == Calendar::BikramSambat {
            return kirat_yele::view(&reading, self.context.settings().calendars.kirat_yele);
        }
        if reading.era.is_some_and(|number| number.era == era) {
            return Ok(reading);
        }
        let eras = system.capabilities().eras;
        let why = match reading.era {
            Some(own) if eras.contains(&era) => format!(
                "{reading} reads in {}, not {}",
                own.era.full_key(),
                era.full_key()
            ),
            _ => format!(
                "the {} calendar gives no year in {}",
                date.calendar.key(),
                era.full_key()
            ),
        };
        let hint = if eras.is_empty() {
            format!(
                "the {} calendar counts its years in no era",
                date.calendar.key()
            )
        } else {
            format!(
                "the {} calendar gives a date in {}",
                date.calendar.key(),
                eras.iter()
                    .copied()
                    .map(Era::full_key)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        Err(Error::unsupported(why).with_field("era").with_hint(hint))
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

    /// The date `span` after `date`, in its own calendar
    /// (`03-design/calendar-bikram-sambat.md` §11): the years and months
    /// first, a day number the month lacks resolved by `overflow`, then
    /// the days. A Bikram Sambat month runs 29 to 32 days, so the 32nd of
    /// Jestha plus a month is the 31st of a 31-day Ashadh under
    /// [`Overflow::Constrain`], the 1st of Shrawan under
    /// [`Overflow::Carry`], and refused under [`Overflow::Reject`].
    ///
    /// ```
    /// use teistro::{CalendarDate, Context, DateSpan, Overflow};
    /// use teistro::catalogue::Calendar;
    ///
    /// let sdk = Context::builder().build()?;
    /// let jestha_32 = CalendarDate::defined(Calendar::BikramSambat, 2081, 2, 32);
    /// let on = sdk.calendar().add(&jestha_32, DateSpan::months(1), Overflow::Constrain)?;
    /// assert_eq!((on.month, on.day), (3, 31));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship; `UNSUPPORTED` for the ISO week
    /// date and the Tibetan calendar, which number no twelve months;
    /// `INVALID_ARG` for a date its calendar refuses, and with
    /// `NONEXISTENT_DATE` on `day` under [`Overflow::Reject`]; and
    /// `OUT_OF_RANGE` past the calendar's span.
    pub fn add(
        self,
        date: &CalendarDate,
        span: DateSpan,
        overflow: Overflow,
    ) -> Result<CalendarDate, Error> {
        arithmetic::add(system_of(date.calendar)?, date, span, overflow)
    }

    /// The years, months and days from `from` to `to`, both in one
    /// calendar, counted as an age is (`03-design/calendar-bikram-sambat.md`
    /// §11): the most whole months whose anniversary of `from` is not past
    /// `to`, then the days from it, so [`CalendarArea::add`] of the answer
    /// to `from` under the same `overflow` is `to`. An anniversary on a
    /// day its month lacks is resolved by `overflow`: one born on the 32nd
    /// of Jestha completes a year on the 31st of a 31-day Jestha under
    /// [`Overflow::Constrain`] (C806). Before `from`, each part is negative
    /// or zero.
    ///
    /// ```
    /// use teistro::{CalendarDate, Context, DateSpan, Overflow};
    /// use teistro::catalogue::Calendar;
    ///
    /// let sdk = Context::builder().build()?;
    /// let born = CalendarDate::defined(Calendar::BikramSambat, 2050, 2, 32);
    /// let birthday = CalendarDate::defined(Calendar::BikramSambat, 2082, 2, 31);
    /// let age = sdk.calendar().between(&born, &birthday, Overflow::Constrain)?;
    /// // Jestha 2082 has 31 days: the birthday is its last.
    /// assert_eq!(age, DateSpan::years(32));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`CalendarArea::add`]; `INVALID_ARG` for dates in two calendars,
    /// and under [`Overflow::Reject`] for a count from a day the month
    /// lacks.
    pub fn between(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        overflow: Overflow,
    ) -> Result<DateSpan, Error> {
        arithmetic::between(system_of(from.calendar)?, from, to, overflow)
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

//! A date in any calendar, the trait every calendar implements, and the
//! shipped calendars by key.

use core::fmt;

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::envelope::CalendarResolution;
use teistro_core::error::{Detail, Error, Status};

use crate::fixed::{FixedDay, Weekday};

/// A year in an era, for presentation.
///
/// ```
/// use teistro_calendar::{BikramSambat, CalendarSystem, EraNumber};
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_core::catalogue::Era;
///
/// let date = BikramSambat::shipped().date_of(fixed_from_gregorian(2024, 1, 1))?;
/// assert_eq!(date.era, Some(EraNumber { era: Era::Vikrama, year: 2080 }));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EraNumber {
    /// The era.
    pub era: Era,
    /// The year in it, 1-based.
    pub year: i32,
}

/// A date in a calendar. Years are astronomical (1 BCE is 0); an era is
/// attached for presentation only.
///
/// ```
/// use teistro_calendar::{CalendarDate, CalendarSystem, Gregorian};
/// use teistro_core::catalogue::Calendar;
///
/// let new_year = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
///
/// assert_eq!(Gregorian.weekday(&new_year)?.name(), "Monday");
/// assert_eq!(new_year.to_string(), "GREGORIAN 2024-01-01");
/// // Years are astronomical: 1 BCE is year 0.
/// let bce = Gregorian.date_of(Gregorian.to_fixed_ymd(0, 1, 1)?)?;
/// assert_eq!(bce.era.map(|era| era.year), Some(1));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// Whether the month or the day is the first of two bearing the same
    /// number, in a calendar that repeats them (the Tibetan); left out of
    /// the wire when absent, and refused by every calendar that repeats
    /// neither.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leap: Option<Leap>,
}

/// A day in any calendar as a request writes it: the date without how
/// it was resolved, which only an answer can say.
///
/// ```
/// use teistro_calendar::{CalendarDate, CalendarDay};
/// use teistro_core::catalogue::Calendar;
///
/// let day: CalendarDay = serde_json::from_str(
///     r#"{"calendar": "calendar.BIKRAM_SAMBAT", "year": 2081, "month": 1, "day": 1}"#,
/// )?;
/// assert_eq!(day.date(), CalendarDate::defined(Calendar::BikramSambat, 2081, 1, 1));
/// assert_eq!(CalendarDay::from(&day.date()), day);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendarDay {
    /// Its calendar.
    pub calendar: Calendar,
    /// The astronomical year.
    pub year: i32,
    /// The month, 1-based.
    pub month: u8,
    /// The day, 1-based.
    pub day: u8,
    /// The era, for a calendar that counts several.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub era: Option<EraNumber>,
    /// A leap month or day, for a calendar that marks one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leap: Option<Leap>,
}

impl CalendarDay {
    /// The day as the calendar defines it, resolved by definition.
    ///
    /// ```
    /// use teistro_calendar::{CalendarDay, Leap};
    /// use teistro_core::catalogue::Calendar;
    ///
    /// let day = CalendarDay {
    ///     calendar: Calendar::TibetanPhugpa,
    ///     year: 2024,
    ///     month: 6,
    ///     day: 1,
    ///     era: None,
    ///     leap: Some(Leap::Month),
    /// };
    /// assert_eq!(day.date().leap, Some(Leap::Month));
    /// ```
    #[must_use]
    pub fn date(&self) -> CalendarDate {
        let mut date = CalendarDate::defined(self.calendar, self.year, self.month, self.day);
        date.era = self.era;
        date.leap = self.leap;
        date
    }
}

impl From<&CalendarDate> for CalendarDay {
    fn from(date: &CalendarDate) -> CalendarDay {
        CalendarDay {
            calendar: date.calendar,
            year: date.year,
            month: date.month,
            day: date.day,
            era: date.era,
            leap: date.leap,
        }
    }
}

/// Which part of a date is the first of a repeated pair: the leap month
/// comes before the month whose number it takes, and the leap day before
/// the day whose number it repeats.
///
/// ```
/// use teistro_calendar::tibetan::{TibetanPhugpa, has_leap_month};
/// use teistro_calendar::{CalendarDate, CalendarSystem, Leap};
/// use teistro_core::catalogue::Calendar;
///
/// // 2024's sixth Tibetan month was doubled; the leap month comes first.
/// assert!(has_leap_month(2024, 6));
/// let regular = CalendarDate::defined(Calendar::TibetanPhugpa, 2024, 6, 1);
/// let leap = regular.clone().with_leap(Some(Leap::Month));
/// let days = TibetanPhugpa.fixed_of(&leap)?.days_until(TibetanPhugpa.fixed_of(&regular)?);
/// assert!((29..=30).contains(&days), "{days}");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Leap {
    /// The leap month.
    Month,
    /// The leap day.
    Day,
    /// The leap day of the leap month.
    MonthAndDay,
}

impl Leap {
    /// The mark of a month and a day, `None` when neither is a leap.
    ///
    /// ```
    /// use teistro_calendar::Leap;
    ///
    /// assert_eq!(Leap::of(false, false), None);
    /// assert_eq!(Leap::of(true, true), Some(Leap::MonthAndDay));
    /// ```
    #[must_use]
    pub const fn of(month: bool, day: bool) -> Option<Leap> {
        match (month, day) {
            (false, false) => None,
            (true, false) => Some(Leap::Month),
            (false, true) => Some(Leap::Day),
            (true, true) => Some(Leap::MonthAndDay),
        }
    }

    /// Whether the month is the leap month.
    ///
    /// ```
    /// use teistro_calendar::Leap;
    ///
    /// assert!(Leap::MonthAndDay.month() && !Leap::Day.month());
    /// ```
    #[must_use]
    pub const fn month(self) -> bool {
        matches!(self, Leap::Month | Leap::MonthAndDay)
    }

    /// Whether the day is the leap day.
    ///
    /// ```
    /// use teistro_calendar::Leap;
    ///
    /// assert!(Leap::MonthAndDay.day() && !Leap::Month.day());
    /// ```
    #[must_use]
    pub const fn day(self) -> bool {
        matches!(self, Leap::Day | Leap::MonthAndDay)
    }

    /// The mark's key, as the wire spells it.
    ///
    /// ```
    /// use teistro_calendar::Leap;
    ///
    /// assert_eq!(Leap::MonthAndDay.key(), "MONTH_AND_DAY");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Leap::Month => "MONTH",
            Leap::Day => "DAY",
            Leap::MonthAndDay => "MONTH_AND_DAY",
        }
    }
}

impl CalendarDate {
    /// A date in a calendar that is a mathematical definition.
    ///
    /// ```
    /// use teistro_calendar::{CalendarDate, CalendarSystem, Gregorian};
    /// use teistro_core::catalogue::Calendar;
    ///
    /// let new_year = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
    ///
    /// assert_eq!((new_year.year, new_year.month, new_year.day), (2024, 1, 1));
    /// assert_eq!((new_year.era, new_year.leap), (None, None));
    /// assert_eq!(Gregorian.fixed_of(&new_year)?.get(), 738_886);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn defined(calendar: Calendar, year: i32, month: u8, day: u8) -> CalendarDate {
        CalendarDate {
            calendar,
            year,
            month,
            day,
            era: None,
            resolution: CalendarResolution::Defined,
            leap: None,
        }
    }

    /// The same date with its leap mark.
    ///
    /// ```
    /// use teistro_calendar::{CalendarDate, CalendarSystem, Gregorian};
    /// use teistro_core::catalogue::Calendar;
    ///
    /// let new_year = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
    /// use teistro_calendar::Leap;
    ///
    /// // A calendar that repeats no month refuses a leap mark.
    /// assert!(Gregorian.fixed_of(&new_year.with_leap(Some(Leap::Month))).is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn with_leap(mut self, leap: Option<Leap>) -> CalendarDate {
        self.leap = leap;
        self
    }

    /// The same date with an era attached.
    ///
    /// ```
    /// use teistro_calendar::{CalendarDate, EraNumber};
    /// use teistro_core::catalogue::{Calendar, Era};
    ///
    /// let date = CalendarDate::defined(Calendar::Gregorian, 0, 1, 1).with_era(Era::BeforeCommonEra, 1);
    /// assert_eq!(date.era, Some(EraNumber { era: Era::BeforeCommonEra, year: 1 }));
    /// ```
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
///
/// ```
/// use teistro_calendar::{BikramSambat, CalendarSystem, Gregorian};
/// use teistro_core::catalogue::Era;
///
/// let gregorian = Gregorian.capabilities();
/// assert!(!gregorian.needs_place && !gregorian.needs_ephemeris);
/// assert_eq!(gregorian.eras, [Era::CommonEra, Era::BeforeCommonEra]);
/// assert_eq!(BikramSambat::shipped().capabilities().eras, [Era::Vikrama]);
/// ```
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
///
/// ```
/// use teistro_calendar::{BikramSambat, CalendarDate, CalendarSystem, Gregorian};
/// use teistro_core::catalogue::Calendar;
///
/// // Every calendar meets the others at the fixed day.
/// let ad = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
/// let bs = Gregorian.convert(&ad, BikramSambat::shipped())?;
/// assert_eq!((bs.year, bs.month, bs.day), (2080, 9, 16));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub trait CalendarSystem: Send + Sync {
    /// The calendar's key.
    ///
    /// ```
    /// use teistro_calendar::{CalendarSystem, Gregorian};
    /// use teistro_core::catalogue::Calendar;
    ///
    /// assert_eq!(Gregorian.id(), Calendar::Gregorian);
    /// ```
    fn id(&self) -> Calendar;

    /// What the calendar declares.
    ///
    /// ```
    /// use teistro_calendar::{CalendarSystem, Gregorian, Julian};
    ///
    /// // The years -9999 to 9999.
    /// let (first, last) = Gregorian.capabilities().range;
    /// assert_eq!((first, last), (Gregorian.to_fixed_ymd(-9999, 1, 1)?, Gregorian.to_fixed_ymd(9999, 12, 31)?));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    fn capabilities(&self) -> CalendarCapabilities;

    /// The date a fixed day falls on.
    ///
    /// ```
    /// use teistro_calendar::{CalendarSystem, FixedDay, Gregorian};
    ///
    /// let date = Gregorian.date_of(FixedDay::UNIX_EPOCH)?;
    /// assert_eq!((date.year, date.month, date.day), (1970, 1, 1));
    /// assert!(Gregorian.date_of(FixedDay::new(i64::MAX / 2)).is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `OUT_OF_RANGE` beyond the calendar's coverage.
    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error>;

    /// The fixed day of a date, validated.
    ///
    /// ```
    /// use teistro_calendar::{CalendarDate, CalendarSystem, Gregorian};
    /// use teistro_core::catalogue::Calendar;
    ///
    /// let new_year = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
    /// use teistro_core::error::{Detail, Status};
    ///
    /// assert_eq!(Gregorian.fixed_of(&new_year)?.get(), 738_886);
    /// let refused = Gregorian.fixed_of(&CalendarDate::defined(Calendar::Gregorian, 2023, 2, 29)).unwrap_err();
    /// assert_eq!((refused.status, refused.detail), (Status::InvalidArg, Some(Detail::NonexistentDate)));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` with `NONEXISTENT_DATE` for a date the calendar does
    /// not have; `OUT_OF_RANGE` beyond the coverage.
    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error>;

    /// The length of a month.
    ///
    /// ```
    /// use teistro_calendar::{CalendarSystem, Gregorian};
    ///
    /// assert_eq!(Gregorian.month_length(2024, 2)?, 29);
    /// assert!(Gregorian.month_length(2024, 13).is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A month the calendar does not have, or a year outside its coverage.
    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error>;

    /// Whether a year is a leap year.
    ///
    /// ```
    /// use teistro_calendar::{CalendarSystem, Gregorian, Julian};
    ///
    /// assert!(Julian.is_leap(1900) && !Gregorian.is_leap(1900));
    /// ```
    fn is_leap(&self, year: i32) -> bool;

    /// The fixed day of a year, month and day in this calendar.
    ///
    /// ```
    /// use teistro_calendar::{CalendarSystem, Gregorian};
    /// use teistro_calendar::gregorian::fixed_from_gregorian;
    ///
    /// assert_eq!(Gregorian.to_fixed_ymd(2024, 1, 1)?, fixed_from_gregorian(2024, 1, 1));
    /// assert!(Gregorian.to_fixed_ymd(2024, 4, 31).is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`CalendarSystem::fixed_of`].
    fn to_fixed_ymd(&self, year: i32, month: u8, day: u8) -> Result<FixedDay, Error> {
        self.fixed_of(&CalendarDate::defined(self.id(), year, month, day))
    }

    /// The weekday of a date.
    ///
    /// ```
    /// use teistro_calendar::{CalendarDate, CalendarSystem, Gregorian};
    /// use teistro_core::catalogue::Calendar;
    ///
    /// let new_year = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
    /// use teistro_calendar::Weekday;
    ///
    /// assert_eq!(Gregorian.weekday(&new_year)?, Weekday::Monday);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`CalendarSystem::fixed_of`].
    fn weekday(&self, date: &CalendarDate) -> Result<Weekday, Error> {
        self.fixed_of(date).map(FixedDay::weekday)
    }

    /// Converts a date into another calendar.
    ///
    /// ```
    /// use teistro_calendar::{CalendarDate, CalendarSystem, Gregorian};
    /// use teistro_core::catalogue::Calendar;
    ///
    /// let new_year = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 1);
    /// use teistro_calendar::Julian;
    ///
    /// let julian = Gregorian.convert(&new_year, &Julian)?;
    /// assert_eq!((julian.calendar, julian.year, julian.month, julian.day), (Calendar::Julian, 2023, 12, 19));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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

/// Refuses a leap mark on a date of a calendar that repeats neither a
/// month nor a day.
pub(crate) fn check_no_leap(date: &CalendarDate) -> Result<(), Error> {
    match date.leap {
        None => Ok(()),
        Some(leap) => Err(nonexistent(format!(
            "{} repeats neither a month nor a day, so a date cannot be marked {}",
            date.calendar.key(),
            leap.key()
        ))
        .with_field("leap")),
    }
}

/// `OUT_OF_RANGE`.
pub(crate) fn out_of_range(message: String) -> Error {
    Error::new(Status::OutOfRange, message)
}

/// Refuses a fixed day outside a calendar's range before its conversion
/// meets it: a day far enough out overflows the year the arithmetic would
/// compute, so the check on that year comes too late.
pub(crate) fn check_fixed(
    calendar: Calendar,
    (first, last): (FixedDay, FixedDay),
    fixed: FixedDay,
) -> Result<(), Error> {
    if fixed.get() < first.get() || fixed.get() > last.get() {
        return Err(out_of_range(format!(
            "{} covers fixed days {} to {}, not {}",
            calendar.key(),
            first.get(),
            last.get(),
            fixed.get()
        ))
        .with_field("fixed"));
    }
    Ok(())
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
///
/// ```
/// use teistro_calendar::shipped;
/// use teistro_core::catalogue::Calendar;
///
/// assert_eq!(shipped(Calendar::BikramSambat).map(|calendar| calendar.id()), Some(Calendar::BikramSambat));
/// // The lunisolar calendar needs an ephemeris, so a context builds it.
/// assert!(shipped(Calendar::IndianLunisolar).is_none());
/// ```
#[must_use]
pub fn shipped(id: Calendar) -> Option<&'static dyn CalendarSystem> {
    match id {
        Calendar::Gregorian => Some(&crate::gregorian::Gregorian),
        Calendar::Julian => Some(&crate::julian::Julian),
        Calendar::Mixed => Some(&crate::mixed::GREGORIAN_1582),
        Calendar::IsoWeek => Some(&crate::iso_week::IsoWeek),
        Calendar::BikramSambat => Some(crate::bikram_sambat::BikramSambat::shipped()),
        Calendar::IndianNational => Some(&crate::indian_national::IndianNational),
        Calendar::HijriTabular => Some(&crate::hijri::HijriTabular),
        Calendar::TibetanPhugpa => Some(&crate::tibetan::TibetanPhugpa),
        Calendar::Bangladeshi => Some(&crate::bangladeshi::Bangladeshi),
        Calendar::TamilSolar
        | Calendar::BengalSolar
        | Calendar::OdiaSolar
        | Calendar::MalayalamSolar => crate::regional_solar::RegionalSolar::shipped(id)
            .map(|calendar| calendar as &dyn CalendarSystem),
        _ => None,
    }
}

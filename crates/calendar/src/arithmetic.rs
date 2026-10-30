//! Adding years, months and days to a date, and the difference of two
//! dates in years, months and days, in any calendar of twelve numbered
//! months (`docs/03-design/calendar-bikram-sambat.md` §11, SR-05).
//!
//! A month is the calendar's own month, as Nepal's Interpretation of Laws
//! Act defines one for Bikram Sambat: "a month reckoned from Sankranti
//! according to the Nepalese Panchanga" (§2(n)), and a year likewise from
//! Baisakh's sankranti (§2(ac)). So a Bikram Sambat month is 29 to 32
//! days long, and a day number a target month lacks is resolved by an
//! [`Overflow`] the caller names, constraining it to the month's last day
//! by default.
//!
//! ```
//! use teistro_calendar::arithmetic::{DateSpan, Overflow, add, between};
//! use teistro_calendar::{BikramSambat, CalendarDate};
//! use teistro_core::catalogue::Calendar;
//!
//! let bs = BikramSambat::shipped();
//! // Jestha 2081 has 32 days and Ashadh 2081 has 31.
//! let last = CalendarDate::defined(Calendar::BikramSambat, 2081, 2, 32);
//! let month_on = add(bs, &last, DateSpan::months(1), Overflow::Constrain)?;
//! assert_eq!((month_on.month, month_on.day), (3, 31));
//! // The difference counts the constrained day as the month's anniversary.
//! assert_eq!(between(bs, &last, &month_on, Overflow::Constrain)?, DateSpan::months(1));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use teistro_core::catalogue::Calendar;
use teistro_core::error::{Detail, Error};

use crate::date::{CalendarDate, CalendarSystem, nonexistent, out_of_range};
use crate::fixed::FixedDay;

/// The months of a year in every calendar the arithmetic takes.
const MONTHS: i64 = 12;

/// What a day number the target month lacks resolves to: a Bikram Sambat
/// month runs 29 to 32 days, so the 32nd of Jestha plus a month may name
/// a day Ashadh does not have. The names are Temporal's `overflow`
/// option's (`constrain`, `reject`), with java.time's lenient resolution
/// as the third, called for what it does.
///
/// ```
/// use teistro_calendar::arithmetic::Overflow;
///
/// assert_eq!(Overflow::default(), Overflow::Constrain);
/// assert_eq!(serde_json::to_string(&Overflow::Carry)?, r#""CARRY""#);
/// assert_eq!(Overflow::Reject.key(), "REJECT");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Overflow {
    /// The month's last day: the 32nd of Jestha plus a month is the 31st
    /// of a 31-day Ashadh. Temporal's `constrain`, java.time's smart
    /// resolution of a day of month.
    #[default]
    Constrain,
    /// The days past the month's end carried into the next month: the
    /// 32nd of a 31-day Ashadh is the 1st of Shrawan. java.time's lenient
    /// resolution.
    Carry,
    /// Refused, `INVALID_ARG` with `NONEXISTENT_DATE` naming `day`.
    /// Temporal's `reject`, java.time's strict resolution.
    Reject,
}

impl Overflow {
    /// Every rule, in the order of their numbers at the C boundary.
    pub const ALL: [Overflow; 3] = [Overflow::Constrain, Overflow::Carry, Overflow::Reject];

    /// Its key, as a request names it.
    ///
    /// ```
    /// use teistro_calendar::arithmetic::Overflow;
    ///
    /// assert_eq!(Overflow::ALL.map(Overflow::key), ["CONSTRAIN", "CARRY", "REJECT"]);
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Overflow::Constrain => "CONSTRAIN",
            Overflow::Carry => "CARRY",
            Overflow::Reject => "REJECT",
        }
    }
}

/// Years, months and days, each signed: what [`add`] adds and [`between`]
/// answers. The years and months are added first, as one count of
/// months, and the days after them.
///
/// ```
/// use teistro_calendar::arithmetic::DateSpan;
///
/// let span = DateSpan::new(30, 0, 1);
/// assert_eq!((span.years, span.months, span.days), (30, 0, 1));
/// assert_eq!(DateSpan::months(14).total_months(), 14);
/// assert_eq!(DateSpan::new(1, 2, 0).total_months(), 14);
/// assert_eq!(serde_json::to_string(&DateSpan::days(-3))?, r#"{"years":0,"months":0,"days":-3}"#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct DateSpan {
    /// Whole years.
    #[serde(default)]
    pub years: i32,
    /// Whole months.
    #[serde(default)]
    pub months: i32,
    /// Days.
    #[serde(default)]
    pub days: i32,
}

impl DateSpan {
    /// A span of years, months and days.
    ///
    /// ```
    /// use teistro_calendar::arithmetic::DateSpan;
    ///
    /// assert_eq!(DateSpan::new(1, 0, 0), DateSpan::years(1));
    /// ```
    #[must_use]
    pub const fn new(years: i32, months: i32, days: i32) -> DateSpan {
        DateSpan {
            years,
            months,
            days,
        }
    }

    /// Whole years.
    ///
    /// ```
    /// use teistro_calendar::arithmetic::DateSpan;
    ///
    /// assert_eq!(DateSpan::years(2).years, 2);
    /// ```
    #[must_use]
    pub const fn years(years: i32) -> DateSpan {
        DateSpan::new(years, 0, 0)
    }

    /// Whole months.
    ///
    /// ```
    /// use teistro_calendar::arithmetic::DateSpan;
    ///
    /// assert_eq!(DateSpan::months(-1).months, -1);
    /// ```
    #[must_use]
    pub const fn months(months: i32) -> DateSpan {
        DateSpan::new(0, months, 0)
    }

    /// Days.
    ///
    /// ```
    /// use teistro_calendar::arithmetic::DateSpan;
    ///
    /// assert_eq!(DateSpan::days(40).days, 40);
    /// ```
    #[must_use]
    pub const fn days(days: i32) -> DateSpan {
        DateSpan::new(0, 0, days)
    }

    /// The years and the months as one count of months.
    ///
    /// ```
    /// use teistro_calendar::arithmetic::DateSpan;
    ///
    /// assert_eq!(DateSpan::new(-1, 1, 0).total_months(), -11);
    /// ```
    #[must_use]
    pub const fn total_months(self) -> i64 {
        self.years as i64 * MONTHS + self.months as i64
    }
}

/// `UNSUPPORTED` for a calendar whose months the arithmetic does not
/// count: the ISO week date numbers weeks, and the Tibetan repeats
/// months.
fn counts_months(calendar: Calendar) -> Result<(), Error> {
    match calendar {
        Calendar::IsoWeek | Calendar::TibetanPhugpa => Err(Error::unsupported(format!(
            "{} has no twelve numbered months to add or count; convert the date into a calendar that has",
            calendar.key()
        ))
        .with_field("calendar")),
        _ => Ok(()),
    }
}

/// The date checked against the calendar answering for it.
fn checked(system: &dyn CalendarSystem, date: &CalendarDate) -> Result<FixedDay, Error> {
    if date.calendar != system.id() {
        return Err(Error::invalid_arg(format!(
            "the date is in {} and the calendar is {}",
            date.calendar.key(),
            system.id().key()
        ))
        .with_field("calendar"));
    }
    counts_months(date.calendar)?;
    system.fixed_of(date)
}

/// The year and month `months` months on from a year and month.
fn month_on(year: i32, month: u8, months: i64) -> Result<(i32, u8), Error> {
    let count = i64::from(year) * MONTHS + i64::from(month) - 1 + months;
    let year = i32::try_from(count.div_euclid(MONTHS)).map_err(|_| {
        out_of_range(format!(
            "{months} months on from year {year} is past any year"
        ))
        .with_detail(Detail::Overflow)
        .with_field("months")
    })?;
    let month = u8::try_from(count.rem_euclid(MONTHS) + 1).unwrap_or(1);
    Ok((year, month))
}

/// The fixed day `months` months on from `date`, a day the month lacks
/// resolved by `overflow`, and whether the month lacked it.
fn anniversary(
    system: &dyn CalendarSystem,
    date: &CalendarDate,
    months: i64,
    overflow: Overflow,
) -> Result<(FixedDay, bool), Error> {
    let (year, month) = month_on(date.year, date.month, months)?;
    let length = system.month_length(year, month)?;
    if date.day <= length {
        return Ok((system.to_fixed_ymd(year, month, date.day)?, false));
    }
    let last = system.to_fixed_ymd(year, month, length)?;
    match overflow {
        Overflow::Constrain => Ok((last, true)),
        Overflow::Carry => Ok((last.plus_days(i64::from(date.day - length)), true)),
        Overflow::Reject => Err(nonexistent(format!(
            "{} {year}-{month:02} has days 1 to {length}, so day {} falls on no day of it; CONSTRAIN takes the {length}th, CARRY the days past it into the next month",
            system.id().key(),
            date.day
        ))
        .with_field("day")),
    }
}

/// The date `span` after `date`: its years and months added first, as
/// one count of months, the day the month lacks resolved by `overflow`,
/// then its days.
///
/// ```
/// use teistro_calendar::arithmetic::{DateSpan, Overflow, add};
/// use teistro_calendar::{BikramSambat, CalendarDate};
/// use teistro_core::catalogue::Calendar;
/// use teistro_core::error::Detail;
///
/// let bs = BikramSambat::shipped();
/// let jestha_32 = CalendarDate::defined(Calendar::BikramSambat, 2081, 2, 32);
/// let carried = add(bs, &jestha_32, DateSpan::months(1), Overflow::Carry)?;
/// assert_eq!((carried.month, carried.day), (4, 1));
/// let refused = add(bs, &jestha_32, DateSpan::months(1), Overflow::Reject).unwrap_err();
/// assert_eq!((refused.detail, refused.field()), (Some(Detail::NonexistentDate), Some("day")));
/// // Days are counted after the months, on the fixed day.
/// let ten_on = add(bs, &jestha_32, DateSpan::days(10), Overflow::Constrain)?;
/// assert_eq!((ten_on.month, ten_on.day), (3, 10));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a date the calendar refuses or one written in
/// another calendar; `UNSUPPORTED` for the ISO week date and the Tibetan
/// calendar; `INVALID_ARG` with `NONEXISTENT_DATE` under
/// [`Overflow::Reject`]; and `OUT_OF_RANGE` past the calendar's span.
pub fn add(
    system: &dyn CalendarSystem,
    date: &CalendarDate,
    span: DateSpan,
    overflow: Overflow,
) -> Result<CalendarDate, Error> {
    let start = checked(system, date)?;
    let (moved, _) = match span.total_months() {
        0 => (start, false),
        months => anniversary(system, date, months, overflow)?,
    };
    system.date_of(moved.plus_days(i64::from(span.days)))
}

/// The years, months and days from `from` to `to`, counted the way an age
/// is: the most whole months whose anniversary of `from` is not past
/// `to`, then the days from that anniversary. An anniversary on a day its
/// month lacks is resolved by `overflow`, so [`add`] of the answer to
/// `from` under the same rule is `to`. Earlier than `from`, every part is
/// negative or zero, counted back the same way.
///
/// One born on the 32nd of Jestha 2050 is 32 years old on the 31st of
/// Jestha 2082, a 31-day month's last day, under
/// [`Overflow::Constrain`]; under [`Overflow::Carry`] on the 1st of
/// Ashadh; and [`Overflow::Reject`] refuses an answer that counts from
/// such a day (C803).
///
/// ```
/// use teistro_calendar::arithmetic::{DateSpan, Overflow, between};
/// use teistro_calendar::{BikramSambat, CalendarDate};
/// use teistro_core::catalogue::Calendar;
///
/// let bs = BikramSambat::shipped();
/// let born = CalendarDate::defined(Calendar::BikramSambat, 2050, 1, 15);
/// let today = CalendarDate::defined(Calendar::BikramSambat, 2081, 7, 10);
/// assert_eq!(between(bs, &born, &today, Overflow::Constrain)?, DateSpan::new(31, 5, 25));
/// assert_eq!(between(bs, &today, &born, Overflow::Constrain)?, DateSpan::new(-31, -5, -26));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`add`]; `INVALID_ARG` for two dates in different calendars.
pub fn between(
    system: &dyn CalendarSystem,
    from: &CalendarDate,
    to: &CalendarDate,
    overflow: Overflow,
) -> Result<DateSpan, Error> {
    let start = checked(system, from)?;
    let end = checked(system, to)?;
    let rough = (i64::from(to.year) - i64::from(from.year)) * MONTHS + i64::from(to.month)
        - i64::from(from.month);
    // Counting forward, the most months whose anniversary is not past the
    // end; back, the fewest whose anniversary is not before it. The month
    // difference overshoots by at most one, and a carried day by one more.
    let forward = start <= end;
    let lenient = match overflow {
        Overflow::Reject => Overflow::Constrain,
        other => other,
    };
    let mut months = rough;
    let (mut day, mut lacked) = anniversary(system, from, months, lenient)?;
    while months != 0 && if forward { day > end } else { day < end } {
        months += if forward { -1 } else { 1 };
        (day, lacked) = anniversary(system, from, months, lenient)?;
    }
    if lacked && overflow == Overflow::Reject && months != 0 {
        return Err(nonexistent(format!(
            "{months} months from {from} fall on a day the month lacks, so the difference counts from no day under REJECT; CONSTRAIN counts from the month's last day, CARRY from the days carried past it"
        ))
        .with_field("day"));
    }
    let too_long = |_| {
        out_of_range(format!("{from} to {to} is past any span"))
            .with_detail(Detail::Overflow)
            .with_field("to")
    };
    let years = i32::try_from(months / MONTHS).map_err(too_long)?;
    let months = i32::try_from(months % MONTHS).map_err(too_long)?;
    let days = i32::try_from(day.days_until(end)).map_err(too_long)?;
    Ok(DateSpan {
        years,
        months,
        days,
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::error::Status;

    use super::*;
    use crate::bikram_sambat::{BikramSambat, FIRST_YEAR, LAST_YEAR};
    use crate::gregorian::Gregorian;
    use crate::shipped;

    fn bs(year: i32, month: u8, day: u8) -> CalendarDate {
        CalendarDate::defined(Calendar::BikramSambat, year, month, day)
    }

    fn ymd(date: &CalendarDate) -> (i32, u8, u8) {
        (date.year, date.month, date.day)
    }

    /// Every adjacent pair of months in the table whose first is longer
    /// than its second, as (year, month, its length, the next's length).
    fn longer_before_shorter() -> Vec<(i32, u8, u8, u8)> {
        let table = BikramSambat::shipped();
        let mut found = Vec::new();
        for year in FIRST_YEAR..=LAST_YEAR {
            for month in 1..=12u8 {
                let (next_year, next_month) = month_on(year, month, 1).unwrap();
                if next_year > LAST_YEAR {
                    continue;
                }
                let here = table.month_length(year, month).unwrap();
                let there = table.month_length(next_year, next_month).unwrap();
                if here > there {
                    found.push((year, month, here, there));
                }
            }
        }
        found
    }

    #[test]
    fn a_32_day_month_meets_a_shorter_one_under_each_rule() {
        let table = BikramSambat::shipped();
        let edges = longer_before_shorter();
        // The table's adjacent months step down from 32 to 31 and 30 days,
        // and from 31 and 30 to 29; never from 32 straight to 29.
        for pair in [(32, 31), (32, 30), (31, 30), (31, 29), (30, 29)] {
            assert!(
                edges.iter().any(|&(_, _, a, b)| (a, b) == pair),
                "no {}-day month before a {}-day one",
                pair.0,
                pair.1
            );
        }
        assert!(!edges.iter().any(|&(_, _, a, b)| (a, b) == (32, 29)));
        for &(year, month, here, there) in &edges {
            let last = bs(year, month, here);
            let constrained = add(table, &last, DateSpan::months(1), Overflow::Constrain).unwrap();
            let (next_year, next_month) = month_on(year, month, 1).unwrap();
            assert_eq!(ymd(&constrained), (next_year, next_month, there));
            let carried = add(table, &last, DateSpan::months(1), Overflow::Carry);
            // Carried past the table's last day is out of its range.
            if let Ok(carried) = carried {
                let lacked = i64::from(here - there);
                assert_eq!(
                    table.fixed_of(&carried).unwrap(),
                    table.fixed_of(&constrained).unwrap().plus_days(lacked)
                );
                assert_eq!(carried.day, here - there);
            }
            let refused = add(table, &last, DateSpan::months(1), Overflow::Reject).unwrap_err();
            assert_eq!(refused.detail, Some(Detail::NonexistentDate));
            assert_eq!(refused.field(), Some("day"));
            // The difference reads the constrained day as the anniversary,
            // and the carried day as the carried one.
            assert_eq!(
                between(table, &last, &constrained, Overflow::Constrain).unwrap(),
                DateSpan::months(1)
            );
            // The table's last day has no day after it.
            if let Ok(day_after) = add(table, &constrained, DateSpan::days(1), Overflow::Constrain)
            {
                assert_eq!(
                    between(table, &last, &day_after, Overflow::Constrain).unwrap(),
                    DateSpan::new(0, 1, 1)
                );
            }
            assert!(between(table, &last, &constrained, Overflow::Reject).is_err());
        }
    }

    #[test]
    fn a_32nd_moved_into_a_29_day_month() {
        // A 32nd moved by months into a 29-day month: three days lacked.
        let table = BikramSambat::shipped();
        let mut met = 0;
        for year in 2000..=2090 {
            for month in 1..=12u8 {
                if table.month_length(year, month).unwrap() != 32 {
                    continue;
                }
                let start = bs(year, month, 32);
                for months in 1..=12 {
                    let (y, m) = month_on(year, month, months).unwrap();
                    if table.month_length(y, m).unwrap() != 29 {
                        continue;
                    }
                    met += 1;
                    let span = DateSpan::months(i32::try_from(months).unwrap());
                    let constrained = add(table, &start, span, Overflow::Constrain).unwrap();
                    assert_eq!(ymd(&constrained), (y, m, 29));
                    let carried = add(table, &start, span, Overflow::Carry).unwrap();
                    assert_eq!(carried.day, 3);
                    assert_eq!(
                        between(table, &start, &constrained, Overflow::Constrain).unwrap(),
                        span
                    );
                    assert_eq!(
                        between(table, &start, &carried, Overflow::Carry).unwrap(),
                        span
                    );
                    assert!(add(table, &start, span, Overflow::Reject).is_err());
                }
            }
        }
        assert!(met > 0);
    }

    #[test]
    fn an_age_is_counted_from_the_birthday_in_its_month() {
        let table = BikramSambat::shipped();
        // A year whose Jestha has 32 days and a later one where it has 31.
        let (born_year, later) = (FIRST_YEAR..LAST_YEAR - 30)
            .find_map(|year| {
                let later =
                    (year + 1..=LAST_YEAR).find(|y| table.month_length(*y, 2).unwrap() == 31)?;
                (table.month_length(year, 2).unwrap() == 32).then_some((year, later))
            })
            .unwrap();
        let born = bs(born_year, 2, 32);
        let years = later - born_year;
        let constrained_birthday = bs(later, 2, 31);
        assert_eq!(
            between(table, &born, &constrained_birthday, Overflow::Constrain).unwrap(),
            DateSpan::years(years)
        );
        let eve = bs(later, 2, 30);
        assert_eq!(
            between(table, &born, &eve, Overflow::Constrain).unwrap(),
            DateSpan::new(years - 1, 11, eve_days(table, &born, years, &eve))
        );
        // Carried, the year completes on the 1st of Ashadh.
        let ashadh_1 = bs(later, 3, 1);
        assert_eq!(
            between(table, &born, &ashadh_1, Overflow::Carry).unwrap(),
            DateSpan::years(years)
        );
        assert_eq!(
            between(table, &born, &constrained_birthday, Overflow::Carry)
                .unwrap()
                .years,
            years - 1
        );
    }

    /// The days from the 11th monthly anniversary of the year before.
    fn eve_days(table: &BikramSambat, born: &CalendarDate, years: i32, eve: &CalendarDate) -> i32 {
        let months = i64::from(years) * 12 - 1;
        let (from, _) = anniversary(table, born, months, Overflow::Constrain).unwrap();
        i32::try_from(from.days_until(table.fixed_of(eve).unwrap())).unwrap()
    }

    #[test]
    fn adding_the_difference_gives_the_end_on_every_rule_and_both_ways() {
        let table = BikramSambat::shipped();
        let (first, last) = table.capabilities().range;
        let span = first.days_until(last);
        // A spread of starts and ends over the table, the edges among
        // them, and every month's last day.
        let mut days: Vec<FixedDay> = (0..=40).map(|i| first.plus_days(span * i / 40)).collect();
        for year in [FIRST_YEAR, 2000, 2081, LAST_YEAR] {
            for month in 1..=12 {
                let length = table.month_length(year, month).unwrap();
                days.push(table.to_fixed_ymd(year, month, length).unwrap());
            }
        }
        for &a in &days {
            for &b in days.iter().step_by(3) {
                let (from, to) = (table.date_of(a).unwrap(), table.date_of(b).unwrap());
                for overflow in [Overflow::Constrain, Overflow::Carry] {
                    let Ok(found) = between(table, &from, &to, overflow) else {
                        // Only a carried anniversary past the table's end.
                        assert_eq!(overflow, Overflow::Carry);
                        continue;
                    };
                    let signs = [found.years, found.months, found.days];
                    assert!(
                        signs.iter().all(|p| *p >= 0) || signs.iter().all(|p| *p <= 0),
                        "{from} to {to}: {found:?}"
                    );
                    assert!(found.months.abs() < 12);
                    let back = add(table, &from, found, overflow).unwrap();
                    assert_eq!(table.fixed_of(&back).unwrap(), b, "{from} + {found:?}");
                }
            }
        }
    }

    #[test]
    fn the_tables_first_and_last_years_bound_it() {
        let table = BikramSambat::shipped();
        let first = bs(FIRST_YEAR, 1, 1);
        let last_length = table.month_length(LAST_YEAR, 12).unwrap();
        let last = bs(LAST_YEAR, 12, last_length);
        let whole = between(table, &first, &last, Overflow::Constrain).unwrap();
        assert_eq!((whole.years, whole.months), (LAST_YEAR - FIRST_YEAR, 11));
        assert_eq!(
            ymd(&add(table, &first, whole, Overflow::Constrain).unwrap()),
            ymd(&last)
        );
        let before = add(table, &first, DateSpan::days(-1), Overflow::Constrain).unwrap_err();
        assert_eq!(before.status, Status::OutOfRange);
        let after = add(table, &last, DateSpan::months(1), Overflow::Constrain).unwrap_err();
        assert_eq!(after.status, Status::OutOfRange);
        assert_eq!(
            add(
                table,
                &last,
                DateSpan::years(-(LAST_YEAR - FIRST_YEAR)),
                Overflow::Constrain
            )
            .map(|d| d.year)
            .unwrap(),
            FIRST_YEAR
        );
    }

    #[test]
    fn a_leap_like_year_and_a_plain_one() {
        let table = BikramSambat::shipped();
        // A 366-day year and a 365-day one: a year on from the new year is
        // the next new year either way, and the days say which.
        let long = (2000..=2090).find(|y| table.is_leap(*y)).unwrap();
        let short = (2000..=2090).find(|y| !table.is_leap(*y)).unwrap();
        for (year, days) in [(long, 366), (short, 365)] {
            let new_year = bs(year, 1, 1);
            let next = add(table, &new_year, DateSpan::years(1), Overflow::Constrain).unwrap();
            assert_eq!(ymd(&next), (year + 1, 1, 1));
            let by_days = add(table, &new_year, DateSpan::days(days), Overflow::Constrain).unwrap();
            assert_eq!(ymd(&by_days), (year + 1, 1, 1));
            assert_eq!(
                between(table, &new_year, &next, Overflow::Reject).unwrap(),
                DateSpan::years(1)
            );
        }
    }

    #[test]
    fn other_calendars_and_refusals() {
        let gregorian = shipped(Calendar::Gregorian).unwrap();
        let jan_31 = CalendarDate::defined(Calendar::Gregorian, 2024, 1, 31);
        let feb = add(gregorian, &jan_31, DateSpan::months(1), Overflow::Constrain).unwrap();
        assert_eq!(ymd(&feb), (2024, 2, 29));
        let carried = add(gregorian, &jan_31, DateSpan::months(1), Overflow::Carry).unwrap();
        assert_eq!(ymd(&carried), (2024, 3, 2));
        let back = add(
            gregorian,
            &feb,
            DateSpan::new(-1, -1, 0),
            Overflow::Constrain,
        )
        .unwrap();
        assert_eq!(ymd(&back), (2023, 1, 29));
        let none = add(gregorian, &jan_31, DateSpan::default(), Overflow::Reject).unwrap();
        assert_eq!(ymd(&none), (2024, 1, 31));
        assert_eq!(
            between(&Gregorian, &jan_31, &jan_31, Overflow::Reject).unwrap(),
            DateSpan::default()
        );
        // A calendar without twelve numbered months, a date in another
        // calendar, two calendars, a date the calendar refuses, a span past
        // any year.
        for calendar in [Calendar::IsoWeek, Calendar::TibetanPhugpa] {
            let system = shipped(calendar).unwrap();
            let date = system.date_of(FixedDay::new(739_000)).unwrap();
            let refused = add(system, &date, DateSpan::days(1), Overflow::Constrain).unwrap_err();
            assert_eq!(refused.status, Status::Unsupported);
        }
        let in_bs = bs(2081, 1, 1);
        assert_eq!(
            add(gregorian, &in_bs, DateSpan::days(1), Overflow::Constrain)
                .unwrap_err()
                .field(),
            Some("calendar")
        );
        assert!(between(gregorian, &jan_31, &in_bs, Overflow::Constrain).is_err());
        let bad = CalendarDate::defined(Calendar::Gregorian, 2023, 2, 29);
        assert!(add(gregorian, &bad, DateSpan::days(1), Overflow::Constrain).is_err());
        let past = add(
            gregorian,
            &jan_31,
            DateSpan::years(i32::MAX),
            Overflow::Constrain,
        );
        assert!(past.is_err());
        assert_eq!(
            month_on(i32::MAX, 12, 1).unwrap_err().detail,
            Some(Detail::Overflow)
        );
        // Under REJECT the difference refuses only a count from a lacking day.
        let mar_31 = CalendarDate::defined(Calendar::Gregorian, 2024, 3, 31);
        let apr_30 = CalendarDate::defined(Calendar::Gregorian, 2024, 4, 30);
        assert!(between(gregorian, &mar_31, &apr_30, Overflow::Reject).is_err());
        assert_eq!(
            between(gregorian, &mar_31, &apr_30, Overflow::Constrain).unwrap(),
            DateSpan::months(1)
        );
        assert_eq!(
            between(gregorian, &apr_30, &mar_31, Overflow::Reject).unwrap(),
            DateSpan::days(-30)
        );
    }
}

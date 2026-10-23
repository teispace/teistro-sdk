//! The Tibetan calendar, Phugpa: the most common version, the one the
//! Dalai Lama follows (Svante Janson, *Tibetan Calendar Mathematics*,
//! Uppsala 2007, revised 2014; Edward Henning, *Kālacakra and the Tibetan
//! Calendar*, 2007; Reingold and Dershowitz, *Calendrical Calculations*,
//! ch. 21).
//!
//! It needs no ephemeris, no sunrise and no place: every constant is an
//! exact fraction, so the calendar is integer arithmetic, with two short
//! tables standing in for a sine. The epoch is E806 (month 3 of 806,
//! β* = 61). A month is a lunation; a lunar day's number is given to the
//! calendar day in which that lunar day ends. When two end in one day the
//! second's number is skipped; when none ends, the day repeats the next
//! day's number and the first of the pair is the leap day. Two leap months
//! fall in every 65 years, a leap month taking the number of the month
//! after it and coming first.
//!
//! Years are Janson's *Y*, the Gregorian year in which the Tibetan year
//! starts and most of it falls. The royal year, counted from the
//! traditional ascent of the first king in 127 BC, is *Y* + 127; a date
//! carries no era, so the year a date form prints is *Y*.
//!
//! The moon's anomaly per lunar day is the rounded 1/28 the almanacs use,
//! not Henning's exact 3781/105840 (Janson, Remark 14).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Calendar, Gender, Kind, Samvatsara, TibetanAnimal, TibetanElement};
use teistro_core::error::Error;

use crate::date::{
    CalendarCapabilities, CalendarDate, CalendarSystem, Leap, check_fixed, nonexistent,
    out_of_range,
};
use crate::fixed::FixedDay;

/// The Tibetan calendar, Phugpa.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TibetanPhugpa;

/// The epoch year, *Y*₀ (E806).
const Y0: i64 = 806;
/// The epoch month, *M*₀.
const M0: i64 = 3;
/// The intercalation index at the epoch, β* (5.4).
const BETA_STAR: i64 = 61;
/// 184 − β* (5.15).
const BETA: i64 = 184 - BETA_STAR;

/// The common denominator of the true date: the least common multiple of
/// the mean date's 11 312, the moon equation's 126 · 60 and the sun
/// equation's 402 · 60.
const D: i64 = 102_317_040;

/// The moon's equation at the eight points of a quadrant (7.18).
const MOON: [i64; 8] = [0, 5, 10, 15, 19, 22, 24, 25];
/// The sun's equation at the four points of a quadrant (7.21).
const SUN: [i64; 4] = [0, 6, 10, 11];

/// The Tibetan years converted: from the first year of the first rabjung,
/// 1027, to 9998, inside the Gregorian calendar's own range.
pub const TIBETAN_RANGE: (i32, i32) = (1_027, 9_998);

/// The difference between a Julian day number and a fixed day.
const JD_OFFSET: i64 = 1_721_425;

/// A table of one quadrant read over the whole circle: symmetric about
/// the quadrant's end and negated in the second half, so `table[q − i]`
/// for the second quadrant and `−table[i]` past the half (7.18, 7.21).
fn wave(table: &[i64], i: i64) -> i64 {
    let quadrant = i64::try_from(table.len()).unwrap_or(1) - 1;
    let half = 2 * quadrant;
    let i = i.rem_euclid(2 * half);
    let (sign, i) = if i >= half { (-1, i - half) } else { (1, i) };
    let i = if i > quadrant { half - i } else { i };
    sign * usize::try_from(i)
        .ok()
        .and_then(|i| table.get(i))
        .copied()
        .unwrap_or(0)
}

fn moon_tab(i: i64) -> i64 {
    wave(&MOON, i)
}

fn sun_tab(i: i64) -> i64 {
    wave(&SUN, i)
}

/// The end of lunar day `d` of true month `n`, in units of 1/[`D`] of a
/// day, as a Julian date (7.1, 7.5, 7.11, 7.17–7.22).
fn true_date(n: i64, d: i64) -> i64 {
    // The mean date: n · 167025/5656 + d · 11135/11312 + 2015501 + 4783/5656.
    let mean = (n * 334_050 + d * 11_135 + 9_566) * (D / 11_312) + 2_015_501 * D;
    // The moon's anomaly, in 1/126 of a twenty-eighth: 28 · (n · 253/3528 +
    // d/28 + 475/3528), its fractional part.
    let anomaly = (n * 253 + d * 126 + 475).rem_euclid(3_528);
    let (i, r) = (anomaly / 126, anomaly % 126);
    let moon = moon_tab(i) * 126 + (moon_tab(i + 1) - moon_tab(i)) * r;
    // The sun's anomaly, the mean sun less a quarter, in 1/402 of a
    // twelfth: 12 · (n · 65/804 + d · 13/4824 + 743/804 − 1/4).
    let anomaly = (n * 390 + d * 13 + 4_458 - 1_206).rem_euclid(4_824);
    let (i, r) = (anomaly / 402, anomaly % 402);
    let sun = sun_tab(i) * 402 + (sun_tab(i + 1) - sun_tab(i)) * r;
    mean + moon * (D / (126 * 60)) - sun * (D / (402 * 60))
}

/// The Julian day number of the calendar day in which lunar day `k`
/// ends, counting lunar days from true month 0's first as `30n + d − 1`.
fn ends_on(k: i64) -> i64 {
    true_date(k.div_euclid(30), k.rem_euclid(30) + 1).div_euclid(D)
}

/// The true month count of month `month` of `year`, the leap month when
/// `leap` (5.10).
fn true_month(year: i64, month: i64, leap: bool) -> i64 {
    let m_star = 12 * (year - Y0) + month - M0;
    (67 * m_star + BETA_STAR + 17).div_euclid(65) - i64::from(leap)
}

/// Whether month `month` of `year` is doubled, a leap month before it
/// (5.24).
#[must_use]
pub fn has_leap_month(year: i32, month: u8) -> bool {
    leap_month_in(i64::from(year), i64::from(month))
}

fn leap_month_in(year: i64, month: i64) -> bool {
    let index = (24 * (year - Y0) + 2 * month).rem_euclid(65);
    index == BETA.rem_euclid(65) || index == (BETA + 1).rem_euclid(65)
}

/// The year, the month and whether it is the leap month, of true month
/// `n` (5.19–5.22).
fn month_of(n: i64) -> (i64, i64, bool) {
    let x = (65 * n + BETA + 66).div_euclid(67);
    let month = (x - 1).rem_euclid(12) + 1;
    let year = (x - month).div_euclid(12) + Y0;
    let leap = matches!((65 * n + BETA).rem_euclid(67), 1 | 2);
    (year, month, leap)
}

/// The fixed day of a Tibetan date and its marks; `None` when the date is
/// skipped, or marked a leap day or month the calendar does not have.
fn fixed_from_tibetan(year: i64, month: i64, leap: Option<Leap>, day: i64) -> Option<i64> {
    let (leap_month, leap_day) = leap.map_or((false, false), |leap| (leap.month(), leap.day()));
    if leap_month && !leap_month_in(year, month) {
        return None;
    }
    let k = 30 * true_month(year, month, leap_month) + day - 1;
    let (ends, before) = (ends_on(k), ends_on(k - 1));
    match (ends - before, leap_day) {
        (2, true) => Some(ends - 1 - JD_OFFSET),
        (0, _) | (_, true) => None,
        _ => Some(ends - JD_OFFSET),
    }
}

/// The Tibetan date of a fixed day: the year, the month, its marks and the
/// day (Janson §8).
fn tibetan_from_fixed(fixed: i64) -> (i64, i64, Option<Leap>, i64) {
    let jd = fixed + JD_OFFSET;
    // The mean lunar day is 11135/11312 of a day, and the equations move a
    // lunar day's end by at most 36/60 of one: start a few lunar days back
    // and step to the first lunar day ending on or after the day.
    let mut k = ((jd - 2_015_501) * 11_312).div_euclid(11_135) - 2;
    while ends_on(k) >= jd {
        k -= 3;
    }
    while ends_on(k) < jd {
        k += 1;
    }
    let (n, day) = (k.div_euclid(30), k.rem_euclid(30) + 1);
    let (year, month, leap_month) = month_of(n);
    let leap_day = ends_on(k) > jd;
    (year, month, Leap::of(leap_month, leap_day), day)
}

/// The fixed day of Losar, the first day of `year`: the day after the last
/// day of the year before's (regular) month 12, which is the first day of
/// a leap month 1 when there is one, and day 2 when day 1 is skipped
/// (Janson §8).
#[must_use]
pub fn losar(year: i32) -> FixedDay {
    let k = 30 * true_month(i64::from(year) - 1, 12, false) + 29;
    FixedDay::new(ends_on(k) + 1 - JD_OFFSET)
}

/// Which neighbour keeps an observance whose date's number is skipped.
///
/// Janson §11, after his \[21\]: "if that date is skipped, the holiday is
/// on the preceding day", and FPMT kept Chotrul Duchen 2018, whose 15th
/// of month 1 was skipped, on 1 March, the 14th; Tushita, in Dharamsala,
/// printed 2 March, the 16th. The preceding day is the one in which the
/// skipped lunar day ends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkippedDay {
    /// The day before, the one the skipped lunar day ends in.
    #[default]
    Before,
    /// The day after, the next number's.
    After,
}

/// The day an observance kept on `day` of `month` of `year` falls on. The
/// month is the first of a doubled pair, the leap month; in it, the day
/// is the first of a repeated number, and a neighbour of a skipped one
/// (`skipped`), never the other month's day of that number. Month 1's
/// first day is [`losar`], the year's first day, which is day 2 when day
/// 1 is skipped (Janson §8, Remark 17).
///
/// The first month and the first day are the almanac's: the Tibetan Nuns
/// Project, in Dharamsala, printed Saga Dawa Duchen 2016 on 21 May, the
/// 15th of leap month 4, Lhabab Duchen 2021 on 27 October, the 22nd of
/// leap month 9, and Chokhor Duchen 2024 on 9 July, the first of the two
/// 4ths of leap month 6. Janson's \[21\] holds holidays out of a leap month,
/// a reading those three years do not follow (`calendars/tibetan.md`).
///
/// ```
/// use teistro_calendar::tibetan::{SkippedDay, observed};
/// use teistro_calendar::{CalendarSystem, Gregorian};
///
/// let chokhor = observed(2024, 6, 4, SkippedDay::Before)?;
/// let date = Gregorian.date_of(chokhor)?;
/// assert_eq!((date.year, date.month, date.day), (2024, 7, 9));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `OUT_OF_RANGE` for a year outside [`TIBETAN_RANGE`], named `year`, or
/// a day past the calendar's range; `INVALID_ARG` with the
/// `NONEXISTENT_DATE` detail for a month outside 1 to 12 or a day
/// outside 1 to 30, named `month` or `day`.
pub fn observed(year: i32, month: u8, day: u8, skipped: SkippedDay) -> Result<FixedDay, Error> {
    check_year(year)?;
    check_month_and_day(&CalendarDate::defined(
        Calendar::TibetanPhugpa,
        year,
        month,
        day,
    ))?;
    if (month, day) == (1, 1) {
        return Ok(losar(year));
    }
    let (year, month) = (i64::from(year), i64::from(month));
    let k = 30 * true_month(year, month, leap_month_in(year, month)) + i64::from(day) - 1;
    let (ends, before) = (ends_on(k), ends_on(k - 1));
    let jd = match (ends - before, skipped) {
        // No lunar day ends the day before: it repeats the number, first.
        (2, _) => ends - 1,
        // Two end in one day, the second's number skipped: that day holds
        // the number before, and the day after the next.
        (0, SkippedDay::After) => ends + 1,
        // One ends in the day, or a skipped number kept the day before.
        _ => ends,
    };
    let fixed = FixedDay::new(jd - JD_OFFSET);
    check_fixed(
        Calendar::TibetanPhugpa,
        TibetanPhugpa.capabilities().range,
        fixed,
    )?;
    Ok(fixed)
}

/// A Tibetan year's names (Janson §4, Appendix B): its place in the
/// numbered sixty-year rabjung cycles, with the Indian cycle's name, and
/// the Chinese cycle's element, gender and animal, which name it as
/// "Fire–male–Horse".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TibetanYear {
    /// The year, Janson's *Y*.
    pub year: i32,
    /// The rabjung cycle, the first beginning in 1027 (4.3): 2026 is in
    /// the 17th, which began in 1987.
    pub rabjung: i32,
    /// The year's place in its cycle, 1 to 60 (4.2): 1 is *rab byung*,
    /// Prabhava, which names the cycle.
    pub year_in_rabjung: u8,
    /// The Indian cycle's name for that place, which the Tibetan year
    /// takes without expunging any (Appendix B): the samvatsara list read
    /// in order.
    pub samvatsara: Samvatsara,
    /// The element, held for two years (table 2).
    pub element: TibetanElement,
    /// Male in a year of odd place in the Chinese cycle, female in an even
    /// one; the animal decides it too (table 3).
    pub gender: Gender,
    /// The animal (table 3).
    pub animal: TibetanAnimal,
}

impl TibetanYear {
    /// Where a year names catalogue members, and of which kind: what the
    /// boundary writes in full. The crate's tests hold it to serde, both
    /// ways.
    pub const MEMBERS: [(&'static str, Kind); 4] = [
        ("samvatsara", Kind::Samvatsara),
        ("element", Kind::TibetanElement),
        ("gender", Kind::Gender),
        ("animal", Kind::TibetanAnimal),
    ];
}

/// The names of a Tibetan year (Janson (4.1)–(4.4)).
///
/// ```
/// use teistro_calendar::tibetan::year_name;
/// use teistro_core::catalogue::{Gender, Samvatsara, TibetanAnimal, TibetanElement};
///
/// let year = year_name(2026)?;
/// assert_eq!((year.rabjung, year.year_in_rabjung), (17, 40));
/// assert_eq!(year.samvatsara, Samvatsara::Parabhava);
/// assert_eq!(
///     (year.element, year.gender, year.animal),
///     (TibetanElement::Fire, Gender::Male, TibetanAnimal::Horse)
/// );
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `OUT_OF_RANGE` for a year outside [`TIBETAN_RANGE`], named `year`.
pub fn year_name(year: i32) -> Result<TibetanYear, Error> {
    check_year(year)?;
    // `amod`: the remainder in 1..=n rather than 0..n.
    let amod = |value: i32, n: i32| (value - 1).rem_euclid(n) + 1;
    let place = amod(year - 1_026, 60);
    let stem = amod(year - 3, 10);
    let branch = amod(year - 3, 12);
    // Each index is in range by construction; a miss is a catalogue that
    // lost a member.
    let id = |index: i32| u16::try_from(index).ok();
    let member = |what: &str| Error::internal(format!("the {what} of {year} is no member"));
    Ok(TibetanYear {
        year,
        rabjung: (year - 1_026 + 59).div_euclid(60),
        year_in_rabjung: u8::try_from(place).map_err(|_| member("place"))?,
        samvatsara: id(place - 1)
            .and_then(Samvatsara::from_id)
            .ok_or_else(|| member("samvatsara"))?,
        element: id((stem + 1) / 2 - 1)
            .and_then(TibetanElement::from_id)
            .ok_or_else(|| member("element"))?,
        gender: if stem % 2 == 1 {
            Gender::Male
        } else {
            Gender::Female
        },
        animal: id(branch - 1)
            .and_then(TibetanAnimal::from_id)
            .ok_or_else(|| member("animal"))?,
    })
}

fn check_year(year: i32) -> Result<(), Error> {
    if year < TIBETAN_RANGE.0 || year > TIBETAN_RANGE.1 {
        return Err(out_of_range(format!(
            "{} covers Tibetan years {} to {}, not {year}",
            Calendar::TibetanPhugpa.key(),
            TIBETAN_RANGE.0,
            TIBETAN_RANGE.1
        ))
        .with_field("year"));
    }
    Ok(())
}

fn check_month_and_day(date: &CalendarDate) -> Result<(), Error> {
    if !(1..=12).contains(&date.month) {
        return Err(nonexistent(format!(
            "{} has months 1 to 12, not {}",
            Calendar::TibetanPhugpa.key(),
            date.month
        ))
        .with_field("month"));
    }
    if !(1..=30).contains(&date.day) {
        return Err(nonexistent(format!(
            "{} numbers a month's days 1 to 30, not {}",
            Calendar::TibetanPhugpa.key(),
            date.day
        ))
        .with_field("day"));
    }
    Ok(())
}

/// Why a date in range is not a day: its leap month or leap day does not
/// exist, or its number is skipped.
fn why_not(date: &CalendarDate) -> Error {
    let key = Calendar::TibetanPhugpa.key();
    let (year, month, day) = (date.year, date.month, date.day);
    let leap_month = date.leap.is_some_and(Leap::month);
    if leap_month && !has_leap_month(year, month) {
        return nonexistent(format!(
            "{key} {year} has no leap month {month}{}",
            leap_months(year)
        ))
        .with_field("leap");
    }
    let unmarked = Leap::of(leap_month, false);
    let plain = fixed_from_tibetan(i64::from(year), i64::from(month), unmarked, i64::from(day));
    if date.leap.is_some_and(Leap::day) && plain.is_some() {
        return nonexistent(format!(
            "{key} {year}-{month:02}-{day:02} is not repeated, so it has no leap day"
        ))
        .with_field("leap");
    }
    let stand_in = if day == 1 {
        String::from("its lunar day ends on the month before's last day")
    } else {
        format!("its lunar day ends on the day numbered {}", day - 1)
    };
    nonexistent(format!(
        "{key} {year}-{month:02}-{day:02} is skipped: {stand_in}"
    ))
    .with_field("day")
}

/// The leap months of a year, for a refusal: "; its leap month is 5", or
/// nothing when it has none.
fn leap_months(year: i32) -> String {
    let months: Vec<String> = (1..=12u8)
        .filter(|&month| has_leap_month(year, month))
        .map(|month| month.to_string())
        .collect();
    match months.as_slice() {
        [] => String::new(),
        [one] => format!("; its leap month is {one}"),
        many => format!("; its leap months are {}", many.join(" and ")),
    }
}

impl CalendarSystem for TibetanPhugpa {
    fn id(&self) -> Calendar {
        Calendar::TibetanPhugpa
    }

    fn capabilities(&self) -> CalendarCapabilities {
        CalendarCapabilities {
            range: (
                losar(TIBETAN_RANGE.0),
                losar(TIBETAN_RANGE.1 + 1).plus_days(-1),
            ),
            needs_place: false,
            needs_ephemeris: false,
            eras: &[],
        }
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "inside the range a year fits an i32, a month 1 to 12 and a day 1 to 30 a u8"
    )]
    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        check_fixed(Calendar::TibetanPhugpa, self.capabilities().range, fixed)?;
        let (year, month, leap, day) = tibetan_from_fixed(fixed.get());
        let year = year as i32;
        Ok(
            CalendarDate::defined(Calendar::TibetanPhugpa, year, month as u8, day as u8)
                .with_leap(leap),
        )
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        check_year(date.year)?;
        check_month_and_day(date)?;
        let fixed = fixed_from_tibetan(
            i64::from(date.year),
            i64::from(date.month),
            date.leap,
            i64::from(date.day),
        )
        .ok_or_else(|| why_not(date))?;
        let fixed = FixedDay::new(fixed);
        // Day 1 of the first year, when it is skipped, would fall before
        // the range's first day; nothing else can.
        check_fixed(Calendar::TibetanPhugpa, self.capabilities().range, fixed)?;
        Ok(fixed)
    }

    /// The days of the regular month, 29 or 30; its days are numbered 1
    /// to 30, a skipped number missing and a repeated one counted twice.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a month holds 29 or 30 days"
    )]
    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        check_year(year)?;
        check_month_and_day(&CalendarDate::defined(
            Calendar::TibetanPhugpa,
            year,
            month,
            1,
        ))?;
        let n = true_month(i64::from(year), i64::from(month), false);
        Ok((ends_on(30 * n + 29) - ends_on(30 * n - 1)) as u8)
    }

    /// Whether the year holds a leap month.
    fn is_leap(&self, year: i32) -> bool {
        (1..=12).any(|month| has_leap_month(year, month))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::gregorian::{fixed_from_gregorian as gregorian_day, gregorian_from_fixed};

    /// Janson's Table 1: Losar, Phugpa, 1927 to 2046, as printed.
    const LOSAR: [(i32, u8, u8); 120] = [
        (1927, 3, 4),
        (1928, 2, 22),
        (1929, 2, 10),
        (1930, 3, 1),
        (1931, 2, 18),
        (1932, 2, 7),
        (1933, 2, 25),
        (1934, 2, 14),
        (1935, 2, 4),
        (1936, 2, 23),
        (1937, 2, 12),
        (1938, 3, 3),
        (1939, 2, 20),
        (1940, 2, 9),
        (1941, 2, 26),
        (1942, 2, 16),
        (1943, 2, 5),
        (1944, 2, 24),
        (1945, 2, 13),
        (1946, 3, 4),
        (1947, 2, 21),
        (1948, 2, 10),
        (1949, 2, 28),
        (1950, 2, 17),
        (1951, 2, 7),
        (1952, 2, 26),
        (1953, 2, 14),
        (1954, 2, 4),
        (1955, 2, 23),
        (1956, 2, 12),
        (1957, 3, 2),
        (1958, 2, 19),
        (1959, 2, 8),
        (1960, 2, 27),
        (1961, 2, 16),
        (1962, 2, 5),
        (1963, 2, 24),
        (1964, 2, 14),
        (1965, 3, 4),
        (1966, 2, 21),
        (1967, 2, 10),
        (1968, 2, 29),
        (1969, 2, 17),
        (1970, 2, 7),
        (1971, 2, 26),
        (1972, 2, 15),
        (1973, 3, 5),
        (1974, 2, 22),
        (1975, 2, 11),
        (1976, 3, 1),
        (1977, 2, 19),
        (1978, 2, 8),
        (1979, 2, 27),
        (1980, 2, 17),
        (1981, 2, 5),
        (1982, 2, 24),
        (1983, 2, 13),
        (1984, 3, 3),
        (1985, 2, 20),
        (1986, 2, 9),
        (1987, 2, 28),
        (1988, 2, 18),
        (1989, 2, 7),
        (1990, 2, 26),
        (1991, 2, 15),
        (1992, 3, 5),
        (1993, 2, 22),
        (1994, 2, 11),
        (1995, 3, 2),
        (1996, 2, 19),
        (1997, 2, 8),
        (1998, 2, 27),
        (1999, 2, 17),
        (2000, 2, 6),
        (2001, 2, 24),
        (2002, 2, 13),
        (2003, 3, 3),
        (2004, 2, 21),
        (2005, 2, 9),
        (2006, 2, 28),
        (2007, 2, 18),
        (2008, 2, 7),
        (2009, 2, 25),
        (2010, 2, 14),
        (2011, 3, 5),
        (2012, 2, 22),
        (2013, 2, 11),
        (2014, 3, 2),
        (2015, 2, 19),
        (2016, 2, 9),
        (2017, 2, 27),
        (2018, 2, 16),
        (2019, 2, 5),
        (2020, 2, 24),
        (2021, 2, 12),
        (2022, 3, 3),
        (2023, 2, 21),
        (2024, 2, 10),
        (2025, 2, 28),
        (2026, 2, 18),
        (2027, 2, 7),
        (2028, 2, 26),
        (2029, 2, 14),
        (2030, 3, 5),
        (2031, 2, 22),
        (2032, 2, 12),
        (2033, 3, 2),
        (2034, 2, 19),
        (2035, 2, 9),
        (2036, 2, 27),
        (2037, 2, 15),
        (2038, 3, 6),
        (2039, 2, 23),
        (2040, 2, 13),
        (2041, 3, 3),
        (2042, 2, 21),
        (2043, 2, 10),
        (2044, 2, 29),
        (2045, 2, 17),
        (2046, 2, 6),
    ];

    fn date(year: i32, month: u8, day: u8, leap: Option<Leap>) -> CalendarDate {
        CalendarDate::defined(Calendar::TibetanPhugpa, year, month, day).with_leap(leap)
    }

    #[test]
    fn every_losar_of_jansons_table() {
        for (year, month, day) in LOSAR {
            assert_eq!(
                gregorian_from_fixed(losar(year)),
                (year, month, day),
                "{year}"
            );
            // Losar is the year's first day, whatever its number.
            let first = TibetanPhugpa.date_of(losar(year)).unwrap();
            let before = TibetanPhugpa.date_of(losar(year).plus_days(-1)).unwrap();
            assert_eq!((first.year, before.year), (year, year - 1));
            assert_eq!(first.month, 1);
            assert!(first.day <= 2, "{year}: {first:?}");
            assert_eq!(
                (before.month, before.leap.is_some_and(Leap::month)),
                (12, false)
            );
        }
        // 2000 began with a leap month 1 (Janson §8), on 6 February.
        let first = TibetanPhugpa.date_of(losar(2000)).unwrap();
        assert_eq!(first.leap, Some(Leap::Month));
        assert_eq!(first.era, None);
    }

    #[test]
    fn every_day_round_trips_with_its_marks_and_in_order() {
        let (first, last) = (gregorian_day(1900, 1, 1), gregorian_day(2101, 1, 1));
        let mut previous: Option<(i32, u8, bool, u8, bool)> = None;
        let mut leap_months = std::collections::BTreeSet::new();
        let (mut leap_days, mut fixed) = (0, first);
        while fixed.get() < last.get() {
            let date = TibetanPhugpa.date_of(fixed).unwrap();
            assert_eq!(TibetanPhugpa.fixed_of(&date).unwrap(), fixed, "{date:?}");
            let leap = date.leap.unwrap_or(Leap::Day);
            let (month_leap, day_leap) = (
                date.leap.is_some_and(|_| leap.month()),
                date.leap.is_some_and(|_| leap.day()),
            );
            // A leap month and a leap day come before their twins.
            let key = (date.year, date.month, !month_leap, date.day, !day_leap);
            assert!(previous < Some(key), "{date:?} after {previous:?}");
            previous = Some(key);
            if month_leap {
                assert!(has_leap_month(date.year, date.month), "{date:?}");
                leap_months.insert((date.year, date.month));
            }
            leap_days += i32::from(day_leap);
            fixed = fixed.plus_days(1);
        }
        // Two leap months in 65 years, each where (5.24) puts it.
        let rule = (1900..=2100)
            .flat_map(|year| (1..=12).map(move |month| (year, month)))
            .filter(|&(year, month)| has_leap_month(year, month))
            .count();
        let inside = leap_months.iter().filter(|(year, _)| *year >= 1900).count();
        assert_eq!(inside, rule);
        assert!(leap_days > 1_500, "{leap_days}");
    }

    #[test]
    fn a_missing_day_is_refused_by_name() {
        // Find a skipped day and a repeated one in 2026.
        let mut skipped = None;
        let mut repeated = None;
        let mut fixed = gregorian_day(2026, 1, 1);
        while skipped.is_none() || repeated.is_none() {
            let date = TibetanPhugpa.date_of(fixed).unwrap();
            let next = TibetanPhugpa.date_of(fixed.plus_days(1)).unwrap();
            if date.leap.is_some_and(Leap::day) {
                repeated.get_or_insert(date.clone());
            }
            if next.month == date.month && next.day == date.day + 2 {
                skipped.get_or_insert(date.clone().with_leap(None));
            }
            fixed = fixed.plus_days(1);
        }
        let skipped = skipped.unwrap();
        let refused = TibetanPhugpa
            .fixed_of(&CalendarDate {
                day: skipped.day + 1,
                ..skipped.clone()
            })
            .unwrap_err();
        assert!(
            refused.message.contains("is skipped"),
            "{}",
            refused.message
        );
        assert!(
            refused
                .message
                .contains(&format!("numbered {}", skipped.day)),
            "{}",
            refused.message
        );
        let plain = TibetanPhugpa.fixed_of(&skipped.clone().with_leap(Some(Leap::Day)));
        assert!(plain.unwrap_err().message.contains("not repeated"));
        let repeated = repeated.unwrap();
        let twin = TibetanPhugpa
            .fixed_of(&repeated.clone().with_leap(None))
            .unwrap();
        assert_eq!(
            TibetanPhugpa.fixed_of(&repeated).unwrap().plus_days(1),
            twin
        );
        // 2026 has no leap month 1.
        let refused = TibetanPhugpa
            .fixed_of(&date(2026, 1, 1, Some(Leap::Month)))
            .unwrap_err();
        assert!(
            refused.message.contains("no leap month 1"),
            "{}",
            refused.message
        );
        // A calendar that repeats nothing refuses a mark.
        let marked =
            CalendarDate::defined(Calendar::Gregorian, 2026, 2, 18).with_leap(Some(Leap::Month));
        let refused = crate::gregorian::Gregorian.fixed_of(&marked).unwrap_err();
        assert_eq!(refused.field(), Some("leap"), "{}", refused.message);
        assert_eq!(
            TibetanPhugpa
                .fixed_of(&date(1026, 1, 1, None))
                .unwrap_err()
                .status,
            teistro_core::error::Status::OutOfRange
        );
    }

    #[test]
    fn a_month_holds_twenty_nine_or_thirty_days() {
        for year in 2020..2030 {
            let mut total = 0;
            for month in 1..=12 {
                let length = TibetanPhugpa.month_length(year, month).unwrap();
                assert!((29..=30).contains(&length), "{year}-{month}: {length}");
                total += i64::from(length);
            }
            let leaps: i64 = (1..=12)
                .filter(|&month| has_leap_month(year, month))
                .map(|month| {
                    let n = true_month(i64::from(year), i64::from(month), true);
                    ends_on(30 * n + 29) - ends_on(30 * n - 1)
                })
                .sum();
            assert_eq!(
                total + leaps,
                losar(year + 1).get() - losar(year).get(),
                "{year}"
            );
        }
        assert_eq!(TibetanPhugpa.is_leap(2000), has_leap_month(2000, 1));
    }

    /// Rows of Janson's Appendix B, table 10 (from Henning), read off the
    /// page: the place, the element and animal, and the Sanskrit name, for
    /// the years of the current cycle, with the gender table 3 gives the
    /// animal; and the first year of the first cycle.
    #[test]
    fn a_year_takes_the_names_janson_tabulates() {
        use teistro_core::catalogue::{Gender, Samvatsara, TibetanAnimal, TibetanElement};
        let rows = [
            (
                1_027,
                1,
                1,
                TibetanElement::Fire,
                TibetanAnimal::Rabbit,
                Samvatsara::Prabhava,
            ),
            (
                1_987,
                17,
                1,
                TibetanElement::Fire,
                TibetanAnimal::Rabbit,
                Samvatsara::Prabhava,
            ),
            (
                2_000,
                17,
                14,
                TibetanElement::Iron,
                TibetanAnimal::Dragon,
                Samvatsara::Vikrama,
            ),
            (
                2_007,
                17,
                21,
                TibetanElement::Fire,
                TibetanAnimal::Pig,
                Samvatsara::Sarvajit,
            ),
            (
                2_026,
                17,
                40,
                TibetanElement::Fire,
                TibetanAnimal::Horse,
                Samvatsara::Parabhava,
            ),
            (
                2_044,
                17,
                58,
                TibetanElement::Wood,
                TibetanAnimal::Mouse,
                Samvatsara::Raktakshi,
            ),
            (
                2_046,
                17,
                60,
                TibetanElement::Fire,
                TibetanAnimal::Tiger,
                Samvatsara::Akshaya,
            ),
            (
                2_047,
                18,
                1,
                TibetanElement::Fire,
                TibetanAnimal::Rabbit,
                Samvatsara::Prabhava,
            ),
        ];
        for (year, rabjung, place, element, animal, samvatsara) in rows {
            let named = year_name(year).unwrap();
            assert_eq!(
                (
                    named.rabjung,
                    named.year_in_rabjung,
                    named.element,
                    named.animal,
                    named.samvatsara
                ),
                (rabjung, place, element, animal, samvatsara),
                "{year}"
            );
            let male = matches!(
                animal,
                TibetanAnimal::Mouse
                    | TibetanAnimal::Tiger
                    | TibetanAnimal::Dragon
                    | TibetanAnimal::Horse
                    | TibetanAnimal::Monkey
                    | TibetanAnimal::Dog
            );
            assert_eq!(
                named.gender,
                if male { Gender::Male } else { Gender::Female },
                "{year}"
            );
        }
        assert_eq!(year_name(1_026).unwrap_err().field(), Some("year"));
    }

    /// The names recur every sixty years, and the cycle's number counts up.
    #[test]
    fn a_year_names_the_year_sixty_after_it_alike() {
        for year in TIBETAN_RANGE.0..TIBETAN_RANGE.0 + 120 {
            let (this, next) = (year_name(year).unwrap(), year_name(year + 60).unwrap());
            assert_eq!(
                (this.samvatsara, this.element, this.gender, this.animal),
                (next.samvatsara, next.element, next.gender, next.animal)
            );
            assert_eq!(next.rabjung, this.rabjung + 1);
        }
    }

    /// `TibetanYear::MEMBERS` names every string a year serialises to, and
    /// what it writes in full reads back.
    #[test]
    fn a_year_names_its_members_where_its_table_says() {
        let year = year_name(2026).unwrap();
        let value = serde_json::to_value(year).unwrap();
        // Sorted: serde_json keeps insertion order when a workspace build
        // turns on its `preserve_order`, and sorts the keys otherwise.
        let mut strings: Vec<&str> = value
            .as_object()
            .unwrap()
            .iter()
            .filter(|(_, field)| field.is_string())
            .map(|(key, _)| key.as_str())
            .collect();
        let mut listed: Vec<&str> = TibetanYear::MEMBERS.iter().map(|(path, _)| *path).collect();
        listed.sort_unstable();
        strings.sort_unstable();
        assert_eq!(strings, listed);
        let mut full = value;
        for (path, kind) in TibetanYear::MEMBERS {
            teistro_core::catalogue::write_in_full(&mut full, path, kind);
        }
        assert_eq!(
            full.get("animal"),
            Some(&serde_json::json!("tibetan_animal.HORSE"))
        );
        assert_eq!(serde_json::from_value::<TibetanYear>(full).unwrap(), year);
    }

    /// Holy days the Tibetan Nuns Project (Dharamsala) printed for 2016 to
    /// 2024, with FPMT's Saga Dawa Duchen 2025 and Chokhor Duchen 2026 as
    /// calendars print it: the Tibetan month and day, and the day kept.
    /// Three are in a leap month and one on the first of a repeated day.
    const PRINTED: [(u8, u8, (i32, u8, u8)); 12] = [
        (4, 15, (2016, 5, 21)),
        (4, 15, (2019, 6, 17)),
        (4, 15, (2021, 5, 26)),
        (5, 15, (2021, 6, 24)),
        (6, 4, (2021, 7, 14)),
        (9, 22, (2021, 10, 27)),
        (4, 15, (2024, 5, 23)),
        (5, 15, (2024, 6, 22)),
        (6, 4, (2024, 7, 9)),
        (9, 22, (2024, 11, 22)),
        (4, 15, (2025, 6, 11)),
        (6, 4, (2026, 7, 18)),
    ];

    #[test]
    fn an_observance_falls_on_the_day_the_almanac_printed() {
        for (month, day, (year, g_month, g_day)) in PRINTED {
            let kept = observed(year, month, day, SkippedDay::Before).unwrap();
            assert_eq!(
                kept,
                gregorian_day(year, g_month, g_day),
                "{year} {month}/{day}"
            );
        }
        // Losar is the year's first day, in leap month 1 in 2000 and 2019.
        for year in [2000, 2019, 2026] {
            assert_eq!(
                observed(year, 1, 1, SkippedDay::After).unwrap(),
                losar(year)
            );
        }
    }

    /// Chotrul Duchen 2018: the 15th of month 1 was skipped. FPMT kept the
    /// 14th, 1 March; Tushita printed the 16th, 2 March.
    #[test]
    fn a_skipped_date_is_kept_on_the_neighbour_asked() {
        let day = |skipped| gregorian_from_fixed(observed(2018, 1, 15, skipped).unwrap());
        assert_eq!(day(SkippedDay::Before), (2018, 3, 1));
        assert_eq!(day(SkippedDay::After), (2018, 3, 2));
    }

    /// Over two centuries, every date's observance is the leap month's when
    /// the month is doubled, the first day carrying the number in it, and
    /// beside the days around a skipped one; Losar aside.
    #[test]
    fn an_observance_is_the_first_day_of_its_number_in_the_first_month() {
        use std::collections::BTreeMap;
        let first = gregorian_day(1_900, 1, 1).get();
        let last = gregorian_day(2_100, 12, 31).get();
        // (year, month, leap month) to its days, in order, with numbers.
        let mut months: BTreeMap<(i64, i64, bool), Vec<(i64, i64)>> = BTreeMap::new();
        for fixed in first..=last {
            let (year, month, leap, day) = tibetan_from_fixed(fixed);
            let leap_month = leap.is_some_and(Leap::month);
            months
                .entry((year, month, leap_month))
                .or_default()
                .push((day, fixed));
        }
        let mut skips = 0;
        for (&(year, month, leap_month), days) in &months {
            // The first and last months are cut by the range; the regular
            // month of a doubled pair keeps nothing.
            if year <= 1_900 || year >= 2_100 || (!leap_month && leap_month_in(year, month)) {
                continue;
            }
            let (y, m) = (i32::try_from(year).unwrap(), u8::try_from(month).unwrap());
            for number in 1..=30 {
                if (m, number) == (1, 1) {
                    continue;
                }
                let held = days.iter().find(|(day, _)| *day == i64::from(number));
                let before = observed(y, m, number, SkippedDay::Before).unwrap().get();
                let after = observed(y, m, number, SkippedDay::After).unwrap().get();
                if let Some(&(_, fixed)) = held {
                    assert_eq!((before, after), (fixed, fixed));
                    continue;
                }
                skips += 1;
                // The day before carries a smaller number, or ends the month
                // before; the day after a larger one.
                let (_, _, _, below) = tibetan_from_fixed(before);
                let (_, _, _, above) = tibetan_from_fixed(after);
                assert_eq!(after, before + 1, "{y} {m}/{number}");
                assert!(below < i64::from(number) || number == 1, "{y} {m}/{number}");
                assert!(
                    above > i64::from(number) || number == 30,
                    "{y} {m}/{number}"
                );
            }
        }
        // About one lunar day in sixty-four is skipped.
        assert!(skips > 1_000, "{skips}");
    }
}

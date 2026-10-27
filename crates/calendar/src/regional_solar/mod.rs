//! The regional solar calendars of India: Tamil, Bengal, Odia and
//! Malayalam (`docs/calendars/regional-solar.md`).
//!
//! Each begins its months at a sankranti, the Sun's entry into a sidereal
//! sign, and differs from the others only in the civil day it puts that
//! sankranti on, the place that day is reckoned at, the sign its year
//! opens with and the era it counts. Sewell and Dikshit (*The Indian
//! Calendar*, 1896, art. 28, p. 12) give the four rules; the *Rashtriya
//! Panchang*'s regional tables (Positional Astronomy Centre, Kolkata;
//! 1992–93 and 2018–19 to 2026–27, 120 month beginnings a region) measured
//! which one each region prints and where:
//!
//! | calendar | rule | place | year | fit |
//! |---|---|---|---|---:|
//! | Tamil | before sunset | Chennai | Saka, from Mesha | 120/120 |
//! | Bengal | the civil day after | Kolkata | Bengali San, from Mesha | 119/120 |
//! | Odia | sunrise to sunrise | Bhubaneswar | Saka, from Mesha | 117/120 |
//! | Malayalam | before three fifths of the daylight | Thiruvananthapuram | Kollam, from Simha | 109/110 (Mesha aside) |
//!
//! The sankrantis are the modern Sun's under Lahiri's ayanamsha on the
//! true basis (with the nutation), as the *Rashtriya Panchang* reckons
//! them, on Indian Standard Time; the days
//! are Gregorian civil days. The tables are computed once by the solar
//! engine ([`Engine`] with a [`YearStart`]) over the SDK's built-in
//! ephemeris, written by `cargo xtask gen calendars` and held to the
//! engine by `check-calendars`, so a date needs no ephemeris at run time
//! and says which frame computed it.
//!
//! ```
//! use teistro_calendar::regional_solar::RegionalSolar;
//! use teistro_calendar::{CalendarSystem, Gregorian};
//! use teistro_core::catalogue::Calendar;
//!
//! // The Tamil year Saka 1946 began on 14 April 2024, 1 Chittirai: the
//! // Mesha sankranti fell at 21:04 on the 13th, after sunset at Chennai.
//! let tamil = RegionalSolar::shipped(Calendar::TamilSolar).expect("shipped");
//! let date = tamil.date_of(Gregorian.to_fixed_ymd(2024, 4, 14)?)?;
//! assert_eq!((date.year, date.month, date.day), (1946, 1, 1));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use std::borrow::Cow;
use std::sync::OnceLock;

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::envelope::CalendarResolution;
use teistro_core::error::Error;
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::time::UtcOffset;

use crate::bikram_sambat::engine::{Engine, YearStart};
use crate::bikram_sambat::{BikramSambat, year_starts};
use crate::date::{
    CalendarCapabilities, CalendarDate, CalendarSystem, EraNumber, check_day, check_no_leap,
    out_of_range,
};
use crate::fixed::FixedDay;
use crate::gregorian::fixed_from_gregorian;
use crate::solar::{MonthStartRule, SolarModel};

#[rustfmt::skip]
mod generated;

/// The Gregorian years the shipped tables' years begin in: every year
/// opening from 1800 to 2398, so the last ends inside the built-in
/// ephemeris's span (1800 to 2399).
pub const GREGORIAN_SPAN: (i32, i32) = (1800, 2398);

/// Indian Standard Time, the clock the *Rashtriya Panchang* prints and
/// every regional table here is reckoned on, before 1947 as well.
pub const IST: UtcOffset = UtcOffset::literal(5, 30, 0);

/// The clock the engine reads, a static so an [`Engine`] can borrow it.
static IST_CLOCK: UtcOffset = IST;

/// A regional solar calendar's definition: the place its days are
/// reckoned at, the month-start rule, the year's first sign and era.
///
/// ```
/// use teistro_calendar::MonthStartRule;
/// use teistro_calendar::regional_solar::TAMIL;
///
/// assert_eq!(TAMIL.rule, MonthStartRule::BeforeSunset);
/// assert_eq!(TAMIL.place_name, "Chennai");
/// assert_eq!(TAMIL.year_start.gregorian_year(1946), 2024);
/// ```
#[derive(Debug)]
pub struct Region {
    /// The catalogue's key.
    pub calendar: Calendar,
    /// The calendar's English name.
    pub name: &'static str,
    /// The place the day's sunrise and sunset are reckoned at, by name.
    pub place_name: &'static str,
    /// That place.
    pub place: Place,
    /// Which civil day a sankranti begins its month on.
    pub rule: MonthStartRule,
    /// The sign the year opens with and how its years are numbered.
    pub year_start: YearStart,
    /// The era the years count.
    pub era: Era,
}

impl Region {
    /// The solar engine for this calendar over a model: the place, the
    /// rule and the year start, on Indian Standard Time.
    ///
    /// ```
    /// use teistro_calendar::gregorian::gregorian_from_fixed;
    /// use teistro_calendar::regional_solar::MALAYALAM;
    /// use teistro_siddhanta::SuryaSiddhanta;
    ///
    /// // Over the Surya Siddhanta's Sun, Kollam 1200 opens in August 2024.
    /// let text = SuryaSiddhanta::text();
    /// let row = MALAYALAM.engine(&text).year(1200)?;
    /// assert_eq!(gregorian_from_fixed(row.start).1, 8);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn engine<'a>(&self, model: &'a dyn SolarModel) -> Engine<'a> {
        Engine::new(model, &IST_CLOCK, self.place, self.rule).with_year_start(self.year_start)
    }

    /// The sign a month of the year is entered by, 0 for Mesha, or `None`
    /// for a month outside 1 to 12: the month's name in every one of these
    /// calendars, whatever the region calls it.
    ///
    /// ```
    /// use teistro_calendar::regional_solar::{MALAYALAM, TAMIL};
    ///
    /// // Chittirai is Mesha's month; Chingam, Malayalam's first, is Simha's.
    /// assert_eq!(TAMIL.sign_of_month(1), Some(0));
    /// assert_eq!(MALAYALAM.sign_of_month(1), Some(4));
    /// assert_eq!(MALAYALAM.sign_of_month(9), Some(0));
    /// assert_eq!(TAMIL.sign_of_month(13), None);
    /// ```
    #[must_use]
    pub fn sign_of_month(&self, month: u8) -> Option<u8> {
        (1..=12)
            .contains(&month)
            .then(|| self.year_start.sign_of(usize::from(month - 1)))
    }
}

/// Chennai, the place the Tamil calendar's sunsets are reckoned at.
pub const CHENNAI: Place = Place::new(
    Latitude::literal(13.0827),
    Longitude::literal(80.2707),
    Altitude::literal(0.0),
);

/// Kolkata, the place the Bengal calendar is printed for; its rule reads
/// no sunrise, so the place names the calendar and moves nothing.
pub const KOLKATA: Place = Place::new(
    Latitude::literal(22.5726),
    Longitude::literal(88.3639),
    Altitude::literal(0.0),
);

/// Bhubaneswar, the place the Odia calendar's sunrises are reckoned at.
pub const BHUBANESWAR: Place = Place::new(
    Latitude::literal(20.2961),
    Longitude::literal(85.8245),
    Altitude::literal(0.0),
);

/// Thiruvananthapuram, the place the Malayalam calendar's daylight is
/// reckoned at.
pub const THIRUVANANTHAPURAM: Place = Place::new(
    Latitude::literal(8.5241),
    Longitude::literal(76.9366),
    Altitude::literal(0.0),
);

/// The Tamil calendar: a sankranti before sunset at Chennai begins its
/// month that day, after sunset the next (Sewell and Dikshit's Tamil
/// rule, art. 28). The year opens at Mesha (Chittirai) and counts the
/// solar Saka year, which a Tamil solar panchang printed at Madras
/// numbered (art. 28, n. 3), here as years elapsed: Saka = CE − 78.
pub static TAMIL: Region = Region {
    calendar: Calendar::TamilSolar,
    name: "Tamil",
    place_name: "Chennai",
    place: CHENNAI,
    rule: MonthStartRule::BeforeSunset,
    year_start: YearStart {
        sign: 0,
        gregorian_offset: 78,
    },
    era: Era::Shaka,
};

/// The Bengal calendar, West Bengal's: a month begins on the civil day
/// after the sankranti's, or the day after next when it falls after
/// midnight (Sewell and Dikshit's Bengal rule, art. 28), which is the
/// civil day after the one it falls in. Not Bangladesh's revised calendar
/// (`calendar.BANGLADESHI`). The year opens at Mesha (Vaisakha) and counts
/// the Bengali San: Bengali San 1300 began in 1893 (art. 71, p. 43) and
/// 1425 on 15 April 2018 (*Indian Astronomical Ephemeris* 2018), so
/// Bengali San = CE − 593.
pub static BENGAL: Region = Region {
    calendar: Calendar::BengalSolar,
    name: "Bengal",
    place_name: "Kolkata",
    place: KOLKATA,
    rule: MonthStartRule::FollowingDay,
    year_start: YearStart {
        sign: 0,
        gregorian_offset: 593,
    },
    era: Era::Bengali,
};

/// The Odia calendar: a month begins on the day, sunrise to sunrise at
/// Bhubaneswar, that holds the sankranti (Sewell and Dikshit's Orissa
/// rule, "the same day as the sankranti", art. 28, with the day counted
/// from sunrise to sunrise). The year opens at Mesha, as the *Rashtriya
/// Panchang*'s Odisha column does, and counts the solar Saka year; the
/// Amli and Vilayati eras open elsewhere (art. 71) and are not counted.
pub static ODIA: Region = Region {
    calendar: Calendar::OdiaSolar,
    name: "Odia",
    place_name: "Bhubaneswar",
    place: BHUBANESWAR,
    rule: MonthStartRule::SunriseToSunrise,
    year_start: YearStart {
        sign: 0,
        gregorian_offset: 78,
    },
    era: Era::Shaka,
};

/// The Malayalam calendar, South Malabar's: a sankranti in the first three
/// fifths of the daylight at Thiruvananthapuram begins its month that
/// day, later the next (Sewell and Dikshit's Malabar rule, art. 28), for
/// every sign, Mesha included (crux C764). The year opens at Simha
/// (Chingam) and counts the Kollam era: Kollam 1070 was A.D. 1894–95
/// (art. 71, p. 45), so Kollam = CE − 824 from Chingam.
pub static MALAYALAM: Region = Region {
    calendar: Calendar::MalayalamSolar,
    name: "Malayalam",
    place_name: "Thiruvananthapuram",
    place: THIRUVANANTHAPURAM,
    rule: MonthStartRule::BeforeAparahna,
    year_start: YearStart {
        sign: 4,
        gregorian_offset: 824,
    },
    era: Era::Kollam,
};

/// The four calendars, in the catalogue's order.
pub static REGIONS: [&Region; 4] = [&TAMIL, &BENGAL, &ODIA, &MALAYALAM];

/// A calendar's table: what `cargo xtask gen calendars` writes from the
/// engine and `check-calendars` holds to it.
///
/// ```
/// use teistro_calendar::regional_solar::RegionalSolar;
/// use teistro_core::catalogue::Calendar;
///
/// let bengal = RegionalSolar::shipped(Calendar::BengalSolar).expect("shipped");
/// let table = bengal.table();
/// assert_eq!(table.first_year, 1800 - 593);
/// assert_eq!(table.month_lengths.len(), 599);
/// assert!(table.frame.contains("rule FOLLOWING_DAY"));
/// ```
#[derive(Debug)]
pub struct Table {
    /// The first year of the rows, in the calendar's era.
    pub first_year: i32,
    /// The first year's first day, as a Gregorian date.
    pub first_start: (i32, u8, u8),
    /// Month lengths per year, from the year's first month.
    pub month_lengths: &'static [[u8; 12]],
    /// The frame the rows were computed in: model, clock, place and rule,
    /// the model a computed date names.
    pub frame: &'static str,
}

/// A regional solar calendar over its table.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::regional_solar::RegionalSolar;
/// use teistro_calendar::CalendarSystem;
/// use teistro_core::catalogue::{Calendar, Era};
///
/// // 1 Chingam, Kollam 1201: 17 August 2025.
/// let malayalam = RegionalSolar::shipped(Calendar::MalayalamSolar).expect("shipped");
/// assert_eq!(malayalam.to_fixed_ymd(1201, 1, 1)?, fixed_from_gregorian(2025, 8, 17));
/// let date = malayalam.date_of(fixed_from_gregorian(2025, 8, 17))?;
/// assert_eq!(date.era.map(|era| era.era), Some(Era::Kollam));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Debug)]
pub struct RegionalSolar {
    region: &'static Region,
    table: &'static Table,
    year_starts: Vec<FixedDay>,
}

/// The shipped calendars, built on first use, in [`REGIONS`]' order.
static SHIPPED: [OnceLock<RegionalSolar>; 4] = [
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
];

impl RegionalSolar {
    /// The shipped calendar a key names, or `None` for any other key.
    ///
    /// ```
    /// use teistro_calendar::regional_solar::RegionalSolar;
    /// use teistro_core::catalogue::Calendar;
    ///
    /// assert!(RegionalSolar::shipped(Calendar::OdiaSolar).is_some());
    /// assert!(RegionalSolar::shipped(Calendar::BikramSambat).is_none());
    /// ```
    #[must_use]
    pub fn shipped(calendar: Calendar) -> Option<&'static RegionalSolar> {
        let index = REGIONS
            .iter()
            .position(|region| region.calendar == calendar)?;
        let (region, table, slot) = (
            REGIONS.get(index)?,
            generated::TABLES.get(index)?,
            SHIPPED.get(index)?,
        );
        Some(slot.get_or_init(|| RegionalSolar::over(region, table)))
    }

    /// A calendar over a definition and a table.
    ///
    /// ```
    /// use teistro_calendar::regional_solar::{RegionalSolar, Table, TAMIL};
    /// use teistro_calendar::CalendarSystem;
    ///
    /// // One year of a table: 365 days from 11 April 1800.
    /// static ONE: Table = Table {
    ///     first_year: 1722,
    ///     first_start: (1800, 4, 11),
    ///     month_lengths: &[[31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30]],
    ///     frame: "a test",
    /// };
    /// let calendar = RegionalSolar::over(&TAMIL, &ONE);
    /// assert_eq!(calendar.years(), (1722, 1722));
    /// assert!(calendar.to_fixed_ymd(1723, 1, 1).is_err());
    /// ```
    #[must_use]
    pub fn over(region: &'static Region, table: &'static Table) -> RegionalSolar {
        let (y, m, d) = table.first_start;
        RegionalSolar {
            region,
            table,
            year_starts: year_starts(table.month_lengths, 0, fixed_from_gregorian(y, m, d)),
        }
    }

    /// The definition.
    ///
    /// ```
    /// use teistro_calendar::regional_solar::RegionalSolar;
    /// use teistro_core::catalogue::Calendar;
    ///
    /// let odia = RegionalSolar::shipped(Calendar::OdiaSolar).expect("shipped");
    /// assert_eq!(odia.region().place_name, "Bhubaneswar");
    /// ```
    #[must_use]
    pub const fn region(&self) -> &'static Region {
        self.region
    }

    /// The table.
    ///
    /// ```
    /// use teistro_calendar::regional_solar::RegionalSolar;
    /// use teistro_core::catalogue::Calendar;
    ///
    /// let tamil = RegionalSolar::shipped(Calendar::TamilSolar).expect("shipped");
    /// assert_eq!(tamil.table().first_start, (1800, 4, 11));
    /// ```
    #[must_use]
    pub const fn table(&self) -> &'static Table {
        self.table
    }

    /// The first and last year of the table, in the calendar's era.
    ///
    /// ```
    /// use teistro_calendar::regional_solar::RegionalSolar;
    /// use teistro_core::catalogue::Calendar;
    ///
    /// let malayalam = RegionalSolar::shipped(Calendar::MalayalamSolar).expect("shipped");
    /// assert_eq!(malayalam.years(), (976, 1574));
    /// ```
    #[must_use]
    pub fn years(&self) -> (i32, i32) {
        let count = i32::try_from(self.table.month_lengths.len()).unwrap_or(0);
        (self.table.first_year, self.table.first_year + count - 1)
    }

    /// A year's first day, or `None` outside the table.
    ///
    /// ```
    /// use teistro_calendar::gregorian::fixed_from_gregorian;
    /// use teistro_calendar::regional_solar::RegionalSolar;
    /// use teistro_core::catalogue::Calendar;
    ///
    /// // Pohela Boishakh 1431: 14 April 2024.
    /// let bengal = RegionalSolar::shipped(Calendar::BengalSolar).expect("shipped");
    /// assert_eq!(bengal.year_start(1431), Some(fixed_from_gregorian(2024, 4, 14)));
    /// assert_eq!(bengal.year_start(3000), None);
    /// ```
    #[must_use]
    pub fn year_start(&self, year: i32) -> Option<FixedDay> {
        let index = usize::try_from(year - self.table.first_year).ok()?;
        if index >= self.table.month_lengths.len() {
            return None;
        }
        self.year_starts.get(index).copied()
    }

    /// The sign a date's month is entered by, 0 for Mesha.
    ///
    /// ```
    /// use teistro_calendar::gregorian::fixed_from_gregorian;
    /// use teistro_calendar::regional_solar::RegionalSolar;
    /// use teistro_calendar::CalendarSystem;
    /// use teistro_core::catalogue::Calendar;
    ///
    /// // Chingam is Simha's month.
    /// let malayalam = RegionalSolar::shipped(Calendar::MalayalamSolar).expect("shipped");
    /// let date = malayalam.date_of(fixed_from_gregorian(2025, 8, 20))?;
    /// assert_eq!(malayalam.sign(&date), Some(4));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn sign(&self, date: &CalendarDate) -> Option<u8> {
        self.region.sign_of_month(date.month)
    }

    fn row(&self, year: i32) -> Result<&'static [u8; 12], Error> {
        let (first, last) = self.years();
        usize::try_from(year - first)
            .ok()
            .and_then(|i| self.table.month_lengths.get(i))
            .ok_or_else(|| {
                out_of_range(format!(
                    "the {} solar calendar covers {first} to {last}, not {year}",
                    self.region.name
                ))
                .with_field("year")
            })
    }
}

impl CalendarSystem for RegionalSolar {
    fn id(&self) -> Calendar {
        self.region.calendar
    }

    fn capabilities(&self) -> CalendarCapabilities {
        let region: &'static Region = self.region;
        let first = self.year_starts.first().copied().unwrap_or_default();
        let last = self
            .year_starts
            .last()
            .copied()
            .unwrap_or_default()
            .plus_days(-1);
        CalendarCapabilities {
            range: (first, last),
            needs_place: false,
            needs_ephemeris: false,
            eras: core::slice::from_ref(&region.era),
        }
    }

    fn date_of(&self, fixed: FixedDay) -> Result<CalendarDate, Error> {
        let (first_year, last_year) = self.years();
        let name = self.region.name;
        let index = match self.year_starts.binary_search(&fixed) {
            Ok(i) => i,
            Err(0) => {
                return Err(out_of_range(format!(
                    "{fixed} is before the first day of {first_year}, where the {name} solar calendar's table begins"
                )));
            }
            Err(i) => i - 1,
        };
        let after = || {
            out_of_range(format!(
                "{fixed} is after the last day of {last_year}, where the {name} solar calendar's table ends"
            ))
        };
        let (Some(start), Some(months)) = (
            self.year_starts.get(index),
            self.table.month_lengths.get(index),
        ) else {
            return Err(after());
        };
        let year = first_year + i32::try_from(index).unwrap_or(0);
        let label = BikramSambat::label(months, start.days_until(fixed)).ok_or_else(after)?;
        Ok(CalendarDate {
            calendar: self.region.calendar,
            year,
            month: label.month,
            day: label.day,
            era: Some(EraNumber {
                era: self.region.era,
                year,
            }),
            resolution: CalendarResolution::Computed {
                model: Cow::Borrowed(self.table.frame),
            },
            leap: None,
        })
    }

    fn fixed_of(&self, date: &CalendarDate) -> Result<FixedDay, Error> {
        check_no_leap(date)?;
        let length = self.month_length(date.year, date.month)?;
        check_day(
            self.region.calendar,
            date.year,
            date.month,
            date.day,
            12,
            length,
        )?;
        let months = self.row(date.year)?;
        let start = self.year_start(date.year).unwrap_or_default();
        let before: i64 = months
            .iter()
            .take(usize::from(date.month - 1))
            .map(|d| i64::from(*d))
            .sum();
        Ok(start.plus_days(before + i64::from(date.day) - 1))
    }

    fn month_length(&self, year: i32, month: u8) -> Result<u8, Error> {
        let months = self.row(year)?;
        let length = months
            .get(usize::from(month.saturating_sub(1)))
            .copied()
            .unwrap_or(0);
        check_day(self.region.calendar, year, month, 1, 12, length)?;
        Ok(length)
    }

    fn is_leap(&self, year: i32) -> bool {
        self.row(year)
            .is_ok_and(|months| months.iter().map(|d| u32::from(*d)).sum::<u32>() == 366)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests;

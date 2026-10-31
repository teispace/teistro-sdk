//! Ashauch (*āśauca*): the period of impurity after a birth or a death,
//! from when, until when, and by which clause (`03-design/ashauch.md`).
//!
//! The span is a count of sunrise days, not a lunar rule. A day is the
//! sunrise to the next (the Mitākṣarā on Yājñavalkya III.18: "a day or a
//! night named here means a day and a night"); the day an event falls in
//! is day 1 however little of it is left (on III.20); an event at night
//! belongs to the day before or the day after by the table's
//! [`NightRule`]; and `n` days end at the sunrise that begins day `n + 1`
//! (*Garuḍa-purāṇa* 2.5.19, *Dharmasindhu* pp. 305–306).
//!
//! Everything a tradition differs on is data in an [`AshauchTable`]: the
//! night rule, each band of the deceased's age, each relation's length,
//! the late-news tables and the overlap rules, every row with its clause
//! and source. The one input the texts key on caste, the full period
//! (10, 12, 15 or 30 days), is a plain number the family gives
//! ([`AshauchRequest::full_days`]); nothing here asks or stores a caste.

mod dharmasindhu;
mod household;
mod kin;
mod mitakshara;
mod rite;

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::{Kind, write_in_full};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::LocalClock;

use crate::festival::FestivalDay;

pub use household::{
    Household, HouseholdAnswer, HouseholdEvent, HouseholdObserver, HouseholdPeriods, HouseholdTie,
    MOST_TIES,
};
pub use kin::{
    Kin, KinAsked, KinDegree, KinKeeps, KinObserver, KinRow, KinStep, KinTable, resolve,
};
pub use rite::{Rite, RiteAnswer, RiteAsked, RiteBeginning, RiteRow, RiteTable, RiteVerdict, rite};

/// The full period when the request gives none: ten days, for every
/// family (Aṅgiras in the Mitākṣarā on Yājñavalkya III.22).
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// let found = ashauch(&table, &asked(death(10.25), Relation::Child), &days, &UtcOffset::UTC)?;
/// use teistro_panchanga::ashauch::FULL_DAYS;
///
/// assert_eq!(found.days, Some(FULL_DAYS));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub const FULL_DAYS: u8 = 10;

/// The longest count a table row or a request may name, in days.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::MOST_DAYS;
///
/// let mut long = asked(death(10.25), Relation::Child);
/// long.full_days = Some(MOST_DAYS + 1);
/// assert_eq!(long.check().unwrap_err().field(), Some("fullDays"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub const MOST_DAYS: u8 = 60;

/// A month of the age bands, in days.
const MONTH_DAYS: f64 = 30.0;

/// A year of the age bands, in days.
const YEAR_DAYS: f64 = 365.2425;

/// Two sunrises a gapless run joins differ by less than this, in days:
/// the run's days are built one after the other, so its sunrises are the
/// same numbers.
const JOINED: f64 = 1e-6;

/// Which day an event at night belongs to (the Mitākṣarā on Yājñavalkya
/// III.20: three *kalpas*, "arranged by the custom of the land").
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::{DayOneBy, NightRule};
///
/// // At 10.9, in the night's last third: the day after's.
/// let found = ashauch(&table, &asked(death(10.9), Relation::Child), &days, &UtcOffset::UTC)?;
/// assert_eq!(found.day_one_by, DayOneBy::Night { rule: NightRule::Thirds });
/// assert_eq!((found.day_one, found.end.get()), (date(11), 21.0));
/// // From sunrise to sunrise, the day before's.
/// let sunrise = AshauchTable { night: NightRule::Sunrise, ..table };
/// let found = ashauch(&sunrise, &asked(death(10.9), Relation::Child), &days, &UtcOffset::UTC)?;
/// assert_eq!(found.day_one, date(10));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NightRule {
    /// The night, sunset to sunrise, in three: the first two thirds
    /// belong to the day before, the last to the day after (the
    /// Mitākṣarā's second kalpa; *Dharmasindhu* p. 304 first).
    Thirds,
    /// Before the night's middle, the day before; after it, the day
    /// after (*Dharmasindhu* p. 304, "यद्वार्धरात्रात्प्राक्").
    HalfNight,
    /// As `HALF_NIGHT`, at the civil midnight of the place's clock.
    CivilMidnight,
    /// Any time before sunrise, the day before: the day is sunrise to
    /// sunrise (the third kalpa, "prāg udayāt").
    Sunrise,
}

/// A span the texts name.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::Length;
///
/// // A maternal grandmother's, a pakṣiṇī: that day, its night, and the next
/// // day to nightfall.
/// let found = ashauch(&table, &asked(death(10.25), Relation::MaternalGrandmother), &days, &UtcOffset::UTC)?;
/// assert_eq!((found.length, found.end.get()), (Length::Pakshini, 11.5));
/// // The ruler's, sajyotiḥ: by day, to nightfall.
/// let found = ashauch(&table, &asked(death(10.25), Relation::Subject), &days, &UtcOffset::UTC)?;
/// assert_eq!((found.length, found.end.get()), (Length::Sajyotih, 10.5));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchLength"))]
#[serde(tag = "length", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Length {
    /// *Sadyaḥ-śauca*: pure by a bath, at once.
    Bath,
    /// *Sajyotiḥ* (*Dharmasindhu* p. 303): an event by day ends at
    /// nightfall, one at night at daybreak.
    Sajyotih,
    /// *Pakṣiṇī* (*Dharmasindhu* p. 303): by day, that day, its night and
    /// the next day to nightfall; at night, by the table's
    /// [`PakshiniNight`].
    Pakshini,
    /// A count of sunrise days, the first day the first.
    Days {
        /// How many, 1 to [`MOST_DAYS`].
        days: u8,
    },
    /// The request's full period, [`AshauchRequest::full_days`].
    Full,
}

impl Length {
    /// Its days as a count, `full` standing for [`Length::Full`]; none for
    /// the lengths that are not one.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::Length;
    ///
    /// assert_eq!(Length::Days { days: 3 }.count(10), Some(3));
    /// assert_eq!(Length::Full.count(12), Some(12));
    /// assert_eq!(Length::Pakshini.count(10), None);
    /// ```
    #[must_use]
    pub const fn count(self, full: u8) -> Option<u8> {
        match self {
            Length::Days { days } => Some(days),
            Length::Full => Some(full),
            Length::Bath | Length::Sajyotih | Length::Pakshini => None,
        }
    }

    /// Its weight in half days, which the overlap rules compare: a bath
    /// none, a sajyotih one, a pakṣiṇī three.
    fn half_days(self, full: u8) -> u16 {
        match self {
            Length::Bath => 0,
            Length::Sajyotih => 1,
            Length::Pakshini => 3,
            Length::Days { days } => 2 * u16::from(days),
            Length::Full => 2 * u16::from(full),
        }
    }
}

/// Where a *pakṣiṇī* begun at night ends (*Dharmasindhu* p. 303).
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::PakshiniNight;
///
/// // A pakṣiṇī begun at night, 10.75: to the second sunrise, or to the
/// // next day's nightfall.
/// let asked = asked(death(10.75), Relation::MaternalGrandmother);
/// assert_eq!(ashauch(&table, &asked, &days, &UtcOffset::UTC)?.end.get(), 12.0);
/// let some = AshauchTable { pakshini_at_night: PakshiniNight::NextNightfall, ..table };
/// assert_eq!(ashauch(&some, &asked, &days, &UtcOffset::UTC)?.end.get(), 11.5);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PakshiniNight {
    /// That night, the next day and its night: the second sunrise.
    #[default]
    SecondSunrise,
    /// "केचित्": the next day's nightfall, as by day.
    NextNightfall,
}

/// A son or a daughter, which the texts' bands and the mother's further
/// bar read.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::Sex;
///
/// // The mother's further bar after a son, to the 30th day.
/// let birth = AshauchEvent::Birth { at: jd(10.25), child: Some(Sex::Male) };
/// let found = ashauch(&table, &asked(birth, Relation::Mother), &days, &UtcOffset::UTC)?;
/// assert_eq!(found.sex, Some(Sex::Male));
/// assert!(found.rites_from.is_some());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchSex"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Sex {
    /// Male.
    Male,
    /// Female.
    Female,
}

/// Whether the deceased had the upanayana.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::{Sex, Upanayana};
///
/// // A son of eight, before his upanayana: three days for the father.
/// let deceased = Deceased {
///     born: Some(jd(-2900.0)),
///     sex: Some(Sex::Male),
///     upanayana: Some(Upanayana::NotDone),
///     ..Deceased::default()
/// };
/// let event = AshauchEvent::Death { at: jd(10.25), cremated_at: None, ahitagni: false, deceased };
/// let found = ashauch(&table, &asked(event, Relation::Father), &days, &UtcOffset::UTC)?;
/// assert_eq!(found.days, Some(3));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Upanayana {
    /// Done.
    Done,
    /// Not yet done.
    NotDone,
    /// Not a rite of the deceased's community: the band before it runs to
    /// marriage, or to the 16th year unmarried (*Dharmasindhu* pp.
    /// 300–301).
    NotPerformed,
}

/// The deceased, by the facts the age bands read. Every fact is optional:
/// left out with no `born`, the deceased is an adult who had every rite;
/// left out with a `born`, a fact a band needs is asked for by a refusal
/// rather than guessed.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::{Clause, Sex};
///
/// // A son of two months, cremated: the parents three days by the band.
/// let deceased = Deceased { born: Some(jd(40.25)), sex: Some(Sex::Male), ..Deceased::default() };
/// let event = AshauchEvent::Death { at: jd(100.25), cremated_at: None, ahitagni: false, deceased };
/// let found = ashauch(&table, &asked(event, Relation::Father), &days, &UtcOffset::UTC)?;
/// assert_eq!(found.days, Some(3));
/// assert!(matches!(found.clause, Clause::Band { row: 1, .. }));
/// // With a birth given, a fact a band needs is asked for, not guessed.
/// let unsexed = Deceased { born: Some(jd(40.25)), ..Deceased::default() };
/// let event = AshauchEvent::Death { at: jd(100.25), cremated_at: None, ahitagni: false, deceased: unsexed };
/// assert!(ashauch(&table, &asked(event, Relation::Father), &days, &UtcOffset::UTC).is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Deceased {
    /// When the deceased was born, from which the age is read in days of
    /// 24 hours, months of 30 days and years of 365.2425.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub born: Option<JulianDay<Utc>>,
    /// The deceased's sex; the bands differ, and a parent's sex picks the
    /// mother-and-father overlap rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<Sex>,
    /// Whether the *cūḍākaraṇa* was done; left out, read from the age
    /// (past the third year, *Dharmasindhu* p. 300).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chudakarana: Option<bool>,
    /// The upanayana.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upanayana: Option<Upanayana>,
    /// A daughter's betrothal (*vāgdāna*).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub betrothed: Option<bool>,
    /// Whether the deceased was married.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub married: Option<bool>,
    /// Buried rather than cremated, as an infant may be.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub buried: bool,
}

/// What happened.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::Clause;
///
/// // A loss in the 5th month: the mother five days, the father three.
/// let loss = AshauchEvent::Loss { at: jd(10.25), month: 5 };
/// let mother = ashauch(&table, &asked(loss, Relation::Mother), &days, &UtcOffset::UTC)?;
/// let father = ashauch(&table, &asked(loss, Relation::Father), &days, &UtcOffset::UTC)?;
/// assert_eq!((mother.days, father.days), (Some(5), Some(3)));
/// assert!(matches!(mother.clause, Clause::Loss { row: 2, .. }));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(
    tag = "event",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AshauchEvent {
    /// A birth from the 7th month on, live or still.
    Birth {
        /// When, a UTC Julian day.
        at: JulianDay<Utc>,
        /// The child's sex, which sets the mother's further bar
        /// (*Dharmasindhu* p. 299); left out, none is reported.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        child: Option<Sex>,
    },
    /// A loss before the 7th month.
    Loss {
        /// When.
        at: JulianDay<Utc>,
        /// The month of the pregnancy, 1 to 6.
        month: u8,
    },
    /// A death.
    Death {
        /// When.
        at: JulianDay<Utc>,
        /// The cremation, from which the count starts for an
        /// *āhitāgni*.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cremated_at: Option<JulianDay<Utc>>,
        /// Whether the deceased kept the sacred fires: the count starts
        /// at the cremation (Aṅgiras and Paiṭhīnasi in the Mitākṣarā on
        /// Yājñavalkya III.20).
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        ahitagni: bool,
        /// The deceased.
        #[serde(default)]
        deceased: Deceased,
    },
}

impl AshauchEvent {
    /// When it happened.
    ///
    /// ```
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::ashauch::AshauchEvent;
    ///
    /// let birth = AshauchEvent::Birth { at: JulianDay::literal(2_460_000.75), child: None };
    /// assert_eq!(birth.at().get(), 2_460_000.75);
    /// ```
    #[must_use]
    pub const fn at(&self) -> JulianDay<Utc> {
        match *self {
            AshauchEvent::Birth { at, .. }
            | AshauchEvent::Loss { at, .. }
            | AshauchEvent::Death { at, .. } => at,
        }
    }

    /// A birth's (a loss's too) or a death's.
    ///
    /// ```
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::ashauch::{AshauchEvent, AshauchKind};
    ///
    /// // A loss is a birth's impurity.
    /// let loss = AshauchEvent::Loss { at: JulianDay::literal(2_460_000.75), month: 2 };
    /// assert_eq!(loss.kind(), AshauchKind::Birth);
    /// ```
    #[must_use]
    pub const fn kind(&self) -> AshauchKind {
        match self {
            AshauchEvent::Birth { .. } | AshauchEvent::Loss { .. } => AshauchKind::Birth,
            AshauchEvent::Death { .. } => AshauchKind::Death,
        }
    }

    /// The instant the count starts from: the cremation for an
    /// āhitāgni's death, otherwise the event.
    ///
    /// ```
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::ashauch::{AshauchEvent, Deceased};
    ///
    /// // An āhitāgni's count starts at the cremation.
    /// let death = AshauchEvent::Death {
    ///     at: JulianDay::literal(2_460_000.25),
    ///     cremated_at: Some(JulianDay::literal(2_460_001.4)),
    ///     ahitagni: true,
    ///     deceased: Deceased::default(),
    /// };
    /// assert_eq!(death.counted_from().get(), 2_460_001.4);
    /// ```
    #[must_use]
    pub const fn counted_from(&self) -> JulianDay<Utc> {
        match *self {
            AshauchEvent::Death {
                cremated_at: Some(cremated),
                ahitagni: true,
                ..
            } => cremated,
            _ => self.at(),
        }
    }

    /// The sex the answer repeats: the deceased's, or the child's.
    const fn sex(&self) -> Option<Sex> {
        match *self {
            AshauchEvent::Birth { child, .. } => child,
            AshauchEvent::Loss { .. } => None,
            AshauchEvent::Death { deceased, .. } => deceased.sex,
        }
    }
}

/// Whose impurity: a birth's (*sūtaka*, a loss's too) or a death's.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// let found = ashauch(&table, &asked(death(10.25), Relation::Child), &days, &UtcOffset::UTC)?;
/// use teistro_panchanga::ashauch::AshauchKind;
///
/// assert_eq!(found.kind, AshauchKind::Death);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AshauchKind {
    /// *Janana*, *sūtaka*.
    Birth,
    /// *Mṛta*, *śāva*.
    Death,
}

/// Who the observer is to the one born or deceased, as the app resolves
/// it.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// // A sagotra keeps one night, a samānodaka three.
/// let sagotra = ashauch(&table, &asked(death(10.25), Relation::Sagotra), &days, &UtcOffset::UTC)?;
/// let samanodaka = ashauch(&table, &asked(death(10.25), Relation::Samanodaka), &days, &UtcOffset::UTC)?;
/// assert_eq!((sagotra.days, samanodaka.days), (Some(1), Some(3)));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchRelation"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Relation {
    /// The mother of the born or the deceased.
    Mother,
    /// The father.
    Father,
    /// A son or daughter of the deceased.
    Child,
    /// The deceased's wife or husband.
    Spouse,
    /// A co-wife of the deceased.
    CoWife,
    /// A step-child: the deceased was the observer's step-mother.
    StepChild,
    /// A *sapiṇḍa* to the 7th generation, not named above.
    Sapinda,
    /// A *samānodaka*, the 8th generation to the 14th.
    Samanodaka,
    /// A *sagotra* beyond.
    Sagotra,
    /// A married daughter, for her parent's death.
    MarriedDaughter,
    /// A married brother or sister.
    MarriedSibling,
    /// The deceased was the observer's maternal grandfather.
    MaternalGrandfather,
    /// The deceased was the observer's maternal grandmother.
    MaternalGrandmother,
    /// The deceased was the observer's maternal uncle.
    MaternalUncle,
    /// A son-in-law: the deceased was his father- or mother-in-law.
    SonInLaw,
    /// A parent-in-law: the deceased was the observer's son-in-law.
    ParentInLaw,
    /// A pupil: the deceased was the observer's *ācārya*.
    Pupil,
    /// The deceased was the observer's ācārya's wife or son.
    AcharyasFamily,
    /// An adopted son and his natural or adoptive parents, to each other.
    Adoption,
    /// One of the three *bandhus*: cousins through a father's or mother's
    /// sister or a mother's brother.
    Bandhu,
    /// A subject: the deceased was the ruler of the land.
    Subject,
    /// A sapiṇḍa of an ascetic (*yati*).
    OfAscetic,
    /// The deceased was the observer's mother's sister.
    MaternalAunt,
    /// The deceased was the observer's married father's sister.
    PaternalAunt,
    /// The deceased was the observer's maternal uncle's wife.
    MaternalUnclesWife,
    /// The deceased was the observer's sister's son (*bhāgineya*).
    SistersSon,
    /// The deceased was the observer's daughter's son (*dauhitra*).
    DaughtersSon,
    /// The deceased was the observer's wife's brother (*śyālaka*).
    WifesBrother,
}

/// Where the death was, to the observer: the rows that turn on it.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// // A married sister dying in one's house is three days, in another
/// // village one.
/// let mut asked = asked(death(10.25), Relation::MarriedSibling);
/// asked.nearness = Nearness::ObserversHouse;
/// assert_eq!(ashauch(&table, &asked, &days, &UtcOffset::UTC)?.days, Some(3));
/// asked.nearness = Nearness::OtherVillage;
/// assert_eq!(ashauch(&table, &asked, &days, &UtcOffset::UTC)?.days, Some(1));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Nearness {
    /// In the observer's house.
    ObserversHouse,
    /// In the observer's village or town.
    #[default]
    SameVillage,
    /// In another village.
    OtherVillage,
}

/// One relation's length, under its conditions.
///
/// ```
/// use teistro_panchanga::ashauch::{AshauchKind, AshauchTable, Length, Relation};
///
/// let table = AshauchTable::dharmasindhu();
/// let child = table
///     .relations
///     .iter()
///     .find(|row| row.relation == Relation::Child && row.kind == AshauchKind::Death)
///     .ok_or("a parent's death")?;
/// assert_eq!((child.length, child.heard_late), (Length::Full, Some(Length::Full)));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchRelationRow"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelationRow {
    /// Whose.
    pub relation: Relation,
    /// A birth's or a death's.
    pub kind: AshauchKind,
    /// Only where the death was, to the observer; any when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nearness: Option<Nearness>,
    /// Only for a married daughter's death (true), or only for any other
    /// (false); either when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub married_daughter: Option<bool>,
    /// Its length.
    pub length: Length,
    /// What news past the period takes, from the hearing; left out, the
    /// late tables answer a full period and a bath any other.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heard_late: Option<Length>,
    /// The clause, with its source.
    pub clause: String,
}

/// A mark of the deceased's age a band ends before.
///
/// ```
/// use teistro_panchanga::ashauch::{AshauchTable, Mark};
///
/// // A son's first band ends at the naming, the 12th day.
/// assert_eq!(AshauchTable::dharmasindhu().bands[0].before, Mark::Day { day: 12 });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchMark"))]
#[serde(tag = "mark", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Mark {
    /// The `day`th day of life, the birth's day the first: the naming's
    /// is the 12th (*Dharmasindhu* p. 300).
    Day {
        /// 1 to 366.
        day: u16,
    },
    /// The `month`th month of life: teething marks the 7th (p. 300).
    Month {
        /// 1 to 24.
        month: u8,
    },
    /// The `year`th year: the 4th begins past "the end of the 3rd".
    Year {
        /// 1 to 20.
        year: u8,
    },
    /// The *cūḍākaraṇa*, or else the 4th year.
    Chudakarana,
    /// The upanayana, or marriage or the 17th year where it is not
    /// performed.
    Upanayana,
    /// A daughter's betrothal, or her marriage.
    Betrothal,
    /// Marriage.
    Marriage,
}

/// What a death before a mark takes (*Dharmasindhu* pp. 300–301).
///
/// ```
/// use teistro_panchanga::ashauch::{AshauchTable, Length, Mark, Sex};
///
/// let table = AshauchTable::dharmasindhu();
/// let band = &table.bands[1];
/// assert_eq!((band.sex, band.before, band.buried), (Sex::Male, Mark::Month { month: 7 }, Some(false)));
/// assert_eq!((band.parents, band.sapindas), (Length::Days { days: 3 }, Length::Days { days: 1 }));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchBand"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Band {
    /// A son's band or a daughter's.
    pub sex: Sex,
    /// The band holds while this mark is not yet reached.
    pub before: Mark,
    /// Only for a burial (true), or only for a cremation (false); either
    /// when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buried: Option<bool>,
    /// Only when the *cūḍākaraṇa* was done (true), or only when it was not
    /// (false), as the *Mitākṣarā* cuts a child's first three years (on
    /// Yā III.23); either when left out. The deceased's `chudakarana`,
    /// read from the age when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chudakarana: Option<bool>,
    /// The parents' length, a step-child's too.
    pub parents: Length,
    /// The sapiṇḍas'.
    pub sapindas: Length,
    /// The samānodakas', when the band names one; a bath otherwise, as
    /// for every relation the band does not name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub samanodakas: Option<Length>,
    /// The clause, with its source.
    pub clause: String,
}

/// A loss before the 7th month, by the month (*Dharmasindhu* p. 299).
///
/// ```
/// use teistro_panchanga::ashauch::{AshauchTable, Length};
///
/// let first = &AshauchTable::dharmasindhu().losses[0];
/// assert_eq!((first.to_month, first.mother, first.others), (3, Length::Days { days: 3 }, Length::Bath));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchLossRow"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LossRow {
    /// The row holds to this month of the pregnancy, the first row whose
    /// month is not passed answering.
    pub to_month: u8,
    /// The mother's length.
    pub mother: Length,
    /// The father's and the sapiṇḍas'; a bath for any other relation.
    pub others: Length,
    /// The clause, with its source.
    pub clause: String,
}

/// Late news, past the period, within some days of the event.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::Clause;
///
/// // A sapiṇḍa's death heard on the 41st day: three days from the hearing.
/// let mut late = asked(death(10.25), Relation::Sapinda);
/// late.heard_at = Some(jd(50.25));
/// let found = ashauch(&table, &late, &days, &UtcOffset::UTC)?;
/// assert_eq!(found.clause, Clause::Late { within_days: 90, abroad: false });
/// assert_eq!((found.start.get(), found.end.get()), (50.25, 53.0));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchLateRow"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LateRow {
    /// Heard within this many days of the event: the texts' months are
    /// of 30 days, a fortnight of 15.
    pub within_days: u16,
    /// The length from the hearing.
    pub length: Length,
}

/// What a period known during another does (*āśauca-saṃpāta*).
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::{Overlapped, Running};
///
/// // A second death known in the first's last night's last quarter: three
/// // days past its end.
/// let first = ashauch(&table, &asked(death(10.25), Relation::Sapinda), &days, &UtcOffset::UTC)?;
/// let mut second = asked(death(19.9), Relation::Sapinda);
/// second.key = String::from("second");
/// second.during = vec![Running::from(&first)];
/// let found = ashauch(&table, &second, &days, &UtcOffset::UTC)?;
/// assert!(matches!(found.overlap, Some(Overlapped::LastNight { days: 3, .. })));
/// assert_eq!(found.end.get(), 23.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchOverlap"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Overlap {
    /// The days a period known in an equal period's last night adds past
    /// its end, and in that night's last quarter (Gautama and Śātātapa
    /// in the Mitākṣarā on Yājñavalkya III.20; *Dharmasindhu* pp.
    /// 305–306): `[2, 3]`; `[2, 2]` adds two throughout, `[0, 0]` none.
    pub last_night: [u8; 2],
    /// Whether a father dying in the mother's period keeps his own full
    /// period. Off, his ends with hers (*Dharmasindhu* p. 306); on, the
    /// Mitākṣarā on III.20 and the *Smṛtyarthasāra*, which DS reports.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub father_keeps_his_own: bool,
    /// Whether a birth known during a shorter death ends with it
    /// ("कश्चित्", *Dharmasindhu* p. 305); "बहवः" say no.
    pub death_absorbs_longer_birth: bool,
}

/// The mother's further bar on rites after a birth period, to the
/// `son`th or `daughter`th day (*Dharmasindhu* p. 299, Paiṭhīnasi).
///
/// ```
/// use teistro_panchanga::ashauch::{AshauchTable, MotherRites};
///
/// assert_eq!(AshauchTable::dharmasindhu().mother_rites, Some(MotherRites { son: 30, daughter: 40 }));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MotherRites {
    /// After a son.
    pub son: u8,
    /// After a daughter.
    pub daughter: u8,
}

/// A tradition, as data: every row with its clause and source. A
/// consumer loads one from JSON and changes any row
/// ([`AshauchTable::dharmasindhu`] is the default).
///
/// ```
/// use teistro_panchanga::ashauch::{AshauchTable, NightRule};
///
/// let table = AshauchTable::dharmasindhu();
/// assert_eq!((table.name.as_str(), table.night), (AshauchTable::DHARMASINDHU, NightRule::Thirds));
/// table.check()?;
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AshauchTable {
    /// Its name, which every answer repeats.
    pub name: String,
    /// Which day a night event belongs to.
    pub night: NightRule,
    /// Where a pakṣiṇī begun at night ends.
    pub pakshini_at_night: PakshiniNight,
    /// The lengths by relation, the first row that matches answering.
    pub relations: Vec<RelationRow>,
    /// The bands of a death before the last rite, in order of age, the
    /// first of the deceased's sex whose mark is not reached answering.
    pub bands: Vec<Band>,
    /// A loss's rows, in order of month.
    pub losses: Vec<LossRow>,
    /// News of a full period past it, by days since the event, in order;
    /// a bath past the last row.
    pub late: Vec<LateRow>,
    /// The same, for an observer abroad.
    pub late_abroad: Vec<LateRow>,
    /// The overlap rules.
    pub overlap: Overlap,
    /// The mother's further bar; none reported when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mother_rites: Option<MotherRites>,
}

/// One request: one event, one observer.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::Clause;
///
/// // Heard on the 16th day of a ten-day period: the days left.
/// let mut heard = asked(death(10.25), Relation::Sapinda);
/// heard.heard_at = Some(jd(15.25));
/// let found = ashauch(&table, &heard, &days, &UtcOffset::UTC)?;
/// assert_eq!((found.clause, found.end.get()), (Clause::Remainder, 20.0));
/// // A family whose period is twelve days.
/// let twelve = AshauchRequest { full_days: Some(12), ..asked(death(10.25), Relation::Sapinda) };
/// assert_eq!(ashauch(&table, &twelve, &days, &UtcOffset::UTC)?.end.get(), 22.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AshauchRequest {
    /// The consumer's own key, which the answer repeats.
    pub key: String,
    /// What happened.
    pub event: AshauchEvent,
    /// Who the observer is to it.
    pub relation: Relation,
    /// When the news reached the observer: the cause is the known event
    /// (the Mitākṣarā on Yājñavalkya III.18). The event's own instant
    /// when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heard_at: Option<JulianDay<Utc>>,
    /// Whether the observer was abroad: a fact the family gives, never
    /// derived, as the texts' tests differ.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub abroad: bool,
    /// Where the death was, to the observer.
    #[serde(default)]
    pub nearness: Nearness,
    /// The family's full period, 1 to [`MOST_DAYS`]; [`FULL_DAYS`] when
    /// left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_days: Option<u8>,
    /// The periods already running for this observer, each from an
    /// earlier answer ([`Running::from`]): the overlap rules read them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub during: Vec<Running>,
    /// Whether the observer lights the pyre. An earlier period takes in
    /// neither the fire-giver's nor the new mother's (*Dharmasindhu* p.
    /// 306).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fire_giver: bool,
}

/// A period already running, as the overlap rules read it: the parts of
/// an earlier [`Ashauch`] they need.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::{Overlapped, Running};
///
/// // A second death in the first's period ends with it.
/// let first = ashauch(&table, &asked(death(10.25), Relation::Sapinda), &days, &UtcOffset::UTC)?;
/// let mut second = asked(death(12.25), Relation::Sapinda);
/// second.during = vec![Running::from(&first)];
/// let found = ashauch(&table, &second, &days, &UtcOffset::UTC)?;
/// assert_eq!(found.overlap, Some(Overlapped::EndsWithEarlier { earlier: String::from("first") }));
/// assert_eq!(found.end, first.end);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchRunning"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Running {
    /// The earlier answer's key, which an overlap names.
    pub key: String,
    /// A birth's or a death's.
    pub kind: AshauchKind,
    /// The earlier request's relation.
    pub relation: Relation,
    /// The deceased's sex, or the child's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sex: Option<Sex>,
    /// When it began.
    pub start: JulianDay<Utc>,
    /// When it ends.
    pub end: JulianDay<Utc>,
    /// Its length.
    pub length: Length,
    /// Its days, when the length is a count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days: Option<u8>,
    /// Where its last-night days begin, when an overlap added them: those
    /// days take in less than the period did (*Dharmasindhu* p. 306).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extended_from: Option<JulianDay<Utc>>,
    /// The days an overlap added past its own end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extended_days: Option<u8>,
}

impl From<&Ashauch> for Running {
    fn from(found: &Ashauch) -> Running {
        let extended = match found.overlap {
            Some(Overlapped::LastNight { from, days, .. }) => Some((from, days)),
            _ => None,
        };
        Running {
            key: found.key.clone(),
            kind: found.kind,
            relation: found.relation,
            sex: found.sex,
            start: found.start,
            end: found.end,
            length: found.length,
            days: found.days,
            extended_from: extended.map(|(from, _)| from),
            extended_days: extended.map(|(_, days)| days),
        }
    }
}

/// Why the first day is the day it is.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// let found = ashauch(&table, &asked(death(10.25), Relation::Child), &days, &UtcOffset::UTC)?;
/// use teistro_panchanga::ashauch::DayOneBy;
///
/// assert_eq!(found.day_one_by, DayOneBy::Daylight);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchDayOneBy"))]
#[serde(tag = "by", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayOneBy {
    /// The instant fell in the day's daylight.
    Daylight,
    /// It fell in the night, and the rule gave the day.
    Night {
        /// The rule.
        rule: NightRule,
    },
}

/// The clause that set the span.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::{Clause, Sex};
///
/// // A child dying inside its birth period: the birth period runs on.
/// let born = AshauchEvent::Birth { at: jd(10.25), child: Some(Sex::Female) };
/// let first = ashauch(&table, &asked(born, Relation::Father), &days, &UtcOffset::UTC)?;
/// let deceased = Deceased { born: Some(jd(10.25)), sex: Some(Sex::Female), ..Deceased::default() };
/// let event = AshauchEvent::Death { at: jd(14.25), cremated_at: None, ahitagni: false, deceased };
/// let found = ashauch(&table, &asked(event, Relation::Father), &days, &UtcOffset::UTC)?;
/// assert_eq!(found.clause, Clause::BirthPeriodRuns);
/// assert_eq!(found.end, first.end);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchClause"))]
#[serde(tag = "clause", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Clause {
    /// A relation's row of the table.
    Relation {
        /// The row's index.
        row: usize,
        /// Its text.
        said: String,
    },
    /// An age band's row.
    Band {
        /// The row's index.
        row: usize,
        /// Its text.
        said: String,
    },
    /// A loss's row.
    Loss {
        /// The row's index.
        row: usize,
        /// Its text.
        said: String,
    },
    /// A child dying inside the birth period: the birth period runs to its
    /// own end, with no death period (*Dharmasindhu* p. 300).
    BirthPeriodRuns,
    /// Heard inside the period: the days left (Manu V.75,
    /// *Dharmasindhu* p. 304).
    Remainder,
    /// Heard past the full period: the late table's row.
    Late {
        /// Within how many days of the event.
        within_days: u16,
        /// Whether the abroad table answered.
        abroad: bool,
    },
    /// Heard past the late table: a bath.
    LateBath,
    /// Heard past the period, by the relation row's own late length
    /// (a parent's or a spouse's death: the full period from the hearing,
    /// however late, Paiṭhīnasi in the Mitākṣarā on III.21, DS p. 304).
    RowHeardLate {
        /// The row's index.
        row: usize,
        /// Its text.
        said: String,
    },
    /// A shorter period heard past its end: a bath (*Dharmasindhu* pp.
    /// 303–304).
    ShortHeardLate,
    /// A birth heard past its period: none, the father a bath (Manu
    /// V.77).
    BirthHeardLate,
}

/// What an overlap did.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// use teistro_panchanga::ashauch::{Overlapped, Running};
///
/// // Known in the first's last night before its last quarter: two days.
/// let first = ashauch(&table, &asked(death(10.25), Relation::Sapinda), &days, &UtcOffset::UTC)?;
/// let mut second = asked(death(19.8), Relation::Sapinda);
/// second.during = vec![Running::from(&first)];
/// let found = ashauch(&table, &second, &days, &UtcOffset::UTC)?;
/// assert!(matches!(found.overlap, Some(Overlapped::LastNight { days: 2, .. })));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchOverlapped"))]
#[serde(tag = "overlap", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Overlapped {
    /// It ends with the earlier period, which is no shorter (Manu V.79).
    EndsWithEarlier {
        /// The earlier answer's key.
        earlier: String,
    },
    /// Known in the earlier equal period's last night: its end and `days`
    /// more (*Dharmasindhu* pp. 305–306).
    LastNight {
        /// The earlier answer's key.
        earlier: String,
        /// The days added.
        days: u8,
        /// The earlier period's own end, where the added days begin.
        from: JulianDay<Utc>,
    },
    /// A mother's death in the father's period: the father's end and a
    /// pakṣiṇī (the Mitākṣarā on III.20, *Dharmasindhu* p. 306).
    MotherAfterFather {
        /// The earlier answer's key.
        earlier: String,
    },
}

/// The answer: the span of impurity, and the clause that set it.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// let found = ashauch(&table, &asked(death(10.25), Relation::Child), &days, &UtcOffset::UTC)?;
/// // Day 1 is the 11th of January; the tenth day's next sunrise ends it.
/// assert_eq!((found.day_one, found.last_day), (date(10), Some(date(19))));
/// assert_eq!((found.start.get(), found.end.get()), (10.25, 20.0));
/// assert_eq!(found.table, AshauchTable::DHARMASINDHU);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ashauch {
    /// The request's key.
    pub key: String,
    /// A birth's or a death's.
    pub kind: AshauchKind,
    /// The request's relation.
    pub relation: Relation,
    /// The deceased's sex, or the child's, as the request gave it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sex: Option<Sex>,
    /// When it began: the event, the cremation, or the hearing.
    pub start: JulianDay<Utc>,
    /// The day it counts from.
    pub day_one: CalendarDate,
    /// Why that day.
    pub day_one_by: DayOneBy,
    /// When it ends: a sunrise, a nightfall, or the start itself for a
    /// bath.
    pub end: JulianDay<Utc>,
    /// The last day it touches, for a calendar to shade; none for a bath.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_day: Option<CalendarDate>,
    /// The length its clause named.
    pub length: Length,
    /// The days, when the length is a count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days: Option<u8>,
    /// The clause.
    pub clause: Clause,
    /// What an overlap did, when it did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlap: Option<Overlapped>,
    /// When the mother may take part in rites again, for the mother of a
    /// birth under a table that reports it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rites_from: Option<JulianDay<Utc>>,
    /// The table's name.
    pub table: String,
}

impl Ashauch {
    /// Where an answer names catalogue members, and of which kind: its
    /// days' calendars and eras. What the boundary writes in full; the
    /// SDK's tests hold it to serde, both ways.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::Ashauch;
    ///
    /// assert_eq!(Ashauch::MEMBERS[0].0, "dayOne.calendar");
    /// ```
    pub const MEMBERS: [(&'static str, Kind); 4] = [
        ("dayOne.calendar", Kind::Calendar),
        ("dayOne.era.era", Kind::Era),
        ("lastDay.calendar", Kind::Calendar),
        ("lastDay.era.era", Kind::Era),
    ];

    /// The answer as JSON, every catalogue member written as its full
    /// key: what the boundary carries and seals.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::quantity::{JulianDay, Utc};
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
    /// # use teistro_panchanga::month;
    /// #
    /// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
    /// # // the (k + 1)th day of 2024.
    /// # let date = |k: u8| {
    /// #     let (mut month, mut left) = (1, k);
    /// #     for length in [31, 29, 31, 30, 31, 30, 31] {
    /// #         if left < length {
    /// #             break;
    /// #         }
    /// #         left -= length;
    /// #         month += 1;
    /// #     }
    /// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
    /// # };
    /// # let days: Vec<FestivalDay> = (0..200u8)
    /// #     .map(|k| FestivalDay {
    /// #         date: date(k),
    /// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #         sunrise: JulianDay::literal(f64::from(k)),
    /// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
    /// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
    /// #         normal: true,
    /// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
    /// #         tithi: Vec::new(),
    /// #         nakshatra: Vec::new(),
    /// #         moonrise: Moonrise::Unsearched,
    /// #         sun_entry: SunEntry::Unsearched,
    /// #         moon_arcs: MoonArcs::Unsearched,
    /// #     })
    /// #     .collect();
    /// # let jd = JulianDay::<Utc>::literal;
    /// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
    /// #
    /// # let table = AshauchTable::dharmasindhu();
    /// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
    /// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
    /// #     key: String::from("first"),
    /// #     event,
    /// #     relation,
    /// #     heard_at: None,
    /// #     abroad: false,
    /// #     nearness: Nearness::SameVillage,
    /// #     full_days: None,
    /// #     during: Vec::new(),
    /// #     fire_giver: false,
    /// # };
    ///
    /// let found = ashauch(&table, &asked(death(10.25), Relation::Child), &days, &UtcOffset::UTC)?;
    /// let json = found.in_full()?;
    /// assert_eq!(json["dayOne"]["calendar"], "calendar.GREGORIAN");
    /// assert_eq!(json["relation"], "CHILD");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INTERNAL` if it does not serialise, which a value this crate
    /// built cannot do.
    pub fn in_full(&self) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(self)
            .map_err(|error| Error::internal(format!("the ashauch did not serialise: {error}")))?;
        for (path, kind) in Self::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        Ok(value)
    }
}

impl AshauchTable {
    /// Refuses a row a count cannot be made from, and tables out of order.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::{AshauchTable, Length};
    ///
    /// let mut table = AshauchTable::dharmasindhu();
    /// table.check()?;
    /// table.relations[0].length = Length::Days { days: 0 };
    /// assert_eq!(table.check().unwrap_err().field(), Some("relations[0].length"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A count of no days or more than [`MOST_DAYS`], a mark out of its
    /// range, a late or loss table out of order.
    pub fn check(&self) -> Result<(), Error> {
        let length = |length: Length, field: String| match length {
            Length::Days { days } if days == 0 || days > MOST_DAYS => Err(Error::invalid_arg(
                format!("a count is 1 to {MOST_DAYS} days, not {days}"),
            )
            .with_field(field)),
            _ => Ok(()),
        };
        for (row, found) in self.relations.iter().enumerate() {
            length(found.length, format!("relations[{row}].length"))?;
            if let Some(late) = found.heard_late {
                length(late, format!("relations[{row}].heardLate"))?;
            }
        }
        for (row, band) in self.bands.iter().enumerate() {
            length(band.parents, format!("bands[{row}].parents"))?;
            length(band.sapindas, format!("bands[{row}].sapindas"))?;
            if let Some(samanodakas) = band.samanodakas {
                length(samanodakas, format!("bands[{row}].samanodakas"))?;
            }
            let fits = match band.before {
                Mark::Day { day } => (1..=366).contains(&day),
                Mark::Month { month } => (1..=24).contains(&month),
                Mark::Year { year } => (1..=20).contains(&year),
                Mark::Chudakarana | Mark::Upanayana | Mark::Betrothal | Mark::Marriage => true,
            };
            if !fits {
                return Err(Error::invalid_arg(format!(
                    "the mark {:?} is outside its range (a day 1 to 366, a month 1 to 24, a year 1 to 20)",
                    band.before
                ))
                .with_field(format!("bands[{row}].before")));
            }
        }
        for (row, loss) in self.losses.iter().enumerate() {
            length(loss.mother, format!("losses[{row}].mother"))?;
            length(loss.others, format!("losses[{row}].others"))?;
        }
        if !self.losses.is_sorted_by(|a, b| a.to_month < b.to_month) {
            return Err(Error::invalid_arg(
                "the loss rows are in order of month, each month named once",
            )
            .with_field("losses"));
        }
        for (name, late) in [("late", &self.late), ("lateAbroad", &self.late_abroad)] {
            for (row, found) in late.iter().enumerate() {
                length(found.length, format!("{name}[{row}].length"))?;
            }
            if !late.is_sorted_by(|a, b| a.within_days < b.within_days) {
                return Err(Error::invalid_arg(
                    "the late rows are in order of days, each named once",
                )
                .with_field(name));
            }
        }
        Ok(())
    }

    /// The longest count any row names, in days.
    fn longest(&self) -> u8 {
        let mut lengths: Vec<Length> = Vec::new();
        for row in &self.relations {
            lengths.push(row.length);
            lengths.extend(row.heard_late);
        }
        for band in &self.bands {
            lengths.extend([band.parents, band.sapindas]);
            lengths.extend(band.samanodakas);
        }
        for loss in &self.losses {
            lengths.extend([loss.mother, loss.others]);
        }
        lengths.extend(
            self.late
                .iter()
                .chain(&self.late_abroad)
                .map(|row| row.length),
        );
        let rites = self
            .mother_rites
            .map_or(0, |rites| rites.son.max(rites.daughter));
        lengths
            .into_iter()
            .filter_map(|length| length.count(0))
            .fold(rites, u8::max)
    }
}

impl AshauchRequest {
    /// Refuses what cannot be counted.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::quantity::{JulianDay, Utc};
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
    /// # use teistro_panchanga::month;
    /// #
    /// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
    /// # // the (k + 1)th day of 2024.
    /// # let date = |k: u8| {
    /// #     let (mut month, mut left) = (1, k);
    /// #     for length in [31, 29, 31, 30, 31, 30, 31] {
    /// #         if left < length {
    /// #             break;
    /// #         }
    /// #         left -= length;
    /// #         month += 1;
    /// #     }
    /// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
    /// # };
    /// # let days: Vec<FestivalDay> = (0..200u8)
    /// #     .map(|k| FestivalDay {
    /// #         date: date(k),
    /// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #         sunrise: JulianDay::literal(f64::from(k)),
    /// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
    /// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
    /// #         normal: true,
    /// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
    /// #         tithi: Vec::new(),
    /// #         nakshatra: Vec::new(),
    /// #         moonrise: Moonrise::Unsearched,
    /// #         sun_entry: SunEntry::Unsearched,
    /// #         moon_arcs: MoonArcs::Unsearched,
    /// #     })
    /// #     .collect();
    /// # let jd = JulianDay::<Utc>::literal;
    /// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
    /// #
    /// # let table = AshauchTable::dharmasindhu();
    /// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
    /// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
    /// #     key: String::from("first"),
    /// #     event,
    /// #     relation,
    /// #     heard_at: None,
    /// #     abroad: false,
    /// #     nearness: Nearness::SameVillage,
    /// #     full_days: None,
    /// #     during: Vec::new(),
    /// #     fire_giver: false,
    /// # };
    ///
    /// let mut early = asked(death(10.25), Relation::Child);
    /// early.heard_at = Some(jd(9.0));
    /// assert_eq!(early.check().unwrap_err().field(), Some("heardAt"));
    /// let loss = asked(AshauchEvent::Loss { at: jd(10.25), month: 7 }, Relation::Mother);
    /// assert_eq!(loss.check().unwrap_err().field(), Some("event.month"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A full period of no days or more than [`MOST_DAYS`], a loss's month
    /// outside 1 to 6, news before the event, a cremation before the
    /// death or an āhitāgni's left out, a birth after the death, an
    /// earlier answer that ends before it starts.
    pub fn check(&self) -> Result<(), Error> {
        if let Some(full) = self.full_days
            && (full == 0 || full > MOST_DAYS)
        {
            return Err(Error::invalid_arg(format!(
                "the full period is 1 to {MOST_DAYS} days, not {full}"
            ))
            .with_field("fullDays"));
        }
        let at = self.event.at();
        match self.event {
            AshauchEvent::Loss { month, .. } if !(1..=6).contains(&month) => {
                return Err(Error::invalid_arg(format!(
                    "a loss is in the 1st to the 6th month, not month {month}; from the 7th it is a birth"
                ))
                .with_field("event.month"));
            }
            AshauchEvent::Death {
                cremated_at: Some(cremated),
                ..
            } if cremated.get() < at.get() => {
                return Err(Error::invalid_arg("the cremation is after the death")
                    .with_field("event.crematedAt"));
            }
            AshauchEvent::Death {
                ahitagni: true,
                cremated_at: None,
                ..
            } => {
                return Err(Error::invalid_arg(
                    "an āhitāgni's period counts from the cremation, so give it",
                )
                .with_field("event.crematedAt"));
            }
            AshauchEvent::Death {
                deceased: Deceased {
                    born: Some(born), ..
                },
                ..
            } if born.get() > at.get() => {
                return Err(Error::invalid_arg("the deceased was born before the death")
                    .with_field("event.deceased.born"));
            }
            _ => {}
        }
        if let Some(heard) = self.heard_at
            && heard.get() < at.get()
        {
            return Err(
                Error::invalid_arg("the news is heard after the event").with_field("heardAt")
            );
        }
        for (index, earlier) in self.during.iter().enumerate() {
            if earlier.end.get() < earlier.start.get() {
                return Err(Error::invalid_arg("an earlier period ends after it starts")
                    .with_field(format!("during[{index}].end")));
            }
        }
        Ok(())
    }

    /// A deceased's birth near enough to the death that its birth period
    /// may still run.
    fn born_lately(&self) -> Option<JulianDay<Utc>> {
        match self.event {
            AshauchEvent::Death {
                at,
                deceased: Deceased {
                    born: Some(born), ..
                },
                ..
            } if at.get() - born.get() < f64::from(MOST_DAYS) + 2.0 => Some(born),
            _ => None,
        }
    }

    /// The full period, in days.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::quantity::{JulianDay, Utc};
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
    /// # use teistro_panchanga::month;
    /// #
    /// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
    /// # // the (k + 1)th day of 2024.
    /// # let date = |k: u8| {
    /// #     let (mut month, mut left) = (1, k);
    /// #     for length in [31, 29, 31, 30, 31, 30, 31] {
    /// #         if left < length {
    /// #             break;
    /// #         }
    /// #         left -= length;
    /// #         month += 1;
    /// #     }
    /// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
    /// # };
    /// # let days: Vec<FestivalDay> = (0..200u8)
    /// #     .map(|k| FestivalDay {
    /// #         date: date(k),
    /// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #         sunrise: JulianDay::literal(f64::from(k)),
    /// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
    /// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
    /// #         normal: true,
    /// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
    /// #         tithi: Vec::new(),
    /// #         nakshatra: Vec::new(),
    /// #         moonrise: Moonrise::Unsearched,
    /// #         sun_entry: SunEntry::Unsearched,
    /// #         moon_arcs: MoonArcs::Unsearched,
    /// #     })
    /// #     .collect();
    /// # let jd = JulianDay::<Utc>::literal;
    /// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
    /// #
    /// # let table = AshauchTable::dharmasindhu();
    /// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
    /// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
    /// #     key: String::from("first"),
    /// #     event,
    /// #     relation,
    /// #     heard_at: None,
    /// #     abroad: false,
    /// #     nearness: Nearness::SameVillage,
    /// #     full_days: None,
    /// #     during: Vec::new(),
    /// #     fire_giver: false,
    /// # };
    ///
    /// let mut request = asked(death(10.25), Relation::Child);
    /// assert_eq!(request.full(), 10);
    /// request.full_days = Some(15);
    /// assert_eq!(request.full(), 15);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn full(&self) -> u8 {
        self.full_days.unwrap_or(FULL_DAYS)
    }

    /// When the news is known: the hearing, or the event.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::quantity::{JulianDay, Utc};
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
    /// # use teistro_panchanga::month;
    /// #
    /// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
    /// # // the (k + 1)th day of 2024.
    /// # let date = |k: u8| {
    /// #     let (mut month, mut left) = (1, k);
    /// #     for length in [31, 29, 31, 30, 31, 30, 31] {
    /// #         if left < length {
    /// #             break;
    /// #         }
    /// #         left -= length;
    /// #         month += 1;
    /// #     }
    /// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
    /// # };
    /// # let days: Vec<FestivalDay> = (0..200u8)
    /// #     .map(|k| FestivalDay {
    /// #         date: date(k),
    /// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #         sunrise: JulianDay::literal(f64::from(k)),
    /// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
    /// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
    /// #         normal: true,
    /// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
    /// #         tithi: Vec::new(),
    /// #         nakshatra: Vec::new(),
    /// #         moonrise: Moonrise::Unsearched,
    /// #         sun_entry: SunEntry::Unsearched,
    /// #         moon_arcs: MoonArcs::Unsearched,
    /// #     })
    /// #     .collect();
    /// # let jd = JulianDay::<Utc>::literal;
    /// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
    /// #
    /// # let table = AshauchTable::dharmasindhu();
    /// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
    /// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
    /// #     key: String::from("first"),
    /// #     event,
    /// #     relation,
    /// #     heard_at: None,
    /// #     abroad: false,
    /// #     nearness: Nearness::SameVillage,
    /// #     full_days: None,
    /// #     during: Vec::new(),
    /// #     fire_giver: false,
    /// # };
    ///
    /// let mut request = asked(death(10.25), Relation::Child);
    /// assert_eq!(request.known().get(), 10.25);
    /// request.heard_at = Some(jd(12.0));
    /// assert_eq!(request.known().get(), 12.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn known(&self) -> JulianDay<Utc> {
        self.heard_at.unwrap_or_else(|| self.event.at())
    }

    /// The instants a run of days must hold, each with the day before it
    /// and [`AshauchRequest::reach`] days after: the birth, the count's
    /// start, the hearing and each earlier period's end.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::quantity::{JulianDay, Utc};
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
    /// # use teistro_panchanga::month;
    /// #
    /// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
    /// # // the (k + 1)th day of 2024.
    /// # let date = |k: u8| {
    /// #     let (mut month, mut left) = (1, k);
    /// #     for length in [31, 29, 31, 30, 31, 30, 31] {
    /// #         if left < length {
    /// #             break;
    /// #         }
    /// #         left -= length;
    /// #         month += 1;
    /// #     }
    /// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
    /// # };
    /// # let days: Vec<FestivalDay> = (0..200u8)
    /// #     .map(|k| FestivalDay {
    /// #         date: date(k),
    /// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #         sunrise: JulianDay::literal(f64::from(k)),
    /// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
    /// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
    /// #         normal: true,
    /// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
    /// #         tithi: Vec::new(),
    /// #         nakshatra: Vec::new(),
    /// #         moonrise: Moonrise::Unsearched,
    /// #         sun_entry: SunEntry::Unsearched,
    /// #         moon_arcs: MoonArcs::Unsearched,
    /// #     })
    /// #     .collect();
    /// # let jd = JulianDay::<Utc>::literal;
    /// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
    /// #
    /// # let table = AshauchTable::dharmasindhu();
    /// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
    /// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
    /// #     key: String::from("first"),
    /// #     event,
    /// #     relation,
    /// #     heard_at: None,
    /// #     abroad: false,
    /// #     nearness: Nearness::SameVillage,
    /// #     full_days: None,
    /// #     during: Vec::new(),
    /// #     fire_giver: false,
    /// # };
    ///
    /// let mut request = asked(death(10.25), Relation::Child);
    /// request.heard_at = Some(jd(12.0));
    /// let anchors: Vec<f64> = request.anchors().iter().map(|at| at.get()).collect();
    /// assert_eq!(anchors, [10.25, 12.0]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn anchors(&self) -> Vec<JulianDay<Utc>> {
        let mut anchors = vec![self.event.counted_from(), self.known()];
        if let Some(born) = self.born_lately() {
            anchors.push(born);
        }
        anchors.extend(self.during.iter().map(|earlier| earlier.end));
        anchors.sort_by(|a, b| a.get().total_cmp(&b.get()));
        anchors.dedup_by(|a, b| a.get().total_cmp(&b.get()).is_eq());
        anchors
    }

    /// The days after an anchor a run must hold under `table`: the
    /// longest count, and the four the overlap and pakṣiṇī rules can add.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::quantity::{JulianDay, Utc};
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
    /// # use teistro_panchanga::month;
    /// #
    /// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
    /// # // the (k + 1)th day of 2024.
    /// # let date = |k: u8| {
    /// #     let (mut month, mut left) = (1, k);
    /// #     for length in [31, 29, 31, 30, 31, 30, 31] {
    /// #         if left < length {
    /// #             break;
    /// #         }
    /// #         left -= length;
    /// #         month += 1;
    /// #     }
    /// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
    /// # };
    /// # let days: Vec<FestivalDay> = (0..200u8)
    /// #     .map(|k| FestivalDay {
    /// #         date: date(k),
    /// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #         sunrise: JulianDay::literal(f64::from(k)),
    /// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
    /// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
    /// #         normal: true,
    /// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
    /// #         tithi: Vec::new(),
    /// #         nakshatra: Vec::new(),
    /// #         moonrise: Moonrise::Unsearched,
    /// #         sun_entry: SunEntry::Unsearched,
    /// #         moon_arcs: MoonArcs::Unsearched,
    /// #     })
    /// #     .collect();
    /// # let jd = JulianDay::<Utc>::literal;
    /// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
    /// #
    /// # let table = AshauchTable::dharmasindhu();
    /// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
    /// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
    /// #     key: String::from("first"),
    /// #     event,
    /// #     relation,
    /// #     heard_at: None,
    /// #     abroad: false,
    /// #     nearness: Nearness::SameVillage,
    /// #     full_days: None,
    /// #     during: Vec::new(),
    /// #     fire_giver: false,
    /// # };
    ///
    /// // The longest row's days, and the four the rules can add.
    /// let request = asked(death(10.25), Relation::Child);
    /// assert!(request.reach(&table) >= 10 + 4);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn reach(&self, table: &AshauchTable) -> u16 {
        let earlier = self
            .during
            .iter()
            .filter_map(|earlier| earlier.days)
            .max()
            .unwrap_or(0);
        u16::from(table.longest().max(self.full()).max(earlier)) + 4
    }
}

/// The days a request reads, sorted, each sunrise to the next; a gap is
/// allowed between runs, never inside a count.
struct Run<'d> {
    days: &'d [FestivalDay],
}

/// Where an instant falls.
#[derive(Clone, Copy)]
struct Held {
    /// The sunrise day holding it.
    index: usize,
    /// Whether it fell in the night after that day's sunset.
    at_night: bool,
}

impl<'d> Run<'d> {
    fn day(&self, index: usize) -> Result<&'d FestivalDay, Error> {
        self.days
            .get(index)
            .ok_or_else(|| short("a day past the run"))
    }

    /// The day at `index + after`, refused across a gap.
    fn after(&self, index: usize, after: usize) -> Result<&'d FestivalDay, Error> {
        for at in index..index + after {
            let (this, next) = (self.day(at)?, self.day(at + 1)?);
            if (next.sunrise.get() - this.next_sunrise.get()).abs() > JOINED {
                return Err(short("a gap inside a count"));
            }
        }
        self.day(index + after)
    }

    /// The day an end falls in or closes: the one whose sunrise is
    /// before it and whose next sunrise is not.
    fn ending(&self, end: JulianDay<Utc>) -> Result<&'d FestivalDay, Error> {
        let index = self
            .days
            .partition_point(|day| day.sunrise.get() < end.get())
            .checked_sub(1)
            .ok_or_else(|| short("an end before the run"))?;
        let day = self.day(index)?;
        if end.get() > day.next_sunrise.get() {
            return Err(short("an end past the run"));
        }
        Ok(day)
    }

    fn holding(&self, instant: JulianDay<Utc>) -> Result<Held, Error> {
        let index = self
            .days
            .partition_point(|day| day.sunrise.get() <= instant.get())
            .checked_sub(1)
            .ok_or_else(|| short("an instant before the run"))?;
        let day = self.day(index)?;
        if instant.get() >= day.next_sunrise.get() {
            return Err(short("an instant past the run"));
        }
        if !day.normal {
            return Err(Error::new(
                Status::OutOfRange,
                format!(
                    "the Sun neither rose nor set on {}, so its night cannot be divided",
                    day.date
                ),
            ));
        }
        Ok(Held {
            index,
            at_night: instant.get() >= day.sunset.get(),
        })
    }

    /// The first day of a count from `instant`, and why.
    fn day_one(
        &self,
        instant: JulianDay<Utc>,
        rule: NightRule,
        clock: &dyn LocalClock,
    ) -> Result<(usize, DayOneBy), Error> {
        let held = self.holding(instant)?;
        if !held.at_night {
            return Ok((held.index, DayOneBy::Daylight));
        }
        let day = self.day(held.index)?;
        let (sunset, sunrise) = (day.sunset.get(), day.next_sunrise.get());
        let split = match rule {
            NightRule::Thirds => sunset + (sunrise - sunset) * 2.0 / 3.0,
            NightRule::HalfNight => f64::midpoint(sunset, sunrise),
            NightRule::CivilMidnight => {
                // The Julian day turns at noon, so the local clock's
                // midnight is where its fraction is a half.
                let shift = clock.offset_at(day.sunset).days();
                (sunset + shift - 0.5).floor() + 1.5 - shift
            }
            NightRule::Sunrise => sunrise,
        };
        let index = if instant.get() < split {
            held.index
        } else {
            held.index + 1
        };
        Ok((index, DayOneBy::Night { rule }))
    }
}

/// The run's index of the first day of a count from `instant` under
/// `rule`, and why: what the first year counts its eleventh day from.
pub(crate) fn day_one(
    days: &[FestivalDay],
    instant: JulianDay<Utc>,
    rule: NightRule,
    clock: &dyn LocalClock,
) -> Result<(usize, DayOneBy), Error> {
    Run { days }.day_one(instant, rule, clock)
}

/// The day `after` days on from the run's `index`, refused across a gap.
pub(crate) fn day_after(
    days: &[FestivalDay],
    index: usize,
    after: usize,
) -> Result<&FestivalDay, Error> {
    Run { days }.after(index, after)
}

fn short(what: &str) -> Error {
    Error::internal(format!(
        "the ashauch's run of days does not hold {what}; build it over AshauchRequest::anchors"
    ))
}

/// A span counted from one instant.
struct Counted {
    start: JulianDay<Utc>,
    day_one: CalendarDate,
    day_one_by: DayOneBy,
    /// The run's index of the first day.
    first: usize,
    end: JulianDay<Utc>,
    last_day: Option<CalendarDate>,
}

/// `length` from `start` (`03-design/ashauch.md` §2.2).
fn count(
    run: &Run<'_>,
    table: &AshauchTable,
    start: JulianDay<Utc>,
    length: Length,
    full: u8,
    clock: &dyn LocalClock,
) -> Result<Counted, Error> {
    let (first, day_one_by) = run.day_one(start, table.night, clock)?;
    let held = run.holding(start)?;
    let here = run.day(held.index)?;
    let (end, last) = match length {
        Length::Bath => (start, None),
        Length::Sajyotih if held.at_night => (here.next_sunrise, Some(here)),
        Length::Sajyotih => (here.sunset, Some(here)),
        Length::Pakshini => {
            let next = run.after(held.index, 1)?;
            match (held.at_night, table.pakshini_at_night) {
                (true, PakshiniNight::SecondSunrise) => (next.next_sunrise, Some(next)),
                _ => (next.sunset, Some(next)),
            }
        }
        Length::Days { .. } | Length::Full => {
            let days = length.count(full).unwrap_or(full);
            let last = run.after(first, usize::from(days) - 1)?;
            (last.next_sunrise, Some(last))
        }
    };
    Ok(Counted {
        start,
        day_one: run.day(first)?.date.clone(),
        day_one_by,
        first,
        end,
        last_day: last.map(|day| day.date.clone()),
    })
}

/// The row a request takes before the news's lateness is read.
struct Chosen {
    length: Length,
    clause: Clause,
    /// The relation row's own late length, when it names one.
    heard_late: Option<Length>,
}

/// Whether a band's mark is reached at the death.
fn reached(mark: Mark, deceased: &Deceased, at: JulianDay<Utc>) -> Result<bool, Error> {
    let elapsed = deceased.born.map(|born| at.get() - born.get());
    let age = |days: f64| elapsed.is_none_or(|elapsed| elapsed >= days);
    let asked = |fact: Option<bool>, field: &str| {
        fact.or(deceased.born.is_none().then_some(true))
            .ok_or_else(|| {
                Error::invalid_arg(format!(
                    "a band of the deceased's age turns on `{field}`, so say it"
                ))
                .with_field(format!("event.deceased.{field}"))
            })
    };
    Ok(match mark {
        Mark::Day { day } => age(f64::from(day - 1)),
        Mark::Month { month } => age(f64::from(month - 1) * MONTH_DAYS),
        Mark::Year { year } => age(f64::from(year - 1) * YEAR_DAYS),
        Mark::Chudakarana => deceased.chudakarana.unwrap_or_else(|| age(3.0 * YEAR_DAYS)),
        Mark::Upanayana => match deceased.upanayana {
            Some(Upanayana::Done) => true,
            Some(Upanayana::NotDone) => false,
            Some(Upanayana::NotPerformed) => {
                age(16.0 * YEAR_DAYS) || asked(deceased.married, "married")?
            }
            None => age(16.0 * YEAR_DAYS) || asked(None, "upanayana")?,
        },
        Mark::Betrothal => {
            asked(deceased.married, "married")? || asked(deceased.betrothed, "betrothed")?
        }
        Mark::Marriage => asked(deceased.married, "married")?,
    })
}

/// The band of `sex` a death falls in, with its index.
fn band_of<'t>(
    table: &'t AshauchTable,
    sex: Sex,
    deceased: &Deceased,
    at: JulianDay<Utc>,
) -> Result<Option<(usize, &'t Band)>, Error> {
    for (index, band) in table.bands.iter().enumerate() {
        if band.sex == sex
            && band.buried.is_none_or(|buried| buried == deceased.buried)
            && !reached(band.before, deceased, at)?
            && band.chudakarana.map_or(Ok(true), |wanted| {
                reached(Mark::Chudakarana, deceased, at).map(|done| done == wanted)
            })?
        {
            return Ok(Some((index, band)));
        }
    }
    Ok(None)
}

fn choose(table: &AshauchTable, request: &AshauchRequest) -> Result<Chosen, Error> {
    let relation = request.relation;
    if let AshauchEvent::Loss { month, .. } = request.event {
        let (row, loss) = table
            .losses
            .iter()
            .enumerate()
            .find(|(_, loss)| month <= loss.to_month)
            .ok_or_else(|| no_row(table, request))?;
        let length = match relation {
            Relation::Mother => loss.mother,
            Relation::Father | Relation::Sapinda => loss.others,
            _ => Length::Bath,
        };
        return Ok(Chosen {
            length,
            clause: Clause::Loss {
                row,
                said: loss.clause.clone(),
            },
            heard_late: None,
        });
    }
    if let AshauchEvent::Death { at, deceased, .. } = request.event {
        let band = if let Some(sex) = deceased.sex {
            band_of(table, sex, &deceased, at)?
        } else {
            let son = band_of(table, Sex::Male, &deceased, at)?;
            let daughter = band_of(table, Sex::Female, &deceased, at)?;
            if son.is_some() || daughter.is_some() {
                return Err(Error::invalid_arg(
                    "a son's and a daughter's bands differ, so say the deceased's sex",
                )
                .with_field("event.deceased.sex"));
            }
            None
        };
        if let Some((row, band)) = band {
            let length = match relation {
                Relation::Mother | Relation::Father | Relation::StepChild => band.parents,
                Relation::Sapinda | Relation::Child => band.sapindas,
                Relation::Samanodaka => band.samanodakas.unwrap_or(Length::Bath),
                _ => Length::Bath,
            };
            return Ok(Chosen {
                length,
                clause: Clause::Band {
                    row,
                    said: band.clause.clone(),
                },
                heard_late: None,
            });
        }
    }
    let kind = request.event.kind();
    let married_daughter = matches!(
        request.event,
        AshauchEvent::Death {
            deceased: Deceased {
                sex: Some(Sex::Female),
                married: Some(true),
                ..
            },
            ..
        }
    );
    let (row, found) = table
        .relations
        .iter()
        .enumerate()
        .find(|(_, row)| {
            row.relation == relation
                && row.kind == kind
                && row
                    .nearness
                    .is_none_or(|nearness| nearness == request.nearness)
                && row
                    .married_daughter
                    .is_none_or(|wanted| wanted == married_daughter)
        })
        .ok_or_else(|| no_row(table, request))?;
    Ok(Chosen {
        length: found.length,
        clause: Clause::Relation {
            row,
            said: found.clause.clone(),
        },
        heard_late: found.heard_late,
    })
}

fn no_row(table: &AshauchTable, request: &AshauchRequest) -> Error {
    Error::new(
        Status::Unsupported,
        format!(
            "the {} table names no length for {:?} at a {:?} ({:?})",
            table.name,
            request.relation,
            request.event.kind(),
            request.nearness
        ),
    )
    .with_field("relation")
    .with_hint("add a row to the table for this relation and placement")
}

/// The ashauch `request` asks under `table`, over `days`: sorted sunrise
/// days holding each of [`AshauchRequest::anchors`] with the day before
/// it and [`AshauchRequest::reach`] days after, `clock` the place's
/// clock for [`NightRule::CivilMidnight`].
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
/// # use teistro_panchanga::month;
/// #
/// # // Synthetic days: day k rises at Julian day k, sets at k + 0.5, and is
/// # // the (k + 1)th day of 2024.
/// # let date = |k: u8| {
/// #     let (mut month, mut left) = (1, k);
/// #     for length in [31, 29, 31, 30, 31, 30, 31] {
/// #         if left < length {
/// #             break;
/// #         }
/// #         left -= length;
/// #         month += 1;
/// #     }
/// #     CalendarDate::defined(Calendar::Gregorian, 2024, month, left + 1)
/// # };
/// # let days: Vec<FestivalDay> = (0..200u8)
/// #     .map(|k| FestivalDay {
/// #         date: date(k),
/// #         vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #         sunrise: JulianDay::literal(f64::from(k)),
/// #         sunset: JulianDay::literal(f64::from(k) + 0.5),
/// #         next_sunrise: JulianDay::literal(f64::from(k) + 1.0),
/// #         normal: true,
/// #         month: month::of(Masa::Pausha, Tithi::ShuklaPratipada, LunarMonth::Amanta, MonthKind::Nija),
/// #         tithi: Vec::new(),
/// #         nakshatra: Vec::new(),
/// #         moonrise: Moonrise::Unsearched,
/// #         sun_entry: SunEntry::Unsearched,
/// #         moon_arcs: MoonArcs::Unsearched,
/// #     })
/// #     .collect();
/// # let jd = JulianDay::<Utc>::literal;
/// # use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation, ashauch};
/// #
/// # let table = AshauchTable::dharmasindhu();
/// # let death = |at: f64| AshauchEvent::Death { at: jd(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() };
/// # let asked = |event: AshauchEvent, relation: Relation| AshauchRequest {
/// #     key: String::from("first"),
/// #     event,
/// #     relation,
/// #     heard_at: None,
/// #     abroad: false,
/// #     nearness: Nearness::SameVillage,
/// #     full_days: None,
/// #     during: Vec::new(),
/// #     fire_giver: false,
/// # };
///
/// // A parent's death by day: ten days from it.
/// let found = ashauch(&table, &asked(death(10.25), Relation::Child), &days, &UtcOffset::UTC)?;
/// assert_eq!((found.days, found.end.get()), (Some(10), 20.0));
/// // An ascetic's sapiṇḍas bathe, and are pure at once.
/// let bath = ashauch(&table, &asked(death(10.25), Relation::OfAscetic), &days, &UtcOffset::UTC)?;
/// assert_eq!((bath.end, bath.last_day), (bath.start, None));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// As [`AshauchRequest::check`] and [`AshauchTable::check`]; a relation
/// the table names no row for; a deceased's fact a band needs and the
/// request leaves out; an instant on a polar day.
pub fn ashauch(
    table: &AshauchTable,
    request: &AshauchRequest,
    days: &[FestivalDay],
    clock: &dyn LocalClock,
) -> Result<Ashauch, Error> {
    table.check()?;
    request.check()?;
    let run = Run { days };
    let full = request.full();
    let known = request.known();
    if let Some(found) = birth_period_runs(&run, table, request, full, clock)? {
        return Ok(found);
    }
    let answer = |counted: Counted, length: Length, clause: Clause| Ashauch {
        key: request.key.clone(),
        kind: request.event.kind(),
        relation: request.relation,
        sex: request.event.sex(),
        start: counted.start,
        day_one: counted.day_one,
        day_one_by: counted.day_one_by,
        end: counted.end,
        last_day: counted.last_day,
        length,
        days: length.count(full),
        clause,
        overlap: None,
        rites_from: None,
        table: table.name.clone(),
    };
    let chosen = choose(table, request)?;
    let own = count(
        &run,
        table,
        request.event.counted_from(),
        chosen.length,
        full,
        clock,
    )?;
    let mut found = if known.get() < own.end.get() || chosen.length == Length::Bath {
        let (heard, _) = run.day_one(known, table.night, clock)?;
        if heard > own.first {
            Ashauch {
                start: known,
                ..answer(own, chosen.length, Clause::Remainder)
            }
        } else {
            let first = own.first;
            Ashauch {
                rites_from: rites_from(&run, table, request, first)?,
                ..answer(own, chosen.length, chosen.clause)
            }
        }
    } else {
        let (length, clause) = late(table, request, &chosen, known);
        answer(
            count(&run, table, known, length, full, clock)?,
            length,
            clause,
        )
    };
    overlap(&run, table, request, known, &mut found, clock)?;
    Ok(found)
}

/// A child dead inside its own birth period keeps the birth period, for
/// the parents and the sapiṇḍas (*Dharmasindhu* p. 300).
fn birth_period_runs(
    run: &Run<'_>,
    table: &AshauchTable,
    request: &AshauchRequest,
    full: u8,
    clock: &dyn LocalClock,
) -> Result<Option<Ashauch>, Error> {
    let (AshauchEvent::Death { at, deceased, .. }, Some(born)) =
        (request.event, request.born_lately())
    else {
        return Ok(None);
    };
    if !matches!(
        request.relation,
        Relation::Mother | Relation::Father | Relation::Sapinda
    ) {
        return Ok(None);
    }
    let birth = count(run, table, born, Length::Full, full, clock)?;
    if at.get() >= birth.end.get() {
        return Ok(None);
    }
    Ok(Some(Ashauch {
        key: request.key.clone(),
        kind: AshauchKind::Birth,
        relation: request.relation,
        sex: deceased.sex,
        start: birth.start,
        day_one: birth.day_one,
        day_one_by: birth.day_one_by,
        end: birth.end,
        last_day: birth.last_day,
        length: Length::Full,
        days: Some(full),
        clause: Clause::BirthPeriodRuns,
        overlap: None,
        rites_from: None,
        table: table.name.clone(),
    }))
}

/// News heard past the period (*Dharmasindhu* pp. 303–304).
fn late(
    table: &AshauchTable,
    request: &AshauchRequest,
    chosen: &Chosen,
    known: JulianDay<Utc>,
) -> (Length, Clause) {
    if request.event.kind() == AshauchKind::Birth {
        return (Length::Bath, Clause::BirthHeardLate);
    }
    if let (Some(length), Clause::Relation { row, said }) = (chosen.heard_late, &chosen.clause) {
        return (
            length,
            Clause::RowHeardLate {
                row: *row,
                said: said.clone(),
            },
        );
    }
    if chosen.length != Length::Full {
        return (Length::Bath, Clause::ShortHeardLate);
    }
    let rows = if request.abroad {
        &table.late_abroad
    } else {
        &table.late
    };
    let since = known.get() - request.event.at().get();
    rows.iter()
        .find(|row| since < f64::from(row.within_days))
        .map_or((Length::Bath, Clause::LateBath), |row| {
            (
                row.length,
                Clause::Late {
                    within_days: row.within_days,
                    abroad: request.abroad,
                },
            )
        })
}

/// The mother's further bar after a birth (*Dharmasindhu* p. 299).
fn rites_from(
    run: &Run<'_>,
    table: &AshauchTable,
    request: &AshauchRequest,
    first: usize,
) -> Result<Option<JulianDay<Utc>>, Error> {
    let (
        Some(rites),
        Relation::Mother,
        AshauchEvent::Birth {
            child: Some(child), ..
        },
    ) = (table.mother_rites, request.relation, request.event)
    else {
        return Ok(None);
    };
    let days = match child {
        Sex::Male => rites.son,
        Sex::Female => rites.daughter,
    };
    if days == 0 {
        return Ok(None);
    }
    Ok(Some(run.after(first, usize::from(days) - 1)?.next_sunrise))
}

/// The overlap rules over the earlier periods known to be running
/// (`03-design/ashauch.md` §2.7): of every earlier period that takes the
/// new one in, the latest end, never past the new one's own.
fn overlap(
    run: &Run<'_>,
    table: &AshauchTable,
    request: &AshauchRequest,
    known: JulianDay<Utc>,
    found: &mut Ashauch,
    clock: &dyn LocalClock,
) -> Result<(), Error> {
    // The birthing mother's own period, and the fire-giver's, are never
    // taken in (DS p. 306).
    if found.length == Length::Bath
        || request.fire_giver
        || (found.kind == AshauchKind::Birth && found.relation == Relation::Mother)
    {
        return Ok(());
    }
    let full = request.full();
    let mut best: Option<(JulianDay<Utc>, Option<CalendarDate>, Overlapped)> = None;
    for earlier in &request.during {
        if !(earlier.start.get() <= known.get() && known.get() < earlier.end.get())
            || earlier.length == Length::Bath
        {
            continue;
        }
        let both_deaths = found.kind == AshauchKind::Death && earlier.kind == AshauchKind::Death;
        // A son's period for a parent, a wife's for her husband, is not
        // taken in by another's (DS p. 306).
        if both_deaths
            && matches!(found.relation, Relation::Child | Relation::Spouse)
            && earlier.relation != found.relation
        {
            continue;
        }
        let parents = both_deaths && found.relation == Relation::Child;
        // The father dying in the mother's period ends with hers (DS p.
        // 306), or keeps his own (the Mitākṣarā on III.20).
        if parents
            && (earlier.sex, found.sex) == (Some(Sex::Female), Some(Sex::Male))
            && table.overlap.father_keeps_his_own
        {
            continue;
        }
        // The added days take in a pakṣiṇī (two added) or three days
        // (three added), and never a ten-day period (DS p. 306).
        if earlier
            .extended_from
            .is_some_and(|from| known.get() >= from.get())
            && found.length.half_days(full) > extended_reach(earlier.extended_days)
        {
            continue;
        }
        // A birth never takes in a death (DS p. 305).
        if found.kind == AshauchKind::Death && earlier.kind == AshauchKind::Birth {
            continue;
        }
        let mother_after_father =
            parents && (earlier.sex, found.sex) == (Some(Sex::Male), Some(Sex::Female));
        let longer_birth = found.kind == AshauchKind::Birth
            && earlier.kind == AshauchKind::Death
            && table.overlap.death_absorbs_longer_birth;
        if found.length.half_days(full) > earlier.length.half_days(full)
            && !mother_after_father
            && !longer_birth
        {
            continue;
        }
        let candidate = if let Some(added) = last_night(run, table, earlier, found, known, full)? {
            let ended = run.holding(earlier.end)?.index;
            let last = run.after(ended, usize::from(added) - 1)?;
            (
                last.next_sunrise,
                Some(last.date.clone()),
                Overlapped::LastNight {
                    earlier: earlier.key.clone(),
                    days: added,
                    from: earlier.end,
                },
            )
        } else if mother_after_father {
            let after = count(run, table, earlier.end, Length::Pakshini, full, clock)?;
            (
                after.end,
                after.last_day,
                Overlapped::MotherAfterFather {
                    earlier: earlier.key.clone(),
                },
            )
        } else {
            (
                earlier.end,
                Some(run.ending(earlier.end)?.date.clone()),
                Overlapped::EndsWithEarlier {
                    earlier: earlier.key.clone(),
                },
            )
        };
        if candidate.0.get() <= found.end.get()
            && best
                .as_ref()
                .is_none_or(|(end, ..)| candidate.0.get() > end.get())
        {
            best = Some(candidate);
        }
    }
    if let Some((end, last_day, overlapped)) = best {
        found.end = end;
        found.last_day = last_day;
        found.overlap = Some(overlapped);
    }
    Ok(())
}

/// The half days an extension of `days` takes in: a pakṣiṇī for two,
/// three days for three (DS p. 306).
fn extended_reach(days: Option<u8>) -> u16 {
    Length::half_days(
        if days.is_some_and(|days| days > 2) {
            Length::Days { days: 3 }
        } else {
            Length::Pakshini
        },
        0,
    )
}

/// The days the last-night rule adds, when a new full period of the same
/// kind is known in an earlier full period's last night (DS p. 306, only
/// between ten-day periods, three-day ones ending with the first; NS p.
/// 381: only between equal periods of a kind).
fn last_night(
    run: &Run<'_>,
    table: &AshauchTable,
    earlier: &Running,
    found: &Ashauch,
    known: JulianDay<Utc>,
    full: u8,
) -> Result<Option<u8>, Error> {
    let (Some(old), Some(new)) = (earlier.days, found.days) else {
        return Ok(None);
    };
    if old != new || old != full || earlier.kind != found.kind {
        return Ok(None);
    }
    let ended = run.holding(earlier.end)?.index;
    let last = ended
        .checked_sub(1)
        .ok_or_else(|| short("the day before an earlier period's end"))?;
    let night = run.day(last)?;
    if known.get() < night.sunset.get() {
        return Ok(None);
    }
    let quarter = earlier.end.get() - (earlier.end.get() - night.sunset.get()) / 4.0;
    let [two, three] = table.overlap.last_night;
    Ok(Some(if known.get() >= quarter { three } else { two }).filter(|&days| days > 0))
}

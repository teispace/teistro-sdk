//! Festival rules: which day an observance falls on
//! (`03-design/festival-rules.md`).
//!
//! An observance takes the tithi that pervades the time of its rite, its
//! **karmakala** (*Dharmasindhu*, pp. 5–6). A tithi touches two days, so
//! a rule says what to do when it holds that time on one of them, on
//! both, on neither, or in part. That is a table the rule carries and not
//! a principle the evaluator assumes: the text's own two tables, for a
//! rite at midday and one at pradosha, differ in four of six cases.
//!
//! A rule is data: a lunar date, a window, and an ordered list of guards
//! over a closed set of predicates, the first that holds deciding. The
//! answer names the case and the guard, so a reader who follows another
//! authority sees where it parts.
//!
//! The evaluator is pure over [`FestivalDay`] values, which a
//! [`Panchanga`] gives and a test can write by hand.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Kind, Masa, Nakshatra, Paksha, Tithi, Vara, write_in_full};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::LunarMonth as Convention;
use teistro_time::local_day::{DayState, LocalDay};

use crate::Panchanga;
use crate::month::{self, LunarMonth};
use crate::span::Span;

mod anniversary;
mod dated;
mod ekadashi;
mod following;
mod india_central;
mod mahalaya;
mod meta;
mod nepal_panchanga;
mod next;
mod published;
mod sait;
mod vrata;

pub use anniversary::{
    Anniversary, AnniversaryDay, AnniversaryKind, DayClause, DayRule, FirstYearAdhika, Half,
    InSamsarpa, JanmaOptions, MonthClause, Origin, OriginFound, TithiShape, anniversaries,
};
pub use dated::{Dated, DatedObservance, DatedRule, dated};
pub use ekadashi::{
    EkadashiFast, EkadashiKinds, EkadashiRule, EkadashiTable, Excess, JoinedNakshatra,
    Mahadvadashi, MahadvadashiKind, MahadvadashiRule, NakshatraBar, NakshatraJoining, Parana,
    ParanaBarred, ParanaEdge, ParanaKept, ParanaRule, Portion, TrisprishaAt, Vedha, VyanjuliPast,
    ekadashis,
};
pub use following::{FollowingRule, following};
pub use meta::{FastEnd, FastStart, FestivalMeta, MetaFilter, VrataFast};
pub use next::{
    FIRST_WINDOW_DAYS, KindRules, LIMIT_DAYS, MOST_LIMIT_DAYS, NextKind, limit_days, windows,
};
pub use published::{
    AddedDay, AppliedSet, NEAREST_DAYS, PUBLISHED_FORMAT, Published, PublishedEntry, PublishedSet,
    PublishedStatus, SubjectTo, Unplaced, apply as apply_published,
};
pub use sait::{
    Navamsas, RisingOf, Sait, SaitAvoid, SaitCandidate, SaitChoice, SaitRising, SaitRule,
    SaitWindow, SaitWindowRule,
};
use vrata::RiteAt;
pub use vrata::{VrataEdge, VrataParana, VrataParanaTime};

/// A fifth of the daylight (*Dharmasindhu*, p. 6). Its schema name is
/// `DayFifth`, the chart's daylight or night being the boundary's
/// `DayPart`.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// use teistro_panchanga::festival::DayPart;
///
/// // Midday on the 2nd, 2.2 to 2.3, holds the ashtami; on the 1st it begins as
/// // midday ends.
/// let found = first(Window::Part { part: DayPart::Madhyahna })?;
/// assert_eq!((found.case, found.day.day), (Case::LaterOnly, 3));
/// // Both afternoons hold it wholly.
/// assert_eq!(first(Window::Part { part: DayPart::Aparahna })?.case, Case::Both);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "DayFifth"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayPart {
    /// The first fifth.
    Pratah,
    /// The second.
    Sangava,
    /// The third: midday.
    Madhyahna,
    /// The fourth: the afternoon.
    Aparahna,
    /// The last.
    Sayahna,
}

impl DayPart {
    const fn index(self) -> u16 {
        match self {
            DayPart::Pratah => 0,
            DayPart::Sangava => 1,
            DayPart::Madhyahna => 2,
            DayPart::Aparahna => 3,
            DayPart::Sayahna => 4,
        }
    }
}

/// The time of a rite, on a sunrise day.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// // The evening and the night's middle hold it on the 1st alone.
/// assert_eq!(first(Window::Pradosha)?.case, Case::EarlierOnly);
/// assert_eq!(first(Window::Nishitha)?.case, Case::EarlierOnly);
/// assert_eq!(first(Window::Sunrise)?.case, Case::LaterOnly);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "window", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Window {
    /// The instant of sunrise: the tithi a day is named for.
    Sunrise,
    /// The instant of sunset, which begins the evening: Nepal's monthly
    /// full-moon fast takes the day whose sunset the full moon holds
    /// (`festival-rules.md` §9.6, C200).
    Sunset,
    /// A fifth of the daylight.
    Part {
        /// Which fifth.
        part: DayPart,
    },
    /// Three muhurtas after sunset, a muhurta a fifteenth of the night
    /// (crux C168).
    Pradosha,
    /// The middle of the night, an instant (C169: p. 49 counts a tithi
    /// present there if "even a kala" of it stands at the end of the
    /// night's first half).
    Nishitha,
    /// A muhurta of the night, a fifteenth of it counted from sunset, 1
    /// to 15: p. 90 calls the eighth niśītha for Shivaratri and weighs a
    /// tithi holding it wholly against one holding part of it.
    NightMuhurta {
        /// Which muhurta, 1 for the first after sunset.
        muhurta: u8,
    },
    /// Muhurtas of the daylight, a fifteenth of it counted from sunrise,
    /// `from` to `to` inclusive, 1 to 15: *Dharmasindhu* p. 265 names the
    /// 8th kutapa and the 9th rauhina, and purvahna is the first six, the
    /// daylight's first two fifths (C602). A fifth is three of them.
    DayMuhurta {
        /// The first, 1 for the muhurta after sunrise.
        from: u8,
        /// The last, at least `from`.
        to: u8,
    },
    /// The instant of the first moonrise after the day's sunrise and
    /// before the next: *Dharmasindhu* p. 9 keeps the Sankashta chaturthi
    /// on the tithi at moonrise (चन्द्रोदयव्यापिनी). A day the Moon does
    /// not rise on leaves the occurrence unjudged.
    Moonrise,
}

impl Window {
    /// The muhurta a [`Window::NightMuhurta`] names, if this is one.
    const fn muhurta(self) -> Option<u8> {
        match self {
            Window::NightMuhurta { muhurta } => Some(muhurta),
            Window::Sunrise
            | Window::Sunset
            | Window::Part { .. }
            | Window::Pradosha
            | Window::Nishitha
            | Window::DayMuhurta { .. }
            | Window::Moonrise => None,
        }
    }

    /// Whether the window lies in the daylight, sunrise to sunset; the
    /// instant of sunset, which begins the evening, the pradosha after it
    /// and the night's windows do not, nor does the moonrise, which the
    /// rites kept at it see after dark.
    ///
    /// ```
    /// use teistro_panchanga::festival::Window;
    ///
    /// assert!(Window::Sunrise.in_daylight());
    /// assert!(!Window::Sunset.in_daylight());
    /// assert!(!Window::Pradosha.in_daylight());
    /// ```
    #[must_use]
    pub const fn in_daylight(self) -> bool {
        match self {
            Window::Sunrise | Window::Part { .. } | Window::DayMuhurta { .. } => true,
            Window::Sunset
            | Window::Pradosha
            | Window::Nishitha
            | Window::NightMuhurta { .. }
            | Window::Moonrise => false,
        }
    }
}

/// One of the two days a tithi is judged between.
///
/// ```
/// use teistro_core::catalogue::Tithi;
/// use teistro_panchanga::festival::{Which, yugma};
///
/// // The new moon is kept on the day it runs into the bright 1st.
/// assert_eq!(yugma(Tithi::Amavasya), Some(Which::Later));
/// assert_eq!(yugma(Tithi::ShuklaTritiya), Some(Which::Earlier));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "FestivalWhich"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Which {
    /// The earlier.
    Earlier,
    /// The later.
    Later,
}

/// How the tithi held the rite's time on the two days (pp. 6–7).
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// let found = first(Window::Sunrise)?;
/// assert_eq!(found.case, Case::LaterOnly);
/// assert_eq!((found.extents[0].held, found.extents[1].held), (0.0, 1.0));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "FestivalCase"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Case {
    /// On the earlier day only, wholly or in part.
    EarlierOnly,
    /// On the later day only.
    LaterOnly,
    /// Wholly, on both.
    Both,
    /// On neither.
    Neither,
    /// On both, in part at least on one, the two equal (C170).
    EqualParts,
    /// On both, in part at least on one, unequally.
    UnequalParts,
}

/// Something a guard asks of the two days.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// use teistro_panchanga::festival::{Edge, Which};
///
/// // The ashtami lasts 24 ghatis past the 2nd's sunrise: at least six.
/// let lasts = FestivalRule {
///     decide: vec![Guard::new(
///         [Predicate::Lasts { day: Which::Later, from: Edge::Sunrise, ghatis: 6 }],
///         Choice::Later,
///     )],
///     ..rule(Window::Pradosha)
/// };
/// let found = observances(&[lasts], &days)?;
/// assert_eq!(found.observances[0].choice, Choice::Later);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "is", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Predicate {
    /// The case is this one.
    Case {
        /// The case.
        case: Case,
    },
    /// The tithi and a nakshatra stand together on a day: within a
    /// window when one is named, anywhere in the sunrise day otherwise;
    /// and only a share of the nakshatra when `portion` names one.
    Joined {
        /// Which day.
        day: Which,
        /// The nakshatra.
        nakshatra: Nakshatra,
        /// Where they must meet; the whole day when absent.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at: Option<Window>,
        /// The share of the nakshatra's arc that must meet the tithi, as
        /// Shravana's last quarter (*Dharmasindhu* p. 70); the whole when
        /// absent. A share is the Moon's stay in its arc, searched at the
        /// arc's own bounds ([`FestivalDay::moon_arcs`]).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        portion: Option<Portion>,
    },
    /// The nakshatra stands in a window of a day, whatever the tithi: the
    /// Nirnaya-sindhu's condition that *Dharmasindhu* p. 71 endorses,
    /// Shravana in the later day's aparahna after the dashami has ended.
    Stands {
        /// Which day.
        day: Which,
        /// The nakshatra.
        nakshatra: Nakshatra,
        /// The window it must stand in, for some time or at its instant.
        at: Window,
    },
    /// The tithi stands at an edge of the day and lasts at least this many
    /// ghatis past it, a ghati being a thirtieth of the daylight after
    /// sunrise or of the night after sunset: three muhurtas after sunrise
    /// are six ghatis (p. 71), and the new moon a ghati into the night
    /// settles Lakshmi puja (p. 77).
    Lasts {
        /// Which day.
        day: Which,
        /// The edge counted from.
        from: Edge,
        /// How many ghatis, 1 to 30.
        ghatis: u8,
    },
    /// The tithi holds the whole of a day's window, or its instant: p. 90
    /// gives Shivaratri to the day holding niśītha wholly when the other
    /// holds it in part.
    Wholly {
        /// Which day.
        day: Which,
    },
    /// This day's window holds more of the tithi, in time, than the other
    /// day's does; at an instant, this day's holds it and the other's
    /// does not. Equal times are a tie and hold for neither day. Time and
    /// not the share of each window: the two differ only when both days
    /// hold part of a window whose length moves a minute or two from one
    /// day to the next (C601, C604).
    Greater {
        /// Which day.
        day: Which,
    },
    /// The day is this vara, sunrise to sunrise: *Dharmasindhu* p. 20
    /// names the new moon on a Monday (Somavati), the 4th on a Tuesday and
    /// the 7th on a Sunday, and the committee prints the 13th's fast on a
    /// Saturday or a Monday by those names (C618).
    OnVara {
        /// Which day.
        day: Which,
        /// The vara.
        vara: Vara,
    },
}

/// The edge of a day a [`Predicate::Lasts`] counts from.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// use teistro_panchanga::festival::{Decided, Edge, Which};
///
/// // Not 30 ghatis past it: the guard fails and the rule's otherwise decides.
/// let lasts = FestivalRule {
///     decide: vec![Guard::new(
///         [Predicate::Lasts { day: Which::Later, from: Edge::Sunrise, ghatis: 30 }],
///         Choice::Later,
///     )],
///     ..rule(Window::Pradosha)
/// };
/// let found = observances(&[lasts], &days)?;
/// assert_eq!(found.observances[0].decided_by, Decided::Otherwise);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "DayEdge"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Edge {
    /// Sunrise, counting in thirtieths of the daylight.
    Sunrise,
    /// Sunset, counting in thirtieths of the night.
    Sunset,
}

/// What a guard, or a rule's `otherwise`, decides.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// let found = first(Window::Sunrise)?;
/// assert_eq!(found.choice, Choice::Later);
/// // By no guard, the earlier day.
/// assert_eq!(first(Window::Pradosha)?.choice, Choice::Earlier);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "FestivalChoice"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Choice {
    /// The earlier day.
    Earlier,
    /// The later day.
    Later,
    /// By the yugma verse (p. 6): the first of a pair on the day it runs
    /// into its partner, the second on the day its partner runs into it.
    ByYugma,
    /// Neither day: the rule is not kept this time. A rite kept only on a
    /// weekday answers this when its tithi's day is another (C618).
    None,
}

/// A choice when every predicate holds.
///
/// ```
/// use teistro_panchanga::festival::{Case, Choice, Guard, Predicate};
///
/// let guard = Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later);
/// assert_eq!((guard.when.len(), guard.choose), (1, Choice::Later));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Guard {
    /// What must all hold.
    pub when: Vec<Predicate>,
    /// What it decides.
    pub choose: Choice,
}

/// The guards of a rite judged by its window's share: the day holding the
/// tithi there alone; the earlier when both hold it wholly, on a tie, or
/// when neither does; else the day holding more of it, in time. The shared
/// table of *Dharmasindhu* p. 266's shraddhas and the committee's fitted
/// days (C601, C603, C663).
pub(crate) fn by_share() -> (Vec<Guard>, Choice) {
    (
        vec![
            Guard::new([case(Case::LaterOnly)], Choice::Later),
            Guard::new([case(Case::Both)], Choice::Earlier),
            Guard::new([Predicate::Greater { day: Which::Later }], Choice::Later),
        ],
        Choice::Earlier,
    )
}

impl Guard {
    /// A guard.
    ///
    /// ```
    /// use teistro_panchanga::festival::{Choice, Guard, Predicate, Which};
    ///
    /// // Every predicate must hold: here the later day holds the window wholly.
    /// let guard = Guard::new(vec![Predicate::Wholly { day: Which::Later }], Choice::Later);
    /// assert_eq!(guard.when, [Predicate::Wholly { day: Which::Later }]);
    /// ```
    #[must_use]
    pub fn new(when: impl Into<Vec<Predicate>>, choose: Choice) -> Guard {
        Guard {
            when: when.into(),
            choose,
        }
    }
}

/// A rule: when an observance falls, and how its day is decided.
///
/// ```
/// use teistro_panchanga::festival::FestivalRule;
///
/// let rules = FestivalRule::dharmasindhu();
/// let janmashtami = rules.iter().find(|rule| rule.key == "JANMASHTAMI").ok_or("shipped")?;
/// assert!(janmashtami.source.starts_with("Dharmasindhu"));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(transform = variants_as_fields))]
#[serde(rename_all = "camelCase")]
pub struct FestivalRule {
    /// The rule's key in its pack, `"JANMASHTAMI"`.
    pub key: String,
    /// Where the rule is stated.
    pub source: String,
    /// The month, under `convention`; every month when `None`, for a rite
    /// kept on the same tithi of each, as the monthly full-moon fast is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub month: Option<Masa>,
    /// The convention the month is named in; amanta by default.
    #[serde(default = "amanta")]
    pub convention: Convention,
    /// What the rite is kept on: a tithi, or a nakshatra in a paksha,
    /// written flat as `tithi`, or as `nakshatra` with `paksha`.
    #[serde(flatten)]
    pub occurs: Occurs,
    /// Which of a doubled month's two holds it (C171): the nija month
    /// by default.
    #[serde(default)]
    pub in_adhika: InAdhika,
    /// The time of the rite.
    pub at: Window,
    /// The guards, in order; the first that holds decides.
    pub decide: Vec<Guard>,
    /// What decides when no guard holds.
    pub otherwise: Choice,
    /// When the fast kept on the day is broken, each reading answered in
    /// order (§8.7); none when left out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parana: Vec<VrataParana>,
    /// What the rule's festival is, as sourced (`03-design/festival-metadata.md`);
    /// a field no source gives stays unset, and a rule with none leaves it out.
    #[serde(default, skip_serializing_if = "FestivalMeta::is_empty")]
    pub meta: FestivalMeta,
}

const fn amanta() -> Convention {
    Convention::Amanta
}

/// What a rule is kept on: a tithi, the usual way, or a nakshatra in a
/// paksha of the month, as the Samavedis' upakarma is kept on Hasta in
/// Bhadrapada's bright half (*Dharmasindhu* p. 47, `festival-rules.md`
/// §9.7). A nakshatra's occurrence is judged between its two days as a
/// tithi's is, by the share of the rite's window it holds; it falls in the
/// month and paksha of the tithi running at its middle.
///
/// ```
/// use teistro_panchanga::festival::Occurs;
/// use teistro_core::catalogue::{Nakshatra, Paksha, Tithi};
///
/// let tithi: Occurs = Tithi::Purnima.into();
/// assert_eq!(serde_json::to_value(tithi)?, serde_json::json!({"tithi": "PURNIMA"}));
/// let hasta = Occurs::Nakshatra { nakshatra: Nakshatra::Hasta, paksha: Paksha::Shukla };
/// assert_eq!(serde_json::to_value(hasta)?, serde_json::json!({"nakshatra": "HASTA", "paksha": "SHUKLA"}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "OccursSchema"))]
#[serde(untagged, try_from = "OccursFields")]
#[non_exhaustive]
pub enum Occurs {
    /// A tithi; its paksha is the tithi's.
    Tithi {
        /// The tithi.
        tithi: Tithi,
    },
    /// A nakshatra, in one paksha of the month.
    Nakshatra {
        /// The nakshatra.
        nakshatra: Nakshatra,
        /// The half of the month it must fall in.
        paksha: Paksha,
    },
    /// The same day of both pakshas, 1 to 14: the committee prints the
    /// pradosha fast on the 13th of each (§3.4). The 15th is the full moon
    /// in one and the new moon in the other, each kept as a `tithi`.
    DayOfPaksha {
        /// The day, 1 to 14.
        #[serde(rename = "dayOfPaksha")]
        day_of_paksha: u8,
    },
}

impl From<Tithi> for Occurs {
    fn from(tithi: Tithi) -> Occurs {
        Occurs::Tithi { tithi }
    }
}

impl Occurs {
    /// The tithi, for a rule kept on one.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Paksha, Tithi};
    /// use teistro_panchanga::festival::Occurs;
    ///
    /// assert_eq!(Occurs::from(Tithi::Purnima).tithi(), Some(Tithi::Purnima));
    /// let hasta = Occurs::Nakshatra { nakshatra: Nakshatra::Hasta, paksha: Paksha::Shukla };
    /// assert_eq!(hasta.tithi(), None);
    /// ```
    #[must_use]
    pub const fn tithi(self) -> Option<Tithi> {
        match self {
            Occurs::Tithi { tithi } => Some(tithi),
            Occurs::Nakshatra { .. } | Occurs::DayOfPaksha { .. } => None,
        }
    }

    /// Whether the rule is kept on a tithi, one or a day of either
    /// paksha, rather than on a nakshatra.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, Paksha};
    /// use teistro_panchanga::festival::Occurs;
    ///
    /// assert!(Occurs::DayOfPaksha { day_of_paksha: 13 }.on_a_tithi());
    /// assert!(!Occurs::Nakshatra { nakshatra: Nakshatra::Hasta, paksha: Paksha::Shukla }.on_a_tithi());
    /// ```
    #[must_use]
    pub const fn on_a_tithi(self) -> bool {
        matches!(self, Occurs::Tithi { .. } | Occurs::DayOfPaksha { .. })
    }

    /// Whether a tithi is one the rule is kept on, when it is kept on a
    /// tithi.
    fn keeps(self, held: Tithi) -> bool {
        match self {
            Occurs::Tithi { tithi } => tithi == held,
            Occurs::DayOfPaksha { day_of_paksha } => held.attributes().number == day_of_paksha,
            Occurs::Nakshatra { .. } => false,
        }
    }
}

/// Which of a doubled month's two a rule is kept in.
///
/// A year with an adhika month names it twice, the adhika first and the
/// nija after (§3.7). `NIJA` and `ALSO` are written `false` and `true`,
/// as the yes-or-no they replace was, so a rule written before keeps its
/// spelling and its hash; each is read in either form.
///
/// ```
/// use teistro_panchanga::festival::InAdhika;
///
/// let first: InAdhika = serde_json::from_str(r#""FIRST""#)?;
/// assert_eq!(first, InAdhika::First);
/// assert_eq!(serde_json::to_string(&InAdhika::Also)?, "true");
/// assert_eq!(serde_json::from_str::<InAdhika>(r#""NIJA""#)?, InAdhika::Nija);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "InAdhikaSchema"))]
#[serde(try_from = "InAdhikaWire", into = "InAdhikaWire")]
#[non_exhaustive]
pub enum InAdhika {
    /// The nija month alone, `false`.
    #[default]
    Nija,
    /// Both, `true`.
    Also,
    /// The adhika month alone: Padmini and Parama ekadashi, with no month
    /// named.
    Only,
    /// The adhika month when the named month is doubled, the nija month
    /// otherwise: Dashahara in a doubled Jyeshtha (*Dharmasindhu* p. 38).
    First,
}

impl InAdhika {
    /// The four, in the order they are written.
    ///
    /// ```
    /// use teistro_panchanga::festival::InAdhika;
    ///
    /// assert_eq!(InAdhika::ALL.map(InAdhika::key), ["NIJA", "ALSO", "ONLY", "FIRST"]);
    /// ```
    pub const ALL: [InAdhika; 4] = [
        InAdhika::Nija,
        InAdhika::Also,
        InAdhika::Only,
        InAdhika::First,
    ];

    /// The value's key, `"FIRST"`.
    ///
    /// ```
    /// use teistro_panchanga::festival::InAdhika;
    ///
    /// assert_eq!(InAdhika::Only.key(), "ONLY");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            InAdhika::Nija => "NIJA",
            InAdhika::Also => "ALSO",
            InAdhika::Only => "ONLY",
            InAdhika::First => "FIRST",
        }
    }
}

/// The two forms [`InAdhika`] is written in.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
enum InAdhikaWire {
    Flag(bool),
    Key(String),
}

impl TryFrom<InAdhikaWire> for InAdhika {
    type Error = String;

    fn try_from(wire: InAdhikaWire) -> Result<InAdhika, String> {
        match wire {
            InAdhikaWire::Flag(true) => Ok(InAdhika::Also),
            InAdhikaWire::Flag(false) => Ok(InAdhika::Nija),
            InAdhikaWire::Key(key) => InAdhika::ALL
                .into_iter()
                .find(|value| value.key() == key)
                .ok_or_else(|| {
                    format!(
                        "`inAdhika` is `true`, `false`, \"NIJA\", \"ALSO\", \"ONLY\" or \"FIRST\", not {key:?}"
                    )
                }),
        }
    }
}

impl From<InAdhika> for InAdhikaWire {
    fn from(value: InAdhika) -> InAdhikaWire {
        match value {
            InAdhika::Nija => InAdhikaWire::Flag(false),
            InAdhika::Also => InAdhikaWire::Flag(true),
            InAdhika::Only | InAdhika::First => InAdhikaWire::Key(value.key().to_owned()),
        }
    }
}

/// The shape [`InAdhika`] is read in, for its schema.
#[cfg(feature = "schema")]
#[derive(schemars::JsonSchema)]
#[schemars(untagged)]
#[allow(dead_code, reason = "a schema's shape, never built")]
enum InAdhikaSchema {
    /// `true` for `ALSO`, `false` for `NIJA`.
    Flag(bool),
    /// One of the four by key.
    Key(InAdhikaKey),
}

/// The four keys, for [`InAdhikaSchema`].
#[cfg(feature = "schema")]
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(dead_code, reason = "a schema's shape, never built")]
enum InAdhikaKey {
    Nija,
    Also,
    Only,
    First,
}

/// Writes the record's flattened enum, which schemars writes as a `oneOf`
/// or `anyOf` beside its fields, as optional fields of the record, with a
/// sentence on each saying which go together: no binding's generator
/// reads a union beside fields, and each such enum here is read from
/// flat fields its reader checks ([`OccursFields`], [`dated::Dated`]).
/// A field two variants share is written once.
#[cfg(feature = "schema")]
pub(crate) fn variants_as_fields(schema: &mut schemars::Schema) {
    let options: Vec<serde_json::Value> = ["oneOf", "anyOf"]
        .into_iter()
        .filter_map(|keyword| schema.remove(keyword))
        .filter_map(|options| options.as_array().cloned())
        .flatten()
        .collect();
    let givens: Vec<String> = options
        .iter()
        .map(|option| {
            let required = option.get("required").and_then(serde_json::Value::as_array);
            let keys: Vec<String> = required
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
                .map(|key| format!("`{key}`"))
                .collect();
            keys.join(" with ")
        })
        .collect();
    let sentence = match givens.split_last() {
        Some((last, rest @ [_, ..])) => {
            format!(" Exactly one of {} or {last} is given.", rest.join(", "))
        }
        _ => String::new(),
    };
    let Some(properties) = schema
        .get_mut("properties")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };
    for option in &options {
        let variant = option
            .get("description")
            .and_then(serde_json::Value::as_str);
        let fields = option
            .get("properties")
            .and_then(serde_json::Value::as_object);
        for (key, field) in fields.into_iter().flatten() {
            if properties.contains_key(key) {
                continue;
            }
            let mut field = field.clone();
            if let Some(object) = field.as_object_mut() {
                let own = object
                    .get("description")
                    .and_then(serde_json::Value::as_str)
                    .or(variant)
                    .unwrap_or_default();
                object.insert("description".into(), format!("{own}{sentence}").into());
            }
            properties.insert(key.clone(), field);
        }
    }
}

/// The shape [`Occurs`] is read in, for its schema: the reader refuses
/// every other combination of the three fields by name.
#[cfg(feature = "schema")]
#[derive(schemars::JsonSchema)]
#[schemars(untagged)]
#[allow(dead_code, reason = "a schema's shape, never built")]
enum OccursSchema {
    /// A tithi; its paksha is the tithi's.
    Tithi {
        /// The tithi.
        tithi: Tithi,
    },
    /// A nakshatra, in one paksha of the month.
    Nakshatra {
        /// The nakshatra.
        nakshatra: Nakshatra,
        /// The half of the month it must fall in.
        paksha: Paksha,
    },
    /// The same day of both pakshas, 1 to 14.
    DayOfPaksha {
        /// The day, 1 to 14.
        #[serde(rename = "dayOfPaksha")]
        #[schemars(range(min = 1, max = 14))]
        day_of_paksha: u8,
    },
}

/// The fields [`Occurs`] is read from, so that a rule naming both a tithi
/// and a nakshatra, or neither, or a paksha without its nakshatra, is
/// refused by name rather than read as one of them.
#[derive(Deserialize)]
struct OccursFields {
    #[serde(default)]
    tithi: Option<Tithi>,
    #[serde(default)]
    nakshatra: Option<Nakshatra>,
    #[serde(default)]
    paksha: Option<Paksha>,
    #[serde(default, rename = "dayOfPaksha")]
    day_of_paksha: Option<u8>,
}

impl TryFrom<OccursFields> for Occurs {
    type Error = String;

    fn try_from(fields: OccursFields) -> Result<Occurs, String> {
        if let Some(day_of_paksha) = fields.day_of_paksha {
            if fields.tithi.is_some() || fields.nakshatra.is_some() || fields.paksha.is_some() {
                return Err(
                    "a rule kept on a `dayOfPaksha` names no `tithi`, `nakshatra` or `paksha`; it is kept in both pakshas"
                        .to_owned(),
                );
            }
            if !(1..=14).contains(&day_of_paksha) {
                return Err(format!(
                    "a `dayOfPaksha` is 1 to 14, not {day_of_paksha}; the 15th is `PURNIMA` in the bright half and `AMAVASYA` in the dark, each kept as a `tithi`"
                ));
            }
            return Ok(Occurs::DayOfPaksha { day_of_paksha });
        }
        match (fields.tithi, fields.nakshatra, fields.paksha) {
            (Some(tithi), None, None) => Ok(Occurs::Tithi { tithi }),
            (None, Some(nakshatra), Some(paksha)) => Ok(Occurs::Nakshatra { nakshatra, paksha }),
            (None, Some(_), None) => Err(
                "a rule kept on a `nakshatra` names the `paksha` it falls in, `SHUKLA` or `KRISHNA`"
                    .to_owned(),
            ),
            (Some(_), _, _) => Err(
                "a rule is kept on a `tithi`, or on a `nakshatra` in a `paksha`, not both".to_owned(),
            ),
            (None, None, _) => Err(
                "a rule is kept on a `tithi`, or on a `nakshatra` in a `paksha`".to_owned(),
            ),
        }
    }
}

/// The yugma verse's reading of a tithi (p. 6): 2 with 3, 4 with 5,
/// 6 with 7, 8 with 9, 11 with 12, the bright 14th with the full moon,
/// the new moon with the bright 1st. `None` for a tithi in no pair.
///
/// ```
/// use teistro_core::catalogue::Tithi;
/// use teistro_panchanga::festival::{Which, yugma};
///
/// // 2 with 3: the 2nd on the day it runs into the 3rd, the 3rd on the day the
/// // 2nd runs into it.
/// assert_eq!(yugma(Tithi::ShuklaDvitiya), Some(Which::Later));
/// assert_eq!(yugma(Tithi::ShuklaTritiya), Some(Which::Earlier));
/// assert_eq!(yugma(Tithi::ShuklaDashami), None);
/// ```
#[must_use]
pub fn yugma(tithi: Tithi) -> Option<Which> {
    match tithi {
        Tithi::ShuklaChaturdashi | Tithi::Amavasya => Some(Which::Later),
        Tithi::Purnima | Tithi::ShuklaPratipada => Some(Which::Earlier),
        Tithi::KrishnaChaturdashi | Tithi::KrishnaPratipada => None,
        _ => match tithi.attributes().number {
            2 | 4 | 6 | 8 | 11 => Some(Which::Later),
            3 | 5 | 7 | 9 | 12 => Some(Which::Earlier),
            _ => None,
        },
    }
}

impl FestivalRule {
    /// The shares of nakshatras its guards join, as each arc in sidereal
    /// degrees, which the days must carry ([`FestivalDay::moon_arcs`]).
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_panchanga::festival::{
    /// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
    /// #     SunEntry, Window, observances,
    /// # };
    /// # use teistro_panchanga::month;
    /// # use teistro_panchanga::span::Span;
    /// #
    /// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
    /// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
    /// # // and day 1's afternoon and evening.
    /// # let ends = [
    /// #     (Tithi::KrishnaSaptami, 1.3),
    /// #     (Tithi::KrishnaAshtami, 2.4),
    /// #     (Tithi::KrishnaNavami, 3.5),
    /// #     (Tithi::KrishnaDashami, 4.6),
    /// #     (Tithi::KrishnaEkadashi, 5.7),
    /// #     (Tithi::KrishnaDwadashi, 6.8),
    /// # ];
    /// # let days: Vec<FestivalDay> = (0..6u8)
    /// #     .map(|k| {
    /// #         let rise = f64::from(k);
    /// #         let window = Interval::literal(rise, rise + 1.0);
    /// #         let mut from = 0.0;
    /// #         let mut tithi = Vec::new();
    /// #         for (member, to) in ends {
    /// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
    /// #             from = to;
    /// #         }
    /// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
    /// #         FestivalDay {
    /// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
    /// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #             sunrise: JulianDay::literal(rise),
    /// #             sunset: JulianDay::literal(rise + 0.5),
    /// #             next_sunrise: JulianDay::literal(rise + 1.0),
    /// #             normal: true,
    /// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
    /// #             tithi,
    /// #             nakshatra: Vec::new(),
    /// #             moonrise: Moonrise::Unsearched,
    /// #             sun_entry: SunEntry::Unsearched,
    /// #             moon_arcs: MoonArcs::Unsearched,
    /// #         }
    /// #     })
    /// #     .collect();
    /// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
    /// # let rule = |at: Window| FestivalRule {
    /// #     key: String::from("ACME_ASHTAMI"),
    /// #     source: String::from("an example"),
    /// #     month: Some(Masa::Shravana),
    /// #     convention: LunarMonth::Amanta,
    /// #     occurs: Tithi::KrishnaAshtami.into(),
    /// #     in_adhika: InAdhika::Nija,
    /// #     at,
    /// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
    /// #     otherwise: Choice::Earlier,
    /// #     parana: Vec::new(),
    /// #     meta: Default::default(),
    /// # };
    /// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
    /// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
    /// # };
    ///
    /// // A rule that joins no share of a nakshatra needs no arcs.
    /// assert!(rule(Window::Sunrise).moon_arcs().is_empty());
    /// let shipped = FestivalRule::dharmasindhu();
    /// assert!(shipped.iter().flat_map(FestivalRule::moon_arcs).all(|(_, (from, to))| from < to));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn moon_arcs(&self) -> Vec<(Nakshatra, (f64, f64))> {
        self.decide
            .iter()
            .flat_map(|guard| &guard.when)
            .filter_map(|predicate| match *predicate {
                Predicate::Joined {
                    nakshatra,
                    portion: Some(portion),
                    ..
                } => Some((nakshatra, portion.arc_deg(nakshatra))),
                _ => None,
            })
            .collect()
    }

    /// Whether the rule reads a moonrise, at its rite, in a guard or as
    /// the end of its fast ([`FastEnd::Moonrise`]): the days it is judged
    /// over must have theirs searched.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let rules = FestivalRule::dharmasindhu();
    /// let reads = |key: &str| rules.iter().any(|rule| rule.key == key && rule.reads_moonrise());
    /// assert!(reads("SANKASHTA_CHATURTHI") && !reads("JANMASHTAMI"));
    /// ```
    #[must_use]
    pub fn reads_moonrise(&self) -> bool {
        self.judged_at_moonrise() || self.breaks_at_moonrise()
    }

    /// Whether the rule's day is decided by a moonrise, at its rite or in
    /// a guard: a day the Moon does not rise on leaves it unjudged.
    fn judged_at_moonrise(&self) -> bool {
        self.at == Window::Moonrise
            || self
                .decide
                .iter()
                .flat_map(|guard| &guard.when)
                .any(|predicate| {
                    matches!(
                        predicate,
                        Predicate::Joined {
                            at: Some(Window::Moonrise),
                            ..
                        } | Predicate::Stands {
                            at: Window::Moonrise,
                            ..
                        }
                    )
                })
    }

    /// Whether the rule's fast is broken at moonrise
    /// ([`FastEnd::Moonrise`]), so its answer carries the day's.
    ///
    /// ```
    /// use teistro_panchanga::festival::{FestivalMeta, FestivalRule};
    ///
    /// let rules = FestivalRule::dharmasindhu();
    /// let mut ashunya = rules.into_iter().find(|rule| rule.key == "ASHUNYA_SHAYANA").ok_or("shipped")?;
    /// assert!(!ashunya.breaks_at_moonrise());
    /// ashunya.meta = FestivalMeta::of_text(&ashunya.key);
    /// assert!(ashunya.breaks_at_moonrise());
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn breaks_at_moonrise(&self) -> bool {
        self.meta
            .fast
            .iter()
            .any(|fast| fast.end == Some(FastEnd::Moonrise))
    }

    /// Refuses a rule that could not be judged: an empty key, a ghati
    /// count outside a daylight's or a night's thirty, a night muhurta
    /// outside its fifteen, the yugma verse asked of a tithi it pairs
    /// with nothing or of a nakshatra, or a tithi joined to a nakshatra
    /// asked of a rule kept on a nakshatra.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_panchanga::festival::{
    /// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
    /// #     SunEntry, Window, observances,
    /// # };
    /// # use teistro_panchanga::month;
    /// # use teistro_panchanga::span::Span;
    /// #
    /// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
    /// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
    /// # // and day 1's afternoon and evening.
    /// # let ends = [
    /// #     (Tithi::KrishnaSaptami, 1.3),
    /// #     (Tithi::KrishnaAshtami, 2.4),
    /// #     (Tithi::KrishnaNavami, 3.5),
    /// #     (Tithi::KrishnaDashami, 4.6),
    /// #     (Tithi::KrishnaEkadashi, 5.7),
    /// #     (Tithi::KrishnaDwadashi, 6.8),
    /// # ];
    /// # let days: Vec<FestivalDay> = (0..6u8)
    /// #     .map(|k| {
    /// #         let rise = f64::from(k);
    /// #         let window = Interval::literal(rise, rise + 1.0);
    /// #         let mut from = 0.0;
    /// #         let mut tithi = Vec::new();
    /// #         for (member, to) in ends {
    /// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
    /// #             from = to;
    /// #         }
    /// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
    /// #         FestivalDay {
    /// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
    /// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #             sunrise: JulianDay::literal(rise),
    /// #             sunset: JulianDay::literal(rise + 0.5),
    /// #             next_sunrise: JulianDay::literal(rise + 1.0),
    /// #             normal: true,
    /// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
    /// #             tithi,
    /// #             nakshatra: Vec::new(),
    /// #             moonrise: Moonrise::Unsearched,
    /// #             sun_entry: SunEntry::Unsearched,
    /// #             moon_arcs: MoonArcs::Unsearched,
    /// #         }
    /// #     })
    /// #     .collect();
    /// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
    /// # let rule = |at: Window| FestivalRule {
    /// #     key: String::from("ACME_ASHTAMI"),
    /// #     source: String::from("an example"),
    /// #     month: Some(Masa::Shravana),
    /// #     convention: LunarMonth::Amanta,
    /// #     occurs: Tithi::KrishnaAshtami.into(),
    /// #     in_adhika: InAdhika::Nija,
    /// #     at,
    /// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
    /// #     otherwise: Choice::Earlier,
    /// #     parana: Vec::new(),
    /// #     meta: Default::default(),
    /// # };
    /// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
    /// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
    /// # };
    ///
    /// rule(Window::Sunrise).check()?;
    /// let unnamed = FestivalRule { key: String::from(" "), ..rule(Window::Sunrise) };
    /// assert_eq!(unnamed.check().unwrap_err().field(), Some("key"));
    /// let muhurta = rule(Window::NightMuhurta { muhurta: 16 });
    /// assert!(muhurta.check().is_err(), "a night has fifteen");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, naming the field that is wrong.
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg("a festival rule needs a key").with_field("key"));
        }
        self.check_size()?;
        let choices = self
            .decide
            .iter()
            .map(|guard| guard.choose)
            .chain(std::iter::once(self.otherwise));
        if choices.into_iter().any(|choice| choice == Choice::ByYugma)
            && self.occurs.tithi().and_then(yugma).is_none()
        {
            return Err(Error::invalid_arg(format!(
                "{}: the yugma verse pairs {:?} with nothing (it pairs 2-3, 4-5, 6-7, 8-9, 11-12, the bright 14th with the full moon, and the new moon with the bright 1st, and no nakshatra or day of either paksha); choose EARLIER or LATER",
                self.key, self.occurs
            ))
            .with_field("decide"));
        }
        check_window(&self.key, self.at, "at")?;
        for (index, reading) in self.parana.iter().enumerate() {
            reading.check(&self.key, index)?;
        }
        for guard in &self.decide {
            for predicate in &guard.when {
                match *predicate {
                    Predicate::Lasts { ghatis, .. } if !(1..=30).contains(&ghatis) => {
                        return Err(Error::invalid_arg(format!(
                            "{}: a daylight or a night has thirty ghatis, not {ghatis}",
                            self.key
                        ))
                        .with_field("decide.when.ghatis"));
                    }
                    Predicate::Joined { .. } if !self.occurs.on_a_tithi() => {
                        return Err(Error::invalid_arg(format!(
                            "{}: JOINED asks a tithi and a nakshatra to stand together, and this rule is kept on a nakshatra; ask STANDS of another nakshatra",
                            self.key
                        ))
                        .with_field("decide.when.is"));
                    }
                    Predicate::Joined { at, portion, .. } => {
                        if let Some(at) = at {
                            check_window(&self.key, at, "decide.when.at")?;
                        }
                        if let Some(portion) = portion {
                            portion.check(&self.key, "decide.when.portion")?;
                        }
                    }
                    Predicate::Stands { at, .. } => {
                        check_window(&self.key, at, "decide.when.at")?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl FestivalRule {
    /// The longest key a rule may have, in bytes.
    pub const MOST_KEY_BYTES: usize = 128;
    /// The most guards a rule may list.
    pub const MOST_GUARDS: usize = 32;
    /// The most predicates a guard may ask together.
    pub const MOST_PREDICATES: usize = 16;

    /// Refuses a rule past the limits a caller's rule is held to
    /// (`03-design/anniversary-span.md` §6), each naming its field: a rule
    /// is judged for every occurrence in a range, so its size is a cost
    /// the request pays many times over.
    fn check_size(&self) -> Result<(), Error> {
        if self.key.len() > FestivalRule::MOST_KEY_BYTES {
            return Err(Error::limit(format!(
                "a rule's key holds at most {} bytes, not {}",
                FestivalRule::MOST_KEY_BYTES,
                self.key.len()
            ))
            .with_field("key"));
        }
        if self.decide.len() > FestivalRule::MOST_GUARDS {
            return Err(Error::limit(format!(
                "{}: a rule lists at most {} guards, not {}",
                self.key,
                FestivalRule::MOST_GUARDS,
                self.decide.len()
            ))
            .with_field("decide"));
        }
        if let Some(guard) = self
            .decide
            .iter()
            .find(|guard| guard.when.len() > FestivalRule::MOST_PREDICATES)
        {
            return Err(Error::limit(format!(
                "{}: a guard asks at most {} predicates, not {}",
                self.key,
                FestivalRule::MOST_PREDICATES,
                guard.when.len()
            ))
            .with_field("decide.when"));
        }
        Ok(())
    }
}

/// Refuses a night muhurta outside the night's fifteen, and a range
/// of day muhurtas outside the daylight's fifteen or running
/// backwards; `key` names the rule and `at` is the window's own field.
fn check_window(key: &str, window: Window, at: &str) -> Result<(), Error> {
    if let Some(muhurta) = window.muhurta()
        && !(1..=15).contains(&muhurta)
    {
        return Err(Error::invalid_arg(format!(
            "{key}: a night has fifteen muhurtas, counted from 1 after sunset, not {muhurta}"
        ))
        .with_field(format!("{at}.muhurta")));
    }
    if let Window::DayMuhurta { from, to } = window {
        for (name, muhurta) in [("from", from), ("to", to)] {
            if !(1..=15).contains(&muhurta) {
                return Err(Error::invalid_arg(format!(
                    "{key}: a daylight has fifteen muhurtas, counted from 1 after sunrise, not {muhurta}"
                ))
                .with_field(format!("{at}.{name}")));
            }
        }
        if from > to {
            return Err(Error::invalid_arg(format!(
                "{key}: day muhurtas {from} to {to} run backwards; give the first as `from`"
            ))
            .with_field(format!("{at}.to")));
        }
    }
    Ok(())
}

/// What the evaluator reads of one sunrise day.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// // The 2nd's sunrise holds the ashtami; its day holds the navami's start.
/// assert_eq!(days[2].tithi[0].member, Tithi::KrishnaAshtami);
/// assert_eq!(days[2].tithi.len(), 2);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FestivalDay {
    /// The civil date.
    pub date: CalendarDate,
    /// Its vara, from this sunrise to the next.
    pub vara: Vara,
    /// Its sunrise.
    pub sunrise: JulianDay<Utc>,
    /// Its sunset.
    pub sunset: JulianDay<Utc>,
    /// The next sunrise.
    pub next_sunrise: JulianDay<Utc>,
    /// Whether the Sun rose and set; a polar day has no windows.
    pub normal: bool,
    /// The lunar month at its sunrise.
    pub month: LunarMonth,
    /// Its tithis, each with its whole bounds.
    pub tithi: Vec<Span<Tithi>>,
    /// Its nakshatras.
    pub nakshatra: Vec<Span<Nakshatra>>,
    /// Its first moonrise, found only for a rule that reads
    /// [`Window::Moonrise`].
    pub moonrise: Moonrise,
    /// The nakshatra the Sun entered in its civil day, midnight to
    /// midnight, read only for a rule kept on that entry
    /// ([`Dated::SunEnters`]).
    pub sun_entry: SunEntry,
    /// The Moon's stays in the arcs a parana bar names, found only for a
    /// rule that reads them ([`ParanaRule::bars`]).
    pub moon_arcs: MoonArcs,
}

/// The Moon's stays in the sidereal arcs the rules read, over a day.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// assert_eq!(days[0].moon_arcs, MoonArcs::Unsearched);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum MoonArcs {
    /// Not searched: no rule read them. A rule reading them over such a
    /// day is unjudged rather than read as a day the Moon stood in none.
    Unsearched,
    /// Each arc's stay that meets the day, sunrise to the next.
    Found(Vec<MoonArc>),
}

/// The Moon's stay in one sidereal arc: from its crossing of `from_deg` to
/// its crossing of `to_deg`.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::festival::MoonArc;
///
/// // Shravana's last quarter, 290° to 293°20′.
/// let arc = (290.0, 290.0 + 10.0 / 3.0);
/// let stay = MoonArc { from_deg: arc.0, to_deg: arc.1, span: Interval::literal(2_460_000.2, 2_460_000.45) };
/// assert!(stay.is_of(arc));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoonArc {
    /// Where the arc starts, in degrees of the sidereal zodiac.
    pub from_deg: f64,
    /// Where it ends.
    pub to_deg: f64,
    /// The Moon's stay in it.
    pub span: Interval,
}

impl MoonArc {
    /// Whether this is the stay in the arc `(from_deg, to_deg)`. Both
    /// come from one rule's [`NakshatraBar::arc_deg`], so the same arc is
    /// the same bits.
    ///
    /// ```
    /// use teistro_core::interval::Interval;
    /// use teistro_panchanga::festival::MoonArc;
    ///
    /// let stay = MoonArc { from_deg: 290.0, to_deg: 293.0, span: Interval::literal(2_460_000.2, 2_460_000.45) };
    /// assert!(stay.is_of((290.0, 293.0)));
    /// assert!(!stay.is_of((290.0, 293.000_000_1)), "the same bits");
    /// ```
    #[must_use]
    pub fn is_of(&self, (from_deg, to_deg): (f64, f64)) -> bool {
        self.from_deg.to_bits() == from_deg.to_bits() && self.to_deg.to_bits() == to_deg.to_bits()
    }
}

/// Whether the sidereal Sun entered a nakshatra in a civil day, local
/// midnight to the next. The Sun is never retrograde and stays more than
/// thirteen days in a nakshatra, so a day holds one entry at most.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_panchanga::festival::SunEntry;
///
/// let entered = SunEntry::Entered(Nakshatra::Ashwini);
/// assert_ne!(entered, SunEntry::Stayed);
/// assert_ne!(SunEntry::Stayed, SunEntry::Unsearched);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SunEntry {
    /// Not read: no rule asked. A rule asking over such a day is refused
    /// rather than read as a day the Sun entered none.
    Unsearched,
    /// The Sun stood in one nakshatra the whole civil day.
    Stayed,
    /// The Sun entered this nakshatra in the civil day.
    Entered(Nakshatra),
}

/// A day's first moonrise, from its sunrise to the next.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// // A rule kept at moonrise over days whose moonrise no search found is
/// // refused; over days the Moon did not rise on, it is unjudged.
/// assert!(observances(&[rule(Window::Moonrise)], &days).is_err());
/// let moonless: Vec<FestivalDay> = days
///     .iter()
///     .cloned()
///     .map(|day| FestivalDay { moonrise: Moonrise::NoRise, ..day })
///     .collect();
/// let found = observances(&[rule(Window::Moonrise)], &moonless)?;
/// assert_eq!(found.unjudged.len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Moonrise {
    /// Not searched: no rule read it. A rule reading it over such a day is
    /// refused rather than read as a day the Moon did not rise.
    Unsearched,
    /// The Moon rose then, there.
    Rose(MoonriseAt),
    /// The Moon did not rise between the two sunrises.
    NoRise,
}

/// A moonrise and the place it was found for (SR-60): what an observance
/// kept or broken at moonrise is answered with, since the instant moves
/// with the place.
///
/// ```
/// use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// use teistro_panchanga::festival::MoonriseAt;
///
/// let delhi = Place::new(Latitude::literal(28.6139), Longitude::literal(77.209), Altitude::literal(216.0));
/// let seen = MoonriseAt { at: JulianDay::literal(2_460_752.16), place: delhi };
/// let json = serde_json::to_value(seen)?;
/// assert_eq!(json["place"]["latitude"], 28.6139);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MoonriseAt {
    /// The instant the Moon rose.
    pub at: JulianDay<Utc>,
    /// Where: the place the days were reckoned at.
    pub place: Place,
}

impl From<&Panchanga> for FestivalDay {
    fn from(day: &Panchanga) -> FestivalDay {
        FestivalDay::of(
            &day.day,
            day.month,
            day.limbs.tithi.clone(),
            day.limbs.nakshatra.clone(),
        )
    }
}

impl FestivalDay {
    /// A day's view, from its local day and the limbs the rules read.
    pub(crate) fn of(
        day: &LocalDay,
        month: LunarMonth,
        tithi: Vec<Span<Tithi>>,
        nakshatra: Vec<Span<Nakshatra>>,
    ) -> FestivalDay {
        FestivalDay {
            date: day.date.clone(),
            vara: day.vara,
            sunrise: day.sunrise,
            sunset: day.sunset,
            next_sunrise: day.next_sunrise,
            normal: matches!(day.state, DayState::Normal),
            month,
            tithi,
            nakshatra,
            moonrise: Moonrise::Unsearched,
            sun_entry: SunEntry::Unsearched,
            moon_arcs: MoonArcs::Unsearched,
        }
    }

    fn daylight(&self) -> Interval {
        Interval::literal(self.sunrise.get(), self.sunset.get())
    }

    fn night(&self) -> Interval {
        Interval::literal(self.sunset.get(), self.next_sunrise.get())
    }

    /// The day a rite is kept on, sunrise to the next: the window a
    /// moonrise is searched in.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_panchanga::festival::{
    /// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
    /// #     SunEntry, Window, observances,
    /// # };
    /// # use teistro_panchanga::month;
    /// # use teistro_panchanga::span::Span;
    /// #
    /// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
    /// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
    /// # // and day 1's afternoon and evening.
    /// # let ends = [
    /// #     (Tithi::KrishnaSaptami, 1.3),
    /// #     (Tithi::KrishnaAshtami, 2.4),
    /// #     (Tithi::KrishnaNavami, 3.5),
    /// #     (Tithi::KrishnaDashami, 4.6),
    /// #     (Tithi::KrishnaEkadashi, 5.7),
    /// #     (Tithi::KrishnaDwadashi, 6.8),
    /// # ];
    /// # let days: Vec<FestivalDay> = (0..6u8)
    /// #     .map(|k| {
    /// #         let rise = f64::from(k);
    /// #         let window = Interval::literal(rise, rise + 1.0);
    /// #         let mut from = 0.0;
    /// #         let mut tithi = Vec::new();
    /// #         for (member, to) in ends {
    /// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
    /// #             from = to;
    /// #         }
    /// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
    /// #         FestivalDay {
    /// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
    /// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #             sunrise: JulianDay::literal(rise),
    /// #             sunset: JulianDay::literal(rise + 0.5),
    /// #             next_sunrise: JulianDay::literal(rise + 1.0),
    /// #             normal: true,
    /// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
    /// #             tithi,
    /// #             nakshatra: Vec::new(),
    /// #             moonrise: Moonrise::Unsearched,
    /// #             sun_entry: SunEntry::Unsearched,
    /// #             moon_arcs: MoonArcs::Unsearched,
    /// #         }
    /// #     })
    /// #     .collect();
    /// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
    /// # let rule = |at: Window| FestivalRule {
    /// #     key: String::from("ACME_ASHTAMI"),
    /// #     source: String::from("an example"),
    /// #     month: Some(Masa::Shravana),
    /// #     convention: LunarMonth::Amanta,
    /// #     occurs: Tithi::KrishnaAshtami.into(),
    /// #     in_adhika: InAdhika::Nija,
    /// #     at,
    /// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
    /// #     otherwise: Choice::Earlier,
    /// #     parana: Vec::new(),
    /// #     meta: Default::default(),
    /// # };
    /// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
    /// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
    /// # };
    ///
    /// assert_eq!(days[1].sunrise_day(), Interval::literal(1.0, 2.0));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn sunrise_day(&self) -> Interval {
        Interval::literal(self.sunrise.get(), self.next_sunrise.get())
    }

    /// The window of a rite on this day: an instant is an interval of no
    /// length.
    fn window(&self, window: Window) -> Result<Interval, Error> {
        match window {
            Window::Sunrise => Ok(Interval::literal(self.sunrise.get(), self.sunrise.get())),
            Window::Sunset => Ok(Interval::literal(self.sunset.get(), self.sunset.get())),
            Window::Part { part } => self.daylight().part(part.index(), 5),
            Window::Pradosha => {
                let muhurta = self.night().days() / 15.0;
                Ok(Interval::literal(
                    self.sunset.get(),
                    muhurta.mul_add(3.0, self.sunset.get()),
                ))
            }
            Window::Nishitha => {
                let middle = self.night().at_fraction(0.5)?;
                Ok(Interval::literal(middle.get(), middle.get()))
            }
            Window::NightMuhurta { muhurta } => {
                self.night().part(u16::from(muhurta).saturating_sub(1), 15)
            }
            Window::DayMuhurta { from, to } => {
                let first = self
                    .daylight()
                    .part(u16::from(from).saturating_sub(1), 15)?;
                let last = self.daylight().part(u16::from(to).saturating_sub(1), 15)?;
                Ok(Interval::literal(first.from.get(), last.to.get()))
            }
            Window::Moonrise => match self.moonrise {
                Moonrise::Rose(seen) => Ok(Interval::literal(seen.at.get(), seen.at.get())),
                Moonrise::NoRise => Err(Error::internal(format!(
                    "{} has no moonrise, which judging leaves unjudged",
                    self.date
                ))),
                Moonrise::Unsearched => Err(Error::invalid_arg(format!(
                    "a rule reads the moonrise of {}, which no search found; fill the days' moonrises first (`Almanac::moonrises`)",
                    self.date
                ))),
            },
        }
    }
}

/// How much of one day's window the tithi held.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// // At the afternoon, 1.3 to 1.4 and 2.3 to 2.4, both wholly.
/// let found = first(Window::Part { part: teistro_panchanga::festival::DayPart::Aparahna })?;
/// assert_eq!((found.extents[0].held, found.extents[1].held), (1.0, 1.0));
/// assert_eq!(found.extents[0].day.day, 2);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Extent {
    /// The day.
    pub day: CalendarDate,
    /// The window; an instant's has no length.
    pub window: Interval,
    /// The fraction of it the tithi held, 0 to 1; an instant's is 0 or 1.
    pub held: f64,
}

/// What decided an observance's day.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// use teistro_panchanga::festival::Decided;
///
/// assert_eq!(first(Window::Sunrise)?.decided_by, Decided::Guard { index: 0 });
/// assert_eq!(first(Window::Pradosha)?.decided_by, Decided::Otherwise);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "FestivalDecided"))]
#[serde(tag = "by", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Decided {
    /// The guard at this place in the rule's list.
    Guard {
        /// Its index.
        index: usize,
    },
    /// No guard held.
    Otherwise,
    /// A [`FollowingRule`]: this many civil days after the day of the rule
    /// named, whose facts the observance carries.
    After {
        /// The key of the rule counted from.
        rule: String,
        /// How many days after its day.
        days: u8,
    },
}

/// An observance: the day a rule falls on, and why.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// let found = first(Window::Sunrise)?;
/// assert_eq!((found.rule.as_str(), found.day.day), ("ACME_ASHTAMI", 3));
/// assert_eq!((found.month, found.adhika), (Masa::Shravana, false));
/// assert_eq!(found.tithi, Interval::literal(1.3, 2.4));
/// // The decision stands while the place moves the sunrises less than
/// // the 0.3 days to the tithi's start.
/// assert!(found.margin_seconds > 0.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Observance {
    /// The rule's key.
    pub rule: String,
    /// The day.
    pub day: CalendarDate,
    /// The occurrence judged: the tithi's, or the nakshatra's for a rule
    /// kept on one ([`Occurs::Nakshatra`]).
    pub tithi: Interval,
    /// Its amanta month, as an Ekadashi fast's is: which month's
    /// occurrence a rule kept every month decided.
    pub month: Masa,
    /// Whether that month is adhika.
    pub adhika: bool,
    /// How the tithi held the rite's time on the two days.
    pub case: Case,
    /// The earlier day's extent and the later's.
    pub extents: [Extent; 2],
    /// What decided.
    pub decided_by: Decided,
    /// What that decided.
    pub choice: Choice,
    /// **How far the place can move**, in seconds of time: the least time
    /// between an edge of what the rule read (the occurrence, and the
    /// nakshatras its guards name) and an instant the day was decided
    /// from (the two days' sunrises, the next sunrise, and every window
    /// the rule and its guards draw on them). Those edges do not move
    /// with the place and those instants do, and every comparison the
    /// rule makes is between the two, so the day stands at any place
    /// whose instants are each within this of these days'. 0 for a
    /// decision on two equal parts (C170), which any shift breaks.
    pub margin_seconds: f64,
    /// When the fast is broken, under each of the rule's readings in
    /// order (§8.7); left out when the rule has none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parana: Vec<VrataParanaTime>,
    /// The day's moonrise and its place, for a rule whose rite is at
    /// moonrise or whose fast is broken then (SR-60); left out otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moonrise: Option<MoonriseAt>,
    /// What a published entry did to it; absent when none touched it,
    /// so the day is the rule's (`03-design/published-overrides.md`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<published::Published>,
    /// When the rite is done on the day, for a rule the request carries a
    /// sait for (`03-design/festival-sait.md`); absent otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sait: Option<Sait>,
}

/// An occurrence a rule could not judge, and why.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// let moonless: Vec<FestivalDay> = days
///     .iter()
///     .cloned()
///     .map(|day| FestivalDay { moonrise: Moonrise::NoRise, ..day })
///     .collect();
/// let found = observances(&[rule(Window::Moonrise)], &moonless)?;
/// assert_eq!(found.unjudged[0].rule, "ACME_ASHTAMI");
/// assert_eq!(found.unjudged[0].tithi, Interval::literal(1.3, 2.4));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Unjudged {
    /// The rule's key.
    pub rule: String,
    /// The occurrence: the tithi's, or the nakshatra's for a rule kept on
    /// one.
    pub tithi: Interval,
    /// Why.
    pub why: String,
}

/// What a set of rules gives over a run of days.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// let found = observances(&[rule(Window::Sunrise)], &days)?;
/// assert_eq!(found.observances.len(), 1);
/// assert!(found.ekadashis.is_empty() && found.unjudged.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Observances {
    /// Each rule's days, in the order of the tithis.
    pub observances: Vec<Observance>,
    /// Each Ekadashi rule's fasts, in the order of the tithis.
    pub ekadashis: Vec<EkadashiFast>,
    /// The occurrences no day could be given to.
    pub unjudged: Vec<Unjudged>,
    /// Each dated rule's days, in the order of the days; left out when
    /// none, so an answer no dated rule was asked of is the one it was.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dated: Vec<DatedObservance>,
    /// Each anniversary's days, in the order of the tithis; left out when
    /// none was asked.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anniversaries: Vec<AnniversaryDay>,
    /// The days published sets added; left out when none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added: Vec<published::AddedDay>,
    /// The published entries in range no occurrence took; left out when
    /// none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unplaced: Vec<published::Unplaced>,
    /// The published sets laid over the answer, in order; left out when
    /// none, so an answer asked without them is the one it was.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applied: Vec<published::AppliedSet>,
}

impl Observances {
    /// Keeps only the days whose rule's key `kept` says, of every list
    /// that names one: what a request's meta filter does to its answer.
    ///
    /// ```
    /// use teistro_panchanga::festival::Observances;
    ///
    /// let mut none = Observances::default();
    /// none.keep(|_| false);
    /// assert_eq!(none, Observances::default());
    /// ```
    pub fn keep(&mut self, kept: impl Fn(&str) -> bool) {
        self.observances.retain(|found| kept(&found.rule));
        self.ekadashis.retain(|found| kept(&found.rule));
        self.unjudged.retain(|found| kept(&found.rule));
        self.dated.retain(|found| kept(&found.rule));
        self.anniversaries.retain(|found| kept(&found.key));
        self.added.retain(|found| kept(&found.key));
    }

    /// Where an answer names catalogue members, and of which kind: a list
    /// of [`Observances`], then a dotted path through each item. What a
    /// boundary section writes in full; the crate's tests hold it to
    /// serde, both ways.
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_panchanga::festival::Observances;
    ///
    /// assert_eq!(Observances::MEMBERS[0], ("observances", "month", Kind::Masa));
    /// ```
    pub const MEMBERS: [(&'static str, &'static str, Kind); 45] = [
        ("observances", "month", Kind::Masa),
        ("observances", "day.calendar", Kind::Calendar),
        ("observances", "day.era.era", Kind::Era),
        ("observances", "extents.day.calendar", Kind::Calendar),
        ("observances", "extents.day.era.era", Kind::Era),
        ("observances", "parana.day.calendar", Kind::Calendar),
        ("observances", "parana.day.era.era", Kind::Era),
        ("observances", "parana.reading.nakshatra", Kind::Nakshatra),
        ("ekadashis", "tithi", Kind::Tithi),
        ("ekadashis", "month", Kind::Masa),
        ("ekadashis", "days.calendar", Kind::Calendar),
        ("ekadashis", "days.era.era", Kind::Era),
        ("ekadashis", "day.calendar", Kind::Calendar),
        ("ekadashis", "day.era.era", Kind::Era),
        ("ekadashis", "parana.day.calendar", Kind::Calendar),
        ("ekadashis", "parana.day.era.era", Kind::Era),
        ("ekadashis", "parana.tithi", Kind::Tithi),
        ("ekadashis", "parana.barred.bar.tithi", Kind::Tithi),
        ("ekadashis", "parana.barred.bar.month", Kind::Masa),
        ("ekadashis", "parana.barred.bar.nakshatra", Kind::Nakshatra),
        ("dated", "day.calendar", Kind::Calendar),
        ("dated", "day.era.era", Kind::Era),
        ("dated", "on.calendar", Kind::Calendar),
        ("dated", "on.era.era", Kind::Era),
        ("anniversaries", "origin.tithi", Kind::Tithi),
        ("anniversaries", "origin.month", Kind::Masa),
        ("anniversaries", "origin.nakshatra", Kind::Nakshatra),
        ("anniversaries", "origin.candidates", Kind::Tithi),
        ("anniversaries", "observance.month", Kind::Masa),
        ("anniversaries", "observance.day.calendar", Kind::Calendar),
        ("anniversaries", "observance.day.era.era", Kind::Era),
        (
            "anniversaries",
            "observance.extents.day.calendar",
            Kind::Calendar,
        ),
        ("anniversaries", "observance.extents.day.era.era", Kind::Era),
        ("observances", "sait.candidates.lagna", Kind::Rashi),
        ("observances", "sait.candidates.navamsa", Kind::Rashi),
        ("observances", "published.computed.calendar", Kind::Calendar),
        ("observances", "published.computed.era.era", Kind::Era),
        ("ekadashis", "published.computed.calendar", Kind::Calendar),
        ("ekadashis", "published.computed.era.era", Kind::Era),
        ("dated", "published.computed.calendar", Kind::Calendar),
        ("dated", "published.computed.era.era", Kind::Era),
        ("added", "day.calendar", Kind::Calendar),
        ("added", "day.era.era", Kind::Era),
        ("unplaced", "day.calendar", Kind::Calendar),
        ("unplaced", "day.era.era", Kind::Era),
    ];

    /// Both answers as one, the occurrences no day was given to in the
    /// order of their tithis: how the karmakala rules' answer and the
    /// Ekadashi rules' are handed back together.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_panchanga::festival::{
    /// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
    /// #     SunEntry, Window, observances,
    /// # };
    /// # use teistro_panchanga::month;
    /// # use teistro_panchanga::span::Span;
    /// #
    /// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
    /// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
    /// # // and day 1's afternoon and evening.
    /// # let ends = [
    /// #     (Tithi::KrishnaSaptami, 1.3),
    /// #     (Tithi::KrishnaAshtami, 2.4),
    /// #     (Tithi::KrishnaNavami, 3.5),
    /// #     (Tithi::KrishnaDashami, 4.6),
    /// #     (Tithi::KrishnaEkadashi, 5.7),
    /// #     (Tithi::KrishnaDwadashi, 6.8),
    /// # ];
    /// # let days: Vec<FestivalDay> = (0..6u8)
    /// #     .map(|k| {
    /// #         let rise = f64::from(k);
    /// #         let window = Interval::literal(rise, rise + 1.0);
    /// #         let mut from = 0.0;
    /// #         let mut tithi = Vec::new();
    /// #         for (member, to) in ends {
    /// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
    /// #             from = to;
    /// #         }
    /// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
    /// #         FestivalDay {
    /// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
    /// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #             sunrise: JulianDay::literal(rise),
    /// #             sunset: JulianDay::literal(rise + 0.5),
    /// #             next_sunrise: JulianDay::literal(rise + 1.0),
    /// #             normal: true,
    /// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
    /// #             tithi,
    /// #             nakshatra: Vec::new(),
    /// #             moonrise: Moonrise::Unsearched,
    /// #             sun_entry: SunEntry::Unsearched,
    /// #             moon_arcs: MoonArcs::Unsearched,
    /// #         }
    /// #     })
    /// #     .collect();
    /// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
    /// # let rule = |at: Window| FestivalRule {
    /// #     key: String::from("ACME_ASHTAMI"),
    /// #     source: String::from("an example"),
    /// #     month: Some(Masa::Shravana),
    /// #     convention: LunarMonth::Amanta,
    /// #     occurs: Tithi::KrishnaAshtami.into(),
    /// #     in_adhika: InAdhika::Nija,
    /// #     at,
    /// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
    /// #     otherwise: Choice::Earlier,
    /// #     parana: Vec::new(),
    /// #     meta: Default::default(),
    /// # };
    /// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
    /// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
    /// # };
    ///
    /// let sunrise = observances(&[rule(Window::Sunrise)], &days)?;
    /// let evening = observances(&[FestivalRule { key: String::from("ACME_EVENING"), ..rule(Window::Pradosha) }], &days)?;
    /// assert_eq!(sunrise.merged(evening).observances.len(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn merged(mut self, other: Observances) -> Observances {
        self.observances.extend(other.observances);
        self.ekadashis.extend(other.ekadashis);
        self.unjudged.extend(other.unjudged);
        self.dated.extend(other.dated);
        self.anniversaries.extend(other.anniversaries);
        self.added.extend(other.added);
        self.unplaced.extend(other.unplaced);
        self.applied.extend(other.applied);
        self.anniversaries.sort_by(|a, b| {
            a.observance
                .tithi
                .from
                .get()
                .total_cmp(&b.observance.tithi.from.get())
        });
        self.unjudged
            .sort_by(|a, b| a.tithi.from.get().total_cmp(&b.tithi.from.get()));
        self
    }

    /// The answer as JSON, every catalogue member written as its full key
    /// where [`Observances::MEMBERS`] says: what a boundary section carries
    /// and seals.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_panchanga::festival::{
    /// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
    /// #     SunEntry, Window, observances,
    /// # };
    /// # use teistro_panchanga::month;
    /// # use teistro_panchanga::span::Span;
    /// #
    /// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
    /// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
    /// # // and day 1's afternoon and evening.
    /// # let ends = [
    /// #     (Tithi::KrishnaSaptami, 1.3),
    /// #     (Tithi::KrishnaAshtami, 2.4),
    /// #     (Tithi::KrishnaNavami, 3.5),
    /// #     (Tithi::KrishnaDashami, 4.6),
    /// #     (Tithi::KrishnaEkadashi, 5.7),
    /// #     (Tithi::KrishnaDwadashi, 6.8),
    /// # ];
    /// # let days: Vec<FestivalDay> = (0..6u8)
    /// #     .map(|k| {
    /// #         let rise = f64::from(k);
    /// #         let window = Interval::literal(rise, rise + 1.0);
    /// #         let mut from = 0.0;
    /// #         let mut tithi = Vec::new();
    /// #         for (member, to) in ends {
    /// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
    /// #             from = to;
    /// #         }
    /// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
    /// #         FestivalDay {
    /// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
    /// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #             sunrise: JulianDay::literal(rise),
    /// #             sunset: JulianDay::literal(rise + 0.5),
    /// #             next_sunrise: JulianDay::literal(rise + 1.0),
    /// #             normal: true,
    /// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
    /// #             tithi,
    /// #             nakshatra: Vec::new(),
    /// #             moonrise: Moonrise::Unsearched,
    /// #             sun_entry: SunEntry::Unsearched,
    /// #             moon_arcs: MoonArcs::Unsearched,
    /// #         }
    /// #     })
    /// #     .collect();
    /// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
    /// # let rule = |at: Window| FestivalRule {
    /// #     key: String::from("ACME_ASHTAMI"),
    /// #     source: String::from("an example"),
    /// #     month: Some(Masa::Shravana),
    /// #     convention: LunarMonth::Amanta,
    /// #     occurs: Tithi::KrishnaAshtami.into(),
    /// #     in_adhika: InAdhika::Nija,
    /// #     at,
    /// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
    /// #     otherwise: Choice::Earlier,
    /// #     parana: Vec::new(),
    /// #     meta: Default::default(),
    /// # };
    /// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
    /// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
    /// # };
    ///
    /// let json = observances(&[rule(Window::Sunrise)], &days)?.in_full()?;
    /// assert_eq!(json["observances"][0]["month"], "masa.SHRAVANA");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the answer does not serialise, which a value this
    /// crate built cannot do.
    pub fn in_full(&self) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(self).map_err(|error| {
            Error::internal(format!("the observances did not serialise: {error}"))
        })?;
        for (list, path, kind) in Observances::MEMBERS {
            if let Some(items) = value.get_mut(list) {
                write_in_full(items, path, kind);
            }
        }
        Ok(value)
    }
}

/// The days each rule falls on over consecutive sunrise days.
///
/// An occurrence is judged when its tithi, or the nakshatra a rule is kept
/// on, begins on one of the days and the days hold both it and the one
/// after (C172); the caller widens the run by a day on each side for a
/// range's edges. The answer is in the order the occurrences begin.
///
/// ```
/// # use teistro_calendar::CalendarDate;
/// # use teistro_calendar::lunisolar::MonthKind;
/// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::LunarMonth;
/// # use teistro_panchanga::festival::{
/// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
/// #     SunEntry, Window, observances,
/// # };
/// # use teistro_panchanga::month;
/// # use teistro_panchanga::span::Span;
/// #
/// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
/// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
/// # // and day 1's afternoon and evening.
/// # let ends = [
/// #     (Tithi::KrishnaSaptami, 1.3),
/// #     (Tithi::KrishnaAshtami, 2.4),
/// #     (Tithi::KrishnaNavami, 3.5),
/// #     (Tithi::KrishnaDashami, 4.6),
/// #     (Tithi::KrishnaEkadashi, 5.7),
/// #     (Tithi::KrishnaDwadashi, 6.8),
/// # ];
/// # let days: Vec<FestivalDay> = (0..6u8)
/// #     .map(|k| {
/// #         let rise = f64::from(k);
/// #         let window = Interval::literal(rise, rise + 1.0);
/// #         let mut from = 0.0;
/// #         let mut tithi = Vec::new();
/// #         for (member, to) in ends {
/// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
/// #             from = to;
/// #         }
/// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
/// #         FestivalDay {
/// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
/// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
/// #             sunrise: JulianDay::literal(rise),
/// #             sunset: JulianDay::literal(rise + 0.5),
/// #             next_sunrise: JulianDay::literal(rise + 1.0),
/// #             normal: true,
/// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
/// #             tithi,
/// #             nakshatra: Vec::new(),
/// #             moonrise: Moonrise::Unsearched,
/// #             sun_entry: SunEntry::Unsearched,
/// #             moon_arcs: MoonArcs::Unsearched,
/// #         }
/// #     })
/// #     .collect();
/// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
/// # let rule = |at: Window| FestivalRule {
/// #     key: String::from("ACME_ASHTAMI"),
/// #     source: String::from("an example"),
/// #     month: Some(Masa::Shravana),
/// #     convention: LunarMonth::Amanta,
/// #     occurs: Tithi::KrishnaAshtami.into(),
/// #     in_adhika: InAdhika::Nija,
/// #     at,
/// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
/// #     otherwise: Choice::Earlier,
/// #     parana: Vec::new(),
/// #     meta: Default::default(),
/// # };
/// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
/// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
/// # };
///
/// // The 3rd of January's sunrise holds the ashtami; the 2nd's does not.
/// let found = observances(&[rule(Window::Sunrise)], &days)?;
/// assert_eq!(found.observances[0].day.day, 3);
/// // A rule its own check refuses is refused here.
/// let unnamed = FestivalRule { key: String::new(), ..rule(Window::Sunrise) };
/// assert!(observances(&[unnamed], &days).is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// A rule's own refusal ([`FestivalRule::check`]), naming it.
pub fn observances(rules: &[FestivalRule], days: &[FestivalDay]) -> Result<Observances, Error> {
    observances_over(rules, &Reckoned::new(days))
}

/// What every rule reads of a run of days, found once: the occurrences a
/// batch of rules over the same days shares (`03-design/anniversary-span.md`
/// §4), so that a rule costs its own occurrences and not a walk of the days.
pub(crate) struct Reckoned<'a> {
    days: &'a [FestivalDay],
    tithis: Vec<Span<Tithi>>,
    nakshatras: Vec<Span<Nakshatra>>,
}

impl<'a> Reckoned<'a> {
    /// The occurrences of consecutive sunrise days.
    pub(crate) fn new(days: &'a [FestivalDay]) -> Reckoned<'a> {
        Reckoned {
            days,
            tithis: occurrences(days),
            nakshatras: distinct(days.iter().flat_map(|day| day.nakshatra.iter())),
        }
    }

    /// How many of the days' sunrises an occurrence holds.
    pub(crate) fn sunrises_in(&self, span: Interval) -> usize {
        let before = |at: f64| self.days.partition_point(|day| day.sunrise.get() < at);
        before(span.to.get()) - before(span.from.get())
    }

    /// The day an occurrence was kept on, of the two it is judged between.
    pub(crate) fn kept_on(&self, span: Interval, date: &CalendarDate) -> Option<&'a FestivalDay> {
        let (earlier, later) = pair(span, self.days)?;
        [earlier, later].into_iter().find(|day| day.date == *date)
    }
}

/// [`observances`] over days already reckoned.
pub(crate) fn observances_over(
    rules: &[FestivalRule],
    reckoned: &Reckoned<'_>,
) -> Result<Observances, Error> {
    let Reckoned {
        days,
        tithis,
        nakshatras,
    } = reckoned;
    let days = *days;
    for rule in rules {
        rule.check()?;
        for (index, reading) in rule.parana.iter().enumerate() {
            if let VrataParana::AfterRite { rite, .. } = reading
                && !rules.iter().any(|other| other.key == *rite)
            {
                return Err(Error::invalid_arg(format!(
                    "{}: its parana follows {rite}, which no rule asked for keeps; ask for that rule too",
                    rule.key
                ))
                .with_field(format!("parana[{index}].rite")));
            }
        }
    }
    let mut kept: Vec<(&FestivalRule, Interval, Result<LunarMonth, String>)> = Vec::new();
    for rule in rules {
        match rule.occurs {
            Occurs::Tithi { .. } | Occurs::DayOfPaksha { .. } => {
                for occurrence in tithis.iter().filter(|held| rule.occurs.keeps(held.member)) {
                    if let Some(month) = month_of(rule, occurrence, days) {
                        kept.push((rule, occurrence.whole, month));
                    }
                }
            }
            Occurs::Nakshatra { nakshatra, paksha } => {
                for occurrence in nakshatras.iter().filter(|held| held.member == nakshatra) {
                    let Some(tithi) = running_at_middle(occurrence.whole, tithis) else {
                        continue;
                    };
                    if tithi.member.attributes().paksha != paksha {
                        continue;
                    }
                    if let Some(month) = month_of(rule, tithi, days) {
                        kept.push((rule, occurrence.whole, month));
                    }
                }
            }
        }
    }
    kept.sort_by(|a, b| a.1.from.get().total_cmp(&b.1.from.get()));
    // A rule whose parana follows another's rite is judged once every
    // rite's day is known (`VrataParana::AfterRite`).
    let (first, after): (Vec<_>, Vec<_>) = kept
        .into_iter()
        .partition(|(rule, ..)| !rule.parana.iter().any(VrataParana::follows_a_rite));
    let mut answer = Observances::default();
    for pass in [first, after] {
        let rites = Rites {
            rules,
            days,
            kept: answer.observances.clone(),
        };
        for (rule, whole, month) in pass {
            let judged = match month {
                Ok(month) => judge(rule, whole, month, days, &rites)?,
                Err(why) => Err(why),
            };
            match judged {
                Ok(Some(observance)) => answer.observances.push(observance),
                Ok(None) => {}
                Err(why) => answer.unjudged.push(Unjudged {
                    rule: rule.key.clone(),
                    tithi: whole,
                    why,
                }),
            }
        }
    }
    answer
        .observances
        .sort_by(|a, b| a.tithi.from.get().total_cmp(&b.tithi.from.get()));
    answer
        .unjudged
        .sort_by(|a, b| a.tithi.from.get().total_cmp(&b.tithi.from.get()));
    Ok(answer)
}

/// The closing rites judged so far, which a parana after a rite reads.
struct Rites<'a> {
    rules: &'a [FestivalRule],
    days: &'a [FestivalDay],
    kept: Vec<Observance>,
}

impl<'a> Rites<'a> {
    /// The first day of `key`'s rite from the fast day on, and its first
    /// instant: the day's sunrise, the rite's tithi's start, and the start
    /// of the share of a nakshatra its deciding guard joined, the last.
    fn after(&self, key: &str, fast: &FestivalDay) -> Option<RiteAt<'a>> {
        let rule = self.rules.iter().find(|rule| rule.key == key)?;
        self.kept
            .iter()
            .filter(|kept| kept.rule == key)
            .find_map(|kept| {
                let day = self.days.iter().find(|day| day.date == kept.day)?;
                (day.sunrise.get() >= fast.sunrise.get()).then_some((kept, day))
            })
            .map(|(kept, day)| {
                let mut at = day.sunrise.get().max(kept.tithi.from.get());
                if let Decided::Guard { index } = kept.decided_by {
                    let joined = rule
                        .decide
                        .get(index)
                        .into_iter()
                        .flat_map(|guard| &guard.when)
                        .filter_map(|predicate| match *predicate {
                            Predicate::Joined {
                                nakshatra,
                                portion: Some(portion),
                                ..
                            } => joined_arcs(day, nakshatra, portion).ok(),
                            _ => None,
                        })
                        .flatten()
                        .filter(|arc| {
                            arc.span.to.get() > day.sunrise.get()
                                && arc.span.from.get() < day.next_sunrise.get()
                        })
                        .map(|arc| arc.span.from.get())
                        .fold(f64::NEG_INFINITY, f64::max);
                    at = at.max(joined);
                }
                RiteAt { day, at }
            })
    }
}

/// Every tithi the days hold, once each, in order.
fn occurrences(days: &[FestivalDay]) -> Vec<Span<Tithi>> {
    distinct(days.iter().flat_map(|day| day.tithi.iter()))
}

/// Every member the spans hold, once each, in order.
fn distinct<'a, T: Copy + 'a>(spans: impl Iterator<Item = &'a Span<T>>) -> Vec<Span<T>> {
    let mut seen: Vec<Span<T>> = Vec::new();
    for span in spans {
        if seen
            .last()
            .is_none_or(|last| last.whole.from.get() < span.whole.from.get())
        {
            seen.push(*span);
        }
    }
    seen
}

/// The tithi running at the middle of a span: what names a nakshatra's
/// month and paksha.
fn running_at_middle(span: Interval, tithis: &[Span<Tithi>]) -> Option<&Span<Tithi>> {
    let middle = span.at_fraction(0.5).ok()?;
    tithis.iter().find(|tithi| tithi.whole.contains(middle))
}

/// The month the occurrence falls in, when it is one the rule keeps.
///
/// The month is read at a sunrise inside it: the first at or after the
/// tithi begins, except for the new moon, which ends its month, and so
/// the last sunrise before it ends.
///
/// `None` when the rule does not keep it; an error when whether it does
/// turns on a month the days do not reach.
fn month_of(
    rule: &FestivalRule,
    occurrence: &Span<Tithi>,
    days: &[FestivalDay],
) -> Option<Result<LunarMonth, String>> {
    // The days run in order, so each is found without walking them, which
    // a batch over a decade would otherwise pay for every occurrence.
    let at = if occurrence.member == Tithi::Amavasya {
        days.partition_point(|day| day.sunrise.get() < occurrence.whole.to.get())
            .checked_sub(1)?
    } else {
        days.partition_point(|day| day.sunrise.get() < occurrence.whole.from.get())
    };
    let day = days.get(at)?;
    let month = month::of(
        day.month.amanta,
        occurrence.member,
        rule.convention,
        day.month.kind,
    );
    let name = month.under(rule.convention);
    if rule.month.is_some_and(|wanted| name != wanted) {
        return None;
    }
    let adhika = month.kind == MonthKind::Adhika;
    match rule.in_adhika {
        InAdhika::Also => Some(Ok(month)),
        InAdhika::Nija => (!adhika).then_some(Ok(month)),
        InAdhika::Only | InAdhika::First if adhika => Some(Ok(month)),
        InAdhika::Only => None,
        InAdhika::First => match doubled(days, at, name) {
            Some(true) => None,
            Some(false) => Some(Ok(month)),
            None => Some(Err(format!(
                "the days do not reach the month next to this {}, which says whether it is doubled",
                name.key()
            ))),
        },
    }
}

/// Whether the year doubles the month a nija day's occurrence is named,
/// read from the lunation beside the day's own: an adhika month is new
/// moon to new moon under both conventions, so it comes just before the
/// amanta nija month of its name, and so just after the dark fortnight a
/// purnimanta almanac names after it. `None` when the days stop first.
fn doubled(days: &[FestivalDay], at: usize, name: Masa) -> Option<bool> {
    let own = &days.get(at)?.month;
    let other = |day: &&FestivalDay| day.month.amanta != own.amanta || day.month.kind != own.kind;
    let beside = if own.amanta == name {
        days.get(..at)?.iter().rev().find(other)
    } else {
        days.get(at..)?.iter().find(other)
    }?;
    Some(beside.month.amanta == name && beside.month.kind == MonthKind::Adhika)
}

/// The two days an occurrence is judged between (C172): the sunrise day
/// it begins in and the next, or, when it holds two sunrises, the two it
/// holds them on.
fn pair(tithi: Interval, days: &[FestivalDay]) -> Option<(&FestivalDay, &FestivalDay)> {
    let begins = days.partition_point(|day| day.next_sunrise.get() <= tithi.from.get());
    if !days.get(begins)?.sunrise_day().contains(tithi.from) {
        return None;
    }
    let first = match days.get(begins + 2) {
        Some(third) if tithi.contains(third.sunrise) => begins + 1,
        _ => begins,
    };
    Some((days.get(first)?, days.get(first + 1)?))
}

fn extent(day: &FestivalDay, window: Window, tithi: Interval) -> Result<Extent, Error> {
    let span = day.window(window)?;
    let held = if span.days() <= 0.0 {
        if tithi.contains(span.from) { 1.0 } else { 0.0 }
    } else {
        tithi
            .clipped_to(span)
            .map_or(0.0, |held| held.days() / span.days())
    };
    Ok(Extent {
        day: day.date.clone(),
        window: span,
        held,
    })
}

fn case_of(earlier: f64, later: f64) -> Case {
    match (earlier > 0.0, later > 0.0) {
        (true, false) => Case::EarlierOnly,
        (false, true) => Case::LaterOnly,
        (false, false) => Case::Neither,
        (true, true) if earlier >= 1.0 && later >= 1.0 => Case::Both,
        #[allow(
            clippy::float_cmp,
            reason = "C170: equal is stated, no tolerance invented"
        )]
        (true, true) if earlier == later => Case::EqualParts,
        (true, true) => Case::UnequalParts,
    }
}

/// The kept day's moonrise, for a rule kept at moonrise or whose fast is
/// broken then (SR-60); none for any other rule.
fn moonrise_kept(rule: &FestivalRule, kept: &FestivalDay) -> Option<MoonriseAt> {
    match kept.moonrise {
        Moonrise::Rose(seen) if rule.at == Window::Moonrise || rule.breaks_at_moonrise() => {
            Some(seen)
        }
        Moonrise::Rose(_) | Moonrise::Unsearched | Moonrise::NoRise => None,
    }
}

/// The observance, none when the rule chose [`Choice::None`], or why the
/// occurrence could not be judged.
type Judged = Result<Option<Observance>, String>;

fn judge(
    rule: &FestivalRule,
    tithi: Interval,
    month: LunarMonth,
    days: &[FestivalDay],
    rites: &Rites<'_>,
) -> Result<Judged, Error> {
    let Some((earlier, later)) = pair(tithi, days) else {
        return Ok(Err(
            "the days do not reach the tithi and the day after it".to_owned()
        ));
    };
    if !(earlier.normal && later.normal) {
        return Ok(Err(format!(
            "{} or {} is a polar day, which has no windows",
            earlier.date, later.date
        )));
    }
    if rule.judged_at_moonrise()
        && let Some(day) = [earlier, later]
            .into_iter()
            .find(|day| day.moonrise == Moonrise::NoRise)
    {
        return Ok(Err(format!(
            "the Moon does not rise on {} between its two sunrises",
            day.date
        )));
    }
    let extents = [
        extent(earlier, rule.at, tithi)?,
        extent(later, rule.at, tithi)?,
    ];
    let case = case_of(extents[0].held, extents[1].held);
    let facts = Facts {
        tithi,
        case,
        held: [extents[0].held, extents[1].held],
        time: [held_time(&extents[0], tithi), held_time(&extents[1], tithi)],
        earlier,
        later,
    };
    let mut decided_by = Decided::Otherwise;
    let mut choice = rule.otherwise;
    for (index, guard) in rule.decide.iter().enumerate() {
        if guard.when.iter().try_fold(true, |all, predicate| {
            Ok::<bool, Error>(all && facts.holds(*predicate)?)
        })? {
            decided_by = Decided::Guard { index };
            choice = guard.choose;
            break;
        }
    }
    let which = match choice {
        Choice::Earlier => Which::Earlier,
        Choice::Later => Which::Later,
        Choice::ByYugma => rule
            .occurs
            .tithi()
            .and_then(yugma)
            .unwrap_or(Which::Earlier),
        Choice::None => return Ok(Ok(None)),
    };
    let kept = match which {
        Which::Earlier => earlier,
        Which::Later => later,
    };
    let day = kept.date.clone();
    let parana = if rule.parana.is_empty() {
        Vec::new()
    } else {
        let from = days
            .iter()
            .position(|held| std::ptr::eq(held, kept))
            .and_then(|at| days.get(at..))
            .unwrap_or_default();
        let rite = |key: &str| rites.after(key, kept);
        match vrata::paranas(&rule.parana, tithi, from, &rite) {
            Ok(parana) => parana,
            Err(why) => return Ok(Err(why)),
        }
    };
    let margin_seconds = if case == Case::EqualParts {
        0.0
    } else {
        margin(rule, tithi, earlier, later, facts.time)? * SECONDS_PER_DAY
    };
    Ok(Ok(Some(Observance {
        rule: rule.key.clone(),
        day,
        tithi,
        month: month.amanta,
        adhika: month.kind == MonthKind::Adhika,
        case,
        extents,
        decided_by,
        choice,
        margin_seconds,
        parana,
        moonrise: moonrise_kept(rule, kept),
        published: None,
        sait: None,
    })))
}

const SECONDS_PER_DAY: f64 = 86_400.0;

/// How long the tithi held a day's window, in days; at an instant, 1
/// when it held it and 0 when not, so that [`Predicate::Greater`] reads
/// an instant as the cases do.
fn held_time(extent: &Extent, tithi: Interval) -> f64 {
    if extent.window.days() <= 0.0 {
        extent.held
    } else {
        tithi.clipped_to(extent.window).map_or(0.0, Interval::days)
    }
}

/// The least time, in days, between an edge the rule read and an instant
/// the two days were decided from ([`Observance::margin_seconds`]); and,
/// for a rule weighing the two days' times ([`Predicate::Greater`]), half
/// their difference, since a shift of the place moves each by as much as
/// it moves the windows.
fn margin(
    rule: &FestivalRule,
    tithi: Interval,
    earlier: &FestivalDay,
    later: &FestivalDay,
    time: [f64; 2],
) -> Result<f64, Error> {
    let mut instants = vec![
        earlier.sunrise.get(),
        later.sunrise.get(),
        later.next_sunrise.get(),
    ];
    let mut edges = vec![tithi.from.get(), tithi.to.get()];
    let mut window = |span: Interval| instants.extend([span.from.get(), span.to.get()]);
    for day in [earlier, later] {
        window(day.window(rule.at)?);
    }
    let mut named = Vec::new();
    for predicate in rule.decide.iter().flat_map(|guard| &guard.when) {
        let day = match *predicate {
            Predicate::Joined { day, .. }
            | Predicate::Stands { day, .. }
            | Predicate::Lasts { day, .. } => match day {
                Which::Earlier => earlier,
                Which::Later => later,
            },
            _ => continue,
        };
        match *predicate {
            Predicate::Joined {
                nakshatra,
                at,
                portion,
                ..
            } => {
                window(at.map_or_else(|| Ok(day.sunrise_day()), |at| day.window(at))?);
                match portion {
                    None => named.push((day, nakshatra)),
                    Some(portion) => edges.extend(
                        joined_arcs(day, nakshatra, portion)?
                            .flat_map(|arc| [arc.span.from.get(), arc.span.to.get()]),
                    ),
                }
            }
            Predicate::Stands { nakshatra, at, .. } => {
                window(day.window(at)?);
                named.push((day, nakshatra));
            }
            Predicate::Lasts { from, ghatis, .. } => {
                let (edge, span) = match from {
                    Edge::Sunrise => (day.sunrise, day.daylight()),
                    Edge::Sunset => (day.sunset, day.night()),
                };
                window(Interval::literal(
                    edge.get(),
                    span.at_fraction(f64::from(ghatis) / 30.0)?.get(),
                ));
            }
            _ => {}
        }
    }
    let weighed = rule
        .decide
        .iter()
        .flat_map(|guard| &guard.when)
        .any(|predicate| matches!(predicate, Predicate::Greater { .. }))
        && earlier.window(rule.at)?.days() > 0.0;
    for (day, nakshatra) in named {
        edges.extend(
            day.nakshatra
                .iter()
                .filter(|held| held.member == nakshatra)
                .flat_map(|held| [held.whole.from.get(), held.whole.to.get()]),
        );
    }
    let edges = instants
        .iter()
        .flat_map(|instant| edges.iter().map(move |edge| (instant - edge).abs()))
        .fold(f64::INFINITY, f64::min);
    Ok(if weighed {
        edges.min((time[0] - time[1]).abs() / 2.0)
    } else {
        edges
    })
}

/// The tithi and the whole of a nakshatra together on a day.
const fn joined(day: Which, nakshatra: Nakshatra, at: Option<Window>) -> Predicate {
    Predicate::Joined {
        day,
        nakshatra,
        at,
        portion: None,
    }
}

/// The Moon's stays in `portion` of `nakshatra`'s arc that meet `day`,
/// refused when no search filled the day's arcs.
fn joined_arcs(
    day: &FestivalDay,
    nakshatra: Nakshatra,
    portion: Portion,
) -> Result<impl Iterator<Item = &MoonArc>, Error> {
    let MoonArcs::Found(arcs) = &day.moon_arcs else {
        return Err(Error::invalid_arg(format!(
            "a rule joins a share of a nakshatra on {}, whose Moon arcs no search found; fill them first (`Almanac::moon_arcs`)",
            day.date
        )));
    };
    let arc = portion.arc_deg(nakshatra);
    Ok(arcs.iter().filter(move |found| found.is_of(arc)))
}

struct Facts<'a> {
    tithi: Interval,
    case: Case,
    /// The fraction of each day's window the tithi held, earlier first.
    held: [f64; 2],
    /// How long it held each, as [`held_time`] reads it.
    time: [f64; 2],
    earlier: &'a FestivalDay,
    later: &'a FestivalDay,
}

impl Facts<'_> {
    fn day(&self, which: Which) -> &FestivalDay {
        match which {
            Which::Earlier => self.earlier,
            Which::Later => self.later,
        }
    }

    fn holds(&self, predicate: Predicate) -> Result<bool, Error> {
        Ok(match predicate {
            Predicate::Case { case } => self.case == case,
            Predicate::Joined {
                day,
                nakshatra,
                at,
                portion,
            } => {
                let day = self.day(day);
                let span = match at {
                    Some(window) => day.window(window)?,
                    None => day.sunrise_day(),
                };
                match portion {
                    None => day
                        .nakshatra
                        .iter()
                        .filter(|held| held.member == nakshatra)
                        .any(|held| meet(&[self.tithi, held.whole], span)),
                    Some(portion) => joined_arcs(day, nakshatra, portion)?
                        .any(|arc| meet(&[self.tithi, arc.span], span)),
                }
            }
            Predicate::Stands { day, nakshatra, at } => {
                let day = self.day(day);
                let window = day.window(at)?;
                day.nakshatra
                    .iter()
                    .filter(|held| held.member == nakshatra)
                    .any(|held| meet(&[held.whole], window))
            }
            Predicate::Lasts { day, from, ghatis } => {
                let day = self.day(day);
                let (edge, span) = match from {
                    Edge::Sunrise => (day.sunrise, day.daylight()),
                    Edge::Sunset => (day.sunset, day.night()),
                };
                let until = span.at_fraction(f64::from(ghatis) / 30.0)?;
                self.tithi.contains(edge) && self.tithi.to.get() >= until.get()
            }
            Predicate::Wholly { day } => {
                let held = match day {
                    Which::Earlier => self.held[0],
                    Which::Later => self.held[1],
                };
                held >= 1.0
            }
            Predicate::OnVara { day, vara } => self.day(day).vara == vara,
            Predicate::Greater { day } => {
                let [earlier, later] = self.time;
                match day {
                    Which::Earlier => earlier > later,
                    Which::Later => later > earlier,
                }
            }
        })
    }
}

/// Whether every span stands in a window together: at its instant, as
/// `contains` reads it, or for some time inside it.
fn meet(spans: &[Interval], window: Interval) -> bool {
    if window.days() <= 0.0 {
        return spans.iter().all(|span| span.contains(window.from));
    }
    let from = spans
        .iter()
        .fold(window.from.get(), |from, span| from.max(span.from.get()));
    let to = spans
        .iter()
        .fold(window.to.get(), |to, span| to.min(span.to.get()));
    from < to
}

/// A guard's predicate: the case is this one.
const fn case(case: Case) -> Predicate {
    Predicate::Case { case }
}

/// The third fifth of the daylight.
const MADHYAHNA: Window = Window::Part {
    part: DayPart::Madhyahna,
};

/// The fourth fifth of the daylight.
const APARAHNA: Window = Window::Part {
    part: DayPart::Aparahna,
};

/// Lakshmi puja (*Dharmasindhu* p. 77): the new moon at pradosha.
fn lakshmi_puja() -> FestivalRule {
    use Choice::{Earlier, Later};
    use Which::Later as L;
    FestivalRule {
        key: "LAKSHMI_PUJA".to_owned(),
        source: "Dharmasindhu p. 77: the new moon at pradosha; the later day when it lasts more than a ghati into that night, which puts the matter beyond doubt, else the earlier, and so when neither day holds it".to_owned(),
        month: Some(Masa::Ashwina),
        convention: Convention::Amanta,
        occurs: Tithi::Amavasya.into(),
        in_adhika: InAdhika::Nija,
        at: Window::Pradosha,
        decide: vec![Guard::new(
            [Predicate::Lasts {
                day: L,
                from: Edge::Sunset,
                ghatis: 1,
            }],
            Later,
        )],
        otherwise: Earlier,
        parana: Vec::new(),
        meta: FestivalMeta::default(),
    }
}

impl FestivalRule {
    /// The rules *Dharmasindhu* states with a table this evaluator reads,
    /// each citing its page in the 1888 Nirnaya-sagara edition: the four
    /// of `festival-rules.md` §1, then the seven Nepal's panchanga prints
    /// (§9), then three schools' upakarma (§9.5, §9.7), then the three
    /// vratas kept at moonrise (`festival-rule-language.md` §3.1),
    /// then the Navaratri's Mahanavami and visarjana (§8.7).
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let rules = FestivalRule::dharmasindhu();
    /// for key in ["RAMA_NAVAMI", "JANMASHTAMI", "VIJAYA_DASHAMI", "LAKSHMI_PUJA"] {
    ///     assert!(rules.iter().any(|rule| rule.key == key), "{key}");
    /// }
    /// assert!(rules.iter().all(|rule| rule.check().is_ok()));
    /// ```
    #[must_use]
    pub fn dharmasindhu() -> Vec<FestivalRule> {
        let mut rules = FestivalRule::printed_in_nepal();
        rules.extend(FestivalRule::at_moonrise());
        rules.extend(FestivalRule::navaratri());
        rules
    }

    /// The Sharada Navaratri's close (*Dharmasindhu* pp. 69–71): the 9th
    /// of the puja and the fast, and the Devi's visarjana on the 10th,
    /// after which the fast is broken; with *Nirnayasindhu* p. 139's
    /// reading second, the first daylight of the 10th. The keys are the
    /// ones `NEPAL_PANCHANGA` prints these days under.
    fn navaratri() -> Vec<FestivalRule> {
        use Choice::{Earlier, Later};
        use Which::Earlier as E;
        let morning_to_aparahna = Window::DayMuhurta { from: 1, to: 12 };
        vec![
            FestivalRule {
                key: "MAHA_NAVAMI".to_owned(),
                source: "Dharmasindhu p. 69: the 9th of the puja and the fast is taken joined with the 8th when it stands three muhurtas in that evening (सायं त्रिमुहूर्ता), the later day otherwise; the fast is broken on the visarjana's day after it, or inside the 9th left at the next sunrise when the visarjana falls a day later (pp. 70–71); Nirnayasindhu p. 139 breaks it on the 10th, at its first daylight".to_owned(),
                month: Some(Masa::Ashwina),
                convention: Convention::Amanta,
                occurs: Tithi::ShuklaNavami.into(),
                in_adhika: InAdhika::Nija,
                at: Window::DayMuhurta { from: 13, to: 15 },
                decide: vec![Guard::new([Predicate::Wholly { day: E }], Earlier)],
                otherwise: Later,
                parana: vec![
                    VrataParana::AfterRite {
                        rite: "DEVI_VISARJAN".to_owned(),
                        remnant: Some(0),
                    },
                    VrataParana::NextTithi,
                ],
                meta: FestivalMeta::default(),
            },
            FestivalRule {
                key: "DEVI_VISARJAN".to_owned(),
                source: "Dharmasindhu p. 70: the visarjana is on the 10th; of two, on the earlier when Shravana's last quarter joins it (श्रवणान्त्यपादयोगे), by the aparahna since a visarjana kept for the nakshatra may fall then, and on the later otherwise; on the earlier when the later holds no 10th".to_owned(),
                month: Some(Masa::Ashwina),
                convention: Convention::Amanta,
                occurs: Tithi::ShuklaDashami.into(),
                in_adhika: InAdhika::Nija,
                at: morning_to_aparahna,
                decide: vec![
                    Guard::new(
                        [Predicate::Joined {
                            day: E,
                            nakshatra: Nakshatra::Shravana,
                            at: Some(morning_to_aparahna),
                            portion: Some(Portion { parts: 4, which: 4 }),
                        }],
                        Earlier,
                    ),
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                ],
                otherwise: Later,
                parana: Vec::new(),
                meta: FestivalMeta::default(),
            },
        ]
    }

    /// The text's rules Nepal's national panchanga keeps: all but the three
    /// vratas at moonrise, which it does not print.
    fn printed_in_nepal() -> Vec<FestivalRule> {
        let mut rules = FestivalRule::first_four();
        rules.extend(FestivalRule::nepal_four());
        rules.extend(FestivalRule::nepal_three());
        rules.extend(FestivalRule::upakarma());
        rules
    }

    /// The Sankashta chaturthi, every month's dark 4th, and the Karaka
    /// chaturthi, Ashvina's (*Dharmasindhu* pp. 9, 76): the tithi at
    /// moonrise; the day whose moonrise alone holds it, the earlier,
    /// joined to the 3rd, when both do, and the later when neither does.
    /// Then the Ashunya-shayana vrata, Ashadha's dark 2nd (p. 44), at
    /// moonrise too, but the later day when both or neither hold it, and
    /// broken at moonrise (`festival-rule-language.md` §3.1).
    fn at_moonrise() -> Vec<FestivalRule> {
        let rule = |key: &str, source: &str, month, in_adhika| FestivalRule {
            key: key.to_owned(),
            source: source.to_owned(),
            month,
            convention: Convention::Amanta,
            occurs: Tithi::KrishnaChaturthi.into(),
            in_adhika,
            at: Window::Moonrise,
            decide: vec![
                Guard::new([case(Case::EarlierOnly)], Choice::Earlier),
                Guard::new([case(Case::Both)], Choice::Earlier),
            ],
            otherwise: Choice::Later,
            parana: Vec::new(),
            meta: FestivalMeta::default(),
        };
        vec![
            rule(
                "SANKASHTA_CHATURTHI",
                "Dharmasindhu p. 9: the dark 4th at moonrise (चन्द्रोदयव्यापिनी); the day whose moonrise alone holds it, the earlier, joined to the 3rd, when both do, the later when neither does; every month, the adhika too",
                None,
                InAdhika::Also,
            ),
            rule(
                "KARAKA_CHATURTHI",
                "Dharmasindhu p. 76: Ashvina's dark 4th, at moonrise, decided as the Sankashta chaturthi (p. 9)",
                Some(Masa::Ashwina),
                InAdhika::Nija,
            ),
            FestivalRule {
                key: "ASHUNYA_SHAYANA".to_owned(),
                source: "Dharmasindhu p. 44: Ashadha's dark 2nd, the tithi at moonrise since the puja is then (चन्द्रोदयव्यापिनी तिथिर्ग्राह्या चन्द्रोदये पूजायुक्तेः); the later day when both days hold it or neither does (दिनद्वये सत्त्वेऽसत्त्वे वा परैव); Nirnayasindhu p. 94, after the Nirnayamrita, the same; the vow returns on the next three months' dark 2nds, which are not rules here (C1082)".to_owned(),
                month: Some(Masa::Ashadha),
                convention: Convention::Amanta,
                occurs: Tithi::KrishnaDvitiya.into(),
                in_adhika: InAdhika::Nija,
                at: Window::Moonrise,
                decide: vec![Guard::new([case(Case::EarlierOnly)], Choice::Earlier)],
                otherwise: Choice::Later,
                parana: Vec::new(),
                meta: FestivalMeta::default(),
            },
        ]
    }

    /// *Dharmasindhu*'s rules as Nepal's national panchanga keeps them
    /// (`festival-rules.md` §9.5, C197): a rite of the night or the evening
    /// on the text's day, and a rite of the daylight on the day whose
    /// sunrise holds its tithi ([`FestivalRule::udaya`]). The Nepal
    /// Panchanga Decision Committee's printed days for VS 2082 and 2083
    /// agree with this reading on every row, and with the text alone on all
    /// but the three daylight rites where the two part.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let nepal = FestivalRule::nepal();
    /// let tika = nepal.iter().find(|rule| rule.key == "VIJAYA_DASHAMI").ok_or("shipped")?;
    /// assert!(tika.decide.len() == 1 && tika.source.contains("C197"));
    /// let lakshmi = nepal.iter().find(|rule| rule.key == "LAKSHMI_PUJA").ok_or("shipped")?;
    /// assert_eq!(Some(lakshmi), FestivalRule::dharmasindhu().iter().find(|rule| rule.key == "LAKSHMI_PUJA"));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn nepal() -> Vec<FestivalRule> {
        let mut rules: Vec<FestivalRule> = FestivalRule::printed_in_nepal()
            .into_iter()
            .map(|rule| {
                if rule.at.in_daylight() {
                    rule.udaya()
                } else {
                    rule
                }
            })
            .collect();
        rules.push(FestivalRule::purnima_vrata());
        rules
    }

    /// Nepal's monthly full-moon fast, पूर्णिमाव्रत (`festival-rules.md`
    /// §9.6, C200): the day whose sunset the full moon holds, the later
    /// day when both do, and the later when neither does. Every month,
    /// the adhika month too.
    ///
    /// No text in hand states it. *Dharmasindhu* p. 20 gives the full moon
    /// the later day; the committee prints the fast on the earlier one
    /// whenever the full moon holds only that evening, and this rule holds
    /// all 24 of its printed rows for VS 2082 and 2083.
    fn purnima_vrata() -> FestivalRule {
        FestivalRule {
            key: "PURNIMA_VRATA".to_owned(),
            source: "the Nepal Panchanga Nirnayak Samiti's national panchanga, VS 2082 and 2083 (24 printed rows): the day whose sunset the full moon holds, the later when both or neither do; Dharmasindhu p. 20 gives the later day (C200)".to_owned(),
            month: None,
            convention: Convention::Amanta,
            occurs: Tithi::Purnima.into(),
            in_adhika: InAdhika::Also,
            at: Window::Sunset,
            decide: vec![Guard::new([case(Case::EarlierOnly)], Choice::Earlier)],
            otherwise: Choice::Later,
            parana: Vec::new(),
            meta: FestivalMeta::default(),
        }
    }

    /// The same rule read as the day whose sunrise holds its tithi, or its
    /// nakshatra for a rule kept on one: the later day when only its
    /// sunrise does, else the earlier (both sunrises, or the day an
    /// occurrence holding none falls in). Its source says so, and names
    /// C197.
    ///
    /// ```
    /// # use teistro_calendar::CalendarDate;
    /// # use teistro_calendar::lunisolar::MonthKind;
    /// # use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::LunarMonth;
    /// # use teistro_panchanga::festival::{
    /// #     Case, Choice, FestivalDay, FestivalRule, Guard, InAdhika, MoonArcs, Moonrise, Predicate,
    /// #     SunEntry, Window, observances,
    /// # };
    /// # use teistro_panchanga::month;
    /// # use teistro_panchanga::span::Span;
    /// #
    /// # // Six synthetic days: day k rises at Julian day k and sets at k + 0.5.
    /// # // The krishna ashtami runs from 1.3 to 2.4, so it holds day 2's sunrise
    /// # // and day 1's afternoon and evening.
    /// # let ends = [
    /// #     (Tithi::KrishnaSaptami, 1.3),
    /// #     (Tithi::KrishnaAshtami, 2.4),
    /// #     (Tithi::KrishnaNavami, 3.5),
    /// #     (Tithi::KrishnaDashami, 4.6),
    /// #     (Tithi::KrishnaEkadashi, 5.7),
    /// #     (Tithi::KrishnaDwadashi, 6.8),
    /// # ];
    /// # let days: Vec<FestivalDay> = (0..6u8)
    /// #     .map(|k| {
    /// #         let rise = f64::from(k);
    /// #         let window = Interval::literal(rise, rise + 1.0);
    /// #         let mut from = 0.0;
    /// #         let mut tithi = Vec::new();
    /// #         for (member, to) in ends {
    /// #             tithi.extend(Span::new(member, Interval::literal(from, to), window));
    /// #             from = to;
    /// #         }
    /// #         let at_sunrise = tithi.first().map_or(Tithi::KrishnaSaptami, |span| span.member);
    /// #         FestivalDay {
    /// #             date: CalendarDate::defined(Calendar::Gregorian, 2024, 1, k + 1),
    /// #             vara: Vara::ALL[(usize::from(k) + 1) % 7],
    /// #             sunrise: JulianDay::literal(rise),
    /// #             sunset: JulianDay::literal(rise + 0.5),
    /// #             next_sunrise: JulianDay::literal(rise + 1.0),
    /// #             normal: true,
    /// #             month: month::of(Masa::Shravana, at_sunrise, LunarMonth::Amanta, MonthKind::Nija),
    /// #             tithi,
    /// #             nakshatra: Vec::new(),
    /// #             moonrise: Moonrise::Unsearched,
    /// #             sun_entry: SunEntry::Unsearched,
    /// #             moon_arcs: MoonArcs::Unsearched,
    /// #         }
    /// #     })
    /// #     .collect();
    /// # // The ashtami of Shravana's dark half, kept on the day whose sunrise holds it.
    /// # let rule = |at: Window| FestivalRule {
    /// #     key: String::from("ACME_ASHTAMI"),
    /// #     source: String::from("an example"),
    /// #     month: Some(Masa::Shravana),
    /// #     convention: LunarMonth::Amanta,
    /// #     occurs: Tithi::KrishnaAshtami.into(),
    /// #     in_adhika: InAdhika::Nija,
    /// #     at,
    /// #     decide: vec![Guard::new([Predicate::Case { case: Case::LaterOnly }], Choice::Later)],
    /// #     otherwise: Choice::Earlier,
    /// #     parana: Vec::new(),
    /// #     meta: Default::default(),
    /// # };
    /// # let first = |at: Window| -> Result<_, Box<dyn std::error::Error>> {
    /// #     Ok(observances(&[rule(at)], &days)?.observances.into_iter().next().ok_or("an observance")?)
    /// # };
    ///
    /// // At the evening the 1st keeps it; read by its sunrise, the 2nd.
    /// let evening = rule(Window::Pradosha);
    /// let by_sunrise = evening.clone().udaya();
    /// assert_eq!(by_sunrise.at, Window::Sunrise);
    /// assert!(by_sunrise.source.contains("C197"));
    /// let found = observances(&[by_sunrise], &days)?;
    /// assert_eq!(found.observances[0].day.day, 3);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn udaya(self) -> FestivalRule {
        FestivalRule {
            source: format!(
                "{}; read as the day whose sunrise holds the tithi or the nakshatra, as Nepal's national panchanga keeps a rite of the daylight (C197)",
                self.source
            ),
            at: Window::Sunrise,
            decide: vec![Guard::new([case(Case::LaterOnly)], Choice::Later)],
            otherwise: Choice::Earlier,
            ..self
        }
    }

    /// Rama Navami, Janmashtami, Vijaya Dashami and Lakshmi puja (§1).
    fn first_four() -> Vec<FestivalRule> {
        use Choice::{Earlier, Later};
        use Which::{Earlier as E, Later as L};
        let shravana = |day| joined(day, Nakshatra::Shravana, None);
        vec![
            FestivalRule {
                key: "RAMA_NAVAMI".to_owned(),
                source: "Dharmasindhu p. 33: madhyahna; the earlier day only if it alone holds it, since the 9th pierced by the 8th is forbidden; parana at the 9th's end (p. 34), which cannot run past p. 7's three yamas of the next day".to_owned(),
                month: Some(Masa::Chaitra),
                convention: Convention::Amanta,
                occurs: Tithi::ShuklaNavami.into(),
                in_adhika: InAdhika::Nija,
                at: MADHYAHNA,
                decide: vec![Guard::new([case(Case::EarlierOnly)], Earlier)],
                otherwise: Later,
                parana: vec![VrataParana::TithiEnd {
                    nakshatra: None,
                    yamas: None,
                }],
                meta: FestivalMeta::default(),
            },
            FestivalRule {
                key: "JANMASHTAMI".to_owned(),
                source: "Dharmasindhu pp. 49-50: nishitha; Rohini joined there outranks the tithi alone; both or neither, the later; parana at the end of the 8th and of Rohini joined to it, by day when one ends by day, else at the first end, or for one who cannot wait at sunrise (p. 52)".to_owned(),
                month: Some(Masa::Shravana),
                convention: Convention::Amanta,
                occurs: Tithi::KrishnaAshtami.into(),
                in_adhika: InAdhika::Nija,
                at: Window::Nishitha,
                decide: vec![
                    Guard::new([joined(L, Nakshatra::Rohini, Some(Window::Nishitha))], Later),
                    Guard::new([joined(E, Nakshatra::Rohini, Some(Window::Nishitha))], Earlier),
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                ],
                otherwise: Later,
                parana: vec![
                    VrataParana::TithiEnd {
                        nakshatra: Some(Nakshatra::Rohini),
                        yamas: None,
                    },
                    VrataParana::Sunrise,
                ],
                meta: FestivalMeta::default(),
            },
            FestivalRule {
                key: "VIJAYA_DASHAMI".to_owned(),
                source: "Dharmasindhu p. 71: aparahna; both days or neither, the earlier, unless Shravana joins one day only; the earlier day alone holding it yields to a later day the dashami holds three muhurtas and Shravana joins alone, standing in its aparahna (the Nirnaya-sindhu's condition, endorsed); the later alone yields to Shravana joined only on the earlier (the author's own view)".to_owned(),
                month: Some(Masa::Ashwina),
                convention: Convention::Amanta,
                occurs: Tithi::ShuklaDashami.into(),
                in_adhika: InAdhika::Nija,
                at: APARAHNA,
                decide: vec![
                    Guard::new([case(Case::LaterOnly), shravana(L)], Later),
                    Guard::new([case(Case::LaterOnly), shravana(E)], Earlier),
                    Guard::new([case(Case::LaterOnly)], Later),
                    Guard::new([shravana(E), shravana(L)], Earlier),
                    Guard::new(
                        [
                            case(Case::EarlierOnly),
                            Predicate::Lasts {
                                day: L,
                                from: Edge::Sunrise,
                                ghatis: 6,
                            },
                            shravana(L),
                            Predicate::Stands {
                                day: L,
                                nakshatra: Nakshatra::Shravana,
                                at: APARAHNA,
                            },
                        ],
                        Later,
                    ),
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                    Guard::new([shravana(L)], Later),
                ],
                otherwise: Earlier,
                parana: Vec::new(),
                meta: FestivalMeta::default(),
            },
            lakshmi_puja(),
        ]
    }

    /// Haritalika, Navaratra arambha, Yama dwitiya and Shivaratri (§9).
    fn nepal_four() -> Vec<FestivalRule> {
        use Choice::{Earlier, Later};
        use Which::{Earlier as E, Later as L};
        vec![
            FestivalRule {
                key: "HARITALIKA".to_owned(),
                source: "Dharmasindhu p. 55: the 3rd at sunrise; the later day whenever its sunrise holds it, though for less than a muhurta and the earlier day for all sixty ghatis, for the 4th joined to it; the earlier, joined to the 2nd, only when the 3rd is kshaya and no sunrise holds it".to_owned(),
                month: Some(Masa::Bhadrapada),
                convention: Convention::Amanta,
                occurs: Tithi::ShuklaTritiya.into(),
                in_adhika: InAdhika::Nija,
                at: Window::Sunrise,
                decide: vec![Guard::new([case(Case::Neither)], Earlier)],
                otherwise: Later,
                parana: Vec::new(),
                meta: FestivalMeta::default(),
            },
            FestivalRule {
                key: "NAVARATRA_ARAMBHA".to_owned(),
                source: "Dharmasindhu p. 65: the 1st at sunrise and after it, three muhurtas ideally, two failing that, one by some; never on the day the new moon joins it unless the later day holds it less than a muhurta or not at its sunrise; the earlier when it holds the whole earlier day and grows into the later (C196)".to_owned(),
                month: Some(Masa::Ashwina),
                convention: Convention::Amanta,
                occurs: Tithi::ShuklaPratipada.into(),
                in_adhika: InAdhika::Nija,
                at: Window::Sunrise,
                decide: vec![
                    Guard::new([case(Case::Both)], Earlier),
                    Guard::new(
                        [Predicate::Lasts {
                            day: L,
                            from: Edge::Sunrise,
                            ghatis: 2,
                        }],
                        Later,
                    ),
                ],
                otherwise: Earlier,
                parana: Vec::new(),
                meta: FestivalMeta::default(),
            },
            FestivalRule {
                key: "YAMA_DWITIYA".to_owned(),
                source: "Dharmasindhu p. 79: aparahna; the earlier day only when it alone holds it, the later in every other case".to_owned(),
                month: Some(Masa::Kartika),
                convention: Convention::Amanta,
                occurs: Tithi::ShuklaDvitiya.into(),
                in_adhika: InAdhika::Nija,
                at: APARAHNA,
                decide: vec![Guard::new([case(Case::EarlierOnly)], Earlier)],
                otherwise: Later,
                parana: Vec::new(),
                meta: FestivalMeta::default(),
            },
            FestivalRule {
                key: "SHIVARATRI".to_owned(),
                source: "Dharmasindhu p. 90: nishitha, the night's eighth muhurta; the earlier day when it alone holds it, or holds it whole where the later holds part; the later when the later alone holds it, when neither does, and when both do, with Madhava, the Nirnayasindhu and the Purusharthachintamani, where the Kaustubha takes the earlier (C195); parana inside the 14th within three yamas, the Nirnayasindhu's, or at its end where too little of it is left (pp. 90-91)".to_owned(),
                month: Some(Masa::Magha),
                convention: Convention::Amanta,
                occurs: Tithi::KrishnaChaturdashi.into(),
                in_adhika: InAdhika::Nija,
                at: Window::NightMuhurta { muhurta: 8 },
                decide: vec![
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                    Guard::new(
                        [case(Case::UnequalParts), Predicate::Wholly { day: E }],
                        Earlier,
                    ),
                ],
                otherwise: Later,
                parana: vec![
                    VrataParana::Within { yamas: 3 },
                    VrataParana::TithiEnd {
                        nakshatra: None,
                        yamas: None,
                    },
                ],
                meta: FestivalMeta::default(),
            },
        ]
    }

    /// Rakshabandhan, Bali pratipada and Holika (§9.4): bhadra settles
    /// the hour of the first and the last, not their day.
    fn nepal_three() -> Vec<FestivalRule> {
        use Choice::{Earlier, Later};
        use Which::Later as L;
        let lasts = |ghatis| Predicate::Lasts {
            day: L,
            from: Edge::Sunrise,
            ghatis,
        };
        vec![
            FestivalRule {
                key: "RAKSHABANDHAN".to_owned(),
                source: "Dharmasindhu p. 49: on the full moon holding the sunrise more than three muhurtas, at aparahna or a pradosha free of bhadra; less than three there, the earlier day at a pradosha free of bhadra".to_owned(),
                month: Some(Masa::Shravana),
                convention: Convention::Amanta,
                occurs: Tithi::Purnima.into(),
                in_adhika: InAdhika::Nija,
                at: Window::Sunrise,
                decide: vec![Guard::new([lasts(6)], Later)],
                otherwise: Earlier,
                parana: Vec::new(),
                meta: FestivalMeta::default(),
            },
            FestivalRule {
                key: "BALI_PRATIPADA".to_owned(),
                source: "Dharmasindhu p. 78: Bali puja, the cows' play, Govardhan puja and Margapali on the later day when its 1st holds nine muhurtas past sunrise, since the Moon is not seen then; less, on the 1st the new moon pierces (C198)".to_owned(),
                month: Some(Masa::Kartika),
                convention: Convention::Amanta,
                occurs: Tithi::ShuklaPratipada.into(),
                in_adhika: InAdhika::Nija,
                at: Window::Sunrise,
                decide: vec![Guard::new([lasts(18)], Later)],
                otherwise: Earlier,
                parana: Vec::new(),
                meta: FestivalMeta::default(),
            },
            FestivalRule {
                key: "HOLIKA".to_owned(),
                source: "Dharmasindhu p. 94: pradosha, free of bhadra; the later day when both days hold it or the later holds part of it, the earlier's bhadra standing against it; the earlier when the later's pradosha has none of it; the later day's pratipada when the full moon lasts three and a half yamas there and the pratipada grows is not encoded (C199)".to_owned(),
                month: Some(Masa::Phalguna),
                convention: Convention::Amanta,
                occurs: Tithi::Purnima.into(),
                in_adhika: InAdhika::Nija,
                at: Window::Pradosha,
                decide: vec![
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                    Guard::new([case(Case::Neither)], Earlier),
                ],
                otherwise: Later,
                parana: Vec::new(),
                meta: FestivalMeta::default(),
            },
        ]
    }

    /// The Yajurvedis' upakarma on Shravana's full moon (§9.5), for the
    /// Madhyandina, whose rule is held to the Janai purnima Nepal's
    /// panchanga prints, and for the Taittiriya: both take the earlier day
    /// when the full moon holds both sunrises, and part only on how long
    /// it must hold the later.
    fn upakarma() -> Vec<FestivalRule> {
        use Choice::{Earlier, Later};
        let rule = |key: &str, source: &str, ghatis| FestivalRule {
            key: key.to_owned(),
            source: source.to_owned(),
            month: Some(Masa::Shravana),
            convention: Convention::Amanta,
            occurs: Tithi::Purnima.into(),
            in_adhika: InAdhika::Nija,
            at: Window::Sunrise,
            decide: vec![
                Guard::new([case(Case::Both)], Earlier),
                Guard::new(
                    [Predicate::Lasts {
                        day: Which::Later,
                        from: Edge::Sunrise,
                        ghatis,
                    }],
                    Later,
                ),
            ],
            otherwise: Earlier,
            parana: Vec::new(),
            meta: FestivalMeta::default(),
        };
        vec![
            rule(
                "UPAKARMA_MADHYANDINA",
                "Dharmasindhu p. 47: for the Kanva, the Madhyandina and the other Katyayanas, Shravana's full moon, joined to Shravana or alone; split over two days, the later when it holds more than six muhurtas past the later sunrise, the earlier when less; and for every Yajurvedi the earlier when it holds both sunrises",
                12,
            ),
            rule(
                "UPAKARMA_TAITTIRIYA",
                "Dharmasindhu p. 47: for the Taittiriya, Shravana's full moon; the later day when it holds two muhurtas or more past the later sunrise, though fewer than six, where the other Yajurvedis take the earlier; and for every Yajurvedi the earlier when it holds both sunrises, or fewer than two muhurtas of the later",
                4,
            ),
            FestivalRule::upakarma_samavedi(),
        ]
    }

    /// The Samavedis' upakarma (§9.7): Hasta in Bhadrapada's bright half,
    /// at aparahna. *Dharmasindhu* p. 47 gives the earlier day only when
    /// Hasta pervades that day's aparahna wholly and the later's not at
    /// all; both pervaded, the later touched in part, the earlier only
    /// touched, or neither, the later.
    fn upakarma_samavedi() -> FestivalRule {
        FestivalRule {
            key: "UPAKARMA_SAMAVEDI".to_owned(),
            source: "Dharmasindhu p. 47: for the Samavedis, Hasta in Bhadrapada's bright half; split over two days, the earlier when it pervades the earlier aparahna wholly and the later's not at all, else the later, whether both are pervaded, the later is touched in part, the earlier only touched or neither; Shravana's Hasta where a sankranti or the like bars Bhadrapada's is not encoded (C207)".to_owned(),
            month: Some(Masa::Bhadrapada),
            convention: Convention::Amanta,
            occurs: Occurs::Nakshatra {
                nakshatra: Nakshatra::Hasta,
                paksha: Paksha::Shukla,
            },
            in_adhika: InAdhika::Nija,
            at: APARAHNA,
            decide: vec![Guard::new(
                [
                    case(Case::EarlierOnly),
                    Predicate::Wholly { day: Which::Earlier },
                ],
                Choice::Earlier,
            )],
            otherwise: Choice::Later,
            parana: Vec::new(),
            meta: FestivalMeta::default(),
        }
    }
}

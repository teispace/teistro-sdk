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
use teistro_core::catalogue::{Masa, Nakshatra, Tithi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::LunarMonth as Convention;
use teistro_time::local_day::DayState;

use crate::Panchanga;
use crate::month::{self, LunarMonth};
use crate::span::Span;

/// A fifth of the daylight (*Dharmasindhu*, p. 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "window", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Window {
    /// The instant of sunrise: the tithi a day is named for.
    Sunrise,
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
}

/// One of the two days a tithi is judged between.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Which {
    /// The earlier.
    Earlier,
    /// The later.
    Later,
}

/// How the tithi held the rite's time on the two days (pp. 6–7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// window when one is named, anywhere in the sunrise day otherwise.
    Joined {
        /// Which day.
        day: Which,
        /// The nakshatra.
        nakshatra: Nakshatra,
        /// Where they must meet; the whole day when absent.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at: Option<Window>,
    },
    /// The tithi stands at least this many muhurtas, fifteenths of the
    /// daylight, after the day's sunrise.
    Lasts {
        /// Which day.
        day: Which,
        /// How many muhurtas, 1 to 15.
        muhurtas: u8,
    },
}

/// What a guard, or a rule's `otherwise`, decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Choice {
    /// The earlier day.
    Earlier,
    /// The later day.
    Later,
    /// By the yugma verse (p. 6): the first of a pair on the day it runs
    /// into its partner, the second on the day its partner runs into it.
    ByYugma,
}

/// A choice when every predicate holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Guard {
    /// What must all hold.
    pub when: Vec<Predicate>,
    /// What it decides.
    pub choose: Choice,
}

impl Guard {
    /// A guard.
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
#[serde(rename_all = "camelCase")]
pub struct FestivalRule {
    /// The rule's key in its pack, `"JANMASHTAMI"`.
    pub key: String,
    /// Where the rule is stated.
    pub source: String,
    /// The month, under `convention`.
    pub month: Masa,
    /// The convention the month is named in; amanta by default.
    #[serde(default = "amanta")]
    pub convention: Convention,
    /// The tithi.
    pub tithi: Tithi,
    /// Whether an adhika month holds it too (C171); only the nija month
    /// by default.
    #[serde(default)]
    pub in_adhika: bool,
    /// The time of the rite.
    pub at: Window,
    /// The guards, in order; the first that holds decides.
    pub decide: Vec<Guard>,
    /// What decides when no guard holds.
    pub otherwise: Choice,
}

const fn amanta() -> Convention {
    Convention::Amanta
}

/// The yugma verse's reading of a tithi (p. 6): 2 with 3, 4 with 5,
/// 6 with 7, 8 with 9, 11 with 12, the bright 14th with the full moon,
/// the new moon with the bright 1st. `None` for a tithi in no pair.
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
    /// Refuses a rule that could not be judged: an empty key, a muhurta
    /// count outside a day's fifteen, or the yugma verse asked of a tithi
    /// it pairs with nothing.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, naming the field that is wrong.
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg("a festival rule needs a key").with_field("key"));
        }
        let choices = self
            .decide
            .iter()
            .map(|guard| guard.choose)
            .chain(std::iter::once(self.otherwise));
        if choices.into_iter().any(|choice| choice == Choice::ByYugma)
            && yugma(self.tithi).is_none()
        {
            return Err(Error::invalid_arg(format!(
                "{}: the yugma verse pairs {:?} with nothing (it pairs 2-3, 4-5, 6-7, 8-9, 11-12, the bright 14th with the full moon, and the new moon with the bright 1st); choose EARLIER or LATER",
                self.key, self.tithi
            ))
            .with_field("decide"));
        }
        for guard in &self.decide {
            for predicate in &guard.when {
                if let Predicate::Lasts { muhurtas, .. } = predicate {
                    if !(1..=15).contains(muhurtas) {
                        return Err(Error::invalid_arg(format!(
                            "{}: a day has fifteen muhurtas, not {muhurtas}",
                            self.key
                        ))
                        .with_field("decide.when.muhurtas"));
                    }
                }
            }
        }
        Ok(())
    }
}

/// What the evaluator reads of one sunrise day.
#[derive(Clone, Debug, PartialEq)]
pub struct FestivalDay {
    /// The civil date.
    pub date: CalendarDate,
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
}

impl From<&Panchanga> for FestivalDay {
    fn from(day: &Panchanga) -> FestivalDay {
        FestivalDay {
            date: day.day.date.clone(),
            sunrise: day.day.sunrise,
            sunset: day.day.sunset,
            next_sunrise: day.day.next_sunrise,
            normal: matches!(day.day.state, DayState::Normal),
            month: day.month,
            tithi: day.limbs.tithi.clone(),
            nakshatra: day.limbs.nakshatra.clone(),
        }
    }
}

impl FestivalDay {
    fn daylight(&self) -> Interval {
        Interval::literal(self.sunrise.get(), self.sunset.get())
    }

    fn night(&self) -> Interval {
        Interval::literal(self.sunset.get(), self.next_sunrise.get())
    }

    fn sunrise_day(&self) -> Interval {
        Interval::literal(self.sunrise.get(), self.next_sunrise.get())
    }

    /// The window of a rite on this day: an instant is an interval of no
    /// length.
    fn window(&self, window: Window) -> Result<Interval, Error> {
        match window {
            Window::Sunrise => Ok(Interval::literal(self.sunrise.get(), self.sunrise.get())),
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
        }
    }
}

/// How much of one day's window the tithi held.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "by", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Decided {
    /// The guard at this place in the rule's list.
    Guard {
        /// Its index.
        index: usize,
    },
    /// No guard held.
    Otherwise,
}

/// An observance: the day a rule falls on, and why.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Observance {
    /// The rule's key.
    pub rule: String,
    /// The day.
    pub day: CalendarDate,
    /// The tithi's occurrence judged.
    pub tithi: Interval,
    /// How the tithi held the rite's time on the two days.
    pub case: Case,
    /// The earlier day's extent and the later's.
    pub extents: [Extent; 2],
    /// What decided.
    pub decided_by: Decided,
    /// What that decided.
    pub choice: Choice,
}

/// An occurrence a rule could not judge, and why.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Unjudged {
    /// The rule's key.
    pub rule: String,
    /// The tithi's occurrence.
    pub tithi: Interval,
    /// Why.
    pub why: String,
}

/// What a set of rules gives over a run of days.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Observances {
    /// Each rule's days, in the order of the tithis.
    pub observances: Vec<Observance>,
    /// The occurrences no day could be given to.
    pub unjudged: Vec<Unjudged>,
}

/// The days each rule falls on over consecutive sunrise days.
///
/// An occurrence is judged when its tithi begins on one of the days and
/// the days hold both it and the one after (C172); the caller widens the
/// run by a day on each side for a range's edges.
///
/// # Errors
///
/// A rule's own refusal ([`FestivalRule::check`]), naming it.
pub fn observances(rules: &[FestivalRule], days: &[FestivalDay]) -> Result<Observances, Error> {
    for rule in rules {
        rule.check()?;
    }
    let mut answer = Observances::default();
    for occurrence in occurrences(days) {
        for rule in rules.iter().filter(|rule| rule.tithi == occurrence.member) {
            if !in_month(rule, &occurrence, days) {
                continue;
            }
            match judge(rule, occurrence.whole, days)? {
                Ok(observance) => answer.observances.push(observance),
                Err(why) => answer.unjudged.push(Unjudged {
                    rule: rule.key.clone(),
                    tithi: occurrence.whole,
                    why,
                }),
            }
        }
    }
    Ok(answer)
}

/// Every tithi the days hold, once each, in order.
fn occurrences(days: &[FestivalDay]) -> Vec<Span<Tithi>> {
    let mut seen: Vec<Span<Tithi>> = Vec::new();
    for span in days.iter().flat_map(|day| day.tithi.iter()) {
        if seen
            .last()
            .is_none_or(|last| last.whole.from.get() < span.whole.from.get())
        {
            seen.push(*span);
        }
    }
    seen
}

/// Whether the occurrence falls in the rule's month.
///
/// The month is read at a sunrise inside it: the first at or after the
/// tithi begins, except for the new moon, which ends its month, and so
/// the last sunrise before it ends.
fn in_month(rule: &FestivalRule, occurrence: &Span<Tithi>, days: &[FestivalDay]) -> bool {
    let day = if occurrence.member == Tithi::Amavasya {
        days.iter()
            .rev()
            .find(|day| day.sunrise.get() < occurrence.whole.to.get())
    } else {
        days.iter()
            .find(|day| day.sunrise.get() >= occurrence.whole.from.get())
    };
    day.is_some_and(|day| {
        let month = month::of(
            day.month.amanta,
            occurrence.member,
            rule.convention,
            day.month.kind,
        );
        month.under(rule.convention) == rule.month
            && (rule.in_adhika || month.kind != MonthKind::Adhika)
    })
}

/// The two days an occurrence is judged between (C172): the sunrise day
/// it begins in and the next, or, when it holds two sunrises, the two it
/// holds them on.
fn pair(tithi: Interval, days: &[FestivalDay]) -> Option<(&FestivalDay, &FestivalDay)> {
    let begins = days
        .iter()
        .position(|day| day.sunrise_day().contains(tithi.from))?;
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

type Judged = Result<Observance, String>;

fn judge(rule: &FestivalRule, tithi: Interval, days: &[FestivalDay]) -> Result<Judged, Error> {
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
    let extents = [
        extent(earlier, rule.at, tithi)?,
        extent(later, rule.at, tithi)?,
    ];
    let case = case_of(extents[0].held, extents[1].held);
    let facts = Facts {
        tithi,
        case,
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
        Choice::ByYugma => yugma(rule.tithi).unwrap_or(Which::Earlier),
    };
    let day = match which {
        Which::Earlier => earlier.date.clone(),
        Which::Later => later.date.clone(),
    };
    Ok(Ok(Observance {
        rule: rule.key.clone(),
        day,
        tithi,
        case,
        extents,
        decided_by,
        choice,
    }))
}

struct Facts<'a> {
    tithi: Interval,
    case: Case,
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
            Predicate::Joined { day, nakshatra, at } => {
                let day = self.day(day);
                let span = match at {
                    Some(window) => day.window(window)?,
                    None => day.sunrise_day(),
                };
                day.nakshatra
                    .iter()
                    .filter(|held| held.member == nakshatra)
                    .any(|held| meet(self.tithi, held.whole, span))
            }
            Predicate::Lasts { day, muhurtas } => {
                let day = self.day(day);
                let until = day.daylight().at_fraction(f64::from(muhurtas) / 15.0)?;
                self.tithi.contains(day.sunrise) && self.tithi.to.get() >= until.get()
            }
        })
    }
}

/// Whether the tithi and a nakshatra stand together in a window: at its
/// instant, as `contains` reads it, or for some time inside it.
fn meet(tithi: Interval, nakshatra: Interval, window: Interval) -> bool {
    if window.days() <= 0.0 {
        return tithi.contains(window.from) && nakshatra.contains(window.from);
    }
    let from = tithi
        .from
        .get()
        .max(nakshatra.from.get())
        .max(window.from.get());
    let to = tithi.to.get().min(nakshatra.to.get()).min(window.to.get());
    from < to
}

impl FestivalRule {
    /// The rules *Dharmasindhu* states with a table this evaluator reads,
    /// each citing its page in the 1888 Nirnaya-sagara edition.
    #[must_use]
    pub fn dharmasindhu() -> Vec<FestivalRule> {
        use Choice::{Earlier, Later};
        use Which::{Earlier as E, Later as L};
        let case = |case| Predicate::Case { case };
        let joined = |day, nakshatra, at| Predicate::Joined { day, nakshatra, at };
        let madhyahna = Window::Part {
            part: DayPart::Madhyahna,
        };
        let aparahna = Window::Part {
            part: DayPart::Aparahna,
        };
        vec![
            FestivalRule {
                key: "RAMA_NAVAMI".to_owned(),
                source: "Dharmasindhu p. 33: madhyahna; the earlier day only if it alone holds it, since the 9th pierced by the 8th is forbidden".to_owned(),
                month: Masa::Chaitra,
                convention: Convention::Amanta,
                tithi: Tithi::ShuklaNavami,
                in_adhika: false,
                at: madhyahna,
                decide: vec![Guard::new([case(Case::EarlierOnly)], Earlier)],
                otherwise: Later,
            },
            FestivalRule {
                key: "JANMASHTAMI".to_owned(),
                source: "Dharmasindhu pp. 49-50: nishitha; Rohini joined there outranks the tithi alone; both or neither, the later".to_owned(),
                month: Masa::Shravana,
                convention: Convention::Amanta,
                tithi: Tithi::KrishnaAshtami,
                in_adhika: false,
                at: Window::Nishitha,
                decide: vec![
                    Guard::new([joined(L, Nakshatra::Rohini, Some(Window::Nishitha))], Later),
                    Guard::new([joined(E, Nakshatra::Rohini, Some(Window::Nishitha))], Earlier),
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                ],
                otherwise: Later,
            },
            FestivalRule {
                key: "VIJAYA_DASHAMI".to_owned(),
                source: "Dharmasindhu p. 72: aparahna; both days, the earlier; Shravana on one day only, that day".to_owned(),
                month: Masa::Ashwina,
                convention: Convention::Amanta,
                tithi: Tithi::ShuklaDashami,
                in_adhika: false,
                at: aparahna,
                decide: vec![
                    Guard::new([case(Case::LaterOnly)], Later),
                    Guard::new(
                        [
                            case(Case::EarlierOnly),
                            Predicate::Lasts {
                                day: L,
                                muhurtas: 3,
                            },
                            joined(L, Nakshatra::Shravana, None),
                        ],
                        Later,
                    ),
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                    Guard::new(
                        [
                            joined(E, Nakshatra::Shravana, None),
                            joined(L, Nakshatra::Shravana, None),
                        ],
                        Earlier,
                    ),
                    Guard::new([joined(L, Nakshatra::Shravana, None)], Later),
                ],
                otherwise: Earlier,
            },
            FestivalRule {
                key: "LAKSHMI_PUJA".to_owned(),
                source: "Dharmasindhu p. 77: the new moon at pradosha; the later day if it holds it, else the earlier, and so when neither does".to_owned(),
                month: Masa::Ashwina,
                convention: Convention::Amanta,
                tithi: Tithi::Amavasya,
                in_adhika: false,
                at: Window::Pradosha,
                decide: vec![
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                    Guard::new([case(Case::Neither)], Earlier),
                ],
                otherwise: Later,
            },
        ]
    }
}

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
use teistro_core::catalogue::{Kind, Masa, Nakshatra, Tithi, write_in_full};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::LunarMonth as Convention;
use teistro_time::local_day::DayState;

use crate::Panchanga;
use crate::month::{self, LunarMonth};
use crate::span::Span;

mod ekadashi;
mod following;

pub use ekadashi::{
    EkadashiFast, EkadashiKinds, EkadashiRule, EkadashiTable, Excess, Vedha, ekadashis,
};
pub use following::{FollowingRule, following};

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
            | Window::Nishitha => None,
        }
    }

    /// Whether the window lies in the daylight, sunrise to sunset; the
    /// instant of sunset, which begins the evening, the pradosha after it
    /// and the night's windows do not.
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
            Window::Sunrise | Window::Part { .. } => true,
            Window::Sunset | Window::Pradosha | Window::Nishitha | Window::NightMuhurta { .. } => {
                false
            }
        }
    }
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
}

/// The edge of a day a [`Predicate::Lasts`] counts from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Edge {
    /// Sunrise, counting in thirtieths of the daylight.
    Sunrise,
    /// Sunset, counting in thirtieths of the night.
    Sunset,
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
    /// The month, under `convention`; every month when `None`, for a rite
    /// kept on the same tithi of each, as the monthly full-moon fast is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub month: Option<Masa>,
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
    /// Refuses a rule that could not be judged: an empty key, a ghati
    /// count outside a daylight's or a night's thirty, a night muhurta
    /// outside its fifteen, or the yugma verse asked of a tithi it pairs
    /// with nothing.
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
        self.check_window(self.at, "at.muhurta")?;
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
                    Predicate::Joined { at: Some(at), .. } | Predicate::Stands { at, .. } => {
                        self.check_window(at, "decide.when.at.muhurta")?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Refuses a night muhurta outside the night's fifteen.
    fn check_window(&self, window: Window, field: &str) -> Result<(), Error> {
        match window.muhurta() {
            Some(muhurta) if !(1..=15).contains(&muhurta) => Err(Error::invalid_arg(format!(
                "{}: a night has fifteen muhurtas, counted from 1 after sunset, not {muhurta}",
                self.key
            ))
            .with_field(field)),
            _ => Ok(()),
        }
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Each Ekadashi rule's fasts, in the order of the tithis.
    pub ekadashis: Vec<EkadashiFast>,
    /// The occurrences no day could be given to.
    pub unjudged: Vec<Unjudged>,
}

impl Observances {
    /// Where an answer names catalogue members, and of which kind: a list
    /// of [`Observances`], then a dotted path through each item. What a
    /// boundary section writes in full; the crate's tests hold it to
    /// serde, both ways.
    pub const MEMBERS: [(&'static str, &'static str, Kind); 11] = [
        ("observances", "month", Kind::Masa),
        ("observances", "day.calendar", Kind::Calendar),
        ("observances", "day.era.era", Kind::Era),
        ("observances", "extents.day.calendar", Kind::Calendar),
        ("observances", "extents.day.era.era", Kind::Era),
        ("ekadashis", "tithi", Kind::Tithi),
        ("ekadashis", "month", Kind::Masa),
        ("ekadashis", "days.calendar", Kind::Calendar),
        ("ekadashis", "days.era.era", Kind::Era),
        ("ekadashis", "day.calendar", Kind::Calendar),
        ("ekadashis", "day.era.era", Kind::Era),
    ];

    /// Both answers as one, the occurrences no day was given to in the
    /// order of their tithis: how the karmakala rules' answer and the
    /// Ekadashi rules' are handed back together.
    #[must_use]
    pub fn merged(mut self, other: Observances) -> Observances {
        self.observances.extend(other.observances);
        self.ekadashis.extend(other.ekadashis);
        self.unjudged.extend(other.unjudged);
        self.unjudged
            .sort_by(|a, b| a.tithi.from.get().total_cmp(&b.tithi.from.get()));
        self
    }

    /// The answer as JSON, every catalogue member written as its full key
    /// where [`Observances::MEMBERS`] says: what a boundary section carries
    /// and seals.
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
            let Some(month) = month_of(rule, &occurrence, days) else {
                continue;
            };
            match judge(rule, occurrence.whole, month, days)? {
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

/// The month the occurrence falls in, when it is one the rule keeps.
///
/// The month is read at a sunrise inside it: the first at or after the
/// tithi begins, except for the new moon, which ends its month, and so
/// the last sunrise before it ends.
fn month_of(
    rule: &FestivalRule,
    occurrence: &Span<Tithi>,
    days: &[FestivalDay],
) -> Option<LunarMonth> {
    let day = if occurrence.member == Tithi::Amavasya {
        days.iter()
            .rev()
            .find(|day| day.sunrise.get() < occurrence.whole.to.get())
    } else {
        days.iter()
            .find(|day| day.sunrise.get() >= occurrence.whole.from.get())
    }?;
    let month = month::of(
        day.month.amanta,
        occurrence.member,
        rule.convention,
        day.month.kind,
    );
    let named = rule
        .month
        .is_none_or(|wanted| month.under(rule.convention) == wanted);
    (named && (rule.in_adhika || month.kind != MonthKind::Adhika)).then_some(month)
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

fn judge(
    rule: &FestivalRule,
    tithi: Interval,
    month: LunarMonth,
    days: &[FestivalDay],
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
    let extents = [
        extent(earlier, rule.at, tithi)?,
        extent(later, rule.at, tithi)?,
    ];
    let case = case_of(extents[0].held, extents[1].held);
    let facts = Facts {
        tithi,
        case,
        held: [extents[0].held, extents[1].held],
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
        month: month.amanta,
        adhika: month.kind == MonthKind::Adhika,
        case,
        extents,
        decided_by,
        choice,
    }))
}

struct Facts<'a> {
    tithi: Interval,
    case: Case,
    /// The fraction of each day's window the tithi held, earlier first.
    held: [f64; 2],
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
                    .any(|held| meet(&[self.tithi, held.whole], span))
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

impl FestivalRule {
    /// The rules *Dharmasindhu* states with a table this evaluator reads,
    /// each citing its page in the 1888 Nirnaya-sagara edition: the four
    /// of `festival-rules.md` §1, then the seven Nepal's panchanga prints
    /// (§9), then two schools' upakarma (§9.5).
    #[must_use]
    pub fn dharmasindhu() -> Vec<FestivalRule> {
        let mut rules = FestivalRule::first_four();
        rules.extend(FestivalRule::nepal_four());
        rules.extend(FestivalRule::nepal_three());
        rules.extend(FestivalRule::upakarma());
        rules
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
        let mut rules: Vec<FestivalRule> = FestivalRule::dharmasindhu()
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
            tithi: Tithi::Purnima,
            in_adhika: true,
            at: Window::Sunset,
            decide: vec![Guard::new([case(Case::EarlierOnly)], Choice::Earlier)],
            otherwise: Choice::Later,
        }
    }

    /// The same rule read as the day whose sunrise holds its tithi: the
    /// later day when only its sunrise does, else the earlier (both
    /// sunrises, or the day a tithi holding none falls in). Its source
    /// says so, and names C197.
    #[must_use]
    pub fn udaya(self) -> FestivalRule {
        FestivalRule {
            source: format!(
                "{}; read as the day whose sunrise holds the tithi, as Nepal's national panchanga keeps a rite of the daylight (C197)",
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
        let joined = |day, nakshatra, at| Predicate::Joined { day, nakshatra, at };
        let shravana = |day| joined(day, Nakshatra::Shravana, None);
        vec![
            FestivalRule {
                key: "RAMA_NAVAMI".to_owned(),
                source: "Dharmasindhu p. 33: madhyahna; the earlier day only if it alone holds it, since the 9th pierced by the 8th is forbidden".to_owned(),
                month: Some(Masa::Chaitra),
                convention: Convention::Amanta,
                tithi: Tithi::ShuklaNavami,
                in_adhika: false,
                at: MADHYAHNA,
                decide: vec![Guard::new([case(Case::EarlierOnly)], Earlier)],
                otherwise: Later,
            },
            FestivalRule {
                key: "JANMASHTAMI".to_owned(),
                source: "Dharmasindhu pp. 49-50: nishitha; Rohini joined there outranks the tithi alone; both or neither, the later".to_owned(),
                month: Some(Masa::Shravana),
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
                source: "Dharmasindhu p. 71: aparahna; both days or neither, the earlier, unless Shravana joins one day only; the earlier day alone holding it yields to a later day the dashami holds three muhurtas and Shravana joins alone, standing in its aparahna (the Nirnaya-sindhu's condition, endorsed); the later alone yields to Shravana joined only on the earlier (the author's own view)".to_owned(),
                month: Some(Masa::Ashwina),
                convention: Convention::Amanta,
                tithi: Tithi::ShuklaDashami,
                in_adhika: false,
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
            },
            FestivalRule {
                key: "LAKSHMI_PUJA".to_owned(),
                source: "Dharmasindhu p. 77: the new moon at pradosha; the later day when it lasts more than a ghati into that night, which puts the matter beyond doubt, else the earlier, and so when neither day holds it".to_owned(),
                month: Some(Masa::Ashwina),
                convention: Convention::Amanta,
                tithi: Tithi::Amavasya,
                in_adhika: false,
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
            },
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
                tithi: Tithi::ShuklaTritiya,
                in_adhika: false,
                at: Window::Sunrise,
                decide: vec![Guard::new([case(Case::Neither)], Earlier)],
                otherwise: Later,
            },
            FestivalRule {
                key: "NAVARATRA_ARAMBHA".to_owned(),
                source: "Dharmasindhu p. 65: the 1st at sunrise and after it, three muhurtas ideally, two failing that, one by some; never on the day the new moon joins it unless the later day holds it less than a muhurta or not at its sunrise; the earlier when it holds the whole earlier day and grows into the later (C196)".to_owned(),
                month: Some(Masa::Ashwina),
                convention: Convention::Amanta,
                tithi: Tithi::ShuklaPratipada,
                in_adhika: false,
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
            },
            FestivalRule {
                key: "YAMA_DWITIYA".to_owned(),
                source: "Dharmasindhu p. 79: aparahna; the earlier day only when it alone holds it, the later in every other case".to_owned(),
                month: Some(Masa::Kartika),
                convention: Convention::Amanta,
                tithi: Tithi::ShuklaDvitiya,
                in_adhika: false,
                at: APARAHNA,
                decide: vec![Guard::new([case(Case::EarlierOnly)], Earlier)],
                otherwise: Later,
            },
            FestivalRule {
                key: "SHIVARATRI".to_owned(),
                source: "Dharmasindhu p. 90: nishitha, the night's eighth muhurta; the earlier day when it alone holds it, or holds it whole where the later holds part; the later when the later alone holds it, when neither does, and when both do, with Madhava, the Nirnayasindhu and the Purusharthachintamani, where the Kaustubha takes the earlier (C195)".to_owned(),
                month: Some(Masa::Magha),
                convention: Convention::Amanta,
                tithi: Tithi::KrishnaChaturdashi,
                in_adhika: false,
                at: Window::NightMuhurta { muhurta: 8 },
                decide: vec![
                    Guard::new([case(Case::EarlierOnly)], Earlier),
                    Guard::new(
                        [case(Case::UnequalParts), Predicate::Wholly { day: E }],
                        Earlier,
                    ),
                ],
                otherwise: Later,
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
                tithi: Tithi::Purnima,
                in_adhika: false,
                at: Window::Sunrise,
                decide: vec![Guard::new([lasts(6)], Later)],
                otherwise: Earlier,
            },
            FestivalRule {
                key: "BALI_PRATIPADA".to_owned(),
                source: "Dharmasindhu p. 78: Bali puja, the cows' play, Govardhan puja and Margapali on the later day when its 1st holds nine muhurtas past sunrise, since the Moon is not seen then; less, on the 1st the new moon pierces (C198)".to_owned(),
                month: Some(Masa::Kartika),
                convention: Convention::Amanta,
                tithi: Tithi::ShuklaPratipada,
                in_adhika: false,
                at: Window::Sunrise,
                decide: vec![Guard::new([lasts(18)], Later)],
                otherwise: Earlier,
            },
            FestivalRule {
                key: "HOLIKA".to_owned(),
                source: "Dharmasindhu p. 94: pradosha, free of bhadra; the later day when both days hold it or the later holds part of it, the earlier's bhadra standing against it; the earlier when the later's pradosha has none of it; the later day's pratipada when the full moon lasts three and a half yamas there and the pratipada grows is not encoded (C199)".to_owned(),
                month: Some(Masa::Phalguna),
                convention: Convention::Amanta,
                tithi: Tithi::Purnima,
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
            tithi: Tithi::Purnima,
            in_adhika: false,
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
        ]
    }
}

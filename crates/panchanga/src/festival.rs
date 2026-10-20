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
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::LunarMonth as Convention;
use teistro_time::local_day::{DayState, LocalDay};

use crate::Panchanga;
use crate::month::{self, LunarMonth};
use crate::span::Span;

mod dated;
mod ekadashi;
mod following;
mod nepal_panchanga;

pub use dated::{Dated, DatedObservance, DatedRule, dated};
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
    /// Neither day: the rule is not kept this time. A rite kept only on a
    /// weekday answers this when its tithi's day is another (C618).
    None,
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
    /// What the rite is kept on: a tithi, or a nakshatra in a paksha,
    /// written flat as `tithi`, or as `nakshatra` with `paksha`.
    #[serde(flatten)]
    pub occurs: Occurs,
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
    #[must_use]
    pub const fn tithi(self) -> Option<Tithi> {
        match self {
            Occurs::Tithi { tithi } => Some(tithi),
            Occurs::Nakshatra { .. } | Occurs::DayOfPaksha { .. } => None,
        }
    }

    /// Whether the rule is kept on a tithi, one or a day of either
    /// paksha, rather than on a nakshatra.
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
    /// Whether the rule reads a moonrise, at its rite or in a guard: the
    /// days it is judged over must have theirs searched.
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

    /// Refuses a rule that could not be judged: an empty key, a ghati
    /// count outside a daylight's or a night's thirty, a night muhurta
    /// outside its fifteen, the yugma verse asked of a tithi it pairs
    /// with nothing or of a nakshatra, or a tithi joined to a nakshatra
    /// asked of a rule kept on a nakshatra.
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
            && self.occurs.tithi().and_then(yugma).is_none()
        {
            return Err(Error::invalid_arg(format!(
                "{}: the yugma verse pairs {:?} with nothing (it pairs 2-3, 4-5, 6-7, 8-9, 11-12, the bright 14th with the full moon, and the new moon with the bright 1st, and no nakshatra or day of either paksha); choose EARLIER or LATER",
                self.key, self.occurs
            ))
            .with_field("decide"));
        }
        self.check_window(self.at, "at")?;
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
                    Predicate::Joined { at: Some(at), .. } | Predicate::Stands { at, .. } => {
                        self.check_window(at, "decide.when.at")?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Refuses a night muhurta outside the night's fifteen, and a range
    /// of day muhurtas outside the daylight's fifteen or running
    /// backwards; `at` is the window's own field.
    fn check_window(&self, window: Window, at: &str) -> Result<(), Error> {
        if let Some(muhurta) = window.muhurta()
            && !(1..=15).contains(&muhurta)
        {
            return Err(Error::invalid_arg(format!(
                "{}: a night has fifteen muhurtas, counted from 1 after sunset, not {muhurta}",
                self.key
            ))
            .with_field(format!("{at}.muhurta")));
        }
        if let Window::DayMuhurta { from, to } = window {
            for (name, muhurta) in [("from", from), ("to", to)] {
                if !(1..=15).contains(&muhurta) {
                    return Err(Error::invalid_arg(format!(
                        "{}: a daylight has fifteen muhurtas, counted from 1 after sunrise, not {muhurta}",
                        self.key
                    ))
                    .with_field(format!("{at}.{name}")));
                }
            }
            if from > to {
                return Err(Error::invalid_arg(format!(
                    "{}: day muhurtas {from} to {to} run backwards; give the first as `from`",
                    self.key
                ))
                .with_field(format!("{at}.to")));
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
}

/// A day's first moonrise, from its sunrise to the next.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Moonrise {
    /// Not searched: no rule read it. A rule reading it over such a day is
    /// refused rather than read as a day the Moon did not rise.
    Unsearched,
    /// The Moon rose at this instant.
    Rose(JulianDay<Utc>),
    /// The Moon did not rise between the two sunrises.
    NoRise,
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
                Moonrise::Rose(at) => Ok(Interval::literal(at.get(), at.get())),
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
}

/// An occurrence a rule could not judge, and why.
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
}

impl Observances {
    /// Where an answer names catalogue members, and of which kind: a list
    /// of [`Observances`], then a dotted path through each item. What a
    /// boundary section writes in full; the crate's tests hold it to
    /// serde, both ways.
    pub const MEMBERS: [(&'static str, &'static str, Kind); 15] = [
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
        ("dated", "day.calendar", Kind::Calendar),
        ("dated", "day.era.era", Kind::Era),
        ("dated", "on.calendar", Kind::Calendar),
        ("dated", "on.era.era", Kind::Era),
    ];

    /// Both answers as one, the occurrences no day was given to in the
    /// order of their tithis: how the karmakala rules' answer and the
    /// Ekadashi rules' are handed back together.
    #[must_use]
    pub fn merged(mut self, other: Observances) -> Observances {
        self.observances.extend(other.observances);
        self.ekadashis.extend(other.ekadashis);
        self.unjudged.extend(other.unjudged);
        self.dated.extend(other.dated);
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
/// An occurrence is judged when its tithi, or the nakshatra a rule is kept
/// on, begins on one of the days and the days hold both it and the one
/// after (C172); the caller widens the run by a day on each side for a
/// range's edges. The answer is in the order the occurrences begin.
///
/// # Errors
///
/// A rule's own refusal ([`FestivalRule::check`]), naming it.
pub fn observances(rules: &[FestivalRule], days: &[FestivalDay]) -> Result<Observances, Error> {
    for rule in rules {
        rule.check()?;
    }
    let tithis = occurrences(days);
    let nakshatras = distinct(days.iter().flat_map(|day| day.nakshatra.iter()));
    let mut kept: Vec<(&FestivalRule, Interval, LunarMonth)> = Vec::new();
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
                    let Some(tithi) = running_at_middle(occurrence.whole, &tithis) else {
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
    let mut answer = Observances::default();
    for (rule, whole, month) in kept {
        match judge(rule, whole, month, days)? {
            Ok(Some(observance)) => answer.observances.push(observance),
            Ok(None) => {}
            Err(why) => answer.unjudged.push(Unjudged {
                rule: rule.key.clone(),
                tithi: whole,
                why,
            }),
        }
    }
    Ok(answer)
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

/// The observance, none when the rule chose [`Choice::None`], or why the
/// occurrence could not be judged.
type Judged = Result<Option<Observance>, String>;

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
    if rule.reads_moonrise()
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
    let day = match which {
        Which::Earlier => earlier.date.clone(),
        Which::Later => later.date.clone(),
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
            Predicate::Joined { nakshatra, at, .. } => {
                window(at.map_or_else(|| Ok(day.sunrise_day()), |at| day.window(at))?);
                named.push((day, nakshatra));
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

impl FestivalRule {
    /// The rules *Dharmasindhu* states with a table this evaluator reads,
    /// each citing its page in the 1888 Nirnaya-sagara edition: the four
    /// of `festival-rules.md` §1, then the seven Nepal's panchanga prints
    /// (§9), then three schools' upakarma (§9.5, §9.7), then the two
    /// chaturthis kept at moonrise (`festival-rule-language.md` §3.1).
    #[must_use]
    pub fn dharmasindhu() -> Vec<FestivalRule> {
        let mut rules = FestivalRule::printed_in_nepal();
        rules.extend(FestivalRule::at_moonrise());
        rules
    }

    /// The text's rules Nepal's national panchanga keeps: all but the two
    /// chaturthis at moonrise, which it does not print.
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
        };
        vec![
            rule(
                "SANKASHTA_CHATURTHI",
                "Dharmasindhu p. 9: the dark 4th at moonrise (चन्द्रोदयव्यापिनी); the day whose moonrise alone holds it, the earlier, joined to the 3rd, when both do, the later when neither does; every month, the adhika too",
                None,
                true,
            ),
            rule(
                "KARAKA_CHATURTHI",
                "Dharmasindhu p. 76: Ashvina's dark 4th, at moonrise, decided as the Sankashta chaturthi (p. 9)",
                Some(Masa::Ashwina),
                false,
            ),
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
            in_adhika: true,
            at: Window::Sunset,
            decide: vec![Guard::new([case(Case::EarlierOnly)], Choice::Earlier)],
            otherwise: Choice::Later,
        }
    }

    /// The same rule read as the day whose sunrise holds its tithi, or its
    /// nakshatra for a rule kept on one: the later day when only its
    /// sunrise does, else the earlier (both sunrises, or the day an
    /// occurrence holding none falls in). Its source says so, and names
    /// C197.
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
        let joined = |day, nakshatra, at| Predicate::Joined { day, nakshatra, at };
        let shravana = |day| joined(day, Nakshatra::Shravana, None);
        vec![
            FestivalRule {
                key: "RAMA_NAVAMI".to_owned(),
                source: "Dharmasindhu p. 33: madhyahna; the earlier day only if it alone holds it, since the 9th pierced by the 8th is forbidden".to_owned(),
                month: Some(Masa::Chaitra),
                convention: Convention::Amanta,
                occurs: Tithi::ShuklaNavami.into(),
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
                occurs: Tithi::KrishnaAshtami.into(),
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
                occurs: Tithi::ShuklaDashami.into(),
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
                occurs: Tithi::Amavasya.into(),
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
                occurs: Tithi::ShuklaTritiya.into(),
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
                occurs: Tithi::ShuklaPratipada.into(),
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
                occurs: Tithi::ShuklaDvitiya.into(),
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
                occurs: Tithi::KrishnaChaturdashi.into(),
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
                occurs: Tithi::Purnima.into(),
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
                occurs: Tithi::ShuklaPratipada.into(),
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
                occurs: Tithi::Purnima.into(),
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
            occurs: Tithi::Purnima.into(),
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
            in_adhika: false,
            at: APARAHNA,
            decide: vec![Guard::new(
                [
                    case(Case::EarlierOnly),
                    Predicate::Wholly { day: Which::Earlier },
                ],
                Choice::Earlier,
            )],
            otherwise: Choice::Later,
        }
    }
}

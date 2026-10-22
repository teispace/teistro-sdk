//! Anniversaries kept by a lunar date (`03-design/shraddha.md`): a death's
//! yearly shraddha, the barkhi, and a birthday kept by the tithi, the
//! janma tithi.
//!
//! Each is a tithi fixed at a moment, its **origin**, kept every year on
//! the day the tithi holds a rite's time. The day itself is the festival
//! evaluator's: an anniversary becomes the year's [`FestivalRule`], so
//! the two cannot judge a day differently. What this adds is the origin
//! (the tithi, month and half at an instant), the year counted from it,
//! and the adhika and kshaya policies that depend on that count, which no
//! single rule can state.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Masa, Nakshatra, Paksha, Tithi};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::LunarMonth as Convention;

use crate::month;

use super::{
    Case, Choice, DayPart, Edge, FestivalDay, FestivalRule, Guard, InAdhika, Observance,
    Observances, Predicate, Which, Window, by_share, case, observances,
};

/// A sidereal year, days: what an anniversary's number is counted in. A
/// lunar anniversary falls within a month of its solar one, so the count
/// rounds to the right year with five months to spare.
const SIDEREAL_YEAR_DAYS: f64 = 365.256_363;

/// Which half of a tithi: the first or the second of its two karanas, an
/// equal arc of the Moon's gain on the Sun each.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "TithiHalf"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Half {
    /// The first.
    #[default]
    First,
    /// The second.
    Second,
}

/// Where an anniversary is counted from.
///
/// ```
/// use teistro_panchanga::festival::Origin;
///
/// let origin: Origin = serde_json::from_str(r#"{"by": "LUNAR", "month": "BHADRAPADA", "tithi": "KRISHNA_PANCHAMI"}"#)?;
/// assert!(matches!(origin, Origin::Lunar { adhika: false, .. }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "by", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Origin {
    /// The moment of the death or the birth: the tithi running then, as
    /// the Naradiya has it for a death and a parana (NS p. 982, C661). A
    /// death before sunrise takes the tithi of that hour, not the civil
    /// day's sunrise tithi.
    Instant {
        /// The moment.
        at: JulianDay<Utc>,
    },
    /// Only the civil date: its sunrise tithi, marked as assumed, with
    /// every tithi the date held beside it (C661).
    Date {
        /// The date.
        date: CalendarDate,
    },
    /// The lunar date itself, as a family keeps it.
    Lunar {
        /// The month, amanta.
        month: Masa,
        /// The tithi.
        tithi: Tithi,
        /// Whether the month was adhika.
        #[serde(default)]
        adhika: bool,
        /// The half of the tithi, which names the month in a kshaya
        /// lunation (C668); the month given is taken as named already.
        #[serde(default)]
        half: Half,
        /// The moment, if known, which only counts the years: without it
        /// no anniversary has a number, and the first one's adhika policy
        /// (C666) cannot apply.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at: Option<JulianDay<Utc>>,
    },
}

/// An origin as found: the lunar date an anniversary keeps.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct OriginFound {
    /// The tithi.
    pub tithi: Tithi,
    /// The month it names, amanta: in a kshaya lunation, the first name
    /// for the tithi's first half and the second for its second (C668).
    pub month: Masa,
    /// Whether that lunation was ordinary, adhika or kshaya.
    pub kind: MonthKind,
    /// The half of the tithi.
    pub half: Half,
    /// The nakshatra then, which a janma tithi may be tied to (C670); none
    /// for a lunar origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nakshatra: Option<Nakshatra>,
    /// The moment the years are counted from: the instant, a date's
    /// sunrise, or a lunar origin's own when given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<JulianDay<Utc>>,
    /// Whether the tithi is the date's sunrise tithi, assumed for want of
    /// an hour.
    pub assumed_sunrise: bool,
    /// Every tithi the date held, midnight to midnight, in order, when it
    /// held more than one: the candidates a reader with the hour chooses
    /// between. Empty otherwise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<Tithi>,
}

impl OriginFound {
    /// A lunar origin as found: nothing to search.
    #[must_use]
    pub const fn lunar(
        month: Masa,
        tithi: Tithi,
        adhika: bool,
        half: Half,
        at: Option<JulianDay<Utc>>,
    ) -> OriginFound {
        OriginFound {
            tithi,
            month,
            kind: if adhika {
                MonthKind::Adhika
            } else {
                MonthKind::Nija
            },
            half,
            nakshatra: None,
            at,
            assumed_sunrise: false,
            candidates: Vec::new(),
        }
    }

    /// Whether a death on this date is kept as a parvana: on a new moon, or
    /// in the fortnight of the ancestors, Bhadrapada's dark half amanta
    /// (Shankha in NS p. 983, C665, C672).
    #[must_use]
    pub fn is_parvana(&self) -> bool {
        self.tithi == Tithi::Amavasya
            || (self.month == Masa::Bhadrapada && self.tithi.attributes().paksha == Paksha::Krishna)
    }
}

/// What an anniversary is, which sets the rite's time and how the day is
/// chosen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum AnniversaryKind {
    /// A death's yearly shraddha, an ekoddishta or a parvana by its
    /// origin ([`OriginFound::is_parvana`], C665). An answer names which.
    #[default]
    Shraddha,
    /// The ekoddishta, for one person: kutapa to rauhina, the daylight's
    /// 8th and 9th muhurtas (NS p. 987, DS p. 265, C662).
    Ekoddishta,
    /// The parvana, for the ancestors together: the aparahna (DS p. 266).
    Parvana,
    /// A birthday by the tithi: the sunrise tithi (NS p. 531, C670), in
    /// the ordinary month (DS, C671).
    JanmaTithi,
}

impl AnniversaryKind {
    /// The rite's time when the request names none.
    #[must_use]
    pub const fn window(self) -> Window {
        match self {
            AnniversaryKind::Shraddha | AnniversaryKind::Ekoddishta => {
                Window::DayMuhurta { from: 8, to: 9 }
            }
            AnniversaryKind::Parvana => Window::Part {
                part: DayPart::Aparahna,
            },
            AnniversaryKind::JanmaTithi => Window::Sunrise,
        }
    }
}

/// Which month the first anniversary takes when its month is doubled
/// (C666).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum FirstYearAdhika {
    /// The adhika, the thirteenth month: *Dharmasindhu*, the
    /// *Nirnayasindhu* (pp. 25–26) and Kane IV p. 546.
    #[default]
    Adhika,
    /// The ordinary month: Dhakal's note for Nepal's barkhi (NS p. 25
    /// n. 14), which counts the eleventh day's rite as the first abdika.
    Nija,
}

/// *Dharmasindhu*'s two options for a janma tithi, both off by default
/// (C670).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct JanmaOptions {
    /// When both days' sunrises hold the tithi, the day whose sunrise holds
    /// the birth nakshatra.
    #[serde(default)]
    pub nakshatra_tie: bool,
    /// The sunrise tithi only when it lasts two muhurtas past sunrise,
    /// else the day before.
    #[serde(default)]
    pub two_muhurta_floor: bool,
}

impl JanmaOptions {
    #[allow(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde's skip_serializing_if passes a reference"
    )]
    fn is_default(&self) -> bool {
        *self == JanmaOptions::default()
    }
}

/// An anniversary a request asks the days of.
///
/// ```
/// use teistro_panchanga::festival::{Anniversary, AnniversaryKind};
///
/// let barkhi: Anniversary = serde_json::from_str(r#"{
///     "key": "GRANDFATHER",
///     "origin": {"by": "LUNAR", "month": "KARTIKA", "tithi": "SHUKLA_DASHAMI"}
/// }"#)?;
/// assert_eq!(barkhi.kind, AnniversaryKind::Shraddha);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Anniversary {
    /// The consumer's own key, `"GRANDFATHER"`.
    pub key: String,
    /// Where it is counted from.
    pub origin: Origin,
    /// What it is.
    #[serde(default)]
    pub kind: AnniversaryKind,
    /// The rite's time, in place of the kind's ([`AnniversaryKind::window`]):
    /// `{"window": "PART", "part": "MADHYAHNA"}` is DS's madhyahna, the
    /// 7th to 9th muhurtas (C662).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<Window>,
    /// The first anniversary's month when it is doubled.
    #[serde(default)]
    pub first_year_adhika: FirstYearAdhika,
    /// A janma tithi's options.
    #[serde(default, skip_serializing_if = "JanmaOptions::is_default")]
    pub janma: JanmaOptions,
}

/// Which clause chose an anniversary's month.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum MonthClause {
    /// The origin's month, ordinary.
    Nija,
    /// The first anniversary in the adhika month, its month being doubled
    /// (C666).
    FirstYearAdhika,
    /// An origin in an adhika month, kept in the adhika when the month is
    /// adhika again (C667).
    AdhikaOrigin,
    /// The joined lunation of a kshaya year, which answers to both its
    /// names (C669).
    KshayaJoined,
}

/// An anniversary's day.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AnniversaryDay {
    /// The request's key.
    pub key: String,
    /// **Which anniversary**: 1 for the first recurrence after the origin,
    /// the origin's day itself being 0 and never answered; a janma tithi's
    /// is the age turned. None for a lunar origin given no moment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u16>,
    /// What it was kept as: never [`AnniversaryKind::Shraddha`], which
    /// names the ekoddishta or the parvana the origin chose.
    pub kind: AnniversaryKind,
    /// The origin, as found.
    pub origin: OriginFound,
    /// The clause that chose the month.
    pub month_by: MonthClause,
    /// The day, as the festival evaluator judged it: the tithi's two days,
    /// the case, the window on each and what decided.
    pub observance: Observance,
}

/// Which of an anniversary's year rules an observance came from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Year {
    /// Every year but the first under its own policy.
    Later,
    /// The first, kept in the adhika when its month is doubled.
    First,
    /// A kshaya year's joined lunation, under its first name.
    Kshaya,
}

impl Anniversary {
    /// Refuses what cannot be kept: an empty key, a janma tithi with a
    /// window of its own, or a janma option on another kind.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, naming the field.
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg("an anniversary needs a key").with_field("key"));
        }
        let janma = self.kind == AnniversaryKind::JanmaTithi;
        if janma && self.window.is_some() {
            return Err(Error::invalid_arg(format!(
                "{}: a janma tithi is kept by the sunrise tithi, so it takes no window",
                self.key
            ))
            .with_field("window"));
        }
        if !janma && !self.janma.is_default() {
            return Err(Error::invalid_arg(format!(
                "{}: `janma` is a janma tithi's options, and this anniversary is a shraddha",
                self.key
            ))
            .with_field("janma"));
        }
        Ok(())
    }

    /// The kind this origin keeps it as.
    #[must_use]
    pub fn kept_as(&self, origin: &OriginFound) -> AnniversaryKind {
        match self.kind {
            AnniversaryKind::Shraddha if origin.is_parvana() => AnniversaryKind::Parvana,
            AnniversaryKind::Shraddha => AnniversaryKind::Ekoddishta,
            kind => kind,
        }
    }

    /// One year rule: the anniversary as a festival rule in `month`.
    fn rule(&self, origin: &OriginFound, month: Masa, in_adhika: InAdhika) -> FestivalRule {
        let kind = self.kept_as(origin);
        let at = self.window.unwrap_or_else(|| kind.window());
        let (decide, otherwise) = if kind == AnniversaryKind::JanmaTithi {
            self.janma.guards(origin.nakshatra)
        } else {
            by_share()
        };
        FestivalRule {
            key: self.key.clone(),
            source: kind.source().to_owned(),
            month: Some(month),
            convention: Convention::Amanta,
            occurs: origin.tithi.into(),
            in_adhika,
            at,
            decide,
            otherwise,
            parana: Vec::new(),
        }
    }

    /// The year rules, each with the year it answers for; a kshaya year's
    /// only when the days hold a kshaya lunation.
    fn rules(&self, origin: &OriginFound, kshaya: bool) -> Vec<(Year, FestivalRule)> {
        let janma = self.kept_as(origin) == AnniversaryKind::JanmaTithi;
        let later = if origin.kind == MonthKind::Adhika && !janma {
            InAdhika::First
        } else {
            InAdhika::Nija
        };
        let mut rules = vec![(Year::Later, self.rule(origin, origin.month, later))];
        if !janma
            && origin.at.is_some()
            && later == InAdhika::Nija
            && self.first_year_adhika == FirstYearAdhika::Adhika
        {
            rules.push((
                Year::First,
                self.rule(origin, origin.month, InAdhika::First),
            ));
        }
        if kshaya {
            rules.push((
                Year::Kshaya,
                self.rule(origin, month::previous(origin.month), InAdhika::Nija),
            ));
        }
        rules
    }
}

impl AnniversaryKind {
    const fn source(self) -> &'static str {
        match self {
            AnniversaryKind::Shraddha | AnniversaryKind::Ekoddishta => {
                "Dharmasindhu pp. 265-266, Nirnayasindhu p. 987: the ekoddishta at kutapa to rauhina; the day holding it alone, the earlier when both or neither do or on a tie, else the greater"
            }
            AnniversaryKind::Parvana => {
                "Dharmasindhu p. 266: the parvana at aparahna; the day holding it alone, the earlier when both or neither do or on a tie, else the greater"
            }
            AnniversaryKind::JanmaTithi => {
                "Nirnayasindhu p. 531: the sunrise tithi, the earlier when both sunrises hold it; Dharmasindhu's options when asked"
            }
        }
    }
}

impl JanmaOptions {
    /// The sunrise tithi's guards, with *Dharmasindhu*'s options.
    fn guards(self, nakshatra: Option<Nakshatra>) -> (Vec<Guard>, Choice) {
        let mut decide = Vec::new();
        if let (true, Some(nakshatra)) = (self.nakshatra_tie, nakshatra) {
            let stands = |day| Predicate::Stands {
                day,
                nakshatra,
                at: Window::Sunrise,
            };
            decide.push(Guard::new(
                [case(Case::Both), stands(Which::Later)],
                Choice::Later,
            ));
            decide.push(Guard::new(
                [case(Case::Both), stands(Which::Earlier)],
                Choice::Earlier,
            ));
        }
        let later_only = case(Case::LaterOnly);
        decide.push(if self.two_muhurta_floor {
            // Two muhurtas are four ghatis, a thirtieth of the daylight
            // each.
            Guard::new(
                [
                    later_only,
                    Predicate::Lasts {
                        day: Which::Later,
                        from: Edge::Sunrise,
                        ghatis: 4,
                    },
                ],
                Choice::Later,
            )
        } else {
            Guard::new([later_only], Choice::Later)
        });
        (decide, Choice::Earlier)
    }
}

/// Which anniversary an observance is, counted from its origin.
enum Counted {
    /// The origin's own day or before it: no anniversary.
    Not,
    /// The origin has no moment, so the anniversary has no number.
    Unnumbered,
    /// This one, 1 for the first.
    Numbered(u16),
}

impl Counted {
    fn of(origin: &OriginFound, observance: &Observance) -> Counted {
        let Some(at) = origin.at else {
            return Counted::Unnumbered;
        };
        let years = ((observance.tithi.from.get() - at.get()) / SIDEREAL_YEAR_DAYS).round();
        if years < 1.0 || years > f64::from(u16::MAX) {
            return Counted::Not;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "checked to lie in 1 to u16::MAX first"
        )]
        Counted::Numbered(years as u16)
    }
}

/// Every anniversary's days over consecutive sunrise days, each with its
/// origin already found; what the festival evaluator leaves unjudged is
/// added to `unjudged` under the anniversary's key.
///
/// # Errors
///
/// An anniversary [`Anniversary::check`] refuses, or the year rule the
/// evaluator refuses, naming it.
pub fn anniversaries(
    asked: &[(Anniversary, OriginFound)],
    days: &[FestivalDay],
) -> Result<Observances, Error> {
    let mut answer = Observances::default();
    let any_kshaya = days.iter().any(|day| day.month.kind == MonthKind::Kshaya);
    for (anniversary, origin) in asked {
        anniversary.check()?;
        let kind = anniversary.kept_as(origin);
        let rules = anniversary.rules(origin, any_kshaya);
        let has_first = rules.iter().any(|(year, _)| *year == Year::First);
        for (year, rule) in rules {
            let found = observances(core::slice::from_ref(&rule), days)?;
            for observance in found.observances {
                let lunation = days
                    .iter()
                    .find(|day| day.date == observance.day)
                    .map(|day| day.month.kind);
                let kshaya = lunation == Some(MonthKind::Kshaya);
                let count = match Counted::of(origin, &observance) {
                    Counted::Not => continue,
                    Counted::Unnumbered => None,
                    Counted::Numbered(count) => Some(count),
                };
                let first = count == Some(1);
                let month_by = match year {
                    Year::Kshaya if !kshaya => continue,
                    Year::First if !first => continue,
                    Year::Later if first && has_first => continue,
                    Year::Kshaya => MonthClause::KshayaJoined,
                    _ if observance.adhika && year == Year::First => MonthClause::FirstYearAdhika,
                    _ if observance.adhika => MonthClause::AdhikaOrigin,
                    _ if kshaya => MonthClause::KshayaJoined,
                    _ => MonthClause::Nija,
                };
                answer.anniversaries.push(AnniversaryDay {
                    key: anniversary.key.clone(),
                    count,
                    kind,
                    origin: origin.clone(),
                    month_by,
                    observance,
                });
            }
            if year == Year::Later {
                answer.unjudged.extend(found.unjudged);
            }
        }
    }
    answer.anniversaries.sort_by(|a, b| {
        a.observance
            .tithi
            .from
            .get()
            .total_cmp(&b.observance.tithi.from.get())
    });
    Ok(answer)
}

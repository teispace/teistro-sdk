//! A rule in words (`03-design/rule-doc.md`).
//!
//! The kernel ships rules as data, and the people who can say whether a rule
//! is right read verses rather than JSON. This module is the one place the
//! kernel writes English about a rule: [`Condition`] renders as one sentence
//! and [`Rule`] as a short passage, both from the same vocabulary, so the
//! trace, the `rule-doc` command and a consumer's own display cannot drift
//! apart.
//!
//! Bodies, signs and the rest are named by their catalogue keys, because a
//! reviewer reads the prose beside the rule's own JSON and the key is the word
//! both carry. Houses are ordinals, lists are joined with a final "or" or
//! "and", and a condition inside a combinator is parenthesised, so nothing
//! about the shape of a rule is left to be guessed.
//!
//! ```
//! use teistro_rules::Rule;
//!
//! let rule: Rule = serde_json::from_str(r#"{
//!     "key": "RUCHAKA",
//!     "category": "mahapurusha",
//!     "source": { "text": "BPHS", "chapter": "76", "verse": "1-5" },
//!     "conditions": [
//!         { "type": "planet-in-kendra", "planet": "MARS" },
//!         { "type": "planet-dignity", "planet": "MARS", "dignities": ["EXALTED", "OWN_SIGN"] }
//!     ],
//!     "outcomes": [{ "type": "effect", "text": "a warrior" }]
//! }"#)?;
//! assert_eq!(
//!     rule.to_string(),
//!     "RUCHAKA — mahapurusha. BPHS ch. 76 vv. 1-5.\n\
//!      When:\n\
//!      \u{20} MARS stands in a kendra, and\n\
//!      \u{20} the dignity of MARS is exalted or own sign\n\
//!      Then: a warrior.\n"
//! );
//! assert_eq!(
//!     rule.conditions[0].to_string(),
//!     "MARS stands in a kendra"
//! );
//! # Ok::<(), serde_json::Error>(())
//! ```

use core::fmt::{self, Display, Formatter};

use teistro_core::catalogue::{Karana, Nakshatra, Rashi, Tithi, Vara, Yoga};

use crate::chart::Limb;
use crate::language::{
    Body, Condition, EclipseKind, Edge, House, Karaka, KarakaScheme, NodeSide, Pada, Relation,
    Source,
};
use crate::reference::{BodySubject, Class, Subject};
use crate::rule::{Cancellation, Group, LifeClass, Outcome, Rule, Scope, Severity, Unit};
use crate::timing::Timing;

// ---- The vocabulary --------------------------------------------------------

/// An English ordinal: 1st, 2nd, 3rd, 4th, 11th, 12th.
struct Ordinal(u8);

impl Display for Ordinal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let suffix = match (self.0 % 10, self.0 % 100) {
            (_, 11..=13) => "th",
            (1, _) => "st",
            (2, _) => "nd",
            (3, _) => "rd",
            _ => "th",
        };
        write!(f, "{}{suffix}", self.0)
    }
}

/// A count of degrees, written as short as it can be said exactly: `30`,
/// `26.6667`, `3.3333`.
struct Degrees(f64);

impl Display for Degrees {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let written = format!("{:.4}", self.0);
        let trimmed = written.trim_end_matches('0').trim_end_matches('.');
        write!(f, "{trimmed}°")
    }
}

/// The word a list of alternatives ends with.
const OR: &str = " or ";
/// The word a list of requirements ends with.
const AND: &str = " and ";

/// Writes the items separated by commas, the last preceded by `last`.
fn joined<T>(
    f: &mut Formatter<'_>,
    items: &[T],
    last: &str,
    mut each: impl FnMut(&mut Formatter<'_>, &T) -> fmt::Result,
) -> fmt::Result {
    for (at, item) in items.iter().enumerate() {
        if at > 0 {
            f.write_str(if at + 1 == items.len() { last } else { ", " })?;
        }
        each(f, item)?;
    }
    Ok(())
}

/// Writes the items by their catalogue keys.
fn keys<T: Copy>(
    f: &mut Formatter<'_>,
    items: &[T],
    last: &str,
    key: impl Fn(T) -> &'static str,
) -> fmt::Result {
    joined(f, items, last, |f, item| f.write_str(key(*item)))
}

/// Writes the houses as ordinals: `1st, 4th or 10th`.
fn ordinals(f: &mut Formatter<'_>, houses: &[House], last: &str) -> fmt::Result {
    joined(f, houses, last, |f, house| {
        write!(f, "{}", Ordinal(house.get()))
    })
}

/// Writes `the 1st, 4th or 10th house`, or `the 1st house`.
fn the_houses(f: &mut Formatter<'_>, houses: &[House]) -> fmt::Result {
    f.write_str("the ")?;
    ordinals(f, houses, OR)?;
    f.write_str(" house")
}

/// Writes the bodies by their keys.
fn bodies(f: &mut Formatter<'_>, bodies: &[Body], last: &str) -> fmt::Result {
    keys(f, bodies, last, Body::key)
}

/// Writes the padas as `its 1st or 4th pada`, and nothing for none.
fn padas(f: &mut Formatter<'_>, padas: &[Pada]) -> fmt::Result {
    if padas.is_empty() {
        return Ok(());
    }
    f.write_str(", in its ")?;
    joined(f, padas, OR, |f, pada| write!(f, "{}", Ordinal(pada.get())))?;
    f.write_str(" pada")
}

/// Writes the bodies a list names, or `the nine grahas` for the nine.
fn whichever(f: &mut Formatter<'_>, planets: &[Body]) -> fmt::Result {
    if Body::is_nine(planets) {
        f.write_str("the nine grahas")
    } else {
        bodies(f, planets, OR)
    }
}

impl Display for Subject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Subject::Ref(reference) => write!(f, "{reference}"),
            Subject::AnyBenefic => f.write_str("any benefic"),
            Subject::AnyMalefic => f.write_str("any malefic"),
            Subject::AnyMaraka => f.write_str("any maraka"),
        }
    }
}

impl Display for BodySubject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BodySubject::Ref(reference) => write!(f, "{reference}"),
            BodySubject::AnyBenefic => f.write_str("any benefic"),
            BodySubject::AnyMalefic => f.write_str("any malefic"),
            BodySubject::AnyMaraka => f.write_str("any maraka"),
        }
    }
}

/// What a subject is called where its members are counted: `malefic`,
/// `malefics`, or the reference itself where a rule counts one named body.
fn counted(f: &mut Formatter<'_>, subject: &BodySubject, count: u8) -> fmt::Result {
    let class = match subject {
        BodySubject::Ref(reference) => return write!(f, "{reference}"),
        BodySubject::AnyBenefic => "benefic",
        BodySubject::AnyMalefic => "malefic",
        BodySubject::AnyMaraka => "maraka",
    };
    f.write_str(class)?;
    plural(f, u16::from(count))
}

/// The form of a verb that agrees with a count.
const fn agrees(count: u8, one: &'static str, many: &'static str) -> &'static str {
    if count == 1 { one } else { many }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Class::Benefic => "a benefic",
            Class::Malefic => "a malefic",
            Class::Maraka => "a maraka",
        })
    }
}

impl Display for Relation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Relation::Friend => "a friend",
            Relation::Neutral => "a neutral",
            Relation::Enemy => "an enemy",
        })
    }
}

impl Display for Limb {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Limb::Tithi => "the tithi",
            Limb::Nakshatra => "the nakshatra",
            Limb::Lagna => "the rising sign",
        })
    }
}

impl Display for Karaka {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.abbreviation())
    }
}

/// A chara karaka as a condition names it, with its scheme.
fn karaka(f: &mut Formatter<'_>, karaka: Karaka, scheme: KarakaScheme) -> fmt::Result {
    match scheme {
        KarakaScheme::Seven => write!(f, "the {karaka}"),
        KarakaScheme::Eight => write!(f, "the {karaka} among eight"),
    }
}

/// What a quantifier or a division says before the condition inside it:
/// `some one of the nine grahas meets`, `at least 4 of MARS or SATURN meet`,
/// `in the D9`. The sentence and the opening a trace prints share it, so the
/// two cannot say it differently.
fn over(f: &mut Formatter<'_>, condition: &Condition) -> fmt::Result {
    match condition {
        Condition::ForAny { planets, .. } => {
            f.write_str("some one of ")?;
            whichever(f, planets)?;
            f.write_str(" meets")
        }
        Condition::CountOf {
            planets,
            at_least,
            at_most,
            ..
        } => {
            write!(f, "at least {at_least}")?;
            if let Some(most) = at_most {
                write!(f, " and at most {most}")?;
            }
            f.write_str(" of ")?;
            whichever(f, planets)?;
            f.write_str(" meet")
        }
        Condition::InVarga { varga, .. } => write!(f, "in the {}", varga.key()),
        other => write!(f, "{other}"),
    }
}

/// The opening of a condition's sentence, with the conditions inside it left
/// to the steps that checked them: what a trace prints, so that a reader of
/// an explanation reads the rule and not the schema.
///
/// ```
/// use teistro_rules::{Condition, prose};
///
/// let condition: Condition = serde_json::from_str(
///     r#"{"type": "for-any", "then": {"type": "planet-combust", "planet": "SELF"}}"#,
/// )?;
/// assert_eq!(
///     condition.to_string(),
///     "some one of the nine grahas meets: the body found is combust"
/// );
/// assert_eq!(
///     prose::opening(&condition).to_string(),
///     "some one of the nine grahas meets this"
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[must_use]
pub const fn opening(condition: &Condition) -> Opening<'_> {
    Opening(condition)
}

/// What [`opening`] writes.
#[derive(Clone, Copy, Debug)]
pub struct Opening<'c>(&'c Condition);

impl Display for Opening<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Condition::And { .. } => f.write_str("all of these"),
            Condition::Or { .. } => f.write_str("one of these"),
            Condition::Not { .. } => f.write_str("not this"),
            condition @ (Condition::ForAny { .. } | Condition::CountOf { .. }) => {
                over(f, condition)?;
                f.write_str(" this")
            }
            condition @ Condition::InVarga { .. } => {
                over(f, condition)?;
                f.write_str(", this")
            }
            leaf => write!(f, "{leaf}"),
        }
    }
}

/// A condition inside another, parenthesised when it is itself a combination
/// so that no reader has to guess where it ends.
struct Inner<'c>(&'c Condition);

impl Display for Inner<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.0.children().is_empty() {
            write!(f, "{}", self.0)
        } else {
            write!(f, "({})", self.0)
        }
    }
}

// ---- A condition -----------------------------------------------------------

impl Display for Condition {
    #[allow(clippy::too_many_lines, reason = "one arm a predicate of the language")]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Condition::And { conditions } if conditions.is_empty() => {
                f.write_str("nothing is required")
            }
            Condition::Or { conditions } if conditions.is_empty() => {
                f.write_str("nothing can hold")
            }
            Condition::And { conditions } => joined(f, conditions, AND, |f, condition| {
                write!(f, "{}", Inner(condition))
            }),
            Condition::Or { conditions } => joined(f, conditions, OR, |f, condition| {
                write!(f, "{}", Inner(condition))
            }),
            Condition::Not { condition } => {
                write!(f, "it is not the case that {}", Inner(condition))
            }
            Condition::PlanetInHouse { planet, houses } => {
                write!(f, "{planet} stands in ")?;
                the_houses(f, houses)
            }
            Condition::PlanetInSign { planet, signs } => {
                write!(f, "{planet} stands in ")?;
                keys(f, signs, OR, Rashi::key)
            }
            Condition::PlanetDignity { planet, dignities } => {
                write!(f, "the dignity of {planet} is ")?;
                joined(f, dignities, OR, |f, dignity| {
                    f.write_str(&dignity.doc().to_ascii_lowercase())
                })
            }
            Condition::PlanetInKendra { planet } => write!(f, "{planet} stands in a kendra"),
            Condition::PlanetInTrikona { planet } => write!(f, "{planet} stands in a trikona"),
            Condition::PlanetInKendraFrom { planet, reference } => {
                write!(f, "{planet} stands in a kendra from {reference}")
            }
            Condition::LordOfHouseInKendra { house_ruled } => write!(
                f,
                "the lord of the {} house stands in a kendra",
                Ordinal(house_ruled.get())
            ),
            Condition::LordOfHouseInHouse {
                house_ruled,
                house_occupied,
            } => write!(
                f,
                "the lord of the {} house stands in the {} house",
                Ordinal(house_ruled.get()),
                Ordinal(house_occupied.get())
            ),
            Condition::PlanetConjunct { planets, max_orb } => {
                joined(f, planets, AND, |f, planet| write!(f, "{planet}"))?;
                match max_orb {
                    None => f.write_str(" share a sign"),
                    Some(orb) => write!(f, " stand within {} of one another", Degrees(*orb)),
                }
            }
            Condition::PlanetInHouseFrom {
                planet,
                reference,
                houses,
            } => {
                write!(f, "{planet} stands in the ")?;
                ordinals(f, houses, OR)?;
                write!(f, " from {reference}")
            }
            Condition::NoPlanetInHousesFrom {
                reference,
                houses,
                except,
            } => {
                f.write_str("no graha stands in the ")?;
                ordinals(f, houses, OR)?;
                write!(f, " from {reference}")?;
                if !except.is_empty() {
                    f.write_str(", excepting ")?;
                    bodies(f, except, AND)?;
                }
                Ok(())
            }
            Condition::MutualExchange { house1, house2 } => write!(
                f,
                "the lords of the {} and {} houses stand each in the other's sign",
                Ordinal(house1.get()),
                Ordinal(house2.get())
            ),
            Condition::LordConjunctLord { house1, house2 } => write!(
                f,
                "the lord of the {} house shares a sign with the lord of the {}",
                Ordinal(house1.get()),
                Ordinal(house2.get())
            ),
            Condition::AllPlanetsBetweenNodes { side } => {
                f.write_str("the seven classical grahas all stand between the nodes")?;
                match side {
                    None => Ok(()),
                    Some(NodeSide::Rahu) => f.write_str(", from RAHU to KETU"),
                    Some(NodeSide::Ketu) => f.write_str(", from KETU to RAHU"),
                }
            }
            Condition::OccupiedSignCount { planets, count } => {
                joined(f, planets, AND, |f, planet| write!(f, "{planet}"))?;
                write!(f, " occupy exactly {count} sign")?;
                plural(f, u16::from(*count))
            }
            Condition::AllClassicalGrahasInHouses {
                houses,
                require_all_houses_filled,
            } => {
                f.write_str("the seven classical grahas stand only in ")?;
                the_houses(f, houses)?;
                f.write_str(if *require_all_houses_filled {
                    ", filling every one of them"
                } else {
                    ", not every one of them filled"
                })
            }
            Condition::NGrahasConjunctWith { anchor, min_count } => write!(
                f,
                "at least {min_count} classical grahas share the sign of {anchor}, it among them"
            ),
            Condition::CharaKarakaInHouse {
                karaka: which,
                houses,
                karaka_scheme,
            } => {
                karaka(f, *which, *karaka_scheme)?;
                f.write_str(" stands in ")?;
                the_houses(f, houses)
            }
            Condition::PlanetCombust { planet } => write!(f, "{planet} is combust"),
            Condition::PlanetRetrograde { planet } => write!(f, "{planet} is retrograde"),
            Condition::PlanetAspectsPlanet { from, target } => {
                write!(f, "{from} aspects {target}")
            }
            Condition::PlanetAspectsHouse { from, house_ruled } => {
                write!(f, "{from} aspects the {} house", Ordinal(house_ruled.get()))
            }
            Condition::PlanetAtTableDegree { planet, table } => write!(
                f,
                "{planet} stands at the degree {table} gives it in its sign"
            ),
            Condition::PlanetInTableSign { planet, table } => {
                write!(f, "{planet} stands in a sign {table} gives the birth tithi")
            }
            Condition::LordOfHouseDebilitated { house_ruled } => write!(
                f,
                "the lord of the {} house is debilitated",
                Ordinal(house_ruled.get())
            ),
            Condition::LordOfHouseCombust { house_ruled } => write!(
                f,
                "the lord of the {} house is combust",
                Ordinal(house_ruled.get())
            ),
            Condition::LordOfHouseStrong { house_ruled } => write!(
                f,
                "the lord of the {} house is exalted, in its own sign or in its mooltrikona",
                Ordinal(house_ruled.get())
            ),
            Condition::LordOfHouseIs {
                house_ruled,
                planets,
            } => {
                write!(
                    f,
                    "the lord of the {} house is ",
                    Ordinal(house_ruled.get())
                )?;
                bodies(f, planets, OR)
            }
            Condition::LordOfHouseConjunctPlanet {
                house_ruled,
                with_planet,
            } => write!(
                f,
                "the lord of the {} house shares a sign with {}",
                Ordinal(house_ruled.get()),
                with_planet.key()
            ),
            Condition::LagnaInSign { signs } => {
                f.write_str("the lagna rises in ")?;
                keys(f, signs, OR, Rashi::key)
            }
            Condition::PlanetInHouseAndSign {
                planet,
                houses,
                signs,
            } => {
                write!(f, "{} stands in ", planet.key())?;
                the_houses(f, houses)?;
                f.write_str(" and in ")?;
                keys(f, signs, OR, Rashi::key)
            }
            Condition::PlanetInDegrees { planet, from, to } => write!(
                f,
                "{planet} stands between {} and {} of its sign",
                Degrees(*from),
                Degrees(*to)
            ),
            Condition::PlanetAtGandanta {
                planet,
                orb_degrees,
            } => {
                write!(f, "{} stands at a gandanta junction, within ", planet.key())?;
                match orb_degrees {
                    None => f.write_str("the usual 3°20′"),
                    Some(orb) => write!(f, "{}", Degrees(*orb)),
                }
            }
            Condition::PanchangaTithi { tithis } => {
                f.write_str("the birth tithi is ")?;
                keys(f, tithis, OR, Tithi::key)
            }
            Condition::PanchangaPaksha { paksha } => {
                write!(f, "the birth falls in the {} paksha", paksha.key())
            }
            Condition::PanchangaVara { varas } => {
                f.write_str("the birth weekday is ")?;
                keys(f, varas, OR, Vara::key)
            }
            Condition::PanchangaNakshatra { nakshatras, padas } => {
                f.write_str("the Moon's birth nakshatra is ")?;
                keys(f, nakshatras, OR, Nakshatra::key)?;
                self::padas(f, padas)
            }
            Condition::PanchangaYoga { yogas } => {
                f.write_str("the panchanga yoga at birth is ")?;
                keys(f, yogas, OR, Yoga::key)
            }
            Condition::PanchangaKarana { karanas } => {
                f.write_str("the karana at birth is ")?;
                keys(f, karanas, OR, Karana::key)
            }
            Condition::BirthDuringEclipse { kind } => {
                f.write_str("the birth falls in ")?;
                f.write_str(match kind {
                    None => "an eclipse",
                    Some(EclipseKind::Solar) => "an eclipse of the Sun",
                    Some(EclipseKind::Lunar) => "an eclipse of the Moon",
                    Some(EclipseKind::Any) => "an eclipse of either kind",
                })
            }
            Condition::BirthOnSankranti { window_hours } => {
                f.write_str("the birth falls on a sankranti")?;
                match window_hours {
                    None => Ok(()),
                    Some(hours) => write!(f, ", within {hours} hours of it"),
                }
            }
            Condition::PlanetInNakshatra {
                planet,
                nakshatras,
                padas,
            } => {
                write!(f, "{planet} stands in the nakshatra ")?;
                keys(f, nakshatras, OR, Nakshatra::key)?;
                self::padas(f, padas)
            }
            Condition::SameNakshatra { of, as_body } => {
                write!(f, "{of} and {as_body} stand in one nakshatra")
            }
            Condition::PlanetStrong { planet } => write!(f, "{planet} is strong"),
            Condition::PlanetWeak { planet } => write!(f, "{planet} is weak"),
            Condition::PlanetStrongerThan { planet, than } => {
                write!(f, "{planet} is stronger than {than}")
            }
            Condition::RashiAspects { from, target } => {
                write!(f, "{from} aspects {target} by rashi drishti")
            }
            Condition::Argala { on, place } => {
                let (from, blocked) = place.houses();
                write!(
                    f,
                    "an intervention from the {}, unobstructed from the {}, stands on {on}",
                    Ordinal(from),
                    Ordinal(blocked)
                )
            }
            Condition::VipareetaArgala { on } => write!(
                f,
                "three or more malefics stand in the 3rd from {on}, a contrary intervention"
            ),
            Condition::ForAny { then, .. }
            | Condition::CountOf { then, .. }
            | Condition::InVarga {
                condition: then, ..
            } => {
                over(f, self)?;
                f.write_str(match self {
                    Condition::InVarga { .. } => ", ",
                    _ => ": ",
                })?;
                write!(f, "{}", Inner(then))
            }
            Condition::CountInHouses {
                planets,
                houses,
                from,
                at_least,
                except,
            } => {
                let class = planets.class().is_some();
                if class {
                    write!(f, "at least {at_least} ")?;
                    counted(f, planets, *at_least)?;
                    write!(f, " {} in the ", agrees(*at_least, "stands", "stand"))?;
                } else {
                    f.write_str("the count of ")?;
                    counted(f, planets, *at_least)?;
                    f.write_str(" in the ")?;
                }
                ordinals(f, houses, OR)?;
                write!(f, " from {from}")?;
                if !class {
                    write!(f, " is at least {at_least}")?;
                }
                if !except.is_empty() {
                    f.write_str(", not counting ")?;
                    joined(f, except, AND, |f, body| write!(f, "{body}"))?;
                }
                Ok(())
            }
            Condition::CountAspecting {
                planets,
                target,
                at_least,
            } => {
                if planets.class().is_some() {
                    write!(f, "at least {at_least} ")?;
                    counted(f, planets, *at_least)?;
                    write!(f, " {} {target}", agrees(*at_least, "aspects", "aspect"))
                } else {
                    f.write_str("the count of ")?;
                    counted(f, planets, *at_least)?;
                    write!(f, " aspecting {target} is at least {at_least}")
                }
            }
            Condition::RuleHolds { key } => write!(f, "the rule {key} holds"),
            Condition::AtLimbEdge {
                limb,
                edge,
                ghatikas,
            } => write!(
                f,
                "the birth falls within the {} {ghatikas} ghatikas of {limb}",
                match edge {
                    Edge::First => "first",
                    Edge::Last => "last",
                }
            ),
            Condition::BirthByDay => f.write_str("the birth fell by day"),
            Condition::SameSign { of, as_sign } => {
                write!(f, "{of} and {as_sign} stand in one sign")
            }
            Condition::SameBody { of, as_body } => write!(f, "{of} and {as_body} are one body"),
            Condition::PlanetIs { planet, class } => write!(f, "{planet} is {class}"),
            Condition::NaturalRelation { of, to, relations } => {
                write!(f, "{of} counts {to} ")?;
                joined(f, relations, OR, |f, relation| write!(f, "{relation}"))
            }
        }
    }
}

/// An `s`, unless there is exactly one of them.
fn plural(f: &mut Formatter<'_>, count: u16) -> fmt::Result {
    if count == 1 { Ok(()) } else { f.write_str("s") }
}

// ---- A rule ----------------------------------------------------------------

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)?;
        if let Some(chapter) = &self.chapter {
            write!(f, " ch. {chapter}")?;
        }
        if let Some(verse) = &self.verse {
            let many = verse.contains(['-', ',', '–']) || verse.contains(" to ");
            write!(f, " {} {verse}", if many { "vv." } else { "v." })?;
        }
        if let Some(rank) = self.rank {
            write!(f, " (rank {})", u8::from(rank))?;
        }
        Ok(())
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Scope::Natal => "a birth chart",
            Scope::Milan => "a match",
            Scope::Muhurta => "a chosen moment",
            Scope::All => "any chart",
        })
    }
}

impl Display for LifeClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LifeClass::Balarishta => "a death in infancy",
            LifeClass::Yogarishta => "an evil from a yoga",
            LifeClass::Short => "a short life",
            LifeClass::Medium => "a medium life",
            LifeClass::Long => "a long life",
            LifeClass::Divine => "a super-natural life",
            LifeClass::Unlimited => "an illimitable life",
        })?;
        match self.years() {
            Some(years) => write!(f, ", {years} years"),
            None => Ok(()),
        }
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::LifeSpan { count, unit } => {
                write!(
                    f,
                    "a life of {count} {}",
                    match unit {
                        Unit::Days => "days",
                        Unit::Months => "months",
                        Unit::Years => "years",
                    }
                )
            }
            Outcome::LifeClass { class } => write!(f, "{class}"),
            Outcome::Effect { text } => f.write_str(text),
        }
    }
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Fixed { value } => write!(f, "{value}"),
            Severity::HouseWeighted {
                planet,
                weights,
                default,
            } => {
                write!(
                    f,
                    "by the house {} stands in from each place it is found from — ",
                    planet.key()
                )?;
                let weights: Vec<(House, u16)> =
                    weights.iter().map(|(house, at)| (*house, *at)).collect();
                joined(f, &weights, AND, |f, (house, at)| {
                    write!(f, "the {}: {at}", Ordinal(house.get()))
                })?;
                write!(f, " — and {default} elsewhere")
            }
            Severity::PlanetStrengthInverse {
                base_planet,
                max,
                min,
            } => write!(
                f,
                "{max} when {} is debilitated, {min} when it is exalted or in its own sign, halfway otherwise",
                base_planet.key()
            ),
            Severity::CountBased {
                per_occurrence,
                cap,
            } => write!(
                f,
                "{per_occurrence} for each place it is found from, at most {cap}"
            ),
            Severity::KootShortfall {
                full,
                per_point_missing,
            } => write!(
                f,
                "{full} with no koota score, less by {per_point_missing} for each point regained"
            ),
        }
    }
}

impl Display for Timing {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Timing::Concerned => {
                "in the periods of the grahas concerned, and of the signs they stand in"
            }
            Timing::Throughout => "in every period",
        })
    }
}

/// One line of a rule's conditions, indented, the last without the comma.
fn conditions(f: &mut Formatter<'_>, conditions: &[Condition], indent: &str) -> fmt::Result {
    let alone = conditions.len() == 1;
    for (at, condition) in conditions.iter().enumerate() {
        f.write_str(indent)?;
        if alone {
            write!(f, "{condition}")?;
        } else {
            write!(f, "{}", Inner(condition))?;
        }
        writeln!(
            f,
            "{}",
            if at + 1 == conditions.len() {
                ""
            } else {
                ", and"
            }
        )?;
    }
    Ok(())
}

impl Display for Group {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "  {}, \"{}\"", self.reference.key(), self.label)?;
        if self.weight != 1 {
            write!(f, ", weight {}", self.weight)?;
        }
        writeln!(f, ":")?;
        conditions(f, &self.conditions, "    ")
    }
}

impl Display for Cancellation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.label {
            Some(label) => write!(f, "\"{label}\": {}", self.condition),
            None => write!(f, "{}", self.condition),
        }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} — {}", self.key, self.category)?;
        if self.scope != Scope::Natal {
            write!(f, ", {}", self.scope)?;
        }
        writeln!(f, ". {}.", self.source)?;
        if let Some(note) = &self.source.note {
            writeln!(f, "Note: {note}")?;
        }
        if !self.conditions.is_empty() {
            writeln!(f, "When:")?;
            conditions(f, &self.conditions, "  ")?;
        }
        if !self.groups.is_empty() {
            writeln!(f, "Found from any of:")?;
            for group in &self.groups {
                write!(f, "{group}")?;
            }
        }
        if !self.cancellations.is_empty() {
            writeln!(f, "Cancelled by:")?;
            for cancellation in &self.cancellations {
                writeln!(f, "  {cancellation}")?;
            }
            if let Some(threshold) = self.full_cancellation_threshold {
                writeln!(
                    f,
                    "  {threshold} of them {} it fully; fewer cancel it in part.",
                    agrees(threshold, "cancels", "cancel")
                )?;
            }
        }
        if let Some(severity) = &self.severity {
            writeln!(f, "Severity: {severity}.")?;
        }
        if !self.outcomes.is_empty() {
            f.write_str("Then: ")?;
            joined(f, &self.outcomes, AND, |f, outcome| write!(f, "{outcome}"))?;
            writeln!(f, ".")?;
        }
        if self.timing != Timing::default() {
            writeln!(f, "Felt {}.", self.timing)?;
        }
        if !self.remedies.is_empty() {
            f.write_str("Remedies: ")?;
            joined(f, &self.remedies, AND, |f, remedy| f.write_str(remedy))?;
            writeln!(f, ".")?;
        }
        if let Some(computed) = &self.computed {
            writeln!(
                f,
                "Computed in code as {computed}: the language does not say it."
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "tests unwrap what they read and fail by panicking"
    )]

    use std::collections::BTreeSet;

    use super::*;
    use crate::language::KINDS;
    use crate::shipped;

    /// One written condition for each kind of the language, with the sentence
    /// it must read as. Hand-written rather than captured: a phrasing that
    /// changes is a phrasing a reviewer agreed to change.
    const SENTENCES: [(&str, &str); 64] = [
        (
            r#"{"type": "and", "conditions": [{"type": "birth-by-day"}, {"type": "planet-combust", "planet": "MOON"}]}"#,
            "the birth fell by day and MOON is combust",
        ),
        (
            r#"{"type": "or", "conditions": [{"type": "birth-by-day"}, {"type": "planet-combust", "planet": "MOON"}]}"#,
            "the birth fell by day or MOON is combust",
        ),
        (
            r#"{"type": "not", "condition": {"type": "birth-by-day"}}"#,
            "it is not the case that the birth fell by day",
        ),
        (
            r#"{"type": "planet-in-house", "planet": "MARS", "houses": [1, 4, 7, 10]}"#,
            "MARS stands in the 1st, 4th, 7th or 10th house",
        ),
        (
            r#"{"type": "planet-in-sign", "planet": "VENUS", "signs": ["PISCES"]}"#,
            "VENUS stands in PISCES",
        ),
        (
            r#"{"type": "planet-dignity", "planet": "SUN", "dignities": ["EXALTED", "OWN_SIGN"]}"#,
            "the dignity of SUN is exalted or own sign",
        ),
        (
            r#"{"type": "planet-in-kendra", "planet": "JUPITER"}"#,
            "JUPITER stands in a kendra",
        ),
        (
            r#"{"type": "planet-in-trikona", "planet": "JUPITER"}"#,
            "JUPITER stands in a trikona",
        ),
        (
            r#"{"type": "planet-in-kendra-from", "planet": "JUPITER", "reference": "MOON"}"#,
            "JUPITER stands in a kendra from MOON",
        ),
        (
            r#"{"type": "lord-of-house-in-kendra", "houseRuled": 9}"#,
            "the lord of the 9th house stands in a kendra",
        ),
        (
            r#"{"type": "lord-of-house-in-house", "houseRuled": 9, "houseOccupied": 10}"#,
            "the lord of the 9th house stands in the 10th house",
        ),
        (
            r#"{"type": "planet-conjunct", "planets": ["MOON", "MARS"]}"#,
            "MOON and MARS share a sign",
        ),
        (
            r#"{"type": "planet-in-house-from", "planet": "SUN", "reference": "MOON", "houses": [6, 8, 12]}"#,
            "SUN stands in the 6th, 8th or 12th from MOON",
        ),
        (
            r#"{"type": "no-planet-in-houses-from", "reference": "LAGNA", "houses": [2, 12]}"#,
            "no graha stands in the 2nd or 12th from LAGNA",
        ),
        (
            r#"{"type": "mutual-exchange", "house1": 1, "house2": 7}"#,
            "the lords of the 1st and 7th houses stand each in the other's sign",
        ),
        (
            r#"{"type": "lord-conjunct-lord", "house1": 1, "house2": 7}"#,
            "the lord of the 1st house shares a sign with the lord of the 7th",
        ),
        (
            r#"{"type": "all-planets-between-nodes"}"#,
            "the seven classical grahas all stand between the nodes",
        ),
        (
            r#"{"type": "occupied-sign-count", "planets": ["SUN", "MOON"], "count": 1}"#,
            "SUN and MOON occupy exactly 1 sign",
        ),
        (
            r#"{"type": "all-classical-grahas-in-houses", "houses": [1, 7]}"#,
            "the seven classical grahas stand only in the 1st or 7th house, filling every one of them",
        ),
        (
            r#"{"type": "n-grahas-conjunct-with", "anchor": "MOON", "minCount": 4}"#,
            "at least 4 classical grahas share the sign of MOON, it among them",
        ),
        (
            r#"{"type": "chara-karaka-in-house", "karaka": "AK", "houses": [1]}"#,
            "the AK stands in the 1st house",
        ),
        (
            r#"{"type": "planet-combust", "planet": "MERCURY"}"#,
            "MERCURY is combust",
        ),
        (
            r#"{"type": "planet-retrograde", "planet": "SATURN"}"#,
            "SATURN is retrograde",
        ),
        (
            r#"{"type": "planet-aspects-planet", "from": "JUPITER", "target": "MOON"}"#,
            "JUPITER aspects MOON",
        ),
        (
            r#"{"type": "planet-aspects-house", "from": "SATURN", "houseRuled": 7}"#,
            "SATURN aspects the 7th house",
        ),
        (
            r#"{"type": "planet-at-table-degree", "planet": "MOON", "table": "MRITYU_BHAGA"}"#,
            "MOON stands at the degree MRITYU_BHAGA gives it in its sign",
        ),
        (
            r#"{"type": "planet-in-table-sign", "planet": "MOON", "table": "DAGDHA_RASHI"}"#,
            "MOON stands in a sign DAGDHA_RASHI gives the birth tithi",
        ),
        (
            r#"{"type": "lord-of-house-debilitated", "houseRuled": 6}"#,
            "the lord of the 6th house is debilitated",
        ),
        (
            r#"{"type": "lord-of-house-combust", "houseRuled": 6}"#,
            "the lord of the 6th house is combust",
        ),
        (
            r#"{"type": "lord-of-house-strong", "houseRuled": 1}"#,
            "the lord of the 1st house is exalted, in its own sign or in its mooltrikona",
        ),
        (
            r#"{"type": "lord-of-house-is", "houseRuled": 1, "planets": ["MARS", "VENUS"]}"#,
            "the lord of the 1st house is MARS or VENUS",
        ),
        (
            r#"{"type": "lord-of-house-conjunct-planet", "houseRuled": 1, "withPlanet": "SATURN"}"#,
            "the lord of the 1st house shares a sign with SATURN",
        ),
        (
            r#"{"type": "lagna-in-sign", "signs": ["ARIES", "LEO"]}"#,
            "the lagna rises in ARIES or LEO",
        ),
        (
            r#"{"type": "planet-in-house-and-sign", "planet": "MARS", "houses": [1], "signs": ["ARIES"]}"#,
            "MARS stands in the 1st house and in ARIES",
        ),
        (
            r#"{"type": "planet-in-degrees", "planet": "MOON", "from": 26.6667, "to": 30}"#,
            "MOON stands between 26.6667° and 30° of its sign",
        ),
        (
            r#"{"type": "planet-at-gandanta", "planet": "MOON"}"#,
            "MOON stands at a gandanta junction, within the usual 3°20′",
        ),
        (
            r#"{"type": "panchanga-tithi", "tithis": ["AMAVASYA"]}"#,
            "the birth tithi is AMAVASYA",
        ),
        (
            r#"{"type": "panchanga-paksha", "paksha": "krishna"}"#,
            "the birth falls in the KRISHNA paksha",
        ),
        (
            r#"{"type": "panchanga-vara", "varas": ["SHANIVARA"]}"#,
            "the birth weekday is SHANIVARA",
        ),
        (
            r#"{"type": "panchanga-nakshatra", "nakshatras": ["MULA"], "padas": [1]}"#,
            "the Moon's birth nakshatra is MULA, in its 1st pada",
        ),
        (
            r#"{"type": "panchanga-yoga", "yogas": ["VYATIPATA"]}"#,
            "the panchanga yoga at birth is VYATIPATA",
        ),
        (
            r#"{"type": "panchanga-karana", "karanas": ["VISHTI"]}"#,
            "the karana at birth is VISHTI",
        ),
        (
            r#"{"type": "birth-during-eclipse"}"#,
            "the birth falls in an eclipse",
        ),
        (
            r#"{"type": "birth-on-sankranti"}"#,
            "the birth falls on a sankranti",
        ),
        (
            r#"{"type": "planet-in-nakshatra", "planet": "KETU", "nakshatras": ["MULA"]}"#,
            "KETU stands in the nakshatra MULA",
        ),
        (
            r#"{"type": "same-nakshatra", "of": "MOON", "as": "KETU"}"#,
            "MOON and KETU stand in one nakshatra",
        ),
        (
            r#"{"type": "planet-strong", "planet": {"lordOf": 1}}"#,
            "the lord of house 1 is strong",
        ),
        (r#"{"type": "planet-weak", "planet": "SUN"}"#, "SUN is weak"),
        (
            r#"{"type": "planet-stronger-than", "planet": "JUPITER", "than": "SATURN"}"#,
            "JUPITER is stronger than SATURN",
        ),
        (
            r#"{"type": "rashi-aspects", "from": "MARS", "target": 7}"#,
            "MARS aspects house 7 by rashi drishti",
        ),
        (
            r#"{"type": "argala", "on": "MOON", "place": "fourth"}"#,
            "an intervention from the 4th, unobstructed from the 10th, stands on MOON",
        ),
        (
            r#"{"type": "vipareeta-argala", "on": "MOON"}"#,
            "three or more malefics stand in the 3rd from MOON, a contrary intervention",
        ),
        (
            r#"{"type": "for-any", "then": {"type": "planet-in-kendra", "planet": "SELF"}}"#,
            "some one of the nine grahas meets: the body found stands in a kendra",
        ),
        (
            r#"{"type": "count-of", "then": {"type": "planet-in-kendra", "planet": "SELF"}, "atLeast": 4}"#,
            "at least 4 of the nine grahas meet: the body found stands in a kendra",
        ),
        (
            r#"{"type": "in-varga", "varga": "D9", "condition": {"type": "planet-in-kendra", "planet": "MARS"}}"#,
            "in the D9, MARS stands in a kendra",
        ),
        (
            r#"{"type": "count-in-houses", "planets": "any-malefic", "houses": [6, 8, 12], "atLeast": 3}"#,
            "at least 3 malefics stand in the 6th, 8th or 12th from LAGNA",
        ),
        (
            r#"{"type": "count-aspecting", "planets": "any-benefic", "target": "MOON", "atLeast": 2}"#,
            "at least 2 benefics aspect MOON",
        ),
        (
            r#"{"type": "rule", "key": "NEECHA_BHANGA"}"#,
            "the rule NEECHA_BHANGA holds",
        ),
        (
            r#"{"type": "at-limb-edge", "limb": "nakshatra", "edge": "last", "ghatikas": 2}"#,
            "the birth falls within the last 2 ghatikas of the nakshatra",
        ),
        (r#"{"type": "birth-by-day"}"#, "the birth fell by day"),
        (
            r#"{"type": "same-sign", "of": "MOON", "as": 7}"#,
            "MOON and house 7 stand in one sign",
        ),
        (
            r#"{"type": "same-body", "of": {"lordOf": 1}, "as": "MARS"}"#,
            "the lord of house 1 and MARS are one body",
        ),
        (
            r#"{"type": "planet-is", "planet": "SATURN", "class": "maraka"}"#,
            "SATURN is a maraka",
        ),
        (
            r#"{"type": "natural-relation", "of": {"lordOf": 1}, "to": "SUN", "relations": ["friend"]}"#,
            "the lord of house 1 counts SUN a friend",
        ),
    ];

    fn read(json: &str) -> Condition {
        serde_json::from_str(json).unwrap_or_else(|err| panic!("{json} reads: {err}"))
    }

    #[test]
    fn every_kind_of_condition_has_a_sentence_and_reads_as_it_should() {
        let mut covered = BTreeSet::new();
        for (json, expected) in SENTENCES {
            let condition = read(json);
            assert_eq!(condition.to_string(), expected, "for {json}");
            assert!(
                covered.insert(condition.kind()),
                "two sentences for {}",
                condition.kind()
            );
        }
        let listed: BTreeSet<&str> = KINDS.into_iter().collect();
        assert_eq!(covered, listed, "every kind of the language has a sentence");
    }

    /// A field a rule may leave out changes the sentence when it is there:
    /// the collision gate over the corpus finds a dropped field only once two
    /// rules differ in exactly that field, and these are the fields rare
    /// enough that they might not.
    #[test]
    fn an_optional_field_is_never_silent() {
        for (bare, with, changed) in [
            (
                r#"{"type": "planet-conjunct", "planets": ["MOON", "MARS"]}"#,
                r#"{"type": "planet-conjunct", "planets": ["MOON", "MARS"], "maxOrb": 10}"#,
                "MOON and MARS stand within 10° of one another",
            ),
            (
                r#"{"type": "no-planet-in-houses-from", "reference": "LAGNA", "houses": [2]}"#,
                r#"{"type": "no-planet-in-houses-from", "reference": "LAGNA", "houses": [2], "except": ["MOON", "KETU"]}"#,
                "no graha stands in the 2nd from LAGNA, excepting MOON and KETU",
            ),
            (
                r#"{"type": "all-planets-between-nodes"}"#,
                r#"{"type": "all-planets-between-nodes", "side": "rahu"}"#,
                "the seven classical grahas all stand between the nodes, from RAHU to KETU",
            ),
            (
                r#"{"type": "all-classical-grahas-in-houses", "houses": [1]}"#,
                r#"{"type": "all-classical-grahas-in-houses", "houses": [1], "requireAllHousesFilled": false}"#,
                "the seven classical grahas stand only in the 1st house, not every one of them filled",
            ),
            (
                r#"{"type": "chara-karaka-in-house", "karaka": "AK", "houses": [1]}"#,
                r#"{"type": "chara-karaka-in-house", "karaka": "AK", "houses": [1], "karakaScheme": 8}"#,
                "the AK among eight stands in the 1st house",
            ),
            (
                r#"{"type": "planet-at-gandanta", "planet": "MOON"}"#,
                r#"{"type": "planet-at-gandanta", "planet": "MOON", "orbDegrees": 5}"#,
                "MOON stands at a gandanta junction, within 5°",
            ),
            (
                r#"{"type": "planet-in-nakshatra", "planet": "KETU", "nakshatras": ["MULA"]}"#,
                r#"{"type": "planet-in-nakshatra", "planet": "KETU", "nakshatras": ["MULA"], "padas": [2, 3]}"#,
                "KETU stands in the nakshatra MULA, in its 2nd or 3rd pada",
            ),
            (
                r#"{"type": "birth-on-sankranti"}"#,
                r#"{"type": "birth-on-sankranti", "windowHours": 12}"#,
                "the birth falls on a sankranti, within 12 hours of it",
            ),
            (
                r#"{"type": "birth-during-eclipse"}"#,
                r#"{"type": "birth-during-eclipse", "kind": "solar"}"#,
                "the birth falls in an eclipse of the Sun",
            ),
            (
                r#"{"type": "count-of", "then": {"type": "birth-by-day"}, "atLeast": 1}"#,
                r#"{"type": "count-of", "then": {"type": "birth-by-day"}, "atLeast": 1, "atMost": 3}"#,
                "at least 1 and at most 3 of the nine grahas meet: the birth fell by day",
            ),
            (
                r#"{"type": "count-in-houses", "planets": "any-malefic", "houses": [2], "atLeast": 1}"#,
                r#"{"type": "count-in-houses", "planets": "any-malefic", "houses": [2], "atLeast": 1, "except": [{"lordOf": 1}]}"#,
                "at least 1 malefic stands in the 2nd from LAGNA, not counting the lord of house 1",
            ),
            (
                r#"{"type": "count-in-houses", "planets": "any-malefic", "houses": [2], "atLeast": 1}"#,
                r#"{"type": "count-in-houses", "planets": "any-malefic", "houses": [2], "from": "MOON", "atLeast": 1}"#,
                "at least 1 malefic stands in the 2nd from MOON",
            ),
        ] {
            let bare = read(bare).to_string();
            let with = read(with).to_string();
            assert_ne!(bare, with, "a field is dropped from {with}");
            assert_eq!(with, changed);
        }
    }

    /// A combination inside a combination is parenthesised, so nothing about
    /// the shape of a rule has to be guessed from the commas.
    #[test]
    fn a_nested_combination_is_parenthesised() {
        let condition = read(
            r#"{"type": "and", "conditions": [
                {"type": "or", "conditions": [
                    {"type": "birth-by-day"},
                    {"type": "planet-combust", "planet": "MOON"}
                ]},
                {"type": "not", "condition": {"type": "planet-retrograde", "planet": "MARS"}}
            ]}"#,
        );
        assert_eq!(
            condition.to_string(),
            "(the birth fell by day or MOON is combust) \
             and (it is not the case that MARS is retrograde)"
        );
    }

    /// Everything a rule carries beyond its conditions appears in the
    /// passage: a severity or a threshold changed and not shown is the review
    /// this renderer exists to make possible, failing silently.
    #[test]
    fn a_rule_says_everything_it_carries() {
        let rule: Rule = serde_json::from_str(
            r#"{
                "key": "AN_EXAMPLE",
                "category": "planetary",
                "applicableTo": "milan",
                "source": { "text": "BPHS", "chapter": "33", "verse": "12", "rank": 1 },
                "conditions": [{ "type": "birth-by-day" }],
                "referenceConditions": [{
                    "reference": "lagna",
                    "label": "Mars in an afflicted house",
                    "weight": 2,
                    "conditions": [{ "type": "planet-in-house", "planet": "MARS", "houses": [1, 4] }]
                }],
                "cancellations": [
                    { "type": "planet-aspects-planet", "from": "JUPITER", "target": "MARS", "label": "Jupiter aspects Mars" },
                    { "type": "planet-retrograde", "planet": "MARS" }
                ],
                "severityRule": { "type": "count-based", "perOccurrence": 25, "cap": 100 },
                "fullCancellationThreshold": 2,
                "remedyKeys": ["MANGAL_SHANTI"],
                "outcomes": [
                    { "type": "effect", "text": "trouble in marriage" },
                    { "type": "life-class", "class": "medium" }
                ],
                "timing": "throughout",
                "customResultKey": "KUJA_DOSHA"
            }"#,
        )
        .unwrap();
        assert_eq!(
            rule.to_string(),
            "AN_EXAMPLE — planetary, a match. BPHS ch. 33 v. 12 (rank 1).\n\
             When:\n\
             \u{20} the birth fell by day\n\
             Found from any of:\n\
             \u{20} LAGNA, \"Mars in an afflicted house\", weight 2:\n\
             \u{20}   MARS stands in the 1st or 4th house\n\
             Cancelled by:\n\
             \u{20} \"Jupiter aspects Mars\": JUPITER aspects MARS\n\
             \u{20} MARS is retrograde\n\
             \u{20} 2 of them cancel it fully; fewer cancel it in part.\n\
             Severity: 25 for each place it is found from, at most 100.\n\
             Then: trouble in marriage and a medium life, 64 years.\n\
             Felt in every period.\n\
             Remedies: MANGAL_SHANTI.\n\
             Computed in code as KUJA_DOSHA: the language does not say it.\n"
        );
    }

    /// Every way a rule can say how grave it is, and every way it can say
    /// what follows, reads as words: four of the five severities and two of
    /// the three outcomes occur in no pack, so nothing else would read them.
    #[test]
    fn every_severity_and_every_outcome_reads() {
        for (json, expected) in [
            (r#"{"type": "fixed", "value": 80}"#, "80"),
            (
                r#"{"type": "house-weighted", "weights": {"1": 70, "7": 100}, "default": 50}"#,
                "by the house MARS stands in from each place it is found from \
                 — the 1st: 70 and the 7th: 100 — and 50 elsewhere",
            ),
            (
                r#"{"type": "house-weighted", "planet": "SATURN", "weights": {"1": 70}, "default": 50}"#,
                "by the house SATURN stands in from each place it is found \
                 from — the 1st: 70 — and 50 elsewhere",
            ),
            (
                r#"{"type": "planet-strength-inverse", "basePlanet": "SATURN", "max": 90, "min": 30}"#,
                "90 when SATURN is debilitated, 30 when it is exalted or in \
                 its own sign, halfway otherwise",
            ),
            (
                r#"{"type": "count-based", "perOccurrence": 25, "cap": 100}"#,
                "25 for each place it is found from, at most 100",
            ),
            (
                r#"{"type": "koot-shortfall", "full": 60, "perPointMissing": 2}"#,
                "60 with no koota score, less by 2 for each point regained",
            ),
        ] {
            let severity: Severity =
                serde_json::from_str(json).unwrap_or_else(|err| panic!("{json} reads: {err}"));
            assert_eq!(severity.to_string(), expected, "for {json}");
        }
        for (json, expected) in [
            (
                r#"{"type": "life-span", "count": 30, "unit": "days"}"#,
                "a life of 30 days",
            ),
            (
                r#"{"type": "life-span", "count": 24.5, "unit": "years"}"#,
                "a life of 24.5 years",
            ),
            (
                r#"{"type": "life-class", "class": "balarishta"}"#,
                "a death in infancy, 8 years",
            ),
            (
                r#"{"type": "life-class", "class": "unlimited"}"#,
                "an illimitable life",
            ),
            (
                r#"{"type": "effect", "text": "wealth and a kingdom"}"#,
                "wealth and a kingdom",
            ),
        ] {
            let outcome: Outcome =
                serde_json::from_str(json).unwrap_or_else(|err| panic!("{json} reads: {err}"));
            assert_eq!(outcome.to_string(), expected, "for {json}");
        }
    }

    /// Every rule the kernel ships renders, and no passage is empty.
    #[test]
    fn every_shipped_rule_renders() {
        let packs = [
            shipped::nabhasas(),
            shipped::arishtas(),
            shipped::gandantas(),
            shipped::readings(),
            shipped::computed_doshas(),
            shipped::computed_yogas(),
        ];
        let mut rendered = 0;
        for pack in packs {
            for rule in pack {
                let passage = rule.to_string();
                assert!(passage.starts_with(&rule.key), "{} has no header", rule.key);
                assert!(passage.ends_with('\n'), "{} has no last line", rule.key);
                rendered += 1;
            }
        }
        assert!(rendered > 900, "only {rendered} rules rendered");
    }
}

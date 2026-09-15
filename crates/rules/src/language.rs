//! The condition language: what a rule says, typed.
//!
//! Its shape is the recording engine's, so a rule written for that engine
//! reads here unchanged (`03-design/rules-engine.md`): a condition is an object
//! whose `type` names the predicate, fields in camel case. Reading is strict —
//! an unknown field, an unknown predicate, a house past twelve or a body no
//! rule can name is refused with its path, not skipped — because a rule that
//! quietly reads as something else is a yoga reported wrongly.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{
    CharaKaraka, Dignity, Graha, Karana, Nakshatra, Paksha, Rashi, Tithi, Vara, Yoga,
};

use crate::reference::{BodyRef, SignRef, Subject};
use crate::table::TableKey;

/// What a rule can name: one of the nine grahas, or the lagna.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Body {
    /// A graha, the Sun to Ketu.
    Graha(Graha),
    /// The lagna, a point and not a graha.
    Lagna,
}

impl Body {
    /// Every body a rule can name, in the order the kernel walks them: the Sun
    /// to Ketu, then the lagna.
    pub const ALL: [Body; 10] = [
        Body::Graha(Graha::Sun),
        Body::Graha(Graha::Moon),
        Body::Graha(Graha::Mars),
        Body::Graha(Graha::Mercury),
        Body::Graha(Graha::Jupiter),
        Body::Graha(Graha::Venus),
        Body::Graha(Graha::Saturn),
        Body::Graha(Graha::Rahu),
        Body::Graha(Graha::Ketu),
        Body::Lagna,
    ];

    /// The seven classical grahas, the Sun to Saturn.
    pub const SEVEN: [Body; 7] = [
        Body::Graha(Graha::Sun),
        Body::Graha(Graha::Moon),
        Body::Graha(Graha::Mars),
        Body::Graha(Graha::Mercury),
        Body::Graha(Graha::Jupiter),
        Body::Graha(Graha::Venus),
        Body::Graha(Graha::Saturn),
    ];

    /// Its place in [`Body::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Body::Graha(graha) => graha as usize,
            Body::Lagna => 9,
        }
    }

    /// Its key, as a rule spells it.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Body::Graha(graha) => graha.key(),
            Body::Lagna => "LAGNA",
        }
    }

    /// Whether it is Rahu or Ketu.
    #[must_use]
    pub const fn is_node(self) -> bool {
        matches!(self, Body::Graha(Graha::Rahu | Graha::Ketu))
    }
}

impl Serialize for Body {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.key())
    }
}

impl<'de> Deserialize<'de> for Body {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Body, D::Error> {
        Body::from_key(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl Body {
    /// The body a key names.
    ///
    /// # Errors
    ///
    /// A key that is not one of the nine grahas or `LAGNA`, named.
    pub fn from_key(key: &str) -> Result<Body, String> {
        if key == "LAGNA" {
            return Ok(Body::Lagna);
        }
        Graha::from_key(key)
            .filter(|graha| (*graha as usize) < 9)
            .map(Body::Graha)
            .ok_or_else(|| {
                format!(
                    "`{key}` is not a body a rule names: the nine grahas, SUN to KETU, or LAGNA"
                )
            })
    }
}

/// A house, 1 to 12.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct House(u8);

impl House {
    /// A house.
    ///
    /// # Errors
    ///
    /// A number outside 1 to 12, named.
    pub fn try_new(house: u8) -> Result<House, String> {
        if (1..=12).contains(&house) {
            Ok(House(house))
        } else {
            Err(format!("house {house} is not 1 to 12"))
        }
    }

    /// Its number, 1 to 12.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// The house `sign` is counted from `from`, whole signs, 1 to 12.
    #[must_use]
    pub const fn between(from: Rashi, sign: Rashi) -> House {
        House((sign as u8 + 12 - from as u8) % 12 + 1)
    }

    /// The four kendras.
    pub const KENDRAS: [House; 4] = [House(1), House(4), House(7), House(10)];
    /// The three trikonas.
    pub const TRIKONAS: [House; 3] = [House(1), House(5), House(9)];
}

impl TryFrom<u8> for House {
    type Error = String;

    fn try_from(house: u8) -> Result<House, String> {
        House::try_new(house)
    }
}

impl From<House> for u8 {
    fn from(house: House) -> u8 {
        house.0
    }
}

/// A chara karaka as a rule spells it: `AK`, `AmK`, `BK`, `MK`, `PK`, `GK`,
/// `DK` or `PiK`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Karaka(pub CharaKaraka);

/// Each karaka's abbreviation, in the catalogue's order.
const KARAKAS: [(&str, CharaKaraka); 8] = [
    ("AK", CharaKaraka::Atmakaraka),
    ("AmK", CharaKaraka::Amatyakaraka),
    ("BK", CharaKaraka::Bhratrikaraka),
    ("MK", CharaKaraka::Matrikaraka),
    ("PK", CharaKaraka::Putrakaraka),
    ("GK", CharaKaraka::Gnatikaraka),
    ("DK", CharaKaraka::Darakaraka),
    ("PiK", CharaKaraka::Pitrikaraka),
];

impl Serialize for Karaka {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for Karaka {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Karaka, D::Error> {
        Karaka::from_abbreviation(&String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl Karaka {
    /// Its abbreviation, as a rule writes it.
    #[must_use]
    pub fn abbreviation(self) -> &'static str {
        KARAKAS
            .iter()
            .find(|(_, k)| *k == self.0)
            .map_or("AK", |(a, _)| *a)
    }

    /// The karaka an abbreviation names.
    ///
    /// # Errors
    ///
    /// An abbreviation that is not one of the eight, named.
    pub fn from_abbreviation(abbreviation: &str) -> Result<Karaka, String> {
        KARAKAS
            .iter()
            .find(|(a, _)| *a == abbreviation)
            .map(|(_, k)| Karaka(*k))
            .ok_or_else(|| {
                format!(
                    "`{abbreviation}` is not a chara karaka: AK, AmK, BK, MK, PK, GK, DK or PiK"
                )
            })
    }
}

/// Which chara karaka scheme a rule reads: seven grahas, or eight with Rahu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum KarakaScheme {
    /// The Sun to Saturn.
    #[default]
    Seven,
    /// With Rahu.
    Eight,
}

impl TryFrom<u8> for KarakaScheme {
    type Error = String;

    fn try_from(scheme: u8) -> Result<KarakaScheme, String> {
        match scheme {
            7 => Ok(KarakaScheme::Seven),
            8 => Ok(KarakaScheme::Eight),
            _ => Err(format!("karaka scheme {scheme} is not 7 or 8")),
        }
    }
}

impl From<KarakaScheme> for u8 {
    fn from(scheme: KarakaScheme) -> u8 {
        match scheme {
            KarakaScheme::Seven => 7,
            KarakaScheme::Eight => 8,
        }
    }
}

const fn yes() -> bool {
    true
}

/// A condition: a predicate over the chart, or a combination of conditions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Condition {
    /// Every condition holds, tried in order until one does not.
    #[serde(alias = "dosha-and")]
    And {
        /// The conditions.
        conditions: Vec<Condition>,
    },
    /// Some condition holds, tried in order until one does.
    #[serde(alias = "dosha-or")]
    Or {
        /// The conditions.
        conditions: Vec<Condition>,
    },
    /// The condition does not hold.
    #[serde(alias = "dosha-not")]
    Not {
        /// The condition.
        condition: Box<Condition>,
    },
    /// The subject stands in one of the houses.
    PlanetInHouse {
        /// Who.
        planet: Subject,
        /// Where.
        houses: Vec<House>,
    },
    /// The body stands in one of the signs.
    PlanetInSign {
        /// Who.
        planet: SignRef,
        /// Where.
        signs: Vec<Rashi>,
    },
    /// The body has one of the dignities.
    PlanetDignity {
        /// Who.
        planet: BodyRef,
        /// Which.
        dignities: Vec<Dignity>,
    },
    /// The body stands in a kendra.
    PlanetInKendra {
        /// Who.
        planet: SignRef,
    },
    /// The body stands in a trikona.
    PlanetInTrikona {
        /// Who.
        planet: SignRef,
    },
    /// The body stands in a kendra, whole signs, from the reference.
    PlanetInKendraFrom {
        /// Who.
        planet: SignRef,
        /// From whom.
        reference: SignRef,
    },
    /// The lord of a house stands in a kendra.
    LordOfHouseInKendra {
        /// Whose lord.
        house_ruled: House,
    },
    /// The lord of a house stands in a house.
    LordOfHouseInHouse {
        /// Whose lord.
        house_ruled: House,
        /// Where.
        house_occupied: House,
    },
    /// The bodies are together: in one sign, or within an orb when the rule
    /// gives one.
    PlanetConjunct {
        /// Who.
        planets: Vec<BodyRef>,
        /// The orb, degrees, when the rule measures one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_orb: Option<f64>,
    },
    /// The subject stands in one of the houses, whole signs, from the
    /// reference.
    PlanetInHouseFrom {
        /// Who.
        planet: Subject,
        /// From whom.
        reference: SignRef,
        /// Where.
        houses: Vec<House>,
    },
    /// No graha but the excepted stands in the houses from the reference.
    NoPlanetInHousesFrom {
        /// From whom.
        reference: SignRef,
        /// Where.
        houses: Vec<House>,
        /// Who is not counted.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        except: Vec<Body>,
    },
    /// The lords of the two houses stand each in the other's house.
    MutualExchange {
        /// One house.
        house1: House,
        /// The other.
        house2: House,
    },
    /// The lords of the two houses share a sign, or are one graha.
    LordConjunctLord {
        /// One house.
        house1: House,
        /// The other.
        house2: House,
    },
    /// The seven classical grahas all stand between the nodes.
    AllPlanetsBetweenNodes,
    /// The bodies occupy exactly so many signs.
    OccupiedSignCount {
        /// Who.
        planets: Vec<SignRef>,
        /// How many signs.
        count: u8,
    },
    /// The seven classical grahas stand only in the houses, and by default
    /// fill every one of them.
    AllClassicalGrahasInHouses {
        /// Where.
        houses: Vec<House>,
        /// Whether every house must hold one.
        #[serde(default = "yes")]
        require_all_houses_filled: bool,
    },
    /// At least so many classical grahas share the anchor's sign.
    NGrahasConjunctWith {
        /// Whose sign.
        anchor: SignRef,
        /// How many, the anchor among them.
        min_count: u8,
    },
    /// The body holding a chara karaka stands in one of the houses.
    CharaKarakaInHouse {
        /// Which karaka.
        karaka: Karaka,
        /// Where.
        houses: Vec<House>,
        /// Which scheme.
        #[serde(default)]
        karaka_scheme: KarakaScheme,
    },
    /// The body is combust.
    PlanetCombust {
        /// Who.
        planet: BodyRef,
    },
    /// The body is retrograde.
    PlanetRetrograde {
        /// Who.
        planet: BodyRef,
    },
    /// One body aspects another by graha drishti, whole signs.
    PlanetAspectsPlanet {
        /// Who aspects.
        from: BodyRef,
        /// Who is aspected.
        target: SignRef,
    },
    /// A body aspects a house by graha drishti, whole signs.
    PlanetAspectsHouse {
        /// Who aspects.
        from: BodyRef,
        /// Which house.
        house_ruled: House,
    },
    /// A body stands in the degree of its sign that its row of a
    /// degrees-by-sign table gives, as [`Bhaga`](crate::Bhaga) counts a degree;
    /// never, when the table has no row for it.
    PlanetAtTableDegree {
        /// Who.
        planet: BodyRef,
        /// Which table.
        table: TableKey,
    },
    /// The reference stands in one of the signs a signs-by-tithi table gives
    /// the chart's tithi; never, when the chart carries no panchanga.
    PlanetInTableSign {
        /// Who.
        planet: SignRef,
        /// Which table.
        table: TableKey,
    },
    /// The lord of a house is debilitated. Like every predicate below that the
    /// recording engine's dosha evaluator reads itself, it adds no participant.
    LordOfHouseDebilitated {
        /// Whose lord.
        house_ruled: House,
    },
    /// The lord of a house is combust; the Sun, who cannot burn himself, never.
    LordOfHouseCombust {
        /// Whose lord.
        house_ruled: House,
    },
    /// The lord of a house is in its own sign, exalted or in its
    /// moolatrikona.
    LordOfHouseStrong {
        /// Whose lord.
        house_ruled: House,
    },
    /// The lord of a house is one of the bodies: who rules it, not where it
    /// stands.
    LordOfHouseIs {
        /// Whose lord.
        house_ruled: House,
        /// Which.
        planets: Vec<Body>,
    },
    /// The lord of a house shares a sign with a body.
    LordOfHouseConjunctPlanet {
        /// Whose lord.
        house_ruled: House,
        /// With whom.
        with_planet: Body,
    },
    /// The lagna rises in one of the signs.
    LagnaInSign {
        /// Which.
        signs: Vec<Rashi>,
    },
    /// A body stands in one of the houses and in one of the signs at once.
    PlanetInHouseAndSign {
        /// Who.
        planet: Body,
        /// Which houses.
        houses: Vec<House>,
        /// Which signs.
        signs: Vec<Rashi>,
    },
    /// A body stands within an orb of a gandanta junction: the end of Cancer,
    /// Scorpio or Pisces, or the start of Leo, Sagittarius or Aries.
    PlanetAtGandanta {
        /// Who.
        planet: Body,
        /// The orb, degrees; 3°20′ unless the rule gives one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        orb_degrees: Option<f64>,
    },
    /// The birth tithi is one of these.
    PanchangaTithi {
        /// Which.
        tithis: Vec<Tithi>,
    },
    /// The birth tithi is in this paksha.
    PanchangaPaksha {
        /// Which, `shukla` or `krishna`.
        #[serde(with = "paksha")]
        paksha: Paksha,
    },
    /// The birth weekday is one of these.
    PanchangaVara {
        /// Which.
        varas: Vec<Vara>,
    },
    /// The Moon's nakshatra at birth is one of these, in one of the padas when
    /// the rule names any.
    PanchangaNakshatra {
        /// Which.
        nakshatras: Vec<Nakshatra>,
        /// Which padas, 1 to 4; any when none.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        padas: Vec<Pada>,
    },
    /// The panchanga yoga at birth is one of these.
    PanchangaYoga {
        /// Which.
        yogas: Vec<Yoga>,
    },
    /// The karana at birth is one of these.
    PanchangaKarana {
        /// Which.
        karanas: Vec<Karana>,
    },
    /// The birth falls in an eclipse, of this kind or of any.
    BirthDuringEclipse {
        /// Which kind; any when unset.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<EclipseKind>,
    },
    /// The birth falls on a sankranti, as the chart's panchanga says.
    BirthOnSankranti {
        /// The window, hours either side, that the chart's flag was computed
        /// under; kept with the rule, read by whoever builds the chart.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window_hours: Option<f64>,
    },
}

/// A nakshatra's pada, 1 to 4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Pada(u8);

impl Pada {
    /// A pada.
    ///
    /// # Errors
    ///
    /// A number outside 1 to 4, named.
    pub fn try_new(pada: u8) -> Result<Pada, String> {
        if (1..=4).contains(&pada) {
            Ok(Pada(pada))
        } else {
            Err(format!("pada {pada} is not 1 to 4"))
        }
    }

    /// Its number, 1 to 4.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Pada {
    type Error = String;

    fn try_from(pada: u8) -> Result<Pada, String> {
        Pada::try_new(pada)
    }
}

impl From<Pada> for u8 {
    fn from(pada: Pada) -> u8 {
        pada.0
    }
}

/// Which eclipse.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EclipseKind {
    /// Of the Sun.
    Solar,
    /// Of the Moon.
    Lunar,
    /// Either.
    Any,
}

/// A paksha as the engine writes it, in lower case.
mod paksha {
    use serde::{Deserialize, Deserializer, Serializer};
    use teistro_core::catalogue::Paksha;

    #[allow(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde passes a field by reference"
    )]
    pub(super) fn serialize<S: Serializer>(
        paksha: &Paksha,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&paksha.key().to_ascii_lowercase())
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Paksha, D::Error> {
        let key = String::deserialize(deserializer)?;
        Paksha::from_key(&key.to_ascii_uppercase()).ok_or_else(|| {
            serde::de::Error::custom(format!("`{key}` is not a paksha: shukla or krishna"))
        })
    }
}

impl Condition {
    /// The predicate's name, as a rule spells its `type`.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Condition::And { .. } => "and",
            Condition::Or { .. } => "or",
            Condition::Not { .. } => "not",
            Condition::PlanetInHouse { .. } => "planet-in-house",
            Condition::PlanetInSign { .. } => "planet-in-sign",
            Condition::PlanetDignity { .. } => "planet-dignity",
            Condition::PlanetInKendra { .. } => "planet-in-kendra",
            Condition::PlanetInTrikona { .. } => "planet-in-trikona",
            Condition::PlanetInKendraFrom { .. } => "planet-in-kendra-from",
            Condition::LordOfHouseInKendra { .. } => "lord-of-house-in-kendra",
            Condition::LordOfHouseInHouse { .. } => "lord-of-house-in-house",
            Condition::PlanetConjunct { .. } => "planet-conjunct",
            Condition::PlanetInHouseFrom { .. } => "planet-in-house-from",
            Condition::NoPlanetInHousesFrom { .. } => "no-planet-in-houses-from",
            Condition::MutualExchange { .. } => "mutual-exchange",
            Condition::LordConjunctLord { .. } => "lord-conjunct-lord",
            Condition::AllPlanetsBetweenNodes => "all-planets-between-nodes",
            Condition::OccupiedSignCount { .. } => "occupied-sign-count",
            Condition::AllClassicalGrahasInHouses { .. } => "all-classical-grahas-in-houses",
            Condition::NGrahasConjunctWith { .. } => "n-grahas-conjunct-with",
            Condition::CharaKarakaInHouse { .. } => "chara-karaka-in-house",
            Condition::PlanetCombust { .. } => "planet-combust",
            Condition::PlanetRetrograde { .. } => "planet-retrograde",
            Condition::PlanetAspectsPlanet { .. } => "planet-aspects-planet",
            Condition::PlanetAspectsHouse { .. } => "planet-aspects-house",
            Condition::PlanetAtTableDegree { .. } => "planet-at-table-degree",
            Condition::PlanetInTableSign { .. } => "planet-in-table-sign",
            Condition::LordOfHouseDebilitated { .. } => "lord-of-house-debilitated",
            Condition::LordOfHouseCombust { .. } => "lord-of-house-combust",
            Condition::LordOfHouseStrong { .. } => "lord-of-house-strong",
            Condition::LordOfHouseIs { .. } => "lord-of-house-is",
            Condition::LordOfHouseConjunctPlanet { .. } => "lord-of-house-conjunct-planet",
            Condition::LagnaInSign { .. } => "lagna-in-sign",
            Condition::PlanetInHouseAndSign { .. } => "planet-in-house-and-sign",
            Condition::PlanetAtGandanta { .. } => "planet-at-gandanta",
            Condition::PanchangaTithi { .. } => "panchanga-tithi",
            Condition::PanchangaPaksha { .. } => "panchanga-paksha",
            Condition::PanchangaVara { .. } => "panchanga-vara",
            Condition::PanchangaNakshatra { .. } => "panchanga-nakshatra",
            Condition::PanchangaYoga { .. } => "panchanga-yoga",
            Condition::PanchangaKarana { .. } => "panchanga-karana",
            Condition::BirthDuringEclipse { .. } => "birth-during-eclipse",
            Condition::BirthOnSankranti { .. } => "birth-on-sankranti",
        }
    }

    /// The conditions a combinator combines, and none for a predicate.
    #[must_use]
    pub fn children(&self) -> &[Condition] {
        match self {
            Condition::And { conditions } | Condition::Or { conditions } => conditions,
            Condition::Not { condition } => core::slice::from_ref(condition),
            _ => &[],
        }
    }

    /// Whether this condition itself reads the chart's panchanga.
    #[must_use]
    pub const fn reads_panchanga(&self) -> bool {
        matches!(
            self,
            Condition::PlanetInTableSign { .. }
                | Condition::PanchangaTithi { .. }
                | Condition::PanchangaPaksha { .. }
                | Condition::PanchangaVara { .. }
                | Condition::PanchangaNakshatra { .. }
                | Condition::PanchangaYoga { .. }
                | Condition::PanchangaKarana { .. }
                | Condition::BirthDuringEclipse { .. }
                | Condition::BirthOnSankranti { .. }
        )
    }

    /// This condition and every one inside it, depth first.
    pub fn walk(&self) -> impl Iterator<Item = &Condition> {
        let mut stack = vec![self];
        core::iter::from_fn(move || {
            let next = stack.pop()?;
            stack.extend(next.children().iter().rev());
            Some(next)
        })
    }
}

/// Where a rule or a table comes from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// The text.
    pub text: String,
    /// Its chapter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapter: Option<String>,
    /// Its verse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verse: Option<String>,
    /// A note on the reading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Source {
    /// A text, with no chapter, verse or note.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Source {
        Source {
            text: text.into(),
            chapter: None,
            verse: None,
            note: None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests unwrap what they read")]

    use super::*;

    fn condition(json: &str) -> Result<Condition, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn a_condition_is_read_strictly_and_names_what_it_refuses() {
        assert!(condition(r#"{"type": "planet-in-kendra", "planet": "JUPITER"}"#).is_ok());
        let refusals = [
            (
                r#"{"type": "planet-in-kendra", "planet": "JUPITER", "extra": 1}"#,
                "extra",
            ),
            (
                r#"{"type": "planet-in-kendra", "planet": "URANUS"}"#,
                "URANUS",
            ),
            (
                r#"{"type": "planet-in-house", "planet": "SUN", "houses": [13]}"#,
                "13",
            ),
            (
                r#"{"type": "chara-karaka-in-house", "karaka": "XK", "houses": [1]}"#,
                "XK",
            ),
            (
                r#"{"type": "planet-everywhere", "planet": "SUN"}"#,
                "planet-everywhere",
            ),
            (
                r#"{"type": "lord-of-house-in-house", "houseRuled": 1}"#,
                "houseOccupied",
            ),
        ];
        for (json, named) in refusals {
            let error = condition(json).unwrap_err().to_string();
            assert!(error.contains(named), "{json}: {error}");
        }
    }

    #[test]
    fn subjects_bodies_karakas_and_defaults_read_as_a_rule_writes_them() {
        let c = condition(
            r#"{"type": "planet-in-house-from", "planet": "any-benefic", "reference": "LAGNA", "houses": [1, 7]}"#,
        )
        .unwrap();
        assert_eq!(
            c,
            Condition::PlanetInHouseFrom {
                planet: Subject::AnyBenefic,
                reference: Body::Lagna.into(),
                houses: vec![House::try_new(1).unwrap(), House::try_new(7).unwrap()],
            }
        );
        let c = condition(r#"{"type": "all-classical-grahas-in-houses", "houses": [1]}"#).unwrap();
        assert!(matches!(
            c,
            Condition::AllClassicalGrahasInHouses {
                require_all_houses_filled: true,
                ..
            }
        ));
        let c = condition(r#"{"type": "chara-karaka-in-house", "karaka": "AmK", "houses": [10], "karakaScheme": 8}"#)
            .unwrap();
        assert!(matches!(
            c,
            Condition::CharaKarakaInHouse {
                karaka: Karaka(CharaKaraka::Amatyakaraka),
                karaka_scheme: KarakaScheme::Eight,
                ..
            }
        ));
        assert_eq!(c.kind(), "chara-karaka-in-house");
        let back: Condition = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
        assert_eq!(back, c);
        assert_eq!(House::between(Rashi::Capricorn, Rashi::Aries).get(), 4);
    }

    #[test]
    fn a_walk_visits_every_condition_depth_first_in_written_order() {
        let c = condition(
            r#"{"type": "and", "conditions": [
                {"type": "not", "condition": {"type": "planet-retrograde", "planet": "SATURN"}},
                {"type": "or", "conditions": [
                    {"type": "planet-combust", "planet": "MERCURY"},
                    {"type": "all-planets-between-nodes"}
                ]}
            ]}"#,
        )
        .unwrap();
        let kinds: Vec<&str> = c.walk().map(Condition::kind).collect();
        assert_eq!(
            kinds,
            [
                "and",
                "not",
                "planet-retrograde",
                "or",
                "planet-combust",
                "all-planets-between-nodes"
            ]
        );
        assert_eq!(c.children().len(), 2);
        assert!(c.walk().skip(1).all(|sub| sub.walk().count() <= 3));
    }
}

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
    CharaKaraka, Dignity, Graha, Karana, Nakshatra, Paksha, Rashi, Tithi, Vara, Varga, Yoga,
};

use crate::chart::Limb;
use crate::reference::{BodyRef, BodySubject, Class, SignRef, Subject};
use crate::table::TableKey;

/// What a rule can name: one of the nine grahas, or the lagna.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::Body;
///
/// // Written as its key.
/// assert_eq!(serde_json::to_string(&Body::Graha(Graha::Saturn))?, r#""SATURN""#);
/// assert_eq!(serde_json::from_str::<Body>(r#""LAGNA""#)?, Body::Lagna);
/// # Ok::<(), serde_json::Error>(())
/// ```
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
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    ///
    /// assert_eq!(Body::ALL[0], Body::Graha(Graha::Sun));
    /// assert_eq!(Body::ALL[9], Body::Lagna);
    /// assert!(Body::ALL.iter().enumerate().all(|(at, body)| body.index() == at));
    /// ```
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

    /// The nine grahas, as a list a rule may leave out.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    ///
    /// let nine = Body::nine();
    /// assert_eq!(nine.len(), 9);
    /// assert!(!nine.contains(&Body::Lagna));
    /// ```
    #[must_use]
    pub fn nine() -> Vec<Body> {
        Body::ALL[..9].to_vec()
    }

    /// Whether a list is the nine grahas.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    ///
    /// assert!(Body::is_nine(&Body::nine()));
    /// assert!(!Body::is_nine(&Body::SEVEN));
    /// ```
    #[must_use]
    pub fn is_nine(planets: &[Body]) -> bool {
        planets == &Body::ALL[..9]
    }

    /// The seven classical grahas, the Sun to Saturn.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    ///
    /// // Neither node.
    /// assert!(Body::SEVEN.iter().all(|body| !body.is_node()));
    /// assert_eq!(Body::SEVEN[6], Body::Graha(Graha::Saturn));
    /// ```
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
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    ///
    /// assert_eq!(Body::Graha(Graha::Moon).index(), 1);
    /// assert_eq!(Body::Lagna.index(), 9);
    /// ```
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Body::Graha(graha) => graha as usize,
            Body::Lagna => 9,
        }
    }

    /// Its key, as a rule spells it.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    ///
    /// assert_eq!(Body::Graha(Graha::Rahu).key(), "RAHU");
    /// assert_eq!(Body::Lagna.key(), "LAGNA");
    /// ```
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Body::Graha(graha) => graha.key(),
            Body::Lagna => "LAGNA",
        }
    }

    /// Whether it is Rahu or Ketu.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    ///
    /// assert!(Body::Graha(Graha::Ketu).is_node());
    /// assert!(!Body::Graha(Graha::Saturn).is_node() && !Body::Lagna.is_node());
    /// ```
    #[must_use]
    pub const fn is_node(self) -> bool {
        matches!(self, Body::Graha(Graha::Rahu | Graha::Ketu))
    }
}

/// A graha's key or `LAGNA`, as the reader below takes it.
#[cfg(feature = "schema")]
impl schemars::JsonSchema for Body {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("RuleBody")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "description": "A graha's key (`SUN`, `graha.SUN`, …) or `LAGNA`.",
        })
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
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    ///
    /// assert_eq!(Body::from_key("VENUS")?, Body::Graha(Graha::Venus));
    /// assert_eq!(Body::from_key("LAGNA")?, Body::Lagna);
    /// // The outer planets are not bodies a rule names.
    /// assert!(Body::from_key("URANUS").is_err());
    /// # Ok::<(), String>(())
    /// ```
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

// A house, 1 to 12, lives in `teistro_core` because four crates need it
// and none of them may depend on the others. It is re-exported here so
// that every `crate::language::House` in this crate still reads as the
// rule language's own word for it.
pub use teistro_core::house::House;

/// A chara karaka as a rule spells it: `AK`, `AmK`, `BK`, `MK`, `PK`, `GK`,
/// `DK` or `PiK`.
///
/// ```
/// use teistro_core::catalogue::CharaKaraka;
/// use teistro_rules::Karaka;
///
/// let darakaraka: Karaka = serde_json::from_str(r#""DK""#)?;
/// assert_eq!(darakaraka, Karaka(CharaKaraka::Darakaraka));
/// assert_eq!(serde_json::to_string(&darakaraka)?, r#""DK""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
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

/// A chara karaka by its abbreviation.
#[cfg(feature = "schema")]
impl schemars::JsonSchema for Karaka {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Karaka")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let abbreviations: Vec<&str> = KARAKAS.iter().map(|(a, _)| *a).collect();
        schemars::json_schema!({
            "type": "string",
            "enum": abbreviations,
            "description": "A chara karaka, by its abbreviation.",
        })
    }
}

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
    ///
    /// ```
    /// use teistro_core::catalogue::CharaKaraka;
    /// use teistro_rules::Karaka;
    ///
    /// assert_eq!(Karaka(CharaKaraka::Amatyakaraka).abbreviation(), "AmK");
    /// assert_eq!(Karaka(CharaKaraka::Pitrikaraka).abbreviation(), "PiK");
    /// ```
    #[must_use]
    pub fn abbreviation(self) -> &'static str {
        KARAKAS
            .iter()
            .find(|(_, k)| *k == self.0)
            .map_or("AK", |(a, _)| *a)
    }

    /// The karaka an abbreviation names.
    ///
    /// ```
    /// use teistro_core::catalogue::CharaKaraka;
    /// use teistro_rules::Karaka;
    ///
    /// assert_eq!(Karaka::from_abbreviation("AK")?, Karaka(CharaKaraka::Atmakaraka));
    /// // Spelt as a rule writes it, case and all.
    /// assert!(Karaka::from_abbreviation("ak").is_err());
    /// # Ok::<(), String>(())
    /// ```
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
///
/// ```
/// use teistro_rules::KarakaScheme;
///
/// // Written as its number of karakas.
/// assert_eq!(serde_json::from_str::<KarakaScheme>("8")?, KarakaScheme::Eight);
/// assert_eq!(serde_json::to_string(&KarakaScheme::default())?, "7");
/// assert!(serde_json::from_str::<KarakaScheme>("9").is_err());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

/// The lagna's own sign, which a house is counted from unless a rule says.
fn lagna() -> SignRef {
    SignRef::Of(BodyRef::Body(Body::Lagna))
}

fn is_lagna(from: &SignRef) -> bool {
    *from == lagna()
}

/// Every kind of condition the language has, as a rule spells its `type`,
/// in the order [`Condition::kind`] answers them.
///
/// A generator that walks the language — the prose pass reports one rendering
/// for each kind — needs the list, and a list a reader keeps privately is a
/// list that goes stale. `check-lints`' `every-predicate-is-listed` holds this
/// one against the arms of [`Condition::kind`], both ways.
///
/// ```
/// use teistro_rules::language::KINDS;
///
/// assert_eq!(KINDS.len(), 67);
/// assert!(KINDS.contains(&"planet-in-kendra"));
/// ```
pub const KINDS: [&str; 67] = [
    "and",
    "or",
    "not",
    "planet-in-house",
    "planet-in-sign",
    "planet-dignity",
    "planet-in-kendra",
    "planet-in-trikona",
    "planet-in-kendra-from",
    "lord-of-house-in-kendra",
    "lord-of-house-in-house",
    "planet-conjunct",
    "planet-in-house-from",
    "no-planet-in-houses-from",
    "mutual-exchange",
    "lord-conjunct-lord",
    "all-planets-between-nodes",
    "occupied-sign-count",
    "all-classical-grahas-in-houses",
    "n-grahas-conjunct-with",
    "chara-karaka-in-house",
    "planet-combust",
    "planet-retrograde",
    "planet-aspects-planet",
    "planet-aspects-house",
    "planet-at-table-degree",
    "planet-in-table-sign",
    "lord-of-house-debilitated",
    "lord-of-house-combust",
    "lord-of-house-strong",
    "lord-of-house-is",
    "lord-of-house-conjunct-planet",
    "lagna-in-sign",
    "planet-in-house-and-sign",
    "planet-at-gandanta",
    "planet-in-degrees",
    "panchanga-tithi",
    "panchanga-paksha",
    "panchanga-vara",
    "panchanga-nakshatra",
    "panchanga-yoga",
    "panchanga-karana",
    "birth-during-eclipse",
    "birth-on-sankranti",
    "transit-in-house",
    "planet-in-nakshatra",
    "same-nakshatra",
    "planet-strong",
    "planet-weak",
    "planet-stronger-than",
    "rashi-aspects",
    "argala",
    "vipareeta-argala",
    "for-any",
    "count-of",
    "n-of",
    "in-varga",
    "count-in-houses",
    "count-aspecting",
    "rule",
    "at-limb-edge",
    "limb-part",
    "birth-by-day",
    "same-sign",
    "same-body",
    "planet-is",
    "natural-relation",
];

/// A condition: a predicate over the chart, or a combination of conditions.
///
/// ```
/// use teistro_rules::Condition;
///
/// let either: Condition = serde_json::from_str(r#"{"type": "or", "conditions": [{"type": "planet-strong", "planet": "JUPITER"}, {"type": "birth-by-day"}]}"#)?;
/// assert_eq!(either.kind(), "or");
/// assert_eq!(either.children().len(), 2);
/// // A field a condition does not have is refused.
/// assert!(serde_json::from_str::<Condition>(r#"{"type": "planet-strong", "planet": "JUPITER", "strong": true}"#).is_err());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// The seven classical grahas all stand between the nodes, on the side the
    /// rule names, or on whichever side [`NodeSides`](crate::NodeSides) reads
    /// when it names none.
    AllPlanetsBetweenNodes {
        /// Whose side they stand on.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        side: Option<NodeSide>,
    },
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
        /// Who aspects: a body, or whichever benefic or malefic does.
        from: BodySubject,
        /// Who is aspected.
        target: SignRef,
    },
    /// A body aspects a house by graha drishti, whole signs.
    PlanetAspectsHouse {
        /// Who aspects: a body, or whichever benefic or malefic does.
        from: BodySubject,
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
    /// A body stands between two degrees of its sign, the first counted in and
    /// the last counted out: the last navamsa of a sign is 26.6667 to 30.
    PlanetInDegrees {
        /// Who.
        planet: BodyRef,
        /// From, degrees into the sign.
        from: f64,
        /// To, degrees into the sign.
        to: f64,
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
        #[cfg_attr(feature = "schema", schemars(with = "String"))]
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
    /// Some one of the bodies meets the condition, which names it `SELF`. Every
    /// body is tried, and a rule's participants take each that met it, not the
    /// bodies the inner conditions consulted.
    ForAny {
        /// Which bodies, the nine grahas unless the rule says.
        #[serde(default = "Body::nine", skip_serializing_if = "Body::is_nine")]
        planets: Vec<Body>,
        /// What one of them must meet.
        then: Box<Condition>,
    },
    /// So many of the bodies meet the condition, which names each `SELF` in
    /// turn: at least `atLeast`, and at most `atMost` when the rule gives one.
    /// "Four or more planets aspect the Moon" and "one or two or three planets
    /// in exaltation" (BPHS ch. 39 vv. 42 and 44) are counts over a condition
    /// no single predicate carries. `for-any` is this with at least one. Every
    /// body that met it takes part when the count holds.
    CountOf {
        /// Which bodies, the nine grahas unless the rule says.
        #[serde(default = "Body::nine", skip_serializing_if = "Body::is_nine")]
        planets: Vec<Body>,
        /// What each counted must meet.
        then: Box<Condition>,
        /// How many there must be at least.
        at_least: u8,
        /// How many there may be at most.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at_most: Option<u8>,
    },
    /// So many of the conditions hold: at least `atLeast`, and at most
    /// `atMost` when the rule gives one. A graha "in five of its own vargas"
    /// (Jataka Parijata ch. 4 vv. 102 to 104, the Vaiseshikamsha names) is a
    /// count over ten divisions no single predicate reads. `or` is this with
    /// at least one and `and` with every one. Every condition tried gives its
    /// bodies as `or`'s do: the held ones, or all under `EVERY_HELD`.
    NOf {
        /// The conditions counted.
        conditions: Vec<Condition>,
        /// How many must hold at least.
        at_least: u8,
        /// How many may hold at most.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at_most: Option<u8>,
    },
    /// The condition, read in a divisional chart: every body in its sign
    /// there, its houses counted whole-sign from that chart's lagna and its
    /// dignity from that sign. A chart the evaluator was not given makes it
    /// false, and a condition that reads a longitude is refused inside one.
    InVarga {
        /// Which division.
        varga: Varga,
        /// What must hold in it.
        condition: Box<Condition>,
    },
    /// So many bodies aspect the reference by graha drishti.
    CountAspecting {
        /// Who is counted: a body, or every benefic or every malefic.
        planets: BodySubject,
        /// Whom they aspect.
        target: SignRef,
        /// How many there must be at least.
        at_least: u8,
    },
    /// Another rule holds. The rule is named by key, and an evaluator reads it
    /// from the set it was given; a key the set does not hold never holds.
    #[serde(rename = "rule")]
    RuleHolds {
        /// Which rule.
        key: String,
    },
    /// So many bodies stand in the houses, counted from the reference.
    CountInHouses {
        /// Who is counted: a body, or every benefic or every malefic.
        planets: BodySubject,
        /// Which houses.
        houses: Vec<House>,
        /// What they are counted from; the lagna unless the rule says.
        #[serde(default = "lagna", skip_serializing_if = "is_lagna")]
        from: SignRef,
        /// How many there must be at least.
        at_least: u8,
        /// Who is not counted even when of that kind and standing there: the
        /// graha a verse says the others join, so "the lagna lord joined by a
        /// maraka" does not count the lord when it is a maraka itself.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        except: Vec<BodyRef>,
    },
    /// Two references stand in one sign.
    SameSign {
        /// One.
        of: SignRef,
        /// The other.
        #[serde(rename = "as")]
        as_sign: SignRef,
    },
    /// The body belongs to a class of grahas under the readings: a benefic, a
    /// malefic or a maraka. What a verse names by nature and then narrows,
    /// "a malefic, excepting the lords of the 10th and 9th" (BPHS ch. 42
    /// v. 8), binds a graha with `for-any` and asks this of it.
    PlanetIs {
        /// Who.
        planet: BodyRef,
        /// Of which class.
        class: Class,
    },
    /// How one body regards another by natural relationship (BPHS ch. 3's
    /// naisargika maitri, as the catalogue lists it): a friend, a neutral or
    /// an enemy. The relationship is not symmetric — Mercury counts the Sun a
    /// friend and the Sun counts Mercury neutral — so it is `of`'s regard for
    /// `to`, as "the ascendant lord friendly to the Sun" (BPHS ch. 43 vv. 71 to
    /// 73) is the lord's. A body is none of these to itself, and the lagna to
    /// anything.
    NaturalRelation {
        /// Whose regard.
        of: BodyRef,
        /// For whom.
        to: BodyRef,
        /// Which relations meet it.
        relations: Vec<Relation>,
    },
    /// Two references resolve to one body.
    SameBody {
        /// One.
        of: BodyRef,
        /// The other.
        #[serde(rename = "as")]
        as_body: BodyRef,
    },
    /// The birth stands within so many ghatikas of a limb's edge: the first
    /// ghatikas of it, or the last. BPHS ch. 92 measures every gandanta this
    /// way, and ch. 9 v. 14 each twilight as the night's edge; a limb the
    /// chart does not measure never holds.
    AtLimbEdge {
        /// Which limb.
        limb: Limb,
        /// Which end of it.
        edge: Edge,
        /// How many ghatikas of 24 minutes.
        ghatikas: f64,
    },
    /// The birth falls in one of a limb's equal parts, counted from its
    /// start: BPHS ch. 87 divides Krishna Chaturdashi into six and reads
    /// each part apart. The parts are of the limb's own span, so a long
    /// tithi has long parts; a limb the chart does not measure never holds.
    LimbPart {
        /// Which limb.
        limb: Limb,
        /// Into how many equal parts it is divided, at least two.
        of: u8,
        /// Which of them, the first being 1.
        parts: Vec<u8>,
    },
    /// The birth fell by day, between sunrise and sunset; a chart that does
    /// not say never holds it, and `not` of it is not "by night" there.
    BirthByDay,
    /// The body stands in one of these nakshatras, in one of the padas when
    /// the rule names any. It is read from the body's own sidereal longitude,
    /// 13°20′ to a nakshatra and a quarter of that to a pada, where
    /// `panchanga-nakshatra` reads the one the chart recorded for the Moon.
    PlanetInNakshatra {
        /// Who.
        planet: BodyRef,
        /// Which.
        nakshatras: Vec<Nakshatra>,
        /// Which padas, 1 to 4; any when none.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        padas: Vec<Pada>,
    },
    /// Two bodies stand in one nakshatra: "the birth star identical with the
    /// one in which Ketu rises" (Saravali ch. 10 v. 12).
    SameNakshatra {
        /// One.
        of: BodyRef,
        /// The other.
        #[serde(rename = "as")]
        as_body: BodyRef,
    },
    /// The body reaches what its measure requires of it: "the ascendant lord
    /// is strong" (BPHS ch. 36 vv. 9 to 28). A chart that carries no strength
    /// never holds it.
    PlanetStrong {
        /// Who.
        planet: BodyRef,
    },
    /// The body falls short of what is required of it. A chart that carries no
    /// strength never holds this either, so it is not the `not` of
    /// `planet-strong`: on a silent chart both are false, which is the honest
    /// answer to both questions.
    PlanetWeak {
        /// Who.
        planet: BodyRef,
    },
    /// One body's strength exceeds another's.
    PlanetStrongerThan {
        /// Who.
        planet: BodyRef,
        /// Than whom.
        than: BodyRef,
    },
    /// One reference aspects another by rashi drishti (BPHS ch. 26 vv. 1 to
    /// 3): a movable sign aspects the three fixed signs but the one next to
    /// it, a fixed sign the three movable but the one before it, and a dual
    /// sign the other three dual signs. A body lends the aspect of the sign it
    /// stands in, so naming a body names its sign.
    RashiAspects {
        /// Who aspects: a sign, a body's sign, or whichever benefic's or
        /// malefic's sign does.
        from: Subject,
        /// What is aspected.
        target: SignRef,
    },
    /// An intervention stands on a reference (BPHS ch. 31 vv. 2 to 9): the
    /// grahas in the intervening house outnumber those in the house that
    /// obstructs it, and there is at least one of them. The nine are counted;
    /// from a node the houses are counted backwards, the nodes moving so.
    Argala {
        /// What is intervened on: a sign, or the sign a body stands in.
        on: SignRef,
        /// Which intervention, each with the house that obstructs it.
        place: ArgalaPlace,
    },
    /// Three or more malefics stand in the third from a reference, which the
    /// same verses call a contrary intervention, harmless and very
    /// favourable; nothing obstructs it.
    VipareetaArgala {
        /// What is intervened on.
        on: SignRef,
    },
    /// The birth falls on a sankranti: on the day, sunrise to sunrise, the Sun
    /// enters a sign, or within so many hours of the entry when the rule
    /// says. BPHS ch. 90 names each sankranti by its day of the week.
    BirthOnSankranti {
        /// How many hours either side of the entry; the whole day when unset.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window_hours: Option<f64>,
    },
    /// A graha in transit stands in one of the houses counted from a natal
    /// sign: Saturn in the twelfth, first or second from the natal Moon is
    /// sade sati, and gochar is counted from the Moon (Phaladeepika ch. 26
    /// v. 1). It reads the sky at an instant an evaluator was given
    /// ([`Evaluator::with_transits`](crate::Evaluator::with_transits)) and
    /// never holds without one; the transiting graha is not a natal body,
    /// so it adds no participant.
    TransitInHouse {
        /// The graha in transit.
        planet: Graha,
        /// Which houses, counted from `from`.
        houses: Vec<House>,
        /// The natal sign counted from.
        from: SignRef,
    },
}

/// A natural relationship between two grahas.
///
/// ```
/// use teistro_rules::Relation;
///
/// assert_eq!(serde_json::from_str::<Relation>(r#""enemy""#)?, Relation::Enemy);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    /// A friend.
    Friend,
    /// Neither friend nor enemy.
    Neutral,
    /// An enemy.
    Enemy,
}

impl Relation {
    /// How `of` regards `to` by the catalogue's natural relationships; none
    /// for a graha and itself, or one the catalogue relates to nothing.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Relation;
    ///
    /// // The Sun's friends, neutral and enemies (BPHS ch. 3).
    /// assert_eq!(Relation::between(Graha::Sun, Graha::Moon), Some(Relation::Friend));
    /// assert_eq!(Relation::between(Graha::Sun, Graha::Mercury), Some(Relation::Neutral));
    /// assert_eq!(Relation::between(Graha::Sun, Graha::Saturn), Some(Relation::Enemy));
    /// assert_eq!(Relation::between(Graha::Sun, Graha::Sun), None);
    /// ```
    #[must_use]
    pub fn between(of: Graha, to: Graha) -> Option<Relation> {
        let attributes = of.attributes();
        if attributes.friends.contains(&to) {
            Some(Relation::Friend)
        } else if attributes.neutrals.contains(&to) {
            Some(Relation::Neutral)
        } else if attributes.enemies.contains(&to) {
            Some(Relation::Enemy)
        } else {
            None
        }
    }
}

/// Where an intervention comes from, and what obstructs it (BPHS ch. 31
/// v. 9's table: 4 2 11 5 over 10 12 3 9).
///
/// ```
/// use teistro_rules::ArgalaPlace;
///
/// assert_eq!(serde_json::from_str::<ArgalaPlace>(r#""eleventh""#)?, ArgalaPlace::Eleventh);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ArgalaPlace {
    /// The second, obstructed from the twelfth.
    Second,
    /// The fourth, obstructed from the tenth.
    Fourth,
    /// The fifth, obstructed from the ninth.
    Fifth,
    /// The eleventh, obstructed from the third.
    Eleventh,
}

impl ArgalaPlace {
    /// The house it intervenes from, and the house that obstructs it.
    ///
    /// ```
    /// use teistro_rules::ArgalaPlace;
    ///
    /// // From the fourth, obstructed from the tenth.
    /// assert_eq!(ArgalaPlace::Fourth.houses(), (4, 10));
    /// assert_eq!(ArgalaPlace::Eleventh.houses(), (11, 3));
    /// ```
    #[must_use]
    pub const fn houses(self) -> (u8, u8) {
        match self {
            ArgalaPlace::Second => (2, 12),
            ArgalaPlace::Fourth => (4, 10),
            ArgalaPlace::Fifth => (5, 9),
            ArgalaPlace::Eleventh => (11, 3),
        }
    }
}

/// How `SELF`, the body a `for-any` binds, is written.
///
/// ```
/// use teistro_rules::Condition;
///
/// use teistro_rules::language::SELF;
///
/// let each: Condition = serde_json::from_str(r#"{"type": "for-any", "then": {"type": "planet-in-kendra", "planet": "SELF"}}"#)?;
/// assert!(each.children()[0].names_self());
/// assert_eq!(SELF, "SELF");
/// # Ok::<(), serde_json::Error>(())
/// ```
pub const SELF: &str = "SELF";

/// Which node's side the seven stand on.
///
/// ```
/// use teistro_rules::NodeSide;
///
/// assert_eq!(serde_json::from_str::<NodeSide>(r#""ketu""#)?, NodeSide::Ketu);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum NodeSide {
    /// Between Rahu and Ketu in the zodiac's own direction: the ascending arc.
    Rahu,
    /// Between Ketu and Rahu: the descending arc.
    Ketu,
}

/// A nakshatra's pada, 1 to 4.
///
/// ```
/// use teistro_rules::Pada;
///
/// // Written as its number.
/// assert_eq!(serde_json::from_str::<Pada>("3")?.get(), 3);
/// assert!(serde_json::from_str::<Pada>("5").is_err());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(try_from = "u8", into = "u8")]
pub struct Pada(u8);

impl Pada {
    /// A pada.
    ///
    /// ```
    /// use teistro_rules::Pada;
    ///
    /// assert!(Pada::try_new(4).is_ok());
    /// assert_eq!(Pada::try_new(0).unwrap_err(), "pada 0 is not 1 to 4");
    /// ```
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
    ///
    /// ```
    /// use teistro_rules::Pada;
    ///
    /// assert_eq!(Pada::try_new(2)?.get(), 2);
    /// # Ok::<(), String>(())
    /// ```
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

/// Which end of a limb a condition measures.
///
/// ```
/// use teistro_rules::language::Edge;
///
/// assert_eq!(serde_json::from_str::<Edge>(r#""last""#)?, Edge::Last);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Edge {
    /// Its beginning: so many ghatikas after it started.
    First,
    /// Its end: so many ghatikas before it ends.
    Last,
}

/// Which eclipse.
///
/// ```
/// use teistro_rules::EclipseKind;
///
/// assert_eq!(serde_json::from_str::<EclipseKind>(r#""any""#)?, EclipseKind::Any);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let strong: Condition = serde_json::from_str(r#"{"type": "planet-strong", "planet": "JUPITER"}"#)?;
    /// assert_eq!(strong.kind(), "planet-strong");
    /// // An alias reads as the name it stands for.
    /// let both: Condition = serde_json::from_str(r#"{"type": "dosha-and", "conditions": []}"#)?;
    /// assert_eq!(both.kind(), "and");
    /// # Ok::<(), serde_json::Error>(())
    /// ```
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
            Condition::AllPlanetsBetweenNodes { .. } => "all-planets-between-nodes",
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
            Condition::PlanetInDegrees { .. } => "planet-in-degrees",
            Condition::PanchangaTithi { .. } => "panchanga-tithi",
            Condition::PanchangaPaksha { .. } => "panchanga-paksha",
            Condition::PanchangaVara { .. } => "panchanga-vara",
            Condition::PanchangaNakshatra { .. } => "panchanga-nakshatra",
            Condition::PanchangaYoga { .. } => "panchanga-yoga",
            Condition::PanchangaKarana { .. } => "panchanga-karana",
            Condition::BirthDuringEclipse { .. } => "birth-during-eclipse",
            Condition::BirthOnSankranti { .. } => "birth-on-sankranti",
            Condition::TransitInHouse { .. } => "transit-in-house",
            Condition::PlanetInNakshatra { .. } => "planet-in-nakshatra",
            Condition::SameNakshatra { .. } => "same-nakshatra",
            Condition::PlanetStrong { .. } => "planet-strong",
            Condition::PlanetWeak { .. } => "planet-weak",
            Condition::PlanetStrongerThan { .. } => "planet-stronger-than",
            Condition::RashiAspects { .. } => "rashi-aspects",
            Condition::Argala { .. } => "argala",
            Condition::VipareetaArgala { .. } => "vipareeta-argala",
            Condition::ForAny { .. } => "for-any",
            Condition::CountOf { .. } => "count-of",
            Condition::NOf { .. } => "n-of",
            Condition::InVarga { .. } => "in-varga",
            Condition::CountInHouses { .. } => "count-in-houses",
            Condition::CountAspecting { .. } => "count-aspecting",
            Condition::RuleHolds { .. } => "rule",
            Condition::AtLimbEdge { .. } => "at-limb-edge",
            Condition::LimbPart { .. } => "limb-part",
            Condition::BirthByDay => "birth-by-day",
            Condition::SameSign { .. } => "same-sign",
            Condition::SameBody { .. } => "same-body",
            Condition::PlanetIs { .. } => "planet-is",
            Condition::NaturalRelation { .. } => "natural-relation",
        }
    }

    /// The conditions a combinator combines, and none for a predicate.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let ninth: Condition = serde_json::from_str(r#"{"type": "in-varga", "varga": "D9", "condition": {"type": "planet-in-kendra", "planet": "JUPITER"}}"#)?;
    /// assert_eq!(ninth.children()[0].kind(), "planet-in-kendra");
    /// // A predicate has none.
    /// assert!(ninth.children()[0].children().is_empty());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn children(&self) -> &[Condition] {
        match self {
            Condition::And { conditions }
            | Condition::Or { conditions }
            | Condition::NOf { conditions, .. } => conditions,
            Condition::Not { condition }
            | Condition::ForAny {
                then: condition, ..
            }
            | Condition::CountOf {
                then: condition, ..
            }
            | Condition::InVarga { condition, .. } => core::slice::from_ref(condition),
            _ => &[],
        }
    }

    /// The conditions a combinator combines, to change in place.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let mut ninth: Condition = serde_json::from_str(r#"{"type": "in-varga", "varga": "D9", "condition": {"type": "planet-in-kendra", "planet": "JUPITER"}}"#)?;
    /// ninth.children_mut()[0] = serde_json::from_str(r#"{"type": "planet-strong", "planet": "JUPITER"}"#)?;
    /// assert_eq!(ninth.children()[0].kind(), "planet-strong");
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn children_mut(&mut self) -> &mut [Condition] {
        match self {
            Condition::And { conditions }
            | Condition::Or { conditions }
            | Condition::NOf { conditions, .. } => conditions,
            Condition::Not { condition }
            | Condition::ForAny {
                then: condition, ..
            }
            | Condition::CountOf {
                then: condition, ..
            }
            | Condition::InVarga { condition, .. } => core::slice::from_mut(condition),
            _ => &mut [],
        }
    }

    /// Whether it binds `SELF` for the conditions inside it.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let each: Condition = serde_json::from_str(r#"{"type": "for-any", "then": {"type": "planet-in-kendra", "planet": "SELF"}}"#)?;
    /// assert!(each.binds_self());
    /// assert!(!each.children()[0].binds_self());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn binds_self(&self) -> bool {
        matches!(self, Condition::ForAny { .. } | Condition::CountOf { .. })
    }

    /// Whether any reference it names is `SELF`, itself and not inside it.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let each: Condition = serde_json::from_str(r#"{"type": "for-any", "then": {"type": "planet-in-kendra", "planet": "SELF"}}"#)?;
    /// // The `for-any` binds it and its child names it.
    /// assert!(!each.names_self());
    /// assert!(each.children()[0].names_self());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn names_self(&self) -> bool {
        // A condition's references are what its own serialisation holds, and
        // `SELF` is written as that word; its children are written under
        // these keys, and name for themselves.
        const CHILDREN: [&str; 3] = ["condition", "conditions", "then"];
        fn names(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::String(text) => text == SELF,
                serde_json::Value::Array(items) => items.iter().any(names),
                serde_json::Value::Object(fields) => fields.values().any(names),
                _ => false,
            }
        }
        serde_json::to_value(self).is_ok_and(|value| match value {
            serde_json::Value::Object(fields) => fields
                .iter()
                .filter(|(key, _)| !CHILDREN.contains(&key.as_str()))
                .any(|(_, value)| names(value)),
            other => names(&other),
        })
    }

    /// Whether it reads a body's longitude, which a divisional chart does not
    /// move, or the sky in transit, which no divisional chart holds: such a
    /// condition is refused inside an `in-varga`.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let degrees: Condition = serde_json::from_str(r#"{"type": "planet-in-degrees", "planet": "MOON", "from": 0.0, "to": 10.0}"#)?;
    /// let strong: Condition = serde_json::from_str(r#"{"type": "planet-strong", "planet": "JUPITER"}"#)?;
    /// assert!(degrees.reads_a_longitude());
    /// assert!(!strong.reads_a_longitude());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn reads_a_longitude(&self) -> bool {
        matches!(
            self,
            Condition::PlanetAtTableDegree { .. }
                | Condition::TransitInHouse { .. }
                | Condition::PlanetAtGandanta { .. }
                | Condition::PlanetInDegrees { .. }
                | Condition::PlanetConjunct {
                    max_orb: Some(_),
                    ..
                }
        )
    }

    /// The first condition inside it, itself included, that reads a longitude
    /// under an `in-varga`.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let moved: Condition = serde_json::from_str(r#"{"type": "in-varga", "varga": "D9", "condition": {"type": "planet-in-degrees", "planet": "MOON", "from": 0.0, "to": 10.0}}"#)?;
    /// // A divisional chart does not move a longitude, so this is refused.
    /// assert_eq!(moved.longitude_in_varga(), Some("planet-in-degrees"));
    /// let ninth: Condition = serde_json::from_str(r#"{"type": "in-varga", "varga": "D9", "condition": {"type": "planet-in-kendra", "planet": "JUPITER"}}"#)?;
    /// assert_eq!(ninth.longitude_in_varga(), None);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn longitude_in_varga(&self) -> Option<&'static str> {
        fn walk(condition: &Condition, within: bool) -> Option<&'static str> {
            if within && condition.reads_a_longitude() {
                return Some(condition.kind());
            }
            let within = within || matches!(condition, Condition::InVarga { .. });
            condition
                .children()
                .iter()
                .find_map(|child| walk(child, within))
        }
        walk(self, false)
    }

    /// The first `limb-part` inside it, itself included, that divides a
    /// limb into fewer than two parts or names a part it does not have,
    /// with what is wrong.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let fifth: Condition = serde_json::from_str(r#"{"type": "not", "condition": {"type": "limb-part", "limb": "tithi", "of": 4, "parts": [5]}}"#)?;
    /// assert_eq!(fifth.malformed_part().as_deref(), Some("`limb-part` names part 5 of 4; the parts are 1 to 4"));
    /// let first: Condition = serde_json::from_str(r#"{"type": "limb-part", "limb": "tithi", "of": 4, "parts": [1]}"#)?;
    /// assert_eq!(first.malformed_part(), None);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn malformed_part(&self) -> Option<String> {
        if let Condition::LimbPart { of, parts, .. } = self {
            if *of < 2 {
                return Some(format!(
                    "`limb-part` divides into {of} parts, and needs at least 2"
                ));
            }
            if parts.is_empty() {
                return Some("`limb-part` names no part".to_owned());
            }
            if let Some(part) = parts.iter().find(|part| **part == 0 || **part > *of) {
                return Some(format!(
                    "`limb-part` names part {part} of {of}; the parts are 1 to {of}"
                ));
            }
        }
        self.children().iter().find_map(Condition::malformed_part)
    }

    /// Whether it asks a question of strength, so a caller knows to give the
    /// chart [`Strengths`](crate::Strengths); without them it answers false.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let strong: Condition = serde_json::from_str(r#"{"type": "planet-strong", "planet": "JUPITER"}"#)?;
    /// assert!(strong.reads_strength());
    /// // Only itself: an `and` around it reads nothing of its own.
    /// let both: Condition = serde_json::from_str(r#"{"type": "and", "conditions": [{"type": "planet-strong", "planet": "JUPITER"}]}"#)?;
    /// assert!(!both.reads_strength());
    /// assert!(both.walk().any(Condition::reads_strength));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn reads_strength(&self) -> bool {
        matches!(
            self,
            Condition::PlanetStrong { .. }
                | Condition::PlanetWeak { .. }
                | Condition::PlanetStrongerThan { .. }
                | Condition::Argala { .. }
        )
    }

    /// Whether this condition itself reads the sky in transit, so a caller
    /// knows to give the evaluator one; without it the condition is false.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let transit: Condition = serde_json::from_str(r#"{"type": "transit-in-house", "planet": "SATURN", "houses": [8], "from": "MOON"}"#)?;
    /// assert!(transit.reads_transits());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn reads_transits(&self) -> bool {
        matches!(self, Condition::TransitInHouse { .. })
    }

    /// Whether this condition itself reads the chart's panchanga.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let day: Condition = serde_json::from_str(r#"{"type": "birth-by-day"}"#)?;
    /// let strong: Condition = serde_json::from_str(r#"{"type": "planet-strong", "planet": "JUPITER"}"#)?;
    /// assert!(day.reads_panchanga());
    /// assert!(!strong.reads_panchanga());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
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
                | Condition::AtLimbEdge { .. }
                | Condition::LimbPart { .. }
                | Condition::BirthByDay
        )
    }

    /// Whether this condition itself reads what the chart's instant falls in
    /// rather than its day: the rising sign's edge or an eclipse. Each also
    /// reads the panchanga, which carries both to the evaluator.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let lagna: Condition = serde_json::from_str(r#"{"type": "limb-part", "limb": "lagna", "of": 3, "parts": [1]}"#)?;
    /// let tithi: Condition = serde_json::from_str(r#"{"type": "limb-part", "limb": "tithi", "of": 3, "parts": [1]}"#)?;
    /// // The rising sign's part is the instant's; the tithi's is the day's.
    /// assert!(lagna.reads_moment() && lagna.reads_panchanga());
    /// assert!(!tithi.reads_moment());
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn reads_moment(&self) -> bool {
        matches!(
            self,
            Condition::BirthDuringEclipse { .. }
                | Condition::AtLimbEdge {
                    limb: Limb::Lagna,
                    ..
                }
                | Condition::LimbPart {
                    limb: Limb::Lagna,
                    ..
                }
        )
    }

    /// The first condition inside it, itself included, that names `SELF` with
    /// no `for-any` above it to bind one.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let bare: Condition = serde_json::from_str(r#"{"type": "planet-in-kendra", "planet": "SELF"}"#)?;
    /// let each: Condition = serde_json::from_str(r#"{"type": "for-any", "then": {"type": "planet-in-kendra", "planet": "SELF"}}"#)?;
    /// // `SELF` with nothing to bind it.
    /// assert_eq!(bare.unbound_self(), Some("planet-in-kendra"));
    /// assert_eq!(each.unbound_self(), None);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn unbound_self(&self) -> Option<&'static str> {
        fn walk(condition: &Condition, bound: bool) -> Option<&'static str> {
            if !bound && condition.names_self() {
                return Some(condition.kind());
            }
            let bound = bound || condition.binds_self();
            condition
                .children()
                .iter()
                .find_map(|child| walk(child, bound))
        }
        walk(self, false)
    }

    /// This condition and every one inside it, depth first.
    ///
    /// ```
    /// use teistro_rules::Condition;
    ///
    /// let tree: Condition = serde_json::from_str(r#"{"type": "and", "conditions": [{"type": "in-varga", "varga": "D9", "condition": {"type": "planet-in-kendra", "planet": "JUPITER"}}, {"type": "planet-strong", "planet": "JUPITER"}]}"#)?;
    /// let kinds: Vec<&str> = tree.walk().map(Condition::kind).collect();
    /// assert_eq!(kinds, ["and", "in-varga", "planet-in-kendra", "planet-strong"]);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn walk(&self) -> impl Iterator<Item = &Condition> {
        let mut stack = vec![self];
        core::iter::from_fn(move || {
            let next = stack.pop()?;
            stack.extend(next.children().iter().rev());
            Some(next)
        })
    }
}

/// How good the evidence behind a citation is, as the project ranks it
/// (`01-research/feature-universe/19-verification-cruxes.md`): 1 a classical
/// text or a faithful translation, 2 an implementation, 3 a secondary source,
/// 4 nothing found.
///
/// ```
/// use teistro_rules::EvidenceRank;
///
/// // A classical text outranks an implementation.
/// assert!(EvidenceRank::try_new(1)? < EvidenceRank::try_new(2)?);
/// assert_eq!(serde_json::to_string(&EvidenceRank::try_new(3)?).map_err(|e| e.to_string())?, "3");
/// # Ok::<(), String>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(try_from = "u8", into = "u8")]
pub struct EvidenceRank(u8);

impl EvidenceRank {
    /// A rank.
    ///
    /// ```
    /// use teistro_rules::EvidenceRank;
    ///
    /// assert!(EvidenceRank::try_new(4).is_ok());
    /// assert_eq!(EvidenceRank::try_new(5).unwrap_err(), "evidence rank 5 is not 1 to 4");
    /// ```
    ///
    /// # Errors
    ///
    /// A number outside 1 to 4, named.
    pub fn try_new(rank: u8) -> Result<EvidenceRank, String> {
        if (1..=4).contains(&rank) {
            Ok(EvidenceRank(rank))
        } else {
            Err(format!("evidence rank {rank} is not 1 to 4"))
        }
    }

    /// Its number, 1 to 4.
    ///
    /// ```
    /// use teistro_rules::EvidenceRank;
    ///
    /// assert_eq!(EvidenceRank::try_new(1)?.get(), 1);
    /// # Ok::<(), String>(())
    /// ```
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for EvidenceRank {
    type Error = String;

    fn try_from(rank: u8) -> Result<EvidenceRank, String> {
        EvidenceRank::try_new(rank)
    }
}

impl From<EvidenceRank> for u8 {
    fn from(rank: EvidenceRank) -> u8 {
        rank.0
    }
}

/// Where a rule or a table comes from.
///
/// ```
/// use teistro_rules::Source;
///
/// let source: Source = serde_json::from_str(r#"{"text": "Saravali", "chapter": "31", "verse": "87", "rank": 1}"#)?;
/// assert_eq!(source.rank.map(|rank| rank.get()), Some(1));
/// // Only these fields.
/// assert!(serde_json::from_str::<Source>(r#"{"text": "Saravali", "page": 4}"#).is_err());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "RuleSource"))]
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
    /// How good the evidence is, when the author says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<EvidenceRank>,
}

impl Source {
    /// A text, with no chapter, verse, note or rank.
    ///
    /// ```
    /// use teistro_rules::Source;
    ///
    /// let source = Source::text("Brihat Jataka");
    /// assert_eq!(source.chapter, None);
    /// // Written with its text alone.
    /// assert_eq!(serde_json::to_string(&source)?, r#"{"text":"Brihat Jataka"}"#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub fn text(text: impl Into<String>) -> Source {
        Source {
            text: text.into(),
            chapter: None,
            verse: None,
            note: None,
            rank: None,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests unwrap and index what they read"
    )]

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

    /// `SELF` is bound only under a `for-any` or `count-of`: named bare, in a
    /// list or in a reference's parts, and under any other combinator, it is
    /// unbound and the condition that names it is reported; the binder's own
    /// `then` is not its naming.
    #[test]
    fn self_is_unbound_outside_a_binder_wherever_it_is_written() {
        let unbound = |json: &str| condition(json).unwrap().unbound_self();
        let bare = r#"{"type": "planet-in-kendra", "planet": "SELF"}"#;
        assert_eq!(unbound(bare), Some("planet-in-kendra"));
        assert_eq!(
            unbound(r#"{"type": "planet-conjunct", "planets": ["SUN", "SELF"]}"#),
            Some("planet-conjunct")
        );
        assert_eq!(
            unbound(r#"{"type": "planet-in-kendra", "planet": {"lordOf": "SELF"}}"#),
            Some("planet-in-kendra")
        );
        assert_eq!(
            unbound(&format!(r#"{{"type": "and", "conditions": [{bare}]}}"#)),
            Some("planet-in-kendra")
        );
        assert_eq!(
            unbound(&format!(r#"{{"type": "for-any", "then": {bare}}}"#)),
            None
        );
        assert_eq!(
            unbound(&format!(
                r#"{{"type": "not", "condition": {{"type": "for-any", "then": {bare}}}}}"#
            )),
            None
        );
        assert_eq!(
            unbound(r#"{"type": "planet-in-kendra", "planet": "JUPITER"}"#),
            None
        );
        let and = condition(&format!(r#"{{"type": "and", "conditions": [{bare}]}}"#)).unwrap();
        assert!(!and.binds_self() && !and.names_self());
    }

    /// A limb divides into two parts at least, and its parts run 1 to that.
    #[test]
    fn a_limb_part_needs_two_parts_and_names_one_it_has() {
        let malformed = |of: u8, parts: &str| {
            let json = format!(
                r#"{{"type": "limb-part", "limb": "tithi", "of": {of}, "parts": {parts}}}"#
            );
            condition(&json).unwrap().malformed_part()
        };
        assert_eq!(malformed(2, "[1, 2]"), None);
        assert!(malformed(1, "[1]").unwrap().contains("at least 2"));
        assert!(malformed(3, "[]").unwrap().contains("names no part"));
        assert!(malformed(3, "[4]").unwrap().contains("part 4 of 3"));
        assert!(malformed(3, "[0]").unwrap().contains("part 0 of 3"));
        let inside = condition(
            r#"{"type": "not", "condition": {"type": "limb-part", "limb": "tithi", "of": 1, "parts": [1]}}"#,
        )
        .unwrap();
        assert!(inside.malformed_part().is_some());
    }

    /// What a condition itself reads, so a caller knows what to give it.
    #[test]
    fn a_condition_says_whether_it_reads_the_moment_or_the_transits() {
        let kendra = condition(r#"{"type": "planet-in-kendra", "planet": "JUPITER"}"#).unwrap();
        let eclipse = condition(r#"{"type": "birth-during-eclipse"}"#).unwrap();
        assert!(eclipse.reads_moment() && !kendra.reads_moment());
        assert!(!kendra.reads_transits() && !eclipse.reads_transits());
        assert_eq!(EvidenceRank::try_new(3).unwrap().get(), 3);
    }

    #[test]
    fn a_combinator_s_children_change_in_place() {
        let mut not = condition(
            r#"{"type": "not", "condition": {"type": "planet-in-kendra", "planet": "JUPITER"}}"#,
        )
        .unwrap();
        let children = not.children_mut();
        assert_eq!(children.len(), 1);
        children[0] = condition(r#"{"type": "birth-by-day"}"#).unwrap();
        assert_eq!(not.children()[0].kind(), "birth-by-day");
        let mut leaf = condition(r#"{"type": "birth-by-day"}"#).unwrap();
        assert_eq!(leaf.children_mut(), []);
    }
}

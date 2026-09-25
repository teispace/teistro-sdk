//! A nakshatra-seeded dasha system as a row over the K-udu kernel
//! (`03-design/dasha-kernels.md`): its lords and their years, and the map
//! from the Moon's nakshatra to the lord it starts with.
//!
//! A system is data. Vimshottari, Ashtottari and Dwadashottari differ in
//! the table and in four fields of the map, and Tribhagi is Vimshottari
//! scaled, so each is a constant and not a module; a row the checks refuse
//! is refused by the field it gets wrong. Every row here is reproduced over
//! the conformance corpus (`03-design/dasha-measured.md`,
//! `03-design/dasha-systems-measured.md`).
//!
//! A consumer's own system is a [`UduDefinition`], checked by the same
//! rules and run by the same kernel (`03-design/dasha-kernels.md`, "A
//! consumer's own system").

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra};
use teistro_core::error::Error;
use teistro_core::key::is_key_name;
use teistro_core::quantity::Depth;
use teistro_core::settings::{AshtottariGrouping, YearLength};

use crate::wheel::Wheel;

/// How many nakshatras a seed is counted over.
pub const NAKSHATRAS: u8 = 27;

/// One lord of a system and its years.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Lord {
    /// The graha.
    pub graha: Graha,
    /// Its whole years in the cycle.
    pub years: u8,
}

/// Which way a system counts from its reference nakshatra to the seed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Count {
    #[default]
    /// Forwards from the reference: Vimshottari from Ashwini.
    FromReference,
    /// Backwards to the reference: Dwadashottari to Revati.
    ToReference,
}

/// A factor on every mahadasha's years, and how many times the sequence
/// runs in one cycle: Tribhagi's two thirds, twice round.
///
/// The sub-periods are still shares of the row's whole years, so a scale
/// changes how long a mahadasha is and not how it divides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Scale {
    /// The factor's numerator.
    pub numerator: u8,
    /// The factor's denominator.
    pub denominator: u8,
    /// How many times the sequence runs before the cycle ends.
    pub rounds: u8,
}

impl Default for Scale {
    fn default() -> Scale {
        Scale::WHOLE
    }
}

impl Scale {
    /// Every lord's own years, the sequence once.
    pub const WHOLE: Scale = Scale {
        numerator: 1,
        denominator: 1,
        rounds: 1,
    };

    /// The factor on a mahadasha's years.
    #[must_use]
    pub fn factor(self) -> f64 {
        f64::from(self.numerator) / f64::from(self.denominator.max(1))
    }
}

/// A dasha system's name: a member the catalogue has, or the key a context
/// registered a consumer's system under.
///
/// It serialises as the bare key either way (`VIMSHOTTARI`, `ACME_SAPTA`),
/// and a registry refuses a key the catalogue has, so the two never meet.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DashaName {
    /// A catalogued system.
    Catalogued(DashaSystem),
    /// A consumer's system, by the key it was registered under.
    Registered(String),
}

impl DashaName {
    /// The key: the catalogue's, or the registered one.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            DashaName::Catalogued(system) => system.key(),
            DashaName::Registered(key) => key,
        }
    }

    /// The key under its kind, `dasha_system.VIMSHOTTARI`, which is how a
    /// binding spells either.
    #[must_use]
    pub fn full_key(&self) -> String {
        format!("dasha_system.{}", self.key())
    }

    /// The catalogued system, when it is one.
    #[must_use]
    pub const fn catalogued(&self) -> Option<DashaSystem> {
        match self {
            DashaName::Catalogued(system) => Some(*system),
            DashaName::Registered(_) => None,
        }
    }
}

impl From<DashaSystem> for DashaName {
    fn from(system: DashaSystem) -> DashaName {
        DashaName::Catalogued(system)
    }
}

impl From<&DashaName> for DashaName {
    fn from(name: &DashaName) -> DashaName {
        name.clone()
    }
}

impl From<&str> for DashaName {
    /// A catalogued system when the catalogue has the key, else a registered
    /// one's name.
    fn from(key: &str) -> DashaName {
        DashaSystem::from_key(key).map_or_else(
            || DashaName::Registered(key.to_owned()),
            DashaName::Catalogued,
        )
    }
}

impl PartialEq<DashaSystem> for DashaName {
    fn eq(&self, other: &DashaSystem) -> bool {
        self.catalogued() == Some(*other)
    }
}

impl std::fmt::Display for DashaName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

impl Serialize for DashaName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.key())
    }
}

impl<'de> Deserialize<'de> for DashaName {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<DashaName, D::Error> {
        let key = String::deserialize(deserializer)?;
        if let Some(system) = DashaSystem::from_key(&key) {
            return Ok(DashaName::Catalogued(system));
        }
        if is_key_name(&key) {
            Ok(DashaName::Registered(key))
        } else {
            Err(serde::de::Error::custom(format!(
                "`{key}` is neither a catalogued dasha system nor a key name"
            )))
        }
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for DashaName {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DashaName".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // The catalogue's own schema first, so its keys and its kind stay
        // described (and the gates that read a document's kinds still see
        // this one), then any key a context could have registered.
        let catalogued = generator.subschema_for::<DashaSystem>();
        schemars::json_schema!({
            "description": "A catalogued dasha system's key, or the key a context registered a consumer's system under.",
            "anyOf": [
                catalogued,
                { "type": "string", "pattern": "^[A-Z][A-Z0-9_]{0,47}$" }
            ]
        })
    }
}

/// A nakshatra-seeded system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UduRow {
    /// Which system the row is.
    pub system: DashaName,
    /// The lords, in the order they run: borrowed for a shipped row, owned
    /// for a registered one.
    pub lords: Cow<'static, [Lord]>,
    /// The nakshatra, 0 for Ashwini, that maps to the first lord.
    pub reference: u8,
    /// Which way the seed is counted.
    pub count: Count,
    /// How many nakshatras each lord covers: 1 for most systems, 3 for
    /// the recording engine's Ashtottari. The balance is the elapsed part of
    /// this window. A row with [`groups`](UduRow::groups) reads those
    /// instead.
    pub span: u8,
    /// How many nakshatras each lord covers, lord by lord, when they are not
    /// all alike: BPHS's Ashtottari gives four and three alternately from
    /// Ardra. Empty for a row whose every lord covers
    /// [`span`](UduRow::span).
    pub groups: Cow<'static, [u8]>,
    /// The circle the seed is counted round: the twenty-seven nakshatras,
    /// or the twenty-eight with Abhijit that BPHS counts Ashtottari and
    /// Shashtihayani over.
    pub wheel: Wheel,
    /// What is added after the division, before the modulo: 3 for Yogini.
    pub offset: u8,
    /// Whether the lords run round the nakshatras again once they are all
    /// used — Vimshottari's nine three times — or cover them once, leaving
    /// a seed past them outside the cycle, as Ashtottari's eight windows of
    /// three leave three. Stated rather than inferred: Yogini's eight lords
    /// do not divide 27 either, and repeat.
    pub repeats: bool,
    /// The factor on the mahadashas' years and the rounds in a cycle.
    pub scale: Scale,
}

/// Where a seed nakshatra falls in a row's cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seat {
    /// The first lord, as an index into [`UduRow::lords`].
    pub lord: usize,
    /// How many whole nakshatras of the lord's window are already behind
    /// the seed: 0 when each lord covers one.
    pub within: u8,
    /// Whether the seed lies past the nakshatras the lords cover, which a
    /// conditional system such as Ashtottari has, and which the settings'
    /// `seed_overflow` decides.
    pub overflow: bool,
}

impl UduRow {
    /// The cycle's years, the sum of the lords': what every sub-period is a
    /// share of, whatever the scale.
    #[must_use]
    pub fn total_years(&self) -> u32 {
        self.lords.iter().map(|lord| u32::from(lord.years)).sum()
    }

    /// How many mahadashas one cycle runs: the lords, once each round.
    #[must_use]
    pub fn mahadashas(&self) -> usize {
        self.lords.len() * usize::from(self.scale.rounds.max(1))
    }

    /// The years of the lord at `index` round the sequence, scaled: how long
    /// its mahadasha runs.
    #[must_use]
    pub fn scaled_years(&self, index: usize) -> f64 {
        self.lords
            .get(index % self.lords.len().max(1))
            .map_or(0.0, |lord| f64::from(lord.years) * self.scale.factor())
    }

    /// How many nakshatras the lord at `index` covers: its own group, or
    /// the row's span when every lord covers alike.
    #[must_use]
    pub fn window_of(&self, index: usize) -> u8 {
        self.groups
            .get(index % self.lords.len().max(1))
            .copied()
            .unwrap_or(self.span)
            .max(1)
    }

    /// How many nakshatras one round of the lords covers.
    fn covered(&self) -> u16 {
        (0..self.lords.len())
            .map(|index| u16::from(self.window_of(index)))
            .sum()
    }

    /// The lord a seed starts with, and where it sits in that lord's
    /// window. `place` is the seed's place on the row's
    /// [`wheel`](UduRow::wheel), which on the twenty-seven is the
    /// nakshatra's own index.
    #[must_use]
    pub fn seat(&self, place: u8) -> Seat {
        let cycle = i16::from(self.wheel.size());
        let reference = i16::from(self.wheel.place_of(self.reference));
        let seed = i16::from(place);
        let counted = match self.count {
            Count::FromReference => (seed - reference).rem_euclid(cycle),
            Count::ToReference => (reference - seed).rem_euclid(cycle),
        };
        let counted = u16::try_from(counted).unwrap_or_default();
        let lords = self.lords.len().max(1);
        let covered = self.covered().max(1);
        let overflow = !self.repeats && counted >= covered;
        // Round and round the lords' windows until the seed falls in one; a
        // row that repeats covers the wheel as often as it takes.
        let mut left = if overflow { counted } else { counted % covered };
        let mut group = 0_usize;
        while left >= u16::from(self.window_of(group)) {
            left -= u16::from(self.window_of(group));
            group += 1;
        }
        Seat {
            lord: (group + usize::from(self.offset)) % lords,
            within: u8::try_from(left).unwrap_or_default(),
            overflow,
        }
    }

    /// The checks every row passes before it is used, naming the field it
    /// fails.
    ///
    /// # Errors
    ///
    /// No lords, a lord of no years, a reference past the last nakshatra, a
    /// span of nothing, windows that cover more nakshatras than there are, a
    /// scale of nothing, or more mahadashas in a cycle than a path can
    /// place.
    pub fn validate(&self) -> Result<(), Error> {
        let refuse = |field: &str, message: String| {
            Err(Error::invalid_arg(message).with_field(field.to_owned()))
        };
        if self.lords.is_empty() {
            return refuse("lords", String::from("a dasha system needs a lord"));
        }
        if let Some(at) = self.lords.iter().position(|lord| lord.years == 0) {
            return refuse(
                &format!("lords[{at}].years"),
                String::from("a lord of no years"),
            );
        }
        if self.reference >= NAKSHATRAS {
            return refuse(
                "reference",
                format!("nakshatra {} is past Revati", self.reference),
            );
        }
        if self.span == 0 {
            return refuse("span", String::from("a lord covers at least one nakshatra"));
        }
        if !self.groups.is_empty() && self.groups.len() != self.lords.len() {
            return refuse(
                "groups",
                format!(
                    "{} groups for {} lords: one each, or none for a span",
                    self.groups.len(),
                    self.lords.len()
                ),
            );
        }
        if let Some(at) = self.groups.iter().position(|count| *count == 0) {
            return refuse(
                &format!("groups[{at}]"),
                String::from("a lord covers at least one nakshatra"),
            );
        }
        let covered = self.covered();
        let size = self.wheel.size();
        if covered > u16::from(size) {
            return refuse(
                if self.groups.is_empty() {
                    "span"
                } else {
                    "groups"
                },
                format!(
                    "{} lords cover {covered} nakshatras, past the {size} of the wheel",
                    self.lords.len()
                ),
            );
        }
        if self.scale.numerator == 0 || self.scale.denominator == 0 {
            return refuse(
                "scale",
                String::from("a scale's numerator and denominator are at least one"),
            );
        }
        if self.scale.rounds == 0 || self.mahadashas() > usize::from(u8::MAX) {
            return refuse(
                "scale.rounds",
                format!(
                    "{} rounds of {} lords is not 1 to {} mahadashas",
                    self.scale.rounds,
                    self.lords.len(),
                    u8::MAX
                ),
            );
        }
        Ok(())
    }
}

const fn lord(graha: Graha, years: u8) -> Lord {
    Lord { graha, years }
}

/// Vimshottari: nine lords over 120 years, counted from Ashwini, one
/// nakshatra each. Measured over the whole corpus
/// (`03-design/dasha-measured.md`).
pub const VIMSHOTTARI: UduRow = UduRow {
    system: DashaName::Catalogued(DashaSystem::Vimshottari),
    lords: Cow::Borrowed(&[
        lord(Graha::Ketu, 7),
        lord(Graha::Venus, 20),
        lord(Graha::Sun, 6),
        lord(Graha::Moon, 10),
        lord(Graha::Mars, 7),
        lord(Graha::Rahu, 18),
        lord(Graha::Jupiter, 16),
        lord(Graha::Saturn, 19),
        lord(Graha::Mercury, 17),
    ]),
    reference: 0,
    count: Count::FromReference,
    span: 1,
    offset: 0,
    repeats: true,
    scale: Scale::WHOLE,
    groups: Cow::Borrowed(&[]),
    wheel: Wheel::Nakshatras,
};

/// Ashtottari: eight lords over 108 years, counted from Ardra, three
/// nakshatras each. The eight windows cover 24 nakshatras once, so the three
/// before Ardra lie outside the cycle, which `dasha.seed_overflow` decides
/// (crux C5); whether a chart is one Ashtottari applies to is a rule and not
/// this row (crux C3).
pub const ASHTOTTARI: UduRow = UduRow {
    system: DashaName::Catalogued(DashaSystem::Ashtottari),
    lords: Cow::Borrowed(&[
        lord(Graha::Sun, 6),
        lord(Graha::Moon, 15),
        lord(Graha::Mars, 8),
        lord(Graha::Mercury, 17),
        lord(Graha::Saturn, 10),
        lord(Graha::Jupiter, 19),
        lord(Graha::Rahu, 12),
        lord(Graha::Venus, 21),
    ]),
    reference: 5,
    count: Count::FromReference,
    span: 3,
    offset: 0,
    repeats: false,
    scale: Scale::WHOLE,
    groups: Cow::Borrowed(&[]),
    wheel: Wheel::Nakshatras,
};

/// Dwadashottari: eight lords over 112 years, counted from the seed back to
/// Revati.
pub const DWADASHOTTARI: UduRow = UduRow {
    system: DashaName::Catalogued(DashaSystem::Dwadashottari),
    lords: Cow::Borrowed(&[
        lord(Graha::Sun, 7),
        lord(Graha::Jupiter, 9),
        lord(Graha::Ketu, 11),
        lord(Graha::Mercury, 13),
        lord(Graha::Rahu, 15),
        lord(Graha::Mars, 17),
        lord(Graha::Saturn, 19),
        lord(Graha::Moon, 21),
    ]),
    reference: 26,
    count: Count::ToReference,
    span: 1,
    offset: 0,
    repeats: true,
    scale: Scale::WHOLE,
    groups: Cow::Borrowed(&[]),
    wheel: Wheel::Nakshatras,
};

/// Panchottari: seven lords over 105 years, counted from Anuradha.
pub const PANCHOTTARI: UduRow = UduRow {
    system: DashaName::Catalogued(DashaSystem::Panchottari),
    lords: Cow::Borrowed(&[
        lord(Graha::Sun, 12),
        lord(Graha::Mercury, 13),
        lord(Graha::Saturn, 14),
        lord(Graha::Mars, 15),
        lord(Graha::Venus, 16),
        lord(Graha::Moon, 17),
        lord(Graha::Jupiter, 18),
    ]),
    reference: 16,
    count: Count::FromReference,
    span: 1,
    offset: 0,
    repeats: true,
    scale: Scale::WHOLE,
    groups: Cow::Borrowed(&[]),
    wheel: Wheel::Nakshatras,
};

/// Shatabdika: seven lords over 100 years, counted from Revati.
pub const SHATABDIKA: UduRow = UduRow {
    system: DashaName::Catalogued(DashaSystem::Shatabdika),
    lords: Cow::Borrowed(&[
        lord(Graha::Sun, 5),
        lord(Graha::Moon, 5),
        lord(Graha::Venus, 10),
        lord(Graha::Mercury, 10),
        lord(Graha::Jupiter, 20),
        lord(Graha::Mars, 20),
        lord(Graha::Saturn, 30),
    ]),
    reference: 26,
    count: Count::FromReference,
    span: 1,
    offset: 0,
    repeats: true,
    scale: Scale::WHOLE,
    groups: Cow::Borrowed(&[]),
    wheel: Wheel::Nakshatras,
};

/// Chaturashiti-sama: seven lords of twelve years each, 84 in all, counted
/// from Swati.
pub const CHATURASHITI_SAMA: UduRow = UduRow {
    system: DashaName::Catalogued(DashaSystem::ChaturashitiSama),
    lords: Cow::Borrowed(&[
        lord(Graha::Sun, 12),
        lord(Graha::Moon, 12),
        lord(Graha::Mars, 12),
        lord(Graha::Mercury, 12),
        lord(Graha::Jupiter, 12),
        lord(Graha::Venus, 12),
        lord(Graha::Saturn, 12),
    ]),
    reference: 14,
    count: Count::FromReference,
    span: 1,
    offset: 0,
    repeats: true,
    scale: Scale::WHOLE,
    groups: Cow::Borrowed(&[]),
    wheel: Wheel::Nakshatras,
};

/// Dwisaptati-sama: eight lords of nine years each, 72 in all, counted from
/// Mula.
pub const DWISAPTATI_SAMA: UduRow = UduRow {
    system: DashaName::Catalogued(DashaSystem::DwisaptatiSama),
    lords: Cow::Borrowed(&[
        lord(Graha::Sun, 9),
        lord(Graha::Moon, 9),
        lord(Graha::Mars, 9),
        lord(Graha::Mercury, 9),
        lord(Graha::Jupiter, 9),
        lord(Graha::Venus, 9),
        lord(Graha::Saturn, 9),
        lord(Graha::Rahu, 9),
    ]),
    reference: 18,
    count: Count::FromReference,
    span: 1,
    offset: 0,
    repeats: true,
    scale: Scale::WHOLE,
    groups: Cow::Borrowed(&[]),
    wheel: Wheel::Nakshatras,
};

/// Yogini: the eight yoginis' lords over 36 years, counted from Ashwini
/// with three added, so Ashwini's is the fourth.
pub const YOGINI: UduRow = UduRow {
    system: DashaName::Catalogued(DashaSystem::Yogini),
    lords: Cow::Borrowed(&[
        lord(Graha::Moon, 1),
        lord(Graha::Sun, 2),
        lord(Graha::Jupiter, 3),
        lord(Graha::Mars, 4),
        lord(Graha::Mercury, 5),
        lord(Graha::Saturn, 6),
        lord(Graha::Venus, 7),
        lord(Graha::Rahu, 8),
    ]),
    reference: 0,
    count: Count::FromReference,
    span: 1,
    offset: 3,
    repeats: true,
    scale: Scale::WHOLE,
    groups: Cow::Borrowed(&[]),
    wheel: Wheel::Nakshatras,
};

/// Tribhagi: Vimshottari's lords at two thirds of their years, the sequence
/// twice round, eighty years a round; the sub-periods still shares of 120.
pub const TRIBHAGI: UduRow = UduRow {
    system: DashaName::Catalogued(DashaSystem::Tribhagi),
    lords: VIMSHOTTARI.lords,
    reference: VIMSHOTTARI.reference,
    count: VIMSHOTTARI.count,
    span: VIMSHOTTARI.span,
    offset: VIMSHOTTARI.offset,
    repeats: VIMSHOTTARI.repeats,
    scale: Scale {
        numerator: 2,
        denominator: 3,
        rounds: 2,
    },
    groups: Cow::Borrowed(&[]),
    wheel: Wheel::Nakshatras,
};

/// Ashtottari as BPHS ch. 46 vv. 17 to 20 count it: the same eight lords
/// and years, from Ardra in groups of four and three alternately over the
/// twenty-eight nakshatras with Abhijit, so every seed has a lord and none
/// lies outside the cycle (crux C5). A nakshatra is a quarter of a
/// malefic's dasha and a third of a benefic's (v. 21), an equal share of
/// its lord's window either way.
pub const ASHTOTTARI_BPHS: UduRow = UduRow {
    groups: Cow::Borrowed(&[4, 3, 4, 3, 4, 3, 4, 3]),
    wheel: Wheel::WithAbhijit,
    ..ASHTOTTARI
};

/// Shashtihayani, BPHS ch. 46 vv. 40 and 41: Jupiter, the Sun and Mars ten
/// years each and the Moon, Mercury, Venus, Saturn and Rahu six, which is
/// the sixty the name states, from Ashvini in groups of three and four
/// alternately over the twenty-eight with Abhijit (crux C1). The received
/// English's "13 years" is contradicted by its own worked example and by
/// the verse, दशा दश दशाब्दकाः.
pub const SHASHTIHAYANI: UduRow = UduRow {
    system: DashaName::Catalogued(DashaSystem::Shashtihayani),
    lords: Cow::Borrowed(&[
        lord(Graha::Jupiter, 10),
        lord(Graha::Sun, 10),
        lord(Graha::Mars, 10),
        lord(Graha::Moon, 6),
        lord(Graha::Mercury, 6),
        lord(Graha::Venus, 6),
        lord(Graha::Saturn, 6),
        lord(Graha::Rahu, 6),
    ]),
    reference: 0,
    count: Count::FromReference,
    span: 1,
    offset: 0,
    repeats: false,
    scale: Scale::WHOLE,
    groups: Cow::Borrowed(&[3, 4, 3, 4, 3, 4, 3, 4]),
    wheel: Wheel::WithAbhijit,
};

/// Every row this build implements, in the catalogue's order. A system
/// the catalogue names and no row implements is refused by name.
pub const ROWS: &[UduRow] = &[
    VIMSHOTTARI,
    ASHTOTTARI,
    DWADASHOTTARI,
    PANCHOTTARI,
    SHATABDIKA,
    CHATURASHITI_SAMA,
    DWISAPTATI_SAMA,
    YOGINI,
    TRIBHAGI,
    SHASHTIHAYANI,
];

/// The row of a system, when this build implements one: Ashtottari's
/// is the one `grouping` names, and every other system has one.
#[must_use]
pub fn row(system: DashaSystem, grouping: AshtottariGrouping) -> Option<&'static UduRow> {
    if system == DashaSystem::Ashtottari && grouping == AshtottariGrouping::FourAndThree {
        return Some(&ASHTOTTARI_BPHS);
    }
    ROWS.iter().find(|row| row.system == system)
}

/// A consumer's nakshatra-seeded system, as a context registers it and a
/// document carries it: a K-udu row with a key of its own, its sources, and
/// the year length and depth the settings' per-system tables would give a
/// catalogued one.
///
/// Every field but `key`, `lords` and `reference` defaults to Vimshottari's
/// shape, so a definition says only how it differs.
///
/// ```
/// use teistro_core::catalogue::{Graha, Nakshatra};
/// use teistro_dasha::UduDefinition;
///
/// let saptaka: UduDefinition = serde_json::from_str(r#"{
///     "key": "ACME_SAPTAKA",
///     "sources": ["a consumer's own table"],
///     "lords": [
///         {"graha": "SUN", "years": 10}, {"graha": "MOON", "years": 10},
///         {"graha": "MARS", "years": 10}, {"graha": "MERCURY", "years": 10},
///         {"graha": "JUPITER", "years": 10}, {"graha": "VENUS", "years": 10},
///         {"graha": "SATURN", "years": 10}
///     ],
///     "reference": "KRITTIKA"
/// }"#)?;
/// assert_eq!(saptaka.row().total_years(), 70);
/// assert_eq!(saptaka.reference, Nakshatra::Krittika);
/// saptaka.row().validate()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UduDefinition {
    /// The key it is registered under, in the key grammar and not one the
    /// catalogue has.
    pub key: String,
    /// Where the table comes from.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    /// The lords, in the order they run.
    pub lords: Vec<Lord>,
    /// The nakshatra that maps to the first lord.
    pub reference: Nakshatra,
    /// Which way the seed is counted; forwards from the reference by default.
    #[serde(default)]
    pub count: Count,
    /// How many nakshatras each lord covers; one by default.
    #[serde(default = "one")]
    pub span: u8,
    /// How many nakshatras each lord covers, lord by lord, when they differ;
    /// none by default, when every lord covers `span`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<u8>,
    /// The circle the seed is counted round: the twenty-seven nakshatras by
    /// default, or the twenty-eight with Abhijit.
    #[serde(default, skip_serializing_if = "is_nakshatras")]
    pub wheel: Wheel,
    /// What is added after the division, before the modulo; none by default.
    #[serde(default)]
    pub offset: u8,
    /// Whether the lords run round the nakshatras again; they do by default.
    #[serde(default = "yes")]
    pub repeats: bool,
    /// The factor on the mahadashas' years and the rounds in a cycle; whole
    /// and once by default.
    #[serde(default)]
    pub scale: Scale,
    /// The length of its year; the Julian year by default, which every
    /// catalogued system takes unless the settings say otherwise.
    #[serde(default = "julian")]
    pub year_length: YearLength,
    /// How many levels of periods a reading carries; three by default.
    #[serde(default = "three")]
    pub depth: Depth,
}

const fn one() -> u8 {
    1
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde's `skip_serializing_if` hands the field by reference"
)]
fn is_nakshatras(wheel: &Wheel) -> bool {
    *wheel == Wheel::Nakshatras
}

const fn yes() -> bool {
    true
}

pub(crate) const fn julian() -> YearLength {
    YearLength::Julian36525
}

pub(crate) fn three() -> Depth {
    Depth::try_new(3).unwrap_or(Depth::MIN)
}

impl UduDefinition {
    /// A definition with a key and a reference and every other field its
    /// default: no lords yet, which a caller then gives.
    #[must_use]
    pub fn of(key: impl Into<String>, reference: Nakshatra) -> UduDefinition {
        UduDefinition {
            key: key.into(),
            sources: Vec::new(),
            lords: Vec::new(),
            reference,
            count: Count::FromReference,
            span: one(),
            groups: Vec::new(),
            wheel: Wheel::Nakshatras,
            offset: 0,
            repeats: yes(),
            scale: Scale::WHOLE,
            year_length: julian(),
            depth: three(),
        }
    }

    /// The row the kernel runs, its lords owned.
    #[must_use]
    pub fn row(&self) -> UduRow {
        UduRow {
            system: DashaName::Registered(self.key.clone()),
            lords: Cow::Owned(self.lords.clone()),
            reference: u8::try_from(self.reference.id()).unwrap_or(NAKSHATRAS),
            count: self.count,
            span: self.span,
            offset: self.offset,
            repeats: self.repeats,
            scale: self.scale,
            groups: Cow::Owned(self.groups.clone()),
            wheel: self.wheel,
        }
    }
}

impl teistro_core::registry::Definition for UduDefinition {
    fn key(&self) -> &str {
        &self.key
    }

    fn validate(&self) -> Result<(), Error> {
        self.row().validate()
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::*;

    #[test]
    fn every_shipped_row_passes_its_checks_and_sums_to_its_cycle() {
        for row in ROWS {
            row.validate().unwrap();
        }
        let totals: Vec<u32> = ROWS.iter().map(UduRow::total_years).collect();
        assert_eq!(totals, [120, 108, 112, 105, 100, 84, 72, 36, 120, 60]);
        ASHTOTTARI_BPHS.validate().unwrap();
        assert_eq!(ASHTOTTARI_BPHS.total_years(), 108);
        assert_eq!(TRIBHAGI.mahadashas(), 18);
        assert!((TRIBHAGI.scaled_years(0) - 7.0 * 2.0 / 3.0).abs() < 1e-12);
        // In the catalogue's order, one row a system.
        let ids: Vec<u16> = ROWS
            .iter()
            .filter_map(|row| row.system.catalogued())
            .map(DashaSystem::id)
            .collect();
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]), "{ids:?}");
    }

    #[test]
    fn every_row_seats_every_nakshatra_and_only_ashtottari_overflows() {
        for row in ROWS.iter().chain([&ASHTOTTARI_BPHS]) {
            for place in 0..row.wheel.size() {
                let seat = row.seat(place);
                assert!(seat.lord < row.lords.len());
                assert!(seat.within < row.window_of(seat.lord));
                // Only the recording engine's Ashtottari, three each over
                // twenty-four, leaves seeds outside its cycle.
                assert_eq!(
                    seat.overflow,
                    row.system == DashaSystem::Ashtottari
                        && row.wheel == Wheel::Nakshatras
                        && (2..5).contains(&place),
                    "{:?} at {place}",
                    row.system
                );
            }
        }
        // Ashwini is Yogini's fourth lord, Mars; Revati is Dwadashottari's
        // first, the Sun.
        assert_eq!(YOGINI.lords[YOGINI.seat(0).lord].graha, Graha::Mars);
        assert_eq!(DWADASHOTTARI.seat(26).lord, 0);
        assert_eq!(DWADASHOTTARI.seat(25).lord, 1);
    }

    /// BPHS ch. 46, the translation's note 2 to v. 23: a birth in
    /// Mrigashira falls in Venus's Ashtottari, Krittika and Rohini behind
    /// it, and Mrigashira is a third of Venus's 21 years.
    #[test]
    fn the_texts_ashtottari_seats_mrigashira_where_bphs_does() {
        let row = &ASHTOTTARI_BPHS;
        let mrigashira = row.wheel.place_of(4);
        let seat = row.seat(mrigashira);
        assert_eq!(row.lords[seat.lord].graha, Graha::Venus);
        assert_eq!((seat.within, row.window_of(seat.lord)), (2, 3));
        assert!(!seat.overflow);
        // The whole table, place by place on the twenty-eight: the Sun from
        // Ardra, Saturn's four taking Abhijit, Rahu's running past Revati
        // into Ashvini and Bharani.
        let lords: Vec<Graha> = (0..28)
            .map(|place| row.lords[row.seat(place).lord].graha)
            .collect();
        let (su, mo, ma, me, sa, ju, ra, ve) = (
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Saturn,
            Graha::Jupiter,
            Graha::Rahu,
            Graha::Venus,
        );
        assert_eq!(
            lords,
            [
                ra, ra, ve, ve, ve, su, su, su, su, mo, mo, mo, ma, ma, ma, ma, me, me, me, sa, sa,
                sa, sa, ju, ju, ju, ra, ra
            ]
        );
    }

    /// BPHS ch. 46 vv. 40 and 41 and the translation's example beside them:
    /// from Ashvini in threes and fours, Mrigashira is the Sun's with Rohini
    /// behind it, four nakshatras of ten years, thirty months each.
    #[test]
    fn shashtihayani_counts_sixty_years_as_the_verse_does() {
        let row = &SHASHTIHAYANI;
        assert_eq!(row.total_years(), 60);
        let seat = row.seat(row.wheel.place_of(4));
        assert_eq!(row.lords[seat.lord].graha, Graha::Sun);
        assert_eq!((seat.within, row.window_of(seat.lord)), (1, 4));
        assert_eq!(row.lords[seat.lord].years, 10);
        assert_eq!(row.lords[row.seat(0).lord].graha, Graha::Jupiter);
        assert_eq!(row.lords[row.seat(21).lord].graha, Graha::Saturn, "Abhijit");
        assert_eq!(row.lords[row.seat(27).lord].graha, Graha::Rahu, "Revati");
    }

    #[test]
    fn a_group_count_that_does_not_fit_is_refused_by_its_field() {
        let refused = |row: UduRow| row.validate().unwrap_err().field().map(str::to_owned);
        let short = UduRow {
            groups: Cow::Borrowed(&[4, 3]),
            ..ASHTOTTARI_BPHS
        };
        assert_eq!(refused(short).as_deref(), Some("groups"));
        let empty = UduRow {
            groups: Cow::Borrowed(&[4, 3, 4, 3, 4, 3, 4, 0]),
            ..ASHTOTTARI_BPHS
        };
        assert_eq!(refused(empty).as_deref(), Some("groups[7]"));
        let over = UduRow {
            groups: Cow::Borrowed(&[4, 4, 4, 4, 4, 4, 4, 4]),
            ..ASHTOTTARI_BPHS
        };
        assert_eq!(refused(over).as_deref(), Some("groups"));
        // Twenty-eight is too many on the twenty-seven.
        let unwheeled = UduRow {
            wheel: Wheel::Nakshatras,
            ..ASHTOTTARI_BPHS
        };
        assert_eq!(refused(unwheeled).as_deref(), Some("groups"));
    }

    #[test]
    fn vimshottari_seats_every_nakshatra_once_round_the_nine() {
        for nakshatra in 0..NAKSHATRAS {
            let seat = VIMSHOTTARI.seat(nakshatra);
            assert_eq!(seat.lord, usize::from(nakshatra) % 9);
            assert_eq!(seat.within, 0);
            assert!(!seat.overflow);
        }
        // Anuradha, the seventeenth, is Saturn's.
        assert_eq!(
            VIMSHOTTARI.lords[VIMSHOTTARI.seat(16).lord].graha,
            Graha::Saturn
        );
    }

    #[test]
    fn a_wide_window_a_backward_count_and_an_offset_each_move_the_seat() {
        let wide = UduRow {
            reference: 5,
            span: 3,
            lords: Cow::Owned(VIMSHOTTARI.lords[..8].to_vec()),
            repeats: false,
            ..VIMSHOTTARI
        };
        // Ardra itself starts the first lord's window; two past it is still
        // the first lord, two nakshatras into its window.
        assert_eq!(
            wide.seat(5),
            Seat {
                lord: 0,
                within: 0,
                overflow: false
            }
        );
        assert_eq!(
            wide.seat(7),
            Seat {
                lord: 0,
                within: 2,
                overflow: false
            }
        );
        assert_eq!(wide.seat(8).lord, 1);
        // Eight lords of three cover 24; the three before Ardra overflow.
        assert!(wide.seat(4).overflow);
        let backward = UduRow {
            reference: 26,
            count: Count::ToReference,
            ..VIMSHOTTARI
        };
        assert_eq!(backward.seat(26).lord, 0);
        assert_eq!(backward.seat(25).lord, 1);
        let offset = UduRow {
            offset: 3,
            ..VIMSHOTTARI
        };
        assert_eq!(offset.seat(0).lord, 3);
    }

    #[test]
    fn a_row_is_refused_by_the_field_it_gets_wrong() {
        const IDLE: &[Lord] = &[lord(Graha::Sun, 0)];
        let empty = UduRow {
            lords: Cow::Borrowed(&[]),
            ..VIMSHOTTARI
        };
        assert_eq!(empty.validate().unwrap_err().field(), Some("lords"));
        let far = UduRow {
            reference: 27,
            ..VIMSHOTTARI
        };
        assert_eq!(far.validate().unwrap_err().field(), Some("reference"));
        let wide = UduRow {
            span: 4,
            ..VIMSHOTTARI
        };
        assert_eq!(wide.validate().unwrap_err().field(), Some("span"));
        let idle = UduRow {
            lords: Cow::Borrowed(IDLE),
            ..VIMSHOTTARI
        };
        assert_eq!(idle.validate().unwrap_err().field(), Some("lords[0].years"));
        let nothing = UduRow {
            scale: Scale {
                numerator: 0,
                ..Scale::WHOLE
            },
            ..VIMSHOTTARI
        };
        assert_eq!(nothing.validate().unwrap_err().field(), Some("scale"));
        let endless = UduRow {
            scale: Scale {
                rounds: 29,
                ..Scale::WHOLE
            },
            ..VIMSHOTTARI
        };
        assert_eq!(
            endless.validate().unwrap_err().field(),
            Some("scale.rounds")
        );
    }
}

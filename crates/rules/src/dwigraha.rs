//! The dwigraha generator: two texts' readings of one shape, built from a
//! table of what changes (`03-design/rules-engine.md`, "One shape, ninety-three
//! rules").
//!
//! Brihat Jataka ch. 14 reads every pair of the seven grahas sharing a sign,
//! and Phaladeepika ch. 18 the Moon in every sign under one graha's aspect.
//! Neither grades anything: each verse says in words what follows, so every
//! rule here carries an [`Outcome::Effect`] and no severity and no span. The
//! conditions repeat, so only the pair, the sign, the aspecting graha and the
//! reading are data, and the ninety-three rules are built from them.

use serde::Deserialize;
use teistro_core::catalogue::{Graha, Rashi};

use crate::language::{Body, Condition, EvidenceRank, Source};
use crate::reference::{BodyRef, BodySubject, SignRef};
use crate::rule::{Outcome, Rule};

/// The Moon, whom every rule of the second family is about.
const MOON: Body = Body::Graha(Graha::Moon);

/// The table the generator reads.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Table {
    dwigraha: Dwigraha,
    moon_aspected: MoonAspected,
}

/// What every rule of a family shares: its text, and how it was read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Family {
    text: String,
    chapter: String,
    rank: u8,
    note: String,
}

/// Brihat Jataka ch. 14: two grahas in one sign.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Dwigraha {
    #[serde(flatten)]
    family: Family,
    readings: Vec<Pair>,
}

/// One pair, and what its verse says of the native.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pair {
    verse: String,
    planets: [Body; 2],
    effect: String,
}

/// Phaladeepika ch. 18: the Moon in a sign, aspected.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoonAspected {
    #[serde(flatten)]
    family: Family,
    /// The grahas the verses list, in the order they list them.
    aspects: Vec<Body>,
    signs: Vec<SignRow>,
}

/// One sign's row: a reading for each graha of `aspects`, in that order.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SignRow {
    verse: String,
    sign: Rashi,
    effects: Vec<String>,
}

impl Family {
    /// The citation a rule of this family carries, with its own verse.
    fn source(&self, verse: &str) -> Source {
        Source {
            text: self.text.clone(),
            chapter: Some(self.chapter.clone()),
            verse: Some(verse.to_owned()),
            note: Some(self.note.clone()),
            rank: EvidenceRank::try_new(self.rank).ok(),
        }
    }
}

/// A rule whose verse states its effect in words.
fn stating(
    key: String,
    category: &str,
    source: Source,
    conditions: Vec<Condition>,
    effect: &str,
) -> Rule {
    Rule {
        conditions,
        outcome: Some(Outcome::Effect {
            text: effect.to_owned(),
        }),
        ..Rule::new(key, category, source)
    }
}

impl Table {
    /// Every rule the table stands for: the twenty-one pairs of Brihat Jataka
    /// ch. 14, then Phaladeepika ch. 18's Moon in each sign under each of the
    /// six aspects, sign by sign and in the order the verses list them.
    pub(crate) fn rules(&self) -> Vec<Rule> {
        let pairs = self.dwigraha.readings.iter().map(|pair| {
            let [one, two] = pair.planets;
            stating(
                format!("DWIGRAHA_{}_{}", one.key(), two.key()),
                "dwigraha",
                self.dwigraha.family.source(&pair.verse),
                vec![Condition::PlanetConjunct {
                    planets: pair.planets.iter().copied().map(BodyRef::Body).collect(),
                    max_orb: None,
                }],
                &pair.effect,
            )
        });
        let moon = self.moon_aspected.signs.iter().flat_map(|row| {
            self.moon_aspected
                .aspects
                .iter()
                .zip(&row.effects)
                .map(move |(from, effect)| {
                    stating(
                        format!("CHANDRA_IN_{}_ASPECTED_BY_{}", row.sign.key(), from.key()),
                        "chandra-drishti",
                        self.moon_aspected.family.source(&row.verse),
                        vec![
                            Condition::PlanetInSign {
                                planet: SignRef::Of(BodyRef::Body(MOON)),
                                signs: vec![row.sign],
                            },
                            Condition::PlanetAspectsPlanet {
                                from: BodySubject::Ref(BodyRef::Body(*from)),
                                target: SignRef::Of(BodyRef::Body(MOON)),
                            },
                        ],
                        effect,
                    )
                })
        });
        pairs.chain(moon).collect()
    }
}

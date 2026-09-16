//! The readings generator: the texts' families of one shape, built from a
//! table of what changes (`03-design/rules-engine.md`, "One shape, many
//! rules").
//!
//! Brihat Jataka ch. 14 reads every pair of the seven grahas sharing a sign;
//! Phaladeepika ch. 18 the Moon in every sign under one graha's aspect; Jataka
//! Parijata every combination of the seven from two to six sharing a sign; and
//! Saravali a graha in each of the twelve signs (chs. 22 to 29) and in each of
//! the twelve bhavas (ch. 30). None of them grades anything: each verse says
//! in words what follows, so every rule here carries an [`Outcome::Effect`]
//! and no severity and no span. The conditions repeat, so only what changes is
//! data — the pair, the sign, the aspecting graha, the house and the reading —
//! and the rules are built from it.

use serde::Deserialize;
use teistro_core::catalogue::{Graha, Rashi};

use crate::language::{Body, Condition, EvidenceRank, House, Source};
use crate::reference::{BodyRef, BodySubject, SignRef, Subject};
use crate::rule::{Outcome, Rule};

/// The Moon, whom every rule of the second family is about.
const MOON: Body = Body::Graha(Graha::Moon);

/// The table the generator reads.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Table {
    dwigraha: Dwigraha,
    moon_aspected: MoonAspected,
    together: Together,
    in_rasi: InRasi,
    in_bhava: InBhava,
}

/// What every rule of a family shares: its text, and how it was read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Family {
    text: String,
    /// The chapter, when every rule of the family shares one.
    #[serde(default)]
    chapter: Option<String>,
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

/// Jataka Parijata's lists: two to six of the seven sharing one sign.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Together {
    #[serde(flatten)]
    family: Family,
    sets: Vec<Set>,
}

/// One list: a reading for each combination of that size, in the order the
/// appendix prints them, which is the combinations' own order.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Set {
    size: usize,
    /// Which section of the appendix prints it.
    section: String,
    effects: Vec<String>,
}

/// Saravali's chapters of a graha in the twelve signs.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InRasi {
    #[serde(flatten)]
    family: Family,
    grahas: Vec<GrahaRows>,
}

/// Saravali ch. 30, a graha in the twelve bhavas.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InBhava {
    #[serde(flatten)]
    family: Family,
    grahas: Vec<BhavaRows>,
}

/// One graha's readings, a house at a time from the ascendant.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BhavaRows {
    planet: Body,
    readings: Vec<Reading>,
}

/// One graha's chapter: a reading for each sign, from Aries.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GrahaRows {
    planet: Body,
    chapter: String,
    readings: Vec<Reading>,
}

/// One sign's reading, and the verses it stands on.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reading {
    verse: String,
    effect: String,
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
        self.in_chapter(self.chapter.clone(), verse)
    }

    /// The same, for a family whose chapter changes rule by rule.
    fn in_chapter(&self, chapter: Option<String>, verse: &str) -> Source {
        Source {
            text: self.text.clone(),
            chapter,
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
        pairs
            .chain(moon)
            .chain(self.together())
            .chain(self.in_rasi())
            .chain(self.in_bhava())
            .collect()
    }

    /// Jataka Parijata's lists, whose grahas are generated: the combinations of
    /// the seven of each size, in order, one reading each.
    fn together(&self) -> impl Iterator<Item = Rule> {
        let family = &self.together.family;
        self.together.sets.iter().flat_map(move |set| {
            combinations(&Body::SEVEN, set.size)
                .into_iter()
                .zip(&set.effects)
                .enumerate()
                .map(move |(at, (grahas, effect))| {
                    let keys: Vec<&str> = grahas.iter().map(|body| body.key()).collect();
                    stating(
                        format!("PARIJATA_TOGETHER_{}", keys.join("_")),
                        "grahas-together",
                        family.source(&format!("{} {}", set.section, at + 1)),
                        vec![Condition::PlanetConjunct {
                            planets: grahas.iter().copied().map(BodyRef::Body).collect(),
                            max_orb: None,
                        }],
                        effect,
                    )
                })
        })
    }
}

impl Table {
    /// Saravali's chapters: each graha in each of the twelve signs, the signs
    /// in the catalogue's order, which is the chapters' order from Aries.
    fn in_rasi(&self) -> impl Iterator<Item = Rule> {
        let family = &self.in_rasi.family;
        self.in_rasi.grahas.iter().flat_map(move |graha| {
            Rashi::ALL
                .into_iter()
                .zip(&graha.readings)
                .map(move |(sign, reading)| {
                    stating(
                        format!("SARAVALI_{}_IN_{}", graha.planet.key(), sign.key()),
                        "graha-in-rasi",
                        family.in_chapter(Some(graha.chapter.clone()), &reading.verse),
                        vec![Condition::PlanetInSign {
                            planet: SignRef::Of(BodyRef::Body(graha.planet)),
                            signs: vec![sign],
                        }],
                        &reading.effect,
                    )
                })
        })
    }
}

impl Table {
    /// Saravali ch. 30: each graha in each of the twelve bhavas, the houses in
    /// order from the ascendant, as the verses run.
    fn in_bhava(&self) -> impl Iterator<Item = Rule> {
        let family = &self.in_bhava.family;
        self.in_bhava.grahas.iter().flat_map(move |graha| {
            graha
                .readings
                .iter()
                .enumerate()
                .filter_map(move |(at, reading)| {
                    let house = House::try_new(u8::try_from(at).ok()? + 1).ok()?;
                    Some(stating(
                        format!("SARAVALI_{}_IN_BHAVA_{}", graha.planet.key(), house.get()),
                        "graha-in-bhava",
                        family.source(&reading.verse),
                        vec![Condition::PlanetInHouse {
                            planet: Subject::Ref(SignRef::Of(BodyRef::Body(graha.planet))),
                            houses: vec![house],
                        }],
                        &reading.effect,
                    ))
                })
        })
    }
}

/// The combinations of a size, in the bodies' own order: the appendix's order,
/// and the only one that lets a reading be data and its grahas be generated.
fn combinations(bodies: &[Body], size: usize) -> Vec<Vec<Body>> {
    if size == 0 {
        return vec![Vec::new()];
    }
    let mut built = Vec::new();
    let mut rest = bodies;
    while let Some((body, after)) = rest.split_first() {
        for mut tail in combinations(after, size - 1) {
            let mut one = vec![*body];
            one.append(&mut tail);
            built.push(one);
        }
        rest = after;
    }
    built
}

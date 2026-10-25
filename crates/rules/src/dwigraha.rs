//! The readings generator: the texts' families of one shape, built from a
//! table of what changes (`03-design/rules-engine.md`, "One shape, many
//! rules").
//!
//! Brihat Jataka ch. 14 reads every pair of the seven grahas sharing a sign;
//! Phaladeepika ch. 18 the Moon in every sign under one graha's aspect; Jataka
//! Parijata every combination of the seven from two to six sharing a sign; and
//! Saravali the same combinations (chs. 15 to 19), a graha in each of the
//! twelve signs (chs. 22 to 29), the same
//! graha there under each other graha's aspect, and a graha in each of the
//! twelve bhavas (ch. 30); and Brihat Jataka chs. 16 to 20 the Moon in each
//! nakshatra, each graha in each sign, the Moon there and in each navamsa under
//! each graha's aspect, and each graha in each bhava. None of them grades anything: each verse says
//! in words what follows, so every rule here carries an [`Outcome::Effect`]
//! and no severity and no span. The conditions repeat, so only what changes is
//! data — the pair, the sign, the aspecting graha, the house and the reading —
//! and the rules are built from it.

use serde::Deserialize;
use teistro_core::catalogue::{Graha, Nakshatra, Rashi, Varga};

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
    saravali_together: SaravaliTogether,
    in_rasi: InRasi,
    aspected_in_rasi: AspectedInRasi,
    in_bhava: InBhava,
    rising_part: RisingPart,
    pair_in_angle: PairInAngle,
    bj_moon_in_nakshatra: BjMoonInNakshatra,
    bj_in_rasi: BjInRasi,
    bj_moon_aspected: BjMoonAspected,
    bj_navamsa_aspected: BjNavamsaAspected,
    bj_in_bhava: BjInBhava,
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

/// Saravali chs. 15 to 19: two to six of the seven sharing one sign, a
/// chapter for each size.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaravaliTogether {
    #[serde(flatten)]
    family: Family,
    chapters: Vec<Listed>,
}

/// One chapter's sets. Its verses name their grahas, and ch. 18's do not
/// keep the combinations' order, so each reading carries its grahas rather
/// than taking them from its place.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Listed {
    chapter: String,
    readings: Vec<Named>,
}

/// One verse, the grahas it names in the seven's own order, and what it says
/// of the native.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Named {
    verse: String,
    planets: Vec<Body>,
    effect: String,
}

/// Saravali's chapters of a graha in the twelve signs.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InRasi {
    #[serde(flatten)]
    family: Family,
    grahas: Vec<GrahaRows>,
}

/// Saravali chs. 22 to 29 again: a graha in a sign under another's aspect.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AspectedInRasi {
    #[serde(flatten)]
    family: Family,
    grahas: Vec<AspectedRows>,
}

/// One graha's aspect verses: its signs in the groups the chapter reads.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AspectedRows {
    planet: Body,
    chapter: String,
    groups: Vec<SignGroup>,
}

/// The signs one run of verses covers — a lord's two, or the Moon's one —
/// and a reading under each graha that aspects the graha there.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SignGroup {
    signs: Vec<Rashi>,
    readings: Vec<AspectedReading>,
}

/// What a verse says of the graha under one aspect.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AspectedReading {
    from: Body,
    verse: String,
    effect: String,
}

/// Saravali ch. 30, a graha in the twelve bhavas.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InBhava {
    #[serde(flatten)]
    family: Family,
    grahas: Vec<BhavaRows>,
}

/// Saravali chs. 48 to 51: the sign that rises and the part of it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RisingPart {
    #[serde(flatten)]
    family: Family,
    parts: Vec<PartRows>,
    /// Divisions that read another division's readings rather than their
    /// own: ch. 51 v. 110's dwadasamsas, which say the signs'.
    carried: Vec<Carried>,
}

/// A division of each sign whose parts take the readings of a whole-sign
/// division (`count` 1), the n-th part the reading of the sign n − 1 on
/// from the sign it divides, as the dwadasamsas count (Saravali ch. 3 v. 13).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Carried {
    /// What the division is called, which names the rules.
    name: String,
    chapter: String,
    verse: String,
    /// How many parts a sign is cut into.
    count: u8,
    /// The name of the whole-sign division whose readings it takes.
    from: String,
    /// How the carrying was read, in place of the family's note.
    note: String,
}

/// Saravali ch. 31: two grahas sharing a named house.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PairInAngle {
    #[serde(flatten)]
    family: Family,
    /// The houses the chapter reads, in its order.
    angles: Vec<House>,
    pairs: Vec<PartSign>,
}

/// One division of a sign — halves, thirds — and its chapter.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartRows {
    /// What the division is called, which names the rules.
    name: String,
    chapter: String,
    /// How many parts a sign is cut into.
    count: u8,
    signs: Vec<PartSign>,
}

/// One sign's readings, a part at a time from its beginning.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartSign {
    verse: String,
    effects: Vec<String>,
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

/// Brihat Jataka ch. 16: the Moon in each nakshatra.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BjMoonInNakshatra {
    #[serde(flatten)]
    family: Family,
    readings: Vec<NakshatraReading>,
}

/// One nakshatra's reading.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NakshatraReading {
    verse: String,
    nakshatra: Nakshatra,
    effect: String,
}

/// Brihat Jataka chs. 17 and 18: each graha in the signs, the Moon's chapter
/// a sign to a reading and the others' often a lord's two signs at once.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BjInRasi {
    #[serde(flatten)]
    family: Family,
    grahas: Vec<SignedRows>,
}

/// One graha's readings, each naming the signs it is read in.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SignedRows {
    planet: Body,
    chapter: String,
    readings: Vec<SignedReading>,
}

/// A reading of the graha in any of `signs`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SignedReading {
    verse: String,
    signs: Vec<Rashi>,
    effect: String,
}

/// Brihat Jataka ch. 19 vv. 1 to 3: the Moon in a sign under each graha's
/// aspect, in each verse's own order of the grahas.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BjMoonAspected {
    #[serde(flatten)]
    family: Family,
    signs: Vec<AspectedSign>,
}

/// One sign's readings, a graha's aspect to a reading.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AspectedSign {
    verse: String,
    sign: Rashi,
    readings: Vec<Aspecting>,
}

/// What a verse says under one graha's aspect.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Aspecting {
    from: Body,
    effect: String,
}

/// Brihat Jataka ch. 19 vv. 5 to 8: the Moon in a lord's navamsa under each
/// graha's aspect, which v. 8 reads for the Sun too.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BjNavamsaAspected {
    #[serde(flatten)]
    family: Family,
    navamsas: Vec<NavamsaGroup>,
}

/// One lord's navamsas: the signs whose navamsa it is, and a reading under
/// each graha's aspect.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NavamsaGroup {
    verse: String,
    lord: Body,
    signs: Vec<Rashi>,
    readings: Vec<Aspecting>,
}

/// Brihat Jataka ch. 20: each graha in the twelve bhavas, with the readings a
/// verse changes when a named sign rises.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BjInBhava {
    #[serde(flatten)]
    family: Family,
    grahas: Vec<BjBhavaRows>,
}

/// One graha's houses from the ascendant, and its rising-sign variants.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BjBhavaRows {
    planet: Body,
    readings: Vec<Reading>,
    variants: Vec<Variant>,
}

/// A reading of the graha in `house` when one of `lagnas` rises.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Variant {
    verse: String,
    house: House,
    lagnas: Vec<Rashi>,
    effect: String,
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
        outcomes: vec![Outcome::Effect {
            text: effect.to_owned(),
        }],
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
            .chain(self.saravali_together())
            .chain(self.in_rasi())
            .chain(self.aspected_in_rasi())
            .chain(self.in_bhava())
            .chain(self.rising_part())
            .chain(self.pair_in_angle())
            .chain(self.bj_moon_in_nakshatra())
            .chain(self.bj_in_rasi())
            .chain(self.bj_moon_aspected())
            .chain(self.bj_navamsa_aspected())
            .chain(self.bj_in_bhava())
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
    /// Saravali's sets, one rule for each verse, named by its grahas.
    fn saravali_together(&self) -> impl Iterator<Item = Rule> {
        let family = &self.saravali_together.family;
        self.saravali_together
            .chapters
            .iter()
            .flat_map(move |listed| {
                listed.readings.iter().map(move |reading| {
                    let keys: Vec<&str> = reading.planets.iter().map(|body| body.key()).collect();
                    stating(
                        format!("SARAVALI_TOGETHER_{}", keys.join("_")),
                        "grahas-together",
                        family.in_chapter(Some(listed.chapter.clone()), &reading.verse),
                        vec![Condition::PlanetConjunct {
                            planets: reading.planets.iter().copied().map(BodyRef::Body).collect(),
                            max_orb: None,
                        }],
                        &reading.effect,
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
    /// Saravali chs. 22 to 29: each graha in each sign under each graha that
    /// can aspect it there. A verse reads a lord's two signs at once, so it
    /// stands for a rule in each of them, and the key names the one sign.
    fn aspected_in_rasi(&self) -> impl Iterator<Item = Rule> {
        let family = &self.aspected_in_rasi.family;
        self.aspected_in_rasi.grahas.iter().flat_map(move |graha| {
            let planet = graha.planet;
            graha.groups.iter().flat_map(move |group| {
                group.signs.iter().flat_map(move |sign| {
                    group.readings.iter().map(move |reading| {
                        stating(
                            format!(
                                "SARAVALI_{}_IN_{}_ASPECTED_BY_{}",
                                planet.key(),
                                sign.key(),
                                reading.from.key()
                            ),
                            "graha-aspected-in-rasi",
                            family.in_chapter(Some(graha.chapter.clone()), &reading.verse),
                            vec![
                                Condition::PlanetInSign {
                                    planet: SignRef::Of(BodyRef::Body(planet)),
                                    signs: vec![*sign],
                                },
                                Condition::PlanetAspectsPlanet {
                                    from: BodySubject::Ref(BodyRef::Body(reading.from)),
                                    target: SignRef::Of(BodyRef::Body(planet)),
                                },
                            ],
                            &reading.effect,
                        )
                    })
                })
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

impl Table {
    /// Saravali chs. 48 to 51: the lagna in a sign, or in a part of it, the
    /// signs in the catalogue's order and the parts in order from each
    /// sign's start; then the divisions that carry a whole sign's readings.
    fn rising_part(&self) -> impl Iterator<Item = Rule> {
        let family = &self.rising_part.family;
        let own = self.rising_part.parts.iter().flat_map(move |part| {
            let width = 30.0 / f64::from(part.count);
            Rashi::ALL
                .into_iter()
                .zip(&part.signs)
                .flat_map(move |(sign, row)| {
                    row.effects.iter().enumerate().map(move |(at, effect)| {
                        let nth = at + 1;
                        // A sign has at most twelve parts in any of these
                        // chapters, so the count fits a byte and its degrees
                        // an f64 exactly.
                        let (at, nth) = (
                            u8::try_from(at).unwrap_or(0),
                            u8::try_from(nth).unwrap_or(0),
                        );
                        // A whole sign is not a part of one, and is named
                        // as the sign alone.
                        let key = if part.count == 1 {
                            format!("SARAVALI_{}_{}", part.name, sign.key())
                        } else {
                            format!("SARAVALI_{}_{nth}_OF_{}", part.name, sign.key())
                        };
                        stating(
                            key,
                            "rising-part",
                            family.in_chapter(Some(part.chapter.clone()), &row.verse),
                            vec![
                                Condition::PlanetInSign {
                                    planet: SignRef::Of(BodyRef::Body(Body::Lagna)),
                                    signs: vec![sign],
                                },
                                Condition::PlanetInDegrees {
                                    planet: BodyRef::Body(Body::Lagna),
                                    from: width * f64::from(at),
                                    to: width * f64::from(nth),
                                },
                            ],
                            effect,
                        )
                    })
                })
        });
        own.chain(self.carried())
    }

    /// Ch. 51 v. 110: each dwadasamsa of a sign reads as the lagna in the
    /// sign it counts to, so the twelfth of Aries says what Pisces rising
    /// says. A division whose source is missing carries nothing, which the
    /// generator's test refuses.
    fn carried(&self) -> impl Iterator<Item = Rule> {
        let family = &self.rising_part.family;
        self.rising_part.carried.iter().flat_map(move |carried| {
            let whole = self
                .rising_part
                .parts
                .iter()
                .find(|part| part.name == carried.from)
                .filter(|part| part.count == 1);
            let width = 30.0 / f64::from(carried.count);
            Rashi::ALL
                .into_iter()
                .enumerate()
                .flat_map(move |(from, sign)| {
                    (0..carried.count).filter_map(move |at| {
                        let read = (from + usize::from(at)) % Rashi::ALL.len();
                        let row = whole?.signs.get(read)?;
                        let effect = row.effects.first()?;
                        let nth = at + 1;
                        Some(stating(
                            format!("SARAVALI_{}_{nth}_OF_{}", carried.name, sign.key()),
                            "rising-part",
                            Source {
                                note: Some(carried.note.clone()),
                                ..family.in_chapter(Some(carried.chapter.clone()), &carried.verse)
                            },
                            vec![
                                Condition::PlanetInSign {
                                    planet: SignRef::Of(BodyRef::Body(Body::Lagna)),
                                    signs: vec![sign],
                                },
                                Condition::PlanetInDegrees {
                                    planet: BodyRef::Body(Body::Lagna),
                                    from: width * f64::from(at),
                                    to: width * f64::from(nth),
                                },
                            ],
                            effect,
                        ))
                    })
                })
        })
    }
}

impl Table {
    /// Saravali ch. 31: each pair of the seven in each of the four angles. The
    /// pair is generated, as Jataka Parijata's assemblies are, and the house is
    /// pinned by a second condition — a conjunction says one sign, and
    /// `planet-in-house` says which sign that is.
    fn pair_in_angle(&self) -> impl Iterator<Item = Rule> {
        let family = &self.pair_in_angle.family;
        let angles = &self.pair_in_angle.angles;
        combinations(&Body::SEVEN, 2)
            .into_iter()
            .zip(&self.pair_in_angle.pairs)
            .filter_map(|(pair, row)| pair.first().copied().map(|first| (pair, first, row)))
            .flat_map(move |(pair, first, row)| {
                angles.iter().zip(&row.effects).map(move |(house, effect)| {
                    let keys: Vec<&str> = pair.iter().map(|body| body.key()).collect();
                    stating(
                        format!("SARAVALI_{}_IN_BHAVA_{}", keys.join("_"), house.get()),
                        "pair-in-angle",
                        family.source(&row.verse),
                        vec![
                            Condition::PlanetConjunct {
                                planets: pair.iter().copied().map(BodyRef::Body).collect(),
                                max_orb: None,
                            },
                            Condition::PlanetInHouse {
                                planet: Subject::Ref(SignRef::Of(BodyRef::Body(first))),
                                houses: vec![*house],
                            },
                        ],
                        effect,
                    )
                })
            })
    }
}

impl Table {
    /// Brihat Jataka ch. 16: the Moon in each of the twenty-seven nakshatras.
    fn bj_moon_in_nakshatra(&self) -> impl Iterator<Item = Rule> {
        let family = &self.bj_moon_in_nakshatra.family;
        self.bj_moon_in_nakshatra
            .readings
            .iter()
            .map(move |reading| {
                stating(
                    format!("BJ_MOON_IN_{}", reading.nakshatra.key()),
                    "chandra-nakshatra",
                    family.source(&reading.verse),
                    vec![Condition::PlanetInNakshatra {
                        planet: BodyRef::Body(MOON),
                        nakshatras: vec![reading.nakshatra],
                        padas: Vec::new(),
                    }],
                    &reading.effect,
                )
            })
    }

    /// Brihat Jataka chs. 17 and 18: each graha in each sign. A verse reading
    /// a lord's two signs at once stands for a rule in each, and the key names
    /// the one sign.
    fn bj_in_rasi(&self) -> impl Iterator<Item = Rule> {
        let family = &self.bj_in_rasi.family;
        self.bj_in_rasi.grahas.iter().flat_map(move |graha| {
            graha.readings.iter().flat_map(move |reading| {
                reading.signs.iter().map(move |sign| {
                    stating(
                        format!("BJ_{}_IN_{}", graha.planet.key(), sign.key()),
                        "graha-in-rasi",
                        family.in_chapter(Some(graha.chapter.clone()), &reading.verse),
                        vec![Condition::PlanetInSign {
                            planet: SignRef::Of(BodyRef::Body(graha.planet)),
                            signs: vec![*sign],
                        }],
                        &reading.effect,
                    )
                })
            })
        })
    }

    /// Brihat Jataka ch. 19 vv. 1 to 3: the Moon in each sign under each
    /// graha's aspect, cast in the rasi chart.
    fn bj_moon_aspected(&self) -> impl Iterator<Item = Rule> {
        let family = &self.bj_moon_aspected.family;
        self.bj_moon_aspected.signs.iter().flat_map(move |row| {
            row.readings.iter().map(move |reading| {
                stating(
                    format!(
                        "BJ_MOON_IN_{}_ASPECTED_BY_{}",
                        row.sign.key(),
                        reading.from.key()
                    ),
                    "chandra-drishti",
                    family.source(&row.verse),
                    vec![
                        Condition::PlanetInSign {
                            planet: SignRef::Of(BodyRef::Body(MOON)),
                            signs: vec![row.sign],
                        },
                        aspected_by(reading.from, MOON),
                    ],
                    &reading.effect,
                )
            })
        })
    }

    /// Brihat Jataka ch. 19 vv. 5 to 8: the Moon in each lord's navamsa under
    /// each graha's aspect, cast in the rasi chart; then v. 8's Sun in the same
    /// navamsas, the Moon's aspect read where the Sun's is named. Mercury and
    /// Venus never stand seventh from the Sun, so the Sun has no rules under
    /// their aspects.
    fn bj_navamsa_aspected(&self) -> impl Iterator<Item = Rule> {
        const SUN: Body = Body::Graha(Graha::Sun);
        let family = &self.bj_navamsa_aspected.family;
        [MOON, SUN].into_iter().flat_map(move |planet| {
            self.bj_navamsa_aspected
                .navamsas
                .iter()
                .flat_map(move |group| {
                    group.readings.iter().filter_map(move |reading| {
                        let from = match (planet, reading.from) {
                            (SUN, SUN) => MOON,
                            (SUN, Body::Graha(Graha::Mercury | Graha::Venus)) => return None,
                            (_, from) => from,
                        };
                        Some(stating(
                            format!(
                                "BJ_{}_IN_{}_NAVAMSA_ASPECTED_BY_{}",
                                planet.key(),
                                group.lord.key(),
                                from.key()
                            ),
                            "navamsa-drishti",
                            family.source(&group.verse),
                            vec![
                                Condition::InVarga {
                                    varga: Varga::D9,
                                    condition: Box::new(Condition::PlanetInSign {
                                        planet: SignRef::Of(BodyRef::Body(planet)),
                                        signs: group.signs.clone(),
                                    }),
                                },
                                aspected_by(from, planet),
                            ],
                            &reading.effect,
                        ))
                    })
                })
        })
    }

    /// Brihat Jataka ch. 20: each graha in each bhava from the ascendant, and
    /// the readings a verse gives the same place when a named sign rises.
    fn bj_in_bhava(&self) -> impl Iterator<Item = Rule> {
        let family = &self.bj_in_bhava.family;
        self.bj_in_bhava.grahas.iter().flat_map(move |graha| {
            let in_house = move |house: House| Condition::PlanetInHouse {
                planet: Subject::Ref(SignRef::Of(BodyRef::Body(graha.planet))),
                houses: vec![house],
            };
            let plain = graha
                .readings
                .iter()
                .enumerate()
                .filter_map(move |(at, reading)| {
                    let house = House::try_new(u8::try_from(at).ok()? + 1).ok()?;
                    Some(stating(
                        format!("BJ_{}_IN_BHAVA_{}", graha.planet.key(), house.get()),
                        "graha-in-bhava",
                        family.source(&reading.verse),
                        vec![in_house(house)],
                        &reading.effect,
                    ))
                });
            let varied = graha.variants.iter().flat_map(move |variant| {
                variant.lagnas.iter().map(move |lagna| {
                    stating(
                        format!(
                            "BJ_{}_IN_BHAVA_{}_{}_RISING",
                            graha.planet.key(),
                            variant.house.get(),
                            lagna.key()
                        ),
                        "graha-in-bhava",
                        family.source(&variant.verse),
                        vec![
                            in_house(variant.house),
                            Condition::PlanetInSign {
                                planet: SignRef::Of(BodyRef::Body(Body::Lagna)),
                                signs: vec![*lagna],
                            },
                        ],
                        &variant.effect,
                    )
                })
            });
            plain.chain(varied)
        })
    }
}

/// `from` aspects `target` by graha drishti in the rasi chart.
fn aspected_by(from: Body, target: Body) -> Condition {
    Condition::PlanetAspectsPlanet {
        from: BodySubject::Ref(BodyRef::Body(from)),
        target: SignRef::Of(BodyRef::Body(target)),
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "tests unwrap what they read and fail by panicking"
    )]

    use super::*;

    /// Each of Saravali's chapters reads one size and names every combination
    /// of the seven of that size once, each in the seven's own order, so that
    /// its key is the one the combination is known by.
    #[test]
    fn saravali_names_every_set_of_each_size_once() {
        let table: Table = serde_json::from_str(include_str!("../rules/classical-readings.json"))
            .expect("the dwigraha table reads");
        let mut sizes = Vec::new();
        for listed in &table.saravali_together.chapters {
            let size = listed
                .readings
                .first()
                .map_or(0, |reading| reading.planets.len());
            let mut named: Vec<Vec<Body>> = listed
                .readings
                .iter()
                .map(|reading| reading.planets.clone())
                .collect();
            named.sort();
            let mut every = combinations(&Body::SEVEN, size);
            every.sort();
            assert_eq!(named, every, "ch. {}", listed.chapter);
            sizes.push(size);
        }
        assert_eq!(sizes, [2, 3, 4, 5, 6]);
    }

    /// Ch. 51 v. 110: each of a sign's twelve dwadasamsas reads as the lagna
    /// in the sign it counts to, under the carrying's own note: the 2nd of
    /// Aries says what Taurus rising says.
    #[test]
    fn a_dwadasamsa_says_what_the_sign_it_counts_to_says_rising() {
        let table: Table = serde_json::from_str(include_str!("../rules/classical-readings.json"))
            .expect("the dwigraha table reads");
        let rules = table.rules();
        let carried = table
            .rising_part
            .carried
            .first()
            .expect("one carried division");
        let of = |key: &str| rules.iter().find(|rule| rule.key == key).expect(key);
        let parts = rules
            .iter()
            .filter(|rule| rule.key.starts_with("SARAVALI_DWADASAMSA_"))
            .count();
        assert_eq!(parts, 144);
        let second = of("SARAVALI_DWADASAMSA_2_OF_ARIES");
        assert_eq!(second.source.note.as_deref(), Some(carried.note.as_str()));
        assert_eq!(second.effect(), of("SARAVALI_LAGNA_TAURUS").effect());
    }
}

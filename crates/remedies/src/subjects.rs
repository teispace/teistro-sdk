//! Whom a remedy is for (`03-design/remedies.md`, step 3, C351).
//!
//! BPHS 84.26 and *Yājñavalkya* I.307 say to worship the graha that is
//! *duḥstha* for the person at that time. Neither verse defines the
//! word, so the answer is the grahas each with the reasons that make it
//! a subject, and no ranking among them. The running antardaśā's printed
//! condition (chs. 37–45) is judged where its predicate can be: a
//! condition the verse leaves open is reported as `None`, never decided.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::house::House;

use crate::dasha::{Condition, DashaShanti, dasha_shanti};
use crate::functional::{Functional, Nature};

/// The nine grahas a remedy can be for, in catalogue order.
pub const NINE: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// The daśā running at the instant the remedy is asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Running {
    /// The mahādaśā lord.
    pub mahadasha: Graha,
    /// The antardaśā lord.
    pub antardasha: Graha,
}

/// The chart, as the remedies read it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemedySky {
    /// The lagna's sign.
    pub lagna: Rashi,
    /// The sign each of the nine stands in, in the order of [`NINE`].
    pub signs: [Rashi; 9],
    /// Whether each of the nine is combust, in the order of [`NINE`].
    pub combust: [bool; 9],
    /// Whether the Moon is waxing, from new to full.
    pub moon_waxing: bool,
    /// The daśā running at the instant, when one is asked about.
    pub running: Option<Running>,
}

impl RemedySky {
    /// The sign `graha` stands in.
    fn sign_of(&self, graha: Graha) -> Option<Rashi> {
        index(graha).and_then(|at| self.signs.get(at).copied())
    }

    /// The house `graha` stands in from the lagna.
    fn house_of(&self, graha: Graha) -> Option<u8> {
        self.sign_of(graha)
            .map(|sign| House::between(self.lagna, sign).get())
    }

    /// The lord of house `house` from the lagna.
    fn lord_of(&self, house: u8) -> Option<Graha> {
        House::try_from(house)
            .ok()
            .map(|house| house.sign_from(self.lagna).attributes().lord)
    }

    /// Whether `graha` stands in a sign with another of `others`.
    fn joined(&self, graha: Graha, others: impl Fn(Graha) -> bool) -> bool {
        let Some(sign) = self.sign_of(graha) else {
            return false;
        };
        NINE.into_iter()
            .filter(|&other| other != graha && others(other))
            .any(|other| self.sign_of(other) == Some(sign))
    }

    /// Whether `graha` is a natural malefic: the Sun, Mars, Saturn, the
    /// nodes, a waning Moon, or a Mercury joined with one of those.
    fn malefic(&self, graha: Graha) -> bool {
        let plain = |graha: Graha| match graha {
            Graha::Sun | Graha::Mars | Graha::Saturn | Graha::Rahu | Graha::Ketu => true,
            Graha::Moon => !self.moon_waxing,
            _ => false,
        };
        match graha {
            Graha::Mercury => self.joined(Graha::Mercury, plain),
            _ => plain(graha),
        }
    }

    /// Whether `graha` stands in its sign of debilitation.
    fn debilitated(&self, graha: Graha) -> bool {
        graha
            .attributes()
            .debilitation
            .is_some_and(|fall| self.sign_of(graha) == Some(fall.sign))
    }
}

/// Where `graha` sits in [`NINE`].
fn index(graha: Graha) -> Option<usize> {
    NINE.iter().position(|&one| one == graha)
}

/// Why a graha is a subject of a remedy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Reason {
    /// It is the running mahādaśā's lord.
    Mahadasha,
    /// It is the running antardaśā's lord.
    Antardasha,
    /// It stands in its sign of debilitation.
    Debilitated,
    /// It is combust.
    Combust,
    /// It stands in the 6th, 8th or 12th from the lagna.
    Dusthana,
    /// Its lordships make it a malefic for the lagna (step 1).
    FunctionalMalefic,
    /// It owns the 2nd or the 7th (*Laghu Parashari* vv. 23–25).
    Maraka,
    /// It owns the badhaka house.
    Badhakesha,
}

/// A graha with every reason that makes it a subject.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    /// The graha.
    pub graha: Graha,
    /// Why, in the order of [`Reason`].
    pub reasons: Vec<Reason>,
}

/// The running antardaśā's printed śānti, with whether each of its
/// conditions holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AntardashaShanti {
    /// What the chapter prints.
    pub shanti: DashaShanti,
    /// Whether each condition holds, in the printed order: `None` where
    /// the verse leaves the predicate open (C348, C351).
    pub holds: Vec<Option<bool>>,
}

/// Whom a remedy is for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Subjects {
    /// Each graha with at least one reason, in catalogue order, with no
    /// ranking among them.
    pub subjects: Vec<Subject>,
    /// The running antardaśā's śānti, when a daśā is given.
    pub antardasha: Option<AntardashaShanti>,
}

/// Whether `condition`, printed for the antardaśā of `running`, holds in
/// `sky`, or `None` when the verse leaves its predicate open.
///
/// A node owns no sign here, so a lordship condition on a node is open
/// rather than false (C351).
#[must_use]
pub fn holds(condition: Condition, sky: &RemedySky, running: Running) -> Option<bool> {
    let ad = running.antardasha;
    let node = matches!(ad, Graha::Rahu | Graha::Ketu);
    let lord_of = |houses: &[u8]| -> Option<bool> {
        if node {
            return None;
        }
        Some(houses.iter().any(|&house| sky.lord_of(house) == Some(ad)))
    };
    let placed_in = |houses: &[u8]| sky.house_of(ad).map(|house| houses.contains(&house));
    let from_dasha_lord = || {
        let (md, at) = (sky.sign_of(running.mahadasha)?, sky.sign_of(ad)?);
        Some(House::between(md, at).get())
    };
    let with_malefic = || sky.joined(ad, |other| sky.malefic(other));
    match condition {
        Condition::LordOfSecondOrSeventh => lord_of(&[2, 7]),
        Condition::LordOfSeventh => lord_of(&[7]),
        Condition::InSecondOrSeventh => placed_in(&[2, 7]),
        Condition::InSecondSeventhOrEighth => placed_in(&[2, 7, 8]),
        Condition::SixthOrEighthFromDashaLordOrWeak => {
            let far = from_dasha_lord()?;
            Some([6, 8].contains(&far) || sky.debilitated(ad) || with_malefic())
        }
        Condition::DusthanaFromDashaLordWithMalefic => {
            let far = from_dasha_lord()?;
            Some([6, 8, 12].contains(&far) && with_malefic())
        }
        Condition::InSecondOrSeventhWithItsLord => {
            let house = sky.house_of(ad)?;
            if ![2, 7].contains(&house) {
                return Some(false);
            }
            let lord = sky.lord_of(house)?;
            Some(sky.sign_of(lord) == sky.sign_of(ad))
        }
        Condition::WithSecondOrSeventhLords => {
            let lords = [sky.lord_of(2), sky.lord_of(7)];
            Some(sky.joined(ad, |other| lords.contains(&Some(other))))
        }
        Condition::LordOfEighthOrSeventhOrInSecond => {
            let in_second = placed_in(&[2])?;
            match lord_of(&[8, 7]) {
                Some(lord) => Some(lord || in_second),
                None if in_second => Some(true),
                None => None,
            }
        }
        // Open in the verse: what "with the 8th or 12th" joins, the mixed
        // grammar of 38.12 and 40.7, a relation left unnamed, joined or
        // aspected by what, and a 2nd-and-6th lord the Moon, owning one
        // sign, can never be (C349).
        Condition::WithEighthOrTwelfth
        | Condition::EighthLordInEighth
        | Condition::RelatedToSecondOrSeventhLord
        | Condition::EighthLordWithSecondOrSeventhLord
        | Condition::LordOfSecondAndSixth
        | Condition::SecondOrSeventhLordInSeventh
        | Condition::JoinedOrAspected => None,
    }
}

/// Whom a remedy is for in `sky`, by the lordships `functional` read for
/// its lagna.
#[must_use]
pub fn subjects(sky: &RemedySky, functional: &Functional) -> Subjects {
    let subjects = NINE
        .into_iter()
        .filter_map(|graha| {
            let row = functional.row(graha);
            let running = sky.running;
            let reasons: Vec<Reason> = [
                (
                    Reason::Mahadasha,
                    running.is_some_and(|run| run.mahadasha == graha),
                ),
                (
                    Reason::Antardasha,
                    running.is_some_and(|run| run.antardasha == graha),
                ),
                (Reason::Debilitated, sky.debilitated(graha)),
                (
                    Reason::Combust,
                    index(graha)
                        .and_then(|at| sky.combust.get(at))
                        .copied()
                        .unwrap_or(false),
                ),
                (
                    Reason::Dusthana,
                    sky.house_of(graha)
                        .is_some_and(|house| [6, 8, 12].contains(&house)),
                ),
                (
                    Reason::FunctionalMalefic,
                    row.is_some_and(|row| row.nature == Nature::Malefic),
                ),
                (Reason::Maraka, functional.marakas.contains(&graha)),
                (Reason::Badhakesha, functional.badhaka.lord == graha),
            ]
            .into_iter()
            .filter_map(|(reason, held)| held.then_some(reason))
            .collect();
            (!reasons.is_empty()).then_some(Subject { graha, reasons })
        })
        .collect();
    let antardasha = sky.running.and_then(|running| {
        dasha_shanti(running.mahadasha, running.antardasha).map(|shanti| AntardashaShanti {
            shanti: *shanti,
            holds: shanti
                .conditions
                .iter()
                .map(|&condition| holds(condition, sky, running))
                .collect(),
        })
    });
    Subjects {
        subjects,
        antardasha,
    }
}

//! Functional nature: what a lagna's lordships make of each graha
//! (`03-design/remedies.md`, C329 to C333).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Modality, Rashi};
use teistro_core::house::House;

/// The seven grahas that own signs, in catalogue order. The nodes own
/// none, so no lordship clause can be about them (LP v. 13 reads them
/// through their bhava and their associate instead).
const LORDS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// The natural benefics LP v. 7 means, Jupiter, Venus, Mercury and the
/// Moon, whose kendra-lordship blemish vv. 10 and 11 rank.
const NATURAL_BENEFICS: [Graha; 4] = [Graha::Jupiter, Graha::Venus, Graha::Mercury, Graha::Moon];

/// How a graha's summary nature is derived from its lordships.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Scheme {
    /// *Laghu Parashari* vv. 6 to 20 and BPHS ch. 13 vv. 1 to 13: the
    /// lords of 3, 6 and 11 are evil, the 8th lord is not good unless it
    /// is the lagna lord or a luminary, a kendra lord loses its natural
    /// nature, the 2nd and 12th act by association, and a trikona lord is
    /// good whatever else it owns (C329, C330).
    #[default]
    LaghuParashari,
    /// The baseline engine's: the lords of 1, 5 and 9 good, the lords of
    /// 6, 8 and 12 bad unless they also own a trikona, and a yogakaraka
    /// any other kendra lord that owns a trikona. No text read gives the
    /// 12th lord as evil or exempts no one from the 8th (C331).
    Baseline,
}

/// The readings functional nature is derived under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct FunctionalRules {
    /// Which precedence derives the summary nature.
    pub scheme: Scheme,
}

impl FunctionalRules {
    /// Every reading the baseline engine takes.
    #[must_use]
    pub const fn baseline() -> FunctionalRules {
        FunctionalRules {
            scheme: Scheme::Baseline,
        }
    }
}

/// A graha's functional nature for a lagna, summarised.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Nature {
    /// One graha owning a kendra and a trikona (LP v. 20).
    Yogakaraka,
    /// Good for the lagna: a trikona lord.
    Benefic,
    /// Neither: a kendra lord that lost its natural nature, a lord of the
    /// 2nd or 12th acting by association, or an 8th lord the text exempts.
    Neutral,
    /// Evil for the lagna: a lord of 3, 6 or 11, or of the 8th.
    Malefic,
}

/// What a clause says of a graha's lordship. Each names the verse it
/// reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClauseKind {
    /// It owns the 1st (BPHS ch. 13 v. 12; LP v. 9).
    Lagnesha,
    /// It owns the 5th or the 9th, and gives good (LP v. 6).
    TrikonaLord,
    /// It owns the 3rd, 6th or 11th, and gives evil (LP v. 6).
    TrishadayaLord,
    /// It owns the 8th, the 12th from the 9th, and is not good (LP v. 9).
    Randhresha,
    /// It owns the 8th and the 1st, so the 8th's blemish is void (LP v. 9;
    /// BPHS ch. 13 v. 12).
    RandhreshaVoidLagnesha,
    /// It owns the 8th and is the Sun or the Moon, which carry no 8th-lord
    /// blemish (LP v. 11).
    RandhreshaVoidLuminary,
    /// A natural benefic owning a kendra, which then does not give good
    /// (LP v. 7; the blemish ranked in vv. 10 and 11).
    KendraBeneficLoses,
    /// A natural malefic owning a kendra, which then does not give evil
    /// (LP v. 7).
    KendraMaleficLoses,
    /// It owns the 2nd or the 12th, and gives the results of what it joins
    /// (LP v. 8).
    ByAssociation,
    /// It owns a kendra and a trikona (LP v. 20).
    Yogakaraka,
    /// It owns the 2nd or the 7th, the maraka houses (LP vv. 23 to 25).
    Maraka,
    /// It owns the lagna's badhaka sthana: the 11th from a movable lagna,
    /// the 9th from a fixed one, the 7th from a dual one.
    Badhakesha,
}

impl ClauseKind {
    /// The verse the clause reads, as the design page cites it.
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            ClauseKind::Lagnesha | ClauseKind::RandhreshaVoidLagnesha => {
                "Laghu Parashari v. 9; BPHS ch. 13 v. 12"
            }
            ClauseKind::TrikonaLord | ClauseKind::TrishadayaLord => "Laghu Parashari v. 6",
            ClauseKind::Randhresha => "Laghu Parashari v. 9",
            ClauseKind::RandhreshaVoidLuminary => "Laghu Parashari v. 11",
            ClauseKind::KendraBeneficLoses | ClauseKind::KendraMaleficLoses => {
                "Laghu Parashari v. 7"
            }
            ClauseKind::ByAssociation => "Laghu Parashari v. 8",
            ClauseKind::Yogakaraka => "Laghu Parashari v. 20",
            ClauseKind::Maraka => "Laghu Parashari vv. 23-25",
            ClauseKind::Badhakesha => "BPHS (badhaka sthana)",
        }
    }
}

/// One clause that holds for a graha, and the house that makes it hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Clause {
    /// What it says.
    pub kind: ClauseKind,
    /// The house, counted from the lagna as 1, whose lordship makes it
    /// hold.
    pub house: u8,
}

/// One graha's lordships for a lagna and what they make of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FunctionalRow {
    /// The graha.
    pub graha: Graha,
    /// The houses it owns, counted from the lagna as 1, ascending.
    pub houses: Vec<u8>,
    /// Every clause that holds for it, in house order and then in the
    /// order [`ClauseKind`] lists them.
    pub clauses: Vec<Clause>,
    /// The summary, by the rules' scheme.
    pub nature: Nature,
}

impl FunctionalRow {
    /// Whether a clause of this kind holds for the graha.
    #[must_use]
    pub fn holds(&self, kind: ClauseKind) -> bool {
        self.clauses.iter().any(|clause| clause.kind == kind)
    }
}

/// A lagna's badhaka sthana and its lord.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Badhaka {
    /// The house: 11 from a movable lagna, 9 from a fixed, 7 from a dual.
    pub house: u8,
    /// Its lord.
    pub lord: Graha,
}

/// What a lagna's lordships make of the seven grahas.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Functional {
    /// The lagna.
    pub lagna: Rashi,
    /// The scheme the natures were derived under.
    pub scheme: Scheme,
    /// The seven grahas that own signs, Sun to Saturn.
    pub rows: Vec<FunctionalRow>,
    /// The grahas whose nature is [`Nature::Yogakaraka`], Sun to Saturn.
    pub yogakarakas: Vec<Graha>,
    /// The lords of the 2nd and the 7th, Sun to Saturn (LP v. 23).
    pub marakas: Vec<Graha>,
    /// The badhaka sthana and its lord.
    pub badhaka: Badhaka,
}

impl Functional {
    /// The row of a graha, or `None` for Rahu, Ketu or an outer planet,
    /// which own no sign.
    #[must_use]
    pub fn row(&self, graha: Graha) -> Option<&FunctionalRow> {
        self.rows.iter().find(|row| row.graha == graha)
    }
}

/// The badhaka sthana of a lagna, by its modality.
fn badhaka_house(lagna: Rashi) -> u8 {
    match lagna.attributes().modality {
        Modality::Chara => 11,
        Modality::Sthira => 9,
        // Dual, and any modality a later catalogue adds: the dual reading.
        _ => 7,
    }
}

/// Every clause that holds for a graha owning `houses`.
fn clauses_of(graha: Graha, houses: &[u8], badhaka: u8) -> Vec<Clause> {
    let owns = |house: u8| houses.contains(&house);
    let natural_benefic = NATURAL_BENEFICS.contains(&graha);
    let luminary = matches!(graha, Graha::Sun | Graha::Moon);
    let yogakaraka = [4, 7, 10].into_iter().any(owns) && [5, 9].into_iter().any(owns);
    let mut clauses = Vec::new();
    for &house in houses {
        let mut add = |kind: ClauseKind| clauses.push(Clause { kind, house });
        match house {
            1 => add(ClauseKind::Lagnesha),
            5 | 9 => add(ClauseKind::TrikonaLord),
            3 | 6 | 11 => add(ClauseKind::TrishadayaLord),
            8 if owns(1) => add(ClauseKind::RandhreshaVoidLagnesha),
            8 if luminary => add(ClauseKind::RandhreshaVoidLuminary),
            8 => add(ClauseKind::Randhresha),
            2 | 12 => add(ClauseKind::ByAssociation),
            _ => {}
        }
        if matches!(house, 4 | 7 | 10) {
            add(if natural_benefic {
                ClauseKind::KendraBeneficLoses
            } else {
                ClauseKind::KendraMaleficLoses
            });
            if yogakaraka {
                add(ClauseKind::Yogakaraka);
            }
        }
        if matches!(house, 2 | 7) {
            add(ClauseKind::Maraka);
        }
        if house == badhaka {
            add(ClauseKind::Badhakesha);
        }
    }
    clauses
}

/// The summary nature under *Laghu Parashari*'s precedence: a yogakaraka
/// first (v. 20), then a trikona lord, whatever else it owns (v. 6; BPHS
/// ch. 13 shows Jupiter good for Cancer though it owns the 6th), then a
/// lord of 3, 6 or 11, then the 8th lord where nothing voids it; anything
/// else is neutral (C330).
fn parashari(row: &[Clause]) -> Nature {
    let holds = |kind: ClauseKind| row.iter().any(|clause| clause.kind == kind);
    if holds(ClauseKind::Yogakaraka) {
        Nature::Yogakaraka
    } else if holds(ClauseKind::Lagnesha) || holds(ClauseKind::TrikonaLord) {
        Nature::Benefic
    } else if holds(ClauseKind::TrishadayaLord) || holds(ClauseKind::Randhresha) {
        Nature::Malefic
    } else {
        Nature::Neutral
    }
}

/// The baseline engine's summary: a lord of 1, 5 or 9 good, and a
/// yogakaraka if it also owns a kendra and is not the lagna lord; else a
/// lord of 6, 8 or 12 bad; else neutral (C331).
fn baseline(houses: &[u8]) -> Nature {
    let owns = |set: &[u8]| houses.iter().any(|house| set.contains(house));
    if owns(&[1, 5, 9]) {
        if !owns(&[1]) && owns(&[4, 7, 10]) {
            Nature::Yogakaraka
        } else {
            Nature::Benefic
        }
    } else if owns(&[6, 8, 12]) {
        Nature::Malefic
    } else {
        Nature::Neutral
    }
}

/// What a lagna's lordships make of the seven grahas that own signs: each
/// one's houses, every clause that holds for it with its verse, and its
/// summary nature under the rules' scheme; the yogakarakas, the marakas
/// and the badhaka sthana with its lord.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_remedies::{ClauseKind, FunctionalRules, Nature, functional};
///
/// // Capricorn: the Sun owns only the 8th, and a luminary carries no
/// // 8th-lord blemish (LP v. 11), so it is neutral; the baseline engine
/// // makes it malefic.
/// let capricorn = functional(Rashi::Capricorn, FunctionalRules::default());
/// let sun = capricorn.row(Graha::Sun).expect("the Sun owns a sign");
/// assert!(sun.holds(ClauseKind::RandhreshaVoidLuminary));
/// assert_eq!(sun.nature, Nature::Neutral);
/// let baseline = functional(Rashi::Capricorn, FunctionalRules::baseline());
/// assert_eq!(baseline.row(Graha::Sun).map(|row| row.nature), Some(Nature::Malefic));
/// ```
#[must_use]
pub fn functional(lagna: Rashi, rules: FunctionalRules) -> Functional {
    let badhaka = badhaka_house(lagna);
    let rows: Vec<FunctionalRow> = LORDS
        .iter()
        .map(|&graha| {
            let houses: Vec<u8> = House::ALL
                .into_iter()
                .filter(|house| house.sign_from(lagna).attributes().lord == graha)
                .map(House::get)
                .collect();
            let clauses = clauses_of(graha, &houses, badhaka);
            let nature = match rules.scheme {
                Scheme::LaghuParashari => parashari(&clauses),
                Scheme::Baseline => baseline(&houses),
            };
            FunctionalRow {
                graha,
                houses,
                clauses,
                nature,
            }
        })
        .collect();
    let with = |pick: &dyn Fn(&FunctionalRow) -> bool| -> Vec<Graha> {
        rows.iter()
            .filter(|row| pick(row))
            .map(|row| row.graha)
            .collect()
    };
    let yogakarakas = with(&|row| row.nature == Nature::Yogakaraka);
    let marakas = with(&|row| row.holds(ClauseKind::Maraka));
    Functional {
        lagna,
        scheme: rules.scheme,
        yogakarakas,
        marakas,
        badhaka: Badhaka {
            house: badhaka,
            lord: House::try_from(badhaka)
                .map_or(lagna, |house| house.sign_from(lagna))
                .attributes()
                .lord,
        },
        rows,
    }
}

//! The Bhuvanapradīpa's graded yogas, as the Prashna Tantra's general
//! section quotes them (Tajika Nilakanthi, 1893, leaf n178; repeated at
//! n200–n201 vv. 1–5): how fully the lords and the benefics look at the
//! lagna, named as the text names them, never weighed (crux C373).

use serde::{Deserialize, Serialize};
use teistro_aspect::drishti;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_tajika::Drishti;

use crate::{Disposition, GRAHAS, Placed, PrashnaSky, disposition};

/// Whether the Bhuvanapradīpa's graded yogas are read, and by which
/// aspect (crux C373, open: off by default).
///
/// The passage never uses a Tajika term, yet the Prashna Tantra quotes it
/// among Tajika rules, so either aspect may be the one that "sees".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GradedRule {
    /// Not read.
    #[default]
    Off,
    /// A graha sees a sign it aspects fully, by Parashara's aspects: the
    /// seventh, and Mars's, Jupiter's and Saturn's special ones; a graha
    /// does not see the sign it stands in.
    Parashari,
    /// A graha sees a sign by Tajika's sign aspects, the conjunction
    /// included.
    Tajika,
}

impl GradedRule {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `PARASHARI`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            GradedRule::Off => "OFF",
            GradedRule::Parashari => "PARASHARI",
            GradedRule::Tajika => "TAJIKA",
        }
    }
}

/// How the lagnesha and the karyesha look at the two houses, one of the
/// three configurations of the full yoga (*paripūrṇa*).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LordsSee {
    /// The lagnesha looks at the lagna and the karyesha at the house
    /// asked about (*lagnapatir yadi lagnaṁ kāryādhipatiś ca vīkṣate
    /// kāryam*).
    OwnHouses,
    /// The lagnesha looks at the house asked about and the karyesha at
    /// the lagna (*lagnādhīśaḥ kāryaṁ kāryeśaḥ paśyati vilagnam*).
    EachOthersHouses,
    /// The lagnesha looks at the karyesha and the karyesha at the
    /// lagnesha (*lagneśaḥ kāryeśaṁ vilokayel lagnapaṁ tu kāryeśaḥ*).
    EachOther,
}

impl LordsSee {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `OWN_HOUSES`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            LordsSee::OwnHouses => "OWN_HOUSES",
            LordsSee::EachOthersHouses => "EACH_OTHERS_HOUSES",
            LordsSee::EachOther => "EACH_OTHER",
        }
    }

    /// Every configuration, in the passage's order.
    pub const ALL: [LordsSee; 3] = [
        LordsSee::OwnHouses,
        LordsSee::EachOthersHouses,
        LordsSee::EachOther,
    ];
}

/// A graded yoga, by the name the passage gives it. The names are
/// fractions, but the passage's order does not rank them as numbers
/// would ("less a third" for two or three benefics, "less a quarter" for
/// one), so they are reported as names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PrashnaGrade"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Grade {
    /// *Paripūrṇa*: one of the three [`LordsSee`] configurations, with
    /// the Moon looking at the lagna (*śītagu-dṛṣṭau satyām*).
    Paripurna,
    /// *Pāda*, a quarter: the lagnesha looks at the lagna and no benefic
    /// does (*paśyati saumyo na lagnapo lagnam*, n178; n200 reads *yal*
    /// for *lagnapo*).
    Pada,
    /// *Ardha*, a half: the quarter's conditions, and a benefic looks at
    /// the lagnesha.
    Ardha,
    /// *Pādona*, less a quarter: one benefic looks at both the lagna and
    /// the lagnesha.
    Padona,
    /// *Tribhāgona*, less a third: the lagnesha looking at the lagna, two
    /// or three benefics look at it too.
    Tribhagona,
    /// *Pūrṇa*, full: the Moon free of a malefic's aspect, the benefics
    /// (two or more) and the lagnesha look at the lagna.
    Purna,
}

impl Grade {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `PARIPURNA`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Grade::Paripurna => "PARIPURNA",
            Grade::Pada => "PADA",
            Grade::Ardha => "ARDHA",
            Grade::Padona => "PADONA",
            Grade::Tribhagona => "TRIBHAGONA",
            Grade::Purna => "PURNA",
        }
    }

    /// Every grade, in the passage's order.
    pub const ALL: [Grade; 6] = [
        Grade::Paripurna,
        Grade::Pada,
        Grade::Ardha,
        Grade::Padona,
        Grade::Tribhagona,
        Grade::Purna,
    ];
}

/// The Bhuvanapradīpa's graded yogas, read: every clause the grades are
/// built from, and the grades whose clauses hold.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PrashnaGraded"))]
#[serde(rename_all = "camelCase")]
pub struct Graded {
    /// The aspect the clauses were read by.
    pub rule: GradedRule,
    /// The configurations of the lords that hold, in the passage's
    /// order; empty when no house is asked.
    pub lords: Vec<LordsSee>,
    /// Whether the Moon looks at the lagna.
    pub moon_sees_lagna: bool,
    /// Whether the lagnesha looks at the lagna.
    pub lagnesha_sees_lagna: bool,
    /// The benefics, the lagnesha left out, looking at the lagna, in the
    /// order of [`GRAHAS`].
    pub on_lagna: Vec<Graha>,
    /// The benefics looking at the lagnesha, in the same order.
    pub on_lagnesha: Vec<Graha>,
    /// Whether no malefic looks at the Moon (*krūrāvekṣaṇa-varjyaḥ*).
    pub moon_free: bool,
    /// The grades whose clauses hold, in the passage's order.
    pub grades: Vec<Grade>,
    /// The grahas that "give ill" (*aniṣṭa-phala-dāyakaḥ*): sharing a
    /// sign with a malefic, looked at by one, or burnt by the Sun.
    pub ill: Vec<Graha>,
}

/// Reads the graded yogas, or `None` under [`GradedRule::Off`].
///
/// `karya` is the karyesha and the sign of the house asked about, when a
/// house is.
pub(crate) fn graded(
    sky: &PrashnaSky,
    karya: Option<(Graha, Rashi)>,
    rule: GradedRule,
) -> Option<Graded> {
    if rule == GradedRule::Off {
        return None;
    }
    let sees = |graha: Graha, to: Rashi| {
        sky.of(graha).is_some_and(|placed| match rule {
            GradedRule::Off => false,
            GradedRule::Parashari => drishti::between(graha, placed.sign(), to).is_full(),
            GradedRule::Tajika => Drishti::between_signs(placed.sign(), to).is_aspect(),
        })
    };
    let sign_of = |graha: Graha| sky.of(graha).map(Placed::sign);
    let sees_graha =
        |graha: Graha, other: Graha| sign_of(other).is_some_and(|sign| sees(graha, sign));
    let lagna = sky.lagna();
    let lagnesha = lagna.attributes().lord;
    let of = |kind: Disposition| {
        GRAHAS
            .into_iter()
            .filter(move |&graha| disposition(sky, graha) == Some(kind))
    };
    let benefics = || of(Disposition::Benefic).filter(|&graha| graha != lagnesha);
    let on_lagna: Vec<Graha> = benefics().filter(|&graha| sees(graha, lagna)).collect();
    let on_lagnesha: Vec<Graha> = benefics()
        .filter(|&graha| sees_graha(graha, lagnesha))
        .collect();
    let lords = karya.map_or_else(Vec::new, |(karyesha, sign)| {
        LordsSee::ALL
            .into_iter()
            .filter(|lords| match lords {
                LordsSee::OwnHouses => sees(lagnesha, lagna) && sees(karyesha, sign),
                LordsSee::EachOthersHouses => sees(lagnesha, sign) && sees(karyesha, lagna),
                LordsSee::EachOther => {
                    sees_graha(lagnesha, karyesha) && sees_graha(karyesha, lagnesha)
                }
            })
            .collect()
    });
    let moon_sees_lagna = sees(Graha::Moon, lagna);
    let lagnesha_sees_lagna = sees(lagnesha, lagna);
    let moon_free = !of(Disposition::Malefic)
        .any(|malefic| malefic != Graha::Moon && sees_graha(malefic, Graha::Moon));
    let grades = Grade::ALL
        .into_iter()
        .filter(|grade| match grade {
            Grade::Paripurna => !lords.is_empty() && moon_sees_lagna,
            Grade::Pada => lagnesha_sees_lagna && on_lagna.is_empty(),
            Grade::Ardha => lagnesha_sees_lagna && on_lagna.is_empty() && !on_lagnesha.is_empty(),
            Grade::Padona => on_lagna.iter().any(|graha| on_lagnesha.contains(graha)),
            Grade::Tribhagona => lagnesha_sees_lagna && (2..=3).contains(&on_lagna.len()),
            Grade::Purna => lagnesha_sees_lagna && on_lagna.len() >= 2 && moon_free,
        })
        .collect();
    let ill = GRAHAS
        .into_iter()
        .filter(|&graha| disposition(sky, graha).is_some())
        .filter(|&graha| {
            sky.of(graha).is_some_and(|placed| placed.combust)
                || of(Disposition::Malefic).any(|malefic| {
                    malefic != graha
                        && (sign_of(malefic) == sign_of(graha) || sees_graha(malefic, graha))
                })
        })
        .collect();
    Some(Graded {
        rule,
        lords,
        moon_sees_lagna,
        lagnesha_sees_lagna,
        on_lagna,
        on_lagnesha,
        moon_free,
        grades,
        ill,
    })
}

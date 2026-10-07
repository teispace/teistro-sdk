//! The baseline engine's prashna readings, transcribed so a consumer
//! migrating from it gets its answers. None has a rank-1 source (C337,
//! C339, C340), and each is reached only when the caller asks for it.
//!
//! What is transcribed is what the engine **runs**. Its score names
//! three dignity factors (the lagna lord exalted, in its own sign, or
//! debilitated) and its topic two (exalted, own sign), but the planets it
//! scores never carry those flags, so none of the five can fire and none
//! is reproduced here.

use serde::{Deserialize, Serialize};
use teistro_core::Error;
use teistro_core::catalogue::{Graha, Rashi};

use crate::{PrashnaSky, unplaced};

/// The order the baseline lists the grahas in, which decides its ties.
const ORDER: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mercury,
    Graha::Venus,
    Graha::Mars,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// The baseline's natural benefics: the Moon whatever her phase.
const BENEFICS: [Graha; 4] = [Graha::Moon, Graha::Mercury, Graha::Jupiter, Graha::Venus];

/// The baseline's natural malefics.
const MALEFICS: [Graha; 5] = [
    Graha::Sun,
    Graha::Mars,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// The kendras, by whole signs from the lagna.
const KENDRAS: [u8; 4] = [1, 4, 7, 10];

/// The trikonas, by whole signs from the lagna.
const TRIKONAS: [u8; 3] = [1, 5, 9];

/// The Ptolemaic aspects the baseline's void Moon looks for, degrees.
const ASPECTS_DEG: [f64; 5] = [0.0, 60.0, 90.0, 120.0, 180.0];

/// Whether a prashna carries the baseline's points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScoreRule {
    /// No points: the verdict's clauses are the answer (C337).
    #[default]
    Off,
    /// `BASELINE`: the baseline engine's points beside the clauses, never
    /// replacing them. Unsourced.
    Baseline,
}

/// A factor the baseline scores, in the order it scores them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactorKind {
    /// The lagna lord in a kendra or a trikona: +2.
    LagnaLordKendraOrTrikona,
    /// The lagna lord in the 6th, 8th or 12th: −2.
    LagnaLordDusthana,
    /// The Moon less than 180° past the Sun: +1.
    MoonWaxing,
    /// The Moon void of course, as [`Score::void`] reads it: −3.
    MoonVoid,
    /// +1 for each benefic in a kendra.
    BeneficsInKendras,
    /// −1 for each malefic in a kendra.
    MaleficsInKendras,
    /// The Moon applying to a benefic: +1.
    MoonApplyingToBenefic,
    /// The Moon applying to a malefic: −1.
    MoonApplyingToMalefic,
}

/// One factor and the points it gave.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Factor {
    /// What was scored.
    pub kind: FactorKind,
    /// The points, signed.
    pub points: i8,
}

/// The baseline's answer over its points.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Answer {
    /// More than 1 point.
    Yes,
    /// Fewer than −1.
    No,
    /// −1 to 1.
    Uncertain,
}

/// The baseline engine's points for a query chart. Unsourced (C337).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Score {
    /// The sum of the factors.
    pub points: i8,
    /// The answer the baseline draws from them.
    pub answer: Answer,
    /// Each factor that gave points, in the baseline's order.
    pub factors: Vec<Factor>,
    /// Whether the Moon perfects none of the Ptolemaic aspects to a graha
    /// held where it stands, the nodes included, before she leaves her
    /// sign. The other grahas do not move, which neither Lilly's void
    /// (`crates/hellenistic`) nor Tajika's empty road is.
    pub void: bool,
    /// The graha nearest ahead of the Moon in longitude, within 180°, by
    /// the conjunction alone.
    pub applying_to: Option<Graha>,
}

/// A graha's longitude in the sky, folded into a circle.
fn at(sky: &PrashnaSky, graha: Graha) -> Result<f64, Error> {
    sky.of(graha)
        .map(|placed| placed.longitude_deg.rem_euclid(360.0))
        .ok_or_else(unplaced)
}

/// The house `graha` stands in.
fn house(sky: &PrashnaSky, graha: Graha) -> Result<u8, Error> {
    sky.house_of(graha).ok_or_else(unplaced)
}

/// The baseline's points.
pub(crate) fn score(sky: &PrashnaSky) -> Result<Score, Error> {
    let moon = at(sky, Graha::Moon)?;
    let others: Vec<(Graha, f64)> = ORDER
        .into_iter()
        .filter(|&graha| graha != Graha::Moon)
        .map(|graha| at(sky, graha).map(|longitude| (graha, longitude)))
        .collect::<Result<_, _>>()?;

    let remaining = 30.0 - moon % 30.0;
    let void = !others.iter().any(|&(_, longitude)| {
        ASPECTS_DEG.iter().any(|aspect| {
            [longitude - aspect, longitude + aspect]
                .iter()
                .any(|target| {
                    let ahead = (target - moon).rem_euclid(360.0);
                    ahead > 0.0 && ahead < remaining
                })
        })
    });
    let mut applying_to: Option<(Graha, f64)> = None;
    for &(graha, longitude) in &others {
        let ahead = 360.0 - (moon - longitude).rem_euclid(360.0);
        if ahead > 0.0 && ahead < 180.0 && applying_to.is_none_or(|(_, best)| ahead < best) {
            applying_to = Some((graha, ahead));
        }
    }
    let applying_to = applying_to.map(|(graha, _)| graha);

    let lord = house(sky, sky.lagna().attributes().lord)?;
    let in_kendras = |set: &[Graha]| -> Result<i8, Error> {
        let mut count = 0;
        for &graha in set {
            if KENDRAS.contains(&house(sky, graha)?) {
                count += 1;
            }
        }
        Ok(count)
    };
    let (benefics, malefics) = (in_kendras(&BENEFICS)?, in_kendras(&MALEFICS)?);
    let waxing = (moon - at(sky, Graha::Sun)?).rem_euclid(360.0) < 180.0;
    let factors: Vec<Factor> = [
        (
            FactorKind::LagnaLordKendraOrTrikona,
            KENDRAS.contains(&lord) || TRIKONAS.contains(&lord),
            2,
        ),
        (
            FactorKind::LagnaLordDusthana,
            [6, 8, 12].contains(&lord),
            -2,
        ),
        (FactorKind::MoonWaxing, waxing, 1),
        (FactorKind::MoonVoid, void, -3),
        (FactorKind::BeneficsInKendras, benefics > 0, benefics),
        (FactorKind::MaleficsInKendras, malefics > 0, -malefics),
        (
            FactorKind::MoonApplyingToBenefic,
            applying_to.is_some_and(|graha| BENEFICS.contains(&graha)),
            1,
        ),
        (
            FactorKind::MoonApplyingToMalefic,
            applying_to.is_some_and(|graha| MALEFICS.contains(&graha)),
            -1,
        ),
    ]
    .into_iter()
    .filter(|&(_, held, _)| held)
    .map(|(kind, _, points)| Factor { kind, points })
    .collect();
    let points = factors.iter().map(|factor| factor.points).sum();
    Ok(Score {
        points,
        answer: match points {
            2.. => Answer::Yes,
            ..=-2 => Answer::No,
            _ => Answer::Uncertain,
        },
        factors,
        void,
        applying_to,
    })
}

/// The baseline's strongest graha for an unspoken question: 2 for a
/// kendra and 1 for a trikona, the first in its order taking a tie, and
/// whether another equals it. The nodes are left out.
pub(crate) fn topic(sky: &PrashnaSky) -> Result<(Graha, bool), Error> {
    let mut scored = Vec::with_capacity(7);
    for graha in ORDER {
        if matches!(graha, Graha::Rahu | Graha::Ketu) {
            continue;
        }
        let house = house(sky, graha)?;
        let points = 2 * u8::from(KENDRAS.contains(&house)) + u8::from(TRIKONAS.contains(&house));
        scored.push((graha, points));
    }
    let best = scored.iter().map(|&(_, points)| points).max().unwrap_or(0);
    let mut top = scored.iter().filter(|&&(_, points)| points == best);
    let graha = top.next().map_or(Graha::Sun, |&(graha, _)| graha);
    Ok((graha, top.next().is_some()))
}

/// The sign the baseline gives a querent's number, 1 to 108: (n − 1)
/// mod 12 from Aries. `BASELINE` and unsourced: no text read gives a
/// number method (C340).
///
/// # Errors
///
/// A number outside 1 to 108 (`number`).
pub fn number_sign(number: u8) -> Result<Rashi, Error> {
    if !(1..=108).contains(&number) {
        return Err(
            Error::invalid_arg(format!("number {number} is not 1 to 108")).with_field("number"),
        );
    }
    Rashi::from_id(u16::from((number - 1) % 12)).ok_or_else(unplaced)
}

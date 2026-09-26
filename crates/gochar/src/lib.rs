//! Gochar: each transiting graha read from the natal Moon's sign
//! (`03-design/gochar.md`).
//!
//! Phaladeepika ch. 26, read in the Sanskrit on the printed page: of all
//! the lagnas, transits are counted from the Moon's (v. 1); each graha's
//! transit is good in certain houses from it (v. 2), and a good transit
//! holds only if no other graha then stands in its paired **vedha** house,
//! the verses naming who does not obstruct whom (vv. 3 to 8); and a transit
//! bears its fruit in one decanate of the sign (v. 25).
//!
//! ```
//! use teistro_core::catalogue::{Graha, Rashi};
//! use teistro_gochar::{GocharRules, Transit, Verdict, gochar};
//!
//! // The natal Moon in Aries; every graha transiting Taurus but the Sun in
//! // Gemini, the 3rd from the Moon, where v. 2 makes its transit good.
//! let mut transits = [Transit::new(Rashi::Taurus, 15.0); 9];
//! transits[Graha::Sun as usize] = Transit::new(Rashi::Gemini, 5.0);
//! let reading = gochar(Rashi::Aries, &transits, GocharRules::TEXT);
//! let sun = &reading.grahas[Graha::Sun as usize];
//! assert_eq!(sun.house, 3);
//! assert_eq!(sun.verdict, Verdict::Good);
//! // Its vedha house is the 9th, Sagittarius, which nobody transits.
//! assert_eq!(sun.vedha_house, Some(9));
//! assert!(sun.obstructed_by.is_empty());
//! // And v. 25: the Sun bears fruit in a sign's first decanate.
//! assert!(sun.fruitful_now);
//! ```

#![doc(html_no_source)]

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::settings::{NodeObstruction, NodeVedha, Settings};

/// The nine grahas, in the catalogue's order.
pub const GRAHAS: [Graha; 9] = [
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

/// A graha in transit: the sign it stands in and its degrees within it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Transit {
    /// The sign.
    pub sign: Rashi,
    /// Degrees within the sign, 0 to 30.
    pub degrees: f64,
}

impl Transit {
    /// A graha in `sign`, `degrees` into it.
    #[must_use]
    pub const fn new(sign: Rashi, degrees: f64) -> Transit {
        Transit { sign, degrees }
    }

    /// A graha at a sidereal longitude, in degrees.
    #[must_use]
    pub fn at_longitude(longitude_deg: f64) -> Transit {
        let at = longitude_deg.rem_euclid(360.0);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a longitude in [0, 360) over 30 is a sign index 0 to 11"
        )]
        let index = (at / 30.0) as usize;
        Transit {
            sign: Rashi::ALL
                .get(index.min(11))
                .copied()
                .unwrap_or(Rashi::Aries),
            degrees: at - 30.0 * f64::from(u8::try_from(index.min(11)).unwrap_or(0)),
        }
    }
}

/// The readings the text leaves open, from the settings' `gochar` group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GocharRules {
    /// The nodes' vedha (crux C136).
    pub node_vedha: NodeVedha,
    /// Whether the nodes obstruct another graha's transit (crux C137).
    pub node_obstruction: NodeObstruction,
}

impl GocharRules {
    /// The text read whole: the nodes "like the Sun", and every graha
    /// obstructing but the verses' exceptions.
    pub const TEXT: GocharRules = GocharRules {
        node_vedha: NodeVedha::LikeTheSun,
        node_obstruction: NodeObstruction::Obstruct,
    };

    /// The readings the settings' `gochar` group gives.
    #[must_use]
    pub const fn of(settings: &Settings) -> GocharRules {
        GocharRules {
            node_vedha: settings.gochar.node_vedha,
            node_obstruction: settings.gochar.node_obstruction,
        }
    }
}

/// What a transit comes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Verdict {
    /// In a good house, and nothing stands in its vedha house.
    Good,
    /// In a good house, and another graha stands in its vedha house.
    Obstructed,
    /// Not in a house v. 2 names good.
    NotGood,
}

/// Where in a sign a graha's transit bears fruit (v. 25).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Fruition {
    /// The first ten degrees: the Sun and Mars.
    First,
    /// The middle ten: Jupiter and Venus.
    Middle,
    /// The last ten: the Moon and Saturn.
    Last,
    /// The whole sign: Mercury and Rahu, and Ketu with him (crux C138).
    Throughout,
}

impl Fruition {
    /// A graha's decanate of fruition.
    #[must_use]
    pub const fn of(graha: Graha) -> Fruition {
        match graha {
            Graha::Sun | Graha::Mars => Fruition::First,
            Graha::Jupiter | Graha::Venus => Fruition::Middle,
            Graha::Moon | Graha::Saturn => Fruition::Last,
            _ => Fruition::Throughout,
        }
    }

    /// Whether `degrees` into a sign fall in this part of it.
    #[must_use]
    pub fn holds(self, degrees: f64) -> bool {
        match self {
            Fruition::First => degrees < 10.0,
            Fruition::Middle => (10.0..20.0).contains(&degrees),
            Fruition::Last => degrees >= 20.0,
            Fruition::Throughout => true,
        }
    }
}

/// One graha's transit read from the reference sign.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GrahaGochar {
    /// Which graha.
    pub graha: Graha,
    /// Where it stands.
    pub transit: Transit,
    /// Its house from the reference sign, 1 to 12.
    pub house: u8,
    /// Whether v. 2 makes a transit of this house good.
    pub good_house: bool,
    /// The house whose occupant obstructs it, when the house is good and a
    /// vedha applies (vv. 3 to 8).
    pub vedha_house: Option<u8>,
    /// The grahas standing in the vedha house that obstruct it, in the
    /// catalogue's order, the verses' exemptions left out.
    pub obstructed_by: Vec<Graha>,
    /// What the transit comes to.
    pub verdict: Verdict,
    /// The decanate in which its transit bears fruit (v. 25).
    pub fruition: Fruition,
    /// Whether it stands in that decanate now.
    pub fruitful_now: bool,
}

/// Every graha's transit read from one reference sign.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GocharReading {
    /// The sign the houses are counted from: the natal Moon's by v. 1.
    pub reference: Rashi,
    /// The readings under which the transits were judged.
    pub rules: GocharRules,
    /// Each graha's, the Sun to Ketu.
    pub grahas: [GrahaGochar; 9],
}

/// The Sun's good houses and their vedha houses (v. 3).
const SUN: &[(u8, u8)] = &[(11, 5), (3, 9), (10, 4), (6, 12)];

/// Each graha's good houses and their vedha houses, the Sun to Saturn
/// (vv. 3 to 8); the nodes take the Sun's under `LIKE_THE_SUN` (v. 2).
const VEDHA: [&[(u8, u8)]; 7] = [
    SUN,
    &[(7, 2), (1, 5), (6, 12), (11, 8), (10, 4), (3, 9)],
    &[(3, 12), (11, 5), (6, 9)],
    &[(2, 5), (4, 3), (6, 9), (8, 1), (10, 8), (11, 12)],
    &[(2, 12), (11, 8), (9, 10), (5, 4), (7, 3)],
    &[
        (1, 8),
        (2, 7),
        (3, 1),
        (4, 10),
        (5, 9),
        (8, 5),
        (9, 11),
        (12, 6),
        (11, 3),
    ],
    &[(3, 12), (11, 5), (6, 9)],
];

/// The graha the verses exempt from obstructing another: the Sun and
/// Saturn each other (vv. 3, 5), the Moon and Mercury each other (vv. 4,
/// 6). The Sun's exemption is father and son and is not the nodes'.
const fn exempt(graha: Graha) -> Option<Graha> {
    match graha {
        Graha::Sun => Some(Graha::Saturn),
        Graha::Saturn => Some(Graha::Sun),
        Graha::Moon => Some(Graha::Mercury),
        Graha::Mercury => Some(Graha::Moon),
        _ => None,
    }
}

const fn is_node(graha: Graha) -> bool {
    matches!(graha, Graha::Rahu | Graha::Ketu)
}

/// A graha's good houses with their vedha houses; `None` in the vedha
/// house of a node read as unobstructable.
fn pairs(graha: Graha, rules: GocharRules) -> impl Iterator<Item = (u8, Option<u8>)> {
    let (table, obstructable) = match graha {
        Graha::Rahu | Graha::Ketu => (SUN, rules.node_vedha == NodeVedha::LikeTheSun),
        other => (VEDHA.get(other as usize).copied().unwrap_or(&[]), true),
    };
    table
        .iter()
        .map(move |(good, vedha)| (*good, obstructable.then_some(*vedha)))
}

/// The house `sign` stands in counted from `reference`, 1 to 12.
fn house(reference: Rashi, sign: Rashi) -> u8 {
    u8::try_from((sign as usize + 12 - reference as usize) % 12 + 1).unwrap_or(1)
}

/// Every graha's transit read from `reference`, the Sun to Ketu, under the
/// settings' readings of what the text leaves open.
#[must_use]
pub fn gochar(reference: Rashi, transits: &[Transit; 9], rules: GocharRules) -> GocharReading {
    let houses = transits.map(|transit| house(reference, transit.sign));
    let grahas = GRAHAS.map(|graha| {
        let index = graha as usize;
        let transit = transits
            .get(index)
            .copied()
            .unwrap_or(Transit::new(reference, 0.0));
        let at = houses.get(index).copied().unwrap_or(1);
        let pair = pairs(graha, rules).find(|(good, _)| *good == at);
        let vedha_house = pair.and_then(|(_, vedha)| vedha);
        let obstructed_by: Vec<Graha> = vedha_house.map_or_else(Vec::new, |vedha| {
            GRAHAS
                .into_iter()
                .filter(|other| {
                    *other != graha
                        && exempt(graha) != Some(*other)
                        && !(is_node(*other) && rules.node_obstruction == NodeObstruction::None)
                        && houses.get(*other as usize) == Some(&vedha)
                })
                .collect()
        });
        let verdict = match (pair.is_some(), obstructed_by.is_empty()) {
            (false, _) => Verdict::NotGood,
            (true, true) => Verdict::Good,
            (true, false) => Verdict::Obstructed,
        };
        let fruition = Fruition::of(graha);
        GrahaGochar {
            graha,
            transit,
            house: at,
            good_house: pair.is_some(),
            vedha_house,
            obstructed_by,
            verdict,
            fruition,
            fruitful_now: fruition.holds(transit.degrees),
        }
    });
    GocharReading {
        reference,
        rules,
        grahas,
    }
}

#[cfg(test)]
mod tests;

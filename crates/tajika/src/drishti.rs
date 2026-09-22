//! The **Tajika aspects**, their orbs, and whether two planets are coming
//! together or drawing apart (`03-design/tajika-aspects.md`).
//!
//! Two different things wear the word "aspect" in Tajika, and keeping them
//! apart is most of this module:
//!
//! - **the aspect itself**, which is a relation between two *signs* —
//!   friendly at houses 3, 5, 9 and 11, inimical at the kendras 1, 4, 7
//!   and 10, and **nothing at all** at 2, 6, 8 and 12. That is what the
//!   lord of the year is asked for ([`crate::varshesha`]);
//! - **the orb**, the *deeptamsha* or "orb of radiance", which is a
//!   distance between two *planets* in degrees. Each planet has its own,
//!   and the one that governs a pair is the **mean** of the two.
//!
//! A pair inside its orb is doing one of two things. If the **faster**
//! planet is behind the slower — at fewer degrees within its sign, which
//! is how this tradition counts "behind" — they are coming together:
//! **Ithasala**, and the source calls it generally favourable. If the
//! faster is already past, they are drawing apart: **Ishrafa**. Neither
//! means anything without the sign aspect as well.
//!
//! "Behind" is **degrees within the sign**, not longitude along the
//! zodiac. The source is explicit: delete the completed signs and compare
//! what is left. So a planet at 3° of Leo is behind one at 7° of Scorpio,
//! four signs away.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;

use crate::bala::{AnnualSky, SEVEN, sign_of_longitude};

/// Where a planet is called to be at the end of its sign, degrees.
///
/// A planet at 29° or more "extends its influence to the next house
/// also", and so can make an Ithasala it would otherwise be too far
/// advanced for.
pub const RASHYANTA_DEG: f64 = 29.0;

/// The Tajika aspect between two signs.
///
/// Four kinds and a fifth that is no aspect at all. The source divides
/// each of the two aspecting kinds in two — the wholehearted and the
/// secret — and the year lord's rule ignores the division while the yogas
/// do not, so it is carried rather than collapsed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Drishti {
    /// **Pratyaksha Mitra**, at houses 5 and 9: openly friendly.
    Friendly,
    /// **Gupta Mitra**, at houses 3 and 11: secretly friendly.
    SecretlyFriendly,
    /// **Pratyaksha Shatru**, at houses 1 and 7: openly inimical — and an
    /// aspect, which is why two planets in one sign aspect each other.
    Inimical,
    /// **Gupta Shatru**, at houses 4 and 10: secretly inimical.
    SecretlyInimical,
    /// **Sama**, at houses 2, 6, 8 and 12: no aspect at all, which is what
    /// disqualifies an office-bearer however strong it is.
    None,
}

impl Drishti {
    /// The aspect a planet in `from` casts on `to`.
    ///
    /// Symmetric, as every one of the four sets is: 3 and 11 are each
    /// other's, as are 5 and 9, 1 and 7, 4 and 10.
    #[must_use]
    pub fn between_signs(from: Rashi, to: Rashi) -> Drishti {
        match (to.id() + 12 - from.id()) % 12 + 1 {
            5 | 9 => Drishti::Friendly,
            3 | 11 => Drishti::SecretlyFriendly,
            1 | 7 => Drishti::Inimical,
            4 | 10 => Drishti::SecretlyInimical,
            _ => Drishti::None,
        }
    }

    /// Whether this is an aspect at all.
    #[must_use]
    pub const fn is_aspect(self) -> bool {
        !matches!(self, Drishti::None)
    }

    /// Whether it is one of the two friendly kinds.
    #[must_use]
    pub const fn is_friendly(self) -> bool {
        matches!(self, Drishti::Friendly | Drishti::SecretlyFriendly)
    }
}

/// Each planet's **deeptamsha**, its orb of radiance, degrees (K.S.
/// Charak, *A Textbook of Varshaphala*, Table X-1).
///
/// It reaches that far on either side of the planet. The orb that governs
/// a **pair** is the mean of their two, which the source states in the
/// same breath: "for the purpose of the Ithasala yoga, the term
/// Deeptamsha range means a mean of the individual Deeptamshas of the two
/// planets participating".
const DEEPTAMSHA_DEG: [(Graha, f64); 7] = [
    (Graha::Sun, 15.0),
    (Graha::Moon, 12.0),
    (Graha::Mars, 8.0),
    (Graha::Mercury, 7.0),
    (Graha::Jupiter, 9.0),
    (Graha::Venus, 7.0),
    (Graha::Saturn, 9.0),
];

/// One planet's orb of radiance, degrees; nothing for a body that has
/// none, which is every body but the seven.
#[must_use]
pub fn deeptamsha(graha: Graha) -> Option<f64> {
    DEEPTAMSHA_DEG
        .into_iter()
        .find(|(who, _)| *who == graha)
        .map(|(_, orb)| orb)
}

/// The orb governing a pair: the mean of their two.
#[must_use]
pub fn orb_between(a: Graha, b: Graha) -> Option<f64> {
    Some(f64::midpoint(deeptamsha(a)?, deeptamsha(b)?))
}

/// The seven in order of speed, **fastest first**, as the source ranks
/// them: "the planets Moon, Mercury, Venus, Sun, Mars, Jupiter and Saturn
/// are considered to be progressively slower in motion in this order".
///
/// A ranking and not a measurement: the tradition treats the order as
/// fixed, so a retrograde Mars is still slower than the Sun here.
pub const BY_SPEED: [Graha; 7] = [
    Graha::Moon,
    Graha::Mercury,
    Graha::Venus,
    Graha::Sun,
    Graha::Mars,
    Graha::Jupiter,
    Graha::Saturn,
];

/// Where a planet stands in the speed order; nothing for a body outside
/// the seven.
#[must_use]
pub fn speed_rank(graha: Graha) -> Option<usize> {
    BY_SPEED.iter().position(|who| *who == graha)
}

/// What two planets inside each other's orb are doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Yoga {
    /// **Ithasala**: the faster planet is behind the slower and coming to
    /// it. Generally favourable.
    Ithasala,
    /// **Ithasala from the sign's end** (Rashyanta Muthsila): the faster
    /// planet is past the slower, but stands at 29° or more and so acts
    /// from the start of the next sign, where it is behind again.
    RashyantaIthasala,
    /// **Ishrafa**: the faster planet is already past the slower and
    /// drawing away. Generally unfavourable, though the source says an
    /// Ishrafa between two benefics is not.
    Ishrafa,
}

/// How two planets of an annual chart stand to each other.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Between {
    /// The faster of the two, by the tradition's ranking.
    pub faster: Graha,
    /// The slower.
    pub slower: Graha,
    /// The aspect between the signs they stand in.
    pub drishti: Drishti,
    /// The orb governing them, degrees: the mean of their deeptamshas.
    pub orb_deg: f64,
    /// How far apart they are **within their signs**, degrees: the
    /// slower's degrees less the faster's, signed, which is what decides
    /// behind from ahead.
    pub apart_deg: f64,
    /// Whether they are inside that orb.
    pub within: bool,
    /// Whether the faster stands at [`RASHYANTA_DEG`] or beyond.
    pub rashyanta: bool,
    /// What they are doing, when they aspect each other and are inside
    /// the orb; nothing when either fails.
    pub yoga: Option<Yoga>,
}

/// How two of the seven stand to each other in an annual chart.
///
/// # Errors
///
/// A body outside the seven, named `graha`; a longitude that is not a
/// number, named by its own field.
pub fn between(a: Graha, b: Graha, sky: &AnnualSky) -> Result<Between, Error> {
    let outside = |graha: Graha| {
        Error::invalid_arg(format!("{graha:?} has no deeptamsha; the seven have"))
            .with_field(String::from("graha"))
    };
    let (rank_a, rank_b) = (
        speed_rank(a).ok_or_else(|| outside(a))?,
        speed_rank(b).ok_or_else(|| outside(b))?,
    );
    let orb_deg = orb_between(a, b).ok_or_else(|| outside(a))?;
    // The faster of the two; on the same planet, itself.
    let (faster, slower) = if rank_a <= rank_b { (a, b) } else { (b, a) };
    let at = |graha: Graha| -> Result<f64, Error> {
        let longitude = sky.longitude_of(graha);
        if longitude.is_finite() {
            Ok(longitude)
        } else {
            Err(outside(graha))
        }
    };
    // "Behind" is degrees **within the sign**, the completed signs
    // deleted, which is the source's own instruction and not a longitude.
    let within_sign = |longitude: f64| longitude.rem_euclid(30.0);
    let (fast_deg, slow_deg) = (within_sign(at(faster)?), within_sign(at(slower)?));
    let drishti = Drishti::between_signs(
        sign_of_longitude(at(faster)?),
        sign_of_longitude(at(slower)?),
    );
    let apart_deg = slow_deg - fast_deg;
    let within = apart_deg.abs() <= orb_deg;
    let rashyanta = fast_deg >= RASHYANTA_DEG;
    let yoga = if !drishti.is_aspect() || faster == slower {
        None
    } else if within && apart_deg >= 0.0 {
        Some(Yoga::Ithasala)
    } else if within {
        Some(Yoga::Ishrafa)
    } else if rashyanta && slow_deg <= orb_deg {
        // Past the slower and at the sign's end, so it acts from the next
        // sign's beginning, where it is behind again and inside the orb.
        Some(Yoga::RashyantaIthasala)
    } else {
        None
    };
    Ok(Between {
        faster,
        slower,
        drishti,
        orb_deg,
        apart_deg,
        within,
        rashyanta,
        yoga,
    })
}

/// Every pair of the seven, in the catalogue's order: twenty-one of them.
///
/// # Errors
///
/// As [`between`].
pub fn all(sky: &AnnualSky) -> Result<Vec<Between>, Error> {
    let mut out = Vec::with_capacity(21);
    for (index, a) in SEVEN.into_iter().enumerate() {
        for b in SEVEN.into_iter().skip(index + 1) {
            out.push(between(a, b, sky)?);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use super::{
        BY_SPEED, Between, Drishti, RASHYANTA_DEG, Yoga, all, between, deeptamsha, orb_between,
        speed_rank,
    };
    use crate::bala::AnnualSky;
    use teistro_core::catalogue::{Graha, Rashi};

    fn worked() -> AnnualSky {
        let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
        AnnualSky {
            sun_deg: at(4.0, 3.0, 50.0),
            moon_deg: at(1.0, 9.0, 40.0),
            mars_deg: at(7.0, 7.0, 42.0),
            mercury_deg: at(4.0, 18.0, 20.0),
            jupiter_deg: at(8.0, 9.0, 38.0),
            venus_deg: at(4.0, 21.0, 45.0),
            saturn_deg: at(6.0, 17.0, 13.0),
        }
    }

    /// The source's worked Ithasala: the Sun at Leo 3°50′ and Mars at
    /// Scorpio 7°42′, whose orb is the mean of 15° and 8° — **11°30′** —
    /// and which stand **3°52′** apart within their signs. The Sun is the
    /// faster and behind, so they are in Ithasala.
    #[test]
    fn the_sources_worked_ithasala_is_reproduced() {
        let found = between(Graha::Sun, Graha::Mars, &worked()).unwrap();
        assert_eq!(found.faster, Graha::Sun);
        assert_eq!(found.slower, Graha::Mars);
        assert!((found.orb_deg - 11.5).abs() < 1e-12, "{}", found.orb_deg);
        let printed = 3.0 + 52.0 / 60.0;
        assert!(
            (found.apart_deg - printed).abs() < 1.0 / 60.0,
            "{} against 3°52′",
            found.apart_deg
        );
        assert!(found.within);
        assert_eq!(found.yoga, Some(Yoga::Ithasala));
        // Leo and Scorpio are four houses apart: an aspect, and an
        // inimical one, which the source says counts the same here.
        assert_eq!(found.drishti, Drishti::SecretlyInimical);
        assert!(found.drishti.is_aspect());
    }

    /// The same pair asked the other way round answers the same thing:
    /// which of two is faster is the tradition's ranking, not the order
    /// of the arguments.
    #[test]
    fn a_pair_reads_the_same_from_either_end() {
        let one = between(Graha::Sun, Graha::Mars, &worked()).unwrap();
        let other = between(Graha::Mars, Graha::Sun, &worked()).unwrap();
        assert_eq!(one, other);
    }

    /// The orbs are the source's table, and a pair's orb is their mean.
    #[test]
    fn the_orbs_are_the_sources_and_a_pair_takes_their_mean() {
        let printed = [
            (Graha::Sun, 15.0),
            (Graha::Moon, 12.0),
            (Graha::Mars, 8.0),
            (Graha::Mercury, 7.0),
            (Graha::Jupiter, 9.0),
            (Graha::Venus, 7.0),
            (Graha::Saturn, 9.0),
        ];
        for (graha, orb) in printed {
            assert_eq!(deeptamsha(graha), Some(orb), "{graha:?}");
        }
        assert_eq!(orb_between(Graha::Sun, Graha::Mars), Some(11.5));
        assert_eq!(orb_between(Graha::Mercury, Graha::Venus), Some(7.0));
        // The nodes have none, so no pair with one has an orb.
        assert_eq!(deeptamsha(Graha::Rahu), None);
        assert_eq!(orb_between(Graha::Rahu, Graha::Sun), None);
    }

    /// The speed order is the tradition's ranking, fastest first, and
    /// every one of the seven is in it exactly once.
    #[test]
    fn the_seven_are_ranked_by_speed_fastest_first() {
        assert_eq!(BY_SPEED[0], Graha::Moon);
        assert_eq!(BY_SPEED[6], Graha::Saturn);
        assert_eq!(speed_rank(Graha::Sun), Some(3));
        let mut seen: Vec<Graha> = BY_SPEED.to_vec();
        seen.sort_by_key(|graha| graha.id());
        seen.dedup();
        assert_eq!(seen.len(), 7);
        assert_eq!(speed_rank(Graha::Ketu), None);
    }

    /// The four aspecting sets and the one that is no aspect, each
    /// symmetric, over the whole circle.
    #[test]
    fn the_aspects_are_the_four_sets_and_the_neutral_houses_are_none() {
        let mut counts = (0, 0, 0, 0, 0);
        for from in Rashi::ALL {
            for to in Rashi::ALL {
                let drishti = Drishti::between_signs(from, to);
                assert_eq!(drishti, Drishti::between_signs(to, from), "symmetric");
                match drishti {
                    Drishti::Friendly => counts.0 += 1,
                    Drishti::SecretlyFriendly => counts.1 += 1,
                    Drishti::Inimical => counts.2 += 1,
                    Drishti::SecretlyInimical => counts.3 += 1,
                    Drishti::None => counts.4 += 1,
                }
            }
        }
        // Two houses to each aspecting kind and four to the neutral, over
        // twelve signs: 24, 24, 24, 24 and 48.
        assert_eq!(counts, (24, 24, 24, 24, 48));
        assert!(Drishti::Friendly.is_friendly() && Drishti::SecretlyFriendly.is_friendly());
        assert!(!Drishti::Inimical.is_friendly() && Drishti::Inimical.is_aspect());
        assert!(!Drishti::None.is_aspect());
    }

    /// Behind is **degrees within the sign**, not longitude: the Sun at
    /// Leo 3°50′ is behind Mars at Scorpio 7°42′ although its longitude is
    /// the smaller by three whole signs.
    #[test]
    fn behind_is_degrees_within_the_sign_and_not_longitude() {
        let sky = worked();
        let found = between(Graha::Sun, Graha::Mars, &sky).unwrap();
        assert!(found.apart_deg > 0.0, "the faster is behind");
        assert!(sky.longitude_of(Graha::Sun) < sky.longitude_of(Graha::Mars));
        // Move Mars back within its own sign and the Sun is now ahead:
        // the same three signs of longitude between them, the other yoga.
        let moved = AnnualSky {
            mars_deg: 7.0 * 30.0 + 1.0,
            ..sky
        };
        let apart = between(Graha::Sun, Graha::Mars, &moved).unwrap();
        assert!(apart.apart_deg < 0.0, "the faster is now ahead");
        assert_eq!(apart.yoga, Some(Yoga::Ishrafa));
    }

    /// A planet past the slower and at the end of its sign makes the
    /// Ithasala anyway, from the next sign's beginning.
    #[test]
    fn a_planet_at_the_signs_end_reaches_across_it() {
        // The Moon at Taurus 29°30′ and Venus at Leo 2°: the Moon is the
        // faster and is past, so no plain Ithasala; being at Rashyanta it
        // acts from Gemini 0°, where it is behind Venus and inside the
        // orb of 9°30′.
        let sky = AnnualSky {
            moon_deg: 30.0 + 29.5,
            venus_deg: 4.0 * 30.0 + 2.0,
            ..worked()
        };
        let found = between(Graha::Moon, Graha::Venus, &sky).unwrap();
        assert_eq!(found.faster, Graha::Moon);
        assert!(found.rashyanta);
        assert!(!found.within, "too far apart within their signs");
        assert_eq!(found.yoga, Some(Yoga::RashyantaIthasala));
    }

    /// No aspect, no yoga, however close the two stand.
    #[test]
    fn the_neutral_houses_make_no_yoga_however_close() {
        // Mars at Scorpio 7° and Jupiter at Sagittarius 7°: the second
        // house from the first, which aspects nothing.
        let sky = AnnualSky {
            mars_deg: 7.0 * 30.0 + 7.0,
            jupiter_deg: 8.0 * 30.0 + 7.0,
            ..worked()
        };
        let found = between(Graha::Mars, Graha::Jupiter, &sky).unwrap();
        assert_eq!(found.drishti, Drishti::None);
        assert!(found.within, "inside the orb and still nothing");
        assert_eq!(found.yoga, None);
    }

    /// Every pair of the seven, once each.
    #[test]
    fn every_pair_of_the_seven_is_read_once() {
        let pairs: Vec<Between> = all(&worked()).unwrap();
        assert_eq!(pairs.len(), 21);
        let mut seen: Vec<(u16, u16)> = pairs
            .iter()
            .map(|pair| {
                let (a, b) = (pair.faster.id(), pair.slower.id());
                if a <= b { (a, b) } else { (b, a) }
            })
            .collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 21, "no pair twice");
        // And the faster is never ranked slower than the slower.
        assert!(
            pairs
                .iter()
                .all(|pair| speed_rank(pair.faster) <= speed_rank(pair.slower))
        );
    }

    #[test]
    fn a_body_outside_the_seven_is_refused_by_that_field() {
        let why = between(Graha::Rahu, Graha::Sun, &worked()).expect_err("refused");
        assert_eq!(why.field(), Some("graha"));
        assert!(why.to_string().contains("deeptamsha"), "{why}");
    }

    #[test]
    fn the_signs_end_is_the_sources_own_figure() {
        assert!((RASHYANTA_DEG - 29.0).abs() < f64::EPSILON);
    }
}

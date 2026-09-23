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
//! The source's Table X-3 gives the Ithasala **three kinds**, which this
//! module ships as three variants of one [`Yoga`]: *Vartamana*, the
//! present, where the faster is behind by a degree or more; *Poorna*, the
//! full, within a single degree and so already fulfilled; and
//! *Bhavishyat*, the future, where the faster is past but stands at a
//! sign's end and so acts from the next sign, where it is behind again.
//! The chapter's prose names that last one *Rashyanta* instead and knows
//! no Poorna at all; where the two accounts differ, [`SubDegree`] carries
//! the readings and `03-design/tajika-yogas.md` counts what turns on
//! them.
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

/// Where a pair is called complete, degrees apart within their signs.
///
/// The source's Table X-3 gives the Ithasala **three** kinds and marks
/// the one within a single degree as immediate fulfilment; the same table
/// puts the same degree on the far side of Ishrafa, which begins "one
/// degree or more" past. One threshold, named once.
pub const POORNA_DEG: f64 = 1.0;

/// What two planets inside each other's orb are doing.
///
/// Three of these four are kinds of **Ithasala**, the coming-together:
/// Table X-3 enumerates them and [`Yoga::is_ithasala`] asks the question
/// the other fifteen yogas actually ask, which is whether a pair is in an
/// Ithasala of any kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Yoga {
    /// **Vartamana Ithasala**, the present one: the faster planet is
    /// behind the slower by [`POORNA_DEG`] or more, inside the orb, and
    /// coming to it. Generally favourable, and the commonest of the four.
    IthasalaVartamana,
    /// **Poorna Ithasala**, the full one: as Vartamana but within a
    /// single degree, which the source marks as immediate fulfilment
    /// rather than a promise.
    ///
    /// Whether a pair a fraction of a degree *past* is also Poorna is the
    /// one thing the source's two accounts do not settle; see
    /// [`SubDegree`], which decides it, and `03-design/tajika-yogas.md`,
    /// which counts what turns on it.
    IthasalaPoorna,
    /// **Bhavishyat Ithasala**, the future one: the faster planet is past
    /// the slower, but stands at [`RASHYANTA_DEG`] or more and so acts
    /// from the start of the next sign, where it is behind again.
    ///
    /// The chapter's prose calls this same configuration a *Rashyanta*
    /// Ithasala. Table X-3's name ships, because the table is what
    /// enumerates the kinds (crux C111).
    IthasalaBhavishyat,
    /// **Ishrafa**: the faster planet is past the slower by
    /// [`POORNA_DEG`] or more and drawing away. Generally unfavourable,
    /// though the source says an Ishrafa between two benefics is not.
    ///
    /// The degree is the table's; the chapter's prose asks only that the
    /// faster be past (crux C110).
    Ishrafa,
}

impl Yoga {
    /// Whether this is an Ithasala of any of its three kinds.
    ///
    /// Fourteen of the sixteen yogas are built on "an Ithasala", without
    /// caring which; this is that question, asked once.
    #[must_use]
    pub const fn is_ithasala(self) -> bool {
        matches!(
            self,
            Yoga::IthasalaVartamana | Yoga::IthasalaPoorna | Yoga::IthasalaBhavishyat
        )
    }
}

/// What a pair less than a degree **past** is doing — the one place the
/// source's two accounts of a pair disagree.
///
/// Table X-3 says Ishrafa begins a whole degree past and gives Poorna as
/// a narrowing of Vartamana, which is stated only for a faster planet
/// *behind*. Between them sits a band the table names twice and places
/// once. The chapter's prose knows no Poorna at all and calls everything
/// past an Ishrafa.
///
/// Over the recorded births' first forty years this is **934 pairs of the
/// 29 166 that aspect at all** — a twelfth of every Ishrafa — so the
/// three readings are carried and named rather than one being chosen
/// silently. `03-design/tajika-yogas.md` holds the count and the
/// argument.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SubDegree {
    /// **Poorna**, the default: the table's two rows read so that they
    /// interlock — a pair within a degree either way is complete, and
    /// Ishrafa begins at exactly the degree the table names. The only
    /// reading under which the table's own threshold does any work.
    #[default]
    Poorna,
    /// **Ishrafa**: the chapter's prose, where anything past is drawing
    /// away however near. What this module shipped before Table X-3 was
    /// read.
    Ishrafa,
    /// **Nothing**: the table at its narrowest, where Poorna only ever
    /// narrows Vartamana and the band belongs to no yoga. Carried because
    /// it is a defensible reading of the printed words, though it leaves
    /// those 934 pairs saying nothing at all.
    None,
}

/// Which readings of the source this module applies.
///
/// One field today. It is a struct rather than a bare enum so that a
/// later disagreement costs callers nothing, as [`crate::VarsheshaRules`]
/// already does for the lord of the year.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct DrishtiRules {
    /// What a pair less than a degree past is doing.
    pub sub_degree: SubDegree,
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

impl Between {
    /// Whether this pair falls in the band the source's two accounts
    /// place differently: aspecting, inside the orb, and the faster less
    /// than [`POORNA_DEG`] past the slower.
    ///
    /// True here means [`Between::yoga`] is whatever
    /// [`DrishtiRules::sub_degree`] said, and a reader who wants to show
    /// the judgement as contested can ask rather than reconstructing the
    /// range from [`Between::apart_deg`].
    #[must_use]
    pub fn disputed(&self) -> bool {
        self.drishti.is_aspect()
            && self.faster != self.slower
            && self.within
            && self.apart_deg < 0.0
            && self.apart_deg > -POORNA_DEG
    }
}

/// How two of the seven stand to each other in an annual chart, under the
/// readings this module defaults to.
///
/// [`between_with_rules`] takes the readings; this is that with
/// [`DrishtiRules::default`].
///
/// # Errors
///
/// A body outside the seven, named `graha`; a longitude that is not a
/// number, named by its own field.
pub fn between(a: Graha, b: Graha, sky: &AnnualSky) -> Result<Between, Error> {
    between_with_rules(a, b, sky, DrishtiRules::default())
}

/// How two of the seven stand to each other, under stated readings.
///
/// # Errors
///
/// As [`between`].
pub fn between_with_rules(
    a: Graha,
    b: Graha,
    sky: &AnnualSky,
    rules: DrishtiRules,
) -> Result<Between, Error> {
    let orb_deg = orb_between(a, b).ok_or_else(|| {
        Error::invalid_arg(format!("{a:?} has no deeptamsha; the seven have"))
            .with_field(String::from("graha"))
    })?;
    between_within(a, b, sky, orb_deg, rules)
}

/// How two of the seven stand, under an orb the caller names rather than
/// the mean of their two deeptamshas.
///
/// Nakta and Yamaya need this: the source asks whether a third planet
/// reaches both of a pair **from within its own deeptamsha**, not from
/// within the mean it would share with each. Everything else about the
/// reckoning is the same, so it is the same function with the orb lifted
/// out rather than a second copy of the four bands.
///
/// It is crate-private on purpose. The orb is a real knob and a consumer
/// composing a yoga of their own would want it, but nothing outside has
/// asked; making it public is one line the day something does.
///
/// # Errors
///
/// As [`between`].
pub(crate) fn between_within(
    a: Graha,
    b: Graha,
    sky: &AnnualSky,
    orb_deg: f64,
    rules: DrishtiRules,
) -> Result<Between, Error> {
    let outside = |graha: Graha| {
        Error::invalid_arg(format!("{graha:?} has no deeptamsha; the seven have"))
            .with_field(String::from("graha"))
    };
    let (rank_a, rank_b) = (
        speed_rank(a).ok_or_else(|| outside(a))?,
        speed_rank(b).ok_or_else(|| outside(b))?,
    );
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
    } else if within && apart_deg >= POORNA_DEG {
        // Behind by a degree or more: coming to it, and not yet there.
        Some(Yoga::IthasalaVartamana)
    } else if within && apart_deg >= 0.0 {
        // Behind by less than a degree. Poorna on every reading of the
        // source: this is the side Table X-3 states it for.
        Some(Yoga::IthasalaPoorna)
    } else if within && apart_deg > -POORNA_DEG {
        // Past by less than a degree: the one band the source's two
        // accounts place differently, so the caller's reading decides.
        match rules.sub_degree {
            SubDegree::Poorna => Some(Yoga::IthasalaPoorna),
            SubDegree::Ishrafa => Some(Yoga::Ishrafa),
            SubDegree::None => None,
        }
    } else if within {
        // Past by a degree or more: Ishrafa on every reading.
        Some(Yoga::Ishrafa)
    } else if rashyanta && slow_deg <= orb_deg {
        // Past the slower and at the sign's end, so it acts from the next
        // sign's beginning, where it is behind again and inside the orb.
        Some(Yoga::IthasalaBhavishyat)
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
    all_with_rules(sky, DrishtiRules::default())
}

/// Every pair of the seven, under stated readings.
///
/// # Errors
///
/// As [`between`].
pub fn all_with_rules(sky: &AnnualSky, rules: DrishtiRules) -> Result<Vec<Between>, Error> {
    let mut out = Vec::with_capacity(21);
    for (index, a) in SEVEN.into_iter().enumerate() {
        for b in SEVEN.into_iter().skip(index + 1) {
            out.push(between_with_rules(a, b, sky, rules)?);
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
        BY_SPEED, Between, Drishti, DrishtiRules, POORNA_DEG, RASHYANTA_DEG, SubDegree, Yoga, all,
        all_with_rules, between, between_with_rules, deeptamsha, orb_between, speed_rank,
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
    /// faster and behind, so they are in Ithasala — and by more than a
    /// degree, so it is the **Vartamana** kind of the three.
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
        assert_eq!(found.yoga, Some(Yoga::IthasalaVartamana));
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
    /// Ithasala anyway, from the next sign's beginning: the **Bhavishyat**
    /// kind, which Table X-3 names and the chapter's prose calls
    /// Rashyanta (crux C111).
    #[test]
    fn a_planet_at_the_signs_end_reaches_across_it() {
        // The Moon at Taurus 29°30′ and Venus at Leo 2°: the Moon is the
        // faster and is past, so no Vartamana; being at Rashyanta it
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
        assert_eq!(found.yoga, Some(Yoga::IthasalaBhavishyat));
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

    /// A pair the Sun leads by a stated number of degrees within their
    /// signs, in signs that aspect each other: Leo and Scorpio, the
    /// fourth from the first.
    ///
    /// `behind` is how far the Sun stands **before** Mars; negative is
    /// past it.
    fn sun_behind_mars_by(behind: f64) -> AnnualSky {
        AnnualSky {
            sun_deg: 4.0 * 30.0 + 10.0,
            mars_deg: 7.0 * 30.0 + 10.0 + behind,
            ..worked()
        }
    }

    /// Table X-3's three kinds of Ithasala, each at its own distance, and
    /// [`Yoga::is_ithasala`] answering for all three.
    #[test]
    fn the_ithasala_has_three_kinds_and_a_degree_divides_two_of_them() {
        let kind = |behind: f64| {
            between(Graha::Sun, Graha::Mars, &sun_behind_mars_by(behind))
                .unwrap()
                .yoga
        };
        // A degree or more behind is the present kind; inside a degree it
        // is already full. The boundary belongs to Vartamana.
        assert_eq!(kind(5.0), Some(Yoga::IthasalaVartamana));
        assert_eq!(kind(POORNA_DEG), Some(Yoga::IthasalaVartamana));
        assert_eq!(kind(POORNA_DEG - 0.001), Some(Yoga::IthasalaPoorna));
        assert_eq!(kind(0.0), Some(Yoga::IthasalaPoorna));
        // Past by a degree or more is Ishrafa on every reading.
        assert_eq!(kind(-POORNA_DEG), Some(Yoga::Ishrafa));
        assert_eq!(kind(-5.0), Some(Yoga::Ishrafa));
        // Outside the orb of 11°30′ is nothing at all.
        assert_eq!(kind(12.0), None);
        for yoga in [
            Yoga::IthasalaVartamana,
            Yoga::IthasalaPoorna,
            Yoga::IthasalaBhavishyat,
        ] {
            assert!(yoga.is_ithasala(), "{yoga:?} is a kind of Ithasala");
        }
        assert!(!Yoga::Ishrafa.is_ithasala());
    }

    /// The one band the source's two accounts place differently: the
    /// faster less than a degree past. All three readings are reachable
    /// and the default is the table's.
    #[test]
    fn a_pair_less_than_a_degree_past_answers_to_the_reading_asked_for() {
        let sky = sun_behind_mars_by(-0.5);
        let under = |sub_degree| {
            between_with_rules(Graha::Sun, Graha::Mars, &sky, DrishtiRules { sub_degree })
                .unwrap()
                .yoga
        };
        assert_eq!(under(SubDegree::Poorna), Some(Yoga::IthasalaPoorna));
        assert_eq!(under(SubDegree::Ishrafa), Some(Yoga::Ishrafa));
        assert_eq!(under(SubDegree::None), None);
        // The default is the table read so that its two rows interlock.
        assert_eq!(
            between(Graha::Sun, Graha::Mars, &sky).unwrap().yoga,
            Some(Yoga::IthasalaPoorna)
        );
        assert_eq!(DrishtiRules::default().sub_degree, SubDegree::Poorna);
    }

    /// `disputed` marks exactly the band the rules decide, and nothing
    /// else: not the Poorna the table states outright, not an Ishrafa a
    /// degree or more past, and not a close pair that does not aspect.
    #[test]
    fn disputed_marks_the_band_the_readings_differ_over() {
        let at =
            |behind: f64| between(Graha::Sun, Graha::Mars, &sun_behind_mars_by(behind)).unwrap();
        assert!(at(-0.5).disputed());
        assert!(at(-0.001).disputed());
        assert!(!at(0.0).disputed(), "stated by the table, not disputed");
        assert!(!at(0.5).disputed());
        assert!(!at(-POORNA_DEG).disputed(), "Ishrafa on every reading");
        assert!(!at(12.0).disputed(), "outside the orb");
        // Mars and Jupiter stand in the neutral second house: close, past,
        // and making nothing on any reading.
        let neutral = AnnualSky {
            mars_deg: 7.0 * 30.0 + 7.5,
            jupiter_deg: 8.0 * 30.0 + 7.0,
            ..worked()
        };
        let found = between(Graha::Mars, Graha::Jupiter, &neutral).unwrap();
        assert_eq!(found.drishti, Drishti::None);
        assert!(!found.disputed(), "no aspect, nothing to dispute");
    }

    /// The rules reach every pair, not only the one asked about: the
    /// whole-sky reading takes them too.
    #[test]
    fn the_whole_sky_takes_the_reading_as_well() {
        let sky = sun_behind_mars_by(-0.5);
        let under = |sub_degree| {
            all_with_rules(&sky, DrishtiRules { sub_degree })
                .unwrap()
                .into_iter()
                .find(|pair| pair.faster == Graha::Sun && pair.slower == Graha::Mars)
                .unwrap()
                .yoga
        };
        assert_eq!(under(SubDegree::Poorna), Some(Yoga::IthasalaPoorna));
        assert_eq!(under(SubDegree::Ishrafa), Some(Yoga::Ishrafa));
        assert_eq!(under(SubDegree::None), None);
        assert_eq!(all(&sky).unwrap().len(), 21);
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

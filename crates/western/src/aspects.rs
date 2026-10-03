//! Which Western aspects a set of bodies holds, under a model of orbs
//! (`03-design/western-aspects.md`).
//!
//! The set and the default orbs are Alan Leo's (*How to Judge a Nativity*,
//! pp. 43–47); the moieties are William Lilly's (*Christian Astrology*,
//! pp. 107, 110). The separation and whether a pair is closing are the
//! shared orb engine's, `teistro_aspect::orb`; this module only chooses
//! each angle's orb.
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_western::{AspectRequest, Placed, WesternAspect, aspects};
//!
//! // The Sun at 10° and Mars at 72°: a sextile two degrees wide, inside
//! // Leo's seven.
//! let bodies = [
//!     Placed::new(Graha::Sun, 10.0, 1.0),
//!     Placed::new(Graha::Mars, 72.0, 0.5),
//! ];
//! let found = aspects(&bodies, &AspectRequest::default())?;
//! assert_eq!(found.len(), 1);
//! assert_eq!(found[0].aspect, WesternAspect::Sextile);
//! assert!((found[0].from_exact_deg - 2.0).abs() < 1e-9);
//! assert!(found[0].applying, "the faster Sun closes the gap to 60°");
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_aspect::orb::{Angle, Moving, WIDEST_ORB_DEG, hits};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

/// An aspect Leo lists, by the angle between the two bodies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum WesternAspect {
    /// 0°.
    Conjunction,
    /// 30°, half a sextile.
    SemiSextile,
    /// 45°, half a square.
    SemiSquare,
    /// 60°.
    Sextile,
    /// 90°.
    Square,
    /// 120°.
    Trine,
    /// 135°, a square and a half.
    Sesquiquadrate,
    /// 150°.
    Quincunx,
    /// 180°.
    Opposition,
}

impl WesternAspect {
    /// Leo's nine, by their angle (pp. 43, 47).
    pub const ALL: [WesternAspect; 9] = [
        WesternAspect::Conjunction,
        WesternAspect::SemiSextile,
        WesternAspect::SemiSquare,
        WesternAspect::Sextile,
        WesternAspect::Square,
        WesternAspect::Trine,
        WesternAspect::Sesquiquadrate,
        WesternAspect::Quincunx,
        WesternAspect::Opposition,
    ];

    /// The five Ptolemaic aspects, which Lilly reads.
    pub const PTOLEMAIC: [WesternAspect; 5] = [
        WesternAspect::Conjunction,
        WesternAspect::Sextile,
        WesternAspect::Square,
        WesternAspect::Trine,
        WesternAspect::Opposition,
    ];

    /// The angle the orb engine measures it as.
    #[must_use]
    pub const fn angle(self) -> Angle {
        match self {
            WesternAspect::Conjunction => Angle::CONJUNCTION,
            WesternAspect::SemiSextile => Angle::SEMI_SEXTILE,
            WesternAspect::SemiSquare => Angle::SEMI_SQUARE,
            WesternAspect::Sextile => Angle::SEXTILE,
            WesternAspect::Square => Angle::SQUARE,
            WesternAspect::Trine => Angle::TRINE,
            WesternAspect::Sesquiquadrate => Angle::SESQUIQUADRATE,
            WesternAspect::Quincunx => Angle::QUINCUNX,
            WesternAspect::Opposition => Angle::OPPOSITION,
        }
    }

    /// Its key, as a request spells it: `SEMI_SQUARE`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        self.angle().key
    }

    /// The separation it names, degrees, 0 to 180.
    #[must_use]
    pub const fn degrees(self) -> f64 {
        self.angle().degrees
    }
}

/// One body's orb, for the moieties.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyOrb {
    /// Whose.
    pub graha: Graha,
    /// Its whole orb, degrees; a pair reads the mean of its two.
    pub orb_deg: f64,
}

/// One aspect's orb, for a caller's own table.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AspectOrb {
    /// Which aspect.
    pub aspect: WesternAspect,
    /// Its orb, degrees, whichever two bodies stand at it.
    pub orb_deg: f64,
}

/// How wide an aspect may be before it no longer holds (C240).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "model", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OrbModel {
    /// Leo's, by aspect (p. 47): the conjunction and opposition 12° for
    /// the Sun and the Moon, 10° for a luminary and a planet, and 8°
    /// between planets; the square and trine 8°; the sextile 7°; the
    /// semi-square and sesquiquadrate 4°; the semi-sextile and quincunx 2°.
    #[default]
    Leo,
    /// An orb for each body, a pair reading the mean of its two, their
    /// moieties (Lilly, p. 110). [`OrbModel::lilly`] is his table.
    Moieties {
        /// Each body's whole orb. A body the table leaves out is refused.
        orbs: Vec<BodyOrb>,
    },
    /// A caller's own orb for each aspect, whichever bodies stand at it.
    ByAspect {
        /// Each aspect's orb. An aspect asked for and left out is refused.
        orbs: Vec<AspectOrb>,
    },
}

impl OrbModel {
    /// Lilly's moieties: the seven planets' whole orbs (p. 107), Saturn
    /// 10°, Jupiter 12°, Mars 7½°, the Sun 17°, Venus 8°, Mercury 7°, the
    /// Moon 12½°. He gives none for the outer three.
    #[must_use]
    pub fn lilly() -> OrbModel {
        let orb = |graha, orb_deg| BodyOrb { graha, orb_deg };
        OrbModel::Moieties {
            orbs: vec![
                orb(Graha::Saturn, 10.0),
                orb(Graha::Jupiter, 12.0),
                orb(Graha::Mars, 7.5),
                orb(Graha::Sun, 17.0),
                orb(Graha::Venus, 8.0),
                orb(Graha::Mercury, 7.0),
                orb(Graha::Moon, 12.5),
            ],
        }
    }

    /// The orb an aspect between two bodies is allowed, degrees.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the body or aspect the model has no orb for.
    pub fn orb_deg(
        &self,
        aspect: WesternAspect,
        first: Graha,
        second: Graha,
    ) -> Result<f64, Error> {
        match self {
            OrbModel::Leo => Ok(leo_orb_deg(aspect, first, second)),
            OrbModel::Moieties { orbs } => {
                let of = |graha: Graha| {
                    orbs.iter()
                        .find(|one| one.graha == graha)
                        .map(|one| one.orb_deg)
                        .ok_or_else(|| {
                            Error::invalid_arg(format!("the moieties give {} no orb", graha.key()))
                                .with_field("orbs.orbs")
                                .with_hint(format!(
                                    "add an orb for {}, or leave it out of the bodies",
                                    graha.key()
                                ))
                        })
                };
                Ok(f64::midpoint(of(first)?, of(second)?))
            }
            OrbModel::ByAspect { orbs } => orbs
                .iter()
                .find(|one| one.aspect == aspect)
                .map(|one| one.orb_deg)
                .ok_or_else(|| {
                    Error::invalid_arg(format!(
                        "the orbs give {} no orb, though it is asked for",
                        aspect.key()
                    ))
                    .with_field("orbs.orbs")
                    .with_hint("give every aspect asked for an orb, or ask for fewer aspects")
                }),
        }
    }

    /// Every orb the model states is a finite angle from 0 to 90°, and
    /// no body or aspect is listed twice.
    fn check(&self) -> Result<(), Error> {
        let orbs: Vec<f64> = match self {
            OrbModel::Leo => return Ok(()),
            OrbModel::Moieties { orbs } => {
                refuse_repeats(orbs.iter().map(|one| one.graha.key()), "a body")?;
                orbs.iter().map(|one| one.orb_deg).collect()
            }
            OrbModel::ByAspect { orbs } => {
                refuse_repeats(orbs.iter().map(|one| one.aspect.key()), "an aspect")?;
                orbs.iter().map(|one| one.orb_deg).collect()
            }
        };
        match orbs
            .into_iter()
            .find(|orb| !(orb.is_finite() && (0.0..=WIDEST_ORB_DEG).contains(orb)))
        {
            Some(bad) => Err(Error::invalid_arg(format!(
                "an orb of {bad}° is outside 0 to {WIDEST_ORB_DEG}"
            ))
            .with_field("orbs.orbs")
            .with_hint(
                "past a right angle an orb no longer separates one aspect from its neighbours",
            )),
            None => Ok(()),
        }
    }
}

/// Leo's orb (p. 47).
const fn leo_orb_deg(aspect: WesternAspect, first: Graha, second: Graha) -> f64 {
    match aspect {
        WesternAspect::Conjunction | WesternAspect::Opposition => {
            match (is_luminary(first), is_luminary(second)) {
                (true, true) => 12.0,
                (true, false) | (false, true) => 10.0,
                (false, false) => 8.0,
            }
        }
        WesternAspect::Square | WesternAspect::Trine => 8.0,
        WesternAspect::Sextile => 7.0,
        WesternAspect::SemiSquare | WesternAspect::Sesquiquadrate => 4.0,
        WesternAspect::SemiSextile | WesternAspect::Quincunx => 2.0,
    }
}

/// The Sun or the Moon.
const fn is_luminary(graha: Graha) -> bool {
    matches!(graha, Graha::Sun | Graha::Moon)
}

/// Refuses a list naming one member twice, by its key.
fn refuse_repeats(members: impl Iterator<Item = &'static str>, what: &str) -> Result<(), Error> {
    let seen: Vec<&str> = members.collect();
    for (at, one) in seen.iter().enumerate() {
        if seen.iter().skip(at + 1).any(|other| other == one) {
            return Err(Error::invalid_arg(format!("{what} is named twice: {one}"))
                .with_field("orbs.orbs")
                .with_hint("name each once"));
        }
    }
    Ok(())
}

/// The record's name where a binding sends it, which a refusal is named
/// under.
const ROOT: &str = "westernAspects";

/// What an aspect table is asked: which aspects, under which orbs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct AspectRequest {
    /// The aspects looked for; Leo's nine unless a caller names fewer.
    pub aspects: Vec<WesternAspect>,
    /// How wide each may be.
    pub orbs: OrbModel,
}

impl Default for AspectRequest {
    fn default() -> AspectRequest {
        AspectRequest {
            aspects: WesternAspect::ALL.to_vec(),
            orbs: OrbModel::Leo,
        }
    }
}

impl AspectRequest {
    /// Looks for these aspects only.
    #[must_use]
    pub fn with_aspects(
        mut self,
        aspects: impl IntoIterator<Item = WesternAspect>,
    ) -> AspectRequest {
        self.aspects = aspects.into_iter().collect();
        self
    }

    /// Reads them under this model of orbs.
    #[must_use]
    pub fn with_orbs(mut self, orbs: OrbModel) -> AspectRequest {
        self.orbs = orbs;
        self
    }

    /// Lilly's reading: the Ptolemaic five under his moieties.
    #[must_use]
    pub fn lilly() -> AspectRequest {
        AspectRequest {
            aspects: WesternAspect::PTOLEMAIC.to_vec(),
            orbs: OrbModel::lilly(),
        }
    }

    /// The request a binding sends, as JSON: `aspects`, a list of
    /// [`WesternAspect`] keys (Leo's nine when left out), and `orbs`,
    /// `{"model": "LEO"}` (the default), `{"model": "MOIETIES", "orbs":
    /// [{"graha": "SUN", "orbDeg": 17}, …]}` or `{"model": "BY_ASPECT",
    /// "orbs": [{"aspect": "TRINE", "orbDeg": 6}, …]}`.
    ///
    /// ```
    /// use teistro_western::{AspectRequest, WesternAspect};
    ///
    /// let asked = AspectRequest::from_json(r#"{"aspects": ["TRINE", "SQUARE"]}"#)?;
    /// assert_eq!(asked.aspects, [WesternAspect::Trine, WesternAspect::Square]);
    /// // A refusal names the field the caller wrote, under the record's root.
    /// let wide = AspectRequest::from_json(
    ///     r#"{"orbs": {"model": "BY_ASPECT", "orbs": [{"aspect": "TRINE", "orbDeg": 91}]}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(wide.field(), Some("westernAspects.orbs.orbs"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, and whatever [`AspectRequest::check`] refuses, each named
    /// under `westernAspects`.
    pub fn from_json(text: &str) -> Result<AspectRequest, Error> {
        let asked: AspectRequest = teistro_core::strict::read(text, ROOT)?;
        asked.check().map_err(|why| why.under(ROOT))?;
        Ok(asked)
    }

    /// Refuses a request no table could answer, by field.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for no aspects, an aspect named twice, or an orb the
    /// model states outside 0 to 90°.
    pub fn check(&self) -> Result<(), Error> {
        if self.aspects.is_empty() {
            return Err(Error::invalid_arg("no aspect is asked for")
                .with_field("aspects")
                .with_hint("leave `aspects` out for Leo's nine"));
        }
        refuse_repeats(self.aspects.iter().map(|one| one.key()), "an aspect")
            .map_err(|why| why.with_field("aspects"))?;
        self.orbs.check()
    }
}

/// A body as the table reads it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placed {
    /// Which.
    pub graha: Graha,
    /// Its longitude, degrees, in whichever zodiac every body shares.
    pub longitude_deg: f64,
    /// Its motion in longitude, degrees a day; negative when retrograde.
    pub speed_deg_per_day: f64,
}

impl Placed {
    /// A body at a longitude, moving at a rate.
    #[must_use]
    pub const fn new(graha: Graha, longitude_deg: f64, speed_deg_per_day: f64) -> Placed {
        Placed {
            graha,
            longitude_deg,
            speed_deg_per_day,
        }
    }
}

/// One aspect a pair holds.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WesternAspectRow {
    /// The first body, in the order the bodies were given.
    pub first: Graha,
    /// The second.
    pub second: Graha,
    /// Which aspect.
    pub aspect: WesternAspect,
    /// How far apart the two stand, degrees, 0 to 180.
    pub apart_deg: f64,
    /// How far from exact, degrees; the smaller, the stronger (p. 47).
    pub from_exact_deg: f64,
    /// The orb the model allowed this pair at this aspect, degrees.
    pub orb_deg: f64,
    /// Whether the gap is closing on the aspect rather than leaving it.
    pub applying: bool,
}

/// Every aspect each pair of bodies holds, closest first.
///
/// A pair is read once, in the order the bodies are given, and each
/// aspect under its own orb. Two aspects can share a pair only where a
/// caller's orbs overlap, and then both rows are kept.
///
/// # Errors
///
/// What [`AspectRequest::check`] refuses; a body given twice; a body
/// the model has no orb for; a longitude or speed that is not finite.
pub fn aspects(bodies: &[Placed], request: &AspectRequest) -> Result<Vec<WesternAspectRow>, Error> {
    request.check()?;
    refuse_repeats(bodies.iter().map(|one| one.graha.key()), "a body")
        .map_err(|why| why.with_field("bodies"))?;
    let mut rows = Vec::new();
    for (at, first) in bodies.iter().enumerate() {
        for second in bodies.iter().skip(at + 1) {
            for &aspect in &request.aspects {
                let orb_deg = request.orbs.orb_deg(aspect, first.graha, second.graha)?;
                let found = hits(
                    Moving::new(first.longitude_deg, first.speed_deg_per_day),
                    Moving::new(second.longitude_deg, second.speed_deg_per_day),
                    &[aspect.angle()],
                    orb_deg,
                )?;
                rows.extend(found.into_iter().map(|hit| WesternAspectRow {
                    first: first.graha,
                    second: second.graha,
                    aspect,
                    apart_deg: hit.apart_deg,
                    from_exact_deg: hit.from_exact_deg,
                    orb_deg,
                    applying: hit.applying,
                }));
            }
        }
    }
    rows.sort_by(|a, b| a.from_exact_deg.total_cmp(&b.from_exact_deg));
    Ok(rows)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    #[test]
    fn leos_orbs_widen_for_the_luminaries_only_at_the_conjunction_and_opposition() {
        let leo = OrbModel::Leo;
        let orb = |aspect, a, b| leo.orb_deg(aspect, a, b).unwrap();
        assert!((orb(WesternAspect::Opposition, Graha::Sun, Graha::Moon) - 12.0).abs() < 1e-12);
        assert!((orb(WesternAspect::Conjunction, Graha::Mars, Graha::Moon) - 10.0).abs() < 1e-12);
        assert!((orb(WesternAspect::Conjunction, Graha::Mars, Graha::Venus) - 8.0).abs() < 1e-12);
        assert!((orb(WesternAspect::Square, Graha::Sun, Graha::Moon) - 8.0).abs() < 1e-12);
        assert!((orb(WesternAspect::Quincunx, Graha::Sun, Graha::Uranus) - 2.0).abs() < 1e-12);
    }

    #[test]
    fn the_moieties_are_the_mean_and_name_a_body_they_lack() {
        let lilly = OrbModel::lilly();
        // The Sun 17 and Saturn 10: 13½.
        assert!(
            (lilly
                .orb_deg(WesternAspect::Trine, Graha::Sun, Graha::Saturn)
                .unwrap()
                - 13.5)
                .abs()
                < 1e-12
        );
        let refused = lilly
            .orb_deg(WesternAspect::Trine, Graha::Sun, Graha::Uranus)
            .unwrap_err();
        assert!(refused.to_string().contains("URANUS"), "{refused}");
    }

    #[test]
    fn a_request_no_table_could_answer_is_refused_by_field() {
        let field =
            |request: AspectRequest| request.check().unwrap_err().field().map(str::to_owned);
        assert_eq!(
            field(AspectRequest::default().with_aspects([])).as_deref(),
            Some("aspects")
        );
        assert_eq!(
            field(
                AspectRequest::default().with_aspects([WesternAspect::Trine, WesternAspect::Trine])
            )
            .as_deref(),
            Some("aspects")
        );
        let wide = OrbModel::ByAspect {
            orbs: vec![AspectOrb {
                aspect: WesternAspect::Trine,
                orb_deg: 91.0,
            }],
        };
        assert_eq!(
            field(AspectRequest::default().with_orbs(wide)).as_deref(),
            Some("orbs.orbs")
        );
        let missing = AspectRequest::default().with_orbs(OrbModel::ByAspect {
            orbs: vec![AspectOrb {
                aspect: WesternAspect::Trine,
                orb_deg: 5.0,
            }],
        });
        let bodies = [
            Placed::new(Graha::Sun, 0.0, 1.0),
            Placed::new(Graha::Moon, 1.0, 13.0),
        ];
        assert!(
            aspects(&bodies, &missing)
                .unwrap_err()
                .to_string()
                .contains("CONJUNCTION")
        );
    }

    #[test]
    fn overlapping_orbs_keep_both_rows_and_the_closest_comes_first() {
        // At 40° apart, a semi-square 5° off and a sextile 20° off; with
        // every orb 25°, both hold, the semi-square first.
        let wide = OrbModel::ByAspect {
            orbs: WesternAspect::ALL
                .iter()
                .map(|&aspect| AspectOrb {
                    aspect,
                    orb_deg: 25.0,
                })
                .collect(),
        };
        let bodies = [
            Placed::new(Graha::Venus, 0.0, 1.0),
            Placed::new(Graha::Mars, 40.0, 0.5),
        ];
        let found = aspects(&bodies, &AspectRequest::default().with_orbs(wide)).unwrap();
        let named: Vec<WesternAspect> = found.iter().map(|row| row.aspect).collect();
        assert_eq!(named[0], WesternAspect::SemiSquare);
        assert!(named.contains(&WesternAspect::Sextile));
        assert!(
            found
                .windows(2)
                .all(|pair| pair[0].from_exact_deg <= pair[1].from_exact_deg)
        );
    }

    #[test]
    fn a_body_given_twice_is_refused() {
        let bodies = [
            Placed::new(Graha::Sun, 0.0, 1.0),
            Placed::new(Graha::Sun, 1.0, 1.0),
        ];
        assert_eq!(
            aspects(&bodies, &AspectRequest::default())
                .unwrap_err()
                .field(),
            Some("bodies")
        );
    }

    #[test]
    fn a_request_reads_back_from_its_own_json() {
        let request = AspectRequest::lilly();
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(
            serde_json::from_str::<AspectRequest>(&json).unwrap(),
            request
        );
        let leo: AspectRequest = serde_json::from_str("{}").unwrap();
        assert_eq!(leo, AspectRequest::default());
        let one: AspectRequest =
            serde_json::from_str(r#"{"aspects":["SEMI_SQUARE"],"orbs":{"model":"LEO"}}"#).unwrap();
        assert_eq!(one.aspects, [WesternAspect::SemiSquare]);
    }
}

//! The aspect configurations, the shapes several aspects make together
//! (`03-design/western-configurations.md`).
//!
//! No text before the twentieth century names them. The names are modern
//! and the definitions are geometry, so the rule here is the geometry,
//! stated once per shape, over the aspects a chart **already holds** under
//! the caller's [`AspectRequest`](crate::AspectRequest). A configuration
//! therefore inherits that request's orbs: nothing here widens or narrows
//! one, and a shape holds exactly when every aspect it is made of does.
//!
//! A smaller shape inside a larger one (a T-square inside a grand cross, a
//! grand trine inside a kite) is reported too, and names the larger ones
//! it is inside, so a reader decides whether to show it.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;

use crate::aspects::{AspectRequest, OrbModel, PlanetAt, WesternAspect, WesternAspectRow};

/// Which shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ConfigurationKind {
    /// Three bodies, each trine the other two.
    GrandTrine,
    /// Two bodies in opposition and a third square to both: the
    /// **apex**.
    TSquare,
    /// Four bodies in two oppositions, each square to both ends of the
    /// other.
    GrandCross,
    /// Two bodies in sextile, both quincunx a third: the **apex**.
    Yod,
    /// A grand trine with a fourth body opposite one of its three and
    /// sextile the other two: the fourth is the **apex**.
    Kite,
    /// Four bodies in two oppositions whose ends are joined alternately
    /// by a trine and a sextile.
    MysticRectangle,
    /// At least [`ConfigurationRules::stellium_least`] bodies in one sign.
    Stellium,
}

impl ConfigurationKind {
    /// Every shape, the three-body ones first.
    pub const ALL: [ConfigurationKind; 7] = [
        ConfigurationKind::GrandTrine,
        ConfigurationKind::TSquare,
        ConfigurationKind::Yod,
        ConfigurationKind::GrandCross,
        ConfigurationKind::Kite,
        ConfigurationKind::MysticRectangle,
        ConfigurationKind::Stellium,
    ];

    /// The member's key, as serde writes it: `GRAND_TRINE`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            ConfigurationKind::GrandTrine => "GRAND_TRINE",
            ConfigurationKind::TSquare => "T_SQUARE",
            ConfigurationKind::GrandCross => "GRAND_CROSS",
            ConfigurationKind::Yod => "YOD",
            ConfigurationKind::Kite => "KITE",
            ConfigurationKind::MysticRectangle => "MYSTIC_RECTANGLE",
            ConfigurationKind::Stellium => "STELLIUM",
        }
    }
}

/// The readings the shapes part on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigurationRules {
    /// How many bodies in one sign make a stellium: three by default,
    /// the commonest modern count; some authors ask four.
    pub stellium_least: u8,
}

impl Default for ConfigurationRules {
    fn default() -> ConfigurationRules {
        ConfigurationRules { stellium_least: 3 }
    }
}

impl ConfigurationRules {
    /// Whether the rules can be applied.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `stelliumLeast` under three: two bodies in a
    /// sign are a conjunction's business, not a stellium's.
    pub fn check(&self) -> Result<(), Error> {
        if self.stellium_least < 3 {
            return Err(Error::invalid_arg(format!(
                "a stellium is three bodies or more in a sign, not {}",
                self.stellium_least
            ))
            .with_field("stelliumLeast"));
        }
        Ok(())
    }
}

/// The record's name where a binding sends it, which a refusal is named
/// under.
const ROOT: &str = "westernConfigurations";

/// What a chart's configurations are asked, as a binding sends them: the
/// [`AspectRequest`] the shapes are read over and the
/// [`ConfigurationRules`], side by side in one record, each defaulted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct ConfigurationRequest {
    /// The aspects the shapes are made of; Leo's nine unless a caller
    /// names fewer.
    pub aspects: Vec<WesternAspect>,
    /// How wide each may be.
    pub orbs: OrbModel,
    /// How many bodies in one sign make a stellium.
    pub stellium_least: u8,
}

impl Default for ConfigurationRequest {
    fn default() -> ConfigurationRequest {
        ConfigurationRequest::new(&AspectRequest::default(), ConfigurationRules::default())
    }
}

impl ConfigurationRequest {
    /// The shapes over `aspects`' table, under `rules`.
    #[must_use]
    pub fn new(aspects: &AspectRequest, rules: ConfigurationRules) -> ConfigurationRequest {
        ConfigurationRequest {
            aspects: aspects.aspects.clone(),
            orbs: aspects.orbs.clone(),
            stellium_least: rules.stellium_least,
        }
    }

    /// The aspect table the shapes are read over.
    #[must_use]
    pub fn aspect_request(&self) -> AspectRequest {
        AspectRequest {
            aspects: self.aspects.clone(),
            orbs: self.orbs.clone(),
        }
    }

    /// The readings the shapes part on.
    #[must_use]
    pub const fn rules(&self) -> ConfigurationRules {
        ConfigurationRules {
            stellium_least: self.stellium_least,
        }
    }

    /// The request a binding sends, as JSON: `aspects` and `orbs` as
    /// [`AspectRequest::from_json`] reads them, and `stelliumLeast`, each
    /// defaulted, so `{}` is Leo's nine with a stellium of three.
    ///
    /// ```
    /// use teistro_western::ConfigurationRequest;
    ///
    /// let four = ConfigurationRequest::from_json(r#"{"stelliumLeast": 4}"#)?;
    /// assert_eq!(four.rules().stellium_least, 4);
    /// let two = ConfigurationRequest::from_json(r#"{"stelliumLeast": 2}"#).unwrap_err();
    /// assert_eq!(two.field(), Some("westernConfigurations.stelliumLeast"));
    /// let none = ConfigurationRequest::from_json(r#"{"aspects": []}"#).unwrap_err();
    /// assert_eq!(none.field(), Some("westernConfigurations.aspects"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, and whatever [`ConfigurationRequest::check`] refuses, each
    /// named under `westernConfigurations`.
    pub fn from_json(text: &str) -> Result<ConfigurationRequest, Error> {
        let asked: ConfigurationRequest = teistro_core::strict::read(text, ROOT)?;
        asked.check().map_err(|why| why.under(ROOT))?;
        Ok(asked)
    }

    /// Refuses what [`AspectRequest::check`] and
    /// [`ConfigurationRules::check`] refuse, by the field written here.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `aspects`, `orbs` or `stelliumLeast`.
    pub fn check(&self) -> Result<(), Error> {
        self.aspect_request().check()?;
        self.rules().check()
    }
}

/// One shape a chart holds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    /// Which shape.
    pub kind: ConfigurationKind,
    /// Its bodies, in the order they were given.
    pub bodies: Vec<Graha>,
    /// The body the shape points at: a T-square's, a yod's or a kite's;
    /// `None` for the symmetric ones.
    pub apex: Option<Graha>,
    /// The sign a stellium stands in; `None` for the others.
    pub sign: Option<Rashi>,
    /// How far from exact the loosest of its aspects is, degrees: how
    /// tightly the shape holds. `None` for a stellium, which is read by
    /// sign and has no aspects of its own.
    pub widest_from_exact_deg: Option<f64>,
    /// The positions in the answer of the larger shapes this one is
    /// wholly inside, by bodies; empty when it stands alone.
    pub inside: Vec<usize>,
}

/// Every shape the aspects make, in [`ConfigurationKind::ALL`]'s order and,
/// within a kind, by the bodies' order.
///
/// `bodies` are the bodies the aspects were read among, at their
/// longitudes, which the stellium reads by sign and which fix the order of
/// every answer; `aspects` are those bodies' rows, as
/// [`aspects`](crate::aspects) answered them.
///
/// # Errors
///
/// What [`ConfigurationRules::check`] refuses; a longitude that is not a
/// number, named `bodies`.
pub fn configurations(
    bodies: &[PlanetAt],
    aspects: &[WesternAspectRow],
    rules: ConfigurationRules,
) -> Result<Vec<Configuration>, Error> {
    rules.check()?;
    if let Some(bad) = bodies.iter().find(|one| !one.longitude_deg.is_finite()) {
        return Err(Error::invalid_arg(format!(
            "{} stands at a longitude that is not a number",
            bad.graha.key()
        ))
        .with_field("bodies"));
    }
    let sky = Sky { bodies, aspects };
    let mut found = Vec::new();
    for kind in ConfigurationKind::ALL {
        match kind {
            ConfigurationKind::Stellium => found.extend(stellia(bodies, rules)),
            _ => found.extend(sky.shapes(kind)),
        }
    }
    mark_inside(&mut found);
    Ok(found)
}

/// The aspects among the bodies, asked of by pair.
struct Sky<'a> {
    bodies: &'a [PlanetAt],
    aspects: &'a [WesternAspectRow],
}

impl Sky<'_> {
    /// How far from exact `a` and `b` hold `aspect`, if they hold it.
    fn holds(&self, a: usize, b: usize, aspect: WesternAspect) -> Option<f64> {
        let (a, b) = (self.graha(a)?, self.graha(b)?);
        self.aspects
            .iter()
            .filter(|row| {
                row.aspect == aspect
                    && ((row.first == a && row.second == b) || (row.first == b && row.second == a))
            })
            .map(|row| row.from_exact_deg)
            .reduce(f64::min)
    }

    fn graha(&self, at: usize) -> Option<Graha> {
        self.bodies.get(at).map(|one| one.graha)
    }

    /// Every instance of one shape, each set of bodies once.
    fn shapes(&self, kind: ConfigurationKind) -> Vec<Configuration> {
        let n = self.bodies.len();
        let mut out = Vec::new();
        let size = if matches!(
            kind,
            ConfigurationKind::GrandTrine | ConfigurationKind::TSquare | ConfigurationKind::Yod
        ) {
            3
        } else {
            4
        };
        for set in combinations(n, size) {
            if let Some((apex, widest)) = self.shape(kind, &set) {
                out.push(Configuration {
                    kind,
                    bodies: set.iter().filter_map(|at| self.graha(*at)).collect(),
                    apex: apex.and_then(|at| self.graha(at)),
                    sign: None,
                    widest_from_exact_deg: Some(widest),
                    inside: Vec::new(),
                });
            }
        }
        out
    }

    /// Whether `set` makes `kind` in some arrangement of its members: the
    /// apex where the shape has one, and its loosest aspect.
    fn shape(&self, kind: ConfigurationKind, set: &[usize]) -> Option<(Option<usize>, f64)> {
        use WesternAspect::{Opposition, Quincunx, Sextile, Square, Trine};
        let all = |pairs: &[(usize, usize, WesternAspect)]| {
            pairs
                .iter()
                .map(|(a, b, aspect)| self.holds(*a, *b, *aspect))
                .try_fold(0.0_f64, |widest, gap| gap.map(|gap| widest.max(gap)))
        };
        match (kind, set) {
            (ConfigurationKind::GrandTrine, &[a, b, c]) => {
                all(&[(a, b, Trine), (b, c, Trine), (a, c, Trine)]).map(|w| (None, w))
            }
            // The apex is the one square to the other two, which oppose.
            (ConfigurationKind::TSquare, &[a, b, c]) => {
                rotations3(a, b, c).find_map(|(x, y, apex)| {
                    all(&[(x, y, Opposition), (x, apex, Square), (y, apex, Square)])
                        .map(|w| (Some(apex), w))
                })
            }
            (ConfigurationKind::Yod, &[a, b, c]) => rotations3(a, b, c).find_map(|(x, y, apex)| {
                all(&[(x, y, Sextile), (x, apex, Quincunx), (y, apex, Quincunx)])
                    .map(|w| (Some(apex), w))
            }),
            // Four in a ring w-x-y-z: the ring's sides and its diagonals.
            (ConfigurationKind::GrandCross, &[a, b, c, d]) => {
                rings(a, b, c, d).find_map(|[w, x, y, z]| {
                    all(&[
                        (w, y, Opposition),
                        (x, z, Opposition),
                        (w, x, Square),
                        (x, y, Square),
                        (y, z, Square),
                        (z, w, Square),
                    ])
                    .map(|width| (None, width))
                })
            }
            (ConfigurationKind::MysticRectangle, &[a, b, c, d]) => {
                rings(a, b, c, d).find_map(|[w, x, y, z]| {
                    all(&[
                        (w, y, Opposition),
                        (x, z, Opposition),
                        (w, x, Sextile),
                        (x, y, Trine),
                        (y, z, Sextile),
                        (z, w, Trine),
                    ])
                    .map(|width| (None, width))
                })
            }
            // The apex opposes one corner of the grand trine and is sextile
            // the other two.
            (ConfigurationKind::Kite, &[a, b, c, d]) => [
                (a, [b, c, d]),
                (b, [a, c, d]),
                (c, [a, b, d]),
                (d, [a, b, c]),
            ]
            .into_iter()
            .find_map(|(apex, [x, y, z])| {
                rotations3(x, y, z).find_map(|(p, q, opposed)| {
                    all(&[
                        (x, y, Trine),
                        (y, z, Trine),
                        (x, z, Trine),
                        (apex, opposed, Opposition),
                        (apex, p, Sextile),
                        (apex, q, Sextile),
                    ])
                    .map(|width| (Some(apex), width))
                })
            }),
            _ => None,
        }
    }
}

/// The three ways of picking which of three is the odd one, last.
fn rotations3(a: usize, b: usize, c: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    [(a, b, c), (a, c, b), (b, c, a)].into_iter()
}

/// The three distinct rings four bodies can be read round.
fn rings(a: usize, b: usize, c: usize, d: usize) -> impl Iterator<Item = [usize; 4]> {
    [[a, b, c, d], [a, b, d, c], [a, c, b, d]].into_iter()
}

/// Every increasing choice of `size` positions below `n`.
fn combinations(n: usize, size: usize) -> Vec<Vec<usize>> {
    fn grow(from: usize, n: usize, size: usize, held: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if held.len() == size {
            out.push(held.clone());
            return;
        }
        for at in from..n {
            held.push(at);
            grow(at + 1, n, size, held, out);
            held.pop();
        }
    }
    let mut out = Vec::new();
    grow(0, n, size, &mut Vec::with_capacity(size), &mut out);
    out
}

/// The signs holding at least `stellium_least` bodies, in zodiac order.
fn stellia(bodies: &[PlanetAt], rules: ConfigurationRules) -> Vec<Configuration> {
    let mut out = Vec::new();
    for sign in Rashi::ALL {
        let held: Vec<Graha> = bodies
            .iter()
            .filter(|one| Rashi::of_longitude(one.longitude_deg) == sign)
            .map(|one| one.graha)
            .collect();
        if held.len() >= usize::from(rules.stellium_least) {
            out.push(Configuration {
                kind: ConfigurationKind::Stellium,
                bodies: held,
                apex: None,
                sign: Some(sign),
                widest_from_exact_deg: None,
                inside: Vec::new(),
            });
        }
    }
    out
}

/// Marks each aspect shape with the larger aspect shapes holding all its
/// bodies. A stellium is read by sign, so it neither holds nor is held.
fn mark_inside(found: &mut [Configuration]) {
    let shapes: Vec<(ConfigurationKind, Vec<Graha>)> = found
        .iter()
        .map(|one| (one.kind, one.bodies.clone()))
        .collect();
    for (at, one) in found.iter_mut().enumerate() {
        if one.kind == ConfigurationKind::Stellium {
            continue;
        }
        one.inside = shapes
            .iter()
            .enumerate()
            .filter(|(other, (kind, bodies))| {
                *other != at
                    && *kind != ConfigurationKind::Stellium
                    && bodies.len() > one.bodies.len()
                    && one.bodies.iter().all(|graha| bodies.contains(graha))
            })
            .map(|(other, _)| other)
            .collect();
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::aspects::{AspectRequest, Placed, aspects};

    fn found(at: &[(Graha, f64)], rules: ConfigurationRules) -> Vec<Configuration> {
        let placed: Vec<Placed> = at
            .iter()
            .map(|(g, deg)| Placed::new(*g, *deg, 0.0))
            .collect();
        let bodies: Vec<PlanetAt> = at.iter().map(|(g, deg)| PlanetAt::new(*g, *deg)).collect();
        let rows = aspects(&placed, &AspectRequest::default()).unwrap();
        configurations(&bodies, &rows, rules).unwrap()
    }

    fn kinds(of: &[Configuration]) -> Vec<ConfigurationKind> {
        of.iter().map(|one| one.kind).collect()
    }

    #[test]
    fn a_grand_cross_holds_four_t_squares_and_says_so() {
        let cross = found(
            &[
                (Graha::Sun, 10.0),
                (Graha::Moon, 101.0),
                (Graha::Mars, 188.0),
                (Graha::Saturn, 279.0),
            ],
            ConfigurationRules::default(),
        );
        assert_eq!(
            kinds(&cross),
            [
                ConfigurationKind::TSquare,
                ConfigurationKind::TSquare,
                ConfigurationKind::TSquare,
                ConfigurationKind::TSquare,
                ConfigurationKind::GrandCross,
            ]
        );
        for t in &cross[..4] {
            assert_eq!(t.inside, [4], "each T-square inside the cross");
            assert!(t.apex.is_some());
        }
        assert_eq!(cross[4].inside, Vec::<usize>::new());
        // The Moon and Mars, 87° apart, are the loosest square.
        assert!((cross[4].widest_from_exact_deg.unwrap() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn a_kite_is_a_grand_trine_with_an_apex() {
        let kite = found(
            &[
                (Graha::Sun, 0.0),
                (Graha::Jupiter, 120.0),
                (Graha::Saturn, 240.0),
                (Graha::Venus, 180.0),
            ],
            ConfigurationRules::default(),
        );
        assert_eq!(
            kinds(&kite),
            [ConfigurationKind::GrandTrine, ConfigurationKind::Kite]
        );
        assert_eq!(kite[0].inside, [1]);
        assert_eq!(kite[1].apex, Some(Graha::Venus));
    }

    #[test]
    fn a_yod_points_at_its_apex() {
        let yod = found(
            &[
                (Graha::Mars, 0.0),
                (Graha::Venus, 60.0),
                (Graha::Saturn, 210.0),
            ],
            ConfigurationRules::default(),
        );
        assert_eq!(kinds(&yod), [ConfigurationKind::Yod]);
        assert_eq!(yod[0].apex, Some(Graha::Saturn));
    }

    #[test]
    fn a_mystic_rectangle_alternates_trine_and_sextile() {
        let rectangle = found(
            &[
                (Graha::Sun, 0.0),
                (Graha::Moon, 60.0),
                (Graha::Mars, 180.0),
                (Graha::Venus, 240.0),
            ],
            ConfigurationRules::default(),
        );
        assert_eq!(kinds(&rectangle), [ConfigurationKind::MysticRectangle]);
    }

    #[test]
    fn a_shape_inherits_the_orbs_of_the_aspects_it_is_made_of() {
        // A trine 9° wide is past Leo's 8° orb, so no grand trine.
        let loose = found(
            &[
                (Graha::Mars, 0.0),
                (Graha::Venus, 120.0),
                (Graha::Saturn, 249.0),
            ],
            ConfigurationRules::default(),
        );
        assert_eq!(kinds(&loose), Vec::<ConfigurationKind>::new());
    }

    #[test]
    fn a_stellium_is_counted_by_sign_under_a_knob() {
        let at = [
            (Graha::Sun, 121.0),
            (Graha::Mercury, 129.0),
            (Graha::Venus, 148.0),
            (Graha::Mars, 151.0),
        ];
        let three = found(&at, ConfigurationRules::default());
        assert_eq!(kinds(&three), [ConfigurationKind::Stellium]);
        assert_eq!(three[0].sign, Some(Rashi::Leo));
        assert_eq!(three[0].bodies, [Graha::Sun, Graha::Mercury, Graha::Venus]);
        assert_eq!(
            kinds(&found(&at, ConfigurationRules { stellium_least: 4 })),
            Vec::<ConfigurationKind>::new()
        );
        let refused =
            configurations(&[], &[], ConfigurationRules { stellium_least: 2 }).unwrap_err();
        assert_eq!(refused.field(), Some("stelliumLeast"));
    }
}

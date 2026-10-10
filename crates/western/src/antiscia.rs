//! Antiscia: each planet's reflection about the solstices and the
//! equinoxes, and the pairs standing in one (Lilly, *Christian Astrology*,
//! pp. 90–92; `03-design/western-antiscia.md`, C244).

use serde::{Deserialize, Serialize};
use teistro_core::angle::{difference_deg, normalise_deg};
use teistro_core::catalogue::{Graha, HouseSystem};
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::aspects::{OrbModel, PlanetAt, WesternAspect, refuse_unreadable};
use crate::houses::HouseRequest;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const ROOT: &str = "antiscia";

/// A degree's antiscion: its reflection about the solstices, the degree
/// as far from the first of Cancer or of Capricorn on the other side, at
/// 180° less it (Lilly, pp. 90–91).
///
/// ```
/// use teistro_western::antiscion_deg;
///
/// // Lilly's Saturn in 20°35′ Leo has his antiscion in 9°25′ Taurus.
/// let saturn = 120.0 + 20.0 + 35.0 / 60.0;
/// assert!((antiscion_deg(saturn) - (30.0 + 9.0 + 25.0 / 60.0)).abs() < 1e-9);
/// ```
#[must_use]
pub fn antiscion_deg(longitude_deg: f64) -> f64 {
    normalise_deg(180.0 - longitude_deg)
}

/// A degree's contrantiscion: the degree opposite its antiscion, its
/// reflection about the equinoxes, at 360° less it (Lilly, p. 92).
///
/// ```
/// use teistro_western::contrantiscion_deg;
///
/// // Lilly's Saturn in 20°35′ Leo has his contrantiscion in 9°25′ Scorpio.
/// let saturn = 120.0 + 20.0 + 35.0 / 60.0;
/// assert!((contrantiscion_deg(saturn) - (210.0 + 9.0 + 25.0 / 60.0)).abs() < 1e-9);
/// ```
#[must_use]
pub fn contrantiscion_deg(longitude_deg: f64) -> f64 {
    normalise_deg(-longitude_deg)
}

/// What the antiscia are asked: the orbs a pair is read under, and whether
/// a reflection is read on the cusps.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct AntisciaRequest {
    /// How wide a pair may be, read at the conjunction: Lilly's moieties
    /// unless a caller says otherwise (C244).
    pub orbs: OrbModel,
    /// The cusps a reflection is read on, in the division named, else
    /// Lilly's [`crate::LILLY_HOUSE_SYSTEM`]; `None` reads none
    /// (`western-houses.md`, decision 6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cusps: Option<HouseRequest>,
}

impl Default for AntisciaRequest {
    fn default() -> AntisciaRequest {
        AntisciaRequest {
            orbs: OrbModel::lilly(),
            cusps: None,
        }
    }
}

impl AntisciaRequest {
    /// Reads the pairs under this model of orbs.
    #[must_use]
    pub fn with_orbs(mut self, orbs: OrbModel) -> AntisciaRequest {
        self.orbs = orbs;
        self
    }

    /// Reads each reflection on the cusps too, in `houses`' division, or
    /// Lilly's Regiomontanus when it names none.
    #[must_use]
    pub fn with_cusps(mut self, houses: HouseRequest) -> AntisciaRequest {
        self.cusps = Some(houses);
        self
    }

    /// The request a binding sends, as JSON: `{}` for Lilly's moieties, or
    /// `orbs` as the aspect table reads it (`{"model": "LEO"}`,
    /// `{"model": "MOIETIES", "orbs": […]}`, or `{"model": "BY_ASPECT",
    /// "orbs": [{"aspect": "CONJUNCTION", "orbDeg": 3}]}`).
    ///
    /// ```
    /// use teistro_western::{AntisciaRequest, OrbModel};
    ///
    /// assert_eq!(AntisciaRequest::from_json("{}")?.orbs, OrbModel::lilly());
    /// let wide = AntisciaRequest::from_json(
    ///     r#"{"orbs": {"model": "BY_ASPECT", "orbs": [{"aspect": "CONJUNCTION", "orbDeg": 91}]}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(wide.field(), Some("antiscia.orbs.orbs"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, and whatever [`AntisciaRequest::check`] refuses, each named
    /// under `antiscia`.
    pub fn from_json(text: &str) -> Result<AntisciaRequest, Error> {
        let asked: AntisciaRequest = teistro_core::strict::read(text, ROOT)?;
        asked.check().map_err(|why| why.under(ROOT))?;
        Ok(asked)
    }

    /// Refuses orbs no pair could be read under, and orbs by aspect that
    /// give the conjunction none, since a pair is read at it.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `orbs.orbs`.
    pub fn check(&self) -> Result<(), Error> {
        self.orbs.check()?;
        if let OrbModel::ByAspect { orbs } = &self.orbs
            && !orbs
                .iter()
                .any(|one| one.aspect == WesternAspect::Conjunction)
        {
            return Err(Error::invalid_arg(
                "an antiscion is read at the conjunction's orb, and none is given",
            )
            .with_field("orbs.orbs")
            .with_hint("give the conjunction an orb"));
        }
        Ok(())
    }

    /// The orb this request reads a pair at, or `None` for a planet whose
    /// moieties are not given (decision 5).
    fn orb_deg(&self, first: Graha, second: Graha) -> Result<Option<f64>, Error> {
        if !(self.pairs(first) && self.pairs(second)) {
            return Ok(None);
        }
        self.orbs
            .orb_deg(WesternAspect::Conjunction, first, second)
            .map(Some)
    }

    /// Whether the orbs give this planet any, so it can stand in a pair.
    fn pairs(&self, graha: Graha) -> bool {
        match &self.orbs {
            OrbModel::Moieties { orbs } => orbs.iter().any(|one| one.graha == graha),
            _ => true,
        }
    }
}

/// A planet's two reflections, tropical degrees, as Lilly tabulates them
/// beside a figure (p. 181).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Antiscion {
    /// Which planet.
    pub graha: Graha,
    /// Its antiscion, the reflection about the solstices.
    pub antiscion_deg: f64,
    /// Its contrantiscion, the reflection about the equinoxes.
    pub contrantiscion_deg: f64,
}

/// A planet's reflection upon "the very degree" of a cusp (Lilly, p. 165;
/// C251): in the cusp's own sign and whole degree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CuspAntiscion {
    /// Which planet.
    pub graha: Graha,
    /// The house whose cusp its reflection falls on.
    pub house: House,
    /// Whether it is the contrantiscion; false for the antiscion.
    pub contrary: bool,
}

/// Every planet whose antiscion or contrantiscion falls upon "the very
/// degree" of a cusp (Lilly, p. 165; C251): in the same sign and the same
/// whole degree, the degree Lilly prints a cusp and a reflection to, in
/// the planets' order and then the houses'. The cusps are tropical, as the
/// reflections are.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_western::{PlanetAt, antiscia_on_cusps};
///
/// // The Sun in 10°20′ Taurus reflects onto 19°40′ Leo, the very degree
/// // of a cusp at 19°05′ Leo, the fourth here.
/// let mut cusps = [45.0, 75.0, 105.0, 0.0, 165.0, 195.0, 225.0, 255.0, 285.0, 315.0, 345.0, 15.0];
/// cusps[3] = 139.0 + 5.0 / 60.0;
/// let rows = antiscia_on_cusps(&[PlanetAt::new(Graha::Sun, 40.0 + 20.0 / 60.0)], &cusps)?;
/// assert_eq!(rows.len(), 1);
/// assert_eq!((rows[0].house.get(), rows[0].contrary), (4, false));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A planet given twice or at a longitude that is not finite, named under
/// `bodies`; a cusp that is not finite, named by its index.
pub fn antiscia_on_cusps(
    bodies: &[PlanetAt],
    cusps_deg: &[f64; 12],
) -> Result<Vec<CuspAntiscion>, Error> {
    refuse_unreadable(bodies, "bodies")?;
    if let Some(at) = cusps_deg.iter().position(|cusp| !cusp.is_finite()) {
        return Err(Error::invalid_arg("a cusp is a finite number of degrees")
            .with_field(format!("cuspsDeg[{at}]")));
    }
    let degree = |at: f64| normalise_deg(at).floor();
    let mut rows = Vec::new();
    for body in bodies {
        for (contrary, reflection) in [
            (false, antiscion_deg(body.longitude_deg)),
            (true, contrantiscion_deg(body.longitude_deg)),
        ] {
            for (house, cusp) in House::ALL.into_iter().zip(cusps_deg) {
                if degree(reflection).total_cmp(&degree(*cusp)).is_eq() {
                    rows.push(CuspAntiscion {
                        graha: body.graha,
                        house,
                        contrary,
                    });
                }
            }
        }
    }
    Ok(rows)
}

/// Two planets in antiscion: one's antiscion within the orb of the other,
/// which is the other's within the orb of the first.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AntiscionRow {
    /// The earlier planet of the pair, in the order given.
    pub first: Graha,
    /// The later.
    pub second: Graha,
    /// Whether it is the contrantiscion, the reflection about the
    /// equinoxes; false for the antiscion.
    pub contrary: bool,
    /// How far the one's reflection stands from the other, degrees.
    pub apart_deg: f64,
    /// The orb the request allowed the pair, degrees.
    pub orb_deg: f64,
}

/// A chart's antiscia: every planet's reflections, and the pairs standing
/// in one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Antiscia {
    /// Each planet's two reflections, in the order given.
    pub points: Vec<Antiscion>,
    /// The pairs within the orb, closest first.
    pub pairs: Vec<AntiscionRow>,
    /// The planets the orbs give none, so they stand in no pair: under
    /// Lilly's moieties, the outer three (decision 5).
    pub unpaired: Vec<Graha>,
    /// The reflections upon a cusp's very degree, when the request asks
    /// for the cusps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_cusps: Vec<CuspAntiscion>,
    /// The division the cusps were read in, when the request asks for
    /// them: the one named, or the one a polar policy fell back to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cusp_system: Option<HouseSystem>,
}

/// A chart's antiscia (Lilly, pp. 90–92, C244): each planet's antiscion
/// and contrantiscion, and every pair whose longitudes sum to 180° (the
/// antiscion) or to 0° (the contrantiscion) within the orb the request
/// reads at the conjunction. A pair is read once, in the order the planets
/// are given; pairs equally close keep that order.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_western::{AntisciaRequest, PlanetAt, antiscia};
///
/// // Lilly's p. 181 figure: Saturn's contrantiscion falls "neer" Jupiter.
/// let read = antiscia(
///     &[PlanetAt::new(Graha::Saturn, 255.32), PlanetAt::new(Graha::Jupiter, 107.52)],
///     &AntisciaRequest::default(),
/// )?;
/// assert!(read.pairs[0].contrary);
/// assert!((read.pairs[0].apart_deg - 2.84).abs() < 0.01);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// What [`AntisciaRequest::check`] refuses; a planet given twice; a
/// longitude that is not a finite number.
pub fn antiscia(bodies: &[PlanetAt], request: &AntisciaRequest) -> Result<Antiscia, Error> {
    request.check()?;
    refuse_unreadable(bodies, "bodies")?;
    let pairs = bodies.iter().enumerate().flat_map(|(at, first)| {
        bodies
            .iter()
            .skip(at + 1)
            .map(move |second| (first, second))
    });
    Ok(Antiscia {
        points: bodies
            .iter()
            .map(|one| Antiscion {
                graha: one.graha,
                antiscion_deg: antiscion_deg(one.longitude_deg),
                contrantiscion_deg: contrantiscion_deg(one.longitude_deg),
            })
            .collect(),
        pairs: reflected(pairs, request)?,
        unpaired: bodies
            .iter()
            .map(|one| one.graha)
            .filter(|&graha| !request.pairs(graha))
            .collect(),
        on_cusps: Vec::new(),
        cusp_system: None,
    })
}

/// The **antiscia across two charts**: every planet of `first` whose
/// reflection falls within the orb of a planet of `second`, closest first,
/// the chart's planet `first` in each row and the partner's `second`. A
/// planet the orbs give none stands in no pair, as in one chart.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_western::{AntisciaRequest, PlanetAt, synastry_antiscia};
///
/// // His Sun in 10° Taurus reflects onto her Sun in 20° Leo (Lilly, p. 90).
/// let rows = synastry_antiscia(
///     &[PlanetAt::new(Graha::Sun, 40.0)],
///     &[PlanetAt::new(Graha::Sun, 140.0)],
///     &AntisciaRequest::default(),
/// )?;
/// assert!(!rows[0].contrary && rows[0].apart_deg < 1e-9);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// What [`AntisciaRequest::check`] refuses; a planet given twice on one
/// side; a longitude that is not a finite number.
pub fn synastry_antiscia(
    first: &[PlanetAt],
    second: &[PlanetAt],
    request: &AntisciaRequest,
) -> Result<Vec<AntiscionRow>, Error> {
    request.check()?;
    refuse_unreadable(first, "first")?;
    refuse_unreadable(second, "second")?;
    reflected(
        first
            .iter()
            .flat_map(|a| second.iter().map(move |b| (a, b))),
        request,
    )
}

/// Every pair whose longitudes sum to 180° (the antiscion) or 0° (the
/// contrantiscion) within the orb the request reads at the conjunction,
/// closest first; pairs equally close keep the order given.
fn reflected<'a>(
    pairs: impl Iterator<Item = (&'a PlanetAt, &'a PlanetAt)>,
    request: &AntisciaRequest,
) -> Result<Vec<AntiscionRow>, Error> {
    let mut rows = Vec::new();
    for (first, second) in pairs {
        let Some(orb_deg) = request.orb_deg(first.graha, second.graha)? else {
            continue;
        };
        let sum = first.longitude_deg + second.longitude_deg;
        for (contrary, mirror) in [(false, 180.0), (true, 0.0)] {
            let apart_deg = difference_deg(sum, mirror).abs();
            if apart_deg <= orb_deg {
                rows.push(AntiscionRow {
                    first: first.graha,
                    second: second.graha,
                    contrary,
                    apart_deg,
                    orb_deg,
                });
            }
        }
    }
    rows.sort_by(|a, b| a.apart_deg.total_cmp(&b.apart_deg));
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
    use crate::aspects::AspectOrb;

    /// Degrees from a sign (0 for Aries), degrees and minutes.
    fn at(sign: u8, degrees: u8, minutes: u8) -> f64 {
        f64::from(sign) * 30.0 + f64::from(degrees) + f64::from(minutes) / 60.0
    }

    /// Lilly's p. 181 table, "The Antiscions of the Planets" beside "If
    /// the Querent should be Rich", read off the page image: each planet's
    /// antiscion, and its contrantiscion in the opposite sign.
    const PRINTED: [(Graha, u8, u8, u8); 7] = [
        (Graha::Saturn, 9, 14, 41),
        (Graha::Jupiter, 2, 12, 29),
        (Graha::Mars, 11, 13, 48),
        (Graha::Sun, 1, 26, 50),
        (Graha::Venus, 1, 4, 26),
        (Graha::Mercury, 1, 12, 15),
        (Graha::Moon, 1, 10, 53),
    ];

    /// The figure's planets, recovered from the printed antiscions.
    fn figure() -> Vec<PlanetAt> {
        PRINTED
            .iter()
            .map(|&(graha, sign, degrees, minutes)| {
                PlanetAt::new(graha, antiscion_deg(at(sign, degrees, minutes)))
            })
            .collect()
    }

    #[test]
    fn lillys_table_reads_back_to_the_minute() {
        let read = antiscia(&figure(), &AntisciaRequest::default()).unwrap();
        for (point, &(graha, sign, degrees, minutes)) in read.points.iter().zip(&PRINTED) {
            assert_eq!(point.graha, graha);
            let printed = at(sign, degrees, minutes);
            assert!(
                difference_deg(point.antiscion_deg, printed).abs() < 1e-9,
                "{graha:?}"
            );
            assert!(
                difference_deg(point.contrantiscion_deg, printed + 180.0).abs() < 1e-9,
                "{graha:?}"
            );
        }
        // Jupiter stands in 17°31′ Cancer, "in his exaltation".
        assert!((figure()[1].longitude_deg - at(3, 17, 31)).abs() < 1e-9);
    }

    #[test]
    fn the_figure_holds_the_one_pair_lilly_reads() {
        let read = antiscia(&figure(), &AntisciaRequest::default()).unwrap();
        assert_eq!(read.pairs.len(), 1, "{:?}", read.pairs);
        let pair = read.pairs[0];
        assert_eq!((pair.first, pair.second), (Graha::Saturn, Graha::Jupiter));
        assert!(pair.contrary, "Saturn's contrantiscion");
        assert!((pair.apart_deg - (2.0 + 50.0 / 60.0)).abs() < 1e-9);
        assert!(
            (pair.orb_deg - 11.0).abs() < 1e-12,
            "the moieties of 10° and 12°"
        );
        assert_eq!(read.unpaired, []);
    }

    /// The figure's Regiomontanus cusps, first to twelfth, recast by
    /// pyswisseph (Moshier) at the printed Ascendant, Libra 14°13′, at
    /// London on 16 July 1634 (Old Style).
    const CUSPS: [f64; 12] = [
        194.222, 216.954, 247.568, 288.638, 326.164, 352.742, 14.222, 36.954, 67.568, 108.638,
        146.164, 172.742,
    ];

    #[test]
    fn no_reflection_of_the_figure_falls_upon_a_cusp() {
        // "None of them fell exactly" (p. 181): the nearest is Venus's
        // antiscion, 2.47° short of the eighth cusp.
        assert_eq!(antiscia_on_cusps(&figure(), &CUSPS).unwrap(), []);
    }

    #[test]
    fn the_very_degree_is_the_cusps_sign_and_whole_degree() {
        // A reflection in 19°59′ Leo is on a cusp in 19°00′ Leo and not on
        // one in 20°00′ Leo, a minute away.
        let sun = [PlanetAt::new(Graha::Sun, 180.0 - (139.0 + 59.0 / 60.0))];
        let mut cusps = CUSPS;
        cusps[3] = 139.0;
        let rows = antiscia_on_cusps(&sun, &cusps).unwrap();
        assert_eq!(
            rows,
            [CuspAntiscion {
                graha: Graha::Sun,
                house: House::try_new(4).unwrap(),
                contrary: false,
            }]
        );
        cusps[3] = 140.0;
        assert_eq!(antiscia_on_cusps(&sun, &cusps).unwrap(), []);
        cusps[5] = f64::NAN;
        assert_eq!(
            antiscia_on_cusps(&sun, &cusps).unwrap_err().field(),
            Some("cuspsDeg[5]")
        );
    }

    #[test]
    fn a_planet_the_moieties_leave_out_is_listed() {
        let mut bodies = figure();
        bodies.push(PlanetAt::new(
            Graha::Uranus,
            antiscion_deg(bodies[0].longitude_deg),
        ));
        let read = antiscia(&bodies, &AntisciaRequest::default()).unwrap();
        assert_eq!(read.unpaired, [Graha::Uranus]);
        assert_eq!(read.points.len(), 8);
        assert!(read.pairs.iter().all(|pair| pair.second != Graha::Uranus));
        // Leo's orbs give it one, and it stands exactly on Saturn's antiscion.
        let leo = antiscia(
            &bodies,
            &AntisciaRequest::default().with_orbs(OrbModel::Leo),
        )
        .unwrap();
        assert_eq!(leo.unpaired, []);
        let exact = leo.pairs[0];
        assert_eq!(
            (exact.first, exact.second, exact.contrary),
            (Graha::Saturn, Graha::Uranus, false)
        );
        assert!(exact.apart_deg < 1e-9);
    }

    #[test]
    fn the_same_engine_reads_across_two_charts() {
        let figure = figure();
        let (saturn, rest) = figure.split_first().unwrap();
        // Across, as within: Saturn's contrantiscion on Jupiter.
        let across = synastry_antiscia(&[*saturn], rest, &AntisciaRequest::default()).unwrap();
        let within = antiscia(&figure, &AntisciaRequest::default()).unwrap();
        assert_eq!(across, within.pairs);
        // A planet meets its own kind across: Saturn on the partner's Saturn
        // reflected.
        let mirror = [PlanetAt::new(
            Graha::Saturn,
            antiscion_deg(saturn.longitude_deg),
        )];
        let rows = synastry_antiscia(&[*saturn], &mirror, &AntisciaRequest::default()).unwrap();
        assert_eq!(
            (rows[0].first, rows[0].second),
            (Graha::Saturn, Graha::Saturn)
        );
        assert!(rows[0].apart_deg < 1e-9);
        assert_eq!(
            synastry_antiscia(&[*saturn, *saturn], &mirror, &AntisciaRequest::default())
                .unwrap_err()
                .field(),
            Some("first")
        );
    }

    #[test]
    fn a_refusal_names_its_field() {
        let no_conjunction = AntisciaRequest::default().with_orbs(OrbModel::ByAspect {
            orbs: vec![AspectOrb {
                aspect: WesternAspect::Trine,
                orb_deg: 3.0,
            }],
        });
        assert_eq!(
            antiscia(&figure(), &no_conjunction).unwrap_err().field(),
            Some("orbs.orbs")
        );
        let mut twice = figure();
        twice.push(twice[0]);
        assert_eq!(
            antiscia(&twice, &AntisciaRequest::default())
                .unwrap_err()
                .field(),
            Some("bodies")
        );
        let lost = [PlanetAt::new(Graha::Sun, f64::NAN)];
        assert_eq!(
            antiscia(&lost, &AntisciaRequest::default())
                .unwrap_err()
                .field(),
            Some("bodies[0].longitudeDeg")
        );
    }
}

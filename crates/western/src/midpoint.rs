//! Equal distances: a planet equally distant from two others along the
//! zodiac, on either point of their midpoint axis (Leo, *How to Judge a
//! Nativity*, pp. 47–48; `03-design/western-midpoints.md`, C245, C246).

use serde::{Deserialize, Serialize};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::aspects::{PlanetAt, refuse_repeats};
use crate::declination::{LEO_PARALLEL_ORB_DEG, MAX_PARALLEL_ORB_DEG};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const ROOT: &str = "midpoints";

/// How far a planet may stand from the axis by default, degrees: half the
/// parallel's 1° (C245), where its two distances differ by that 1°.
pub const DEFAULT_MIDPOINT_ORB_DEG: f64 = LEO_PARALLEL_ORB_DEG / 2.0;

/// The widest orb a caller may ask, degrees: the parallel's own limit.
pub const MAX_MIDPOINT_ORB_DEG: f64 = MAX_PARALLEL_ORB_DEG;

/// What the equal distances are asked: how far from the axis a planet may
/// stand.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct MidpointRequest {
    /// The orb from the nearer point of the axis, degrees:
    /// [`DEFAULT_MIDPOINT_ORB_DEG`] unless a caller says otherwise.
    pub orb_deg: f64,
}

impl Default for MidpointRequest {
    fn default() -> MidpointRequest {
        MidpointRequest {
            orb_deg: DEFAULT_MIDPOINT_ORB_DEG,
        }
    }
}

impl MidpointRequest {
    /// Holds an equal distance to this orb from the axis, degrees.
    #[must_use]
    pub const fn with_orb_deg(mut self, orb_deg: f64) -> MidpointRequest {
        self.orb_deg = orb_deg;
        self
    }

    /// The request a binding sends, as JSON: `{"orbDeg": 1}`, or `{}` for
    /// the default 0.5°.
    ///
    /// ```
    /// use teistro_western::MidpointRequest;
    ///
    /// assert_eq!(MidpointRequest::from_json("{}")?.orb_deg, 0.5);
    /// let wide = MidpointRequest::from_json(r#"{"orbDeg": 30}"#).unwrap_err();
    /// assert_eq!(wide.field(), Some("midpoints.orbDeg"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, and whatever [`MidpointRequest::check`] refuses, each named
    /// under `midpoints`.
    pub fn from_json(text: &str) -> Result<MidpointRequest, Error> {
        let asked: MidpointRequest = teistro_core::strict::read(text, ROOT)?;
        asked.check().map_err(|why| why.under(ROOT))?;
        Ok(asked)
    }

    /// Refuses an orb that is not a positive number of degrees up to
    /// [`MAX_MIDPOINT_ORB_DEG`].
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `orbDeg`.
    pub fn check(&self) -> Result<(), Error> {
        if self.orb_deg.is_finite() && self.orb_deg > 0.0 && self.orb_deg <= MAX_MIDPOINT_ORB_DEG {
            return Ok(());
        }
        Err(Error::invalid_arg(format!(
            "an equal distance's orb is more than 0° and at most {MAX_MIDPOINT_ORB_DEG}°, not {}",
            self.orb_deg
        ))
        .with_field("orbDeg")
        .with_hint(format!(
            "{DEFAULT_MIDPOINT_ORB_DEG}° from the axis is Leo's parallel of {LEO_PARALLEL_ORB_DEG}° on the two distances (p. 47)"
        )))
    }
}

/// A planet equally distant from two others, within the orb of the axis
/// through their midpoint.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MidpointRow {
    /// The earlier planet of the pair, in the order given.
    pub first: Graha,
    /// The later.
    pub second: Graha,
    /// The planet equally distant from the two.
    pub middle: Graha,
    /// Whether it stands opposite the midpoint of the pair's shorter arc,
    /// on the longer arc's midpoint (C246); false on the shorter's.
    pub far: bool,
    /// How far it stands from each of the two, the mean of the two arcs,
    /// degrees.
    pub distance_deg: f64,
    /// How far it stands from the nearer point of the axis, degrees: half
    /// what its two distances differ by.
    pub from_axis_deg: f64,
    /// The orb the request allowed, degrees.
    pub orb_deg: f64,
}

/// A chart's **equal distances** (Leo, pp. 47–48, C245, C246): every
/// planet standing within the orb of the axis through the midpoint of two
/// others, on the shorter arc's midpoint or opposite it, closest first.
/// Every pair is read whatever its distance, since Leo leaves the limit
/// uncertain. Rows equally close keep the order the planets are given.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_western::{MidpointRequest, PlanetAt, midpoints};
///
/// // Leo's p. 47: the Sun at 0° Aries, the Moon at 10° Libra, and Mars at
/// // 5° Cancer, 95° from each, opposite the shorter arc's midpoint.
/// let rows = midpoints(
///     &[
///         PlanetAt::new(Graha::Sun, 0.0),
///         PlanetAt::new(Graha::Moon, 190.0),
///         PlanetAt::new(Graha::Mars, 95.0),
///     ],
///     &MidpointRequest::default(),
/// )?;
/// assert_eq!((rows[0].first, rows[0].second, rows[0].middle), (Graha::Sun, Graha::Moon, Graha::Mars));
/// assert!(rows[0].far && rows[0].from_axis_deg < 1e-9);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// What [`MidpointRequest::check`] refuses; a planet given twice; a
/// longitude that is not a finite number.
pub fn midpoints(
    bodies: &[PlanetAt],
    request: &MidpointRequest,
) -> Result<Vec<MidpointRow>, Error> {
    request.check()?;
    refuse_repeats(bodies.iter().map(|one| one.graha.key()), "a body")
        .map_err(|why| why.with_field("bodies"))?;
    if let Some(at) = bodies.iter().position(|one| !one.longitude_deg.is_finite()) {
        return Err(
            Error::invalid_arg("a longitude is a finite number of degrees")
                .with_field(format!("bodies[{at}].longitudeDeg")),
        );
    }
    let mut rows = Vec::new();
    for (at, first) in bodies.iter().enumerate() {
        for second in bodies.iter().skip(at + 1) {
            let near = first.longitude_deg
                + difference_deg(second.longitude_deg, first.longitude_deg) / 2.0;
            for middle in bodies {
                if middle.graha == first.graha || middle.graha == second.graha {
                    continue;
                }
                let from_near = difference_deg(middle.longitude_deg, near).abs();
                let far = from_near > 90.0;
                let from_axis_deg = if far { 180.0 - from_near } else { from_near };
                if from_axis_deg <= request.orb_deg {
                    rows.push(MidpointRow {
                        first: first.graha,
                        second: second.graha,
                        middle: middle.graha,
                        far,
                        distance_deg: f64::midpoint(
                            difference_deg(middle.longitude_deg, first.longitude_deg).abs(),
                            difference_deg(middle.longitude_deg, second.longitude_deg).abs(),
                        ),
                        from_axis_deg,
                        orb_deg: request.orb_deg,
                    });
                }
            }
        }
    }
    rows.sort_by(|a, b| a.from_axis_deg.total_cmp(&b.from_axis_deg));
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

    /// Leo's p. 47 configuration, read off the page image: the Sun at 0°
    /// Aries, the Moon at 10° Libra, Mars at 5° Cancer.
    fn leo() -> [PlanetAt; 3] {
        [
            PlanetAt::new(Graha::Sun, 0.0),
            PlanetAt::new(Graha::Moon, 190.0),
            PlanetAt::new(Graha::Mars, 95.0),
        ]
    }

    #[test]
    fn leos_mars_stands_on_the_far_point() {
        let rows = midpoints(&leo(), &MidpointRequest::default()).unwrap();
        assert_eq!(rows.len(), 1, "{rows:?}");
        let row = rows[0];
        assert_eq!(
            (row.first, row.second, row.middle),
            (Graha::Sun, Graha::Moon, Graha::Mars)
        );
        assert!(
            row.far,
            "5° Cancer is opposite 5° Capricorn, the shorter arc's midpoint"
        );
        assert!(row.from_axis_deg < 1e-9);
        assert!((row.distance_deg - 95.0).abs() < 1e-9);
        assert_eq!(row.orb_deg, 0.5);
    }

    #[test]
    fn the_near_point_counts_too() {
        // Mars at 5° Capricorn, the shorter arc's own midpoint, 85° from each.
        let mut bodies = leo();
        bodies[2].longitude_deg = 275.0;
        let rows = midpoints(&bodies, &MidpointRequest::default()).unwrap();
        assert!(!rows[0].far);
        assert!((rows[0].distance_deg - 85.0).abs() < 1e-9);
    }

    #[test]
    fn the_orb_is_half_the_difference_of_the_two_distances() {
        // 0.4° off the axis: the two distances differ by 0.8°.
        let mut bodies = leo();
        bodies[2].longitude_deg = 95.4;
        let row = midpoints(&bodies, &MidpointRequest::default()).unwrap()[0];
        let to_sun = difference_deg(95.4, 0.0).abs();
        let to_moon = difference_deg(95.4, 190.0).abs();
        assert!(((to_sun - to_moon).abs() - 2.0 * row.from_axis_deg).abs() < 1e-9);
        // 0.6° off falls outside the default, inside a caller's 1°.
        bodies[2].longitude_deg = 95.6;
        assert_eq!(midpoints(&bodies, &MidpointRequest::default()).unwrap(), []);
        let wider = MidpointRequest::default().with_orb_deg(1.0);
        assert_eq!(midpoints(&bodies, &wider).unwrap().len(), 1);
    }

    #[test]
    fn a_close_pair_holds_only_what_stands_on_its_axis() {
        // The Sun and Moon 0.4° apart: a difference of arcs would hold
        // every planet, the axis holds the one conjunct and the one opposite.
        let bodies = [
            PlanetAt::new(Graha::Sun, 10.0),
            PlanetAt::new(Graha::Moon, 10.4),
            PlanetAt::new(Graha::Mars, 10.3),
            PlanetAt::new(Graha::Jupiter, 190.4),
            PlanetAt::new(Graha::Saturn, 100.0),
        ];
        let rows: Vec<(Graha, Graha, Graha, bool)> =
            midpoints(&bodies, &MidpointRequest::default())
                .unwrap()
                .iter()
                .filter(|row| (row.first, row.second) == (Graha::Sun, Graha::Moon))
                .map(|row| (row.first, row.second, row.middle, row.far))
                .collect();
        assert_eq!(
            rows,
            [
                (Graha::Sun, Graha::Moon, Graha::Mars, false),
                (Graha::Sun, Graha::Moon, Graha::Jupiter, true),
            ]
        );
    }

    #[test]
    fn an_opposition_has_its_axis_square_to_it() {
        let bodies = [
            PlanetAt::new(Graha::Sun, 0.0),
            PlanetAt::new(Graha::Moon, 180.0),
            PlanetAt::new(Graha::Mars, 270.2),
        ];
        let rows = midpoints(&bodies, &MidpointRequest::default()).unwrap();
        assert_eq!(rows.len(), 1);
        assert!((rows[0].from_axis_deg - 0.2).abs() < 1e-9);
        assert!((rows[0].distance_deg - 90.0).abs() < 1e-9);
    }

    #[test]
    fn rows_come_closest_first_and_a_rotation_moves_none() {
        let bodies = [
            PlanetAt::new(Graha::Sun, 0.0),
            PlanetAt::new(Graha::Moon, 100.0),
            PlanetAt::new(Graha::Mars, 50.3),
            PlanetAt::new(Graha::Venus, 230.1),
        ];
        let rows = midpoints(&bodies, &MidpointRequest::default()).unwrap();
        assert!(
            rows.windows(2)
                .all(|two| two[0].from_axis_deg <= two[1].from_axis_deg)
        );
        assert_eq!(rows.len(), 2);
        // A sidereal frame turns every longitude alike (decision 5).
        let turned: Vec<PlanetAt> = bodies
            .iter()
            .map(|one| PlanetAt::new(one.graha, (one.longitude_deg - 24.1).rem_euclid(360.0)))
            .collect();
        let again = midpoints(&turned, &MidpointRequest::default()).unwrap();
        assert_eq!(again.len(), rows.len());
        for (a, b) in rows.iter().zip(&again) {
            assert_eq!(
                (a.first, a.second, a.middle, a.far),
                (b.first, b.second, b.middle, b.far)
            );
            assert!((a.from_axis_deg - b.from_axis_deg).abs() < 1e-9);
        }
    }

    #[test]
    fn a_refusal_names_its_field() {
        for orb in [0.0, -1.0, 10.5, f64::NAN] {
            let why = midpoints(&leo(), &MidpointRequest::default().with_orb_deg(orb)).unwrap_err();
            assert_eq!(why.field(), Some("orbDeg"), "{orb}");
        }
        let twice = [
            PlanetAt::new(Graha::Sun, 0.0),
            PlanetAt::new(Graha::Sun, 1.0),
        ];
        assert_eq!(
            midpoints(&twice, &MidpointRequest::default())
                .unwrap_err()
                .field(),
            Some("bodies")
        );
        let lost = [PlanetAt::new(Graha::Sun, f64::INFINITY)];
        assert_eq!(
            midpoints(&lost, &MidpointRequest::default())
                .unwrap_err()
                .field(),
            Some("bodies[0].longitudeDeg")
        );
        assert_eq!(
            MidpointRequest::from_json(r#"{"orb": 1}"#)
                .unwrap_err()
                .field(),
            Some("midpoints.orb")
        );
    }
}

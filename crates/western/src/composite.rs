//! Composite charts: one chart of two, each planet and angle at the near
//! midpoint of the two charts' (Townley; Astrolog; `03-design/
//! western-composites.md`, C247).

use serde::{Deserialize, Serialize};
use teistro_core::angle::{difference_deg, near_midpoint_deg, normalise_deg};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::house::{House, house_of};

use crate::aspects::Placed;

/// One chart as a composite reads it: its planets, and its two angles,
/// all in one zodiac.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ChartPoints {
    /// Its planets, with their speeds.
    pub planets: Vec<Placed>,
    /// Its lagna, degrees.
    pub lagna_deg: f64,
    /// Its midheaven, degrees.
    pub midheaven_deg: f64,
    /// Its twelve house cusps, first to twelfth, degrees, when its
    /// division is defined at its birthplace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cusps_deg: Option<[f64; 12]>,
}

/// A composite chart: every planet and both angles at the near midpoint of
/// two charts'.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Composite {
    /// Each planet at the near midpoint of its two places, moving at the
    /// mean of its two speeds, in the first chart's order.
    pub planets: Vec<Placed>,
    /// The lagna, degrees: the near midpoint of the two, turned when
    /// [`Composite::lagna_turned`] says.
    pub lagna_deg: f64,
    /// The midheaven, degrees: the near midpoint of the two.
    pub midheaven_deg: f64,
    /// Whether the near midpoint of the two lagnas fell more than 90° from
    /// the midheaven's eastern quadrature, and was turned by 180° to stand
    /// after the midheaven, as a lagna does (C247).
    pub lagna_turned: bool,
    /// The twelve cusps, first to twelfth, degrees, when both charts carry
    /// theirs: each the near midpoint of the two charts' same cusp, turned
    /// by 180° when it falls more than 90° from where the midheaven puts
    /// it, the midheaven plus 30° a house from the tenth (Astrolog;
    /// `western-houses.md`, decision 5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cusps_deg: Option<[f64; 12]>,
}

impl Composite {
    /// The house a composite planet stands in by its cusps, or `None`
    /// when the composite has no cusps or places no such planet.
    #[must_use]
    pub fn house_of(&self, graha: Graha) -> Option<House> {
        let cusps = self.cusps_deg.as_ref()?;
        let at = self.planets.iter().find(|one| one.graha == graha)?;
        Some(house_of(at.longitude_deg, cusps, 0.0))
    }
}

/// The **composite** of two charts (Townley; Astrolog, C247): each planet
/// at the near midpoint of its places in the two, at the mean of its two
/// speeds; the midheaven at the near midpoint of the two midheavens; and
/// the lagna at the near midpoint of the two lagnas, turned by 180° when
/// that falls more than 90° from the midheaven plus 90°, so that it stands
/// in the half of the zodiac after the midheaven, as every natal lagna
/// does.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_western::{ChartPoints, Placed, composite};
///
/// let his = ChartPoints {
///     planets: vec![Placed::new(Graha::Sun, 350.0, 1.0)],
///     lagna_deg: 10.0,
///     midheaven_deg: 0.0,
///     cusps_deg: None,
/// };
/// let hers = ChartPoints {
///     planets: vec![Placed::new(Graha::Sun, 20.0, 0.96)],
///     lagna_deg: 340.0,
///     midheaven_deg: 170.0,
///     cusps_deg: None,
/// };
/// let both = composite(&his, &hers)?;
/// assert_eq!(both.planets[0].longitude_deg, 5.0);
/// // The midheavens meet at 85°, the lagnas at 355°, which stands before
/// // the midheaven; turned, the lagna is 175°.
/// assert_eq!((both.midheaven_deg, both.lagna_deg, both.lagna_turned), (85.0, 175.0, true));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// Two charts that place different planets, or one planet twice; a
/// longitude, a speed or an angle that is not a finite number.
pub fn composite(first: &ChartPoints, second: &ChartPoints) -> Result<Composite, Error> {
    for (side, points) in [("first", first), ("second", second)] {
        refuse_unreadable(points).map_err(|why| why.under(side))?;
    }
    let planets = first
        .planets
        .iter()
        .enumerate()
        .map(|(at, one)| {
            let other = second
                .planets
                .iter()
                .find(|other| other.graha == one.graha)
                .ok_or_else(|| different_planets(format!("first.planets[{at}]")))?;
            Ok(Placed::new(
                one.graha,
                near_midpoint_deg(one.longitude_deg, other.longitude_deg),
                f64::midpoint(one.speed_deg_per_day, other.speed_deg_per_day),
            ))
        })
        .collect::<Result<Vec<_>, Error>>()?;
    if second.planets.len() != first.planets.len() {
        return Err(different_planets(String::from("second.planets")));
    }
    let midheaven_deg = near_midpoint_deg(first.midheaven_deg, second.midheaven_deg);
    let lagna = near_midpoint_deg(first.lagna_deg, second.lagna_deg);
    let lagna_turned = turned(lagna, midheaven_deg, 1);
    let cusps_deg = first
        .cusps_deg
        .zip(second.cusps_deg)
        .map(|(mut cusps, theirs)| {
            for ((cusp, other), house) in cusps.iter_mut().zip(theirs).zip(1..) {
                let between = near_midpoint_deg(*cusp, other);
                *cusp = if turned(between, midheaven_deg, house) {
                    normalise_deg(between + 180.0)
                } else {
                    between
                };
            }
            cusps
        });
    Ok(Composite {
        planets,
        lagna_deg: if lagna_turned {
            normalise_deg(lagna + 180.0)
        } else {
            lagna
        },
        midheaven_deg,
        lagna_turned,
        cusps_deg,
    })
}

/// Whether a composite point standing for house `house`'s cusp falls more
/// than 90° from where the midheaven puts that cusp, 30° a house from the
/// tenth, and so is turned by 180° (C247, Astrolog).
fn turned(at: f64, midheaven_deg: f64, house: i32) -> bool {
    difference_deg(at, midheaven_deg + 30.0 * f64::from(house - 10)).abs() > 90.0
}

/// A refusal of two charts that do not place the same planets.
fn different_planets(field: String) -> Error {
    Error::invalid_arg(
        "a composite pairs each planet with itself, so both charts place the same planets",
    )
    .with_field(field)
    .with_hint("found both charts with the same planets: both with the outer three, or neither")
}

/// Refuses a chart naming a planet twice, or a number that is not finite.
fn refuse_unreadable(points: &ChartPoints) -> Result<(), Error> {
    for (at, one) in points.planets.iter().enumerate() {
        if points
            .planets
            .iter()
            .take(at)
            .any(|other| other.graha == one.graha)
        {
            return Err(Error::invalid_arg(format!(
                "a planet is given twice: {}",
                one.graha.key()
            ))
            .with_field(format!("planets[{at}]")));
        }
        if !(one.longitude_deg.is_finite() && one.speed_deg_per_day.is_finite()) {
            return Err(
                Error::invalid_arg("a planet's longitude and speed are finite numbers")
                    .with_field(format!("planets[{at}]")),
            );
        }
    }
    if let Some(at) = points
        .cusps_deg
        .and_then(|cusps| cusps.iter().position(|cusp| !cusp.is_finite()))
    {
        return Err(Error::invalid_arg("a cusp is a finite number of degrees")
            .with_field(format!("cuspsDeg[{at}]")));
    }
    for (field, value) in [
        ("lagnaDeg", points.lagna_deg),
        ("midheavenDeg", points.midheaven_deg),
    ] {
        if !value.is_finite() {
            return Err(
                Error::invalid_arg("an angle is a finite number of degrees").with_field(field)
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use teistro_core::catalogue::Graha;

    use super::*;

    fn chart(planets: &[(Graha, f64, f64)], lagna_deg: f64, midheaven_deg: f64) -> ChartPoints {
        ChartPoints {
            planets: planets
                .iter()
                .map(|&(graha, longitude, speed)| Placed::new(graha, longitude, speed))
                .collect(),
            lagna_deg,
            midheaven_deg,
            cusps_deg: None,
        }
    }

    /// A chart with quadrant-like cusps: the lagna first, the midheaven
    /// tenth, each quadrant cut in three.
    fn housed(planets: &[(Graha, f64, f64)], lagna_deg: f64, midheaven_deg: f64) -> ChartPoints {
        let east = (lagna_deg - midheaven_deg).rem_euclid(360.0) / 3.0;
        let west = 60.0 - east;
        let mut cusps = [0.0; 12];
        for (at, cusp) in cusps.iter_mut().enumerate() {
            // Houses 10, 11, 12 span `east` each, 1, 2, 3 span `west`,
            // and the opposite six mirror them.
            let steps = [
                0.0,
                west,
                2.0 * west,
                3.0 * west,
                3.0 * west + east,
                3.0 * west + 2.0 * east,
            ];
            let from_lagna = steps[at % 6] + if at >= 6 { 180.0 } else { 0.0 };
            *cusp = normalise_deg(lagna_deg + from_lagna);
        }
        ChartPoints {
            cusps_deg: Some(cusps),
            ..chart(planets, lagna_deg, midheaven_deg)
        }
    }

    #[test]
    fn the_composite_cusps_are_the_near_midpoints_and_hold_its_angles() {
        let his = housed(&[(Graha::Sun, 20.0, 1.0)], 100.0, 10.0);
        let hers = housed(&[(Graha::Sun, 60.0, 1.0)], 140.0, 50.0);
        let both = composite(&his, &hers).unwrap();
        let cusps = both.cusps_deg.unwrap();
        for (at, cusp) in cusps.iter().enumerate() {
            let expected =
                near_midpoint_deg(his.cusps_deg.unwrap()[at], hers.cusps_deg.unwrap()[at]);
            assert!((cusp - expected).abs() < 1e-9, "house {}: {cusp}", at + 1);
        }
        // The first and tenth are its lagna and midheaven.
        assert_eq!((cusps[0], cusps[9]), (both.lagna_deg, both.midheaven_deg));
        // The Sun at 40° stands between the tenth cusp (30°) and the
        // eleventh.
        assert_eq!(both.house_of(Graha::Sun).unwrap().get(), 10);
        assert_eq!(both.house_of(Graha::Mars), None);
    }

    #[test]
    fn a_cusp_far_from_its_quadrant_place_is_turned_as_the_lagna_is() {
        // The pair the lagna test turns: every cusp's midpoint lands on the
        // wrong side, and every one is turned with the lagna.
        let his = housed(&[], 10.0, 0.0);
        let hers = housed(&[], 340.0, 170.0);
        let both = composite(&his, &hers).unwrap();
        let cusps = both.cusps_deg.unwrap();
        assert!(both.lagna_turned);
        assert!((cusps[0] - both.lagna_deg).abs() < 1e-9);
        for (at, cusp) in cusps.iter().enumerate() {
            let house = f64::from(u8::try_from(at).unwrap()) + 1.0;
            let place = both.midheaven_deg + 30.0 * (house - 10.0);
            assert!(
                difference_deg(*cusp, place).abs() <= 90.0,
                "house {house}: {cusp}"
            );
        }
        // Without both charts' cusps there are none.
        let bare = composite(&chart(&[], 10.0, 0.0), &hers).unwrap();
        assert_eq!(bare.cusps_deg, None);
        assert_eq!(bare.house_of(Graha::Sun), None);
    }

    #[test]
    fn each_planet_takes_the_near_midpoint_and_the_mean_speed() {
        let his = chart(
            &[(Graha::Sun, 10.0, 1.0), (Graha::Mars, 300.0, -0.2)],
            120.0,
            30.0,
        );
        let hers = chart(
            &[(Graha::Mars, 100.0, 0.6), (Graha::Sun, 50.0, 0.96)],
            140.0,
            50.0,
        );
        let both = composite(&his, &hers).unwrap();
        // In the first chart's order, whatever the second's.
        assert_eq!(both.planets[0], Placed::new(Graha::Sun, 30.0, 0.98));
        // Mars 300° and 100°: the shorter arc runs through 20°.
        assert_eq!(both.planets[1].graha, Graha::Mars);
        assert!((both.planets[1].longitude_deg - 20.0).abs() < 1e-9);
        assert!((both.planets[1].speed_deg_per_day - 0.2).abs() < 1e-12);
        assert_eq!(
            (both.lagna_deg, both.midheaven_deg, both.lagna_turned),
            (130.0, 40.0, false)
        );
    }

    #[test]
    fn a_lagna_before_its_midheaven_is_turned() {
        // His lagna 10° after his midheaven, hers 170° after hers: the
        // midheavens meet at 85° and the lagnas at 355°, 270° after it.
        let his = chart(&[], 10.0, 0.0);
        let hers = chart(&[], 340.0, 170.0);
        let both = composite(&his, &hers).unwrap();
        assert!((both.midheaven_deg - 85.0).abs() < 1e-9);
        assert!(both.lagna_turned);
        assert!((both.lagna_deg - 175.0).abs() < 1e-9);
    }

    #[test]
    fn the_composite_lagna_always_follows_its_midheaven() {
        // Every pair of natal angles a quarter apart, lagnas 10° to 170°
        // after their midheavens, in steps.
        for first_mc in (0..360).step_by(23) {
            for second_mc in (0..360).step_by(29) {
                for (first_after, second_after) in
                    [(10.0, 170.0), (90.0, 90.0), (170.0, 10.0), (45.0, 135.0)]
                {
                    let first_mc = f64::from(first_mc);
                    let second_mc = f64::from(second_mc);
                    let both = composite(
                        &chart(&[], normalise_deg(first_mc + first_after), first_mc),
                        &chart(&[], normalise_deg(second_mc + second_after), second_mc),
                    )
                    .unwrap();
                    let ahead = (both.lagna_deg - both.midheaven_deg).rem_euclid(360.0);
                    assert!(
                        (0.0..=180.0).contains(&ahead),
                        "{first_mc} {second_mc}: {ahead}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_refusal_names_its_field() {
        let his = chart(&[(Graha::Sun, 10.0, 1.0)], 120.0, 30.0);
        let more = chart(
            &[(Graha::Sun, 10.0, 1.0), (Graha::Uranus, 1.0, 0.01)],
            120.0,
            30.0,
        );
        assert_eq!(
            composite(&his, &more).unwrap_err().field(),
            Some("second.planets")
        );
        assert_eq!(
            composite(&more, &his).unwrap_err().field(),
            Some("first.planets[1]")
        );
        let twice = chart(
            &[(Graha::Sun, 10.0, 1.0), (Graha::Sun, 11.0, 1.0)],
            120.0,
            30.0,
        );
        assert_eq!(
            composite(&twice, &his).unwrap_err().field(),
            Some("first.planets[1]")
        );
        let lost = chart(&[(Graha::Sun, 10.0, 1.0)], f64::NAN, 30.0);
        assert_eq!(
            composite(&his, &lost).unwrap_err().field(),
            Some("second.lagnaDeg")
        );
    }
}

//! Astrocartography lines: where on the Earth each body stands on one of
//! the four angles at one instant (`03-design/astrocartography.md`).
//!
//! Geometry only, and the map is the consumer's: each line is a polyline
//! of `[longitude, latitude]` points sampled at a stated step of
//! latitude. A body culminates where the local sidereal time is its right
//! ascension, so its MC line is a meridian, and its IC line the opposite
//! one. It rises and sets where its hour angle is the semi-diurnal arc
//! `H₀`, `cos H₀ = −tan φ tan δ`, which has no solution beyond the
//! latitude `90° − |δ|`: there the body never rises or never sets, and the
//! rising and setting lines meet on the lower or upper meridian. That
//! latitude is the line's polar limit, and the line stops there and says
//! so (C1156).
//!
//! Two projections are practised (C1155). **In mundo**, the default, reads
//! the body where it is, with its ecliptic latitude, against the true
//! horizon (altitude zero, no refraction). **Zodiacal** reads the body's
//! degree of the ecliptic instead, latitude zeroed, so a point on its
//! ASC line has the body's longitude rising, the chart's own ascendant;
//! inside the polar circles a chart's ascendant is no longer the ecliptic
//! degree on the eastern horizon in every house convention, so a zodiacal
//! rising or setting line stops at the polar circle.

use serde::{Deserialize, Serialize};
use teistro_astro::sky::{Spherical, ecliptic_to_equatorial};
use teistro_core::angle::normalise_deg;
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::math;

/// How far a sampled point may stand from its angle, degrees, read back
/// through a chart cast there: the lines are closed-form, so this is the
/// rounding of a few trigonometric calls, held with a wide margin.
///
/// ```
/// assert!(teistro_western::LINE_TOLERANCE_DEG < 1e-5);
/// ```
pub const LINE_TOLERANCE_DEG: f64 = 1e-6;

/// The coarsest step of latitude a line is sampled at, degrees.
pub const MAX_STEP_DEG: f64 = 10.0;

/// How a body is placed on the angles (C1155), written `"IN_MUNDO"` or
/// `"ZODIACAL"`.
///
/// ```
/// use teistro_western::Projection;
///
/// assert_eq!(Projection::default(), Projection::InMundo);
/// assert_eq!(serde_json::to_string(&Projection::Zodiacal).unwrap(), r#""ZODIACAL""#);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Projection {
    /// The body where it stands, its ecliptic latitude kept, against the
    /// true horizon: the practice of astrocartography's maps.
    #[default]
    InMundo,
    /// The body's degree of the ecliptic, latitude zeroed: a point on a
    /// line has the body's longitude on the chart's own angle there.
    Zodiacal,
}

/// One of the four angles a line follows, written `"MC"`, `"IC"`,
/// `"ASC"` or `"DSC"`.
///
/// ```
/// use teistro_western::MapAngle;
///
/// assert_eq!(serde_json::to_string(&MapAngle::Asc).unwrap(), r#""ASC""#);
/// assert_eq!(MapAngle::ALL.len(), 4);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MapAngle {
    /// The upper meridian: the body culminates.
    Mc,
    /// The lower meridian.
    Ic,
    /// The eastern horizon: the body rises.
    Asc,
    /// The western horizon: the body sets.
    Dsc,
}

impl MapAngle {
    /// The four, in the order a body's lines are answered.
    pub const ALL: [MapAngle; 4] = [MapAngle::Mc, MapAngle::Ic, MapAngle::Asc, MapAngle::Dsc];
}

/// A body as the lines read it: its tropical ecliptic longitude and
/// latitude of date, degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MappedBody {
    /// Which body.
    pub graha: Graha,
    /// Tropical ecliptic longitude of date, degrees.
    pub longitude_deg: f64,
    /// Ecliptic latitude, degrees.
    pub latitude_deg: f64,
}

/// The sky the lines are drawn under: the right ascension of the
/// meridian at Greenwich (the apparent sidereal time) and the obliquity
/// of date, degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapSky {
    /// The right ascension of the Greenwich meridian, degrees.
    pub greenwich_ramc_deg: f64,
    /// The true obliquity of the ecliptic, degrees.
    pub obliquity_deg: f64,
}

/// How a request samples its lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sampling {
    /// The projection.
    pub projection: Projection,
    /// The step of latitude between points, degrees, above zero and at
    /// most [`MAX_STEP_DEG`].
    pub step_deg: f64,
    /// The furthest latitude north or south sampled, degrees, above zero
    /// and below ninety.
    pub latitude_limit_deg: f64,
}

/// One body's line on one angle.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AstroLine {
    /// The body.
    pub body: Graha,
    /// The angle it stands on along the line.
    pub angle: MapAngle,
    /// The line, `[longitudeDeg, latitudeDeg]` from south to north,
    /// longitude east positive in `[-180, 180)`; two points more than 180°
    /// apart in longitude are either side of the antimeridian.
    pub points: Vec<[f64; 2]>,
    /// For a rising or setting line, the latitude beyond which, north and
    /// south, the line does not run, degrees: `90° − |δ|`, where the body
    /// stops rising or setting, or the polar circle for a zodiacal line.
    /// A sampled latitude past it has no point. Absent for a meridian.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polar_limit_deg: Option<f64>,
}

/// Where a body stands on the celestial sphere for the lines: its right
/// ascension and declination of date, degrees, under `projection`.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_western::{MappedBody, Projection, equatorial_of};
///
/// let body = MappedBody { graha: Graha::Moon, longitude_deg: 90.0, latitude_deg: 5.0 };
/// // At the solstice degree, the zodiacal point's declination is the obliquity.
/// assert!((equatorial_of(body, Projection::Zodiacal, 23.44).lat_deg - 23.44).abs() < 1e-9);
/// assert!((equatorial_of(body, Projection::InMundo, 23.44).lat_deg - 28.44).abs() < 1e-9);
/// ```
#[must_use]
pub fn equatorial_of(body: MappedBody, projection: Projection, obliquity_deg: f64) -> Spherical {
    let latitude_deg = match projection {
        Projection::InMundo => body.latitude_deg,
        Projection::Zodiacal => 0.0,
    };
    ecliptic_to_equatorial(
        Spherical {
            lon_deg: body.longitude_deg,
            lat_deg: latitude_deg,
        },
        obliquity_deg,
    )
}

/// A longitude east of Greenwich in `[-180, 180)`.
fn east(longitude_deg: f64) -> f64 {
    normalise_deg(longitude_deg + 180.0) - 180.0
}

/// The latitudes a line is sampled at, south to north.
fn latitudes(sampling: Sampling) -> Vec<f64> {
    let limit = sampling.latitude_limit_deg;
    let mut out = Vec::new();
    let mut k = 0.0;
    loop {
        let at = -limit + k * sampling.step_deg;
        if at > limit + 1e-9 {
            break;
        }
        out.push(at.min(limit));
        k += 1.0;
    }
    if out.last().is_some_and(|last| *last < limit - 1e-9) {
        out.push(limit);
    }
    out
}

/// Every line of every body: each body's MC, IC, ASC and DSC lines in
/// that order, the bodies in the order given.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_western::{MapAngle, MapSky, MappedBody, Projection, Sampling, lines};
///
/// let sky = MapSky { greenwich_ramc_deg: 100.0, obliquity_deg: 23.44 };
/// let sun = MappedBody { graha: Graha::Sun, longitude_deg: 90.0, latitude_deg: 0.0 };
/// let sampling = Sampling { projection: Projection::InMundo, step_deg: 5.0, latitude_limit_deg: 80.0 };
/// let drawn = lines(sky, &[sun], sampling)?;
/// assert_eq!(drawn.len(), 4);
/// // The Sun at the solstice culminates where the sidereal time is 90°: 10° west.
/// assert!((drawn[0].points[0][0] + 10.0).abs() < 1e-9);
/// // It never sets north of 66.56°, where its rising line ends.
/// assert_eq!(drawn[2].angle, MapAngle::Asc);
/// assert!((drawn[2].polar_limit_deg.unwrap() - 66.56).abs() < 1e-9);
/// assert!(drawn[2].points.iter().all(|[_, lat]| lat.abs() <= 66.56 + 1e-9));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` naming `stepDeg` outside `(0, 10]` or
/// `latitudeLimitDeg` outside `(0, 90)`.
pub fn lines(
    sky: MapSky,
    bodies: &[MappedBody],
    sampling: Sampling,
) -> Result<Vec<AstroLine>, Error> {
    if !(sampling.step_deg > 0.0 && sampling.step_deg <= MAX_STEP_DEG) {
        return Err(Error::invalid_arg(format!(
            "a line's step is above 0° and at most {MAX_STEP_DEG}°, not {}",
            sampling.step_deg
        ))
        .with_field("stepDeg"));
    }
    if !(sampling.latitude_limit_deg > 0.0 && sampling.latitude_limit_deg < 90.0) {
        return Err(Error::invalid_arg(format!(
            "the latitude a map reaches is above 0° and below 90°, not {}",
            sampling.latitude_limit_deg
        ))
        .with_field("latitudeLimitDeg"));
    }
    let grid = latitudes(sampling);
    let mut out = Vec::with_capacity(bodies.len() * 4);
    for body in bodies {
        let at = equatorial_of(*body, sampling.projection, sky.obliquity_deg);
        let culminates = east(at.lon_deg - sky.greenwich_ramc_deg);
        for angle in [MapAngle::Mc, MapAngle::Ic] {
            let longitude = if angle == MapAngle::Mc {
                culminates
            } else {
                east(culminates + 180.0)
            };
            out.push(AstroLine {
                body: body.graha,
                angle,
                points: grid.iter().map(|lat| [longitude, *lat]).collect(),
                polar_limit_deg: None,
            });
        }
        let own_limit = 90.0 - at.lat_deg.abs();
        let polar_circle = 90.0 - sky.obliquity_deg.abs();
        // The rising and setting lines meet where the body stops rising
        // or setting, and are closed there when the map reaches it; a
        // zodiacal line cut at the polar circle first stays open.
        let (limit, own) = match sampling.projection {
            Projection::Zodiacal if polar_circle < own_limit => (polar_circle, false),
            _ => (own_limit, true),
        };
        let meets = own && limit < sampling.latitude_limit_deg;
        for angle in [MapAngle::Asc, MapAngle::Dsc] {
            let mut points = Vec::with_capacity(grid.len() + 2);
            let mut push = |latitude: f64, at_limit: bool| {
                let (tan_phi, tan_delta) = (
                    math::tan(latitude.to_radians()),
                    math::tan(at.lat_deg.to_radians()),
                );
                let cos_semi_arc = (-tan_phi * tan_delta).clamp(-1.0, 1.0);
                // At its limit the body only grazes the horizon, so the
                // semi-arc is 0° or 180° exactly: the cosine rounds a hair
                // short of ±1 there, and acos turns that hair into
                // microdegrees, parting the two lines that meet.
                let cos_semi_arc = if at_limit {
                    cos_semi_arc.signum()
                } else {
                    cos_semi_arc
                };
                let semi_arc = math::acos(cos_semi_arc).to_degrees();
                let hour_angle = if angle == MapAngle::Asc {
                    -semi_arc
                } else {
                    semi_arc
                };
                points.push([east(culminates + hour_angle), latitude]);
            };
            if meets {
                push(-limit, true);
            }
            for latitude in grid.iter().filter(|lat| lat.abs() < limit) {
                push(*latitude, false);
            }
            if meets {
                push(limit, true);
            }
            out.push(AstroLine {
                body: body.graha,
                angle,
                points,
                polar_limit_deg: Some(limit),
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "a test fails by panicking"
    )]

    use super::*;
    use teistro_astro::sky::altitude_by_midheaven_deg;

    const SKY: MapSky = MapSky {
        greenwich_ramc_deg: 213.7,
        obliquity_deg: 23.437,
    };

    fn sampled(projection: Projection) -> Sampling {
        Sampling {
            projection,
            step_deg: 2.5,
            latitude_limit_deg: 85.0,
        }
    }

    /// The midheaven's ecliptic longitude at a place: the ecliptic point
    /// whose right ascension is the local sidereal time.
    fn midheaven(longitude_deg: f64) -> f64 {
        let ramc = (SKY.greenwich_ramc_deg + longitude_deg).to_radians();
        let eps = SKY.obliquity_deg.to_radians();
        normalise_deg(math::atan2(math::sin(ramc), math::cos(ramc) * math::cos(eps)).to_degrees())
    }

    #[test]
    fn every_point_of_a_rising_or_setting_line_is_on_the_horizon() {
        let moon = MappedBody {
            graha: Graha::Moon,
            longitude_deg: 77.0,
            latitude_deg: -4.9,
        };
        for line in lines(SKY, &[moon], sampled(Projection::InMundo)).unwrap() {
            assert_ne!(line.points.len(), 0, "{:?} {:?}", line.body, line.angle);
            for [lon, lat] in &line.points {
                let altitude = altitude_by_midheaven_deg(
                    Spherical {
                        lon_deg: moon.longitude_deg,
                        lat_deg: moon.latitude_deg,
                    },
                    midheaven(*lon),
                    SKY.obliquity_deg,
                    *lat,
                );
                let expected = match line.angle {
                    MapAngle::Asc | MapAngle::Dsc => 0.0,
                    MapAngle::Mc => {
                        90.0 - (lat
                            - equatorial_of(moon, Projection::InMundo, SKY.obliquity_deg).lat_deg)
                            .abs()
                    }
                    MapAngle::Ic => {
                        -(90.0
                            - (lat
                                + equatorial_of(moon, Projection::InMundo, SKY.obliquity_deg)
                                    .lat_deg)
                                .abs())
                    }
                };
                assert!(
                    (altitude - expected).abs() < LINE_TOLERANCE_DEG,
                    "{:?} at {lon} {lat}: {altitude} against {expected}",
                    line.angle
                );
            }
        }
    }

    #[test]
    fn a_zodiacal_line_stops_at_the_polar_circle_and_a_line_closes_at_its_limit() {
        let venus = MappedBody {
            graha: Graha::Venus,
            longitude_deg: 10.0,
            latitude_deg: 3.0,
        };
        let zodiacal = lines(SKY, &[venus], sampled(Projection::Zodiacal)).unwrap();
        let limit = zodiacal[2].polar_limit_deg.unwrap();
        assert!((limit - (90.0 - SKY.obliquity_deg)).abs() < 1e-12);
        assert!(zodiacal[2].points.iter().all(|[_, lat]| lat.abs() < limit));
        let mundo = lines(SKY, &[venus], sampled(Projection::InMundo)).unwrap();
        let (asc, dsc) = (&mundo[2], &mundo[3]);
        let limit = asc.polar_limit_deg.unwrap();
        assert!(limit < 85.0);
        // The two lines meet at each limit.
        assert_eq!(asc.points.first(), dsc.points.first());
        assert_eq!(asc.points.last(), dsc.points.last());
        assert_eq!(asc.points.last().unwrap()[1], limit);
    }

    #[test]
    fn a_step_or_a_limit_out_of_range_is_refused_by_name() {
        let sun = MappedBody {
            graha: Graha::Sun,
            longitude_deg: 0.0,
            latitude_deg: 0.0,
        };
        for (step, limit, field) in [
            (0.0, 80.0, "stepDeg"),
            (11.0, 80.0, "stepDeg"),
            (f64::NAN, 80.0, "stepDeg"),
            (1.0, 90.0, "latitudeLimitDeg"),
            (1.0, 0.0, "latitudeLimitDeg"),
        ] {
            let sampling = Sampling {
                projection: Projection::InMundo,
                step_deg: step,
                latitude_limit_deg: limit,
            };
            assert_eq!(
                lines(SKY, &[sun], sampling).unwrap_err().field(),
                Some(field)
            );
        }
        // A step that does not divide the range still reaches the limit.
        let grid = latitudes(Sampling {
            projection: Projection::InMundo,
            step_deg: 7.0,
            latitude_limit_deg: 80.0,
        });
        assert_eq!((grid[0], *grid.last().unwrap()), (-80.0, 80.0));
    }
}

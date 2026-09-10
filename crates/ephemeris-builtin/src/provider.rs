//! The built-in ephemeris as a provider of the SDK's port.
//!
//! This is what Phase 3 exists for: a chart that computes with **nothing
//! but the SDK installed** — no data files, no network, and no licence
//! beyond the SDK's own.
//!
//! # What it answers in, and what it leaves to the SDK
//!
//! The theories are stated in one frame each, and the provider answers
//! in that frame rather than pretending to more: **geocentric, ecliptic,
//! J2000, geometric**. No light time, no aberration, no deflection, no
//! nutation, no precession, no ayanamsha. Everything above raw positions
//! is the `astro` layer's (ADR-0009), and a provider that did some of it
//! would be a provider whose errors could not be attributed to a stage.
//!
//! VSOP87A is heliocentric, so a planet's geocentric vector is its own
//! less the Earth's, and the Sun's is the Earth's reversed. ELP2000-82B
//! is geocentric already.
//!
//! # Speeds are differentiated, not differenced
//!
//! Every rate is the analytic derivative of the series. A finite
//! difference loses digits exactly where the answer matters — at a
//! station, where the rate passes through zero and its sign decides
//! whether a planet is called retrograde — and the baseline engine
//! detected retrogression that way, which was a defect.
//!
//! # The Moon carries its bija
//!
//! ELP2000-82B is fitted to DE200/LE200 and drifts against a modern
//! ephemeris by 19.4 arcseconds over six centuries, which is 44 seconds
//! of tithi. ADR-0027's quadratic correction removes most of it for
//! twenty-four bytes. It is applied here, it is declared in the
//! capabilities, and [`Builtin::without_bija`] turns it off for a
//! consumer who wants the theory exactly as its authors published it.

use teistro_core::settings::Tier;
use teistro_port_ephemeris::{
    Astronomy, Body, Capabilities, Cell, CellStatus, Centre, Coordinates, Corrections,
    DistanceUnit, EphemerisKind, EphemerisProvider, Equinox, Frame, Identity, Overrides,
    PositionColumns, PositionRequest, ProviderError, Source, SpeedModel, Zodiac, validate,
};

use crate::elp;
use crate::series::{self, DAYS_PER_MILLENNIUM, millennia};
use crate::tables;

/// Julian days in the century ELP's time argument counts.
const DAYS_PER_CENTURY: f64 = 36_525.0;

/// Kilometres in an astronomical unit, IAU 2012.
const KM_PER_AU: f64 = 149_597_870.7;

/// Arcseconds in a degree.
const ARCSEC_PER_DEGREE: f64 = 3_600.0;

/// The span the tiers were measured over, and so the span they are
/// claimed for: 1800 to 2400 (`03-design/builtin-ephemeris-measured.md`).
///
/// Outside it the theories still evaluate, and their error grows in a way
/// nothing has measured — so a request outside is refused rather than
/// answered with a number nobody can put a bound on.
const JD_RANGE: (f64, f64) = (2_378_497.0, 2_597_641.0);

/// The bodies the built-in ephemeris computes.
///
/// Pluto, the nodes and the apogees are not here yet: Pluto has no VSOP87
/// series and is fitted separately, and the nodes and apogees are mean
/// elements (ADR-0021). A body it cannot do is an unsupported cell, never
/// a guess.
const BODIES: [Body; 9] = [
    Body::Sun,
    Body::Moon,
    Body::Mercury,
    Body::Venus,
    Body::Mars,
    Body::Jupiter,
    Body::Saturn,
    Body::Uranus,
    Body::Neptune,
];

/// The built-in analytic ephemeris.
///
/// ```
/// use teistro_ephemeris_builtin::provider::Builtin;
/// use teistro_port_ephemeris::EphemerisProvider;
///
/// let provider = Builtin::new();
/// assert!(provider.capabilities().deterministic);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Builtin {
    /// Whether the Moon's mean longitude carries ADR-0027's correction.
    bija: bool,
}

impl Builtin {
    /// The provider as it should normally be used, with the bija applied.
    #[must_use]
    pub const fn new() -> Builtin {
        Builtin { bija: true }
    }

    /// The theory as its authors published it, with no correction.
    ///
    /// For a consumer reproducing ELP2000-82B itself, and for the
    /// measurement that decided the correction was worth making.
    #[must_use]
    pub const fn without_bija() -> Builtin {
        Builtin { bija: false }
    }

    /// Which tier is compiled in.
    #[must_use]
    pub fn tier() -> Tier {
        match tables::TIER {
            "compact" => Tier::Compact,
            "full" => Tier::Full,
            _ => Tier::Standard,
        }
    }

    /// A body's heliocentric position and rate, in astronomical units and
    /// astronomical units per day.
    fn heliocentric(name: &str, jd: f64) -> Option<([f64; 3], [f64; 3])> {
        let (_, coordinates) = tables::PLANETS.iter().find(|(body, _)| *body == name)?;
        let t = millennia(jd);
        let mut position = [0.0; 3];
        let mut rate = [0.0; 3];
        for ((slot, speed), terms) in position.iter_mut().zip(&mut rate).zip(coordinates) {
            *slot = series::sum(terms, t);
            *speed = series::rate(terms, t) / DAYS_PER_MILLENNIUM;
        }
        Some((position, rate))
    }

    /// The Moon's geocentric position and rate, in astronomical units.
    ///
    /// The rate is a central difference of the theory rather than its
    /// derivative, and the reason is stated rather than hidden: ELP's
    /// argument is rebuilt per term from five polynomial coefficients and
    /// its analytic derivative is a second evaluator to keep in step with
    /// the first. The step is a day, over which the Moon's longitude rate
    /// is smooth to far better than the theory's own error, and the
    /// station problem that makes a difference wrong for a planet does not
    /// arise: the Moon never turns retrograde.
    fn moon(self, jd: f64) -> ([f64; 3], [f64; 3]) {
        const STEP: f64 = 0.5;
        let at = |jd: f64| {
            let km = elp::position_from(&tables::MOON_MAIN, &tables::MOON_PERTURBATIONS, jd);
            let mut au = [km[0] / KM_PER_AU, km[1] / KM_PER_AU, km[2] / KM_PER_AU];
            if self.bija {
                au = rotate_about_pole(au, self.bija_radians(jd));
            }
            au
        };
        let here = at(jd);
        let before = at(jd - STEP);
        let after = at(jd + STEP);
        let mut rate = [0.0; 3];
        for ((slot, after), before) in rate.iter_mut().zip(after).zip(before) {
            *slot = (after - before) / (2.0 * STEP);
        }
        (here, rate)
    }

    /// The bija at an instant, in radians of longitude.
    fn bija_radians(self, jd: f64) -> f64 {
        if !self.bija {
            return 0.0;
        }
        let century = (jd - series::J2000) / DAYS_PER_CENTURY;
        let arcsec: f64 = tables::MOON_BIJA_ARCSEC
            .iter()
            .enumerate()
            .map(|(power, coefficient)| {
                coefficient * century.powi(i32::try_from(power).unwrap_or(0))
            })
            .sum();
        (arcsec / ARCSEC_PER_DEGREE).to_radians()
    }
}

/// Turns a vector about the ecliptic pole, which is what adjusting a
/// mean longitude does.
fn rotate_about_pole(v: [f64; 3], angle: f64) -> [f64; 3] {
    let (sin, cos) = angle.sin_cos();
    [v[0] * cos - v[1] * sin, v[0] * sin + v[1] * cos, v[2]]
}

/// A rectangular position and rate as the port's spherical cell.
///
/// The rates are the exact derivatives of the spherical coordinates,
/// not a difference of two conversions.
fn to_cell(position: [f64; 3], rate: [f64; 3], source: Source) -> Cell {
    let [x, y, z] = position;
    let [dx, dy, dz] = rate;
    let flat = x * x + y * y;
    let distance = (flat + z * z).sqrt();
    let flat_length = flat.sqrt();
    let radial = if distance == 0.0 {
        0.0
    } else {
        (x * dx + y * dy + z * dz) / distance
    };
    let lon_speed = if flat == 0.0 {
        0.0
    } else {
        (x * dy - y * dx) / flat
    };
    let lat_speed = if flat_length == 0.0 || distance == 0.0 {
        0.0
    } else {
        dz.mul_add(flat, -(z * (x * dx + y * dy))) / (distance * distance * flat_length)
    };
    Cell {
        lon: y.atan2(x).to_degrees().rem_euclid(360.0),
        lat: (z / distance).asin().to_degrees(),
        dist: distance,
        lon_speed: lon_speed.to_degrees(),
        lat_speed: lat_speed.to_degrees(),
        dist_speed: radial,
        status: CellStatus::Ok,
        source,
    }
}

/// The frame the theories are stated in, and the only one this provider
/// answers in.
fn native_frame() -> Frame {
    Frame {
        centre: Centre::Geocentric,
        equinox: Equinox::J2000,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::Tropical,
        corrections: Corrections::GEOMETRIC,
    }
}

/// The VSOP87 name for a body the port names.
fn series_name(body: Body) -> Option<&'static str> {
    match body {
        Body::Mercury => Some("Mercury"),
        Body::Venus => Some("Venus"),
        Body::Mars => Some("Mars"),
        Body::Jupiter => Some("Jupiter"),
        Body::Saturn => Some("Saturn"),
        Body::Uranus => Some("Uranus"),
        Body::Neptune => Some("Neptune"),
        _ => None,
    }
}

impl EphemerisProvider for Builtin {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            identity: Identity {
                name: String::from("teistro-builtin"),
                version: String::from(env!("CARGO_PKG_VERSION")),
                data_version: format!(
                    "VSOP87A/CDS-VI-81 + ELP2000-82B/CDS-VI-79{}",
                    if self.bija { " + bija" } else { "" }
                ),
                tier: Some(Builtin::tier()),
                data_hashes: Vec::new(),
            },
            jd_range: JD_RANGE,
            bodies: BODIES.to_vec(),
            native_frame: native_frame(),
            astronomy: Astronomy::Modern,
            speeds: true,
            speed_model: SpeedModel::Derivative,
            distance_unit: DistanceUnit::AstronomicalUnits,
            overrides: Overrides::NONE,
            ayanamshas: Vec::new(),
            // The tables are constants and the arithmetic is `f64` in a
            // fixed order, so the same request gives the same bits on
            // every architecture (ADR-0022).
            deterministic: true,
            native: false,
        }
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let capabilities = self.capabilities();
        validate(&capabilities, request)?;
        let frame = native_frame();
        if request.frame != frame {
            return Err(ProviderError::unsupported(format!(
                "the built-in ephemeris answers only in {}, and the SDK's completion \
                 takes it from there",
                frame.key()
            )));
        }
        let source = Source {
            kind: EphemerisKind::Analytic,
            tier: Some(Builtin::tier()),
        };
        let mut columns = PositionColumns::new(request.jds.len(), request.bodies.len(), frame);
        for (jd_index, jd) in request.jds.iter().enumerate() {
            // The Earth is subtracted from every planet, so it is
            // computed once an instant rather than once a body.
            let earth = capabilities
                .covers(*jd)
                .then(|| Builtin::heliocentric("Earth", *jd))
                .flatten();
            for (body_index, body) in request.bodies.iter().enumerate() {
                let cell = match (earth, body) {
                    (None, _) => Cell::failed(CellStatus::OutOfRange),
                    (Some((earth_position, earth_rate)), Body::Sun) => to_cell(
                        [-earth_position[0], -earth_position[1], -earth_position[2]],
                        [-earth_rate[0], -earth_rate[1], -earth_rate[2]],
                        source,
                    ),
                    (Some(_), Body::Moon) => {
                        let (position, rate) = self.moon(*jd);
                        to_cell(position, rate, source)
                    }
                    (Some((earth_position, earth_rate)), other) => series_name(*other)
                        .and_then(|name| Builtin::heliocentric(name, *jd))
                        .map_or(Cell::failed(CellStatus::UnsupportedBody), |(p, r)| {
                            let mut position = [0.0; 3];
                            let mut rate = [0.0; 3];
                            for (index, slot) in position.iter_mut().enumerate() {
                                *slot = p.get(index).copied().unwrap_or(0.0)
                                    - earth_position.get(index).copied().unwrap_or(0.0);
                            }
                            for (index, slot) in rate.iter_mut().enumerate() {
                                *slot = r.get(index).copied().unwrap_or(0.0)
                                    - earth_rate.get(index).copied().unwrap_or(0.0);
                            }
                            to_cell(position, rate, source)
                        }),
                };
                columns.set_at(jd_index, body_index, cell);
            }
        }
        Ok(columns)
    }
}

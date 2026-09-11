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
//! # The time scale is honoured, not assumed
//!
//! The theories are stated in dynamical time, and a request may be in
//! UT1 — the scale of civil time, which is where a birth instant comes
//! from. The completion passes the scale through rather than converting
//! it, so a provider that ignored it would be seventy seconds wrong
//! today and minutes wrong at the edges of the span. The Moon moves
//! 0.01 degrees in seventy seconds, which is twenty seconds of tithi.
//!
//! The conversion uses the SDK's own Delta T rather than a second copy
//! of it, because two tables of the same quantity are two tables that
//! can disagree.
//!
//! # The Moon carries its bija
//!
//! ELP2000-82B is fitted to DE200/LE200 and drifts against a modern
//! ephemeris by 19.4 arcseconds over six centuries, which is 44 seconds
//! of tithi. ADR-0027's quadratic correction removes most of it for
//! twenty-four bytes. It is applied here, it is declared in the
//! capabilities, and [`Builtin::without_bija`] turns it off for a
//! consumer who wants the theory exactly as its authors published it.

use teistro_astro::DeltaTModel;
use teistro_astro::scale::tt_of;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_core::settings::Tier;
use teistro_port_ephemeris::{
    Astronomy, Body, Capabilities, Cell, CellStatus, Centre, Coordinates, Corrections,
    DistanceUnit, EphemerisKind, EphemerisProvider, Equinox, Frame, Identity, Overrides,
    PositionColumns, PositionRequest, ProviderError, Source, SpeedModel, TimeScale, Zodiac,
    validate,
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
/// Rahu is here under both its names — the mean node and the true one —
/// because a Vedic chart has nine grahas and two of them are the nodes.
/// Ketu is not a body: it is Rahu's opposite point, which the chart layer
/// derives.
///
/// Pluto and the osculating apogee are not here. Pluto has no VSOP87
/// series and is fitted from a kernel separately (ADR-0021); the
/// osculating apogee needs the Earth-Moon mass parameter, which is a
/// constant neither theory carries. A body without a theory is refused by
/// name rather than guessed at.
const BODIES: [Body; 12] = [
    Body::Sun,
    Body::Moon,
    Body::Mercury,
    Body::Venus,
    Body::Mars,
    Body::Jupiter,
    Body::Saturn,
    Body::Uranus,
    Body::Neptune,
    Body::MeanNode,
    Body::TrueNode,
    Body::MeanApogee,
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
    /// Which Delta T model turns a UT1 request into the dynamical time
    /// the theories are stated in.
    delta_t: DeltaTModel,
}

impl Builtin {
    /// The provider as it should normally be used, with the bija applied.
    #[must_use]
    pub const fn new() -> Builtin {
        Builtin {
            bija: true,
            delta_t: DeltaTModel::TableThenModel,
        }
    }

    /// The same provider under another Delta T model, for a consumer who
    /// pins one (ADR-0020 counts it in the calculation version).
    #[must_use]
    pub const fn with_delta_t(mut self, model: DeltaTModel) -> Builtin {
        self.delta_t = model;
        self
    }

    /// The instant the theories want, from the instant the request gives.
    ///
    /// A UT1 request is civil time and the series are dynamical, so the
    /// difference is Delta T — about seventy seconds now and minutes at
    /// the ends of the span. Ignoring it would move the Moon by twenty
    /// seconds of tithi.
    fn dynamical(self, jd: f64, scale: TimeScale) -> Result<f64, ProviderError> {
        match scale {
            TimeScale::Tt => Ok(jd),
            TimeScale::Ut1 => {
                let instant = JulianDay::<Ut1>::try_new(jd)
                    .map_err(|error| ProviderError::invalid(error.to_string()))?;
                let (tt, _) = tt_of(instant, self.delta_t)
                    .map_err(|error| ProviderError::invalid(error.to_string()))?;
                Ok(tt.get())
            }
        }
    }

    /// The theory as its authors published it, with no correction.
    ///
    /// For a consumer reproducing ELP2000-82B itself, and for the
    /// measurement that decided the correction was worth making.
    #[must_use]
    pub const fn without_bija() -> Builtin {
        Builtin {
            bija: false,
            delta_t: DeltaTModel::TableThenModel,
        }
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

    /// The Moon's geocentric position and rate, in astronomical units
    /// and astronomical units per day.
    ///
    /// Analytic, like the planets'. It began as a half-day central
    /// difference with a stated reason, and the provider kit measured
    /// what that reason cost: 0.036 degrees a day against a bound of
    /// 0.002, because a difference over any step long enough to be well
    /// conditioned returns an average rate and the Moon's own rate swings
    /// by a third across a month. The reason was true and the trade was
    /// not, which is the kind of thing a kit is for.
    fn moon(self, jd: f64) -> ([f64; 3], [f64; 3]) {
        let (km, km_per_day) =
            elp::position_and_rate_from(&tables::MOON_MAIN, &tables::MOON_PERTURBATIONS, jd);
        let mut position = [km[0] / KM_PER_AU, km[1] / KM_PER_AU, km[2] / KM_PER_AU];
        let mut rate = [
            km_per_day[0] / KM_PER_AU,
            km_per_day[1] / KM_PER_AU,
            km_per_day[2] / KM_PER_AU,
        ];
        if self.bija {
            // A rotation about the pole, applied to both. The bija's own
            // rate is arcseconds a century — a hundred-millionth of the
            // Moon's — so the rotation is taken as constant across the
            // instant, which costs the rate far less than the bound.
            let angle = self.bija_radians(jd);
            position = rotate_about_pole(position, angle);
            rate = rotate_about_pole(rate, angle);
        }
        (position, rate)
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

/// A direction on the ecliptic at a longitude, as a unit vector.
fn on_the_ecliptic(longitude: f64) -> [f64; 3] {
    let (sin, cos) = longitude.sin_cos();
    [cos, sin, 0.0]
}

/// A node or an apogee: a **direction**, not a place.
///
/// The port is explicit that nothing sits at a node — it is where the
/// Moon's orbit crosses the ecliptic — so the latitude is zero by
/// construction and the distance is zero rather than a nominal figure
/// dressed up as a measurement. No observer sees such a point displaced,
/// which is why the conformance corpus records them identically under
/// every centre.
fn direction_cell(longitude: f64, rate: f64, source: Source) -> Cell {
    Cell {
        lon: longitude.to_degrees().rem_euclid(360.0),
        lat: 0.0,
        dist: 0.0,
        lon_speed: rate.to_degrees(),
        lat_speed: 0.0,
        dist_speed: 0.0,
        status: CellStatus::Ok,
        source,
    }
}

/// The vector product, which both halves of a node need.
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// The ascending node of the Moon's osculating orbit, as a direction in
/// the frame `pole` is given in.
///
/// The orbit's plane is fixed by `r x v`, and the line of nodes is where
/// that plane meets **the ecliptic that `pole` is the pole of** — `pole x
/// h`, pointing at the ascending crossing. It needs no mass parameter and
/// no element solution: the position and the velocity are the orbit's
/// orientation.
///
/// *Which* ecliptic is not a detail. The node is an intersection of two
/// planes, so tilting the reference plane slides it along the orbit, and
/// the ecliptic of date turns about 47 arcseconds a century away from
/// J2000's. Divided by the sine of the orbit's five-degree inclination
/// that is some 520 arcseconds a century of node, and cutting against the
/// wrong plane put this 2044 arcseconds out by 2400 — against a mean node
/// from the same crate that was right to 0.985, because the mean node
/// already took its longitude of date and carried the direction back.
fn true_node_longitude(position: [f64; 3], velocity: [f64; 3], pole: [f64; 3]) -> f64 {
    let node = cross(pole, cross(position, velocity));
    node[1].atan2(node[0])
}

/// The pole of the mean ecliptic of date, in the J2000 coordinates the
/// provider answers in, by the same rotation the Moon's own position
/// takes.
fn ecliptic_pole_of_date(jd: f64) -> [f64; 3] {
    elp::to_j2000([0.0, 0.0, 1.0], jd)
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
            // The scale is converted once an instant, not once a body.
            let dynamical = self.dynamical(*jd, request.scale)?;
            // The Earth is subtracted from every planet, so it too is
            // computed once an instant rather than once a body.
            let earth = capabilities
                .covers(*jd)
                .then(|| Builtin::heliocentric("Earth", dynamical))
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
                        let (position, rate) = self.moon(dynamical);
                        to_cell(position, rate, source)
                    }
                    (Some(_), Body::MeanNode | Body::MeanApogee) => {
                        let arguments = elp::Arguments::at(dynamical);
                        let coefficients = if *body == Body::MeanNode {
                            arguments.w3
                        } else {
                            arguments.w2
                        };
                        let (mut longitude, rate) = arguments.mean_longitude(coefficients);
                        // The apogee is half a turn from the perigee,
                        // which is what the theory carries.
                        if *body == Body::MeanApogee {
                            longitude += std::f64::consts::PI;
                        }
                        // The mean longitudes are of date; the direction
                        // reaches J2000 by the path the Moon's own
                        // position takes.
                        let rotated = elp::to_j2000(on_the_ecliptic(longitude), dynamical);
                        direction_cell(rotated[1].atan2(rotated[0]), rate, source)
                    }
                    (Some(_), Body::TrueNode) => {
                        // The node's own motion is a wobble of about half
                        // a month, so a difference over a day is well
                        // conditioned and there is no station to fall in.
                        const STEP: f64 = 0.5;
                        let (position, velocity) = self.moon(dynamical);
                        let at = |jd: f64| {
                            let (p, v) = self.moon(jd);
                            true_node_longitude(p, v, ecliptic_pole_of_date(jd))
                        };
                        let before = at(dynamical - STEP);
                        let after = at(dynamical + STEP);
                        let mut moved = after - before;
                        while moved > std::f64::consts::PI {
                            moved -= std::f64::consts::TAU;
                        }
                        while moved < -std::f64::consts::PI {
                            moved += std::f64::consts::TAU;
                        }
                        direction_cell(
                            true_node_longitude(
                                position,
                                velocity,
                                ecliptic_pole_of_date(dynamical),
                            ),
                            moved / (2.0 * STEP),
                            source,
                        )
                    }
                    (Some((earth_position, earth_rate)), other) => series_name(*other)
                        .and_then(|name| Builtin::heliocentric(name, dynamical))
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

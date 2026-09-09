//! The falsification pass over the topocentric centre, which the
//! completion's `centre` step is designed from.
//!
//! The corpus records six charts **twice**: once from the centre of the
//! Earth and once from the place they were cast for, with every other
//! setting equal. So this pass is the `points` and `vargas` shape rather
//! than the `almanac` one — the corpus holds the input and the answer
//! beside it, and a proposed reading of the step either reproduces the
//! recorded numbers or it does not.
//!
//! That matters here more than usual, because the step has a designed
//! description already: `astro-timescales-and-frames.md` §4 calls it "the
//! observer's geocentric position (WGS84) and the parallax", and a
//! displacement is what a topocentric position obviously is. This pass
//! measures that reading and three others against the recorded pairs, and
//! the obvious one is the one it falsifies.
//!
//! `cargo xtask topocentric` writes the page; `check-topocentric`
//! regenerates it in memory and fails on any difference, so the numbers
//! on the page are the numbers this build produces.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::iau::earth::{self, Ellipsoid};
use teistro_astro::iau::vector::{self, Vector3};
use teistro_astro::iau::{self as iau, DAU, DAYSEC, DEG2RAD, RAD2DEG};
use teistro_astro::scale::tt_of;
use teistro_astro::sky::{self, Spherical};
use teistro_core::quantity::{JulianDay, Place, Tt, Ut1};

use crate::generated::{Output, check, write};
use crate::measure::{
    Claim, Verdict, count, fill, spaced, spelled, table, times, verdict_of, worst,
};

const PAGE: &str = "docs/03-design/topocentric-measured.md";
const CHARTS: &str = "fixtures/baseline/charts";
const VARIANTS: &str = "fixtures/baseline/variants";

/// The suffix a variant recorded from the Earth's centre carries.
const GEOCENTRIC: &str = "--geocentric";

/// The points that are a direction on the Moon's orbit rather than a
/// body in space; the corpus records them under these keys.
const DIRECTIONS: [&str; 2] = ["RAHU", "KETU"];

/// An arcsecond in degrees, for reporting.
const ARCSECOND: f64 = 1.0 / 3600.0;

/// One body as a fixture records it, in the tropical zodiac of date.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Recorded {
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
    lon_speed_deg: f64,
    sign: i64,
    nakshatra: i64,
    pada: i64,
}

/// A chart the corpus records from both centres, and nothing else about
/// the two differs.
struct Pair {
    fixture: String,
    place: Place,
    ut1: JulianDay<Ut1>,
    geocentric: BTreeMap<String, Recorded>,
    topocentric: BTreeMap<String, Recorded>,
    /// The Vimshottari lord the Moon's nakshatra starts the dasha with,
    /// under each centre.
    lords: (String, String),
}

/// One reading of the step, as the choices that make it up. The reading
/// the design page describes is `aberration: false`; the other two
/// switches are the shape of the Earth the observer stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "one switch per choice, so a reading is named by the choices it makes"
)]
struct Reading {
    /// Aberrate the direction by the station's own velocity, as well as
    /// displacing the position by where the station stands.
    aberration: bool,
    /// Stand the observer at their height above the ellipsoid rather
    /// than on it.
    height: bool,
    /// Stand them on the WGS84 ellipsoid rather than on a sphere of its
    /// equatorial radius.
    flattening: bool,
    /// Take the annual aberration off the recorded direction before
    /// displacing it, and put it back after. A displacement applied to a
    /// direction the Earth's own motion has already turned by twenty
    /// arcseconds is a displacement in the wrong direction by that much
    /// of itself.
    geometric: bool,
    /// Put the station where it was when the light left the body rather
    /// than where it is when the light arrives: the light time turns the
    /// Earth under it, by six hundred metres for the Moon.
    retarded: bool,
    /// Carry the body forward by its own motion over the light time the
    /// displacement changes: light reaches a station nearer the body
    /// sooner than it reaches the centre, and the body has not moved as
    /// far when it leaves.
    light_time: bool,
    /// Refer the ecliptic to the true obliquity rather than the mean
    /// one. The choice cancels everywhere except on the observer's own
    /// displacement, which is the only thing in a chart that can see it.
    true_obliquity: bool,
}

impl Reading {
    /// The reading this pass settles on.
    const MEASURED: Reading = Reading {
        aberration: true,
        height: true,
        flattening: true,
        geometric: true,
        retarded: false,
        light_time: true,
        true_obliquity: true,
    };

    /// The station's geocentric position and velocity in the true
    /// equator and equinox of date, astronomical units and astronomical
    /// units a day: `eraPvtob`'s composition with this reading's choices
    /// substituted, so that one of the four is the shipped routine and
    /// the other three are what it would be if a choice went the other
    /// way.
    #[allow(
        clippy::many_single_char_names,
        reason = "ERFA's own names, as `iau::earth` carries them"
    )]
    fn observer(self, place: Place, greenwich_sidereal_deg: f64) -> (Vector3, Vector3) {
        let (a, f) = Ellipsoid::Wgs84.parameters();
        let height = if self.height {
            place.altitude.get()
        } else {
            0.0
        };
        let xyz = earth::gd2gce(
            a,
            if self.flattening { f } else { 0.0 },
            place.longitude.get() * DEG2RAD,
            place.latitude.get() * DEG2RAD,
            height,
        )
        .unwrap_or([0.0, 0.0, 0.0]);
        let (s, c) = (greenwich_sidereal_deg * DEG2RAD).sin_cos();
        let (x, y, z) = (xyz[0], xyz[1], xyz[2]);
        let turn = earth::ROTATION_RATE * DAYSEC;
        (
            [(c * x - s * y) / DAU, (s * x + c * y) / DAU, z / DAU],
            [
                turn * (-s * x - c * y) / DAU,
                turn * (c * x - s * y) / DAU,
                0.0,
            ],
        )
    }

    /// The topocentric position and longitude speed this reading gives
    /// for one recorded geocentric body, degrees and degrees a day.
    ///
    /// The corpus records a longitude speed and neither a latitude nor a
    /// distance speed, so those two enter as nought; §5 says what that
    /// leaves in the residual.
    #[allow(
        clippy::too_many_lines,
        reason = "one reading of the step, written in the order it happens"
    )]
    fn apply(
        self,
        body: Recorded,
        observer: (Vector3, Vector3),
        sky_at: SkyAt,
    ) -> (Spherical, f64) {
        let obliquity_deg = if self.true_obliquity {
            sky_at.obliquity_deg
        } else {
            sky_at.mean_obliquity_deg
        };
        let equatorial = sky::ecliptic_to_equatorial(
            Spherical {
                lon_deg: body.lon_deg,
                lat_deg: body.lat_deg,
            },
            obliquity_deg,
        );
        let seen_from_the_centre =
            vector::s2c(equatorial.lon_deg * DEG2RAD, equatorial.lat_deg * DEG2RAD);
        // The recorded position is apparent, so the Earth's own motion has
        // already turned it. The displacement belongs on the direction the
        // light actually came from.
        let natural = if self.geometric {
            natural_direction(&seen_from_the_centre, &sky_at.earth, sky_at.sun_au)
        } else {
            seen_from_the_centre
        };
        // The geocentric velocity, from the recorded longitude speed with
        // the two rates the corpus does not record set to nought, rotated
        // into the equator with the position.
        let ecliptic_speed = rate(
            body.lon_deg,
            body.lat_deg,
            body.dist_au,
            body.lon_speed_deg * DEG2RAD,
        );
        let velocity = rotate_rate(ecliptic_speed, obliquity_deg);
        let mut position = vector::sxp(body.dist_au, &natural);
        if self.light_time {
            // The station stands `natural · observer` nearer the body than
            // the centre does, so the light it sees left that much earlier,
            // and the body had not travelled as far. The velocity that
            // matters is the body's barycentric one, which is the Earth's
            // plus the recorded geocentric rate.
            let straight = [
                position[0] - observer.0[0],
                position[1] - observer.0[1],
                position[2] - observer.0[2],
            ];
            let saved = (body.dist_au - vector::pm(&straight)) * iau::AULT / DAYSEC;
            position = [
                position[0] + (velocity[0] + sky_at.earth[0]) * saved,
                position[1] + (velocity[1] + sky_at.earth[1]) * saved,
                position[2] + (velocity[2] + sky_at.earth[2]) * saved,
            ];
        }
        // The light took `dist / c` to arrive, and the station turned
        // under it in that time.
        let observer = if self.retarded {
            turned_back(observer, body.dist_au * iau::AULT * earth::ROTATION_RATE)
        } else {
            observer
        };
        let relative = [
            position[0] - observer.0[0],
            position[1] - observer.0[1],
            position[2] - observer.0[2],
        ];
        let relative_rate = [
            velocity[0] - observer.1[0],
            velocity[1] - observer.1[1],
            velocity[2] - observer.1[2],
        ];
        let (distance, unit) = vector::pn(&relative);
        let closing = vector::pdp(&relative, &relative_rate);
        // The direction's own rate: the velocity with the part along the
        // line of sight taken out and the distance divided through.
        let scale = closing / (distance * distance * distance);
        let turning = [
            relative_rate[0] / distance - relative[0] * scale,
            relative_rate[1] / distance - relative[1] * scale,
            relative_rate[2] / distance - relative[2] * scale,
        ];
        let (direction, direction_rate) = if self.aberration {
            // The aberration by the observer's velocity: the station's own
            // when the direction still carries the Earth's, both when the
            // direction has been taken back to the light's own. It is
            // applied by the same routine the inverse used, so the two
            // cannot disagree at the second order in the velocity. The
            // rate is the station's centripetal acceleration; leaving it
            // out would cost two arcseconds a day.
            let per_c = DAU / (DAYSEC * iau::CMPS);
            let turn = earth::ROTATION_RATE * DAYSEC;
            let inward = -turn * turn * per_c;
            let carried = if self.geometric {
                [
                    (observer.1[0] + sky_at.earth[0]) * per_c,
                    (observer.1[1] + sky_at.earth[1]) * per_c,
                    (observer.1[2] + sky_at.earth[2]) * per_c,
                ]
            } else {
                [
                    observer.1[0] * per_c,
                    observer.1[1] * per_c,
                    observer.1[2] * per_c,
                ]
            };
            let bm1 = (1.0 - vector::pdp(&carried, &carried)).sqrt();
            (
                iau::apparent::ab(&unit, &carried, sky_at.sun_au, bm1),
                [
                    turning[0] + observer.0[0] * inward,
                    turning[1] + observer.0[1] * inward,
                    turning[2],
                ],
            )
        } else {
            (unit, turning)
        };
        let (ra, dec) = vector::c2s(&direction);
        let seen = sky::equatorial_to_ecliptic(
            Spherical {
                lon_deg: ra * RAD2DEG,
                lat_deg: dec * RAD2DEG,
            },
            obliquity_deg,
        );
        let speed = longitude_rate(&direction, &direction_rate, obliquity_deg);
        (seen, speed)
    }
}

/// A station's position and velocity as they were `angle` radians of the
/// Earth's rotation ago.
fn turned_back(observer: (Vector3, Vector3), angle: f64) -> (Vector3, Vector3) {
    let (sin, cos) = (-angle).sin_cos();
    let turn = |v: Vector3| [cos * v[0] - sin * v[1], sin * v[0] + cos * v[1], v[2]];
    (turn(observer.0), turn(observer.1))
}

/// What the sky is at one instant, which every body in a chart shares.
#[derive(Clone, Copy, Debug)]
struct SkyAt {
    /// The true obliquity, degrees.
    obliquity_deg: f64,
    /// The mean obliquity, degrees, which the nutation in obliquity —
    /// nine arcseconds at most — separates from the true one.
    mean_obliquity_deg: f64,
    /// The Earth's barycentric velocity in the true equator and equinox
    /// of date, astronomical units a day.
    earth: Vector3,
    /// The Sun's distance, astronomical units, which the aberration's own
    /// gravitational term is divided by.
    sun_au: f64,
}

/// The Earth's barycentric velocity in the true equator and equinox of
/// date, astronomical units a day: `eraEpv00` answers in the celestial
/// frame, so the bias-precession and nutation matrices carry it to the
/// frame the recorded positions are in.
fn earth_velocity(tt: JulianDay<Tt>) -> Vector3 {
    let (date1, date2) = tt.split();
    let state = iau::epv00::epv00(date1, date2);
    let nutation = iau::nut00b(date1, date2);
    let to_true = vector::rxr(
        &iau::apparent::numat(iau::obl06(date1, date2), nutation.dpsi, nutation.deps),
        &iau::p06::pmat06(date1, date2),
    );
    vector::rxp(&to_true, &state.barycentric.velocity)
}

/// The direction the light came from, given the direction it appears to
/// come from and the observer's velocity: `eraAb` inverted by three
/// fixed-point steps, which converges at the square of an angle that is
/// only twenty arcseconds to begin with.
fn natural_direction(apparent: &Vector3, velocity: &Vector3, sun_au: f64) -> Vector3 {
    let per_c = DAU / (DAYSEC * iau::CMPS);
    let beta = [
        velocity[0] * per_c,
        velocity[1] * per_c,
        velocity[2] * per_c,
    ];
    let bm1 = (1.0 - vector::pdp(&beta, &beta)).sqrt();
    let mut natural = *apparent;
    for _ in 0..3 {
        let forward = iau::apparent::ab(&natural, &beta, sun_au, bm1);
        natural = vector::pn(&[
            natural[0] + apparent[0] - forward[0],
            natural[1] + apparent[1] - forward[1],
            natural[2] + apparent[2] - forward[2],
        ])
        .1;
    }
    natural
}

/// The Cartesian rate of a spherical position whose latitude and
/// distance are held still: the ecliptic velocity the corpus's one
/// recorded rate implies.
fn rate(lon_deg: f64, lat_deg: f64, dist_au: f64, lon_speed_rad: f64) -> Vector3 {
    let (sl, cl) = (lon_deg * DEG2RAD).sin_cos();
    let (_, cb) = (lat_deg * DEG2RAD).sin_cos();
    [
        -dist_au * cb * sl * lon_speed_rad,
        dist_au * cb * cl * lon_speed_rad,
        0.0,
    ]
}

/// A rate rotated from the ecliptic into the equator, which is the same
/// rotation the position takes.
fn rotate_rate(v: Vector3, obliquity_deg: f64) -> Vector3 {
    let (se, ce) = (obliquity_deg * DEG2RAD).sin_cos();
    [v[0], v[1] * ce - v[2] * se, v[1] * se + v[2] * ce]
}

/// The ecliptic longitude rate of an equatorial position and velocity,
/// degrees a day.
fn longitude_rate(p: &Vector3, v: &Vector3, obliquity_deg: f64) -> f64 {
    let (se, ce) = (obliquity_deg * DEG2RAD).sin_cos();
    let ecliptic = |w: &Vector3| [w[0], w[1] * ce + w[2] * se, -w[1] * se + w[2] * ce];
    let p = ecliptic(p);
    let v = ecliptic(v);
    let planar = p[0] * p[0] + p[1] * p[1];
    (p[0] * v[1] - p[1] * v[0]) / planar * RAD2DEG
}

/// The signed difference between two longitudes, arcseconds.
fn apart(left: f64, right: f64) -> f64 {
    teistro_core::angle::difference_deg(left, right) / ARCSECOND
}

pub(crate) fn generate(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => {
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask topocentric") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// Every chart the corpus records from both centres.
fn pairs(root: &Path) -> Result<Vec<Pair>, String> {
    let variants = root.join(VARIANTS);
    let mut names: Vec<String> = std::fs::read_dir(&variants)
        .map_err(|err| {
            format!(
                "cannot read {}: {err}. The corpus is a submodule; `git submodule update --init`",
                variants.display()
            )
        })?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .filter_map(|name| name.strip_suffix(".json").map(str::to_string))
        .filter_map(|stem| stem.strip_suffix(GEOCENTRIC).map(str::to_string))
        .collect();
    names.sort();
    let mut found = Vec::new();
    for name in names {
        let topocentric = read_json(&root.join(CHARTS).join(format!("{name}.json")))?;
        let geocentric = read_json(&variants.join(format!("{name}{GEOCENTRIC}.json")))?;
        // The two must differ in the centre and nothing else, or the
        // difference measured here is not the centre's.
        let differing: Vec<String> = settings_apart(&topocentric, &geocentric);
        if differing != ["profile", "topocentric"] {
            return Err(format!(
                "{name}: the pair differs in {differing:?}, not only in the centre"
            ));
        }
        found.push(Pair {
            fixture: name,
            place: place_of(&topocentric)?,
            ut1: JulianDay::try_new(
                topocentric["input"]["resolved"]["jd_ut"]
                    .as_f64()
                    .ok_or("a fixture without an instant")?,
            )
            .map_err(|err| err.to_string())?,
            geocentric: bodies_of(&geocentric),
            topocentric: bodies_of(&topocentric),
            lords: (lord_of(&geocentric), lord_of(&topocentric)),
        });
    }
    if found.is_empty() {
        return Err(String::from(
            "the corpus records no chart from both centres",
        ));
    }
    Ok(found)
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))
}

/// The setting keys two fixtures disagree on.
fn settings_apart(left: &Value, right: &Value) -> Vec<String> {
    let (Some(left), Some(right)) = (left["settings"].as_object(), right["settings"].as_object())
    else {
        return vec![String::from("settings")];
    };
    let mut keys: Vec<String> = left
        .keys()
        .chain(right.keys())
        .filter(|key| left.get(*key) != right.get(*key))
        .cloned()
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

fn place_of(fixture: &Value) -> Result<Place, String> {
    let place = &fixture["input"]["place"];
    Place::try_from_degrees(
        place["latitude"]
            .as_f64()
            .ok_or("a place without a latitude")?,
        place["longitude"]
            .as_f64()
            .ok_or("a place without a longitude")?,
        place["altitude_m"].as_f64().unwrap_or(0.0),
    )
    .map_err(|err| err.to_string())
}

fn bodies_of(fixture: &Value) -> BTreeMap<String, Recorded> {
    fixture["positions"]["bodies"]
        .as_object()
        .map(|bodies| {
            bodies
                .iter()
                .filter_map(|(key, body)| {
                    Some((
                        key.clone(),
                        Recorded {
                            lon_deg: body["tropical_longitude_deg"].as_f64()?,
                            lat_deg: body["latitude_deg"].as_f64()?,
                            dist_au: body["distance_au"].as_f64()?,
                            lon_speed_deg: body["speed_deg_per_day"].as_f64()?,
                            sign: body["sign_index"].as_i64().unwrap_or(-1),
                            nakshatra: body["nakshatra_index"].as_i64().unwrap_or(-1),
                            pada: body["pada"].as_i64().unwrap_or(-1),
                        },
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn lord_of(fixture: &Value) -> String {
    fixture["dashas"]["starting_lord"]
        .as_str()
        .unwrap_or("none")
        .to_string()
}

/// Every fixture in the corpus that records a node, and the latitude it
/// gives it: the evidence that a direction is left where it is, over
/// every chart rather than over the six pairs.
fn node_latitudes(root: &Path) -> Result<Vec<(String, bool, f64)>, String> {
    let mut found = Vec::new();
    for directory in [CHARTS, VARIANTS] {
        let dir = root.join(directory);
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .map_err(|err| format!("cannot read {}: {err}", dir.display()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|e| e == "json"))
            .collect();
        paths.sort();
        for path in &paths {
            let fixture = read_json(path)?;
            let topocentric = fixture["settings"]["topocentric"]
                .as_bool()
                .unwrap_or(false);
            for key in DIRECTIONS {
                if let Some(lat) = fixture["positions"]["bodies"][key]["latitude_deg"].as_f64() {
                    found.push((
                        format!(
                            "{}/{key}",
                            path.file_stem().unwrap_or_default().to_string_lossy()
                        ),
                        topocentric,
                        lat,
                    ));
                }
            }
        }
    }
    Ok(found)
}

/// The sky at a pair's instant, which every body in it shares.
fn sky_at(pair: &Pair) -> Result<SkyAt, String> {
    let (tt, _) = tt_of(pair.ut1, DeltaTModel::TableThenModel).map_err(|e| e.to_string())?;
    let obliquity = sky::obliquity(tt);
    Ok(SkyAt {
        obliquity_deg: obliquity.true_deg,
        mean_obliquity_deg: obliquity.mean_deg,
        earth: earth_velocity(tt),
        sun_au: pair.geocentric.get("SUN").map_or(1.0, |sun| sun.dist_au),
    })
}

/// One reading measured over every pair: the worst it leaves in
/// longitude, in latitude and in the longitude speed, in arcseconds.
struct Residual {
    longitude: f64,
    latitude: f64,
    speed: f64,
    /// Where the worst longitude residual falls, so a reader can go and
    /// look at the row rather than take the number on trust.
    at: String,
}

/// The residuals a reading leaves over every recorded pair.
fn residual(pairs: &[Pair], reading: Reading) -> Result<Residual, String> {
    let mut longitude = 0.0_f64;
    let mut latitude = 0.0_f64;
    let mut speed = 0.0_f64;
    let mut at = String::new();
    for pair in pairs {
        let sky_at = sky_at(pair)?;
        let (tt, _) = tt_of(pair.ut1, DeltaTModel::TableThenModel).map_err(|e| e.to_string())?;
        let sidereal = sky::greenwich_sidereal_time_deg(pair.ut1, tt);
        let observer = reading.observer(pair.place, sidereal);
        for (key, geocentric) in &pair.geocentric {
            if DIRECTIONS.contains(&key.as_str()) {
                continue;
            }
            let Some(recorded) = pair.topocentric.get(key) else {
                continue;
            };
            let (seen, rate) = reading.apply(*geocentric, observer, sky_at);
            let off = apart(seen.lon_deg, recorded.lon_deg).abs();
            if off > longitude {
                longitude = off;
                at = format!("{} {key}", pair.fixture);
            }
            latitude = latitude.max((seen.lat_deg - recorded.lat_deg).abs() / ARCSECOND);
            speed = speed.max((rate - recorded.lon_speed_deg).abs() / ARCSECOND);
        }
    }
    Ok(Residual {
        longitude,
        latitude,
        speed,
        at,
    })
}

/// What the step does to the Moon's latitude, which is what it would do
/// to any point at the Moon's distance — the nodes among them.
fn node_would_show(pairs: &[Pair]) -> f64 {
    worst(pairs.iter().filter_map(|pair| {
        let geocentric = pair.geocentric.get("MOON")?;
        let topocentric = pair.topocentric.get("MOON")?;
        Some((topocentric.lat_deg - geocentric.lat_deg).abs() / ARCSECOND)
    }))
}

/// A distance in kilometres, rounded to the nearest and spaced every
/// three digits as the house style writes a long number.
fn spaced_kilometres(value: f64) -> String {
    let digits = format!("{value:.0}");
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(' ');
        }
        out.push(digit);
    }
    out
}

/// An angle in arcseconds, written at the scale it is.
fn angle(arcseconds: f64) -> String {
    if arcseconds.abs() < 60.0 {
        format!("{arcseconds:.3}″")
    } else if arcseconds.abs() < 3600.0 {
        format!("{:.2}′", arcseconds / 60.0)
    } else {
        format!("{:.3}°", arcseconds / 3600.0)
    }
}

/// A residual, always in arcseconds and always to the same place, so a
/// column of them can be read down.
fn residual_of(arcseconds: f64) -> String {
    format!("{arcseconds:.6}″")
}

/// The readings this pass measures: the design page's own, three that
/// each add one term, and five that take one choice back out of the
/// reading those three settle on.
fn readings() -> Vec<(&'static str, &'static str, Reading)> {
    let plain = Reading {
        aberration: false,
        geometric: false,
        retarded: false,
        light_time: false,
        true_obliquity: true,
        height: true,
        flattening: true,
    };
    vec![
        (
            "the observer's displacement alone, which §4 designs",
            "displacement",
            plain,
        ),
        (
            "and the station's own aberration",
            "+ aberration",
            Reading {
                aberration: true,
                ..plain
            },
        ),
        (
            "and on the direction the light came from",
            "+ geometric",
            Reading {
                aberration: true,
                geometric: true,
                ..plain
            },
        ),
        (
            "and carrying the body over the light time the station saves",
            "+ light time",
            Reading::MEASURED,
        ),
        (
            "the light time without taking the Earth's aberration off first",
            "light time alone",
            Reading {
                aberration: true,
                light_time: true,
                ..plain
            },
        ),
        (
            "the settled reading referred to the mean obliquity",
            "mean obliquity",
            Reading {
                true_obliquity: false,
                ..Reading::MEASURED
            },
        ),
        (
            "the settled reading with the station where the light left it",
            "retarded station",
            Reading {
                retarded: true,
                ..Reading::MEASURED
            },
        ),
        (
            "the settled reading on a sphere of the equatorial radius",
            "a sphere",
            Reading {
                flattening: false,
                ..Reading::MEASURED
            },
        ),
        (
            "the settled reading at sea level rather than the place's height",
            "sea level",
            Reading {
                height: false,
                ..Reading::MEASURED
            },
        ),
    ]
}

/// Every claim this pass decides, in the order the page argues them.
#[allow(
    clippy::too_many_lines,
    reason = "one claim after another, in the order the page argues them"
)]
fn claims_of(
    pairs: &[Pair],
    bodies: &[String],
    measured: &[(&str, &str, Reading, Residual)],
    nodes: &[(String, bool, f64)],
    bound: f64,
) -> Result<(Vec<Claim>, f64, usize), String> {
    let mut claims = Vec::new();
    for (name, _, _, residual) in measured {
        claims.push(Claim::stated(
            (*name).to_string(),
            verdict_of(residual.longitude <= bound && residual.latitude <= bound),
            format!(
                "worst {} in longitude, {} in latitude",
                residual_of(residual.longitude),
                residual_of(residual.latitude)
            ),
        ));
    }

    // The shipped routine is the measured reading's WGS84 case; the two
    // must be the same station.
    let mut station_apart = 0.0_f64;
    for pair in pairs {
        let (tt, _) = tt_of(pair.ut1, DeltaTModel::TableThenModel).map_err(|e| e.to_string())?;
        let sidereal = sky::greenwich_sidereal_time_deg(pair.ut1, tt);
        let mine = Reading::MEASURED.observer(pair.place, sidereal);
        let shipped = sky::observer(pair.place, pair.ut1, tt);
        for axis in 0..3 {
            station_apart =
                station_apart.max((mine.0[axis] - shipped.position_au[axis]).abs() * DAU);
        }
    }
    // The settled reading, judged where the corpus can judge it: the Moon
    // is the only body whose parallax is large enough to leave anything.
    let rows = moved(pairs, bodies);
    let placed: Vec<&Moved> = rows
        .iter()
        .filter(|row| !DIRECTIONS.contains(&row.body.as_str()))
        .collect();
    let moon = placed.iter().find(|row| row.body == "MOON");
    let moon_left = moon.map_or(0.0, |row| row.left);
    let moon_step = moon.map_or(1.0, |row| row.longitude);
    let others = worst(
        placed
            .iter()
            .filter(|row| row.body != "MOON")
            .map(|row| row.left),
    );
    claims.push(Claim::stated(
        "the settled reading, on every body but the Moon",
        verdict_of(others <= bound),
        format!(
            "worst {} over the other {} bodies, {:.0} times inside the bound",
            residual_of(others),
            spelled(placed.len() - 1),
            bound / others.max(f64::MIN_POSITIVE)
        ),
    ));
    claims.push(Claim::stated(
        "the settled reading, on the Moon",
        verdict_of(moon_left <= bound),
        format!(
            "worst {}, {:.0} times the bound and {:.0e} of the step it makes",
            residual_of(moon_left),
            moon_left / bound,
            moon_left / moon_step
        ),
    ));
    claims.push(Claim::stated(
        "`sky::observer` is that reading's station",
        verdict_of(station_apart < 1e-6),
        format!("worst {station_apart:.3e} m apart over {}", pairs.len()),
    ));

    // A direction takes neither step.
    let mut direction_rows = 0_usize;
    let mut direction_moved = 0_usize;
    for pair in pairs {
        for key in DIRECTIONS {
            let (Some(geocentric), Some(topocentric)) =
                (pair.geocentric.get(key), pair.topocentric.get(key))
            else {
                continue;
            };
            direction_rows += 1;
            // Exact comparison is the claim: a node that took the step
            // would differ in the first decimal, not the last bit.
            let same = [
                (geocentric.lon_deg, topocentric.lon_deg),
                (geocentric.lat_deg, topocentric.lat_deg),
                (geocentric.dist_au, topocentric.dist_au),
            ]
            .iter()
            .all(|(left, right)| left.to_bits() == right.to_bits());
            if !same {
                direction_moved += 1;
            }
        }
    }
    claims.push(
        Claim::counted(
            "a direction on the Moon's orbit is where it was",
            direction_moved,
            direction_rows,
        )
        .with_note("longitude, latitude and distance, bit for bit"),
    );
    let seen_from_a_place: Vec<&(String, bool, f64)> = nodes.iter().filter(|row| row.1).collect();
    let worst_node_latitude = worst(seen_from_a_place.iter().map(|row| row.2.abs()));
    claims.push(
        Claim::counted(
            "and it stays on the ecliptic in every chart cast from a place",
            seen_from_a_place
                .iter()
                .filter(|row| row.2.abs() > 1e-12)
                .count(),
            seen_from_a_place.len(),
        )
        .with_note(format!(
            "worst latitude {worst_node_latitude:.1e}°, where a displaced point at that distance would show {}",
            angle(node_would_show(pairs))
        )),
    );

    // The speed, which this corpus cannot decide.
    let mut speed_change = 0.0_f64;
    for pair in pairs {
        for (key, geocentric) in &pair.geocentric {
            if let Some(topocentric) = pair.topocentric.get(key) {
                speed_change = speed_change
                    .max((topocentric.lon_speed_deg - geocentric.lon_speed_deg).abs() / ARCSECOND);
            }
        }
    }
    let settled = measured
        .iter()
        .find(|(_, _, reading, _)| *reading == Reading::MEASURED)
        .ok_or("the settled reading is one of them")?;
    claims.push(Claim::stated(
        "the speed is the velocity difference",
        Verdict::Untested,
        format!(
            "a change of up to {} left within {}, which is the size of the two rates the corpus does not record",
            angle(speed_change),
            angle(settled.3.speed)
        ),
    ));

    Ok((claims, speed_change, seen_from_a_place.len()))
}

fn page(root: &Path) -> Result<String, String> {
    let pairs = pairs(root)?;
    let nodes = node_latitudes(root)?;
    let bodies: Vec<String> = pairs
        .first()
        .map(|pair| pair.geocentric.keys().cloned().collect())
        .unwrap_or_default();

    // The corpus's own bound for a run against the ephemeris the fixtures
    // were produced with (`fixtures/tolerances.json`, `same-ephemeris`).
    let bound = 1e-6 / ARCSECOND;

    let readings = readings();
    let mut measured = Vec::new();
    for (name, short, reading) in readings {
        measured.push((name, short, reading, residual(&pairs, reading)?));
    }
    let (claims, speed_change, node_rows) = claims_of(&pairs, &bodies, &measured, &nodes, bound)?;
    Ok(fill(&text(
        &pairs,
        &bodies,
        &measured,
        &claims,
        bound,
        speed_change,
        node_rows,
    )?))
}

/// What the step moves, per body, over the recorded pairs.
struct Moved {
    body: String,
    longitude: f64,
    latitude: f64,
    speed: f64,
    /// What the reading this pass settles on leaves on this body.
    left: f64,
    /// What the displacement alone leaves on it, which is the evidence
    /// that the missing piece is not a displacement.
    left_plain: f64,
    signs: usize,
    nakshatras: usize,
    padas: usize,
}

fn moved(pairs: &[Pair], bodies: &[String]) -> Vec<Moved> {
    bodies
        .iter()
        .map(|body| {
            let mut row = Moved {
                body: body.clone(),
                longitude: 0.0,
                latitude: 0.0,
                speed: 0.0,
                left: 0.0,
                left_plain: 0.0,
                signs: 0,
                nakshatras: 0,
                padas: 0,
            };
            for pair in pairs {
                let (Some(geocentric), Some(topocentric)) =
                    (pair.geocentric.get(body), pair.topocentric.get(body))
                else {
                    continue;
                };
                row.longitude = row
                    .longitude
                    .max(apart(topocentric.lon_deg, geocentric.lon_deg).abs());
                row.latitude = row
                    .latitude
                    .max((topocentric.lat_deg - geocentric.lat_deg).abs() / ARCSECOND);
                row.speed = row
                    .speed
                    .max((topocentric.lon_speed_deg - geocentric.lon_speed_deg).abs() / ARCSECOND);
                if let (Ok(sky_at), Ok((tt, _))) =
                    (sky_at(pair), tt_of(pair.ut1, DeltaTModel::TableThenModel))
                {
                    let sidereal = sky::greenwich_sidereal_time_deg(pair.ut1, tt);
                    let plain = Reading {
                        aberration: false,
                        geometric: false,
                        light_time: false,
                        ..Reading::MEASURED
                    };
                    for (reading, worst) in [(Reading::MEASURED, true), (plain, false)] {
                        let observer = reading.observer(pair.place, sidereal);
                        let (seen, _) = reading.apply(*geocentric, observer, sky_at);
                        let off = apart(seen.lon_deg, topocentric.lon_deg).abs();
                        if worst {
                            row.left = row.left.max(off);
                        } else {
                            row.left_plain = row.left_plain.max(off);
                        }
                    }
                }
                row.signs += usize::from(topocentric.sign != geocentric.sign);
                row.nakshatras += usize::from(topocentric.nakshatra != geocentric.nakshatra);
                row.padas += usize::from(topocentric.pada != geocentric.pada);
            }
            row
        })
        .collect()
}

#[allow(
    clippy::too_many_lines,
    reason = "one generated page, written in the order it reads"
)]
fn text(
    pairs: &[Pair],
    bodies: &[String],
    measured: &[(&str, &str, Reading, Residual)],
    claims: &[Claim],
    bound: f64,
    speed_change: f64,
    node_rows: usize,
) -> Result<String, String> {
    let rows = moved(pairs, bodies);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# The topocentric centre, measured\n\n\
         Status: `generated` by `cargo xtask topocentric` over the {} charts\n\
         the conformance corpus records from both centres. Do not edit:\n\
         `check-topocentric` regenerates this page and fails on any\n\
         difference. The design written from it is\n\
         [`astro-timescales-and-frames.md`](astro-timescales-and-frames.md) §4.\n",
        pairs.len()
    );

    let _ = writeln!(
        out,
        "## 1. What the corpus records\n\n\
         {} charts are recorded twice — once from the centre of the Earth\n\
         and once from the place they were cast for — with every other\n\
         setting equal: {}. Each pair is an input and its answer, so a\n\
         proposed reading of the step reproduces the recorded numbers or it\n\
         does not, and nothing here is a comparison inside a tolerance of\n\
         this pass's own choosing. The bound below is the corpus's own for a\n\
         run against the ephemeris the fixtures came from\n\
         (`fixtures/tolerances.json`, `same-ephemeris`): a millionth of a\n\
         degree of longitude, which is {}.\n",
        pairs.len(),
        pairs
            .iter()
            .map(|pair| format!("`{}`", pair.fixture))
            .collect::<Vec<_>>()
            .join(", "),
        residual_of(bound)
    );

    let _ = writeln!(
        out,
        "Every other fixture in the corpus is recorded from a place only, so\n\
         none of them can decide the step; one of their fields can still\n\
         falsify part of it, and §4 uses all {} rows of it.\n",
        count(node_rows)
    );

    let _ = writeln!(
        out,
        "## 2. {} readings of one step\n\n\
         `astro-timescales-and-frames.md` §4 already describes the step:\n\
         \"the observer's geocentric position (WGS84) and the parallax\". A\n\
         displacement is what a topocentric position obviously is — the body\n\
         is seen from a point some six thousand kilometres off the centre —\n\
         and the first reading below is exactly that. Each of the next three\n\
         adds one term; the rest take one choice back out of the reading\n\
         those three settle on, to check it rather than assume it.\n",
        {
            let word = spelled(measured.len());
            let mut chars = word.chars();
            chars.next().map_or(word.clone(), |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        }
    );
    let mut table_out = String::from(
        "| reading | worst longitude | worst latitude | worst at |\n|---|---:|---:|---|\n",
    );
    for (name, _, _, residual) in measured {
        let _ = writeln!(
            table_out,
            "| {name} | {} | {} | {} |",
            residual_of(residual.longitude),
            residual_of(residual.latitude),
            residual.at
        );
    }
    let _ = writeln!(out, "{table_out}");

    let displacement = measured.first().ok_or("the readings were measured")?;
    let best = measured
        .iter()
        .find(|(_, _, reading, _)| *reading == Reading::MEASURED)
        .ok_or("the settled reading is one of them")?;
    let _ = writeln!(
        out,
        "The designed reading is falsified, and the per-body table in §6\n\
         says why in one column: the displacement alone leaves about a\n\
         third of an arcsecond on **every** body — on Saturn, whose whole\n\
         parallax is under an arcsecond, as much as on the Moon, whose\n\
         parallax is forty arcminutes. A residual that does not shrink\n\
         with distance is not a displacement gone wrong; it is a rotation\n\
         the reading has left out. The station is moving, four hundred\n\
         metres a second eastward at these latitudes, and the light it\n\
         receives arrives from a direction aberrated by its own velocity:\n\
         one and a half parts in a million, a third of an arcsecond,\n\
         whatever the body's distance. With it the planets come inside a\n\
         thousandth of an arcsecond and stay there for every reading\n\
         below, so from here on the Moon is the only witness.\n"
    );
    let _ = writeln!(
        out,
        "It takes two more terms, and neither is a displacement either.\n\
         The recorded position is **apparent**: the Earth's own motion has\n\
         already turned it by twenty arcseconds, and displacing a\n\
         direction that has been turned displaces it in the wrong\n\
         direction by that much of itself. And the station stands nearer\n\
         the body than the centre does, by up to an Earth radius, so the\n\
         light it sees left the body later — by up to a fiftieth of a\n\
         second, in which the Moon travels six hundred metres of its\n\
         barycentric path. Each is worth about a third of an arcsecond on\n\
         the Moon and neither is worth anything on anything else; put in\n\
         one at a time they make the answer worse, and put in together\n\
         they take it from {} to {}.\n",
        residual_of(displacement.3.longitude),
        residual_of(best.3.longitude)
    );

    let mut witness = format!(
        "| reading | {} |\n|---|{}\n",
        pairs
            .iter()
            .map(|pair| pair.fixture.get(..4).unwrap_or("?").to_string())
            .collect::<Vec<_>>()
            .join(" | "),
        "---:|".repeat(pairs.len())
    );
    let mut shifts = Vec::new();
    for pair in pairs {
        let (Some(geocentric), Some(topocentric)) =
            (pair.geocentric.get("MOON"), pair.topocentric.get("MOON"))
        else {
            continue;
        };
        shifts.push(angle(apart(topocentric.lon_deg, geocentric.lon_deg)));
    }
    let _ = writeln!(witness, "| *the step itself* | {} |", shifts.join(" | "));
    for (_, short, reading, _) in measured {
        let mut cells = Vec::new();
        for pair in pairs {
            let (Some(geocentric), Some(topocentric)) =
                (pair.geocentric.get("MOON"), pair.topocentric.get("MOON"))
            else {
                continue;
            };
            let sky_at = sky_at(pair)?;
            let (tt, _) =
                tt_of(pair.ut1, DeltaTModel::TableThenModel).map_err(|e| e.to_string())?;
            let sidereal = sky::greenwich_sidereal_time_deg(pair.ut1, tt);
            let observer = reading.observer(pair.place, sidereal);
            let (seen, _) = reading.apply(*geocentric, observer, sky_at);
            cells.push(residual_of(apart(seen.lon_deg, topocentric.lon_deg)));
        }
        let _ = writeln!(witness, "| {short} | {} |", cells.join(" | "));
    }
    let _ = writeln!(
        out,
        "What the Moon says, pair by pair and signed, so that a reading\n\
         which is right on average and wrong everywhere cannot hide behind\n\
         a worst case. The first row is the size of the step itself; every\n\
         row under it is what the reading failed to account for:\n"
    );
    let _ = writeln!(out, "{witness}");

    let sphere = measured
        .iter()
        .find(|(_, _, reading, _)| !reading.flattening)
        .map(|row| row.3.longitude)
        .unwrap_or_default();
    let sea_level = measured
        .iter()
        .find(|(_, _, reading, _)| !reading.height)
        .map(|row| row.3.longitude)
        .unwrap_or_default();
    let _ = writeln!(
        out,
        "## 3. The Earth the observer stands on\n\n\
         Two more choices are the kind a reader would call harmless, and\n\
         the same six pairs falsify both. Standing the observer on a\n\
         sphere of the Earth's equatorial radius rather than on the WGS84\n\
         ellipsoid moves them by up to twenty-one kilometres and leaves\n\
         {}; standing them at sea level rather than at the place's own\n\
         height — 1 400 m at Kathmandu, 3 640 m at La Paz — leaves {}.\n\
         Against a bound of {} both are decisive, and the second is the\n\
         one worth naming: a field a consumer may not bother to fill in\n\
         is worth two hundred times the tolerance the corpus is compared\n\
         at.\n",
        residual_of(sphere),
        residual_of(sea_level),
        residual_of(bound)
    );

    let node_distance = pairs
        .first()
        .and_then(|pair| pair.geocentric.get("RAHU"))
        .map_or(0.0, |node| node.dist_au);
    let _ = writeln!(
        out,
        "## 4. A direction is not a place\n\n\
         The lunar nodes are recorded in every one of these charts, and in\n\
         the pairs they are the same numbers under both centres — not close,\n\
         **identical**, to the last bit of a double, in longitude, latitude\n\
         and distance alike. Their recorded distance is the same constant in\n\
         every fixture in the corpus, {} astronomical units, which is\n\
         {} km: a nominal figure and not a measurement, and the first sign\n\
         that what is recorded is a direction rather than a place.\n",
        spaced(node_distance, 9),
        spaced_kilometres(node_distance * teistro_astro::iau::DAU / 1_000.0)
    );
    let _ = writeln!(
        out,
        "The fixtures recorded from a place only cannot compare\n\
         the two centres, but they can still falsify this: a node displaced\n\
         by an observer would leave the ecliptic by up to {}, and the node's\n\
         recorded latitude is zero to the last bits of a double in every one\n\
         of them, the true node included. So the rule is about what a point\n\
         **is**, and not about which of them the corpus happened to record:\n\
         a point defined as a direction on the Moon's orbit is not anywhere,\n\
         and no observer sees it displaced.\n",
        angle(node_would_show(pairs))
    );

    let _ = writeln!(
        out,
        "## 5. The speed, which this corpus cannot decide\n\n\
         The step changes a speed far more than it changes a position: the\n\
         Moon's longitude speed differs between the two centres by up to {},\n\
         a third of its own value, because the observer is carried eastward\n\
         at close to a twentieth of the Moon's own rate. A step that moved\n\
         positions and left speeds alone would be wrong by more than the\n\
         positions it corrected.\n",
        angle(speed_change)
    );
    let _ = writeln!(
        out,
        "The transform is the position's: subtract the station's velocity\n\
         from the body's and read the longitude rate off the difference. It\n\
         cannot be **decided** here, because the corpus records a longitude\n\
         speed and neither a latitude speed nor a distance speed, and both\n\
         enter the answer. Setting the two it does not record to nought\n\
         reproduces the recorded topocentric speed to {}, which is the size\n\
         of what is missing rather than of the rule. What settles it is a\n\
         provider that answers both centres natively — the Teimeris adapter\n\
         declares the topocentric override — compared over a grid, which is\n\
         `05-testing/ACCURACY.md`'s business and not this page's.\n",
        angle(best.3.speed)
    );

    let _ = writeln!(out, "## 6. What the step is worth\n");
    let mut magnitudes = String::from(
        "| body | worst longitude | worst latitude | worst speed | displacement alone leaves | the settled reading leaves | signs | nakshatras | padas |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    for row in &rows {
        let _ = writeln!(
            magnitudes,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.body,
            angle(row.longitude),
            angle(row.latitude),
            angle(row.speed),
            residual_of(row.left_plain),
            residual_of(row.left),
            row.signs,
            row.nakshatras,
            row.padas
        );
    }
    let _ = writeln!(out, "{magnitudes}");
    let _ = writeln!(
        out,
        "The two columns on the right are the argument of §2 in one place.\n\
         The displacement alone leaves a quarter to a third of an\n\
         arcsecond on every body from the Sun to Saturn, whose parallaxes\n\
         differ by a factor of thirteen; the settled reading leaves under a\n\
         thousandth of an arcsecond on all of them. `RAHU` and `KETU` are\n\
         in the table under both readings because a reading that displaced\n\
         them would be out by three quarters of a degree — which is §4's\n\
         claim, measured rather than asserted.\n"
    );
    let lords: Vec<String> = pairs
        .iter()
        .filter(|pair| pair.lords.0 != pair.lords.1)
        .map(|pair| format!("`{}` ({} → {})", pair.fixture, pair.lords.0, pair.lords.1))
        .collect();
    let _ = writeln!(
        out,
        "Over {} pairs the step moves a body across a pada {}, across a\n\
         nakshatra {} and across a sign {}. It also changes which lord the\n\
         Vimshottari dasha begins with in {} of the {}: {}. The centre is\n\
         not a refinement; it is a setting that changes what the chart\n\
         says.\n",
        pairs.len(),
        times(rows.iter().map(|row| row.padas).sum::<usize>()),
        times(rows.iter().map(|row| row.nakshatras).sum::<usize>()),
        times(rows.iter().map(|row| row.signs).sum::<usize>()),
        count(lords.len()),
        pairs.len(),
        if lords.is_empty() {
            String::from("none of them")
        } else {
            lords.join(", ")
        }
    );

    let _ = writeln!(out, "## 7. What this pass decides\n");
    let _ = writeln!(out, "{}", table(claims));
    let _ = writeln!(
        out,
        "The step is a displacement, an aberration, a light time and a\n\
         direction taken back to the light's own, on the WGS84 ellipsoid at\n\
         the place's own height, applied to every body that is somewhere\n\
         and to no point that is a direction, with the velocity transformed\n\
         as the position is.\n"
    );
    let _ = writeln!(
        out,
        "Two things are left open and are recorded rather than rounded\n\
         away. The Moon keeps {} — three parts in a hundred thousand of a\n\
         step of forty arcminutes, and every other body is a thousand times\n\
         further inside the bound — and nothing this pass could construct\n\
         accounts for it; the candidates are the recording engine's own\n\
         bookkeeping for the Moon's light time and the two rates the corpus\n\
         does not record, and neither can be told apart here. And the\n\
         velocity transform cannot be decided by this corpus at all. Both\n\
         belong to a comparison against a provider that answers both\n\
         centres natively, which `05-testing/ACCURACY.md` is for.\n",
        residual_of(
            rows.iter()
                .find(|row| row.body == "MOON")
                .map_or(0.0, |row| row.left)
        )
    );
    Ok(out)
}

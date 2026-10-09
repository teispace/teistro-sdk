//! A provider against JPL's DE440 itself, through the corpus's `jpl/`
//! recording (CSPICE over NAIF's kernels, evidence rank 1).
//!
//! The geometric rung isolates the ephemeris: a geometric state carries no
//! light time, aberration or deflection, so what a provider's geometric
//! position misses it by is the provider's theory and nothing else. The
//! provider is asked for geocentric positions on the J2000 ecliptic, which
//! the corpus records as `ECLIPJ2000` states over DE440, and each body's
//! direction and distance are compared under the band `tolerances.json`
//! gives the provider's class (`jpl.geometric.*`).
//!
//! The states are stamped in TDB, and the port asks in TT, so each instant
//! is turned into TT by the two leading terms of TDB − TT (Fairhead and
//! Bretagnon's series; the rest are below two microseconds, which the
//! Moon covers in a micro-arcsecond).
//!
//! The answer is a [`CorpusReport`] like the recorded charts', one result
//! per instant, so it is read, written and judged the same way: against
//! [`KNOWN`], the divergences from DE440 measured and explained.
//!
//! ```no_run
//! use std::path::Path;
//! use teistro_ephemeris_kit::corpus::Corpus;
//! use teistro_ephemeris_kit::jpl;
//! use teistro_port_ephemeris::TestProvider;
//!
//! let corpus = Corpus::open(Path::new("fixtures"))?;
//! let run = jpl::geometric(&corpus, &TestProvider::new(), "builtin-compact")?;
//! println!("{}", run.report.markdown());
//! # Ok::<(), String>(())
//! ```

use serde::Deserialize;
use teistro_core::math;
use teistro_port_ephemeris::{
    Body, CellStatus, Centre, Coordinates, Corrections, EphemerisProvider, Equinox, Frame,
    PositionColumns, PositionRequest, TimeScale, Zodiac, ask_positions,
};

use crate::corpus::{
    Corpus, CorpusId, CorpusReport, Counts, Divergence, Field, FixtureResult, Implementation,
    Outcome, REPORT_SCHEMA,
};

/// Where the states are, under the corpus's root.
const STATES: &str = "jpl/cspice/states.json";

/// The schema the states file declares.
const STATES_SCHEMA: &str = "teistro-conformance/jpl-cspice-states/1";

/// The astronomical unit in kilometres (IAU 2012 Resolution B2), which is
/// the unit the port's distances are in.
const AU_KM: f64 = 149_597_870.7;

/// The frame a geometric comparison asks for: what the states are, with no
/// correction and the J2000 equinox.
pub const GEOMETRIC: Frame = Frame {
    centre: Centre::Geocentric,
    equinox: Equinox::J2000,
    coordinates: Coordinates::Ecliptic,
    zodiac: Zodiac::Tropical,
    corrections: Corrections::GEOMETRIC,
};

/// Each recorded target by its NAIF id, the port's body, and whether its
/// band is the outer planets'. Mars to Pluto are system barycentres,
/// which is all `de440.bsp` carries for them: a planet's centre stands off
/// its barycentre by at most a tenth of an arcsecond (Pluto, by Charon),
/// inside every class but `de`, which a DE reader answers in barycentres
/// anyway.
const TARGETS: [(&str, Body, bool); 10] = [
    ("10", Body::Sun, false),
    ("301", Body::Moon, false),
    ("199", Body::Mercury, false),
    ("299", Body::Venus, false),
    ("4", Body::Mars, false),
    ("5", Body::Jupiter, true),
    ("6", Body::Saturn, true),
    ("7", Body::Uranus, true),
    ("8", Body::Neptune, true),
    ("9", Body::Pluto, true),
];

#[derive(Deserialize)]
struct States {
    schema: String,
    instants: Vec<Instant>,
    series: Vec<Series>,
}

#[derive(Deserialize)]
struct Instant {
    jd: f64,
}

#[derive(Deserialize)]
struct Series {
    target: String,
    kernel: String,
    epoch: String,
    frame: String,
    correction: String,
    /// One row an instant: x, y, z in km, then velocity and light time.
    rows: Vec<Vec<f64>>,
}

impl Series {
    fn geometric_de440(&self) -> bool {
        self.kernel == "de440"
            && self.epoch == "tdb"
            && self.frame == "ECLIPJ2000"
            && self.correction == "NONE"
    }
}

/// A geometric run: the report, and the instants the provider does not
/// cover, which were not asked and are not in it.
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    /// One result per instant the provider covers.
    pub report: CorpusReport,
    /// The recorded instants outside the provider's declared range.
    pub outside: Vec<f64>,
}

/// TDB − TT in days at a TDB instant: the two leading terms of Fairhead
/// and Bretagnon (1990), 1.657 ms and 22 µs.
fn tdb_minus_tt(jd_tdb: f64) -> f64 {
    let days = jd_tdb - 2_451_545.0;
    let g = (357.53 + 0.985_600_28 * days).to_radians();
    let l = (246.11 + 0.902_517_92 * days).to_radians();
    let lj = (355.0 + 0.083_092 * days).to_radians();
    let seconds = 0.001_657 * math::sin(g) + 0.000_022 * math::sin(l - lj);
    seconds / 86_400.0
}

/// The angle between two directions, by the arctangent of the cross and
/// dot products, which keeps its precision at small angles.
fn separation_deg(a: [f64; 3], b: [f64; 3]) -> f64 {
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let along = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let across = math::hypot(math::hypot(cross[0], cross[1]), cross[2]);
    math::atan2(across, along).to_degrees()
}

/// A unit direction from an ecliptic longitude and latitude in degrees.
fn direction(lon_deg: f64, lat_deg: f64) -> [f64; 3] {
    let (sin_lon, cos_lon) = math::sin_cos(lon_deg.to_radians());
    let (sin_lat, cos_lat) = math::sin_cos(lat_deg.to_radians());
    [cos_lat * cos_lon, cos_lat * sin_lon, sin_lat]
}

/// One recorded body: the port's body, whether its band is the outer
/// planets', and its position at each instant, in km.
struct Track {
    body: Body,
    outer: bool,
    positions: Vec<[f64; 3]>,
}

/// The recorded geometric states: the instants, and each body's track.
struct Recorded {
    instants: Vec<f64>,
    tracks: Vec<Track>,
}

/// Reads the geometric DE440 states of every target.
fn read(corpus: &Corpus) -> Result<Recorded, String> {
    let path = corpus.path(STATES);
    let text = std::fs::read_to_string(&path)
        .map_err(|why| format!("{}: {why}; the corpus is older than 0.12.0", path.display()))?;
    let states: States = serde_json::from_str(&text).map_err(|why| format!("{STATES}: {why}"))?;
    if states.schema != STATES_SCHEMA {
        return Err(format!(
            "{STATES} is `{}`, and this reads `{STATES_SCHEMA}`",
            states.schema
        ));
    }
    let instants: Vec<f64> = states.instants.iter().map(|instant| instant.jd).collect();
    let mut tracks = Vec::new();
    for (target, body, outer) in TARGETS {
        let series = states
            .series
            .iter()
            .find(|series| series.target == target && series.geometric_de440())
            .ok_or_else(|| format!("{STATES} has no geometric ECLIPJ2000 series for {target}"))?;
        if series.rows.len() != instants.len() {
            return Err(format!(
                "{STATES}: target {target} has {} rows for {} instants",
                series.rows.len(),
                instants.len()
            ));
        }
        let positions = series
            .rows
            .iter()
            .map(|row| match row.as_slice() {
                [x, y, z, ..] => Ok([*x, *y, *z]),
                _ => Err(format!(
                    "{STATES}: a row of target {target} has no position"
                )),
            })
            .collect::<Result<Vec<_>, String>>()?;
        tracks.push(Track {
            body,
            outer,
            positions,
        });
    }
    Ok(Recorded { instants, tracks })
}

/// The field a body's comparison is filed under.
fn path(body: Body, outer: bool, quantity: &str) -> String {
    let key = body.key();
    if outer && quantity == "direction_deg" {
        format!("jpl.geometric.outer.{key}.{quantity}")
    } else {
        format!("jpl.geometric.{key}.{quantity}")
    }
}

/// One recorded instant against the provider's row of cells: each body's
/// direction and distance, or a skip naming the bodies it gave no cell for.
fn compare(
    tracks: &[Track],
    columns: &PositionColumns,
    row: usize,
    (index, jd): (usize, f64),
    band: impl Fn(&str) -> f64,
) -> FixtureResult {
    let mut fields = Vec::new();
    let mut refused = Vec::new();
    for (column, track) in tracks.iter().enumerate() {
        let (Some(cell), Some(&[x, y, z])) = (columns.at(row, column), track.positions.get(index))
        else {
            refused.push(track.body.key());
            continue;
        };
        if cell.status != CellStatus::Ok {
            refused.push(track.body.key());
            continue;
        }
        let compared = [
            (
                path(track.body, track.outer, "direction_deg"),
                0.0,
                separation_deg(direction(cell.lon, cell.lat), [x, y, z]),
            ),
            (
                path(track.body, track.outer, "distance_km"),
                math::hypot(math::hypot(x, y), z),
                cell.dist * AU_KM,
            ),
        ];
        for (path, expected, found) in compared {
            let tolerance = band(&path);
            let difference = (found - expected).abs();
            fields.push(Field {
                within: difference <= tolerance,
                path,
                expected,
                found,
                difference,
                tolerance,
            });
        }
    }
    let missed: Vec<&str> = fields
        .iter()
        .filter(|field| !field.within)
        .map(|field| field.path.as_str())
        .collect();
    let (outcome, reason) = if !refused.is_empty() {
        (
            Outcome::Skip,
            Some(format!(
                "inside its range, the provider gave no {}",
                refused.join(", ")
            )),
        )
    } else if missed.is_empty() {
        (Outcome::Pass, None)
    } else {
        (
            Outcome::Fail,
            Some(format!("outside its band: {}", missed.join(", "))),
        )
    };
    FixtureResult {
        fixture: format!("jd-{jd}"),
        outcome,
        reason,
        fields,
    }
}

/// The provider's geometric positions against DE440's at every recorded
/// instant it covers, under the band of `class`.
///
/// # Errors
///
/// A class the corpus does not name, a corpus without the `jpl/` states,
/// or a provider that refuses the geometric J2000 frame or a body.
pub fn geometric<P: EphemerisProvider + ?Sized>(
    corpus: &Corpus,
    provider: &P,
    class: &str,
) -> Result<Run, String> {
    corpus.tolerances.class(class)?;
    let recorded = read(corpus)?;
    let capabilities = provider.capabilities();
    let tt = |jd: f64| jd - tdb_minus_tt(jd);
    let mut covered = Vec::new();
    let mut outside = Vec::new();
    for (index, jd) in recorded.instants.iter().copied().enumerate() {
        if capabilities.covers(tt(jd)) {
            covered.push((index, jd));
        } else {
            outside.push(jd);
        }
    }
    let jds: Vec<f64> = covered.iter().map(|(_, jd)| tt(*jd)).collect();
    let bodies: Vec<Body> = recorded.tracks.iter().map(|track| track.body).collect();
    let request = PositionRequest {
        speeds: false,
        ..PositionRequest::new(&jds, TimeScale::Tt, &bodies, GEOMETRIC)
    };
    let columns = ask_positions(provider, &capabilities, &request).map_err(|why| {
        format!(
            "{} refused the geometric request: {why}",
            capabilities.identity.name
        )
    })?;
    let band = |path: &str| corpus.tolerances.band(path, class);
    let results: Vec<FixtureResult> = covered
        .iter()
        .enumerate()
        .map(|(row, instant)| compare(&recorded.tracks, &columns, row, *instant, band))
        .collect();
    Ok(Run {
        report: CorpusReport {
            schema: REPORT_SCHEMA,
            corpus: CorpusId {
                version: corpus.version.clone(),
                commit: corpus.commit.clone(),
            },
            implementation: Implementation {
                name: capabilities.identity.name,
                version: capabilities.identity.version,
                binding: "rust",
                provider_class: class.to_owned(),
                platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
                settings_hash: None,
            },
            counts: Counts::of(&results),
            results,
        },
        outside,
    })
}

/// Where a provider parts from DE440's geometric states, and why; each
/// measured on 2026-10-09 over every built-in tier against corpus 0.12.0,
/// and judged both ways like the charts' (`CorpusReport::against`).
pub const KNOWN: [Divergence; 3] = [
    Divergence {
        name: "vsop87-radius-vectors",
        fields: &[
            "jpl.geometric.MARS.distance_km",
            "jpl.geometric.JUPITER.distance_km",
            "jpl.geometric.SATURN.distance_km",
            "jpl.geometric.URANUS.distance_km",
            "jpl.geometric.NEPTUNE.distance_km",
        ],
        fixtures: &[],
        classes: &["builtin-compact", "builtin-standard", "builtin-full"],
        why: "VSOP87's radius vectors as published part from DE440's by 760 km at Mars, \
              540 at Jupiter and 720 at Saturn, past the full tier's band of 150, and by \
              28 000 km at Uranus and 14 000 at Neptune, past every tier's; the same at \
              every tier, so it is the theory and not its truncation",
    },
    Divergence {
        name: "pluto-table-truncation",
        fields: &["jpl.geometric.PLUTO.distance_km"],
        fixtures: &[],
        classes: &["builtin-compact", "builtin-standard"],
        why: "the compact and standard tiers' Pluto tables carry its distance to 690 000 \
              and 18 000 km, against 120 km at the full tier; its direction stays inside \
              each tier's band",
    },
    Divergence {
        name: "full-tier-theory-drift",
        fields: &[
            "jpl.geometric.MOON.direction_deg",
            "jpl.geometric.MARS.direction_deg",
            "jpl.geometric.outer.URANUS.direction_deg",
            "jpl.geometric.outer.NEPTUNE.direction_deg",
        ],
        fixtures: &[],
        classes: &["builtin-full"],
        why: "ELP2000-82B and VSOP87 drift from DE440 away from 2000: the Moon and Mars \
              stay under 0.7 arcseconds from 1800 to 2100 and reach 3.0 and 3.2 by 2400, \
              Uranus 4.8 and Neptune 6.8 (Neptune is past 2 already at 2000); the standard \
              tier misses by the same, so the theories as published, not the tier's \
              truncation, are what part from the arcsecond its class promises",
    },
];

#[cfg(test)]
mod tests {
    use super::{TARGETS, path, separation_deg, tdb_minus_tt};

    #[test]
    fn tdb_runs_ahead_of_tt_by_under_two_milliseconds_with_the_year() {
        let mut most: f64 = 0.0;
        for day in 0..366 {
            let offset = tdb_minus_tt(2_451_545.0 + f64::from(day)) * 86_400.0;
            most = most.max(offset.abs());
        }
        assert!((0.001_6..0.001_7).contains(&most), "{most}");
        // A quarter-orbit past perihelion's anomaly, TDB is ahead by the most.
        let ahead = tdb_minus_tt(2_451_545.0 + 94.0) * 86_400.0;
        assert!((0.001_6..0.001_7).contains(&ahead), "{ahead}");
    }

    #[test]
    fn a_separation_keeps_its_precision_at_a_milliarcsecond() {
        let small = 1e-6_f64.to_radians();
        let found = separation_deg([1.0, 0.0, 0.0], [small.cos(), small.sin(), 0.0]);
        assert!((found - 1e-6).abs() < 1e-15, "{found}");
        assert!((separation_deg([1.0, 0.0, 0.0], [0.0, 0.0, 2.0]) - 90.0).abs() < 1e-12);
    }

    #[test]
    fn an_outer_planets_direction_is_filed_under_the_outer_band() {
        for (_, body, outer) in TARGETS {
            let direction = path(body, outer, "direction_deg");
            assert_eq!(direction.contains(".outer."), outer, "{direction}");
            assert!(!path(body, outer, "distance_km").contains(".outer."));
        }
    }
}

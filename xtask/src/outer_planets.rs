//! The outer planets, measured (`western-outer-planets.md`, C239): the
//! SDK's Uranus, Neptune and Pluto held to what the corpus recorded for
//! each of its births, and the parallax C239's choice of centre moves
//! measured rather than argued.
//!
//! The corpus records the three under `positions.outer`, from the Earth's
//! centre even under its topocentric settings. So the ephemeris is held
//! by founding each birth geocentrically, and the chart's own frame by
//! the difference between that and the topocentric chart the profile
//! founds.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::{Context, Ephemeris, GrahaPosition};

use crate::births::{Birth, CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, arcsec, count, fill, median, table, verdict_of, worst};
use crate::rules_corpus::read_json;

const PAGE: &str = "docs/03-design/outer-planets-measured.md";

/// Each outer planet, its key in the recording, and the bound its
/// longitude is held to, arcseconds: what the built-in ephemeris can
/// promise, built from its parts rather than read off a sample.
///
/// - Uranus and Neptune: the theory's own floor, the heliocentric worst
///   against a modern ephemeris (4.73″ and 6.57″,
///   `builtin-ephemeris-measured.md`), seen from the Earth at most
///   `r / (r − 1)` larger, at opposition (1.058 for Uranus at 18.3 AU,
///   1.035 for Neptune at 29.8), and then the `standard` tier's
///   arcsecond of truncation: 6.0″ and 7.8″, held to 6 and 8.
/// - Pluto: the `standard` tier's fit is held to an arcsecond
///   (`pluto-measured.md`), and has no theory beneath it.
const BODIES: [(Graha, &str, f64); 3] = [
    (Graha::Uranus, "URANUS", 6.0),
    (Graha::Neptune, "NEPTUNE", 8.0),
    (Graha::Pluto, "PLUTO", 1.0),
];

/// The most the observer's place can move a planet past Saturn: the
/// Earth's radius seen from Uranus at its nearest, 17.3 AU, is 0.51″
/// of parallax, and the observer's own speed adds at most 0.32″ of
/// diurnal aberration at the equator.
const PARALLAX_BOUND: f64 = 1.0;

/// How far apart two longitudes are, arcseconds, the shorter way round.
fn apart_arcsec(one: f64, other: f64) -> f64 {
    let gap = (one - other).rem_euclid(360.0);
    gap.min(360.0 - gap) * 3600.0
}

/// What the recording says of one outer planet: its sidereal longitude,
/// latitude and speed.
struct Recorded {
    longitude_deg: f64,
    latitude_deg: f64,
    speed_deg_per_day: f64,
}

fn recorded(root: &Path, birth: &Birth, key: &str) -> Result<Recorded, String> {
    let file = read_json(&root.join(CHARTS).join(format!("{}.json", birth.name)))?;
    let at = &file["positions"]["outer"][key];
    let number = |field: &str| {
        at[field]
            .as_f64()
            .ok_or_else(|| format!("{}: no recorded {key} {field}", birth.name))
    };
    if at["frame"] != "geocentric" {
        return Err(format!(
            "{}: {key} is recorded {}, where every chart so far records the outer planets geocentric (C239)",
            birth.name, at["frame"]
        ));
    }
    Ok(Recorded {
        longitude_deg: number("sidereal_longitude_deg")?,
        latitude_deg: number("latitude_deg")?,
        speed_deg_per_day: number("speed_deg_per_day")?,
    })
}

/// The outer planets a context places for a birth, in `BODIES`' order.
fn placed(sdk: &Context, birth: &Birth) -> Result<Vec<GrahaPosition>, String> {
    let document = sdk
        .chart()
        .reading(
            birth.document.foundation.instant,
            &birth.request().with_outer_planets(),
        )
        .map_err(|why| format!("{}: {why}", birth.name))?
        .value;
    BODIES
        .iter()
        .map(|(graha, key, _)| {
            document
                .foundation
                .graha(*graha)
                .copied()
                .ok_or_else(|| format!("{}: asked for {key} and not placed", birth.name))
        })
        .collect()
}

/// One body's measurements over the births.
#[derive(Default)]
struct Measured {
    longitude: Vec<f64>,
    latitude: Vec<f64>,
    speed: Vec<f64>,
    parallax: Vec<f64>,
}

/// Each body's measurements over the births: against the recording from
/// the Earth's centre, and from the observer against the Earth's centre.
fn measure(root: &Path, sdk: &Context, births: &[Birth]) -> Result<Vec<Measured>, String> {
    let geocentric = Context::builder()
        .profile("conformance-baseline")
        .settings_json(r#"{"frame": {"centre": "GEOCENTRIC"}}"#)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the geocentric conformance profile: {why}"))?;
    let mut measured: Vec<Measured> = BODIES.iter().map(|_| Measured::default()).collect();
    for birth in births {
        let topo = placed(sdk, birth)?;
        let geo = placed(&geocentric, birth)?;
        for (index, (_, key, _)) in BODIES.iter().enumerate() {
            let want = recorded(root, birth, key)?;
            let (Some(one), Some(topo), Some(geo)) =
                (measured.get_mut(index), topo.get(index), geo.get(index))
            else {
                return Err(format!("{}: {key} missing", birth.name));
            };
            one.longitude
                .push(apart_arcsec(geo.longitude_deg, want.longitude_deg));
            one.latitude
                .push((geo.latitude_deg - want.latitude_deg).abs() * 3600.0);
            one.speed
                .push((geo.speed_deg_per_day - want.speed_deg_per_day).abs() * 3600.0);
            one.parallax
                .push(apart_arcsec(topo.longitude_deg, geo.longitude_deg));
        }
    }
    Ok(measured)
}

fn claims(measured: &[Measured]) -> Vec<Claim> {
    let held = BODIES.iter().zip(measured).map(|((_, key, bound), one)| {
        let worst_deg = worst(one.longitude.iter().copied());
        Claim::stated(
            format!(
                "{key}'s longitude, placed from the Earth's centre, is the recording's within {}″",
                arcsec(*bound)
            ),
            verdict_of(worst_deg <= *bound),
            format!(
                "worst {}″ over {}",
                arcsec(worst_deg),
                count(one.longitude.len())
            ),
        )
    });
    let moved = BODIES.iter().zip(measured).map(|((_, key, _), one)| {
        let worst_parallax = worst(one.parallax.iter().copied());
        Claim::stated(
            format!(
                "C239: placing {key} from the observer moves it under {}″",
                arcsec(PARALLAX_BOUND)
            ),
            verdict_of(worst_parallax < PARALLAX_BOUND),
            format!("worst {}″", arcsec(worst_parallax)),
        )
    });
    held.chain(moved).collect()
}

/// Each body's worst and median, a row a body.
fn differences(measured: &[Measured]) -> String {
    let mut out = String::from(
        "| body | longitude | latitude | speed | from the observer |\n|---|---|---|---|---|\n",
    );
    let both = |values: &[f64]| {
        format!(
            "{} / {}",
            arcsec(worst(values.iter().copied())),
            arcsec(median(values.iter().copied()))
        )
    };
    for ((_, key, _), one) in BODIES.iter().zip(measured) {
        let _ = writeln!(
            out,
            "| {key} | {} | {} | {} | {} |",
            both(&one.longitude),
            both(&one.latitude),
            both(&one.speed),
            both(&one.parallax)
        );
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = conformance()?;
    let births = births(root, &sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    let measured = measure(root, &sdk, &births)?;
    let mut out = String::from(
        "# The outer planets, measured\n\n\
         Status: `generated` by `cargo xtask outer-planets` from the \
         corpus's recorded births, 2026-10-03. Do not edit: \
         `check-outer-planets` regenerates this page and fails on any \
         difference.\n\n\
         A chart asked `with_outer_planets` places Uranus, Neptune and \
         Pluto beside the nine (`western-outer-planets.md`). The corpus \
         records the three under `positions.outer`, from the Earth's \
         centre even under its topocentric settings. ",
    );
    let _ = write!(
        out,
        "This page founds each of its {} births under \
         `conformance-baseline` twice: geocentrically, to hold the \
         ephemeris to the recording, and as the profile founds it, \
         topocentrically, to measure what C239's choice of centre moves. \
         Each longitude bound is what the built-in ephemeris can promise \
         for the body: the theory's own floor seen from the Earth, and the \
         `standard` tier's arcsecond of truncation.\n\n",
        count(births.len())
    );
    out.push_str(&table(&claims(&measured)));
    out.push_str(
        "\n## Against the recording, from the Earth's centre\n\n\
         The worst and the median difference over the births, in \
         arcseconds; the speed's in arcseconds a day.\n\n",
    );
    out.push_str(&differences(&measured));
    out.push_str(
        "\n## What it means\n\n\
         The first three rows hold the ephemeris: what the chart places \
         from the Earth's centre is what the recording engine placed, \
         inside the bound the built-in ephemeris can promise for each body. \
         The next three are C239's size. The SDK places the outer planets \
         from the chart's own centre, as it places the nine, and the last \
         column is how far that moves each one from the recording's \
         geocentric reading.\n",
    );
    Ok(fill(&out))
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
        Ok(text) => i32::from(
            check(
                root,
                &[Output::new(PAGE, text)],
                "cargo xtask outer-planets",
            ) != 0,
        ),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

//! The modern Western doctrine against the corpus's `immanuel/modern`:
//! immanuel's aspects under its own orbs, its composite charts, its
//! secondary progression's date and its solar returns
//! (`03-design/western-hellenistic-oracle.md`, C46).
//!
//! The aspects, the composite and the progression are fed the tool's own
//! recorded places and instants, so they measure the doctrine and never the
//! ephemeris. A solar return is a search over the sky, so it is the one
//! comparison made on the SDK's own built-in ephemeris, from a radix founded
//! at the fixture's instant and place, within the bound the two ephemerides'
//! Suns allow.
//!
//! immanuel is rank 3 (AGPL-3.0, run as a black box: its outputs are
//! recorded facts, its code is never read), a witness and never a
//! correction.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index JSON by key and print what they skip"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[path = "../../core/tests/support/conformance.rs"]
mod conformance;

use conformance::{Tally, carried};

use serde_json::Value;
use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::western::{
    AspectOrb, AspectRequest, ChartPoints, OrbModel, Placed, Progression, TROPICAL_YEAR_DAYS,
    WesternAspect, aspects, composite,
};
use teistro::{ChartRequest, Context, Ephemeris, Status, UtcOffset};

const SECTION: &str = "immanuel/modern";

/// Agreement on a quantity the SDK computes by the same arithmetic from the
/// same recorded values: rounding only.
const SAME_DEG: f64 = 1e-9;

/// A composite planet or midheaven, degrees and degrees a day. The tool
/// computes the native's chart afresh for a composite, and for c048
/// (30 December 2399) its Moon there is not the Moon it recorded for the
/// birth: the composite Moon stands 2.15e-8° off the midpoint of the two
/// recorded Moons, and its speed 1.1e-8° a day, a fraction of a
/// millisecond of the Moon's motion. Every other planet of every chart is
/// the midpoint to 3e-13°.
const COMPOSITE_DEG: f64 = 1e-7;

/// A progressed instant, days: the same arithmetic, rounding only.
const SAME_DAYS: f64 = 1e-9;

/// A solar return, seconds: the one comparison over the SDK's own
/// ephemeris, searched in universal time. The two agree within 5 s for
/// all but twelve returns, each thirty years after a birth: 5.4 to 12.5 s
/// for the returns of 2049 to 2054 and 33.9 s for c046's of 2380. Only
/// returns ahead of the present part, as ΔT, the clock the Sun is
/// computed on, is extrapolated differently by two ephemerides; a second
/// of ΔT moves a return a second. That cause is inferred, not measured.
/// Two returns past 2399 lie beyond the built-in ephemeris and are not
/// compared.
const RETURN_SECONDS: f64 = 60.0;

/// The cruxes a miss is counted against.
const COMPOSITE_LAGNA: &str =
    "the composite lagna kept at the near midpoint (western-composites.md, C247)";
const YEAR: &str = "the tool's tropical year of the birth's epoch (western-progressions.md, C236)";
const RETROGRADE: &str =
    "a retrograde planet's aspect read as applying (western-hellenistic-oracle.md, C792)";

/// The tool's aspects by its own name, each with the orb its default
/// configuration gives two planets (the manifest's `planet_orbs_deg`).
const ASPECTS: [(&str, WesternAspect, f64); 6] = [
    ("Conjunction", WesternAspect::Conjunction, 10.0),
    ("Opposition", WesternAspect::Opposition, 10.0),
    ("Square", WesternAspect::Square, 10.0),
    ("Trine", WesternAspect::Trine, 10.0),
    ("Sextile", WesternAspect::Sextile, 6.0),
    ("Quincunx", WesternAspect::Quincunx, 3.0),
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/immanuel")
}

fn read(path: &Path) -> Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("{}: {err}; the corpus lists {SECTION}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// Every file the section's manifest lists, each read; a listed file that
/// is missing fails rather than reading as a skip.
fn files() -> (Value, Vec<(String, Value)>) {
    let manifest = read(&root().join("modern/manifest.json"));
    let files = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let file = entry["file"].as_str().unwrap();
            (file.to_owned(), read(&root().join(file)))
        })
        .collect();
    (manifest, files)
}

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap()
}

fn graha(key: &str) -> Graha {
    Graha::from_key(key).unwrap_or_else(|| panic!("{key} is not a graha"))
}

fn day(jd: f64) -> JulianDay<Utc> {
    JulianDay::<Utc>::try_new(jd).unwrap()
}

/// The arc between two longitudes, either way round.
fn apart(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// The ten planets the tool placed, as the SDK takes them.
fn placed(planets: &Value) -> Vec<Placed> {
    planets
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, at)| {
            Placed::new(
                graha(key),
                number(&at["longitude_deg"]),
                number(&at["speed_deg_per_day"]),
            )
        })
        .collect()
}

fn points(chart: &Value) -> ChartPoints {
    ChartPoints {
        planets: placed(&chart["planets"]),
        lagna_deg: number(&chart["ascendant_deg"]),
        midheaven_deg: number(&chart["midheaven_deg"]),
        cusps_deg: None,
    }
}

/// The tool's default orbs between two planets: one orb an aspect.
fn request() -> AspectRequest {
    AspectRequest::default()
        .with_aspects(ASPECTS.iter().map(|(_, aspect, _)| *aspect))
        .with_orbs(OrbModel::ByAspect {
            orbs: ASPECTS
                .iter()
                .map(|(_, aspect, orb_deg)| AspectOrb {
                    aspect: *aspect,
                    orb_deg: *orb_deg,
                })
                .collect(),
        })
}

fn pair(a: Graha, b: Graha) -> (u8, u8) {
    let (a, b) = (a as u8, b as u8);
    (a.min(b), a.max(b))
}

/// The tool's aspects against the SDK's, row by row both ways.
fn score_aspects(name: &str, file: &Value, tally: &mut Tally) {
    let rows = aspects(&placed(&file["natal"]["planets"]), &request()).unwrap();
    let mut ours: BTreeMap<((u8, u8), WesternAspect), _> = rows
        .iter()
        .map(|row| ((pair(row.first, row.second), row.aspect), row))
        .collect();
    for tool in file["aspects"].as_array().unwrap() {
        let kind = tool["type"].as_str().unwrap();
        let (_, aspect, orb) = ASPECTS
            .iter()
            .find(|(named, ..)| *named == kind)
            .unwrap_or_else(|| panic!("{name}: the tool's {kind} is not one of its defaults"));
        tally.within(number(&tool["orb_deg"]), *orb, 0.0, || {
            format!("{name}: the tool's orb for a {kind}")
        });
        let key = (
            pair(
                graha(tool["active"].as_str().unwrap()),
                graha(tool["passive"].as_str().unwrap()),
            ),
            *aspect,
        );
        let Some(row) = ours.remove(&key) else {
            panic!("{name}: the tool holds {tool} and the SDK does not");
        };
        tally.within(
            row.from_exact_deg,
            number(&tool["difference_deg"]).abs(),
            SAME_DEG,
            || format!("{name}: {kind} {key:?}, degrees from exact"),
        );
        // Within its exact orb the tool calls an aspect exact and neither
        // applying nor separating, so only the others are compared.
        if tool["exact"].as_bool().unwrap() {
            continue;
        }
        let applicative = tool["applicative"].as_bool().unwrap();
        let active = tool["active"].as_str().unwrap();
        let retrograde = number(&file["natal"]["planets"][active]["speed_deg_per_day"]) < 0.0;
        if row.applying != applicative && retrograde && applicative {
            // The tool calls every aspect whose faster planet is retrograde
            // applying, whichever way the separation moves; the SDK reads
            // the separation closing on the angle (C792).
            tally.explained(RETROGRADE, 1);
        } else {
            tally.same(row.applying, applicative, || {
                format!("{name}: {kind} {key:?}, applying")
            });
        }
    }
    assert!(
        ours.is_empty(),
        "{name}: the SDK holds {:?} and the tool does not",
        ours.keys().collect::<Vec<_>>()
    );
}

/// The tool's composite of this chart and the next against the SDK's, from
/// the two natal charts the tool recorded.
fn score_composite(name: &str, file: &Value, tally: &mut Tally) {
    let recorded = &file["composite"];
    let native = points(&file["natal"]);
    let partner = points(&recorded["partner_natal"]);
    let ours = composite(&native, &partner).unwrap();
    for one in &ours.planets {
        let tool = &recorded["planets"][one.graha.key()];
        tally.within(
            apart(one.longitude_deg, number(&tool["longitude_deg"])),
            0.0,
            COMPOSITE_DEG,
            || format!("{name}: the composite {:?}", one.graha),
        );
        tally.within(
            one.speed_deg_per_day,
            number(&tool["speed_deg_per_day"]),
            COMPOSITE_DEG,
            || format!("{name}: the composite {:?}'s speed", one.graha),
        );
    }
    tally.within(
        apart(ours.midheaven_deg, number(&recorded["midheaven_deg"])),
        0.0,
        COMPOSITE_DEG,
        || format!("{name}: the composite midheaven"),
    );
    let lagna = number(&recorded["ascendant_deg"]);
    if apart(ours.lagna_deg, lagna) <= SAME_DEG {
        tally.agreed(1);
    } else {
        // The SDK turns a lagna more than 90° from the midheaven's eastern
        // quadrature by half a circle (C247); the tool keeps the near
        // midpoint, which the SDK's unturned lagna is.
        assert!(
            ours.lagna_turned && apart(ours.lagna_deg + 180.0, lagna) <= SAME_DEG,
            "{name}: the composite lagna {} against {lagna}",
            ours.lagna_deg
        );
        tally.explained(COMPOSITE_LAGNA, 1);
    }
}

/// The tool's secondary progression: the instant of the sky that measures
/// each recorded age, a day for a tropical year.
///
/// The SDK's year is the constant 365.242 189 days. The tool's, read back
/// from its own instants, is a tropical year of the birth's epoch, from
/// 365.242 203 days for a birth in 1800 to 365.242 166 in 2399, which its
/// manifest's constant (365.242 198 93) is not: under a third of a second
/// at sixty years, and counted against the year measure (C236).
fn score_progressions(name: &str, file: &Value, tally: &mut Tally) {
    let birth = day(number(&file["inputs"]["jd_ut"]));
    for one in file["progressions"].as_array().unwrap() {
        let life = day(number(&one["progression_jd_ut"]));
        let sky = Progression::SECONDARY.sky_at(birth, life).unwrap();
        let recorded = number(&one["progressed_jd_ut"]);
        if (sky.get() - recorded).abs() <= SAME_DAYS {
            tally.agreed(1);
            continue;
        }
        let tool_year = (life.get() - birth.get()) / (recorded - birth.get());
        assert!(
            (tool_year - TROPICAL_YEAR_DAYS).abs() < 5e-5,
            "{name}: the progressed instant at {}, {} against {recorded}, a year of {tool_year} days",
            one["age_julian_years"],
            sky.get()
        );
        tally.explained(YEAR, 1);
    }
}

/// The tool's solar returns against the SDK's, searched on the built-in
/// ephemeris from a radix founded at the fixture's instant and place.
fn score_returns(
    sdk: &Context,
    name: &str,
    file: &Value,
    tally: &mut Tally,
    beyond: &mut usize,
) -> f64 {
    let inputs = &file["inputs"];
    let place = Place::new(
        Latitude::literal(number(&inputs["latitude_deg"])),
        Longitude::literal(number(&inputs["longitude_deg"])),
        Altitude::literal(0.0),
    );
    let radix = sdk
        .chart()
        .reading(
            day(number(&inputs["jd_ut"])),
            &ChartRequest::at(place, UtcOffset::UTC),
        )
        .unwrap()
        .value;
    let mut worst: f64 = 0.0;
    for one in file["solar_returns"].as_array().unwrap() {
        let recorded = number(&one["jd_ut"]);
        let found = match sdk.chart().returns(
            &radix,
            day(recorded - 5.0),
            day(recorded + 5.0),
            [Graha::Sun],
        ) {
            Ok(found) => found.value,
            // A return past the built-in ephemeris's last day (2399) is
            // not compared: the tool's own ephemeris reaches further.
            Err(error) if error.status == Status::OutOfRange => {
                *beyond += 1;
                continue;
            }
            Err(error) => panic!("{name}: the solar return near {recorded}: {error:?}"),
        };
        tally.same(found.len(), 1, || {
            format!("{name}: one solar return near {recorded}")
        });
        let seconds = (found[0].at.get() - recorded).abs() * 86_400.0;
        worst = worst.max(seconds);
        tally.within(seconds, 0.0, RETURN_SECONDS, || {
            format!("{name}: the solar return of {}", one["year"])
        });
    }
    worst
}

#[test]
fn immanuel_aspects_composites_progressions_and_returns() {
    if let Err(skip) = carried(SECTION) {
        println!("{skip}");
        return;
    }
    let (manifest, files) = files();
    // The orbs the scorer gives the SDK are the ones the tool recorded.
    let orbs = &manifest["config"]["planet_orbs_deg"];
    for (_, aspect, orb) in ASPECTS {
        let key = format!("{:.1}", aspect.degrees());
        assert_eq!(number(&orbs[key.as_str()]), orb, "the tool's orb for {key}");
    }
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .profile("western-tropical-default")
        .build()
        .unwrap();
    let mut tally = Tally::new(
        SECTION,
        "immanuel 1.6.0's recorded places fed to the SDK's aspects, composite and progression; its solar returns against the built-in ephemeris",
    );
    let mut worst: f64 = 0.0;
    let mut beyond = 0;
    for (name, file) in &files {
        score_aspects(name, file, &mut tally);
        score_composite(name, file, &mut tally);
        score_progressions(name, file, &mut tally);
        worst = worst.max(score_returns(&sdk, name, file, &mut tally, &mut beyond));
    }
    println!(
        "the furthest solar return: {worst:.1} s; {beyond} past the built-in ephemeris, not compared"
    );
    assert_eq!(
        files.len(),
        usize::try_from(manifest["counts"]["files"].as_u64().unwrap()).unwrap()
    );
    tally.record();
}

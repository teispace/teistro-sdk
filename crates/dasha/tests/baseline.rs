//! The Vimshottari dasha against every recorded one: the balance and its
//! written form, every period of every recorded tree, the children sampled
//! at deeper paths, and the chains running at two instants, over the 93
//! dashas of the conformance corpus (`docs/03-design/dasha-measured.md`).
//!
//! The recorded Moon longitude and nakshatra span are the inputs, so what
//! is compared is the dasha and not the astronomy. The bound is far inside
//! the corpus's own tolerance of a thousandth of a day: the recording
//! engine's boundaries are not reproducible to the bit, and agree with this
//! crate's exact shares to well under a millisecond.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::too_many_lines,
    reason = "tests fail by panicking, read fixtures, walk one corpus each and print the measurement under --nocapture"
)]

use std::path::Path as FsPath;

use serde_json::Value;
use teistro_core::angle::Nas;
use teistro_core::interval::Interval;
use teistro_core::quantity::{Degrees, Depth, JulianDay};
use teistro_core::settings::{AfterCycle, Balance, BirthPeriod, SeedOverflow, YearLength};
use teistro_dasha::{Birth, Dasha, Period, Rules, VIMSHOTTARI};

/// A boundary's bound, in days: a tenth of a millisecond above the worst
/// the measurement found, and ten thousand times inside the corpus's
/// tolerance.
const BOUND_DAYS: f64 = 1e-8;

/// A remaining fraction's bound: the recorded longitude is the input, so
/// only the classification's integer arithmetic stands between them.
const BOUND_FRACTION: f64 = 1e-12;

fn records() -> Vec<(String, Value)> {
    let root = FsPath::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline");
    let mut found = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap_or_else(|err| panic!("{dir}: {err}; is the fixtures submodule checked out?"))
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.sort();
        for path in paths {
            let json: Value =
                serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            if !json["dashas"].is_null() {
                found.push((
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    json,
                ));
            }
        }
    }
    found
}

fn jd(value: &Value) -> f64 {
    value.as_f64().unwrap()
}

/// The dasha a recorded method describes.
fn dasha(json: &Value, method: &str, recorded: &Value) -> Dasha {
    let dashas = &json["dashas"];
    let degrees = dashas["moon_sidereal_longitude_deg"].as_f64().unwrap();
    let birth = Birth {
        instant: JulianDay::literal(jd(&json["input"]["resolved"]["jd_ut"])),
        moon: Nas::from_degrees(Degrees::try_new(degrees).unwrap()),
        moon_span: recorded["nakshatra_span"]
            .as_object()
            .map(|span| Interval::literal(jd(&span["entry_jd"]), jd(&span["exit_jd"]))),
    };
    assert_eq!(
        dashas["year_length_days"].as_f64(),
        Some(YearLength::Julian36525.days())
    );
    let rules = Rules {
        balance: if method == "temporal" {
            Balance::Temporal
        } else {
            Balance::Spatial
        },
        year_length: YearLength::Julian36525,
        birth_period: BirthPeriod::Compressed,
        after_cycle: AfterCycle::End,
        seed_overflow: SeedOverflow::WrapToStart,
    };
    Dasha::new(&VIMSHOTTARI, &birth, rules).unwrap()
}

/// The recorded `[path, lord, start, end]` row a period should be.
fn agrees(period: &Period, path: &str, lord: &str, start: f64, end: f64, worst: &mut f64) -> bool {
    *worst = worst
        .max((period.interval.from.get() - start).abs())
        .max((period.interval.to.get() - end).abs());
    period.path.to_string() == path && period.lord.key() == lord
}

/// Every period of the birth cycle to `depth`, depth first.
fn tree(dasha: &Dasha, depth: usize) -> Vec<Period> {
    fn walk(dasha: &Dasha, period: Period, depth: usize, out: &mut Vec<Period>) {
        out.push(period);
        if period.path.depth() < depth {
            for child in dasha.children(&period) {
                walk(dasha, child, depth, out);
            }
        }
    }
    let mut out = Vec::new();
    for maha in dasha.mahadashas() {
        walk(dasha, maha, depth, &mut out);
    }
    out
}

#[test]
fn every_recorded_vimshottari_is_reproduced() {
    let records = records();
    assert_eq!(records.len(), 93, "the corpus's dashas");
    let (mut methods, mut rows, mut children, mut chains, mut past_end) = (0, 0, 0, 0, 0);
    let mut worst = 0.0_f64;
    for (name, json) in &records {
        for (method, recorded) in json["dashas"]["methods"].as_object().unwrap() {
            methods += 1;
            let dasha = dasha(json, method, recorded);
            let at = format!("{name} {method}");

            // The balance, and how it is written.
            let balance = dasha.balance();
            let remaining = recorded["remaining_fraction"].as_f64().unwrap();
            assert!(
                (balance.remaining - remaining).abs() < BOUND_FRACTION,
                "{at}: remaining {} against {remaining}",
                balance.remaining
            );
            let written = &recorded["balance"];
            assert!(
                (balance.days - written["total_days"].as_f64().unwrap()).abs() < BOUND_DAYS,
                "{at}: balance days"
            );
            let parts = [
                written["years"].as_u64(),
                written["months"].as_u64(),
                written["days"].as_u64(),
                written["hours"].as_u64(),
                written["minutes"].as_u64(),
            ];
            let ours = [
                balance.written.years.into(),
                balance.written.months.into(),
                balance.written.days.into(),
                balance.written.hours.into(),
                balance.written.minutes.into(),
            ]
            .map(Some);
            assert_eq!(ours, parts, "{at}: the written balance");

            // Every period of the recorded tree, in its order.
            let depth = usize::try_from(recorded["tree_depth"].as_u64().unwrap()).unwrap();
            let periods = tree(&dasha, depth);
            let recorded_rows = recorded["periods"].as_array().unwrap();
            assert_eq!(periods.len(), recorded_rows.len(), "{at}: rows");
            for (period, row) in periods.iter().zip(recorded_rows) {
                rows += 1;
                assert!(
                    agrees(
                        period,
                        row[0].as_str().unwrap(),
                        row[1].as_str().unwrap(),
                        jd(&row[2]),
                        jd(&row[3]),
                        &mut worst
                    ),
                    "{at}: {period:?} against {row}"
                );
            }

            // The children the corpus samples at deeper paths.
            for (path, kids) in recorded["children_at_path"].as_object().unwrap() {
                let mut period = dasha
                    .mahadasha(0, path.split('/').next().unwrap().parse().unwrap())
                    .unwrap();
                for step in path.split('/').skip(1) {
                    period = dasha.child(&period, step.parse().unwrap()).unwrap();
                }
                let ours: Vec<Period> = dasha.children(&period).collect();
                let kids = kids.as_array().unwrap();
                assert_eq!(ours.len(), kids.len(), "{at}: children of {path}");
                for (child, kid) in ours.iter().zip(kids) {
                    children += 1;
                    let index = kid[0].as_u64().unwrap().to_string();
                    let expected_path = format!("{path}/{index}");
                    assert!(
                        agrees(
                            child,
                            &expected_path,
                            kid[1].as_str().unwrap(),
                            jd(&kid[2]),
                            jd(&kid[3]),
                            &mut worst
                        ),
                        "{at}: child {expected_path}"
                    );
                }
            }

            // The chains running at the recorded instants.
            for active in recorded["active_at"].as_array().unwrap() {
                chains += 1;
                let instant = JulianDay::literal(jd(&active["jd"]));
                let recorded_chain = active["chain"].as_array().unwrap();
                let chain = dasha.at(instant, Depth::try_new(5).unwrap());
                if recorded_chain.is_empty() {
                    past_end += 1;
                    assert!(chain.is_empty(), "{at}: the cycle had ended");
                    continue;
                }
                assert_eq!(chain.len(), recorded_chain.len(), "{at}: chain depth");
                for (period, link) in chain.iter().zip(recorded_chain) {
                    assert_eq!(
                        u64::from(*period.path.indices().last().unwrap()),
                        link["index"].as_u64().unwrap(),
                        "{at}: index at level {}",
                        link["level"]
                    );
                    worst = worst
                        .max((period.interval.from.get() - jd(&link["start_jd"])).abs())
                        .max((period.interval.to.get() - jd(&link["end_jd"])).abs());
                    assert_eq!(period.lord.key(), link["lord"].as_str().unwrap(), "{at}");
                }
            }
        }
    }
    println!(
        "{methods} methods, {rows} rows, {children} sampled children, {chains} chains ({past_end} past the cycle): worst boundary {worst:.2e} days"
    );
    assert_eq!(methods, 148);
    assert!(worst < BOUND_DAYS, "worst boundary {worst:e} days");
}

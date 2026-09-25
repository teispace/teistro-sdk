//! Every row beside Vimshottari against every recorded answer: the first
//! lord, the balance and its written form, every period of the recorded
//! tree and the chains running at two instants, over the corpus's
//! `baseline/dasha-systems` (`docs/03-design/dasha-systems-measured.md`).
//!
//! Each file was computed from its fixture's recorded Moon, birth and
//! nakshatra span, and those are this test's inputs too, so what is
//! compared is the dasha and not the astronomy. A row's own rule data is
//! compared with the data the corpus states, so a row edited by hand fails
//! here by the field it changed as well as by the answers it moved.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::too_many_lines,
    reason = "tests fail by panicking, read fixtures, walk one corpus each and print the measurement under --nocapture"
)]

mod common;

use common::{BOUND_DAYS, jd, tree};
use serde_json::Value;
use teistro_core::angle::Nas;
use teistro_core::catalogue::DashaSystem;
use teistro_core::interval::Interval;
use teistro_core::quantity::{Degrees, Depth, JulianDay};
use teistro_core::settings::{AfterCycle, Balance, BirthPeriod, SeedOverflow, YearLength};
use teistro_dasha::{Birth, Count, Dasha, DashaName, ROWS, Rules, Timeline, UduRow, row};

/// The row the corpus's key names, held to the rule data it states.
fn the_row(key: &str, stated: &Value) -> &'static UduRow {
    let system = DashaSystem::from_key(&key.to_ascii_uppercase()).expect("a catalogue system");
    // The corpus records the recording engine's rows, Ashtottari three each.
    let row = row(
        system,
        teistro_core::settings::AshtottariGrouping::ThreeEach,
    )
    .unwrap_or_else(|| panic!("no row implements {key}"));
    let lords: Vec<(String, u64)> = row
        .lords
        .iter()
        .map(|lord| (lord.graha.key().to_owned(), u64::from(lord.years)))
        .collect();
    let recorded: Vec<(String, u64)> = stated["sequence"]
        .as_array()
        .unwrap()
        .iter()
        .map(|lord| {
            (
                lord["lord"].as_str().unwrap().to_owned(),
                lord["years"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(lords, recorded, "{key}: lords");
    assert_eq!(
        u64::from(row.total_years()),
        stated["total_years"].as_u64().unwrap(),
        "{key}: total_years"
    );
    assert_eq!(
        u64::from(row.reference),
        stated["reference_nakshatra"].as_u64().unwrap(),
        "{key}: reference"
    );
    let count = match row.count {
        Count::FromReference => "from_reference",
        Count::ToReference => "to_reference",
    };
    assert_eq!(count, stated["count"].as_str().unwrap(), "{key}: count");
    assert_eq!(
        u64::from(row.span),
        stated["span"].as_u64().unwrap(),
        "{key}: span"
    );
    assert_eq!(
        u64::from(row.offset),
        stated["offset"].as_u64().unwrap(),
        "{key}: offset"
    );
    let scale = match &stated["scale"] {
        Value::Null => (1, 1, 1),
        scale => (
            scale["numerator"].as_u64().unwrap(),
            scale["denominator"].as_u64().unwrap(),
            scale["rounds"].as_u64().unwrap(),
        ),
    };
    assert_eq!(
        (
            u64::from(row.scale.numerator),
            u64::from(row.scale.denominator),
            u64::from(row.scale.rounds)
        ),
        scale,
        "{key}: scale"
    );
    row
}

#[test]
fn every_other_nakshatra_seeded_system_is_reproduced() {
    let files: Vec<_> = ["charts", "variants"]
        .into_iter()
        .flat_map(|dir| common::files(&format!("dasha-systems/{dir}")))
        .collect();
    assert_eq!(files.len(), 93, "one file for each recorded dasha");
    let (mut answers, mut rows, mut chains, mut past_end) = (0, 0, 0, 0);
    let mut worst = 0.0_f64;
    let mut seen = std::collections::BTreeSet::new();

    for (name, file) in &files {
        let fixture = &common::read(file["fixture"].as_str().unwrap());
        let dashas = &fixture["dashas"];
        let degrees = dashas["moon_sidereal_longitude_deg"].as_f64().unwrap();
        for (key, stated) in file["systems"].as_object().unwrap() {
            let row = the_row(key, stated);
            seen.insert(row.system.clone());
            for (method, recorded) in stated["methods"].as_object().unwrap() {
                answers += 1;
                let at = format!("{name} {key} {method}");
                let birth = Birth {
                    instant: JulianDay::literal(jd(&fixture["input"]["resolved"]["jd_ut"])),
                    moon: Nas::from_degrees(Degrees::try_new(degrees).unwrap()),
                    moon_span: dashas["methods"][method]["nakshatra_span"]
                        .as_object()
                        .map(|span| Interval::literal(jd(&span["entry_jd"]), jd(&span["exit_jd"]))),
                };
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
                    ashtottari_grouping: teistro_core::settings::AshtottariGrouping::ThreeEach,
                };
                assert_eq!(
                    file["year_length_days"].as_f64(),
                    Some(YearLength::Julian36525.days())
                );
                let dasha =
                    Dasha::new(row, &birth, rules).unwrap_or_else(|err| panic!("{at}: {err}"));

                // The first lord and the balance.
                let first = dasha.mahadashas().next().unwrap();
                assert_eq!(
                    first.lord.key(),
                    recorded["first_lord"].as_str().unwrap(),
                    "{at}"
                );
                let balance = dasha.balance();
                let written = &recorded["balance"];
                assert!(
                    (balance.days - jd(&written["total_days"])).abs() < BOUND_DAYS,
                    "{at}: balance {} against {}",
                    balance.days,
                    written["total_days"]
                );
                let parts = [
                    u64::from(balance.written.years),
                    balance.written.months.into(),
                    balance.written.days.into(),
                    balance.written.hours.into(),
                    balance.written.minutes.into(),
                ];
                let recorded_parts = ["years", "months", "days", "hours", "minutes"]
                    .map(|part| written[part].as_u64().unwrap());
                assert_eq!(parts, recorded_parts, "{at}: the written balance");

                // Every period of the recorded tree, in its order.
                let depth = usize::try_from(recorded["tree_depth"].as_u64().unwrap()).unwrap();
                let periods = tree(&dasha, depth);
                let recorded_rows = recorded["periods"].as_array().unwrap();
                assert_eq!(periods.len(), recorded_rows.len(), "{at}: rows");
                for (period, cells) in periods.iter().zip(recorded_rows) {
                    rows += 1;
                    assert_eq!(period.path.to_string(), cells[0].as_str().unwrap(), "{at}");
                    assert_eq!(
                        period.lord.key(),
                        cells[1].as_str().unwrap(),
                        "{at} {}",
                        period.path
                    );
                    worst = worst
                        .max((period.interval.from.get() - jd(&cells[2])).abs())
                        .max((period.interval.to.get() - jd(&cells[3])).abs());
                }

                // The chains running at the recorded instants.
                let chain_depth = u8::try_from(recorded["chain_depth"].as_u64().unwrap()).unwrap();
                for active in recorded["active_at"].as_array().unwrap() {
                    chains += 1;
                    let chain = dasha.at(
                        JulianDay::literal(jd(&active["jd"])),
                        Depth::try_new(chain_depth).unwrap(),
                    );
                    let links = active["chain"].as_array().unwrap();
                    if links.is_empty() {
                        past_end += 1;
                        assert!(chain.is_empty(), "{at}: the cycle had ended");
                        continue;
                    }
                    assert_eq!(chain.len(), links.len(), "{at}: chain depth");
                    for (period, link) in chain.iter().zip(links) {
                        assert_eq!(
                            u64::from(*period.path.indices().last().unwrap()),
                            link[1].as_u64().unwrap(),
                            "{at}: index at level {}",
                            link[0]
                        );
                        assert_eq!(period.lord.key(), link[2].as_str().unwrap(), "{at}");
                        worst = worst
                            .max((period.interval.from.get() - jd(&link[3])).abs())
                            .max((period.interval.to.get() - jd(&link[4])).abs());
                    }
                }
            }
        }
    }
    println!(
        "{answers} answers, {rows} rows, {chains} chains ({past_end} past the cycle): worst boundary {worst:.2e} days"
    );
    assert_eq!(answers, 1184);
    assert!(worst < BOUND_DAYS, "worst boundary {worst:e} days");
    // Every row this build ships is measured here but two: Vimshottari by
    // its own test, and Shashtihayani, which the corpus does not record and
    // the row's tests hold to BPHS ch. 46's worked answers.
    let elsewhere = [DashaSystem::Vimshottari, DashaSystem::Shashtihayani].map(DashaName::from);
    let unmeasured: Vec<_> = ROWS
        .iter()
        .map(|row| row.system.clone())
        .filter(|system| !seen.contains(system))
        .collect();
    assert_eq!(unmeasured, elsewhere);
}

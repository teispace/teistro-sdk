//! Every sign-based row against every recorded answer: every period of the
//! recorded tree, its sign, lord and bounds, and the chains running at two
//! instants, over the corpus's `baseline/rashi-dashas`
//! (`docs/03-design/rashi-dashas-measured.md`).
//!
//! Each file repeats the chart it was computed from, and that chart is this
//! test's input, so what is compared is the dasha rules and not the
//! astronomy.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, read fixtures, and print the measurement under --nocapture"
)]

mod common;

use common::{BOUND_DAYS, jd, tree};
use serde_json::Value;
use teistro_core::catalogue::{DashaSystem, Dignity, Graha, Rashi};
use teistro_core::quantity::{Depth, JulianDay};
use teistro_core::settings::{AfterCycle, YearLength};
use teistro_dasha::{RASHI_ROWS, RashiChart, RashiDasha, Timeline, rashi_row};

const GRAHAS: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

fn sign(value: &Value) -> Rashi {
    Rashi::from_id(u16::try_from(value.as_u64().unwrap()).unwrap()).unwrap()
}

/// The chart a file repeats.
fn chart(inputs: &Value) -> RashiChart {
    RashiChart {
        lagna: sign(&inputs["lagna_sign_index"]),
        arudha_lagna: sign(&inputs["arudha_lagna_sign_index"]),
        navamsa_lagna: sign(&inputs["navamsha_lagna_sign_index"]),
        signs: GRAHAS.map(|graha| sign(&inputs["graha_sign_index"][graha.key()])),
        dignities: GRAHAS.map(|graha| {
            Dignity::from_key(inputs["graha_dignity"][graha.key()].as_str().unwrap()).unwrap()
        }),
    }
}

#[test]
fn every_sign_based_system_is_reproduced() {
    let files: Vec<_> = ["charts", "variants"]
        .into_iter()
        .flat_map(|dir| common::files(&format!("rashi-dashas/{dir}")))
        .collect();
    assert_eq!(files.len(), 77, "one file for each recorded chart");
    let (mut answers, mut rows, mut chains, mut past_end) = (0, 0, 0, 0);
    let mut worst = 0.0_f64;
    let mut seen = std::collections::BTreeSet::new();

    for (name, file) in &files {
        assert_eq!(
            file["year_length_days"].as_f64(),
            Some(YearLength::Julian36525.days())
        );
        let chart = chart(&file["inputs"]);
        let birth = JulianDay::literal(jd(&file["inputs"]["jd_ut"]));
        for (key, recorded) in file["systems"].as_object().unwrap() {
            answers += 1;
            let at = format!("{name} {key}");
            let system =
                DashaSystem::from_key(&key.to_ascii_uppercase()).expect("a catalogue system");
            let row = rashi_row(system).unwrap_or_else(|| panic!("no row implements {key}"));
            seen.insert(system);
            let dasha = RashiDasha::new(
                row,
                &chart,
                birth,
                YearLength::Julian36525,
                AfterCycle::End,
                teistro_dasha::RashiRules::RECORDING_ENGINE,
            )
            .unwrap();

            let depth = usize::try_from(recorded["tree_depth"].as_u64().unwrap()).unwrap();
            let periods = tree(&dasha, depth);
            let recorded_rows = recorded["periods"].as_array().unwrap();
            assert_eq!(periods.len(), recorded_rows.len(), "{at}: rows");
            for (period, cells) in periods.iter().zip(recorded_rows) {
                rows += 1;
                assert_eq!(period.path.to_string(), cells[0].as_str().unwrap(), "{at}");
                assert_eq!(period.sign, Some(sign(&cells[1])), "{at} {}", period.path);
                assert_eq!(
                    period.lord.key(),
                    cells[2].as_str().unwrap(),
                    "{at} {}",
                    period.path
                );
                worst = worst
                    .max((period.interval.from.get() - jd(&cells[3])).abs())
                    .max((period.interval.to.get() - jd(&cells[4])).abs());
            }

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
                    assert_eq!(period.sign, Some(sign(&link[2])), "{at}");
                    assert_eq!(period.lord.key(), link[3].as_str().unwrap(), "{at}");
                    worst = worst
                        .max((period.interval.from.get() - jd(&link[4])).abs())
                        .max((period.interval.to.get() - jd(&link[5])).abs());
                }
            }
        }
    }
    println!(
        "{answers} answers, {rows} rows, {chains} chains ({past_end} past the cycle): worst boundary {worst:.2e} days"
    );
    assert_eq!(answers, 616);
    assert!(worst < BOUND_DAYS, "worst boundary {worst:e} days");
    assert_eq!(
        seen.len(),
        RASHI_ROWS.len(),
        "every row this build ships is measured"
    );
}

/// How many of the recorded answers each of BPHS ch. 46's readings moves
/// (cruxes C51, C53): an answer moves when any mahadasha's sign, lord or end
/// differs from the recording engine's reading of the same chart.
#[test]
fn bphs_readings_move_the_answers_they_reach() {
    use teistro_dasha::RashiRules;
    let (mut by_dual_lord, mut by_start) = (0, 0);
    for (_, file) in ["charts", "variants"]
        .into_iter()
        .flat_map(|dir| common::files(&format!("rashi-dashas/{dir}")))
    {
        let chart = chart(&file["inputs"]);
        let birth = JulianDay::literal(jd(&file["inputs"]["jd_ut"]));
        for key in file["systems"].as_object().unwrap().keys() {
            let system = DashaSystem::from_key(&key.to_ascii_uppercase()).unwrap();
            let row = rashi_row(system).unwrap();
            let under = |rules| {
                let dasha = RashiDasha::new(
                    row,
                    &chart,
                    birth,
                    YearLength::Julian36525,
                    AfterCycle::End,
                    rules,
                )
                .unwrap();
                dasha
                    .mahadashas()
                    .map(|p| (p.sign, p.lord, p.interval.to.get().to_bits()))
                    .collect::<Vec<_>>()
            };
            let engine = RashiRules::RECORDING_ENGINE;
            let recorded = under(engine);
            let dual = RashiRules {
                dual_lord: RashiRules::BPHS.dual_lord,
                ..engine
            };
            let start = RashiRules {
                start: RashiRules::BPHS.start,
                ..engine
            };
            by_dual_lord += usize::from(under(dual) != recorded);
            by_start += usize::from(under(start) != recorded);
        }
    }
    // Counted, so a change to either reading shows: the dual-lord rule reaches
    // every system that counts to or names a stronger lord, and the start the
    // three systems BPHS starts from a stronger sign.
    assert_eq!((by_dual_lord, by_start), (360, 98));
}

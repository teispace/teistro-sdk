//! The Kalachakra against every recorded answer, under the recording
//! engine's readings: the balance, every period of the recorded tree with
//! its sign and lord, and the periods running at two instants, over the
//! corpus's `baseline/kalachakra` (`docs/03-design/kalachakra-measured.md`).
//! The kernel's tables are held to the manifest's too.

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
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Nakshatra, Rashi};
use teistro_core::interval::Interval;
use teistro_core::quantity::{Degrees, Depth, JulianDay};
use teistro_core::settings::{
    AfterCycle, Balance, KalachakraAfterNinth, KalachakraBalance, KalachakraMembership, YearLength,
};
use teistro_dasha::kalachakra::{SIGN_YEARS, pada_row};
use teistro_dasha::{Birth, KalachakraDasha, KalachakraRules, Timeline};

#[test]
fn every_recorded_kalachakra_is_reproduced() {
    let manifest = common::read("kalachakra/manifest.json");
    let years: Vec<u64> = manifest["rules"]["sign_years"]
        .as_array()
        .unwrap()
        .iter()
        .map(|y| y.as_u64().unwrap())
        .collect();
    assert_eq!(years, SIGN_YEARS.map(u64::from), "the signs' years");
    // Every nakshatra's four rows, as the manifest's tables serve them.
    for table in manifest["rules"]["tables"].as_array().unwrap() {
        for n in table["nakshatras"].as_array().unwrap() {
            let nakshatra =
                Nakshatra::from_id(u16::try_from(n.as_u64().unwrap()).unwrap()).unwrap();
            for (pada, signs) in table["padas"].as_array().unwrap().iter().enumerate() {
                let want: Vec<Option<Rashi>> = signs
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|s| Rashi::from_id(u16::try_from(s.as_u64().unwrap()).unwrap()))
                    .collect();
                let ours = pada_row(
                    nakshatra,
                    u8::try_from(pada).unwrap(),
                    KalachakraMembership::Listed,
                );
                assert_eq!(ours.map(Some).to_vec(), want, "{nakshatra:?} pada {pada}");
            }
        }
    }

    let files: Vec<_> = ["charts", "variants"]
        .into_iter()
        .flat_map(|dir| common::files(&format!("kalachakra/{dir}")))
        .collect();
    assert_eq!(files.len(), 93);
    let (mut answers, mut rows, mut chains, mut past_end) = (0, 0, 0, 0);
    let mut worst = 0.0_f64;
    for (name, file) in &files {
        let fixture = common::read(file["fixture"].as_str().unwrap());
        for (method, recorded) in file["methods"].as_object().unwrap() {
            answers += 1;
            let at = format!("{name} {method}");
            let birth = Birth {
                instant: JulianDay::literal(jd(&fixture["input"]["resolved"]["jd_ut"])),
                moon: Nas::from_degrees(
                    Degrees::try_new(jd(&file["moon_sidereal_longitude_deg"])).unwrap(),
                ),
                moon_span: fixture["dashas"]["methods"][method]["nakshatra_span"]
                    .as_object()
                    .map(|span| Interval::literal(jd(&span["entry_jd"]), jd(&span["exit_jd"]))),
            };
            let rules = KalachakraRules {
                balance: if method == "temporal" {
                    Balance::Temporal
                } else {
                    Balance::Spatial
                },
                year_length: YearLength::Julian36525,
                after_cycle: AfterCycle::End,
                membership: KalachakraMembership::Listed,
                balance_of: KalachakraBalance::FirstSign,
                after_ninth: KalachakraAfterNinth::Reverse,
            };
            let dasha =
                KalachakraDasha::new(&birth, rules).unwrap_or_else(|err| panic!("{at}: {err}"));
            assert_eq!(
                u64::from(dasha.pada()),
                file["pada_index"].as_u64().unwrap(),
                "{at}"
            );

            let balance = dasha.balance();
            let written = &recorded["balance"];
            assert!(
                (balance.days - jd(&written["total_days"])).abs() < BOUND_DAYS,
                "{at}: balance"
            );
            let parts = [
                u64::from(balance.written.years),
                balance.written.months.into(),
                balance.written.days.into(),
                balance.written.hours.into(),
                balance.written.minutes.into(),
            ];
            assert_eq!(
                parts,
                ["years", "months", "days", "hours", "minutes"]
                    .map(|p| written[p].as_u64().unwrap()),
                "{at}: the written balance"
            );

            let periods = tree(
                &dasha,
                usize::try_from(recorded["tree_depth"].as_u64().unwrap()).unwrap(),
            );
            let recorded_rows = recorded["periods"].as_array().unwrap();
            assert_eq!(periods.len(), recorded_rows.len(), "{at}: rows");
            for (period, cells) in periods.iter().zip(recorded_rows) {
                rows += 1;
                assert_eq!(period.path.to_string(), cells[0].as_str().unwrap(), "{at}");
                let sign = Rashi::from_id(u16::try_from(cells[1].as_u64().unwrap()).unwrap());
                assert_eq!(period.sign, sign, "{at} {}", period.path);
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

            let depth = u8::try_from(recorded["chain_depth"].as_u64().unwrap()).unwrap();
            for active in recorded["active_at"].as_array().unwrap() {
                chains += 1;
                let chain = dasha.at(
                    JulianDay::literal(jd(&active["jd"])),
                    Depth::try_new(depth).unwrap(),
                );
                let links = active["chain"].as_array().unwrap();
                if links.is_empty() {
                    past_end += 1;
                    assert!(chain.is_empty(), "{at}: the cycles had ended");
                    continue;
                }
                assert_eq!(chain.len(), links.len(), "{at}: chain depth");
                for (period, link) in chain.iter().zip(links) {
                    assert_eq!(
                        u64::from(*period.path.indices().last().unwrap()),
                        link[1].as_u64().unwrap(),
                        "{at}"
                    );
                    worst = worst
                        .max((period.interval.from.get() - jd(&link[4])).abs())
                        .max((period.interval.to.get() - jd(&link[5])).abs());
                }
            }
        }
    }
    println!(
        "{answers} answers, {rows} rows, {chains} chains ({past_end} past the cycles): worst boundary {worst:.2e} days"
    );
    assert_eq!(answers, 148);
    assert!(worst < BOUND_DAYS, "worst boundary {worst:e} days");
}

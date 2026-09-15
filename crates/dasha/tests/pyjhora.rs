//! The Vimshottari kernel held beside `PyJHora`, a rank-3 implementation
//! (`CLEAN_ROOM.md`): the corpus's `pyjhora/vimshottari` records the tool's
//! Moon and its mahadashas and antardashas under each year it offers, run as
//! a black box, and the kernel given the same Moon and year answers the same
//! periods (`03-design/dasha-kernels.md`, "`PyJHora`, beside the kernel").
//!
//! Only the arithmetic is compared: both sides read one Moon, so an ephemeris
//! difference cannot hide in the answer.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use serde_json::Value;
use teistro_core::angle::Nas;
use teistro_core::quantity::{Degrees, JulianDay};
use teistro_core::settings::{AfterCycle, Balance, BirthPeriod, SeedOverflow, YearLength};
use teistro_dasha::{Birth, Dasha, Rules, Timeline, VIMSHOTTARI};

fn corpus() -> std::path::PathBuf {
    common::baseline().join("../pyjhora/vimshottari")
}

#[test]
fn the_kernel_answers_pyjhora_s_vimshottari_from_its_moon() {
    let manifest: Value =
        serde_json::from_str(&std::fs::read_to_string(corpus().join("manifest.json")).unwrap())
            .unwrap();
    let mut worst: std::collections::BTreeMap<&str, f64> = std::collections::BTreeMap::new();
    let (mut compared, mut balances, mut written_agree) = (0, 0, 0);
    for entry in manifest["files"].as_array().unwrap() {
        let name = entry["file"]
            .as_str()
            .unwrap()
            .trim_start_matches("vimshottari/");
        let document: Value =
            serde_json::from_str(&std::fs::read_to_string(corpus().join(name)).unwrap()).unwrap();
        let birth = Birth {
            instant: JulianDay::literal(document["inputs"]["jd_ut"].as_f64().unwrap()),
            moon: Nas::from_degrees(
                Degrees::try_new(document["moon_sidereal_deg"].as_f64().unwrap()).unwrap(),
            ),
            moon_span: None,
        };
        for (duration, year) in [
            ("MEAN_SIDEREAL_YEAR", YearLength::Sidereal),
            ("MEAN_TROPICAL_YEAR", YearLength::Tropical),
            ("MEAN_LUNAR_YEAR", YearLength::Lunar),
            ("SAVANA_YEAR", YearLength::Savana360),
        ] {
            let recorded = &document["durations"][duration];
            // A start drifts by the two years' difference for every year
            // since the cycle began, at most 120 and a mahadasha's worth
            // before birth; the arithmetic itself agrees to a millisecond.
            let year_days = recorded["year_days"].as_f64().unwrap();
            let bound = (year_days - year.days()).abs() * 140.0 + 1e-8;
            let dasha = Dasha::new(
                &VIMSHOTTARI,
                &birth,
                Rules {
                    balance: Balance::Spatial,
                    year_length: year,
                    birth_period: BirthPeriod::Elapsed,
                    after_cycle: AfterCycle::End,
                    seed_overflow: SeedOverflow::WrapToStart,
                },
            )
            .unwrap();
            let periods = recorded["periods"].as_array().unwrap();
            // The tool lists the birth mahadasha's antardashas from its
            // start, and the kernel from the one running at birth; each is
            // found by its mahadasha's place and its two lords.
            let mut matched = 0;
            for (index, maha) in dasha.mahadashas().enumerate() {
                for child in dasha.children(&maha) {
                    let row = periods
                        .iter()
                        .skip(index * 9)
                        .take(9)
                        .find(|row| {
                            row[0].as_str() == Some(maha.lord.key())
                                && row[1].as_str() == Some(child.lord.key())
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "{name} {duration}: {} under {}",
                                child.lord.key(),
                                maha.lord.key()
                            )
                        });
                    let off = (child.interval.from.get().max(birth.instant.get())
                        - row[2].as_f64().unwrap().max(birth.instant.get()))
                    .abs();
                    assert!(
                        off <= bound,
                        "{name} {duration}: {} under {} starts {off} days apart, past {bound}",
                        child.lord.key(),
                        maha.lord.key()
                    );
                    let slot = worst.entry(duration).or_insert(0.0);
                    *slot = slot.max(off);
                    matched += 1;
                    compared += 1;
                }
            }
            assert!(matched > 72, "{name} {duration}: {matched} periods");
            // The balance written in years, months and days: the tool's
            // truncation against the kernel's, where both use the same year.
            let written = dasha.balance().written;
            let theirs: Vec<u64> = recorded["balance_ymd"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_u64().unwrap())
                .collect();
            balances += 1;
            if theirs
                == [
                    u64::from(written.years),
                    u64::from(written.months),
                    u64::from(written.days),
                ]
            {
                written_agree += 1;
            }
        }
    }
    // Where the two years are the same number, the tropical, the arithmetic
    // is the same to the grain of a day's double.
    assert!(worst["MEAN_TROPICAL_YEAR"] < 1e-8, "{worst:?}");
    assert!(compared > 16_000, "{compared}");
    // The written balance is a convention and the two differ in it: under
    // the sidereal and tropical years the tool's day is the one begun, one
    // above the kernel's whole days, and under the lunar and savana years it
    // does not write against the dasha's own year. Counted, so a change to
    // either reading shows (`03-design/dasha-kernels.md`, "PyJHora, beside the kernel").
    assert_eq!((written_agree, balances), (75, 212));
}

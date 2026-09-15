//! The falsification pass over the Bhava bala (Phase 5).
//!
//! The corpus's `baseline/bhava-bala` records the recording engine's Bhava
//! bala for every fixture its Shadbala corpus holds, computed from the
//! recorded lagna and houses and the engine's Shadbala of the same chart, with
//! the grahas it counted as aspecting each house.
//!
//! Unlike the Shadbala's pass, this one measures the **built** module:
//! `teistro_strength::bhava_bala` under the engine's reading must reproduce
//! every recorded house within the engine's rounding, and the readings of
//! BPHS ch. 27 vv. 26 to 31 and of Sripati (B.V. Raman's working, which
//! `crates/strength/tests/sripati.rs` holds the module to) are measured
//! against it one fork at a time.
//!
//! `cargo xtask bhava-bala` writes the page; `check-bhava-bala` regenerates it
//! in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro::settings::{BhavaDig, BhavaDrishti, BhavaSpecialRules};
use teistro::strength::bhava_bala::{
    BhavaBalaChart, BhavaBalaReading, BhavaBalaRules, BhavaGraha, NINE,
};
use teistro_core::catalogue::Graha;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table};

const PAGE: &str = "docs/03-design/bhava-bala-measured.md";
const ROOT: &str = "fixtures/baseline/bhava-bala";
const SEVEN: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];
/// The special aspects the engine adds to every graha's seventh.
const SPECIAL: [(Graha, [u8; 2]); 3] = [
    (Graha::Mars, [4, 8]),
    (Graha::Jupiter, [5, 9]),
    (Graha::Saturn, [3, 10]),
];

/// One recorded Bhava bala and the chart the module reads it from.
struct Record {
    chart: BhavaBalaChart,
    /// Each house's recorded aspecting grahas.
    aspected_by: Vec<Vec<Graha>>,
    /// Each house's recorded lord's strength, dig, drishti and total.
    bhavas: Vec<[f64; 4]>,
}

fn number(value: &Value) -> Result<f64, String> {
    value.as_f64().ok_or_else(|| String::from("a number"))
}

fn read(path: &Path) -> Result<Value, String> {
    serde_json::from_str(
        &std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))
}

fn record(root: &Path, file: &Value) -> Result<Record, String> {
    let inputs = &file["inputs"];
    let fixture = read(
        &root
            .join("fixtures/baseline")
            .join(file["fixture"].as_str().ok_or("fixture")?),
    )?;
    let lagna = inputs["lagna_sign_index"]
        .as_u64()
        .and_then(|l| u8::try_from(l).ok())
        .ok_or("lagna")?;
    let mut madhya = [0.0; 12];
    for (slot, step) in madhya.iter_mut().zip(0_u8..) {
        *slot = f64::from((lagna + step) % 12) * 30.0 + 15.0;
    }
    let bodies = &fixture["positions"]["bodies"];
    let mut grahas = [BhavaGraha {
        longitude: 0.0,
        house: 1,
    }; 9];
    for (slot, graha) in grahas.iter_mut().zip(NINE) {
        *slot = BhavaGraha {
            longitude: number(&bodies[graha.key()]["sidereal_longitude_deg"])?,
            house: inputs["graha_house"][graha.key()]
                .as_u64()
                .and_then(|h| u8::try_from(h).ok())
                .ok_or("house")?,
        };
    }
    let mut shadbala = [(0.0, 0.0); 7];
    for (slot, graha) in shadbala.iter_mut().zip(SEVEN) {
        let at = &inputs["shadbala"][graha.key()];
        *slot = (number(&at["total_shashtiamshas"])?, number(&at["drik"])?);
    }
    // The Hindu day holding the birth: the previous day's arcs before sunrise.
    let foundation = &fixture["foundation"];
    let instant = number(&foundation["jd_ut"])?;
    let today = &foundation["sunrise"];
    let day = if instant < number(&today["sunrise_jd"])? {
        let previous = &foundation["previous_day"];
        (
            number(&previous["sunrise_jd"])?,
            number(&previous["sunset_jd"])?,
            number(&today["sunrise_jd"])?,
        )
    } else {
        (
            number(&today["sunrise_jd"])?,
            number(&today["sunset_jd"])?,
            number(&foundation["next_day"]["sunrise_jd"])?,
        )
    };
    let aspected_by = (1..=12)
        .map(|house| {
            file["aspected_by"][house.to_string()]
                .as_array()
                .map(|names| {
                    names
                        .iter()
                        .filter_map(|n| n.as_str().and_then(Graha::from_key))
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();
    let bhavas = file["bhavas"]
        .as_array()
        .ok_or("bhavas")?
        .iter()
        .map(|b| {
            Ok([
                number(&b["bhavadhipati"])?,
                number(&b["dig"])?,
                number(&b["drishti"])?,
                number(&b["total"])?,
            ])
        })
        .collect::<Result<_, String>>()?;
    Ok(Record {
        chart: BhavaBalaChart {
            madhya,
            grahas,
            shadbala,
            instant,
            day,
        },
        aspected_by,
        bhavas,
    })
}

fn records(root: &Path) -> Result<Vec<Record>, String> {
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(ROOT).join(dir))
            .map_err(|e| format!("{dir}: {e}"))?
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.sort();
        for path in paths {
            out.push(record(root, &read(&path)?)?);
        }
    }
    Ok(out)
}

/// The engine's full aspects on a house: every graha's seventh, and the
/// special grahas' own two.
fn aspecting(chart: &BhavaBalaChart, house: u8) -> Vec<Graha> {
    NINE.iter()
        .zip(chart.grahas)
        .filter(|(graha, at)| {
            let count = (house + 12 - at.house) % 12 + 1;
            count == 7
                || SPECIAL
                    .iter()
                    .any(|(special, counts)| special == *graha && counts.contains(&count))
        })
        .map(|(graha, _)| *graha)
        .collect()
}

/// A recorded component: the rule, its index among the recorded values, and
/// where the module keeps it.
type Component = (
    &'static str,
    usize,
    fn(&teistro::strength::BhavaStrength) -> f64,
);

fn claims(records: &[Record]) -> Vec<Claim> {
    let houses = records.len() * 12;
    let engine = BhavaBalaRules::RECORDING_ENGINE;
    let mut claims = Vec::new();

    let wrong = records
        .iter()
        .flat_map(|r| {
            (1..=12_u8).map(move |house| {
                let recorded = r.aspected_by.get(usize::from(house) - 1);
                recorded != Some(&aspecting(&r.chart, house))
            })
        })
        .filter(|differs| *differs)
        .count();
    claims.push(Claim::counted(
        "a house is aspected by every graha's seventh and by Mars's fourth and eighth, Jupiter's fifth and ninth and Saturn's third and tenth, counted in whole-sign houses, the nodes the seventh alone",
        wrong,
        houses,
    ));

    let readings: Vec<BhavaBalaReading> = records
        .iter()
        .map(|r| BhavaBalaReading::of(&r.chart, engine))
        .collect();
    let components: [Component; 4] = [
        ("the lord's strength is the lord's whole Shadbala", 0, |b| {
            b.adhipati
        }),
        (
            "Dig is ten virupas a house from the house the sign's class makes weakest, Cancer and Scorpio insects, Sagittarius human and Capricorn quadruped throughout",
            1,
            |b| b.dig,
        ),
        (
            "the drishti bala is a quarter of the Dig added when only the Moon, Mercury, Jupiter or Venus aspect the house and taken away when only the others do, with Jupiter's and Mercury's Drik bala added when they aspect it",
            2,
            |b| b.drishti,
        ),
        (
            "the whole is the three together, with no special rules",
            3,
            |b| b.virupas,
        ),
    ];
    for (rule, index, ours) in components {
        let (mut wrong, mut far) = (0, 0.0_f64);
        for (r, reading) in records.iter().zip(&readings) {
            for (bhava, recorded) in reading.bhavas.iter().zip(&r.bhavas) {
                let gap = (ours(bhava) - recorded.get(index).copied().unwrap_or(f64::NAN)).abs();
                far = far.max(gap);
                // A missing record's NaN gap counts as wrong.
                wrong += usize::from(gap.is_nan() || gap > 0.005 + 1e-9);
            }
        }
        claims.push(
            Claim::counted(
                format!("{rule}, within the engine's hundredths"),
                wrong,
                houses,
            )
            .with_note(format!("worst {far:.1e}")),
        );
    }
    claims
}

/// A rival reading's houses that differ from the engine's by more than half
/// a virupa, and the widest gap.
fn rival(records: &[Record], rules: BhavaBalaRules) -> (usize, f64) {
    let (mut differ, mut far) = (0, 0.0_f64);
    for r in records {
        let engine = BhavaBalaReading::of(&r.chart, BhavaBalaRules::RECORDING_ENGINE);
        let other = BhavaBalaReading::of(&r.chart, rules);
        for (a, b) in engine.bhavas.iter().zip(&other.bhavas) {
            let gap = (a.virupas - b.virupas).abs();
            far = far.max(gap);
            differ += usize::from(gap > 0.5);
        }
    }
    (differ, far)
}

fn rival_claims(records: &[Record]) -> Vec<Claim> {
    let houses = records.len() * 12;
    let engine = BhavaBalaRules::RECORDING_ENGINE;
    let forks = [
        (
            "Sripati's classes: Scorpio the only insect and Cancer watery, Sagittarius and Capricorn split at their halves (B.V. Raman, Arts. 126–131)",
            BhavaBalaRules {
                dig: BhavaDig::Sripati,
                ..engine
            },
        ),
        (
            "the verses' classes and arc: Cancer and Scorpio insects, the halves split, the Dig a third of the arc between madhyas (vv. 26 to 28)",
            BhavaBalaRules {
                dig: BhavaDig::Bphs,
                ..engine
            },
        ),
        (
            "Sripati's drishti: the sphuta drishti on the bhava madhya, Jupiter's and Mercury's in full and a quarter of each other graha's (Arts. 132–133)",
            BhavaBalaRules {
                drishti: BhavaDrishti::Sphuta,
                ..engine
            },
        ),
        (
            "the special rules: a rupa for each Jupiter and Mercury in the house and less one for each Sun, Mars and Saturn, and 15 for a sign rising the way the hour favours (vv. 30 and 31)",
            BhavaBalaRules {
                special_rules: BhavaSpecialRules::Bphs,
                ..engine
            },
        ),
    ];
    forks
        .into_iter()
        .map(|(rule, rules)| {
            let (differ, far) = rival(records, rules);
            Claim::counted(rule, differ, houses).with_note(format!("worst gap {far:.2}"))
        })
        .collect()
}

fn page(root: &Path) -> Result<String, String> {
    let records = records(root)?;
    if records.is_empty() {
        return Err(String::from("the corpus records no Bhava bala"));
    }
    let mut out = String::new();
    let _ = write!(
        out,
        "# The Bhava bala, measured\n\n\
         Status: `generated` by `cargo xtask bhava-bala` over the conformance corpus's \
         `baseline/bhava-bala`, 2026-09-15. Do not edit: `check-bhava-bala` regenerates this page and \
         fails on any difference. The design it measures is \
         [`strength-schemes.md`](strength-schemes.md).\n\n\
         The corpus records the recording engine's Bhava bala for {charts} charts, every house's \
         lord's strength, Dig, drishti and total, computed from the recorded lagna and houses and the \
         engine's Shadbala of the same chart. This page measures the built module against it: \
         `teistro_strength::bhava_bala` under the engine's reading, and then each other reading \
         one fork at a time.\n\n\
         ## The engine's reading, reproduced\n\n{engine}\n\
         ## The other readings, one fork at a time\n\n\
         Each switches one fork from the engine's reading and counts the houses whose total moves \
         by more than half a virupa.\n\n{rivals}\n\
         ## What it means for the module\n\n\
         **The engine's reading is settled**, so the conformance profile takes it, the engine's \
         rounding to hundredths left to the caller.\n\n\
         **Every other fork moves real houses.** The verses as translated are the default (C73 to \
         C75): the Dig by the madhyas' arc with Cancer an insect and the two halves split, a quarter \
         of it for a one-sided drishti, and the special rules, whose twilight the verses leave \
         undefined and the module takes as two ghatis either side of the night (C75). Sripati's \
         reading is the other published one, and B.V. Raman's worked Standard Horoscope holds the \
         module to it house by house (`crates/strength/tests/sripati.rs`).\n",
        charts = count(records.len()),
        engine = table(&claims(&records)),
        rivals = table(&rival_claims(&records)),
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
        Ok(text) => {
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask bhava-bala") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

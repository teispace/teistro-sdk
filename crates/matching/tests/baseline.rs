//! Milan against the corpus's `baseline/milan`: every default-profile birth
//! of the corpus matched as the groom with every one as the bride, from each
//! side's recorded Moon (nakshatra, pada, sign), and the eight kootas'
//! points, their total, the Nadi and Bhakoot doshas and four of the ten
//! considerations as the recording engine read them
//! (`05-testing/01-golden-vectors.md`, the second export's milan).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::float_cmp,
    clippy::too_many_lines,
    reason = "tests fail by panicking, index JSON by key, print what they measure, compare points that are whole or half and walk a match's every comparison in one test"
)]

use std::collections::BTreeMap;
use std::path::Path;

#[path = "../../core/tests/support/conformance.rs"]
mod conformance;

use conformance::{Tally, carried};

use serde_json::Value;
use teistro_core::catalogue::{Koota, Nakshatra};
use teistro_matching::{
    ASHTA_KOOTA, BhakootDosha, DeerghaBeyond, KootaReading, KootaRules, Native, PoruthamRules,
    YoniRelation, ashta_koota, porutham,
};

const SECTION: &str = "baseline/milan";

/// The recorded points' order.
const KOOTAS: [&str; 8] = [
    "VARNA",
    "VASHYA",
    "TARA",
    "YONI",
    "GRAHA_MAITRI",
    "GANA",
    "BHAKOOT",
    "NADI",
];

fn files() -> Vec<(String, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/milan");
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
            // Only the default profile's births are matched, so the corpus
            // has no variants here.
            assert_eq!(dir, "variants", "{dir}: the corpus lists {SECTION}");
            continue;
        };
        let mut paths: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
        paths.sort();
        for path in paths {
            let file = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            out.push((
                format!("{dir}/{}", path.file_name().unwrap().to_string_lossy()),
                file,
            ));
        }
    }
    out
}

/// A side's Moon as the file repeats it.
fn native(inputs: &Value) -> Native {
    let star = Nakshatra::from_id(
        u16::try_from(inputs["moon_nakshatra_index"].as_u64().unwrap()).unwrap(),
    )
    .unwrap();
    let moon = Native::of_pada(
        star,
        u8::try_from(inputs["moon_pada"].as_u64().unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        u64::from(moon.rashi.id()),
        inputs["moon_sign_index"].as_u64().unwrap()
    );
    moon
}

fn bhakoot_type(dosha: Option<BhakootDosha>) -> Option<&'static str> {
    dosha.map(|d| match d {
        BhakootDosha::SixEight => "6/8",
        BhakootDosha::FiveNine => "5/9",
        BhakootDosha::TwoTwelve => "2/12",
    })
}

/// The engine's vashya, where the verse leaves the table to usage.
const VASHYA: &str =
    "the engine's vashya where the verse leaves the table to usage (matching.md, C260)";
/// The engine's yoni friends and enemies, which the verse does not list.
const YONI: &str =
    "the engine's yoni friends and enemies beyond the seven great enmities (matching.md, C261)";
/// The engine's gana points between two ganas.
const GANA: &str = "the engine's gana points between two ganas (matching.md, C262)";
/// The engine's tara for two stars nine or eighteen apart.
const TARA: &str = "the engine's tara for two stars nine or eighteen apart, each the Janma's tenth or nineteenth from the other (01-golden-vectors.md, Janma)";
/// A dosha the source lifts, whose points the engine gives back.
const LIFTED: &str = "a lifted dosha's points, which the engine gives back and the SDK reports beside them (matching.md, C284)";
/// A Bhakoot exception the engine counts and Garga's count does not.
const BHAKOOT: &str =
    "a bad Bhakoot the engine lifts by fewer exceptions than Garga's (matching.md, C263)";
/// A total moved by a koota above it.
const TOTAL: &str = "the total, moved by an explained koota (01-golden-vectors.md)";

/// The entry that explains a koota's miss, when its reading is one the
/// entry names.
fn explaining(
    reading: KootaReading,
    ours: f64,
    recorded: f64,
    same_star: bool,
) -> Option<&'static str> {
    match reading {
        KootaReading::Vashya { .. } => Some(VASHYA),
        KootaReading::Yoni {
            relation: YoniRelation::Neutral,
            ..
        } => Some(YONI),
        KootaReading::Gana { bride, groom, .. } if bride != groom => Some(GANA),
        KootaReading::Tara {
            bride_to_groom: 1,
            groom_to_bride: 1,
        } if !same_star && ours == 3.0 && recorded == 0.0 => Some(TARA),
        KootaReading::Nadi {
            dosha: true,
            lifted: true,
            ..
        } if recorded == 8.0 => Some(LIFTED),
        KootaReading::Bhakoot {
            dosha: Some(_),
            lifted: true,
            ..
        } if recorded == 7.0 => Some(LIFTED),
        KootaReading::Bhakoot {
            dosha: Some(_),
            lifted: false,
            exceptions,
            ..
        } if recorded == 7.0
            && (exceptions.one_lord
                || exceptions.lords_friends
                || exceptions.navamsha_lords_friends
                || exceptions.tara_pure
                || exceptions.vashya) =>
        {
            Some(BHAKOOT)
        }
        _ => None,
    }
}

#[test]
fn every_recorded_match_is_the_sdk_s_under_the_engine_s_reading() {
    if let Err(skipped) = carried(SECTION) {
        println!("{skipped}");
        return;
    }
    let files = files();
    let sides: BTreeMap<&str, Native> = files
        .iter()
        .map(|(name, file)| (name.as_str(), native(&file["inputs"])))
        .collect();
    // The printed Nepali reading of the vashya and the gana is the nearer
    // of the SDK's to the engine's (01-golden-vectors.md); the stree deergha
    // is the engine's own convention, `deerghaBeyond` (C779).
    let rules = KootaRules::PRINTED_IN_NEPAL;
    let ten_rules = PoruthamRules {
        deergha_beyond: DeerghaBeyond::TenthOrSameStar,
        ..PoruthamRules::default()
    };
    let mut tally = Tally::new(SECTION, "the engine's reading");
    let mut cells: BTreeMap<String, usize> = BTreeMap::new();
    let mut unexplained = Vec::new();
    let mut matches = 0;
    for (groom_name, file) in &files {
        let groom = sides[groom_name.as_str()];
        for m in file["matches"].as_array().unwrap() {
            matches += 1;
            let bride_name = m["bride"].as_str().unwrap();
            let bride = sides[bride_name];
            let what = || format!("{groom_name} with {bride_name}");
            let ours = ashta_koota(bride, groom, rules);
            let mut koota_missed = false;
            for (i, koota) in ASHTA_KOOTA.into_iter().enumerate() {
                let row = ours.row(koota).unwrap();
                let recorded = m["points"][i].as_f64().unwrap();
                if row.points == recorded {
                    tally.agreed(1);
                    continue;
                }
                koota_missed = true;
                let cell = format!(
                    "{} {:?} ours {} recorded {recorded}",
                    KOOTAS[i], row.reading, row.points
                );
                match explaining(
                    row.reading,
                    row.points,
                    recorded,
                    bride.nakshatra == groom.nakshatra,
                ) {
                    Some(entry) => {
                        *cells.entry(format!("{entry}: {cell}")).or_default() += 1;
                        tally.explained(entry, 1);
                    }
                    None => unexplained.push(format!("{}: {cell}", what())),
                }
            }
            if ours.total == m["total"].as_f64().unwrap() {
                tally.agreed(1);
            } else if koota_missed {
                tally.explained(TOTAL, 1);
            } else {
                unexplained.push(format!(
                    "{}: total ours {} recorded {}",
                    what(),
                    ours.total,
                    m["total"]
                ));
            }
            // The doshas' presence: the Nadi's, and the Bhakoot's kind.
            let (nadi, bhakoot) = (
                ours.row(Koota::Nadi).unwrap().reading,
                ours.row(Koota::Bhakoot).unwrap().reading,
            );
            let KootaReading::Nadi { dosha, .. } = nadi else {
                panic!("{nadi:?}")
            };
            if dosha == m["nadi_dosha"]["present"].as_bool().unwrap() {
                tally.agreed(1);
            } else {
                unexplained.push(format!("{}: nadi dosha {nadi:?}", what()));
            }
            let KootaReading::Bhakoot { dosha, .. } = bhakoot else {
                panic!("{bhakoot:?}")
            };
            if bhakoot_type(dosha) == m["bhakoot_dosha"]["type"].as_str() {
                tally.agreed(1);
            } else {
                unexplained.push(format!("{}: bhakoot dosha {bhakoot:?}", what()));
            }
            // Four of the ten, each the rule's own verdict: agreeing and
            // not by the p. 76 exception, which the engine does not read.
            let ten = porutham(bride, groom, ten_rules);
            for (koota, recorded) in [
                (Koota::Rajju, m["rajju_compatible"].as_bool().unwrap()),
                (Koota::Vedha, !m["vedha"].as_bool().unwrap()),
                (
                    Koota::Mahendra,
                    m["mahendra"]["favorable"].as_bool().unwrap(),
                ),
                (
                    Koota::StreeDeergha,
                    m["stree_deergha"]["favorable"].as_bool().unwrap(),
                ),
            ] {
                let row = ten.row(koota).unwrap();
                if (row.agrees && !row.lifted) == recorded {
                    tally.agreed(1);
                } else {
                    unexplained.push(format!(
                        "{}: {koota:?} {:?} recorded {recorded}",
                        what(),
                        row.reading
                    ));
                }
            }
        }
    }
    for (cell, count) in &cells {
        println!("cell {count} x {cell}");
    }
    for miss in &unexplained {
        println!("unexplained: {miss}");
    }
    println!(
        "{} grooms, {matches} matches; compared {}, {} unexplained",
        files.len(),
        tally.compared(),
        unexplained.len()
    );
    assert!(
        unexplained.is_empty(),
        "{} misses no entry explains",
        unexplained.len()
    );
    assert_eq!((files.len(), matches), (55, 3025));
    tally.record();
}

//! The conformance corpus's `baseline/yogas`, read for the tests and the
//! benchmark: the rules strictly, and each recorded chart as the kernel's.

#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "each target uses part of it, and a bad fixture stops the run"
)]

use std::path::{Path, PathBuf};

use serde::Deserialize as _;
use serde_json::Value;
use teistro_core::catalogue::{CharaKaraka, Dignity, Rashi};
use teistro_rules::{Body, House, Karaka, Placement, Rule, RuleChart};

pub(crate) fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/yogas")
}

pub(crate) fn read(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Every rule the engine ships, read strictly.
pub(crate) fn rules() -> Vec<Rule> {
    serde_json::from_value(read(&corpus().join("rules.json"))["rules"].clone())
        .expect("every rule reads strictly")
}

/// Every recorded file, charts then variants, each in name order.
pub(crate) fn files() -> Vec<(PathBuf, Value)> {
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(corpus().join(dir))
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .collect();
        paths.sort();
        out.extend(paths.into_iter().map(|path| {
            let file = read(&path);
            (path, file)
        }));
    }
    out
}

/// The chara karaka a recorded abbreviation names, if the body holds one.
fn karaka(value: &Value) -> Option<CharaKaraka> {
    (!value.is_null()).then(|| Karaka::deserialize(value).unwrap().0)
}

/// The chart a file repeats.
pub(crate) fn chart(inputs: &Value) -> RuleChart {
    let placements = Body::ALL.map(|body| {
        let b = &inputs["bodies"][body.key()];
        Placement {
            longitude: b["sidereal_longitude_deg"].as_f64().unwrap(),
            sign: Rashi::from_id(u16::try_from(b["sign_index"].as_u64().unwrap()).unwrap())
                .unwrap(),
            house: House::try_new(u8::try_from(b["house"].as_u64().unwrap()).unwrap()).unwrap(),
            dignity: Dignity::from_key(b["dignity"].as_str().unwrap()).unwrap(),
            retrograde: b["is_retrograde"].as_bool().unwrap(),
            combust: b["combust"].as_str() != Some("none"),
            karaka7: karaka(&inputs["chara_karaka_7"][body.key()]),
            karaka8: karaka(&inputs["chara_karaka_8"][body.key()]),
        }
    });
    RuleChart { placements }
}

pub(crate) fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

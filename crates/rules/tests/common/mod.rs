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
use teistro_core::angle::Nas;
use teistro_core::catalogue::{
    CharaKaraka, Dignity, Karana, Nakshatra, Rashi, Tithi, Vara, Varga, Yoga,
};
use teistro_core::quantity::Degrees;
use teistro_rules::{
    Body, House, Karaka, Pada, Panchanga, Placement, Rule, RuleChart, Spans, StrengthMeasure,
    Strengths, VargaSigns,
};
use teistro_vargas::{Scheme, sign};

pub(crate) fn corpus() -> PathBuf {
    baseline("yogas")
}

/// A directory of the conformance corpus's `baseline`.
pub(crate) fn baseline(directory: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/baseline")
        .join(directory)
}

pub(crate) fn read(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Every yoga rule the engine ships, read strictly.
pub(crate) fn rules() -> Vec<Rule> {
    rules_in("yogas")
}

/// Every rule a rule corpus holds, read strictly.
pub(crate) fn rules_in(directory: &str) -> Vec<Rule> {
    serde_json::from_value(read(&baseline(directory).join("rules.json"))["rules"].clone())
        .expect("every rule reads strictly")
}

/// Every recorded yoga file, charts then variants, each in name order.
pub(crate) fn files() -> Vec<(PathBuf, Value)> {
    files_in("yogas")
}

/// Every recorded file of a rule corpus, charts then variants, each in name
/// order.
pub(crate) fn files_in(directory: &str) -> Vec<(PathBuf, Value)> {
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(baseline(directory).join(dir))
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

/// The recorded chart a yoga file repeats: the fixture of the same name, a
/// chart or a variant.
pub(crate) fn recorded_chart(yogas: &Path) -> Value {
    let dir = yogas.parent().and_then(Path::file_name).unwrap();
    read(
        &corpus()
            .join("..")
            .join(dir)
            .join(yogas.file_name().unwrap()),
    )
}

/// The chara karaka a recorded abbreviation names, if the body holds one.
fn karaka(value: &Value) -> Option<CharaKaraka> {
    (!value.is_null()).then(|| Karaka::deserialize(value).unwrap().0)
}

/// The sign a sidereal longitude falls in, in a division under the SDK's
/// classical scheme for it.
pub(crate) fn varga_sign(varga: Varga, longitude: f64) -> Rashi {
    let longitude = Nas::from_degrees(Degrees::try_new(longitude.rem_euclid(360.0)).unwrap());
    sign(&Scheme::of(varga), longitude)
}

/// The navamsha sign of a sidereal longitude.
pub(crate) fn navamsha(longitude: f64) -> Rashi {
    varga_sign(Varga::D9, longitude)
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
            navamsha: navamsha(b["sidereal_longitude_deg"].as_f64().unwrap()),
        }
    });
    RuleChart {
        placements,
        panchanga: panchanga(&inputs["panchanga"]),
        strengths: None,
    }
}

/// The divisional charts BPHS ch. 39 reads the lagna in — the hora,
/// drekkana, navamsha, dwadashamsha and trimshamsha — each computed from the
/// recorded sidereal longitudes under the SDK's classical scheme for that
/// division, the lagna's included.
pub(crate) fn vargas(chart: &RuleChart) -> Vec<VargaSigns> {
    [Varga::D2, Varga::D3, Varga::D9, Varga::D12, Varga::D30]
        .into_iter()
        .map(|varga| chart.varga_signs(varga, |longitude| varga_sign(varga, longitude)))
        .collect()
}

/// The same chart with the Shadbala the corpus recorded for it, when it
/// recorded any: `baseline/shadbala` holds 71 of the 93, so the tests read both
/// a chart that can answer a question of strength and one that cannot.
pub(crate) fn chart_at(path: &Path, inputs: &Value) -> RuleChart {
    RuleChart {
        strengths: strengths(path),
        ..chart(inputs)
    }
}

/// The recorded Shadbala of a chart: each classical graha's total in rupas and
/// the minimum it must reach, both as the recording engine computed them
/// (crux C71 on which reading of the minimum that is). The nodes and the lagna
/// have none, which the measure does not reach.
pub(crate) fn strengths(path: &Path) -> Option<Strengths> {
    let file = baseline("shadbala")
        .join(path.parent().and_then(Path::file_name).unwrap())
        .join(path.file_name().unwrap());
    let recorded = std::fs::read_to_string(file).ok()?;
    let recorded: Value = serde_json::from_str(&recorded).unwrap();
    let shadbala = &recorded["shadbala"];
    let mut of = [None; 10];
    let mut required = [None; 10];
    for body in Body::ALL {
        let at = &shadbala[body.key()];
        of[body.index()] = at["total_rupas"].as_f64();
        required[body.index()] = at["minimum_rupas"].as_f64();
    }
    Some(Strengths {
        measure: StrengthMeasure::Shadbala,
        of,
        required,
    })
}

/// The panchanga a file repeats, if it records one: the engine's karana and
/// yoga indices are the catalogue's ids, and it passes no sankranti or
/// eclipse.
fn panchanga(recorded: &Value) -> Option<Panchanga> {
    let id = |field: &str| u16::try_from(recorded[field].as_u64()?).ok();
    recorded.as_object()?;
    Some(Panchanga {
        tithi: Tithi::from_id(id("tithi_number")? - 1)?,
        vara: Vara::from_key(recorded["vara"].as_str()?)?,
        nakshatra: Nakshatra::from_id(id("nakshatra_index")?)?,
        pada: Pada::try_new(u8::try_from(id("moon_pada")?).ok()?).ok()?,
        yoga: Yoga::from_id(id("yoga_index")?)?,
        karana: Karana::from_id(id("karana_index")?)?,
        // The yogas' and doshas' inputs record no ghatikas of any limb.
        spans: Spans::default(),
        by_day: None,
        on_sankranti: false,
        eclipse: None,
    })
}

pub(crate) fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

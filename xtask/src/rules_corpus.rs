//! What the passes that run the rules kernel over a rule corpus share: its
//! JSON, the rules it ships, and each file's recorded chart.

use std::path::Path;

use serde::Deserialize as _;
use serde_json::Value;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{
    CharaKaraka, Dignity, Karana, Nakshatra, Rashi, Tithi, Vara, Varga, Yoga,
};
use teistro_core::quantity::Degrees;
use teistro_rules::{Body, House, Karaka, Pada, Panchanga, Placement, Rule, RuleChart};

pub(crate) fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_str(
        &std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))
}

pub(crate) fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn rules(root: &Path, directory: &str) -> Result<Vec<Rule>, String> {
    let file = read_json(
        &root
            .join("fixtures/baseline")
            .join(directory)
            .join("rules.json"),
    )?;
    serde_json::from_value(file["rules"].clone()).map_err(|e| format!("rules.json: {e}"))
}

/// The chara karaka a recorded abbreviation names, if the body holds one.
fn karaka(value: &Value) -> Option<CharaKaraka> {
    Karaka::deserialize(value).ok().map(|Karaka(karaka)| karaka)
}

/// The chart a file repeats.
pub(crate) fn chart(inputs: &Value) -> Result<RuleChart, String> {
    let mut placements = Vec::with_capacity(Body::ALL.len());
    for body in Body::ALL {
        let b = &inputs["bodies"][body.key()];
        let sign = b["sign_index"]
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .and_then(Rashi::from_id)
            .ok_or_else(|| format!("{}: sign", body.key()))?;
        let house = b["house"]
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .and_then(|n| House::try_new(n).ok())
            .ok_or_else(|| format!("{}: house", body.key()))?;
        let longitude = b["sidereal_longitude_deg"]
            .as_f64()
            .ok_or_else(|| format!("{}: longitude", body.key()))?;
        placements.push(Placement {
            longitude,
            sign,
            house,
            dignity: b["dignity"]
                .as_str()
                .and_then(Dignity::from_key)
                .ok_or_else(|| format!("{}: dignity", body.key()))?,
            retrograde: b["is_retrograde"].as_bool().unwrap_or_default(),
            combust: b["combust"].as_str() != Some("none"),
            karaka7: karaka(&inputs["chara_karaka_7"][body.key()]),
            karaka8: karaka(&inputs["chara_karaka_8"][body.key()]),
            navamsha: teistro_vargas::sign(
                &teistro_vargas::Scheme::of(Varga::D9),
                Nas::from_degrees(
                    Degrees::try_new(longitude.rem_euclid(360.0)).map_err(|e| e.to_string())?,
                ),
            ),
        });
    }
    let placements = placements
        .try_into()
        .map_err(|_| String::from("ten placements"))?;
    Ok(RuleChart {
        placements,
        panchanga: panchanga(&inputs["panchanga"])?,
    })
}

/// The panchanga a file repeats, if it records one: the recording engine's
/// karana and yoga indices are the catalogue's ids, and it passes no sankranti
/// or eclipse. A file that records none (the yogas') carries null.
fn panchanga(recorded: &Value) -> Result<Option<Panchanga>, String> {
    if recorded.is_null() {
        return Ok(None);
    }
    let id = |field: &str| -> Result<u16, String> {
        recorded[field]
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .ok_or_else(|| format!("panchanga: {field}"))
    };
    let tithi = Tithi::from_id(id("tithi_number")? - 1).ok_or("panchanga: tithi")?;
    let vara = recorded["vara"]
        .as_str()
        .and_then(Vara::from_key)
        .ok_or("panchanga: vara")?;
    let nakshatra = Nakshatra::from_id(id("nakshatra_index")?).ok_or("panchanga: nakshatra")?;
    let pada = u8::try_from(id("moon_pada")?)
        .map_err(|_| String::from("panchanga: pada"))
        .and_then(Pada::try_new)?;
    let yoga = Yoga::from_id(id("yoga_index")?).ok_or("panchanga: yoga")?;
    let karana = Karana::from_id(id("karana_index")?).ok_or("panchanga: karana")?;
    Ok(Some(Panchanga {
        tithi,
        vara,
        nakshatra,
        pada,
        yoga,
        karana,
        on_sankranti: false,
        eclipse: None,
    }))
}

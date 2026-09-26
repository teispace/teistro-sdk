//! Every closed enum at the boundary is spelled as serde spells the Rust
//! type it carries, so a binding reads back the word a stored document,
//! a settings patch and a request hold (`INVALID_ARG`, `IANA`,
//! `TIME_UNKNOWN_FALLBACK`), and never a spelling of its own.
//!
//! The description records each member's key; this reads every key
//! through the Rust type's own deserializer (a tagged enum's with a
//! sample of its fields), asks it to serialise back to the same word, and
//! follows the boundary's own conversion to the id the description gives
//! it, so every member is reached. A closed enum with no serialised Rust
//! form is in [`UNSERIALISED`], with the reason; the list is refused both
//! ways.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests fail by panicking"
)]

use std::collections::BTreeSet;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use teistro_core::catalogue::{Nakshatra, Tithi, Vara};
use teistro_core::envelope::CalendarResolution;
use teistro_core::settings;
use teistro_ffi::calendar::TsResolution;
use teistro_ffi::chart::{
    TsBalance, TsBrahmaOutcome, TsBrahmaRule, TsBurning, TsDashaPhase, TsDayPart, TsDayState,
    TsEkadhipatya, TsGhatiReckoning, TsHarshaGrade, TsHoraReckoning, TsPolarDayPolicy, TsPolarKind,
    TsQuadrant, TsReading, TsSaham, TsSahamStrong, TsSahamWeak, TsShodhana, TsStrength, TsSunrise,
    TsTajikaDrishti, TsTajikaRelation, TsTajikaYoga, TsVarsheshaChosen, TsVimshopakaScoring,
    TsYearYoga,
};
use teistro_ffi::panchanga::{TsLunarMonth, TsMonthKind, TsYogaCause};
use teistro_ffi::time::{TsChosen, TsDeltaTSource, TsDst, TsZoneEra, TsZoneSource, TsZoneWarning};
use teistro_ffi::{SDK_VERSION, schemas};
use teistro_idl::model::{Api, EnumDef};
use teistro_idl::sdk::describe;
use teistro_panchanga::omen::YogaCause;
use teistro_port_ephemeris::ProviderError;
use teistro_time::{DayState, DstOutcome};

fn api() -> Api {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    describe(&root, schemas::schemas(), SDK_VERSION).unwrap_or_else(|e| panic!("{e}"))
}

/// The closed enums whose members no Rust type serialises, each with why
/// its key is the only spelling there is.
const UNSERIALISED: [(&str, &str); 5] = [
    (
        "TsEphemeris",
        "an option of the C context's constructor; the Rust builder takes a provider",
    ),
    (
        "TsAffliction",
        "a bit position of an affliction mask; the Rust record holds five flags",
    ),
    (
        "TsScale",
        "the conversions' choice of scale; the Rust side has one type per scale",
    ),
    (
        "TsZoneKind",
        "what a zone specification names; the Rust side has one constructor each",
    ),
    (
        "TsMoonEvent",
        "which of two event lists a row came from; the Rust day holds the lists",
    ),
];

/// The description's enum of that name.
fn enum_def<'a>(api: &'a Api, ts: &str) -> &'a EnumDef {
    api.enums
        .iter()
        .find(|e| e.name == ts)
        .unwrap_or_else(|| panic!("the description has no enum `{ts}`"))
}

/// The word a value is spelled by: a unit variant's string, or a tagged
/// one's `tag` field.
fn spelling(value: &serde_json::Value, tag: &str) -> Option<String> {
    value
        .as_str()
        .or_else(|| value.get(tag).and_then(serde_json::Value::as_str))
        .map(String::from)
}

/// Holds `ts`'s keys to serde: each sample serialises to the key of the
/// member the boundary crosses it as, and every member but `unsampled` is
/// reached by one.
fn spelled_as<D: Serialize>(
    api: &Api,
    ts: &'static str,
    tag: &str,
    samples: &[D],
    id: impl Fn(&D) -> Option<i64>,
    unsampled: &[&str],
) -> &'static str {
    let def = enum_def(api, ts);
    let mut reached = BTreeSet::new();
    for sample in samples {
        let value = serde_json::to_value(sample).unwrap();
        let spelled = spelling(&value, tag)
            .unwrap_or_else(|| panic!("`{ts}`: a sample serialises as {value}"));
        let Some(id) = id(sample) else { continue };
        let member = def
            .values
            .iter()
            .find(|m| m.value == id)
            .unwrap_or_else(|| panic!("`{ts}` has no member {id}, which `{spelled}` crosses as"));
        assert_eq!(member.key, spelled, "`{ts}`'s key and its serde spelling");
        reached.insert(member.key.as_str());
    }
    let missed: Vec<&str> = def
        .values
        .iter()
        .map(|m| m.key.as_str())
        .filter(|key| !reached.contains(key) && !unsampled.contains(key))
        .collect();
    assert!(missed.is_empty(), "`{ts}`: no sample reached {missed:?}");
    ts
}

/// Every member of `ts` read back through `D`'s deserializer: the key as
/// a unit variant, or tagged under `tag` with the fields `fields` gives
/// that key. A key serde does not read fails here, by name.
fn read_keys<D: DeserializeOwned>(
    api: &Api,
    ts: &str,
    tag: &str,
    fields: impl Fn(&str) -> serde_json::Value,
) -> Vec<D> {
    enum_def(api, ts)
        .values
        .iter()
        .map(|member| {
            let key = member.key.as_str();
            let mut tagged = fields(key);
            let read = match tagged.as_object_mut() {
                Some(object) => {
                    object.insert(tag.to_string(), serde_json::Value::from(key));
                    serde_json::from_value(tagged)
                }
                None => serde_json::from_value(serde_json::Value::from(key)),
            };
            read.unwrap_or_else(|e| panic!("`{ts}`'s key `{key}` is not how serde spells it: {e}"))
        })
        .collect()
}

/// [`spelled_as`] over [`read_keys`] of a unit enum.
fn unit<D: Serialize + DeserializeOwned>(
    api: &Api,
    ts: &'static str,
    id: impl Fn(&D) -> Option<i64>,
) -> &'static str {
    let samples: Vec<D> = read_keys(api, ts, "", |_| serde_json::Value::Null);
    spelled_as(api, ts, "", &samples, id, &[])
}

/// The port's own enums.
fn port(api: &Api) -> Vec<&'static str> {
    vec![
        unit(api, "Status", |s: &teistro_core::Status| Some(*s as i64)),
        unit(api, "Body", |b: &teistro_port_ephemeris::Body| {
            Some(*b as i64)
        }),
        unit(api, "TimeScale", |s: &teistro_port_ephemeris::TimeScale| {
            Some(*s as i64)
        }),
        unit(
            api,
            "DistanceUnit",
            |u: &teistro_port_ephemeris::DistanceUnit| Some(*u as i64),
        ),
        unit(
            api,
            "SpeedModel",
            |m: &teistro_port_ephemeris::SpeedModel| Some(*m as i64),
        ),
        unit(api, "Astronomy", |a: &teistro_port_ephemeris::Astronomy| {
            Some(*a as i64)
        }),
        unit(api, "Centre", |c: &teistro_port_ephemeris::Centre| {
            Some(*c as i64)
        }),
        unit(api, "Equinox", |e: &teistro_port_ephemeris::Equinox| {
            Some(*e as i64)
        }),
        unit(
            api,
            "Coordinates",
            |c: &teistro_port_ephemeris::Coordinates| Some(*c as i64),
        ),
    ]
}

/// The boundary's tags of enums that carry fields, and a vtable's code.
fn tagged(api: &Api) -> Vec<&'static str> {
    let id = |value: u8| Some(i64::from(value));
    vec![
        // A vtable's return code is spelled by the error the port makes of
        // it; success makes none.
        spelled_as(
            api,
            "ProviderCode",
            "kind",
            &[
                ProviderError::unsupported("positions"),
                ProviderError::OutOfRange { jd: 0.0 },
                ProviderError::DataMissing {
                    detail: String::new(),
                },
                ProviderError::Refused {
                    detail: String::new(),
                },
                ProviderError::invalid("a request"),
            ],
            |e| Some(i64::from(e.code())),
            &["OK"],
        ),
        {
            let samples: Vec<CalendarResolution> =
                read_keys(api, "TsResolution", "kind", |key| match key {
                    "TABULAR" => json!({"authority": "a", "edition": "e"}),
                    "COMPUTED" => json!({"model": "m"}),
                    "DIVERGENT" => json!({
                        "tabular": {"month": 1, "day": 1},
                        "computed": {"month": 1, "day": 2},
                        "model": "m",
                    }),
                    _ => json!({}),
                });
            spelled_as(
                api,
                "TsResolution",
                "kind",
                &samples,
                |r| id(TsResolution::from(r) as u8),
                &[],
            )
        },
        {
            let samples: Vec<DstOutcome> = read_keys(api, "TsDst", "kind", |key| match key {
                "GAP" => json!({"shifted_by_seconds": 3600}),
                "OVERLAP" => json!({"chosen": "LATER"}),
                _ => json!({}),
            });
            spelled_as(
                api,
                "TsDst",
                "kind",
                &samples,
                |d| id(TsDst::from(*d) as u8),
                &[],
            )
        },
        {
            let samples: Vec<DayState> = read_keys(api, "TsDayState", "state", |key| match key {
                "POLAR" => json!({"kind": "DAY", "policy": "CIVIL_MIDNIGHT"}),
                _ => json!({}),
            });
            spelled_as(
                api,
                "TsDayState",
                "state",
                &samples,
                |d| id(TsDayState::split(*d).0 as u8),
                &[],
            )
        },
        spelled_as(
            api,
            "TsYogaCause",
            "cause",
            &[
                YogaCause::VaraNakshatra {
                    vara: Vara::Somavara,
                    nakshatra: Nakshatra::Hasta,
                },
                YogaCause::VaraTithiNakshatra {
                    vara: Vara::Somavara,
                    tithi: Tithi::ShuklaSaptami,
                    nakshatra: Nakshatra::Hasta,
                },
            ],
            |c| id(TsYogaCause::from(*c) as u8),
            &[],
        ),
    ]
}

/// A chart's enums, the Tajika ones among them.
fn chart(api: &Api) -> Vec<&'static str> {
    let id = |value: u8| Some(i64::from(value));
    vec![
        unit(api, "TsReading", |r: &teistro_chart::bhava::Reading| {
            id(TsReading::from(*r) as u8)
        }),
        unit(api, "TsBurning", |b: &teistro_state::burn::Burning| {
            id(TsBurning::from(*b) as u8)
        }),
        unit(
            api,
            "TsQuadrant",
            |q: &teistro_houses::classify::Quadrant| id(TsQuadrant::from(*q) as u8),
        ),
        unit(
            api,
            "TsStrength",
            |s: &teistro_aspect::drishti::Strength| id(TsStrength::from(*s) as u8),
        ),
        unit(api, "TsBalance", |b: &settings::Balance| {
            id(TsBalance::from(*b) as u8)
        }),
        unit(api, "TsShodhana", |s: &settings::Shodhana| {
            id(TsShodhana::from(*s) as u8)
        }),
        unit(api, "TsEkadhipatya", |e: &settings::Ekadhipatya| {
            id(TsEkadhipatya::from(*e) as u8)
        }),
        unit(api, "TsVimshopakaScoring", |v: &settings::Vimshopaka| {
            id(TsVimshopakaScoring::from(*v) as u8)
        }),
        unit(api, "TsDashaPhase", |p: &teistro::strength::DashaPhase| {
            id(TsDashaPhase::from(*p) as u8)
        }),
        unit(api, "TsBrahmaRule", |r: &settings::BrahmaRule| {
            id(TsBrahmaRule::from(*r) as u8)
        }),
        brahma_outcome(api),
        unit(api, "TsDayPart", |p: &teistro_chart::day::DayPart| {
            id(TsDayPart::from(*p) as u8)
        }),
        unit(api, "TsSunrise", |s: &settings::Sunrise| {
            TsSunrise::of(*s).and_then(|s| id(s as u8))
        }),
        unit(api, "TsGhatiReckoning", |g: &settings::GhatiReckoning| {
            TsGhatiReckoning::of(*g).and_then(|g| id(g as u8))
        }),
        unit(api, "TsHoraReckoning", |h: &settings::HoraReckoning| {
            TsHoraReckoning::of(*h).and_then(|h| id(h as u8))
        }),
        unit(
            api,
            "TsPolarKind",
            |k: &teistro_time::local_day::PolarKind| id(TsPolarKind::from(*k) as u8),
        ),
        unit(api, "TsPolarDayPolicy", |p: &settings::PolarDayPolicy| {
            TsPolarDayPolicy::of(*p).and_then(|p| id(p as u8))
        }),
        unit(api, "TsVarsheshaChosen", |c: &teistro::Chosen| {
            id(TsVarsheshaChosen::from(*c) as u8)
        }),
        unit(api, "TsTajikaDrishti", |d: &teistro::TajikaDrishti| {
            id(TsTajikaDrishti::from(*d) as u8)
        }),
        unit(api, "TsTajikaYoga", |y: &teistro::TajikaYoga| {
            id(TsTajikaYoga::from(*y) as u8)
        }),
        unit(api, "TsYearYoga", |y: &teistro::YearYoga| {
            id(TsYearYoga::from(*y) as u8)
        }),
        unit(api, "TsSaham", |s: &teistro::Saham| {
            id(TsSaham::from(*s) as u8)
        }),
        unit(api, "TsTajikaRelation", |r: &teistro::TajikaRelation| {
            id(TsTajikaRelation::from(*r) as u8)
        }),
        unit(api, "TsHarshaGrade", |g: &teistro::HarshaGrade| {
            id(TsHarshaGrade::from(*g) as u8)
        }),
        unit(api, "TsSahamStrong", |c: &teistro::StrongClause| {
            id(TsSahamStrong::from(*c) as u8)
        }),
        unit(api, "TsSahamWeak", |c: &teistro::WeakClause| {
            id(TsSahamWeak::from(*c) as u8)
        }),
    ]
}

/// Why a chart has no Brahma graha, and `FOUND` for one that has: the
/// reasons are `NoBrahma`'s serde spellings, and `FOUND` is the absence of a
/// reason, which no Rust type spells.
fn brahma_outcome(api: &Api) -> &'static str {
    let samples: Vec<teistro::dasha::jaimini::NoBrahma> = enum_def(api, "TsBrahmaOutcome")
        .values
        .iter()
        .filter(|member| member.key != "FOUND")
        .map(|member| serde_json::from_value(serde_json::Value::from(member.key.as_str())))
        .collect::<Result<_, _>>()
        .unwrap_or_else(|e| panic!("`TsBrahmaOutcome`'s reasons are `NoBrahma`'s: {e}"));
    spelled_as(
        api,
        "TsBrahmaOutcome",
        "",
        &samples,
        |none| Some(i64::from(TsBrahmaOutcome::from(Some(*none)) as u8)),
        &["FOUND"],
    )
}

/// The time, calendar and almanac enums.
fn time_and_calendar(api: &Api) -> Vec<&'static str> {
    let id = |value: u8| Some(i64::from(value));
    vec![
        unit(api, "TsZoneSource", |s: &teistro_time::ZoneSource| {
            id(TsZoneSource::from(*s) as u8)
        }),
        unit(api, "TsZoneEra", |e: &teistro_time::ZoneEra| {
            id(TsZoneEra::from(*e) as u8)
        }),
        unit(api, "TsChosen", |c: &teistro_time::Chosen| {
            id(TsChosen::from(*c) as u8)
        }),
        unit(api, "TsZoneWarning", |w: &teistro_time::Warning| {
            id(TsZoneWarning::from(*w) as u8)
        }),
        unit(
            api,
            "TsDeltaTSource",
            |s: &teistro_astro::delta_t::DeltaTSource| id(TsDeltaTSource::from(*s) as u8),
        ),
        unit(api, "TsLunarMonth", |m: &settings::LunarMonth| {
            TsLunarMonth::of(*m).and_then(|m| id(m as u8))
        }),
        unit(
            api,
            "TsMonthKind",
            |k: &teistro_calendar::lunisolar::MonthKind| id(TsMonthKind::from(*k) as u8),
        ),
    ]
}

#[test]
fn every_closed_enum_is_spelled_as_serde_spells_its_rust_type() {
    let api = api();
    let checked: Vec<&str> = [
        port(&api),
        tagged(&api),
        chart(&api),
        time_and_calendar(&api),
    ]
    .concat();

    // Both ways: every closed enum is checked or excused, and nothing is
    // both, and an excuse names an enum the description has.
    let closed: BTreeSet<&str> = api
        .enums
        .iter()
        .filter(|e| e.kind.is_none() && e.name != "Kind")
        .map(|e| e.name.as_str())
        .collect();
    let checked: BTreeSet<&str> = checked.into_iter().collect();
    let excused: BTreeSet<&str> = UNSERIALISED.iter().map(|(name, _)| *name).collect();
    assert!(
        checked.is_disjoint(&excused),
        "checked and excused: {:?}",
        checked.intersection(&excused).collect::<Vec<_>>()
    );
    let accounted: BTreeSet<&str> = checked.union(&excused).copied().collect();
    assert_eq!(
        closed.difference(&accounted).collect::<Vec<_>>(),
        Vec::<&&str>::new(),
        "closed enums neither checked nor excused"
    );
    assert_eq!(
        accounted.difference(&closed).collect::<Vec<_>>(),
        Vec::<&&str>::new(),
        "checked or excused, and not a closed enum of the description"
    );
}

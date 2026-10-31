//! Every ranged field of the API description refused outside its range
//! (`05-testing/01-quality-bar.md`, "input validation"). The cases are
//! read from `idl/api.json` rather than written here: each field a caller
//! fills that declares `range=` is sent the nearest value outside each end
//! of it, and a float field not a number and both infinities, through the
//! entry point that reads it, from a request that is otherwise answered.
//! Each must come back refused, naming the field. A struct that gains a
//! ranged field fails this test until its field can be set here, so no
//! range the description states goes untested.

#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "it crosses the boundary, fails by panicking, and reports what it checked"
)]

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::ptr;

use common::{Ctx, Record, chart_blob, chart_request, sized};
use teistro_core::Status;
use teistro_core::catalogue::Calendar;
use teistro_ffi::TS_CONTEXT_TEST_PROVIDER;
use teistro_ffi::blob::{TsBlob, ts_blob_free};
use teistro_ffi::calendar::{TsCalendarDate, ts_calendar_to_fixed};
use teistro_ffi::panchanga::{TsPanchangaRequest, ts_panchanga_days};
use teistro_ffi::time::{
    TsCivilDateTime, TsCivilTime, TsZoneKind, TsZoneResolution, TsZoneSpec, ts_time_resolve,
};

/// The structs the provider port fills or reads, not a caller: their
/// ranges describe what a provider answers, and the conformance kit holds
/// a provider to them (`provider conformance kit`).
const PROVIDER_SIDE: [&str; 5] = [
    "ObserverC",
    "ObliquityC",
    "HorizonRequestC",
    "CrossingRequestC",
    "CrossingEventC",
];

/// A field the description gives a range.
#[derive(Debug)]
struct Ranged {
    field: String,
    scalar: String,
    range: String,
}

/// Every caller-filled struct's ranged fields, from the description.
fn described() -> BTreeMap<String, Vec<Ranged>> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../idl/api.json");
    let api: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut structs = BTreeMap::new();
    for entry in api.get("structs").and_then(|s| s.as_array()).unwrap() {
        let name = entry.get("name").and_then(|n| n.as_str()).unwrap();
        let fields: Vec<Ranged> = entry
            .get("fields")
            .and_then(|f| f.as_array())
            .unwrap()
            .iter()
            .filter_map(|field| {
                let range = field.pointer("/meta/range")?.as_str()?;
                Some(Ranged {
                    field: field.get("name")?.as_str()?.to_owned(),
                    scalar: field.pointer("/ty/scalar")?.as_str()?.to_owned(),
                    range: range.to_owned(),
                })
            })
            .collect();
        if !fields.is_empty() {
            structs.insert(name.to_owned(), fields);
        }
    }
    structs
}

/// The float next to a finite `x`, above it or below: `next_up` and
/// `next_down`, which are newer than the workspace's MSRV.
fn adjacent(x: f64, up: bool) -> f64 {
    if x == 0.0 {
        let least = f64::from_bits(1);
        return if up { least } else { -least };
    }
    let bits = x.to_bits();
    f64::from_bits(if (x > 0.0) == up { bits + 1 } else { bits - 1 })
}

/// The values just outside a range written `[a,b]`, `(a,b]` or `[a,b)`
/// that the field's type can hold, and for a float the three it cannot
/// order.
fn outside(scalar: &str, range: &str) -> Vec<f64> {
    let (low, high) = range
        .trim_matches(|c| "[]()".contains(c))
        .split_once(',')
        .unwrap();
    let (low, high): (f64, f64) = (low.trim().parse().unwrap(), high.trim().parse().unwrap());
    let float = scalar.starts_with('f');
    // An open end is itself outside; a closed one is a step past it.
    let past = |end: f64, open: bool, up: bool| match (open, float) {
        (true, _) => end,
        (false, true) => adjacent(end, up),
        (false, false) if up => end + 1.0,
        (false, false) => end - 1.0,
    };
    let below = past(low, range.starts_with('('), false);
    let above = past(high, range.ends_with(')'), true);
    let (least, most) = match scalar {
        "u8" => (0.0, f64::from(u8::MAX)),
        "u16" => (0.0, f64::from(u16::MAX)),
        "u32" => (0.0, f64::from(u32::MAX)),
        "i32" => (f64::from(i32::MIN), f64::from(i32::MAX)),
        _ => (f64::MIN, f64::MAX),
    };
    let mut values: Vec<f64> = [below, above]
        .into_iter()
        .filter(|value| (least..=most).contains(value))
        .collect();
    if float {
        values.extend([f64::NAN, f64::INFINITY, f64::NEG_INFINITY]);
    }
    values
}

fn date(month: u8, day: u8) -> TsCalendarDate {
    sized(
        TsCalendarDate {
            struct_size: 0,
            calendar: Calendar::Gregorian.id(),
            era: 0xFFFF,
            year: 2026,
            era_year: 0,
            month,
            day,
            resolution: 0,
            computed_month: 0,
            computed_day: 0,
            leap: 0xFF,
            reserved: [0; 2],
        },
        |d, s| d.struct_size = s,
    )
}

fn civil(time: TsCivilTime) -> TsCivilDateTime {
    sized(
        TsCivilDateTime {
            struct_size: 0,
            reserved: 0,
            date: date(10, 23),
            time,
        },
        |c, s| c.struct_size = s,
    )
}

fn time() -> TsCivilTime {
    sized(
        TsCivilTime {
            struct_size: 0,
            hour: 6,
            minute: 15,
            second: 0,
            has_time: 1,
            nanos: 0,
        },
        |t, s| t.struct_size = s,
    )
}

/// What a call's status says: answered, or refused with the context's
/// record.
fn answered(ctx: &Ctx, status: Status) -> Result<(), Record> {
    if status == Status::Ok {
        Ok(())
    } else {
        Err(ctx.last_error())
    }
}

fn resolve(ctx: &Ctx, civil: &TsCivilDateTime, zone: &TsZoneSpec) -> Result<(), Record> {
    let mut resolution = sized(
        TsZoneResolution {
            struct_size: 0,
            offset_seconds: 0,
            dst_shift_seconds: 0,
            warnings: 0,
            instant_jd_utc: 0.0,
            tzdb_version: ptr::null(),
            abbreviation: ptr::null(),
            source: 0,
            era: 0,
            dst: 0,
            chosen: 0,
            time_known: 0,
            reserved: [0; 3],
        },
        |r, s| r.struct_size = s,
    );
    // SAFETY: a live context, valid requests and a valid slot.
    let status = unsafe {
        ts_time_resolve(
            ctx.handle,
            &raw const *civil,
            &raw const *zone,
            &raw mut resolution,
        )
    };
    answered(ctx, status)
}

/// Asks for a range of days with `field` set to `value`.
fn days(ctx: &Ctx, field: Option<&str>, value: f64) -> Result<(), Record> {
    let mut asked = sized(
        TsPanchangaRequest {
            struct_size: 0,
            calendar: Calendar::Gregorian.id(),
            sky_step_minutes: 0,
            from_year: 2026,
            from_month: 10,
            from_day: 23,
            to_month: 10,
            to_day: 23,
            to_year: 2026,
            latitude_deg: 27.7172,
            longitude_deg: 85.324,
            altitude_m: 1400.0,
            utc_offset_seconds: 20_700,
            sections: 0,
            muhurta_json: ptr::null(),
            festivals_json: ptr::null(),
            zone_json: ptr::null(),
        },
        |r, s| r.struct_size = s,
    );
    match field {
        None => {}
        Some("sky_step_minutes") => asked.sky_step_minutes = value as u16,
        Some("from_month") => asked.from_month = value as u8,
        Some("from_day") => asked.from_day = value as u8,
        Some("to_month") => asked.to_month = value as u8,
        Some("to_day") => asked.to_day = value as u8,
        Some("latitude_deg") => asked.latitude_deg = value,
        Some("longitude_deg") => asked.longitude_deg = value,
        Some("altitude_m") => asked.altitude_m = value,
        Some("utc_offset_seconds") => asked.utc_offset_seconds = value as i32,
        Some(other) => panic!("TsPanchangaRequest.{other} has a range and no setter here"),
    }
    let mut blob = TsBlob::empty();
    // SAFETY: a live context, a valid request and a valid slot.
    let status = unsafe { ts_panchanga_days(ctx.handle, &raw const asked, &raw mut blob) };
    // SAFETY: a descriptor the library wrote, or the empty one.
    unsafe { ts_blob_free(&raw mut blob) };
    answered(ctx, status)
}

/// Asks the entry point that reads `name` with `field` set to `value`,
/// or with nothing changed when `field` is `None`.
fn ask(ctx: &Ctx, name: &str, field: Option<&str>, value: f64) -> Result<(), Record> {
    let unknown = |field: &str| -> ! { panic!("{name}.{field} has a range and no setter here") };
    match name {
        "TsCalendarDate" => {
            let mut asked = date(10, 23);
            match field {
                None => {}
                Some("month") => asked.month = value as u8,
                Some("day") => asked.day = value as u8,
                Some(other) => unknown(other),
            }
            let mut fixed = 0i64;
            // SAFETY: a live context, a valid date and a valid slot.
            let status =
                unsafe { ts_calendar_to_fixed(ctx.handle, &raw const asked, &raw mut fixed) };
            answered(ctx, status)
        }
        "TsChartRequest" => {
            let instants = [2_460_483.2];
            let mut asked = chart_request(&instants, (27.7172, 85.324), 20_700);
            match field {
                None => {}
                Some("latitude_deg") => asked.latitude_deg = value,
                Some("longitude_deg") => asked.longitude_deg = value,
                Some("altitude_m") => asked.altitude_m = value,
                Some("utc_offset_seconds") => asked.utc_offset_seconds = value as i32,
                Some(other) => unknown(other),
            }
            chart_blob(ctx, &asked)
                .map(drop)
                .map_err(|_| ctx.last_error())
        }
        "TsCivilTime" => {
            let mut asked = time();
            match field {
                None => {}
                Some("hour") => asked.hour = value as u8,
                Some("minute") => asked.minute = value as u8,
                Some("second") => asked.second = value as u8,
                Some("nanos") => asked.nanos = value as u32,
                Some(other) => unknown(other),
            }
            let zone = sized(
                TsZoneSpec {
                    kind: TsZoneKind::Fixed as u8,
                    offset_seconds: 20_700,
                    ..TsZoneSpec::default()
                },
                |z, s| z.struct_size = s,
            );
            resolve(ctx, &civil(asked), &zone)
        }
        "TsZoneSpec" => {
            // Each field is read under the kind of zone it describes.
            let mut zone = sized(
                TsZoneSpec {
                    kind: TsZoneKind::Fixed as u8,
                    offset_seconds: 20_700,
                    longitude_deg: 85.324,
                    ..TsZoneSpec::default()
                },
                |z, s| z.struct_size = s,
            );
            match field {
                None => {}
                Some("offset_seconds") => zone.offset_seconds = value as i32,
                Some("longitude_deg") => {
                    zone.kind = TsZoneKind::LocalMean as u8;
                    zone.longitude_deg = value;
                }
                Some(other) => unknown(other),
            }
            resolve(ctx, &civil(time()), &zone)
        }
        "TsPanchangaRequest" => days(ctx, field, value),
        other => panic!("{other} has ranged fields and no entry point here"),
    }
}

#[test]
fn every_described_range_is_refused_outside_it_naming_the_field() {
    let structs = described();
    let named: BTreeSet<&str> = structs.keys().map(String::as_str).collect();
    let driven: BTreeSet<&str> = named
        .iter()
        .copied()
        .filter(|name| !PROVIDER_SIDE.contains(name))
        .collect();
    for side in PROVIDER_SIDE {
        assert!(
            named.contains(side),
            "{side} is listed as the provider's and has no ranged field"
        );
    }

    let ctx = Ctx::new(TS_CONTEXT_TEST_PROVIDER, None, None, None).unwrap();
    let (mut checked, mut faults) = (0usize, Vec::new());
    for name in &driven {
        if let Err(record) = ask(&ctx, name, None, 0.0) {
            panic!("{name}: the request every case starts from is refused: {record:?}");
        }
        for ranged in structs.get(*name).unwrap() {
            for value in outside(&ranged.scalar, &ranged.range) {
                checked += 1;
                let case = format!("{name}.{} = {value} ({})", ranged.field, ranged.range);
                match ask(&ctx, name, Some(&ranged.field), value) {
                    Ok(()) => faults.push(format!("{case}: answered")),
                    Err((status, message, field, ..)) => {
                        let stem = ranged
                            .field
                            .trim_end_matches("_deg")
                            .trim_end_matches("_m")
                            .trim_end_matches("_seconds");
                        if !matches!(status, Status::InvalidArg | Status::OutOfRange)
                            || !field.as_deref().is_some_and(|field| field.contains(stem))
                        {
                            faults.push(format!("{case}: {status:?} naming {field:?}: {message}"));
                        }
                    }
                }
            }
        }
    }
    println!(
        "{checked} values outside {} described ranges",
        structs.len()
    );
    assert!(
        faults.is_empty(),
        "{} of {checked} values outside a described range:\n{}",
        faults.len(),
        faults.join("\n")
    );
}

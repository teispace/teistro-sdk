//! What the corpus tests share: a recorded fixture, and the corpus's first
//! chart founded with the built-in ephemeris under `conformance-baseline`.

#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "each test binary uses what it needs of these, indexes the fixtures it reads, and fails by panicking"
)]

use serde_json::Value;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};

/// A file of the corpus's baseline directory.
pub(crate) fn fixture(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/baseline")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

/// The corpus's first chart (Kathmandu, 1990-04-14 05:30) read under
/// `conformance-baseline` with a settings patch, asking for what `ask` adds
/// to the request.
pub(crate) fn reading(
    patch: &str,
    ask: impl FnOnce(ChartRequest) -> ChartRequest,
) -> (Context, Document) {
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .settings_json(patch)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the conformance profile and the built-in ephemeris");
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    let request = ask(ChartRequest::at(
        place,
        UtcOffset::try_from_seconds(20_700).unwrap(),
    ));
    let document = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(2_447_995.489_583_333_5), &request)
        .expect("a reading")
        .value;
    (sdk, document)
}

/// Every recorded chart of the corpus's `charts` directory, in name order: its
/// file name and the chart as recorded.
pub(crate) fn charts() -> Vec<(String, Value)> {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/charts");
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .filter_map(|path| Some(path.file_name()?.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let chart = fixture(&format!("charts/{name}"));
            (name, chart)
        })
        .collect()
}

/// A recorded chart read again by the SDK under `conformance-baseline` from
/// the birth it records, asking for what `ask` adds to the request; the
/// SDK's refusal when it refuses.
pub(crate) fn reading_of(
    sdk: &Context,
    chart: &Value,
    ask: impl FnOnce(ChartRequest) -> ChartRequest,
) -> Result<Document, teistro::Error> {
    let input = &chart["input"];
    let place = &input["place"];
    let number = |value: &Value| value.as_f64().unwrap();
    let request = ask(ChartRequest::at(
        Place::new(
            Latitude::try_new(number(&place["latitude"])).unwrap(),
            Longitude::try_new(number(&place["longitude"])).unwrap(),
            Altitude::try_new(number(&place["altitude_m"])).unwrap(),
        ),
        UtcOffset::try_from_seconds(
            i32::try_from(input["resolved"]["tz_offset_min"].as_i64().unwrap() * 60).unwrap(),
        )
        .unwrap(),
    ));
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(number(&input["resolved"]["jd_ut"])),
            &request,
        )
        .map(|reading| reading.value)
}

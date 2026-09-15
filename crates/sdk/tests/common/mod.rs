//! What the corpus tests share: a recorded fixture, and the corpus's first
//! chart founded with the built-in ephemeris under `conformance-baseline`.

#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "each test binary uses what it needs of these, and fails by panicking"
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

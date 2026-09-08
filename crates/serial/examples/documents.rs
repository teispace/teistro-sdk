//! Sample chart documents, written as canonical JSON for a gate to read.
//!
//! `cargo xtask schema` runs this and measures what comes out, which is
//! how the JSON Schema for the document is **derived** rather than
//! proposed: a schema is a set of claims about a document's shape, and
//! the shapes here are the ones the chart layer really produces.
//!
//! Three of them, because the interesting claims are about what is
//! *optional*. Every section but the foundation carries a
//! `skip_serializing_if`, so a Rust `Option` and an absent JSON key do
//! not line up one to one and a schema's `required` cannot be read off
//! the struct:
//!
//! - `whole` — every section the layer can produce, on one chart.
//! - `day` — the foundation and the almanac of its day, which is what a
//!   panchanga application stores.
//! - `bare` — the foundation alone, the smallest document there is.
//!
//! Run by hand:
//!
//! ```sh
//! cargo run -p teistro-serial --example documents -- target/documents
//! ```

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::missing_panics_doc,
    reason = "an example fails by panicking and reports what it wrote"
)]

use teistro_aspect::Aspects;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::Gregorian;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::foundation::{ChartFoundation, Founder};
use teistro_core::catalogue::{Ayanamsha, ChartKind, Varga};
use teistro_core::envelope::{Envelope, Provenance};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{OverridePolicy, Profile, Settings, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_houses::Houses;
use teistro_panchanga::almanac::{Almanac, Panchanga};
use teistro_points::Points;
use teistro_port_ephemeris::test_provider::TestProvider;
use teistro_serial::Document;
use teistro_serial::canonical::to_hash_form;
use teistro_state::state;
use teistro_vargas::chart::{Axis, chart as varga_chart};

/// The instant every sample is founded on: a fixed one, so that two runs
/// write the same bytes and a gate can compare them.
pub const INSTANT: f64 = 2_460_482.5;

/// Kathmandu, where the corpus's own charts are cast.
#[must_use]
pub fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

/// The settings every sample resolves under: the shipped default
/// profile, unpatched.
#[must_use]
pub fn resolved() -> teistro_core::settings::Resolved {
    Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .expect("the default profile")
        .resolve(&SettingsPatch::default())
        .expect("it resolves")
}

/// The chart every sample is built from, with the settings it resolved
/// under, over the analytic test provider so this needs no ephemeris.
#[must_use]
pub fn founded() -> (Envelope<ChartFoundation>, Settings) {
    let provider = TestProvider;
    let settings = resolved();
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let founded = Founder::new(
        &provider,
        &settings,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    )
    .found_one(
        JulianDay::<Utc>::literal(INSTANT),
        &place(),
        ChartKind::Natal,
    )
    .expect("a founded chart");
    (founded, settings.settings)
}

/// The almanac of the founded chart's own day.
#[must_use]
pub fn almanac() -> Panchanga {
    let provider = TestProvider;
    let settings = resolved();
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    Almanac::new(
        &provider,
        &settings,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    )
    .at(JulianDay::<Utc>::literal(INSTANT), &place())
    .expect("an almanac")
    .value
}

/// Every section the layer can produce, on one chart.
#[must_use]
pub fn whole() -> (Document, Provenance) {
    let (envelope, settings) = founded();
    let provenance = envelope.provenance.clone();
    let foundation = envelope.value;
    let document = Document::of(foundation.clone())
        .with_panchanga(almanac())
        .with_varga(varga_chart(&foundation, Axis::of(Varga::D9)).expect("a navamsha"))
        .with_state(state(&foundation, &settings).expect("a state"))
        .with_aspects(Aspects::of(&foundation, &settings).expect("the aspects"))
        .with_points(Points::from_longitudes(&foundation).expect("the points"))
        .with_houses(Houses::of(&foundation).expect("the houses"));
    (document, provenance)
}

/// The samples, by the name each is written under.
#[must_use]
pub fn samples() -> Vec<(&'static str, Document)> {
    let (whole_document, _) = whole();
    let (envelope, _) = founded();
    vec![
        ("whole", whole_document),
        (
            "day",
            Document::of(envelope.value.clone()).with_panchanga(almanac()),
        ),
        ("bare", Document::of(envelope.value)),
    ]
}

/// Every number in a canonical document, by scanning its text.
///
/// The text rather than a parsed value, because the parser is the thing
/// under measurement here. The form has no whitespace and escapes a
/// quote as `\"`, and none of the strings it writes — catalogue keys,
/// member names, a model's description — carries one, so splitting on
/// the quote alternates between outside a string and inside it.
#[must_use]
pub fn numbers(text: &str) -> Vec<f64> {
    let mut found = Vec::new();
    for (index, outside) in text.split('"').enumerate() {
        if index % 2 == 1 {
            continue;
        }
        for token in outside.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')) {
            if let Ok(value) = token.parse::<f64>() {
                found.push(value);
            }
        }
    }
    found
}

fn main() {
    let into = std::env::args().nth(1).unwrap_or_else(|| {
        panic!("usage: cargo run -p teistro-serial --example documents -- <directory>")
    });
    let directory = std::path::Path::new(&into);
    std::fs::create_dir_all(directory).expect("the directory is writable");
    for (name, document) in samples() {
        let path = directory.join(format!("{name}.json"));
        let text = to_hash_form(&document);
        std::fs::write(&path, &text).expect("the sample is written");
        // And the canonical form of what that text parses to, which must
        // be the same bytes: the content hash rests on a consumer that
        // reads a stored document hashing it to the producer's hash. A
        // gate cannot check that without a second file, because only this
        // crate can write the form.
        // How the numbers fare through each of the two parsers, written
        // beside the document because only this crate can write the
        // form and a gate cannot recompute it.
        //
        // `str::parse` is correctly rounded, as JavaScript's
        // `JSON.parse` and Python's `json` are; `serde_json`'s own
        // number path is not, and lands a unit in the last place low
        // often enough to matter.
        let mut counted = 0_usize;
        let mut by_std = 0_usize;
        let mut by_serde = 0_usize;
        for value in numbers(&text) {
            counted += 1;
            let written = to_hash_form(&value);
            if written.parse::<f64>().map(f64::to_bits) != Ok(value.to_bits()) {
                by_std += 1;
            }

            let serde_bits = serde_json::from_str::<f64>(&written)
                .map(f64::to_bits)
                .unwrap_or_default();
            if serde_bits != value.to_bits() {
                by_serde += 1;
            }
        }
        let report = directory.join(format!("{name}.parsers.txt"));
        std::fs::write(&report, format!("{counted} {by_std} {by_serde}\n"))
            .expect("the parser report is written");
        println!(
            "{} ({} bytes, {counted} numbers, {by_std} moved by a correct parser, \
             {by_serde} by serde_json's)",
            path.display(),
            text.len(),
        );
    }
}

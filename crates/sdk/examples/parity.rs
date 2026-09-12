//! One scenario through the Rust surface, printed as the parity report:
//! `key<TAB>value` lines, sorted by key.
//!
//! `bindings/{node,dart,python}/parity.*` print the same scenario, and
//! `cargo xtask check-parity` compares them, so a difference between two
//! layers is a failed gate rather than something a reader has to notice.
//! This is the fourth runner.
//!
//! **It cannot print every key the other three do, and that is the
//! design rather than a gap.** Among theirs are `abi`, `build-commit`,
//! `build-target` and the result blobs' sections and hashes: a Rust
//! consumer has none of them, because Cargo resolved the versions,
//! `Drop` freed the memory, and the crates handed back their own types
//! instead of a blob (`03-design/rust-consumer-surface.md` §6). The
//! absences are declared in §7 and are what `check-parity` will hold
//! this report to once it is wired in — an inventory printed every run,
//! so an absence that stops being deliberate becomes a failure.
//!
//! Run it against the others by hand until then:
//!
//! ```sh
//! cargo run -p teistro --example parity > /tmp/rust.tsv
//! (cd bindings/node && node parity.mjs) > /tmp/node.tsv
//! join -t $'\t' /tmp/rust.tsv /tmp/node.tsv | awk -F'\t' '$2 != $3'
//! ```

#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    reason = "a report is printed, and a runner fails by panicking"
)]

use std::collections::BTreeMap;

use teistro::catalogue::{Calendar, Graha};
use teistro::{Body, CalendarDate, Context, Ephemeris, Frame, PositionRequest, Scale, TimeScale};
use teistro_core::envelope::CalendarResolution;
use teistro_time::{CivilDateTime, CivilTime, ZoneSpec};

/// A number as every binding spells it: nine decimals, never an
/// exponent, and an integer as an integer.
fn number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{value:.0}")
    } else {
        format!("{value:.9}")
    }
}

/// FNV-1a over UTF-8 bytes, so a JSON section can be compared without a
/// parser -- the same eight hex digits the other three print.
fn fnv(text: &str) -> String {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in text.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    format!("{hash:08x}")
}

/// A variant's name in kebab case, which is how every generated
/// catalogue spells one: `LeapSeconds` is `leap-seconds`.
fn kebab(variant: &str) -> String {
    let mut out = String::with_capacity(variant.len() + 2);
    for (at, letter) in variant.chars().enumerate() {
        if letter.is_ascii_uppercase() {
            if at != 0 {
                out.push('-');
            }
            out.extend(letter.to_lowercase());
        } else {
            out.push(letter);
        }
    }
    out
}

/// A variant's name in screaming snake case, which is how a detail is
/// spelled: `UnknownKey` is `UNKNOWN_KEY`.
fn screaming(variant: &str) -> String {
    kebab(variant).replace('-', "_").to_uppercase()
}

/// How a resolution's kind is spelled, which is the tag its JSON
/// carries.
fn resolution(of: &CalendarResolution) -> &'static str {
    match of {
        CalendarResolution::Defined => "defined",
        CalendarResolution::Tabular { .. } => "tabular",
        CalendarResolution::Computed { .. } => "computed",
        CalendarResolution::Divergent { .. } => "divergent",
    }
}

/// The report, as a map so the keys come out sorted whatever order the
/// sections are written in.
type Report = BTreeMap<String, String>;

/// One line of it.
fn put(report: &mut Report, key: &str, value: String) {
    report.insert(key.to_owned(), value);
}

/// The context itself, as the report prints it.
fn a_context(report: &mut Report, sdk: &Context) {
    put(report, "profile", sdk.profile().to_owned());
    put(report, "locale", sdk.intl().locale());
    put(report, "settings-hash", sdk.settings_hash().to_string());
    put(report, "settings-fnv", fnv(&sdk.settings_json()));
}

/// The calendars, as the report prints it.
fn the_calendars(report: &mut Report, sdk: &Context) {
    let date = CalendarDate::defined(Calendar::Gregorian, 2015, 4, 14);
    let bs = sdk
        .calendar()
        .convert(&date, Calendar::BikramSambat)
        .expect("a date inside the table");
    put(report, "bs-year", bs.year.to_string());
    put(report, "bs-month", bs.month.to_string());
    put(report, "bs-day", bs.day.to_string());
    let era = bs.era.expect("Bikram Sambat has an era");
    put(report, "bs-era", era.era.full_key().to_owned());
    put(report, "bs-era-year", era.year.to_string());
    put(
        report,
        "bs-resolution",
        resolution(&bs.resolution).to_owned(),
    );
    let fixed = sdk.calendar().fixed_of(&date).expect("a Gregorian date");
    put(report, "fixed", fixed.get().to_string());
    put(
        report,
        "weekday",
        (sdk.calendar().weekday_of(&date).expect("a Gregorian date") as u8).to_string(),
    );
    put(
        report,
        "month-length",
        sdk.calendar()
            .month_length(Calendar::Gregorian, 2024, 2)
            .expect("February 2024")
            .to_string(),
    );
    put(
        report,
        "is-leap",
        sdk.calendar()
            .is_leap(Calendar::Gregorian, 2024)
            .expect("a shipped calendar")
            .to_string(),
    );
}

/// Time, as the report prints it.
fn time(report: &mut Report, sdk: &Context) {
    let civil = CivilDateTime::at(
        CalendarDate::defined(Calendar::Gregorian, 1986, 1, 1),
        CivilTime::new(0, 20, 0).expect("a time of day"),
    );
    let zone = ZoneSpec::Iana {
        zone: String::from("Asia/Kathmandu"),
    };
    let resolved = sdk.time().resolve(&civil, &zone).expect("a known zone");
    put(report, "resolve-jd", number(resolved.instant.get()));
    put(
        report,
        "resolve-offset",
        resolved.zone.offset.seconds().to_string(),
    );
    put(
        report,
        "resolve-era",
        format!("{:?}", resolved.zone.era).to_lowercase(),
    );
    put(
        report,
        "resolve-source",
        format!("{:?}", resolved.zone.source).to_lowercase(),
    );
    put(
        report,
        "resolve-time-known",
        resolved.zone.time_known.to_string(),
    );
    put(report, "resolve-tzdb", resolved.zone.tzdb_version.clone());
    put(
        report,
        "resolve-warnings",
        resolved.zone.warnings.len().to_string(),
    );
    let (back, resolution_back) = sdk
        .time()
        .civil_of(resolved.instant, &zone, Calendar::Gregorian)
        .expect("a known zone");
    put(report, "civil-year", back.date.year.to_string());
    put(
        report,
        "civil-minute",
        back.time.expect("the time is known").minute().to_string(),
    );
    put(
        report,
        "civil-offset",
        resolution_back.offset.seconds().to_string(),
    );
    let tt = sdk
        .time()
        .convert(2_451_544.5, Scale::Utc, Scale::Tt)
        .expect("inside the model's range");
    put(report, "tt-jd", number(tt.jd));
    let applied = tt.delta_t.expect("UTC to TT needs a ΔT");
    put(report, "tt-delta-t", number(applied.seconds));
    put(
        report,
        "tt-delta-t-source",
        kebab(&format!("{:?}", applied.source)),
    );
    put(report, "tt-delta-t-model", applied.model.key().to_owned());
    let delta = sdk
        .time()
        .delta_t(teistro::quantity::JulianDay::try_new(2_451_544.5).expect("a Julian day"))
        .expect("inside the model's range");
    put(report, "delta-t-seconds", number(delta.seconds));
    put(
        report,
        "delta-t-source",
        kebab(&format!("{:?}", delta.source)),
    );
}

/// Keys, as the report prints it.
fn keys(report: &mut Report, sdk: &Context) {
    let id = sdk.keys().id("graha.SUN").expect("a catalogued key");
    put(report, "key-id", id.bits().to_string());
    put(report, "key-name", sdk.keys().name(id).expect("a live id"));
    let refusal = sdk.keys().id("graha.SUNN").expect_err("no such key");
    put(
        report,
        "refusal-status",
        format!("{:?}", refusal.status).to_lowercase(),
    );
    put(
        report,
        "refusal-detail",
        refusal.detail.map_or_else(
            || String::from("none"),
            |detail| screaming(&format!("{detail:?}")),
        ),
    );
    put(
        report,
        "refusal-hint-names-sun",
        refusal
            .hint()
            .is_some_and(|hint| hint.contains("SUN"))
            .to_string(),
    );
}

/// Positions, as the report prints it.
fn positions(report: &mut Report, sdk: &Context) {
    // The same grid the other three ask for: two instants by three
    // bodies, every cell printed, so a disagreement names the cell.
    let jds = [2_451_545.0, 2_451_546.0];
    let bodies = [Body::Sun, Body::Moon, Body::Mars];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
    let sky = sdk.positions(&request).expect("the test provider");
    put(report, "cells", sky.columns.len().to_string());
    put(
        report,
        "positions-scale",
        kebab(&format!("{:?}", request.scale)),
    );
    put(
        report,
        "positions-bodies",
        bodies
            .iter()
            .map(|body| kebab(&format!("{body:?}")))
            .collect::<Vec<_>>()
            .join(","),
    );
    for index in 0..sky.columns.len() {
        let instant = index / bodies.len();
        let body = index % bodies.len();
        let cell = sky.columns.at(instant, body).expect("a cell");
        put(report, &format!("cell-{index}-lon"), number(cell.lon));
        put(report, &format!("cell-{index}-lat"), number(cell.lat));
        put(report, &format!("cell-{index}-dist"), number(cell.dist));
        put(
            report,
            &format!("cell-{index}-lon-speed"),
            number(cell.lon_speed),
        );
        // The id, as the other three print it: a status is a number on
        // the boundary and every binding reports what it received.
        put(
            report,
            &format!("cell-{index}-status"),
            cell.status.code().to_string(),
        );
    }
    // Built here rather than from `step_keys`, which is the astronomy
    // crate's `{:?}` and gives `PassThrough` where the report wants
    // `PASS_THROUGH` -- the spelling every generated catalogue uses.
    put(
        report,
        "steps",
        sky.steps
            .iter()
            .map(|step| {
                format!(
                    "{}:{}",
                    step.name,
                    screaming(&format!("{:?}", step.implementation))
                )
            })
            .collect::<Vec<_>>()
            .join(","),
    );
}

/// The locale engine, as the report prints it.
fn the_locale(report: &mut Report, sdk: &Context) {
    let rendered = sdk
        .intl()
        .render_typed(&teistro::messages::sdk::reason::GrahaInBhava {
            graha: Graha::Jupiter,
            bhava: 7,
        });
    put(report, "render-fnv", fnv(&rendered.text));
    put(
        report,
        "render-length",
        rendered.text.chars().count().to_string(),
    );
    put(
        report,
        "render-resolved-from",
        rendered
            .resolved_from
            .clone()
            .unwrap_or_else(|| String::from("none")),
    );
    put(report, "render-fallback", rendered.is_fallback.to_string());
    put(
        report,
        "has-message",
        sdk.intl().has("sdk.reason.grahaInBhava").to_string(),
    );
    put(
        report,
        "has-missing-message",
        sdk.intl().has("sdk.nope.missing").to_string(),
    );
    let sun_entity = sdk.intl().entity("graha.SUN").expect("a catalogued graha");
    for form in ["name", "iast", "glyph"] {
        put(
            report,
            &format!("entity-sun-{form}"),
            sun_entity
                .form(form)
                .map_or_else(|| String::from("none"), str::to_owned),
        );
    }
    put(
        report,
        "entity-sun-gender",
        sun_entity
            .gender
            .clone()
            .unwrap_or_else(|| String::from("none")),
    );
    put(report, "message-graha-in-bhava", rendered.text.clone());
}

/// The frame, as the report prints it.
fn the_frame(report: &mut Report, sdk: &Context) {
    let canonical = sdk.frame().canonical();
    put(
        report,
        "frame-centre",
        kebab(&format!("{:?}", canonical.centre)),
    );
    put(
        report,
        "frame-coordinates",
        kebab(&format!("{:?}", canonical.coordinates)),
    );
    put(
        report,
        "frame-bits",
        sdk.frame().pack(canonical).to_string(),
    );
    put(
        report,
        "frame-round-trip",
        (sdk.frame()
            .unpack(sdk.frame().pack(canonical))
            .expect("its own bits")
            == canonical)
            .to_string(),
    );
}

fn main() {
    let mut report = Report::new();
    // The **test** provider, as the other three runners use: the
    // point is that four layers agree, not that they agree with the
    // sky.
    let sdk = Context::builder()
        .profile("nepali-default")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Test])
        .build()
        .expect("the shipped profile, locale and test provider");
    a_context(&mut report, &sdk);
    the_calendars(&mut report, &sdk);
    time(&mut report, &sdk);
    keys(&mut report, &sdk);
    positions(&mut report, &sdk);
    the_locale(&mut report, &sdk);
    the_frame(&mut report, &sdk);

    for (key, value) in &report {
        println!("{key}\t{value}");
    }
}

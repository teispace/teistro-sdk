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

use teistro::catalogue::{Calendar, ChartKind, ChartLayout, DashaSystem, Graha, Varga};
use teistro::eclipse::{LocalMoment, LunarHere, SolarHere, Visible};
use teistro::settings::SunriseConvention;
use teistro::{
    Body, CalendarDate, ChartRequest, Context, Ephemeris, Frame, PositionRequest, Scale, Script,
    TimeScale, Timeline,
};
use teistro::{DayState, LocalDay};
use teistro_core::envelope::Envelope;
use teistro_core::interval::Interval;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1, Utc};
use teistro_core::time::UtcOffset;
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

/// What every chart answers by rule, as the report prints it: the rules of the
/// set that held, by key, and the Pindayu the three spans give
/// (`03-design/rules-at-the-boundary.md`).
fn the_rules(report: &mut Report, by_rule: &[teistro::RulesReading<'_>]) {
    for (index, reading) in by_rule.iter().enumerate() {
        let keys: Vec<&str> = reading
            .present
            .iter()
            .map(|held| held.rule.key.as_str())
            .collect();
        put(
            report,
            &format!("chart-{index}-rules-present"),
            keys.join(","),
        );
        let pindayu = reading
            .longevity
            .as_ref()
            .map_or(0.0, |longevity| longevity.ayurdaya.pindayu.years);
        put(
            report,
            &format!("chart-{index}-rules-pindayu"),
            number(pindayu),
        );
    }
}

/// What every chart *says*, as the report prints it: both composers' plans,
/// every item rendered through the same locale engine the other three
/// bindings render it with (`03-design/plans-at-the-boundary.md`).
///
/// This is the only section compared on text rather than on numbers, so it
/// exercises the composers, the params shape and the locale engine in one
/// comparison. Rust composes through the façade exactly as the boundary
/// does, which is what makes the four comparable at all.
fn the_plans(
    report: &mut Report,
    sdk: &teistro::Context,
    documents: &[teistro::Document],
    by_rule: &[teistro::RulesReading<'_>],
) {
    for (index, document) in documents.iter().enumerate() {
        // The one runner that must name the composers: the other three read
        // a `plans` mapping the boundary filled and iterate whatever is in
        // it, while Rust composes. So this list is where a composer is
        // forgotten, and the parity gate is what says so — a key Node prints
        // and Rust does not is an unaccounted absence, not a silence.
        let composed = [
            (
                "placements",
                sdk.interpret()
                    .placements(document)
                    .expect("the placements"),
            ),
            (
                "readings",
                by_rule
                    .get(index)
                    .map_or_else(teistro::Plan::default, |reading| {
                        sdk.interpret().readings(reading)
                    }),
            ),
            (
                "strength",
                sdk.interpret()
                    .strength(document)
                    .unwrap_or_else(|_| teistro::Plan::default()),
            ),
            (
                "houses",
                sdk.interpret()
                    .houses(document)
                    .unwrap_or_else(|_| teistro::Plan::default()),
            ),
            (
                "positions",
                sdk.interpret()
                    .positions(document)
                    .unwrap_or_else(|_| teistro::Plan::default()),
            ),
            (
                "aspects",
                sdk.interpret()
                    .aspects(document)
                    .unwrap_or_else(|_| teistro::Plan::default()),
            ),
            (
                "conditions",
                sdk.interpret()
                    .conditions(document)
                    .unwrap_or_else(|_| teistro::Plan::default()),
            ),
            (
                "karakas",
                sdk.interpret()
                    .karakas(document)
                    .unwrap_or_else(|_| teistro::Plan::default()),
            ),
        ];
        for (composer, plan) in composed {
            put(
                report,
                &format!("chart-{index}-plan-{composer}-count"),
                plan.items.len().to_string(),
            );
            for (at, item) in plan.items.iter().enumerate() {
                let said = sdk.intl().render(&item.key, &item.params).text;
                put(
                    report,
                    &format!("chart-{index}-plan-{composer}-{at}"),
                    format!("{}: {said}", item.key),
                );
            }
        }
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
    put(report, "bs-resolution", wire_key(&bs.resolution));
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
    put(report, "jd-of-fixed", number(teistro::jd_of_fixed(fixed)));
    let (back, fraction) = teistro::fixed_of_jd(2_457_126.75);
    put(report, "fixed-of-jd", back.get().to_string());
    put(report, "fraction-of-jd", number(fraction));
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
    put(report, "resolve-era", wire_key(&resolved.zone.era));
    put(report, "resolve-source", wire_key(&resolved.zone.source));
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
    put(report, "tt-delta-t-source", wire_key(&applied.source));
    put(report, "tt-delta-t-model", applied.model.key().to_owned());
    let delta = sdk
        .time()
        .delta_t(teistro::quantity::JulianDay::try_new(2_451_544.5).expect("a Julian day"))
        .expect("inside the model's range");
    put(report, "delta-t-seconds", number(delta.seconds));
    put(report, "delta-t-source", wire_key(&delta.source));
}

/// Keys, as the report prints it.
fn keys(report: &mut Report, sdk: &Context) {
    let id = sdk.keys().id("graha.SUN").expect("a catalogued key");
    put(report, "key-id", id.bits().to_string());
    put(report, "key-name", sdk.keys().name(id).expect("a live id"));
    let refusal = sdk.keys().id("graha.SUNN").expect_err("no such key");
    put(report, "refusal-status", wire_key(&refusal.status));
    put(
        report,
        "refusal-detail",
        refusal
            .detail
            .map_or_else(|| String::from("none"), |detail| detail.key().to_owned()),
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
    let stamped = sdk.positions(&request).expect("the test provider");
    let sky = &stamped.value;
    put(report, "cells", sky.columns.len().to_string());
    put(report, "positions-scale", wire_key(&request.scale));
    put(
        report,
        "positions-bodies",
        bodies.iter().map(wire_key).collect::<Vec<_>>().join(","),
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
    // What the other three read off the positions blob's provenance
    // envelope, read off the same envelope: the boundary stamps its blob
    // with the façade's own function, so the canonical JSON is compared
    // whole as well as by the fields a consumer reads.
    let provenance = &stamped.provenance;
    put(
        report,
        "provenance-fnv",
        fnv(&teistro::canonical_json(provenance)),
    );
    put(report, "provenance-profile", provenance.profile.clone());
    put(
        report,
        "provenance-settings-hash",
        provenance.settings_hash.to_string(),
    );
    put(
        report,
        "provenance-provider-frame",
        provenance.provider.frame.clone(),
    );
    // `Implementation::key`, not `step_keys`, which is the astronomy
    // crate's `{:?}` and gives `PassThrough` where the report wants
    // `PASS_THROUGH` -- the spelling every generated catalogue uses.
    put(
        report,
        "steps",
        sky.steps
            .iter()
            .map(|step| format!("{}:{}", step.name, step.implementation.key()))
            .collect::<Vec<_>>()
            .join(","),
    );
}

/// A rendered message's parts as every binding's runner spells them.
fn part_shape(parts: &[teistro_intl::OutPart]) -> String {
    parts
        .iter()
        .map(|part| match part {
            teistro_intl::OutPart::Text(value) => format!("text:{value}"),
            teistro_intl::OutPart::Markup {
                kind,
                name,
                options,
            } => {
                let mut options: Vec<String> = options
                    .iter()
                    .map(|(name, value)| format!("{name}={value}"))
                    .collect();
                options.sort();
                let kind = format!("{kind:?}").to_lowercase();
                format!("{kind}:{name}({})", options.join(","))
            }
        })
        .collect::<Vec<_>>()
        .join("|")
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
    // A rendered message's parts, which is what a rich renderer walks.
    // `sdk.reason.lordship` is one of the two shipped messages carrying
    // `{#b}`; the plain one beside it holds every binding to the rule
    // that no markup means the one text part. Rust reads the parts from
    // the value rather than from a blob, which is exactly why it belongs
    // in the comparison: the two paths must agree.
    let rich = sdk
        .intl()
        .render_typed(&teistro::messages::sdk::reason::Lordship {
            graha: Graha::Jupiter,
            bhava: 5,
        });
    put(report, "render-rich-parts", part_shape(&rich.parts));
    put(report, "render-plain-parts", part_shape(&rendered.parts));
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
    put(
        report,
        "message-bs-date",
        sdk.intl()
            .render_typed(
                &teistro::messages::sdk::calendar::bikram_sambat::date::Long {
                    day: 1,
                    month_name: String::from("बैशाख"),
                    year: 2072,
                },
            )
            .text,
    );
    put(
        report,
        "transliterated",
        sdk.intl()
            .transliterate("सूर्य बृहस्पति", Script::Devanagari, Script::Iast)
            .expect("both scripts are shipped"),
    );
}

/// The frame, as the report prints it.
fn the_frame(report: &mut Report, sdk: &Context) {
    let canonical = sdk.frame().canonical();
    put(report, "frame-centre", wire_key(&canonical.centre));
    put(
        report,
        "frame-coordinates",
        wire_key(&canonical.coordinates),
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

/// One chart of the batch, as the report prints it.
///
/// A function of its own because the report is per chart, and because
/// two hundred lines in one `fn` is a section nobody reads.
fn one_chart(
    report: &mut Report,
    index: usize,
    chart: &teistro_chart::foundation::ChartFoundation,
) {
    put(
        report,
        &format!("chart-{index}-lagna"),
        number(chart.lagna_deg),
    );
    put(
        report,
        &format!("chart-{index}-day-lagna"),
        number(chart.day_lagna_deg),
    );
    put(
        report,
        &format!("chart-{index}-ayanamsha"),
        number(chart.zodiac.offset_deg),
    );
    put(
        report,
        &format!("chart-{index}-day-part"),
        wire_key(&chart.day.part),
    );
    put(
        report,
        &format!("chart-{index}-instant"),
        number(chart.instant.get()),
    );
    put(
        report,
        &format!("chart-{index}-day-elapsed"),
        number(chart.day.elapsed),
    );
    put_day(report, &format!("chart-{index}"), &chart.day.day);
    put(
        report,
        &format!("chart-{index}-ghati"),
        chart.timing.ishtakaal.ghati.to_string(),
    );
    put(
        report,
        &format!("chart-{index}-pala"),
        chart.timing.ishtakaal.pala.to_string(),
    );
    put(
        report,
        &format!("chart-{index}-vipala"),
        chart.timing.ishtakaal.vipala.to_string(),
    );
    put(
        report,
        &format!("chart-{index}-hora-number"),
        chart.timing.hora.number.to_string(),
    );
    put(
        report,
        &format!("chart-{index}-hora-lord"),
        chart.timing.hora.lord.full_key().to_owned(),
    );
    one_chart_placements(report, index, chart);
}

/// One chart's grahas and bhavas, which are the bulk of its rows.
fn one_chart_placements(
    report: &mut Report,
    index: usize,
    chart: &teistro_chart::foundation::ChartFoundation,
) {
    // Every graha and every bhava, because a per-chart section that
    // ran the loops the wrong way round would otherwise show as
    // nothing at all.
    for (at, graha) in chart.grahas.iter().enumerate() {
        let key = |what: &str| format!("chart-{index}-graha-{at}{what}");
        put(report, &key(""), graha.graha.full_key().to_owned());
        put(report, &key("-lon"), number(graha.longitude_deg));
        put(report, &key("-lat"), number(graha.latitude_deg));
        put(report, &key("-speed"), number(graha.speed_deg_per_day));
        put(
            report,
            &key("-retro"),
            (graha.speed_deg_per_day < 0.0).to_string(),
        );
        put(report, &key("-house"), graha.house.bhava.to_string());
        put(
            report,
            &key("-house-method"),
            graha.house.method.full_key().to_owned(),
        );
        put(
            report,
            &key("-placement"),
            graha.placement.bhava.to_string(),
        );
    }
    // Uranus, Neptune and Pluto, which the request asks beside the nine.
    for (at, outer) in chart.outer.iter().enumerate() {
        put(
            report,
            &format!("chart-{index}-outer-{at}"),
            format!(
                "{} {} {} {} {} {}",
                outer.graha.full_key(),
                number(outer.longitude_deg),
                number(outer.latitude_deg),
                number(outer.speed_deg_per_day),
                outer.house.bhava,
                outer.placement.bhava
            ),
        );
    }
    for at in 0..12_usize {
        let Some((madhya, sandhi)) = chart.houses.madhya.get(at).zip(chart.houses.sandhi.get(at))
        else {
            continue;
        };
        put(
            report,
            &format!("chart-{index}-house-{at}-madhya"),
            number(*madhya),
        );
        put(
            report,
            &format!("chart-{index}-house-{at}-sandhi"),
            number(*sandhi),
        );
        if let Some(chalit) = chart.chalit.madhya.get(at) {
            put(
                report,
                &format!("chart-{index}-chalit-{at}-madhya"),
                number(*chalit),
            );
        }
    }
}

/// Every operation this layer declares, listed.
///
/// The other three runners **probe** for each member and print `present`
/// or `missing`, because in those languages an operation that is not
/// there is a `undefined` a reader finds at run time. Here it is a
/// compile error: every one of these is called somewhere in this file or
/// in `crates/sdk/tests/surface.rs`, so an operation removed from the
/// façade takes the runner with it.
///
/// So this section is a **list** and not a probe, and its value is the
/// list: `check-areas`' property *every operation the layer declares is
/// listed by every parity runner* reads these rows against the
/// operations the Node layer wires, so an operation added to one
/// binding and forgotten in the Rust façade is a failed gate. That was
/// the reason the property was written and the reason it names the
/// runners rather than the bindings.
///
/// The rows are pairs, in the shape `check-areas` reads in all four
/// files: the path first and quoted, then what the runner prints for it.
/// Every value here is `present` for the reason above.
///
/// `(root).dispose` is **not** here and cannot be: a `Context` is
/// dropped. Listing it as `missing` would be a disagreement where §6 of
/// `03-design/rust-consumer-surface.md` intends an absence, so
/// `check-areas` carries it as this runner's one allowance.
fn the_surface(report: &mut Report) {
    for (path, state) in [
        ("(root).engine", "present"),
        ("(root).positions", "present"),
        ("(root).profile", "present"),
        ("(root).settings", "present"),
        ("(root).settings_hash", "present"),
        ("(root).settings_json", "present"),
        ("almanac.day", "present"),
        ("almanac.of", "present"),
        ("calendar.convert", "present"),
        ("calendar.date_of", "present"),
        ("calendar.fixed_of", "present"),
        ("calendar.is_leap", "present"),
        ("calendar.month_length", "present"),
        ("calendar.weekday_of", "present"),
        ("chart.layout", "present"),
        ("chart.found", "present"),
        ("chart.found_many", "present"),
        ("engine.call", "present"),
        ("engine.call_json", "present"),
        ("engine.manifest", "present"),
        ("engine.manifest_json", "present"),
        ("engine.names", "present"),
        ("engine.signature", "present"),
        ("frame.canonical", "present"),
        ("frame.pack", "present"),
        ("frame.unpack", "present"),
        ("intl.entity", "present"),
        ("intl.has", "present"),
        ("intl.load_pack", "present"),
        ("intl.locale", "present"),
        ("intl.messages", "present"),
        ("intl.render", "present"),
        ("intl.transliterate", "present"),
        ("keys.id", "present"),
        ("keys.name", "present"),
        ("time.civil_of", "present"),
        ("time.convert", "present"),
        ("time.delta_t", "present"),
        ("time.resolve", "present"),
    ] {
        put(report, &format!("surface.{path}"), state.to_owned());
    }
}

/// What a consumer knows without asking the library, which in Rust is
/// the versions and the default.
///
/// The other three read these off the boundary; here they are
/// **constants**, because Cargo resolved the graph and a `const` is what
/// a resolved graph looks like. `abi` and the `build-*` keys have no
/// counterpart at all and §6 says why.
fn the_constants(report: &mut Report) {
    put(report, "sdk", String::from(env!("CARGO_PKG_VERSION")));
    put(
        report,
        "catalogue-version",
        teistro::catalogue::SCHEMA_VERSION.to_string(),
    );
    put(
        report,
        "default-profile",
        teistro::settings::DEFAULT_PROFILE.to_owned(),
    );
}

/// A chart founded on a classical astronomy: the Surya Siddhanta by
/// name, the text's zodiac, places, Lagna and day
/// (`docs/03-design/classical-chart.md`), which every binding reaches
/// through the selector and must read back alike, deviation and all.
fn a_classical_chart(report: &mut Report, place: &Place, offset: UtcOffset) {
    let classical = Context::builder()
        .profile("surya-siddhanta")
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .expect("a context over the text");
    let text = classical
        .chart()
        .found(
            JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
            place,
            offset,
            ChartKind::Natal,
        )
        .expect("the text founds a chart at Kathmandu");
    put(report, "classical-steps", text.value.steps.join(","));
    put(report, "classical-lagna", number(text.value.lagna_deg));
    put(
        report,
        "classical-sunrise",
        number(text.value.day.day.sunrise.get()),
    );
    let deviation = text
        .provenance
        .deviation
        .expect("a classical chart deviates");
    put(
        report,
        "classical-deviation",
        format!("{}: {}", deviation.model, deviation.detail),
    );
    for (at, graha) in text.value.grahas.iter().enumerate() {
        put(
            report,
            &format!("classical-graha-{at}-lon"),
            number(graha.longitude_deg),
        );
    }
}

/// The chart the **topocentric** profile founds, as the report prints
/// it.
///
/// `nepali-default` is topocentric — inherited from the baseline engine,
/// and what every recorded chart in the corpus is. Until the
/// completion's centre step this could not found a chart at all, and the
/// refusal was what the bindings compared. Now the chart is, which is
/// the stronger comparison: the step runs per body, per instant, inside
/// the library, so four bindings agreeing on its output is four
/// bindings agreeing on the whole of it.
fn a_topocentric_chart(report: &mut Report, sdk: &Context, place: &Place, offset: UtcOffset) {
    let placed = sdk
        .chart()
        .found(
            JulianDay::<Utc>::literal(2_451_545.0),
            place,
            offset,
            ChartKind::Natal,
        )
        .expect("the test provider under a topocentric profile");
    put(report, "chart-under-topocentric", String::from("founded"));
    put(report, "topocentric-steps", placed.value.steps.join(","));
    put(report, "topocentric-lagna", number(placed.value.lagna_deg));
    for (at, graha) in placed.value.grahas.iter().enumerate() {
        put(
            report,
            &format!("topocentric-graha-{at}"),
            graha.graha.full_key().to_owned(),
        );
        put(
            report,
            &format!("topocentric-graha-{at}-lon"),
            number(graha.longitude_deg),
        );
        put(
            report,
            &format!("topocentric-graha-{at}-lat"),
            number(graha.latitude_deg),
        );
        put(
            report,
            &format!("topocentric-graha-{at}-speed"),
            number(graha.speed_deg_per_day),
        );
    }
}

/// A chart founded under the geocentric profile, as the report prints
/// it.
///
/// Everything here runs on `parashari-classical`, which is geocentric,
/// so the two centres are both exercised -- the same reason the other
/// three runners build a second context.
fn charts(report: &mut Report) -> (Context, Place, UtcOffset) {
    let geo = the_geo_context();
    put(report, "geo-profile", geo.profile().to_owned());
    put(report, "geo-settings-hash", geo.settings_hash().to_string());

    let place = Place::new(
        Latitude::try_new(27.7172).expect("a latitude"),
        Longitude::try_new(85.324).expect("a longitude"),
        Altitude::try_new(1400.0).expect("an altitude"),
    );
    let offset = UtcOffset::try_from_seconds(20700).expect("+05:45");
    // Two instants, so a per-chart section that ran charts-outermost the
    // wrong way round shows as the second chart's values in the first's
    // place rather than as nothing at all.
    let instants = [
        JulianDay::<Utc>::literal(2_460_482.5),
        JulianDay::<Utc>::literal(2_460_600.25),
    ];
    let asked = the_chart_request(place, offset, &geo);
    // The foundation alone, as the boundary founds a progressed chart.
    let bare = ChartRequest::at(place, offset).with_kind(ChartKind::Natal);
    // The text-written rules and the longevity readings, as the other three
    // ask for them, so the four agree on what every chart answers by rule.
    let rules = teistro::RuleRequest::shipped([teistro::ShippedRules::Nabhasas])
        .with_longevity()
        .rule_set()
        .expect("a valid set");
    let answered = geo
        .chart()
        .readings_with_rules(&instants, &asked, &rules)
        .expect("the test provider");
    let own: Vec<teistro::Hash> = answered.value.iter().map(teistro::content_hash).collect();
    let (documents, by_rule): (Vec<_>, Vec<_>) = answered.value.into_iter().unzip();
    let read = Envelope::new(documents, answered.provenance);
    put(
        report,
        "chart-varga-count",
        read.value
            .first()
            .map_or(0, |document| document.vargas.len())
            .to_string(),
    );
    put(
        report,
        "chart-drishti-table",
        read.value
            .first()
            .and_then(|d| d.aspects.as_ref())
            .map_or_else(String::new, |a| a.table().to_owned()),
    );
    the_rules(report, &by_rule);
    the_plans(report, &geo, &read.value, &by_rule);
    for (index, document) in read.value.iter().enumerate() {
        one_document(report, &geo, index, document);
        the_progressions(report, &geo, index, document, &bare);
        the_western_aspects(report, &geo, index, document);
        the_parallels(report, &geo, index, document);
        the_antiscia(report, &geo, index, document);
        the_midpoints(report, &geo, index, document);
    }
    the_synastry(report, &geo, &read.value, offset);
    // **One call, as the other three make one.** The foundations are the
    // reading's own, and the provenance below is the reading's too --
    // which is what the blob carries, and what made this row disagree
    // when it was still hashing a separate `found_many`'s envelope.
    let founded = Envelope::new(
        read.value
            .iter()
            .map(|document| document.foundation.clone())
            .collect::<Vec<_>>(),
        read.provenance.clone(),
    );
    put(report, "chart-count", founded.value.len().to_string());
    put(report, "chart-kind", ChartKind::Natal.full_key().to_owned());
    put(report, "chart-place-lat", number(place.latitude.get()));
    put(report, "chart-place-lon", number(place.longitude.get()));
    // The batch's own rows: what the other three read off the blob's
    // header, here read off the first chart, because a batch shares them
    // by construction rather than by a header saying so.
    if let Some(first) = founded.value.first() {
        put(report, "chart-steps", first.steps.join(","));
        put(report, "chart-model-fnv", fnv(&first.day.day.model));
        put(report, "chart-graha-count", first.grahas.len().to_string());
    }
    // The provenance as the boundary seals it: the same canonical JSON,
    // so the same eight hex digits. Not a re-encoding of a decoded
    // envelope -- that was a real disagreement in two bindings once.
    put(
        report,
        "chart-provenance-fnv",
        fnv(&teistro::canonical_json(&founded.provenance)),
    );
    put(
        report,
        "chart-provenance-profile",
        founded.provenance.profile.clone(),
    );
    for (index, chart) in founded.value.iter().enumerate() {
        one_chart(report, index, chart);
    }
    own_hashes(report, "chart", &own);

    // `found` is the batch of one unwrapped, and must agree with the
    // batch -- which is the property the other three assert too.
    let single = geo
        .chart()
        .found(instants[0], &place, offset, ChartKind::Natal)
        .expect("the test provider");
    put(report, "chart-single-lagna", number(single.value.lagna_deg));
    put(
        report,
        "chart-single-agrees",
        founded
            .value
            .first()
            // Bit for bit: `found` is the same computation as the batch
            // of one, so anything but an identical number would mean the
            // convenience had taken a different path.
            .is_some_and(|first| first.lagna_deg.to_bits() == single.value.lagna_deg.to_bits())
            .to_string(),
    );
    (geo, place, offset)
}

/// One chart's divisional charts, as the report prints them.
///
/// The grahas are named from the **foundation's** own list rather than
/// from the divisional chart's, because the two are the same list in the
/// same order and the other three bindings read the name from the
/// foundation's column: a runner that read it from its own section would
/// agree with them and prove less.
/// The context the charts are read under, with what the consumer registers
/// on it: the South Indian layout renamed (`03-design/chart-geometry.md`
/// §7f) and a dasha system of its own, as every runner registers them.
fn the_geo_context() -> Context {
    let mut kerala = teistro::geometry::rows::south_indian();
    kerala.key = String::from("ACME_KERALA");
    Context::builder()
        .profile("parashari-classical")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Test])
        .layout(kerala)
        .dasha_system(parity_dasha())
        .build()
        .expect("a shipped profile")
}

/// The dasha system of the consumer's own every runner registers: a
/// backward count, a two-nakshatra window, an offset, a savana year and a
/// depth of two, so each field a definition may set crosses
/// (`03-design/dasha-kernels.md`).
/// How many annual charts the parity report prints for each chart.
/// Twelve is enough for the three readings to separate visibly and short
/// enough to keep the report a report.
const PARITY_YEARS: u16 = 12;

const PARITY_DASHA: &str = r#"{"kernel":"UDU","key":"ACME_PARITY","sources":["the parity scenario"],"lords":[{"graha":"SUN","years":5},{"graha":"MOON","years":10},{"graha":"MARS","years":7},{"graha":"MERCURY","years":12}],"reference":"MULA","count":"TO_REFERENCE","span":2,"offset":1,"repeats":true,"year_length":"SAVANA_360","depth":2}"#;

fn parity_dasha() -> teistro::dasha::UduDefinition {
    serde_json::from_str(PARITY_DASHA).expect("the parity definition")
}

/// The request every runner makes: two divisional charts, six dashas (one
/// a time lord of each Hellenistic kernel, one the consumer's own), four
/// drawings and every section.
fn the_chart_request(place: Place, offset: UtcOffset, geo: &Context) -> ChartRequest {
    let kerala = geo
        .keys()
        .id("chart_layout.ACME_KERALA")
        .expect("registered");
    let own = geo
        .keys()
        .id("dasha_system.ACME_PARITY")
        .expect("registered");
    // Two divisional charts asked for, and two rather than one because
    // the layout the other three decode is charts outermost then charts
    // asked for: only two of each can catch a transposed stride.
    ChartRequest::at(place, offset)
        .with_kind(ChartKind::Natal)
        .with_vargas([Varga::D9, Varga::D10])
        .with_dashas([
            DashaSystem::Vimshottari.key_id(),
            DashaSystem::Chara.key_id(),
            DashaSystem::Kalachakra.key_id(),
            DashaSystem::ReleasingFortune.key_id(),
            DashaSystem::Profection.key_id(),
            DashaSystem::Firdaria.key_id(),
            DashaSystem::Decennials.key_id(),
            own,
        ])
        // The other runners' `lots` record sets the rules releasing reads
        // its lots under, so this request carries the same.
        .with_lot_rules(teistro::LotRequest::from_json(LOTS_JSON).expect("a valid request"))
        .with_drawings([
            (ChartLayout::NorthIndian.key_id(), Varga::D1),
            (ChartLayout::SouthIndian.key_id(), Varga::D9),
            (ChartLayout::WesternWheel.key_id(), Varga::D1),
            (kerala, Varga::D9),
        ])
        .with_aspects()
        .with_points()
        .with_houses()
        .with_ashtakavarga()
        .with_vimshopaka()
        .with_vaiseshikamsa()
        .with_dasha_phala()
        .with_jaimini()
        .with_outer_planets()
        .with_shadbala()
        .with_bhava_bala()
        .with_state()
}

/// Every drawing the request named: its layout and chart, and each cell's
/// sign, house, anchors, outline start and the kinds of its steps, and each
/// mark, as the other three print them.
fn the_drawings(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    use teistro::geometry::Point;
    let pair = |point: Point| format!("{},{}", number(point.x), number(point.y));
    for (d, drawing) in document.drawings.iter().enumerate() {
        let key = format!("chart-{index}-drawing-{d}");
        let placed = &drawing.placed;
        put(report, &key, format!("chart_layout.{}", placed.layout));
        put(
            report,
            &format!("{key}-varga"),
            drawing.varga.full_key().to_owned(),
        );
        // Written as SVG in the dark theme, as the other three ask for.
        let svg = sdk
            .chart()
            .svg(document, d, &teistro::render_svg::Theme::dark())
            .unwrap_or_else(|error| format!("refused: {error}"));
        put(report, &format!("{key}-svg"), svg);
        put(
            report,
            &format!("{key}-cells"),
            placed.cells.len().to_string(),
        );
        put(
            report,
            &format!("{key}-frames"),
            placed.frame.len().to_string(),
        );
        put(
            report,
            &format!("{key}-marks"),
            placed.marks.len().to_string(),
        );
        for (c, cell) in placed.cells.iter().enumerate() {
            let at = format!("{key}-cell-{c}");
            put(
                report,
                &format!("{at}-sign"),
                cell.sign.full_key().to_owned(),
            );
            put(report, &format!("{at}-house"), cell.house.to_string());
            put(report, &format!("{at}-lagna"), cell.lagna.to_string());
            put(report, &format!("{at}-ring"), cell.ring.to_string());
            let bodies: Vec<String> = cell.bodies.iter().map(ToString::to_string).collect();
            put(
                report,
                &format!("{at}-bodies"),
                if bodies.is_empty() {
                    String::from("none")
                } else {
                    bodies.join(",")
                },
            );
            put(report, &format!("{at}-label"), pair(cell.label));
            put(report, &format!("{at}-anchor"), pair(cell.anchor));
            put(report, &format!("{at}-start"), pair(cell.outline.start));
            let steps: Vec<String> = cell.outline.segments.iter().map(wire_key).collect();
            put(report, &format!("{at}-steps"), steps.join(","));
        }
        for (m, mark) in placed.marks.iter().enumerate() {
            let at = format!("{key}-mark-{m}");
            put(report, &at, mark.body.to_string());
            put(report, &format!("{at}-at"), pair(mark.at));
            put(report, &format!("{at}-lon"), number(mark.longitude_deg));
        }
    }
}

fn one_varga_chart(report: &mut Report, index: usize, document: &teistro::Document) {
    for (at, varga) in document.vargas.iter().enumerate() {
        let key = |what: &str| format!("chart-{index}-varga-{at}{what}");
        put(
            report,
            &key(""),
            varga
                .axis
                .grahas
                .varga
                .map_or_else(|| String::from("none"), |v| v.full_key().to_owned()),
        );
        put(
            report,
            &key("-lagna-rashi"),
            varga.lagna.rashi.full_key().to_owned(),
        );
        put(report, &key("-lagna-part"), varga.lagna.part.to_string());
        put(
            report,
            &key("-lagna-sign"),
            varga.lagna.sign.full_key().to_owned(),
        );
        for (j, placed) in varga.grahas.iter().enumerate() {
            let row = |what: &str| format!("chart-{index}-varga-{at}-graha-{j}{what}");
            put(
                report,
                &row(""),
                document
                    .foundation
                    .grahas
                    .get(j)
                    .map_or_else(|| String::from("none"), |g| g.graha.full_key().to_owned()),
            );
            put(
                report,
                &row("-rashi"),
                placed.at.rashi.full_key().to_owned(),
            );
            put(report, &row("-part"), placed.at.part.to_string());
            put(report, &row("-sign"), placed.at.sign.full_key().to_owned());
        }
    }
}

/// A graha's Sayanadi as the runners spell it: the state, then its five
/// sub-states, or `none`.
fn sayanadi(sayanadi: Option<teistro::Sayanadi>) -> String {
    sayanadi.map_or_else(
        || String::from("none"),
        |s| {
            let cheshtas: Vec<&str> = s.cheshtas.iter().map(|c| c.full_key()).collect();
            format!("{} {}", s.avastha.full_key(), cheshtas.join(","))
        },
    )
}

/// One chart's planetary states, as the report prints them.
fn the_states(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(states) = document.state.as_ref() else {
        return;
    };
    let members = |set: &[teistro::catalogue::AvasthaLajjitadi]| {
        if set.is_empty() {
            String::from("none")
        } else {
            set.iter()
                .map(|m| m.full_key().to_owned())
                .collect::<Vec<_>>()
                .join(",")
        }
    };
    let or_none = |value: Option<f64>| value.map_or_else(|| String::from("none"), number);
    for (at, state) in states.iter().enumerate() {
        let key = |what: &str| format!("chart-{index}-state-{at}{what}");
        put(report, &key(""), state.graha.full_key().to_owned());
        put(report, &key("-sign"), state.sign.full_key().to_owned());
        put(report, &key("-house"), state.house.to_string());
        put(
            report,
            &key("-dignity"),
            state.dignity.full_key().to_owned(),
        );
        put(
            report,
            &key("-natural"),
            state.friendship.natural.full_key().to_owned(),
        );
        put(
            report,
            &key("-compound"),
            state.friendship.compound.full_key().to_owned(),
        );
        put(
            report,
            &key("-dispositor"),
            state
                .friendship
                .dispositor
                .map_or_else(|| String::from("none"), |g| g.full_key().to_owned()),
        );
        put(
            report,
            &key("-burning"),
            wire_key(&state.combustion.burning),
        );
        put(
            report,
            &key("-from-sun"),
            or_none(state.combustion.from_sun_deg),
        );
        put(
            report,
            &key("-orb"),
            or_none(state.combustion.orbs.map(|o| o.orb_deg)),
        );
        put(report, &key("-age"), state.age.full_key().to_owned());
        put(
            report,
            &key("-wakefulness"),
            state.wakefulness.full_key().to_owned(),
        );
        put(
            report,
            &key("-deeptadi"),
            state
                .deeptadi
                .map_or_else(|| String::from("none"), |d| d.full_key().to_owned()),
        );
        put(report, &key("-holding"), members(&state.lajjitadi.holding));
        put(
            report,
            &key("-undecided"),
            members(&state.lajjitadi.undecided),
        );
        put(
            report,
            &key("-war"),
            state.war.map_or_else(
                || String::from("none"),
                |w| format!("{}:{}", w.opponent.full_key(), w.is_winner),
            ),
        );
        put(report, &key("-sayanadi"), sayanadi(state.sayanadi));
        put(
            report,
            &key("-sign-edge"),
            number(state.boundaries.sign_deg),
        );
    }
}

/// One chart's bhavas as the houses service reads them.
fn the_bhavas(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(houses) = document.houses.as_ref() else {
        return;
    };
    for number in 1..=12_u8 {
        let Some(bhava) = houses.bhava(number) else {
            continue;
        };
        let key = |what: &str| format!("chart-{index}-bhava-{number}{what}");
        put(report, &key("-sign"), bhava.sign.full_key().to_owned());
        put(report, &key("-lord"), bhava.lord.full_key().to_owned());
        put(report, &key("-quadrant"), wire_key(&bhava.quadrant));
    }
}

/// One chart's derived points, as the report prints them.
/// The annual charts a birth opens: the Sun's returns to where it stood,
/// under all three readings, as the other three runners print them.
///
/// All three, because the point of naming a reading is that a consumer can
/// ask for the one they mean — and a reading that crossed as another would
/// be invisible in a report that only ever printed the default.
///
/// Each reading also asks the sixteen yogas, the sahams and the annual
/// dashas a different way, so every way crosses: every matter, saham and
/// annual dasha under the sources' readings; every matter under Tambira's
/// "some authorities", every saham under each rival rule and every annual
/// dasha under a rival clock, balance and birth period, three levels
/// deep; and none of them at all.
fn the_praveshas(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let either = teistro::YogaRules {
        tambira: teistro::TambiraMover::EitherLord,
        ..teistro::YogaRules::default()
    };
    let rivals = teistro::SahamRules {
        add_sign: teistro::AddSign::Signs,
        houses: teistro::HousePoints::Equal,
        roga: teistro::RogaReading::Saturn,
    };
    let rival_dashas = teistro::AnnualDashaRules {
        clock: teistro::YearClock::Even,
        balance: teistro::MuddaBalance::EntryMoon,
        birth_period: teistro::settings::BirthPeriod::Elapsed,
        depth: teistro::quantity::Depth::try_new(3).expect("three levels"),
        ..teistro::AnnualDashaRules::default()
    };
    for (reading, asked) in [
        (
            teistro::VarshaReading::Sidereal,
            Asked {
                matters: Some(teistro::YogaRules::default()),
                sahams: Some(teistro::SahamRules::default()),
                dashas: Some(teistro::AnnualDashaRules::default()),
            },
        ),
        (
            teistro::VarshaReading::Tropical,
            Asked {
                matters: Some(either),
                sahams: Some(rivals),
                dashas: Some(rival_dashas),
            },
        ),
        (teistro::VarshaReading::Mean, Asked::default()),
    ] {
        let name = wire_key(&reading);
        let Ok(years) = sdk.chart().praveshas(document, reading, PARITY_YEARS) else {
            continue;
        };
        let key = |what: &str| format!("chart-{index}-varsha-{name}{what}");
        put(report, &key("-count"), years.len().to_string());
        the_natal_sahams(report, sdk, document, &key, asked.sahams);
        for one in &years {
            the_year(report, sdk, document, &key, one, asked);
        }
    }
}

/// What one reading asks of each year beside its chart, each under its
/// rules; none, not asked.
#[derive(Clone, Copy, Default)]
struct Asked {
    matters: Option<teistro::YogaRules>,
    sahams: Option<teistro::SahamRules>,
    dashas: Option<teistro::AnnualDashaRules>,
}

/// One year's Muntha, or false when there is none to read and so no year.
///
/// The Muntha is progressed by the year's own count and by nothing the
/// reading decides, so recording it under each of the three holds that
/// independence across all four runners as well as holding the bindings
/// to one another.
fn the_muntha(
    report: &mut Report,
    sdk: &Context,
    document: &teistro::Document,
    key: &dyn Fn(&str) -> String,
    one: &teistro::Pravesha,
) -> bool {
    let Ok(muntha) = sdk
        .chart()
        .muntha(document, one.year, teistro::MunthaDegree::default())
    else {
        return false;
    };
    put(
        report,
        &key(&format!("-{}-muntha", one.year)),
        muntha.sign.full_key().to_owned(),
    );
    put(
        report,
        &key(&format!("-{}-muntha-lord", one.year)),
        muntha.lord.full_key().to_owned(),
    );
    put(
        report,
        &key(&format!("-{}-muntha-deg", one.year)),
        number(muntha.longitude_deg),
    );
    true
}

/// One year of one reading: its instant, its Muntha, its own chart at the
/// birthplace, that chart's office-bearers and the lord of the year with
/// every claim it was chosen over.
fn the_year(
    report: &mut Report,
    sdk: &Context,
    document: &teistro::Document,
    key: &dyn Fn(&str) -> String,
    one: &teistro::Pravesha,
    asked: Asked,
) {
    put(
        report,
        &key(&format!("-{}", one.year)),
        number(one.at.get()),
    );
    if !the_muntha(report, sdk, document, key, one) {
        return;
    }
    // The year's own chart at the birthplace, as `"place":"birth"`
    // founds it at the boundary, and its office-bearers.
    let at_birth = ChartRequest::at(
        document.foundation.place,
        UtcOffset::try_from_seconds(20700).expect("+05:45"),
    );
    let Ok(annual) = sdk.chart().reading(one.at, &at_birth) else {
        return;
    };
    let Ok(bearers) = sdk
        .chart()
        .office_bearers(document, &annual.value, one.year)
    else {
        return;
    };
    put(
        report,
        &key(&format!("-{}-annual-lagna", one.year)),
        number(annual.value.foundation.lagna_deg),
    );
    put(
        report,
        &key(&format!("-{}-annual-by-day", one.year)),
        bearers.by_day.to_string(),
    );
    put(
        report,
        &key(&format!("-{}-annual-bearers", one.year)),
        teistro::Office::ALL
            .map(|office| bearers.holder(office).full_key())
            .join(" "),
    );
    // The lord of that year, and the reckoning it came out of.
    let Ok(lord) = sdk.chart().varshesha(
        document,
        &annual.value,
        one.year,
        teistro::VarsheshaRules::default(),
    ) else {
        return;
    };
    put(
        report,
        &key(&format!("-{}-year-lord", one.year)),
        lord.graha.full_key().to_owned(),
    );
    put(
        report,
        &key(&format!("-{}-year-lord-chosen", one.year)),
        wire_key(&lord.chosen),
    );
    put(
        report,
        &key(&format!("-{}-year-lord-bala", one.year)),
        lord.vishwa.to_string(),
    );
    the_yogas(report, sdk, &annual.value, key, one);
    the_matters(report, sdk, &annual.value, key, one, asked.matters);
    the_sahams(
        report,
        sdk,
        &annual.value,
        key,
        one,
        asked.sahams,
        lord.graha,
    );
    the_harsha(report, sdk, &annual.value, key, one);
    the_annual_dashas(
        report,
        sdk,
        (document, &annual.value),
        key,
        one,
        asked.dashas,
    );
    put(
        report,
        &key(&format!("-{}-year-claims", one.year)),
        lord.claims
            .iter()
            .map(|claim| {
                format!(
                    "{}:{}:{}:{}",
                    claim.graha.full_key(),
                    claim.vishwa,
                    claim.portfolios,
                    claim.aspects_lagna
                )
            })
            .collect::<Vec<String>>()
            .join(" "),
    );
}

/// Every saham of one year's chart under one set of rules, when the
/// reading asked for them, with its strength under the year's lord.
fn the_sahams(
    report: &mut Report,
    sdk: &Context,
    annual: &teistro::Document,
    key: &dyn Fn(&str) -> String,
    one: &teistro::Pravesha,
    rules: Option<teistro::SahamRules>,
    year_lord: teistro::catalogue::Graha,
) {
    let Some(rules) = rules else {
        return;
    };
    let Ok(read) = sdk.chart().saham_strength_with_rules(
        annual,
        &teistro::Saham::ALL,
        Some(year_lord),
        teistro::SahamStrengthRules {
            sahams: rules,
            ..teistro::SahamStrengthRules::default()
        },
    ) else {
        return;
    };
    for point in &read {
        put(
            report,
            &key(&format!("-{}-saham-{}", one.year, wire_key(&point.saham))),
            saham_said(point),
        );
    }
}

/// A birth's own sahams under one set of rules, when the reading asked
/// for them: the same line a year's saham prints.
fn the_natal_sahams(
    report: &mut Report,
    sdk: &Context,
    birth: &teistro::Document,
    key: &dyn Fn(&str) -> String,
    rules: Option<teistro::SahamRules>,
) {
    let Some(rules) = rules else {
        return;
    };
    let Ok(read) = sdk.chart().saham_strength_with_rules(
        birth,
        &teistro::Saham::ALL,
        None,
        teistro::SahamStrengthRules {
            sahams: rules,
            ..teistro::SahamStrengthRules::default()
        },
    ) else {
        return;
    };
    for point in &read {
        put(
            report,
            &key(&format!("-natal-saham-{}", wire_key(&point.saham))),
            saham_said(point),
        );
    }
}

/// Every annual dasha of one year under one set of rules, when the
/// reading asked for them: its seed, ring and year on one line and its
/// periods on another, each as text so the four runners round alike.
fn the_annual_dashas(
    report: &mut Report,
    sdk: &Context,
    (natal, annual): (&teistro::Document, &teistro::Document),
    key: &dyn Fn(&str) -> String,
    one: &teistro::Pravesha,
    rules: Option<teistro::AnnualDashaRules>,
) {
    let Some(rules) = rules else {
        return;
    };
    let Ok(dashas) = sdk.chart().annual_dashas(
        natal,
        annual,
        one.year,
        &teistro::tajika::ANNUAL_DASHAS,
        rules,
    ) else {
        return;
    };
    for dasha in &dashas {
        let at = key(&format!("-{}-dasha-{}", one.year, dasha.system.full_key()));
        let ring = &dasha.ring;
        let shares: Vec<String> = ring
            .ring
            .iter()
            .map(|share| {
                format!(
                    "{}/{}/{:.3}",
                    share.lord.full_key(),
                    share.sign.map_or("-", |sign| sign.full_key()),
                    share.weight
                )
            })
            .collect();
        put(
            report,
            &at,
            format!(
                "{} {} {} {:.9} {:.9} | {}",
                dasha.seed.map_or("-", |seed| seed.full_key()),
                ring.first,
                ring.remaining
                    .map_or_else(|| String::from("null"), |left| format!("{left:.9}")),
                dasha.year.from.get(),
                dasha.year.to.get(),
                shares.join(" "),
            ),
        );
        let periods: Vec<String> = dasha
            .periods
            .iter()
            .map(|period| {
                format!(
                    "{}:{}:{}:{:.9}:{:.9}",
                    period.path,
                    period.lord.full_key(),
                    period.sign.map_or("-", |sign| sign.full_key()),
                    period.interval.from.get(),
                    period.interval.to.get(),
                )
            })
            .collect();
        put(report, &format!("{at}-periods"), periods.join(" "));
    }
}

/// One year's Harsha bala, the seven in the catalogue's order.
fn the_harsha(
    report: &mut Report,
    sdk: &Context,
    annual: &teistro::Document,
    key: &dyn Fn(&str) -> String,
    one: &teistro::Pravesha,
) {
    let Ok(seven) = sdk.chart().harsha(annual) else {
        return;
    };
    let said: Vec<String> = seven
        .iter()
        .map(|h| {
            format!(
                "{}:{}:{}{}{}{}:{}:{}",
                h.graha.full_key(),
                h.house.get(),
                u8::from(h.sthana),
                u8::from(h.uchcha_swakshetra),
                u8::from(h.stri_purusha),
                u8::from(h.dina_ratri),
                h.total.units(),
                wire_key(&h.grade),
            )
        })
        .collect();
    put(
        report,
        &key(&format!("-{}-harsha", one.year)),
        said.join(" "),
    );
}

/// A member as every binding keys it, which is serde's own spelling: a
/// unit variant's string, or a tagged one's `kind` (`LEAP_SECONDS`,
/// `TABULAR`).
/// The instants every runner reads the transits at, counted from the Moon.
const GOCHAR_INSTANTS: [f64; 2] = [2_460_676.5, 2_460_736.5];

/// The transits as the other three print them: each reading's instant,
/// reference and node readings, and each graha's transit, house, vedha,
/// obstructors, verdict and fruition.
fn the_gochar(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let asked = teistro::GocharRequest::over(
        GOCHAR_INSTANTS.map(teistro::quantity::JulianDay::<teistro::quantity::Utc>::literal),
    )
    .with_ashtakavarga();
    let readings = sdk
        .chart()
        .gochar(document, &asked)
        .expect("the test provider")
        .value;
    for (at, (reading, instant)) in readings.iter().zip(GOCHAR_INSTANTS).enumerate() {
        put(
            report,
            &format!("chart-{index}-gochar-{at}"),
            format!(
                "{} {} {} {} {} {}",
                number(instant),
                wire_key(&reading.reference.from),
                reading.reference.sign.full_key(),
                wire_key(&reading.rules.node_vedha),
                wire_key(&reading.rules.node_obstruction),
                wire_key(&reading.rules.ashtakavarga_good_from),
            ),
        );
        for (k, judged) in reading.ashtakavarga.iter().flatten().enumerate() {
            put(
                report,
                &format!("chart-{index}-gochar-{at}-av-{k}"),
                format!(
                    "{} {} {} {} {} {} {} {}",
                    judged.graha.full_key(),
                    judged.bindus,
                    judged.good,
                    judged.kakshya.index,
                    wire_key(&judged.kakshya.lord),
                    judged.kakshya_bindu,
                    judged.sarva,
                    wire_key(&judged.sarva_standing),
                ),
            );
        }
        for (g, read) in reading.grahas.iter().enumerate() {
            let by: Vec<&str> = read.obstructed_by.iter().map(|g| g.full_key()).collect();
            put(
                report,
                &format!("chart-{index}-gochar-{at}-{g}"),
                format!(
                    "{} {} {} {} {} {} {} {} {} {}",
                    read.graha.full_key(),
                    read.transit.sign.full_key(),
                    number(read.transit.degrees),
                    read.house,
                    read.good_house,
                    read.vedha_house
                        .map_or_else(|| "-".to_owned(), |h| h.to_string()),
                    if by.is_empty() {
                        "-".to_owned()
                    } else {
                        by.join(",")
                    },
                    wire_key(&read.verdict),
                    wire_key(&read.fruition),
                    read.fruitful_now,
                ),
            );
        }
    }
}

/// Everything one chart of the batch prints, in the order the other three
/// print it.
fn one_document(report: &mut Report, geo: &Context, index: usize, document: &teistro::Document) {
    the_drawings(report, geo, index, document);
    one_varga_chart(report, index, document);
    the_states(report, index, document);
    the_bhavas(report, index, document);
    the_points(report, index, document);
    the_drishti(report, index, document);
    the_strength(report, index, document);
    the_dashas(report, geo, index, document);
    the_praveshas(report, geo, index, document);
    the_gochar(report, geo, index, document);
    the_hits(report, geo, index, document);
    the_sade_sati(report, geo, index, document);
    the_kp(report, geo, index, document);
    the_dignities(report, geo, index, document);
    the_fortitudes(report, geo, index, document);
    the_lots(report, geo, index, document);
    the_considerations(report, geo, index, document);
    the_perfection(report, geo, index, document);
}

/// The progressions every runner asks for: a life in 2050 under the noon
/// sidereal year, the angles by the solar arc in longitude and Naibod's
/// direction, and ten years of the Moon's and the Sun's contacts.
const PROGRESSIONS_JSON: &str = r#"{"at":2470000.5,"year":"NOON_SIDEREAL_TIME","angles":"SOLAR_ARC_LONGITUDE","direction":"NAIBOD","contacts":{"from":2462000.5,"to":2465652.5,"grahas":["MOON","SUN"],"points":["LAGNA","MARS","VENUS"],"aspects":[0,45,90,135,180]}}"#;

/// The progressions as the other three print them: the progressed chart's
/// instants and angles, each of its grahas, the direction, and each
/// contact in turn.
fn the_progressions(
    report: &mut Report,
    sdk: &Context,
    index: usize,
    document: &teistro::Document,
    request: &ChartRequest,
) {
    let asked =
        teistro::ProgressionsRequest::from_json(PROGRESSIONS_JSON).expect("a valid request");
    let read = sdk
        .chart()
        .progressions(document, &asked, request)
        .expect("the test provider");
    let key = |what: &str| format!("chart-{index}-{what}");
    let progressed = read.progressed.expect("asked at an instant");
    put(
        report,
        &key("progressed"),
        format!(
            "{} {} {} {} {}",
            number(progressed.life.get()),
            number(progressed.sky.get()),
            number(progressed.armc_deg),
            number(progressed.angles.ascendant_deg),
            number(progressed.angles.midheaven_deg)
        ),
    );
    let later = &progressed.chart.value.foundation;
    for (n, at) in later.grahas.iter().chain(&later.outer).enumerate() {
        put(
            report,
            &key(&format!("progressed-graha-{n}")),
            format!(
                "{} {} {} {}",
                at.graha.full_key(),
                number(at.longitude_deg),
                number(at.tropical_deg),
                number(at.speed_deg_per_day)
            ),
        );
    }
    let directed = read.directed.expect("asked at an instant");
    put(
        report,
        &key("directed"),
        format!(
            "{} {} {}",
            number(directed.arc_deg),
            number(directed.ascendant_deg),
            number(directed.midheaven_deg)
        ),
    );
    for (n, at) in directed.planets.iter().enumerate() {
        put(
            report,
            &key(&format!("directed-graha-{n}")),
            format!("{} {}", at.graha.full_key(), number(at.longitude_deg)),
        );
    }
    let contacts = read.contacts.expect("asked over a window");
    put(
        report,
        &key("progressed-contact-count"),
        contacts.len().to_string(),
    );
    for (n, at) in contacts.iter().enumerate() {
        put(
            report,
            &key(&format!("progressed-contact-{n}")),
            format!(
                "{} {} {} {} {} {}",
                number(at.life.get()),
                number(at.sky.get()),
                at.graha.full_key(),
                natal_key(at.to),
                at.angle,
                wire_key(&at.motion)
            ),
        );
    }
}

/// The Western aspects every runner asks for: six of the nine under
/// moieties over the ten planets, so a model other than the default and an
/// aspect beyond Ptolemy's both cross.
const WESTERN_ASPECTS_JSON: &str = r#"{"aspects":["CONJUNCTION","SEXTILE","SQUARE","TRINE","QUINCUNX","OPPOSITION"],"orbs":{"model":"MOIETIES","orbs":[{"graha":"SUN","orbDeg":17},{"graha":"MOON","orbDeg":12.5},{"graha":"MERCURY","orbDeg":7},{"graha":"VENUS","orbDeg":8},{"graha":"MARS","orbDeg":7.5},{"graha":"JUPITER","orbDeg":12},{"graha":"SATURN","orbDeg":10},{"graha":"URANUS","orbDeg":5},{"graha":"NEPTUNE","orbDeg":5},{"graha":"PLUTO","orbDeg":5}]}}"#;

/// The Western aspect table as the other three print it: its length, then
/// each row's pair, aspect, arcs, orb and whether it applies.
fn the_western_aspects(
    report: &mut Report,
    sdk: &Context,
    index: usize,
    document: &teistro::Document,
) {
    let asked = teistro::AspectRequest::from_json(WESTERN_ASPECTS_JSON).expect("a valid request");
    let rows = sdk
        .chart()
        .western_aspects(document, &asked)
        .expect("the outer planets placed");
    put(
        report,
        &format!("chart-{index}-western-aspect-count"),
        rows.len().to_string(),
    );
    for (n, row) in rows.iter().enumerate() {
        put(
            report,
            &format!("chart-{index}-western-aspect-{n}"),
            format!(
                "{} {} {} {} {} {} {}",
                row.first.full_key(),
                row.second.full_key(),
                row.aspect.key(),
                number(row.apart_deg),
                number(row.from_exact_deg),
                number(row.orb_deg),
                u8::from(row.applying)
            ),
        );
    }
}

/// The parallels every runner asks for: a wider orb than Leo's, so the
/// record's own value crosses.
const PARALLELS_JSON: &str = r#"{"orbDeg":1.5}"#;

/// A chart's declinations and parallels as the other three print them:
/// the obliquity and the angles', each planet's, then the parallels.
fn the_parallels(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let asked = teistro::ParallelRequest::from_json(PARALLELS_JSON).expect("a valid request");
    let declined = sdk.chart().declinations(document).expect("the angles");
    put(
        report,
        &format!("chart-{index}-declinations"),
        format!(
            "{} {} {}",
            number(declined.obliquity_deg),
            number(declined.lagna_deg),
            number(declined.midheaven_deg)
        ),
    );
    for at in &declined.grahas {
        put(
            report,
            &format!("chart-{index}-declination-{}", at.graha.full_key()),
            number(at.declination_deg),
        );
    }
    let rows = sdk.chart().parallels(document, &asked).expect("the angles");
    put(
        report,
        &format!("chart-{index}-parallel-count"),
        rows.len().to_string(),
    );
    for (n, row) in rows.iter().enumerate() {
        put(
            report,
            &format!("chart-{index}-parallel-{n}"),
            format!(
                "{} {} {} {} {}",
                row.first.full_key(),
                row.second.full_key(),
                u8::from(row.contrary),
                number(row.apart_deg),
                number(row.orb_deg)
            ),
        );
    }
}

/// The antiscia as the other three print them: each planet's two
/// reflections, the planets left unpaired, then each pair under Lilly's
/// moieties, the default every runner asks with `{}`.
fn the_antiscia(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let read = sdk
        .chart()
        .antiscia(document, &teistro::AntisciaRequest::default())
        .expect("a valid request");
    for at in &read.points {
        put(
            report,
            &format!("chart-{index}-antiscion-{}", at.graha.full_key()),
            format!(
                "{} {}",
                number(at.antiscion_deg),
                number(at.contrantiscion_deg)
            ),
        );
    }
    let unpaired: Vec<&str> = read.unpaired.iter().map(|one| one.full_key()).collect();
    put(
        report,
        &format!("chart-{index}-antiscia-unpaired"),
        if unpaired.is_empty() {
            String::from("-")
        } else {
            unpaired.join(",")
        },
    );
    put_antiscion_rows(report, &format!("chart-{index}-antiscia"), &read.pairs);
}

/// The equal distances as the other three print them, at the 1.5° every
/// runner asks: their count, then each row's pair, planet between, side,
/// distance, distance from the axis and orb.
fn the_midpoints(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let rows = sdk
        .chart()
        .midpoints(
            document,
            &teistro::MidpointRequest::default().with_orb_deg(1.5),
        )
        .expect("a valid request");
    put(
        report,
        &format!("chart-{index}-midpoint-count"),
        rows.len().to_string(),
    );
    for (n, row) in rows.iter().enumerate() {
        put(
            report,
            &format!("chart-{index}-midpoint-{n}"),
            format!(
                "{} {} {} {} {} {} {}",
                row.first.full_key(),
                row.second.full_key(),
                row.middle.full_key(),
                u8::from(row.far),
                number(row.distance_deg),
                number(row.from_axis_deg),
                number(row.orb_deg)
            ),
        );
    }
}

/// Pairs in antiscion as every runner prints them, a chart's own or
/// across a synastry: their count, then each pair's planets, side, gap
/// and orb.
fn put_antiscion_rows(report: &mut Report, key: &str, rows: &[teistro::AntiscionRow]) {
    put(report, &format!("{key}-count"), rows.len().to_string());
    for (n, row) in rows.iter().enumerate() {
        put(
            report,
            &format!("{key}-{n}"),
            format!(
                "{} {} {} {} {}",
                row.first.full_key(),
                row.second.full_key(),
                u8::from(row.contrary),
                number(row.apart_deg),
                number(row.orb_deg)
            ),
        );
    }
}

/// A natal point as every runner prints it: `LAGNA`, or the graha's full
/// key.
fn natal_key(point: teistro::NatalPoint) -> &'static str {
    match point {
        teistro::NatalPoint::Lagna => "LAGNA",
        teistro::NatalPoint::Graha { graha } => graha.full_key(),
    }
}

/// The synastry every runner asks for: four aspects against a partner born
/// in Sydney at J2000, in each chart's own zodiac, so the partner's own
/// clock, the lagna and C241's sidereal reading all cross; the parallels
/// across under a widened orb; the antiscia across under Leo's orbs,
/// which give every planet one; the equal distances across under a
/// widened orb; and the composite and the Davison birth,
/// the partner's clock and the charts' meeting in it.
const SYNASTRY_JSON: &str = r#"{"partner":{"instant":2451545.25,"place":{"latitude":-33.87,"longitude":151.21,"altitude":0},"utcOffsetSeconds":36000},"aspects":["CONJUNCTION","SQUARE","TRINE","OPPOSITION"],"zodiac":"CHARTS","parallels":{"orbDeg":1.5},"antiscia":{"orbs":{"model":"LEO"}},"midpoints":{"orbDeg":1.5},"composite":true,"davison":true}"#;

/// Every chart's synastry as the other three print it: its length, then
/// each row's two points, aspect, arcs and orb; then the parallels across,
/// each row's two points, side, gap and orb; then the antiscia across;
/// then the composite, its angles and each planet's place and speed; then
/// the Davison birth on the charts' clock.
fn the_synastry(
    report: &mut Report,
    sdk: &Context,
    documents: &[teistro::Document],
    clock: UtcOffset,
) {
    let asked = teistro::PartnerSynastry::from_json(SYNASTRY_JSON).expect("a valid request");
    let read = sdk
        .chart()
        .synastry_with(documents, &asked)
        .expect("one zodiac for both");
    let davisons = asked
        .davisons(documents, clock)
        .expect("valid births")
        .expect("the record asks for them");
    for ((index, one), davison) in read.iter().enumerate().zip(&davisons) {
        put(
            report,
            &format!("chart-{index}-synastry-count"),
            one.aspects.len().to_string(),
        );
        for (n, row) in one.aspects.iter().enumerate() {
            put(
                report,
                &format!("chart-{index}-synastry-{n}"),
                format!(
                    "{} {} {} {} {} {}",
                    natal_key(row.first),
                    natal_key(row.second),
                    row.aspect.key(),
                    number(row.apart_deg),
                    number(row.from_exact_deg),
                    number(row.orb_deg)
                ),
            );
        }
        let parallels = one.parallels.as_deref().expect("the record asks for them");
        put(
            report,
            &format!("chart-{index}-synastry-parallel-count"),
            parallels.len().to_string(),
        );
        for (n, row) in parallels.iter().enumerate() {
            put(
                report,
                &format!("chart-{index}-synastry-parallel-{n}"),
                format!(
                    "{} {} {} {} {}",
                    natal_key(row.first),
                    natal_key(row.second),
                    u8::from(row.contrary),
                    number(row.apart_deg),
                    number(row.orb_deg)
                ),
            );
        }
        put_antiscion_rows(
            report,
            &format!("chart-{index}-synastry-antiscia"),
            one.antiscia.as_deref().expect("the record asks for them"),
        );
        put_synastry_midpoints(
            report,
            index,
            one.midpoints.as_deref().expect("the record asks for them"),
        );
        put_one_chart(
            report,
            index,
            one.composite.as_ref().expect("the record asks for it"),
            davison,
        );
    }
}

/// The equal distances across a synastry as the other three print them:
/// their count, then each row's pair, the planet between, whose pair,
/// side, distance, gap from the axis and orb.
fn put_synastry_midpoints(
    report: &mut Report,
    index: usize,
    across: &[teistro::SynastryMidpointRow],
) {
    put(
        report,
        &format!("chart-{index}-synastry-midpoint-count"),
        across.len().to_string(),
    );
    for (n, row) in across.iter().enumerate() {
        put(
            report,
            &format!("chart-{index}-synastry-midpoint-{n}"),
            format!(
                "{} {} {} {} {} {} {} {}",
                row.first.full_key(),
                row.second.full_key(),
                row.middle.full_key(),
                u8::from(row.partners_pair),
                u8::from(row.far),
                number(row.distance_deg),
                number(row.from_axis_deg),
                number(row.orb_deg)
            ),
        );
    }
}

/// The composite and the Davison birth as the other three print them: the
/// composite's angles and each planet's place and speed, then the
/// Davison birth.
fn put_one_chart(
    report: &mut Report,
    index: usize,
    composite: &teistro::Composite,
    davison: &teistro::Partner,
) {
    put(
        report,
        &format!("chart-{index}-composite"),
        format!(
            "{} {} {} {}",
            number(composite.lagna_deg),
            number(composite.midheaven_deg),
            u8::from(composite.lagna_turned),
            composite.planets.len()
        ),
    );
    for (n, at) in composite.planets.iter().enumerate() {
        put(
            report,
            &format!("chart-{index}-composite-{n}"),
            format!(
                "{} {} {}",
                at.graha.full_key(),
                number(at.longitude_deg),
                number(at.speed_deg_per_day)
            ),
        );
    }
    put(
        report,
        &format!("chart-{index}-davison"),
        format!(
            "{} {} {} {} {}",
            number(davison.instant.get()),
            number(davison.place.latitude.get()),
            number(davison.place.longitude.get()),
            number(davison.place.altitude.get()),
            davison.utc_offset.seconds()
        ),
    );
}

/// The lots every runner asks for: III.11's Fortune, the one that reads
/// the Moon's altitude.
const LOTS_JSON: &str = r#"{"fortune":"REVERSED_WHILE_MOON_UP"}"#;

/// The lots as the other three print them: the sect and rules, then each
/// lot's longitude, sign, lord and house, in the catalogue's order.
fn the_lots(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let asked = teistro::LotRequest::from_json(LOTS_JSON).expect("a valid request");
    let read = sdk
        .chart()
        .lots_with_request(document, &teistro::Lot::ALL, asked)
        .expect("the test provider");
    put(
        report,
        &format!("chart-{index}-lots"),
        format!(
            "{} {} {} {}",
            wire_key(&read.sect),
            wire_key(&read.request.sect_rule()),
            wire_key(&read.request.fortune()),
            u8::from(read.fortune_reversed)
        ),
    );
    for placed in &read.lots {
        let place = &placed.place;
        put(
            report,
            &format!("chart-{index}-lot-{}", wire_key(&placed.lot)),
            format!(
                "{} {} {} {}",
                number(place.longitude_deg),
                place.sign.full_key(),
                place.lord.full_key(),
                place.house.get()
            ),
        );
    }
}

/// The considerations every runner asks for: a late Moon from 25°, read
/// from the fortitudes every runner asks for.
const CONSIDERATIONS_JSON: &str = r#"{"moonLateFromDeg":25}"#;

/// A perfection as the runners print it, or `-` when the Moon is void by
/// that reading.
fn perfection(found: Option<teistro::Perfection>) -> String {
    found.map_or_else(
        || String::from("-"),
        |at| {
            format!(
                "{} {} {} {}",
                at.planet.full_key(),
                wire_key(&at.aspect),
                number(at.days),
                number(at.gap_deg)
            )
        },
    )
}

/// The Moon's clause and her two perfections, as the runners print them.
fn the_moon_clause(report: &mut Report, index: usize, moon: &teistro::MoonClause) {
    put(
        report,
        &format!("chart-{index}-considerations-moon"),
        format!(
            "{} {} {} {} {} {} {}",
            moon.sign.full_key(),
            number(moon.degree),
            u8::from(moon.late),
            u8::from(moon.late_sign),
            u8::from(moon.via_combusta),
            number(moon.course.days_in_sign),
            u8::from(moon.course.eased)
        ),
    );
    put(
        report,
        &format!("chart-{index}-considerations-next"),
        perfection(moon.course.next),
    );
    put(
        report,
        &format!("chart-{index}-considerations-within"),
        perfection(moon.course.within_orb),
    );
}

/// The considerations as the other three print them: radicality and the
/// Ascendant, the Moon, her two perfections, the seventh, Saturn and the
/// rules.
fn the_considerations(
    report: &mut Report,
    sdk: &Context,
    index: usize,
    document: &teistro::Document,
) {
    let rules =
        teistro::ConsiderationRules::from_json(CONSIDERATIONS_JSON).expect("a valid request");
    let fortitudes =
        teistro::FortitudeRequest::from_json(&fortitudes_json()).expect("a valid request");
    let read = sdk
        .chart()
        .considerations(document, &fortitudes, rules)
        .expect("the test provider");
    let flag = u8::from;
    let radicality = &read.radicality;
    let ascendant = &read.ascendant;
    put(
        report,
        &format!("chart-{index}-considerations"),
        format!(
            "{} {} {} {} {} {} {} {}",
            radicality.hour_lord.full_key(),
            radicality.ascendant_lord.full_key(),
            comma_listed(
                &radicality
                    .grounds
                    .iter()
                    .map(wire_key)
                    .collect::<Vec<_>>()
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            ),
            ascendant.sign.full_key(),
            number(ascendant.degree),
            flag(ascendant.early),
            flag(ascendant.late),
            flag(ascendant.short_ascension)
        ),
    );
    the_moon_clause(report, index, &read.moon);
    let seventh = &read.seventh;
    put(
        report,
        &format!("chart-{index}-considerations-seventh"),
        format!(
            "{} {} {} {} {} {} {} {}",
            number(seventh.cusp_deg),
            seventh.lord.full_key(),
            comma_listed(
                &seventh
                    .infortunes_in_house
                    .iter()
                    .map(|graha| graha.full_key())
                    .collect::<Vec<_>>()
            ),
            flag(seventh.lord_retrograde),
            flag(seventh.lord_combust),
            flag(seventh.lord_in_fall),
            flag(seventh.lord_in_infortune_term),
            seventh.lord_net
        ),
    );
    put(
        report,
        &format!("chart-{index}-considerations-saturn"),
        format!(
            "{} {} {}",
            read.saturn_house.get(),
            flag(read.saturn_retrograde),
            flag(read.ascendant_lord_combust)
        ),
    );
    put(
        report,
        &format!("chart-{index}-considerations-rules"),
        format!(
            "{} {}",
            number(read.rules.moon_late_from_deg),
            read.rules
                .orbs_deg
                .iter()
                .map(|orb| number(*orb))
                .collect::<Vec<_>>()
                .join(",")
        ),
    );
}

/// The perfection every runner asks for: the matter of the seventh house
/// over 120 days, long enough for the parity charts to apply and be
/// hindered, weighed on the fortitudes every runner asks for.
const PERFECTION_JSON: &str = r#"{"house":7,"rules":{"horizonDays":120}}"#;

/// A planet's dignities as the runners print them: the flags it holds, in
/// `EssentialDignity`'s order, or `-` for none.
fn dignities_held(d: teistro::EssentialDignity) -> String {
    let held: Vec<&str> = [
        ("house", d.house),
        ("exaltation", d.exaltation),
        ("triplicity", d.triplicity),
        ("term", d.term),
        ("face", d.face),
        ("detriment", d.detriment),
        ("fall", d.fall),
    ]
    .into_iter()
    .filter_map(|(name, holds)| holds.then_some(name))
    .collect();
    comma_listed(&held)
}

/// The perfection as the other three print them: the significators and
/// counts, the application and separation, the ways, each impediment,
/// translation and collection in turn, and the rules.
fn the_perfection(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let asked = teistro::PerfectionRequest::from_json(PERFECTION_JSON).expect("a valid request");
    let fortitudes =
        teistro::FortitudeRequest::from_json(&fortitudes_json()).expect("a valid request");
    let read = sdk
        .chart()
        .perfection(document, &fortitudes, &asked)
        .expect("the test provider");
    let key = |what: &str| format!("chart-{index}-perfection{what}");
    let flag = u8::from;
    put(
        report,
        &key(""),
        format!(
            "{} {} {} {} {} {}",
            read.querent.full_key(),
            read.quesited.full_key(),
            number(read.horizon_days),
            read.impediments.len(),
            read.translations.len(),
            read.collections.len()
        ),
    );
    put(
        report,
        &key("-application"),
        read.application.map_or_else(
            || String::from("-"),
            |at| {
                format!(
                    "{} {} {} {} {} {}",
                    wire_key(&at.aspect),
                    number(at.days),
                    at.applying.full_key(),
                    wire_key(&at.kind),
                    number(at.gap_deg),
                    flag(at.within_moieties)
                )
            },
        ),
    );
    put(
        report,
        &key("-separation"),
        read.separation.map_or_else(
            || String::from("-"),
            |at| format!("{} {}", wire_key(&at.aspect), number(at.past_deg)),
        ),
    );
    let ways = &read.ways;
    let infortunes: Vec<&str> = ways
        .infortunes_between
        .iter()
        .map(|g| g.full_key())
        .collect();
    let held: Vec<String> = ways.held.iter().map(wire_key).collect();
    put(
        report,
        &key("-ways"),
        format!(
            "{} {} {} {} {} {} {} {} {}",
            ways.querent.house.get(),
            dignities_held(ways.querent.dignity),
            ways.quesited.house.get(),
            dignities_held(ways.quesited.dignity),
            flag(ways.mutual_by_house),
            comma_listed(&infortunes),
            flag(ways.moon_relays),
            flag(ways.quesited_in_ascendant),
            comma_listed(&held.iter().map(String::as_str).collect::<Vec<_>>())
        ),
    );
    the_perfection_rows(report, index, &read);
    put(
        report,
        &key("-rules"),
        format!(
            "{} {}",
            asked
                .rules
                .orbs_deg
                .iter()
                .map(|orb| number(*orb))
                .collect::<Vec<_>>()
                .join(","),
            flag(asked.rules.within_sign)
        ),
    );
}

/// A perfection's ragged rows as the runners print them: each impediment,
/// translation and collection in turn.
fn the_perfection_rows(report: &mut Report, index: usize, read: &teistro::Matter) {
    let key = |what: &str| format!("chart-{index}-perfection{what}");
    for (n, at) in read.impediments.iter().enumerate() {
        put(
            report,
            &key(&format!("-impediment-{n}")),
            format!(
                "{} {} {} {} {}",
                wire_key(&at.kind),
                at.significator.full_key(),
                at.third.map_or("-", |g| g.full_key()),
                wire_key(&at.aspect),
                number(at.days)
            ),
        );
    }
    for (n, at) in read.translations.iter().enumerate() {
        put(
            report,
            &key(&format!("-translation-{n}")),
            format!(
                "{} {} {} {} {} {} {} {}",
                at.translator.full_key(),
                at.from.full_key(),
                at.to.full_key(),
                wire_key(&at.separating.aspect),
                number(at.separating.past_deg),
                wire_key(&at.aspect),
                number(at.days),
                dignities_held(at.received)
            ),
        );
    }
    for (n, at) in read.collections.iter().enumerate() {
        put(
            report,
            &key(&format!("-collection-{n}")),
            format!(
                "{} {} {} {} {} {} {} {} {}",
                at.collector.full_key(),
                wire_key(&at.from_querent.aspect),
                number(at.from_querent.days),
                wire_key(&at.from_quesited.aspect),
                number(at.from_quesited.days),
                dignities_held(at.collector_in_querent),
                dignities_held(at.collector_in_quesited),
                dignities_held(at.querent_in_collector),
                dignities_held(at.quesited_in_collector)
            ),
        );
    }
}

/// The fortitudes every runner asks for: the dignities' own request as the
/// essential half, which the other three ask through this one, and an
/// accidental knob of each kind turned.
fn fortitudes_json() -> String {
    format!(
        r#"{{"dignities":{DIGNITIES_JSON},"rules":{{"beamsDeg":15,"combustionInSign":false,"partile":{{"WITHIN":{{"orbDeg":1}}}},"siege":{{"WITHIN":{{"spanDeg":30}}}}}},"scores":{{"regulus":5}},"almuten":{{"fortune":"REVERSED_BY_NIGHT"}}}}"#
    )
}

/// The accidental fortitudes as the other three print them: the sky, the
/// rules and scores applied, the cusps, then each planet's speed, house,
/// lines with their points, and totals.
fn the_fortitudes(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let asked = teistro::FortitudeRequest::from_json(&fortitudes_json()).expect("a valid request");
    let read = sdk
        .chart()
        .fortitudes(document, &asked)
        .expect("the test provider");
    let numbers = |values: &[f64]| {
        values
            .iter()
            .map(|v| number(*v))
            .collect::<Vec<_>>()
            .join(",")
    };
    let sky = &read.sky;
    put(
        report,
        &format!("chart-{index}-fortitudes"),
        format!(
            "{} {} {} {} {}",
            sky.houses.full_key(),
            number(sky.north_node_deg),
            number(sky.regulus_deg),
            number(sky.spica_deg),
            number(sky.algol_deg)
        ),
    );
    put(
        report,
        &format!("chart-{index}-fortitude-rules"),
        fortitude_rules(&read.rules),
    );
    let lines = serde_json::to_value(read.scores).expect("scores serialise");
    let worth: Vec<String> = ACCIDENTAL_LINES
        .iter()
        .map(|line| {
            lines
                .get(line)
                .map_or_else(String::new, ToString::to_string)
        })
        .collect();
    put(
        report,
        &format!("chart-{index}-fortitude-scores"),
        format!(
            "{} {}",
            read.scores.houses.map(|one| one.to_string()).join(","),
            worth.join(",")
        ),
    );
    put(
        report,
        &format!("chart-{index}-fortitude-houses"),
        numbers(&sky.cusps_deg),
    );
    for (at, speed) in read.planets.iter().zip(sky.speeds_deg_per_day) {
        let accidents: Vec<String> = at
            .accidents
            .iter()
            .map(|&line| {
                format!(
                    "{}:{}",
                    wire_key(&line),
                    read.scores.points(at.planet, line)
                )
            })
            .collect();
        put(
            report,
            &format!("chart-{index}-fortitude-{}", at.planet.full_key()),
            format!(
                "{} {} {} {} {} {}",
                number(speed),
                at.house.get(),
                if accidents.is_empty() {
                    String::from("-")
                } else {
                    accidents.join(",")
                },
                at.fortitude,
                at.debility,
                read.net(at.planet)
                    .map_or_else(|| String::from("-"), |net| net.to_string())
            ),
        );
    }
    the_almutens(report, index, &read);
}

/// The almutens as every runner prints them: the rules, Fortune and the
/// angles, then each ranking's totals, almutens and partakers.
fn the_almutens(report: &mut Report, index: usize, read: &teistro::Fortitudes) {
    let almutens = &read.almutens;
    put(
        report,
        &format!("chart-{index}-almuten-rules"),
        format!(
            "{} {} {} {} {}",
            wire_key(&almutens.rules.place),
            wire_key(&almutens.rules.fortune),
            number(almutens.fortune_deg),
            number(read.sky.ascendant_deg),
            number(read.sky.midheaven_deg)
        ),
    );
    let ranked = |almuten: &teistro::Almuten| {
        let planets = |of: Vec<teistro::catalogue::Graha>| {
            let keys: Vec<&str> = of.iter().map(|planet| planet.full_key()).collect();
            if keys.is_empty() {
                String::from("-")
            } else {
                keys.join(",")
            }
        };
        let totals: Vec<String> = almuten.totals.iter().map(i16::to_string).collect();
        format!(
            "{} {} {}",
            totals.join(","),
            planets(almuten.almutens()),
            planets(almuten.partakers())
        )
    };
    put(
        report,
        &format!("chart-{index}-almuten-figure"),
        ranked(&almutens.figure),
    );
    put(
        report,
        &format!("chart-{index}-almuten-places"),
        ranked(&almutens.places),
    );
    for (house, almuten) in almutens.houses.iter().enumerate() {
        put(
            report,
            &format!("chart-{index}-almuten-house-{}", house + 1),
            ranked(almuten),
        );
    }
}

/// The accidental rules as every runner prints them: the orbs, the sign
/// clause as 1 or 0, partile and siege with any orb, and the mean motions.
fn fortitude_rules(r: &teistro::AccidentalRules) -> String {
    let with_orb = |within: Option<f64>, key: String| {
        within.map_or(key, |orb| format!("WITHIN:{}", number(orb)))
    };
    let partile = match r.partile {
        teistro::Partile::Within { orb_deg } => Some(orb_deg),
        _ => None,
    };
    let siege = match r.siege {
        teistro::Siege::Within { span_deg } => Some(span_deg),
        _ => None,
    };
    let means: Vec<String> = r.mean_motion_deg.iter().map(|v| number(*v)).collect();
    format!(
        "{} {} {} {} {} {} {} {} {}",
        number(r.combustion_deg),
        u8::from(r.combustion_in_sign),
        number(r.beams_deg),
        number(r.cazimi_deg),
        number(r.cusp_orb_deg),
        number(r.star_orb_deg),
        with_orb(partile, wire_key(&r.partile)),
        with_orb(siege, wire_key(&r.siege)),
        means.join(",")
    )
}

/// Lilly's accidental lines by the names a request spells them, in the
/// order every runner prints their scores.
const ACCIDENTAL_LINES: [&str; 26] = [
    "direct",
    "retrograde",
    "swift",
    "slow",
    "superiorOriental",
    "superiorOccidental",
    "inferiorOriental",
    "inferiorOccidental",
    "increasing",
    "decreasing",
    "freeFromCombustion",
    "cazimi",
    "combust",
    "underBeams",
    "conjunctBenefic",
    "conjunctNorthNode",
    "trineBenefic",
    "sextileBenefic",
    "conjunctMalefic",
    "conjunctSouthNode",
    "opposedMalefic",
    "squareMalefic",
    "besieged",
    "regulus",
    "spica",
    "algol",
];

/// The dignities every runner asks for, every knob turned from its default.
const DIGNITIES_JSON: &str = r#"{"sectRule":"DAYLIGHT","rules":{"terms":"EGYPTIAN","triplicities":"PTOLEMY"},"scores":{"peregrine":0}}"#;

/// The dignities as the other three print them: what was applied, then
/// each planet's longitude, the dignities it holds by name and its score.
fn the_dignities(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let asked = teistro::DignityRequest::from_json(DIGNITIES_JSON).expect("a valid request");
    let read = sdk
        .chart()
        .dignities(document, &asked)
        .expect("the test provider");
    let s = read.scores;
    let worth = [
        s.house,
        s.exaltation,
        s.triplicity,
        s.term,
        s.face,
        s.detriment,
        s.fall,
        s.peregrine,
    ];
    put(
        report,
        &format!("chart-{index}-dignities"),
        format!(
            "{} {} {} {} {}",
            wire_key(&read.sect),
            wire_key(&read.sect_rule),
            wire_key(&read.rules.terms),
            wire_key(&read.rules.triplicities),
            worth.map(|one| one.to_string()).join(",")
        ),
    );
    for at in read.planets {
        let mut held = flags_held(at.dignity);
        if at.dignity.peregrine() {
            held.push("peregrine");
        }
        put(
            report,
            &format!("chart-{index}-dignity-{}", at.planet.full_key()),
            format!(
                "{} {} {} {}",
                number(at.longitude_deg),
                comma_listed(&held),
                at.score,
                at.reception
            ),
        );
    }
    for (k, one) in read.receptions.iter().enumerate() {
        let mutual: Vec<&str> = one
            .mutual()
            .map(|kind| match kind {
                teistro::DignityKind::House => "house",
                teistro::DignityKind::Exaltation => "exaltation",
                teistro::DignityKind::Triplicity => "triplicity",
                teistro::DignityKind::Term => "term",
                teistro::DignityKind::Face => "face",
                _ => "unknown",
            })
            .collect();
        put(
            report,
            &format!("chart-{index}-reception-{k}"),
            format!(
                "{} {} {} {} {}",
                one.planets[0].full_key(),
                one.planets[1].full_key(),
                flags_held(one.first_in).join(","),
                flags_held(one.second_in).join(","),
                comma_listed(&mutual)
            ),
        );
    }
}

/// The dignities and debilities that hold, by the names every runner
/// prints them by.
fn flags_held(d: teistro::EssentialDignity) -> Vec<&'static str> {
    [
        (d.house, "house"),
        (d.exaltation, "exaltation"),
        (d.triplicity, "triplicity"),
        (d.term, "term"),
        (d.face, "face"),
        (d.detriment, "detriment"),
        (d.fall, "fall"),
    ]
    .into_iter()
    .filter_map(|(holds, name)| holds.then_some(name))
    .collect()
}

/// Names comma-joined, or `-` for none.
fn comma_listed(names: &[&str]) -> String {
    if names.is_empty() {
        String::from("-")
    } else {
        names.join(",")
    }
}

/// The KP reading every runner asks for: a horary number, in the geo
/// context's own zodiac.
const KP_JSON: &str = r#"{"number":74,"anyAyanamsha":true}"#;

/// KP as the other three print it: the rules, then every cusp, planet,
/// house, node and ruler, a key in full and a longitude or bound in whole
/// nanoarcseconds.
fn the_kp(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let asked = teistro::KpRequest::from_json(KP_JSON).expect("a valid request");
    let read = sdk
        .chart()
        .kp_reading(document, &asked)
        .expect("the test provider");
    let rules = &read.ruling.rules;
    put(
        report,
        &format!("chart-{index}-kp"),
        format!(
            "{} {} {} {}",
            read.chart.system.full_key(),
            wire_key(&rules.count),
            wire_key(&rules.node_rulers),
            wire_key(&rules.retrograde_rejection)
        ),
    );
    kp_chart(report, index, &read.chart);
    kp_significators(report, index, &read.significators);
    kp_rulers(report, index, &read.ruling.rulers);
}

/// Full keys joined by commas, `-` for none.
fn kp_keys<'k>(keys: impl Iterator<Item = &'k str>) -> String {
    let keys: Vec<&str> = keys.collect();
    if keys.is_empty() {
        "-".to_owned()
    } else {
        keys.join(",")
    }
}

/// A point's lords, each level below the sign with its span.
fn kp_lords(lords: &teistro::kp::Lords) -> String {
    let level = |level: &teistro::kp::Level| {
        format!(
            "{} {} {}",
            level.lord.full_key(),
            level.span.start.get(),
            level.span.end.get()
        )
    };
    format!(
        "{} {} {} {}",
        lords.sign.full_key(),
        level(&lords.star),
        level(&lords.sub),
        level(&lords.sub_sub)
    )
}

/// Every cusp and planet of a KP chart.
fn kp_chart(report: &mut Report, index: usize, chart: &teistro::KpChart) {
    for cusp in &chart.cusps {
        put(
            report,
            &format!("chart-{index}-kp-cusp-{}", cusp.house),
            format!("{} {}", cusp.longitude.get(), kp_lords(&cusp.lords)),
        );
    }
    for planet in &chart.planets {
        put(
            report,
            &format!("chart-{index}-kp-planet-{}", planet.graha.full_key()),
            format!(
                "{} {} {} {}",
                planet.longitude.get(),
                planet.retrograde,
                planet.house,
                kp_lords(&planet.lords)
            ),
        );
    }
}

/// Every house's significators, and every node's agency.
fn kp_significators(report: &mut Report, index: usize, read: &teistro::kp::Significators) {
    let grahas = |grahas: &[Graha]| kp_keys(grahas.iter().map(|graha| graha.full_key()));
    for house in &read.houses {
        put(
            report,
            &format!("chart-{index}-kp-house-{}", house.house),
            format!(
                "{} {} {} {} {} {} {}",
                grahas(&house.in_occupants_stars),
                grahas(&house.occupants),
                grahas(&house.in_lords_star),
                house.lord.full_key(),
                grahas(&house.conjoined),
                grahas(&house.aspected),
                kp_keys(house.intercepted.iter().map(|sign| sign.full_key()))
            ),
        );
    }
    for node in &read.nodes {
        put(
            report,
            &format!("chart-{index}-kp-node-{}", node.node.full_key()),
            format!(
                "{} {} {} {}",
                grahas(&node.conjoined),
                node.star_lord.full_key(),
                grahas(&node.aspecting),
                node.sign_lord.full_key()
            ),
        );
    }
}

/// Every ruling planet, its reasons and what rejects it under either
/// reading of C153.
fn kp_rulers(report: &mut Report, index: usize, rulers: &[teistro::kp::Ruler]) {
    let rejection = |by: Option<teistro::kp::Rejection>| {
        by.map_or_else(
            || "-".to_owned(),
            |by| format!("{}:{}", by.retrograde.full_key(), by.by_star),
        )
    };
    for (k, ruler) in rulers.iter().enumerate() {
        let reasons: Vec<String> = ruler
            .reasons
            .iter()
            .map(|reason| match reason {
                teistro::kp::Reason::Agent { of, by } => {
                    format!("AGENT:{}:{}", of.full_key(), wire_key(by))
                }
                other => wire_key(other),
            })
            .collect();
        put(
            report,
            &format!("chart-{index}-kp-ruler-{k}"),
            format!(
                "{} {} {} {} {}",
                ruler.graha.full_key(),
                reasons.join(","),
                ruler.retrograde,
                rejection(ruler.rejected_by),
                rejection(ruler.rejected_by_sub)
            ),
        );
    }
}

/// The hit list every runner asks for: two months, three grahas, three
/// aspects with an orb, so every kind of event and every phase appears.
const HITS_JSON: &str = r#"{"from":2460676.5,"to":2460736.5,"grahas":["SUN","MERCURY","SATURN"],"aspects":[0,90,180],"orbDeg":2}"#;

/// The hit list as the other three print it: each hit's instant, graha
/// and kind, then the fields its kind carries, `-` for the rest.
fn the_hits(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    use teistro::gochar::hits::HitEvent;
    let asked = teistro::HitRequest::from_json(HITS_JSON).expect("a valid request");
    let hits = sdk
        .chart()
        .hits(document, &asked)
        .expect("the test provider")
        .value;
    for (k, hit) in hits.iter().enumerate() {
        let dash = || "-".to_owned();
        let (into, motion, to, angle, phase) = match hit.event {
            HitEvent::SignIngress { into, motion } => (
                into.full_key().to_owned(),
                wire_key(&motion),
                dash(),
                dash(),
                dash(),
            ),
            HitEvent::NakshatraIngress { into, motion } => (
                into.full_key().to_owned(),
                wire_key(&motion),
                dash(),
                dash(),
                dash(),
            ),
            HitEvent::Station { turns } => (dash(), wire_key(&turns), dash(), dash(), dash()),
            HitEvent::Aspect {
                to,
                angle,
                phase,
                motion,
            } => (
                dash(),
                wire_key(&motion),
                natal_key(to).to_owned(),
                angle.to_string(),
                wire_key(&phase),
            ),
        };
        put(
            report,
            &format!("chart-{index}-hit-{k}"),
            format!(
                "{} {} {} {into} {motion} {to} {angle} {phase}",
                number(hit.instant.get()),
                hit.graha.full_key(),
                wire_key(&hit.event),
            ),
        );
    }
}

/// The Sade Sati every runner asks for: ten years under the degree
/// reckoning, with the 7th beside the default smaller spells.
const SADE_SATI_JSON: &str =
    r#"{"from":2460676.5,"to":2464329.0,"reckoning":"DEGREE","spells":[4,7,8]}"#;

/// Sade Sati as the other three print it: what it was counted from, then
/// each visit with its period's ordinal (the Sade Satis, then the smaller
/// spells) and its house, `-` for a bound past the ephemeris.
fn the_sade_sati(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let asked = teistro::SadeSatiRequest::from_json(SADE_SATI_JSON).expect("a valid request");
    let found = sdk
        .chart()
        .sade_sati(document, &asked)
        .expect("the test provider")
        .value;
    put(
        report,
        &format!("chart-{index}-sade-sati"),
        format!(
            "{} {} {}",
            wire_key(&found.reference.from),
            found.reference.sign.full_key(),
            wire_key(&found.reckoning),
        ),
    );
    let periods = found
        .sade_sati
        .iter()
        .map(|one| one.phases.as_slice())
        .chain(found.spells.iter().map(std::slice::from_ref));
    let bound =
        |at: Option<JulianDay<Utc>>| at.map_or_else(|| "-".to_owned(), |at| number(at.get()));
    let mut k = 0;
    for (period, spells) in periods.enumerate() {
        for spell in spells {
            for visit in &spell.visits {
                put(
                    report,
                    &format!("chart-{index}-sade-sati-{k}"),
                    format!(
                        "{period} {} {} {}",
                        spell.house,
                        bound(visit.from),
                        bound(visit.to)
                    ),
                );
                k += 1;
            }
        }
    }
}

fn wire_key<T: serde::Serialize>(member: &T) -> String {
    serde_json::to_value(member)
        .ok()
        .and_then(|value| match value {
            serde_json::Value::String(key) => Some(key),
            tagged => tagged.get("kind")?.as_str().map(String::from),
        })
        .unwrap_or_default()
}

/// One saham as every runner prints it: its place, its clauses, its
/// lord's strengths and how the seven stand to it. Inside a string, so
/// compared as text, as the yogas' are.
fn saham_said(point: &teistro::SahamStrength) -> String {
    let place = &point.place;
    let held = |list: Vec<String>| list.join(",");
    let strong = held(
        point
            .strong()
            .iter()
            .filter(|(_, holds)| *holds)
            .map(|(clause, _)| wire_key(clause))
            .collect(),
    );
    let weak = held(
        point
            .weak()
            .iter()
            .filter(|(_, holds)| *holds)
            .map(|(clause, _)| wire_key(clause))
            .collect(),
    );
    let axis = point
        .in_node_axis
        .map_or_else(|| String::from("null"), |axis| axis.to_string());
    let seven: Vec<String> = (0..7)
        .map(|at| {
            format!(
                "{}/{}/{}",
                point.aspects.get(at).map(wire_key).unwrap_or_default(),
                point.relations.get(at).map(wire_key).unwrap_or_default(),
                u8::from(point.company.get(at).copied().unwrap_or(false)),
            )
        })
        .collect();
    format!(
        "{:.6} {} {} {} {} | S:{strong} W:{weak} | {} {} {axis} | {}",
        place.longitude_deg,
        place.sign.full_key(),
        place.lord.full_key(),
        place.house.get(),
        place.added_sign,
        point.lord_vishwa,
        wire_key(&point.lord_harsha),
        seven.join(" "),
    )
}

/// The pairs of one year's chart that make a yoga, in the order the seven
/// are read, so a binding that ordered them otherwise shows here.
fn the_yogas(
    report: &mut Report,
    sdk: &Context,
    annual: &teistro::Document,
    key: &dyn Fn(&str) -> String,
    one: &teistro::Pravesha,
) {
    let Ok(pairs) = sdk.chart().drishtis(annual) else {
        return;
    };
    let said: Vec<String> = pairs
        .iter()
        .filter_map(|pair| {
            let yoga = pair.yoga?;
            // Inside a string, so it is compared as text: the four runners
            // must round the degrees the same way.
            let apart = format!("{:.6}", pair.apart_deg);
            Some(format!(
                "{}>{}:{}:{}:{apart}",
                pair.faster.full_key(),
                pair.slower.full_key(),
                wire_key(&pair.drishti),
                wire_key(&yoga)
            ))
        })
        .collect();
    put(
        report,
        &key(&format!("-{}-yogas", one.year)),
        said.join(" "),
    );
}

/// One year's retrograde and combust planets, and the sixteen yogas for
/// every matter when the reading asked for them: the question each asked,
/// the pair it names, what it could not answer, and every yoga that held
/// with what made it hold.
fn the_matters(
    report: &mut Report,
    sdk: &Context,
    annual: &teistro::Document,
    key: &dyn Fn(&str) -> String,
    one: &teistro::Pravesha,
    rules: Option<teistro::YogaRules>,
) {
    let Ok(states) = sdk.chart().annual_states(annual) else {
        return;
    };
    let keys = |grahas: &[teistro::catalogue::Graha]| {
        grahas
            .iter()
            .map(|graha| graha.full_key())
            .collect::<Vec<&str>>()
            .join(",")
    };
    put(
        report,
        &key(&format!("-{}-states", one.year)),
        format!("R:{} C:{}", keys(&states.retrograde), keys(&states.combust)),
    );
    let Some(rules) = rules else {
        return;
    };
    let Ok(every) = sdk
        .chart()
        .tajika_yogas_many(annual, &teistro::House::ALL, rules)
    else {
        return;
    };
    for matter in &every {
        let at = |what: &str| {
            key(&format!(
                "-{}-matter-{}{what}",
                one.year,
                matter.house.get()
            ))
        };
        put(
            report,
            &at(""),
            format!(
                "{} {}>{} {}",
                matter.sign.full_key(),
                matter.lagnesha.full_key(),
                matter.karyesha.full_key(),
                matter.same_lord
            ),
        );
        put(
            report,
            &at("-pair"),
            matter
                .between
                .as_ref()
                .map_or_else(|| String::from("-"), pair_said),
        );
        put(
            report,
            &at("-unanswered"),
            matter
                .unanswered
                .iter()
                .map(wire_key)
                .collect::<Vec<String>>()
                .join(","),
        );
        put(
            report,
            &at("-held"),
            matter
                .held
                .iter()
                .map(held_said)
                .collect::<Vec<String>>()
                .join(" "),
        );
    }
}

/// A pair as every runner writes it; the degrees inside a string, so the
/// four must round them alike.
fn pair_said(pair: &teistro::Between) -> String {
    format!(
        "{}>{}:{}:{}:{:.6}",
        pair.faster.full_key(),
        pair.slower.full_key(),
        wire_key(&pair.drishti),
        pair.yoga
            .map_or_else(|| String::from("-"), |yoga| wire_key(&yoga)),
        pair.apart_deg
    )
}

/// A yoga that held, and what made it: the third planet, the one entering
/// the next sign, whether the pair's own relation did, its legs and the
/// lords' afflictions — `-` wherever there is none.
fn held_said(held: &teistro::Held) -> String {
    let graha = |one: Option<teistro::catalogue::Graha>| one.map_or("-", |graha| graha.full_key());
    let legs = held.legs.as_ref().map_or_else(
        || String::from("-"),
        |legs| {
            legs.iter()
                .map(pair_said)
                .collect::<Vec<String>>()
                .join("/")
        },
    );
    let afflictions = held.afflictions.map_or_else(
        || String::from("-"),
        |both| both.map(affliction_said).join("/"),
    );
    format!(
        "{}:{}:{}:{}:{legs}:{afflictions}",
        wire_key(&held.yoga),
        graha(held.through),
        graha(held.entering),
        if held.between.is_some() { "pair" } else { "-" },
    )
}

/// One lord's afflictions as the boundary spells its clauses (its
/// `TsAffliction` keys), `+`-joined, or `none`.
fn affliction_said(affliction: teistro::Affliction) -> String {
    let teistro::Affliction {
        graha: _,
        retrograde,
        combust,
        debilitated,
        trika,
        under_malefic,
    } = affliction;
    let said: Vec<&str> = [
        ("RETROGRADE", retrograde),
        ("COMBUST", combust),
        ("DEBILITATED", debilitated),
        ("TRIKA", trika),
        ("UNDER_MALEFIC", under_malefic),
    ]
    .into_iter()
    .filter_map(|(name, holds)| holds.then_some(name))
    .collect();
    if said.is_empty() {
        String::from("none")
    } else {
        said.join("+")
    }
}

fn the_points(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(points) = document.points.as_ref() else {
        return;
    };
    put(
        report,
        &format!("chart-{index}-point-count"),
        points.all().len().to_string(),
    );
    for (at, found) in points.all().iter().enumerate() {
        let key = |what: &str| format!("chart-{index}-point-{at}{what}");
        put(report, &key(""), found.point.full_key().to_owned());
        put(report, &key("-lon"), number(found.longitude_deg));
        put(report, &key("-sign"), found.sign.full_key().to_owned());
        put(
            report,
            &key("-sign-edge"),
            number(found.boundaries.sign_deg),
        );
    }
}

/// One chart's drishti, as the report prints them.
///
/// **Every one**, because the count differs from chart to chart -- two
/// charts of the same nine grahas hold 47 relations and 40 -- so a
/// runner that printed only the count would agree with the others while
/// the rows disagreed. It is why the boundary's section is ragged.
/// Every dasha the request named: its seed, its balance, every period to the
/// settings' depth and the chain running 5000 days after birth, the chain
/// asked of the cursor rebuilt from the document where the other three walk
/// the periods they decoded.
/// The strength measures as the other three print them.
fn the_strength(report: &mut Report, index: usize, document: &teistro::Document) {
    the_ashtakavarga(report, index, document);
    the_vimshopaka(report, index, document);
    the_vaiseshikamsa(report, index, document);
    the_dasha_phala(report, index, document);
    the_jaimini(report, index, document);
    the_shadbala(report, index, document);
    the_bhava_bala(report, index, document);
}

/// The Bhava bala as the other three print it: each bhava's lord and its
/// components.
fn the_bhava_bala(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(reading) = document.bhava_bala.as_ref() else {
        return;
    };
    for bhava in &reading.bhavas {
        let values = [
            bhava.adhipati,
            bhava.dig,
            bhava.drishti,
            bhava.special,
            bhava.virupas,
        ];
        put(
            report,
            &format!("chart-{index}-bhava-bala-{}", bhava.bhava),
            format!("{} {}", bhava.lord.full_key(), values.map(number).join(",")),
        );
    }
}

/// The Shadbala as the other three print it: each graha's seventeen
/// components, then its totals and whether it is strong.
fn the_shadbala(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(shadbala) = document.shadbala.as_ref() else {
        return;
    };
    for graha in &shadbala.grahas {
        let key = format!("chart-{index}-shadbala-{}", graha.graha.full_key());
        let (st, ka) = (&graha.sthana, &graha.kaala);
        let parts = [
            st.uchcha,
            st.saptavargaja,
            st.ojayugma,
            st.kendradi,
            st.drekkana,
            graha.dig,
            ka.nathonnatha,
            ka.paksha,
            ka.tribhaga,
            ka.abda,
            ka.masa,
            ka.vara,
            ka.hora,
            ka.ayana,
            ka.yuddha,
            graha.cheshta,
            graha.naisargika,
            graha.drik,
        ];
        put(report, &key, parts.map(number).join(","));
        put(
            report,
            &format!("{key}-total"),
            format!(
                "{},{},{},{},{},{},{},{}",
                number(graha.virupas),
                number(graha.rupas),
                number(graha.required_rupas),
                graha.strong,
                number(graha.ishta),
                number(graha.kashta),
                number(graha.subha_rashmi),
                number(graha.ashubha_rashmi)
            ),
        );
    }
}

/// The Vaiseshikamsa as the other three print it: each scheme's count and
/// name, and whether the graha is impaired.
fn the_vaiseshikamsa(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(reading) = document.vaiseshikamsa.as_ref() else {
        return;
    };
    for graha in &reading.grahas {
        let standings = [
            graha.shadvarga,
            graha.saptavarga,
            graha.dashavarga,
            graha.shodashavarga,
        ];
        let named = standings
            .map(|s| {
                format!(
                    "{}:{}",
                    s.good_vargas,
                    s.name.map_or("null", |n| n.full_key())
                )
            })
            .join(",");
        put(
            report,
            &format!("chart-{index}-vaiseshikamsa-{}", graha.graha.full_key()),
            format!("{named} {}", graha.impaired),
        );
    }
}

/// The dasha phala as the other three print it: the seven Subhankas, the
/// nature, the phase and the two flags.
fn the_dasha_phala(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(reading) = document.dasha_phala.as_ref() else {
        return;
    };
    for graha in &reading.grahas {
        put(
            report,
            &format!("chart-{index}-dasha-phala-{}", graha.graha.full_key()),
            format!(
                "{} {} {} {} {}",
                graha.subhankas.map(number).join(","),
                graha.nature.full_key(),
                wire_key(&graha.phase),
                graha.favourable,
                graha.unfavourable
            ),
        );
    }
}

/// Jaimini's significators as the other three print them: the karakamsha
/// with its houses in both charts, and the Brahma graha's rule, count, marks
/// and answer, `-` for each that is absent.
fn the_jaimini(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(reading) = document.jaimini.as_ref() else {
        return;
    };
    let (k, b) = (&reading.karakamsha, &reading.brahma);
    let houses = |of: [u8; 9]| of.map(|house| house.to_string()).join(",");
    put(
        report,
        &format!("chart-{index}-jaimini"),
        format!(
            "{} {} {} {}",
            k.atmakaraka.full_key(),
            k.sign.full_key(),
            houses(k.in_rasi),
            houses(k.in_navamsha)
        ),
    );
    let or_dash = |key: Option<String>| key.unwrap_or_else(|| "-".to_owned());
    let arudhas: Vec<String> = reading
        .graha_arudhas
        .iter()
        .map(|sign| or_dash(sign.map(|s| s.full_key().to_owned())))
        .collect();
    put(
        report,
        &format!("chart-{index}-graha-arudhas"),
        arudhas.join(","),
    );
    let qualified: Vec<&str> = b.qualified.iter().map(|g| g.full_key()).collect();
    put(
        report,
        &format!("chart-{index}-brahma"),
        format!(
            "{} {} {} {} {} {}",
            wire_key(&b.rule),
            b.counted_from.full_key(),
            or_dash((!qualified.is_empty()).then(|| qualified.join(","))),
            or_dash(b.graha.map(|g| g.full_key().to_owned())),
            or_dash(b.passed_from.map(|g| g.full_key().to_owned())),
            or_dash(b.none.as_ref().map(wire_key)),
        ),
    );
}

/// The Vimshopaka as the other three print it: the scoring, and each
/// graha's four scores.
fn the_vimshopaka(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(vs) = document.vimshopaka.as_ref() else {
        return;
    };
    put(
        report,
        &format!("chart-{index}-vimshopaka"),
        wire_key(&vs.scoring),
    );
    for graha in &vs.grahas {
        put(
            report,
            &format!("chart-{index}-vimshopaka-{}", graha.graha.full_key()),
            [
                graha.shadvarga,
                graha.saptavarga,
                graha.dashavarga,
                graha.shodashavarga,
            ]
            .map(number)
            .join(","),
        );
    }
}

/// The Ashtakavarga as the other three print it: the reading, each graha's
/// bindus, reductions and pindas, and the chart's sums.
fn the_ashtakavarga(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(av) = document.ashtakavarga.as_ref() else {
        return;
    };
    let join = |values: &[u16]| {
        values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };
    put(
        report,
        &format!("chart-{index}-ashtakavarga"),
        format!(
            "{} {}",
            wire_key(&av.rules.shodhana),
            wire_key(&av.rules.ekadhipatya)
        ),
    );
    for graha in &av.grahas {
        let key = format!("chart-{index}-ashtakavarga-{}", graha.graha.full_key());
        put(report, &key, join(&graha.bindus.map(u16::from)));
        put(
            report,
            &format!("{key}-reduced"),
            graha
                .reduced
                .map_or_else(|| String::from("null"), |r| join(&r.map(u16::from))),
        );
        put(
            report,
            &format!("{key}-pindas"),
            format!(
                "{},{},{}",
                graha.rashi_pinda, graha.graha_pinda, graha.yoga_pinda
            ),
        );
    }
    put(
        report,
        &format!("chart-{index}-sarvashtakavarga"),
        format!(
            "{};{};{}",
            join(&av.sarva),
            join(&av.trikona),
            join(&av.reduced)
        ),
    );
}

fn the_dashas(report: &mut Report, sdk: &Context, index: usize, document: &teistro::Document) {
    let null = || String::from("null");
    put(
        report,
        &format!("chart-{index}-dasha-count"),
        document.dashas.len().to_string(),
    );
    for (at, dasha) in document.dashas.iter().enumerate() {
        let key = |what: &str| format!("chart-{index}-dasha-{at}{what}");
        put(report, &key(""), dasha.system.full_key());
        put(
            report,
            &key("-seed"),
            dasha
                .seed
                .map_or_else(null, |seed| seed.full_key().to_owned()),
        );
        put(
            report,
            &key("-first-lord"),
            dasha.first_lord.full_key().to_owned(),
        );
        put(report, &key("-overflow"), dasha.overflow.to_string());
        let balance = dasha.balance;
        put(
            report,
            &key("-balance"),
            balance.map_or_else(null, |b| wire_key(&b.method)),
        );
        put(
            report,
            &key("-remaining"),
            balance.map_or_else(null, |b| number(b.remaining)),
        );
        put(
            report,
            &key("-balance-days"),
            balance.map_or_else(null, |b| number(b.days)),
        );
        put(
            report,
            &key("-balance-written"),
            balance.map_or_else(null, |b| {
                let w = b.written;
                format!(
                    "{},{},{},{},{}",
                    w.years, w.months, w.days, w.hours, w.minutes
                )
            }),
        );
        let span = |end: fn(&Interval) -> f64| {
            dasha
                .moon_span
                .as_ref()
                .map_or_else(null, |span| number(end(span)))
        };
        put(report, &key("-moon-span-from"), span(|s| s.from.get()));
        put(report, &key("-moon-span-to"), span(|s| s.to.get()));
        put(report, &key("-depth"), dasha.depth.get().to_string());
        put(report, &key("-periods"), dasha.periods.len().to_string());
        // The first two levels of every period: enough to hold the order,
        // the signs, the lords and the shares, without printing a tree of
        // every depth four times.
        for (k, period) in dasha.periods.iter().enumerate() {
            if period.path.matches('/').count() > 1 {
                continue;
            }
            let sign = period
                .sign
                .map_or_else(String::new, |sign| format!(" {}", sign.full_key()));
            put(
                report,
                &key(&format!("-period-{k}")),
                format!("{}{sign} {}", period.path, period.lord.full_key()),
            );
            put(
                report,
                &key(&format!("-period-{k}-from")),
                number(period.interval.from.get()),
            );
            put(
                report,
                &key(&format!("-period-{k}-to")),
                number(period.interval.to.get()),
            );
        }
        let chain = sdk.chart().dasha(document, &dasha.system).map_or_else(
            |error| format!("refused: {error}"),
            |cursor| {
                let instant = JulianDay::literal(document.foundation.instant.get() + 5000.0);
                cursor
                    .at(instant, dasha.depth)
                    .iter()
                    .map(|period| period.path.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            },
        );
        put(report, &key("-at"), chain);
    }
}

fn the_drishti(report: &mut Report, index: usize, document: &teistro::Document) {
    let Some(aspects) = document.aspects.as_ref() else {
        return;
    };
    put(
        report,
        &format!("chart-{index}-aspect-count"),
        aspects.all().len().to_string(),
    );
    for (at, drishti) in aspects.all().iter().enumerate() {
        let key = |what: &str| format!("chart-{index}-aspect-{at}{what}");
        put(
            report,
            &key(""),
            format!("{}>{}", drishti.from.full_key(), drishti.to.full_key()),
        );
        put(report, &key("-houses"), drishti.houses.to_string());
        put(report, &key("-strength"), wire_key(&drishti.strength));
        put(
            report,
            &key("-from-sign"),
            number(drishti.from_edge.sign_deg),
        );
        put(report, &key("-to-sign"), number(drishti.to_edge.sign_deg));
    }
}

/// A local day's every field, under the same keys for a chart's day and
/// an almanac's, because every binding hands back one record for both.
fn put_day(report: &mut Report, prefix: &str, day: &LocalDay) {
    let key = |what: &str| format!("{prefix}-{what}");
    put(report, &key("vara"), day.vara.full_key().to_owned());
    put(report, &key("sunrise"), number(day.sunrise.get()));
    put(report, &key("sunset"), number(day.sunset.get()));
    put(report, &key("next-sunrise"), number(day.next_sunrise.get()));
    put(
        report,
        &key("date"),
        format!("{}-{}-{}", day.date.year, day.date.month, day.date.day),
    );
    put(
        report,
        &key("calendar"),
        day.date.calendar.full_key().to_owned(),
    );
    put(
        report,
        &key("era"),
        day.date
            .era
            .map_or_else(|| String::from("none"), |era| era.era.full_key().to_owned()),
    );
    put(
        report,
        &key("era-year"),
        day.date.era.map_or(0, |era| era.year).to_string(),
    );
    put(report, &key("resolution"), wire_key(&day.date.resolution));
    put(
        report,
        &key("polar"),
        match day.state {
            DayState::Normal => String::from("none"),
            DayState::Polar { kind, policy } => {
                format!("{}/{}", wire_key(&kind), wire_key(&policy))
            }
        },
    );
    put(
        report,
        &key("convention"),
        match day.convention {
            SunriseConvention::Named { which } => wire_key(&which),
            SunriseConvention::Custom { altitude_deg } => {
                format!("custom {}", number(altitude_deg))
            }
            SunriseConvention::Atmospheric { which, air } => {
                let air = air.at(day.place.altitude);
                format!(
                    "{} {} hPa {} C",
                    wire_key(&which),
                    number(air.pressure_hpa),
                    number(air.temperature_c)
                )
            }
        },
    );
}

/// One day of the almanac, as the report prints it.
fn one_day(report: &mut Report, index: usize, day: &teistro_panchanga::almanac::Panchanga) {
    put_day(report, &format!("day-{index}"), &day.day);
    put(
        report,
        &format!("day-{index}-window-from"),
        number(day.window.from.get()),
    );
    put(
        report,
        &format!("day-{index}-window-to"),
        number(day.window.to.get()),
    );
    one_day_shape(report, index, day);
}

/// One day's month, ayana, omens and the muhurtas it may not have.
fn one_day_shape(report: &mut Report, index: usize, day: &teistro_panchanga::almanac::Panchanga) {
    // The month, the ayana, the omens and the muhurtas that a day
    // may not have -- an absent value must be **absent** in all
    // four, not nought in one.
    put(
        report,
        &format!("day-{index}-month"),
        day.month.month.full_key().to_owned(),
    );
    put(
        report,
        &format!("day-{index}-amanta"),
        day.month.amanta.full_key().to_owned(),
    );
    put(
        report,
        &format!("day-{index}-purnimanta"),
        day.month.purnimanta.full_key().to_owned(),
    );
    put(
        report,
        &format!("day-{index}-paksha"),
        day.month.paksha.full_key().to_owned(),
    );
    put(
        report,
        &format!("day-{index}-convention"),
        wire_key(&day.month.convention),
    );
    put(
        report,
        &format!("day-{index}-month-kind"),
        wire_key(&day.month.kind),
    );
    put(
        report,
        &format!("day-{index}-ayana"),
        day.sun.ayana.full_key().to_owned(),
    );
    put(
        report,
        &format!("day-{index}-ritu"),
        day.sun.ritu.full_key().to_owned(),
    );
    put(
        report,
        &format!("day-{index}-disha-shool"),
        day.omens.disha_shool.full_key().to_owned(),
    );
    let absent_or = |value: Option<f64>| value.map_or_else(|| String::from("none"), number);
    put(
        report,
        &format!("day-{index}-sankranti"),
        absent_or(day.sun.sankranti.map(teistro::quantity::JulianDay::get)),
    );
    put(
        report,
        &format!("day-{index}-abhijit"),
        absent_or(day.muhurtas.abhijit.map(|span| span.from.get())),
    );
    put(
        report,
        &format!("day-{index}-abhijit-effective"),
        if day.muhurtas.abhijit.is_none() {
            String::from("none")
        } else {
            day.muhurtas.abhijit_effective.to_string()
        },
    );
    put(
        report,
        &format!("day-{index}-brahma"),
        absent_or(day.muhurtas.brahma.map(|span| span.from.get())),
    );

    // The counts are what the ragged layout turns on: a prefix sum
    // off by a day would leave these agreeing and the spans below
    // disagreeing.
    for (what, count) in [
        ("tithi", day.limbs.tithi.len()),
        ("nakshatra", day.limbs.nakshatra.len()),
        ("yoga", day.limbs.yoga.len()),
        ("karana", day.limbs.karana.len()),
        ("kaala", day.kaalas.len()),
        ("choghadiya", day.choghadiya.len()),
        ("hora", day.horas.len()),
        ("moon-event", day.moon.rises.len() + day.moon.sets.len()),
        ("panchaka", day.omens.panchaka.len()),
        ("moon-sign", day.moon.signs.len()),
        ("sun-sign", day.sun.signs.len()),
        ("muhurta-yoga", day.omens.yogas.len()),
    ] {
        put(
            report,
            &format!("day-{index}-{what}-count"),
            count.to_string(),
        );
    }
    one_day_limbs(report, index, day);
    one_day_items(report, index, day);
}

/// One day's periods, item by item: the kaalas, the first choghadiya,
/// the horas at both ends, the muhurtas and what the Moon did.
///
/// Counted above and **named** here, because a count agreeing is not the
/// same as the items agreeing: two lists of three can hold different
/// threes. The horas are read at both ends for the same reason a chart
/// section walks two charts -- a list built backwards agrees on its
/// length and on nothing else.
fn one_day_items(report: &mut Report, index: usize, day: &teistro_panchanga::almanac::Panchanga) {
    for (at, kaala) in day.kaalas.iter().enumerate() {
        put(
            report,
            &format!("day-{index}-kaala-{at}"),
            kaala.kaala.full_key().to_owned(),
        );
        put(
            report,
            &format!("day-{index}-kaala-{at}-from"),
            number(kaala.at.from.get()),
        );
    }
    if let Some(first) = day.horas.first() {
        put(
            report,
            &format!("day-{index}-hora-0-lord"),
            first.lord.full_key().to_owned(),
        );
        put(
            report,
            &format!("day-{index}-hora-0-start"),
            number(first.start.get()),
        );
    }
    if let Some(last) = day.horas.last() {
        put(
            report,
            &format!("day-{index}-hora-23-lord"),
            last.lord.full_key().to_owned(),
        );
    }
    if let Some(first) = day.choghadiya.first() {
        put(
            report,
            &format!("day-{index}-choghadiya-0"),
            first.choghadiya.full_key().to_owned(),
        );
        put(
            report,
            &format!("day-{index}-choghadiya-0-daytime"),
            first.is_daytime.to_string(),
        );
    }
    // The other three read one flat list of thirty with a `daylight`
    // flag, where this surface has the daylight's fifteen and the
    // night's fifteen apart. So the count is the sum, the first is the
    // daylight's first, and "is the last one a daylight muhurta" is
    // "does this day have no night" -- which is the polar case and
    // nothing else.
    put(
        report,
        &format!("day-{index}-muhurta-count"),
        (day.muhurtas.daylight.len() + day.muhurtas.night.len()).to_string(),
    );
    if let Some(first) = day.muhurtas.daylight.first() {
        put(
            report,
            &format!("day-{index}-muhurta-0-from"),
            number(first.from.get()),
        );
    }
    put(
        report,
        &format!("day-{index}-muhurta-last-daylight"),
        day.muhurtas.night.is_empty().to_string(),
    );
    // A rise and a set are two lists here and one discriminated list at
    // the boundary, concatenated rises first -- so the flattening is
    // what the other three decode, and this is it in reverse.
    for (at, (kind, instant)) in day
        .moon
        .rises
        .iter()
        .map(|at| ("RISE", at))
        .chain(day.moon.sets.iter().map(|at| ("SET", at)))
        .enumerate()
    {
        put(
            report,
            &format!("day-{index}-moon-{at}-kind"),
            kind.to_owned(),
        );
        put(
            report,
            &format!("day-{index}-moon-{at}-instant"),
            number(instant.get()),
        );
    }
}

/// One span of one limb, flattened so the four lists can be walked as
/// one: the member's key, when it ran, the part inside the day, which of
/// the day's sunrises it held, and its end in ghatis from sunrise.
type LimbSpan = (
    String,
    Interval,
    Interval,
    teistro::Sunrises,
    teistro::GhatiPala,
);

/// A limb's span as `LimbSpan` holds it.
fn limb_span<T>(
    day: &teistro_panchanga::almanac::Panchanga,
    span: &teistro::Span<T>,
    key: &str,
) -> LimbSpan {
    (
        key.to_owned(),
        span.whole,
        span.inside,
        day.sunrises(span),
        day.ghati_pala(span.whole.to).expect("an end the day reads"),
    )
}

/// One day's limb spans, which are the bulk of its rows.
fn one_day_limbs(report: &mut Report, index: usize, day: &teistro_panchanga::almanac::Panchanga) {
    // And every span of every moving limb, member and bounds.
    let limbs: [(&str, Vec<LimbSpan>); 4] = [
        (
            "tithi",
            day.limbs
                .tithi
                .iter()
                .map(|span| limb_span(day, span, span.member.full_key()))
                .collect(),
        ),
        (
            "nakshatra",
            day.limbs
                .nakshatra
                .iter()
                .map(|span| limb_span(day, span, span.member.full_key()))
                .collect(),
        ),
        (
            "yoga",
            day.limbs
                .yoga
                .iter()
                .map(|span| limb_span(day, span, span.member.full_key()))
                .collect(),
        ),
        (
            "karana",
            day.limbs
                .karana
                .iter()
                .map(|span| limb_span(day, span, span.member.full_key()))
                .collect(),
        ),
    ];
    for (what, spans) in &limbs {
        for (at, (member, whole, inside, sunrises, ends)) in spans.iter().enumerate() {
            put(report, &format!("day-{index}-{what}-{at}"), member.clone());
            put(
                report,
                &format!("day-{index}-{what}-{at}-whole-from"),
                number(whole.from.get()),
            );
            put(
                report,
                &format!("day-{index}-{what}-{at}-inside-to"),
                number(inside.to.get()),
            );
            put(
                report,
                &format!("day-{index}-{what}-{at}-sunrises"),
                wire_key(sunrises),
            );
            put(
                report,
                &format!("day-{index}-{what}-{at}-ends"),
                format!("{}-{}-{}", ends.ghati, ends.pala, ends.vipala),
            );
        }
    }
}

/// Each member's own content hash, as a member of a batch handed out alone
/// carries it: the hash of what the batch lists for it — a chart's document
/// with what it answers by rule, a day's panchanga — which the other three
/// read off the blob's `content_hashes` section.
fn own_hashes(report: &mut Report, what: &str, hashes: &[teistro::Hash]) {
    for (index, hash) in hashes.iter().enumerate() {
        put(
            report,
            &format!("{what}-{index}-content-hash"),
            hash.to_string(),
        );
    }
}

/// An almanac over three days, as the report prints it.
///
/// Three, because a day's lists are ragged and two consecutive days with
/// the same counts would not exercise the offsets -- the same reason the
/// other three runners ask for three.
fn an_almanac(report: &mut Report, geo: &Context, place: &Place, offset: UtcOffset) {
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 17);
    let to = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 19);
    let (week, own) = geo
        .almanac()
        .of_each(&from, &to, place, offset)
        .expect("the test provider");
    put(report, "almanac-days", week.value.len().to_string());
    if let Some(first) = week.value.first() {
        put(report, "almanac-model-fnv", fnv(&first.day.model));
    }
    put(
        report,
        "almanac-provenance-fnv",
        fnv(&teistro::canonical_json(&week.provenance)),
    );
    put(
        report,
        "almanac-calendar",
        from.calendar.full_key().to_owned(),
    );
    put(report, "almanac-place-lat", number(place.latitude.get()));

    for (index, day) in week.value.iter().enumerate() {
        one_day(report, index, day);
    }
    own_hashes(report, "day", &own);

    // `day` is the range of one, unwrapped, and must agree with it.
    let one = geo
        .almanac()
        .day(&from, place, offset)
        .expect("the test provider");
    put(
        report,
        "almanac-single-agrees",
        week.value
            .first()
            .is_some_and(|first| {
                first.day.sunrise.get().to_bits() == one.value.day.sunrise.get().to_bits()
            })
            .to_string(),
    );
}

/// A rule of a consumer's own the runners hand in beside the shipped pack:
/// Lakshmi puja on whichever day holds the new moon at sunrise.
const OWN_RULE: &str = r#"{"key":"LAKSHMI_PUJA","source":"the tithi at sunrise","month":"masa.ASHWINA","tithi":"tithi.AMAVASYA","at":{"window":"SUNRISE"},"decide":[],"otherwise":"LATER"}"#;

/// A following rule of a consumer's own the runners hand in beside the
/// Nepal pack: an observance two days after Lakshmi puja's.
const OWN_FOLLOWING: &str =
    r#"{"key":"TWO_AFTER","source":"two days after Lakshmi puja","after":"LAKSHMI_PUJA","days":2}"#;

/// The festivals a panchanga request carries, the shipped pack, the pack
/// amended by a rule of the consumer's own and the Nepal pack with a
/// following rule of the consumer's own, as the report prints them.
///
/// 2024-10-10..11-03 at the test provider holds Vijaya Dashami, Lakshmi
/// puja and two Ekadashis, so an observance's case, guard and dates and a
/// fast's vedha and excess cross every layer, and the amended pack proves
/// a rule is replaced by its key; the Nepal pack's counted day proves a
/// following rule's `after` and `days` cross too.
fn festivals(report: &mut Report, geo: &Context, place: &Place, offset: UtcOffset) {
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 10, 10);
    let to = CalendarDate::defined(Calendar::Gregorian, 2024, 11, 3);
    for (name, rules) in [
        ("shipped", r#""DHARMASINDHU""#.to_owned()),
        ("amended", format!(r#"["DHARMASINDHU",{OWN_RULE}]"#)),
        ("nepal", format!(r#"["NEPAL",{OWN_FOLLOWING}]"#)),
    ] {
        let asked = teistro::FestivalRequest::from_json(&format!(r#"{{"rules":{rules}}}"#))
            .expect("a festival request");
        let answer = geo
            .almanac()
            .festivals(&from, &to, place, offset, &asked)
            .expect("the test provider")
            .value;
        let key = |what: &str| format!("festivals-{name}{what}");
        put(
            report,
            &key("-counts"),
            format!("{} {}", answer.observances.len(), answer.unjudged.len()),
        );
        // The boundary seals the full-keyed value, so the hash every
        // binding reads is of that.
        let written = answer.in_full().expect("a written answer");
        put(
            report,
            &key("-hash"),
            teistro_core::envelope::content_hash(&written).to_string(),
        );
        for (k, observance) in answer.observances.iter().enumerate() {
            let by = match &observance.decided_by {
                teistro::festival::Decided::Guard { index } => format!("guard:{index}"),
                teistro::festival::Decided::Otherwise => "otherwise".to_owned(),
                teistro::festival::Decided::After { rule, days } => format!("after:{rule}:{days}"),
            };
            let [earlier, later] = &observance.extents;
            put(
                report,
                &key(&format!("-{k}")),
                format!(
                    "{} {} {} {}-{} {} {by} {} {} {} {}",
                    observance.rule,
                    observance.month.full_key(),
                    observance.adhika,
                    observance.day.month,
                    observance.day.day,
                    tag(&observance.case),
                    tag(&observance.choice),
                    number(observance.tithi.from.get()),
                    number(earlier.held),
                    number(later.held),
                ),
            );
        }
        for (k, fast) in answer.ekadashis.iter().enumerate() {
            put(
                report,
                &key(&format!("-ekadashi-{k}")),
                format!(
                    "{} {} {} {} {}-{} {} {} {} {} {}",
                    fast.rule,
                    fast.tithi.full_key(),
                    fast.month.full_key(),
                    fast.adhika,
                    fast.day.month,
                    fast.day.day,
                    fast.pierced_at.as_ref().map_or_else(|| "-".to_owned(), tag),
                    fast.pierced,
                    tag(&fast.excess),
                    tag(&fast.choice),
                    number(fast.tithis[1].from.get()),
                ),
            );
        }
    }
}

/// The lunar years a panchanga request carries, as the report prints them.
///
/// 2024-03-20..04-20 at the test provider holds a Chaitra Shukla
/// Pratipada, so two years, their bounds and their Jovian years cross
/// every layer.
fn lunar_years(report: &mut Report, geo: &Context, place: &Place, offset: UtcOffset) {
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 3, 20);
    let to = CalendarDate::defined(Calendar::Gregorian, 2024, 4, 20);
    let years = geo
        .almanac()
        .years(&from, &to, place, offset)
        .expect("the test provider")
        .value;
    put(report, "years-count", years.len().to_string());
    // The boundary seals the full-keyed value, so the hash every binding
    // reads is of that.
    let written = teistro::LunarYear::in_full(&years).expect("written years");
    put(
        report,
        "years-hash",
        teistro_core::envelope::content_hash(&written).to_string(),
    );
    for (k, year) in years.iter().enumerate() {
        put(
            report,
            &format!("years-{k}"),
            format!(
                "{} {} {} {} {} {} {} {}",
                year.samvatsara.full_key(),
                tag(&year.count),
                year.vikrama,
                year.shaka,
                number(year.opened.get()),
                number(year.began.get()),
                number(year.ended.get()),
                year.lupta.map_or("-", |lupta| lupta.full_key()),
            ),
        );
        put(
            report,
            &format!("years-{k}-jovian"),
            listed(year.jovian.iter().map(|jovian| {
                format!(
                    "{}:{}:{}",
                    jovian.member.full_key(),
                    jovian.count,
                    number(jovian.from.get())
                )
            })),
        );
    }
}

/// The Nepal Sambat dates a panchanga request carries, as the report
/// prints them.
///
/// 2024-10-30..11-03 at the test provider holds Kartika's new moon, where
/// the year turns.
fn nepal_sambat(report: &mut Report, geo: &Context, place: &Place, offset: UtcOffset) {
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 10, 30);
    let to = CalendarDate::defined(Calendar::Gregorian, 2024, 11, 3);
    let dates: Vec<teistro::NepalSambatDate> = geo
        .almanac()
        .of(&from, &to, place, offset)
        .expect("the test provider")
        .value
        .iter()
        .map(teistro::Panchanga::nepal_sambat)
        .collect();
    // The boundary seals the full-keyed value, so the hash every binding
    // reads is of that.
    let written = teistro::NepalSambatDate::in_full(&dates).expect("written dates");
    put(
        report,
        "nepal-sambat-hash",
        teistro_core::envelope::content_hash(&written).to_string(),
    );
    put(
        report,
        "nepal-sambat",
        listed(dates.iter().map(|date| {
            format!(
                "{}:{}:{}:{}",
                date.year,
                date.month,
                tag(&date.kind),
                date.paksha.full_key()
            )
        })),
    );
}

/// The eclipses a panchanga request carries, as the report prints them.
///
/// September 2025 at Kathmandu over the built-in sky, which the test
/// provider cannot complete: a total lunar eclipse seen whole and a
/// partial solar one the place does not see (`03-design/eclipses.md`).
fn eclipses(report: &mut Report, place: &Place, offset: UtcOffset) {
    let sky = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a context over the built-in sky");
    let found = sky
        .almanac()
        .eclipses(
            &CalendarDate::defined(Calendar::Gregorian, 2025, 9, 1),
            &CalendarDate::defined(Calendar::Gregorian, 2025, 9, 30),
            place,
            offset,
        )
        .expect("the built-in sky finds eclipses");
    // The boundary seals the full-keyed value, as it does the years.
    let written = found.value.in_full().expect("written eclipses");
    put(
        report,
        "eclipses-hash",
        teistro_core::envelope::content_hash(&written).to_string(),
    );
    let value = &found.value;
    put(
        report,
        "eclipses-count",
        format!("{} {}", value.lunar.len(), value.solar.len()),
    );
    for (k, LunarHere { eclipse, here }) in value.lunar.iter().enumerate() {
        put(
            report,
            &format!("eclipses-lunar-{k}"),
            format!(
                "{} {} {} {} {} {}",
                eclipse.kind.full_key().to_owned(),
                tag(&eclipse.shadow),
                number(eclipse.greatest.get()),
                number(eclipse.gamma),
                number(eclipse.umbral_magnitude),
                number(eclipse.penumbral_magnitude),
            ),
        );
        let c = &eclipse.contacts;
        put(
            report,
            &format!("eclipses-lunar-{k}-contacts"),
            [Some(c.p1), c.u1, c.u2, c.u3, c.u4, Some(c.p4)]
                .map(maybe_at)
                .join(" "),
        );
        let mut line = [
            Some(&here.p1),
            here.u1.as_ref(),
            here.u2.as_ref(),
            Some(&here.greatest),
            here.u3.as_ref(),
            here.u4.as_ref(),
            Some(&here.p4),
        ]
        .map(moment)
        .to_vec();
        line.push(seen(here.seen.as_ref()));
        line.push(seen(here.umbral_seen.as_ref()));
        put(report, &format!("eclipses-lunar-{k}-here"), line.join(" "));
    }
    solar_eclipses(report, &value.solar);
}

/// A contact, or `-` for one the eclipse never reaches.
fn maybe_at(at: Option<JulianDay<Ut1>>) -> String {
    at.map_or_else(|| "-".to_owned(), |at| number(at.get()))
}

/// A moment at the place as `at@altitude`, or `-`.
fn moment(m: Option<&LocalMoment>) -> String {
    m.map_or_else(
        || "-".to_owned(),
        |m| format!("{}@{}", number(m.at.get()), number(m.altitude_deg)),
    )
}

/// The stretch seen as `from..to`, or `-`.
fn seen(s: Option<&Visible>) -> String {
    s.map_or_else(
        || "-".to_owned(),
        |s| format!("{}..{}", number(s.from.get()), number(s.to.get())),
    )
}

/// The solar eclipses of [`eclipses`], as the report prints them.
fn solar_eclipses(report: &mut Report, solar: &[SolarHere]) {
    for (k, SolarHere { eclipse, here }) in solar.iter().enumerate() {
        put(
            report,
            &format!("eclipses-solar-{k}"),
            format!(
                "{} {} {} {} {} {}",
                eclipse.kind.full_key().to_owned(),
                number(eclipse.greatest.get()),
                number(eclipse.gamma),
                number(eclipse.magnitude),
                number(eclipse.point.latitude.get()),
                number(eclipse.point.longitude.get()),
            ),
        );
        let line = here.as_ref().map_or_else(
            || "-".to_owned(),
            |here| {
                let mut line = vec![
                    here.kind.full_key().to_owned(),
                    number(here.magnitude),
                    number(here.obscuration),
                ];
                line.extend(
                    [
                        Some(&here.first),
                        here.second.as_ref(),
                        here.third.as_ref(),
                        Some(&here.fourth),
                        Some(&here.maximum),
                    ]
                    .map(moment),
                );
                line.push(seen(here.seen.as_ref()));
                line.join(" ")
            },
        );
        put(report, &format!("eclipses-solar-{k}-here"), line);
    }
}

/// A muhurta window's unwanted placements, as the report prints them.
fn placed(window: &teistro::muhurta::Judgement) -> String {
    listed(
        window
            .clauses
            .iter()
            .filter_map(|clause| match &clause.kind {
                teistro::muhurta::ClauseKind::UnwantedPlacement { house, by } => Some(format!(
                    "{house}:{}",
                    by.iter()
                        .map(|g| g.full_key())
                        .collect::<Vec<_>>()
                        .join(",")
                )),
                _ => None,
            }),
    )
}

/// A muhurta window's score, as the report prints it.
fn scored(window: &teistro::muhurta::Judgement) -> String {
    window.score.as_ref().map_or_else(
        || "none".to_owned(),
        |score| {
            format!(
                "{} {} {}",
                score.value,
                score
                    .capped_at
                    .map_or_else(|| "none".to_owned(), |cap| cap.to_string()),
                listed(score.factors.iter().map(|factor| format!(
                    "{}:{}:{}",
                    tag(&factor.dimension),
                    factor.weight,
                    factor.graha.map_or("none", |graha| graha.full_key()),
                ))),
            )
        },
    )
}

/// The muhurta search a panchanga request carries, under both rankings,
/// as the report prints it.
///
/// 2024-11-25..27 at the test provider: the texts bar all of the day's
/// windows for different reasons, and the baseline scores them, so both
/// the bars and the scores cross every layer; and a thread ceremony,
/// whose unwanted placements cross with their houses and grahas.
fn a_muhurta(report: &mut Report, geo: &Context, place: &Place, offset: UtcOffset) {
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 11, 25);
    let to = CalendarDate::defined(Calendar::Gregorian, 2024, 11, 27);
    for (name, rules, ranking) in [
        ("raman", "RAMAN_MARRIAGE", "TEXTS"),
        ("baseline", "BASELINE_MARRIAGE", "BASELINE"),
        ("upanayana", "RAMAN_UPANAYANA", "TEXTS"),
    ] {
        let asked = teistro::MuhurtaRequest::from_json(&format!(
            r#"{{"rules":"{rules}","ranking":"{ranking}","native":{{"star":"ROHINI","moonSign":"TAURUS","lagna":"LEO"}},"daysWithWindows":3,"most":12}}"#
        ))
        .expect("a muhurta request");
        let answer = geo
            .almanac()
            .muhurta(&from, &to, place, offset, &asked)
            .expect("the test provider")
            .value;
        let key = |what: &str| format!("muhurta-{name}{what}");
        put(
            report,
            &key("-counts"),
            format!(
                "{} {} {} {} {} {}",
                answer.windows.len(),
                answer.closed.len(),
                answer.days_judged,
                answer.days_cut,
                answer.windows_blacked_out,
                tag(&answer.ranking),
            ),
        );
        // The boundary seals the full-keyed value, so the hash every
        // binding reads is of that.
        let written = teistro::muhurta::spelling::in_full(&answer).expect("a written answer");
        put(
            report,
            &key("-hash"),
            teistro_core::envelope::content_hash(&written).to_string(),
        );
        for (k, window) in answer.windows.iter().enumerate() {
            let at = format!("-{k}");
            put(
                report,
                &key(&at),
                format!(
                    "{} {}",
                    number(window.at.from.get()),
                    number(window.at.to.get())
                ),
            );
            put(
                report,
                &key(&format!("{at}-clauses")),
                window
                    .clauses
                    .iter()
                    .map(|clause| tag(&clause.kind.key()))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            put(
                report,
                &key(&format!("{at}-bars")),
                listed(window.barred_by.iter().map(|bar| match bar {
                    teistro::muhurta::Bar::Key(key) => tag(key),
                    teistro::muhurta::Bar::Clause(kind) => tag(&kind.key()),
                })),
            );
            put(report, &key(&format!("{at}-placed")), placed(window));
            put(report, &key(&format!("{at}-score")), scored(window));
        }
        for (j, day) in answer.closed.iter().enumerate() {
            put(
                report,
                &key(&format!("-closed-{j}")),
                format!(
                    "{}-{} {}",
                    day.date.month,
                    day.date.day,
                    listed(day.by.iter().map(|kind| kind.full_key().to_owned())),
                ),
            );
        }
    }
}

/// A unit enum's key, as serde writes it and every binding reads it.
fn tag<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|written| written.as_str().map(str::to_owned))
        .expect("a unit enum writes a string")
}

/// The items joined by spaces, or `none`.
fn listed(items: impl Iterator<Item = String>) -> String {
    let joined = items.collect::<Vec<_>>().join(" ");
    if joined.is_empty() {
        "none".to_owned()
    } else {
        joined
    }
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
    the_constants(&mut report);
    the_surface(&mut report);
    let place = Place::new(
        Latitude::try_new(27.7172).expect("a latitude"),
        Longitude::try_new(85.324).expect("a longitude"),
        Altitude::try_new(1400.0).expect("an altitude"),
    );
    let offset = UtcOffset::try_from_seconds(20700).expect("+05:45");
    a_topocentric_chart(&mut report, &sdk, &place, offset);
    a_classical_chart(&mut report, &place, offset);
    let (geo, place, offset) = charts(&mut report);
    an_almanac(&mut report, &geo, &place, offset);
    a_muhurta(&mut report, &geo, &place, offset);
    festivals(&mut report, &geo, &place, offset);
    lunar_years(&mut report, &geo, &place, offset);
    nepal_sambat(&mut report, &geo, &place, offset);
    eclipses(&mut report, &place, offset);

    for (key, value) in &report {
        println!("{key}\t{value}");
    }
}

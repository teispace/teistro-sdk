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

use teistro::catalogue::{Calendar, ChartKind, Graha};
use teistro::{Body, CalendarDate, Context, Ephemeris, Frame, PositionRequest, Scale, TimeScale};
use teistro_core::envelope::CalendarResolution;
use teistro_core::interval::Interval;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
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
        kebab(&format!("{:?}", chart.day.part)),
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
    put(
        report,
        &format!("chart-{index}-vara"),
        chart.day.day.vara.full_key().to_owned(),
    );
    put(
        report,
        &format!("chart-{index}-sunrise"),
        number(chart.day.day.sunrise.get()),
    );
    put(
        report,
        &format!("chart-{index}-sunset"),
        number(chart.day.day.sunset.get()),
    );
    put(
        report,
        &format!("chart-{index}-date"),
        format!(
            "{}-{}-{}",
            chart.day.day.date.year, chart.day.day.date.month, chart.day.day.date.day
        ),
    );
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

/// A chart founded under the geocentric profile, as the report prints
/// it.
///
/// Everything here runs on `parashari-classical`, which is geocentric,
/// so the two centres are both exercised -- the same reason the other
/// three runners build a second context.
fn charts(report: &mut Report) -> (Context, Place, UtcOffset) {
    let geo = Context::builder()
        .profile("parashari-classical")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Test])
        .build()
        .expect("a shipped profile");
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
    let founded = geo
        .chart()
        .found_many(&instants, &place, offset, ChartKind::Natal)
        .expect("the test provider");
    put(report, "chart-count", founded.value.len().to_string());
    put(report, "chart-place-lat", number(place.latitude.get()));
    put(report, "chart-place-lon", number(place.longitude.get()));
    put(
        report,
        "chart-provenance-profile",
        founded.provenance.profile.clone(),
    );
    for (index, chart) in founded.value.iter().enumerate() {
        one_chart(report, index, chart);
    }

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

/// One day of the almanac, as the report prints it.
fn one_day(report: &mut Report, index: usize, day: &teistro_panchanga::almanac::Panchanga) {
    put(
        report,
        &format!("day-{index}-vara"),
        day.day.vara.full_key().to_owned(),
    );
    put(
        report,
        &format!("day-{index}-sunrise"),
        number(day.day.sunrise.get()),
    );
    put(
        report,
        &format!("day-{index}-sunset"),
        number(day.day.sunset.get()),
    );
    put(
        report,
        &format!("day-{index}-next-sunrise"),
        number(day.day.next_sunrise.get()),
    );
    put(
        report,
        &format!("day-{index}-date"),
        format!(
            "{}-{}-{}",
            day.day.date.year, day.day.date.month, day.day.date.day
        ),
    );
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
        kebab(&format!("{:?}", day.month.convention)),
    );
    put(
        report,
        &format!("day-{index}-month-kind"),
        kebab(&format!("{:?}", day.month.kind)),
    );
    put(
        report,
        &format!("day-{index}-ayana"),
        day.sun.ayana.full_key().to_owned(),
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
}

/// One span of one limb, flattened so the four lists can be walked as
/// one: the member's key, when it ran, and the part inside the day.
type LimbSpan = (String, Interval, Interval);

/// One day's limb spans, which are the bulk of its rows.
fn one_day_limbs(report: &mut Report, index: usize, day: &teistro_panchanga::almanac::Panchanga) {
    // And every span of every moving limb, member and bounds.
    let limbs: [(&str, Vec<LimbSpan>); 4] = [
        (
            "tithi",
            day.limbs
                .tithi
                .iter()
                .map(|span| (span.member.full_key().to_owned(), span.whole, span.inside))
                .collect(),
        ),
        (
            "nakshatra",
            day.limbs
                .nakshatra
                .iter()
                .map(|span| (span.member.full_key().to_owned(), span.whole, span.inside))
                .collect(),
        ),
        (
            "yoga",
            day.limbs
                .yoga
                .iter()
                .map(|span| (span.member.full_key().to_owned(), span.whole, span.inside))
                .collect(),
        ),
        (
            "karana",
            day.limbs
                .karana
                .iter()
                .map(|span| (span.member.full_key().to_owned(), span.whole, span.inside))
                .collect(),
        ),
    ];
    for (what, spans) in &limbs {
        for (at, (member, whole, inside)) in spans.iter().enumerate() {
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
        }
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
    let week = geo
        .almanac()
        .of(&from, &to, place, offset)
        .expect("the test provider");
    put(report, "almanac-days", week.value.len().to_string());
    put(
        report,
        "almanac-calendar",
        from.calendar.full_key().to_owned(),
    );
    put(report, "almanac-place-lat", number(place.latitude.get()));

    for (index, day) in week.value.iter().enumerate() {
        one_day(report, index, day);
    }

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
    let (geo, place, offset) = charts(&mut report);
    an_almanac(&mut report, &geo, &place, offset);

    for (key, value) in &report {
        println!("{key}\t{value}");
    }
}

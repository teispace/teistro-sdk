//! The rashifal through the façade (`03-design/rashifal.md` step 2): read
//! at the reference day's sunrise, its places the founder's own, and its
//! events found to the period's last minute.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "tests fail by panicking, saying why, and index the nine and the twelve"
)]

use teistro::catalogue::{Calendar, ChartKind, Graha, Rashi};
use teistro::gochar::hits::HitEvent;
use teistro::gochar::{GocharRules, Reference, Transit, gochar};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rashifal::baseline::{Period, ScoreBand, ScoreBands};
use teistro::{
    CalendarDate, Context, Ephemeris, RashifalBatch, RashifalPeriod, RashifalRequest, Snapshot,
    UtcOffset,
};

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

const NEPAL: UtcOffset = UtcOffset::literal(5, 45, 0);

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// A date's year, month and day, whatever era it carries.
fn ymd(date: &CalendarDate) -> (i32, u8, u8) {
    (date.year, date.month, date.day)
}

fn period(sdk: &Context, request: &RashifalRequest) -> RashifalPeriod {
    sdk.chart().rashifal(request).unwrap().value
}

/// The civil day at Nepal's offset an instant falls in.
fn civil_day(sdk: &Context, instant: JulianDay<Utc>) -> CalendarDate {
    let local = instant.get() + f64::from(NEPAL.seconds()) / 86_400.0;
    sdk.calendar()
        .date_of(Calendar::Gregorian, teistro::fixed_of_jd(local).0)
        .unwrap()
}

#[test]
fn the_sky_is_read_at_the_middle_days_sunrise() {
    let sdk = context();
    let week = RashifalRequest::between(date(2026, 10, 4), date(2026, 10, 10), kathmandu(), NEPAL);
    let read = period(&sdk, &week);
    assert_eq!(ymd(&read.reference), (2026, 10, 7));
    let day = sdk
        .almanac()
        .day(&read.reference, &kathmandu(), NEPAL)
        .unwrap()
        .value;
    assert_eq!(read.instant, day.day.sunrise);
    // The places are the founder's at that instant.
    let chart = sdk
        .chart()
        .found(read.instant, &kathmandu(), NEPAL, ChartKind::Natal)
        .unwrap()
        .value;
    for one in &chart.grahas {
        assert_eq!(
            read.transits[one.graha as usize],
            Transit::at_longitude(one.longitude_deg),
            "{:?}",
            one.graha
        );
    }
    // Each sign's gochar is the kernel's from it.
    let rules = GocharRules::of(sdk.settings());
    for reading in &read.readings {
        assert_eq!(
            reading.gochar,
            gochar(Reference::moon(reading.rashi), &read.transits, rules)
        );
    }
    // The nodes move backwards and the luminaries never do.
    assert!(read.retrograde[Graha::Rahu as usize] || read.retrograde[Graha::Ketu as usize]);
    assert!(!read.retrograde[Graha::Sun as usize] && !read.retrograde[Graha::Moon as usize]);
}

#[test]
fn a_clock_snapshot_is_that_minute_of_the_reference_day() {
    let sdk = context();
    let six = RashifalRequest::day(date(2026, 10, 7), kathmandu(), NEPAL)
        .at(Snapshot::Clock { hour: 6, minute: 0 });
    let read = period(&sdk, &six);
    // 06:00 at +05:45 is 00:15 UTC.
    let expected =
        teistro::jd_of_fixed(sdk.calendar().fixed_of(&date(2026, 10, 7)).unwrap()) + 15.0 / 1440.0;
    assert!((read.instant.get() - expected).abs() < 1e-9);
    assert_eq!(
        sdk.chart()
            .rashifal(&six)
            .unwrap()
            .provenance
            .applied_conventions
            .last()
            .unwrap()
            .value,
        "06:00"
    );
}

#[test]
fn every_nodes_ingress_is_found_and_counted_from_each_sign() {
    let sdk = context();
    // The nodes leave Aquarius and Leo in the winter of 2026.
    let year = RashifalRequest::between(date(2026, 1, 1), date(2026, 12, 31), kathmandu(), NEPAL);
    let read = period(&sdk, &year);
    let aries = read.of(Rashi::Aries);
    for node in [Graha::Rahu, Graha::Ketu] {
        let ingress = aries
            .events
            .iter()
            .find(|one| {
                one.event.hit.graha == node
                    && matches!(one.event.hit.event, HitEvent::SignIngress { .. })
            })
            .expect("each node enters a sign in 2026");
        let into = ingress.event.sign;
        for reading in &read.readings {
            let same = reading
                .events
                .iter()
                .find(|one| one.event == ingress.event)
                .unwrap();
            assert_eq!(
                same.house,
                teistro::House::between(reading.rashi, into).get()
            );
        }
    }
}

#[test]
fn an_ingress_on_the_last_evening_belongs_to_the_period() {
    let sdk = context();
    // Saturn leaves Pisces in 2027; find the day, then end a week on it.
    let year = RashifalRequest::between(date(2027, 1, 1), date(2027, 12, 31), kathmandu(), NEPAL)
        .with_events([Graha::Saturn]);
    let read = period(&sdk, &year);
    let ingress = read
        .of(Rashi::Aries)
        .events
        .iter()
        .find(|one| matches!(one.event.hit.event, HitEvent::SignIngress { .. }))
        .unwrap()
        .event;
    let day = civil_day(&sdk, ingress.hit.instant);
    let fixed = sdk.calendar().fixed_of(&day).unwrap();
    let week_before = sdk
        .calendar()
        .date_of(Calendar::Gregorian, fixed.plus_days(-6))
        .unwrap();
    let ending =
        RashifalRequest::between(week_before, day, kathmandu(), NEPAL).with_events([Graha::Saturn]);
    let found = period(&sdk, &ending);
    assert!(
        found
            .of(Rashi::Aries)
            .events
            .iter()
            .any(|one| one.event == ingress)
    );
    // The day after holds it no longer.
    let next = sdk
        .calendar()
        .date_of(Calendar::Gregorian, fixed.plus_days(1))
        .unwrap();
    let after = RashifalRequest::day(next, kathmandu(), NEPAL).with_events([Graha::Saturn]);
    assert_eq!(period(&sdk, &after).of(Rashi::Aries).events, []);
}

#[test]
fn the_baseline_score_reads_the_period_it_was_given() {
    let sdk = context();
    let month = RashifalRequest::between(date(2026, 10, 1), date(2026, 10, 31), kathmandu(), NEPAL);
    let read = period(&sdk, &month);
    assert_eq!(ymd(&read.reference), (2026, 10, 16));
    for rashi in Rashi::ALL {
        let score = read.baseline_score(rashi, Period::Monthly, ScoreBands::DEFAULT);
        assert!(score.overall <= 100, "{rashi:?}");
        assert!(score.key_influences.len() <= 5);
    }
}

/// C2331: what a period reports does not move its score. October 2026
/// asked with no events, with the Moon's, with Saturn's alone and with the
/// default set scores every sign alike, while the events each reports
/// differ; the score always counts C360's set.
#[test]
fn a_score_is_the_same_whatever_events_are_reported() {
    let sdk = context();
    let month =
        || RashifalRequest::between(date(2026, 10, 1), date(2026, 10, 31), kathmandu(), NEPAL);
    let default = period(&sdk, &month());
    let scores = |read: &RashifalPeriod| -> Vec<(u8, ScoreBand)> {
        Rashi::ALL
            .iter()
            .map(|&rashi| {
                let score = read.baseline_score(rashi, Period::Monthly, ScoreBands::DEFAULT);
                (score.overall, score.band())
            })
            .collect()
    };
    let reported = |read: &RashifalPeriod| -> Vec<Graha> {
        read.of(Rashi::Aries)
            .events
            .iter()
            .map(|one| one.event.hit.graha)
            .collect()
    };
    let with_moon = period(
        &sdk,
        &month().with_events(teistro::EVENT_GRAHAS.into_iter().chain([Graha::Moon])),
    );
    let none = period(&sdk, &month().with_events([]));
    let saturn = period(&sdk, &month().with_events([Graha::Saturn]));
    // October's Moon changes sign a dozen times; only the Moon's ask hears of it.
    assert!(reported(&with_moon).contains(&Graha::Moon));
    assert!(!reported(&default).contains(&Graha::Moon));
    assert_eq!(reported(&none), []);
    assert!(
        reported(&saturn)
            .iter()
            .all(|&graha| graha == Graha::Saturn)
    );
    for other in [&with_moon, &none, &saturn] {
        assert_eq!(scores(other), scores(&default));
        assert_eq!(other.scored, default.scored);
    }
    assert!(
        default
            .scored
            .iter()
            .all(|event| event.hit.graha != Graha::Moon),
        "the Moon is never scored"
    );
}

#[test]
fn a_bad_request_is_refused_by_the_field_it_names() {
    let sdk = context();
    let field = |request: &RashifalRequest| {
        sdk.chart()
            .rashifal(request)
            .unwrap_err()
            .field()
            .map(String::from)
    };
    let day = || RashifalRequest::day(date(2026, 10, 7), kathmandu(), NEPAL);
    assert_eq!(
        field(&RashifalRequest::between(
            date(2026, 10, 7),
            date(2026, 10, 6),
            kathmandu(),
            NEPAL
        ))
        .as_deref(),
        Some("last")
    );
    assert_eq!(
        field(&day().at(Snapshot::Clock {
            hour: 24,
            minute: 0
        }))
        .as_deref(),
        Some("snapshot")
    );
    for spells in [vec![1], vec![13], vec![4, 4]] {
        assert_eq!(
            field(&day().with_spells(spells.clone())).as_deref(),
            Some("spells"),
            "{spells:?}"
        );
    }
    assert_eq!(
        field(&day().with_events([Graha::Mars, Graha::Mars])).as_deref(),
        Some("events")
    );
}

#[test]
fn a_batch_reads_each_period_as_it_reads_alone() {
    let sdk = context();
    let periods = [
        RashifalRequest::day(date(2026, 10, 7), kathmandu(), NEPAL),
        RashifalRequest::between(date(2026, 10, 4), date(2026, 10, 10), kathmandu(), NEPAL)
            .at(Snapshot::Clock { hour: 6, minute: 0 }),
    ];
    let batch = sdk.chart().rashifal_many(&periods).unwrap();
    for (one, request) in batch.value.iter().zip(&periods) {
        assert_eq!(*one, period(&sdk, request));
    }
    let stamped: Vec<&str> = batch
        .provenance
        .applied_conventions
        .iter()
        .filter(|one| one.knob == "rashifal.snapshot")
        .map(|one| one.value.as_str())
        .collect();
    assert_eq!(stamped, ["SUNRISE", "06:00"]);
    assert_eq!(
        sdk.chart().rashifal_many(&[]).unwrap_err().field(),
        Some("requests")
    );
}

/// SR-37: each sign's reading in words, through the batch a binding sends,
/// said in every locale that carries the messages, and final in each (Q42).
/// A period's verdict holds for a graha that keeps its sign through it:
/// over Mangsir 2083 (17 November to 15 December 2026) Saturn stays in
/// Pisces while the Moon goes round, and a graha that enters a sign in the
/// period does not hold; a day the Moon stays in her sign holds hers. The
/// plan says a transit that does not hold as the reading's alone.
#[test]
fn a_verdict_holds_for_the_period_only_where_the_graha_keeps_its_sign() {
    let sdk = context();
    let mangsir =
        RashifalRequest::between(date(2026, 11, 17), date(2026, 12, 15), kathmandu(), NEPAL);
    let read = period(&sdk, &mangsir);
    let aries = read.of(Rashi::Aries);
    assert!(aries.holds[Graha::Saturn as usize]);
    assert!(!aries.holds[Graha::Moon as usize]);
    for one in &aries.events {
        if matches!(one.event.hit.event, HitEvent::SignIngress { .. }) {
            assert!(
                !aries.holds[one.event.hit.graha as usize],
                "{:?}",
                one.event.hit.graha
            );
        }
    }
    // Every sign reads the same motion.
    assert!(
        read.readings
            .iter()
            .all(|reading| reading.holds == aries.holds)
    );
    let plan = sdk.interpret().rashifal(aries);
    let passing = plan
        .iter()
        .filter(|item| item.key == "sdk.rashifal.passing")
        .count();
    let moving = aries.holds.iter().filter(|&&kept| !kept).count();
    assert_eq!(passing, moving);
    // 22 November 2026: the Moon stands at 29.5° Pisces at sunrise and
    // enters Aries that day, so even a day's reading does not hold hers.
    let day = period(
        &sdk,
        &RashifalRequest::day(date(2026, 11, 22), kathmandu(), NEPAL),
    );
    let held = day.of(Rashi::Aries).holds;
    assert!(held[Graha::Saturn as usize] && !held[Graha::Moon as usize]);
}

#[test]
fn a_batch_asked_for_words_says_each_sign_in_each_locale() {
    let sdk = context();
    let year = r#"{"periods": [{"first": {"year": 2026, "month": 1, "day": 1},
        "last": {"year": 2026, "month": 12, "day": 31},
        "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
        "utcOffsetSeconds": 20700}], "plans": true}"#;
    let asked = RashifalBatch::from_json(year).unwrap();
    assert!(asked.plans);
    let answers = sdk.chart().rashifal_answers(&asked).unwrap().value;
    let answer = &answers[0];
    assert_eq!(answer.baseline, None);
    let plans = answer.plans.as_ref().expect("words were asked for");
    assert_eq!(plans.len(), 12);
    for (plan, reading) in plans.iter().zip(&answer.period.readings) {
        assert_eq!(*plan, sdk.interpret().rashifal(reading));
    }
    // A year holds both kinds of event, and Saturn's standing for each sign.
    let keys: Vec<&str> = plans.iter().flat_map(|plan| plan.keys()).collect();
    for key in [
        "sdk.rashifal.saturn",
        "sdk.rashifal.ingress",
        "sdk.rashifal.station",
        // The Sun, the Moon and the faster grahas change sign in a year.
        "sdk.rashifal.passing",
    ] {
        assert!(keys.contains(&key), "{key}");
    }
    for tag in ["en-Latn", "ne-Deva-NP", "hi-Deva-IN", "sa-Deva", "sa-Latn"] {
        sdk.intl().set_locale(tag).unwrap();
        for plan in plans {
            for item in plan {
                let said = sdk.intl().render(&item.key, &item.params);
                assert!(
                    !said.is_fallback && said.warnings.is_empty() && !said.text.is_empty(),
                    "{tag} {}: {said:?}",
                    item.key
                );
                if item.key.starts_with("sdk.rashifal.") {
                    assert!(!said.is_draft, "{tag} {} is final", item.key);
                    // Each locale answers the rashifal's lines itself
                    // (SR-151: Sanskrit's written, `sa-Latn`'s derived).
                    assert_eq!(said.resolved_from.as_deref(), Some(tag), "{}", item.key);
                }
            }
        }
    }
    // Without the flag the answer carries no words.
    let plain = RashifalBatch::from_json(&year.replace(r#", "plans": true"#, "")).unwrap();
    assert!(!plain.plans);
    let read = sdk.chart().rashifal_answers(&plain).unwrap().value;
    assert_eq!(read[0].plans, None);
}

/// SR-151: each of the rashifal's four lines in Sanskrit, every variant a
/// reading can reach (the three phases, both named spells, an unnamed
/// spell and none, the four passing verdicts, an ingress either way and
/// both station turns), answered by `sa-Deva` itself and by `sa-Latn`
/// from its derivation, final and distinct. Sade Sati is the loan
/// साढेसाती (C89, C2333).
#[test]
fn the_rashifal_lines_are_said_in_sanskrit_in_both_scripts() {
    use teistro::{Value, params};
    let sdk = context();
    let engine = sdk.locale_engine();
    let saturn = Value::catalogued(Graha::Saturn);
    let leo = Value::catalogued(Rashi::Leo);
    let mut lines = Vec::new();
    for (house, phase, spell) in [
        (12, "RISING", "no"),
        (1, "PEAK", "no"),
        (2, "SETTING", "no"),
        (4, "NONE", "yes"),
        (8, "NONE", "yes"),
        (7, "NONE", "yes"),
        (5, "NONE", "no"),
    ] {
        lines.push((
            "sdk.rashifal.saturn",
            params([
                ("house", Value::Int(house)),
                ("phase", Value::from(phase)),
                ("spell", Value::from(spell)),
            ]),
        ));
    }
    for verdict in ["GOOD", "OBSTRUCTED", "NOT_GOOD", "NONE"] {
        lines.push((
            "sdk.rashifal.passing",
            params([
                ("graha", saturn.clone()),
                ("house", Value::Int(3)),
                ("verdict", Value::from(verdict)),
            ]),
        ));
    }
    for good in ["yes", "no"] {
        lines.push((
            "sdk.rashifal.ingress",
            params([
                ("graha", saturn.clone()),
                ("rashi", leo.clone()),
                ("house", Value::Int(5)),
                ("good", Value::from(good)),
            ]),
        ));
    }
    for turns in ["RETROGRADE", "DIRECT"] {
        lines.push((
            "sdk.rashifal.station",
            params([
                ("graha", saturn.clone()),
                ("turns", Value::from(turns)),
                ("rashi", leo.clone()),
                ("house", Value::Int(5)),
                ("good", Value::from("no")),
            ]),
        ));
    }
    for tag in ["sa-Deva", "sa-Latn"] {
        let mut said = std::collections::BTreeSet::new();
        for (key, asked) in &lines {
            let rendered = engine.render_in(tag, key, asked);
            assert!(rendered.warnings.is_empty(), "{tag} {key}: {rendered:?}");
            assert_eq!(rendered.resolved_from.as_deref(), Some(tag), "{tag} {key}");
            assert!(!rendered.is_draft, "{tag} {key} is final");
            let devanagari = rendered
                .text
                .chars()
                .any(|c| ('\u{0900}'..='\u{097F}').contains(&c));
            assert_eq!(devanagari, tag == "sa-Deva", "{tag}: {}", rendered.text);
            said.insert(rendered.text);
        }
        assert_eq!(said.len(), lines.len(), "{tag}: every variant its own line");
    }
    let rising = engine.render_in("sa-Deva", lines[0].0, &lines[0].1).text;
    assert!(rising.contains("साढेसात्याः"), "{rising}");
}

/// A batch of the days from `first` to `last` of November 2026 at
/// Kathmandu, a binding's record, with the baseline read for `baseline`.
fn november(first: u8, last: u8, baseline: &str) -> RashifalBatch {
    RashifalBatch::from_json(&format!(
        r#"{{"periods": [{{"first": {{"year": 2026, "month": 11, "day": {first}}},
            "last": {{"year": 2026, "month": 11, "day": {last}}},
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
            "utcOffsetSeconds": 20700}}], "baseline": "{baseline}"}}"#
    ))
    .unwrap()
}

/// Each sign's overall and band, Aries to Pisces.
fn bands_of(sdk: &Context, batch: &RashifalBatch) -> Vec<(u8, ScoreBand)> {
    let answers = sdk.chart().rashifal_answers(batch).unwrap().value;
    answers[0]
        .baseline
        .as_ref()
        .expect("a baseline was asked for")
        .iter()
        .map(|score| (score.overall, score.band()))
        .collect()
}

/// `rashifal.score_bands` reaches a score by the context's settings, by a
/// patch the context was built with and by one a call derives with, and
/// moves the band and nothing else.
#[test]
fn a_score_is_banded_by_the_settings_it_was_read_under() {
    let sdk = context();
    assert_eq!(sdk.settings().rashifal.score_bands, ScoreBands::DEFAULT);
    let batch = november(15, 15, "DAILY");
    let default = bands_of(&sdk, &batch);
    for (overall, band) in &default {
        assert_eq!(*band, ScoreBands::DEFAULT.band(*overall));
    }
    // Every score excellent, by a context's own patch and by a call's.
    let lenient = r#"{"rashifal": {"score_bands": [2, 1, 0]}}"#;
    let built = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(lenient)
        .build()
        .unwrap();
    let derived = sdk.with_patch_json(lenient, None).unwrap();
    assert_ne!(derived.settings_hash(), sdk.settings_hash());
    for other in [&built, &derived] {
        assert_eq!(other.settings().rashifal.score_bands, ScoreBands([2, 1, 0]));
        let banded = bands_of(other, &batch);
        for ((overall, band), (before, _)) in banded.iter().zip(&default) {
            assert_eq!(overall, before, "the score does not move");
            assert_eq!(*band, ScoreBands([2, 1, 0]).band(*overall));
        }
    }
    // The answer's JSON carries the band beside the score.
    let answers = sdk.chart().rashifal_answers(&batch).unwrap().value;
    let json = serde_json::to_value(&answers[0]).unwrap();
    let first = &json["baseline"][0];
    assert_eq!(first["band"], default[0].1.key());
    assert_eq!(first["overall"], default[0].0);
}

/// Thresholds that are not strictly descending, or pass 100, are refused,
/// each naming the threshold at fault, wherever the settings are given.
#[test]
fn score_bands_out_of_order_or_range_are_refused_by_their_field() {
    let sdk = context();
    for (bands, field) in [
        ("[101, 50, 30]", "settings.rashifal.score_bands[0]"),
        ("[75, 75, 30]", "settings.rashifal.score_bands[1]"),
        ("[75, 50, 60]", "settings.rashifal.score_bands[2]"),
        ("[30, 50, 75]", "settings.rashifal.score_bands[1]"),
    ] {
        let patch = format!(r#"{{"rashifal": {{"score_bands": {bands}}}}}"#);
        let refused = sdk.with_patch_json(&patch, None).unwrap_err();
        assert_eq!(refused.field(), Some(field), "{bands}: {refused}");
        let built = Context::builder()
            .ephemeris([Ephemeris::Builtin])
            .settings_json(patch.as_str())
            .build()
            .unwrap_err();
        // A built context names the field within its settings, as every
        // coherence refusal at build does.
        assert_eq!(
            built.field(),
            field.strip_prefix("settings."),
            "{bands}: {built}"
        );
    }
    // Neither two thresholds nor a fraction nor a number past a byte reads.
    for bands in [
        "[75, 50]",
        "[75, 50, 30.5]",
        "[300, 50, 30]",
        "[-1, 50, 30]",
    ] {
        let patch = format!(r#"{{"rashifal": {{"score_bands": {bands}}}}}"#);
        let refused = sdk.with_patch_json(&patch, None).unwrap_err();
        assert!(
            refused
                .field()
                .is_some_and(|field| field.starts_with("settings.rashifal.score_bands")),
            "{bands}: {refused}"
        );
    }
}

/// Tithipatra's acceptance (SR-124): the home page's day, every sign, a day
/// and a month, at Kathmandu under the Nepal committee's profile. Every
/// area, colour, verdict and band the answer carries is named, `name` and
/// `short`, in English and in each Devanagari locale in its own script.
#[test]
fn every_word_a_rashifal_answer_carries_is_named_in_each_locale() {
    let sdk = Context::builder()
        .nepal_official([Ephemeris::Builtin])
        .build()
        .unwrap();
    assert_eq!(sdk.profile(), "nepali-committee");
    let mut keys = std::collections::BTreeSet::new();
    // The day, and the month it falls in.
    for batch in [november(15, 15, "DAILY"), november(1, 30, "MONTHLY")] {
        let answers = sdk.chart().rashifal_answers(&batch).unwrap().value;
        let answer = &answers[0];
        let scores = answer.baseline.as_ref().expect("a baseline was asked for");
        assert_eq!(scores.len(), 12);
        for score in scores {
            keys.insert(score.band().full_key());
            keys.insert(score.lucky.colour.full_key());
            keys.extend(score.areas.iter().map(|(area, _)| area.full_key()));
            keys.extend(
                score
                    .key_influences
                    .iter()
                    .map(|one| one.verdict.full_key()),
            );
        }
        for reading in &answer.period.readings {
            keys.extend(
                reading
                    .gochar
                    .grahas
                    .iter()
                    .map(|one| one.verdict.full_key()),
            );
        }
    }
    // Eight areas, a colour, a band and a verdict at the least.
    assert!(keys.len() > 8 + 1 + 1, "{keys:?}");
    for tag in ["en-Latn", "ne-Deva-NP", "hi-Deva-IN", "sa-Deva"] {
        sdk.intl().set_locale(tag).unwrap();
        for key in &keys {
            let entity = sdk.intl().entity(key).unwrap();
            for form in ["name", "short"] {
                let said = entity.form(form);
                assert!(said.is_some(), "{tag} {key} {form}");
                let said = said.unwrap_or_default();
                let devanagari = said.chars().any(|c| ('\u{0900}'..='\u{097F}').contains(&c));
                assert_eq!(devanagari, tag != "en-Latn", "{tag} {key} {form}: {said}");
            }
        }
    }
}

/// A batch asking for the reading voice with `readings`, its period the
/// baseline's.
fn november_read(first: u8, last: u8, baseline: &str) -> RashifalBatch {
    RashifalBatch::from_json(&format!(
        r#"{{"periods": [{{"first": {{"year": 2026, "month": 11, "day": {first}}},
            "last": {{"year": 2026, "month": 11, "day": {last}}},
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
            "utcOffsetSeconds": 20700}}], "baseline": "{baseline}", "readings": true}}"#
    ))
    .unwrap()
}

/// The reading voice through the façade (`03-design/rashifal-reading.md`):
/// twelve plans a period, every item naming facts the answer carries, a
/// week's days read one by one and its best day only when one leads.
#[test]
fn a_period_is_said_in_the_reading_voice_with_its_facts() {
    use teistro::interpret::rashifal_reading::{Fact, ReadingItem, ReadingPlan};
    let sdk = context();
    let items = |plan: &ReadingPlan| -> Vec<ReadingItem> {
        std::iter::once(plan.headline.clone())
            .chain(plan.parts.iter().flat_map(|part| part.items.clone()))
            .chain(plan.why.clone())
            .chain(plan.areas.iter().map(|area| area.note.clone()))
            .collect()
    };
    for (first, last, period) in [(15, 15, "DAILY"), (15, 21, "WEEKLY"), (1, 30, "MONTHLY")] {
        let answers = sdk
            .chart()
            .rashifal_answers(&november_read(first, last, period))
            .unwrap()
            .value;
        let answer = &answers[0];
        let plans = answer.readings.as_ref().expect("readings were asked for");
        assert_eq!(plans.len(), 12, "{period}");
        let scores = answer
            .period
            .readings
            .iter()
            .map(|read| read.rashi)
            .collect::<Vec<Rashi>>();
        assert_eq!(
            plans.iter().map(|plan| plan.rashi).collect::<Vec<_>>(),
            scores
        );
        for (plan, read) in plans.iter().zip(&answer.period.readings) {
            assert_eq!(plan.areas.len(), 4);
            assert_eq!(plan.syllables.len(), 9);
            for item in items(plan) {
                assert!(!item.rests_on.is_empty(), "{period} {}", item.intent);
                for fact in &item.rests_on {
                    if let Fact::Event { index } = fact {
                        assert!(*index < read.events.len(), "{period} {}", item.intent);
                    }
                    if let Fact::Day { index } = fact {
                        assert!(*index < 7, "{period} {}", item.intent);
                    }
                }
            }
            if period == "WEEKLY" {
                // A best day is the week's strict highest, read as its own day.
                if let Some(best) = plan.best_day {
                    assert!(best < 7);
                }
            } else {
                assert_eq!(plan.best_day, None);
            }
        }
        // The same batch reads alike.
        let again = sdk
            .chart()
            .rashifal_answers(&november_read(first, last, period))
            .unwrap()
            .value;
        assert_eq!(again[0].readings, answer.readings, "{period}");
    }
    // A reading needs its period.
    let refused = RashifalBatch::from_json(
        r#"{"periods": [{"first": {"year": 2026, "month": 11, "day": 15},
            "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20700}], "readings": true}"#,
    )
    .unwrap_err();
    assert_eq!(refused.field(), Some("rashifal.readings"));
    // Not asked, not said.
    let plain = sdk
        .chart()
        .rashifal_answers(&november(15, 15, "DAILY"))
        .unwrap()
        .value;
    assert!(plain[0].readings.is_none());
}

/// The voice packs say every plan (SR-152 S2 and S3): each intent the
/// composer can name is in en-Latn, ne-Deva-NP, hi-Deva-IN and sa-Deva, and
/// every item of a month of days, its weeks, the month and a year renders in
/// each with no warning, from the locale itself, final (Q42: reviewed after
/// shipping).
#[test]
fn every_reading_renders_in_all_four_languages() {
    use teistro::interpret::rashifal_reading::{ReadingItem, intents};
    const TAGS: [&str; 4] = ["en-Latn", "ne-Deva-NP", "hi-Deva-IN", "sa-Deva"];
    let sdk = context();
    let engine = sdk.locale_engine();
    let mut asked: Vec<RashifalBatch> = (1..=30)
        .map(|day| november_read(day, day, "DAILY"))
        .collect();
    asked.extend([(1, 7), (8, 14), (15, 21), (22, 28)].map(|(a, b)| november_read(a, b, "WEEKLY")));
    asked.push(november_read(1, 30, "MONTHLY"));
    asked.push(
        RashifalBatch::from_json(
            r#"{"periods": [{"first": {"year": 2026, "month": 4, "day": 14},
            "last": {"year": 2027, "month": 4, "day": 13},
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
            "utcOffsetSeconds": 20700}], "baseline": "YEARLY", "readings": true}"#,
        )
        .unwrap(),
    );
    for tag in TAGS {
        for key in intents() {
            let rendered = engine.render_in(tag, &key, &teistro::Params::new());
            assert_eq!(
                rendered.resolved_from.as_deref(),
                Some(tag),
                "{tag} lacks {key}"
            );
        }
    }
    let mut said = std::collections::BTreeSet::new();
    for batch in &asked {
        for answer in sdk.chart().rashifal_answers(batch).unwrap().value {
            for plan in answer.readings.expect("readings were asked for") {
                let items: Vec<ReadingItem> = std::iter::once(plan.headline.clone())
                    .chain(plan.parts.iter().flat_map(|part| part.items.clone()))
                    .chain(plan.why.clone())
                    .chain(plan.areas.iter().map(|area| area.note.clone()))
                    .collect();
                for item in items {
                    said.insert(item.intent.clone());
                    for tag in TAGS {
                        let rendered = engine.render_in(tag, &item.intent, &item.params);
                        assert!(
                            rendered.warnings.is_empty(),
                            "{tag} {}: {rendered:?}",
                            item.intent
                        );
                        assert_eq!(
                            rendered.resolved_from.as_deref(),
                            Some(tag),
                            "{tag} {}",
                            item.intent
                        );
                        assert!(!rendered.is_draft, "{tag} {} is final", item.intent);
                        assert!(!rendered.text.contains('{'), "{tag}: {}", rendered.text);
                    }
                }
            }
        }
    }
    // A month of real skies reaches most of the voice, not a corner of it.
    assert!(
        said.len() >= 40,
        "only {} intents said: {said:?}",
        said.len()
    );
}

/// A year is scored from its months (C2422): each Gregorian month of the
/// window asked on its own as `MONTHLY`, the twelve signs' rounded mean is
/// the year's overall, each area's mean its area, the band read from that
/// overall, and a headline that names a band names that one.
#[test]
fn a_year_is_scored_as_the_mean_of_its_months() {
    let sdk = context();
    let period = |first: (i32, u8, u8), last: (i32, u8, u8), baseline: &str| {
        RashifalBatch::from_json(&format!(
            r#"{{"periods": [{{"first": {{"year": {}, "month": {}, "day": {}}},
            "last": {{"year": {}, "month": {}, "day": {}}},
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
            "utcOffsetSeconds": 20700}}], "baseline": "{baseline}", "readings": true}}"#,
            first.0, first.1, first.2, last.0, last.1, last.2
        ))
        .unwrap()
    };
    let answer =
        |batch: &RashifalBatch| sdk.chart().rashifal_answers(batch).unwrap().value.remove(0);
    let year = answer(&period((2026, 4, 14), (2027, 4, 13), "YEARLY"));
    // April's tail, May to March whole, and April's head: thirteen months.
    let mut bounds = vec![((2026, 4, 14), (2026, 4, 30))];
    for (y, m, days) in [
        (2026, 5, 31),
        (2026, 6, 30),
        (2026, 7, 31),
        (2026, 8, 31),
        (2026, 9, 30),
        (2026, 10, 31),
        (2026, 11, 30),
        (2026, 12, 31),
        (2027, 1, 31),
        (2027, 2, 28),
        (2027, 3, 31),
    ] {
        bounds.push(((y, m, 1), (y, m, days)));
    }
    bounds.push(((2027, 4, 1), (2027, 4, 13)));
    let months: Vec<_> = bounds
        .iter()
        .map(|&(first, last)| answer(&period(first, last, "MONTHLY")).baseline.unwrap())
        .collect();
    let mean = |scores: Vec<u8>| {
        let count = u32::try_from(scores.len()).unwrap();
        let total: u32 = scores.into_iter().map(u32::from).sum();
        u8::try_from((total * 2 + count) / (count * 2)).unwrap()
    };
    let scores = year.baseline.unwrap();
    let plans = year.readings.unwrap();
    for (sign, (score, plan)) in scores.iter().zip(&plans).enumerate() {
        assert_eq!(
            score.overall,
            mean(months.iter().map(|month| month[sign].overall).collect()),
            "sign {sign}"
        );
        for (at, (area, value)) in score.areas.iter().enumerate() {
            assert_eq!(
                *value,
                mean(months.iter().map(|month| month[sign].areas[at].1).collect()),
                "sign {sign} {area:?}"
            );
        }
        assert_eq!(score.band(), ScoreBands::DEFAULT.band(score.overall));
        let headline = &plan.headline.intent;
        assert!(
            headline.ends_with("SPLIT") || headline.ends_with(score.band().key()),
            "sign {sign}: {headline} under {:?}",
            score.band()
        );
    }
}

/// Each band is named for each period a score is weighed for, in every
/// locale the consumer shows (SR-162): a year's reading never says "a day".
#[test]
fn every_band_is_named_for_every_period() {
    let periods = [
        Period::Daily,
        Period::Weekly,
        Period::Monthly,
        Period::Yearly,
    ];
    for locale in ["en-Latn", "ne-Deva-NP", "hi-Deva-IN", "sa-Deva", "sa-Latn"] {
        let sdk = Context::builder().locale(locale).build().unwrap();
        for band in ScoreBand::ALL {
            let entity = sdk
                .intl()
                .entity(&format!("score_band.{}", band.key()))
                .unwrap();
            let names: Vec<&str> = periods
                .iter()
                .map(|period| {
                    entity
                        .form(period.band_form())
                        .unwrap_or_else(|| panic!("{locale} {band:?} {period:?}"))
                })
                .collect();
            let distinct: std::collections::BTreeSet<&&str> = names.iter().collect();
            assert_eq!(distinct.len(), 4, "{locale} {band:?}: {names:?}");
        }
    }
    let sdk = Context::builder().build().unwrap();
    let care = sdk.intl().entity("score_band.CARE").unwrap();
    assert_eq!(
        care.form(Period::Yearly.band_form()),
        Some("A year for care")
    );
}

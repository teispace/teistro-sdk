//! Rectification through the façade: the kernel's sky read back through
//! the charts it stands for (`docs/03-design/rectification.md`).

#![allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "tests fail by panicking on what they asked for and index what they built"
)]

use teistro::catalogue::{Graha, Point, Rashi};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rectification::{
    Accuracy, Confidence, DatePrecision, EventKind, GulikaAt, LifeEvent, PranapadaRule, Purifier,
    Reference, Rules, Sex, Window,
};
use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};

/// A moment at Kathmandu, 14 April 1990, the corpus's first.
const MOMENT: f64 = 2_447_995.489_583_333_5;

const OFFSET: UtcOffset = UtcOffset::literal(5, 45, 0);

fn sdk() -> Context {
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

fn window(hours: f64) -> Window {
    Window::between(
        JulianDay::literal(MOMENT),
        JulianDay::literal(MOMENT + hours / 24.0),
    )
    .unwrap()
}

fn chart(sdk: &Context, at: f64) -> Document {
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(at),
            &ChartRequest::at(kathmandu(), OFFSET).with_points(),
        )
        .unwrap()
        .value
}

fn point(document: &Document, point: Point) -> Rashi {
    document
        .points
        .as_ref()
        .unwrap()
        .all()
        .iter()
        .find(|found| found.point == point)
        .unwrap()
        .sign
}

/// The Svarodaya is counted in the almanac's own day: its sunrise, and the
/// nadi the almanac's tithi at that sunrise starts (v. 62); a window's runs
/// tile it, and each is what the reading at its middle says.
#[test]
fn the_svarodaya_is_counted_in_the_almanacs_day() {
    use teistro::CalendarDate;
    use teistro::catalogue::Calendar;
    use teistro::rectification::svarodaya::Nadi;

    let sdk = sdk();
    let place = kathmandu();
    let at = JulianDay::literal(MOMENT);
    let read = sdk.chart().svarodaya(at, &place, OFFSET).unwrap().value;
    // 05:15 on the 14th is before its sunrise: the night of the 13th (X9).
    let date = CalendarDate::defined(Calendar::Gregorian, 1990, 4, 13);
    let day = sdk
        .almanac()
        .of(&date, &date, &place, OFFSET)
        .unwrap()
        .value
        .remove(0);
    assert_eq!(read.sunrise, day.day.sunrise);
    let tithi = day.tithi_at(day.day.sunrise).unwrap().member;
    assert_eq!(read.tithi, tithi);
    assert_eq!(read.sunrise_nadi, Nadi::at_sunrise(tithi));
    assert!(read.run.from.get() <= MOMENT && MOMENT < read.run.to.get());

    let window = window(3.0);
    let runs = sdk
        .chart()
        .svarodaya_runs(window, &place, OFFSET)
        .unwrap()
        .value;
    assert_eq!(runs[0].from, window.from);
    assert_eq!(runs[runs.len() - 1].to, window.to);
    for pair in runs.windows(2) {
        assert_eq!(pair[0].to, pair[1].from);
    }
    for run in &runs {
        let middle = JulianDay::literal(f64::midpoint(run.from.get(), run.to.get()));
        let there = sdk
            .chart()
            .svarodaya(middle, &place, OFFSET)
            .unwrap()
            .value
            .run;
        assert_eq!(
            (there.nadi, there.tattva, there.turn),
            (run.nadi, run.tattva, run.turn)
        );
    }
}

#[test]
fn every_clause_is_what_the_chart_of_its_instant_says() {
    // The SDK's own points, so each clause has a chart value to meet.
    let rules = Rules {
        gulika_at: GulikaAt::Start,
        pranapada_rule: PranapadaRule::SdkPoint,
        purify_as: teistro::rectification::PurifyAs::Weight,
        ..Rules::default()
    };
    let sdk = sdk();
    let answer = sdk
        .chart()
        .rectify(window(3.0), &kathmandu(), OFFSET, &rules)
        .unwrap()
        .value;
    assert!(
        answer.intervals.len() > 1,
        "a lagna changes sign in three hours"
    );
    for run in &answer.intervals {
        let middle = f64::midpoint(run.from.get(), run.to.get());
        let document = chart(&sdk, middle);
        let lagna = Rashi::of_longitude(document.foundation.lagna_deg);
        let moon = document
            .foundation
            .graha(Graha::Moon)
            .unwrap()
            .longitude_deg;
        for clause in &run.verdict.clauses {
            assert_eq!(clause.lagna, lagna, "at {middle}");
            let expected = match (clause.purifier, clause.reference) {
                (Purifier::Moon, _) => Rashi::of_longitude(moon),
                (Purifier::Gulika, Reference::Itself) => point(&document, Point::Gulika),
                (Purifier::Pranapada, _) => point(&document, Point::PranapadaLagna),
                _ => continue,
            };
            assert_eq!(clause.sign, expected, "{clause:?} at {middle}");
        }
    }
}

#[test]
fn the_runs_tile_the_window_and_a_bar_keeps_only_the_pure() {
    let window = window(2.0);
    let answer = sdk()
        .chart()
        .rectify(window, &kathmandu(), OFFSET, &Rules::default())
        .unwrap();
    let mut runs: Vec<_> = answer
        .value
        .intervals
        .iter()
        .chain(&answer.value.removed)
        .collect();
    runs.sort_by(|a, b| a.from.get().total_cmp(&b.from.get()));
    assert_eq!(runs.first().unwrap().from, window.from);
    assert_eq!(runs.last().unwrap().to, window.to);
    assert!(
        runs.iter()
            .zip(runs.iter().skip(1))
            .all(|(a, b)| a.to == b.from)
    );
    assert!(answer.value.intervals.iter().all(|run| run.verdict.pure));
    assert!(answer.value.removed.iter().all(|run| !run.verdict.pure));
    // The answer is sealed and says what it was asked.
    assert_ne!(answer.provenance.input_hash, answer.provenance.content_hash);
}

#[test]
fn a_refused_rule_names_its_field_before_any_sky_is_read() {
    let refused = Context::builder()
        .build()
        .unwrap()
        .chart()
        .rectify(
            window(1.0),
            &kathmandu(),
            OFFSET,
            &Rules {
                seed_minutes: 0.0,
                ..Rules::default()
            },
        )
        .unwrap_err();
    assert_eq!(refused.field(), Some("rectification.seedMinutes"));
}

/// Step 4's reports read back through the charts of the candidate and of
/// the conception it counts back to.
#[test]
fn the_conception_reports_are_what_the_charts_of_both_instants_say() {
    use teistro::rectification::ConceptionRules;

    let sdk = sdk();
    let at = JulianDay::literal(MOMENT);
    let read = sdk
        .chart()
        .conception(at, &kathmandu(), OFFSET, &ConceptionRules::default())
        .unwrap()
        .value;
    let birth = chart(&sdk, MOMENT);
    let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
    let points = read.nisheka.count.points;
    assert!(close(points.lagna_deg, birth.foundation.lagna_deg));
    let saturn = birth.foundation.graha(Graha::Saturn).unwrap().longitude_deg;
    assert!(close(points.saturn_deg, saturn));
    let moon = birth.foundation.graha(Graha::Moon).unwrap().longitude_deg;
    assert!(close(points.moon_deg, moon));
    // Mandi at the start of Saturn's eighth is the SDK's Gulika point.
    assert_eq!(
        Rashi::of_longitude(points.mandi_deg),
        point(&birth, Point::Gulika)
    );
    // The conception is the birth less the span, and its chart's lagna
    // and Moon are the ones judged and counted.
    let instant = read.nisheka.count.instant.get();
    assert!(close(instant, MOMENT - read.nisheka.count.span.days_before));
    let conceived = chart(&sdk, instant);
    assert!(
        close(read.nisheka.lagna_deg, conceived.foundation.lagna_deg),
        "{} {} {}",
        read.nisheka.lagna_deg,
        conceived.foundation.lagna_deg,
        read.nisheka.count.span.days_before
    );
    let conceived_moon = conceived
        .foundation
        .graha(Graha::Moon)
        .unwrap()
        .longitude_deg;
    let count = teistro::rectification::moon_count(
        conceived_moon,
        teistro::rectification::ConceptionCount::default(),
    );
    assert_eq!(read.moon.predicted, count);
    assert_eq!(read.moon.moon_sign, Rashi::of_longitude(moon));
    assert!((0.0..1.0).contains(&read.moon.risen_fraction));
    assert!((1..=12).contains(&read.pranapada_house.house));
    // A gestation of some months, as the arcs can only give 0 to 25.
    assert!((0.0..750.0).contains(&read.nisheka.count.span.days_before));
    assert!(read.nisheka.count.days_per_birth_minute.is_finite());
}

/// Step 5's report read back through the chart of the candidate.
#[test]
fn the_circumstances_are_what_the_candidates_chart_says() {
    use teistro::rectification::{CircumstanceRules, Facts, Level, PresentationBy};

    let sdk = sdk();
    let at = JulianDay::literal(MOMENT);
    let facts = Facts {
        father_present: Some(true),
        oil: Some(Level::Half),
        ..Facts::default()
    };
    let rules = CircumstanceRules {
        presentation_by: PresentationBy::LagnaLordMotion,
        ..CircumstanceRules::default()
    };
    let read = sdk
        .chart()
        .circumstance(at, &kathmandu(), OFFSET, &facts, &rules)
        .unwrap()
        .value;
    let birth = chart(&sdk, MOMENT);
    let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
    assert!(close(read.sky.lagna_deg, birth.foundation.lagna_deg));
    for graha in teistro::rectification::circumstance::SEVEN {
        let founded = birth.foundation.graha(graha).unwrap();
        assert!(
            close(read.sky.of(graha), founded.longitude_deg),
            "{graha:?}"
        );
    }
    // The lord's motion differenced over two hours, against the chart's
    // own speed.
    let lord = birth.foundation.graha(read.presentation.lord).unwrap();
    assert_eq!(read.sky.lord_retrograde, lord.speed_deg_per_day < 0.0);
    let moon = birth.foundation.graha(Graha::Moon).unwrap().longitude_deg;
    assert!(close(read.lamp.oil, 1.0 - moon.rem_euclid(30.0) / 30.0));
    // Two facts given, two weighed, in the order the clauses run.
    assert_eq!(read.weights.len(), 2);
}

// ── step 6: the baseline engine's rectification, reproduced ────────────

mod baseline {
    use teistro::catalogue::Graha;
    use teistro::dasha::{Birth, Dasha, Timeline, VIMSHOTTARI};
    use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    use teistro::rectification::baseline::{
        BaselineAnswer, BaselineRequest, Candidate, NINE, baseline_dasha_rules, signifier_score,
    };
    use teistro::rectification::{Accuracy, BaselineStage, EventKind, LifeEvent, Note, Sex};
    use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};

    /// The baseline's own fixture: Pokhara, the true birth at this instant.
    const TRUE_JD: f64 = 2_451_779.135_417;
    const OFFSET: UtcOffset = UtcOffset::literal(5, 45, 0);
    const MINUTE: f64 = 1.0 / 1440.0;
    const YEAR: f64 = 365.25;

    fn sdk() -> Context {
        Context::builder()
            .ephemeris([Ephemeris::Builtin])
            .build()
            .unwrap()
    }

    fn pokhara() -> Place {
        Place::new(
            Latitude::literal(28.2096),
            Longitude::literal(83.9856),
            Altitude::literal(0.0),
        )
    }

    fn run(sdk: &Context, request: &BaselineRequest) -> BaselineAnswer {
        sdk.chart()
            .rectify_baseline(&pokhara(), OFFSET, request)
            .unwrap()
            .value
    }

    /// The record a binding sends answers what the façade's own calls
    /// answer around the chart's instant, every reading to the bit.
    #[test]
    fn the_record_reads_what_the_facade_reads_around_the_chart() {
        use teistro::rectification::{CircumstanceRules, ConceptionRules, Facts, Rules, Window};
        use teistro::{
            BaselineAsked, CircumstanceAsked, DashaAsked, Purify, RectificationRequest,
            SvarodayaAsked,
        };

        let sdk = sdk();
        let at = JulianDay::literal(TRUE_JD);
        let document = sdk
            .chart()
            .reading(at, &ChartRequest::at(pokhara(), OFFSET))
            .unwrap()
            .value;
        let facts = Facts {
            father_present: Some(false),
            ..Facts::default()
        };
        let asked = RectificationRequest {
            purify: Some(Purify {
                minutes: 30.0,
                rules: Rules::default(),
            }),
            conception: Some(ConceptionRules::default()),
            circumstance: Some(CircumstanceAsked {
                facts,
                rules: CircumstanceRules::default(),
            }),
            baseline: Some(BaselineAsked {
                uncertainty_minutes: 60.0,
                accuracy: Accuracy::Approximate,
                events: events(&sdk),
                sex: Some(Sex::Male),
                coverage: 0.8,
                dasha: DashaAsked::default(),
            }),
            svarodaya: Some(SvarodayaAsked { minutes: 30.0 }),
        };
        // A dasha left out is the baseline engine's own.
        assert_eq!(DashaAsked::default().rules(), baseline_dasha_rules());
        let read = sdk
            .chart()
            .rectification(&document, OFFSET, &asked)
            .unwrap();
        // Every key a binding reads is camelCase, a tagged note's own
        // fields included.
        let snake = snake_keys(&serde_json::to_value(&read).unwrap());
        assert!(snake.is_empty(), "snake_case keys on the wire: {snake:?}");
        let half = 30.0 * MINUTE;
        let window = Window::between(
            JulianDay::literal(TRUE_JD - half),
            JulianDay::literal(TRUE_JD + half),
        )
        .unwrap();
        let place = pokhara();
        let chart = sdk.chart();
        assert_eq!(
            read.purified.unwrap(),
            chart
                .rectify(window, &place, OFFSET, &Rules::default())
                .unwrap()
                .value
        );
        assert_eq!(
            read.conception.unwrap(),
            chart
                .conception(at, &place, OFFSET, &ConceptionRules::default())
                .unwrap()
                .value
        );
        assert_eq!(
            read.circumstance.unwrap(),
            chart
                .circumstance(at, &place, OFFSET, &facts, &CircumstanceRules::default())
                .unwrap()
                .value
        );
        let svarodaya = read.svarodaya.unwrap();
        assert_eq!(
            svarodaya.at,
            chart.svarodaya(at, &place, OFFSET).unwrap().value
        );
        assert_eq!(
            svarodaya.runs,
            chart.svarodaya_runs(window, &place, OFFSET).unwrap().value
        );
        let baseline = asked.baseline.as_ref().unwrap().at(at);
        assert_eq!(read.baseline.unwrap(), run(&sdk, &baseline));
        // A reading not asked for is not read.
        let none = sdk
            .chart()
            .rectification(&document, OFFSET, &RectificationRequest::default())
            .unwrap();
        assert!(none.purified.is_none() && none.baseline.is_none());
    }

    /// The keys anywhere in an answer that are not camelCase.
    fn snake_keys(value: &serde_json::Value) -> Vec<String> {
        match value {
            serde_json::Value::Object(map) => map
                .iter()
                .flat_map(|(key, inner)| {
                    let own = key.contains('_').then(|| key.clone());
                    own.into_iter().chain(snake_keys(inner))
                })
                .collect(),
            serde_json::Value::Array(items) => items.iter().flat_map(snake_keys).collect(),
            _ => Vec::new(),
        }
    }

    /// Every refusal names the field the binding sent, under the member
    /// that asked, never the kernel's own name for it.
    #[test]
    fn a_refused_record_names_the_field_it_sent() {
        use teistro::RectificationRequest;

        let sdk = sdk();
        let document = sdk
            .chart()
            .reading(
                JulianDay::literal(TRUE_JD),
                &ChartRequest::at(pokhara(), OFFSET),
            )
            .unwrap()
            .value;
        let refused = |json: &str| {
            let asked = RectificationRequest::from_json(json).unwrap();
            let error = sdk
                .chart()
                .rectification(&document, OFFSET, &asked)
                .unwrap_err();
            error.field().unwrap().to_owned()
        };
        assert_eq!(
            refused(r#"{"purify": {"minutes": 0}}"#),
            "rectification.purify.minutes"
        );
        assert_eq!(
            refused(r#"{"purify": {"minutes": 1200}}"#),
            "rectification.purify.minutes"
        );
        assert_eq!(
            refused(r#"{"purify": {"minutes": 30, "rules": {"seedMinutes": 0}}}"#),
            "rectification.purify.rules.seedMinutes"
        );
        assert_eq!(
            refused(r#"{"baseline": {"uncertaintyMinutes": 0}}"#),
            "rectification.baseline.uncertaintyMinutes"
        );
        assert_eq!(
            refused(r#"{"svarodaya": {"minutes": 1200}}"#),
            "rectification.svarodaya.minutes"
        );
        let typo =
            RectificationRequest::from_json(r#"{"baseline": {"uncertainty": 60}}"#).unwrap_err();
        assert_eq!(typo.field(), Some("rectification.baseline.uncertainty"));
    }

    fn stage(
        answer: &BaselineAnswer,
        which: BaselineStage,
    ) -> &teistro::rectification::StageOutcome {
        answer.stages.iter().find(|s| s.stage == which).unwrap()
    }

    fn width(answer: &BaselineAnswer) -> f64 {
        answer.interval_width_minutes
    }

    /// The baseline spec's events: for each kind, the antardasha wholly
    /// between 18 and 45 years that the true chart's lords fit best, the
    /// event at its middle.
    fn events(sdk: &Context) -> Vec<LifeEvent> {
        let document = sdk
            .chart()
            .reading(
                JulianDay::literal(TRUE_JD),
                &ChartRequest::at(pokhara(), OFFSET),
            )
            .unwrap()
            .value;
        let foundation = &document.foundation;
        let mut grahas_deg = [0.0; 9];
        for (deg, graha) in grahas_deg.iter_mut().zip(NINE) {
            *deg = foundation.graha(graha).unwrap().longitude_deg;
        }
        let at_birth = Candidate {
            at: TRUE_JD,
            lagna_deg: foundation.lagna_deg,
            grahas_deg,
            weekday: 0,
        };
        let moon = foundation.graha(Graha::Moon).unwrap().longitude_deg;
        let dasha = Dasha::new(
            &VIMSHOTTARI,
            &Birth {
                instant: JulianDay::literal(TRUE_JD),
                moon: teistro::Nas::from_degrees(
                    teistro::quantity::Degrees::try_new(moon).unwrap(),
                ),
                moon_span: None,
            },
            baseline_dasha_rules(),
        )
        .unwrap();
        let (from, to) = (TRUE_JD + 18.0 * YEAR, TRUE_JD + 45.0 * YEAR);
        let mut events = Vec::new();
        for kind in [
            EventKind::Marriage,
            EventKind::ChildBirth,
            EventKind::FirstJob,
            EventKind::Property,
            EventKind::JobChange,
        ] {
            let mut best: Option<(f64, f64)> = None;
            for maha in dasha.mahadashas() {
                for antar in dasha.children(&maha) {
                    let (a, b) = (antar.interval.from.get(), antar.interval.to.get());
                    if a < from || b > to {
                        continue;
                    }
                    let score = signifier_score(antar.lord, kind, &at_birth);
                    if best.is_none_or(|(s, _)| score > s) {
                        best = Some((score, f64::midpoint(a, b)));
                    }
                }
            }
            if let Some((score, middle)) = best.filter(|(s, _)| *s > 0.0) {
                assert!(score > 0.0);
                events.push(LifeEvent::on(kind, JulianDay::literal(middle)));
            }
        }
        assert!(events.len() >= 3, "{} events", events.len());
        events
    }

    #[test]
    fn with_no_evidence_the_reported_time_is_the_mode() {
        let sdk = sdk();
        let answer = run(
            &sdk,
            &BaselineRequest::around(JulianDay::literal(TRUE_JD), 120.0),
        );
        let offset = (answer.suggested.get() - TRUE_JD) / MINUTE;
        assert!(offset.abs() < 0.5, "{offset}");
        assert!(width(&answer) > 30.0);
        assert!(!stage(&answer, BaselineStage::DashaBoundary).applied);
        let unknown = run(
            &sdk,
            &BaselineRequest {
                accuracy: Accuracy::Unknown,
                ..BaselineRequest::around(JulianDay::literal(TRUE_JD), 120.0)
            },
        );
        assert!(unknown.concentration < 0.2, "{}", unknown.concentration);
    }

    #[test]
    fn events_narrow_the_window_onto_the_true_time() {
        let sdk = sdk();
        let request = BaselineRequest {
            events: events(&sdk),
            ..BaselineRequest::around(JulianDay::literal(TRUE_JD), 60.0)
        };
        let answer = run(&sdk, &request);
        assert!(
            answer
                .intervals
                .iter()
                .any(|i| i.from.get() <= TRUE_JD && TRUE_JD <= i.to.get()),
            "{:?}",
            answer.intervals
        );
        assert!(width(&answer) < 120.0);
        let fit = stage(&answer, BaselineStage::DashaBoundary);
        assert!(fit.applied && !fit.flat && !fit.notes.is_empty());
        assert!(width(&answer) >= answer.resolution_minutes);
        // No evidence is wider and less concentrated.
        let bare = run(
            &sdk,
            &BaselineRequest::around(JulianDay::literal(TRUE_JD), 60.0),
        );
        assert!(width(&bare) > width(&answer));
        assert!(bare.concentration < answer.concentration);
        // The candidates come most probable first.
        assert!(
            answer
                .candidates
                .windows(2)
                .all(|pair| pair[0].probability >= pair[1].probability)
        );
        // Dated to the year, the same events concentrate less.
        let yearly = BaselineRequest {
            events: request
                .events
                .iter()
                .cloned()
                .map(|e| LifeEvent {
                    precision: teistro::rectification::DatePrecision::Year,
                    ..e
                })
                .collect(),
            ..request.clone()
        };
        assert!(run(&sdk, &yearly).concentration < answer.concentration);
    }

    #[test]
    fn a_held_out_event_is_tested_and_not_fitted() {
        let sdk = sdk();
        let mut events = events(&sdk);
        events[0].held_out = true;
        let n = events.len();
        let answer = run(
            &sdk,
            &BaselineRequest {
                events,
                ..BaselineRequest::around(JulianDay::literal(TRUE_JD), 60.0)
            },
        );
        assert_eq!(answer.hold_out.len(), 1);
        let held = &answer.hold_out[0];
        assert_eq!(held.event, 0);
        assert!((0.0..=1.0).contains(&held.score_at_fit));
        assert!((0.0..=1.0).contains(&held.baseline));
        assert_eq!(held.supported, held.score_at_fit > held.baseline + 0.05);
        assert_eq!((answer.events_held_out, answer.events_used), (1, n - 1));
    }

    #[test]
    fn a_sex_given_adds_the_tattva_prior_and_none_does_not() {
        let sdk = sdk();
        let around = BaselineRequest::around(JulianDay::literal(TRUE_JD), 45.0);
        let with = run(
            &sdk,
            &BaselineRequest {
                sex: Some(Sex::Female),
                ..around.clone()
            },
        );
        let tattva = |answer: &BaselineAnswer| {
            stage(answer, BaselineStage::Prior)
                .notes
                .iter()
                .any(|n| matches!(n, Note::TattvaSex { .. }))
        };
        assert!(tattva(&with));
        let without = run(&sdk, &around);
        assert!(!tattva(&without));
        let window = without.window;
        assert!(((window.to.get() - window.from.get()) / MINUTE - 90.0).abs() < 1e-6);
        // The sunrise counted from opens the reported time's civil day.
        assert!(without.sunrise.get() < TRUE_JD && TRUE_JD - without.sunrise.get() < 1.0);
    }
}

// The record below keeps the baseline engine's digits as it printed them.
#[allow(
    clippy::unreadable_literal,
    reason = "the baseline engine's own digits"
)]
mod record {
    use super::*;

    /// An event as the black-box run gave it: kind, `on`, `until`, precision,
    /// confidence, held out.
    type Event = (EventKind, f64, Option<f64>, DatePrecision, Confidence, bool);

    const FIVE: &[Event] = &[
        (
            EventKind::Marriage,
            2459000.5,
            None,
            DatePrecision::Day,
            Confidence::Certain,
            false,
        ),
        (
            EventKind::ChildBirth,
            2459600.5,
            None,
            DatePrecision::Day,
            Confidence::Certain,
            false,
        ),
        (
            EventKind::FirstJob,
            2457000.5,
            None,
            DatePrecision::Day,
            Confidence::Certain,
            false,
        ),
        (
            EventKind::Property,
            2460000.5,
            None,
            DatePrecision::Day,
            Confidence::Certain,
            false,
        ),
        (
            EventKind::JobChange,
            2458500.5,
            None,
            DatePrecision::Day,
            Confidence::Certain,
            false,
        ),
    ];

    const LOOSE: &[Event] = &[
        (
            EventKind::Education,
            2452000.5,
            None,
            DatePrecision::Year,
            Confidence::Probable,
            false,
        ),
        (
            EventKind::Marriage,
            2455000.5,
            None,
            DatePrecision::Month,
            Confidence::Certain,
            false,
        ),
        (
            EventKind::Relocation,
            2456000.5,
            Some(2456090.5),
            DatePrecision::Day,
            Confidence::Uncertain,
            false,
        ),
        (
            EventKind::Accident,
            2457500.5,
            None,
            DatePrecision::Day,
            Confidence::Certain,
            true,
        ),
    ];

    /// A case the baseline engine was run over, and what it answered.
    struct Case {
        id: &'static str,
        place: (f64, f64),
        offset_minutes: i32,
        reported: f64,
        uncertainty_minutes: f64,
        accuracy: Accuracy,
        sex: Option<Sex>,
        events: &'static [Event],
        coverage: f64,
        sunrise: f64,
        intervals: &'static [(f64, f64)],
        suggested: f64,
        concentration: f64,
        /// The five likeliest candidates, instant and probability.
        top: [(f64, f64); 5],
        /// Each held-out event's score at the fit, its mean over the spread,
        /// and whether the fit supports it.
        hold_out: &'static [(f64, f64, bool)],
    }

    /// The baseline engine's rectification over these cases, recorded
    /// black-box on 2026-10-09: geocentric, true node, Lahiri, its own sunrise.
    const CASES: &[Case] = &[
        Case {
            id: "pokhara-60",
            place: (28.2096, 83.9856),
            offset_minutes: 345,
            reported: 2451779.135417,
            uncertainty_minutes: 60.0,
            accuracy: Accuracy::Approximate,
            sex: None,
            events: &[],
            coverage: 0.8,
            sunrise: 2451778.498226405,
            intervals: &[(2451779.1121531134, 2451779.158680891)],
            suggested: 2451779.1354170023,
            concentration: 0.15550111982769366,
            top: [
                (2451779.1354170023, 0.00749705339585412),
                (2451779.13506978, 0.007495803991223697),
                (2451779.1357642245, 0.007495803991223697),
                (2451779.134722558, 0.007492057026494003),
                (2451779.1361114467, 0.007492057026494003),
            ],
            hold_out: &[],
        },
        Case {
            id: "pokhara-120-exact",
            place: (28.2096, 83.9856),
            offset_minutes: 345,
            reported: 2451779.135417,
            uncertainty_minutes: 120.0,
            accuracy: Accuracy::Exact,
            sex: None,
            events: &[],
            coverage: 0.8,
            sunrise: 2451778.498226405,
            intervals: &[(2451779.1048614467, 2451779.1666670023)],
            suggested: 2451779.1354170023,
            concentration: 0.40585040020326124,
            top: [
                (2451779.1354170023, 0.011522607221010927),
                (2451779.134722558, 0.01151780713532835),
                (2451779.1361114467, 0.01151780713532835),
                (2451779.136805891, 0.011503418872662041),
                (2451779.134028113, 0.011503418859805969),
            ],
            hold_out: &[],
        },
        Case {
            id: "pokhara-60-unknown-female",
            place: (28.2096, 83.9856),
            offset_minutes: 345,
            reported: 2451779.135417,
            uncertainty_minutes: 60.0,
            accuracy: Accuracy::Unknown,
            sex: Some(Sex::Female),
            events: &[],
            coverage: 0.8,
            sunrise: 2451778.498226405,
            intervals: &[
                (2451779.094097558, 2451779.1107642245),
                (2451779.1357642245, 2451779.1493058912),
            ],
            suggested: 2451779.094097558,
            concentration: 0.4090953950954336,
            top: [
                (2451779.094097558, 0.009274696493328722),
                (2451779.09444478, 0.009274696493328722),
                (2451779.0947920023, 0.009274696493328722),
                (2451779.0951392245, 0.009274696493328722),
                (2451779.0954864467, 0.009274696493328722),
            ],
            hold_out: &[],
        },
        Case {
            id: "pokhara-60-five",
            place: (28.2096, 83.9856),
            offset_minutes: 345,
            reported: 2451779.135417,
            uncertainty_minutes: 60.0,
            accuracy: Accuracy::Approximate,
            sex: None,
            events: FIVE,
            coverage: 0.8,
            sunrise: 2451778.498226405,
            intervals: &[
                (2451779.1104170023, 2451779.1166670024),
                (2451779.1201392245, 2451779.1361114467),
            ],
            suggested: 2451779.1149308914,
            concentration: 0.6004527160047173,
            top: [
                (2451779.1149308914, 0.027488797277321564),
                (2451779.114583669, 0.026948973783003975),
                (2451779.114236447, 0.026410946197760396),
                (2451779.113889225, 0.025875033712165613),
                (2451779.1135420026, 0.025341546999319432),
            ],
            hold_out: &[],
        },
        Case {
            id: "pokhara-240-five-male",
            place: (28.2096, 83.9856),
            offset_minutes: 345,
            reported: 2451779.135417,
            uncertainty_minutes: 240.0,
            accuracy: Accuracy::Rectified,
            sex: Some(Sex::Male),
            events: FIVE,
            coverage: 0.8,
            sunrise: 2451778.498226405,
            intervals: &[
                (2451779.0243058912, 2451779.027083669),
                (2451779.0694447802, 2451779.0854170024),
                (2451779.1111114467, 2451779.1166670024),
            ],
            suggested: 2451779.084722558,
            concentration: 0.7671989644251387,
            top: [
                (2451779.084722558, 0.0797399172415232),
                (2451779.0840281136, 0.07933394838783003),
                (2451779.083333669, 0.07892456532517407),
                (2451779.082639225, 0.07851184238180985),
                (2451779.0819447804, 0.07809585419384964),
            ],
            hold_out: &[],
        },
        Case {
            id: "pokhara-600-five",
            place: (28.2096, 83.9856),
            offset_minutes: 345,
            reported: 2451779.135417,
            uncertainty_minutes: 600.0,
            accuracy: Accuracy::Unknown,
            sex: None,
            events: FIVE,
            coverage: 0.8,
            sunrise: 2451778.498226405,
            intervals: &[
                (2451778.7927086693, 2451778.800347558),
                (2451778.8010420026, 2451778.8121531135),
                (2451778.834375336, 2451778.839236447),
            ],
            suggested: 2451778.7927086693,
            concentration: 0.6979691152903194,
            top: [
                (2451778.7927086693, 0.08291844532798923),
                (2451778.7934031137, 0.08291844532798923),
                (2451778.794097558, 0.08291844532798923),
                (2451778.7947920025, 0.08291844532798923),
                (2451778.795486447, 0.08291844532798923),
            ],
            hold_out: &[],
        },
        Case {
            id: "kathmandu-dawn-90-loose-female",
            place: (27.7172, 85.324),
            offset_minutes: 345,
            reported: 2447995.4895833335,
            uncertainty_minutes: 90.0,
            accuracy: Accuracy::Approximate,
            sex: Some(Sex::Female),
            events: LOOSE,
            coverage: 0.8,
            sunrise: 2447995.497036061,
            intervals: &[(2447995.48125, 2447995.497916667)],
            suggested: 2447995.4895833335,
            concentration: 0.8062464656006367,
            top: [
                (2447995.4895833335, 0.02595337244223534),
                (2447995.4890625, 0.02594904724112068),
                (2447995.490104167, 0.02594904724112068),
                (2447995.488541667, 0.025936075962136405),
                (2447995.490625, 0.025936075962136405),
            ],
            hold_out: &[(0.3725806451612903, 0.1977150537634409, true)],
        },
        Case {
            id: "new-york-30-loose",
            place: (40.7128, -74.006),
            offset_minutes: -300,
            reported: 2451560.85,
            uncertainty_minutes: 30.0,
            accuracy: Accuracy::Exact,
            sex: None,
            events: LOOSE,
            coverage: 0.9,
            sunrise: 2451561.012070737,
            intervals: &[(2451560.839409722, 2451560.8598958333)],
            suggested: 2451560.85,
            concentration: 0.38544276415530776,
            top: [
                (2451560.85, 0.011179137875616222),
                (2451560.849826389, 0.011174480872332937),
                (2451560.850173611, 0.011174480872332937),
                (2451560.849652778, 0.011160521499332351),
                (2451560.8503472223, 0.011160521499332351),
            ],
            hold_out: &[(0.5806451612903226, 0.6096774193548387, false)],
        },
    ];

    /// The SDK's reproduction answers what the baseline engine answered, case
    /// by case: the same intervals and mode to a second, the same posterior to
    /// a millionth, the same hold-out verdicts, and its own sunrise within
    /// three seconds of the engine's.
    #[test]
    fn the_baseline_engines_rectification_is_reproduced() {
        use teistro::rectification::baseline::BaselineRequest;

        const SECOND: f64 = 1.0 / 86_400.0;
        let sdk = Context::builder()
            .ephemeris([Ephemeris::Builtin])
            .profile("conformance-baseline")
            .settings_json(r#"{"frame": {"centre": "GEOCENTRIC", "node": "TRUE"}}"#)
            .build()
            .unwrap();
        let near = |a: JulianDay<Utc>, b: f64, within: f64| (a.get() - b).abs() <= within;
        for case in CASES {
            let id = case.id;
            let events = case
                .events
                .iter()
                .enumerate()
                .map(
                    |(i, &(kind, on, until, precision, confidence, held_out))| LifeEvent {
                        id: Some(format!("e{i}")),
                        kind,
                        on: JulianDay::literal(on),
                        until: until.map(JulianDay::literal),
                        precision,
                        confidence,
                        held_out,
                    },
                )
                .collect();
            let request = BaselineRequest {
                accuracy: case.accuracy,
                events,
                sex: case.sex,
                coverage: case.coverage,
                ..BaselineRequest::around(
                    JulianDay::literal(case.reported),
                    case.uncertainty_minutes,
                )
            };
            let place = Place::new(
                Latitude::literal(case.place.0),
                Longitude::literal(case.place.1),
                Altitude::literal(0.0),
            );
            let offset = UtcOffset::try_from_seconds(case.offset_minutes * 60).unwrap();
            let ours = sdk
                .chart()
                .rectify_baseline(&place, offset, &request)
                .unwrap()
                .value;
            assert!(
                near(ours.sunrise, case.sunrise, 3.0 * SECOND),
                "{id}: sunrise"
            );
            assert_eq!(
                ours.intervals.len(),
                case.intervals.len(),
                "{id}: intervals"
            );
            for (mine, &(from, to)) in ours.intervals.iter().zip(case.intervals) {
                assert!(
                    near(mine.from, from, SECOND) && near(mine.to, to, SECOND),
                    "{id}: {mine:?}"
                );
            }
            assert!(near(ours.suggested, case.suggested, SECOND), "{id}: mode");
            assert!(
                (ours.concentration - case.concentration).abs() < 1e-6,
                "{id}: concentration"
            );
            for (mine, &(at, probability)) in ours.candidates.iter().zip(&case.top) {
                assert!(near(mine.at, at, SECOND), "{id}: {mine:?}");
                assert!(
                    (mine.probability - probability).abs() < 1e-6,
                    "{id}: {mine:?}"
                );
            }
            assert_eq!(ours.hold_out.len(), case.hold_out.len(), "{id}: hold-out");
            for (mine, &(fit, spread, supported)) in ours.hold_out.iter().zip(case.hold_out) {
                assert!((mine.score_at_fit - fit).abs() < 1e-9, "{id}: {mine:?}");
                assert!((mine.baseline - spread).abs() < 1e-9, "{id}: {mine:?}");
                assert_eq!(mine.supported, supported, "{id}: {mine:?}");
            }
        }
    }
}

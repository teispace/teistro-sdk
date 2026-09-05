//! The heliacal phenomena over Teimeris's positions under the SDK's three
//! criteria, against the day the engine's own photometric visibility model
//! names. The criteria are different definitions, so the days differ by
//! their nature; what is held is that every criterion dates the event
//! within a week of the engine's, and what is printed is the measured gap
//! for the design page. Run by hand with the engine present.
#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    reason = "a by-hand measurement prints what it finds and fails by panicking"
)]

use teimeris::{Body as EngineBody, HeliacalKind as EngineHeliacalKind, HeliacalOptions, Observer};
use teistro_astro::sky::local_mean_midnight;
use teistro_astro::visibility::{Criterion, Heliacal, HeliacalKind};
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::{Body, Horizon};

const KATHMANDU: (f64, f64, f64) = (27.7172, 85.324, 1400.0);

/// The events: Venus's heliacal rising after its inferior conjunction of
/// 2020-06-03, its heliacal setting before it (last seen in the evening,
/// May 2020), and Jupiter's heliacal rising after its conjunction of
/// 2021-01-29, each searched from a day before the conjunction's
/// neighbourhood.
const CASES: [(
    Body,
    EngineBody,
    HeliacalKind,
    EngineHeliacalKind,
    f64,
    &str,
); 3] = [
    (
        Body::Venus,
        EngineBody::VENUS,
        HeliacalKind::MorningFirst,
        EngineHeliacalKind::MORNING_FIRST,
        2_459_003.5,
        "Venus's heliacal rising, June 2020",
    ),
    (
        Body::Venus,
        EngineBody::VENUS,
        HeliacalKind::EveningLast,
        EngineHeliacalKind::EVENING_LAST,
        2_458_960.5,
        "Venus's heliacal setting, May 2020",
    ),
    (
        Body::Jupiter,
        EngineBody::JUPITER,
        HeliacalKind::MorningFirst,
        EngineHeliacalKind::MORNING_FIRST,
        2_459_244.5,
        "Jupiter's heliacal rising, February 2021",
    ),
];

/// The criteria differ by definition; ten days covers the spread between
/// the classical thresholds and a photometric model for these bright bodies
/// (measured from three days before to seven after).
const BOUND_DAYS: f64 = 10.0;

fn day_of(jd: f64) -> f64 {
    local_mean_midnight(
        JulianDay::<Ut1>::literal(jd),
        Longitude::literal(KATHMANDU.1),
    )
    .get()
}

#[test]
fn the_criteria_date_the_heliacal_events_within_a_week_of_the_engines_model() {
    let provider = TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"));
    let completion = Completion::new(
        &provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let place = Place::new(
        Latitude::literal(KATHMANDU.0),
        Longitude::literal(KATHMANDU.1),
        Altitude::literal(KATHMANDU.2),
    );
    let mut worst = 0.0f64;
    for (body, engine_body, kind, engine_kind, from, name) in CASES {
        let theirs = provider
            .with_context(|ctx| {
                ctx.heliacal_events(
                    from,
                    engine_body,
                    Observer::new(KATHMANDU.1, KATHMANDU.0, KATHMANDU.2),
                    &HeliacalOptions {
                        kind: engine_kind,
                        jd_end: from + 60.0,
                        ..HeliacalOptions::default()
                    },
                )
                .next()
                .expect("an event inside sixty days")
            })
            .unwrap();
        assert!(theirs.found, "{name}: the engine found no event");
        let their_day = day_of(theirs.optimum);
        println!(
            "{name}: the engine's model sees it at JD {:.3} (day {their_day})",
            theirs.optimum
        );
        for criterion in [
            Criterion::SURYA_SIDDHANTA,
            Criterion::COMBUSTION_ORB,
            Criterion::PTOLEMY,
        ] {
            let heliacal = Heliacal::new(
                &completion,
                place,
                criterion,
                Horizon::CENTRE_NO_REFRACTION,
                DeltaTModel::TableThenModel,
            );
            let events = heliacal
                .events(
                    body,
                    JulianDay::literal(from),
                    JulianDay::literal(from + 60.0),
                )
                .unwrap();
            let ours = events
                .iter()
                .find(|event| event.kind == kind)
                .unwrap_or_else(|| panic!("{name} under {criterion}: {events:?}"));
            let apart = ours.day.day_start.get() - their_day;
            println!(
                "  {criterion}: day {} ({:+.0} days from the engine; measure {:.2}° against {:.2}°, {:?}, {} evaluations)",
                ours.day.day_start.get(),
                apart,
                ours.day.measure_deg,
                ours.day.threshold_deg,
                ours.day.motion,
                ours.day.evaluations
            );
            assert!(
                apart.abs() <= BOUND_DAYS,
                "{name} under {criterion}: {apart} days from the engine's day"
            );
            assert!(
                (ours.day.instant.get() - ours.day.day_start.get()).abs() < 1.5,
                "{name} under {criterion}: the instant is not the day's"
            );
            worst = worst.max(apart.abs());
        }
    }
    println!("worst gap between a criterion's day and the engine's: {worst} days");
}

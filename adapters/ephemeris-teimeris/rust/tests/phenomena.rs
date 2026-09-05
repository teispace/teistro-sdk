//! The SDK's phenomena over Teimeris's positions, through the completion and
//! the engine's heliocentric answer, against the engine's own `phenomena`;
//! and the equation of time against the engine's. Run by hand with the
//! engine present; the measured agreement is published in
//! `docs/03-design/astro-planetary-phenomena.md`.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    reason = "a by-hand measurement prints what it finds and fails by panicking"
)]

use teimeris::{Body as EngineBody, Flags};
use teistro_astro::phenomena::{HeliocentricLeg, phenomena};
use teistro_astro::sky::equation_of_time_seconds;
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::quantity::{JulianDay, Tt, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_port_ephemeris::Body;

const INSTANTS: [f64; 6] = [
    2_415_020.0,
    2_451_545.0,
    2_451_551.0,
    2_459_003.5,
    2_460_000.5,
    2_488_070.0,
];

const BODIES: [(Body, EngineBody); 10] = [
    (Body::Sun, EngineBody::SUN),
    (Body::Moon, EngineBody::MOON),
    (Body::Mercury, EngineBody::MERCURY),
    (Body::Venus, EngineBody::VENUS),
    (Body::Mars, EngineBody::MARS),
    (Body::Jupiter, EngineBody::JUPITER),
    (Body::Saturn, EngineBody::SATURN),
    (Body::Uranus, EngineBody::URANUS),
    (Body::Neptune, EngineBody::NEPTUNE),
    (Body::Pluto, EngineBody::PLUTO),
];

#[test]
fn the_phenomena_over_the_engines_positions_reproduce_its_own() {
    let provider = TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"));
    let completion = Completion::new(
        &provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let mut worst: [(f64, String); 5] = Default::default();
    let names = [
        "phase angle (°)",
        "elongation (°)",
        "diameter (″)",
        "magnitude",
        "parallax (″)",
    ];
    for jd in INSTANTS {
        let tt = JulianDay::<Tt>::literal(jd);
        for (body, engine) in BODIES {
            let ours = phenomena(&completion, body, tt).unwrap();
            let theirs = provider
                .with_context(|ctx| ctx.phenomena(jd, engine, Flags::EPH_SWISS))
                .unwrap();
            if body != Body::Sun {
                assert_eq!(ours.heliocentric_leg, HeliocentricLeg::Provider, "{body:?}");
            }
            let phase_angle = ours.phase.map_or(0.0, |p| p.angle_deg);
            let magnitude = match (ours.magnitude, theirs.magnitude) {
                (Some(a), Some(b)) => (a - b).abs(),
                (None, None) => 0.0,
                (a, b) => panic!("{body:?} at JD {jd}: magnitude {a:?} against {b:?}"),
            };
            let apart = [
                (phase_angle - theirs.phase_angle).abs(),
                (ours.elongation_deg - theirs.elongation).abs(),
                (ours.apparent_diameter_deg() - theirs.diameter).abs() * 3600.0,
                magnitude,
                // The engine reports the parallax for the Moon alone.
                if body == Body::Moon {
                    (ours.disc.parallax_deg - theirs.horizontal_parallax).abs() * 3600.0
                } else {
                    0.0
                },
            ];
            for (slot, value) in worst.iter_mut().zip(apart) {
                if value > slot.0 {
                    *slot = (value, format!("{body:?} at JD {jd}"));
                }
            }
        }
    }
    for (name, (value, where_)) in names.iter().zip(&worst) {
        println!("{name:<16} worst {value:.7} at {where_}");
    }
    // The Sun's disc differs by the radius each side uses (the IAU 2015
    // nominal against the older 696 000 km): 0.84″ and 0.001 magnitude.
    assert!(worst[0].0 < 1e-6, "phase angle {worst:?}");
    assert!(worst[1].0 < 1e-6, "elongation {worst:?}");
    assert!(worst[2].0 < 1.0, "diameter {worst:?}");
    assert!(worst[3].0 < 2e-3, "magnitude {worst:?}");
    // The engine's parallax comes from a distance up to 40 km from its disc's.
    assert!(worst[4].0 < 0.5, "parallax {worst:?}");
}

#[test]
fn the_equation_of_time_reproduces_the_engines() {
    let provider = TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"));
    let completion = Completion::new(
        &provider,
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let mut worst = 0.0f64;
    for day in 0..366 {
        let jd = 2_451_544.5 + f64::from(day);
        let ours = equation_of_time_seconds(
            &completion,
            JulianDay::<Ut1>::literal(jd),
            DeltaTModel::TableThenModel,
        )
        .unwrap();
        let theirs = provider
            .with_context(|ctx| ctx.equation_of_time(jd))
            .unwrap();
        worst = worst.max((ours - theirs).abs());
    }
    println!("equation of time: worst {worst:.6} s over the year 2000");
    assert!(worst < 0.01, "{worst} s");
}

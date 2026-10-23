//! Every solver ends on hostile input (`05-testing/01-quality-bar.md`,
//! "iteration caps"): a curve that is constant, not a number, infinite,
//! discontinuous, wild or failing, over windows and tolerances that are
//! reversed, empty, infinite or absurd, under caps of nothing. A solver
//! may answer or refuse; it may not run on. Each case runs on a thread
//! of its own against a clock, so a loop that never ends fails the test
//! rather than hanging it, and every curve counts what it was asked, so
//! a loop that ends only after a billion evaluations fails as well.

#![allow(clippy::unwrap_used, clippy::panic, reason = "tests fail by panicking")]

use std::sync::mpsc;
use std::time::Duration;

use proptest::prelude::*;
use teistro_astro::houses::{Input, houses};
use teistro_astro::rise_set::Solver;
use teistro_astro::solve::{
    Caps, Scan, first_zero, first_zero_gridded, minimum, next_crossing, refine, refine_known,
    refine_with_rates,
};
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::catalogue::HouseSystem;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::{OverridePolicy, PolarPolicy};
use teistro_port_ephemeris::{Body, Horizon, TestProvider};

/// The most evaluations any case may ask for. Every legitimate search
/// below is a scan of at most this many steps or a narrowing of a few
/// hundred, so more is a solver running on.
const MOST_EVALUATIONS: u32 = 2_000_000;

/// How long a case may take. A case is microseconds; this is slack for a
/// loaded machine and still far short of a hang.
const MOST_TIME: Duration = Duration::from_secs(20);

/// A hostile quantity of time, in days.
#[derive(Clone, Copy, Debug)]
enum Curve {
    /// A line through zero at `root`, the one honest curve.
    Line { root: f64, rate: f64 },
    /// The same value everywhere: zero, a sign, not a number, infinite.
    Constant(f64),
    /// Below zero before `at` and above after, with no zero between.
    Step { at: f64 },
    /// A sine so fast it changes sign between any two samples.
    Wild { frequency: f64 },
    /// An honest line that answers not a number past `at`.
    Holed { root: f64, at: f64 },
    /// An honest line whose evaluation fails after `after` readings.
    Failing { root: f64, after: u32 },
}

impl Curve {
    fn at(self, t: f64, asked: u32) -> Result<f64, &'static str> {
        match self {
            Curve::Line { root, rate } => Ok((t - root) * rate),
            Curve::Constant(value) => Ok(value),
            Curve::Step { at } => Ok(if t < at { -1.0 } else { 1.0 }),
            Curve::Wild { frequency } => Ok((t * frequency).sin()),
            Curve::Holed { root, at } => Ok(if t > at { f64::NAN } else { t - root }),
            Curve::Failing { root, after } => {
                if asked > after {
                    Err("the source failed")
                } else {
                    Ok(t - root)
                }
            }
        }
    }
}

fn curves() -> impl Strategy<Value = Curve> {
    prop_oneof![
        (instants(), rates()).prop_map(|(root, rate)| Curve::Line { root, rate }),
        values().prop_map(Curve::Constant),
        instants().prop_map(|at| Curve::Step { at }),
        prop::sample::select(vec![1e3, 1e6, 1e12, 1e300])
            .prop_map(|frequency| Curve::Wild { frequency }),
        (instants(), instants()).prop_map(|(root, at)| Curve::Holed { root, at }),
        (instants(), 0u32..40).prop_map(|(root, after)| Curve::Failing { root, after }),
    ]
}

/// Instants: ordinary, at a Julian day's magnitude, and absurd.
fn instants() -> impl Strategy<Value = f64> {
    prop_oneof![
        -10.0f64..10.0,
        2_451_545.0f64..2_451_555.0,
        prop::sample::select(vec![
            0.0,
            -0.0,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
            1e300,
            -1e300,
            1e15,
        ]),
    ]
}

/// Steps, tolerances and rates: ordinary, tiny, huge, and not positive.
fn sizes() -> impl Strategy<Value = f64> {
    prop_oneof![
        1e-9f64..10.0,
        prop::sample::select(vec![
            0.0,
            -0.0,
            -1.0,
            f64::NAN,
            f64::INFINITY,
            f64::MIN_POSITIVE,
            1e-300,
            1e300,
            f64::MAX,
        ]),
    ]
}

fn rates() -> impl Strategy<Value = f64> {
    prop_oneof![
        -10.0f64..10.0,
        prop::sample::select(vec![0.0, f64::NAN, f64::INFINITY, 1e300, 1e-300]),
    ]
}

fn values() -> impl Strategy<Value = f64> {
    prop::sample::select(vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ])
}

fn caps() -> impl Strategy<Value = Caps> {
    prop_oneof![
        Just(Caps::DEFAULT),
        (0u32..3, 0u32..3).prop_map(|(bracket_steps, refinements)| Caps {
            bracket_steps,
            refinements
        }),
    ]
}

/// Runs a case against the clock, on its own thread.
fn ends(case: impl FnOnce() + Send + 'static) {
    let (done, finished) = mpsc::channel();
    std::thread::spawn(move || {
        case();
        let _ = done.send(());
    });
    match finished.recv_timeout(MOST_TIME) {
        Ok(()) => {}
        Err(mpsc::RecvTimeoutError::Timeout) => panic!("a solver ran past {MOST_TIME:?}"),
        // The case's thread panicked: the assertion inside it said why.
        Err(mpsc::RecvTimeoutError::Disconnected) => panic!("a solver panicked"),
    }
}

/// A curve that counts what it is asked and refuses past the most a case
/// may ask, so a solver that runs on is caught when it does.
fn counted(curve: Curve) -> impl FnMut(f64) -> Result<f64, &'static str> {
    let mut asked = 0u32;
    move |t| {
        asked += 1;
        assert!(
            asked <= MOST_EVALUATIONS,
            "{curve:?} asked {asked} times: a solver running on"
        );
        curve.at(t, asked)
    }
}

/// The same with the rate a line's slope would give, for the solvers that
/// read one.
fn sloped(curve: Curve) -> impl FnMut(f64) -> Result<(f64, f64), &'static str> {
    let mut value = counted(curve);
    move |t| Ok((value(t)?, 1.0))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn a_scan_ends(
        curve in curves(),
        from in instants(),
        to in instants(),
        step in sizes(),
        tolerance in sizes(),
        upward in any::<bool>(),
        caps in caps(),
    ) {
        ends(move || {
            let _ = first_zero(counted(curve), from, to, step, upward, tolerance, caps);
            let mut value = counted(curve);
            let scan = Scan::new(from, to, step, upward, tolerance).with_caps(caps);
            let _ = first_zero_gridded(
                |instants, out| {
                    out.clear();
                    for at in instants {
                        out.push(value(*at)?);
                    }
                    Ok(())
                },
                counted(curve),
                &scan,
            );
        });
    }

    #[test]
    fn a_narrowing_ends(
        curve in curves(),
        lo in instants(),
        hi in instants(),
        ends_at in (values(), values()),
        tolerance in sizes(),
        caps in caps(),
    ) {
        ends(move || {
            let _ = refine(counted(curve), lo, hi, tolerance, caps);
            let _ = refine_known(counted(curve), (lo, ends_at.0), (hi, ends_at.1), tolerance, caps);
            let _ = refine_with_rates(
                sloped(curve),
                (lo, ends_at.0, 1.0),
                (hi, ends_at.1, f64::NAN),
                tolerance,
                caps,
            );
            let _ = minimum(counted(curve), lo, hi, tolerance, caps);
        });
    }

    /// Placidus iterates and the rest solve in closed form; under every
    /// polar policy, at the poles and past them, on angles that are not
    /// numbers, each system answers or refuses, and an answer from finite
    /// angles is twelve finite cusps.
    #[test]
    fn every_house_system_ends(
        armc in prop_oneof![0.0f64..360.0, values(), instants()],
        latitude in prop_oneof![-90.0f64..90.0, Just(90.0), Just(-90.0), Just(66.56), values()],
        obliquity in prop_oneof![22.0f64..25.0, Just(0.0), Just(90.0), values()],
        sun in prop::option::of(prop_oneof![-25.0f64..25.0, values()]),
        sidereal in prop_oneof![0.0f64..30.0, values()],
    ) {
        let input = Input {
            armc_deg: armc,
            latitude_deg: latitude,
            obliquity_deg: obliquity,
            sun_declination_deg: sun,
            sidereal_offset_deg: sidereal,
        };
        ends(move || {
            let finite = [armc, latitude, obliquity, sun.unwrap_or(0.0), sidereal]
                .iter()
                .all(|value| value.is_finite());
            for system in HouseSystem::ALL {
                for policy in [
                    PolarPolicy::Error,
                    PolarPolicy::Clamp,
                    PolarPolicy::FallbackPorphyry,
                    PolarPolicy::FallbackWholeSign,
                ] {
                    if let Ok(answer) = houses(system, &input, policy) {
                        assert!(
                            !finite || answer.cusps.iter().all(|cusp| cusp.is_finite()),
                            "{system:?} under {policy:?} at {input:?}: {:?}",
                            answer.cusps
                        );
                    }
                }
            }
        });
    }

    /// A day's rise and set at any place a `Place` admits: at the poles,
    /// a hair from them and on the polar circles, on a mountain, under
    /// every horizon, for the bodies a panchanga asks about. A day with no
    /// rise is an answer; a day the search cannot settle is a refusal;
    /// either way it ends.
    #[test]
    fn a_day_of_rising_and_setting_ends(
        body in prop::sample::select(vec![Body::Sun, Body::Moon, Body::Mercury, Body::Mars]),
        latitude in prop_oneof![
            -90.0f64..90.0,
            prop::sample::select(vec![90.0, -90.0, 89.999_999, -89.999_999, 66.56, -66.56, 0.0]),
        ],
        longitude in -180.0f64..180.0,
        altitude in prop_oneof![0.0f64..9_000.0, Just(-430.0)],
        day in 0u32..219_000,
        horizon in prop::sample::select(vec![
            Horizon::CENTRE_NO_REFRACTION,
            Horizon::UPPER_LIMB_REFRACTION,
            Horizon::LOWER_LIMB_REFRACTION,
            Horizon::CENTRE_REFRACTION,
        ]),
    ) {
        ends(move || {
            let provider = TestProvider::new();
            let completion = Completion::new(
                &provider,
                OverridePolicy::SdkOnly,
                DeltaTModel::TableThenModel,
            );
            let place = Place::new(
                Latitude::literal(latitude),
                Longitude::literal(longitude),
                Altitude::literal(altitude),
            );
            let solver = Solver::new(&completion, body, place, horizon, DeltaTModel::TableThenModel);
            // A day from 1800 to 2399, the default data's range.
            let midnight = JulianDay::<Ut1>::literal(2_378_496.5 + f64::from(day));
            let _ = solver.day(midnight);
        });
    }

    #[test]
    fn a_crossing_ends(
        curve in curves(),
        target in instants(),
        from in instants(),
        rate in sizes(),
        tolerance in sizes(),
        caps in caps(),
    ) {
        ends(move || {
            let _ = next_crossing(counted(curve), target, from, rate, tolerance, caps);
        });
    }
}

/// Ends that are finite and a width that is not: `f64::MAX - f64::MIN`
/// overflows, and halving an infinite width never brings it under the
/// tolerance. The count of halvings once ran on for ever here.
#[test]
fn a_bracket_too_wide_to_measure_ends() {
    let line = Curve::Line {
        root: 0.0,
        rate: 1.0,
    };
    ends(move || {
        let refused = refine(counted(line), f64::MIN, f64::MAX, 1e-9, Caps::DEFAULT);
        assert!(refused.is_err(), "{refused:?}");
        let known = refine_known(
            counted(line),
            (f64::MIN, -1.0),
            (f64::MAX, 1.0),
            1e-9,
            Caps::DEFAULT,
        );
        assert!(known.is_err(), "{known:?}");
        let rated = refine_with_rates(
            sloped(line),
            (f64::MIN, -1.0, 1.0),
            (f64::MAX, 1.0, 1.0),
            1e-9,
            Caps::DEFAULT,
        );
        assert!(rated.is_err(), "{rated:?}");
        // A scan steps between finite samples, so its brackets are
        // finite: from the least double to the greatest in steps of the
        // greatest it samples nought, and finds the line's own root.
        let scanned = first_zero(
            counted(line),
            f64::MIN,
            f64::MAX,
            f64::MAX,
            true,
            1e-9,
            Caps::DEFAULT,
        );
        assert!(
            scanned
                .as_ref()
                .is_ok_and(|found| found.is_some_and(|crossing| crossing.instant.abs() < 1e-9)),
            "{scanned:?}"
        );
        let least = minimum(counted(line), f64::MIN, f64::MAX, 1e-9, Caps::DEFAULT);
        assert!(least.is_err(), "{least:?}");
    });
}

//! The crossings and stations kernel over a provider with retrograde
//! motion: a synthetic looping planet whose longitude runs forward on
//! average and swings back on an epicycle, so that a sign boundary is
//! crossed forward, back and forward again around each station, and the
//! stations themselves are where its speed changes sign.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking and read small lists"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::events::{Direction, Lattice, Quantity, Search, StationKind, stations};
use teistro_astro::{Completion, events::Longitudes};
use teistro_core::angle::{difference_deg, normalise_deg};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{
    Astronomy, Body, Capabilities, Cell, CellStatus, DistanceUnit, EphemerisProvider, Frame,
    Identity, Overrides, PositionColumns, PositionRequest, ProviderError, Source, SpeedModel,
    validate,
};

const J2000: f64 = 2_451_545.0;

/// A planet that loops: 0.5° a day forward with a 12° epicycle of 100 days,
/// so its speed swings between +1.25 and −0.25 degrees a day and it runs
/// back over a 4.4° arc once a loop. From 5° the first and the fourth of
/// those arcs straddle a sign boundary (27.8°–32.2° and 177.8°–182.2°).
struct Looping;

impl Looping {
    const START: f64 = 5.0;
    const RATE: f64 = 0.5;
    const AMPLITUDE: f64 = 12.0;
    const PERIOD: f64 = 100.0;

    fn longitude(t: f64) -> f64 {
        let phase = core::f64::consts::TAU * t / Self::PERIOD;
        normalise_deg(Self::START + Self::RATE * t + Self::AMPLITUDE * phase.sin())
    }

    fn speed(t: f64) -> f64 {
        let phase = core::f64::consts::TAU * t / Self::PERIOD;
        Self::RATE + Self::AMPLITUDE * core::f64::consts::TAU / Self::PERIOD * phase.cos()
    }
}

impl EphemerisProvider for Looping {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            identity: Identity {
                name: "looping".to_owned(),
                version: "1".to_owned(),
                data_version: String::new(),
                tier: None,
                data_hashes: Vec::new(),
            },
            jd_range: (0.0, 1e7),
            bodies: vec![Body::Mars],
            native_frame: Frame::CANONICAL,
            astronomy: Astronomy::Modern,
            speeds: true,
            speed_model: SpeedModel::Derivative,
            distance_unit: DistanceUnit::AstronomicalUnits,
            overrides: Overrides::NONE,
            ayanamshas: Vec::new(),
            deterministic: true,
        }
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        validate(&self.capabilities(), request)?;
        let mut columns =
            PositionColumns::new(request.jds.len(), request.bodies.len(), request.frame);
        for (jd_index, jd) in request.jds.iter().enumerate() {
            for body_index in 0..request.bodies.len() {
                let t = jd - J2000;
                columns.set_at(
                    jd_index,
                    body_index,
                    Cell {
                        lon: Looping::longitude(t),
                        lat: 0.0,
                        dist: 1.5,
                        lon_speed: Looping::speed(t),
                        lat_speed: 0.0,
                        dist_speed: 0.0,
                        status: CellStatus::Ok,
                        source: Source::UNKNOWN,
                    },
                );
            }
        }
        Ok(columns)
    }
}

#[test]
fn a_looping_planet_crosses_a_boundary_three_times_and_stations_bracket_the_loop() {
    let provider = Looping;
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let from = JulianDay::<Ut1>::literal(J2000);
    let to = JulianDay::<Ut1>::literal(J2000 + 400.0);

    // Stations: two per 100-day loop, alternating, at zero speed.
    let found = stations(&longitudes, Body::Mars, from, to, 1e-7).unwrap();
    assert_eq!(found.len(), 8, "{found:?}");
    for pair in found.windows(2) {
        assert!(pair[1].instant.get() > pair[0].instant.get());
        assert_ne!(pair[0].kind, pair[1].kind);
    }
    for station in &found {
        let speed = Looping::speed(station.instant.get() - J2000);
        assert!(speed.abs() < 1e-5, "{speed}");
        let (lon, _) = longitudes
            .longitude_and_speed(Body::Mars, station.instant)
            .unwrap();
        assert_eq!(lon, station.longitude_deg);
    }
    // The first station is where the speed turns negative: retrograde.
    assert_eq!(found[0].kind, StationKind::Retrograde);

    // Sign ingresses: every crossing sits on a 30° line, the falling ones are
    // the retrograde re-entries, and each rising-falling-rising triple spans
    // one loop.
    let crossings = Search::new(&longitudes, Quantity::Longitude(Body::Mars), Lattice::SIGNS)
        .between(from, to)
        .unwrap();
    assert!(!crossings.is_empty());
    // The narrowing places an event in a handful of evaluations: two for
    // the bracket's ends and at most seven steps (nine measured).
    let most = crossings.iter().map(|e| e.evaluations).max().unwrap();
    assert!(most <= 10, "{most}");
    let mut falling = 0;
    for event in &crossings {
        let lon = Looping::longitude(event.instant.get() - J2000);
        assert!(
            difference_deg(lon, event.boundary_deg).abs() < 1e-5,
            "{lon} {}",
            event.boundary_deg
        );
        assert_eq!(event.boundary_deg % 30.0, 0.0);
        if event.direction == Direction::Falling {
            falling += 1;
        }
    }
    assert_eq!(falling, 2, "{crossings:?}");
    for pair in crossings.windows(2) {
        assert!(pair[1].instant.get() > pair[0].instant.get());
        // Consecutive crossings of the same line alternate in direction.
        if difference_deg(pair[0].boundary_deg, pair[1].boundary_deg).abs() < 1e-9 {
            assert_ne!(pair[0].direction, pair[1].direction);
        }
    }
    // The net advance over the window is what the mean motion says: four
    // whole loops carry the planet 200° forward, from 5° to 205°, past six
    // sign boundaries net, and two of them three times each.
    let net = crossings
        .iter()
        .map(|e| {
            if e.direction == Direction::Rising {
                1i32
            } else {
                -1
            }
        })
        .sum::<i32>();
    assert_eq!(net, 6, "{crossings:?}");
    assert_eq!(crossings.len(), 10, "{crossings:?}");

    // A single target is one line of the lattice: the same instants.
    let single = Search::new(
        &longitudes,
        Quantity::Longitude(Body::Mars),
        Lattice::single(60.0),
    )
    .between(from, to)
    .unwrap();
    let from_lattice: Vec<_> = crossings
        .iter()
        .filter(|e| e.boundary_deg == 60.0)
        .collect();
    assert_eq!(single.len(), from_lattice.len());
    for (a, b) in single.iter().zip(from_lattice) {
        assert!((a.instant.get() - b.instant.get()).abs() < 1e-7);
    }
}

/// A source that counts the requests made of it and forwards them, so a
/// test can say how many round trips a search made as well as what it
/// answered.
struct Counted<'a, S: Longitudes + ?Sized> {
    inner: &'a S,
    requests: std::sync::atomic::AtomicUsize,
}

impl<'a, S: Longitudes + ?Sized> Counted<'a, S> {
    fn new(inner: &'a S) -> Counted<'a, S> {
        Counted {
            inner,
            requests: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn requests(&self) -> usize {
        self.requests.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl<S: Longitudes + ?Sized> Longitudes for Counted<'_, S> {
    fn longitude_and_speed(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<(f64, f64), Error> {
        self.requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.inner.longitude_and_speed(body, ut1)
    }

    fn longitude_and_speed_pair(
        &self,
        bodies: [Body; 2],
        ut1: JulianDay<Ut1>,
    ) -> Result<[(f64, f64); 2], Error> {
        self.requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.inner.longitude_and_speed_pair(bodies, ut1)
    }

    fn longitudes_and_speeds(
        &self,
        body: Body,
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<(f64, f64)>,
    ) -> Result<(), Error> {
        self.requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.inner.longitudes_and_speeds(body, ut1, out)
    }

    fn longitudes_and_speeds_pair(
        &self,
        bodies: [Body; 2],
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<[(f64, f64); 2]>,
    ) -> Result<(), Error> {
        self.requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.inner.longitudes_and_speeds_pair(bodies, ut1, out)
    }

    fn describe(&self) -> String {
        self.inner.describe()
    }
}

/// The grid answers what the walk answered, to the bit.
///
/// A scan asks for its instants a grid at a time (A1b of
/// `07-roadmap/02-plan-performance-and-passthrough.md`), and a chunk of
/// one is the walk it replaced. Every crossing must be identical — not
/// close, identical — because the instants are the same instants and the
/// values are the same values, so the brackets handed to the refinement
/// are the same brackets. Anything less would move every instant the SDK
/// publishes with nothing to say so.
///
/// The looping planet is the hard case on purpose: it crosses a boundary
/// forward, back and forward again, so the scan meets rising and falling
/// brackets and lines met more than once.
#[test]
fn a_scan_that_asks_for_a_grid_answers_what_the_walk_answered_to_the_bit() {
    let provider = Looping;
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let from = JulianDay::<Ut1>::literal(J2000);
    let to = JulianDay::<Ut1>::literal(J2000 + 400.0);

    for quantity in [
        Quantity::Longitude(Body::Mars),
        Quantity::Speed(Body::Mars),
        // A composite, which reads a pair at each instant. The looping
        // provider answers for one body, so the pair is that body twice
        // and the combination is its own longitude — which is the point:
        // it is the *pair* path being exercised, not the arithmetic.
        Quantity::Composite {
            a: 2.0,
            first: Body::Mars,
            b: -1.0,
            second: Body::Mars,
        },
    ] {
        for lattice in [Lattice::SIGNS, Lattice::single(60.0)] {
            let walked = Counted::new(&longitudes);
            let walk = Search::new(&walked, quantity, lattice)
                .with_chunk(1)
                .between(from, to)
                .unwrap();
            for chunk in [2usize, 7, 64, 512, 100_000] {
                let gridded = Counted::new(&longitudes);
                let grid = Search::new(&gridded, quantity, lattice)
                    .with_chunk(chunk)
                    .between(from, to)
                    .unwrap();
                assert_eq!(
                    grid.len(),
                    walk.len(),
                    "{quantity:?} chunk {chunk}: the same crossings"
                );
                for (grid, walk) in grid.iter().zip(&walk) {
                    assert_eq!(
                        grid.instant.get().to_bits(),
                        walk.instant.get().to_bits(),
                        "{quantity:?} chunk {chunk}: {} against {}",
                        grid.instant.get(),
                        walk.instant.get()
                    );
                    assert_eq!(grid.boundary_deg, walk.boundary_deg);
                    assert_eq!(grid.direction, walk.direction);
                    assert_eq!(grid.evaluations, walk.evaluations);
                }
                assert!(
                    gridded.requests() < walked.requests(),
                    "{quantity:?} chunk {chunk}: {} requests against the walk's {}",
                    gridded.requests(),
                    walked.requests()
                );
            }
        }
    }
}

/// What the grid actually saves, on the numbers rather than in principle.
///
/// The scan over four hundred days at a one-day step is 401 instants. As
/// a walk that is 401 round trips; as one grid it is one, and the
/// refinements — which cannot be asked for in advance, since each step
/// is chosen from the answer to the last — are what remains.
#[test]
fn the_grid_turns_a_scans_round_trips_into_one() {
    let provider = Looping;
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let from = JulianDay::<Ut1>::literal(J2000);
    let to = JulianDay::<Ut1>::literal(J2000 + 400.0);
    let quantity = Quantity::Longitude(Body::Mars);

    let walked = Counted::new(&longitudes);
    let walk = Search::new(&walked, quantity, Lattice::SIGNS)
        .with_chunk(1)
        .between(from, to)
        .unwrap();
    let refinements: u32 = walk.iter().map(|event| event.evaluations).sum();

    let gridded = Counted::new(&longitudes);
    Search::new(&gridded, quantity, Lattice::SIGNS)
        .between(from, to)
        .unwrap();

    // The walk asks once per sample and once per refinement step.
    assert_eq!(walked.requests(), 401 + refinements as usize);
    // The grid asks once for all 401, and the refinements are unchanged.
    assert_eq!(gridded.requests(), 1 + refinements as usize);
}

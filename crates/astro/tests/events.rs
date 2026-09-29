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

/// A body on an epicycle: `rate` degrees a day forward, carried round an
/// `amplitude`-degree circle every `period` days, so its speed swings by
/// `amplitude × 2π / period` either side of the mean and it turns back
/// when that exceeds the mean.
#[derive(Clone, Copy)]
struct Epicycle {
    body: Body,
    start: f64,
    rate: f64,
    amplitude: f64,
    period: f64,
}

impl Epicycle {
    fn longitude(&self, t: f64) -> f64 {
        let phase = core::f64::consts::TAU * t / self.period;
        normalise_deg(self.start + self.rate * t + self.amplitude * phase.sin())
    }

    fn speed(&self, t: f64) -> f64 {
        let phase = core::f64::consts::TAU * t / self.period;
        self.rate + self.amplitude * core::f64::consts::TAU / self.period * phase.cos()
    }
}

/// A planet that loops: 0.5° a day forward with a 12° epicycle of 100 days,
/// so its speed swings between +1.25 and −0.25 degrees a day and it runs
/// back over a 4.4° arc once a loop. From 5° the first and the fourth of
/// those arcs straddle a sign boundary (27.8°–32.2° and 177.8°–182.2°).
///
/// It turns every few weeks, faster than any planet does, so it answers as
/// the **true node**, the body whose turns `events::shortest_run_days`
/// does not bound: every search over it scans the fine grid, which is what
/// the tests below hold. A planet's name would promise a run it breaks.
const LOOPING: Epicycle = Epicycle {
    body: Body::TrueNode,
    start: 5.0,
    rate: 0.5,
    amplitude: 12.0,
    period: 100.0,
};

impl EphemerisProvider for Epicycle {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            identity: Identity {
                name: "epicycle".to_owned(),
                version: "1".to_owned(),
                data_version: String::new(),
                tier: None,
                data_hashes: Vec::new(),
            },
            jd_range: (0.0, 1e7),
            bodies: vec![self.body],
            native_frame: Frame::CANONICAL,
            astronomy: Astronomy::Modern,
            speeds: true,
            speed_model: SpeedModel::Derivative,
            distance_unit: DistanceUnit::AstronomicalUnits,
            overrides: Overrides::NONE,
            ayanamshas: Vec::new(),
            deterministic: true,
            native: false,
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
                        lon: self.longitude(t),
                        lat: 0.0,
                        dist: 1.5,
                        lon_speed: self.speed(t),
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
    let provider = LOOPING;
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let from = JulianDay::<Ut1>::literal(J2000);
    let to = JulianDay::<Ut1>::literal(J2000 + 400.0);

    // Stations: two per 100-day loop, alternating, at zero speed.
    let found = stations(&longitudes, LOOPING.body, from, to, 1e-7).unwrap();
    assert_eq!(found.len(), 8, "{found:?}");
    for pair in found.windows(2) {
        assert!(pair[1].instant.get() > pair[0].instant.get());
        assert_ne!(pair[0].kind, pair[1].kind);
    }
    for station in &found {
        let speed = LOOPING.speed(station.instant.get() - J2000);
        assert!(speed.abs() < 1e-5, "{speed}");
        let (lon, _) = longitudes
            .longitude_and_speed(LOOPING.body, station.instant)
            .unwrap();
        assert_eq!(lon, station.longitude_deg);
    }
    // The first station is where the speed turns negative: retrograde.
    assert_eq!(found[0].kind, StationKind::Retrograde);

    // Sign ingresses: every crossing sits on a 30° line, the falling ones are
    // the retrograde re-entries, and each rising-falling-rising triple spans
    // one loop.
    let crossings = Search::new(
        &longitudes,
        Quantity::Longitude(LOOPING.body),
        Lattice::SIGNS,
    )
    .between(from, to)
    .unwrap();
    assert!(!crossings.is_empty());
    // The narrowing places an event in a handful of evaluations, the
    // bracket's ends being the scan's own samples with their speeds: the
    // cubic through them, a Newton step, and one evaluation closing the
    // bracket past it; a line crossed while the planet turns takes more.
    let most = crossings.iter().map(|e| e.evaluations).max().unwrap();
    assert!(most <= 4, "{most}");
    let mut falling = 0;
    for event in &crossings {
        let lon = LOOPING.longitude(event.instant.get() - J2000);
        assert!(
            // The Newton point from the nearer end, to the instant's last
            // bits (an ulp of a Julian day is 5e-10 days), where a
            // bracket's middle could be half the tolerance away.
            difference_deg(lon, event.boundary_deg).abs() < 1e-9,
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
        Quantity::Longitude(LOOPING.body),
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
    /// The instants asked for, over every request: what a scan costs an
    /// ephemeris, where a request is what it costs a boundary.
    instants: std::sync::atomic::AtomicUsize,
}

impl<'a, S: Longitudes + ?Sized> Counted<'a, S> {
    fn new(inner: &'a S) -> Counted<'a, S> {
        Counted {
            inner,
            requests: std::sync::atomic::AtomicUsize::new(0),
            instants: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn requests(&self) -> usize {
        self.requests.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn instants(&self) -> usize {
        self.instants.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn count(&self, instants: usize) {
        self.requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.instants
            .fetch_add(instants, std::sync::atomic::Ordering::Relaxed);
    }
}

impl<S: Longitudes + ?Sized> Longitudes for Counted<'_, S> {
    fn longitude_and_speed(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<(f64, f64), Error> {
        self.count(1);
        self.inner.longitude_and_speed(body, ut1)
    }

    fn longitude_and_speed_pair(
        &self,
        bodies: [Body; 2],
        ut1: JulianDay<Ut1>,
    ) -> Result<[(f64, f64); 2], Error> {
        self.count(1);
        self.inner.longitude_and_speed_pair(bodies, ut1)
    }

    fn longitudes_and_speeds(
        &self,
        body: Body,
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<(f64, f64)>,
    ) -> Result<(), Error> {
        self.count(ut1.len());
        self.inner.longitudes_and_speeds(body, ut1, out)
    }

    fn longitudes_and_speeds_pair(
        &self,
        bodies: [Body; 2],
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<[(f64, f64); 2]>,
    ) -> Result<(), Error> {
        self.count(ut1.len());
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
    let provider = LOOPING;
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let from = JulianDay::<Ut1>::literal(J2000);
    let to = JulianDay::<Ut1>::literal(J2000 + 400.0);

    for quantity in [
        Quantity::Longitude(LOOPING.body),
        Quantity::Speed(LOOPING.body),
        // A composite, which reads a pair at each instant. The looping
        // provider answers for one body, so the pair is that body twice
        // and the combination is its own longitude — which is the point:
        // it is the *pair* path being exercised, not the arithmetic.
        Quantity::Composite {
            a: 2.0,
            first: LOOPING.body,
            b: -1.0,
            second: LOOPING.body,
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
    let provider = LOOPING;
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let from = JulianDay::<Ut1>::literal(J2000);
    let to = JulianDay::<Ut1>::literal(J2000 + 400.0);
    let quantity = Quantity::Longitude(LOOPING.body);

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

/// A crossing is a property of the crossing, not of the question.
///
/// The scan used to step from the caller's own `from`, so where its
/// samples fell — and so which bracket the refinement was handed —
/// depended on where the window started. Measured before this was fixed:
/// the same sign ingress came back up to 2.2 milliseconds apart from
/// windows offset by a fraction of a day, and only four of fifteen
/// comparisons agreed to the bit.
///
/// Samples are aligned to `events::SCAN_ANCHOR_JD` now, so a narrower window's
/// samples are a subset of a wider one's and any two windows that both
/// contain a crossing bracket it identically. That is worth having on its
/// own, and it is what lets a range search once and slice the answer per
/// day without the slices disagreeing with the searches they replace.
#[test]
fn the_same_crossing_answers_the_same_from_any_window() {
    let provider = LOOPING;
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);

    for quantity in [
        Quantity::Longitude(LOOPING.body),
        Quantity::Composite {
            a: 2.0,
            first: LOOPING.body,
            b: -1.0,
            second: LOOPING.body,
        },
    ] {
        let wide = Search::new(&longitudes, quantity, Lattice::SIGNS)
            .between(
                JulianDay::<Ut1>::literal(J2000),
                JulianDay::<Ut1>::literal(J2000 + 400.0),
            )
            .unwrap();
        assert!(!wide.is_empty());
        let mut compared = 0usize;
        // Windows that begin and end at every kind of awkward place: on a
        // sample, a fraction of a step past one, and just short of the
        // next.
        for offset in [0.0_f64, 0.1, 0.25, 0.5, 0.7, 0.999] {
            let narrow = Search::new(&longitudes, quantity, Lattice::SIGNS)
                .between(
                    JulianDay::<Ut1>::literal(J2000 + 20.0 + offset),
                    JulianDay::<Ut1>::literal(J2000 + 380.0 + offset),
                )
                .unwrap();
            assert!(!narrow.is_empty(), "offset {offset}");
            for near in &narrow {
                let far = wide
                    .iter()
                    .find(|far| (far.instant.get() - near.instant.get()).abs() < 0.5)
                    .unwrap_or_else(|| {
                        panic!("offset {offset}: {} is in neither", near.instant.get())
                    });
                assert_eq!(
                    near.instant.get().to_bits(),
                    far.instant.get().to_bits(),
                    "offset {offset}: {} against {}",
                    near.instant.get(),
                    far.instant.get()
                );
                assert_eq!(near.boundary_deg, far.boundary_deg);
                assert_eq!(near.direction, far.direction);
                assert_eq!(near.evaluations, far.evaluations);
                compared += 1;
            }
        }
        assert!(compared >= 30, "{quantity:?}: only {compared} compared");
    }
}

/// Every crossing a window reports is inside it.
///
/// The scan's ends are lattice points rather than the caller's own
/// instants, so a bracket reaches outside the window at either end and
/// may hold a crossing that is real but not this window's.
#[test]
fn a_window_reports_only_the_crossings_inside_it() {
    let provider = LOOPING;
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let quantity = Quantity::Longitude(LOOPING.body);

    // A window that begins and ends between samples, so both end brackets
    // reach outside it.
    let from = JulianDay::<Ut1>::literal(J2000 + 25.5);
    let to = JulianDay::<Ut1>::literal(J2000 + 200.5);
    let events = Search::new(&longitudes, quantity, Lattice::SIGNS)
        .between(from, to)
        .unwrap();
    assert!(!events.is_empty());
    for event in &events {
        assert!(
            event.instant.get() >= from.get() && event.instant.get() <= to.get(),
            "{} is outside [{}, {}]",
            event.instant.get(),
            from.get(),
            to.get()
        );
    }
    // And nothing inside was lost: the same crossings the wide search saw
    // in that span.
    let wide = Search::new(&longitudes, quantity, Lattice::SIGNS)
        .between(
            JulianDay::<Ut1>::literal(J2000),
            JulianDay::<Ut1>::literal(J2000 + 400.0),
        )
        .unwrap();
    let inside: Vec<_> = wide
        .iter()
        .filter(|e| e.instant.get() >= from.get() && e.instant.get() <= to.get())
        .collect();
    assert_eq!(events.len(), inside.len());
    for (near, far) in events.iter().zip(inside) {
        assert_eq!(near.instant.get().to_bits(), far.instant.get().to_bits());
    }
}

/// Several lattices scanned once answer what each lattice's own search
/// answers at the same step, to the bit, and the scan is asked for once:
/// the hit list's signs, nakshatras and aspect lines cost one walk and not
/// one each (`03-design/transit-hit-list.md`).
#[test]
fn several_lattices_scanned_once_answer_what_each_answers_alone() {
    let provider = LOOPING;
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let quantity = Quantity::Longitude(LOOPING.body);
    let (from, to) = (
        JulianDay::<Ut1>::literal(J2000),
        JulianDay::<Ut1>::literal(J2000 + 400.0),
    );
    let aspect = Lattice {
        origin_deg: 101.25,
        step_deg: 90.0,
    };
    let lattices = [Lattice::SIGNS, Lattice::NAKSHATRAS, aspect];

    let counted = Counted::new(&longitudes);
    let search = Search::each(
        &counted,
        quantity,
        Lattice::SIGNS,
        [Lattice::NAKSHATRAS, aspect],
    );
    let step = search.step_days();
    let each = search.between_each(from, to).unwrap();
    // The finest lattice sets the step for all of them.
    assert_eq!(
        step,
        Search::new(&longitudes, quantity, Lattice::NAKSHATRAS).step_days()
    );
    assert_eq!(each.len(), lattices.len());
    let mut refinements = 0;
    for (lattice, found) in lattices.iter().zip(&each) {
        let alone = Search::new(&longitudes, quantity, *lattice)
            .with_step_days(step)
            .between(from, to)
            .unwrap();
        assert!(!alone.is_empty(), "{lattice:?}");
        assert_eq!(found.len(), alone.len(), "{lattice:?}");
        for (a, b) in found.iter().zip(&alone) {
            assert_eq!(a.instant.get().to_bits(), b.instant.get().to_bits());
            assert_eq!((a.boundary_deg, a.direction), (b.boundary_deg, b.direction));
        }
        refinements += found.iter().map(|e| e.evaluations as usize).sum::<usize>();
    }
    // One grid for the scan, however many lattices it was tested against:
    // four hundred days at the nakshatras' step is fewer instants than a
    // grid holds, so the scan is one request and the rest refinements.
    assert_eq!(counted.requests(), 1 + refinements);
    // And `between` is every lattice's crossings in time order.
    let merged = search.between(from, to).unwrap();
    assert_eq!(merged.len(), each.iter().map(Vec::len).sum::<usize>());
    assert!(
        merged
            .windows(2)
            .all(|w| w[0].instant.get() <= w[1].instant.get())
    );
}

/// A slow planet's scan strides and still answers what the fine scan
/// answers, to the bit (`astro-events-and-crossings.md` §4, plan A1g).
///
/// Each body keeps the promise its row of `shortest_run_days` makes, since
/// the stride rests on it: a Saturn that retrogrades for some 148 days and
/// runs forward for 230 (the table says no run is under 133.5), a Sun
/// whose speed wavers but never turns, and a node running steadily
/// backwards across 0°. Each is searched over signs, nakshatras and a
/// point's aspects at once, and alone over each, from windows opening at
/// awkward places, and every crossing is held to the fine scan's own:
/// the instant's bits, the line, the direction and the evaluations the
/// refinement took. The stride is only worth having if it saves, so the
/// requests are counted too.
#[test]
fn a_strided_scan_answers_what_the_fine_scan_answers_to_the_bit() {
    use teistro_astro::events::shortest_run_days;

    let bodies = [
        Epicycle {
            body: Body::Saturn,
            start: 25.0,
            rate: 0.0335,
            amplitude: 6.0,
            period: 378.0,
        },
        Epicycle {
            body: Body::Sun,
            start: 280.0,
            rate: 0.9856,
            amplitude: 1.5,
            period: 365.25,
        },
        Epicycle {
            body: Body::MeanNode,
            start: 10.0,
            rate: -0.053,
            amplitude: 0.0,
            period: 1.0,
        },
    ];
    let point = Lattice {
        origin_deg: 101.25,
        step_deg: 90.0,
    };
    for body in bodies {
        let completion =
            Completion::new(&body, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
        let longitudes = completion.longitudes(Frame::CANONICAL);
        let quantity = Quantity::Longitude(body.body);
        let run = shortest_run_days(body.body).unwrap();
        let mut compared = 0usize;
        for lattices in [
            vec![Lattice::SIGNS, Lattice::NAKSHATRAS, point],
            vec![Lattice::SIGNS],
            vec![Lattice::single(0.0)],
        ] {
            for (from, days) in [(0.0, 21_915.0), (123.456, 4_000.0), (7_000.999, 2_555.5)] {
                let (from, to) = (
                    JulianDay::<Ut1>::literal(J2000 + from),
                    JulianDay::<Ut1>::literal(J2000 + from + days),
                );
                let rest = lattices[1..].iter().copied();
                let strided = Counted::new(&longitudes);
                let search = Search::each(&strided, quantity, lattices[0], rest.clone());
                let fine = search.step_days();
                assert!(
                    search.scan_step_days() > fine && search.scan_step_days() <= run * 0.75,
                    "{:?}: scans every {} days on a {fine}-day grid",
                    body.body,
                    search.scan_step_days()
                );
                let found = search.between_each(from, to).unwrap();
                let walked = Counted::new(&longitudes);
                let expected = Search::each(&walked, quantity, lattices[0], rest)
                    .with_step_days(fine)
                    .between_each(from, to)
                    .unwrap();
                assert_eq!(found.len(), expected.len());
                for (found, expected) in found.iter().zip(&expected) {
                    assert_eq!(
                        found.len(),
                        expected.len(),
                        "{:?} {lattices:?} from {}",
                        body.body,
                        from.get()
                    );
                    for (a, b) in found.iter().zip(expected) {
                        assert_eq!(a.instant.get().to_bits(), b.instant.get().to_bits());
                        assert_eq!(
                            (a.boundary_deg, a.direction, a.evaluations),
                            (b.boundary_deg, b.direction, b.evaluations)
                        );
                        compared += 1;
                    }
                }
                assert!(
                    strided.instants() * 2 < walked.instants(),
                    "{:?}: {} instants against the fine scan's {}",
                    body.body,
                    strided.instants(),
                    walked.instants()
                );
            }
        }
        assert!(compared > 200, "{:?}: only {compared} compared", body.body);
    }
}

/// The Saturn above does turn: the strided scan's crossings of one line
/// alternate in direction exactly where the fine scan's do, which is the
/// loop the stride must not step over.
#[test]
fn a_strided_scan_keeps_every_retrograde_recrossing() {
    let saturn = Epicycle {
        body: Body::Saturn,
        start: 25.0,
        rate: 0.0335,
        amplitude: 6.0,
        period: 378.0,
    };
    let completion = Completion::new(
        &saturn,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let crossings = Search::new(
        &longitudes,
        Quantity::Longitude(Body::Saturn),
        Lattice::SIGNS,
    )
    .between(
        JulianDay::<Ut1>::literal(J2000),
        JulianDay::<Ut1>::literal(J2000 + 21_915.0),
    )
    .unwrap();
    let falling = crossings
        .iter()
        .filter(|event| event.direction == Direction::Falling)
        .count();
    // Sixty years carry it round twice, and each loop that straddles a
    // line crosses it three times: the net is the lines between the ends.
    let (start, end) = (saturn.start, saturn.start + saturn.rate * 21_915.0);
    let end = end + saturn.amplitude * (core::f64::consts::TAU * 21_915.0 / saturn.period).sin();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a few dozen lines, forward"
    )]
    let net = ((end / 30.0).floor() - (start / 30.0).floor()) as usize;
    let rising = crossings.len() - falling;
    assert_eq!(rising - falling, net, "{crossings:?}");
    assert!(falling >= 10, "only {falling} retrograde crossings");
    for event in &crossings {
        let lon = saturn.longitude(event.instant.get() - J2000);
        assert!(difference_deg(lon, event.boundary_deg).abs() < 1e-9);
    }
}

//! The measurement pass over **how long a body runs between stations**
//! (`03-design/astro-events-and-crossings.md` §4, "a slow body's scan";
//! plan A1g).
//!
//! `cargo xtask stations` writes `docs/03-design/station-runs-measured.md`;
//! `check-stations` regenerates it in memory and fails on any difference.
//!
//! A crossing search samples a slow body every `stride` steps of its fine
//! grid and reads the grid only where a line or a station is, which is
//! only sound while no two stations fall inside one stride. The stride is
//! bounded by `teistro_astro::events::shortest_run_days`, a table, and
//! this pass is what holds that table to the sky: every body's stations
//! over the built-in ephemeris's whole coverage, sampled daily, and every
//! crossing the strided scan finds compared, to the bit, with the one the
//! fine scan finds. A body per thread, since each is a quarter of a
//! million instants twice over.

use std::fmt::Write as _;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use teistro_astro::events::{Event, Lattice, Longitudes, Quantity, Search, shortest_run_days};
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_port_ephemeris::{Body, Centre, EphemerisProvider, Frame};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, share, spelled, table, verdict_of};

const PAGE: &str = "docs/03-design/station-runs-measured.md";

/// How many instants a scan of the speeds asks for at once.
const CHUNK: usize = 4096;

/// The share of a planet's shortest measured run its table entry must
/// reach: below it the table is cautious enough to cost strides for
/// nothing, and the bound is what makes that a finding rather than a
/// preference.
const TABLE_FLOOR: f64 = 0.95;

/// The window the geocentric comparison runs over, 1900 to 2100, and the
/// topocentric one, 2000 to 2020. The table is held to the whole
/// coverage, which is what makes the stride sound; the comparison checks
/// the kernel that relies on it, and six centuries of the Moon seen from
/// three places cost half an hour to say nothing two centuries do not.
const GEOCENTRIC: (f64, f64) = (2_415_020.5, 2_488_069.5);
const TOPOCENTRIC: (f64, f64) = (2_451_544.5, 2_458_849.5);

/// The bodies a chart reads, in the catalogue's order.
const BODIES: [Body; 12] = [
    Body::Sun,
    Body::Moon,
    Body::Mercury,
    Body::Venus,
    Body::Mars,
    Body::Jupiter,
    Body::Saturn,
    Body::Uranus,
    Body::Neptune,
    Body::Pluto,
    Body::MeanNode,
    Body::TrueNode,
];

/// The bodies that move one way only, whose table entry is unbounded.
const NEVER_TURN: [Body; 3] = [Body::Sun, Body::Moon, Body::MeanNode];

/// Where the topocentric comparison stands: Delhi, the corpus's commonest
/// latitude band, and Tromsø, where the parallax's daily swing is widest
/// against the horizon.
const PLACES: [(&str, f64, f64); 2] = [("Delhi", 28.6139, 77.2090), ("Tromsø", 69.6492, 18.9553)];

/// The lattices the comparison crosses together: the two a chart names
/// and a single point off both, so that no line of one is a line of
/// another.
fn lattices() -> [Lattice; 3] {
    [Lattice::SIGNS, Lattice::NAKSHATRAS, Lattice::single(101.25)]
}

pub(crate) fn generate(root: &Path) -> i32 {
    match page() {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page() {
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask stations") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// A source of longitudes that counts the instants asked of it.
struct Counting<'a, S: Longitudes + ?Sized> {
    inner: &'a S,
    instants: AtomicUsize,
}

impl<'a, S: Longitudes + ?Sized> Counting<'a, S> {
    fn new(inner: &'a S) -> Self {
        Counting {
            inner,
            instants: AtomicUsize::new(0),
        }
    }

    fn instants(&self) -> usize {
        self.instants.load(Ordering::Relaxed)
    }
}

impl<S: Longitudes + ?Sized> Longitudes for Counting<'_, S> {
    fn longitude_and_speed(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<(f64, f64), Error> {
        self.instants.fetch_add(1, Ordering::Relaxed);
        self.inner.longitude_and_speed(body, ut1)
    }

    fn longitude_and_speed_pair(
        &self,
        bodies: [Body; 2],
        ut1: JulianDay<Ut1>,
    ) -> Result<[(f64, f64); 2], Error> {
        self.instants.fetch_add(1, Ordering::Relaxed);
        self.inner.longitude_and_speed_pair(bodies, ut1)
    }

    fn longitudes_and_speeds(
        &self,
        body: Body,
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<(f64, f64)>,
    ) -> Result<(), Error> {
        self.instants.fetch_add(ut1.len(), Ordering::Relaxed);
        self.inner.longitudes_and_speeds(body, ut1, out)
    }

    fn longitudes_and_speeds_pair(
        &self,
        bodies: [Body; 2],
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<[(f64, f64); 2]>,
    ) -> Result<(), Error> {
        self.instants.fetch_add(ut1.len(), Ordering::Relaxed);
        self.inner.longitudes_and_speeds_pair(bodies, ut1, out)
    }

    fn describe(&self) -> String {
        self.inner.describe()
    }
}

/// One scan compared with the fine scan it stands in for.
#[derive(Clone, Copy, Default)]
struct Compared {
    /// Crossings the fine scan found.
    crossings: usize,
    /// Crossings the two scans do not agree on to the bit, or that one
    /// found and the other did not.
    differing: usize,
    fine_instants: usize,
    strided_instants: usize,
}

impl Compared {
    fn add(&mut self, other: Compared) {
        self.crossings += other.crossings;
        self.differing += other.differing;
        self.fine_instants += other.fine_instants;
        self.strided_instants += other.strided_instants;
    }
}

/// What one body's stations and scans came to.
struct Measured {
    body: Body,
    stations: usize,
    /// The shortest run the body was retrograde and the shortest it was
    /// direct, days; infinite for a body that never turned twice.
    retrograde: f64,
    direct: f64,
    /// The table's entry, if the body has one.
    table: Option<f64>,
    /// The scan's stride over the compared lattices, in fine steps.
    stride: usize,
    geocentric: Compared,
    topocentric: Compared,
}

/// Every station over the window, with which way the body turned: `true`
/// when it turned retrograde. Sampled daily, each placed by the speed's
/// straight line between the two samples either side.
fn stations(
    source: &impl Longitudes,
    body: Body,
    (from, to): (f64, f64),
) -> Result<Vec<(f64, bool)>, String> {
    let mut found = Vec::new();
    let mut previous: Option<(f64, f64)> = None;
    let (mut at, mut instants, mut values) =
        (from, Vec::with_capacity(CHUNK), Vec::with_capacity(CHUNK));
    while at <= to {
        instants.clear();
        while instants.len() < CHUNK && at <= to {
            instants.push(JulianDay::<Ut1>::literal(at));
            at += 1.0;
        }
        source
            .longitudes_and_speeds(body, &instants, &mut values)
            .map_err(|why| format!("{body:?}'s speeds: {why}"))?;
        for (t, &(_, speed)) in instants.iter().zip(&values) {
            if let Some((t0, s0)) = previous
                && (s0 > 0.0) != (speed > 0.0)
            {
                found.push((t0 + (t.get() - t0) * s0 / (s0 - speed), speed <= 0.0));
            }
            previous = Some((t.get(), speed));
        }
    }
    Ok(found)
}

/// The shortest retrograde and the shortest direct run between stations.
fn shortest_runs(stations: &[(f64, bool)]) -> (f64, f64) {
    let (mut retrograde, mut direct) = (f64::INFINITY, f64::INFINITY);
    for pair in stations.windows(2) {
        if let (Some(&(start, backwards)), Some(&(end, _))) = (pair.first(), pair.get(1)) {
            let run = end - start;
            if backwards {
                retrograde = retrograde.min(run);
            } else {
                direct = direct.min(run);
            }
        }
    }
    (retrograde, direct)
}

/// Whether two crossings are the same crossing, to the bit.
fn same(a: &Event, b: &Event) -> bool {
    a.instant.get().to_bits() == b.instant.get().to_bits()
        && a.boundary_deg.to_bits() == b.boundary_deg.to_bits()
        && a.direction == b.direction
        && a.evaluations == b.evaluations
}

/// The strided scan against the fine one over the window, both counting
/// what they asked for.
fn compare(
    source: &impl Longitudes,
    body: Body,
    (from, to): (f64, f64),
) -> Result<(Compared, usize), String> {
    let [first, rest @ ..] = lattices();
    let quantity = Quantity::Longitude(body);
    let window = (
        JulianDay::<Ut1>::literal(from),
        JulianDay::<Ut1>::literal(to),
    );
    let strided = Counting::new(source);
    let search = Search::each(&strided, quantity, first, rest);
    let fine_step = search.step_days();
    // A stride is a small whole number of steps.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a stride is a small whole number of steps"
    )]
    let stride = (search.scan_step_days() / fine_step).round() as usize;
    let found = search
        .between_each(window.0, window.1)
        .map_err(|why| format!("{body:?}'s strided scan: {why}"))?;
    let fine = Counting::new(source);
    let expected = Search::each(&fine, quantity, first, rest)
        .with_step_days(fine_step)
        .between_each(window.0, window.1)
        .map_err(|why| format!("{body:?}'s fine scan: {why}"))?;
    let mut compared = Compared {
        fine_instants: fine.instants(),
        strided_instants: strided.instants(),
        ..Compared::default()
    };
    for (found, expected) in found.iter().zip(&expected) {
        compared.crossings += expected.len();
        let agreeing = found
            .iter()
            .zip(expected)
            .filter(|(a, b)| same(a, b))
            .count();
        compared.differing += found.len().max(expected.len()) - agreeing;
    }
    Ok((compared, stride))
}

/// One body, measured on its own provider.
fn measure(body: Body) -> Result<Measured, String> {
    let provider = Builtin::new();
    let (low, high) = provider.capabilities().jd_range;
    // A day inside either end, so that a sample's speed never reaches past
    // the coverage.
    let window = (low.ceil() + 1.0, high.floor() - 1.0);
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let geocentric = completion.longitudes(Frame::CANONICAL);
    let found = stations(&geocentric, body, window)?;
    let (retrograde, direct) = shortest_runs(&found);
    let table = shortest_run_days(body);
    let (mut measured, mut stride) = (Compared::default(), 1);
    let mut topocentric = Compared::default();
    if table.is_some() {
        (measured, stride) = compare(&geocentric, body, GEOCENTRIC)?;
        for (name, latitude, longitude) in PLACES {
            let place = Place::try_from_degrees(latitude, longitude, 0.0)
                .map_err(|why| format!("{name}: {why}"))?;
            let seen = completion
                .longitudes(Frame::CANONICAL.with_centre(Centre::Topocentric))
                .with_observer(place);
            topocentric.add(compare(&seen, body, TOPOCENTRIC)?.0);
        }
    }
    Ok(Measured {
        body,
        stations: found.len(),
        retrograde,
        direct,
        table,
        stride,
        geocentric: measured,
        topocentric,
    })
}

/// Every body, a thread each, in the catalogue's order.
fn measured() -> Result<Vec<Measured>, String> {
    std::thread::scope(|scope| {
        let running: Vec<_> = BODIES
            .iter()
            .map(|&body| scope.spawn(move || measure(body)))
            .collect();
        running
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| String::from("a body's thread panicked"))?
            })
            .collect()
    })
}

fn days(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.2}")
    } else {
        String::from("—")
    }
}

/// What the pass decides.
fn claims(all: &[Measured]) -> Vec<Claim> {
    let planets: Vec<&Measured> = all
        .iter()
        .filter(|one| one.table.is_some_and(f64::is_finite))
        .collect();
    let under = planets
        .iter()
        .filter(|one| {
            let shortest = one.retrograde.min(one.direct);
            one.table
                .is_none_or(|table| table > shortest || table < TABLE_FLOOR * shortest)
        })
        .count();
    let still: Vec<&Measured> = all
        .iter()
        .filter(|one| NEVER_TURN.contains(&one.body))
        .collect();
    let turned = still.iter().filter(|one| one.stations > 0).count();
    let true_node = all.iter().find(|one| one.body == Body::TrueNode);
    let node_run = true_node.map_or(f64::INFINITY, |one| one.retrograde.min(one.direct));
    let mut geocentric = Compared::default();
    let mut topocentric = Compared::default();
    for one in all {
        geocentric.add(one.geocentric);
        topocentric.add(one.topocentric);
    }
    let unbounded = all
        .iter()
        .filter(|one| one.table.is_some() != (one.body != Body::TrueNode))
        .count();
    vec![
        Claim::counted(
            format!(
                "each planet's table entry lies between {}% and all of the shortest run it made between two stations",
                TABLE_FLOOR * 100.0
            ),
            under,
            planets.len(),
        ),
        Claim::counted(
            "the Sun, the Moon and the mean node never turn",
            turned,
            still.len(),
        ),
        Claim::stated(
            "the true node turns within a day, so it keeps the fine scan",
            verdict_of(node_run < 1.0 && true_node.is_some_and(|one| one.table.is_none())),
            format!("its shortest run is {} days", days(node_run)),
        ),
        Claim::counted(
            "every body but the true node has a table entry, and the true node has none",
            unbounded,
            all.len(),
        ),
        Claim::counted(
            "the strided scan finds every crossing the fine scan finds, to the bit, geocentric",
            geocentric.differing,
            geocentric.crossings,
        )
        .with_note(format!(
            "{} of the fine scan's instants",
            share(geocentric.strided_instants, geocentric.fine_instants)
        )),
        Claim::counted(
            format!(
                "the same holds topocentric, seen from {}",
                PLACES.map(|(name, ..)| name).join(" and ")
            ),
            topocentric.differing,
            topocentric.crossings,
        )
        .with_note(format!(
            "{} of the fine scan's instants",
            share(topocentric.strided_instants, topocentric.fine_instants)
        )),
    ]
}

fn page() -> Result<String, String> {
    let all = measured()?;
    let provider = Builtin::new();
    let (low, high) = provider.capabilities().jd_range;

    let mut out = String::new();
    let _ = writeln!(
        out,
        "# Station runs, measured\n\n\
         Status: `generated` by `cargo xtask stations`. Do not edit:\n\
         `check-stations` regenerates this page and fails on any difference.\n\n\
         The design this measures is `astro-events-and-crossings.md` §4, \"a\n\
         slow body's scan\". A crossing search samples a slow body every\n\
         *stride* steps of its fine grid and reads the grid only where a line\n\
         or a station lies between two samples. That is sound only while no\n\
         two stations fall inside one stride, so the stride is bounded by\n\
         three quarters of the body's shortest run between stations, which\n\
         `shortest_run_days` states as a table. This page holds the table to\n\
         the built-in ephemeris over its whole coverage, JD {low:.1} to {high:.1},\n\
         sampled daily in the canonical frame. It then compares every\n\
         crossing the strided scan finds with the fine scan's, over signs,\n\
         nakshatras and a single point together: from 1900 to 2100 in the\n\
         canonical frame, and from 2000 to 2020 seen from Delhi and from\n\
         Tromsø, where a slow body near its station wobbles by its daily\n\
         parallax.\n",
    );
    let mut rows = String::from(
        "| body | stations | shortest retrograde | shortest direct | table | stride | crossings | instants |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    for one in &all {
        let _ = writeln!(
            rows,
            "| {:?} | {} | {} | {} | {} | {} | {} | {} |",
            one.body,
            count(one.stations),
            days(one.retrograde),
            days(one.direct),
            one.table.map_or_else(|| String::from("none"), days),
            one.stride,
            count(one.geocentric.crossings),
            share(
                one.geocentric.strided_instants,
                one.geocentric.fine_instants
            ),
        );
    }
    let _ = writeln!(
        out,
        "## 1. What the sky does\n\n\
         Runs are in days, from one station to the next, each station placed\n\
         by the speed's straight line between the daily samples either side.\n\
         **instants** is what the strided scan asked the ephemeris for, as a\n\
         share of what the fine scan asked; a body with no table keeps the\n\
         fine scan, and its stride is 1.\n\n{rows}"
    );

    let claims = claims(&all);
    let _ = writeln!(out, "## 2. What this pass decides\n\n{}", table(&claims));
    let falsified = claims
        .iter()
        .filter(|claim| claim.verdict == Verdict::Falsified)
        .count();
    let _ = writeln!(
        out,
        "{} The comparison is bit for bit, not to a tolerance: the strided\n\
         scan refines the same cells of the same anchored grid from the same\n\
         two samples, so any difference would be a crossing it looked past.",
        if falsified == 0 {
            format!("None of the {} claims is falsified.", spelled(claims.len()))
        } else {
            format!(
                "{} of the {} claims falsified.",
                spelled(falsified),
                spelled(claims.len())
            )
        }
    );
    Ok(crate::measure::fill(&out))
}

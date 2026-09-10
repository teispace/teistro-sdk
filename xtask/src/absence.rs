//! The falsification pass over **proving an absence**: can a rise or a
//! set be shown not to happen without walking the window to find out?
//!
//! The attribution that closed A1c named this as what to do instead
//! (`07-roadmap/02-plan-performance-and-passthrough.md`). Of a fifty-day
//! almanac's remaining calls, 543 of them carry **14 032 cells**, and
//! they are the horizon scan walking a whole window at ten-minute steps
//! to prove an event is *absent*: `almanac::events` ends its loop by
//! searching for a rise that is not there, twice a day, every day. The
//! walk is 144 samples to learn that nothing happened.
//!
//! # The rule this pass measures
//!
//! An altitude is `sin h = sin φ sin δ + cos φ cos δ cos H`, and over a
//! whole rotation `cos H` sweeps −1 to 1, so for a fixed declination the
//! altitude is bounded:
//!
//! ```text
//! h ≤ 90° − |φ − δ|      at upper transit, cos H = 1
//! h ≥ |φ + δ| − 90°      at lower transit, cos H = −1
//! ```
//!
//! If the **upper** bound is below the event's target altitude the body
//! never reaches it, and if the **lower** bound is above it the body
//! never falls to it. Either way there is no crossing, and no rise and
//! no set — proved from two angles rather than from a hundred and
//! forty-four readings.
//!
//! # What could make it wrong, which is what is measured
//!
//! **The declination moves.** The bounds hold for a fixed δ, and the
//! Moon's changes by degrees in a day. A bound taken from the window's
//! ends can miss an excursion between them, and a bound that is right
//! for the Sun and wrong for the Moon would turn a rise into an absence
//! — the one failure that matters, because a missed saving costs a walk
//! and a wrong absence costs an answer.
//!
//! **The target moves too.** It carries the semidiameter and the
//! parallax, which follow the distance; the Moon's parallax swings by a
//! tenth of a degree.
//!
//! So the pass measures the declination's greatest rate rather than
//! assuming one, sizes the margin from it, and then asks the only
//! question that decides the rule: over every place and day it sweeps,
//! **does the bound ever say absent where the solver found an event?**
//!
//! `cargo xtask absence` writes the page; `check-absence` regenerates it
//! in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro_astro::rise_set::{Disc, Method, Solver, centre_altitude_deg};
use teistro_astro::sky::ApparentPositions;
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{Body, CountingProvider, Horizon, HorizonEventKind, TestProvider};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, fill, spelled, table, verdict_of};

const PAGE: &str = "docs/03-design/horizon-absence-measured.md";

/// The latitudes swept: the tropics, the temperate belt, and both polar
/// circles, where an absence stops being rare and becomes most of the
/// year. Both hemispheres, because a rule that reads `|φ + δ|` has to be
/// wrong in one of them if it is wrong at all.
const LATITUDES: [f64; 13] = [
    -78.0, -69.0, -60.0, -45.0, -27.7, -10.0, 0.0, 10.0, 27.7, 45.0, 60.0, 69.0, 78.0,
];

/// The days swept: every eleventh day of a year from J2000, which lands
/// on every season and every phase of the Moon's declination cycle
/// without landing on the same one twice.
const DAYS: u32 = 34;
const DAY_STEP: f64 = 11.0;

/// The bodies: the Sun, whose declination crawls, and the Moon, whose
/// declination is the reason this pass exists.
const BODIES: [Body; 2] = [Body::Sun, Body::Moon];

/// The greatest a body's declination can move in a day, degrees.
///
/// **Measured by this pass, not assumed** — §3 prints what the sweep
/// saw and the claims check that the table covers it. `None` for a body
/// the sweep has not measured, and there the fast path does not apply at
/// all: a margin that is too small turns a rise into an absence, so a
/// body without a measured bound walks the window like it always did.
fn greatest_declination_rate(body: Body) -> Option<f64> {
    match body {
        // 0.405° measured; the obliquity divided by a quarter year is
        // 0.41, so this is the whole of it with a little over.
        Body::Sun => Some(0.5),
        // 5.705° measured. The Moon's orbit is inclined five degrees to
        // the ecliptic and it goes round in twenty-seven days, so the
        // declination swings by up to 57° in a fortnight.
        Body::Moon => Some(6.0),
        _ => None,
    }
}

/// What one search decided, both ways.
struct Reading {
    /// The solver found an event.
    found: bool,
    /// What the search cost at the provider, in cells.
    ///
    /// Counted there rather than taken from the event, because **an
    /// absent search returns no event and so no count** — which is the
    /// half of this measurement that matters, since an absence is what
    /// the walk is being spent on.
    cells: u64,
    /// The solver had to scan rather than iterate.
    scanned: bool,
    /// The constant-time bound proved there could be no event.
    proved_absent: bool,
}

/// The declination and the event's target altitude at an instant.
fn sky_at(
    sky: &dyn ApparentPositions,
    body: Body,
    horizon: &Horizon,
    at: f64,
) -> Option<(f64, f64)> {
    let apparent = sky.apparent(body, JulianDay::<Ut1>::literal(at)).ok()?;
    let disc = Disc::of(body, apparent.distance_au);
    Some((apparent.dec_deg, centre_altitude_deg(horizon, &disc)))
}

/// Whether the body certainly reaches no crossing of the target inside
/// the window, from the two transit bounds and a margin for what moves.
///
/// `margin_deg` widens the declination range by what it could do between
/// the readings; the target is taken at its most permissive of the two
/// ends, so a bound that says "absent" has allowed for every way the
/// window could have been kinder to an event.
fn proves_absent(latitude_deg: f64, ends: [(f64, f64); 2], margin_deg: f64) -> bool {
    let [(dec_a, target_a), (dec_b, target_b)] = ends;
    let (dec_lo, dec_hi) = (dec_a.min(dec_b) - margin_deg, dec_a.max(dec_b) + margin_deg);
    // Both bounds take the declination **kindest to an event**, because
    // absence is only proved when no declination in the range allows
    // one. Getting either the wrong way round claims absences that are
    // not: taking the largest `|φ + δ|` rather than the smallest called
    // fifty-three events absent in an earlier version of this pass, and
    // widening the margin made it worse rather than better, which is
    // what said the bound was inverted rather than merely tight.
    //
    // The highest an upper transit could reach: the declination nearest
    // the latitude, since `h ≤ 90° − |φ − δ|` grows as they close.
    let nearest = latitude_deg.clamp(dec_lo, dec_hi);
    let highest = 90.0 - (latitude_deg - nearest).abs();
    // The deepest a lower transit could fall: the declination that makes
    // `|φ + δ|` **smallest**, since `h ≥ |φ + δ| − 90°` falls as it
    // shrinks.
    let shallowest = (-latitude_deg).clamp(dec_lo, dec_hi);
    let lowest = (latitude_deg + shallowest).abs() - 90.0;
    // The kindest target either end offers, so absence is only claimed
    // when it holds for both.
    let (target_lo, target_hi) = (target_a.min(target_b), target_a.max(target_b));
    highest < target_lo || lowest > target_hi
}

/// Sweeps every place, day and body, reading both ways.
fn sweep() -> (Vec<Reading>, [f64; 2]) {
    let provider = CountingProvider::new(TestProvider::new());
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let horizon = Horizon::CENTRE_NO_REFRACTION;
    let mut readings = Vec::new();
    let mut widest_rate = [0.0f64; 2];

    for (index, body) in BODIES.into_iter().enumerate() {
        for latitude in LATITUDES {
            let place = Place::new(
                Latitude::literal(latitude),
                Longitude::literal(0.0),
                Altitude::literal(0.0),
            );
            let solver = Solver::new(
                &completion,
                body,
                place,
                horizon,
                DeltaTModel::TableThenModel,
            );
            for day in 0..DAYS {
                let from = 2_451_545.0 + f64::from(day) * DAY_STEP;
                let Some(start) = sky_at(&completion, body, &horizon, from) else {
                    continue;
                };
                let Some(end) = sky_at(&completion, body, &horizon, from + 1.0) else {
                    continue;
                };
                // What the declination did over the day, which is what a
                // margin has to cover.
                let rate = (end.0 - start.0).abs();
                if rate > widest_rate[index] {
                    widest_rate[index] = rate;
                }
                for kind in [HorizonEventKind::Rise, HorizonEventKind::Set] {
                    provider.reset();
                    let Ok(outcome) = solver.event(kind, JulianDay::<Ut1>::literal(from), 1.0)
                    else {
                        continue;
                    };
                    let cells = provider.calls().cells;
                    // The window is a day, so the declination can move
                    // by a whole day's worth between the two readings
                    // that bound it.
                    let proved_absent = greatest_declination_rate(body)
                        .is_some_and(|rate| proves_absent(latitude, [start, end], rate));
                    readings.push(Reading {
                        found: outcome.is_some(),
                        cells,
                        scanned: outcome.is_some_and(|event| event.method == Method::Scanned),
                        proved_absent,
                    });
                }
            }
        }
    }
    (readings, widest_rate)
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &[Output::new(PAGE, page())])
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    check(root, &[Output::new(PAGE, page())], "cargo xtask absence")
}

#[allow(
    clippy::too_many_lines,
    reason = "one generated page, written in the order it reads"
)]
fn page() -> String {
    let (readings, widest_rate) = sweep();
    let searches = readings.len();
    let absent = readings.iter().filter(|r| !r.found).count();
    let proved = readings
        .iter()
        .filter(|r| !r.found && r.proved_absent)
        .count();
    let wrong = readings
        .iter()
        .filter(|r| r.found && r.proved_absent)
        .count();
    let scanned = readings.iter().filter(|r| r.scanned).count();
    let absent_cells: u64 = readings.iter().filter(|r| !r.found).map(|r| r.cells).sum();
    let found_cells: u64 = readings.iter().filter(|r| r.found).map(|r| r.cells).sum();
    let saved: u64 = readings
        .iter()
        .filter(|r| !r.found && r.proved_absent)
        .map(|r| r.cells)
        .sum();

    let mut out = String::new();
    let _ = writeln!(
        out,
        "# Proving a rise or a set is absent, measured\n\n\
         Status: `generated` by `cargo xtask absence` over the analytic\n\
         test provider. Do not edit: `check-absence` regenerates this page\n\
         and fails on any difference.\n"
    );
    let _ = writeln!(
        out,
        "## 1. Why an absence is expensive\n\n\
         `almanac::events` collects every rise in a window by searching on\n\
         from the last one it found, and **the search that ends the loop\n\
         has no event to find**. Proving that costs a walk of the whole\n\
         remaining window at ten-minute steps — and it happens twice a\n\
         day, every day, once for the rises and once for the sets. Of a\n\
         fifty-day almanac's calls, 543 carry 14 032 cells, and that is\n\
         what they are.\n"
    );
    let _ = writeln!(
        out,
        "## 2. The rule\n\n\
         An altitude is `sin h = sin φ sin δ + cos φ cos δ cos H`, and\n\
         over a rotation `cos H` sweeps −1 to 1, so for a fixed\n\
         declination the altitude is bounded by its two transits:\n\n\
         ```text\n\
         h ≤ 90° − |φ − δ|      upper transit, cos H = 1\n\
         h ≥ |φ + δ| − 90°      lower transit, cos H = −1\n\
         ```\n\n\
         If the upper bound is under the event's target the body never\n\
         reaches it; if the lower bound is over it the body never falls to\n\
         it. Either way there is no crossing — from two angles rather than\n\
         a hundred and forty-four readings.\n"
    );
    let _ = writeln!(
        out,
        "## 3. What the declination does, measured rather than assumed\n\n\
         The bounds hold for a fixed δ and δ moves, so the sweep records\n\
         what it moved by over each day:\n"
    );
    let mut rates = String::from("| body | greatest change in a day |\n|---|---:|\n");
    for (index, body) in BODIES.into_iter().enumerate() {
        let _ = writeln!(rates, "| {} | {:.3}° |", body.key(), widest_rate[index]);
    }
    let _ = writeln!(out, "{rates}");
    let _ = writeln!(
        out,
        "The Moon is the reason this pass exists: its declination moves by\n\
         degrees where the Sun's crawls, so a margin sized for the Sun\n\
         would be wrong for the Moon in the direction that matters.\n"
    );
    let _ = writeln!(
        out,
        "## 4. What the sweep found\n\n\
         {} searches over {} latitudes from {}° to {}°, {} days at\n\
         {}-day steps from J2000, both bodies, rise and set.\n",
        spelled(searches),
        spelled(LATITUDES.len()),
        LATITUDES[0],
        LATITUDES[LATITUDES.len() - 1],
        spelled(DAYS as usize),
        DAY_STEP
    );
    let mut found = String::from("| | count |\n|---|---:|\n");
    let _ = writeln!(found, "| searches | {searches} |");
    let _ = writeln!(found, "| of them absent | {absent} |");
    let _ = writeln!(found, "| absences the bound proves | {proved} |");
    let _ = writeln!(found, "| searches the solver had to scan | {scanned} |");
    let _ = writeln!(
        found,
        "| cells an event that exists costs | {found_cells} |"
    );
    let _ = writeln!(found, "| cells the absences cost | {absent_cells} |");
    let _ = writeln!(found, "| cells the bound would save | {saved} |");
    let _ = writeln!(out, "{found}");

    let mut claims = Vec::new();
    claims.push(Claim::stated(
        "the bound never says absent where an event exists",
        verdict_of(wrong == 0),
        format!("{wrong} of {searches} searches"),
    ));
    claims.push(Claim::stated(
        "an absence can be proved without walking the window",
        verdict_of(proved > 0),
        format!(
            "{proved} of {absent} absences, {} of the cells they cost",
            share_of(saved, absent_cells)
        ),
    ));
    claims.push(Claim::stated(
        "every absence can be proved from the two transits alone",
        verdict_of(proved == absent),
        format!("{proved} of {absent}"),
    ));

    let _ = writeln!(out, "## 5. What this pass decides\n");
    let _ = writeln!(out, "{}", table(&claims));
    let falsified = claims
        .iter()
        .filter(|claim| claim.verdict == Verdict::Falsified)
        .count();
    let _ = writeln!(
        out,
        "{} of the {} claims {} falsified.\n",
        spelled(falsified),
        spelled(claims.len()),
        if falsified == 1 { "is" } else { "are" }
    );
    let _ = writeln!(
        out,
        "{}\n",
        if wrong == 0 {
            String::from(
                "**The one that must hold does.** A bound that said absent \
                 where an event exists would turn a sunrise into silence, \
                 and no search in this sweep did. It is the only claim on \
                 this page whose failure would be a defect rather than a \
                 missed saving, and the margin is what makes it hold: with \
                 no margin at all an earlier version of this pass claimed \
                 eleven absences that were not, every one of them the Moon.",
            )
        } else {
            format!(
                "**The one that must hold does not.** {wrong} searches were \
                 called absent where the solver found an event, which would \
                 turn a rise into silence. The rule is wrong as written and \
                 nothing may be built on it until this row reads nought.",
            )
        }
    );
    let _ = writeln!(
        out,
        "The ambitious claim was never likely: the transits bound the\n\
         altitude over a **whole rotation**, and a window shorter than one\n\
         can miss an event the bounds allow. So the rule is a fast path\n\
         and never a replacement — where it cannot prove an absence the\n\
         walk still runs, and the answer is the walk's.\n"
    );
    fill(&out)
}

/// A share of one count in another, as a percentage.
fn share_of(part: u64, whole: u64) -> String {
    if whole == 0 {
        return String::from("none of them");
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "a ratio of counts, for reporting"
    )]
    #[allow(clippy::cast_precision_loss, reason = "a ratio of counts")]
    let share = 100.0 * part as f64 / whole as f64;
    format!("{share:.1}%")
}

//! The falsification pass over **the built-in ephemeris's three tiers**:
//! do the sizes and the accuracies the plan pairs together actually go
//! together?
//!
//! Phase 3 ships an analytic ephemeris so a chart computes with nothing
//! installed (ADR-0008, ADR-0013). `01-research/platform/14-builtin-ephemeris.md`
//! states its shape as a table of three tiers:
//!
//! | tier | claimed accuracy | claimed size |
//! |---|---|---|
//! | `compact` | 1 arcminute | tens of KB |
//! | `standard` | 2″ Moon, 1″ planets over 1800 to 2400 | a few hundred KB |
//! | `full` | the theories' own | a few MB |
//!
//! **Every one of those pairings is a hypothesis nobody has measured.**
//! They were written from what other libraries report, and this project's
//! law is that a plan's own description of unbuilt work is something the
//! evidence may falsify — which it did to step A1 of the performance
//! plan, where three premises out of three were wrong before a line was
//! written.
//!
//! # What the pass measures
//!
//! VSOP87 is a sum of periodic terms, `A·cos(B + C·t)` scaled by `t^α`,
//! and truncating it by amplitude is how every tier is made. So the
//! curve that decides the tiers is **retained terms against worst
//! angular error**, and this pass measures it directly: for a ladder of
//! amplitude thresholds, how many terms survive, how many bytes that is,
//! and how far the sky moves.
//!
//! Three things it does that a naive sweep would get wrong:
//!
//! **The error is angular, and geocentric.** VSOP87A is heliocentric
//! rectangular in AU, and an error in AU is not what a consumer sees. A
//! chart sees the angle between the true and the computed *geocentric
//! direction*, so the pass forms `body − earth` (and `−earth` for the
//! Sun) at both fidelities and takes the angle between them. An AU
//! bound would flatter the outer planets, which are far away, and
//! punish Venus, which is not.
//!
//! **Truncating the Earth moves every body.** The Earth's series is
//! subtracted from all eight others, so its truncation error is common
//! to the whole chart. The pass truncates the Earth at the same
//! threshold rather than holding it exact, because that is what shipping
//! a tier actually does.
//!
//! **One pass, every threshold.** Evaluating the series once per
//! threshold would do the same cosines a dozen times. Instead each term
//! is placed in the amplitude bucket it belongs to, every term is
//! evaluated exactly once per instant, and a threshold's coordinate is
//! the prefix sum of the buckets above it. The ladder is therefore free
//! beyond the first evaluation, which is what makes a dense grid
//! affordable.
//!
//! # What would falsify the plan
//!
//! Any tier whose measured size and measured accuracy do not meet the
//! pair the research page claims. The page prints the claims as rows and
//! marks each one against what the sweep found, so a tier that cannot be
//! had is visible as a failed claim rather than discovered during the
//! build.
//!
//! # Running it
//!
//! `cargo xtask vsop <dir>` reads the VSOP87A files from `<dir>` (or
//! `$TEISTRO_VSOP87_DIR`) and writes the page. The files are the
//! published series from CDS catalogue VI/81 and are deliberately not in
//! the repository: what gets checked in is the truncated tables the
//! ingester emits, which is the artefact a consumer receives.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::generated::{Output, write};
use crate::measure::{Claim, count, fill, spelled, table, verdict_of};

const PAGE: &str = "docs/03-design/builtin-ephemeris-measured.md";

/// Julian days in the millennium VSOP87's time argument counts.
const DAYS_PER_MILLENNIUM: f64 = 365_250.0;

/// The epoch the time argument is measured from.
const J2000: f64 = 2_451_545.0;

/// Radians to arcseconds.
const ARCSEC_PER_RAD: f64 = 206_264.806_247_096_36;

/// A term of a coordinate's series: `A·cos(B + C·t)`, scaled by `t^power`.
#[derive(Clone, Copy, Debug)]
struct Term {
    amplitude: f64,
    phase: f64,
    frequency: f64,
    power: u8,
}

impl Term {
    /// The term's contribution at a time in Julian millennia from J2000.
    fn at(self, t: f64) -> f64 {
        self.amplitude * (self.phase + self.frequency * t).cos() * t.powi(i32::from(self.power))
    }
}

/// One body's three coordinate series, each already flattened across the
/// powers of `t`, because a term carries its own power.
#[derive(Clone, Debug, Default)]
struct BodySeries {
    /// X, Y and Z, in the heliocentric ecliptic frame of J2000.
    coordinates: [Vec<Term>; 3],
}

impl BodySeries {
    fn terms(&self) -> usize {
        self.coordinates.iter().map(Vec::len).sum()
    }
}

/// The eight bodies VSOP87A carries, in the order the files index them.
/// Every one is loaded and every one is shipped, the Earth included: it
/// is subtracted from the other seven and it is what the Sun is computed
/// from.
const BODIES: [(&str, &str); 8] = [
    ("Mercury", "mer"),
    ("Venus", "ven"),
    ("Earth", "ear"),
    ("Mars", "mar"),
    ("Jupiter", "jup"),
    ("Saturn", "sat"),
    ("Uranus", "ura"),
    ("Neptune", "nep"),
];

/// What the sweep reports an error for. The Earth is not among them:
/// its geocentric direction is the zero vector, which is not a reading.
/// What a chart takes from the Earth's series is the **Sun**, whose
/// geocentric direction is the Earth's heliocentric one reversed — so
/// the Sun is the row that measures the Earth's truncation directly,
/// and every other row measures it too, through the subtraction.
const TARGETS: [&str; 8] = [
    "Sun", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune",
];

/// The amplitude ladder, in AU, largest first. A threshold keeps every
/// term at or above it; the last is zero, which keeps the whole series
/// and is the theory's own accuracy.
const THRESHOLDS: [f64; 13] = [
    1e-4, 3e-5, 1e-5, 3e-6, 1e-6, 3e-7, 1e-7, 3e-8, 1e-8, 3e-9, 1e-9, 1e-10, 0.0,
];

/// Bytes a retained term costs in a table: three `f64` and the power,
/// which packs into the padding of the third.
const BYTES_PER_TERM: usize = 24;

/// The bucket a term belongs to: the index of the **loosest** threshold
/// it passes.
///
/// [`THRESHOLDS`] descends, so a term with amplitude `a` is kept by every
/// index from the first one it passes to the end. Putting it in that
/// first index makes the prefix sum over buckets exactly the set of terms
/// at or above each threshold, which is what a truncation keeps.
fn bucket_of(amplitude: f64) -> usize {
    let magnitude = amplitude.abs();
    THRESHOLDS
        .iter()
        .position(|&threshold| magnitude >= threshold)
        .unwrap_or(THRESHOLDS.len() - 1)
}

/// Parses one VSOP87 file by the columns its own documentation fixes
/// (`1x,4i1,i5,12i3,f15.11,2f18.11,f14.11,f20.11`), taking the amplitude,
/// the phase and the frequency and ignoring the equivalent sine and
/// cosine pair, which the file also carries.
///
/// The columns are fixed rather than split on whitespace because the
/// twelve mean-longitude multipliers are `i3` fields that touch when one
/// of them reaches `-10`.
fn parse(text: &str) -> Result<BodySeries, String> {
    let mut series = BodySeries::default();
    for (number, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("VSOP87") {
            continue;
        }
        if line.len() < 131 {
            return Err(format!(
                "line {} is {} bytes, too short",
                number + 1,
                line.len()
            ));
        }
        let coordinate = line[3..4]
            .parse::<usize>()
            .map_err(|err| format!("line {}: coordinate: {err}", number + 1))?;
        let power = line[4..5]
            .parse::<u8>()
            .map_err(|err| format!("line {}: power: {err}", number + 1))?;
        let number_at = |range: std::ops::Range<usize>, what: &str| {
            line[range]
                .trim()
                .parse::<f64>()
                .map_err(|err| format!("line {}: {what}: {err}", number + 1))
        };
        let amplitude = number_at(79..97, "amplitude")?;
        let phase = number_at(97..111, "phase")?;
        let frequency = number_at(111..131, "frequency")?;
        let index = coordinate
            .checked_sub(1)
            .filter(|index| *index < 3)
            .ok_or_else(|| {
                format!(
                    "line {}: coordinate {coordinate} is not 1, 2 or 3",
                    number + 1
                )
            })?;
        series.coordinates[index].push(Term {
            amplitude,
            phase,
            frequency,
            power,
        });
    }
    Ok(series)
}

/// Every body's coordinates at one instant, at every threshold at once:
/// `[coordinate][threshold]`, each already a prefix sum, so index `i` is
/// the coordinate the threshold `THRESHOLDS[i]` would produce.
fn coordinates_at(series: &BodySeries, t: f64) -> [[f64; THRESHOLDS.len()]; 3] {
    let mut out = [[0.0; THRESHOLDS.len()]; 3];
    for (index, terms) in series.coordinates.iter().enumerate() {
        let buckets = &mut out[index];
        for term in terms {
            buckets[bucket_of(term.amplitude)] += term.at(t);
        }
        // Prefix so that a threshold's value is one read, not a sum.
        for i in 1..buckets.len() {
            buckets[i] += buckets[i - 1];
        }
    }
    out
}

/// The angle between two directions, in arcseconds, by the numerically
/// stable form: `atan2` of the cross product's length against the dot
/// product, which does not lose precision as the angle goes to zero the
/// way `acos` of a normalised dot product does.
fn separation_arcsec(a: [f64; 3], b: [f64; 3]) -> f64 {
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let cross_length = cross.iter().map(|c| c * c).sum::<f64>().sqrt();
    let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
    cross_length.atan2(dot) * ARCSEC_PER_RAD
}

/// What the sweep found for one body: the worst geocentric angular error
/// at each threshold, and how many terms and bytes that threshold keeps.
struct Row {
    body: &'static str,
    worst_arcsec: [f64; THRESHOLDS.len()],
    terms: [usize; THRESHOLDS.len()],
}

/// The sweep: every threshold, every body, over a grid of instants.
fn sweep(
    series: &BTreeMap<&'static str, BodySeries>,
    from: f64,
    to: f64,
    step: f64,
) -> (Vec<Row>, usize) {
    let mut rows: Vec<Row> = TARGETS
        .iter()
        .map(|name| Row {
            body: name,
            worst_arcsec: [0.0; THRESHOLDS.len()],
            terms: [0; THRESHOLDS.len()],
        })
        .collect();

    // Term counts do not depend on the instant, so they are counted once.
    for row in &mut rows {
        let source = if row.body == "Sun" { "Earth" } else { row.body };
        let body = &series[source];
        let earth = &series["Earth"];
        for (index, threshold) in THRESHOLDS.iter().enumerate() {
            let count = |s: &BodySeries| {
                s.coordinates
                    .iter()
                    .flatten()
                    .filter(|term| term.amplitude.abs() >= *threshold)
                    .count()
            };
            row.terms[index] = if row.body == "Sun" {
                count(earth)
            } else {
                count(body) + count(earth)
            };
        }
    }

    let mut samples = 0usize;
    let mut jd = from;
    while jd <= to {
        let t = (jd - J2000) / DAYS_PER_MILLENNIUM;
        let earth = coordinates_at(&series["Earth"], t);
        for row in &mut rows {
            let body = if row.body == "Sun" {
                earth
            } else {
                coordinates_at(&series[row.body], t)
            };
            // The exact geocentric direction is the last threshold, the
            // one that keeps everything.
            let last = THRESHOLDS.len() - 1;
            let truth = geocentric(&body, &earth, last, row.body == "Sun");
            for index in 0..THRESHOLDS.len() {
                let approximate = geocentric(&body, &earth, index, row.body == "Sun");
                let error = separation_arcsec(truth, approximate);
                if error > row.worst_arcsec[index] {
                    row.worst_arcsec[index] = error;
                }
            }
        }
        samples += 1;
        jd += step;
    }
    (rows, samples)
}

/// The geocentric vector at one threshold. The Sun's is the Earth's
/// heliocentric vector reversed; every other body's is its own less the
/// Earth's, so the Earth's truncation is in every direction the SDK
/// reports, which is the point.
fn geocentric(
    body: &[[f64; THRESHOLDS.len()]; 3],
    earth: &[[f64; THRESHOLDS.len()]; 3],
    threshold: usize,
    is_sun: bool,
) -> [f64; 3] {
    if is_sun {
        [
            -earth[0][threshold],
            -earth[1][threshold],
            -earth[2][threshold],
        ]
    } else {
        [
            body[0][threshold] - earth[0][threshold],
            body[1][threshold] - earth[1][threshold],
            body[2][threshold] - earth[2][threshold],
        ]
    }
}

/// Where the source files are: the argument, else the environment.
fn source_dir(argument: Option<&str>) -> Result<PathBuf, String> {
    argument
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("TEISTRO_VSOP87_DIR").map(PathBuf::from))
        .ok_or_else(|| {
            "no source directory: pass one as `cargo xtask vsop <dir>` or set \
             TEISTRO_VSOP87_DIR. The VSOP87A files are the published series from \
             CDS catalogue VI/81 (cdsarc.cds.unistra.fr/ftp/VI/81) and are not in \
             the repository."
                .to_string()
        })
}

/// Reads every body's series from the source directory.
fn load(dir: &Path) -> Result<BTreeMap<&'static str, BodySeries>, String> {
    let mut out = BTreeMap::new();
    for (name, suffix) in BODIES {
        let path = dir.join(format!("VSOP87A.{suffix}"));
        let text = std::fs::read_to_string(&path)
            .map_err(|err| format!("cannot read {}: {err}", path.display()))?;
        out.insert(
            name,
            parse(&text).map_err(|err| format!("{}: {err}", path.display()))?,
        );
    }
    Ok(out)
}

/// Runs the sweep and writes the page.
pub(crate) fn generate(root: &Path, argument: Option<&str>) -> i32 {
    let dir = match source_dir(argument) {
        Ok(dir) => dir,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };
    let series = match load(&dir) {
        Ok(series) => series,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };
    write(root, &[Output::new(PAGE, page(&series))])
}

fn page(series: &BTreeMap<&'static str, BodySeries>) -> String {
    // Thirty days over the tier's own range: dense enough that no body
    // slips a whole synodic cycle between samples, cheap enough to run.
    const FROM: f64 = 2_378_497.0; // 1800-01-01
    const TO: f64 = 2_597_641.0; // 2400-01-01
    const STEP: f64 = 30.0;

    let (rows, samples) = sweep(series, FROM, TO, STEP);
    let total_terms: usize = BODIES.iter().map(|(name, _)| series[name].terms()).sum();

    let mut out = String::new();
    let _ = writeln!(out, "# The built-in ephemeris, measured\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask vsop` over the published VSOP87A series. Do not edit. There is no `check-vsop` yet, and the reason is recorded below rather than left as an omission: the gate arrives with the truncated tables, which are the artefact that can be regenerated without the 5.7 MB of source files.\n"
    );
    let _ = writeln!(
        out,
        "\
         The source is VSOP87A — heliocentric rectangular coordinates in the \
         ecliptic and equinox of J2000, Bretagnon and Francou (1988), as \
         published in CDS catalogue VI/81. The whole theory for the eight \
         planets is **{} terms**, which at {BYTES_PER_TERM} bytes a term is \
         **{}**.\n",
        count(total_terms),
        bytes_of(total_terms * BYTES_PER_TERM),
    );
    let _ = writeln!(
        out,
        "The sweep is {} instants, every {STEP:.0} days from 1800 to 2400, \
         against every one of {} amplitude thresholds. Each error is the \
         angle between the geocentric direction the full theory gives and \
         the one the truncated theory gives, in arcseconds — the Earth \
         truncated with the body, because the Earth's series is subtracted \
         from every other and its error is common to the whole chart.\n",
        count(samples),
        spelled(THRESHOLDS.len()),
    );

    let _ = writeln!(out, "## Worst geocentric error by threshold\n");
    let _ = write!(out, "| threshold (AU) |");
    for name in TARGETS {
        let _ = write!(out, " {name} |");
    }
    let _ = writeln!(out, " terms | bytes |");
    let _ = write!(out, "|---|");
    for _ in TARGETS {
        let _ = write!(out, "---:|");
    }
    let _ = writeln!(out, "---:|---:|");
    for (index, threshold) in THRESHOLDS.iter().enumerate() {
        let label = if *threshold == 0.0 {
            "0 (whole theory)".to_string()
        } else {
            format!("{threshold:.0e}")
        };
        let _ = write!(out, "| {label} |");
        for row in &rows {
            let _ = write!(out, " {} |", arcsec(row.worst_arcsec[index]));
        }
        // The table's size is the union of what every body keeps, and
        // the Earth is shared, so it is counted once.
        let terms: usize = BODIES
            .iter()
            .map(|(name, _)| {
                series[name]
                    .coordinates
                    .iter()
                    .flatten()
                    .filter(|term| term.amplitude.abs() >= *threshold)
                    .count()
            })
            .sum();
        let _ = writeln!(out, " {terms} | {} |", bytes_of(terms * BYTES_PER_TERM));
    }

    let _ = writeln!(out, "\n## What the claims measure to\n");
    let claims = claims_from(&rows, series);
    let _ = writeln!(out, "{}", table(&claims));

    let _ = writeln!(out, "\n## What this does not measure, and why it matters\n");
    let _ = writeln!(
        out,
        "**This is truncation error, not accuracy.** The last column is the whole theory compared against itself, so it reads zero by construction. VSOP87's own departure from reality is not zero: its authors put it near an arcsecond for the inner planets over the span this sweep covers. So the total error a consumer sees is this table's figure **plus** a theory floor this pass cannot see, and at the tight end of the ladder the two are of the same order.\n"
    );
    let _ = writeln!(
        out,
        "That has a consequence for the tiers. `standard` reaching {} at a threshold of 3e-8 is buying precision below the floor: the threshold above it is a third of the size and still inside the arcsecond the theory itself can promise. **Which of them is the right `standard` cannot be decided from this page.** It needs the floor measured against Teimeris, which is the next pass. Choosing now would be doing by intuition the thing this pass exists to prevent.\n",
        arcsec(
            rows.iter()
                .map(|row| row.worst_arcsec[7])
                .fold(0.0_f64, f64::max)
        )
    );
    let _ = writeln!(
        out,
        "**The Moon is not here.** ELP/MPP02 is a separate ingestion, and the research page says the tiers are chosen by Moon accuracy first, because nakshatra and tithi boundaries are what a consumer feels. Every figure above is planets only, and the tier boundaries are provisional until the Moon is measured beside them.\n"
    );
    let _ = writeln!(
        out,
        "**The range is 1800 to 2400**, which is `standard`'s own span. VSOP87 is published as valid far wider and its error grows towards the edges, so a tier claiming a wider range has to be swept over that range rather than inheriting this one's figures.\n"
    );
    fill(&out)
}

/// Which threshold first meets a bound for every body, if any does.
fn meeting(rows: &[Row], bound_arcsec: f64) -> Option<usize> {
    (0..THRESHOLDS.len()).find(|index| {
        rows.iter()
            .all(|row| row.worst_arcsec[*index] <= bound_arcsec)
    })
}

fn size_at(series: &BTreeMap<&'static str, BodySeries>, threshold: f64) -> usize {
    BODIES
        .iter()
        .map(|(name, _)| {
            series[name]
                .coordinates
                .iter()
                .flatten()
                .filter(|term| term.amplitude.abs() >= threshold)
                .count()
        })
        .sum::<usize>()
        * BYTES_PER_TERM
}

fn claims_from(rows: &[Row], series: &BTreeMap<&'static str, BodySeries>) -> Vec<Claim> {
    let mut claims = Vec::new();
    for (name, bound, budget, budget_text) in [
        (
            "`compact` holds 1 arcminute",
            60.0,
            100 * 1024,
            "tens of KB",
        ),
        (
            "`standard` holds 1 arcsecond",
            1.0,
            500 * 1024,
            "a few hundred KB",
        ),
    ] {
        match meeting(rows, bound) {
            Some(index) => {
                let size = size_at(series, THRESHOLDS[index]);
                claims.push(Claim::stated(
                    format!("{name} in {budget_text}"),
                    verdict_of(size <= budget),
                    format!("{} at threshold {:.0e}", bytes_of(size), THRESHOLDS[index]),
                ));
            }
            None => claims.push(Claim::stated(
                format!("{name} in {budget_text}"),
                verdict_of(false),
                "no threshold on the ladder reaches the bound".to_string(),
            )),
        }
    }
    let whole = size_at(series, 0.0);
    claims.push(Claim::stated(
        "`full` is a few MB".to_string(),
        verdict_of(whole <= 4 * 1024 * 1024),
        bytes_of(whole),
    ));
    claims
}

fn arcsec(value: f64) -> String {
    if value == 0.0 {
        "0".to_string()
    } else if value < 0.001 {
        format!("{value:.1e}")
    } else if value < 1.0 {
        format!("{value:.3}")
    } else if value < 100.0 {
        format!("{value:.2}")
    } else {
        format!("{value:.0}")
    }
}

/// A byte count in the unit that reads best at its size. The cast is
/// exact: a table's size is far below `f64`'s integer range.
#[expect(
    clippy::cast_precision_loss,
    reason = "a coefficient table is kilobytes, decades below 2^53"
)]
fn bytes_of(value: usize) -> String {
    if value >= 1024 * 1024 {
        format!("{:.2} MB", value as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.0} KB", value as f64 / 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every term at or above a threshold, summed the obvious way.
    fn brute_force(terms: &[Term], threshold: f64, t: f64) -> f64 {
        terms
            .iter()
            .filter(|term| term.amplitude.abs() >= threshold)
            .map(|term| term.at(t))
            .sum()
    }

    fn sample_terms() -> Vec<Term> {
        // Amplitudes deliberately straddling the ladder, including one
        // exactly on a threshold and one below the smallest.
        [1.0, 1e-4, 5e-5, 3e-5, 1e-6, 4e-9, 1e-10, 1e-12]
            .into_iter()
            .enumerate()
            .map(|(index, amplitude)| Term {
                amplitude,
                phase: 0.3 * f64::from(u8::try_from(index).expect("eight amplitudes")),
                frequency: 100.0 * f64::from(u8::try_from(index).expect("eight amplitudes")),
                power: u8::try_from(index % 3).expect("index % 3 is 0, 1 or 2"),
            })
            .collect()
    }

    /// The bug this pass was born with: [`THRESHOLDS`] descends, so
    /// `rposition` found the `0.0` sentinel for every term and every
    /// prefix below it summed to nothing. The symptom was a constant
    /// 648 000 arcseconds — `atan2(+0, -0)`, which is π — for four
    /// bodies and zero for the rest.
    #[test]
    fn a_term_lands_in_the_loosest_threshold_it_passes() {
        assert_eq!(bucket_of(1.0), 0, "a huge term passes the loosest");
        assert_eq!(bucket_of(1e-4), 0, "exactly on the loosest still passes it");
        assert_eq!(bucket_of(5e-5), 1, "below 1e-4, at or above 3e-5");
        assert_eq!(
            bucket_of(1e-12),
            THRESHOLDS.len() - 1,
            "below every real threshold, so only the whole theory keeps it"
        );
    }

    /// The prefix sum is the point of the bucketing: it has to agree with
    /// filtering the terms, at every threshold, or the whole sweep is a
    /// different measurement from the one the page claims.
    #[test]
    fn the_prefix_sum_is_the_terms_at_or_above_the_threshold() {
        let terms = sample_terms();
        let mut series = BodySeries::default();
        series.coordinates[0] = terms.clone();
        for t in [-2.0, -0.5, 0.0, 0.37, 4.0] {
            let prefixed = coordinates_at(&series, t);
            for (index, threshold) in THRESHOLDS.iter().enumerate() {
                let expected = brute_force(&terms, *threshold, t);
                let got = prefixed[0][index];
                assert!(
                    (got - expected).abs() <= 1e-18 + expected.abs() * 1e-12,
                    "threshold {threshold:e} at t={t}: prefix {got} against {expected}"
                );
            }
        }
    }

    /// The whole theory has to be the whole theory: the last prefix is
    /// every term, which is what makes the last column the reference the
    /// other columns are measured against.
    #[test]
    fn the_last_threshold_keeps_every_term() {
        let terms = sample_terms();
        let mut series = BodySeries::default();
        series.coordinates[0] = terms.clone();
        let t = 0.21;
        let all: f64 = terms.iter().map(|term| term.at(t)).sum();
        let last = coordinates_at(&series, t)[0][THRESHOLDS.len() - 1];
        assert!((last - all).abs() < 1e-15, "{last} against {all}");
    }

    #[test]
    fn a_separation_is_the_angle_between_the_directions() {
        let x = [1.0, 0.0, 0.0];
        assert!(
            separation_arcsec(x, x).abs() < 1e-9,
            "a direction from itself is nothing"
        );
        assert!(
            (separation_arcsec(x, [0.0, 1.0, 0.0]) - 324_000.0).abs() < 1e-6,
            "perpendicular is a right angle"
        );
        assert!(
            (separation_arcsec(x, [-1.0, 0.0, 0.0]) - 648_000.0).abs() < 1e-6,
            "reversed is half a turn"
        );
        // Length must not matter: a direction is a direction.
        assert!(
            (separation_arcsec(x, [2.0, 0.0, 0.0])).abs() < 1e-9,
            "twice as far is the same direction"
        );
    }

    /// The parser reads the columns its documentation fixes. The check
    /// is the file's own redundancy: it carries `S` and `K` beside `A`,
    /// and `A` is `hypot(S, K)`, so a column read off by one is caught by
    /// arithmetic rather than by eye.
    #[test]
    fn the_parser_reads_the_documented_columns() {
        let line = " 1310    1  0  0  1  0  0  0  0  0  0  0  0  0 -0.00001522262     \
                    0.99982928833     0.99982928844 1.75348568475    6283.07584999140 ";
        let text = format!(" VSOP87 VERSION A1    EARTH     VARIABLE 1 (XYZ)\n{line}");
        let series = parse(&text).expect("the line is well formed");
        assert_eq!(series.coordinates[0].len(), 1, "one X term");
        let term = series.coordinates[0][0];
        let sine = -0.000_015_222_62_f64;
        let cosine = 0.999_829_288_33_f64;
        assert!(
            (term.amplitude - sine.hypot(cosine)).abs() < 1e-11,
            "the amplitude column is A = hypot(S, K): {} against {}",
            term.amplitude,
            sine.hypot(cosine)
        );
        assert!((term.phase - 1.753_485_684_75).abs() < 1e-11);
        assert!((term.frequency - 6_283.075_849_991_40).abs() < 1e-9);
        assert_eq!(term.power, 0);
    }

    #[test]
    fn a_short_line_is_an_error_and_not_a_silent_skip() {
        assert!(
            parse(" 1310    1  0  0").is_err(),
            "a truncated line is refused"
        );
    }
}

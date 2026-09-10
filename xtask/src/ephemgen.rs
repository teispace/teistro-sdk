//! `ephemgen`: the generator that turns the published series into the
//! tables the built-in ephemeris ships.
//!
//! Phase 3's measurements are done and they chose the shape (ADR-0027).
//! This is the build step: read VSOP87A and ELP2000-82B, truncate each
//! to what a tier needs, and emit Rust tables with their provenance
//! embedded.
//!
//! # The thresholds are derived, not chosen
//!
//! A generator that hardcoded `3e-8` would be carrying a number whose
//! reason lived in a page it never reads, and the two would drift. So
//! the rule is stated here and the numbers fall out of it:
//!
//! **For each tier, take the loosest threshold on the ladder whose worst
//! truncation error is inside the tier's target**, and never tighten
//! past the point where the theory's own error dominates — a term whose
//! removal moves the answer by a tenth of what the theory is already
//! wrong by is a term that costs bytes and buys nothing.
//!
//! The targets are ADR-0027's, per body group, because the single
//! numbers the plan began with were false of the Moon and of the outer
//! planets alike.
//!
//! # Why Rust literals and not a blob
//!
//! `include_bytes!` would be smaller in the repository and quicker to
//! compile, and it would need `f64::from_le_bytes` on every access or a
//! transmute. The workspace forbids `unsafe_code`, and the evaluator is
//! the hot path of every chart, so the tables are `&'static [Term]`
//! literals: no decode, no endianness question, and a diff that a
//! reviewer can read.
//!
//! `cargo xtask ephemgen <vsop-dir> <elp-dir>` writes the tables.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use teistro_ephemeris_builtin::elp::ingest::{Theory, load as load_elp};
use teistro_ephemeris_builtin::ingest::{BODIES, BodySeries, load as load_vsop};

use crate::generated::{Output, write};

/// Where the fitted bija is recorded.
const BIJA_SOURCE: &str = "crates/ephemeris-builtin/data/elp82b-floor.json";

/// Where the tables go.
const TABLES: &str = "crates/ephemeris-builtin/src/tables";

/// The amplitude ladder for the planets, in astronomical units.
const VSOP_LADDER: [f64; 8] = [1e-5, 3e-6, 1e-6, 3e-7, 1e-7, 3e-8, 1e-8, 0.0];

/// The amplitude ladder for the Moon, in the coordinate's own unit —
/// arcseconds for longitude and latitude, kilometres for distance, which
/// is the knob the published reader itself takes.
const ELP_LADDER: [f64; 7] = [1.0, 0.3, 0.1, 0.03, 0.01, 0.001, 0.0];

/// One tier: what it targets, per body group.
struct Tier {
    /// The cargo feature and module name.
    name: &'static str,
    /// What the tier's coefficient data may cost, in bytes.
    ///
    /// A budget is a claim, and a claim nothing reads is a claim that
    /// drifts. The generator emits it beside the measured size and the
    /// crate's own tests hold the one to the other, so a tier that grew
    /// fails rather than ships.
    budget_bytes: usize,
    /// What a planet's truncation may cost, in arcseconds.
    planet_target_arcsec: f64,
    /// What the Moon's truncation may cost, in arcseconds.
    moon_target_arcsec: f64,
    /// What the tier is for, in one line, for the generated header.
    purpose: &'static str,
}

/// The three tiers of ADR-0008, with ADR-0027's targets.
const TIERS: [Tier; 3] = [
    Tier {
        name: "compact",
        budget_bytes: 128 * 1024,
        planet_target_arcsec: 60.0,
        moon_target_arcsec: 30.0,
        purpose: "wasm and mobile, where every kilobyte is paid for; one arcminute",
    },
    Tier {
        name: "standard",
        budget_bytes: 640 * 1024,
        planet_target_arcsec: 1.0,
        moon_target_arcsec: 1.0,
        purpose: "the default: production charts with no provider installed",
    },
    Tier {
        name: "full",
        budget_bytes: 4 * 1024 * 1024,
        planet_target_arcsec: 0.0,
        moon_target_arcsec: 0.0,
        purpose: "research, and the second oracle for the astronomy layer's own tests",
    },
];

/// Radians to arcseconds.
const ARCSEC_PER_RAD: f64 = 206_264.806_247_096_36;

/// The worst geocentric angular error each planet threshold costs, for
/// the whole ladder at once, measured against the whole theory exactly
/// as the sweep does.
///
/// Hoisted for the same reason as the Moon's: the exact positions are
/// the same for every rung.
fn planet_errors(series: &BTreeMap<&'static str, BodySeries>, ladder: &[f64]) -> Vec<f64> {
    const FROM: f64 = 2_378_497.0;
    const TO: f64 = 2_597_641.0;
    const STEP: f64 = 40.0;
    let earth = &series["Earth"];
    let mut worst = vec![0.0_f64; ladder.len()];
    let mut jd = FROM;
    while jd <= TO {
        let t = teistro_ephemeris_builtin::series::millennia(jd);
        let earth_exact = earth.at(t, 0.0);
        for (slot, threshold) in worst.iter_mut().zip(ladder) {
            let earth_cut = earth.at(t, *threshold);
            for (name, _) in BODIES {
                if name == "Earth" {
                    *slot = slot.max(separation(
                        [-earth_exact[0], -earth_exact[1], -earth_exact[2]],
                        [-earth_cut[0], -earth_cut[1], -earth_cut[2]],
                    ));
                    continue;
                }
                let body = &series[name];
                let exact = body.at(t, 0.0);
                let cut = body.at(t, *threshold);
                *slot = slot.max(separation(
                    [
                        exact[0] - earth_exact[0],
                        exact[1] - earth_exact[1],
                        exact[2] - earth_exact[2],
                    ],
                    [
                        cut[0] - earth_cut[0],
                        cut[1] - earth_cut[1],
                        cut[2] - earth_cut[2],
                    ],
                ));
            }
        }
        jd += STEP;
    }
    worst
}

/// The worst angular error each Moon threshold costs against the whole
/// theory, for the whole ladder at once.
///
/// The exact position does not depend on the threshold, so computing it
/// per rung meant evaluating thirty-eight thousand terms at every
/// instant seven times over to produce the same vector each time. Once
/// per instant instead, which is the same mistake and the same fix as
/// hoisting the Earth out of the body loop.
fn moon_errors(theory: &Theory, ladder: &[f64]) -> Vec<f64> {
    use teistro_ephemeris_builtin::elp::ingest::position;
    const FROM: f64 = 2_378_497.0;
    const TO: f64 = 2_597_641.0;
    const STEP: f64 = 7.0;
    let mut worst = vec![0.0_f64; ladder.len()];
    let mut jd = FROM;
    while jd <= TO {
        let exact = position(theory, jd, 0.0);
        for (slot, threshold) in worst.iter_mut().zip(ladder) {
            *slot = slot.max(separation(exact, position(theory, jd, *threshold)));
        }
        jd += STEP;
    }
    worst
}

/// The angle between two vectors, in arcseconds.
fn separation(a: [f64; 3], b: [f64; 3]) -> f64 {
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let length = cross.iter().map(|c| c * c).sum::<f64>().sqrt();
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    length.atan2(dot) * ARCSEC_PER_RAD
}

/// A ladder with each rung's measured error, computed once.
///
/// The error of a rung does not depend on which tier is asking, and
/// measuring it inside the tier loop cost three passes over the whole
/// series for every rung to produce the same numbers each time.
struct Ladder {
    rungs: Vec<(f64, f64)>,
}

impl Ladder {
    /// A ladder from thresholds and the errors already measured for them.
    fn of(thresholds: &[f64], errors: &[f64]) -> Ladder {
        Ladder {
            rungs: thresholds
                .iter()
                .copied()
                .zip(errors.iter().copied())
                .collect(),
        }
    }

    /// The loosest rung whose error is inside the target.
    ///
    /// A target of zero means the whole theory, which is the last rung —
    /// and for the Moon that rung provably buys nothing over the one
    /// before it. It is shipped at `full` anyway, because `full` means
    /// the theory as its authors published it and a tier that quietly
    /// dropped terms would be a different claim.
    fn choose(&self, target: f64) -> (f64, f64) {
        let last = self.rungs.last().copied().unwrap_or((0.0, 0.0));
        if target <= 0.0 {
            return last;
        }
        self.rungs
            .iter()
            .find(|(_, measured)| *measured <= target)
            .copied()
            .unwrap_or(last)
    }
}

/// Renders one `Term` literal.
fn term_literal(amplitude: f64, phase: f64, frequency: f64, power: u8) -> String {
    format!("t({amplitude:?},{phase:?},{frequency:?},{power})")
}

/// A slice of `i8` as a Rust array literal.
fn multipliers(values: &[i8]) -> String {
    let inner: Vec<String> = values.iter().map(i8::to_string).collect();
    format!("[{}]", inner.join(","))
}

/// A slice of `f64` as a Rust array literal, round-tripping exactly.
fn coefficients(values: &[f64]) -> String {
    let inner: Vec<String> = values.iter().map(|value| format!("{value:?}")).collect();
    format!("[{}]", inner.join(","))
}

/// The planets' table for one tier.
fn planet_table(series: &BTreeMap<&'static str, BodySeries>, threshold: f64) -> (String, usize) {
    let mut out = String::new();
    let mut terms = 0usize;
    let _ = writeln!(
        out,
        "/// VSOP87A, truncated at {threshold:e} astronomical units.\n\
         ///\n\
         /// Heliocentric rectangular coordinates in the ecliptic and equinox\n\
         /// of J2000, from Bretagnon and Francou (1988), CDS catalogue VI/81.\n\
         /// Generated by `cargo xtask ephemgen`; do not edit.\n\
         ///\n\
         /// `static` rather than `const`: a `const` array is a value that is\n\
         /// copied into every place it is used, and this one has thousands of\n\
         /// terms in it.\n\
         pub static PLANETS: Planets = ["
    );
    for (name, _) in BODIES {
        let body = &series[name];
        let _ = writeln!(out, "    (\"{name}\", [");
        for coordinate in &body.coordinates {
            let kept: Vec<String> = coordinate
                .iter()
                .filter(|term| term.amplitude.abs() >= threshold)
                .map(|term| term_literal(term.amplitude, term.phase, term.frequency, term.power))
                .collect();
            terms += kept.len();
            let _ = writeln!(out, "        &[{}],", kept.join(","));
        }
        let _ = writeln!(out, "    ]),");
    }
    let _ = writeln!(out, "];");
    (out, terms)
}

/// The Moon's table for one tier.
fn moon_table(theory: &Theory, threshold: f64) -> (String, usize) {
    let mut out = String::new();
    let main: Vec<String> = theory
        .main
        .iter()
        .filter(|(_, term)| {
            term.coefficients
                .first()
                .is_some_and(|leading| leading.abs() >= threshold)
        })
        .map(|(file, term)| {
            format!(
                "({file},MainTerm::new({},{}))",
                multipliers(&term.multipliers),
                coefficients(&term.coefficients)
            )
        })
        .collect();
    let perturbations: Vec<String> = theory
        .perturbations
        .iter()
        .filter(|(_, term)| term.amplitude >= threshold)
        .map(|(file, term)| {
            format!(
                "({file},PerturbationTerm::new({},{:?},{:?}))",
                multipliers(&term.multipliers),
                term.phase_deg,
                term.amplitude
            )
        })
        .collect();
    let terms = main.len() + perturbations.len();
    let _ = writeln!(
        out,
        "/// ELP2000-82B, truncated at {threshold} in each coordinate's own\n\
         /// unit — arcseconds for longitude and latitude, kilometres for\n\
         /// distance — which is the knob the published reader itself takes.\n\
         ///\n\
         /// Chapront-Touze and Chapront (1988), CDS catalogue VI/79.\n\
         /// Generated by `cargo xtask ephemgen`; do not edit.\n\
         /// `static` rather than `const`, so the table exists once.\n\
         pub static MOON_MAIN: [(u8, MainTerm); {}] = [{}];\n",
        main.len(),
        main.join(",")
    );
    let _ = writeln!(
        out,
        "/// The Moon's perturbation terms at the same threshold.\n\
         pub static MOON_PERTURBATIONS: [(u8, PerturbationTerm); {}] = [{}];",
        perturbations.len(),
        perturbations.join(",")
    );
    (out, terms)
}

/// What a tier is, as constants rather than as prose in a header.
///
/// A header that says a tier holds 0.62 arcseconds is a claim, and this
/// repository has four lints because a claim nothing reads is a claim
/// that drifts. These are read: the crate's tests hold the term counts
/// to the arrays' lengths and the size to the budget, and a consumer can
/// ask what the tier it compiled promises rather than looking it up in a
/// document that may have moved on.
#[expect(
    clippy::too_many_arguments,
    reason = "one tier's measured facts, which travel together"
)]
fn facts_block(
    tier: &str,
    planet_threshold: f64,
    planet_error: f64,
    planet_terms: usize,
    moon_threshold: f64,
    moon_error: f64,
    moon_terms: usize,
    budget_bytes: usize,
) -> String {
    let bytes = planet_terms * 32 + moon_terms * 32;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "/// Which tier these tables are.\n\
         pub const TIER_NAME: &str = \"{tier}\";\n\
         \n\
         /// The amplitude below which a planetary term was dropped, in\n\
         /// astronomical units. Zero keeps the whole theory.\n\
         pub const PLANET_THRESHOLD_AU: f64 = {planet_threshold:?};\n\
         \n\
         /// The worst the truncation costs a planet's geocentric direction\n\
         /// over 1800 to 2400, in arcseconds, measured by the generator.\n\
         ///\n\
         /// This is what the **table** costs, not what a chart is wrong by:\n\
         /// the theory's own error against a modern ephemeris is separate\n\
         /// and larger, and is published in\n\
         /// `03-design/builtin-ephemeris-measured.md`.\n\
         pub const PLANET_TRUNCATION_ARCSEC: f64 = {planet_error:?};\n\
         \n\
         /// How many planetary terms this tier keeps.\n\
         pub const PLANET_TERMS: usize = {planet_terms};\n\
         \n\
         /// The amplitude below which a lunar term was dropped, in each\n\
         /// coordinate's own unit.\n\
         pub const MOON_THRESHOLD: f64 = {moon_threshold:?};\n\
         \n\
         /// The worst the truncation costs the Moon's direction over 1800\n\
         /// to 2400, in arcseconds.\n\
         pub const MOON_TRUNCATION_ARCSEC: f64 = {moon_error:?};\n\
         \n\
         /// How many lunar terms this tier keeps, of both kinds.\n\
         pub const MOON_TERMS: usize = {moon_terms};\n\
         \n\
         /// What the coefficients cost in a binary: a planetary term is\n\
         /// three `f64` and a power, and a lunar one its multipliers and\n\
         /// two `f64`.\n\
         pub const DATA_BYTES: usize = {bytes};\n\
         \n\
         /// What this tier is allowed to cost. The crate's tests hold\n\
         /// [`DATA_BYTES`] to it, so a tier that grew past its budget\n\
         /// fails rather than ships.\n\
         pub const BUDGET_BYTES: usize = {budget_bytes};"
    );
    out
}

/// The bija: the correction ADR-0027 fixed, with the provenance that
/// makes it inspectable rather than a silent adjustment.
///
/// It is read from the recorded measurement rather than restated here,
/// so the coefficients that ship and the coefficients that were measured
/// cannot drift apart.
fn bija_block(root: &Path) -> String {
    let Some(text) = std::fs::read_to_string(root.join(BIJA_SOURCE)).ok() else {
        return String::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return String::new();
    };
    let Some(correction) = value
        .get("corrections")
        .and_then(serde_json::Value::as_array)
        .and_then(|all| {
            all.iter()
                .find(|c| c.get("degree").and_then(serde_json::Value::as_u64) == Some(2))
        })
    else {
        return String::new();
    };
    let number = |key: &str| correction.get(key).and_then(serde_json::Value::as_f64);
    let coefficients: Vec<f64> = correction
        .get("coefficients_arcsec")
        .and_then(serde_json::Value::as_array)
        .map(|all| all.iter().filter_map(serde_json::Value::as_f64).collect())
        .unwrap_or_default();
    if coefficients.len() != 3 {
        return String::new();
    }
    let fitted_over = correction
        .get("fitted_over")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("an unrecorded span");
    let residual = number("worst_after_arcsec").unwrap_or(f64::NAN);
    let wide: Vec<String> = correction
        .get("out_of_sample")
        .and_then(serde_json::Value::as_array)
        .map(|all| {
            all.iter()
                .filter_map(|span| {
                    Some(format!(
                        "{} to {:.2} arcsec",
                        span.get("label")?.as_str()?,
                        span.get("worst_after_arcsec")?.as_f64()?
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "/// The bija: a quadratic correction to the Moon's mean longitude,\n\
         /// in arcseconds, against Julian centuries from J2000.\n\
         ///\n\
         /// ELP2000-82B's constants are fitted to DE200/LE200, and a lunar\n\
         /// theory ages through the tidal acceleration its source ephemeris\n\
         /// assumed — a term in the square of the time. These three numbers\n\
         /// remove it. The tradition has the device and the name already\n\
         /// (ADR-0027).\n\
         ///\n\
         /// **Provenance.** Fitted over {fitted_over} against the reference\n\
         /// named in the manifest, and scored on spans it never saw:\n\
         /// {residual:.3} arcsec where fitted, {}. Refitting is a\n\
         /// calculation-version change under ADR-0020. A consumer who wants\n\
         /// the theory as its authors published it turns it off.\n\
         pub const MOON_BIJA_ARCSEC: [f64; 3] = [{:?}, {:?}, {:?}];\n",
        if wide.is_empty() {
            "not scored outside it".to_string()
        } else {
            wide.join(" and ")
        },
        coefficients[0],
        coefficients[1],
        coefficients[2]
    );
    let _ = writeln!(
        out,
        "/// The span the bija was fitted over, for provenance.\n\
         pub const MOON_BIJA_FITTED_OVER: &str = \"{fitted_over}\";"
    );
    out
}

/// Positions this tier's tables must reproduce, computed here from the
/// truncated series and checked in the crate's own tests.
///
/// A generator that mangled a literal — a sign, a digit, an exponent —
/// would produce a table that still compiles and still looks like a
/// table. These are what catch that, and they are computed from the
/// source rather than from the emitted text, so a test that passes says
/// the emitted text means what the source meant.
fn checkpoints(
    series: &BTreeMap<&'static str, BodySeries>,
    theory: &Theory,
    planet_threshold: f64,
    moon_threshold: f64,
) -> String {
    use teistro_ephemeris_builtin::elp::ingest::position;
    use teistro_ephemeris_builtin::series::millennia;

    const INSTANTS: [f64; 3] = [2_378_497.0, 2_451_545.0, 2_597_641.0];
    let mut out = String::new();
    let _ = writeln!(
        out,
        "/// Instants and the values this tier's tables must reproduce:\n\
         /// the Julian day, then Mars and the Earth heliocentrically in\n\
         /// astronomical units, then the Moon geocentrically in kilometres.\n\
         ///\n\
         /// Computed by the generator from the truncated series, so a test\n\
         /// that reproduces them says the emitted literals mean what the\n\
         /// published series meant.\n\
         pub static CHECKPOINTS: [Checkpoint; {}] = [",
        INSTANTS.len()
    );
    for jd in INSTANTS {
        let t = millennia(jd);
        let mars = series["Mars"].at(t, planet_threshold);
        let earth = series["Earth"].at(t, planet_threshold);
        let moon = position(theory, jd, moon_threshold);
        let _ = writeln!(
            out,
            "    ({jd:?}, {}, {}, {}),",
            coefficients(&mars),
            coefficients(&earth),
            coefficients(&moon)
        );
    }
    let _ = writeln!(out, "];");
    out
}

/// What one tier came out as, for the manifest.
struct Emitted {
    tier: &'static str,
    planet_threshold: f64,
    planet_error: f64,
    planet_terms: usize,
    moon_threshold: f64,
    moon_error: f64,
    moon_terms: usize,
    bytes: usize,
    source_bytes: usize,
    budget_bytes: usize,
}

/// Every tier's table and the facts about it.
///
/// Lifted out so the entry point reads as the steps it is — find the
/// sources, load them, measure the ladders once, emit the tiers, write
/// the manifest — rather than as one function with a loop in the middle
/// of it.
fn tiers(
    root: &Path,
    series: &BTreeMap<&'static str, BodySeries>,
    theory: &Theory,
    planets_ladder: &Ladder,
    moon_ladder: &Ladder,
) -> (Vec<Output>, Vec<Emitted>) {
    let mut outputs = Vec::new();
    let mut emitted = Vec::new();
    for tier in TIERS {
        let (planet_threshold, planet_error_value) =
            planets_ladder.choose(tier.planet_target_arcsec);
        let (moon_threshold, moon_error_value) = moon_ladder.choose(tier.moon_target_arcsec);
        let bija = bija_block(root);
        let planets_label = if planet_threshold == 0.0 {
            "the whole theory".to_string()
        } else {
            format!("threshold {planet_threshold:e} au")
        };
        let moon_label = if moon_threshold == 0.0 {
            "the whole theory".to_string()
        } else {
            format!("threshold {moon_threshold}")
        };
        let (planets, planet_terms) = planet_table(series, planet_threshold);
        let (moon, moon_terms) = moon_table(theory, moon_threshold);
        let facts = facts_block(
            tier.name,
            planet_threshold,
            planet_error_value,
            planet_terms,
            moon_threshold,
            moon_error_value,
            moon_terms,
            tier.budget_bytes,
        );
        let checkpoints = checkpoints(series, theory, planet_threshold, moon_threshold);
        let text = format!(
            "//! The `{}` tier: {}.\n\
             //!\n\
             //! Generated by `cargo xtask ephemgen`. Do not edit.\n\
             //!\n\
             //! The thresholds are not chosen here. `ephemgen` takes the\n\
             //! loosest rung of each ladder whose worst truncation error is\n\
             //! inside this tier's target, so the number below is a\n\
             //! consequence of the target and the measurement rather than a\n\
             //! constant somebody picked.\n\
             //!\n\
             //! - planets: {planets_label}, worst {planet_error_value:.4} arcsec, {planet_terms} terms\n\
             //! - Moon: {moon_label}, worst {moon_error_value:.4} arcsec, {moon_terms} terms\n\
             \n\
             #![allow(\n    \
             clippy::unreadable_literal,\n    \
             clippy::excessive_precision,\n    \
             clippy::approx_constant,\n    \
             reason = \"generated tables: the digits are the publication's\"\n\
             )]\n\
             \n\
             use crate::elp::{{MainTerm, PerturbationTerm}};\n\
             use crate::series::Term;\n\
             \n\
             /// Every body's three coordinate series, by name.\n\
             pub type Planets = [(&'static str, [&'static [Term]; 3]); 8];\n\
             \n\
             /// An instant and what this tier's tables must give at it:\n\
             /// Mars and the Earth heliocentrically, then the Moon.\n\
             pub type Checkpoint = (f64, [f64; 3], [f64; 3], [f64; 3]);\n\
             \n\
             /// A planetary term, abbreviated so the table stays readable.\n\
             const fn t(amplitude: f64, phase: f64, frequency: f64, power: u8) -> Term {{\n    \
             Term::new(amplitude, phase, frequency, power)\n\
             }}\n\
             \n{facts}\n{planets}\n{moon}\n{bija}\n{checkpoints}\n",
            tier.name, tier.purpose
        );
        // The size that matters is what the data costs in a binary, not
        // what the source costs in the repository: a term is three
        // `f64` and a power for a planet, and eleven multipliers with
        // two `f64` for the Moon.
        let bytes = planet_terms * 32 + moon_terms * 32;
        let source_bytes = text.len();
        outputs.push(Output::new(format!("{TABLES}/{}.rs", tier.name), text));
        emitted.push(Emitted {
            tier: tier.name,
            planet_threshold,
            planet_error: planet_error_value,
            planet_terms,
            moon_threshold,
            moon_error: moon_error_value,
            moon_terms,
            bytes,
            source_bytes,
            budget_bytes: tier.budget_bytes,
        });
    }

    (outputs, emitted)
}

/// Reads both sources and writes every tier's table and the manifest.
pub(crate) fn generate(root: &Path, vsop: Option<&str>, elp: Option<&str>) -> i32 {
    let Some(vsop_dir) = vsop
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("TEISTRO_VSOP87_DIR").map(PathBuf::from))
    else {
        eprintln!("usage: cargo xtask ephemgen <vsop87-dir> <elp-dir>");
        return 1;
    };
    let Some(elp_dir) = elp
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("TEISTRO_ELP_DIR").map(PathBuf::from))
    else {
        eprintln!("usage: cargo xtask ephemgen <vsop87-dir> <elp-dir>");
        return 1;
    };
    let series = match load_vsop(&vsop_dir) {
        Ok(series) => series,
        Err(error) => {
            eprintln!("{error}");
            return 1;
        }
    };
    let theory = match load_elp(&elp_dir) {
        Ok(theory) => theory,
        Err(error) => {
            eprintln!("{error}");
            return 1;
        }
    };

    let planets_ladder = Ladder::of(&VSOP_LADDER, &planet_errors(&series, &VSOP_LADDER));
    let moon_ladder = Ladder::of(&ELP_LADDER, &moon_errors(&theory, &ELP_LADDER));

    let (mut outputs, emitted) = tiers(root, &series, &theory, &planets_ladder, &moon_ladder);
    outputs.push(Output::new(format!("{TABLES}/mod.rs"), module(&emitted)));
    write(root, &outputs)
}

/// A byte count as kilobytes; a table is far below the range an `f64`
/// represents exactly.
#[expect(
    clippy::cast_precision_loss,
    reason = "a table's size is kilobytes, decades below 2^53"
)]
fn kilobytes(bytes: usize) -> f64 {
    bytes as f64 / 1024.0
}

/// The module that selects a tier by cargo feature.
fn module(emitted: &[Emitted]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "//! The generated coefficient tables, one module per tier.\n\
         //!\n\
         //! Generated by `cargo xtask ephemgen`. Do not edit.\n\
         //!\n\
         //! A tier is a cargo feature, and exactly one is in the binary: a\n\
         //! consumer who takes `compact` pays for `compact`, which is what\n\
         //! makes the tier mean anything on a platform that counts\n\
         //! kilobytes.\n\
         //!\n\
         //! | tier | planets | Moon | data of budget | source |\n\
         //! |---|---|---|---:|---:|"
    );
    for tier in emitted {
        let _ = writeln!(
            out,
            "//! | `{}` | {}, {:.4}″, {} terms | {}, {:.4}″, {} terms | {:.0} of {:.0} KB | {:.0} KB |",
            tier.tier,
            if tier.planet_threshold == 0.0 {
                "whole theory".to_string()
            } else {
                format!("{:e} au", tier.planet_threshold)
            },
            tier.planet_error,
            tier.planet_terms,
            if tier.moon_threshold == 0.0 {
                "whole theory".to_string()
            } else {
                format!("{}", tier.moon_threshold)
            },
            tier.moon_error,
            tier.moon_terms,
            kilobytes(tier.bytes),
            kilobytes(tier.budget_bytes),
            kilobytes(tier.source_bytes)
        );
    }
    let _ = writeln!(out);
    // Cargo features are additive: anything that depends on this crate
    // can turn one on and nothing can turn one off, so the tiers cannot
    // be mutually exclusive without breaking a consumer who takes two
    // dependencies that each want a different one. The richest tier
    // enabled wins instead, which makes every combination build and
    // makes `--all-features` mean `full` rather than an error.
    let names: Vec<&str> = emitted.iter().map(|tier| tier.tier).collect();
    for (index, tier) in emitted.iter().enumerate() {
        let richer: Vec<String> = names
            .iter()
            .skip(index + 1)
            .map(|name| format!("not(feature = \"{name}\")"))
            .collect();
        let condition = if richer.is_empty() {
            format!("feature = \"{}\"", tier.tier)
        } else {
            format!("all(feature = \"{}\", {})", tier.tier, richer.join(", "))
        };
        // `rustfmt` would otherwise rewrite tens of thousands of
        // literals one to a line, which is how a generated file in this
        // repository has been silently reformatted before. The skip goes
        // on the module declaration, as `crates/calendar` does it.
        let _ = writeln!(out, "#[cfg({condition})]");
        let _ = writeln!(out, "#[rustfmt::skip]");
        let _ = writeln!(out, "mod {};", tier.tier);
        let _ = writeln!(out, "#[cfg({condition})]");
        let _ = writeln!(out, "pub use {}::*;", tier.tier);
        let _ = writeln!(out);
    }
    let _ = writeln!(
        out,
        "/// The tier these tables came from, for provenance: a chart says\n\
         /// which tier computed it, and a cached one says which tier it was\n\
         /// cached from (ADR-0020).\n\
         pub const TIER: &str = {{\n    \
         #[cfg(feature = \"full\")]\n    \
         {{ \"full\" }}\n    \
         #[cfg(all(feature = \"standard\", not(feature = \"full\")))]\n    \
         {{ \"standard\" }}\n    \
         #[cfg(all(feature = \"compact\", not(feature = \"standard\"), not(feature = \"full\")))]\n    \
         {{ \"compact\" }}\n\
         }};"
    );
    out
}

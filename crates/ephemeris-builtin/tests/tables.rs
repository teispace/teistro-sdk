//! **The generated tables mean what the published series meant.**
//!
//! `cargo xtask ephemgen` emits the coefficient tables from VSOP87A and
//! ELP2000-82B. A generator that mangled a literal — a sign, a digit, an
//! exponent, a term dropped from the middle of an array — would produce
//! a table that still compiles and still looks like a table, and nothing
//! about its shape would say otherwise.
//!
//! So the generator also computes, **from the source series**, the
//! positions its tables must reproduce, and emits them beside the
//! tables. This evaluates the tables at those instants and compares. It
//! needs no source files, so it runs in CI where the 8 MB of published
//! text is not.
//!
//! The tolerance is tight on purpose. This is not an accuracy test —
//! accuracy is measured against an engine, in
//! `03-design/builtin-ephemeris-measured.md` and
//! `03-design/lunar-accuracy-measured.md`. This asks only whether two
//! evaluations of the same coefficients agree.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::explicit_iter_loop,
    reason = "a test fails by panicking and indexes its own fixtures"
)]

use teistro_ephemeris_builtin::elp::position_from;
use teistro_ephemeris_builtin::series::{Term, millennia, sum};
use teistro_ephemeris_builtin::tables::{
    BUDGET_BYTES, CHECKPOINTS, DATA_BYTES, MOON_BIJA_ARCSEC, MOON_BIJA_FITTED_OVER, MOON_MAIN,
    MOON_PERTURBATIONS, MOON_TERMS, MOON_THRESHOLD, MOON_TRUNCATION_ARCSEC, PLANET_TERMS,
    PLANET_THRESHOLD_AU, PLANET_TRUNCATION_ARCSEC, PLANETS, TIER_NAME,
};

/// One body's three coordinates from the table, at a time in millennia.
fn planet_at(name: &str, t: f64) -> [f64; 3] {
    let (_, coordinates) = PLANETS
        .iter()
        .find(|(body, _)| *body == name)
        .unwrap_or_else(|| panic!("{name} is in the table"));
    let mut out = [0.0; 3];
    for (slot, terms) in out.iter_mut().zip(coordinates) {
        *slot = sum(terms, t);
    }
    out
}

/// Relative agreement, or absolute where the value is near zero.
fn agrees(got: f64, want: f64, tolerance: f64) -> bool {
    let scale = want.abs().max(1.0);
    (got - want).abs() <= tolerance * scale
}

#[test]
fn the_planet_tables_reproduce_the_generators_checkpoints() {
    for (jd, mars, earth, _) in &CHECKPOINTS {
        let t = millennia(*jd);
        for (name, want) in [("Mars", mars), ("Earth", earth)] {
            let got = planet_at(name, t);
            for (got, want) in got.iter().zip(want) {
                assert!(
                    agrees(*got, *want, 1e-14),
                    "{name} at {jd}: {got} against {want}"
                );
            }
        }
    }
}

#[test]
fn the_moon_table_reproduces_the_generators_checkpoints() {
    for (jd, _, _, moon) in &CHECKPOINTS {
        let got = position_from(&MOON_MAIN, &MOON_PERTURBATIONS, *jd);
        for (got, want) in got.iter().zip(moon) {
            assert!(
                agrees(*got, *want, 1e-12),
                "the Moon at {jd}: {got} against {want}"
            );
        }
    }
}

/// A table that is present but empty would pass every comparison above
/// by computing nothing, so the shape is asserted as well as the values.
#[test]
fn every_body_is_present_and_carries_terms() {
    assert_eq!(PLANETS.len(), 8, "eight bodies, the Earth among them");
    for (name, coordinates) in &PLANETS {
        for (index, terms) in coordinates.iter().enumerate() {
            assert!(
                !terms.is_empty(),
                "{name} coordinate {index} kept no terms at all"
            );
        }
    }
    assert!(!MOON_MAIN.is_empty(), "the Moon's main problem");
    assert!(!MOON_PERTURBATIONS.is_empty(), "the Moon's perturbations");
    assert_eq!(CHECKPOINTS.len(), 3);
}

/// Every term must be finite, and every field inside the range its
/// meaning allows. A digit lost from an exponent shows up here rather
/// than as a wrong chart.
#[test]
fn every_coefficient_is_finite_and_in_range() {
    for (name, coordinates) in &PLANETS {
        for terms in coordinates {
            for term in terms.iter() {
                let Term {
                    amplitude,
                    phase,
                    frequency,
                    power,
                } = *term;
                assert!(
                    amplitude.is_finite() && phase.is_finite() && frequency.is_finite(),
                    "{name} has a term that is not finite"
                );
                assert!(
                    power <= 5,
                    "{name} has a power of {power}; VSOP87 uses 0 to 5"
                );
                assert!(
                    phase.abs() <= 7.0,
                    "{name} has a phase of {phase} radians, outside a turn"
                );
            }
        }
    }
    // By reference. `for x in MOON_MAIN` copies the whole static onto
    // the stack, which at the `full` tier is thirty-eight thousand terms
    // and about two megabytes — it overflowed, which is how this was
    // found. A large `static` is a thing to borrow, never to move.
    for (file, term) in &MOON_MAIN {
        assert!(
            (1..=3).contains(file),
            "a main-problem file is 1 to 3, not {file}"
        );
        assert!(
            term.coefficients.iter().all(|c| c.is_finite()),
            "ELP{file} has a coefficient that is not finite"
        );
    }
    for (file, term) in &MOON_PERTURBATIONS {
        assert!(
            (4..=36).contains(file),
            "a perturbation file is 4 to 36, not {file}"
        );
        assert!(
            term.amplitude.is_finite() && term.phase_deg.is_finite(),
            "ELP{file} has a term that is not finite"
        );
        assert!(
            (0.0..=360.0).contains(&term.phase_deg),
            "ELP{file} has a phase of {} degrees",
            term.phase_deg
        );
    }
}

/// **The manifest's claims are read.**
///
/// The generated header says how many terms a tier keeps, what the
/// truncation costs, and what the data weighs. This repository has four
/// lints because a claim nothing reads is a claim that drifts, and a
/// table hand-edited after generation would leave every one of those
/// numbers describing a file that no longer exists.
#[test]
fn the_term_counts_are_the_arrays_lengths() {
    let planets: usize = PLANETS
        .iter()
        .map(|(_, coordinates)| coordinates.iter().map(|terms| terms.len()).sum::<usize>())
        .sum();
    assert_eq!(
        planets, PLANET_TERMS,
        "the header claims {PLANET_TERMS} planetary terms and the tables hold {planets}"
    );
    let moon = MOON_MAIN.len() + MOON_PERTURBATIONS.len();
    assert_eq!(
        moon, MOON_TERMS,
        "the header claims {MOON_TERMS} lunar terms and the tables hold {moon}"
    );
}

/// The size claims are **compile-time** assertions, not tests.
///
/// Everything they compare is a constant, so there is no reason to wait
/// for a test run to learn that a tier is over its budget: a consumer who
/// chose `compact` because kilobytes are what they have should be told by
/// the compiler, at the moment the tables are wrong, and not by a suite
/// they might not run.
///
/// The floor is asserted beside the ceiling. A budget nothing approaches
/// measures nothing, and a tier that suddenly cost a tenth of its
/// allowance would mean the tables had been emptied rather than that
/// somebody had been clever.
const _: () = assert!(
    DATA_BYTES == (PLANET_TERMS + MOON_TERMS) * 32,
    "the size claim must be the arithmetic it stands for"
);
const _: () = assert!(DATA_BYTES <= BUDGET_BYTES, "the tier is over its budget");
const _: () = assert!(
    DATA_BYTES * 4 > BUDGET_BYTES,
    "the budget is too loose to catch anything, or the tables are empty"
);

/// The truncation figures are the generator's own measurement, and they
/// have to be the kind of number they claim to be: a threshold that keeps
/// everything costs nothing, and a looser threshold cannot cost less.
#[expect(
    clippy::assertions_on_constants,
    reason = "the figures are constants and that is the point: the check is \
              that the generator's claims about them hold"
)]
#[test]
fn the_truncation_figures_are_consistent_with_the_thresholds() {
    if PLANET_THRESHOLD_AU == 0.0 {
        assert!(
            PLANET_TRUNCATION_ARCSEC < 1e-9,
            "keeping every term cannot cost {PLANET_TRUNCATION_ARCSEC} arcseconds"
        );
    } else {
        assert!(
            PLANET_TRUNCATION_ARCSEC > 0.0,
            "dropping terms cannot be free"
        );
    }
    if MOON_THRESHOLD == 0.0 {
        assert!(MOON_TRUNCATION_ARCSEC < 1e-9);
    } else {
        assert!(MOON_TRUNCATION_ARCSEC > 0.0);
    }
    assert!(
        PLANET_TRUNCATION_ARCSEC.is_finite() && MOON_TRUNCATION_ARCSEC.is_finite(),
        "a measurement that is not a number is not a measurement"
    );
}

/// The tier names itself, and names one of the three. A tier that said
/// it was something else would take its name into every chart's
/// provenance (ADR-0020).
#[test]
fn the_tier_names_itself_and_names_a_real_one() {
    assert!(
        matches!(TIER_NAME, "compact" | "standard" | "full"),
        "a tier calls itself {TIER_NAME}, which is not one of the three"
    );
}

/// The bija must be the quadratic ADR-0027 chose, and it must carry the
/// span it was fitted over. A correction without its provenance is the
/// silent adjustment the decision exists to avoid.
#[test]
fn the_bija_is_a_quadratic_that_says_where_it_came_from() {
    assert_eq!(
        MOON_BIJA_ARCSEC.len(),
        3,
        "a quadratic has three coefficients"
    );
    assert!(
        MOON_BIJA_ARCSEC.iter().all(|c| c.is_finite()),
        "every coefficient is a number"
    );
    assert!(
        MOON_BIJA_ARCSEC.iter().any(|c| *c != 0.0),
        "a bija of nothing would be a correction that does not correct"
    );
    assert!(
        !MOON_BIJA_FITTED_OVER.is_empty(),
        "the span it was fitted over travels with it"
    );
}

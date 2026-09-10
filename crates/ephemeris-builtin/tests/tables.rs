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
use teistro_ephemeris_builtin::tables::{CHECKPOINTS, MOON_MAIN, MOON_PERTURBATIONS, PLANETS};

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
    for (jd, mars, earth, _) in CHECKPOINTS {
        let t = millennia(jd);
        for (name, want) in [("Mars", mars), ("Earth", earth)] {
            let got = planet_at(name, t);
            for (got, want) in got.iter().zip(want) {
                assert!(
                    agrees(*got, want, 1e-14),
                    "{name} at {jd}: {got} against {want}"
                );
            }
        }
    }
}

#[test]
fn the_moon_table_reproduces_the_generators_checkpoints() {
    for (jd, _, _, moon) in CHECKPOINTS {
        let got = position_from(&MOON_MAIN, &MOON_PERTURBATIONS, jd);
        for (got, want) in got.iter().zip(moon) {
            assert!(
                agrees(*got, want, 1e-12),
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
    for (name, coordinates) in PLANETS {
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
    for (name, coordinates) in PLANETS {
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
    for (file, term) in MOON_MAIN {
        assert!(
            (1..=3).contains(&file),
            "a main-problem file is 1 to 3, not {file}"
        );
        assert!(
            term.coefficients.iter().all(|c| c.is_finite()),
            "ELP{file} has a coefficient that is not finite"
        );
    }
    for (file, term) in MOON_PERTURBATIONS {
        assert!(
            (4..=36).contains(&file),
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

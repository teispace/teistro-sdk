//! Every cell of every chart, and the invariants the design asks for.
//!
//! A varga's test space is enumerable — twelve signs by N parts — which
//! is why `03-design/varga-kernel.md` was written before the dasha
//! kernels: there is no seed, no balance and no recursion to search. So
//! nothing here samples. Every assertion runs over the whole table of
//! every shipped chart, and over every arbitrary D-N the crate will
//! answer for.
//!
//! The eight invariants the design lists, in its own order:
//!
//! 1. the spans fill a sign exactly;
//! 2. a chart has one group per classifier group;
//! 3. every cell names a sign;
//! 4. the rule and its materialised table agree;
//! 5. D1 is the identity;
//! 6. vargottama is detectable uniformly;
//! 7. a cyclic chart's cells cover the twelve signs evenly;
//! 8. the part index is integer arithmetic, and a boundary is decided.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index their own tables and print counts under --nocapture"
)]

use teistro_core::angle::Nas;
use teistro_core::catalogue::{Rashi, Varga};
use teistro_core::quantity::Degrees;
use teistro_vargas::scheme::{Map, Spans, part_of, target};
use teistro_vargas::{MOST_DIVISIONS, SCHEMES, Scheme, place, table};

fn at(degrees: f64) -> Nas {
    Nas::from_degrees(Degrees::try_new(degrees.rem_euclid(360.0)).expect("a finite angle"))
}

/// Every chart the crate answers for: the twenty-one shipped, and a
/// sample of the arbitrary ones that reaches both ends of the range.
fn every_scheme() -> Vec<Scheme> {
    let mut schemes: Vec<Scheme> = SCHEMES.to_vec();
    for divisions in [1_u16, 2, 7, 37, 81, 108, 144, 249, MOST_DIVISIONS] {
        schemes.push(Scheme::cyclic(divisions).expect("inside the limit"));
    }
    schemes
}

#[test]
fn the_spans_fill_a_sign_and_the_groups_match_the_classifier() {
    for scheme in every_scheme() {
        assert_eq!(
            scheme.groups.len(),
            usize::from(scheme.classifier.groups()),
            "{}",
            scheme.key()
        );
        for group in scheme.groups {
            if let Spans::Degrees(widths) = group.spans {
                let total: u16 = widths.iter().map(|width| u16::from(*width)).sum();
                assert_eq!(total, 30, "{}: the spans fill a sign", scheme.key());
                assert!(
                    widths.iter().all(|width| *width > 0),
                    "{}: no part is empty",
                    scheme.key()
                );
            }
            if let Map::Listed(signs) = group.map {
                assert_eq!(
                    u16::try_from(signs.len()).unwrap(),
                    group.parts(scheme.divisions),
                    "{}: one sign per part",
                    scheme.key()
                );
            }
        }
    }
}

#[test]
fn every_cell_of_every_chart_names_a_sign_and_the_table_agrees_with_the_rule() {
    let mut cells = 0_usize;
    for scheme in every_scheme() {
        let rows = table(&scheme);
        assert_eq!(rows.len(), 12, "{}", scheme.key());
        for rashi in Rashi::ALL {
            let group = scheme.group(rashi);
            let parts = scheme.parts(rashi);
            assert_eq!(
                u16::try_from(rows[rashi as usize].len()).unwrap(),
                parts,
                "{} {rashi:?}",
                scheme.key()
            );
            for part in 0..parts {
                let from_rule = target(group, rashi, part, scheme.divisions)
                    .unwrap_or_else(|| panic!("{} {rashi:?} part {part}", scheme.key()));
                assert_eq!(
                    from_rule,
                    rows[rashi as usize][usize::from(part)],
                    "{} {rashi:?} part {part}: the rule and its table",
                    scheme.key()
                );
                cells += 1;
            }
        }
    }
    println!("{cells} cells generated and checked against their rules");
    // Twelve signs by the parts of every chart: the shipped rows' parts
    // sum to 465 (D30 counting five, not thirty) and the sampled
    // arbitrary ones to 929.
    assert_eq!(cells, 12 * (465 + 929), "{cells}");
}

#[test]
fn the_rashi_chart_is_the_identity_everywhere() {
    let rashi = Scheme::of(Varga::D1);
    // Every tenth of a degree of the whole circle.
    for tenth in 0..3600 {
        let longitude = at(f64::from(tenth) / 10.0);
        let placement = place(&rashi, longitude);
        assert_eq!(placement.sign, placement.rashi);
        assert_eq!(placement.part, 0);
        assert!(placement.keeps_its_sign());
    }
}

#[test]
fn vargottama_is_detectable_in_every_chart_and_is_never_all_of_them() {
    // The general fact — a chart leaving a body in the sign it was in —
    // is answerable for every chart, and no chart but the rashi leaves
    // every body where it was.
    for scheme in every_scheme() {
        let mut keeping = 0;
        let mut total = 0;
        for rashi in Rashi::ALL {
            for part in 0..scheme.parts(rashi) {
                let target =
                    target(scheme.group(rashi), rashi, part, scheme.divisions).expect("a sign");
                total += 1;
                keeping += usize::from(target == rashi);
            }
        }
        assert!(keeping > 0, "{}: no cell keeps its sign", scheme.key());
        if scheme.divisions > 1 {
            assert!(
                keeping < total,
                "{}: every cell keeps its sign, which only D1 does",
                scheme.key()
            );
        }
    }
}

#[test]
fn a_cyclic_charts_cells_cover_the_twelve_signs_evenly() {
    // The defining property of parivritti: the 12·N cells are shared out
    // equally, so no sign is favoured by the convention itself.
    for divisions in [1_u16, 5, 9, 37, 108, 144, MOST_DIVISIONS] {
        let scheme = Scheme::cyclic(divisions).expect("inside the limit");
        let mut seen = [0_u32; 12];
        for rashi in Rashi::ALL {
            for part in 0..scheme.parts(rashi) {
                let target =
                    target(scheme.group(rashi), rashi, part, scheme.divisions).expect("a sign");
                seen[target as usize] += 1;
            }
        }
        let each = u32::from(divisions);
        assert!(
            seen.iter().all(|count| *count == each),
            "D{divisions}: {seen:?}"
        );
    }
    // The navamsha is the cyclic form of nine, so it has the property too.
    let navamsha = Scheme::of(Varga::D9);
    let mut seen = [0_u32; 12];
    for rashi in Rashi::ALL {
        for part in 0..9 {
            seen[target(navamsha.group(rashi), rashi, part, 9).expect("a sign") as usize] += 1;
        }
    }
    assert!(seen.iter().all(|count| *count == 9), "{seen:?}");
}

#[test]
fn a_part_boundary_is_decided_and_not_a_coin_toss() {
    // The engine divides in floating point, where 30/7, 30/11, 30/27 and
    // 0.2 are unrepresentable and a body exactly on a boundary can land
    // either side depending on the platform. The integer path cannot: a
    // boundary belongs to the part it opens, on every chart.
    for scheme in every_scheme() {
        let rashi = Rashi::Aries;
        let group = scheme.group(rashi);
        let parts = scheme.parts(rashi);
        for part in 1..parts {
            // The boundary in the canonical angle, exactly: the first
            // nanoarcsecond of the part. Computing it in degrees would
            // be the very mistake this asserts against — 30/7 is not a
            // double, so `part * 30.0 / 7.0` is not the boundary.
            let boundary = first_nas_of(&scheme, rashi, part);
            assert_eq!(
                part_of(group, boundary, scheme.divisions),
                part,
                "{} part {part} at {} nas",
                scheme.key(),
                boundary.get()
            );
            assert_eq!(
                part_of(group, Nas::new(boundary.get() - 1), scheme.divisions),
                part - 1,
                "{} one nanoarcsecond below part {part}",
                scheme.key()
            );
        }
    }
}

/// The first nanoarcsecond of one part of one sign, exactly.
///
/// For equal spans that is the smallest angle inside the sign whose
/// `(angle · N) / PER_SIGN` reaches the part, which is
/// `ceil(part · PER_SIGN / N)`; for whole-degree spans it is the
/// cumulative width, which is exact already.
fn first_nas_of(scheme: &Scheme, rashi: Rashi, part: u16) -> Nas {
    let inside = match scheme.group(rashi).spans {
        Spans::Equal => {
            // `div_ceil` is unstable for signed integers, and both
            // operands here are positive.
            let divisions = i64::from(scheme.divisions);
            (i64::from(part) * Nas::PER_SIGN + divisions - 1) / divisions
        }
        Spans::Degrees(widths) => widths
            .iter()
            .take(usize::from(part))
            .map(|width| i64::from(*width) * Nas::PER_DEGREE)
            .sum(),
    };
    Nas::new(i64::from(rashi as u8) * Nas::PER_SIGN + inside)
}

#[test]
fn an_arbitrary_chart_answers_for_every_division_it_accepts() {
    // Every N the crate will take, end to end: a scheme, a table, and a
    // placement at three points of every sign.
    let mut answered = 0_usize;
    for divisions in 1..=MOST_DIVISIONS {
        let scheme = Scheme::cyclic(divisions).expect("inside the limit");
        assert_eq!(scheme.key(), format!("D{divisions}"));
        for rashi in Rashi::ALL {
            for offset in [0.0, 14.9, 29.99] {
                let longitude = at(f64::from(rashi as u8) * 30.0 + offset);
                let placement = place(&scheme, longitude);
                assert_eq!(placement.rashi, rashi);
                assert!(placement.part < divisions, "D{divisions}");
                answered += 1;
            }
        }
    }
    println!("{answered} placements over every arbitrary chart the crate accepts");
    assert_eq!(answered, usize::from(MOST_DIVISIONS) * 12 * 3);
    assert!(Scheme::cyclic(0).is_err());
    assert!(Scheme::cyclic(MOST_DIVISIONS + 1).is_err());
}

#[test]
fn a_longitude_is_placed_the_same_way_wherever_it_is_written() {
    // The same angle reached three ways — as itself, past a whole turn,
    // and from below zero — is the same placement in every chart. A
    // caller that normalises differently gets the same answer.
    for scheme in every_scheme() {
        for degrees in [0.0, 11.25, 179.9, 359.99] {
            let plain = place(&scheme, at(degrees));
            assert_eq!(
                plain,
                place(&scheme, at(degrees + 360.0)),
                "{}",
                scheme.key()
            );
            assert_eq!(
                plain,
                place(&scheme, at(degrees - 360.0)),
                "{}",
                scheme.key()
            );
        }
    }
}

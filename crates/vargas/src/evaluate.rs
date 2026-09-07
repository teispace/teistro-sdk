//! The evaluator: a longitude in, a placement out.
//!
//! Two steps and no arithmetic a platform can disagree about. Sort the
//! sign into its group, take the part of the sign the longitude falls in
//! by integer arithmetic on the canonical angle, and look up where that
//! part sends a body.

use serde::Serialize;
use teistro_core::angle::Nas;
use teistro_core::catalogue::Rashi;

use crate::scheme::{Scheme, part_of, target};

/// Where one body stands in one divisional chart.
///
/// ```
/// use teistro_core::angle::Nas;
/// use teistro_core::catalogue::{Rashi, Varga};
/// use teistro_core::quantity::Degrees;
/// use teistro_vargas::{Scheme, place};
///
/// // Ten degrees of Aries, in the navamsha.
/// let longitude = Nas::from_degrees(Degrees::try_new(10.0).expect("finite"));
/// let placement = place(&Scheme::of(Varga::D9), longitude);
///
/// assert_eq!(placement.rashi, Rashi::Aries);
/// assert_eq!(placement.part, 3, "the fourth navamsha of the sign");
/// assert_eq!(placement.sign, Rashi::Cancer);
/// assert!(!placement.keeps_its_sign());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Placement {
    /// The sign the body stands in.
    pub rashi: Rashi,
    /// Which part of that sign, counted from zero.
    pub part: u16,
    /// The sign the divisional chart puts it in.
    pub sign: Rashi,
}

impl Placement {
    /// Whether the divisional chart leaves the body in the sign it was
    /// already in.
    ///
    /// In the navamsha this is **vargottama**, the term the texts use;
    /// in another chart it is the same fact without the name, which is
    /// why it reads as what it is rather than as `is_vargottama`. The
    /// chart-level accessor names it.
    #[must_use]
    pub const fn keeps_its_sign(&self) -> bool {
        self.rashi as u8 == self.sign as u8
    }
}

/// Where a longitude stands in a divisional chart.
#[must_use]
pub fn place(scheme: &Scheme, longitude: Nas) -> Placement {
    let rashi = longitude.sign();
    let group = scheme.group(rashi);
    let part = part_of(group, longitude, scheme.divisions);
    Placement {
        rashi,
        part,
        // Every group of every shipped row lists or steps to a sign for
        // every part it has, which the crate's exhaustive test asserts;
        // the body's own sign stands in rather than panicking.
        sign: target(group, rashi, part, scheme.divisions).unwrap_or(rashi),
    }
}

/// Which sign a divisional chart puts a longitude in.
#[must_use]
pub fn sign(scheme: &Scheme, longitude: Nas) -> Rashi {
    place(scheme, longitude).sign
}

/// The chart's whole table: for each sign, the sign each of its parts
/// sends a body to.
///
/// Twelve rows, each as long as that sign's parts. What the kernel's own
/// tests generate and compare against the rule, and what a caller
/// printing a chart's definition wants.
#[must_use]
pub fn table(scheme: &Scheme) -> Vec<Vec<Rashi>> {
    Rashi::ALL
        .iter()
        .map(|rashi| {
            let group = scheme.group(*rashi);
            (0..group.parts(scheme.divisions))
                .map(|part| target(group, *rashi, part, scheme.divisions).unwrap_or(*rashi))
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::{place, sign, table};
    use crate::scheme::{SCHEMES, Scheme};
    use teistro_core::angle::Nas;
    use teistro_core::catalogue::{Rashi, Varga};
    use teistro_core::quantity::Degrees;

    fn at(degrees: f64) -> Nas {
        Nas::from_degrees(Degrees::try_new(degrees.rem_euclid(360.0)).expect("finite"))
    }

    #[test]
    fn the_rashi_chart_never_moves_a_body() {
        let rashi = Scheme::of(Varga::D1);
        for degrees in 0..360 {
            let longitude = at(f64::from(degrees) + 0.5);
            let placement = place(&rashi, longitude);
            assert_eq!(placement.sign, placement.rashi);
            assert!(placement.keeps_its_sign());
            assert_eq!(sign(&rashi, longitude), longitude.sign());
        }
    }

    #[test]
    fn a_table_has_a_row_per_sign_and_a_cell_per_part() {
        for scheme in SCHEMES {
            let rows = table(&scheme);
            assert_eq!(rows.len(), 12, "{:?}", scheme.varga);
            for (index, row) in rows.iter().enumerate() {
                let rashi = Rashi::from_id(u16::try_from(index).unwrap()).expect("a sign");
                assert_eq!(
                    u16::try_from(row.len()).unwrap(),
                    scheme.parts(rashi),
                    "{:?} {rashi:?}",
                    scheme.varga
                );
            }
        }
    }

    #[test]
    fn a_placement_agrees_with_the_table_it_came_from() {
        // The evaluator and the materialised table are two ways to the
        // same answer, and the design's fourth invariant is that they
        // agree for every sign and every part of every chart.
        for scheme in SCHEMES {
            let rows = table(&scheme);
            for rashi in Rashi::ALL {
                let parts = scheme.parts(rashi);
                let group = scheme.group(rashi);
                for part in 0..parts {
                    // A longitude in the middle of the part, so no
                    // boundary can decide it either way.
                    let inside = middle(&scheme, rashi, part);
                    let placement = place(&scheme, inside);
                    assert_eq!(placement.rashi, rashi, "{:?}", scheme.varga);
                    assert_eq!(placement.part, part, "{:?} {rashi:?}", scheme.varga);
                    assert_eq!(
                        placement.sign,
                        rows[rashi as usize][usize::from(part)],
                        "{:?} {rashi:?} part {part}",
                        scheme.varga
                    );
                    let _ = group;
                }
            }
        }
    }

    /// A longitude in the middle of one part of one sign.
    fn middle(scheme: &Scheme, rashi: Rashi, part: u16) -> Nas {
        use crate::scheme::Spans;
        let group = scheme.group(rashi);
        let (from, to) = match group.spans {
            Spans::Equal => {
                let width = 30.0 / f64::from(scheme.divisions);
                (f64::from(part) * width, f64::from(part + 1) * width)
            }
            Spans::Degrees(widths) => {
                let before: u16 = widths
                    .iter()
                    .take(usize::from(part))
                    .map(|width| u16::from(*width))
                    .sum();
                let own = widths.get(usize::from(part)).copied().unwrap_or(0);
                (f64::from(before), f64::from(before) + f64::from(own))
            }
        };
        at(f64::from(rashi as u8) * 30.0 + f64::midpoint(from, to))
    }

    #[test]
    fn vargottama_is_the_navamsha_keeping_its_sign() {
        let navamsha = Scheme::of(Varga::D9);
        // The first navamsha of a movable sign is the sign itself.
        for rashi in [Rashi::Aries, Rashi::Cancer, Rashi::Libra, Rashi::Capricorn] {
            let placement = place(&navamsha, at(f64::from(rashi as u8) * 30.0 + 1.0));
            assert!(placement.keeps_its_sign(), "{rashi:?}");
        }
        // A fixed sign's is its fifth, and a dual sign's its ninth.
        for (rashi, part) in [(Rashi::Taurus, 4_u16), (Rashi::Gemini, 8)] {
            let width = 30.0 / 9.0;
            let inside = f64::from(rashi as u8) * 30.0 + (f64::from(part) + 0.5) * width;
            assert!(place(&navamsha, at(inside)).keeps_its_sign(), "{rashi:?}");
        }
    }
}

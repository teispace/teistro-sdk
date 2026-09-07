//! The bhavas of a chart: their boundaries, their middles, and where a
//! graha falls between them.
//!
//! `astro::houses` divides the sky into twelve and returns the division's
//! cusps. That is not yet a bhava. A bhava has a *sandhi*, the boundary a
//! graha crosses to change house, and a *madhya*, the middle, where a
//! graha is strongest — and which of the two a system's cusps are depends
//! on the system. Sripati's cusps are the middles; every other system's
//! are the boundaries. A reading that gets this backwards is half a house
//! out everywhere, which the falsification pass measured as half the
//! chart (`03-design/chart-bhava-chalit.md`).
//!
//! So this module is the one place that turns cusps into bhavas, and
//! every module above it asks for a [`Placement`] rather than comparing
//! longitudes to cusps itself.

use teistro_core::angle::difference_deg;
use teistro_core::catalogue::HouseSystem;

/// Whether a system's cusps are the boundaries of its houses or their
/// middles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    /// The cusps are the sandhi: a house runs from one cusp to the next,
    /// and its madhya is midway between them. Every system but Sripati.
    Sandhi,
    /// The cusps are the madhya: each cusp is the middle of its house,
    /// and the sandhi are midway between consecutive middles. Sripati,
    /// which is the classical Indian bhava.
    Madhya,
}

/// A bhava chalit: which house system's cusps it is built from, and
/// which of the two things those cusps are.
///
/// The distinction is not cosmetic. Sripati and Porphyry are built on the
/// *same cusps* and disagree about a graha's house half the time, because
/// one reads them as middles and the other as boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chalit {
    /// The chalit's own name, which is what a result reports.
    pub method: HouseSystem,
    /// The system whose cusps it is built from. The same as `method`
    /// except for Sripati, whose madhya are Porphyry's cusps.
    pub source: HouseSystem,
    /// What those cusps are.
    pub reading: Reading,
}

impl Chalit {
    /// The chalit a house system names.
    ///
    /// ```
    /// # use teistro_chart::bhava::{Chalit, Reading};
    /// # use teistro_core::catalogue::HouseSystem;
    /// let sripati = Chalit::of(HouseSystem::Sripati);
    /// assert_eq!(sripati.source, HouseSystem::Porphyry);
    /// assert_eq!(sripati.reading, Reading::Madhya);
    ///
    /// let vehlow = Chalit::of(HouseSystem::Vehlow);
    /// assert_eq!(vehlow.source, HouseSystem::Vehlow);
    /// assert_eq!(vehlow.reading, Reading::Sandhi);
    /// ```
    #[must_use]
    pub const fn of(method: HouseSystem) -> Chalit {
        match method {
            // The classical bhava: the trisected quadrants are the
            // middles of the houses, not their edges. `astro`'s own
            // `SRIPATI` system returns the sandhi that follow from that,
            // and the madhya cannot be recovered from them — unequal
            // spacing means the middles are not midway between the
            // boundaries — so the chalit is built from Porphyry's cusps.
            HouseSystem::Sripati => Chalit {
                method,
                source: HouseSystem::Porphyry,
                reading: Reading::Madhya,
            },
            _ => Chalit {
                method,
                source: method,
                reading: Reading::Sandhi,
            },
        }
    }
}

/// The twelve bhavas of a chart: where each begins and where its middle
/// is, bhava 1 first.
///
/// Both are kept because both are asked for and neither can always be
/// derived from the other: under an unequal division the madhya are not
/// midway between the sandhi.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bhavas {
    /// The boundaries, in the chart's own zodiac, degrees.
    pub sandhi: [f64; 12],
    /// The middles, in the chart's own zodiac, degrees.
    pub madhya: [f64; 12],
    /// Which chalit these are, which every placement reports.
    pub chalit: Chalit,
}

impl Bhavas {
    /// The bhavas a chalit makes of a division's cusps.
    ///
    /// The cusps are the ones `astro::houses` returns for
    /// [`Chalit::source`], in whatever zodiac the caller wants the
    /// bhavas in; nothing here converts frames.
    #[must_use]
    pub fn of(chalit: Chalit, cusps: &[f64; 12]) -> Bhavas {
        let at = |index: usize| cusps.get(index % 12).copied().unwrap_or_default();
        let between = |a: f64, b: f64| (a + (b - a).rem_euclid(360.0) / 2.0).rem_euclid(360.0);
        match chalit.reading {
            Reading::Sandhi => Bhavas {
                sandhi: core::array::from_fn(|i| at(i).rem_euclid(360.0)),
                madhya: core::array::from_fn(|i| between(at(i), at(i + 1))),
                chalit,
            },
            Reading::Madhya => Bhavas {
                // The sandhi that *opens* bhava n is midway between the
                // middle before it and its own.
                sandhi: core::array::from_fn(|i| between(at(i + 11), at(i))),
                madhya: core::array::from_fn(|i| at(i).rem_euclid(360.0)),
                chalit,
            },
        }
    }

    /// Where a longitude falls: which bhava, how far through it, and how
    /// far from its middle.
    ///
    /// A bhava runs from its own sandhi forward through the zodiac to the
    /// next, so the bhava that spans 0° is one bhava and not two.
    #[must_use]
    pub fn place(&self, longitude_deg: f64) -> Placement {
        let longitude = longitude_deg.rem_euclid(360.0);
        let madhya = |index: usize| self.madhya.get(index).copied().unwrap_or_default();
        for (index, start) in self.sandhi.iter().enumerate() {
            let width = (self
                .sandhi
                .get((index + 1) % 12)
                .copied()
                .unwrap_or_default()
                - start)
                .rem_euclid(360.0);
            // Two sandhi at one degree can only happen in a degenerate
            // division; a zero span would divide by zero and swallow the
            // whole circle instead.
            let span = if width == 0.0 { 360.0 } else { width };
            let into = (longitude - start).rem_euclid(360.0);
            if into < span {
                return Placement {
                    bhava: u8::try_from(index).unwrap_or(0) + 1,
                    method: self.chalit.method,
                    through: into / span,
                    from_madhya_deg: difference_deg(longitude, madhya(index)),
                };
            }
        }
        // Unreachable while the spans sum to a full circle; a bhava is
        // still reported rather than a panic, because a chart with a
        // degenerate division is a chart a caller may still want to see.
        Placement {
            bhava: 12,
            method: self.chalit.method,
            through: 0.0,
            from_madhya_deg: difference_deg(longitude, madhya(11)),
        }
    }

    /// The width of each bhava in degrees, bhava 1 first. Equal
    /// divisions give twelve thirties; quadrant divisions do not.
    #[must_use]
    pub fn widths(&self) -> [f64; 12] {
        let at = |index: usize| self.sandhi.get(index % 12).copied().unwrap_or_default();
        core::array::from_fn(|i| (at(i + 1) - at(i)).rem_euclid(360.0))
    }
}

/// Where one graha sits in the bhavas.
///
/// It carries the method because the methods disagree: over the corpus's
/// 55 charts the four named chalits put a graha in a different bhava
/// between 10% and 51% of the time, so a bhava number without its method
/// is not a reproducible fact (`03-design/chart-bhava-chalit.md`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// The bhava, 1 to 12.
    pub bhava: u8,
    /// The chalit that put it there.
    pub method: HouseSystem,
    /// How far through the bhava, 0 at the sandhi that opens it and
    /// approaching 1 at the one that closes it.
    pub through: f64,
    /// How far from the bhava's madhya, degrees, signed: negative before
    /// the middle, positive after. What bhava bala is measured from.
    pub from_madhya_deg: f64,
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::{Bhavas, Chalit, Reading};
    use teistro_core::catalogue::HouseSystem;

    /// Equal thirty-degree houses starting at the ascendant.
    fn equal_from(start: f64) -> [f64; 12] {
        let mut cusps = [0.0_f64; 12];
        for (index, cusp) in cusps.iter_mut().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "twelve")]
            let n = index as f64;
            *cusp = (start + 30.0 * n).rem_euclid(360.0);
        }
        cusps
    }

    #[test]
    fn a_sandhi_reading_puts_the_middle_between_two_boundaries() {
        let bhavas = Bhavas::of(Chalit::of(HouseSystem::Equal), &equal_from(10.0));
        assert!((bhavas.sandhi[0] - 10.0).abs() < 1e-9);
        assert!(
            (bhavas.madhya[0] - 25.0).abs() < 1e-9,
            "{}",
            bhavas.madhya[0]
        );
        // The pair that wraps past 0° wraps correctly rather than
        // averaging to the far side of the circle.
        assert!((bhavas.sandhi[11] - 340.0).abs() < 1e-9);
        assert!(
            (bhavas.madhya[11] - 355.0).abs() < 1e-9,
            "{}",
            bhavas.madhya[11]
        );
    }

    #[test]
    fn a_madhya_reading_puts_the_boundary_between_two_middles() {
        // Sripati over an equal division: the middles are the cusps and
        // the boundaries fall fifteen degrees before each.
        let bhavas = Bhavas::of(
            Chalit {
                method: HouseSystem::Sripati,
                source: HouseSystem::Porphyry,
                reading: Reading::Madhya,
            },
            &equal_from(10.0),
        );
        assert!((bhavas.madhya[0] - 10.0).abs() < 1e-9);
        assert!(
            (bhavas.sandhi[0] - 355.0).abs() < 1e-9,
            "{}",
            bhavas.sandhi[0]
        );
        assert!((bhavas.madhya[6] - 190.0).abs() < 1e-9);
        assert!((bhavas.sandhi[6] - 175.0).abs() < 1e-9);
    }

    #[test]
    fn the_two_readings_of_one_division_are_half_a_house_apart() {
        let cusps = equal_from(0.0);
        let as_sandhi = Bhavas::of(Chalit::of(HouseSystem::Porphyry), &cusps);
        let as_madhya = Bhavas::of(
            Chalit {
                method: HouseSystem::Sripati,
                source: HouseSystem::Porphyry,
                reading: Reading::Madhya,
            },
            &cusps,
        );
        // A graha at 20° is in the first house of one and the twelfth of
        // the other: the whole point of the falsification pass.
        assert_eq!(as_sandhi.place(20.0).bhava, 1);
        assert_eq!(as_madhya.place(20.0).bhava, 2);
        assert_eq!(as_sandhi.place(5.0).bhava, 1);
        assert_eq!(as_madhya.place(5.0).bhava, 1);
    }

    #[test]
    fn a_graha_is_placed_by_span_and_reports_where_in_the_bhava_it_is() {
        let bhavas = Bhavas::of(Chalit::of(HouseSystem::Equal), &equal_from(10.0));
        let at_start = bhavas.place(10.0);
        assert_eq!(at_start.bhava, 1);
        assert!(
            at_start.through.abs() < 1e-12,
            "a sandhi opens its own bhava"
        );
        assert!(
            (at_start.from_madhya_deg + 15.0).abs() < 1e-9,
            "fifteen before the middle"
        );

        let at_middle = bhavas.place(25.0);
        assert_eq!(at_middle.bhava, 1);
        assert!((at_middle.through - 0.5).abs() < 1e-12);
        assert!(at_middle.from_madhya_deg.abs() < 1e-9);

        let at_end = bhavas.place(39.999);
        assert_eq!(at_end.bhava, 1);
        assert!(at_end.through > 0.999);
        assert_eq!(bhavas.place(40.0).bhava, 2);

        // The bhava that spans 0° is one bhava.
        assert_eq!(bhavas.place(355.0).bhava, 12);
        assert_eq!(bhavas.place(5.0).bhava, 12);
        assert_eq!(bhavas.place(0.0).bhava, 12);
    }

    #[test]
    fn an_unequal_division_keeps_its_own_widths() {
        // A quadrant division: the houses are not thirty degrees wide,
        // and the middles are not midway between the boundaries of the
        // *madhya* reading, which is why both are stored.
        let cusps = [
            0.0, 20.0, 45.0, 90.0, 135.0, 160.0, 180.0, 200.0, 225.0, 270.0, 315.0, 340.0,
        ];
        let bhavas = Bhavas::of(Chalit::of(HouseSystem::Porphyry), &cusps);
        let widths = bhavas.widths();
        assert!((widths[0] - 20.0).abs() < 1e-9);
        assert!((widths[2] - 45.0).abs() < 1e-9);
        assert!(
            (widths.iter().sum::<f64>() - 360.0).abs() < 1e-9,
            "the widths fill the circle"
        );
        // A graha two thirds through the widest house.
        let placed = bhavas.place(75.0);
        assert_eq!(placed.bhava, 3);
        assert!((placed.through - (30.0 / 45.0)).abs() < 1e-9);
    }

    #[test]
    fn every_chalit_but_sripati_reads_its_own_cusps() {
        for method in [
            HouseSystem::Vehlow,
            HouseSystem::Porphyry,
            HouseSystem::Placidus,
            HouseSystem::WholeSign,
            HouseSystem::Equal,
            HouseSystem::Koch,
        ] {
            let chalit = Chalit::of(method);
            assert_eq!(chalit.source, method, "{method:?}");
            assert_eq!(chalit.reading, Reading::Sandhi, "{method:?}");
            assert_eq!(chalit.method, method);
        }
    }
}

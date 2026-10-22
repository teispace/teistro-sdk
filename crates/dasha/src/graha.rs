//! Graha dashas whose lords and years both come from the chart, as a kernel
//! behind [`Timeline`] (K-graha, `03-design/dasha-kernels.md`).
//!
//! No nakshatra seeds them and no balance cuts the first period: the lords
//! run whole from birth, each for the years the chart gives it, and a
//! period's sub-periods run from its own lord in the same order, in
//! proportion to the lords' years against their total (BPHS ch. 51 vv. 1
//! and 2) or in equal parts (vv. 3 and 4). The Karaka dasha is one (ch. 46
//! v. 178): the chara karakas from the Atmakaraka, each for the signs
//! counted from the lagna to its own, [`karaka_years`]. So is the Tara
//! dasha (vv. 207 to 209): Vimshottari's lords and years from the strongest
//! graha in a kendra, [`tara_lords`], whose antardashas ch. 51 v. 1 gives
//! as Vimshottari's own.
//!
//! ```
//! use teistro_core::catalogue::{Graha, Rashi};
//! use teistro_core::quantity::JulianDay;
//! use teistro_core::settings::{AfterCycle, GrahaDivision, YearLength};
//! use teistro_dasha::{GrahaDasha, Timeline, karaka_years};
//!
//! // An Aquarius lagna: the Sun in Capricorn has eleven signs to run, and
//! // Venus, with the lagna, twelve.
//! assert_eq!(karaka_years(Rashi::Aquarius, Rashi::Capricorn), 11);
//! assert_eq!(karaka_years(Rashi::Aquarius, Rashi::Aquarius), 12);
//! let dasha = GrahaDasha::new(
//!     vec![(Graha::Sun, 11.0), (Graha::Venus, 12.0)],
//!     JulianDay::literal(2_451_545.0),
//!     YearLength::Julian36525,
//!     GrahaDivision::Proportional,
//!     AfterCycle::End,
//! )?;
//! let venus = dasha.mahadasha(0, 1).ok_or("a second period")?;
//! assert!((venus.interval.days() - 12.0 * 365.25).abs() < 1e-6);
//! // Venus's period opens with Venus, her twelve of the twenty-three years.
//! let first = dasha.child(&venus, 0).ok_or("a sub-period")?;
//! assert_eq!(first.lord, Graha::Venus);
//! assert!((first.interval.days() - 12.0 * 365.25 * 12.0 / 23.0).abs() < 1e-6);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::house::House;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::{AfterCycle, GrahaDivision, YearLength};

use crate::row::VIMSHOTTARI_LORDS;
use crate::tree::{Path, Period, Timeline};

/// The Tara dasha's lords and years (BPHS ch. 46 vv. 207 to 209):
/// Vimshottari's, in its order and with its years, begun from the strongest
/// of the grahas given that stands in a kendra, each given with its house
/// from the lagna and its strength. Equal strengths keep the order given.
/// Nothing when none stands in a kendra, the chart v. 209 does not give the
/// dasha to.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_core::house::House;
/// use teistro_dasha::graha::tara_lords;
///
/// let house = House::try_new;
/// // Jupiter in the lagna, stronger than Saturn in the tenth; the Sun,
/// // stronger still, is in the twelfth and not a kendra.
/// let lords = tara_lords(&[
///     (Graha::Sun, house(12)?, 9.0),
///     (Graha::Jupiter, house(1)?, 7.0),
///     (Graha::Saturn, house(10)?, 6.0),
/// ])
/// .ok_or("a graha in a kendra")?;
/// assert_eq!(lords[0], (Graha::Jupiter, 16.0));
/// assert_eq!(lords[1], (Graha::Saturn, 19.0));
/// assert!(tara_lords(&[(Graha::Sun, house(12)?, 9.0)]).is_none());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn tara_lords(grahas: &[(Graha, House, f64)]) -> Option<Vec<(Graha, f64)>> {
    let mut first: Option<(Graha, f64)> = None;
    for &(graha, house, strength) in grahas {
        if house.is_kendra() && first.is_none_or(|(_, best)| strength > best) {
            first = Some((graha, strength));
        }
    }
    let (first, _) = first?;
    let from = VIMSHOTTARI_LORDS
        .iter()
        .position(|lord| lord.graha == first)?;
    let count = VIMSHOTTARI_LORDS.len();
    (0..count)
        .map(|step| {
            VIMSHOTTARI_LORDS
                .get((from + step) % count)
                .map(|lord| (lord.graha, f64::from(lord.years)))
        })
        .collect()
}

/// The years a karaka gives in the Karaka dasha (BPHS ch. 46 v. 178): the
/// signs counted forward from the lagna's to the karaka's, its own sign
/// counting twelve, whatever the lagna's footedness. Santhanam's worked
/// table under the verse counts so for every one of its eight.
#[must_use]
pub fn karaka_years(lagna: Rashi, karaka: Rashi) -> u8 {
    let ahead = (karaka.id() + 12 - lagna.id()) % 12;
    // Below twelve, so it fits.
    let ahead = u8::try_from(ahead).unwrap_or(0);
    if ahead == 0 { 12 } else { ahead }
}

/// A graha dasha of one birth: its lords in the order they run, each with
/// the years the chart gives it.
#[derive(Clone, Debug, PartialEq)]
pub struct GrahaDasha {
    lords: Vec<(Graha, f64)>,
    birth: JulianDay<Utc>,
    year_days: f64,
    total_years: f64,
    division: GrahaDivision,
    after_cycle: AfterCycle,
}

impl GrahaDasha {
    /// The dasha running `lords` in order from `birth`, each for its years
    /// of `year_length`, divided as `division` says at every level, and
    /// begun again after the last lord when `after_cycle` repeats it.
    ///
    /// # Errors
    ///
    /// No lords, or a lord named twice, named `lords`; a lord's years not
    /// a positive number, named `lords[i]`; a year of no days, named
    /// `year_length`.
    pub fn new(
        lords: Vec<(Graha, f64)>,
        birth: JulianDay<Utc>,
        year_length: YearLength,
        division: GrahaDivision,
        after_cycle: AfterCycle,
    ) -> Result<GrahaDasha, Error> {
        if lords.is_empty() {
            return Err(
                Error::invalid_arg("a graha dasha runs at least one lord").with_field("lords")
            );
        }
        if lords.len() > usize::from(u8::MAX) {
            return Err(
                Error::invalid_arg("a graha dasha runs at most 255 lords").with_field("lords")
            );
        }
        for (at, (graha, years)) in lords.iter().enumerate() {
            if !(years.is_finite() && *years > 0.0) {
                return Err(Error::invalid_arg(format!(
                    "{graha:?} is given {years} years, not a positive number"
                ))
                .with_field(format!("lords[{at}]")));
            }
            if lords.iter().take(at).any(|(earlier, _)| earlier == graha) {
                return Err(
                    Error::invalid_arg(format!("{graha:?} is named twice")).with_field("lords")
                );
            }
        }
        let year_days = year_length.days();
        if year_days.is_nan() || year_days <= 0.0 {
            return Err(Error::invalid_arg("a year of no days").with_field("year_length"));
        }
        let total_years = lords.iter().map(|(_, years)| years).sum();
        Ok(GrahaDasha {
            lords,
            birth,
            year_days,
            total_years,
            division,
            after_cycle,
        })
    }

    /// The lords in the order they run, each with its years.
    #[must_use]
    pub fn lords(&self) -> &[(Graha, f64)] {
        &self.lords
    }

    /// How a period divides among its sub-periods.
    #[must_use]
    pub const fn division(&self) -> GrahaDivision {
        self.division
    }

    /// The days of one cycle, every lord's years.
    fn cycle_days(&self) -> f64 {
        self.total_years * self.year_days
    }

    /// The share of a parent's span the lord at `seat` takes.
    fn share(&self, seat: usize) -> Option<f64> {
        let (_, years) = self.lords.get(seat)?;
        Some(match self.division {
            #[expect(clippy::cast_precision_loss, reason = "at most 255 lords")]
            GrahaDivision::Equal => 1.0 / self.lords.len() as f64,
            _ => years / self.total_years,
        })
    }
}

impl Timeline for GrahaDasha {
    fn breadth(&self) -> usize {
        self.lords.len()
    }

    /// The `index`th lord's period of `cycle`; past the first cycle only
    /// when the rules repeat it.
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        if cycle > 0 && self.after_cycle == AfterCycle::End {
            return None;
        }
        let (lord, years) = *self.lords.get(index)?;
        let before: f64 = self.lords.iter().take(index).map(|(_, years)| years).sum();
        let from =
            self.birth.get() + f64::from(cycle) * self.cycle_days() + before * self.year_days;
        let span = Interval::literal(from, from + years * self.year_days);
        Some(Period::of_share(
            lord,
            None,
            Path::root(cycle, u8::try_from(index).ok()?),
            span,
            span,
            index,
        ))
    }

    fn mahadasha_at(&self, instant: f64) -> Option<Period> {
        let into = instant - self.birth.get();
        if into < 0.0 {
            return None;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a non-negative whole number of cycles, far below u32::MAX for any date"
        )]
        let cycle = (into / self.cycle_days()).floor() as u32;
        (0..self.lords.len())
            .filter_map(|index| self.mahadasha(cycle, index))
            .find(|period| instant < period.interval.to.get())
    }

    /// The `index`th sub-period of `parent`, from its own lord onwards in
    /// the lords' order (ch. 51 v. 2), its share as the division says.
    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        let count = self.lords.len();
        if index >= count {
            return None;
        }
        // The lords are distinct, so a lord's place is its seat.
        let from = self
            .lords
            .iter()
            .position(|(graha, _)| *graha == parent.lord)?;
        let seat_of = |step: usize| (from + step) % count;
        let days = parent.interval.days();
        let mut start = parent.interval.from.get();
        for step in 0..index {
            start += days * self.share(seat_of(step))?;
        }
        let seat = seat_of(index);
        let span = Interval::literal(start, start + days * self.share(seat)?);
        Some(Period::of_share(
            self.lords.get(seat)?.0,
            None,
            parent.path.child(u8::try_from(index).ok()?)?,
            span,
            span,
            seat,
        ))
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test fails by panicking and indexes what it built"
)]
mod tests {
    use super::*;
    use teistro_core::quantity::Depth;

    fn house(number: u8) -> House {
        House::try_new(number).unwrap()
    }

    fn karaka(division: GrahaDivision, after_cycle: AfterCycle) -> GrahaDasha {
        GrahaDasha::new(
            vec![(Graha::Sun, 11.0), (Graha::Venus, 12.0), (Graha::Mars, 3.0)],
            JulianDay::literal(2_451_545.0),
            YearLength::Julian36525,
            division,
            after_cycle,
        )
        .unwrap()
    }

    #[test]
    fn every_level_partitions_its_parent_from_its_own_lord() {
        for division in [GrahaDivision::Proportional, GrahaDivision::Equal] {
            let dasha = karaka(division, AfterCycle::End);
            for maha in dasha.mahadashas() {
                let children: Vec<Period> = dasha.children(&maha).collect();
                assert_eq!(children.len(), 3);
                assert_eq!(children[0].lord, maha.lord);
                assert_eq!(children[0].interval.from, maha.interval.from);
                let end = children.last().unwrap().interval.to.get();
                assert!((end - maha.interval.to.get()).abs() < 1e-9, "{division:?}");
                for pair in children.windows(2) {
                    assert_eq!(pair[0].interval.to, pair[1].interval.from);
                }
            }
        }
    }

    #[test]
    fn the_equal_division_gives_each_lord_one_part() {
        let dasha = karaka(GrahaDivision::Equal, AfterCycle::End);
        let mars = dasha.mahadasha(0, 2).unwrap();
        let children: Vec<Period> = dasha.children(&mars).collect();
        assert_eq!(
            children.iter().map(|child| child.lord).collect::<Vec<_>>(),
            [Graha::Mars, Graha::Sun, Graha::Venus]
        );
        for child in children {
            assert!((child.interval.days() - 3.0 * 365.25 / 3.0).abs() < 1e-9);
        }
    }

    #[test]
    fn the_cycle_ends_or_repeats_as_the_rules_say() {
        let after = JulianDay::literal(2_451_545.0 + 27.0 * 365.25);
        let ended = karaka(GrahaDivision::Proportional, AfterCycle::End);
        assert!(ended.at(after, Depth::MIN).deepest().is_none());
        let repeated = karaka(GrahaDivision::Proportional, AfterCycle::Repeat);
        let running = repeated.at(after, Depth::MIN);
        assert_eq!(running.deepest().unwrap().lord, Graha::Sun);
        assert_eq!(running.deepest().unwrap().path.to_string(), "1:0");
    }

    #[test]
    fn a_malformed_list_is_refused_by_its_field() {
        let refused = |lords: Vec<(Graha, f64)>| {
            GrahaDasha::new(
                lords,
                JulianDay::literal(0.0),
                YearLength::Julian36525,
                GrahaDivision::Proportional,
                AfterCycle::End,
            )
            .unwrap_err()
            .field()
            .map(str::to_owned)
        };
        assert_eq!(refused(vec![]).as_deref(), Some("lords"));
        assert_eq!(
            refused(vec![(Graha::Sun, 1.0), (Graha::Sun, 2.0)]).as_deref(),
            Some("lords")
        );
        assert_eq!(
            refused(vec![(Graha::Sun, 1.0), (Graha::Moon, 0.0)]).as_deref(),
            Some("lords[1]")
        );
        assert_eq!(
            refused(vec![(Graha::Sun, f64::NAN)]).as_deref(),
            Some("lords[0]")
        );
    }

    #[test]
    fn the_tara_dasha_runs_vimshottari_from_the_strongest_kendra_graha() {
        // Santhanam's example under vv. 207 to 209: Jupiter in the lagna
        // with Venus and Mercury, Mars in the fourth and Saturn in the
        // tenth, Jupiter the strongest, and the years Vimshottari's.
        let lords = tara_lords(&[
            (Graha::Sun, house(12), 5.0),
            (Graha::Moon, house(5), 5.0),
            (Graha::Mars, house(4), 5.0),
            (Graha::Mercury, house(1), 6.0),
            (Graha::Jupiter, house(1), 8.0),
            (Graha::Venus, house(1), 7.0),
            (Graha::Saturn, house(10), 6.5),
        ])
        .unwrap();
        assert_eq!(
            lords,
            [
                (Graha::Jupiter, 16.0),
                (Graha::Saturn, 19.0),
                (Graha::Mercury, 17.0),
                (Graha::Ketu, 7.0),
                (Graha::Venus, 20.0),
                (Graha::Sun, 6.0),
                (Graha::Moon, 10.0),
                (Graha::Mars, 7.0),
                (Graha::Rahu, 18.0),
            ]
        );
        // Equal strengths keep the order given; none in a kendra, nothing.
        let tied =
            tara_lords(&[(Graha::Mars, house(7), 5.0), (Graha::Venus, house(4), 5.0)]).unwrap();
        assert_eq!(tied[0].0, Graha::Mars);
        assert!(
            tara_lords(&[(Graha::Sun, house(2), 9.0), (Graha::Moon, house(11), 9.0)]).is_none()
        );
    }

    #[test]
    fn a_karaka_counts_forward_from_the_lagna_and_its_own_sign_is_twelve() {
        for lagna in Rashi::ALL {
            for karaka in Rashi::ALL {
                let years = karaka_years(lagna, karaka);
                assert!((1..=12).contains(&years));
                let back = (lagna.id() + u16::from(years)) % 12;
                assert_eq!(back, karaka.id(), "{lagna:?} to {karaka:?}");
            }
        }
    }
}

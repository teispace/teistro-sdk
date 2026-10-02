//! The firdaria as a kernel behind [`Timeline`]
//! (`03-design/hellenistic-firdaria.md`).
//!
//! al-Biruni, *Book of Instruction* §395 and §§438–439: life is divided
//! into nine periods, the seven's and then the nodes', 75 years in all,
//! begun from the Sun in a day birth and from the Moon in a night one,
//! the rest in the descending order of the spheres. A planet's period is
//! shared out in sevenths, the first its own and each next with the planet
//! below; the nodes' are not divided. After 75 years "it returns to the
//! Sun" (Abu Ma'shar, *Revolutions of the Years of Nativities* IV.1).
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_core::quantity::{Depth, JulianDay};
//! use teistro_core::settings::{FirdariaNodes, YearLength};
//! use teistro_dasha::{FirdariaDasha, Timeline};
//!
//! // A night birth: the Moon's 9 years, then Saturn's 11, shared in
//! // sevenths of 11/7 years from Saturn down. Halfway through the 15th
//! // year is 5.5 years into Saturn's, inside its fourth seventh: Saturn,
//! // Jupiter, Mars, then the Sun.
//! let birth = JulianDay::literal(2_451_545.0);
//! let dasha =
//!     FirdariaDasha::new(Graha::Moon, birth, YearLength::Julian36525, FirdariaNodes::End)?;
//! let fifteenth = JulianDay::literal(2_451_545.0 + 14.5 * 365.25);
//! let chain = dasha.at(fifteenth, Depth::try_new(2)?);
//! let lords: Vec<Graha> = chain.iter().map(|period| period.lord).collect();
//! assert_eq!(lords, [Graha::Saturn, Graha::Sun]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::{FirdariaNodes, YearLength};

use crate::tree::{Path, Period, Timeline};

/// The seven in the descending order of their spheres, which both the
/// periods and the sevenths follow (§395).
pub(crate) const DESCENDING: [Graha; 7] = [
    Graha::Saturn,
    Graha::Jupiter,
    Graha::Mars,
    Graha::Sun,
    Graha::Venus,
    Graha::Mercury,
    Graha::Moon,
];

/// The periods of one round: the seven and the two nodes.
pub const FIRDARIA_PERIODS: usize = 9;

/// The years of one round: "a span of 75 years" (§438).
pub const FIRDARIA_YEARS: u32 = 75;

/// The rounds one cycle holds: two, 150 years, so that a stored reading
/// answers the period at any age a life reaches; the cursor carries on
/// past them, as Abu Ma'shar's "returns to the Sun" does.
pub const FIRDARIA_ROUNDS: usize = 2;

/// A firdar's years (§438): the Sun 10, Venus 8, Mercury 13, the Moon 9,
/// Saturn 11, Jupiter 12, Mars 7, the Head 3 and the Tail 2.
#[must_use]
pub const fn firdar_years(lord: Graha) -> u32 {
    match lord {
        Graha::Sun => 10,
        Graha::Venus => 8,
        Graha::Mercury => 13,
        Graha::Moon => 9,
        Graha::Saturn => 11,
        Graha::Jupiter => 12,
        Graha::Mars => 7,
        Graha::Rahu => 3,
        Graha::Ketu => 2,
        _ => 0,
    }
}

/// The nine lords of a round, in order: the seven from `first` in
/// descending order, the nodes at the end or, under
/// [`FirdariaNodes::AfterMars`], after Mars.
#[must_use]
pub fn firdaria_order(first: Graha, nodes: FirdariaNodes) -> [Graha; FIRDARIA_PERIODS] {
    let from = DESCENDING
        .iter()
        .position(|&graha| graha == first)
        .unwrap_or_default();
    let seven = (0..DESCENDING.len()).filter_map(|step| DESCENDING.get((from + step) % 7));
    let mut order = [Graha::Rahu; FIRDARIA_PERIODS];
    let mut slot = 0;
    let mut place = |graha: Graha| {
        if let Some(cell) = order.get_mut(slot) {
            *cell = graha;
            slot += 1;
        }
    };
    for &graha in seven {
        place(graha);
        if graha == Graha::Mars && nodes == FirdariaNodes::AfterMars {
            place(Graha::Rahu);
            place(Graha::Ketu);
        }
    }
    if nodes == FirdariaNodes::End {
        place(Graha::Rahu);
        place(Graha::Ketu);
    }
    order
}

/// The firdaria of a birth: its luminary's round, repeating.
#[derive(Clone, Debug, PartialEq)]
pub struct FirdariaDasha {
    order: [Graha; FIRDARIA_PERIODS],
    birth: JulianDay<Utc>,
    year_days: f64,
}

impl FirdariaDasha {
    /// The firdaria begun from `first`, the Sun for a day birth and the
    /// Moon for a night one, a year being `year_length`, the nodes placed
    /// as `nodes` says.
    ///
    /// # Errors
    ///
    /// A `first` that is neither luminary, named `first`; a year of no
    /// days, named `year_length`.
    pub fn new(
        first: Graha,
        birth: JulianDay<Utc>,
        year_length: YearLength,
        nodes: FirdariaNodes,
    ) -> Result<FirdariaDasha, Error> {
        if !matches!(first, Graha::Sun | Graha::Moon) {
            return Err(Error::invalid_arg(format!(
                "the firdaria begin from the Sun by day or the Moon by night, not {first:?}"
            ))
            .with_field("first"));
        }
        let year_days = year_length.days();
        if year_days.is_nan() || year_days <= 0.0 {
            return Err(Error::invalid_arg("a year of no days").with_field("year_length"));
        }
        Ok(FirdariaDasha {
            order: firdaria_order(first, nodes),
            birth,
            year_days,
        })
    }

    /// The nine lords of a round, in order.
    #[must_use]
    pub const fn order(&self) -> [Graha; FIRDARIA_PERIODS] {
        self.order
    }

    /// The days of one round.
    fn round_days(&self) -> f64 {
        f64::from(FIRDARIA_YEARS) * self.year_days
    }

    /// The days of one cycle, its rounds together.
    fn cycle_days(&self) -> f64 {
        #[expect(clippy::cast_precision_loss, reason = "two rounds")]
        let rounds = FIRDARIA_ROUNDS as f64;
        rounds * self.round_days()
    }

    /// The years of the periods before `index` in its round.
    fn years_before(&self, index: usize) -> u32 {
        self.order
            .iter()
            .take(index % FIRDARIA_PERIODS)
            .map(|&lord| firdar_years(lord))
            .sum()
    }
}

impl Timeline for FirdariaDasha {
    fn breadth(&self) -> usize {
        DESCENDING.len()
    }

    /// The `index`th period of `cycle`, two rounds of nine a cycle; the
    /// cycles never end.
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        if index >= FIRDARIA_PERIODS * FIRDARIA_ROUNDS {
            return None;
        }
        let lord = *self.order.get(index % FIRDARIA_PERIODS)?;
        #[expect(clippy::cast_precision_loss, reason = "a round index below 2")]
        let round = (index / FIRDARIA_PERIODS) as f64;
        let from = self.birth.get()
            + f64::from(cycle) * self.cycle_days()
            + round * self.round_days()
            + f64::from(self.years_before(index)) * self.year_days;
        let span = Interval::literal(from, from + f64::from(firdar_years(lord)) * self.year_days);
        Some(Period::of_share(
            lord,
            None,
            Path::root(cycle, u8::try_from(index).ok()?),
            span,
            span,
            0,
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
        (0..FIRDARIA_PERIODS * FIRDARIA_ROUNDS)
            .filter_map(|index| self.mahadasha(cycle, index))
            .find(|period| instant < period.interval.to.get())
    }

    /// The `index`th seventh of a planet's period, from its lord in
    /// descending order; a node's period and a seventh have none (§439).
    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        if parent.path.depth() > 1 || index >= DESCENDING.len() {
            return None;
        }
        let from = DESCENDING.iter().position(|&graha| graha == parent.lord)?;
        let lord = *DESCENDING.get((from + index) % DESCENDING.len())?;
        let whole = parent.interval;
        #[expect(clippy::cast_precision_loss, reason = "a seventh's index below 7")]
        let (at, sevenths) = (index as f64, DESCENDING.len() as f64);
        let seventh = whole.days() / sevenths;
        let start = whole.from.get() + at * seventh;
        let span = Interval::literal(start, start + seventh);
        Some(Period::of_share(
            lord,
            None,
            parent.path.child(u8::try_from(index).ok()?)?,
            span,
            span,
            0,
        ))
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking, index what they built, and compare exact sums"
)]
mod tests {
    use teistro_core::quantity::Depth;

    use super::*;

    const BIRTH: f64 = 2_451_545.0;
    const YEAR: f64 = 365.25;

    fn dasha(first: Graha, nodes: FirdariaNodes) -> FirdariaDasha {
        FirdariaDasha::new(
            first,
            JulianDay::literal(BIRTH),
            YearLength::Julian36525,
            nodes,
        )
        .unwrap()
    }

    /// The lords and years of the first round.
    fn round(dasha: &FirdariaDasha) -> Vec<(Graha, f64)> {
        (0..FIRDARIA_PERIODS)
            .map(|index| {
                let period = dasha.mahadasha(0, index).unwrap();
                (period.lord, period.interval.days() / YEAR)
            })
            .collect()
    }

    fn close(left: f64, right: f64) -> bool {
        (left - right).abs() < 1e-9
    }

    /// §438, the day table: Sun 10, Venus 8, Mercury 13, Moon 9, Saturn
    /// 11, Jupiter 12, Mars 7, Head 3, Tail 2, 75 years in all.
    #[test]
    fn al_biruni_day_table() {
        let day = round(&dasha(Graha::Sun, FirdariaNodes::End));
        let expected = [
            (Graha::Sun, 10.0),
            (Graha::Venus, 8.0),
            (Graha::Mercury, 13.0),
            (Graha::Moon, 9.0),
            (Graha::Saturn, 11.0),
            (Graha::Jupiter, 12.0),
            (Graha::Mars, 7.0),
            (Graha::Rahu, 3.0),
            (Graha::Ketu, 2.0),
        ];
        for ((lord, years), (want, want_years)) in day.iter().zip(expected) {
            assert_eq!(*lord, want);
            assert!(close(*years, want_years), "{lord:?} {years}");
        }
        assert!(close(day.iter().map(|(_, years)| years).sum(), 75.0));
    }

    /// §438, the night table, and Abu Ma'shar's "year 71" for the Head.
    #[test]
    fn al_biruni_night_table_and_the_head_at_year_71() {
        let night = dasha(Graha::Moon, FirdariaNodes::End);
        let lords: Vec<Graha> = round(&night).iter().map(|(lord, _)| *lord).collect();
        assert_eq!(
            lords,
            [
                Graha::Moon,
                Graha::Saturn,
                Graha::Jupiter,
                Graha::Mars,
                Graha::Sun,
                Graha::Venus,
                Graha::Mercury,
                Graha::Rahu,
                Graha::Ketu,
            ]
        );
        let head = night.mahadasha(0, 7).unwrap();
        assert!(close((head.interval.from.get() - BIRTH) / YEAR, 70.0));
    }

    /// Bonatti's order puts the nodes after Mars in both sects; by day
    /// Mars is last, so only the night changes (C225).
    #[test]
    fn bonatti_puts_the_nodes_after_mars() {
        assert_eq!(
            firdaria_order(Graha::Moon, FirdariaNodes::AfterMars),
            [
                Graha::Moon,
                Graha::Saturn,
                Graha::Jupiter,
                Graha::Mars,
                Graha::Rahu,
                Graha::Ketu,
                Graha::Sun,
                Graha::Venus,
                Graha::Mercury,
            ]
        );
        assert_eq!(
            firdaria_order(Graha::Sun, FirdariaNodes::AfterMars),
            firdaria_order(Graha::Sun, FirdariaNodes::End)
        );
    }

    /// §§438–439, the sevenths: each from the period's lord in descending
    /// order, each years/7; the printed times agree in years, months and
    /// days throughout, and in hours for the Sun, the Moon, Saturn and
    /// Mars (Venus's, Mercury's and Jupiter's printed hours differ).
    #[test]
    fn al_biruni_sevenths() {
        // (lord, months, days, hours or None where the print differs)
        let printed = [
            (Graha::Sun, 5, 4, Some(7)),
            (Graha::Moon, 3, 12, Some(21)),
            (Graha::Venus, 1, 21, None),
            (Graha::Saturn, 6, 25, Some(17)),
            (Graha::Mercury, 10, 8, None),
            (Graha::Jupiter, 8, 17, None),
            (Graha::Mars, 0, 0, Some(0)),
        ];
        for (lord, months, days, hours) in printed {
            let years = f64::from(firdar_years(lord)) / 7.0;
            // One year and the rest in months of 30 days, as printed.
            let rest = (years - 1.0) * 12.0;
            assert_eq!(rest.floor(), f64::from(months), "{lord:?} months");
            let day_rest = rest.fract() * 30.0;
            assert_eq!(day_rest.floor(), f64::from(days), "{lord:?} days");
            if let Some(hours) = hours {
                assert_eq!(
                    (day_rest.fract() * 24.0).round(),
                    f64::from(hours),
                    "{lord:?}"
                );
            }
        }
        let day = dasha(Graha::Sun, FirdariaNodes::End);
        let sun = day.mahadasha(0, 0).unwrap();
        let sevenths: Vec<Graha> = (0..7)
            .map(|index| day.child(&sun, index).unwrap().lord)
            .collect();
        assert_eq!(
            sevenths,
            [
                Graha::Sun,
                Graha::Venus,
                Graha::Mercury,
                Graha::Moon,
                Graha::Saturn,
                Graha::Jupiter,
                Graha::Mars,
            ]
        );
        let moon = day.mahadasha(0, 3).unwrap();
        assert_eq!(day.child(&moon, 1).unwrap().lord, Graha::Saturn);
        let first = day.child(&sun, 0).unwrap();
        assert!(close(first.interval.days(), 10.0 * YEAR / 7.0));
        assert!(close(
            day.child(&sun, 6).unwrap().interval.to.get(),
            sun.interval.to.get()
        ));
    }

    /// §439: the nodes' periods are not divided.
    #[test]
    fn the_nodes_have_no_sevenths() {
        let day = dasha(Graha::Sun, FirdariaNodes::End);
        for index in [7, 8] {
            let node = day.mahadasha(0, index).unwrap();
            assert!(day.child(&node, 0).is_none(), "{:?}", node.lord);
        }
        let chain = day.at(
            JulianDay::literal(BIRTH + 72.0 * YEAR),
            Depth::try_new(2).unwrap(),
        );
        assert_eq!(chain.iter().count(), 1);
    }

    /// IV.1: after 75 years "it returns to the Sun", and the cursor goes on
    /// past the two rounds a cycle stores.
    #[test]
    fn the_round_returns_to_the_sun() {
        let day = dasha(Graha::Sun, FirdariaNodes::End);
        for year in [75.5, 150.5, 225.5] {
            let chain = day.at(JulianDay::literal(BIRTH + year * YEAR), Depth::MIN);
            assert_eq!(chain.iter().next().unwrap().lord, Graha::Sun, "{year}");
        }
        assert_eq!(day.mahadasha(0, FIRDARIA_PERIODS).unwrap().lord, Graha::Sun);
    }

    #[test]
    fn only_a_luminary_begins_them() {
        let refused = FirdariaDasha::new(
            Graha::Mars,
            JulianDay::literal(BIRTH),
            YearLength::Julian36525,
            FirdariaNodes::End,
        )
        .unwrap_err();
        assert_eq!(refused.field(), Some("first"));
    }
}

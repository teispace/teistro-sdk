//! Valens's decennials as a kernel behind [`Timeline`]
//! (`03-design/hellenistic-decennials.md`).
//!
//! *Anthologies* VI.5–6: each of the seven rules 10 years 9 months, 129
//! months, from the apheta (the luminary of the sect) and then "the next
//! star in the zodiacal circle at the nativity", the seven by their places
//! "by sign and by degree". A star's 129 months are shared from itself in
//! the same order, each star taking its minimum years as months, and the
//! third stage divides again in the same proportion. The periods run in
//! 360-day years.
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_core::quantity::{Depth, JulianDay};
//! use teistro_core::settings::{DecennialDivision, YearLength};
//! use teistro_dasha::{DecennialDasha, Timeline, decennial_order};
//!
//! // Valens's night nativity (VI.5): from the Moon in Pisces, the next
//! // stars by longitude are Venus, Jupiter, Saturn and Mars.
//! let places = [
//!     (Graha::Moon, 348.0),
//!     (Graha::Venus, 10.0),
//!     (Graha::Jupiter, 195.0),
//!     (Graha::Saturn, 255.0),
//!     (Graha::Mars, 301.0),
//!     (Graha::Sun, 315.0),
//!     (Graha::Mercury, 340.0),
//! ];
//! let order = decennial_order(Graha::Moon, &places)?;
//! let birth = JulianDay::literal(2_451_545.0);
//! let dasha = DecennialDasha::new(
//!     order,
//!     birth,
//!     YearLength::Savana360,
//!     DecennialDivision::Proportional,
//! )?;
//! // Four cycles are 43 years, and Mars's begins.
//! let mars = dasha.mahadasha(0, 4).ok_or("a fifth period")?;
//! assert_eq!(mars.lord, Graha::Mars);
//! assert_eq!(mars.interval.from.get() - birth.get(), 43.0 * 360.0);
//! # let _ = Depth::MIN;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::{DecennialDivision, YearLength};

use crate::releasing::minimum_years;
use crate::tree::{Path, Period, Timeline};

/// The stars that take periods: the seven.
pub const DECENNIAL_STARS: usize = 7;

/// The months a star rules, 10 years 9 months, which are also the sum of
/// the seven's minimum years.
pub const DECENNIAL_MONTHS: u32 = 129;

/// The rounds of the seven one cycle holds: two, 150 years 6 months of
/// 360-day years, so that a stored reading answers the period at any age
/// a life reaches; the cursor carries on past them.
pub const DECENNIAL_ROUNDS: usize = 2;

/// The days of a cycle at the third level under
/// [`DecennialDivision::Cycles`]: 129, one to a minimum year of each star.
const CYCLE_DAYS: f64 = 129.0;

/// The seven, which an order must name once each.
const SEVEN: [Graha; DECENNIAL_STARS] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// The seven in the order the decennials run: `apheta` first, then the
/// others by their longitude onwards from it, "by sign and by degree"
/// (VI.7). `places` gives each of the seven its longitude in degrees.
///
/// # Errors
///
/// An `apheta` outside the seven, named `apheta`; `places` that do not
/// name each of the seven once with a finite longitude, named `places`.
pub fn decennial_order(
    apheta: Graha,
    places: &[(Graha, f64)],
) -> Result<[Graha; DECENNIAL_STARS], Error> {
    let refuse = || {
        Error::invalid_arg("the decennials need each of the seven placed once").with_field("places")
    };
    let mut ranked = [(Graha::Sun, 0.0); DECENNIAL_STARS];
    for (slot, star) in ranked.iter_mut().zip(SEVEN) {
        let mut found = places.iter().filter(|(graha, _)| *graha == star);
        let (Some(&(_, longitude)), None) = (found.next(), found.next()) else {
            return Err(refuse());
        };
        if !longitude.is_finite() {
            return Err(refuse());
        }
        *slot = (star, longitude);
    }
    if places.len() != DECENNIAL_STARS {
        return Err(refuse());
    }
    let Some(&(_, origin)) = ranked.iter().find(|(graha, _)| *graha == apheta) else {
        return Err(Error::invalid_arg(format!(
            "the decennials begin from one of the seven, not {apheta:?}"
        ))
        .with_field("apheta"));
    };
    let onwards = |longitude: f64| (longitude - origin).rem_euclid(360.0);
    ranked.sort_by(|left, right| onwards(left.1).total_cmp(&onwards(right.1)));
    // The apheta sorts first at zero; a star at its very degree keeps the
    // order of the seven after it.
    let at = ranked
        .iter()
        .position(|(graha, _)| *graha == apheta)
        .unwrap_or_default();
    if let Some(head) = ranked.get_mut(..=at) {
        head.rotate_right(1);
    }
    Ok(ranked.map(|(graha, _)| graha))
}

/// The decennials of a birth: the seven's 129 months each, in the order
/// they run, repeating.
#[derive(Clone, Debug, PartialEq)]
pub struct DecennialDasha {
    order: [Graha; DECENNIAL_STARS],
    birth: JulianDay<Utc>,
    year_days: f64,
    division: DecennialDivision,
}

impl DecennialDasha {
    /// The decennials running in `order` (from [`decennial_order`], or any
    /// order of the seven a consumer chooses), a year being `year_length`,
    /// the levels below the second divided as `division` says.
    ///
    /// # Errors
    ///
    /// An `order` that does not name each of the seven once, named
    /// `order`; a year of no days, named `year_length`.
    pub fn new(
        order: [Graha; DECENNIAL_STARS],
        birth: JulianDay<Utc>,
        year_length: YearLength,
        division: DecennialDivision,
    ) -> Result<DecennialDasha, Error> {
        if !SEVEN.iter().all(|star| order.contains(star)) {
            return Err(
                Error::invalid_arg("the decennials run through each of the seven once")
                    .with_field("order"),
            );
        }
        let year_days = year_length.days();
        if year_days.is_nan() || year_days <= 0.0 {
            return Err(Error::invalid_arg("a year of no days").with_field("year_length"));
        }
        Ok(DecennialDasha {
            order,
            birth,
            year_days,
            division,
        })
    }

    /// The seven in the order they run.
    #[must_use]
    pub const fn order(&self) -> [Graha; DECENNIAL_STARS] {
        self.order
    }

    /// How the levels below the second divide.
    #[must_use]
    pub const fn division(&self) -> DecennialDivision {
        self.division
    }

    /// The days of one star's period: 129 months, each a twelfth of the
    /// year.
    fn period_days(&self) -> f64 {
        f64::from(DECENNIAL_MONTHS) / 12.0 * self.year_days
    }

    /// The days of one cycle, its rounds together.
    fn cycle_days(&self) -> f64 {
        #[expect(clippy::cast_precision_loss, reason = "fourteen periods")]
        let periods = (DECENNIAL_STARS * DECENNIAL_ROUNDS) as f64;
        periods * self.period_days()
    }

    /// The days a child's share runs, and whether it is cut at its
    /// parent's end rather than a proportion of it.
    fn share_days(&self, parent: &Period, lord: Graha) -> (f64, bool) {
        let years = f64::from(minimum_years(lord));
        let proportion = (
            parent.interval.days() * years / f64::from(DECENNIAL_MONTHS),
            false,
        );
        match (self.division, parent.path.depth()) {
            (DecennialDivision::Cycles, 2) => (CYCLE_DAYS, true),
            (DecennialDivision::Cycles, 3) => (years, true),
            _ => proportion,
        }
    }
}

impl Timeline for DecennialDasha {
    fn breadth(&self) -> usize {
        DECENNIAL_STARS
    }

    /// The `index`th period of `cycle`, two rounds of the seven a cycle;
    /// the cycles never end.
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        if index >= DECENNIAL_STARS * DECENNIAL_ROUNDS {
            return None;
        }
        let seat = index % DECENNIAL_STARS;
        let lord = *self.order.get(seat)?;
        #[expect(clippy::cast_precision_loss, reason = "an index below 14")]
        let before = index as f64;
        let from =
            self.birth.get() + f64::from(cycle) * self.cycle_days() + before * self.period_days();
        let span = Interval::literal(from, from + self.period_days());
        Some(Period::of_share(
            lord,
            None,
            Path::root(cycle, u8::try_from(index).ok()?),
            span,
            span,
            seat,
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
        (0..DECENNIAL_STARS * DECENNIAL_ROUNDS)
            .filter_map(|index| self.mahadasha(cycle, index))
            .find(|period| instant < period.interval.to.get())
    }

    /// The `index`th share of `parent`, from its lord onwards in the
    /// order: each star's minimum years in proportion, or, under
    /// [`DecennialDivision::Cycles`], 129-day cycles at the third level
    /// and the minimum years as days at the fourth, each cut at the
    /// parent's end (VI.5).
    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        if index >= DECENNIAL_STARS {
            return None;
        }
        // The order names each star once, so a lord's place is its seat.
        let from = self.order.iter().position(|&star| star == parent.lord)?;
        let seat_of = |step: usize| (from + step) % DECENNIAL_STARS;
        let lord_of = |step: usize| self.order.get(seat_of(step)).copied();
        let lord = lord_of(index)?;
        let mut start = parent.interval.from.get();
        for step in 0..index {
            start += self.share_days(parent, lord_of(step)?).0;
        }
        let (days, cut) = self.share_days(parent, lord);
        let end = parent.interval.to.get();
        if cut && start >= end {
            return None;
        }
        let stop = if cut {
            (start + days).min(end)
        } else {
            start + days
        };
        let span = Interval::literal(start, stop);
        Some(Period::of_share(
            lord,
            None,
            parent.path.child(u8::try_from(index).ok()?)?,
            span,
            span,
            seat_of(index),
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
    const YEAR: f64 = 360.0;
    const MONTH: f64 = 30.0;

    /// Valens's night nativity (VI.5): the Moon in Pisces 18°, Venus in
    /// Aries, Jupiter in Libra, Saturn in Sagittarius, Mars early in
    /// Aquarius, the Sun in Aquarius and Mercury in Pisces before the Moon.
    const VALENS: [(Graha, f64); 7] = [
        (Graha::Moon, 348.0),
        (Graha::Venus, 10.0),
        (Graha::Jupiter, 195.0),
        (Graha::Saturn, 255.0),
        (Graha::Mars, 301.0),
        (Graha::Sun, 315.0),
        (Graha::Mercury, 340.0),
    ];

    fn valens(division: DecennialDivision) -> DecennialDasha {
        let order = decennial_order(Graha::Moon, &VALENS).unwrap();
        DecennialDasha::new(
            order,
            JulianDay::literal(BIRTH),
            YearLength::Savana360,
            division,
        )
        .unwrap()
    }

    fn close(left: f64, right: f64) -> bool {
        (left - right).abs() < 1e-6
    }

    fn depth(levels: u8) -> Depth {
        Depth::try_new(levels).unwrap()
    }

    /// The minimum years are the 129 months a star rules.
    #[test]
    fn the_minimum_years_fill_a_period() {
        let sum: u32 = SEVEN.iter().map(|&star| minimum_years(star)).sum();
        assert_eq!(sum, DECENNIAL_MONTHS);
    }

    /// VI.5: from the Moon the order runs Venus, Jupiter, Saturn, Mars, the
    /// Sun and Mercury; four cycles are 43 years, and Mars's begins.
    #[test]
    fn four_cycles_are_forty_three_years() {
        let dasha = valens(DecennialDivision::Proportional);
        assert_eq!(
            dasha.order(),
            [
                Graha::Moon,
                Graha::Venus,
                Graha::Jupiter,
                Graha::Saturn,
                Graha::Mars,
                Graha::Sun,
                Graha::Mercury,
            ]
        );
        for index in 0..DECENNIAL_STARS * DECENNIAL_ROUNDS {
            let period = dasha.mahadasha(0, index).unwrap();
            assert!(close(period.interval.days(), 10.75 * YEAR));
        }
        let mars = dasha.mahadasha(0, 4).unwrap();
        assert_eq!(mars.lord, Graha::Mars);
        assert!(close(mars.interval.from.get() - BIRTH, 43.0 * YEAR));
        assert!(dasha.mahadasha(0, 14).is_none());
        // The cycle goes on from the Moon after its two rounds.
        let next = dasha.mahadasha(1, 0).unwrap();
        assert_eq!(next.lord, Graha::Moon);
        assert!(close(
            next.interval.from.get() - BIRTH,
            2.0 * 7.0 * 10.75 * YEAR
        ));
    }

    /// VI.5: Mars's 129 months give Mars 15 (44 years 3 months), the Sun
    /// 19, Mercury 20, the Moon 25, Venus 8, Jupiter 12 (51 years 3
    /// months) and Saturn 30 (53 years 9 months).
    #[test]
    fn marss_months_are_the_minimum_years() {
        let dasha = valens(DecennialDivision::Proportional);
        let mars = dasha.mahadasha(0, 4).unwrap();
        let shares: Vec<(Graha, f64)> = dasha
            .children(&mars)
            .map(|child| (child.lord, (child.interval.to.get() - BIRTH) / MONTH))
            .collect();
        let expected = [
            (Graha::Mars, 44.0 * 12.0 + 3.0),
            (Graha::Sun, 45.0 * 12.0 + 10.0),
            (Graha::Mercury, 47.0 * 12.0 + 6.0),
            (Graha::Moon, 49.0 * 12.0 + 7.0),
            (Graha::Venus, 50.0 * 12.0 + 3.0),
            (Graha::Jupiter, 51.0 * 12.0 + 3.0),
            (Graha::Saturn, 53.0 * 12.0 + 9.0),
        ];
        assert_eq!(shares.len(), expected.len());
        for ((lord, end), (want, months)) in shares.iter().zip(expected) {
            assert_eq!(*lord, want);
            assert!(close(*end, months), "{lord:?} ends at {end} months");
        }
    }

    /// VI.5's date: 52 years and 124 days is 667 days into Saturn's 30
    /// months. Counted in cycles, five of 129 days (Saturn, Mars, the Sun,
    /// Mercury, the Moon) leave 22 in Venus's, which gives Venus 8 days,
    /// Jupiter 12 and Saturn the last 2; in proportion the day falls in
    /// the Moon's share (C228).
    #[test]
    fn the_six_hundred_sixty_seventh_day() {
        let elapsed = 52.0 * 365.25 + 124.0;
        let saturn_from = 51.25 * YEAR;
        assert!(close(elapsed - saturn_from, 667.0));
        let instant = JulianDay::literal(BIRTH + elapsed - 0.5);
        let lords = |division| {
            valens(division)
                .at(instant, depth(4))
                .iter()
                .map(|period| period.lord)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            lords(DecennialDivision::Cycles),
            [Graha::Mars, Graha::Saturn, Graha::Venus, Graha::Saturn]
        );
        assert_eq!(
            lords(DecennialDivision::Proportional)[..3],
            [Graha::Mars, Graha::Saturn, Graha::Moon]
        );
        // The seventh cycle in Saturn's 900 days is cut at 126.
        let dasha = valens(DecennialDivision::Cycles);
        let mars = dasha.mahadasha(0, 4).unwrap();
        let saturn = dasha.child(&mars, 6).unwrap();
        let cycles: Vec<f64> = dasha
            .children(&saturn)
            .map(|cycle| cycle.interval.days())
            .collect();
        assert_eq!(cycles.len(), 7);
        assert!(cycles[..6].iter().all(|&days| close(days, 129.0)));
        assert!(close(cycles[6], 900.0 - 6.0 * 129.0));
    }

    /// VI.6, the third stage: a star of `a` minimum years gives `b`'s
    /// share `a·b/129` months of 30 days. The table prints 49 cells as
    /// months, days and hours; 44 agree within a quarter of an hour. The
    /// five that do not disagree with their own twins, since a to b must
    /// equal b to a, so they are printing slips: Saturn to itself,
    /// Jupiter to the Sun and the Sun to Jupiter, Venus to Mercury, and
    /// Mercury to itself.
    #[test]
    fn the_third_stage_divides_in_proportion() {
        use Graha::{
            Jupiter as Ju, Mars as Ma, Mercury as Me, Moon as Mo, Saturn as Sa, Sun as Su,
            Venus as Ve,
        };
        // (giver, receiver, months, days, hours) as printed.
        let printed: [(Graha, Graha, f64, f64, f64); 49] = [
            (Sa, Sa, 6.0, 29.0, 6.25),
            (Sa, Ju, 2.0, 23.0, 52.0 / 3.0),
            (Sa, Ma, 3.0, 14.0, 47.0 / 3.0),
            (Sa, Su, 4.0, 12.0, 161.0 / 12.0),
            (Sa, Ve, 1.0, 25.0, 19.5),
            (Sa, Me, 4.0, 19.0, 38.0 / 3.0),
            (Sa, Mo, 5.0, 24.0, 10.0),
            (Ju, Ju, 1.0, 3.0, 11.75),
            (Ju, Ma, 1.0, 11.0, 62.0 / 3.0),
            (Ju, Su, 1.0, 23.0, 20.5),
            (Ju, Ve, 0.0, 22.0, 47.0 / 6.0),
            (Ju, Me, 1.0, 25.0, 19.5),
            (Ju, Mo, 2.0, 9.0, 55.0 / 3.0),
            (Ju, Sa, 2.0, 23.0, 17.25),
            (Ma, Ma, 1.0, 22.0, 23.0 / 3.0),
            (Ma, Su, 2.0, 6.0, 20.0 / 3.0),
            (Ma, Ve, 0.0, 27.0, 131.0 / 6.0),
            (Ma, Me, 2.0, 9.0, 55.0 / 3.0),
            (Ma, Mo, 2.0, 27.0, 5.0),
            (Ma, Sa, 3.0, 14.0, 47.0 / 3.0),
            (Ma, Ju, 1.0, 11.0, 62.0 / 3.0),
            (Su, Su, 2.0, 23.0, 22.75),
            (Su, Ve, 1.0, 5.0, 25.0 / 3.0),
            (Su, Me, 2.0, 28.0, 107.0 / 12.0),
            (Su, Mo, 3.0, 20.0, 11.25),
            (Su, Sa, 4.0, 12.0, 13.25),
            (Su, Ju, 1.0, 23.0, 8.5),
            (Su, Ma, 2.0, 6.0, 20.0 / 3.0),
            (Ve, Ve, 0.0, 14.0, 21.2),
            (Ve, Me, 1.0, 7.0, 4.25),
            (Ve, Mo, 1.0, 16.0, 12.25),
            (Ve, Sa, 1.0, 25.0, 19.5),
            (Ve, Ju, 0.0, 22.0, 23.0 / 3.0),
            (Ve, Ma, 0.0, 27.0, 21.75),
            (Ve, Su, 1.0, 5.0, 25.0 / 3.0),
            (Me, Me, 3.0, 3.0, 9.5),
            (Me, Mo, 3.0, 26.0, 6.75),
            (Me, Sa, 4.0, 19.0, 12.75),
            (Me, Ju, 1.0, 25.0, 19.5),
            (Me, Ma, 2.0, 9.0, 221.0 / 12.0),
            (Me, Su, 2.0, 28.0, 107.0 / 12.0),
            (Me, Ve, 1.0, 7.0, 5.0),
            (Mo, Mo, 4.0, 25.0, 8.25),
            (Mo, Sa, 5.0, 24.0, 10.0),
            (Mo, Ju, 2.0, 9.0, 221.0 / 12.0),
            (Mo, Ma, 2.0, 27.0, 5.0),
            (Mo, Su, 3.0, 20.0, 67.0 / 6.0),
            (Mo, Ve, 1.0, 16.0, 12.25),
            (Mo, Me, 3.0, 26.0, 6.75),
        ];
        let slips = [(Sa, Sa), (Ju, Su), (Su, Ju), (Ve, Me), (Me, Me)];
        let dasha = valens(DecennialDivision::Proportional);
        // Every second-level period of the first round, by its lord.
        let second: Vec<Period> = (0..DECENNIAL_STARS)
            .flat_map(|index| {
                dasha
                    .children(&dasha.mahadasha(0, index).unwrap())
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut agree = 0;
        for (giver, receiver, months, days, hours) in printed {
            let parent = second.iter().find(|period| period.lord == giver).unwrap();
            let share = dasha
                .children(parent)
                .find(|child| child.lord == receiver)
                .unwrap();
            let printed_hours = (months * MONTH + days) * 24.0 + hours;
            let off = (share.interval.days() * 24.0 - printed_hours).abs();
            let slip = slips.contains(&(giver, receiver));
            assert_eq!(
                off > 0.25,
                slip,
                "{giver:?} to {receiver:?} is {off} hours off"
            );
            agree += usize::from(!slip);
        }
        assert_eq!(agree, 44);
        // The shares fill their parent exactly.
        for parent in &second {
            let filled: f64 = dasha
                .children(parent)
                .map(|child| child.interval.days())
                .sum();
            assert!(close(filled, parent.interval.days()));
        }
    }

    /// The order counts on from the apheta round the circle, and refuses
    /// what is not the seven.
    #[test]
    fn the_order_is_the_zodiacs_and_refuses_the_rest() {
        let from_sun = decennial_order(Graha::Sun, &VALENS).unwrap();
        assert_eq!(from_sun[..3], [Graha::Sun, Graha::Mercury, Graha::Moon]);
        assert_eq!(
            decennial_order(Graha::Rahu, &VALENS).unwrap_err().field(),
            Some("apheta")
        );
        assert_eq!(
            decennial_order(Graha::Sun, &VALENS[..6])
                .unwrap_err()
                .field(),
            Some("places")
        );
        let mut twice = VALENS;
        twice[1] = (Graha::Moon, 1.0);
        assert!(decennial_order(Graha::Moon, &twice).is_err());
        let mut repeated = decennial_order(Graha::Moon, &VALENS).unwrap();
        repeated[1] = Graha::Moon;
        let refused = DecennialDasha::new(
            repeated,
            JulianDay::literal(BIRTH),
            YearLength::Savana360,
            DecennialDivision::Proportional,
        );
        assert_eq!(refused.unwrap_err().field(), Some("order"));
    }
}

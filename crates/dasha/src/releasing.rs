//! Valens's time lords as kernels behind [`Timeline`]: releasing from a
//! lot, and the profected year (`03-design/hellenistic-time-lords.md`).
//!
//! **Releasing** (*Anthologies* IV.4, IV.6–IV.10). A sign allots the
//! minimum years of its lord, Capricorn 27 and Aquarius 30. Each level is a
//! twelfth of the one above. A level begins at its parent's own sign, runs
//! forward, and is cut at the parent's end. Once the twelve signs are
//! spent (211 units), the rest is loosed to the sign opposite the parent's.
//!
//! **Profection** (IV.11). The ordinal year *n* falls *n − 1* signs on
//! from the start, one sign a year, and the count never ends.
//!
//! ```
//! use teistro_core::catalogue::Rashi;
//! use teistro_core::quantity::{Depth, JulianDay};
//! use teistro_core::settings::YearLength;
//! use teistro_dasha::{ReleasingDasha, Timeline};
//!
//! // IV.8: the Lot of Fortune in Leo. Leo 19 years, Virgo 20, Libra 8 and
//! // Scorpio 15 make 62, so the 70th year is Sagittarius's.
//! let birth = JulianDay::literal(2_451_545.0);
//! let dasha = ReleasingDasha::new(Rashi::Leo, birth, YearLength::Savana360)?;
//! let seventieth = JulianDay::literal(2_451_545.0 + 69.5 * 360.0);
//! let chain = dasha.at(seventieth, Depth::try_new(1)?);
//! assert_eq!(chain.iter().next().and_then(|period| period.sign), Some(Rashi::Sagittarius));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use teistro_core::catalogue::{DashaFamily, DashaSystem, Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::YearLength;

use crate::rashi::{Direction, step};
use crate::reading::DashaCursor;
use crate::tree::{MAX_DEPTH, Path, Period, Timeline};

/// The signs of a whole cycle.
const SIGNS: usize = 12;

/// The units a cycle of the twelve signs allots: 17 years 7 months at the
/// second level (IV.4), 528 days at the third and 44 at the fourth (IV.10).
pub const RELEASING_CYCLE: u32 = 211;

/// The most children a releasing period can have: Aquarius's 360 months
/// hold the twelve, then nine more after the loosing.
const RELEASING_BREADTH: usize = 24;

/// The years a sign allots in releasing: its lord's minimum years (IV.1),
/// except Capricorn's 27 and Aquarius's 30 (IV.6).
#[must_use]
pub fn releasing_years(sign: Rashi) -> u32 {
    match sign {
        Rashi::Capricorn => 27,
        Rashi::Aquarius => 30,
        _ => minimum_years(sign.attributes().lord),
    }
}

/// A planet's minimum years (*Anthologies* IV.1): Saturn 30, Jupiter 12,
/// Mars 15, Venus 8, Mercury 20, the Sun 19 and the Moon 25, 129 in all;
/// a node has none. Releasing gives them to signs, and the decennials
/// share their 129 months by them.
#[must_use]
pub const fn minimum_years(planet: Graha) -> u32 {
    match planet {
        Graha::Saturn => 30,
        Graha::Jupiter => 12,
        Graha::Mars => 15,
        Graha::Venus => 8,
        Graha::Mercury => 20,
        Graha::Sun => 19,
        Graha::Moon => 25,
        _ => 0,
    }
}

/// A sign's lord, which a period of the sign names.
fn lord(sign: Rashi) -> Graha {
    sign.attributes().lord
}

/// The days in a year of `length`, refused when there are none.
fn year_days(length: YearLength) -> Result<f64, Error> {
    let days = length.days();
    if days.is_nan() || days <= 0.0 {
        return Err(Error::invalid_arg("a year of no days").with_field("year_length"));
    }
    Ok(days)
}

/// Valens's releasing from a sign: the vital sector begun from a lot.
#[derive(Clone, Debug, PartialEq)]
pub struct ReleasingDasha {
    start: Rashi,
    birth: JulianDay<Utc>,
    year_days: f64,
}

impl ReleasingDasha {
    /// Releasing from `start` at `birth`, a period year being `year_length`
    /// (Valens's is 360 days, IV.9).
    ///
    /// # Errors
    ///
    /// A year length of no days, named `year_length`.
    pub fn new(
        start: Rashi,
        birth: JulianDay<Utc>,
        year_length: YearLength,
    ) -> Result<ReleasingDasha, Error> {
        Ok(ReleasingDasha {
            start,
            birth,
            year_days: year_days(year_length)?,
        })
    }

    /// The sign it is released from.
    #[must_use]
    pub const fn start(&self) -> Rashi {
        self.start
    }

    /// The days one unit lasts at a level, 1 for the years: a twelfth of
    /// the level above, divided by a whole power of twelve so that no libm
    /// stands between a level and Valens's arithmetic.
    fn unit_days(&self, level: usize) -> f64 {
        let twelfths = u32::try_from(level.saturating_sub(1)).unwrap_or(u32::MAX);
        let parts = 12_u32
            .checked_pow(twelfths)
            .map_or(f64::INFINITY, f64::from);
        self.year_days / parts
    }

    /// The days of one first-level cycle of the twelve signs.
    fn cycle_days(&self) -> f64 {
        f64::from(RELEASING_CYCLE) * self.year_days
    }

    /// The `index`th sign of a level begun from `from`: the twelve in
    /// order, then the rest loosed to the sign opposite `from` (IV.4).
    fn sign_at(from: Rashi, index: usize) -> Rashi {
        if index < SIGNS {
            step(from, Direction::Forward, index)
        } else {
            step(from.opposite(), Direction::Forward, index - SIGNS)
        }
    }

    /// The units allotted before the `index`th sign of a level begun from
    /// `from`, counted whole so that no boundary accumulates rounding.
    fn units_before(from: Rashi, index: usize) -> u32 {
        (0..index)
            .map(|k| releasing_years(Self::sign_at(from, k)))
            .sum()
    }
}

impl Timeline for ReleasingDasha {
    fn breadth(&self) -> usize {
        RELEASING_BREADTH
    }

    /// The twelve signs from the start, each its years whole; past the
    /// first cycle, nothing (211 years, which no life reaches).
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        if cycle > 0 || index >= SIGNS {
            return None;
        }
        let sign = step(self.start, Direction::Forward, index);
        let days = |units: u32| self.birth.get() + f64::from(units) * self.year_days;
        let before = Self::units_before(self.start, index);
        Some(Period::of_sign(
            sign,
            lord(sign),
            Path::root(0, u8::try_from(index).ok()?),
            Interval::literal(days(before), days(before + releasing_years(sign))),
        ))
    }

    fn mahadasha_at(&self, instant: f64) -> Option<Period> {
        let into = instant - self.birth.get();
        if into < 0.0 || into >= self.cycle_days() {
            return None;
        }
        (0..SIGNS)
            .filter_map(|index| self.mahadasha(0, index))
            .find(|period| instant < period.interval.to.get())
    }

    /// The `index`th period below `parent`, begun at the parent's sign,
    /// loosed after twelve, cut at the parent's end; nothing once the
    /// parent is spent or below the deepest level.
    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        let from_sign = parent.sign?;
        if index >= RELEASING_BREADTH || parent.path.depth() >= MAX_DEPTH {
            return None;
        }
        let unit = self.unit_days(parent.path.depth() + 1);
        let whole = parent.interval;
        let from = whole.from.get() + f64::from(Self::units_before(from_sign, index)) * unit;
        if from >= whole.to.get() {
            return None;
        }
        let sign = Self::sign_at(from_sign, index);
        let to = (from + f64::from(releasing_years(sign)) * unit).min(whole.to.get());
        Some(Period::of_sign(
            sign,
            lord(sign),
            parent.path.child(u8::try_from(index).ok()?)?,
            Interval::literal(from, to),
        ))
    }
}

/// The years one cycle of the profected year holds: ten circuits of the
/// twelve signs, the life Vimshottari reckons, so that a stored reading
/// answers the year at any age a life reaches. Valens's count itself never
/// ends, and the cursor carries on past it.
pub const PROFECTION_YEARS: u8 = 120;

/// The profected year: one sign a year from a start, without end (IV.11).
#[derive(Clone, Debug, PartialEq)]
pub struct ProfectionDasha {
    start: Rashi,
    birth: JulianDay<Utc>,
    year_days: f64,
}

impl ProfectionDasha {
    /// The years profected from `start` at `birth`, a year being
    /// `year_length` (Valens's is 365¼ days, IV.9).
    ///
    /// # Errors
    ///
    /// A year length of no days, named `year_length`.
    pub fn new(
        start: Rashi,
        birth: JulianDay<Utc>,
        year_length: YearLength,
    ) -> Result<ProfectionDasha, Error> {
        Ok(ProfectionDasha {
            start,
            birth,
            year_days: year_days(year_length)?,
        })
    }

    /// The sign the count starts from.
    #[must_use]
    pub const fn start(&self) -> Rashi {
        self.start
    }

    /// The sign of the ordinal year `year`, 1 for the year that begins at
    /// birth: `year − 1` signs on (IV.11's 35th year, from Virgo, is
    /// Cancer); nothing for year 0, which does not exist.
    #[must_use]
    pub fn sign_of_year(&self, year: u32) -> Option<Rashi> {
        let elapsed = usize::try_from(year.checked_sub(1)?).ok()?;
        Some(step(self.start, Direction::Forward, elapsed))
    }
}

impl Timeline for ProfectionDasha {
    fn breadth(&self) -> usize {
        0
    }

    /// The `index`th year of `cycle`'s [`PROFECTION_YEARS`], so that a
    /// year's index is the native's completed age; the cycles never end.
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        if index >= usize::from(PROFECTION_YEARS) {
            return None;
        }
        let sign = step(self.start, Direction::Forward, index);
        let years =
            f64::from(cycle) * f64::from(PROFECTION_YEARS) + f64::from(u8::try_from(index).ok()?);
        let from = self.birth.get() + years * self.year_days;
        Some(Period::of_sign(
            sign,
            lord(sign),
            Path::root(cycle, u8::try_from(index).ok()?),
            Interval::literal(from, from + self.year_days),
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
            reason = "a non-negative whole number of years, far below u32::MAX for any date"
        )]
        let elapsed = (into / self.year_days).floor() as u32;
        let span = u32::from(PROFECTION_YEARS);
        self.mahadasha(elapsed / span, usize::try_from(elapsed % span).ok()?)
    }

    /// A year has no division here: Valens's months (IV.28) are read from
    /// transits, not cut from the year.
    fn child(&self, _parent: &Period, _index: usize) -> Option<Period> {
        None
    }
}

/// The catalogued systems these kernels run, in the catalogue's order.
pub const TIME_LORDS: [DashaSystem; 3] = [
    DashaSystem::ReleasingFortune,
    DashaSystem::ReleasingDaimon,
    DashaSystem::Profection,
];

/// The time lord a catalogued system names, begun from `start`: releasing
/// for the releasing family, the profected year for the profection family,
/// and nothing for a system of any other kernel.
///
/// # Errors
///
/// A year length of no days, named `year_length`.
pub fn time_lord(
    system: DashaSystem,
    start: Rashi,
    birth: JulianDay<Utc>,
    year_length: YearLength,
) -> Option<Result<DashaCursor, Error>> {
    match system.attributes().family {
        DashaFamily::Releasing => {
            Some(ReleasingDasha::new(start, birth, year_length).map(DashaCursor::Releasing))
        }
        DashaFamily::Profection => {
            Some(ProfectionDasha::new(start, birth, year_length).map(DashaCursor::Profection))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, index what they built, and compare \
                  whole months, which a double holds exactly"
    )]

    use teistro_core::quantity::Depth;

    use super::*;

    const BIRTH: f64 = 2_451_545.0;

    fn releasing(start: Rashi) -> ReleasingDasha {
        ReleasingDasha::new(start, JulianDay::literal(BIRTH), YearLength::Savana360).unwrap()
    }

    /// A period's start and end in months after birth, a month being 30
    /// days of the 360-day year.
    fn months(period: &Period) -> (f64, f64) {
        let month = |jd: JulianDay<Utc>| (jd.get() - BIRTH) / 30.0;
        (month(period.interval.from), month(period.interval.to))
    }

    fn level_one(dasha: &ReleasingDasha, sign: Rashi) -> Period {
        dasha.mahadashas().find(|p| p.sign == Some(sign)).unwrap()
    }

    /// Each child's sign and length in months.
    fn children(dasha: &ReleasingDasha, parent: &Period) -> Vec<(Rashi, f64)> {
        dasha
            .children(parent)
            .map(|child| {
                let (from, to) = months(&child);
                (child.sign.unwrap(), to - from)
            })
            .collect()
    }

    #[test]
    fn the_time_lords_are_the_kernels_their_family_names() {
        let birth = JulianDay::literal(BIRTH);
        for system in TIME_LORDS {
            let cursor = time_lord(system, Rashi::Leo, birth, YearLength::Savana360);
            assert!(cursor.is_some_and(|built| built.is_ok()), "{system:?}");
        }
        assert!(time_lord(DashaSystem::Chara, Rashi::Leo, birth, YearLength::Savana360).is_none());
    }

    #[test]
    fn a_sign_allots_its_lords_minimum_years_but_capricorn_and_aquarius() {
        let years: Vec<u32> = Rashi::ALL.into_iter().map(releasing_years).collect();
        assert_eq!(years, [15, 8, 20, 25, 19, 20, 8, 15, 12, 27, 30, 12]);
        assert_eq!(years.iter().sum::<u32>(), RELEASING_CYCLE);
    }

    /// IV.8: Fortune in Leo; the 70th year falls in Sagittarius, and inside
    /// it Sagittarius 1 year, Capricorn 2 years 3 months, Aquarius 2 years
    /// 6 months and Pisces 1 year, then Aries.
    #[test]
    fn valens_fortune_in_leo_reaches_sagittarius_in_the_seventieth_year() {
        let dasha = releasing(Rashi::Leo);
        let first: Vec<(Rashi, f64)> = dasha
            .mahadashas()
            .take(5)
            .map(|p| (p.sign.unwrap(), months(&p).0 / 12.0))
            .collect();
        assert_eq!(
            first,
            [
                (Rashi::Leo, 0.0),
                (Rashi::Virgo, 19.0),
                (Rashi::Libra, 39.0),
                (Rashi::Scorpio, 47.0),
                (Rashi::Sagittarius, 62.0),
            ]
        );
        let sagittarius = level_one(&dasha, Rashi::Sagittarius);
        assert_eq!(
            children(&dasha, &sagittarius)[..5],
            [
                (Rashi::Sagittarius, 12.0),
                (Rashi::Capricorn, 27.0),
                (Rashi::Aquarius, 30.0),
                (Rashi::Pisces, 12.0),
                (Rashi::Aries, 15.0),
            ]
        );
    }

    /// IV.8: Daimon in Scorpio; Aquarius from 54, and inside it eleven
    /// signs to Sagittarius, which ends at 69 years 4 months.
    #[test]
    fn valens_daimon_in_scorpio_ends_at_sixty_nine_years_four_months() {
        let dasha = releasing(Rashi::Scorpio);
        let aquarius = level_one(&dasha, Rashi::Aquarius);
        assert_eq!(months(&aquarius).0, 54.0 * 12.0);
        let inside = children(&dasha, &aquarius);
        assert_eq!(
            inside[..11],
            [
                (Rashi::Aquarius, 30.0),
                (Rashi::Pisces, 12.0),
                (Rashi::Aries, 15.0),
                (Rashi::Taurus, 8.0),
                (Rashi::Gemini, 20.0),
                (Rashi::Cancer, 25.0),
                (Rashi::Leo, 19.0),
                (Rashi::Virgo, 20.0),
                (Rashi::Libra, 8.0),
                (Rashi::Scorpio, 15.0),
                (Rashi::Sagittarius, 12.0),
            ]
        );
        let sagittarius = dasha.children(&aquarius).nth(10).unwrap();
        assert_eq!(months(&sagittarius).1, 69.0 * 12.0 + 4.0);
    }

    /// IV.4: Gemini's 20 years spend the twelve in 17 years 7 months, and
    /// the rest goes to Sagittarius, opposite, then Capricorn.
    #[test]
    fn the_bond_is_loosed_to_the_opposite_sign() {
        let dasha = releasing(Rashi::Gemini);
        let gemini = level_one(&dasha, Rashi::Gemini);
        let inside = children(&dasha, &gemini);
        assert_eq!(inside.len(), 14);
        assert_eq!(inside[11], (Rashi::Taurus, 8.0));
        assert_eq!(
            inside[12..],
            [(Rashi::Sagittarius, 12.0), (Rashi::Capricorn, 17.0)]
        );
        // A sign of fewer years never reaches it: IV.4's Aries cuts
        // Aquarius to the 11 months that fill out the 15 years.
        let aries = level_one(&releasing(Rashi::Aries), Rashi::Aries);
        let inside = children(&releasing(Rashi::Aries), &aries);
        assert_eq!(inside.last(), Some(&(Rashi::Aquarius, 11.0)));
        assert_eq!(inside.len(), 11);
    }

    /// IV.10: Fortune in Pisces, the fourth year. Gemini runs from 2 years
    /// 11 months; inside it Gemini 50 days, Cancer 62½, Leo 47½, Virgo 50,
    /// Libra 20, then Scorpio; inside Scorpio 3 days 3 hours, Sagittarius
    /// 2½ days, Capricorn 5 days 15 hours, Aquarius 6 days 6 hours, Pisces
    /// 2½ days, Aries 3 days 3 hours, and Taurus the rest.
    #[test]
    fn valens_fortune_in_pisces_goes_down_to_the_hours() {
        let dasha = releasing(Rashi::Pisces);
        let pisces = level_one(&dasha, Rashi::Pisces);
        let gemini = dasha.children(&pisces).nth(3).unwrap();
        assert_eq!(gemini.sign, Some(Rashi::Gemini));
        assert_eq!(months(&gemini).0, 35.0);
        let days = |p: &Period| p.interval.days();
        let third: Vec<(Rashi, f64)> = dasha
            .children(&gemini)
            .take(6)
            .map(|p| (p.sign.unwrap(), days(&p)))
            .collect();
        assert_eq!(
            third,
            [
                (Rashi::Gemini, 50.0),
                (Rashi::Cancer, 62.5),
                (Rashi::Leo, 47.5),
                (Rashi::Virgo, 50.0),
                (Rashi::Libra, 20.0),
                (Rashi::Scorpio, 37.5),
            ]
        );
        let scorpio = dasha.children(&gemini).nth(5).unwrap();
        let hours: Vec<(Rashi, f64)> = dasha
            .children(&scorpio)
            .map(|p| (p.sign.unwrap(), (days(&p) * 24.0 * 1e6).round() / 1e6))
            .collect();
        assert_eq!(
            hours[..6],
            [
                (Rashi::Scorpio, 75.0),
                (Rashi::Sagittarius, 60.0),
                (Rashi::Capricorn, 135.0),
                (Rashi::Aquarius, 150.0),
                (Rashi::Pisces, 60.0),
                (Rashi::Aries, 75.0),
            ]
        );
        assert_eq!(hours[6].0, Rashi::Taurus);
        // The 255th day of Gemini (8 months 15 days in) is in Scorpio's
        // 25th, as Valens counts it.
        let at = gemini.interval.from.get() + 255.0;
        let chain = dasha.at(JulianDay::literal(at), Depth::try_new(4).unwrap());
        let signs: Vec<Rashi> = chain.iter().filter_map(|p| p.sign).collect();
        assert_eq!(signs[..3], [Rashi::Pisces, Rashi::Gemini, Rashi::Scorpio]);
    }

    #[test]
    fn the_children_partition_their_parent() {
        for start in Rashi::ALL {
            let dasha = releasing(start);
            for parent in dasha.mahadashas() {
                let inside: Vec<Period> = dasha.children(&parent).collect();
                assert_eq!(inside[0].interval.from, parent.interval.from);
                assert_eq!(inside.last().unwrap().interval.to, parent.interval.to);
                for pair in inside.windows(2) {
                    assert_eq!(pair[0].interval.to, pair[1].interval.from);
                }
            }
        }
    }

    /// IV.11: the Ascendant in Virgo, the 35th year is Cancer. The ordinal
    /// counts from 1 for the year birth begins, so reading 35 as an age
    /// would give Leo, the rival this pins against.
    #[test]
    fn valens_thirty_fifth_year_from_virgo_is_cancer() {
        let dasha = ProfectionDasha::new(
            Rashi::Virgo,
            JulianDay::literal(BIRTH),
            YearLength::Julian36525,
        )
        .unwrap();
        assert_eq!(dasha.sign_of_year(35), Some(Rashi::Cancer));
        assert_eq!(dasha.sign_of_year(1), Some(Rashi::Virgo));
        assert_eq!(dasha.sign_of_year(13), Some(Rashi::Virgo));
        assert_eq!(dasha.sign_of_year(0), None);
        let thirty_fifth = BIRTH + 34.5 * 365.25;
        let chain = dasha.at(JulianDay::literal(thirty_fifth), Depth::try_new(1).unwrap());
        let period = chain.iter().next().unwrap();
        assert_eq!(period.sign, Some(Rashi::Cancer));
        assert_eq!(period.lord, Graha::Moon);
        // The year's index is the completed age, 34, in the first cycle.
        assert_eq!(period.path.to_string(), "34");
        assert_ne!(Some(period.sign.unwrap()), dasha.sign_of_year(36));
    }
}

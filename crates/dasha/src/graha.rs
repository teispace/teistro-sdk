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
//! use teistro_core::settings::{AfterCycle, AntardashaPlace, GrahaDivision, YearLength};
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
use teistro_core::settings::{AfterCycle, AntardashaPlace, GrahaDivision, YearLength};

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

/// The Naisargika dasha's lords and years (BPHS ch. 43 vv. 16 and 17):
/// "the Moon, Mars, Mercury, Venus, Jupiter, the Sun and Saturn, in order,
/// one, two, nine, twenty, eighteen, twenty and fifty years … from the time
/// of birth, in order, these are the natural dashas", 120 years in all.
pub const NAISARGIKA_LORDS: [(Graha, f64); 7] = [
    (Graha::Moon, 1.0),
    (Graha::Mars, 2.0),
    (Graha::Mercury, 9.0),
    (Graha::Venus, 20.0),
    (Graha::Jupiter, 18.0),
    (Graha::Sun, 20.0),
    (Graha::Saturn, 50.0),
];

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

/// The most seats a graha dasha can run: each is a graha and the sign its
/// period is of, or none, and no two are the same.
pub const MOST_SEATS: usize = Graha::ALL.len() * (Rashi::ALL.len() + 1);

// A seat's place in a path is a `u8`.
const _: () = assert!(MOST_SEATS <= u8::MAX as usize);

/// A graha dasha of one birth: its lords in the order they run, each with
/// the years the chart gives it.
#[derive(Clone, Debug, PartialEq)]
pub struct GrahaDasha {
    lords: Vec<(Graha, f64)>,
    /// The sign each seat's period is of, the lagna's in an ayurdaya dasha;
    /// nothing for a graha's.
    signs: Vec<Option<Rashi>>,
    birth: JulianDay<Utc>,
    year_days: f64,
    total_years: f64,
    shares: Shares,
    after_cycle: AfterCycle,
}

/// How a period divides among its sub-periods.
#[derive(Clone, Debug, PartialEq)]
enum Shares {
    /// From the period's own lord through every lord, as the division says.
    Divided(GrahaDivision),
    /// For each seat, the seats of its sub-periods in order, each with its
    /// part of the whole.
    Table(Vec<Vec<(usize, f64)>>),
}

/// One giver of an ayurdaya dasha's years (BPHS ch. 46 vv. 201 to 203): a
/// graha, or the lagna, whose periods are its sign's under the sign's lord.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Giver {
    /// The graha; nothing for the lagna.
    pub graha: Option<Graha>,
    /// The sign it stands in, or the lagna's.
    pub sign: Rashi,
    /// The years its span gives it, after the reductions.
    pub years: f64,
    /// Its strength, the grahas' and the lagna's in one unit.
    pub strength: f64,
}

impl Giver {
    /// The lord its periods carry: the graha, or the lagna's sign's lord.
    #[must_use]
    pub fn lord(&self) -> Graha {
        self.graha.unwrap_or_else(|| self.sign.attributes().lord)
    }

    /// The sign its periods are of: the lagna's, and none for a graha's.
    #[must_use]
    pub const fn period_sign(&self) -> Option<Rashi> {
        match self.graha {
            Some(_) => None,
            None => Some(self.sign),
        }
    }
}

/// The share ch. 51 vv. 13 and 14 give a body `house` signs from a dasha's
/// lord, counted inclusively, with the place the share is reckoned in: the
/// lord's own sign a half, a trikona a third, the fourth or the eighth a
/// quarter, the seventh a seventh. Nothing in any other house.
fn relation(house: House) -> Option<(u8, f64)> {
    Some(match house.get() {
        1 => (1, 1.0 / 2.0),
        5 | 9 => (5, 1.0 / 3.0),
        4 | 8 => (4, 1.0 / 4.0),
        7 => (7, 1.0 / 7.0),
        _ => return None,
    })
}

/// The order an ayurdaya dasha's givers run in (BPHS ch. 46 v. 202): the
/// one at `first`, then those in kendras from it, then those in
/// panapharas, then those in apoklimas, each group from the strongest.
/// Equal strengths keep the order given; nothing when `first` is not one.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_dasha::graha::{Giver, ayurdaya_order};
///
/// let giver = |graha, sign, strength| Giver { graha, sign, years: 1.0, strength };
/// let givers = [
///     giver(None, Rashi::Aries, 6.0),                        // the lagna
///     giver(Some(Graha::Sun), Rashi::Taurus, 8.0),           // a panaphara
///     giver(Some(Graha::Moon), Rashi::Cancer, 5.0),          // a kendra
///     giver(Some(Graha::Mars), Rashi::Libra, 7.0),           // a kendra
///     giver(Some(Graha::Saturn), Rashi::Gemini, 9.0),        // an apoklima
/// ];
/// assert_eq!(ayurdaya_order(&givers, 0), [0, 3, 2, 1, 4]);
/// ```
#[must_use]
pub fn ayurdaya_order(givers: &[Giver], first: usize) -> Vec<usize> {
    let Some(from) = givers.get(first).map(|giver| giver.sign) else {
        return Vec::new();
    };
    let group = |giver: &Giver| {
        let house = House::between(from, giver.sign);
        if house.is_kendra() {
            0
        } else if house.is_panaphara() {
            1
        } else {
            2
        }
    };
    let mut rest: Vec<(usize, u8, f64)> = givers
        .iter()
        .enumerate()
        .filter(|&(at, _)| at != first)
        .map(|(at, giver)| (at, group(giver), giver.strength))
        .collect();
    // A stable sort, so equal strengths keep the order given.
    rest.sort_by(|a, b| a.1.cmp(&b.1).then(b.2.total_cmp(&a.2)));
    std::iter::once(first)
        .chain(rest.into_iter().map(|(at, _, _)| at))
        .collect()
}

/// The antardashas of an ayurdaya dasha (BPHS ch. 51 vv. 13 to 16), its
/// `seats` the givers in the order they run: for each seat, its own first
/// with a full share, then each giver standing with it, in a trikona, in
/// the fourth or the eighth, or in the seventh from it, in the order the
/// seats run from it, each with its share. Of several in one place only
/// the strongest takes the share, a place as `place` reads it (crux C741),
/// and equal strengths keep the nearer seat. Each row's parts sum to one.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_core::settings::AntardashaPlace;
/// use teistro_dasha::graha::{Giver, ayurdaya_shares};
///
/// let giver = |graha, sign| Giver { graha: Some(graha), sign, years: 1.0, strength: 1.0 };
/// // The Sun's period: Mars with him a half, Jupiter in the fifth a third,
/// // Saturn in the seventh a seventh; Venus in the second takes none.
/// let seats = [
///     giver(Graha::Sun, Rashi::Aries),
///     giver(Graha::Mars, Rashi::Aries),
///     giver(Graha::Jupiter, Rashi::Leo),
///     giver(Graha::Venus, Rashi::Taurus),
///     giver(Graha::Saturn, Rashi::Libra),
/// ];
/// let sun = &ayurdaya_shares(&seats, AntardashaPlace::Share)[0];
/// let sum = 1.0 + 1.0 / 2.0 + 1.0 / 3.0 + 1.0 / 7.0;
/// let seats_of: Vec<usize> = sun.iter().map(|(seat, _)| *seat).collect();
/// assert_eq!(seats_of, [0, 1, 2, 4]);
/// assert!((sun[2].1 - 1.0 / 3.0 / sum).abs() < 1e-12);
/// ```
#[must_use]
pub fn ayurdaya_shares(seats: &[Giver], place: AntardashaPlace) -> Vec<Vec<(usize, f64)>> {
    let count = seats.len();
    seats
        .iter()
        .enumerate()
        .map(|(seat, lord)| {
            // The strongest in each place: its place, its seat, its
            // strength and its share. The seats are met in the order they
            // run from the lord, so a tie keeps the nearer.
            let mut places: Vec<(u8, usize, f64, f64)> = Vec::new();
            let after = seats
                .iter()
                .enumerate()
                .cycle()
                .skip(seat + 1)
                .take(count.saturating_sub(1));
            for (other, giver) in after {
                let house = House::between(lord.sign, giver.sign);
                let Some((category, share)) = relation(house) else {
                    continue;
                };
                let key = match place {
                    AntardashaPlace::House => house.get(),
                    _ => category,
                };
                match places.iter_mut().find(|held| held.0 == key) {
                    Some(held) if giver.strength > held.2 => {
                        *held = (key, other, giver.strength, share);
                    }
                    Some(_) => {}
                    None => places.push((key, other, giver.strength, share)),
                }
            }
            places.sort_by_key(|held| (held.1 + count - seat) % count);
            let row: Vec<(usize, f64)> = std::iter::once((seat, 1.0))
                .chain(
                    places
                        .into_iter()
                        .map(|(_, other, _, share)| (other, share)),
                )
                .collect();
            let total: f64 = row.iter().map(|(_, share)| share).sum();
            row.into_iter()
                .map(|(other, share)| (other, share / total))
                .collect()
        })
        .collect()
}

impl GrahaDasha {
    /// The dasha running `lords` in order from `birth`, each for its years
    /// of `year_length`, divided as `division` says at every level, and
    /// begun again after the last lord when `after_cycle` repeats it.
    ///
    /// # Errors
    ///
    /// No lords, or a lord named twice, named `lords`; a lord's years not
    /// a positive number, named `lords[i]`.
    pub fn new(
        lords: Vec<(Graha, f64)>,
        birth: JulianDay<Utc>,
        year_length: YearLength,
        division: GrahaDivision,
        after_cycle: AfterCycle,
    ) -> Result<GrahaDasha, Error> {
        let signs = vec![None; lords.len()];
        GrahaDasha::checked(
            lords,
            signs,
            birth,
            year_length,
            Shares::Divided(division),
            after_cycle,
        )
    }

    /// The dasha running `seats` in order from `birth`, each a lord, the
    /// sign its period is of when it is a sign's, and its years of
    /// `year_length`. Each seat's sub-periods are its row of `shares`: the
    /// seats they are of, in order, with their parts, scaled to sum to one,
    /// and so at every level. Two seats may share a lord when their signs
    /// differ, as the lagna and its lord do in an ayurdaya dasha.
    ///
    /// # Errors
    ///
    /// As [`GrahaDasha::new`], a seat named twice counting its sign; not
    /// one row of shares a seat, named `shares`; a row empty, longer than
    /// 255, or naming no seat or a part not a positive number, named
    /// `shares[i]`.
    pub fn with_shares(
        seats: Vec<(Graha, Option<Rashi>, f64)>,
        shares: Vec<Vec<(usize, f64)>>,
        birth: JulianDay<Utc>,
        year_length: YearLength,
        after_cycle: AfterCycle,
    ) -> Result<GrahaDasha, Error> {
        let count = seats.len();
        if shares.len() != count {
            return Err(Error::invalid_arg(format!(
                "{} rows of shares for {count} seats",
                shares.len()
            ))
            .with_field("shares"));
        }
        let mut table = Vec::with_capacity(count);
        for (at, row) in shares.into_iter().enumerate() {
            let malformed = row.is_empty()
                || row.len() > usize::from(u8::MAX)
                || row
                    .iter()
                    .any(|&(seat, part)| seat >= count || !(part.is_finite() && part > 0.0));
            if malformed {
                return Err(Error::invalid_arg(
                    "a row holds 1 to 255 sub-periods, each a seat and a positive part",
                )
                .with_field(format!("shares[{at}]")));
            }
            let total: f64 = row.iter().map(|(_, part)| part).sum();
            table.push(
                row.into_iter()
                    .map(|(seat, part)| (seat, part / total))
                    .collect(),
            );
        }
        let (lords, signs) = seats
            .into_iter()
            .map(|(lord, sign, years)| ((lord, years), sign))
            .unzip();
        GrahaDasha::checked(
            lords,
            signs,
            birth,
            year_length,
            Shares::Table(table),
            after_cycle,
        )
    }

    /// An ayurdaya dasha (BPHS ch. 46 vv. 201 to 203): `givers` in the
    /// order [`ayurdaya_order`] runs them from the one at `first`, those
    /// whose span gives them no years left out, each for its years of
    /// `year_length`, the antardashas as [`ayurdaya_shares`] gives them
    /// under `place`.
    ///
    /// # Errors
    ///
    /// `first` not a giver, named `first`; a giver's years negative or not
    /// a number, named `givers[i]`; and as [`GrahaDasha::with_shares`].
    pub fn ayurdaya(
        givers: &[Giver],
        first: usize,
        place: AntardashaPlace,
        birth: JulianDay<Utc>,
        year_length: YearLength,
        after_cycle: AfterCycle,
    ) -> Result<GrahaDasha, Error> {
        if first >= givers.len() {
            return Err(Error::invalid_arg(format!(
                "{first} names none of {} givers",
                givers.len()
            ))
            .with_field("first"));
        }
        for (at, giver) in givers.iter().enumerate() {
            if !(giver.years.is_finite() && giver.years >= 0.0) {
                return Err(Error::invalid_arg(format!(
                    "{:?} is given {} years, not a number of none or more",
                    giver.lord(),
                    giver.years
                ))
                .with_field(format!("givers[{at}]")));
            }
        }
        let seats: Vec<Giver> = ayurdaya_order(givers, first)
            .into_iter()
            .filter_map(|at| givers.get(at).copied())
            .filter(|giver| giver.years > 0.0)
            .collect();
        let shares = ayurdaya_shares(&seats, place);
        let seats = seats
            .iter()
            .map(|giver| (giver.lord(), giver.period_sign(), giver.years))
            .collect();
        GrahaDasha::with_shares(seats, shares, birth, year_length, after_cycle)
    }

    /// A dasha of `lords`, each seat's period of its sign in `signs`,
    /// checked as the constructors say. No seat is named twice, so no more
    /// run than [`MOST_SEATS`], which a path's place holds.
    fn checked(
        lords: Vec<(Graha, f64)>,
        signs: Vec<Option<Rashi>>,
        birth: JulianDay<Utc>,
        year_length: YearLength,
        shares: Shares,
        after_cycle: AfterCycle,
    ) -> Result<GrahaDasha, Error> {
        if lords.is_empty() {
            return Err(
                Error::invalid_arg("a graha dasha runs at least one lord").with_field("lords")
            );
        }
        let seats = || lords.iter().zip(&signs);
        for (at, ((graha, years), sign)) in seats().enumerate() {
            if !(years.is_finite() && *years > 0.0) {
                return Err(Error::invalid_arg(format!(
                    "{graha:?} is given {years} years, not a positive number"
                ))
                .with_field(format!("lords[{at}]")));
            }
            if seats()
                .take(at)
                .any(|((earlier, _), its)| earlier == graha && its == sign)
            {
                return Err(
                    Error::invalid_arg(format!("{graha:?} is named twice")).with_field("lords")
                );
            }
        }
        let year_days = year_length.days();
        let total_years = lords.iter().map(|(_, years)| years).sum();
        Ok(GrahaDasha {
            lords,
            signs,
            birth,
            year_days,
            total_years,
            shares,
            after_cycle,
        })
    }

    /// The lords in the order they run, each with its years.
    #[must_use]
    pub fn lords(&self) -> &[(Graha, f64)] {
        &self.lords
    }

    /// The sign each lord's period is of, in the same order: the lagna's
    /// in an ayurdaya dasha, and nothing for a graha's.
    #[must_use]
    pub fn signs(&self) -> &[Option<Rashi>] {
        &self.signs
    }

    /// How a period divides among its sub-periods; nothing when a table of
    /// shares divides it.
    #[must_use]
    pub const fn division(&self) -> Option<GrahaDivision> {
        match self.shares {
            Shares::Divided(division) => Some(division),
            Shares::Table(_) => None,
        }
    }

    /// The days of one cycle, every lord's years.
    fn cycle_days(&self) -> f64 {
        self.total_years * self.year_days
    }

    /// The `index`th sub-period of the period of the lord at `seat`: the
    /// seat it is of and its share of the parent's span, with the shares of
    /// those before it.
    fn sub_period(&self, seat: usize, index: usize) -> Option<(usize, f64, f64)> {
        match &self.shares {
            Shares::Table(table) => {
                let row = table.get(seat)?;
                let &(of, share) = row.get(index)?;
                let before = row.iter().take(index).map(|(_, part)| part).sum();
                Some((of, share, before))
            }
            Shares::Divided(division) => {
                let count = self.lords.len();
                if index >= count {
                    return None;
                }
                let share = |step: usize| -> Option<f64> {
                    let (_, years) = self.lords.get((seat + step) % count)?;
                    Some(match division {
                        #[expect(clippy::cast_precision_loss, reason = "at most 255 lords")]
                        GrahaDivision::Equal => 1.0 / count as f64,
                        _ => years / self.total_years,
                    })
                };
                let mut before = 0.0;
                for step in 0..index {
                    before += share(step)?;
                }
                Some(((seat + index) % count, share(index)?, before))
            }
        }
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
            *self.signs.get(index)?,
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

    /// The `index`th sub-period of `parent`: from its own lord onwards in
    /// the lords' order (ch. 51 v. 2), its share as the division says, or
    /// its seat's row of shares.
    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        let (seat, share, before) = self.sub_period(parent.seat(), index)?;
        // Both ends from the running sum, so one sub-period ends exactly
        // where the next begins.
        let days = parent.interval.days();
        let from = parent.interval.from.get();
        let span = Interval::literal(from + days * before, from + days * (before + share));
        Some(Period::of_share(
            self.lords.get(seat)?.0,
            *self.signs.get(seat)?,
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

    fn giver(graha: Option<Graha>, sign: Rashi, years: f64, strength: f64) -> Giver {
        Giver {
            graha,
            sign,
            years,
            strength,
        }
    }

    /// An Aries lagna with Mars, its lord, in Aries too; the Sun in Leo,
    /// Jupiter in Sagittarius, the Moon in Cancer, Saturn in Scorpio and
    /// Venus in Libra; Mercury in Taurus gives no years.
    fn ayurdaya_givers() -> Vec<Giver> {
        vec![
            giver(Some(Graha::Sun), Rashi::Leo, 10.0, 7.0),
            giver(Some(Graha::Moon), Rashi::Cancer, 12.0, 5.0),
            giver(Some(Graha::Mars), Rashi::Aries, 8.0, 6.0),
            giver(Some(Graha::Mercury), Rashi::Taurus, 0.0, 9.0),
            giver(Some(Graha::Jupiter), Rashi::Sagittarius, 9.0, 6.5),
            giver(Some(Graha::Venus), Rashi::Libra, 11.0, 5.5),
            giver(Some(Graha::Saturn), Rashi::Scorpio, 15.0, 4.0),
            giver(None, Rashi::Aries, 6.0, 8.0),
        ]
    }

    fn ayurdaya(place: AntardashaPlace) -> GrahaDasha {
        GrahaDasha::ayurdaya(
            &ayurdaya_givers(),
            7,
            place,
            JulianDay::literal(2_451_545.0),
            YearLength::Savana360,
            AfterCycle::End,
        )
        .unwrap()
    }

    #[test]
    fn an_ayurdaya_dasha_runs_from_the_first_through_kendras_panapharas_and_apoklimas() {
        let dasha = ayurdaya(AntardashaPlace::Share);
        // The lagna first, as its sign under Mars; then the kendras from
        // it (Mars with it, the Moon in the fourth, Venus in the seventh)
        // by strength, the panapharas (the Sun in the fifth, Saturn in the
        // eighth), the apoklima (Jupiter in the ninth). Mercury, with no
        // years, runs no period.
        let seats: Vec<(Graha, Option<Rashi>)> = dasha
            .lords()
            .iter()
            .zip(dasha.signs())
            .map(|((lord, _), sign)| (*lord, *sign))
            .collect();
        assert_eq!(
            seats,
            [
                (Graha::Mars, Some(Rashi::Aries)),
                (Graha::Mars, None),
                (Graha::Venus, None),
                (Graha::Moon, None),
                (Graha::Sun, None),
                (Graha::Saturn, None),
                (Graha::Jupiter, None),
            ]
        );
        let total: f64 = dasha.lords().iter().map(|(_, years)| years).sum();
        assert!((total - 71.0).abs() < 1e-12);
        assert_eq!(dasha.division(), None);
    }

    #[test]
    fn an_antardasha_takes_the_share_its_place_from_the_lord_gives() {
        let dasha = ayurdaya(AntardashaPlace::Share);
        // The lagna's period: Mars with it a half, the Moon in the fourth a
        // quarter (Saturn in the eighth is weaker in the same place), Venus
        // in the seventh a seventh, the Sun in the fifth a third (Jupiter
        // in the ninth is weaker).
        let lagna = dasha.mahadasha(0, 0).unwrap();
        let children: Vec<Period> = dasha.children(&lagna).collect();
        let parts = [1.0, 1.0 / 2.0, 1.0 / 7.0, 1.0 / 4.0, 1.0 / 3.0];
        let sum: f64 = parts.iter().sum();
        assert_eq!(
            children
                .iter()
                .map(|child| (child.lord, child.sign))
                .collect::<Vec<_>>(),
            [
                (Graha::Mars, Some(Rashi::Aries)),
                (Graha::Mars, None),
                (Graha::Venus, None),
                (Graha::Moon, None),
                (Graha::Sun, None),
            ]
        );
        for (child, part) in children.iter().zip(parts) {
            assert!((child.interval.days() - lagna.interval.days() * part / sum).abs() < 1e-6);
        }
        // Read by the house, the eighth and the ninth take their own.
        let by_house = ayurdaya(AntardashaPlace::House);
        let lagna = by_house.mahadasha(0, 0).unwrap();
        let lords: Vec<Graha> = by_house.children(&lagna).map(|child| child.lord).collect();
        assert_eq!(
            lords,
            [
                Graha::Mars,
                Graha::Mars,
                Graha::Venus,
                Graha::Moon,
                Graha::Sun,
                Graha::Saturn,
                Graha::Jupiter,
            ]
        );
    }

    #[test]
    fn every_ayurdaya_level_partitions_its_parent_from_its_own_seat() {
        for place in [AntardashaPlace::Share, AntardashaPlace::House] {
            let dasha = ayurdaya(place);
            for maha in dasha.mahadashas() {
                for parent in std::iter::once(maha).chain(dasha.children(&maha)) {
                    let children: Vec<Period> = dasha.children(&parent).collect();
                    assert_eq!(children[0].lord, parent.lord);
                    assert_eq!(children[0].sign, parent.sign);
                    assert_eq!(children[0].interval.from, parent.interval.from);
                    let end = children.last().unwrap().interval.to.get();
                    assert!((end - parent.interval.to.get()).abs() < 1e-6, "{place:?}");
                    for pair in children.windows(2) {
                        assert_eq!(pair[0].interval.to, pair[1].interval.from);
                    }
                }
            }
        }
    }

    #[test]
    fn a_malformed_ayurdaya_dasha_is_refused_by_its_field() {
        let field =
            |result: Result<GrahaDasha, Error>| result.unwrap_err().field().map(str::to_owned);
        let birth = JulianDay::literal(0.0);
        let mut givers = ayurdaya_givers();
        let built = |givers: &[Giver], first| {
            GrahaDasha::ayurdaya(
                givers,
                first,
                AntardashaPlace::Share,
                birth,
                YearLength::Savana360,
                AfterCycle::End,
            )
        };
        assert_eq!(field(built(&givers, 8)).as_deref(), Some("first"));
        givers[2].years = -1.0;
        assert_eq!(field(built(&givers, 7)).as_deref(), Some("givers[2]"));
        let seats = vec![(Graha::Sun, None, 1.0), (Graha::Moon, None, 2.0)];
        let shared = |shares| {
            GrahaDasha::with_shares(
                seats.clone(),
                shares,
                birth,
                YearLength::Savana360,
                AfterCycle::End,
            )
        };
        assert_eq!(
            field(shared(vec![vec![(0, 1.0)]])).as_deref(),
            Some("shares")
        );
        assert_eq!(
            field(shared(vec![vec![(0, 1.0)], vec![(2, 1.0)]])).as_deref(),
            Some("shares[1]")
        );
        assert_eq!(
            field(shared(vec![vec![], vec![(1, 1.0)]])).as_deref(),
            Some("shares[0]")
        );
        let twice = GrahaDasha::with_shares(
            vec![(Graha::Mars, None, 1.0), (Graha::Mars, None, 2.0)],
            vec![vec![(0, 1.0)], vec![(1, 1.0)]],
            birth,
            YearLength::Savana360,
            AfterCycle::End,
        );
        assert_eq!(field(twice).as_deref(), Some("lords"));
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

    /// A giver of a year in `sign` of `strength`.
    fn placed(sign: Rashi, strength: f64) -> Giver {
        giver(None, sign, 1.0, strength)
    }

    /// The strongest giver in a place takes it, and of two as strong the
    /// one met first from the lord keeps it; the place's givers then run
    /// in seat order from the lord, round past the last seat.
    #[test]
    fn the_strongest_in_each_place_gives_and_they_run_from_the_lord() {
        let seats = [
            placed(Rashi::Aries, 1.0),
            // From Aries: the fourth and the eighth share a place, and the
            // eighth is stronger.
            placed(Rashi::Cancer, 5.0),
            placed(Rashi::Scorpio, 9.0),
            // The fifth and the ninth share one, as strong as each other.
            placed(Rashi::Leo, 3.0),
            placed(Rashi::Sagittarius, 3.0),
        ];
        // A row's parts are scaled to sum to one.
        let same = |row: &[(usize, f64)], expected: &[(usize, f64)]| {
            let total: f64 = expected.iter().map(|(_, part)| part).sum();
            assert_eq!(row.len(), expected.len(), "{row:?}");
            for (got, want) in row.iter().zip(expected) {
                assert_eq!(got.0, want.0, "{row:?}");
                assert!((got.1 - want.1 / total).abs() < 1e-12, "{row:?}");
            }
        };
        let rows = ayurdaya_shares(&seats, AntardashaPlace::Share);
        same(&rows[0], &[(0, 1.0), (2, 1.0 / 4.0), (3, 1.0 / 3.0)]);
        // From Leo, the seats after it are the Sagittarius, the Aries (both
        // trines: the Sagittarius, met first and stronger, keeps it), the
        // Cancer (the twelfth, no place) and the Scorpio (the fourth): the
        // Sagittarius runs before the Scorpio, though its seat is later.
        same(&rows[3], &[(3, 1.0), (4, 1.0 / 3.0), (2, 1.0 / 4.0)]);
        // Read by the house, the eighth and the fourth are places of their
        // own.
        let by_house = ayurdaya_shares(&seats, AntardashaPlace::House);
        same(
            &by_house[0],
            &[
                (0, 1.0),
                (1, 1.0 / 4.0),
                (2, 1.0 / 4.0),
                (3, 1.0 / 3.0),
                (4, 1.0 / 3.0),
            ],
        );
    }

    /// A row of shares holds from one to 255 sub-periods, each a seat and
    /// a positive, finite part, and its parts are scaled to sum to one.
    #[test]
    fn a_row_of_shares_is_scaled_and_bounded() {
        let birth = JulianDay::literal(0.0);
        let shared = |shares: Vec<Vec<(usize, f64)>>| {
            GrahaDasha::with_shares(
                vec![(Graha::Sun, None, 4.0), (Graha::Moon, None, 2.0)],
                shares,
                birth,
                YearLength::Savana360,
                AfterCycle::End,
            )
        };
        let dasha = shared(vec![vec![(0, 2.0), (1, 6.0)], vec![(1, 5.0)]]).unwrap();
        assert_eq!(dasha.division(), None);
        let sun = dasha.mahadasha(0, 0).unwrap();
        let parts: Vec<f64> = dasha
            .children(&sun)
            .map(|child| child.interval.days() / sun.interval.days())
            .collect();
        assert_eq!(parts.len(), 2);
        assert!(
            (parts[0] - 0.25).abs() < 1e-12 && (parts[1] - 0.75).abs() < 1e-12,
            "{parts:?}"
        );
        let most = vec![(0, 1.0); usize::from(u8::MAX)];
        assert!(shared(vec![most.clone(), vec![(1, 1.0)]]).is_ok());
        let past = vec![(0, 1.0); usize::from(u8::MAX) + 1];
        let field = |shares| shared(shares).unwrap_err().field().map(str::to_owned);
        assert_eq!(
            field(vec![past, vec![(1, 1.0)]]).as_deref(),
            Some("shares[0]")
        );
        for part in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert_eq!(
                field(vec![vec![(0, 1.0)], vec![(1, part)]]).as_deref(),
                Some("shares[1]"),
                "{part}"
            );
        }
        let divided = GrahaDasha::new(
            vec![(Graha::Sun, 4.0)],
            birth,
            YearLength::Savana360,
            GrahaDivision::Equal,
            AfterCycle::End,
        )
        .unwrap();
        assert_eq!(divided.division(), Some(GrahaDivision::Equal));
    }

    /// Twelve grahas, each with a sign or none.
    #[test]
    fn a_dasha_runs_at_most_twelve_grahas_by_thirteen_signs() {
        assert_eq!(MOST_SEATS, 156);
    }

    /// The mahadasha at an instant: none before birth, the first at it,
    /// and at a boundary the one beginning there.
    #[test]
    fn the_mahadasha_at_an_instant_begins_at_birth() {
        let dasha = karaka(GrahaDivision::Proportional, AfterCycle::End);
        let birth = 2_451_545.0;
        assert!(dasha.mahadasha_at(birth - 1e-6).is_none());
        assert_eq!(dasha.mahadasha_at(birth).unwrap().lord, Graha::Sun);
        let sun_ends = dasha.mahadasha(0, 0).unwrap().interval.to.get();
        assert_eq!(dasha.mahadasha_at(sun_ends).unwrap().lord, Graha::Venus);
        assert_eq!(
            dasha.mahadasha_at(sun_ends - 1e-6).unwrap().lord,
            Graha::Sun
        );
    }
}

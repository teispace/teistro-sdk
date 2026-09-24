//! The annual dashas: the Mudda, the Varsha Yogini and the Patyayini
//! (`03-design/annual-dashas.md`).
//!
//! Each divides one year among a ring of lords, which is
//! `teistro-dasha`'s [`YearDasha`]; this module builds the ring the
//! sources give each one, and the clock the year runs on.
//!
//! - **The Mudda and the Varsha Yogini** are the natal Vimshottari and
//!   Yogini read over one year. Charak's two formulas are each the natal
//!   row's own seat for the birth nakshatra, advanced one lord for each
//!   completed year, so that is what is built, over the rows the natal
//!   systems already ship. The balance is crux C123.
//! - **The Patyayini** orders the seven and the lagna by their longitude
//!   within the sign (the *krishamsha*), and each takes the gap to the
//!   one before it (the *patyamsha*). Ties follow the *Tajika
//!   Nilakanthi*'s v. 4.
//! - **The clock** is crux C122: the Sun's degree, an even spread to the
//!   next return, or plain days.

use serde::{Deserialize, Serialize};
use teistro_astro::events::{Lattice, Longitudes, Quantity, Search};
use teistro_astro::sidereal::{Sidereal, Zodiac};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{Degrees, Depth, JulianDay, Ut1, Utc};
use teistro_core::settings::{Balance, BirthPeriod};
use teistro_dasha::row::YOGINI;
use teistro_dasha::{
    Clock, DashaName, PeriodRow, Share, Timeline, UduRow, VIMSHOTTARI, YearDasha, YearRing,
};
use teistro_port_ephemeris::Body;

use crate::bala::{AnnualSky, Panchavargiya, SEVEN};
use crate::drishti::speed_rank;

/// The annual dashas this build computes, in the catalogue's order.
pub const ANNUAL_DASHAS: [DashaSystem; 3] = [
    DashaSystem::Patyayini,
    DashaSystem::Mudda,
    DashaSystem::VarshaYogini,
];

/// The units the sources divide a year into: 360 "solar days", each the
/// Sun's motion through one degree.
pub const YEAR_UNITS: u16 = 360;

/// What a unit of the year is (crux C122).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum YearClock {
    /// The Sun's motion through one degree from where it stood at the
    /// return: Charak's definition of the 360 "solar days", and the
    /// *Nilakanthi*'s dating. The year ends on the next return.
    #[default]
    SunDegrees,
    /// An equal share of the time from this return to the next: Charak's
    /// "spread proportionately".
    Even,
    /// The whole year as this many civil days from the return: Charak's
    /// printed durations, 360 for the Mudda and the Yogini and 365 for the
    /// Patyayini. The one clock that needs no ephemeris.
    Days(f64),
}

/// Where the balance a nakshatra year opens with comes from (crux C123).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum MuddaBalance {
    /// What remained of the birth Moon's nakshatra: the same every year.
    /// Charak's worked year.
    #[default]
    NatalMoon,
    /// How far the Moon at the return is through its own nakshatra: the
    /// *Nilakanthi*'s appendix, after the *Hayanaratna*.
    EntryMoon,
    /// None: the first lord runs its whole share from the return, as the
    /// appendix says older almanacs do.
    Whole,
}

/// The readings an annual dasha is computed under.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct AnnualDashaRules {
    /// What a unit of the year is.
    pub clock: YearClock,
    /// Where a nakshatra year's balance comes from; the Patyayini has none.
    pub balance: MuddaBalance,
    /// How the balance is measured, by arc or by time; nothing for the
    /// source's own, which is by arc for the birth Moon (Charak) and by
    /// time for the Moon at the return (the appendix counts ghatis).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measure: Option<Balance>,
    /// How the first lord's two pieces are divided among sub-lords: the
    /// natal birth period's choice.
    pub birth_period: BirthPeriod,
    /// How many levels the periods go down: mahadashas and antardashas,
    /// which is what both sources tabulate, unless asked.
    pub depth: Depth,
}

impl Default for AnnualDashaRules {
    fn default() -> AnnualDashaRules {
        AnnualDashaRules {
            clock: YearClock::default(),
            balance: MuddaBalance::default(),
            measure: None,
            birth_period: BirthPeriod::Compressed,
            depth: Depth::try_new(2).unwrap_or(Depth::MIN),
        }
    }
}

impl AnnualDashaRules {
    /// How the balance is measured under these rules: the one asked for,
    /// or the source's own.
    #[must_use]
    pub fn measure(&self) -> Balance {
        self.measure.unwrap_or(match self.balance {
            MuddaBalance::EntryMoon => Balance::Temporal,
            MuddaBalance::NatalMoon | MuddaBalance::Whole => Balance::Spatial,
        })
    }
}

/// The natal row a nakshatra year is read over: Vimshottari for the Mudda,
/// Yogini for the Varsha Yogini; nothing for any other system.
#[must_use]
pub fn natal_row(system: DashaSystem) -> Option<&'static UduRow> {
    match system {
        DashaSystem::Mudda => Some(&VIMSHOTTARI),
        DashaSystem::VarshaYogini => Some(&YOGINI),
        _ => None,
    }
}

/// The ring of a nakshatra year: the natal row's lords with their years
/// as weights, opening at the birth nakshatra's seat advanced one lord for
/// each completed year.
///
/// `remaining` is what is left of the first lord's share when the year
/// opens ([`remaining_by_arc`] or [`remaining_by_time`]), or nothing for
/// [`MuddaBalance::Whole`].
///
/// ```
/// use teistro_core::angle::Nas;
/// use teistro_core::catalogue::{DashaSystem, Graha};
/// use teistro_core::quantity::Degrees;
///
/// // Charak's native: the Moon in Purva Phalguni, the forty-first year.
/// let moon = Nas::from_degrees(Degrees::try_new(137.0 + 8.0 / 60.0)?);
/// let ring = teistro_tajika::nakshatra_ring(DashaSystem::Mudda, moon, 40, None)?;
/// assert_eq!(ring.ring[ring.first].lord, Graha::Rahu);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// A system that is not a nakshatra year, named `system` with the two
/// that are in the hint.
pub fn nakshatra_ring(
    system: DashaSystem,
    natal_moon: Nas,
    completed_years: u16,
    remaining: Option<f64>,
) -> Result<YearRing, Error> {
    let row = natal_row(system).ok_or_else(|| {
        Error::invalid_arg(format!("{system:?} is not read from the birth nakshatra"))
            .with_field("system")
            .with_hint("the MUDDA and the VARSHA_YOGINI are")
    })?;
    let lords = row.lords.len().max(1);
    let seat = row.seat(natal_moon.nakshatra_index().get());
    Ok(YearRing {
        system: DashaName::Catalogued(system),
        ring: row
            .lords
            .iter()
            .map(|lord| Share {
                lord: lord.graha,
                sign: None,
                weight: f64::from(lord.years),
            })
            .collect(),
        first: (seat.lord + usize::from(completed_years)) % lords,
        remaining,
    })
}

/// What remains of a Moon's nakshatra by arc, 0 to 1.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "both are below a nakshatra's nanoarcseconds, 4.8e13, far inside the 2^53 a double holds exactly"
)]
pub fn remaining_by_arc(moon: Nas) -> f64 {
    1.0 - moon.in_nakshatra().get() as f64 / Nas::PER_NAKSHATRA as f64
}

/// What remains of a Moon's stay in its nakshatra by time, 0 to 1: from
/// `at` to its leaving, over the whole stay.
///
/// # Errors
///
/// A stay that does not hold `at`, or that is empty, named `moon_span`.
pub fn remaining_by_time(at: JulianDay<Utc>, moon_span: Interval) -> Result<f64, Error> {
    Ok(1.0 - teistro_dasha::balance::elapsed_in(at, moon_span)?)
}

/// One lord of the Patyayini: a graha, or the lagna.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Owner {
    Graha(Graha),
    Lagna,
}

/// The Patyayini's ring from an annual chart: the seven and the lagna by
/// their krishamsha, ascending, each weighted by its patyamsha in
/// nanoarcseconds. The year opens with the smallest, whole, and closes
/// with the largest.
///
/// Ties, which the *Nilakanthi*'s v. 4 settles and Charak does not raise:
/// the **stronger** by Vishwa bala runs first, and of two equally strong
/// the **slower** by the source's fixed speed order. A graha tied with the
/// lagna is weighed against the **lagna's lord** the same way; where that
/// is the graha itself the graha runs first, which the verse does not
/// decide. The second of a tie has a patyamsha of nothing and keeps its
/// place in the order.
///
/// # Errors
///
/// A longitude that is not a number, named by the sky's checks or
/// `annual_lagna_deg`; strengths that do not hold one of the seven, named
/// `strengths`.
pub fn patyayini_ring(
    sky: &AnnualSky,
    annual_lagna_deg: f64,
    strengths: &[Panchavargiya],
) -> Result<YearRing, Error> {
    sky.check()?;
    let lagna = Nas::try_from_degrees(annual_lagna_deg).map_err(|_| {
        Error::invalid_arg(format!(
            "the annual lagna stands at {annual_lagna_deg}, which is not a longitude"
        ))
        .with_field("annual_lagna_deg")
    })?;
    let lagna_lord = lagna.sign().attributes().lord;
    let vishwa = |graha: Graha| {
        strengths
            .iter()
            .find(|strength| strength.graha == graha)
            .map(|strength| strength.vishwa)
            .ok_or_else(|| {
                Error::invalid_arg(format!("the strengths do not hold {graha:?}"))
                    .with_field("strengths")
            })
    };
    let mut lords = Vec::with_capacity(SEVEN.len() + 1);
    for graha in SEVEN {
        let at = Nas::from_degrees(Degrees::try_new(sky.longitude_of(graha)).map_err(|_| {
            Error::invalid_arg(format!(
                "{graha:?} stands at a longitude that is not a number"
            ))
            .with_field("sky")
        })?);
        lords.push((Owner::Graha(graha), at.in_sign(), vishwa(graha)?, graha));
    }
    lords.push((
        Owner::Lagna,
        lagna.in_sign(),
        vishwa(lagna_lord)?,
        lagna_lord,
    ));
    // Ascending by krishamsha; then the stronger, the slower, and a graha
    // before the lagna it is tied with.
    lords.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then(b.2.cmp(&a.2))
            .then(speed_rank(b.3).cmp(&speed_rank(a.3)))
            .then((a.0 == Owner::Lagna).cmp(&(b.0 == Owner::Lagna)))
    });
    let mut before = Nas::ZERO;
    let ring = lords
        .iter()
        .map(|&(owner, krishamsha, _, lord)| {
            let patyamsha = krishamsha.get() - before.get();
            before = krishamsha;
            #[expect(
                clippy::cast_precision_loss,
                reason = "a sign's nanoarcseconds, 1.08e14, are far inside the 2^53 a double holds exactly"
            )]
            let weight = patyamsha as f64;
            Share {
                lord,
                sign: (owner == Owner::Lagna).then(|| lagna.sign()),
                weight,
            }
        })
        .collect();
    Ok(YearRing {
        system: DashaName::Catalogued(DashaSystem::Patyayini),
        ring,
        first: 0,
        remaining: None,
    })
}

/// The instants that divide the year into `divisions` equal arcs of the
/// Sun's motion, from where it stood at the return through the next
/// return: `divisions + 1` knots, the first the return itself and the last
/// the next, found in **one** search over the year.
///
/// 360 divisions put each of the Sun's degrees at the instant it crosses
/// it ([`YearClock::SunDegrees`]); one division is the two returns alone
/// ([`YearClock::Even`]). The longitude is read on the annual chart's own
/// zodiac, as the return was.
///
/// # Errors
///
/// No divisions, or more than [`YEAR_UNITS`], named `divisions`; a
/// longitude that is not a number, named `sun_deg`; whatever the source
/// refuses while searching; a year the search did not see to its end,
/// which only an ephemeris ending inside it can cause, as `OUT_OF_RANGE`
/// named `opens`.
pub fn sun_knots<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    opens: JulianDay<Utc>,
    sun_deg: f64,
    divisions: u16,
) -> Result<Vec<f64>, Error> {
    if divisions == 0 || divisions > YEAR_UNITS {
        return Err(Error::invalid_arg(format!(
            "a year is divided into 1 to {YEAR_UNITS} arcs of the Sun, not {divisions}"
        ))
        .with_field("divisions"));
    }
    if !sun_deg.is_finite() {
        return Err(
            Error::invalid_arg("the Sun stood at a longitude that is not a number")
                .with_field("sun_deg"),
        );
    }
    let sidereal = Sidereal {
        tropical,
        ayanamsha: zodiac.ayanamsha,
        basis: zodiac.basis,
        precession: zodiac.precession,
        delta_t: zodiac.delta_t,
    };
    let lattice = Lattice {
        origin_deg: sun_deg.rem_euclid(360.0),
        step_deg: if divisions == 1 {
            0.0
        } else {
            360.0 / f64::from(divisions)
        },
    };
    // Half a day in, past the return's own crossing; a sidereal year and a
    // week on, past the next return's.
    let from = JulianDay::<Ut1>::literal(opens.get() + 0.5);
    let to = JulianDay::<Ut1>::literal(opens.get() + crate::varsha::SIDEREAL_YEAR_DAYS + 7.0);
    let events =
        Search::new(&sidereal, Quantity::Longitude(Body::Sun), lattice).between(from, to)?;
    let wanted = usize::from(divisions);
    if events.len() < wanted {
        return Err(Error::new(
            Status::OutOfRange,
            format!(
                "the Sun crossed {} of the year's {wanted} divisions inside the ephemeris",
                events.len()
            ),
        )
        .with_field("opens"));
    }
    let mut knots = Vec::with_capacity(wanted + 1);
    knots.push(opens.get());
    knots.extend(events.iter().take(wanted).map(|event| event.instant.get()));
    Ok(knots)
}

impl YearClock {
    /// How many arcs of the Sun its knots divide the year into: 360 for
    /// the Sun's degrees, one for an even spread between the returns, and
    /// nothing for days, which read no sky.
    #[must_use]
    pub const fn divisions(self) -> Option<u16> {
        match self {
            YearClock::SunDegrees => Some(YEAR_UNITS),
            YearClock::Even => Some(1),
            YearClock::Days(_) => None,
        }
    }
}

/// The clock a year runs on: from `opens` for so many days, or through
/// the Sun's knots ([`sun_knots`] with [`YearClock::divisions`]).
///
/// # Errors
///
/// A [`YearClock::Days`] of no length or not a number, named `clock`;
/// knots missing for a clock that reads the Sun, or knots that do not
/// move forwards, named `knots`.
pub fn year_clock(
    reading: YearClock,
    opens: JulianDay<Utc>,
    sun_knots: Option<Vec<f64>>,
) -> Result<Clock, Error> {
    match reading {
        YearClock::Days(days) if !(days.is_finite() && days > 0.0) => Err(Error::invalid_arg(
            format!("a year of days is a length of more than none, not {days}"),
        )
        .with_field("clock")),
        YearClock::Days(days) => Clock::new(vec![opens.get(), opens.get() + days]),
        YearClock::Even | YearClock::SunDegrees => Clock::new(sun_knots.ok_or_else(|| {
            Error::invalid_arg("a clock that reads the Sun is built from its knots")
                .with_field("knots")
        })?),
    }
}

/// One year's annual dasha, as an answer carries it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AnnualDasha {
    /// Which system.
    pub system: DashaSystem,
    /// The years completed at the return that opens the year: a
    /// [`crate::Pravesha::year`].
    pub completed_years: u16,
    /// The readings it was computed under.
    pub rules: AnnualDashaRules,
    /// The birth nakshatra a nakshatra year is seeded from; nothing for the
    /// Patyayini.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<Nakshatra>,
    /// The ring the year runs round: its lords and weights, where it opens,
    /// and what remained of the first lord's share.
    pub ring: YearRing,
    /// The year, from this return to where the clock ends it.
    pub year: Interval,
    /// Every period to the rules' depth, depth first in time order. A period
    /// that runs for no time (a share tied at nothing, or the empty piece
    /// of a first lord whose share was whole) is not listed; its place is
    /// kept in the others' paths.
    pub periods: Vec<PeriodRow>,
}

impl AnnualDasha {
    /// The answer for a year's dasha under `rules`.
    #[must_use]
    pub fn of(
        dasha: &YearDasha,
        system: DashaSystem,
        completed_years: u16,
        rules: AnnualDashaRules,
        seed: Option<Nakshatra>,
    ) -> AnnualDasha {
        AnnualDasha {
            system,
            completed_years,
            rules,
            seed,
            ring: dasha.ring().clone(),
            year: dasha.year(),
            periods: dasha
                .periods(dasha.year(), rules.depth)
                .iter()
                .map(PeriodRow::of)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, index their own fixtures and compare exact values"
    )]

    use super::*;
    use crate::bala::panchavargiya;

    const OPENS: f64 = 2_445_932.5;

    fn degrees(deg: f64, min: f64) -> f64 {
        deg + min / 60.0
    }

    fn year_of(ring: YearRing, days: f64) -> YearDasha {
        let clock = year_clock(YearClock::Days(days), JulianDay::literal(OPENS), None).unwrap();
        YearDasha::new(ring, clock, BirthPeriod::Compressed).unwrap()
    }

    fn table(dasha: &YearDasha) -> Vec<(Graha, f64)> {
        dasha
            .mahadashas()
            .map(|period| (period.lord, period.interval.days()))
            .collect()
    }

    /// Charak's native: the Moon at Leo 17°08′, 3°48′ into Purva Phalguni.
    fn charak_moon() -> Nas {
        Nas::from_degrees(Degrees::try_new(120.0 + degrees(17.0, 8.0)).unwrap())
    }

    #[test]
    fn charaks_mudda_is_table_v_2() {
        let moon = charak_moon();
        let remaining = remaining_by_arc(moon);
        assert!((remaining - 572.0 / 800.0).abs() < 1e-12);
        let ring = nakshatra_ring(DashaSystem::Mudda, moon, 40, Some(remaining)).unwrap();
        let rows = table(&year_of(ring, 360.0));
        let printed = [
            (Graha::Rahu, 38.61),
            (Graha::Jupiter, 48.0),
            (Graha::Saturn, 57.0),
            (Graha::Mercury, 51.0),
            (Graha::Ketu, 21.0),
            (Graha::Venus, 60.0),
            (Graha::Sun, 18.0),
            (Graha::Moon, 30.0),
            (Graha::Mars, 21.0),
            (Graha::Rahu, 15.39),
        ];
        assert_eq!(rows.len(), printed.len());
        for ((lord, days), (want, printed_days)) in rows.iter().zip(printed) {
            assert_eq!(*lord, want);
            assert!((days - printed_days).abs() < 0.005, "{lord:?} {days}");
        }
    }

    #[test]
    fn charaks_varsha_yogini_is_table_v_5() {
        let moon = charak_moon();
        let ring = nakshatra_ring(
            DashaSystem::VarshaYogini,
            moon,
            40,
            Some(remaining_by_arc(moon)),
        )
        .unwrap();
        let rows = table(&year_of(ring, 360.0));
        let printed = [
            (Graha::Saturn, 42.9),
            (Graha::Venus, 70.0),
            (Graha::Rahu, 80.0),
            (Graha::Moon, 10.0),
            (Graha::Sun, 20.0),
            (Graha::Jupiter, 30.0),
            (Graha::Mars, 40.0),
            (Graha::Mercury, 50.0),
            (Graha::Saturn, 17.1),
        ];
        assert_eq!(rows.len(), printed.len());
        for ((lord, days), (want, printed_days)) in rows.iter().zip(printed) {
            assert_eq!(*lord, want);
            assert!((days - printed_days).abs() < 0.05, "{lord:?} {days}");
        }
    }

    #[test]
    fn the_printed_formulas_are_the_natal_seat_advanced_by_the_years() {
        // Charak's remainders, 1 to 9 (0 is the ninth), in his order.
        let mudda = [
            Graha::Venus,
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Rahu,
            Graha::Jupiter,
            Graha::Saturn,
            Graha::Mercury,
            Graha::Ketu,
        ];
        let yogini = [
            Graha::Rahu,
            Graha::Moon,
            Graha::Sun,
            Graha::Jupiter,
            Graha::Mars,
            Graha::Mercury,
            Graha::Saturn,
            Graha::Venus,
        ];
        for nakshatra in 1..=27_u16 {
            let moon = Nas::new(i64::from(nakshatra - 1) * Nas::PER_NAKSHATRA + 1);
            for years in 0..200_u16 {
                let first = |system| {
                    let ring = nakshatra_ring(system, moon, years, None).unwrap();
                    ring.ring[ring.first].lord
                };
                let by_formula = usize::from((years + nakshatra + 9 * 30 - 2) % 9);
                assert_eq!(first(DashaSystem::Mudda), mudda[by_formula]);
                let by_formula = usize::from((nakshatra + years + 3) % 8);
                assert_eq!(first(DashaSystem::VarshaYogini), yogini[by_formula]);
            }
        }
    }

    /// Charak's Tables V-6 and V-7, laid out as a sky: each graha in a sign
    /// of its own so only the krishamsha matters.
    fn charak_patyayini_sky() -> (AnnualSky, f64) {
        let sky = AnnualSky {
            sun_deg: degrees(3.0, 50.0),
            mars_deg: 30.0 + degrees(7.0, 42.0),
            jupiter_deg: 90.0 + degrees(9.0, 38.0),
            moon_deg: 120.0 + degrees(9.0, 40.0),
            saturn_deg: 150.0 + degrees(17.0, 13.0),
            mercury_deg: 180.0 + degrees(18.0, 20.0),
            venus_deg: 210.0 + degrees(21.0, 45.0),
        };
        (sky, 60.0 + degrees(9.0, 26.0))
    }

    #[test]
    fn charaks_patyayini_is_table_v_8() {
        let (sky, lagna) = charak_patyayini_sky();
        let ring = patyayini_ring(&sky, lagna, &panchavargiya(&sky).unwrap()).unwrap();
        assert_eq!(
            ring.ring[2].sign,
            Some(teistro_core::catalogue::Rashi::Gemini)
        );
        let dasha = year_of(ring, 365.0);
        let rows = table(&dasha);
        let printed = [
            (Graha::Sun, 64.33),
            (Graha::Mars, 64.89),
            (Graha::Mercury, 29.09),
            (Graha::Jupiter, 3.36),
            (Graha::Moon, 0.56),
            (Graha::Saturn, 126.70),
            (Graha::Mercury, 18.74),
            (Graha::Venus, 57.34),
        ];
        assert_eq!(rows.len(), printed.len());
        for ((lord, days), (want, printed_days)) in rows.iter().zip(printed) {
            assert_eq!(*lord, want, "the lagna's period is its sign lord's");
            assert!((days - printed_days).abs() < 0.005, "{lord:?} {days}");
        }
        // Table V-9: the Sun's own antardasha, 11 days 8.11 hours.
        let sun = dasha.mahadasha(0, 0).unwrap();
        let own = dasha.children(&sun).next().unwrap();
        assert_eq!(own.lord, Graha::Sun);
        assert!((own.interval.hours() - (11.0 * 24.0 + 8.11)).abs() < 0.05);
    }

    #[test]
    fn the_nilakanthis_jupiter_runs_thirty_five_days_and_forty_nine_ghatis() {
        // Samjna-tantra's example: Jupiter smallest at 2°49′08″, Venus the
        // largest at 28°19′17″, a year of 360.
        let sky = AnnualSky {
            sun_deg: degrees(12.0, 57.0 + 50.0 / 60.0),
            moon_deg: 30.0 + degrees(5.0, 39.0 + 33.0 / 60.0),
            mars_deg: 60.0 + degrees(22.0, 13.0 + 53.0 / 60.0),
            mercury_deg: 90.0 + degrees(23.0, 59.0 + 9.0 / 60.0),
            jupiter_deg: 120.0 + degrees(2.0, 49.0 + 8.0 / 60.0),
            venus_deg: 150.0 + degrees(28.0, 19.0 + 17.0 / 60.0),
            saturn_deg: 180.0 + degrees(10.0, 55.0 + 13.0 / 60.0),
        };
        let lagna = 210.0 + degrees(27.0, 7.0 + 3.0 / 60.0);
        let ring = patyayini_ring(&sky, lagna, &panchavargiya(&sky).unwrap()).unwrap();
        let rows = table(&year_of(ring, 360.0));
        assert_eq!(rows[0].0, Graha::Jupiter);
        let printed = 35.0 + 49.0 / 60.0 + 52.0 / 3600.0;
        assert!((rows[0].1 - printed).abs() < 0.001, "{}", rows[0].1);
        assert_eq!(rows[7].0, Graha::Venus);
    }

    #[test]
    fn a_tie_goes_to_the_stronger_then_the_slower() {
        // The Sun and Saturn at one krishamsha.
        let (mut sky, lagna) = charak_patyayini_sky();
        sky.saturn_deg = 150.0 + degrees(3.0, 50.0);
        let strengths = panchavargiya(&sky).unwrap();
        let of = |graha: Graha| strengths.iter().find(|s| s.graha == graha).unwrap().vishwa;
        let ring = patyayini_ring(&sky, lagna, &strengths).unwrap();
        let (first, second) = (ring.ring[0], ring.ring[1]);
        // Stronger first; equal, the slower, and Saturn is the slowest.
        let expected = match of(Graha::Sun).cmp(&of(Graha::Saturn)) {
            core::cmp::Ordering::Greater => Graha::Sun,
            core::cmp::Ordering::Less | core::cmp::Ordering::Equal => Graha::Saturn,
        };
        assert_eq!(first.lord, expected);
        assert_eq!(second.weight, 0.0, "the second of a tie runs for no time");
    }

    #[test]
    fn only_the_three_annual_systems_are_built_here() {
        let moon = charak_moon();
        let refused = nakshatra_ring(DashaSystem::Vimshottari, moon, 1, None).unwrap_err();
        assert_eq!(refused.field(), Some("system"));
        assert!(natal_row(DashaSystem::Patyayini).is_none());
    }

    #[test]
    fn a_clock_is_refused_by_what_it_lacks() {
        let opens = JulianDay::literal(OPENS);
        for days in [0.0, -1.0, f64::NAN] {
            let refused = year_clock(YearClock::Days(days), opens, None).unwrap_err();
            assert_eq!(refused.field(), Some("clock"));
        }
        let unread = year_clock(YearClock::SunDegrees, opens, None).unwrap_err();
        assert_eq!(unread.field(), Some("knots"));
        let even = year_clock(YearClock::Even, opens, Some(vec![OPENS, OPENS + 365.25])).unwrap();
        assert_eq!(even.year(), Interval::literal(OPENS, OPENS + 365.25));
        assert_eq!(YearClock::SunDegrees.divisions(), Some(360));
        assert_eq!(YearClock::Days(360.0).divisions(), None);
    }

    #[test]
    fn the_rules_read_camel_case_and_fill_the_rest() {
        let rules: AnnualDashaRules = serde_json::from_str(
            r#"{"clock":{"days":365},"balance":"entry_moon","birthPeriod":"ELAPSED"}"#,
        )
        .unwrap();
        assert_eq!(rules.clock, YearClock::Days(365.0));
        assert_eq!(rules.measure(), Balance::Temporal);
        assert_eq!(rules.birth_period, BirthPeriod::Elapsed);
        assert_eq!(rules.depth.get(), 2);
        assert_eq!(AnnualDashaRules::default().measure(), Balance::Spatial);
    }
}

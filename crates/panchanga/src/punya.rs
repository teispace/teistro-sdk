//! The punya kala of a sankranti: the window around the Sun's entry into
//! a sign in which the bath and the gifts bear fruit
//! (`03-design/panchanga-day.md` §21.5, cruxes C752 to C754).
//!
//! The *Dharmasindhu* (pp. 2–3) and the *Nirnayasindhu* (pp. 3–5) read
//! one rule in three steps:
//!
//! 1. **The ghatis.** Each sign's entry is meritorious for so many ghatis
//!    before it, after it, or both ([`ghatis`]): sixteen before the fixed
//!    signs, sixteen after the dual ones, thirty before Karka, forty
//!    (Madhava twenty) after Makara, and fifteen (Hemadri ten) either side
//!    of the equinoxes, chosen by `panchanga.punya_kala`.
//! 2. **By day only.** Night is forbidden for the bath (*Nirnayasindhu*:
//!    *rātrinişedhataḥ*), so the window is the part of those ghatis in the
//!    daylight, and a side that would fall at night turns over: an entry
//!    after which two ghatis of daylight are not left takes the ghatis
//!    before it, and one less than two ghatis after sunrise takes those
//!    after (*Dharmasindhu*: *ghaṭikādvayādyalpadinaśeṣe … pūrvā eva*).
//! 3. **An entry at night.** Except at the two ayanas, an entry before
//!    midnight makes the afternoon before it meritorious and one after
//!    midnight the morning after; one at midnight both. Makara at night
//!    gives the next day and Karka the day before, but for the twilights:
//!    Makara within three ghatis of sunset keeps the day it set on, and
//!    Karka within three ghatis of sunrise the day that rises.
//!
//! The ghatis are counted under `day.ghati_reckoning`, so a proportional
//! almanac's sixteen ghatis are sixteen thirtieths of the arc they lie in.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Rashi;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::PunyaKalaTable;
use teistro_time::Reckoning;

use crate::span::Span;

/// A sankranti's punya kala, on the day it falls.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_core::interval::Interval;
/// use teistro_core::settings::PunyaKalaTable;
/// use teistro_panchanga::punya::{Arcs, punya_kala};
/// use teistro_time::Reckoning;
///
/// // A day from sunrise at 100.25 to sunset at 100.75, and the Sun entering
/// // Makara at 100.6: the forty ghatis after it run past sunset.
/// let arcs = Arcs {
///     before: Interval::literal(99.75, 100.25),
///     daylight: Interval::literal(100.25, 100.75),
///     after: Interval::literal(100.75, 101.25),
/// };
/// let entry = teistro_core::quantity::JulianDay::literal(100.6);
/// let kala = punya_kala(Rashi::Capricorn, entry, &arcs, PunyaKalaTable::Dharmasindhu, Reckoning::Civil)?
///     .ok_or("the day holds it")?;
/// assert_eq!(kala.span.inside, Interval::literal(100.6, 100.75));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PunyaKala {
    /// When the Sun entered the sign, which may be in the night before
    /// or after the day.
    pub sankranti: JulianDay<Utc>,
    /// The sign entered; `whole` the ghatis the text gives, or the half
    /// day a night's entry makes meritorious; `inside` their part in the
    /// day's daylight, the window itself.
    pub span: Span<Rashi>,
}

/// The day's daylight and the nights either side of it, which an entry
/// is placed by.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::punya::Arcs;
///
/// let arcs = Arcs {
///     before: Interval::literal(99.75, 100.25),
///     daylight: Interval::literal(100.25, 100.75),
///     after: Interval::literal(100.75, 101.25),
/// };
/// assert_eq!(arcs.daylight.days(), 0.5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arcs {
    /// The night before, from the previous sunset to this sunrise.
    pub before: Interval,
    /// The daylight, sunrise to sunset.
    pub daylight: Interval,
    /// The night after, from sunset to the next sunrise.
    pub after: Interval,
}

/// The ghatis before and after the entry into each sign, Mesha first,
/// under a table (crux C752).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_core::settings::PunyaKalaTable;
/// use teistro_panchanga::punya::ghatis;
///
/// assert_eq!(ghatis(Rashi::Aries, PunyaKalaTable::Dharmasindhu)?, (15, 15));
/// assert_eq!(ghatis(Rashi::Aries, PunyaKalaTable::Hemadri)?, (10, 10));
/// assert_eq!(ghatis(Rashi::Cancer, PunyaKalaTable::Madhava)?, (30, 0));
/// assert_eq!(ghatis(Rashi::Capricorn, PunyaKalaTable::Madhava)?, (0, 20));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A table core added before this crate learnt it.
pub fn ghatis(sign: Rashi, table: PunyaKalaTable) -> Result<(u8, u8), Error> {
    let (equinox, makara) = match table {
        PunyaKalaTable::Dharmasindhu => (15, 40),
        PunyaKalaTable::Hemadri => (10, 40),
        PunyaKalaTable::Madhava => (10, 20),
        other => {
            return Err(Error::unsupported(format!(
                "the panchanga does not know the punya kala table {other}; it knows {}",
                PunyaKalaTable::ALL
                    .iter()
                    .map(|known| known.key())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .with_field("panchanga.punya_kala"));
        }
    };
    Ok(match sign {
        Rashi::Aries | Rashi::Libra => (equinox, equinox),
        Rashi::Cancer => (30, 0),
        Rashi::Capricorn => (0, makara),
        Rashi::Taurus | Rashi::Leo | Rashi::Scorpio | Rashi::Aquarius => (16, 0),
        _ => (0, 16),
    })
}

/// The two ghatis of daylight an entry must leave on the side it reads,
/// or the side turns over (*Dharmasindhu* p. 2, crux C753).
const TURNING_GHATIS: f64 = 2.0;

/// The ghatis of a twilight, *sandhyā*: three after sunset and three
/// before sunrise (*Dharmasindhu* p. 2).
const TWILIGHT_GHATIS: f64 = 3.0;

/// The punya kala the day holds of a sign's entry, if it holds one.
///
/// An entry in the daylight is the day's own; one in the night before
/// or after is the day's when the rule of the night gives it this day
/// (the module's step 3). `None` for an entry no rule places on the day,
/// or a day with no daylight.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_core::interval::Interval;
/// use teistro_core::quantity::JulianDay;
/// use teistro_core::settings::PunyaKalaTable;
/// use teistro_panchanga::punya::{Arcs, punya_kala};
/// use teistro_time::Reckoning;
///
/// let arcs = Arcs {
///     before: Interval::literal(99.75, 100.25),
///     daylight: Interval::literal(100.25, 100.75),
///     after: Interval::literal(100.75, 101.25),
/// };
/// let at = |sign, entry| punya_kala(sign, JulianDay::literal(entry), &arcs, PunyaKalaTable::Dharmasindhu, Reckoning::Civil);
/// // Simha entered in the evening: its morning after belongs to tomorrow,
/// // and the afternoon before midnight to today.
/// let evening = at(Rashi::Leo, 100.9)?.ok_or("before midnight")?;
/// assert_eq!(evening.span.inside, Interval::literal(100.5, 100.75));
/// assert_eq!(at(Rashi::Leo, 101.1)?, None);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// A table core added before this crate learnt it.
pub fn punya_kala(
    sign: Rashi,
    sankranti: JulianDay<Utc>,
    arcs: &Arcs,
    table: PunyaKalaTable,
    reckoning: Reckoning,
) -> Result<Option<PunyaKala>, Error> {
    let (before, after) = ghatis(sign, table)?;
    let (before, after) = (f64::from(before), f64::from(after));
    let daylight = arcs.daylight;
    if daylight.is_empty() {
        return Ok(None);
    }
    let walk = Walk { arcs, reckoning };
    let t = sankranti.get();
    let ghatis_from = |from: f64, to: f64| walk.ghatis_between(from, to);
    let around = |back: f64, forward: f64| -> Result<Interval, Error> {
        Interval::new(walk.step(t, -back), walk.step(t, forward))
    };
    let (sunrise, sunset) = (daylight.from.get(), daylight.to.get());
    let whole = if daylight.contains(sankranti) {
        // Step 2: the side read, turned over where it would be night.
        if after == 0.0 && ghatis_from(sunrise, t) < TURNING_GHATIS {
            around(0.0, before)?
        } else if before == 0.0 && ghatis_from(t, sunset) < TURNING_GHATIS {
            around(after, 0.0)?
        } else {
            around(before, after)?
        }
    } else if arcs.after.contains(sankranti) {
        let dusk = ghatis_from(sunset, t) < TWILIGHT_GHATIS;
        let dawn = ghatis_from(t, arcs.after.to.get()) < TWILIGHT_GHATIS;
        match sign {
            Rashi::Capricorn if dusk => around(after, 0.0)?,
            Rashi::Cancer if !dawn => around(before, 0.0)?,
            Rashi::Capricorn | Rashi::Cancer => return Ok(None),
            _ if t <= midpoint(arcs.after) => afternoon(daylight),
            _ => return Ok(None),
        }
    } else if arcs.before.contains(sankranti) {
        let dusk = ghatis_from(arcs.before.from.get(), t) < TWILIGHT_GHATIS;
        let dawn = ghatis_from(t, sunrise) < TWILIGHT_GHATIS;
        match sign {
            Rashi::Capricorn if !dusk => around(0.0, after)?,
            Rashi::Cancer if dawn => around(0.0, before)?,
            Rashi::Capricorn | Rashi::Cancer => return Ok(None),
            _ if t >= midpoint(arcs.before) => morning(daylight),
            _ => return Ok(None),
        }
    } else {
        return Ok(None);
    };
    Ok(Span::new(sign, whole, daylight)
        .filter(|span| !span.inside.is_empty())
        .map(|span| PunyaKala { sankranti, span }))
}

/// The instant halfway through an arc.
fn midpoint(arc: Interval) -> f64 {
    f64::midpoint(arc.from.get(), arc.to.get())
}

/// The daylight's second half, from midday to sunset.
fn afternoon(daylight: Interval) -> Interval {
    Interval {
        from: JulianDay::literal(midpoint(daylight)),
        ..daylight
    }
}

/// The daylight's first half, from sunrise to midday.
fn morning(daylight: Interval) -> Interval {
    Interval {
        to: JulianDay::literal(midpoint(daylight)),
        ..daylight
    }
}

/// Ghatis counted across the arcs: twenty-four minutes each under civil
/// reckoning, a thirtieth of the arc they lie in under proportional, the
/// outermost arcs' rate carried past them.
struct Walk<'a> {
    arcs: &'a Arcs,
    reckoning: Reckoning,
}

/// Days in a civil ghati.
const CIVIL_GHATI: f64 = 1.0 / 60.0;

impl Walk<'_> {
    /// The arcs in order, each with the days of one of its ghatis.
    fn rates(&self) -> [(Interval, f64); 3] {
        let rate = |arc: Interval| match self.reckoning {
            Reckoning::Civil => CIVIL_GHATI,
            Reckoning::Proportional => arc.days() / 30.0,
        };
        [self.arcs.before, self.arcs.daylight, self.arcs.after].map(|arc| (arc, rate(arc)))
    }

    /// The instant `ghatis` from `from`, forward when positive.
    fn step(&self, from: f64, ghatis: f64) -> JulianDay<Utc> {
        let rates = self.rates();
        let mut at = from;
        let mut left = ghatis.abs();
        if ghatis >= 0.0 {
            for (index, (arc, rate)) in rates.iter().enumerate() {
                let end = if index == rates.len() - 1 {
                    f64::INFINITY
                } else {
                    arc.to.get()
                };
                if at >= end || *rate <= 0.0 {
                    continue;
                }
                let room = (end - at) / rate;
                if left <= room {
                    return JulianDay::literal(at + left * rate);
                }
                left -= room;
                at = end;
            }
        } else {
            for (index, (arc, rate)) in rates.iter().enumerate().rev() {
                let start = if index == 0 {
                    f64::NEG_INFINITY
                } else {
                    arc.from.get()
                };
                if at <= start || *rate <= 0.0 {
                    continue;
                }
                let room = (at - start) / rate;
                if left <= room {
                    return JulianDay::literal(at - left * rate);
                }
                left -= room;
                at = start;
            }
        }
        JulianDay::literal(at)
    }

    /// The ghatis from one instant to a later one.
    fn ghatis_between(&self, from: f64, to: f64) -> f64 {
        let rates = self.rates();
        let last = rates.len() - 1;
        rates
            .iter()
            .enumerate()
            .filter(|(_, (_, rate))| *rate > 0.0)
            .map(|(index, (arc, rate))| {
                let start = if index == 0 {
                    f64::NEG_INFINITY
                } else {
                    arc.from.get()
                };
                let end = if index == last {
                    f64::INFINITY
                } else {
                    arc.to.get()
                };
                (to.min(end) - from.max(start)).max(0.0) / rate
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A day from sunrise at 100.25 to sunset at 100.75 between two
    /// twelve-hour nights.
    const ARCS: Arcs = Arcs {
        before: Interval::literal(99.75, 100.25),
        daylight: Interval::literal(100.25, 100.75),
        after: Interval::literal(100.75, 101.25),
    };

    fn at(sign: Rashi, entry: f64) -> Option<Interval> {
        punya_kala(
            sign,
            JulianDay::literal(entry),
            &ARCS,
            PunyaKalaTable::Dharmasindhu,
            Reckoning::Civil,
        )
        .ok()
        .flatten()
        .map(|kala| kala.span.inside)
    }

    fn close(found: Option<Interval>, from: f64, to: f64) -> bool {
        found.is_some_and(|found| {
            (found.from.get() - from).abs() < 1e-9 && (found.to.get() - to).abs() < 1e-9
        })
    }

    #[test]
    fn each_sign_reads_its_side_in_the_daylight() {
        let g = CIVIL_GHATI;
        // Far enough from the night that no side turns over or reaches it.
        assert!(close(at(Rashi::Taurus, 100.6), 100.6 - 16.0 * g, 100.6));
        assert!(close(at(Rashi::Gemini, 100.4), 100.4, 100.4 + 16.0 * g));
        // Thirty ghatis before Karka reach back past sunrise.
        assert!(close(at(Rashi::Cancer, 100.6), 100.25, 100.6));
        assert!(close(
            at(Rashi::Aries, 100.5),
            100.5 - 15.0 * g,
            100.5 + 15.0 * g
        ));
        assert!(close(at(Rashi::Capricorn, 100.5), 100.5, 100.75));
    }

    #[test]
    fn a_side_that_would_be_night_turns_over() {
        let g = CIVIL_GHATI;
        // Mithuna a ghati before sunset reads the sixteen before it.
        assert!(close(
            at(Rashi::Gemini, 100.75 - g),
            100.75 - 17.0 * g,
            100.75 - g
        ));
        // Three ghatis before sunset still reads the side after.
        assert!(close(
            at(Rashi::Gemini, 100.75 - 3.0 * g),
            100.75 - 3.0 * g,
            100.75
        ));
        // Simha a ghati after sunrise reads the sixteen after it.
        assert!(close(
            at(Rashi::Leo, 100.25 + g),
            100.25 + g,
            100.25 + 17.0 * g
        ));
        // Karka at dawn too, its thirty after it reaching sunset.
        assert!(close(at(Rashi::Cancer, 100.25 + g), 100.25 + g, 100.75));
    }

    #[test]
    fn a_night_entry_gives_the_half_day_nearest_it() {
        // Before midnight the afternoon before; after it, tomorrow's.
        assert!(close(at(Rashi::Leo, 100.9), 100.5, 100.75));
        assert_eq!(at(Rashi::Leo, 101.1), None);
        // After the previous midnight, this morning; before it, yesterday's.
        assert!(close(at(Rashi::Leo, 100.1), 100.25, 100.5));
        assert_eq!(at(Rashi::Leo, 99.9), None);
        // At midnight both days.
        assert!(close(at(Rashi::Aries, 101.0), 100.5, 100.75));
        assert!(close(at(Rashi::Aries, 100.0), 100.25, 100.5));
    }

    #[test]
    fn the_ayanas_keep_their_own_night_rule() {
        let g = CIVIL_GHATI;
        // Makara at night is tomorrow's, but within three ghatis of
        // sunset today's, by the forty ghatis before it.
        assert_eq!(at(Rashi::Capricorn, 100.9), None);
        assert!(close(at(Rashi::Capricorn, 100.75 + g), 100.25, 100.75));
        assert!(close(at(Rashi::Capricorn, 100.1), 100.25, 100.75));
        assert!(close(at(Rashi::Capricorn, 99.9), 100.25, 99.9 + 40.0 * g));
        assert_eq!(at(Rashi::Capricorn, 99.75 + g), None);
        // Karka at night is yesterday's, but within three ghatis of
        // sunrise today's, by the thirty after it.
        assert!(close(at(Rashi::Cancer, 101.1), 101.1 - 30.0 * g, 100.75));
        assert_eq!(at(Rashi::Cancer, 101.25 - g), None);
        assert!(close(
            at(Rashi::Cancer, 100.25 - g),
            100.25,
            100.25 - g + 30.0 * g
        ));
        assert_eq!(at(Rashi::Cancer, 100.1), None);
    }

    #[test]
    fn proportional_ghatis_are_the_arcs_thirtieths() {
        // A long day and a short night: sixteen ghatis of daylight are
        // sixteen thirtieths of it.
        let arcs = Arcs {
            before: Interval::literal(99.9, 100.2),
            daylight: Interval::literal(100.2, 100.9),
            after: Interval::literal(100.9, 101.2),
        };
        let kala = punya_kala(
            Rashi::Taurus,
            JulianDay::literal(100.6),
            &arcs,
            PunyaKalaTable::Dharmasindhu,
            Reckoning::Proportional,
        )
        .ok()
        .flatten()
        .map(|kala| kala.span.inside);
        assert!(close(kala, 100.6 - 16.0 * 0.7 / 30.0, 100.6));
        // Forty ghatis from Makara at dawn cross into the night at its rate.
        let walk = Walk {
            arcs: &arcs,
            reckoning: Reckoning::Proportional,
        };
        let end = walk.step(100.8, 40.0).get();
        let expected = 100.9 + (40.0 - 0.1 / (0.7 / 30.0)) * (0.3 / 30.0);
        assert!((end - expected).abs() < 1e-9);
        assert!((walk.ghatis_between(100.8, end) - 40.0).abs() < 1e-9);
        assert!((walk.step(end, -40.0).get() - 100.8).abs() < 1e-9);
    }

    #[test]
    fn the_tables_differ_only_at_the_equinoxes_and_makara() {
        let differing: Vec<Rashi> = Rashi::ALL
            .into_iter()
            .filter(|sign| {
                let read = |table| ghatis(*sign, table).ok();
                read(PunyaKalaTable::Dharmasindhu) != read(PunyaKalaTable::Hemadri)
                    || read(PunyaKalaTable::Hemadri) != read(PunyaKalaTable::Madhava)
            })
            .collect();
        assert_eq!(differing, [Rashi::Aries, Rashi::Libra, Rashi::Capricorn]);
        // Every sign reads one side but the equinoxes.
        for sign in Rashi::ALL {
            let (before, after) = ghatis(sign, PunyaKalaTable::Dharmasindhu).unwrap_or_default();
            let both = matches!(sign, Rashi::Aries | Rashi::Libra);
            assert_eq!(before > 0 && after > 0, both, "{sign:?}");
        }
    }

    #[test]
    fn a_polar_day_holds_none() {
        let arcs = Arcs {
            daylight: Interval::literal(100.5, 100.5),
            ..ARCS
        };
        let found = punya_kala(
            Rashi::Taurus,
            JulianDay::literal(100.5),
            &arcs,
            PunyaKalaTable::Dharmasindhu,
            Reckoning::Civil,
        );
        assert_eq!(found.ok().flatten(), None);
    }
}

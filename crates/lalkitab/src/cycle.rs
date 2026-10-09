//! The 35-year cycle (1952 pp. 33, ~39, ~219).
//!
//! The nine planets rule in Lal Kitab's order for their cycle years —
//! Jupiter 6, Sun 2, Moon 1, Venus 3, Mars 6, Mercury 2, Saturn 6, Rahu 6,
//! Ketu 3, which is 35 — and the order repeats. The book's general table
//! starts it with Saturn in the first year of life; a reader may start it
//! elsewhere (crux LK-C4), as the book's own example starts Venus at 17,
//! and the cycle then runs back from the start as well as on.
//!
//! A **year of life** is the running year: year 1 runs from birth to the
//! first birthday (crux LK-C2), so "the 16th year" is age 15 completed.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::tables::{PLANETS, cycle_years, reads};

/// The cycle's length in years.
pub const CYCLE: u16 = 35;

/// Where a life's cycle starts: a planet, and the year of life its first
/// period begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CycleStart {
    /// The planet whose period begins then.
    pub planet: Graha,
    /// The year of life it begins, from 1.
    pub year: u16,
}

impl CycleStart {
    /// The book's general table: Saturn from the first year of life.
    pub const GENERAL: CycleStart = CycleStart {
        planet: Graha::Saturn,
        year: 1,
    };

    /// A start at `planet` in year of life `year`.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `planet` for a graha Lal Kitab does not read, and
    /// on `year` for year 0.
    pub fn new(planet: Graha, year: u16) -> Result<CycleStart, Error> {
        if !reads(planet) {
            return Err(Error::invalid_arg(format!(
                "Lal Kitab's cycle runs over the nine grahas, and {} is not one",
                planet.key()
            ))
            .with_field("planet"));
        }
        if year == 0 {
            return Err(Error::invalid_arg(
                "a year of life counts from 1, the year from birth to the first birthday",
            )
            .with_field("year"));
        }
        Ok(CycleStart { planet, year })
    }
}

impl Default for CycleStart {
    fn default() -> CycleStart {
        CycleStart::GENERAL
    }
}

/// One planet's period in the cycle: the years of life it runs, both
/// ends included.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Period {
    /// The planet.
    pub planet: Graha,
    /// Its first year of life.
    pub from: u16,
    /// Its last.
    pub to: u16,
}

/// Where a planet's period begins inside one round of the cycle, in years
/// from Jupiter's.
fn offset_of(planet: Graha) -> u16 {
    PLANETS
        .iter()
        .take_while(|each| **each != planet)
        .map(|each| u16::from(cycle_years(*each)))
        .sum()
}

/// The planet ruling year of life `year` under `start`.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::cycle::{CycleStart, ruler};
///
/// // The general table: Saturn in the first year, Jupiter in the 16th.
/// assert_eq!(ruler(1, CycleStart::GENERAL), Graha::Saturn);
/// assert_eq!(ruler(16, CycleStart::GENERAL), Graha::Jupiter);
/// assert_eq!(ruler(120, CycleStart::GENERAL), Graha::Ketu);
/// ```
#[must_use]
pub fn ruler(year: u16, start: CycleStart) -> Graha {
    let within = i32::from(offset_of(start.planet)) + i32::from(year) - i32::from(start.year);
    let within = u16::try_from(within.rem_euclid(i32::from(CYCLE))).unwrap_or(0);
    let mut reached = 0;
    for planet in PLANETS {
        reached += u16::from(cycle_years(planet));
        if within < reached {
            return planet;
        }
    }
    Graha::Ketu
}

/// The periods that fall in years of life `from` to `to`, both included,
/// each cut to the span.
#[must_use]
pub fn periods(from: u16, to: u16, start: CycleStart) -> Vec<Period> {
    let mut periods: Vec<Period> = Vec::new();
    for year in from.max(1)..=to {
        let planet = ruler(year, start);
        match periods.last_mut() {
            Some(last) if last.planet == planet && last.to + 1 == year => last.to = year,
            _ => periods.push(Period {
                planet,
                from: year,
                to: year,
            }),
        }
    }
    periods
}

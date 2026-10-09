//! A teva read over a life: its reading, the 35-year cycle's periods, and
//! one year of life with the planet ruling it, that planet's thirds and,
//! given a varshphal list, the annual teva's reading.

use serde::Serialize;
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::cycle::{CycleStart, Period, periods, ruler};
use crate::reading::{Reading, read};
use crate::tables::thirds;
use crate::teva::Teva;
use crate::varshphal::{VarshphalTable, YEARS};

/// What a life is read under.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LifeRules {
    /// Where the 35-year cycle starts: the general table unless a reader
    /// says otherwise (crux LK-C4).
    pub cycle: CycleStart,
    /// The year of life to read, from 1 (the year from birth to the first
    /// birthday, crux LK-C2); none reads no year.
    pub year: Option<u16>,
    /// The varshphal list the year's annual teva is read from; none reads
    /// no annual teva (crux LK-C9).
    pub varshphal: Option<VarshphalTable>,
}

/// One year of life.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Year {
    /// The year of life, from 1.
    pub year: u16,
    /// The planet whose period of the 35-year cycle it falls in.
    pub ruler: Graha,
    /// The planets ruling its months 1–4, 5–8 and 9–12, the ruler's thirds
    /// (1952 p. 34; crux LK-C12).
    pub thirds: [Graha; 3],
    /// The annual teva's reading, when a varshphal list was given.
    pub annual: Option<Reading>,
}

/// A teva read over a life.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Life {
    /// Everything the birth teva says.
    pub reading: Reading,
    /// Where the cycle was started.
    pub cycle: CycleStart,
    /// The cycle's periods over years 1 to 120 of life.
    pub periods: Vec<Period>,
    /// The year asked for.
    pub year: Option<Year>,
}

/// A teva read over a life under `rules`.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{LifeRules, Teva, life};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let rules = LifeRules { year: Some(16), ..LifeRules::default() };
/// let read = life(&teva, &rules)?;
/// // The general table gives the 16th year to Jupiter.
/// let year = read.year.unwrap();
/// assert_eq!(year.ruler, Graha::Jupiter);
/// assert_eq!(year.thirds, [Graha::Ketu, Graha::Jupiter, Graha::Sun]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` on `year` outside 1 to 120.
pub fn life(teva: &Teva, rules: &LifeRules) -> Result<Life, Error> {
    let year = rules
        .year
        .map(|year| {
            if !(1..=YEARS).contains(&year) {
                return Err(Error::invalid_arg(format!(
                    "a year of life is 1 to {YEARS}, not {year}"
                ))
                .with_field("year"));
            }
            let planet = ruler(year, rules.cycle);
            let annual = rules
                .varshphal
                .as_ref()
                .map(|table| table.annual(teva, year).map(|annual| read(&annual)))
                .transpose()?;
            Ok(Year {
                year,
                ruler: planet,
                thirds: thirds(planet).unwrap_or([planet; 3]),
                annual,
            })
        })
        .transpose()?;
    Ok(Life {
        reading: read(teva),
        cycle: rules.cycle,
        periods: periods(1, YEARS, rules.cycle),
        year,
    })
}

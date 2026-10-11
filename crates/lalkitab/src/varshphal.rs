//! The annual chart (*varshphal*, 1952 Farman 13 and pp. ~222–225).
//!
//! The book's list gives, for every year of life from 1 to 120, the house
//! each natal house's planets arrive in that year; the planets of one
//! natal house move together. The list is not a rule: its rows are not one
//! permutation's powers and only its first column repeats. So it is data,
//! and data a reader supplies: the list is the author's, in copyright to
//! 2042 in India and to 2047 in the United States (crux LK-C9), so the SDK
//! ships the reader and its checks and not the transcription.
//!
//! What a reader supplies is checked as far as the list's own structure
//! goes: every row a permutation of the twelve houses, and every block of
//! twelve years (1 to 12, 13 to 24, …) a Latin square, each house arriving
//! once in each column. Those two checks are what settle a misread cell.

use serde::Serialize;
use teistro_core::error::Error;

use crate::teva::Teva;

/// The years the list covers.
pub const YEARS: u16 = 120;

/// A varshphal list: per year of life, the house each natal house moves to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VarshphalTable {
    rows: Vec<[u8; 12]>,
}

impl VarshphalTable {
    /// A list from its rows: row `y − 1` is year of life `y`, and its
    /// column `h − 1` the house natal house `h` moves to.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming its field: not 120 rows (`rows`); a row that is
    /// not a permutation of 1 to 12 (`rows[i]`); a house arriving twice in
    /// one column of a twelve-year block (`rows[i][j]`, the second arrival).
    pub fn from_rows(rows: Vec<[u8; 12]>) -> Result<VarshphalTable, Error> {
        if rows.len() != usize::from(YEARS) {
            return Err(Error::invalid_arg(format!(
                "a varshphal list has a row for each of the {YEARS} years of life, not {}",
                rows.len()
            ))
            .with_field("rows"));
        }
        for (index, row) in rows.iter().enumerate() {
            let mut seen = [false; 12];
            for house in row {
                match usize::from(*house)
                    .checked_sub(1)
                    .and_then(|at| seen.get_mut(at))
                {
                    Some(slot) if !*slot => *slot = true,
                    _ => {
                        return Err(Error::invalid_arg(format!(
                            "year {} sends two natal houses to one house, or names a house outside 1 to 12: {row:?}",
                            index + 1
                        ))
                        .with_field(format!("rows[{index}]")));
                    }
                }
            }
        }
        for (block, years) in rows.chunks(12).enumerate() {
            for column in 0..12 {
                let mut seen = [false; 12];
                for (offset, row) in years.iter().enumerate() {
                    let house = row.get(column).copied().unwrap_or(0);
                    let Some(slot) = usize::from(house)
                        .checked_sub(1)
                        .and_then(|at| seen.get_mut(at))
                    else {
                        continue;
                    };
                    if *slot {
                        let index = block * 12 + offset;
                        return Err(Error::invalid_arg(format!(
                            "natal house {} reaches house {house} twice between years {} and {}; each twelve years it reaches each house once",
                            column + 1,
                            block * 12 + 1,
                            index + 1
                        ))
                        .with_field(format!("rows[{index}][{column}]")));
                    }
                    *slot = true;
                }
            }
        }
        Ok(VarshphalTable { rows })
    }

    /// The house natal house `house` moves to in year of life `year`.
    #[must_use]
    pub fn house(&self, year: u16, house: u8) -> Option<u8> {
        let row = self.rows.get(usize::from(year).checked_sub(1)?)?;
        row.get(usize::from(house).checked_sub(1)?).copied()
    }

    /// The annual teva of year of life `year`: every planet moved with the
    /// planets of its natal house.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `year` outside 1 to 120.
    pub fn annual(&self, natal: &Teva, year: u16) -> Result<Teva, Error> {
        if !(1..=YEARS).contains(&year) {
            return Err(Error::invalid_arg(format!(
                "the varshphal list runs from year 1 to {YEARS} of life, not {year}"
            ))
            .with_field("year"));
        }
        Ok(natal.moved(|house| self.house(year, house).unwrap_or(house)))
    }
}

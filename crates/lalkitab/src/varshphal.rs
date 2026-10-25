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
///
/// ```
/// use teistro_lalkitab::{VarshphalTable, YEARS};
///
/// let short = VarshphalTable::from_rows(vec![[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]; usize::from(YEARS) - 1]);
/// assert_eq!(short.unwrap_err().field(), Some("rows"));
/// ```
pub const YEARS: u16 = 120;

/// A varshphal list: per year of life, the house each natal house moves to.
///
/// ```
/// use teistro_lalkitab::VarshphalTable;
///
/// // Not the book's list, which a reader supplies: every house moving one on
/// // each year, which has the structure the book's list has.
/// let rows: Vec<[u8; 12]> = (0..120_u8)
///     .map(|year| std::array::from_fn(|house| (house as u8 + year % 12) % 12 + 1))
///     .collect();
/// let table = VarshphalTable::from_rows(rows)?;
/// assert_eq!(table.house(1, 1), Some(1));
/// assert_eq!(table.house(13, 1), Some(1));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VarshphalTable {
    rows: Vec<[u8; 12]>,
}

impl VarshphalTable {
    /// A list from its rows: row `y − 1` is year of life `y`, and its
    /// column `h − 1` the house natal house `h` moves to.
    ///
    /// ```
    /// use teistro_lalkitab::VarshphalTable;
    ///
    /// // Not the book's list, which a reader supplies: every house moving one on
    /// // each year, which has the structure the book's list has.
    /// let rows: Vec<[u8; 12]> = (0..120_u8)
    ///     .map(|year| std::array::from_fn(|house| (house as u8 + year % 12) % 12 + 1))
    ///     .collect();
    /// let table = VarshphalTable::from_rows(rows)?;
    /// // Every row the same: house 1 reaches house 1 twice in years 1 to 12.
    /// let still = VarshphalTable::from_rows(vec![[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]; 120]);
    /// assert_eq!(still.unwrap_err().field(), Some("rows[1][0]"));
    /// # let _ = table;
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_lalkitab::VarshphalTable;
    ///
    /// // Not the book's list, which a reader supplies: every house moving one on
    /// // each year, which has the structure the book's list has.
    /// let rows: Vec<[u8; 12]> = (0..120_u8)
    ///     .map(|year| std::array::from_fn(|house| (house as u8 + year % 12) % 12 + 1))
    ///     .collect();
    /// let table = VarshphalTable::from_rows(rows)?;
    /// assert_eq!(table.house(3, 12), Some(2));
    /// assert_eq!(table.house(121, 1), None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn house(&self, year: u16, house: u8) -> Option<u8> {
        let row = self.rows.get(usize::from(year).checked_sub(1)?)?;
        row.get(usize::from(house).checked_sub(1)?).copied()
    }

    /// The annual teva of year of life `year`: every planet moved with the
    /// planets of its natal house.
    ///
    /// ```
    /// use teistro_lalkitab::VarshphalTable;
    ///
    /// // Not the book's list, which a reader supplies: every house moving one on
    /// // each year, which has the structure the book's list has.
    /// let rows: Vec<[u8; 12]> = (0..120_u8)
    ///     .map(|year| std::array::from_fn(|house| (house as u8 + year % 12) % 12 + 1))
    ///     .collect();
    /// let table = VarshphalTable::from_rows(rows)?;
    /// use teistro_core::catalogue::Graha;
    /// use teistro_lalkitab::Teva;
    /// use teistro_lalkitab::tables::PLANETS;
    ///
    /// let natal = Teva::from_houses(PLANETS.map(|planet| (planet, 12)))?;
    /// // The planets of a house move together: all nine from 12 to 1 in year 2.
    /// assert_eq!(table.annual(&natal, 2)?.occupants(1).len(), 9);
    /// assert_eq!(table.annual(&natal, 0).unwrap_err().field(), Some("year"));
    /// # let _ = Graha::Sun;
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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

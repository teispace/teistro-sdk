//! Lal Kitab's computable parts (`docs/03-design/lalkitab.md`), read from
//! the 1952 edition, *Ilm-e-Samudrik ki Buniyad par ki Lal Kitab*, and
//! cited by its pages.
//!
//! The book is in copyright (to 2042 in India), so what ships is method:
//! house numbers, planet lists and conditions, each a rule where the book
//! follows one, in the SDK's own words. No sentence of the book is here,
//! and the 120-year varshphal list is a table a reader supplies
//! ([`VarshphalTable::from_rows`]), checked for the structure the book's
//! own list has.
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_lalkitab::{Teva, read};
//!
//! let teva = Teva::from_houses([
//!     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
//!     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
//!     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
//! ])?;
//! let reading = read(&teva);
//! // The Sun in 4 and Saturn in 7: a night-blind chart.
//! assert!(reading.flags.ratandha);
//! // The Moon in 9 with Mercury in its root house: the ancestors' debt.
//! assert_eq!(reading.pitri.len(), 1);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

pub mod aspects;
pub mod cycle;
mod life;
mod reading;
pub mod tables;
mod teva;
mod varshphal;

#[cfg(test)]
mod tests;

pub use cycle::{CycleStart, Period};
pub use life::{Life, LifeRules, Year, life};
pub use reading::{
    Cast, Debt, Dignity, Flags, Formed, HouseReading, Look, OwnerRegard, PitriState, PlanetReading,
    Reading, Seated, read,
};
pub use tables::{Masnui, Regard, Rin};
pub use teva::Teva;
pub use varshphal::{VarshphalTable, YEARS};

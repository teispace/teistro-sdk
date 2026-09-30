//! Muhurta: an elected time judged clause by clause, as the texts judge
//! it (`03-design/muhurta.md`).
//!
//! B. V. Raman's *Muhurtha* (1948) holds that "an absolutely good
//! muhurtha is inconceivable": a time is chosen for an excess of good and
//! a deficiency of evil, each named by a source, some cancelling others.
//! So this crate reports **clauses** — one named condition and the
//! interval it held over — rather than a score, and leaves the ordering
//! to a ranking the caller chooses.
//!
//! A day's clauses read its panchanga and, when there is one, the native:
//!
//! ```
//! use teistro_core::catalogue::{Nakshatra, Rashi};
//! use teistro_muhurta::tara::{ChandraBala, Tara, chandra_house, tara};
//!
//! // Born in Pushya with the Moon in Cancer; a day ruled by Magha with
//! // the Moon in Leo.
//! let reading = tara(Nakshatra::Pushya, Nakshatra::Magha);
//! assert_eq!(reading.tara, Tara::Vipat);
//! // Raman: such a day serves if its first seven ghatis are avoided.
//! assert_eq!(reading.tara.spoiled_ghatis(), Some(7));
//! assert!(ChandraBala::raman().holds(chandra_house(Rashi::Cancer, Rashi::Leo)));
//! ```

#![doc(html_no_source)]

pub mod clause;
pub mod day;
pub mod instant;
pub mod season;
pub mod tara;
pub mod window;

pub use clause::{Clause, ClauseKind};
pub use day::{DayRules, Native, clauses};
pub use instant::Sky;
pub use tara::{ChandraBala, Tara, TaraReading};

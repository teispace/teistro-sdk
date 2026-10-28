//! Nāmakaraṇa: a name read into *Svarodaya*'s śatapada cakra, the
//! syllables a pada, a star or a sign names a child by, a name checked
//! against a birth with the clauses of a name in daily use, and the names
//! a birth gives (`03-design/namakarana.md`, C291 to C299, C334, C770 to
//! C775).
//!
//! ```
//! use teistro_core::catalogue::Nakshatra;
//! use teistro_matching::{Birth, NamingRules, name_check};
//!
//! // Dharmasindhu's Cūḍāmaṇi for an Ashvini first-pada birth.
//! let check = name_check("चूडामणि", Birth { nakshatra: Nakshatra::Ashwini, pada: 1 }, NamingRules::default())?;
//! assert!(check.same_pada);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

mod cakra;
mod check;
mod clauses;
mod latin;
mod pada;
mod read;

pub use cakra::{NameSyllable, NameVarga};
pub use check::{Birth, NameCheck, Names, name_check, names_of};
pub use clauses::{Clause, ClauseRow, ClauseSource, FinalA, GrhyaSutra, NameClauses, name_clauses};
pub use latin::{CASUAL, LatinName};
pub use pada::{
    BirthSyllable, Initial, InitialBy, NasalInitial, PadaSyllable, SyllablesOf, birth_syllable,
    syllables,
};
pub use read::{
    AbhijitPada, NameReading, NameRules, NamingRules, Reading, name_syllable, read_name,
};

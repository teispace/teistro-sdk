//! Matching two births through the Moon of each, and Mars in each, and
//! two names through the star of each (`03-design/matching.md`); and the
//! naming of a child by the same cakra (`03-design/namakarana.md`).
//!
//! Everything here is a reading of tables, so there is no ephemeris, no
//! chart and no instant: a [`Native`] is the Moon's nakshatra, pada, sign
//! and navamsha, and every table is transcribed from a cited source and
//! held to it in the tests.
//!
//! ```
//! use teistro_matching::{KootaRules, Native, ashta_koota};
//!
//! // Two Moons in Rohini: the same nakshatra, so the same yoni, gana and
//! // nadi; the shared nadi takes its eight points.
//! let moon = Native::of_moon(45.0)?;
//! let koota = ashta_koota(moon, moon, KootaRules::default());
//! assert_eq!(koota.total, 28.0);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![doc(html_no_source)]

mod ashta;
mod avakahada;
mod doshas;
mod ghata;
mod kuja;
mod name;
mod naming;
mod native;
mod porutham;
mod role;

pub use ashta::{
    ASHTA_KOOTA, ASHTA_KOOTA_POINTS, AshtaKoota, BhakootDosha, BhakootExceptions, BhakootLift,
    DevaBride, EqualVarna, GanaPoints, KootaReading, KootaRow, KootaRules, MaitriRelation,
    NadiDosha, VashyaPoints, VashyaRelation, YoniRelation, ashta_koota, is_vashya, maitri_relation,
    sign_varna, vashya_relation, yoni_relation,
};
pub use avakahada::{Avakahada, avakahada};
pub use doshas::{DoshaSystem, MarriageDosha, marriage_doshas};
pub use ghata::Ghata;
pub use kuja::{
    KUJA_REFERENCES, Kuja, KujaFrom, KujaHouses, KujaLifts, KujaNative, KujaReading, KujaReference,
    KujaRules, KujaSide, KujaSky, kuja, kuja_side,
};
pub use name::{NaamMilan, NaamRules, VargaKoota, VargaRelation, naam_milan, varga_koota};
pub use naming::{
    AbhijitPada, Birth, BirthSyllable, Book, BookId, BookName, CASUAL, Clause, ClauseRow,
    ClauseSource, FinalA, GrhyaSutra, Initial, InitialBy, LatinName, NameBook, NameCheck,
    NameClauses, NameReading, NameRules, NameSyllable, NameVarga, Names, NamingRules, NasalInitial,
    PadaSyllable, Reading, RefusedName, SuggestFilter, Suggested, Suggestions, SyllablesOf,
    birth_syllable, name_check, name_clauses, name_syllable, names_of, read_name, syllables,
};
pub use native::Native;
pub use porutham::{
    CHIEF_FIVE, DeerghaBeyond, DhinamRule, LordsFriendship, PORUTHAM, Porutham, PoruthamException,
    PoruthamReading, PoruthamRow, PoruthamRules, Rajju, TwoSignStar, porutham,
};
pub use role::MatchRole;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod doshas_tests;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod kuja_tests;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod name_tests;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod naming_book_tests;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod naming_tests;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod porutham_tests;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests;

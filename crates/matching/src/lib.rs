//! Matching two births through the Moon of each, and Mars in each, and
//! two names through the star of each (`03-design/matching.md`).
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

#![doc(html_no_source)]

mod ashta;
mod doshas;
mod kuja;
mod name;
mod native;
mod porutham;
mod role;

pub use ashta::{
    ASHTA_KOOTA, ASHTA_KOOTA_POINTS, AshtaKoota, BhakootDosha, BhakootExceptions, BhakootLift,
    DevaBride, EqualVarna, KootaReading, KootaRow, KootaRules, MaitriRelation, NadiDosha,
    VashyaRelation, YoniRelation, ashta_koota, is_vashya, maitri_relation, sign_varna,
    vashya_relation, yoni_relation,
};
pub use doshas::{DoshaSystem, MarriageDosha, marriage_doshas};
pub use kuja::{
    KUJA_REFERENCES, Kuja, KujaFrom, KujaHouses, KujaNative, KujaReading, KujaReference, KujaRules,
    KujaSide, kuja, kuja_side,
};
pub use name::{
    AbhijitPada, BirthSyllable, LatinName, NaamMilan, NaamRules, NameRules, NameSyllable,
    NameVarga, VargaKoota, VargaRelation, birth_syllable, naam_milan, name_syllable, varga_koota,
};
pub use native::Native;
pub use porutham::{
    CHIEF_FIVE, DeerghaBeyond, DhinamRule, LordsFriendship, PORUTHAM, Porutham, PoruthamException,
    PoruthamReading, PoruthamRow, PoruthamRules, Rajju, TwoSignStar, porutham,
};
pub use role::MatchRole;

#[cfg(test)]
mod doshas_tests;
#[cfg(test)]
mod kuja_tests;
#[cfg(test)]
mod name_tests;
#[cfg(test)]
mod porutham_tests;
#[cfg(test)]
mod tests;

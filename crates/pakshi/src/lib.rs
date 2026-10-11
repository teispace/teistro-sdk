//! Pancha Pakshi (`docs/03-design/pakshi.md`): the five birds of the Tamil
//! tradition, read from Agastya's *Pañcapaṭci Sāstiram* (the 1880
//! Kanchipuram print) and P. V. Jagadisa Ayyar's tables in *South Indian
//! Customs* (1925), with the modern readings of Pulippani's *Biorhythms of
//! Natal Moon* (1993) as named choices.
//!
//! The four printed tables of what each bird does in each yama are one
//! rule ([`tables::activity`]): each half has its sequence, each weekday's
//! half its first eater, and neighbouring birds stand a fixed step apart
//! in the sequence. The rule is checked against every cell Ayyar prints.
//!
//! ```
//! use teistro_core::catalogue::{Nakshatra, Paksha, Vara};
//! use teistro_pakshi::{Activity, BirthBird, Bird, Half, birth_bird, tables};
//!
//! // Uttarashadha in the bright half is the cock's star.
//! let bird = birth_bird(Nakshatra::UttaraAshadha, Paksha::Shukla, BirthBird::ByPaksha);
//! assert_eq!(bird, Bird::Cock);
//! // On a bright Wednesday it sleeps in the second yama of the day.
//! let doing = tables::activity(bird, Paksha::Shukla, Half::Day, Vara::Budhavara, 1);
//! assert_eq!(doing, Activity::Sleeping);
//! ```

mod day;
pub mod tables;

#[cfg(test)]
mod tests;

pub use day::{Clock, Day, Now, Reading, Rules, Span, SubPeriod, Yama, now, read_day};
pub use tables::{
    Activity, Bird, BirthBird, Half, Quality, Relation, Relations, Sub, SubLengths, birth_bird,
};

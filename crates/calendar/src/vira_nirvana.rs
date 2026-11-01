//! The Vira Nirvana Samvat: the Jain era counted from Mahavira's nirvana
//! (`03-design/vira-nirvana-and-jain-festivals.md` §2, SR-59).
//!
//! Like Nepal Sambat, the era is a reading of the lunisolar calendar
//! rather than a calendar of its own: its year is a Kartikadi lunar year,
//! the Gujarati Vikrama year, under another number. The sources agree on
//! the shape and differ on two points, and each is a [`Reading`] field
//! rather than a guess:
//!
//! - **The epoch** ([`Epoch`]). The count in use puts the nirvana 470
//!   years before the Vikrama era (Sircar, *Indian Epigraphy*, 1965,
//!   pp. 321-322), 605 years and 5 months before the Shaka (Jinasena's
//!   *Harivamsha*, 783 CE), so 527 BCE. Hemachandra's *Parishishtaparvan*
//!   (VIII.339) has Chandragupta crowned 155 years after the nirvana,
//!   sixty fewer than the count in use, which Jacobi read as 467 BCE.
//!   [`Epoch::Traditional`] is the default: it is the number every Jain
//!   calendar prints, VNS 2552 from Diwali 2025.
//! - **The day the year turns** ([`YearStart`]). The year turns with
//!   Gautama's kevala, the day after the nirvana (JAINA Education
//!   Committee, *Compendium of Jainism*, 3rd ed., p. 248), Kartika Shukla
//!   Pratipada, the same day the Kartikadi Vikrama year opens. Some
//!   calendars print the Jain new year on the nirvana day itself, the
//!   day whose sunrise holds Kartika's new moon (the University of
//!   Southern California's Jain holy days for 2026: 9 November);
//!   [`YearStart::NirvanaDay`] reads that.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Era, Masa, Tithi};
use teistro_core::quantity::{JulianDay, Utc};

use crate::date::EraNumber;
use crate::gregorian::year_from_fixed;
use crate::lunisolar::{MonthKind, kartikadi_middle};

/// The Kartikadi Vikrama year whose middle falls in civil year 0: VS 2082
/// opened at Kartika's new moon in October 2025, and its middle is in
/// 2026.
const VIKRAMA_AT_ZERO: i32 = 56;

/// Where the era's year 1 stands.
///
/// ```
/// use teistro_calendar::vira_nirvana::Epoch;
///
/// assert_eq!(Epoch::Traditional.years_before_vikrama(), 470);
/// assert_eq!(Epoch::Hemachandra.years_before_vikrama(), 410);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Epoch {
    /// The count every Jain calendar prints: the nirvana 470 years
    /// before the Vikrama era, 527 BCE.
    #[default]
    Traditional,
    /// Hemachandra's interval to Chandragupta, sixty years shorter: the
    /// nirvana 410 years before the Vikrama era, 467 BCE.
    Hemachandra,
}

impl Epoch {
    /// The years between the nirvana and the Vikrama era.
    ///
    /// ```
    /// use teistro_calendar::vira_nirvana::Epoch;
    ///
    /// // Hemachandra's count is sixty years shorter.
    /// let gap = Epoch::Traditional.years_before_vikrama() - Epoch::Hemachandra.years_before_vikrama();
    /// assert_eq!(gap, 60);
    /// ```
    #[must_use]
    pub const fn years_before_vikrama(self) -> i32 {
        match self {
            Epoch::Traditional => 470,
            Epoch::Hemachandra => 410,
        }
    }
}

/// The day the era's year turns.
///
/// ```
/// use teistro_calendar::vira_nirvana::YearStart;
///
/// assert_eq!(YearStart::default(), YearStart::KartikaShuklaPratipada);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum YearStart {
    /// The day after the nirvana, the first day of amanta Kartika, when
    /// the Kartikadi Vikrama year opens too.
    #[default]
    KartikaShuklaPratipada,
    /// The nirvana day: the day whose sunrise holds the new moon that
    /// ends the nija Ashwina (amanta), Kartika's dark fifteenth in the
    /// purnimanta count the *Kalpa Sutra* dates the nirvana by.
    NirvanaDay,
}

/// How a day's Vira Nirvana year is read: the two points the sources
/// differ on.
///
/// ```
/// use teistro_calendar::vira_nirvana::{Epoch, Reading, YearStart};
///
/// let reading = Reading::default();
/// assert_eq!((reading.epoch, reading.year_start), (Epoch::Traditional, YearStart::KartikaShuklaPratipada));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    /// Where year 1 stands.
    pub epoch: Epoch,
    /// The day the year turns.
    pub year_start: YearStart,
}

/// The Vira Nirvana year a Kartikadi Vikrama year is.
///
/// ```
/// use teistro_calendar::vira_nirvana::{Epoch, from_kartikadi_vikrama};
///
/// // VS 2082 opened on 22 October 2025, and so did VNS 2552.
/// assert_eq!(from_kartikadi_vikrama(2082, Epoch::Traditional), 2552);
/// assert_eq!(from_kartikadi_vikrama(2082, Epoch::Hemachandra), 2492);
/// ```
#[must_use]
pub const fn from_kartikadi_vikrama(vikrama: i32, epoch: Epoch) -> i32 {
    vikrama + epoch.years_before_vikrama()
}

/// The Vira Nirvana year of a day: its amanta month and that month's
/// kind, the tithi at its sunrise, and the sunrise.
///
/// The year is counted as Nepal Sambat's is, by the civil year the
/// Kartikadi year's middle falls in ([`kartikadi_middle`]). Under
/// [`YearStart::NirvanaDay`] the day whose sunrise holds a nija Ashwina's
/// new moon already counts in the next year; an adhika Ashwina's new
/// moon is not the nirvana's.
///
/// ```
/// use teistro_calendar::lunisolar::MonthKind;
/// use teistro_calendar::vira_nirvana::{Reading, YearStart, year_of};
/// use teistro_core::catalogue::{Masa, Tithi};
/// use teistro_core::quantity::JulianDay;
///
/// let nija = MonthKind::Nija;
/// // 21 October 2025, the nirvana day, still closes VNS 2551; the next
/// // day opens 2552.
/// let nirvana = JulianDay::literal(2_460_969.5);
/// assert_eq!(year_of(Masa::Ashwina, nija, Tithi::Amavasya, nirvana, Reading::default()), 2551);
/// let pratipada = JulianDay::literal(2_460_970.5);
/// assert_eq!(year_of(Masa::Kartika, nija, Tithi::ShuklaPratipada, pratipada, Reading::default()), 2552);
/// // Read from the nirvana day, the year has turned a day earlier.
/// let on_the_day = Reading { year_start: YearStart::NirvanaDay, ..Reading::default() };
/// assert_eq!(year_of(Masa::Ashwina, nija, Tithi::Amavasya, nirvana, on_the_day), 2552);
/// ```
#[must_use]
pub fn year_of(
    amanta: Masa,
    kind: MonthKind,
    at_sunrise: Tithi,
    sunrise: JulianDay<Utc>,
    reading: Reading,
) -> i32 {
    let turned = reading.year_start == YearStart::NirvanaDay
        && amanta == Masa::Ashwina
        && kind != MonthKind::Adhika
        && at_sunrise == Tithi::Amavasya;
    let counted = if turned { Masa::Kartika } else { amanta };
    let vikrama = year_from_fixed(kartikadi_middle(counted, sunrise)) + VIKRAMA_AT_ZERO;
    from_kartikadi_vikrama(vikrama, reading.epoch)
}

/// [`year_of`] as the era number a date carries.
///
/// ```
/// use teistro_calendar::date::EraNumber;
/// use teistro_calendar::lunisolar::MonthKind;
/// use teistro_calendar::vira_nirvana::{Reading, era_of};
/// use teistro_core::catalogue::{Era, Masa, Tithi};
/// use teistro_core::quantity::JulianDay;
///
/// let day = JulianDay::literal(2_461_369.5);
/// let era = era_of(Masa::Kartika, MonthKind::Nija, Tithi::ShuklaPratipada, day, Reading::default());
/// assert_eq!(era, EraNumber { era: Era::ViraNirvana, year: 2553 });
/// ```
#[must_use]
pub fn era_of(
    amanta: Masa,
    kind: MonthKind,
    at_sunrise: Tithi,
    sunrise: JulianDay<Utc>,
    reading: Reading,
) -> EraNumber {
    EraNumber {
        era: Era::ViraNirvana,
        year: year_of(amanta, kind, at_sunrise, sunrise, reading),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    /// 22 October 2025 at 00:00 UT, a sunrise-like instant on the first
    /// day of amanta Kartika.
    const OPENED_2552: f64 = 2_460_970.5;

    /// Every amanta month from Kartika 2025 to Ashwina 2026 is VNS 2552,
    /// and the Ashwina before it 2551.
    #[test]
    fn a_year_runs_from_kartika_to_ashwina() {
        let nija = MonthKind::Nija;
        let reading = Reading::default();
        let months = [
            Masa::Kartika,
            Masa::Margashirsha,
            Masa::Pausha,
            Masa::Magha,
            Masa::Phalguna,
            Masa::Chaitra,
            Masa::Vaishakha,
            Masa::Jyeshtha,
            Masa::Ashadha,
            Masa::Shravana,
            Masa::Bhadrapada,
            Masa::Ashwina,
        ];
        for (month, step) in months.into_iter().zip(0_u8..) {
            let day = OPENED_2552 + 29.53 * f64::from(step);
            let at = JulianDay::literal(day + 10.0);
            assert_eq!(
                year_of(month, nija, Tithi::ShuklaDashami, at, reading),
                2552,
                "{month:?}"
            );
        }
        let before = JulianDay::literal(OPENED_2552 - 5.0);
        assert_eq!(
            year_of(Masa::Ashwina, nija, Tithi::KrishnaEkadashi, before, reading),
            2551
        );
    }

    /// An adhika Ashwina's new moon is not the nirvana's, whichever day
    /// the year turns on.
    #[test]
    fn an_adhika_new_moon_does_not_turn_the_year() {
        let on_the_day = Reading {
            year_start: YearStart::NirvanaDay,
            ..Reading::default()
        };
        let at = JulianDay::literal(OPENED_2552 - 31.0);
        let adhika = year_of(
            Masa::Ashwina,
            MonthKind::Adhika,
            Tithi::Amavasya,
            at,
            on_the_day,
        );
        assert_eq!(adhika, 2551);
    }

    /// Hemachandra's count is sixty years fewer on every day.
    #[test]
    fn hemachandras_epoch_is_sixty_years_later() {
        let at = JulianDay::literal(OPENED_2552);
        let hemachandra = Reading {
            epoch: Epoch::Hemachandra,
            ..Reading::default()
        };
        let read = |reading| {
            year_of(
                Masa::Kartika,
                MonthKind::Nija,
                Tithi::ShuklaPratipada,
                at,
                reading,
            )
        };
        assert_eq!(read(Reading::default()) - read(hemachandra), 60);
    }
}

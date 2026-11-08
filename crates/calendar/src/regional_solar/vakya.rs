//! The Tamil calendar of the vakya school (`docs/calendars/regional-solar.md`,
//! "The two Tamil schools"): the months the vakya almanacs print, from the
//! *Vakyakarana*'s Sun ([`teistro_siddhanta::vakya`]) rather than the
//! modern one.
//!
//! A month begins on the day whose sunrise the sankranti follows when it
//! falls within that day's daylight, the almanac's own day length (thirty
//! nadis less the cara, III.7–8, for a shadow of 2¼ angulas), and on the
//! next day when it falls in the night: the Tamil rule of the sunset on
//! the vakya sky. The sankranti's time is the *Vakyakarana*'s, its
//! fraction of a day read as the time after the day's sunrise, since the
//! almanacs make no correction for the place (the edition's introduction).
//!
//! The Manonmani Vilasam suddha vakya almanac's month beginnings agree
//! with this, every one read (Sarvajit, 1947, to Subhakrit, 1962, and
//! Isvara, 1997, to Plava, 2021: 484 months), with its printed sankranti
//! times to the vinadi and its printed day lengths within two vinadis; so
//! do the Koviloor vakya almanac's for 1978.

use std::sync::OnceLock;

use teistro_siddhanta::vakya::{self, Palabha};

use super::{GREGORIAN_SPAN, TAMIL, Table};
use crate::fixed::FixedDay;
use crate::gregorian::gregorian_from_fixed;

/// The shadow the day length is reckoned for: 2 angulas 15 vyangulas,
/// the shadow the almanac's printed day lengths fit (crux C1201), about
/// latitude 10° 37′ N, the Kaveri delta's.
pub const PALABHA: Palabha = Palabha::from_vyangulas(135);

/// The Kali days from the *Vakyakarana*'s epoch to the SDK's fixed day 0:
/// Kali day 1,843,824 is 13 April 1947.
const KALI_DAY_OF_FIXED_ZERO: i64 = 1_132_960;

/// The frame the vakya calendar's dates name.
pub const FRAME: &str = "Vakyakarana (Kuppanna Sastri and Sarma, 1962): \
    the Kali year rule and the sankranti vakyas, precession unreflected; \
    rule BEFORE_SUNSET on the vakya daytime, palabha 2 angulas 15 vyangulas";

/// The day a sign's month begins in the vakya school, in the Tamil year
/// a Saka year names (years elapsed, as the Tamil calendar counts), 0 for
/// Mesha; `None` for a sign outside 0 to 11.
///
/// ```
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_calendar::regional_solar::vakya;
///
/// // Saka 1869: the Mesha sankranti fell at night (40 nadis after the
/// // Sunday's sunrise), so the year began on Monday, 14 April 1947.
/// assert_eq!(vakya::month_start(1869, 0), Some(fixed_from_gregorian(1947, 4, 14)));
/// assert_eq!(vakya::month_start(1869, 12), None);
/// ```
#[must_use]
pub fn month_start(saka: i32, sign: u8) -> Option<FixedDay> {
    let kali = vakya::kali_years(i64::from(saka));
    let moment = vakya::sankranti(kali, sign)?;
    let daytime = vakya::day_length_vinadis(30.0 * f64::from(sign), kali, PALABHA);
    let night = moment.vinadis_after_sunrise() >= daytime;
    Some(FixedDay::new(
        moment.day() + i64::from(night) - KALI_DAY_OF_FIXED_ZERO,
    ))
}

/// The vakya calendar's table over the shipped span, computed on first
/// use: the arithmetic is the *Vakyakarana*'s, so it needs no ephemeris
/// and no generated rows.
///
/// ```
/// use teistro_calendar::regional_solar::vakya;
///
/// let table = vakya::table();
/// assert_eq!(table.first_year, 1722);
/// assert_eq!(table.month_lengths.len(), 599);
/// assert!(table.frame.starts_with("Vakyakarana"));
/// ```
#[must_use]
pub fn table() -> &'static Table {
    static MONTHS: OnceLock<Vec<[u8; 12]>> = OnceLock::new();
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        let (first, last) = GREGORIAN_SPAN;
        let offset = TAMIL.year_start.gregorian_offset;
        let (first_year, last_year) = (first - offset, last - offset);
        let months = MONTHS.get_or_init(|| {
            (first_year..=last_year)
                .map(|year| month_lengths(year).unwrap_or_default())
                .collect()
        });
        Table {
            first_year,
            first_start: gregorian_from_fixed(month_start(first_year, 0).unwrap_or_default()),
            month_lengths: months.as_slice(),
            frame: FRAME,
        }
    })
}

/// A year's month lengths, from each month's first day to the next's.
fn month_lengths(year: i32) -> Option<[u8; 12]> {
    let mut lengths = [0; 12];
    let mut start = month_start(year, 0)?;
    for (sign, length) in (1..=12).zip(lengths.iter_mut()) {
        let next = if sign == 12 {
            month_start(year + 1, 0)?
        } else {
            month_start(year, sign)?
        };
        *length = u8::try_from(start.days_until(next)).ok()?;
        start = next;
    }
    Some(lengths)
}

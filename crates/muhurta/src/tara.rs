//! Tarabala and Chandrabala: the day read against the native (Raman,
//! *Muhurtha*, ch. III).
//!
//! Both count from the native's birth. **Tarabala** counts the day's
//! nakshatra from the birth nakshatra and reduces it by nine; the nine
//! taras are fixed and four of them are unfavourable. **Chandrabala**
//! counts the day's Moon sign from the birth Moon sign; which houses are
//! to be avoided is a table the sources disagree on (crux C160), so it is
//! a value the caller names.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Nakshatra, Rashi};

/// The nine taras, in the order a count from the birth star reaches
/// them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Tara {
    /// 1, the birth star itself: "danger to body".
    Janma,
    /// 2, wealth and prosperity.
    Sampat,
    /// 3, dangers, losses and accidents.
    Vipat,
    /// 4, prosperity.
    Kshema,
    /// 5, obstacles.
    Pratyak,
    /// 6, the realisation of ambitions.
    Sadhana,
    /// 7, dangers.
    Naidhana,
    /// 8, good.
    Mitra,
    /// 9, very favourable.
    ParamaMitra,
}

impl Tara {
    /// Every tara, in order.
    pub const ALL: [Tara; 9] = [
        Tara::Janma,
        Tara::Sampat,
        Tara::Vipat,
        Tara::Kshema,
        Tara::Pratyak,
        Tara::Sadhana,
        Tara::Naidhana,
        Tara::Mitra,
        Tara::ParamaMitra,
    ];

    /// Its place in the nine, 1 to 9.
    #[must_use]
    pub const fn number(self) -> u8 {
        self as u8 + 1
    }

    /// Whether the tara is favourable: every one but Janma, Vipat,
    /// Pratyak and Naidhana (the 1st, 3rd, 5th and 7th).
    ///
    /// Janma's own exceptions — favourable for some rites and not others
    /// — belong to an activity's rules, not to the tara.
    #[must_use]
    pub const fn favourable(self) -> bool {
        !matches!(
            self,
            Tara::Janma | Tara::Vipat | Tara::Pratyak | Tara::Naidhana
        )
    }

    /// How many ghatis at the start of the day's star an unfavourable
    /// tara spoils, when the day is otherwise good: 1 for Janma, 7 for
    /// Vipat, 3 for Pratyak and 8 for Naidhana (Raman, ch. III, p. 22).
    /// `None` for a favourable tara.
    #[must_use]
    pub const fn spoiled_ghatis(self) -> Option<u8> {
        match self {
            Tara::Janma => Some(1),
            Tara::Vipat => Some(7),
            Tara::Pratyak => Some(3),
            Tara::Naidhana => Some(8),
            _ => None,
        }
    }
}

/// A day's star counted from a birth star.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TaraReading {
    /// The count, 1 (the birth star itself) to 27.
    pub count: u8,
    /// The count reduced by nine.
    pub tara: Tara,
    /// Which round of nine the count is in, 1 to 3.
    pub cycle: u8,
}

/// The day's star counted from the birth star, inclusively.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_muhurta::tara::{Tara, tara};
///
/// // Raman's example (ch. III): born in Ashwini, a journey on a day
/// // ruled by Shravana counts 22, which leaves 4: Kshema, favourable.
/// let reading = tara(Nakshatra::Ashwini, Nakshatra::Shravana);
/// assert_eq!(reading.count, 22);
/// assert_eq!(reading.tara, Tara::Kshema);
/// assert!(reading.tara.favourable());
/// ```
#[must_use]
pub fn tara(birth: Nakshatra, day: Nakshatra) -> TaraReading {
    let count = count_from(birth.id(), day.id(), 27);
    let index = usize::from((count - 1) % 9);
    TaraReading {
        count,
        tara: Tara::ALL.get(index).copied().unwrap_or(Tara::Janma),
        cycle: (count - 1) / 9 + 1,
    }
}

/// The house the day's Moon sign is from the birth Moon sign, 1 to 12.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_muhurta::tara::chandra_house;
///
/// // Raman's second example: a Taurus Moon at birth and the Moon in
/// // Aries on the day is the 12th.
/// assert_eq!(chandra_house(Rashi::Taurus, Rashi::Aries), 12);
/// ```
#[must_use]
pub fn chandra_house(birth: Rashi, day: Rashi) -> u8 {
    count_from(birth.id(), day.id(), 12)
}

/// Which houses from the birth Moon sign spoil a day (crux C160).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ChandraBala {
    /// The houses, 1 to 12, in which the day's Moon gives no Chandrabala.
    pub avoid: Vec<u8>,
}

impl ChandraBala {
    /// Raman's: the 6th, 8th and 12th (ch. III).
    #[must_use]
    pub fn raman() -> ChandraBala {
        ChandraBala {
            avoid: vec![6, 8, 12],
        }
    }

    /// The baseline engine's: good in 1, 3, 6, 7, 10 and 11, so avoided
    /// in the other six.
    #[must_use]
    pub fn baseline() -> ChandraBala {
        ChandraBala {
            avoid: vec![2, 4, 5, 8, 9, 12],
        }
    }

    /// Whether the Moon in a house gives Chandrabala.
    #[must_use]
    pub fn holds(&self, house: u8) -> bool {
        !self.avoid.contains(&house)
    }
}

/// `to` counted from `from` inclusively in a cycle of `n`, 1 to `n`.
fn count_from(from: u16, to: u16, n: u16) -> u8 {
    let steps = (to + n - from % n) % n;
    u8::try_from(steps + 1).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::{ChandraBala, Tara, chandra_house, tara};
    use teistro_core::catalogue::{Nakshatra, Rashi};

    #[test]
    fn ramans_second_example_has_neither_bala() {
        // Born in Mrigashira (Taurus), married on a day ruled by Bharani
        // with the Moon in Aries: Bharani is Naidhana to Mrigashira and
        // Aries the 12th from Taurus, "most inauspicious".
        let reading = tara(Nakshatra::Mrigashira, Nakshatra::Bharani);
        assert_eq!(reading.tara, Tara::Naidhana);
        assert!(!reading.tara.favourable());
        assert!(!ChandraBala::raman().holds(chandra_house(Rashi::Taurus, Rashi::Aries)));
    }

    #[test]
    fn the_birth_star_is_janma_in_the_first_cycle() {
        let reading = tara(Nakshatra::Revati, Nakshatra::Revati);
        assert_eq!(
            (reading.count, reading.tara, reading.cycle),
            (1, Tara::Janma, 1)
        );
        // The count wraps: Ashwini is the 2nd from Revati.
        assert_eq!(
            tara(Nakshatra::Revati, Nakshatra::Ashwini).tara,
            Tara::Sampat
        );
        // And the 27th is the last of the third cycle.
        let last = tara(Nakshatra::Ashwini, Nakshatra::Revati);
        assert_eq!(
            (last.count, last.tara, last.cycle),
            (27, Tara::ParamaMitra, 3)
        );
    }

    #[test]
    fn four_taras_are_unfavourable_and_each_spoils_its_ghatis() {
        let bad: Vec<u8> = Tara::ALL
            .iter()
            .filter(|t| !t.favourable())
            .map(|t| t.number())
            .collect();
        assert_eq!(bad, [1, 3, 5, 7]);
        for t in Tara::ALL {
            assert_eq!(t.spoiled_ghatis().is_some(), !t.favourable(), "{t:?}");
        }
    }

    #[test]
    fn the_two_chandrabala_tables_part_where_crux_c160_says() {
        let (raman, baseline) = (ChandraBala::raman(), ChandraBala::baseline());
        let parts: Vec<u8> = (1..=12)
            .filter(|h| raman.holds(*h) != baseline.holds(*h))
            .collect();
        // Raman avoids the 6th, which the baseline counts good, and
        // keeps the 2nd, 4th, 5th and 9th, which the baseline drains.
        assert_eq!(parts, [2, 4, 5, 6, 9]);
        assert_eq!(chandra_house(Rashi::Leo, Rashi::Leo), 1);
    }
}

//! The birth paya, by the Moon's house from the lagna and by the Moon's
//! nakshatra (`03-design/matching.md`, C1105 to C1107).

use teistro_core::catalogue::{Nakshatra, Paya, Rashi};

/// The paya of a Moon in a nakshatra (C1106).
///
/// The *Vishwavidyalaya Panchang* (Darbhanga, 2005–06, leaf 10): the
/// twelve stars from Ardra to Anuradha are silver, the seven from
/// Jyeshtha to Shatabhisha gold, the five from Purva Bhadrapada to
/// Bharani copper, and Krittika, Rohini and Mrigashira iron.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Paya};
/// use teistro_matching::paya_by_nakshatra;
///
/// assert_eq!(paya_by_nakshatra(Nakshatra::Ardra), Paya::Rajata);
/// assert_eq!(paya_by_nakshatra(Nakshatra::Jyeshtha), Paya::Suvarna);
/// assert_eq!(paya_by_nakshatra(Nakshatra::Ashwini), Paya::Tamra);
/// assert_eq!(paya_by_nakshatra(Nakshatra::Rohini), Paya::Loha);
/// ```
#[must_use]
pub fn paya_by_nakshatra(nakshatra: Nakshatra) -> Paya {
    match nakshatra.id() {
        5..=16 => Paya::Rajata,
        17..=23 => Paya::Suvarna,
        2..=4 => Paya::Loha,
        _ => Paya::Tamra,
    }
}

/// The paya of a Moon in a house counted from the lagna, 1 to 12 (C1105).
///
/// *Sangraha Shiromani* XIII v. 92 and the *Vishwavidyalaya Panchang*:
/// the 1st, 6th and 11th are gold, the 2nd, 5th and 9th silver, the 3rd,
/// 7th and 10th copper, and the 4th, 8th and 12th iron. `None` for a
/// house outside 1 to 12.
///
/// ```
/// use teistro_core::catalogue::Paya;
/// use teistro_matching::paya_by_house;
///
/// assert_eq!(paya_by_house(11), Some(Paya::Suvarna));
/// assert_eq!(paya_by_house(9), Some(Paya::Rajata));
/// assert_eq!(paya_by_house(7), Some(Paya::Tamra));
/// assert_eq!(paya_by_house(12), Some(Paya::Loha));
/// assert_eq!(paya_by_house(13), None);
/// ```
#[must_use]
pub fn paya_by_house(house: u8) -> Option<Paya> {
    match house {
        1 | 6 | 11 => Some(Paya::Suvarna),
        2 | 5 | 9 => Some(Paya::Rajata),
        3 | 7 | 10 => Some(Paya::Tamra),
        4 | 8 | 12 => Some(Paya::Loha),
        _ => None,
    }
}

/// The house a sign is in counted from the lagna's sign, 1 to 12: whole
/// signs, as a patrika counts the Moon's place (C1107).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_matching::house_from_lagna;
///
/// assert_eq!(house_from_lagna(Rashi::Leo, Rashi::Leo), 1);
/// assert_eq!(house_from_lagna(Rashi::Leo, Rashi::Cancer), 12);
/// ```
#[must_use]
pub fn house_from_lagna(lagna: Rashi, sign: Rashi) -> u8 {
    let from = (sign.id() + 12 - lagna.id()) % 12;
    // `from` is below 12, so it fits a u8.
    u8::try_from(from).map_or(1, |from| from + 1)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::catalogue::{Nakshatra, Paya, Rashi};

    use super::{house_from_lagna, paya_by_house, paya_by_nakshatra};

    #[test]
    fn every_star_reads_as_the_panchang_lists_it() {
        // The panchang's four runs, as it prints them: from a star, so many.
        let runs = [
            (Nakshatra::Ardra, 12, Paya::Rajata),
            (Nakshatra::Jyeshtha, 7, Paya::Suvarna),
            (Nakshatra::PurvaBhadrapada, 5, Paya::Tamra),
            (Nakshatra::Krittika, 3, Paya::Loha),
        ];
        let mut seen = 0;
        for (from, count, paya) in runs {
            for step in 0..count {
                let at = usize::from(from.id()) + step;
                let star = *Nakshatra::ALL.get(at % 27).unwrap();
                assert_eq!(paya_by_nakshatra(star), paya, "{star:?}");
                seen += 1;
            }
        }
        assert_eq!(seen, 27, "the four runs cover every star once");
    }

    #[test]
    fn every_house_reads_as_the_verse_lists_it() {
        let verse = [
            ([1, 6, 11], Paya::Suvarna),
            ([2, 5, 9], Paya::Rajata),
            ([3, 7, 10], Paya::Tamra),
            ([4, 8, 12], Paya::Loha),
        ];
        for (houses, paya) in verse {
            for house in houses {
                assert_eq!(paya_by_house(house), Some(paya), "house {house}");
            }
        }
        assert_eq!(paya_by_house(0), None);
    }

    #[test]
    fn a_house_is_counted_by_whole_signs() {
        for lagna in Rashi::ALL {
            for (step, sign) in Rashi::ALL
                .iter()
                .cycle()
                .skip(usize::from(lagna.id()))
                .take(12)
                .enumerate()
            {
                assert_eq!(
                    usize::from(house_from_lagna(lagna, *sign)),
                    step + 1,
                    "{lagna:?} {sign:?}"
                );
            }
        }
    }
}

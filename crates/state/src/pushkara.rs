//! The pushkara navamshas and bhagas: the portions of a sign that nourish
//! what stands in them.
//!
//! *Vidyamadhaviyam* ch. 1 v. 18 defines both in one verse, which its
//! commentary, Vishnusharma's *Muhurtadipika*, decodes sign by sign (ed.
//! R. Shama Sastry, Mysore, 1923, pp. 80 and 81). The **navamshas** are
//! two in each sign, the same for the three signs of a triplicity:
//!
//! | signs | navamshas |
//! |---|---|
//! | Aries, Leo, Sagittarius | the 7th and the 9th |
//! | Taurus, Virgo, Capricorn | the 3rd and the 5th |
//! | Gemini, Libra, Aquarius | the 6th and the 8th |
//! | Cancer, Scorpio, Pisces | the 1st and the 3rd |
//!
//! The table checks itself: every one of the twenty-four falls in a sign
//! that Jupiter, Venus, the Moon or Mercury owns.
//!
//! The **bhagas** are one degree a sign: the 21st, the 14th, the 24th
//! and the 7th of the same four triplicities. *Jataka Parijata* ch. 1
//! v. 58 gives the Moon a row of her own that agrees in four signs of
//! twelve; `state.pushkara_bhaga` chooses (crux C767). A degree is
//! counted as the rules engine counts Jataka Parijata's (crux C82): the
//! 21st degree runs from 20° to 21°.
//!
//! ```
//! use teistro_core::angle::Nas;
//! use teistro_core::settings::knobs::PushkaraBhaga;
//! use teistro_state::pushkara::{in_bhaga, in_navamsha};
//! use teistro_core::catalogue::Graha;
//!
//! // Aries's seventh navamsha, 20° to 23°20′, is pushkara; its eighth is not.
//! assert!(in_navamsha(Nas::try_from_degrees(21.0)?));
//! assert!(!in_navamsha(Nas::try_from_degrees(25.0)?));
//! // 20°30′ Aries is in the 21st degree.
//! let at = Nas::try_from_degrees(20.5)?;
//! assert_eq!(in_bhaga(Graha::Mars, at, PushkaraBhaga::Vidyamadhaviya), Some(true));
//! // Jataka Parijata's row is the Moon's alone.
//! assert_eq!(in_bhaga(Graha::Mars, at, PushkaraBhaga::JatakaParijata), None);
//! assert_eq!(in_bhaga(Graha::Moon, at, PushkaraBhaga::JatakaParijata), Some(true));
//! # Ok::<(), teistro_core::quantity::InvalidValue>(())
//! ```

use teistro_core::angle::Nas;
use teistro_core::catalogue::Graha;
use teistro_core::settings::knobs::PushkaraBhaga;

/// The two pushkara navamshas of each triplicity, counted from 1 in the
/// sign: fiery, earthy, airy, watery (*Vidyamadhaviyam* ch. 1 v. 18).
///
/// ```
/// use teistro_state::pushkara::NAVAMSHAS;
///
/// // Aries, Leo and Sagittarius: the seventh and the ninth.
/// assert_eq!(NAVAMSHAS[0], [7, 9]);
/// ```
pub const NAVAMSHAS: [[u32; 2]; 4] = [[7, 9], [3, 5], [6, 8], [1, 3]];

/// The pushkara bhaga of each triplicity, a degree counted from 1 in the
/// sign: fiery, earthy, airy, watery (*Vidyamadhaviyam* ch. 1 v. 18).
///
/// ```
/// use teistro_state::pushkara::VIDYAMADHAVIYA_BHAGAS;
///
/// assert_eq!(VIDYAMADHAVIYA_BHAGAS, [21, 14, 24, 7]);
/// ```
pub const VIDYAMADHAVIYA_BHAGAS: [u32; 4] = [21, 14, 24, 7];

/// The Moon's pushkara bhagas, Aries first, a degree counted from 1 in
/// the sign (*Jataka Parijata* ch. 1 v. 58; V. Subrahmanya Sastri, 1932,
/// vol. I p. 39). The rules engine ships the same row as the table
/// `PUSHKARA_BHAGA_MOON`.
///
/// ```
/// use teistro_state::pushkara::JATAKA_PARIJATA_MOON;
///
/// // Cancer's is the 8th, where Vidyamadhaviyam has the 7th.
/// assert_eq!(JATAKA_PARIJATA_MOON[3], 8);
/// ```
pub const JATAKA_PARIJATA_MOON: [u32; 12] = [21, 14, 18, 8, 19, 9, 24, 11, 23, 14, 19, 9];

/// Which triplicity a sign is in: 0 fiery, 1 earthy, 2 airy, 3 watery.
/// Aries is fiery and the elements follow in that order round the zodiac.
const fn triplicity(longitude: Nas) -> usize {
    (longitude.sign_index().get() % 4) as usize
}

/// Whether a longitude is in one of its sign's two pushkara navamshas.
///
/// ```
/// use teistro_core::angle::Nas;
/// use teistro_state::pushkara::in_navamsha;
///
/// // Cancer's first navamsha, 0° to 3°20′, is pushkara; its second is not.
/// assert!(in_navamsha(Nas::try_from_degrees(91.0)?));
/// assert!(!in_navamsha(Nas::try_from_degrees(94.0)?));
/// # Ok::<(), teistro_core::quantity::InvalidValue>(())
/// ```
#[must_use]
pub fn in_navamsha(longitude: Nas) -> bool {
    let navamsha = longitude.part(9) + 1;
    NAVAMSHAS
        .get(triplicity(longitude))
        .is_some_and(|pair| pair.contains(&navamsha))
}

/// Whether a graha at a longitude stands on a pushkara bhaga of the table
/// `table` names, or `None` when that table gives the graha no row —
/// Jataka Parijata's gives the Moon's alone.
///
/// ```
/// use teistro_core::angle::Nas;
/// use teistro_core::catalogue::Graha;
/// use teistro_core::settings::knobs::PushkaraBhaga;
/// use teistro_state::pushkara::in_bhaga;
///
/// // 13°10′ Taurus: the 14th degree, which both tables give Taurus.
/// let at = Nas::try_from_degrees(43.0 + 10.0 / 60.0)?;
/// assert_eq!(in_bhaga(Graha::Moon, at, PushkaraBhaga::Vidyamadhaviya), Some(true));
/// assert_eq!(in_bhaga(Graha::Moon, at, PushkaraBhaga::JatakaParijata), Some(true));
/// // 14°10′ is the 15th, which neither does.
/// let past = Nas::try_from_degrees(44.0 + 10.0 / 60.0)?;
/// assert_eq!(in_bhaga(Graha::Moon, past, PushkaraBhaga::Vidyamadhaviya), Some(false));
/// # Ok::<(), teistro_core::quantity::InvalidValue>(())
/// ```
#[must_use]
pub fn in_bhaga(graha: Graha, longitude: Nas, table: PushkaraBhaga) -> Option<bool> {
    let degree = match table {
        PushkaraBhaga::JatakaParijata if graha != Graha::Moon => return None,
        PushkaraBhaga::JatakaParijata => JATAKA_PARIJATA_MOON
            .get(usize::from(longitude.sign_index().get()))
            .copied()?,
        // The knob is non-exhaustive in its own crate only; every later
        // table is a new arm above this one.
        _ => VIDYAMADHAVIYA_BHAGAS.get(triplicity(longitude)).copied()?,
    };
    // The degree running to n: from n − 1° up to n°.
    Some(longitude.part(30) + 1 == degree)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests unwrap what they built and index tables they know the length of"
    )]

    use super::*;
    use teistro_core::catalogue::{Rashi, Tatwa};

    fn at(degrees: f64) -> Nas {
        Nas::try_from_degrees(degrees).unwrap()
    }

    #[test]
    fn the_triplicity_is_the_catalogues_element() {
        for sign in Rashi::ALL {
            let start = at(f64::from(sign.id()) * 30.0);
            let element =
                [Tatwa::Agni, Tatwa::Prithvi, Tatwa::Vayu, Tatwa::Jala][triplicity(start)];
            assert_eq!(element, sign.attributes().element, "{sign}");
        }
    }

    #[test]
    fn every_pushkara_navamsha_is_a_benefics_sign() {
        // The navamsha signs run from Aries, Capricorn, Libra and Cancer
        // for the four triplicities; each pushkara one is owned by Jupiter,
        // Venus, the Moon or Mercury, which is the table checking itself.
        for sign in Rashi::ALL {
            for part in 0..9_u32 {
                let longitude =
                    at(f64::from(sign.id()) * 30.0 + (f64::from(part) + 0.5) * 30.0 / 9.0);
                if !in_navamsha(longitude) {
                    continue;
                }
                let navamsha_sign =
                    Rashi::from_id(u16::try_from(longitude.division_index(108) % 12).unwrap())
                        .unwrap();
                assert!(
                    matches!(
                        navamsha_sign.attributes().lord,
                        Graha::Jupiter | Graha::Venus | Graha::Moon | Graha::Mercury
                    ),
                    "{sign} part {part}: {navamsha_sign}"
                );
            }
        }
    }

    #[test]
    fn each_sign_has_two_navamshas_and_one_degree() {
        for sign in Rashi::ALL {
            let base = f64::from(sign.id()) * 30.0;
            let navamshas = (0..9_u32)
                .filter(|part| in_navamsha(at(base + (f64::from(*part) + 0.5) * 30.0 / 9.0)))
                .count();
            assert_eq!(navamshas, 2, "{sign}");
            for table in PushkaraBhaga::ALL {
                let degrees = (0..30_u32)
                    .filter(|d| {
                        in_bhaga(Graha::Moon, at(base + f64::from(*d) + 0.5), *table) == Some(true)
                    })
                    .count();
                assert_eq!(degrees, 1, "{sign} {table}");
            }
        }
    }

    #[test]
    fn a_navamsha_includes_its_start_and_excludes_its_end() {
        // Aries's seventh navamsha is 20° to 23°20′.
        assert!(!in_navamsha(at(20.0 - 1e-9)));
        assert!(in_navamsha(at(20.0)));
        assert!(!in_navamsha(at(23.0 + 1.0 / 3.0)));
        // The ninth ends the sign; Taurus's first is not pushkara.
        assert!(in_navamsha(at(30.0 - 1e-9)));
        assert!(!in_navamsha(at(30.0)));
    }

    #[test]
    fn a_degree_is_the_one_running_to_it() {
        // Pisces's 7th degree, 336° to 337°, under Vidyamadhaviyam; the
        // Moon's 9th, 338° to 339°, under Jataka Parijata.
        let vm = PushkaraBhaga::Vidyamadhaviya;
        let jp = PushkaraBhaga::JatakaParijata;
        assert_eq!(in_bhaga(Graha::Saturn, at(335.999), vm), Some(false));
        assert_eq!(in_bhaga(Graha::Saturn, at(336.0), vm), Some(true));
        assert_eq!(in_bhaga(Graha::Saturn, at(337.0), vm), Some(false));
        assert_eq!(in_bhaga(Graha::Moon, at(338.5), jp), Some(true));
        assert_eq!(in_bhaga(Graha::Moon, at(336.5), jp), Some(false));
        assert_eq!(in_bhaga(Graha::Rahu, at(338.5), jp), None);
    }

    #[test]
    fn the_two_tables_agree_in_four_signs() {
        let agreeing: Vec<u16> = Rashi::ALL
            .into_iter()
            .filter(|sign| {
                let index = usize::from(sign.id());
                VIDYAMADHAVIYA_BHAGAS[index % 4] == JATAKA_PARIJATA_MOON[index]
            })
            .map(Rashi::id)
            .collect();
        // Aries, Taurus, Libra and Capricorn (crux C767).
        assert_eq!(agreeing, [0, 1, 6, 9]);
    }
}

//! What the zodiac's signs answer of a longitude: the one place a sign is
//! read from a longitude, which ten crates had each written for themselves.

use super::Rashi;

impl Rashi {
    /// The sign a longitude stands in, the longitude taken in any turn:
    /// 30° is Taurus, −1° is Pisces, 390° is Taurus.
    ///
    /// A longitude that is not a number has no sign and is read as Aries,
    /// because a caller that reaches here has already refused it or never
    /// could; checking is the caller's, at the field it names.
    ///
    /// ```
    /// use teistro_core::catalogue::Rashi;
    ///
    /// assert_eq!(Rashi::of_longitude(0.0), Rashi::Aries);
    /// assert_eq!(Rashi::of_longitude(29.999), Rashi::Aries);
    /// assert_eq!(Rashi::of_longitude(30.0), Rashi::Taurus);
    /// assert_eq!(Rashi::of_longitude(-1.0), Rashi::Pisces);
    /// assert_eq!(Rashi::of_longitude(390.0), Rashi::Taurus);
    /// ```
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude folded into 0..360 divides by thirty into 0..12"
    )]
    pub fn of_longitude(longitude_deg: f64) -> Rashi {
        let index = (longitude_deg.rem_euclid(360.0) / 30.0) as usize % 12;
        Rashi::ALL.get(index).copied().unwrap_or(Rashi::Aries)
    }

    /// The sign six from this one: Libra for Aries, Aries for Libra.
    ///
    /// ```
    /// use teistro_core::catalogue::Rashi;
    ///
    /// assert_eq!(Rashi::Aries.opposite(), Rashi::Libra);
    /// assert_eq!(Rashi::Pisces.opposite(), Rashi::Virgo);
    /// ```
    #[must_use]
    pub fn opposite(self) -> Rashi {
        let index = Rashi::ALL
            .iter()
            .position(|sign| *sign == self)
            .unwrap_or_default();
        Rashi::ALL
            .get((index + 6) % 12)
            .copied()
            .unwrap_or(Rashi::Aries)
    }
}

#[cfg(test)]
mod tests {
    use super::Rashi;

    #[test]
    fn every_sign_holds_its_thirty_degrees_and_no_more() {
        for (index, sign) in Rashi::ALL.iter().enumerate() {
            let start = 30.0 * f64::from(u8::try_from(index).unwrap_or_default());
            assert_eq!(Rashi::of_longitude(start), *sign, "{sign:?} at its start");
            assert_eq!(
                Rashi::of_longitude(start + 29.999_999),
                *sign,
                "{sign:?} at its end"
            );
            assert_eq!(
                Rashi::of_longitude(start - 360.0),
                *sign,
                "{sign:?} a turn back"
            );
        }
        assert_eq!(Rashi::of_longitude(360.0), Rashi::Aries);
    }

    #[test]
    fn opposite_is_six_signs_on_and_its_own_inverse() {
        for sign in Rashi::ALL {
            assert_eq!(sign.opposite().opposite(), sign, "{sign:?}");
            assert_ne!(sign.opposite(), sign, "{sign:?}");
        }
        assert_eq!(Rashi::Cancer.opposite(), Rashi::Capricorn);
    }
}

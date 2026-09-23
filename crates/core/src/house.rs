//! A **bhava**, one of the twelve houses, as a number a consumer names.
//!
//! It is a newtype and not a bare `u8` because every consumer of it —
//! the rule language, the interpretation composers, the annual chart's
//! yogas — has to refuse a thirteenth house somewhere, and refusing it
//! once at the constructor is the only way the refusal reads the same in
//! all of them.
//!
//! It lives in `core` rather than beside its first consumer because four
//! crates need it and none of them may depend on the others: the rule
//! language reads one out of a pack, `teistro_interpret` names one in a
//! placement, and `teistro_tajika` takes one as the **matter** a Tajika
//! yoga is asked about. A second copy of a twelve-valued primitive is a
//! second place to get the counting wrong.

use serde::{Deserialize, Serialize};

use crate::catalogue::Rashi;
use crate::error::Error;

/// A house, 1 to 12.
///
/// ```
/// use teistro_core::house::House;
///
/// let seventh = House::try_new(7)?;
/// assert_eq!(seventh.get(), 7);
/// assert!(House::try_new(13).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(try_from = "u8", into = "u8")]
pub struct House(u8);

impl House {
    /// A house.
    ///
    /// # Errors
    ///
    /// A number outside 1 to 12, named, with the field `house`.
    pub fn try_new(house: u8) -> Result<House, Error> {
        if (1..=12).contains(&house) {
            Ok(House(house))
        } else {
            Err(Error::invalid_arg(format!("house {house} is not 1 to 12"))
                .with_field(String::from("house")))
        }
    }

    /// Its number, 1 to 12.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// The house `sign` is counted from `from`, whole signs, 1 to 12.
    ///
    /// ```
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_core::house::House;
    ///
    /// // A sign is always the first house from itself.
    /// assert_eq!(House::between(Rashi::Aries, Rashi::Aries).get(), 1);
    /// assert_eq!(House::between(Rashi::Aries, Rashi::Libra).get(), 7);
    /// ```
    #[must_use]
    pub const fn between(from: Rashi, sign: Rashi) -> House {
        House((sign as u8 + 12 - from as u8) % 12 + 1)
    }

    /// The sign this house falls in, counted from `from` by whole signs.
    ///
    /// The inverse of [`House::between`], which every whole-sign reader
    /// needs and which each of them used to write out as a `% 12`.
    #[must_use]
    pub fn sign_from(self, from: Rashi) -> Rashi {
        let index = (from.id() + u16::from(self.0) - 1) % 12;
        // The twelve signs are a closed set and `index` is taken modulo
        // twelve, so the fallback is unreachable; it is the lagna rather
        // than a panic because a sign this arithmetic cannot name would
        // be a defect in the arithmetic and not in the caller.
        Rashi::from_id(index).unwrap_or(from)
    }

    /// The four kendras.
    pub const KENDRAS: [House; 4] = [House(1), House(4), House(7), House(10)];
    /// The three trikonas.
    pub const TRIKONAS: [House; 3] = [House(1), House(5), House(9)];
    /// The two maraka houses, the second and the seventh (BPHS ch. 44 v. 2).
    pub const MARAKAS: [House; 2] = [House(2), House(7)];
    /// The three **trika** houses, the sixth, eighth and twelfth, which
    /// the Tajika yogas call a planet weak for standing in.
    pub const TRIKA: [House; 3] = [House(6), House(8), House(12)];
    /// The four **panaphara**, the succedent houses.
    pub const PANAPHARA: [House; 4] = [House(2), House(5), House(8), House(11)];
    /// The four **apoklima**, the cadent houses.
    pub const APOKLIMA: [House; 4] = [House(3), House(6), House(9), House(12)];

    /// Whether it is one of the four kendras.
    #[must_use]
    pub fn is_kendra(self) -> bool {
        House::KENDRAS.contains(&self)
    }

    /// Whether it is one of the four panapharas.
    #[must_use]
    pub fn is_panaphara(self) -> bool {
        House::PANAPHARA.contains(&self)
    }

    /// Whether it is one of the four apoklimas.
    #[must_use]
    pub fn is_apoklima(self) -> bool {
        House::APOKLIMA.contains(&self)
    }

    /// Whether it is one of the three trika houses.
    #[must_use]
    pub fn is_trika(self) -> bool {
        House::TRIKA.contains(&self)
    }
}

impl TryFrom<u8> for House {
    type Error = Error;

    fn try_from(house: u8) -> Result<House, Error> {
        House::try_new(house)
    }
}

impl From<House> for u8 {
    fn from(house: House) -> u8 {
        house.0
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use super::House;
    use crate::catalogue::Rashi;

    /// Every house is 1 to 12 and nothing else is a house.
    #[test]
    fn only_the_twelve_are_houses() {
        for number in 1..=12u8 {
            assert_eq!(House::try_new(number).unwrap().get(), number);
        }
        for number in [0u8, 13, 255] {
            let refused = House::try_new(number).unwrap_err();
            assert!(refused.to_string().contains("1 to 12"), "{refused}");
        }
    }

    /// `between` and `sign_from` are each other's inverse, over every
    /// lagna and every sign: the counting is written once and checked
    /// both ways, because an off-by-one here is silent.
    #[test]
    fn counting_a_house_and_naming_its_sign_are_inverses() {
        for lagna in Rashi::ALL {
            for sign in Rashi::ALL {
                let house = House::between(lagna, sign);
                assert_eq!(house.sign_from(lagna), sign, "{lagna:?} to {sign:?}");
            }
            for number in 1..=12u8 {
                let house = House::try_new(number).unwrap();
                assert_eq!(House::between(lagna, house.sign_from(lagna)), house);
            }
        }
    }

    /// The four classes partition the twelve, each house in exactly one.
    #[test]
    fn kendra_panaphara_and_apoklima_partition_the_twelve() {
        for number in 1..=12u8 {
            let house = House::try_new(number).unwrap();
            let classes = u8::from(house.is_kendra())
                + u8::from(house.is_panaphara())
                + u8::from(house.is_apoklima());
            assert_eq!(classes, 1, "house {number} is in exactly one class");
        }
        assert!(House::try_new(6).unwrap().is_trika());
        assert!(!House::try_new(7).unwrap().is_trika());
    }
}

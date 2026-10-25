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

use crate::angle::difference_deg;
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
    /// All twelve, first to twelfth.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert_eq!(House::ALL.map(House::get), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    /// ```
    pub const ALL: [House; 12] = [
        House(1),
        House(2),
        House(3),
        House(4),
        House(5),
        House(6),
        House(7),
        House(8),
        House(9),
        House(10),
        House(11),
        House(12),
    ];

    /// A house.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert_eq!(House::try_new(12)?, House::VYAYA);
    /// let refused = House::try_new(0).unwrap_err();
    /// assert_eq!((refused.message.as_str(), refused.field()), ("house 0 is not 1 to 12", Some("house")));
    /// // JSON reads through the same check.
    /// assert!(serde_json::from_str::<House>("13").is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert_eq!(House::try_new(10)?.get(), 10);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_core::house::House;
    ///
    /// // The seventh from a Capricorn lagna is Cancer.
    /// assert_eq!(House::try_new(7)?.sign_from(Rashi::Capricorn), Rashi::Cancer);
    /// assert_eq!(House::between(Rashi::Capricorn, Rashi::Cancer).get(), 7);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn sign_from(self, from: Rashi) -> Rashi {
        let index = (from.id() + u16::from(self.0) - 1) % 12;
        // The twelve signs are a closed set and `index` is taken modulo
        // twelve, so the fallback is unreachable; it is the lagna rather
        // than a panic because a sign this arithmetic cannot name would
        // be a defect in the arithmetic and not in the caller.
        Rashi::from_id(index).unwrap_or(from)
    }

    /// The twelfth, *vyaya*, the house of loss and of liberation.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert_eq!(House::VYAYA.get(), 12);
    /// ```
    pub const VYAYA: House = House(12);
    /// The four kendras.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert_eq!(House::KENDRAS.map(House::get), [1, 4, 7, 10]);
    /// ```
    pub const KENDRAS: [House; 4] = [House(1), House(4), House(7), House(10)];
    /// The three trikonas.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert_eq!(House::TRIKONAS.map(House::get), [1, 5, 9]);
    /// ```
    pub const TRIKONAS: [House; 3] = [House(1), House(5), House(9)];
    /// The two maraka houses, the second and the seventh (BPHS ch. 44 v. 2).
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert_eq!(House::MARAKAS.map(House::get), [2, 7]);
    /// ```
    pub const MARAKAS: [House; 2] = [House(2), House(7)];
    /// The three **trika** houses, the sixth, eighth and twelfth, which
    /// the Tajika yogas call a planet weak for standing in.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert_eq!(House::TRIKA.map(House::get), [6, 8, 12]);
    /// ```
    pub const TRIKA: [House; 3] = [House(6), House(8), House(12)];
    /// The four **panaphara**, the succedent houses.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert_eq!(House::PANAPHARA.map(House::get), [2, 5, 8, 11]);
    /// ```
    pub const PANAPHARA: [House; 4] = [House(2), House(5), House(8), House(11)];
    /// The four **apoklima**, the cadent houses.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert_eq!(House::APOKLIMA.map(House::get), [3, 6, 9, 12]);
    /// ```
    pub const APOKLIMA: [House; 4] = [House(3), House(6), House(9), House(12)];

    /// Whether it is one of the four kendras.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert!(House::try_new(4)?.is_kendra());
    /// assert!(!House::try_new(5)?.is_kendra());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn is_kendra(self) -> bool {
        House::KENDRAS.contains(&self)
    }

    /// Whether it is one of the four panapharas.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert!(House::try_new(11)?.is_panaphara());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn is_panaphara(self) -> bool {
        House::PANAPHARA.contains(&self)
    }

    /// Whether it is one of the four apoklimas.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// // Kendra, panaphara and apoklima partition the twelve.
    /// assert_eq!(House::ALL.iter().filter(|h| h.is_kendra() || h.is_panaphara() || h.is_apoklima()).count(), 12);
    /// assert!(House::try_new(9)?.is_apoklima());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn is_apoklima(self) -> bool {
        House::APOKLIMA.contains(&self)
    }

    /// Whether it is one of the three trika houses.
    ///
    /// ```
    /// use teistro_core::house::House;
    ///
    /// assert!(House::VYAYA.is_trika());
    /// assert!(!House::try_new(7)?.is_trika());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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

/// The house a longitude is counted in: a planet within `orb_deg` of a
/// cusp, before or after it, is in the house of the nearest such cusp, "his
/// vertue shall be assigned to that house to whose Cusp he is neerest"
/// (Lilly, p. 33); otherwise the house whose cusp it last passed. An orb
/// of zero counts by the cusps alone, as Leo does.
///
/// ```
/// use teistro_core::house::{House, house_of};
///
/// // Twelve houses of 30° from 0° Aries.
/// let cusps = std::array::from_fn(|k| k as f64 * 30.0);
/// assert_eq!(house_of(27.0, &cusps, 5.0), House::try_new(2)?);
/// assert_eq!(house_of(27.0, &cusps, 0.0), House::try_new(1)?);
/// assert_eq!(house_of(357.0, &cusps, 5.0), House::try_new(1)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn house_of(longitude_deg: f64, cusps_deg: &[f64; 12], orb_deg: f64) -> House {
    let past = |cusp: f64| (longitude_deg - cusp).rem_euclid(360.0);
    let near = |cusp: f64| difference_deg(longitude_deg, cusp).abs();
    let by_orb = cusps_deg
        .iter()
        .enumerate()
        .filter(|&(_, &cusp)| near(cusp) <= orb_deg)
        .min_by(|(_, a), (_, b)| near(**a).total_cmp(&near(**b)));
    let (index, _) = by_orb
        .or_else(|| {
            cusps_deg
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| past(**a).total_cmp(&past(**b)))
        })
        .unwrap_or((0, &0.0));
    House::ALL.get(index).copied().unwrap_or(House::ALL[0])
}

/// Sripati's mid-points, as the sources build them: the lagna and the
/// midheaven and their opposites mid-point the kendras, and the quadrant
/// from the midheaven to the lagna, and from the lagna to the nadir, is
/// each cut in three, the houses opposite following by six signs.
///
/// Shared because more than one module names Sripati's division as its
/// source's own, whatever chalit a profile gives the chart: the Tajika
/// sahams read a house's mid-point, and rectification's conception reads
/// the 9th's (`03-design/rectification.md`, NISHEKA).
///
/// ```
/// use teistro_core::house::sripati_mid_points;
///
/// let points = sripati_mid_points(0.0, 270.0);
/// assert_eq!(points[0], 0.0);
/// assert_eq!(points[9], 270.0);
/// assert_eq!(points[3], 90.0);
/// ```
#[must_use]
pub fn sripati_mid_points(lagna_deg: f64, midheaven_deg: f64) -> [f64; 12] {
    let lagna = lagna_deg.rem_euclid(360.0);
    let midheaven = midheaven_deg.rem_euclid(360.0);
    let nadir = (midheaven + 180.0).rem_euclid(360.0);
    let upper = (lagna - midheaven).rem_euclid(360.0) / 3.0;
    let lower = (nadir - lagna).rem_euclid(360.0) / 3.0;
    let at = |deg: f64| deg.rem_euclid(360.0);
    let (second, third) = (at(lagna + lower), at(lagna + 2.0 * lower));
    let (eleventh, twelfth) = (at(midheaven + upper), at(midheaven + 2.0 * upper));
    let opposite = |deg: f64| at(deg + 180.0);
    [
        lagna,
        second,
        third,
        nadir,
        opposite(eleventh),
        opposite(twelfth),
        opposite(lagna),
        opposite(second),
        opposite(third),
        midheaven,
        eleventh,
        twelfth,
    ]
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use super::{House, house_of, sripati_mid_points};
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
        assert_eq!(House::try_from(9_u8).unwrap(), House::try_new(9).unwrap());
        assert!(House::try_from(0_u8).is_err());
        assert_eq!(u8::from(House::try_new(4).unwrap()), 4);
    }

    /// A planet near a cusp joins that cusp's house, from either side and
    /// across 0°; outside every orb it is in the house whose cusp it last
    /// passed, over unequal houses too.
    #[test]
    fn a_planet_takes_the_nearest_cusp_within_its_orb() {
        let house = |n| House::try_new(n).unwrap();
        let equal: [f64; 12] = std::array::from_fn(|k| f64::from(u8::try_from(k).unwrap()) * 30.0);
        assert_eq!(house_of(27.0, &equal, 5.0), house(2));
        assert_eq!(house_of(27.0, &equal, 0.0), house(1));
        assert_eq!(house_of(357.0, &equal, 5.0), house(1));
        assert_eq!(house_of(357.0, &equal, 0.0), house(12));
        assert_eq!(house_of(-3.0, &equal, 0.0), house(12));
        assert_eq!(house_of(30.0, &equal, 0.0), house(2));
        let unequal = [
            100.0, 125.0, 155.0, 190.0, 225.0, 255.0, 280.0, 305.0, 335.0, 10.0, 45.0, 75.0,
        ];
        assert_eq!(house_of(5.0, &unequal, 0.0), house(9));
        assert_eq!(house_of(5.0, &unequal, 6.0), house(10));
        assert_eq!(house_of(99.0, &unequal, 2.0), house(1));
        assert_eq!(house_of(99.0, &unequal, 0.0), house(12));
    }

    /// The mid-points keep the angles, split each quadrant in three and
    /// set every house opposite its seventh.
    #[test]
    fn sripati_mid_points_split_the_quadrants_from_the_angles() {
        let points = sripati_mid_points(400.0, -90.0);
        assert_eq!(points[0], 40.0);
        assert_eq!(points[9], 270.0);
        assert_eq!(points[3], 90.0);
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(close(points[1], 56.666_666_666_666_66), "{points:?}");
        assert!(close(points[2], 73.333_333_333_333_33), "{points:?}");
        assert!(close(points[10], 313.333_333_333_333_3), "{points:?}");
        for (one, other) in points.iter().zip(points.iter().skip(6)) {
            assert!(close((one - other).rem_euclid(360.0), 180.0), "{points:?}");
        }
    }
}

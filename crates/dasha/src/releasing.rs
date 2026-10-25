//! Valens's time lords as kernels behind [`Timeline`]: releasing from a
//! lot, and the profected year (`03-design/hellenistic-time-lords.md`).
//!
//! **Releasing** (*Anthologies* IV.4, IV.6–IV.10). A sign allots the
//! minimum years of its lord, Capricorn 27 and Aquarius 30. Each level is a
//! twelfth of the one above. A level begins at its parent's own sign, runs
//! forward, and is cut at the parent's end. Once the twelve signs are
//! spent (211 units), the rest is loosed to the sign opposite the parent's.
//!
//! **Peaks and the loosing** (IV.4, IV.7; II.17K). A period's sign is
//! judged by its place from the Lot of Fortune, whose angles stand as the
//! Ascendant's do: [`releasing_angle`] says which angle a sign is, and
//! nothing for a sign at no angle. [`ReleasingDasha::loosing`] is the
//! period at which a level jumps to the opposite sign, and
//! [`ReleasingDasha::is_loosed`] says a period runs after the jump. Both
//! read a period the kernel already made; neither moves one.
//!
//! **Profection** (IV.11). The ordinal year *n* falls *n − 1* signs on
//! from the start, one sign a year, and the count never ends.
//!
//! ```
//! use teistro_core::catalogue::Rashi;
//! use teistro_core::quantity::{Depth, JulianDay};
//! use teistro_core::settings::YearLength;
//! use teistro_dasha::{ReleasingDasha, Timeline};
//!
//! // IV.8: the Lot of Fortune in Leo. Leo 19 years, Virgo 20, Libra 8 and
//! // Scorpio 15 make 62, so the 70th year is Sagittarius's.
//! let birth = JulianDay::literal(2_451_545.0);
//! let dasha = ReleasingDasha::new(Rashi::Leo, birth, YearLength::Savana360);
//! let seventieth = JulianDay::literal(2_451_545.0 + 69.5 * 360.0);
//! let chain = dasha.at(seventieth, Depth::try_new(1)?);
//! assert_eq!(chain.iter().next().and_then(|period| period.sign), Some(Rashi::Sagittarius));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use teistro_core::catalogue::{DashaFamily, DashaSystem, Graha, Rashi};
use teistro_core::house::House;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::YearLength;

use crate::rashi::{Direction, step};
use crate::reading::DashaCursor;
use crate::tree::{Path, Period, Timeline};

/// The signs of a whole cycle.
const SIGNS: usize = 12;

/// The units a cycle of the twelve signs allots: 17 years 7 months at the
/// second level (IV.4), 528 days at the third and 44 at the fourth (IV.10).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_dasha::{RELEASING_CYCLE, releasing_years};
///
/// // The twelve signs' years: 211, 17 years 7 months of months.
/// let years: u32 = Rashi::ALL.iter().map(|sign| releasing_years(*sign)).sum();
/// assert_eq!(years, RELEASING_CYCLE);
/// ```
pub const RELEASING_CYCLE: u32 = 211;

/// The most children a releasing period can have: Aquarius's 360 months
/// hold the twelve, then nine more after the loosing.
const RELEASING_BREADTH: usize = 24;

/// The years a sign allots in releasing: its lord's minimum years (IV.1),
/// except Capricorn's 27 (IV.6). IV.6 gives Aquarius 30 as well, which are
/// its lord Saturn's own.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_dasha::releasing_years;
///
/// // Leo the Sun's 19; Capricorn 27, not Saturn's 30 (IV.6); Aquarius Saturn's 30.
/// assert_eq!([Rashi::Leo, Rashi::Capricorn, Rashi::Aquarius].map(releasing_years), [19, 27, 30]);
/// ```
#[must_use]
pub fn releasing_years(sign: Rashi) -> u32 {
    match sign {
        Rashi::Capricorn => 27,
        _ => minimum_years(sign.attributes().lord),
    }
}

/// A planet's minimum years (*Anthologies* IV.1): Saturn 30, Jupiter 12,
/// Mars 15, Venus 8, Mercury 20, the Sun 19 and the Moon 25, 129 in all;
/// a node has none. Releasing gives them to signs, and the decennials
/// share their 129 months by them.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_dasha::minimum_years;
///
/// let seven = [Graha::Saturn, Graha::Jupiter, Graha::Mars, Graha::Venus, Graha::Mercury, Graha::Sun, Graha::Moon];
/// assert_eq!(seven.map(minimum_years).iter().sum::<u32>(), 129);
/// assert_eq!(minimum_years(Graha::Rahu), 0);
/// ```
#[must_use]
pub const fn minimum_years(planet: Graha) -> u32 {
    match planet {
        Graha::Saturn => 30,
        Graha::Jupiter => 12,
        Graha::Mars => 15,
        Graha::Venus => 8,
        Graha::Mercury => 20,
        Graha::Sun => 19,
        Graha::Moon => 25,
        _ => 0,
    }
}

/// A sign's lord, which a period of the sign names.
fn lord(sign: Rashi) -> Graha {
    sign.attributes().lord
}

/// An angle of the circle counted from a lot by whole signs: the lot's own
/// sign stands as the Ascendant, the fourth from it as the IC, the seventh
/// as the Descendant and the tenth as the MC (*Anthologies* II.17K, 79K).
///
/// Valens judges a releasing period by where its sign falls from the Lot
/// of Fortune: a distribution to "MC relative to the Lot of Fortune" or to
/// "the Lot itself" brings rank, and one at the Ascendant's angles brings
/// it "not to the same extent" (IV.7, 166K). Later practice calls such
/// periods the peaks. The angle is reported, not weighed (cruxes C423,
/// C425).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_dasha::{LotAngle, releasing_angle};
///
/// // Fortune in Leo: Taurus is the tenth from it, Fortune's own MC.
/// assert_eq!(releasing_angle(Rashi::Leo, Rashi::Taurus), Some(LotAngle::Tenth));
/// assert_eq!(LotAngle::Tenth.house().get(), 10);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LotAngle {
    /// The lot's own sign, its Ascendant: "Life".
    First,
    /// The fourth sign from the lot, its IC.
    Fourth,
    /// The seventh sign from the lot, its Descendant.
    Seventh,
    /// The tenth sign from the lot, its MC: "Rank".
    Tenth,
}

impl LotAngle {
    /// The four, in the order of the houses.
    ///
    /// ```
    /// use teistro_dasha::LotAngle;
    ///
    /// assert_eq!(LotAngle::ALL.map(|angle| angle.house().get()), [1, 4, 7, 10]);
    /// ```
    pub const ALL: [LotAngle; 4] = [
        LotAngle::First,
        LotAngle::Fourth,
        LotAngle::Seventh,
        LotAngle::Tenth,
    ];

    /// The angle a house counted from the lot is, or nothing for a house
    /// at no angle: a succedent one, or one "preceding an angle".
    ///
    /// ```
    /// use teistro_core::house::House;
    /// use teistro_dasha::LotAngle;
    ///
    /// assert_eq!(LotAngle::of(House::try_new(7)?), Some(LotAngle::Seventh));
    /// assert_eq!(LotAngle::of(House::try_new(6)?), None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn of(house: House) -> Option<LotAngle> {
        match house.get() {
            1 => Some(LotAngle::First),
            4 => Some(LotAngle::Fourth),
            7 => Some(LotAngle::Seventh),
            10 => Some(LotAngle::Tenth),
            _ => None,
        }
    }

    /// The house it is from the lot, the lot's own sign being the first.
    ///
    /// ```
    /// use teistro_dasha::LotAngle;
    ///
    /// assert_eq!(LotAngle::Fourth.house().get(), 4);
    /// ```
    #[must_use]
    pub const fn house(self) -> House {
        let [first, fourth, seventh, tenth] = House::KENDRAS;
        match self {
            LotAngle::First => first,
            LotAngle::Fourth => fourth,
            LotAngle::Seventh => seventh,
            LotAngle::Tenth => tenth,
        }
    }
}

/// The angle `sign` is from `lot` by whole signs, or nothing when it is at
/// none: the reading a releasing period's sign is judged by.
///
/// `lot` is the Lot of Fortune's sign, also when the releasing starts from
/// Daimon, since Valens counts Daimon's distribution to "MC relative to the
/// Lot of Fortune" (IV.7, 166K; crux C424). Any other sign may be passed:
/// the Ascendant's gives the angles Valens ranks below Fortune's, and a
/// parent period's sign gives a reading he does not make (C426). The whole
/// place, angle or not, is `House::between(lot, sign)`.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_core::house::House;
/// use teistro_dasha::{LotAngle, releasing_angle};
///
/// // IV.8: Fortune in Leo, Daimon in Scorpio. Daimon's own sign is the
/// // fourth from Fortune, and Leo is Fortune itself.
/// assert_eq!(releasing_angle(Rashi::Leo, Rashi::Scorpio), Some(LotAngle::Fourth));
/// assert_eq!(releasing_angle(Rashi::Leo, Rashi::Leo), Some(LotAngle::First));
/// // With the Ascendant in Pisces, Fortune in Leo is the sixth sign,
/// // "preceding an angle <the Descendant>" (168K).
/// assert_eq!(releasing_angle(Rashi::Pisces, Rashi::Leo), None);
/// assert!(House::between(Rashi::Pisces, Rashi::Leo).is_apoklima());
/// ```
#[must_use]
pub const fn releasing_angle(lot: Rashi, sign: Rashi) -> Option<LotAngle> {
    LotAngle::of(House::between(lot, sign))
}

/// Valens's releasing from a sign: the vital sector begun from a lot.
///
/// ```
/// # use teistro_core::catalogue::{Graha, Rashi};
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::YearLength;
/// # use teistro_dasha::{ReleasingDasha, Timeline};
/// #
/// # // Released from Leo at birth, in Valens's 360-day years.
/// # let birth = JulianDay::literal(2_451_545.0);
/// # let leo = ReleasingDasha::new(Rashi::Leo, birth, YearLength::Savana360);
/// // Leo's 19 years, then Virgo's 20: one sign after another.
/// let first = leo.mahadasha(0, 0).ok_or(teistro_core::Error::internal("a period"))?;
/// assert_eq!((first.sign, first.lord), (Some(Rashi::Leo), Graha::Sun));
/// assert!((first.interval.days() - 19.0 * 360.0).abs() < 1e-9);
/// assert_eq!(leo.mahadasha(0, 1).and_then(|p| p.sign), Some(Rashi::Virgo));
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ReleasingDasha {
    start: Rashi,
    birth: JulianDay<Utc>,
    year_days: f64,
}

impl ReleasingDasha {
    /// Releasing from `start` at `birth`, a period year being `year_length`
    /// (Valens's is 360 days, IV.9).
    ///
    /// ```
    /// # use teistro_core::catalogue::{Graha, Rashi};
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::YearLength;
    /// # use teistro_dasha::{ReleasingDasha, Timeline};
    /// #
    /// # // Released from Leo at birth, in Valens's 360-day years.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let leo = ReleasingDasha::new(Rashi::Leo, birth, YearLength::Savana360);
    /// // The first sub-period is the sign's own, its years as months.
    /// let first = leo.mahadasha(0, 0).ok_or(teistro_core::Error::internal("a period"))?;
    /// let sub = leo.child(&first, 0).ok_or(teistro_core::Error::internal("a sub-period"))?;
    /// assert_eq!(sub.sign, Some(Rashi::Leo));
    /// assert!((sub.interval.days() - 19.0 * 30.0).abs() < 1e-9);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub const fn new(
        start: Rashi,
        birth: JulianDay<Utc>,
        year_length: YearLength,
    ) -> ReleasingDasha {
        ReleasingDasha {
            start,
            birth,
            year_days: year_length.days(),
        }
    }

    /// The sign it is released from.
    ///
    /// ```
    /// # use teistro_core::catalogue::{Graha, Rashi};
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::YearLength;
    /// # use teistro_dasha::{ReleasingDasha, Timeline};
    /// #
    /// # // Released from Leo at birth, in Valens's 360-day years.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let leo = ReleasingDasha::new(Rashi::Leo, birth, YearLength::Savana360);
    /// assert_eq!(leo.start(), Rashi::Leo);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub const fn start(&self) -> Rashi {
        self.start
    }

    /// The days one unit lasts at a level, 1 for the years: a twelfth of
    /// the level above, divided by a whole power of twelve so that no libm
    /// stands between a level and Valens's arithmetic.
    fn unit_days(&self, level: usize) -> f64 {
        let twelfths = u32::try_from(level.saturating_sub(1)).unwrap_or(u32::MAX);
        let parts = 12_u32
            .checked_pow(twelfths)
            .map_or(f64::INFINITY, f64::from);
        self.year_days / parts
    }

    /// The days of one first-level cycle of the twelve signs.
    fn cycle_days(&self) -> f64 {
        f64::from(RELEASING_CYCLE) * self.year_days
    }

    /// The `index`th sign of a level begun from `from`: the twelve in
    /// order, then the rest loosed to the sign opposite `from` (IV.4).
    fn sign_at(from: Rashi, index: usize) -> Rashi {
        if index < SIGNS {
            step(from, Direction::Forward, index)
        } else {
            // `step` counts round the twelve, so the opposite sign's
            // count is the index itself.
            step(from.opposite(), Direction::Forward, index)
        }
    }

    /// The units allotted before the `index`th sign of a level begun from
    /// `from`, counted whole so that no boundary accumulates rounding.
    fn units_before(from: Rashi, index: usize) -> u32 {
        (0..index)
            .map(|k| releasing_years(Self::sign_at(from, k)))
            .sum()
    }

    /// The period below `parent` at which the bond is loosed: the first
    /// after its twelve signs are spent, in the sign opposite the parent's
    /// (IV.4, IV.10); nothing when the parent ends first, as a sign of
    /// fewer than 211 units does, or below the deepest level (crux C427).
    ///
    /// ```
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_core::settings::YearLength;
    /// use teistro_dasha::{ReleasingDasha, Timeline};
    ///
    /// // IV.4: Gemini's 20 years spend the twelve in 17 years 7 months,
    /// // and the rest goes to Sagittarius, opposite.
    /// let birth = JulianDay::literal(2_451_545.0);
    /// let dasha = ReleasingDasha::new(Rashi::Gemini, birth, YearLength::Savana360);
    /// let gemini = dasha.mahadashas().next().ok_or("no first period")?;
    /// let loosed = dasha.loosing(&gemini).ok_or("no loosing")?;
    /// assert_eq!(loosed.sign, Some(Rashi::Sagittarius));
    /// assert!(ReleasingDasha::is_loosed(&loosed));
    /// // Aries's 15 years end before the twelve are spent.
    /// let dasha = ReleasingDasha::new(Rashi::Aries, birth, YearLength::Savana360);
    /// let aries = dasha.mahadashas().next().ok_or("no first period")?;
    /// assert!(dasha.loosing(&aries).is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn loosing(&self, parent: &Period) -> Option<Period> {
        self.child(parent, SIGNS)
    }

    /// Whether `period` runs after its level's bond was loosed: its place
    /// among its parent's children is past the twelve, so its sign is
    /// counted from the one opposite the parent's. The first such period
    /// is [`ReleasingDasha::loosing`]; a first-level period never is one.
    ///
    /// It reads the period's [`Path`] alone, so a stored row answers the
    /// same: a releasing row is loosed when the last place of its path is
    /// 12 or more.
    ///
    /// ```
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_core::settings::YearLength;
    /// use teistro_dasha::{ReleasingDasha, Timeline};
    ///
    /// let birth = JulianDay::literal(2_451_545.0);
    /// let dasha = ReleasingDasha::new(Rashi::Gemini, birth, YearLength::Savana360);
    /// let gemini = dasha.mahadashas().next().ok_or("no first period")?;
    /// let loosed: Vec<Rashi> = dasha
    ///     .children(&gemini)
    ///     .filter(ReleasingDasha::is_loosed)
    ///     .filter_map(|period| period.sign)
    ///     .collect();
    /// assert_eq!(loosed, [Rashi::Sagittarius, Rashi::Capricorn]);
    /// assert!(!ReleasingDasha::is_loosed(&gemini));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_loosed(period: &Period) -> bool {
        period.path.depth() > 1
            && period
                .path
                .indices()
                .last()
                .is_some_and(|&place| usize::from(place) >= SIGNS)
    }
}

impl Timeline for ReleasingDasha {
    fn breadth(&self) -> usize {
        RELEASING_BREADTH
    }

    /// The twelve signs from the start, each its years whole; past the
    /// first cycle, nothing (211 years, which no life reaches).
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        if cycle > 0 || index >= SIGNS {
            return None;
        }
        let sign = step(self.start, Direction::Forward, index);
        let days = |units: u32| self.birth.get() + f64::from(units) * self.year_days;
        let before = Self::units_before(self.start, index);
        Some(Period::of_sign(
            sign,
            lord(sign),
            Path::root(0, u8::try_from(index).ok()?),
            Interval::literal(days(before), days(before + releasing_years(sign))),
        ))
    }

    fn mahadasha_at(&self, instant: f64) -> Option<Period> {
        let into = instant - self.birth.get();
        if into < 0.0 || into >= self.cycle_days() {
            return None;
        }
        (0..SIGNS)
            .filter_map(|index| self.mahadasha(0, index))
            .find(|period| instant < period.interval.to.get())
    }

    /// The `index`th period below `parent`, begun at the parent's sign,
    /// loosed after twelve, cut at the parent's end; nothing once the
    /// parent is spent or below the deepest level.
    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        let from_sign = parent.sign?;
        // Below the deepest level the parent's path has no child.
        if index >= RELEASING_BREADTH {
            return None;
        }
        let unit = self.unit_days(parent.path.depth() + 1);
        let whole = parent.interval;
        let from = whole.from.get() + f64::from(Self::units_before(from_sign, index)) * unit;
        if from >= whole.to.get() {
            return None;
        }
        let sign = Self::sign_at(from_sign, index);
        let to = (from + f64::from(releasing_years(sign)) * unit).min(whole.to.get());
        Some(Period::of_sign(
            sign,
            lord(sign),
            parent.path.child(u8::try_from(index).ok()?)?,
            Interval::literal(from, to),
        ))
    }
}

/// The years one cycle of the profected year holds: ten circuits of the
/// twelve signs, the life Vimshottari reckons, so that a stored reading
/// answers the year at any age a life reaches. Valens's count itself never
/// ends, and the cursor carries on past it.
///
/// ```
/// # use teistro_core::catalogue::Rashi;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::YearLength;
/// # use teistro_dasha::{ProfectionDasha, Timeline};
/// #
/// # // Profected from Virgo, as Valens's IV.11.
/// # let birth = JulianDay::literal(2_451_545.0);
/// # let virgo = ProfectionDasha::new(Rashi::Virgo, birth, YearLength::Julian36525);
/// use teistro_dasha::PROFECTION_YEARS;
///
/// // Ten circuits, then the count carries on.
/// assert!(virgo.mahadasha(0, usize::from(PROFECTION_YEARS)).is_none());
/// assert_eq!(virgo.mahadasha(1, 0).and_then(|p| p.sign), Some(Rashi::Virgo));
/// ```
pub const PROFECTION_YEARS: u8 = 120;

/// The profected year: one sign a year from a start, without end (IV.11).
///
/// ```
/// # use teistro_core::catalogue::Rashi;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::YearLength;
/// # use teistro_dasha::{ProfectionDasha, Timeline};
/// #
/// # // Profected from Virgo, as Valens's IV.11.
/// # let birth = JulianDay::literal(2_451_545.0);
/// # let virgo = ProfectionDasha::new(Rashi::Virgo, birth, YearLength::Julian36525);
/// // A sign a year, the year's index the completed age; no division below.
/// let fifth = virgo.mahadasha(0, 4).ok_or("a year")?;
/// assert_eq!(fifth.sign, Some(Rashi::Capricorn));
/// assert_eq!(virgo.child(&fifth, 0), None);
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ProfectionDasha {
    start: Rashi,
    birth: JulianDay<Utc>,
    year_days: f64,
}

impl ProfectionDasha {
    /// The years profected from `start` at `birth`, a year being
    /// `year_length` (Valens's is 365¼ days, IV.9).
    ///
    /// ```
    /// # use teistro_core::catalogue::Rashi;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::YearLength;
    /// # use teistro_dasha::{ProfectionDasha, Timeline};
    /// #
    /// # // Profected from Virgo, as Valens's IV.11.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let virgo = ProfectionDasha::new(Rashi::Virgo, birth, YearLength::Julian36525);
    /// // The year at birth is the start's.
    /// assert_eq!(virgo.mahadasha_at(birth.get()).and_then(|p| p.sign), Some(Rashi::Virgo));
    /// ```
    #[must_use]
    pub const fn new(
        start: Rashi,
        birth: JulianDay<Utc>,
        year_length: YearLength,
    ) -> ProfectionDasha {
        ProfectionDasha {
            start,
            birth,
            year_days: year_length.days(),
        }
    }

    /// The sign the count starts from.
    ///
    /// ```
    /// # use teistro_core::catalogue::Rashi;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::YearLength;
    /// # use teistro_dasha::{ProfectionDasha, Timeline};
    /// #
    /// # // Profected from Virgo, as Valens's IV.11.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let virgo = ProfectionDasha::new(Rashi::Virgo, birth, YearLength::Julian36525);
    /// assert_eq!(virgo.start(), Rashi::Virgo);
    /// ```
    #[must_use]
    pub const fn start(&self) -> Rashi {
        self.start
    }

    /// The sign of the ordinal year `year`, 1 for the year that begins at
    /// birth: `year − 1` signs on (IV.11's 35th year, from Virgo, is
    /// Cancer); nothing for year 0, which does not exist.
    ///
    /// ```
    /// # use teistro_core::catalogue::Rashi;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::YearLength;
    /// # use teistro_dasha::{ProfectionDasha, Timeline};
    /// #
    /// # // Profected from Virgo, as Valens's IV.11.
    /// # let birth = JulianDay::literal(2_451_545.0);
    /// # let virgo = ProfectionDasha::new(Rashi::Virgo, birth, YearLength::Julian36525);
    /// // IV.11: the 35th year from Virgo is Cancer.
    /// assert_eq!(virgo.sign_of_year(35), Some(Rashi::Cancer));
    /// assert_eq!(virgo.sign_of_year(0), None);
    /// ```
    #[must_use]
    pub fn sign_of_year(&self, year: u32) -> Option<Rashi> {
        let elapsed = usize::try_from(year.checked_sub(1)?).ok()?;
        Some(step(self.start, Direction::Forward, elapsed))
    }
}

impl Timeline for ProfectionDasha {
    fn breadth(&self) -> usize {
        0
    }

    /// The `index`th year of `cycle`'s [`PROFECTION_YEARS`], so that a
    /// year's index is the native's completed age; the cycles never end.
    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        if index >= usize::from(PROFECTION_YEARS) {
            return None;
        }
        let sign = step(self.start, Direction::Forward, index);
        let years =
            f64::from(cycle) * f64::from(PROFECTION_YEARS) + f64::from(u8::try_from(index).ok()?);
        let from = self.birth.get() + years * self.year_days;
        Some(Period::of_sign(
            sign,
            lord(sign),
            Path::root(cycle, u8::try_from(index).ok()?),
            Interval::literal(from, from + self.year_days),
        ))
    }

    fn mahadasha_at(&self, instant: f64) -> Option<Period> {
        let into = instant - self.birth.get();
        if into < 0.0 {
            return None;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a non-negative whole number of years, far below u32::MAX for any date"
        )]
        let elapsed = (into / self.year_days).floor() as u32;
        let span = u32::from(PROFECTION_YEARS);
        self.mahadasha(elapsed / span, usize::try_from(elapsed % span).ok()?)
    }

    /// A year has no division here: Valens's months (IV.28) are read from
    /// transits, not cut from the year.
    fn child(&self, _parent: &Period, _index: usize) -> Option<Period> {
        None
    }
}

/// The catalogued systems these kernels run, in the catalogue's order.
///
/// ```
/// use teistro_core::catalogue::DashaSystem;
/// use teistro_dasha::TIME_LORDS;
///
/// assert!(TIME_LORDS.contains(&DashaSystem::Profection));
/// assert!(TIME_LORDS.iter().all(|system| teistro_dasha::systems().any(|s| s == *system)));
/// ```
pub const TIME_LORDS: [DashaSystem; 3] = [
    DashaSystem::ReleasingFortune,
    DashaSystem::ReleasingDaimon,
    DashaSystem::Profection,
];

/// The time lord a catalogued system names, begun from `start`: releasing
/// for the releasing family, the profected year for the profection family,
/// and nothing for a system of any other kernel.
///
/// ```
/// use teistro_core::catalogue::{DashaSystem, Rashi};
/// use teistro_core::quantity::JulianDay;
/// use teistro_core::settings::YearLength;
/// use teistro_dasha::time_lord;
///
/// let birth = JulianDay::literal(2_451_545.0);
/// let fortune = time_lord(DashaSystem::ReleasingFortune, Rashi::Leo, birth, YearLength::Savana360).ok_or("a time lord")?;
/// assert!(fortune.releasing().is_some());
/// // Vimshottari is no time lord.
/// assert!(time_lord(DashaSystem::Vimshottari, Rashi::Leo, birth, YearLength::Savana360).is_none());
/// # Ok::<(), &str>(())
/// ```
#[must_use]
pub fn time_lord(
    system: DashaSystem,
    start: Rashi,
    birth: JulianDay<Utc>,
    year_length: YearLength,
) -> Option<DashaCursor> {
    match system.attributes().family {
        DashaFamily::Releasing => Some(DashaCursor::Releasing(ReleasingDasha::new(
            start,
            birth,
            year_length,
        ))),
        DashaFamily::Profection => Some(DashaCursor::Profection(ProfectionDasha::new(
            start,
            birth,
            year_length,
        ))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, index what they built, and compare \
                  whole months, which a double holds exactly"
    )]

    use teistro_core::quantity::Depth;

    use super::*;

    const BIRTH: f64 = 2_451_545.0;

    fn releasing(start: Rashi) -> ReleasingDasha {
        ReleasingDasha::new(start, JulianDay::literal(BIRTH), YearLength::Savana360)
    }

    /// A period's start and end in months after birth, a month being 30
    /// days of the 360-day year.
    fn months(period: &Period) -> (f64, f64) {
        let month = |jd: JulianDay<Utc>| (jd.get() - BIRTH) / 30.0;
        (month(period.interval.from), month(period.interval.to))
    }

    fn level_one(dasha: &ReleasingDasha, sign: Rashi) -> Period {
        dasha.mahadashas().find(|p| p.sign == Some(sign)).unwrap()
    }

    /// Each child's sign and length in months.
    fn children(dasha: &ReleasingDasha, parent: &Period) -> Vec<(Rashi, f64)> {
        dasha
            .children(parent)
            .map(|child| {
                let (from, to) = months(&child);
                (child.sign.unwrap(), to - from)
            })
            .collect()
    }

    #[test]
    fn the_time_lords_are_the_kernels_their_family_names() {
        let birth = JulianDay::literal(BIRTH);
        for system in TIME_LORDS {
            let cursor = time_lord(system, Rashi::Leo, birth, YearLength::Savana360);
            assert!(cursor.is_some(), "{system:?}");
        }
        assert!(time_lord(DashaSystem::Chara, Rashi::Leo, birth, YearLength::Savana360).is_none());
    }

    #[test]
    fn a_sign_allots_its_lords_minimum_years_but_capricorn_and_aquarius() {
        let years: Vec<u32> = Rashi::ALL.into_iter().map(releasing_years).collect();
        assert_eq!(years, [15, 8, 20, 25, 19, 20, 8, 15, 12, 27, 30, 12]);
        assert_eq!(years.iter().sum::<u32>(), RELEASING_CYCLE);
    }

    /// IV.8: Fortune in Leo; the 70th year falls in Sagittarius, and inside
    /// it Sagittarius 1 year, Capricorn 2 years 3 months, Aquarius 2 years
    /// 6 months and Pisces 1 year, then Aries.
    #[test]
    fn valens_fortune_in_leo_reaches_sagittarius_in_the_seventieth_year() {
        let dasha = releasing(Rashi::Leo);
        let first: Vec<(Rashi, f64)> = dasha
            .mahadashas()
            .take(5)
            .map(|p| (p.sign.unwrap(), months(&p).0 / 12.0))
            .collect();
        assert_eq!(
            first,
            [
                (Rashi::Leo, 0.0),
                (Rashi::Virgo, 19.0),
                (Rashi::Libra, 39.0),
                (Rashi::Scorpio, 47.0),
                (Rashi::Sagittarius, 62.0),
            ]
        );
        let sagittarius = level_one(&dasha, Rashi::Sagittarius);
        assert_eq!(
            children(&dasha, &sagittarius)[..5],
            [
                (Rashi::Sagittarius, 12.0),
                (Rashi::Capricorn, 27.0),
                (Rashi::Aquarius, 30.0),
                (Rashi::Pisces, 12.0),
                (Rashi::Aries, 15.0),
            ]
        );
    }

    /// IV.8: Daimon in Scorpio; Aquarius from 54, and inside it eleven
    /// signs to Sagittarius, which ends at 69 years 4 months.
    #[test]
    fn valens_daimon_in_scorpio_ends_at_sixty_nine_years_four_months() {
        let dasha = releasing(Rashi::Scorpio);
        let aquarius = level_one(&dasha, Rashi::Aquarius);
        assert_eq!(months(&aquarius).0, 54.0 * 12.0);
        let inside = children(&dasha, &aquarius);
        assert_eq!(
            inside[..11],
            [
                (Rashi::Aquarius, 30.0),
                (Rashi::Pisces, 12.0),
                (Rashi::Aries, 15.0),
                (Rashi::Taurus, 8.0),
                (Rashi::Gemini, 20.0),
                (Rashi::Cancer, 25.0),
                (Rashi::Leo, 19.0),
                (Rashi::Virgo, 20.0),
                (Rashi::Libra, 8.0),
                (Rashi::Scorpio, 15.0),
                (Rashi::Sagittarius, 12.0),
            ]
        );
        let sagittarius = dasha.children(&aquarius).nth(10).unwrap();
        assert_eq!(months(&sagittarius).1, 69.0 * 12.0 + 4.0);
    }

    /// IV.4: Gemini's 20 years spend the twelve in 17 years 7 months, and
    /// the rest goes to Sagittarius, opposite, then Capricorn.
    #[test]
    fn the_bond_is_loosed_to_the_opposite_sign() {
        let dasha = releasing(Rashi::Gemini);
        let gemini = level_one(&dasha, Rashi::Gemini);
        let inside = children(&dasha, &gemini);
        assert_eq!(inside.len(), 14);
        assert_eq!(inside[11], (Rashi::Taurus, 8.0));
        assert_eq!(
            inside[12..],
            [(Rashi::Sagittarius, 12.0), (Rashi::Capricorn, 17.0)]
        );
        // A sign of fewer years never reaches it: IV.4's Aries cuts
        // Aquarius to the 11 months that fill out the 15 years.
        let aries = level_one(&releasing(Rashi::Aries), Rashi::Aries);
        let inside = children(&releasing(Rashi::Aries), &aries);
        assert_eq!(inside.last(), Some(&(Rashi::Aquarius, 11.0)));
        assert_eq!(inside.len(), 11);
    }

    /// IV.10: Fortune in Pisces, the fourth year. Gemini runs from 2 years
    /// 11 months; inside it Gemini 50 days, Cancer 62½, Leo 47½, Virgo 50,
    /// Libra 20, then Scorpio; inside Scorpio 3 days 3 hours, Sagittarius
    /// 2½ days, Capricorn 5 days 15 hours, Aquarius 6 days 6 hours, Pisces
    /// 2½ days, Aries 3 days 3 hours, and Taurus the rest.
    #[test]
    fn valens_fortune_in_pisces_goes_down_to_the_hours() {
        let dasha = releasing(Rashi::Pisces);
        let pisces = level_one(&dasha, Rashi::Pisces);
        let gemini = dasha.children(&pisces).nth(3).unwrap();
        assert_eq!(gemini.sign, Some(Rashi::Gemini));
        assert_eq!(months(&gemini).0, 35.0);
        let days = |p: &Period| p.interval.days();
        let third: Vec<(Rashi, f64)> = dasha
            .children(&gemini)
            .take(6)
            .map(|p| (p.sign.unwrap(), days(&p)))
            .collect();
        assert_eq!(
            third,
            [
                (Rashi::Gemini, 50.0),
                (Rashi::Cancer, 62.5),
                (Rashi::Leo, 47.5),
                (Rashi::Virgo, 50.0),
                (Rashi::Libra, 20.0),
                (Rashi::Scorpio, 37.5),
            ]
        );
        let scorpio = dasha.children(&gemini).nth(5).unwrap();
        let hours: Vec<(Rashi, f64)> = dasha
            .children(&scorpio)
            .map(|p| (p.sign.unwrap(), (days(&p) * 24.0 * 1e6).round() / 1e6))
            .collect();
        assert_eq!(
            hours[..6],
            [
                (Rashi::Scorpio, 75.0),
                (Rashi::Sagittarius, 60.0),
                (Rashi::Capricorn, 135.0),
                (Rashi::Aquarius, 150.0),
                (Rashi::Pisces, 60.0),
                (Rashi::Aries, 75.0),
            ]
        );
        assert_eq!(hours[6].0, Rashi::Taurus);
        // The 255th day of Gemini (8 months 15 days in) is in Scorpio's
        // 25th, as Valens counts it.
        let at = gemini.interval.from.get() + 255.0;
        let chain = dasha.at(JulianDay::literal(at), Depth::try_new(4).unwrap());
        let signs: Vec<Rashi> = chain.iter().filter_map(|p| p.sign).collect();
        assert_eq!(signs[..3], [Rashi::Pisces, Rashi::Gemini, Rashi::Scorpio]);
    }

    /// II.17K: the lot stands as the Ascendant, the tenth from it as the
    /// MC, the seventh as the Descendant and the fourth as the IC; the
    /// other eight signs are at no angle.
    #[test]
    fn a_lot_has_four_angles_and_the_tenth_is_its_mc() {
        for lot in Rashi::ALL {
            let angles: Vec<(usize, LotAngle)> = (0..SIGNS)
                .filter_map(|on| {
                    releasing_angle(lot, step(lot, Direction::Forward, on)).map(|a| (on, a))
                })
                .collect();
            assert_eq!(
                angles,
                [
                    (0, LotAngle::First),
                    (3, LotAngle::Fourth),
                    (6, LotAngle::Seventh),
                    (9, LotAngle::Tenth),
                ],
                "{lot:?}"
            );
        }
        for angle in LotAngle::ALL {
            assert_eq!(LotAngle::of(angle.house()), Some(angle));
        }
    }

    /// IV.8: the Ascendant in Pisces and the Lot of Fortune in Leo, which
    /// Valens finds "preceding an angle <the Descendant>" (168K): the sixth
    /// sign, no angle. Daimon's distribution to "MC relative to the Lot of
    /// Fortune" (IV.7) is counted from Leo, so it is Taurus.
    #[test]
    fn valens_fortune_in_leo_precedes_the_descendant() {
        assert_eq!(releasing_angle(Rashi::Pisces, Rashi::Leo), None);
        assert_eq!(House::between(Rashi::Pisces, Rashi::Leo).get(), 6);
        assert_eq!(
            releasing_angle(Rashi::Pisces, Rashi::Virgo),
            Some(LotAngle::Seventh)
        );
        assert_eq!(
            releasing_angle(Rashi::Leo, Rashi::Taurus),
            Some(LotAngle::Tenth)
        );
        // Releasing from Daimon in Scorpio, read from Fortune: Daimon's own
        // sign is the fourth from Leo and Aquarius, from 54, the seventh.
        // Counted, not printed.
        let daimon = releasing(Rashi::Scorpio);
        let angles: Vec<(Rashi, Option<LotAngle>)> = daimon
            .mahadashas()
            .take(4)
            .filter_map(|p| p.sign)
            .map(|sign| (sign, releasing_angle(Rashi::Leo, sign)))
            .collect();
        assert_eq!(
            angles,
            [
                (Rashi::Scorpio, Some(LotAngle::Fourth)),
                (Rashi::Sagittarius, None),
                (Rashi::Capricorn, None),
                (Rashi::Aquarius, Some(LotAngle::Seventh)),
            ]
        );
    }

    /// IV.4: the vital sector begun from Gemini, Cancer, Leo, Virgo,
    /// Capricorn or Aquarius is loosed after 17 years 7 months, and no
    /// other sign's years reach it.
    #[test]
    fn valens_six_signs_reach_the_loosing() {
        let reached: Vec<Rashi> = Rashi::ALL
            .into_iter()
            .filter(|&start| {
                let dasha = releasing(start);
                dasha.loosing(&level_one(&dasha, start)).is_some()
            })
            .collect();
        assert_eq!(
            reached,
            [
                Rashi::Gemini,
                Rashi::Cancer,
                Rashi::Leo,
                Rashi::Virgo,
                Rashi::Capricorn,
                Rashi::Aquarius,
            ]
        );
        for start in reached {
            let dasha = releasing(start);
            let parent = level_one(&dasha, start);
            let loosed = dasha.loosing(&parent).unwrap();
            assert_eq!(months(&loosed).0 - months(&parent).0, 211.0, "{start:?}");
            assert_eq!(loosed.sign, Some(start.opposite()), "{start:?}");
            assert!(ReleasingDasha::is_loosed(&loosed));
            assert_eq!(loosed.path.indices().last(), Some(&12));
        }
    }

    /// IV.4 (163K): Leo transmits to Aquarius and Aquarius to Leo, Cancer
    /// and Capricorn to each other, Virgo and Pisces likewise.
    #[test]
    fn valens_loosing_transmits_to_the_sign_in_opposition() {
        for (from, to) in [
            (Rashi::Leo, Rashi::Aquarius),
            (Rashi::Aquarius, Rashi::Leo),
            (Rashi::Cancer, Rashi::Capricorn),
            (Rashi::Capricorn, Rashi::Cancer),
            (Rashi::Virgo, Rashi::Pisces),
        ] {
            let dasha = releasing(from);
            let loosed = dasha.loosing(&level_one(&dasha, from)).unwrap();
            assert_eq!(loosed.sign, Some(to), "{from:?}");
        }
    }

    /// IV.8: Daimon in Scorpio ends at 69 years 4 months inside Aquarius,
    /// before Aquarius's twelve are spent at 71 years 7 months, so none of
    /// the periods Valens prints is a loosed one.
    #[test]
    fn valens_daimon_in_scorpio_ends_before_the_loosing() {
        let dasha = releasing(Rashi::Scorpio);
        let aquarius = level_one(&dasha, Rashi::Aquarius);
        let printed: Vec<Period> = dasha.children(&aquarius).take(11).collect();
        assert!(!printed.iter().any(ReleasingDasha::is_loosed));
        let loosed = dasha.loosing(&aquarius).unwrap();
        assert_eq!(months(&loosed).0, 71.0 * 12.0 + 7.0);
        assert_eq!(loosed.sign, Some(Rashi::Leo));
        let after: Vec<Rashi> = dasha
            .children(&aquarius)
            .filter(ReleasingDasha::is_loosed)
            .map(|p| p.sign.unwrap())
            .collect();
        // Leo 19 months, Virgo 20, Libra 8, Scorpio 15, Sagittarius 12,
        // Capricorn 27, Aquarius 30, Pisces 12, and Aries the 6 left of 360.
        assert_eq!(after.len(), 9);
        assert_eq!(after.last(), Some(&Rashi::Aries));
    }

    /// IV.10: the cycle of days is 528, and the remaining days run from
    /// the sign in opposition. The kernel's is 211 × 2½ = 527½ days, which
    /// Valens rounds. Aquarius's 30 months hold a whole cycle and more,
    /// and Pisces's 12 do not.
    #[test]
    fn valens_days_are_loosed_after_their_cycle() {
        let dasha = releasing(Rashi::Aquarius);
        let aquarius = level_one(&dasha, Rashi::Aquarius);
        let months_of_aquarius = dasha.children(&aquarius).next().unwrap();
        assert_eq!(months_of_aquarius.sign, Some(Rashi::Aquarius));
        let loosed = dasha.loosing(&months_of_aquarius).unwrap();
        let into = loosed.interval.from.get() - months_of_aquarius.interval.from.get();
        assert_eq!(into, 527.5);
        assert_eq!(loosed.sign, Some(Rashi::Leo));
        assert_eq!(loosed.path.depth(), 3);
        let shorter = dasha.children(&aquarius).nth(1).unwrap();
        assert_eq!(shorter.sign, Some(Rashi::Pisces));
        assert!(dasha.loosing(&shorter).is_none());
    }

    #[test]
    fn the_children_partition_their_parent() {
        for start in Rashi::ALL {
            let dasha = releasing(start);
            for parent in dasha.mahadashas() {
                let inside: Vec<Period> = dasha.children(&parent).collect();
                assert_eq!(inside[0].interval.from, parent.interval.from);
                assert_eq!(inside.last().unwrap().interval.to, parent.interval.to);
                for pair in inside.windows(2) {
                    assert_eq!(pair[0].interval.to, pair[1].interval.from);
                }
            }
        }
    }

    /// IV.11: the Ascendant in Virgo, the 35th year is Cancer. The ordinal
    /// counts from 1 for the year birth begins, so reading 35 as an age
    /// would give Leo, the rival this pins against.
    #[test]
    fn valens_thirty_fifth_year_from_virgo_is_cancer() {
        let dasha = ProfectionDasha::new(
            Rashi::Virgo,
            JulianDay::literal(BIRTH),
            YearLength::Julian36525,
        );
        assert_eq!(dasha.sign_of_year(35), Some(Rashi::Cancer));
        assert_eq!(dasha.sign_of_year(1), Some(Rashi::Virgo));
        assert_eq!(dasha.sign_of_year(13), Some(Rashi::Virgo));
        assert_eq!(dasha.sign_of_year(0), None);
        let thirty_fifth = BIRTH + 34.5 * 365.25;
        let chain = dasha.at(JulianDay::literal(thirty_fifth), Depth::try_new(1).unwrap());
        let period = chain.iter().next().unwrap();
        assert_eq!(period.sign, Some(Rashi::Cancer));
        assert_eq!(period.lord, Graha::Moon);
        // The year's index is the completed age, 34, in the first cycle.
        assert_eq!(period.path.to_string(), "34");
        assert_ne!(Some(period.sign.unwrap()), dasha.sign_of_year(36));
    }

    /// The twelve first-level periods are the whole first cycle, and
    /// nothing lies outside it: before birth, at its end, or a later cycle.
    #[test]
    fn releasing_runs_one_cycle_of_the_twelve() {
        let dasha = releasing(Rashi::Gemini);
        assert!(dasha.mahadasha(0, 11).is_some());
        assert!(dasha.mahadasha(0, SIGNS).is_none());
        assert!(dasha.mahadasha(1, 0).is_none());
        let end = BIRTH + f64::from(RELEASING_CYCLE) * 360.0;
        assert_eq!(dasha.mahadasha(0, 11).unwrap().interval.to.get(), end);
        assert!(dasha.mahadasha_at(BIRTH - 1e-6).is_none());
        assert!(dasha.mahadasha_at(end).is_none());
        let first = dasha.mahadasha_at(BIRTH).unwrap();
        assert_eq!(
            (first.sign, first.path.to_string()),
            (Some(Rashi::Gemini), String::from("0"))
        );
        // A period's end is the next one's start: Gemini's 20 years give
        // way to Cancer's.
        let at_end = dasha.mahadasha_at(BIRTH + 20.0 * 360.0).unwrap();
        assert_eq!(at_end.sign, Some(Rashi::Cancer));
        let before_end = dasha.mahadasha_at(BIRTH + 20.0 * 360.0 - 1e-6).unwrap();
        assert_eq!(before_end.sign, Some(Rashi::Gemini));
        let last = dasha.mahadasha_at(end - 1e-6).unwrap();
        assert_eq!(last.sign, Some(Rashi::Taurus));
        // A birth far from the epoch reads the same: the instant is
        // counted from the birth.
        let later = ReleasingDasha::new(
            Rashi::Gemini,
            JulianDay::literal(BIRTH + 1_000.0),
            YearLength::Savana360,
        );
        assert_eq!(
            later.mahadasha_at(BIRTH + 1_000.0).unwrap().sign,
            Some(Rashi::Gemini)
        );
        assert!(later.mahadasha_at(BIRTH + 999.0).is_none());
    }

    /// Only a period below the first level is loosed, however its last
    /// place reads.
    #[test]
    fn a_first_level_period_is_never_loosed() {
        let place = |path: Path| {
            Period::of_sign(
                Rashi::Aries,
                Graha::Mars,
                path,
                Interval::literal(BIRTH, BIRTH + 1.0),
            )
        };
        assert!(!ReleasingDasha::is_loosed(&place(Path::root(0, 12))));
        assert!(ReleasingDasha::is_loosed(&place(
            Path::root(0, 0).child(12).unwrap()
        )));
        assert!(!ReleasingDasha::is_loosed(&place(
            Path::root(0, 0).child(11).unwrap()
        )));
    }

    /// The deepest level has no children, and a place past the most a
    /// period holds has none.
    #[test]
    fn the_deepest_period_has_no_children() {
        let dasha = releasing(Rashi::Aquarius);
        let mut period = dasha.mahadashas().next().unwrap();
        while let Some(child) = dasha.child(&period, 0) {
            period = child;
        }
        assert_eq!(period.path.depth(), crate::tree::MAX_DEPTH);
        let aquarius = dasha.mahadashas().next().unwrap();
        assert_eq!(dasha.children(&aquarius).count(), 21);
        assert!(dasha.child(&aquarius, RELEASING_BREADTH).is_none());
    }

    /// A year a sign, from the start, cycle after cycle without end, the
    /// year that begins at birth included.
    #[test]
    fn the_profected_year_runs_a_sign_a_year_without_end() {
        let dasha = ProfectionDasha::new(
            Rashi::Virgo,
            JulianDay::literal(BIRTH),
            YearLength::Julian36525,
        );
        let year = 365.25;
        let third = dasha.mahadasha(0, 3).unwrap();
        assert_eq!(third.sign, Some(Rashi::Sagittarius));
        assert_eq!(third.interval.from.get(), BIRTH + 3.0 * year);
        assert_eq!(third.interval.to.get(), BIRTH + 4.0 * year);
        let next_cycle = dasha.mahadasha(2, 5).unwrap();
        assert_eq!(next_cycle.interval.from.get(), BIRTH + 245.0 * year);
        assert_eq!(next_cycle.path.to_string(), "2:5");
        assert_eq!(next_cycle.sign, Some(Rashi::Aquarius));
        assert!(dasha.mahadasha(0, usize::from(PROFECTION_YEARS)).is_none());
        let at_birth = dasha.mahadasha_at(BIRTH).unwrap();
        assert_eq!(
            (at_birth.sign, at_birth.path.to_string()),
            (Some(Rashi::Virgo), String::from("0"))
        );
        assert!(dasha.mahadasha_at(BIRTH - 1e-6).is_none());
        let late = dasha.mahadasha_at(BIRTH + 130.5 * year).unwrap();
        assert_eq!(late.path.to_string(), "1:10");
        assert!(dasha.child(&at_birth, 0).is_none());
    }
}

//! Aayadi: a building's measure read by remainders, each system from its
//! own text (`03-design/aayadi.md`, cruxes C1190 to C1199).
//!
//! A vastu text tests a measure before it is built: a side, the sum of two
//! sides, the perimeter or the area is multiplied by a small number and
//! divided by the size of a named series, and the remainder names the
//! series' member — the aaya, the vyaya, the yoni, the nakshatra, the
//! weekday, the tithi and the rest. Four systems are read, each from a
//! public-domain print: *Mayamata* IX, *Mānasāra* IX, *Samarāṅgaṇa
//! Sūtradhāra* XXVI and *Muhūrta Cintāmaṇi* XII. Nothing reads the sky; the
//! arithmetic is exact.
//!
//! The answer reports each quantity and each clause the source judges, never
//! one verdict:
//!
//! ```
//! use teistro_core::catalogue::{Nakshatra, VastuYoni};
//! use teistro_muhurta::aayadi::{AayadiSystem, Plot, aayadi};
//!
//! // Mahidhara's example under Muhurta Chintamani XII v. 3: 29 by 7 hastas,
//! // the area 203, reckons the lion and Rohini.
//! let reading = aayadi(AayadiSystem::Chintamani, Plot::new(29, 7)?, None);
//! assert_eq!(reading.yoni(), Some(VastuYoni::Simha));
//! assert_eq!(reading.nakshatra(), Some(Nakshatra::Rohini));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Nakshatra, Tithi, Vara, VastuYoni, VyayaKind, Yoga};
use teistro_core::error::Error;

use crate::tara::{Tara, tara};

/// The longest side a reckoning takes, in whatever unit it is measured: a
/// million, so that the area times nine stays exact in 64 bits.
pub const LONGEST_SIDE: u32 = 1_000_000;

/// The text whose reckoning is read.
///
/// ```
/// use teistro_muhurta::aayadi::AayadiSystem;
///
/// assert_eq!(AayadiSystem::ALL.len(), 4);
/// assert_eq!(serde_json::to_string(&AayadiSystem::Samarangana)?, "\"SAMARANGANA\"");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AayadiSystem {
    /// *Mayamata* IX vv. 20–24: the sum of the sides for the aaya, vyaya,
    /// yoni and nakshatra, the perimeter for the tithi and the weekday.
    Mayamata,
    /// *Mānasāra* IX vv. 63–77: six reckonings, on the length, the breadth
    /// and the perimeter by default ([`ManasaraMeasure`]).
    Manasara,
    /// *Samarāṅgaṇa Sūtradhāra* XXVI: the area's aaya and nakshatra, the
    /// nakshatra's vyaya, and the owner's tara and gana.
    Samarangana,
    /// *Muhūrta Cintāmaṇi* XII vv. 4–12: the area's nine reckonings and
    /// the nakshatra's vyaya.
    Chintamani,
}

impl AayadiSystem {
    /// Every system, in the order the design page reads them.
    pub const ALL: [AayadiSystem; 4] = [
        AayadiSystem::Mayamata,
        AayadiSystem::Manasara,
        AayadiSystem::Samarangana,
        AayadiSystem::Chintamani,
    ];
}

/// The measure a reckoning starts from.
///
/// ```
/// use teistro_muhurta::aayadi::{Measure, Plot};
///
/// let plot = Plot::new(29, 7)?;
/// assert_eq!((Measure::Sum.of(plot), Measure::Perimeter.of(plot), Measure::Area.of(plot)), (36, 72, 203));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Measure {
    /// The length and the breadth added (*Mayamata* IX v. 20's
    /// *vyāsāyāma-samūha*, crux C1190).
    Sum,
    /// The length alone.
    Length,
    /// The breadth alone.
    Breadth,
    /// The perimeter, twice the sum (*pariṇāha*).
    Perimeter,
    /// The area, the length times the breadth (*kṣetraphala*, *piṇḍa*).
    Area,
    /// The house's nakshatra, counted 1 (Ashvini) to 27: the vyaya of
    /// *Samarāṅgaṇa* XXVI v. 36 and *Muhūrta Cintāmaṇi* XII v. 7.
    HouseStar,
}

impl Measure {
    /// The measure of a plot. [`Measure::HouseStar`] is not a measure of
    /// the plot and reads 0 here; a reckoning takes it from the plot's
    /// nakshatra.
    ///
    /// ```
    /// use teistro_muhurta::aayadi::{Measure, Plot};
    ///
    /// let plot = Plot::new(29, 7)?;
    /// assert_eq!((Measure::Length.of(plot), Measure::Breadth.of(plot), Measure::HouseStar.of(plot)), (29, 7, 0));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn of(self, plot: Plot) -> u64 {
        let (length, breadth) = (u64::from(plot.length), u64::from(plot.breadth));
        match self {
            Measure::Sum => length + breadth,
            Measure::Length => length,
            Measure::Breadth => breadth,
            Measure::Perimeter => 2 * (length + breadth),
            Measure::Area => length * breadth,
            Measure::HouseStar => 0,
        }
    }
}

/// The measure *Mānasāra* IX takes (crux C1194): the split "some" give, or
/// one measure for all six reckonings.
///
/// ```
/// use teistro_muhurta::aayadi::ManasaraMeasure;
///
/// assert_eq!(ManasaraMeasure::default(), ManasaraMeasure::Split);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManasaraMeasure {
    /// The aaya and nakshatra on the length, the vyaya and yoni on the
    /// breadth, the weekday and tithi on the perimeter.
    #[default]
    Split,
    /// The sum of the sides for all six.
    Sum,
    /// The length for all six.
    Length,
    /// The breadth for all six.
    Breadth,
    /// The perimeter for all six.
    Perimeter,
}

impl ManasaraMeasure {
    /// The measure this choice gives a quantity.
    const fn measure(self, quantity: Quantity) -> Measure {
        match self {
            ManasaraMeasure::Split => match quantity {
                Quantity::Aaya | Quantity::Nakshatra => Measure::Length,
                Quantity::Vyaya | Quantity::Yoni => Measure::Breadth,
                _ => Measure::Perimeter,
            },
            ManasaraMeasure::Sum => Measure::Sum,
            ManasaraMeasure::Length => Measure::Length,
            ManasaraMeasure::Breadth => Measure::Breadth,
            ManasaraMeasure::Perimeter => Measure::Perimeter,
        }
    }
}

/// What a reckoning names.
///
/// ```
/// use teistro_muhurta::aayadi::{AayadiSystem, Plot, Quantity, aayadi};
///
/// let reading = aayadi(AayadiSystem::Samarangana, Plot::new(29, 7)?, None);
/// let named: Vec<Quantity> = reading.quantities.iter().map(|reckoned| reckoned.quantity).collect();
/// assert_eq!(named, [Quantity::Aaya, Quantity::Nakshatra, Quantity::Vyaya]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Quantity {
    /// The aaya, the income: twelve in *Mayamata* and *Mānasāra*, the
    /// eight of the banner series in *Samarāṅgaṇa* and *Muhūrta Cintāmaṇi*.
    Aaya,
    /// The vyaya, the expenditure.
    Vyaya,
    /// The yoni, the eight of the banner series.
    Yoni,
    /// The building's nakshatra, from Ashvini.
    Nakshatra,
    /// The weekday, from Sunday.
    Vara,
    /// The tithi.
    Tithi,
    /// The amsha, a ninth (*Muhūrta Cintāmaṇi* XII v. 11; crux C1196).
    Amsha,
    /// The wealth (*dravya*, *dhana*).
    Dhana,
    /// The debt (*ṛṇa*).
    Rina,
    /// The yoga, from Vishkambha.
    Yoga,
    /// The age (*āyu*), in years.
    Ayu,
}

/// A building's two sides, in one unit the caller chooses: hastas, kols,
/// daṇḍas or aṅgulas. The texts take a finer unit when a coarser gives no
/// good aaya (*Samarāṅgaṇa* XXVI vv. 14–16), so a fraction is passed as a
/// whole number of the finer unit.
///
/// ```
/// use teistro_muhurta::aayadi::Plot;
///
/// assert!(Plot::new(29, 7).is_ok());
/// assert_eq!(Plot::new(0, 7).unwrap_err().field(), Some("length"));
/// assert_eq!(Plot::new(29, 1_000_001).unwrap_err().field(), Some("breadth"));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Plot {
    /// The length, 1 to [`LONGEST_SIDE`].
    pub length: u32,
    /// The breadth, 1 to [`LONGEST_SIDE`].
    pub breadth: u32,
}

impl Plot {
    /// A plot of these sides.
    ///
    /// ```
    /// use teistro_muhurta::aayadi::{LONGEST_SIDE, Plot};
    ///
    /// assert_eq!(Plot::new(29, 7)?.length, 29);
    /// assert!(Plot::new(LONGEST_SIDE, LONGEST_SIDE).is_ok());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on a side of 0 or above [`LONGEST_SIDE`], named
    /// `length` or `breadth`.
    pub fn new(length: u32, breadth: u32) -> Result<Plot, Error> {
        for (side, name) in [(length, "length"), (breadth, "breadth")] {
            if side == 0 || side > LONGEST_SIDE {
                return Err(Error::invalid_arg(format!(
                    "a plot's {name} is 1 to {LONGEST_SIDE} whole units, not {side}"
                ))
                .with_field(name)
                .with_hint(String::from(
                    "pass a fraction as a whole number of a finer unit",
                )));
            }
        }
        Ok(Plot { length, breadth })
    }
}

/// One reckoning: a measure times a multiplier, divided by the size of a
/// series, and the member its remainder names.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_muhurta::aayadi::{AayadiSystem, Plot, Quantity, aayadi};
///
/// // Mayamata IX v. 22: the sum 36 times 8 is 288, over 27 leaves 18,
/// // Jyeshtha, and the quotient 10 is the building's age.
/// let reading = aayadi(AayadiSystem::Mayamata, Plot::new(29, 7)?, None);
/// let star = reading.get(Quantity::Nakshatra).copied();
/// assert_eq!(star.map(|star| (star.value, star.quotient, star.count, star.nakshatra)), Some((36, 10, 18, Some(Nakshatra::Jyeshtha))));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Reckoned {
    /// What the remainder names.
    pub quantity: Quantity,
    /// The measure it starts from.
    pub measure: Measure,
    /// The measure's value.
    pub value: u64,
    /// The multiplier.
    pub multiplier: u16,
    /// The divisor, the size of the series.
    pub divisor: u16,
    /// The quotient: *Mayamata* IX v. 22 reads the nakshatra's as the
    /// building's age.
    pub quotient: u64,
    /// The remainder, 0 to the divisor less one.
    pub remainder: u16,
    /// The member's place, 1 to the divisor: the remainder, or the divisor
    /// for a remainder of 0 (crux C1192).
    pub count: u16,
    /// The member of the banner series, for an eightfold aaya or a yoni.
    pub yoni: Option<VastuYoni>,
    /// The nakshatra, for the nakshatra.
    pub nakshatra: Option<Nakshatra>,
    /// The weekday, for the weekday.
    pub vara: Option<Vara>,
    /// The tithi, for a thirtyfold tithi; *Muhūrta Cintāmaṇi*'s fifteen
    /// name a tithi of either half and leave this empty.
    pub tithi: Option<Tithi>,
    /// The yoga, for the yoga.
    pub yoga: Option<Yoga>,
    /// The verse.
    pub source: &'static str,
}

/// A condition a source judges a measure by, reported as holding or not;
/// holding is the good reading.
///
/// ```
/// use teistro_muhurta::aayadi::{AayadiClauseKey, AayadiSystem, Plot, aayadi};
///
/// // 29 by 7 under Muhurta Chintamani: the lion, the 3rd aaya, is odd.
/// let reading = aayadi(AayadiSystem::Chintamani, Plot::new(29, 7)?, None);
/// assert_eq!(reading.holds(AayadiClauseKey::AayaOdd), Some(true));
/// assert_eq!(reading.holds(AayadiClauseKey::GanaSame), None);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AayadiClauseKey {
    /// The yoni (or eightfold aaya) is the banner, the lion, the bull or the
    /// elephant.
    YoniGood,
    /// The aaya's count exceeds the vyaya's.
    AayaExceedsVyaya,
    /// The weekday is the Moon's, Mercury's, Jupiter's or Venus's.
    VaraGood,
    /// The vyaya is less than the aaya: the *yakṣa* vyaya.
    VyayaLessThanAaya,
    /// There is a vyaya: its remainder is not 0.
    VyayaPresent,
    /// The tara from the owner's star is not the 3rd, 5th or 7th.
    TaraNotAvoided,
    /// The house's star is of the owner's star's gana.
    GanaSame,
    /// The eightfold aaya is odd.
    AayaOdd,
    /// The wealth exceeds the debt.
    DhanaExceedsRina,
    /// The tithi is not a rikta, the 4th, 9th or 14th.
    TithiNotRikta,
    /// The house's star is not the owner's own.
    OwnerStarDiffers,
}

/// A clause judged.
///
/// ```
/// use teistro_muhurta::aayadi::{AayadiClauseKey, AayadiSystem, Plot, aayadi};
///
/// let reading = aayadi(AayadiSystem::Mayamata, Plot::new(29, 7)?, None);
/// let first = reading.clauses[0];
/// assert_eq!((first.clause, first.source), (AayadiClauseKey::YoniGood, "Mayamata IX v. 22"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AayadiClause {
    /// The condition.
    pub clause: AayadiClauseKey,
    /// Whether it holds, the good reading.
    pub holds: bool,
    /// The verse or gloss that judges it.
    pub source: &'static str,
}

/// What a source asks of a measure that the reckoning is not given, or
/// cannot read without a choice the source does not make.
///
/// ```
/// use teistro_muhurta::aayadi::{AayadiSystem, AayadiUnjudged, Plot, aayadi};
///
/// let reading = aayadi(AayadiSystem::Mayamata, Plot::new(29, 7)?, None);
/// assert_eq!(reading.unjudged, [AayadiUnjudged::OwnerStars]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AayadiUnjudged {
    /// The measure against the owner's and the site's stars (*Mayamata*
    /// IX v. 19), not reckoned in the verse.
    OwnerStars,
    /// The nakshatra against the owner's (*Mānasāra* IX vv. 78–81).
    NakshatraAgainstOwner,
    /// Indra, Yama or Raja, from the house's name (*Samarāṅgaṇa* XXVI
    /// vv. 37–39, *Muhūrta Cintāmaṇi* XII v. 7; crux C1196).
    Amsha,
    /// The house's name against the owner's (*Samarāṅgaṇa* XXVI v. 52).
    HouseName,
    /// The signs of the owner's and the house's stars (crux C1198).
    Signs,
    /// The life of *Samarāṅgaṇa* XXVI v. 56 (crux C1197).
    Life,
    /// The count of six good *karaṇas* (*Samarāṅgaṇa* XXVI vv. 49–50),
    /// two of which are unjudged.
    Karanas,
    /// The new moon among the fifteen tithis (Mahidhara's gloss of
    /// *Muhūrta Cintāmaṇi* XII v. 12), which cannot be told from the full.
    NewMoon,
    /// "Vyatipata and the other" yogas (the same gloss), not listed.
    Yogas,
}

/// A measure read by one system.
///
/// ```
/// use teistro_muhurta::aayadi::{AayadiSystem, Plot, aayadi};
///
/// let reading = aayadi(AayadiSystem::Chintamani, Plot::new(29, 7)?, None);
/// assert_eq!((reading.system, reading.quantities.len(), reading.vyaya, reading.tara), (AayadiSystem::Chintamani, 10, None, None));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Aayadi {
    /// The system.
    pub system: AayadiSystem,
    /// Every reckoning the system names, in its text's order.
    pub quantities: Vec<Reckoned>,
    /// *Samarāṅgaṇa*'s vyaya by its size beside the aaya (XXVI v. 37), for
    /// that system alone.
    pub vyaya: Option<VyayaKind>,
    /// The tara counted from the owner's star to the house's, where the
    /// system reads it and an owner was given.
    pub tara: Option<Tara>,
    /// Each clause the system judges, in its text's order.
    pub clauses: Vec<AayadiClause>,
    /// What the system asks that is not judged.
    pub unjudged: Vec<AayadiUnjudged>,
}

impl Aayadi {
    /// The reckoning of a quantity, if the system names it.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, VastuYoni};
    /// use teistro_muhurta::aayadi::{AayadiClauseKey, AayadiSystem, Plot, Quantity, aayadi};
    ///
    /// let reading = aayadi(AayadiSystem::Chintamani, Plot::new(29, 7)?, None);
    /// assert_eq!(reading.get(Quantity::Dhana).map(|dhana| dhana.count), Some(4));
    /// assert!(reading.get(Quantity::Yoni).is_none());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn get(&self, quantity: Quantity) -> Option<&Reckoned> {
        self.quantities
            .iter()
            .find(|reckoned| reckoned.quantity == quantity)
    }

    /// The member of the banner series: the eightfold aaya or the yoni.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, VastuYoni};
    /// use teistro_muhurta::aayadi::{AayadiClauseKey, AayadiSystem, Plot, Quantity, aayadi};
    ///
    /// let reading = aayadi(AayadiSystem::Chintamani, Plot::new(29, 7)?, None);
    /// assert_eq!(reading.yoni(), Some(VastuYoni::Simha));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn yoni(&self) -> Option<VastuYoni> {
        self.quantities.iter().find_map(|reckoned| reckoned.yoni)
    }

    /// The building's nakshatra.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, VastuYoni};
    /// use teistro_muhurta::aayadi::{AayadiClauseKey, AayadiSystem, Plot, Quantity, aayadi};
    ///
    /// let reading = aayadi(AayadiSystem::Chintamani, Plot::new(29, 7)?, None);
    /// assert_eq!(reading.nakshatra(), Some(Nakshatra::Rohini));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn nakshatra(&self) -> Option<Nakshatra> {
        self.get(Quantity::Nakshatra)
            .and_then(|reckoned| reckoned.nakshatra)
    }

    /// Whether a clause holds, if the system judged it.
    ///
    /// ```
    /// use teistro_core::catalogue::{Nakshatra, VastuYoni};
    /// use teistro_muhurta::aayadi::{AayadiClauseKey, AayadiSystem, Plot, Quantity, aayadi};
    ///
    /// let reading = aayadi(AayadiSystem::Chintamani, Plot::new(29, 7)?, None);
    /// assert_eq!(reading.holds(AayadiClauseKey::DhanaExceedsRina), Some(true));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn holds(&self, clause: AayadiClauseKey) -> Option<bool> {
        self.clauses
            .iter()
            .find(|judged| judged.clause == clause)
            .map(|judged| judged.holds)
    }
}

/// One row of a system's table: what it names, from which measure, times
/// what, over what, and the verse.
#[derive(Clone, Copy, Debug)]
struct Row {
    quantity: Quantity,
    measure: Measure,
    multiplier: u16,
    divisor: u16,
    source: &'static str,
}

const fn row(
    quantity: Quantity,
    measure: Measure,
    multiplier: u16,
    divisor: u16,
    source: &'static str,
) -> Row {
    Row {
        quantity,
        measure,
        multiplier,
        divisor,
        source,
    }
}

/// *Mayamata* IX vv. 20–23 (TSS 65, 1919, pp. 26–27).
const MAYAMATA: [Row; 6] = [
    row(Quantity::Aaya, Measure::Sum, 8, 12, "Mayamata IX v. 20"),
    row(Quantity::Vyaya, Measure::Sum, 9, 10, "Mayamata IX v. 20"),
    row(Quantity::Yoni, Measure::Sum, 3, 8, "Mayamata IX v. 21"),
    row(
        Quantity::Nakshatra,
        Measure::Sum,
        8,
        27,
        "Mayamata IX v. 22 (C1199)",
    ),
    row(
        Quantity::Tithi,
        Measure::Perimeter,
        1,
        30,
        "Mayamata IX v. 23 (C1191)",
    ),
    row(
        Quantity::Vara,
        Measure::Perimeter,
        1,
        7,
        "Mayamata IX v. 23 (C1191)",
    ),
];

/// *Mānasāra* IX vv. 63–73, as Acharya's *Dictionary of Hindu Architecture*
/// (1927) quotes them; the measure is laid on by [`ManasaraMeasure`].
const MANASARA: [Row; 6] = [
    row(
        Quantity::Aaya,
        Measure::Length,
        8,
        12,
        "Manasara IX vv. 63-73",
    ),
    row(
        Quantity::Nakshatra,
        Measure::Length,
        8,
        27,
        "Manasara IX vv. 63-73",
    ),
    row(
        Quantity::Vyaya,
        Measure::Breadth,
        9,
        10,
        "Manasara IX vv. 63-73",
    ),
    row(
        Quantity::Yoni,
        Measure::Breadth,
        3,
        8,
        "Manasara IX vv. 63-73",
    ),
    row(
        Quantity::Vara,
        Measure::Perimeter,
        9,
        7,
        "Manasara IX vv. 63-73",
    ),
    row(
        Quantity::Tithi,
        Measure::Perimeter,
        9,
        30,
        "Manasara IX vv. 63-73",
    ),
];

/// *Samarāṅgaṇa Sūtradhāra* XXVI (GOS 25, 1924, pp. 147–150).
const SAMARANGANA: [Row; 3] = [
    row(
        Quantity::Aaya,
        Measure::Area,
        1,
        8,
        "Samarangana XXVI vv. 17-18",
    ),
    row(
        Quantity::Nakshatra,
        Measure::Area,
        8,
        27,
        "Samarangana XXVI vv. 35-36",
    ),
    row(
        Quantity::Vyaya,
        Measure::HouseStar,
        1,
        8,
        "Samarangana XXVI v. 36",
    ),
];

/// *Muhūrta Cintāmaṇi* XII v. 11, with v. 7's vyaya (Mahidhara, 1892,
/// pp. 171–173; crux C1193).
const CHINTAMANI: [Row; 10] = [
    row(
        Quantity::Aaya,
        Measure::Area,
        9,
        8,
        "Muhurta Chintamani XII v. 11 (v. 4)",
    ),
    row(
        Quantity::Vara,
        Measure::Area,
        9,
        7,
        "Muhurta Chintamani XII v. 11",
    ),
    row(
        Quantity::Amsha,
        Measure::Area,
        6,
        9,
        "Muhurta Chintamani XII v. 11 (C1196)",
    ),
    row(
        Quantity::Dhana,
        Measure::Area,
        8,
        12,
        "Muhurta Chintamani XII v. 11",
    ),
    row(
        Quantity::Rina,
        Measure::Area,
        3,
        8,
        "Muhurta Chintamani XII v. 11",
    ),
    row(
        Quantity::Nakshatra,
        Measure::Area,
        8,
        27,
        "Muhurta Chintamani XII v. 11",
    ),
    row(
        Quantity::Tithi,
        Measure::Area,
        8,
        15,
        "Muhurta Chintamani XII v. 11",
    ),
    row(
        Quantity::Yoga,
        Measure::Area,
        4,
        27,
        "Muhurta Chintamani XII v. 11",
    ),
    row(
        Quantity::Ayu,
        Measure::Area,
        8,
        120,
        "Muhurta Chintamani XII v. 11 (C1193)",
    ),
    row(
        Quantity::Vyaya,
        Measure::HouseStar,
        1,
        8,
        "Muhurta Chintamani XII v. 7",
    ),
];

/// A series' member by its 1-based count, if the count is in the series.
fn nth<T: Copy>(all: &[T], count: u16) -> Option<T> {
    all.get(usize::from(count).checked_sub(1)?).copied()
}

/// Reckons one row over a measure's value.
fn reckon(row: Row, value: u64) -> Reckoned {
    let product = value * u64::from(row.multiplier);
    let divisor = u64::from(row.divisor);
    let remainder = u16::try_from(product % divisor).unwrap_or(0);
    let count = if remainder == 0 {
        row.divisor
    } else {
        remainder
    };
    let eightfold = matches!(row.quantity, Quantity::Yoni)
        || (matches!(row.quantity, Quantity::Aaya) && row.divisor == 8);
    Reckoned {
        quantity: row.quantity,
        measure: row.measure,
        value,
        multiplier: row.multiplier,
        divisor: row.divisor,
        quotient: product / divisor,
        remainder,
        count,
        yoni: if eightfold {
            nth(&VastuYoni::ALL, count)
        } else {
            None
        },
        nakshatra: if matches!(row.quantity, Quantity::Nakshatra) {
            nth(&Nakshatra::ALL, count)
        } else {
            None
        },
        vara: if matches!(row.quantity, Quantity::Vara) {
            nth(&Vara::ALL, count)
        } else {
            None
        },
        tithi: if matches!(row.quantity, Quantity::Tithi) && row.divisor == 30 {
            nth(&Tithi::ALL, count)
        } else {
            None
        },
        yoga: if matches!(row.quantity, Quantity::Yoga) {
            nth(&Yoga::ALL, count)
        } else {
            None
        },
        source: row.source,
    }
}

/// Reckons a table over a plot, the house's star feeding the rows that
/// start from it.
fn reckon_all(rows: &[Row], plot: Plot, measure: impl Fn(Row) -> Measure) -> Vec<Reckoned> {
    let mut star = 0;
    rows.iter()
        .map(|&row| {
            let row = Row {
                measure: measure(row),
                ..row
            };
            let value = if row.measure == Measure::HouseStar {
                star
            } else {
                row.measure.of(plot)
            };
            let reckoned = reckon(row, value);
            if row.quantity == Quantity::Nakshatra {
                star = u64::from(reckoned.count);
            }
            reckoned
        })
        .collect()
}

/// The banner, the lion, the bull and the elephant.
const GOOD_YONIS: [VastuYoni; 4] = [
    VastuYoni::Dhvaja,
    VastuYoni::Simha,
    VastuYoni::Vrisha,
    VastuYoni::Gaja,
];

/// The Moon's, Mercury's, Jupiter's and Venus's days.
const GOOD_VARAS: [Vara; 4] = [
    Vara::Somavara,
    Vara::Budhavara,
    Vara::Guruvara,
    Vara::Shukravara,
];

/// The clauses a reading judges, read off its quantities.
struct Judge<'a> {
    reckoned: &'a [Reckoned],
    clauses: Vec<AayadiClause>,
}

impl<'a> Judge<'a> {
    fn count(&self, quantity: Quantity) -> u16 {
        self.reckoned
            .iter()
            .find(|reckoned| reckoned.quantity == quantity)
            .map_or(0, |reckoned| reckoned.count)
    }

    fn of(&self, quantity: Quantity) -> Option<&'a Reckoned> {
        self.reckoned
            .iter()
            .find(|reckoned| reckoned.quantity == quantity)
    }

    fn push(&mut self, clause: AayadiClauseKey, holds: bool, source: &'static str) {
        self.clauses.push(AayadiClause {
            clause,
            holds,
            source,
        });
    }

    fn yoni_good(&mut self, quantity: Quantity, source: &'static str) {
        let holds = self
            .of(quantity)
            .and_then(|reckoned| reckoned.yoni)
            .is_some_and(|yoni| GOOD_YONIS.contains(&yoni));
        self.push(AayadiClauseKey::YoniGood, holds, source);
    }

    fn vara_good(&mut self, source: &'static str) {
        let holds = self
            .of(Quantity::Vara)
            .and_then(|reckoned| reckoned.vara)
            .is_some_and(|vara| GOOD_VARAS.contains(&vara));
        self.push(AayadiClauseKey::VaraGood, holds, source);
    }

    fn aaya_exceeds_vyaya(&mut self, source: &'static str) {
        let holds = self.count(Quantity::Aaya) > self.count(Quantity::Vyaya);
        self.push(AayadiClauseKey::AayaExceedsVyaya, holds, source);
    }

    /// The tara from the owner's star to the house's, judged: the 3rd, 5th
    /// and 7th avoided.
    fn tara_not_avoided(
        &mut self,
        owner: Nakshatra,
        house: Nakshatra,
        source: &'static str,
    ) -> Tara {
        let counted = tara(owner, house).tara;
        let avoided = matches!(counted, Tara::Vipat | Tara::Pratyak | Tara::Naidhana);
        self.push(AayadiClauseKey::TaraNotAvoided, !avoided, source);
        counted
    }

    /// *Samarāṅgaṇa* XXVI v. 37's vyaya by its size beside the aaya, and
    /// vv. 48 and 51's clauses on it, the yoni's with them.
    fn samarangana_vyaya(&mut self) -> VyayaKind {
        let aaya = self.count(Quantity::Aaya);
        // v. 36 counts the vyaya from the star; v. 51's "no vyaya" keeps a
        // remainder of 0 as none (C1192).
        let spent = self
            .of(Quantity::Vyaya)
            .map_or(0, |reckoned| reckoned.remainder);
        self.yoni_good(Quantity::Aaya, "Samarangana XXVI vv. 20-23");
        self.push(
            AayadiClauseKey::VyayaLessThanAaya,
            spent < aaya,
            "Samarangana XXVI vv. 37, 48, 51",
        );
        self.push(
            AayadiClauseKey::VyayaPresent,
            spent != 0,
            "Samarangana XXVI v. 51 (C1192)",
        );
        match spent.cmp(&aaya) {
            core::cmp::Ordering::Equal => VyayaKind::Pishacha,
            core::cmp::Ordering::Greater => VyayaKind::Rakshasa,
            core::cmp::Ordering::Less => VyayaKind::Yaksha,
        }
    }

    /// *Samarāṅgaṇa* XXVI vv. 40–48: the owner's tara and gana.
    fn samarangana_owner(&mut self, owner: Nakshatra, house: Nakshatra) -> Tara {
        let counted = self.tara_not_avoided(owner, house, "Samarangana XXVI vv. 40-43");
        let same = owner.attributes().gana == house.attributes().gana;
        self.push(
            AayadiClauseKey::GanaSame,
            same,
            "Samarangana XXVI vv. 44-48",
        );
        counted
    }

    /// *Muhūrta Cintāmaṇi* XII's clauses on the measure alone.
    fn chintamani_measure(&mut self) {
        let odd = self.count(Quantity::Aaya) % 2 == 1;
        self.push(
            AayadiClauseKey::AayaOdd,
            odd,
            "Muhurta Chintamani XII v. 4, Piyushadhara; Vishvakarma Prakasha II v. 52",
        );
        let wealth = self.count(Quantity::Dhana) > self.count(Quantity::Rina);
        self.push(
            AayadiClauseKey::DhanaExceedsRina,
            wealth,
            "Muhurta Chintamani XII vv. 11-12, Mahidhara's gloss",
        );
        self.vara_good("Muhurta Chintamani XII vv. 11-12, Mahidhara's gloss");
        let rikta = matches!(self.count(Quantity::Tithi), 4 | 9 | 14);
        self.push(
            AayadiClauseKey::TithiNotRikta,
            !rikta,
            "Muhurta Chintamani XII vv. 11-12, Mahidhara's gloss",
        );
    }

    /// *Muhūrta Cintāmaṇi* XII v. 12 and the gloss: the owner's own star
    /// and the tara.
    fn chintamani_owner(&mut self, owner: Nakshatra, house: Nakshatra) -> Tara {
        self.push(
            AayadiClauseKey::OwnerStarDiffers,
            owner != house,
            "Muhurta Chintamani XII v. 12",
        );
        self.tara_not_avoided(
            owner,
            house,
            "Muhurta Chintamani XII vv. 11-12, Mahidhara's gloss",
        )
    }
}

/// Reads a plot under a system, with the owner's star where the system
/// judges against it (`03-design/aayadi.md` §3).
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, VastuYoni, VyayaKind};
/// use teistro_muhurta::aayadi::{AayadiClauseKey, AayadiSystem, Plot, Quantity, aayadi};
///
/// // Samarangana XXVI: the area 203 gives the lion (203 over 8 leaves 3),
/// // Rohini (1624 over 27 leaves 4) and a vyaya of 4, greater than the
/// // aaya: a rakshasa vyaya, which v. 51 does not build.
/// let reading = aayadi(AayadiSystem::Samarangana, Plot::new(29, 7)?, Some(Nakshatra::Anuradha));
/// assert_eq!(reading.vyaya, Some(VyayaKind::Rakshasa));
/// assert_eq!(reading.holds(AayadiClauseKey::VyayaLessThanAaya), Some(false));
/// assert_eq!(reading.get(Quantity::Vyaya).map(|vyaya| vyaya.count), Some(4));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn aayadi(system: AayadiSystem, plot: Plot, owner: Option<Nakshatra>) -> Aayadi {
    reading(system, plot, owner, ManasaraMeasure::Split)
}

/// [`aayadi`] under *Mānasāra* with a measure other than the split (crux
/// C1194).
///
/// ```
/// use teistro_muhurta::aayadi::{ManasaraMeasure, Measure, Plot, Quantity, manasara};
///
/// let reading = manasara(Plot::new(29, 7)?, None, ManasaraMeasure::Perimeter);
/// assert!(reading.quantities.iter().all(|reckoned| reckoned.measure == Measure::Perimeter));
/// // The perimeter 72 times 8, over 12, leaves 0: the twelfth aaya, "full".
/// assert_eq!(reading.get(Quantity::Aaya).map(|aaya| (aaya.remainder, aaya.count)), Some((0, 12)));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn manasara(plot: Plot, owner: Option<Nakshatra>, measure: ManasaraMeasure) -> Aayadi {
    reading(AayadiSystem::Manasara, plot, owner, measure)
}

fn reading(
    system: AayadiSystem,
    plot: Plot,
    owner: Option<Nakshatra>,
    measure: ManasaraMeasure,
) -> Aayadi {
    let quantities = match system {
        AayadiSystem::Mayamata => reckon_all(&MAYAMATA, plot, |row| row.measure),
        AayadiSystem::Manasara => reckon_all(&MANASARA, plot, |row| measure.measure(row.quantity)),
        AayadiSystem::Samarangana => reckon_all(&SAMARANGANA, plot, |row| row.measure),
        AayadiSystem::Chintamani => reckon_all(&CHINTAMANI, plot, |row| row.measure),
    };
    let mut judge = Judge {
        reckoned: &quantities,
        clauses: Vec::new(),
    };
    // The owner and the house's star, where both are known.
    let stars = owner.zip(
        judge
            .of(Quantity::Nakshatra)
            .and_then(|reckoned| reckoned.nakshatra),
    );
    let (vyaya, read_tara, unjudged) = match system {
        AayadiSystem::Mayamata => {
            judge.yoni_good(Quantity::Yoni, "Mayamata IX v. 22");
            judge.aaya_exceeds_vyaya("Mayamata IX v. 24");
            (None, None, vec![AayadiUnjudged::OwnerStars])
        }
        AayadiSystem::Manasara => {
            judge.aaya_exceeds_vyaya("Manasara IX vv. 75-77");
            judge.vara_good("Manasara IX vv. 78-81");
            judge.yoni_good(Quantity::Yoni, "Manasara LII vv. 359-360, LXIV v. 73");
            (None, None, vec![AayadiUnjudged::NakshatraAgainstOwner])
        }
        AayadiSystem::Samarangana => {
            let vyaya = judge.samarangana_vyaya();
            let read_tara = stars.map(|(owner, house)| judge.samarangana_owner(owner, house));
            (
                Some(vyaya),
                read_tara,
                vec![
                    AayadiUnjudged::Amsha,
                    AayadiUnjudged::HouseName,
                    AayadiUnjudged::Signs,
                    AayadiUnjudged::Life,
                    AayadiUnjudged::Karanas,
                ],
            )
        }
        AayadiSystem::Chintamani => {
            judge.chintamani_measure();
            let read_tara = stars.map(|(owner, house)| judge.chintamani_owner(owner, house));
            (
                None,
                read_tara,
                vec![
                    AayadiUnjudged::Amsha,
                    AayadiUnjudged::Signs,
                    AayadiUnjudged::NewMoon,
                    AayadiUnjudged::Yogas,
                ],
            )
        }
    };
    let clauses = judge.clauses;
    Aayadi {
        system,
        quantities,
        vyaya,
        tara: read_tara,
        clauses,
        unjudged,
    }
}

/// The area *Muhūrta Cintāmaṇi* XII v. 3 gives for a chosen star and aaya:
/// (152 (star − 1) + 81 (aaya − 1) + 17) mod 216, a larger house adding
/// 216 for each step of `larger`. A remainder of 0 (Revati and the crow)
/// is a house of 216, since no house measures nothing (crux C1192). The
/// area reckons back to the star and the aaya under v. 11 for every pair
/// (crux C1193).
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, VastuYoni};
/// use teistro_muhurta::aayadi::chintamani_area;
///
/// // Mahidhara's example: Rohini and the lion give 203, a larger house 419.
/// assert_eq!(chintamani_area(Nakshatra::Rohini, VastuYoni::Simha, 0), 203);
/// assert_eq!(chintamani_area(Nakshatra::Rohini, VastuYoni::Simha, 1), 419);
/// assert_eq!(chintamani_area(Nakshatra::Revati, VastuYoni::Kaka, 0), 216);
/// ```
#[must_use]
pub fn chintamani_area(star: Nakshatra, aaya: VastuYoni, larger: u32) -> u64 {
    let base = match (152 * u64::from(star.id()) + 81 * u64::from(aaya.id()) + 17) % 216 {
        0 => 216,
        base => base,
    };
    base + 216 * u64::from(larger)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "a test fails by panicking"
    )]

    use super::*;

    fn plot(length: u32, breadth: u32) -> Plot {
        Plot::new(length, breadth).unwrap()
    }

    #[test]
    fn mahidharas_example_reckons_the_lion_rohini_and_a_vyaya_of_four() {
        // Muhurta Chintamani XII v. 3, the gloss (1892, pp. 170-171): an
        // owner in Anuradha, Rohini chosen, the lion: 203, as 29 by 7; v. 7's
        // gloss: Rohini over 8 leaves 4, the vyaya.
        assert_eq!(chintamani_area(Nakshatra::Rohini, VastuYoni::Simha, 0), 203);
        let reading = aayadi(
            AayadiSystem::Chintamani,
            plot(29, 7),
            Some(Nakshatra::Anuradha),
        );
        assert_eq!(reading.yoni(), Some(VastuYoni::Simha));
        assert_eq!(reading.nakshatra(), Some(Nakshatra::Rohini));
        assert_eq!(reading.get(Quantity::Vyaya).unwrap().count, 4);
        assert_eq!(reading.holds(AayadiClauseKey::AayaOdd), Some(true));
        assert_eq!(reading.holds(AayadiClauseKey::OwnerStarDiffers), Some(true));
        // Anuradha to Rohini is the 15th, the 6th tara, Sadhana.
        assert_eq!(reading.tara, Some(Tara::Sadhana));
        assert_eq!(reading.holds(AayadiClauseKey::TaraNotAvoided), Some(true));
    }

    #[test]
    fn verse_three_lands_on_verse_eleven_for_every_star_and_aaya() {
        // The table checks itself: v. 3's inverse reckons back under v. 11.
        for star in Nakshatra::ALL {
            for aaya in VastuYoni::ALL {
                for larger in [0, 3] {
                    let area = chintamani_area(star, aaya, larger);
                    let length = u32::try_from(area).unwrap();
                    let reading = aayadi(AayadiSystem::Chintamani, plot(length, 1), None);
                    assert_eq!(
                        (reading.yoni(), reading.nakshatra()),
                        (Some(aaya), Some(star)),
                        "{area}"
                    );
                }
            }
        }
    }

    #[test]
    fn chintamanis_nine_follow_the_verses_multipliers_and_divisors() {
        let reading = aayadi(AayadiSystem::Chintamani, plot(29, 7), None);
        let table: Vec<(Quantity, u16, u16)> = reading
            .quantities
            .iter()
            .map(|reckoned| (reckoned.quantity, reckoned.multiplier, reckoned.divisor))
            .collect();
        assert_eq!(
            table,
            [
                (Quantity::Aaya, 9, 8),
                (Quantity::Vara, 9, 7),
                (Quantity::Amsha, 6, 9),
                (Quantity::Dhana, 8, 12),
                (Quantity::Rina, 3, 8),
                (Quantity::Nakshatra, 8, 27),
                (Quantity::Tithi, 8, 15),
                (Quantity::Yoga, 4, 27),
                (Quantity::Ayu, 8, 120),
                (Quantity::Vyaya, 1, 8),
            ]
        );
        // 203: × 9 = 1827, over 7 leaves 0 (Saturday); × 6 = 1218 over 9
        // leaves 3; × 8 = 1624 over 12 leaves 4, and × 3 = 609 over 8 leaves
        // 1, so wealth 4 exceeds debt 1; 1624 over 15 leaves 4, a rikta;
        // × 4 = 812 over 27 leaves 2, Priti; 1624 over 120 leaves 64.
        let counts: Vec<u16> = reading
            .quantities
            .iter()
            .map(|reckoned| reckoned.count)
            .collect();
        assert_eq!(counts, [3, 7, 3, 4, 1, 4, 4, 2, 64, 4]);
        assert_eq!(
            reading.get(Quantity::Vara).unwrap().vara,
            Some(Vara::Shanivara)
        );
        assert_eq!(reading.get(Quantity::Yoga).unwrap().yoga, Some(Yoga::Priti));
        assert_eq!(reading.get(Quantity::Tithi).unwrap().tithi, None);
        assert_eq!(reading.holds(AayadiClauseKey::DhanaExceedsRina), Some(true));
        assert_eq!(reading.holds(AayadiClauseKey::VaraGood), Some(false));
        assert_eq!(reading.holds(AayadiClauseKey::TithiNotRikta), Some(false));
        assert_eq!(reading.holds(AayadiClauseKey::OwnerStarDiffers), None);
        assert_eq!(reading.tara, None);
    }

    #[test]
    fn the_owners_own_star_is_death_and_a_seventh_tara_is_avoided() {
        let own = aayadi(
            AayadiSystem::Chintamani,
            plot(29, 7),
            Some(Nakshatra::Rohini),
        );
        assert_eq!(own.holds(AayadiClauseKey::OwnerStarDiffers), Some(false));
        // Bharani to Rohini is the 3rd, Vipat.
        let third = aayadi(
            AayadiSystem::Chintamani,
            plot(29, 7),
            Some(Nakshatra::Bharani),
        );
        assert_eq!(
            (third.tara, third.holds(AayadiClauseKey::TaraNotAvoided)),
            (Some(Tara::Vipat), Some(false))
        );
    }

    #[test]
    fn mayamata_reckons_the_sum_and_the_perimeter() {
        // 29 + 7 = 36: × 8 = 288 over 12 leaves 0 (the twelfth, full); × 9
        // = 324 over 10 leaves 4; × 3 = 108 over 8 leaves 4, the dog;
        // 288 over 27 is 10 (the age) and leaves 18, Jyeshtha; the
        // perimeter 72 over 30 leaves 12, over 7 leaves 2, Monday.
        let reading = aayadi(AayadiSystem::Mayamata, plot(29, 7), None);
        let rows: Vec<(Quantity, Measure, u64, u16, u64)> = reading
            .quantities
            .iter()
            .map(|reckoned| {
                (
                    reckoned.quantity,
                    reckoned.measure,
                    reckoned.value,
                    reckoned.count,
                    reckoned.quotient,
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                (Quantity::Aaya, Measure::Sum, 36, 12, 24),
                (Quantity::Vyaya, Measure::Sum, 36, 4, 32),
                (Quantity::Yoni, Measure::Sum, 36, 4, 13),
                (Quantity::Nakshatra, Measure::Sum, 36, 18, 10),
                (Quantity::Tithi, Measure::Perimeter, 72, 12, 2),
                (Quantity::Vara, Measure::Perimeter, 72, 2, 10),
            ]
        );
        assert_eq!(reading.yoni(), Some(VastuYoni::Shvan));
        assert_eq!(reading.nakshatra(), Some(Nakshatra::Jyeshtha));
        assert_eq!(
            reading.get(Quantity::Tithi).unwrap().tithi,
            Some(Tithi::ShuklaDwadashi)
        );
        assert_eq!(
            reading.get(Quantity::Vara).unwrap().vara,
            Some(Vara::Somavara)
        );
        assert_eq!(reading.holds(AayadiClauseKey::YoniGood), Some(false));
        assert_eq!(reading.holds(AayadiClauseKey::AayaExceedsVyaya), Some(true));
        assert_eq!(reading.unjudged, [AayadiUnjudged::OwnerStars]);
        assert_eq!((reading.vyaya, reading.tara), (None, None));
    }

    #[test]
    fn manasara_splits_its_six_over_three_measures_by_default() {
        // 29 by 7: the length 29 × 8 = 232 over 12 leaves 4 and over 27
        // leaves 16, Vishakha; the breadth 7 × 9 = 63 over 10 leaves 3 and
        // × 3 = 21 over 8 leaves 5, the bull; the perimeter 72 × 9 = 648
        // over 7 leaves 4, Wednesday, and over 30 leaves 18.
        let reading = aayadi(AayadiSystem::Manasara, plot(29, 7), None);
        let rows: Vec<(Quantity, Measure, u16)> = reading
            .quantities
            .iter()
            .map(|reckoned| (reckoned.quantity, reckoned.measure, reckoned.count))
            .collect();
        assert_eq!(
            rows,
            [
                (Quantity::Aaya, Measure::Length, 4),
                (Quantity::Nakshatra, Measure::Length, 16),
                (Quantity::Vyaya, Measure::Breadth, 3),
                (Quantity::Yoni, Measure::Breadth, 5),
                (Quantity::Vara, Measure::Perimeter, 4),
                (Quantity::Tithi, Measure::Perimeter, 18),
            ]
        );
        assert_eq!(reading.nakshatra(), Some(Nakshatra::Vishakha));
        assert_eq!(
            reading.get(Quantity::Tithi).unwrap().tithi,
            Some(Tithi::KrishnaTritiya)
        );
        assert_eq!(reading.holds(AayadiClauseKey::AayaExceedsVyaya), Some(true));
        assert_eq!(reading.holds(AayadiClauseKey::VaraGood), Some(true));
        assert_eq!(reading.holds(AayadiClauseKey::YoniGood), Some(true));
        for measure in [
            ManasaraMeasure::Sum,
            ManasaraMeasure::Length,
            ManasaraMeasure::Breadth,
        ] {
            let one = manasara(plot(29, 7), None, measure);
            let first = one.quantities[0].measure;
            assert!(
                one.quantities
                    .iter()
                    .all(|reckoned| reckoned.measure == first),
                "{measure:?}"
            );
        }
        assert_eq!(manasara(plot(29, 7), None, ManasaraMeasure::Split), reading);
    }

    #[test]
    fn samarangana_names_each_vyaya_by_its_size_beside_the_aaya() {
        // 29 by 7 = 203: the lion (3), Rohini (4), vyaya 4: greater, rakshasa.
        let greater = aayadi(AayadiSystem::Samarangana, plot(29, 7), None);
        assert_eq!(greater.vyaya, Some(VyayaKind::Rakshasa));
        assert_eq!(greater.holds(AayadiClauseKey::YoniGood), Some(true));
        assert_eq!(greater.holds(AayadiClauseKey::VyayaPresent), Some(true));
        assert_eq!(greater.holds(AayadiClauseKey::TaraNotAvoided), None);
        // 1 by 1: the banner (1), × 8 over 27 leaves 8, Pushya, vyaya 0:
        // no vyaya, which v. 51 forbids, and less than the aaya.
        let none = aayadi(AayadiSystem::Samarangana, plot(1, 1), None);
        assert_eq!(none.nakshatra(), Some(Nakshatra::Pushya));
        assert_eq!(
            (none.get(Quantity::Vyaya).unwrap().remainder, none.vyaya),
            (0, Some(VyayaKind::Yaksha))
        );
        assert_eq!(none.holds(AayadiClauseKey::VyayaPresent), Some(false));
        assert_eq!(none.holds(AayadiClauseKey::VyayaLessThanAaya), Some(true));
        // 5 by 1 = 5: the bull (5), 40 over 27 leaves 13, Hasta, vyaya 5:
        // equal, pishacha.
        let equal = aayadi(AayadiSystem::Samarangana, plot(5, 1), None);
        assert_eq!(
            (equal.yoni(), equal.nakshatra()),
            (Some(VastuYoni::Vrisha), Some(Nakshatra::Hasta))
        );
        assert_eq!(equal.vyaya, Some(VyayaKind::Pishacha));
        // 8 by 1: the crow on a remainder of 0 (C1192), its death.
        let crow = aayadi(AayadiSystem::Samarangana, plot(8, 1), None);
        assert_eq!(
            (crow.yoni(), crow.holds(AayadiClauseKey::YoniGood)),
            (Some(VastuYoni::Kaka), Some(false))
        );
    }

    #[test]
    fn samarangana_reads_the_owners_tara_and_gana() {
        // 203's Rohini, a human star: from Anuradha (a god's) the 15th,
        // Sadhana; from Bharani (a human's) the 3rd, Vipat.
        let gods = aayadi(
            AayadiSystem::Samarangana,
            plot(29, 7),
            Some(Nakshatra::Anuradha),
        );
        assert_eq!(gods.tara, Some(Tara::Sadhana));
        assert_eq!(gods.holds(AayadiClauseKey::TaraNotAvoided), Some(true));
        assert_eq!(gods.holds(AayadiClauseKey::GanaSame), Some(false));
        let human = aayadi(
            AayadiSystem::Samarangana,
            plot(29, 7),
            Some(Nakshatra::Bharani),
        );
        assert_eq!(human.holds(AayadiClauseKey::TaraNotAvoided), Some(false));
        assert_eq!(human.holds(AayadiClauseKey::GanaSame), Some(true));
        assert_eq!(human.unjudged.len(), 5);
    }

    #[test]
    fn the_longest_sides_reckon_without_overflow() {
        let reading = aayadi(
            AayadiSystem::Chintamani,
            plot(LONGEST_SIDE, LONGEST_SIDE),
            None,
        );
        assert_eq!(
            reading.get(Quantity::Aaya).unwrap().value,
            1_000_000_000_000
        );
        assert!(Plot::new(LONGEST_SIDE + 1, 1).is_err());
        assert_eq!(Measure::HouseStar.of(plot(3, 4)), 0);
        assert_eq!(Measure::Length.of(plot(3, 4)), 3);
        assert_eq!(Measure::Breadth.of(plot(3, 4)), 4);
    }

    #[test]
    fn every_system_answers_every_row_of_its_table() {
        for system in AayadiSystem::ALL {
            let reading = aayadi(system, plot(12, 9), Some(Nakshatra::Ashwini));
            assert_eq!(reading.system, system);
            assert!(
                reading
                    .quantities
                    .iter()
                    .all(|reckoned| reckoned.count >= 1 && reckoned.count <= reckoned.divisor)
            );
            assert_ne!(reading.clauses.len(), 0);
            assert_ne!(reading.unjudged.len(), 0);
        }
    }
}

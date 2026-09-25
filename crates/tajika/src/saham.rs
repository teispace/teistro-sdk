//! The **sahams**: sensitive points, each the arithmetic of three others,
//! each ruling one matter of a year (`03-design/tajika-sahams.md`).
//!
//! Every saham is one shape. Call its three factors *a*, *b* and *c*, as
//! the source does: the saham is **a − b + c**, and if *c* does not fall
//! between *b* and *a*, counted in the order of the signs from *b*, it is
//! carried **one sign further**. What differs between two sahams is only
//! which three points they read, and whether a year that begins at night
//! reads them in another order.
//!
//! So a saham is **data** here — a [`SahamFormula`] over a closed set of
//! [`SahamTerm`]s — and the forty-one the source gives are a table of them.
//! A caller with a saham of its own, or another authority's reading of
//! one of these, writes a formula and hands it to the same evaluator:
//! nothing about the source's forty-one is privileged but their names.
//!
//! Three things the tradition reads more than one way are
//! [`SahamRules`] fields, each defaulting to the source:
//!
//! - **when the sign is added** ([`AddSign`]): by degrees, which every
//!   worked example the source prints confirms, or by whole signs, which
//!   a widely used program applies and one of those examples refutes;
//! - **where a house stands** ([`HousePoints`]): Sripati's mid-points,
//!   which the source builds from the lagna and the midheaven, the
//!   chart's own chalit, or equal houses from the lagna;
//! - **the Roga saham** ([`RogaReading`]), which the source gives twice
//!   and says which of the two it has found better.

use core::cell::Cell;

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::bala::{AnnualSky, finite_longitude, sign_of_longitude};

/// One of the sahams the source gives, in its own order: the id of each
/// is its number there less one.
///
/// Three pairs share a formula and are still two sahams — the source
/// names each for a different matter, and a reader asks for the matter.
/// [`Saham::Vidya`] is [`Saham::Guru`]'s, [`Saham::Raja`] is
/// [`Saham::Pitri`]'s and [`Saham::Kshama`] is [`Saham::Kali`]'s.
///
/// The source: K. S. Charak, *A Textbook of Varshaphala*, ch. XI, "based
/// mainly on the Tajika Neelakanthi" — which describes fifty, and
/// Venkatesha forty-eight, and Keshava twenty-five. These are the ones
/// that book gives; one it does not is a [`SahamFormula`] away.
///
/// Serialised as its catalogue key in kebab case (`karya-siddhi`), the
/// key every binding reads a saham back as, so a caller names one in a
/// request exactly as an answer spelt it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Saham {
    /// No. 1, **Punya**, general auspiciousness: Moon − Sun + lagna by day,
    /// reversed by night. The one the rest defer to.
    Punya,
    /// No. 2, **Guru**, the preceptor: Punya reversed.
    Guru,
    /// No. 3, **Vidya** (Jnana), knowledge: Guru's formula, "because the
    /// preceptor and knowledge go hand in hand".
    Vidya,
    /// No. 4, **Yasha**, fame: Jupiter − Punya + lagna.
    Yasha,
    /// No. 5, **Mitra**, friends: Guru − Punya + Venus.
    Mitra,
    /// No. 6, **Mahatmya**, the fruits of virtuous living: Punya − Mars +
    /// lagna.
    Mahatmya,
    /// No. 7, **Asha**, hope: Saturn − Venus + lagna.
    Asha,
    /// No. 8, **Samarthya**, capability: Mars − the lagna's lord + lagna.
    Samarthya,
    /// No. 9, **Bhratri**, siblings: Jupiter − Saturn + lagna, day and night.
    Bhratri,
    /// No. 10, **Gaurava**, dignity: Sun − Moon + Jupiter.
    Gaurava,
    /// No. 11, **Pitri** (Taata), the father: Saturn − Sun + lagna.
    Pitri,
    /// No. 12, **Raja**, royal dignity: Pitri's formula.
    Raja,
    /// No. 13, **Matri**, the mother: Moon − Venus + lagna.
    Matri,
    /// No. 14, **Putra**, progeny: Jupiter − Moon + lagna, day and night.
    Putra,
    /// No. 15, **Jeeva**, life: Saturn − Jupiter + lagna.
    Jeeva,
    /// No. 16, **Roga**, disease: lagna − Moon + lagna, or Saturn − Moon +
    /// lagna under [`RogaReading::Saturn`].
    Roga,
    /// No. 17, **Karma**, profession: Mars − Mercury + lagna.
    Karma,
    /// No. 18, **Manmatha**, infatuation: Moon − the lagna's lord + lagna.
    Manmatha,
    /// No. 19, **Kali**, strife: Jupiter − Mars + lagna.
    Kali,
    /// No. 20, **Kshama**, forgiveness: Kali's formula.
    Kshama,
    /// No. 21, **Shastra**, scriptures: Jupiter − Saturn + Mercury.
    Shastra,
    /// No. 22, **Bandhu**, relatives: Mercury − Moon + lagna, day and night.
    Bandhu,
    /// No. 23, **Mrityu**, death: the eighth house − Moon + Saturn, day and
    /// night.
    Mrityu,
    /// No. 24, **Deshantara**, foreign travel: the ninth house − its lord +
    /// lagna, day and night.
    Deshantara,
    /// No. 25, **Artha** (Dhana), wealth: the second house − its lord +
    /// lagna, day and night.
    Artha,
    /// No. 26, **Paradara**, adultery: Venus − Sun + lagna, day and night.
    Paradara,
    /// No. 27, **Anya-karma**, an additional vocation: Moon − Saturn + lagna.
    AnyaKarma,
    /// No. 28, **Vanika**, trade: Moon − Mercury + lagna, day and night.
    Vanika,
    /// No. 29, **Karya-siddhi**, success in a venture: Saturn − Sun + the
    /// lord of the Sun's sign by day; Saturn − Moon + the lord of the
    /// Moon's sign by night. The one saham whose night is not its day
    /// reversed.
    KaryaSiddhi,
    /// No. 30, **Vivaha**, marriage: Venus − Saturn + lagna, day and night.
    Vivaha,
    /// No. 31, **Prasava**, the delivery of a child: Jupiter − Mercury +
    /// lagna.
    Prasava,
    /// No. 32, **Santaapa**, sorrow: Saturn − Moon + the sixth house, day
    /// and night.
    Santaapa,
    /// No. 33, **Shraddha**, devotion: Venus − Mars + lagna, day and night.
    Shraddha,
    /// No. 34, **Preeti**, love: Vidya − Punya + lagna, day and night.
    Preeti,
    /// No. 35, **Jadya**, stupidity: Mars − Saturn + Mercury.
    Jadya,
    /// No. 36, **Vyapara**, business: Mars − Mercury + lagna, day and night.
    /// "Somewhat equivalent" to Vanika in meaning, and not in formula.
    Vyapara,
    /// No. 37, **Paneeya-paata**, falling into water: Saturn − Moon + lagna.
    PaneeyaPaata,
    /// No. 38, **Shatru**, enemies: Mars − Saturn + lagna.
    Shatru,
    /// No. 39, **Jalapatha**, a sea voyage: Cancer 15° − Saturn + lagna.
    Jalapatha,
    /// No. 40, **Bandhana**, imprisonment: Punya − Saturn + lagna.
    Bandhana,
    /// No. 41, **Labha**, monetary gain: the eleventh house − its lord +
    /// lagna, day and night.
    Labha,
}

/// Cancer 15°, Jalapatha's fixed first factor.
pub const CANCER_15_DEG: f64 = 105.0;

impl Saham {
    /// Every saham the source gives, in its order.
    pub const ALL: [Saham; 41] = [
        Saham::Punya,
        Saham::Guru,
        Saham::Vidya,
        Saham::Yasha,
        Saham::Mitra,
        Saham::Mahatmya,
        Saham::Asha,
        Saham::Samarthya,
        Saham::Bhratri,
        Saham::Gaurava,
        Saham::Pitri,
        Saham::Raja,
        Saham::Matri,
        Saham::Putra,
        Saham::Jeeva,
        Saham::Roga,
        Saham::Karma,
        Saham::Manmatha,
        Saham::Kali,
        Saham::Kshama,
        Saham::Shastra,
        Saham::Bandhu,
        Saham::Mrityu,
        Saham::Deshantara,
        Saham::Artha,
        Saham::Paradara,
        Saham::AnyaKarma,
        Saham::Vanika,
        Saham::KaryaSiddhi,
        Saham::Vivaha,
        Saham::Prasava,
        Saham::Santaapa,
        Saham::Shraddha,
        Saham::Preeti,
        Saham::Jadya,
        Saham::Vyapara,
        Saham::PaneeyaPaata,
        Saham::Shatru,
        Saham::Jalapatha,
        Saham::Bandhana,
        Saham::Labha,
    ];

    /// Its place in [`Saham::ALL`]: the source's number less one.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Whether this is one of the four the source says are best **weak**,
    /// "according to some": Shatru, Roga, Kali and Mrityu — the sahams of
    /// enemies, disease, strife and death, "considered best when in
    /// debility".
    #[must_use]
    pub const fn best_weak(self) -> bool {
        matches!(
            self,
            Saham::Shatru | Saham::Roga | Saham::Kali | Saham::Mrityu
        )
    }

    /// The formula the source gives it, under the rules' reading of
    /// Roga.
    ///
    /// ```
    /// use teistro_tajika::{SahamFormula, Saham, SahamRules, SahamTerm};
    /// use teistro_core::catalogue::Graha;
    ///
    /// let punya = Saham::Punya.formula(SahamRules::default());
    /// assert_eq!(
    ///     punya,
    ///     SahamFormula::reversed_by_night(
    ///         SahamTerm::Graha(Graha::Moon),
    ///         SahamTerm::Graha(Graha::Sun),
    ///         SahamTerm::Lagna,
    ///     ),
    /// );
    /// ```
    #[must_use]
    pub fn formula(self, rules: SahamRules) -> SahamFormula {
        use Graha::{Jupiter, Mars, Mercury, Moon, Saturn, Sun, Venus};
        use SahamTerm::Lagna;
        let g = SahamTerm::Graha;
        let [
            first,
            second,
            _,
            _,
            _,
            sixth,
            _,
            eighth,
            ninth,
            _,
            eleventh,
            _,
        ] = House::ALL;
        let flip = SahamFormula::reversed_by_night;
        let same = SahamFormula::same;
        match self {
            Saham::Punya => flip(g(Moon), g(Sun), Lagna),
            Saham::Guru | Saham::Vidya => flip(g(Sun), g(Moon), Lagna),
            Saham::Yasha => flip(g(Jupiter), SahamTerm::Saham(Saham::Punya), Lagna),
            Saham::Mitra => flip(
                SahamTerm::Saham(Saham::Guru),
                SahamTerm::Saham(Saham::Punya),
                g(Venus),
            ),
            Saham::Mahatmya => flip(SahamTerm::Saham(Saham::Punya), g(Mars), Lagna),
            Saham::Asha => flip(g(Saturn), g(Venus), Lagna),
            Saham::Samarthya => flip(g(Mars), SahamTerm::HouseLord(first), Lagna),
            Saham::Bhratri => same(g(Jupiter), g(Saturn), Lagna),
            Saham::Gaurava => flip(g(Sun), g(Moon), g(Jupiter)),
            Saham::Pitri | Saham::Raja => flip(g(Saturn), g(Sun), Lagna),
            Saham::Matri => flip(g(Moon), g(Venus), Lagna),
            Saham::Putra => same(g(Jupiter), g(Moon), Lagna),
            Saham::Jeeva => flip(g(Saturn), g(Jupiter), Lagna),
            Saham::Roga => match rules.roga {
                RogaReading::Lagna => same(Lagna, g(Moon), Lagna),
                RogaReading::Saturn => flip(g(Saturn), g(Moon), Lagna),
            },
            Saham::Karma => flip(g(Mars), g(Mercury), Lagna),
            Saham::Manmatha => flip(g(Moon), SahamTerm::HouseLord(first), Lagna),
            Saham::Kali | Saham::Kshama => flip(g(Jupiter), g(Mars), Lagna),
            Saham::Shastra => flip(g(Jupiter), g(Saturn), g(Mercury)),
            Saham::Bandhu => same(g(Mercury), g(Moon), Lagna),
            Saham::Mrityu => same(SahamTerm::House(eighth), g(Moon), g(Saturn)),
            Saham::Deshantara => same(SahamTerm::House(ninth), SahamTerm::HouseLord(ninth), Lagna),
            Saham::Artha => same(
                SahamTerm::House(second),
                SahamTerm::HouseLord(second),
                Lagna,
            ),
            Saham::Paradara => same(g(Venus), g(Sun), Lagna),
            Saham::AnyaKarma => flip(g(Moon), g(Saturn), Lagna),
            Saham::Vanika => same(g(Moon), g(Mercury), Lagna),
            Saham::KaryaSiddhi => SahamFormula {
                day: SahamTriple::new(g(Saturn), g(Sun), SahamTerm::SignLordOf(Sun)),
                night: SahamTriple::new(g(Saturn), g(Moon), SahamTerm::SignLordOf(Moon)),
            },
            Saham::Vivaha => same(g(Venus), g(Saturn), Lagna),
            Saham::Prasava => flip(g(Jupiter), g(Mercury), Lagna),
            Saham::Santaapa => same(g(Saturn), g(Moon), SahamTerm::House(sixth)),
            Saham::Shraddha => same(g(Venus), g(Mars), Lagna),
            Saham::Preeti => same(
                SahamTerm::Saham(Saham::Vidya),
                SahamTerm::Saham(Saham::Punya),
                Lagna,
            ),
            Saham::Jadya => flip(g(Mars), g(Saturn), g(Mercury)),
            Saham::Vyapara => same(g(Mars), g(Mercury), Lagna),
            Saham::PaneeyaPaata => flip(g(Saturn), g(Moon), Lagna),
            Saham::Shatru => flip(g(Mars), g(Saturn), Lagna),
            Saham::Jalapatha => flip(SahamTerm::Degrees(CANCER_15_DEG), g(Saturn), Lagna),
            Saham::Bandhana => flip(SahamTerm::Saham(Saham::Punya), g(Saturn), Lagna),
            Saham::Labha => same(
                SahamTerm::House(eleventh),
                SahamTerm::HouseLord(eleventh),
                Lagna,
            ),
        }
    }
}

/// One factor of a saham: a point the chart supplies.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SahamTerm {
    /// The lagna — "the mid-point of the ascendant".
    Lagna,
    /// One of the seven. The nodes are refused: the source's sahams
    /// never read them, and the sky a saham is read from holds seven.
    Graha(Graha),
    /// A house's point, as [`HousePoints`] reads it.
    House(House),
    /// Where the lord of a house stands: the lord of the sign its point
    /// falls in.
    HouseLord(House),
    /// Where the lord of the sign a planet stands in stands —
    /// Karya-siddhi's third factor.
    SignLordOf(Graha),
    /// Another of the source's sahams, as the rules read it.
    Saham(Saham),
    /// A fixed longitude, degrees: Jalapatha's Cancer 15°.
    Degrees(f64),
}

/// A saham's three factors, read as **a − b + c**.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SahamTriple {
    /// The point counted **to**.
    pub a: SahamTerm,
    /// The point counted **from**, subtracted.
    pub b: SahamTerm,
    /// The point the distance is added to: whether it falls between
    /// `b` and `a` decides whether the sign is added.
    pub c: SahamTerm,
}

impl SahamTriple {
    /// `a − b + c`.
    #[must_use]
    pub const fn new(a: SahamTerm, b: SahamTerm, c: SahamTerm) -> SahamTriple {
        SahamTriple { a, b, c }
    }
}

/// How a saham is computed, by day and by night.
///
/// ```
/// use teistro_tajika::{SahamFormula, SahamTerm};
/// use teistro_core::catalogue::Graha;
/// use teistro_core::house::House;
///
/// // A saham another authority gives and the source does not: the
/// // eighth house − Mars + lagna, reversed by night.
/// let apamrityu = SahamFormula::reversed_by_night(
///     SahamTerm::House(House::try_new(8)?),
///     SahamTerm::Graha(Graha::Mars),
///     SahamTerm::Lagna,
/// );
/// assert_eq!(apamrityu.night.a, SahamTerm::Graha(Graha::Mars));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SahamFormula {
    /// For a chart cast between sunrise and sunset.
    pub day: SahamTriple,
    /// For a chart cast between sunset and sunrise.
    pub night: SahamTriple,
}

impl SahamFormula {
    /// The same three factors by day and by night.
    #[must_use]
    pub const fn same(a: SahamTerm, b: SahamTerm, c: SahamTerm) -> SahamFormula {
        let triple = SahamTriple::new(a, b, c);
        SahamFormula {
            day: triple,
            night: triple,
        }
    }

    /// `a − b + c` by day and `b − a + c` by night: the shape most of
    /// the source's sahams take.
    #[must_use]
    pub const fn reversed_by_night(a: SahamTerm, b: SahamTerm, c: SahamTerm) -> SahamFormula {
        SahamFormula {
            day: SahamTriple::new(a, b, c),
            night: SahamTriple::new(b, a, c),
        }
    }

    /// The triple a chart reads.
    #[must_use]
    pub const fn at(&self, by_day: bool) -> SahamTriple {
        if by_day { self.day } else { self.night }
    }
}

/// How a saham is read where the tradition divides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct SahamRules {
    /// When a saham is carried one sign further.
    pub add_sign: AddSign,
    /// Where a house's point stands.
    pub houses: HousePoints,
    /// Which of the source's two Roga sahams [`Saham::Roga`] is.
    pub roga: RogaReading,
}

/// When a saham is carried one sign further: the source's "essential
/// consideration".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AddSign {
    /// **When c is not between b and a by degrees**, counted in the
    /// order of the signs from b: the default. All seven worked sahams
    /// the source prints agree with it, among them a birth chart's
    /// Punya where the Sun, the lagna and the Moon all stand in Leo,
    /// 3°49′, 14°33′ and 17°08′ — between by degrees and not by signs,
    /// and printed without the added sign.
    #[default]
    Degrees,
    /// **When c's sign is not after b's and up to a's**, counted in
    /// whole signs, with a and b in one sign counting as the whole
    /// circle. What a widely used program applies (measured, not read:
    /// `03-design/tajika-sahams.md`); the source's Leo birth chart
    /// refutes it, and it is here so that a caller can reproduce what
    /// that program prints.
    Signs,
    /// **Never**: a − b + c and nothing more, as the Arabic parts of
    /// other traditions are read.
    Never,
}

/// Where a house's point stands, for the sahams that read one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HousePoints {
    /// **Sripati's mid-points**, the default, and the source's own recipe:
    /// the lagna, the midheaven and their opposites are the mid-points of
    /// the four kendras, and each quadrant between is trisected. Built
    /// from the chart's lagna and midheaven, so it is the source's
    /// whatever chalit a profile gives the chart. It reproduces every
    /// mid-point the source prints for its Example Chart.
    #[default]
    Sripati,
    /// **The chart's own chalit middles**, under whatever division its
    /// profile names — Sripati's too under most, equal houses centred on
    /// the lagna under a profile whose chalit is Vehlow's.
    Chalit,
    /// **Equal houses from the lagna**: the house *n* stands (n − 1) × 30°
    /// past the lagna's degree — for a caller with no midheaven, and what
    /// a program reading only planets and a lagna can do. The source's
    /// Chart X-9 prints an eighth mid-point, Capricorn 4°12′ under a lagna
    /// of Gemini 7°15′, that no equal division gives.
    Equal,
}

/// Which of the source's two Roga sahams is meant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RogaReading {
    /// **Lagna − Moon + lagna**, day and night: the formula the source
    /// gives as the saham, and the default.
    #[default]
    Lagna,
    /// **Saturn − Moon + lagna**, reversed by night: "according to
    /// another authority", and the one the source says it has "found
    /// giving better results".
    Saturn,
}

/// The chart a saham is read from.
///
/// Any chart: the source reads the sahams of the annual chart **and** of
/// the birth chart, since one weak at birth "cannot produce results"
/// in a year however strong it stands there.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SahamSky {
    /// Where the seven stand.
    pub sky: AnnualSky,
    /// The lagna, degrees.
    pub lagna_deg: f64,
    /// The midheaven, degrees: what Sripati's mid-points are built from.
    pub midheaven_deg: f64,
    /// The chart's own chalit middles, the first house first, degrees:
    /// read only under [`HousePoints::Chalit`], which refuses a chart
    /// without them.
    pub chalit_deg: Option<[f64; 12]>,
    /// Whether the chart was cast between sunrise and sunset.
    pub by_day: bool,
    /// Rahu, degrees, when the chart places the nodes: what a saham's
    /// strength reads the Rahu-Ketu axis from, and nothing else.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rahu_deg: Option<f64>,
}

impl SahamSky {
    /// A chart from its seven, its two angles and its day or night; its
    /// own chalit middles are [`SahamSky::with_chalit`]'s.
    #[must_use]
    pub const fn new(sky: AnnualSky, lagna_deg: f64, midheaven_deg: f64, by_day: bool) -> SahamSky {
        SahamSky {
            sky,
            lagna_deg,
            midheaven_deg,
            chalit_deg: None,
            by_day,
            rahu_deg: None,
        }
    }

    /// The same chart, carrying its own chalit middles for
    /// [`HousePoints::Chalit`].
    #[must_use]
    pub const fn with_chalit(mut self, chalit_deg: [f64; 12]) -> SahamSky {
        self.chalit_deg = Some(chalit_deg);
        self
    }

    /// The same chart, carrying Rahu for a saham's strength.
    #[must_use]
    pub const fn with_rahu(mut self, rahu_deg: f64) -> SahamSky {
        self.rahu_deg = Some(rahu_deg);
        self
    }

    pub(crate) fn check(&self) -> Result<(), Error> {
        self.sky.check()?;
        if let Some(rahu) = self.rahu_deg {
            finite_longitude("rahu_deg", rahu)?;
        }
        for (field, value) in [
            ("lagna_deg", self.lagna_deg),
            ("midheaven_deg", self.midheaven_deg),
        ] {
            finite_longitude(field, value)?;
        }
        if let Some(house) = self
            .chalit_deg
            .and_then(|houses| houses.iter().position(|deg| !deg.is_finite()))
        {
            return Err(Error::invalid_arg(format!(
                "house {} has a middle that is not a number",
                house + 1
            ))
            .with_field(format!("chalit_deg[{house}]")));
        }
        Ok(())
    }

    /// The twelve house points a reading asks for.
    fn houses(&self, points: HousePoints) -> Result<[f64; 12], Error> {
        match points {
            HousePoints::Sripati => Ok(sripati_mid_points(self.lagna_deg, self.midheaven_deg)),
            HousePoints::Chalit => self.chalit_deg.ok_or_else(|| {
                Error::invalid_arg("the chart's own chalit was asked for and not given")
                    .with_field("chalit_deg")
            }),
            HousePoints::Equal => Ok(equal_houses(self.lagna_deg)),
        }
    }
}

/// Sripati's mid-points, as the source builds them: the lagna and the
/// midheaven and their opposites mid-point the kendras, and the quadrant
/// from the midheaven to the lagna, and from the lagna to the nadir, is
/// each cut in three, the houses opposite following by six signs.
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

fn equal_houses(lagna_deg: f64) -> [f64; 12] {
    let mut houses = [0.0; 12];
    for (offset, house) in (0_u32..).zip(houses.iter_mut()) {
        *house = (lagna_deg + 30.0 * f64::from(offset)).rem_euclid(360.0);
    }
    houses
}

/// Where a saham fell, and what the source reads off it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SahamPlace {
    /// The saham, degrees in the chart's zodiac.
    pub longitude_deg: f64,
    /// The sign it falls in.
    pub sign: Rashi,
    /// That sign's lord: the **saham lord**, whose strength the source
    /// judges the saham by.
    pub lord: Graha,
    /// Its house counted in whole signs from the lagna, as the source
    /// says a saham "falls in the tenth house".
    pub house: House,
    /// Whether it was carried one sign further, because c was not
    /// between b and a.
    pub added_sign: bool,
}

/// One of the source's sahams, where it fell.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SahamPoint {
    /// Which.
    pub saham: Saham,
    /// Where it fell.
    pub point: SahamPlace,
}

/// The sahams a chart was asked for, and how it was read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SahamReading {
    /// Whether the chart was read by its day formulas.
    pub by_day: bool,
    /// The rules it was read under.
    pub rules: SahamRules,
    /// One per saham asked, in the order asked.
    pub points: Vec<SahamPoint>,
}

/// The sahams asked for, in a chart.
///
/// Each is computed once however many ask for it: Yasha, Mitra,
/// Mahatmya, Preeti and Bandhana each read Punya, and a caller asking
/// for all forty-one computes it once.
///
/// ```
/// use teistro_tajika::{AnnualSky, Saham, SahamRules, SahamSky, sahams};
///
/// // The source's Example Chart, its forty-first year, cast by day.
/// let sky = AnnualSky {
///     sun_deg: 123.0 + 50.0 / 60.0,
///     moon_deg: 39.0 + 40.0 / 60.0,
///     mars_deg: 217.0 + 42.0 / 60.0,
///     mercury_deg: 138.0 + 20.0 / 60.0,
///     jupiter_deg: 249.0 + 38.0 / 60.0,
///     venus_deg: 141.0 + 45.0 / 60.0,
///     saturn_deg: 197.0 + 13.0 / 60.0,
/// };
/// // Its lagna Scorpio 9°26′, its midheaven Leo 13°10′.
/// let chart = SahamSky::new(sky, 219.0 + 26.0 / 60.0, 133.0 + 10.0 / 60.0, true);
/// let read = sahams(&chart, &[Saham::Punya, Saham::Raja], SahamRules::default())?;
///
/// // Punya: Leo 15°16′, the lagna between the Sun and the Moon.
/// let punya = read.points[0].point;
/// assert_eq!(punya.longitude_deg.floor(), 135.0);
/// assert!(!punya.added_sign);
/// // Raja: Aquarius 22°49′, one sign added.
/// let raja = read.points[1].point;
/// assert_eq!(raja.longitude_deg.floor(), 322.0);
/// assert!(raja.added_sign);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a longitude that is not a number, named by its
/// field.
pub fn sahams(sky: &SahamSky, which: &[Saham], rules: SahamRules) -> Result<SahamReading, Error> {
    let reading = Evaluation::of(sky, rules)?;
    let points = which
        .iter()
        .map(|&saham| SahamPoint {
            saham,
            point: reading.saham(saham),
        })
        .collect();
    Ok(SahamReading {
        by_day: sky.by_day,
        rules,
        points,
    })
}

/// Where a saham of the caller's own falls: any formula over the same
/// terms, read under the same rules.
///
/// ```
/// use teistro_tajika::{AnnualSky, SahamFormula, SahamRules, SahamSky, SahamTerm, saham_point};
/// use teistro_core::catalogue::Graha;
///
/// let sky = AnnualSky {
///     sun_deg: 10.0, moon_deg: 100.0, mars_deg: 200.0, mercury_deg: 20.0,
///     jupiter_deg: 250.0, venus_deg: 40.0, saturn_deg: 300.0,
/// };
/// let chart = SahamSky::new(sky, 50.0, 320.0, true);
/// let own = SahamFormula::same(SahamTerm::Graha(Graha::Moon), SahamTerm::Graha(Graha::Sun), SahamTerm::Lagna);
/// // 100 − 10 + 50 = 140, and the lagna is between the Sun and the Moon.
/// assert_eq!(saham_point(&chart, &own, SahamRules::default())?.longitude_deg, 140.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a longitude that is not a number, a node as a
/// factor, or a fixed degree that is not a number — each named by the
/// factor it stands in.
pub fn saham_point(
    sky: &SahamSky,
    formula: &SahamFormula,
    rules: SahamRules,
) -> Result<SahamPlace, Error> {
    let reading = Evaluation::of(sky, rules)?;
    let triple = formula.at(sky.by_day);
    for (name, term) in [("a", triple.a), ("b", triple.b), ("c", triple.c)] {
        check_term(term).map_err(|why| {
            why.with_field(format!(
                "formula.{}.{name}",
                if sky.by_day { "day" } else { "night" }
            ))
        })?;
    }
    Ok(reading.point(triple))
}

/// A factor the sky cannot supply, refused.
fn check_term(term: SahamTerm) -> Result<(), Error> {
    match term {
        SahamTerm::Graha(graha) | SahamTerm::SignLordOf(graha)
            if !crate::SEVEN.contains(&graha) =>
        {
            Err(Error::invalid_arg(format!(
                "{graha:?} is not one of the seven a saham is read from"
            )))
        }
        SahamTerm::Degrees(deg) if !deg.is_finite() => Err(Error::invalid_arg(
            "a fixed factor is a longitude that is not a number",
        )),
        _ => Ok(()),
    }
}

/// One chart's sahams, each computed the first time it is read.
struct Evaluation<'a> {
    sky: &'a SahamSky,
    rules: SahamRules,
    houses: [f64; 12],
    memo: [Cell<Option<SahamPlace>>; 41],
}

impl<'a> Evaluation<'a> {
    fn of(sky: &'a SahamSky, rules: SahamRules) -> Result<Evaluation<'a>, Error> {
        sky.check()?;
        let houses = sky.houses(rules.houses)?;
        Ok(Evaluation {
            sky,
            rules,
            houses,
            memo: [const { Cell::new(None) }; 41],
        })
    }

    fn saham(&self, saham: Saham) -> SahamPlace {
        // `memo` holds one slot per member of `Saham::ALL`, so every
        // saham finds its own; the fallback computes without keeping.
        let slot = self.memo.get(saham.index());
        if let Some(point) = slot.and_then(Cell::get) {
            return point;
        }
        let point = self.point(saham.formula(self.rules).at(self.sky.by_day));
        if let Some(slot) = slot {
            slot.set(Some(point));
        }
        point
    }

    fn point(&self, triple: SahamTriple) -> SahamPlace {
        let (a, b, c) = (self.at(triple.a), self.at(triple.b), self.at(triple.c));
        let added_sign = !self.between(a, b, c);
        let raw = a - b + c + if added_sign { 30.0 } else { 0.0 };
        let longitude_deg = raw.rem_euclid(360.0);
        let sign = sign_of_longitude(longitude_deg);
        SahamPlace {
            longitude_deg,
            sign,
            lord: sign.attributes().lord,
            house: House::between(sign_of_longitude(self.sky.lagna_deg), sign),
            added_sign,
        }
    }

    /// Whether c falls between b and a, counted in the order of the
    /// signs from b, as the rules read "between".
    fn between(&self, a: f64, b: f64, c: f64) -> bool {
        match self.rules.add_sign {
            AddSign::Never => true,
            // Both ends inside: a point standing on one is between them.
            // That matters only where a formula reads one factor twice —
            // Roga's first reading, whose c *is* a — and it keeps that
            // saham from taking a sign it has no arc to take it from.
            // An a on b spans the whole circle, as it does by signs.
            AddSign::Degrees => {
                let span = (a - b).rem_euclid(360.0);
                span == 0.0 || (c - b).rem_euclid(360.0) <= span
            }
            AddSign::Signs => {
                let sign = |deg: f64| i32::from(sign_of_longitude(deg).id());
                let span = match (sign(a) - sign(b)).rem_euclid(12) {
                    0 => 12,
                    span => span,
                };
                let reach = (sign(c) - sign(b)).rem_euclid(12);
                reach > 0 && reach <= span
            }
        }
    }

    fn at(&self, term: SahamTerm) -> f64 {
        match term {
            SahamTerm::Lagna => self.sky.lagna_deg.rem_euclid(360.0),
            SahamTerm::Graha(graha) => self.sky.sky.longitude_of(graha),
            SahamTerm::House(house) => self.house(house),
            SahamTerm::HouseLord(house) => {
                let lord = sign_of_longitude(self.house(house)).attributes().lord;
                self.sky.sky.longitude_of(lord)
            }
            SahamTerm::SignLordOf(graha) => {
                let lord = self.sky.sky.sign_of(graha).attributes().lord;
                self.sky.sky.longitude_of(lord)
            }
            SahamTerm::Saham(saham) => self.saham(saham).longitude_deg,
            SahamTerm::Degrees(deg) => deg.rem_euclid(360.0),
        }
    }

    fn house(&self, house: House) -> f64 {
        // A `House` is 1 to 12, so the index is always inside the twelve.
        self.houses
            .get(usize::from(house.get() - 1))
            .copied()
            .unwrap_or_default()
            .rem_euclid(360.0)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use super::{
        AddSign, HousePoints, RogaReading, Saham, SahamFormula, SahamRules, SahamSky, SahamTerm,
        saham_point, sahams,
    };
    use crate::AnnualSky;
    use teistro_core::catalogue::{Graha, Rashi};
    use teistro_core::error::Status;
    use teistro_core::house::House;

    /// Sign (0 for Aries), degrees and minutes, as the source prints them.
    fn at(sign: u8, degrees: u8, minutes: u8) -> f64 {
        f64::from(sign) * 30.0 + f64::from(degrees) + f64::from(minutes) / 60.0
    }

    /// A sky with the planets the source prints, the rest anywhere: a
    /// saham reads only its own three factors.
    fn sky(sun: f64, moon: f64, saturn: f64, venus: f64) -> AnnualSky {
        AnnualSky {
            sun_deg: sun,
            moon_deg: moon,
            mars_deg: 0.0,
            mercury_deg: 0.0,
            jupiter_deg: 0.0,
            venus_deg: venus,
            saturn_deg: saturn,
        }
    }

    /// Asserts a saham lands on the source's printed arcminute, and
    /// whether it took the added sign.
    fn lands(chart: &SahamSky, saham: Saham, rules: SahamRules, printed: f64, added: bool) {
        let read = sahams(chart, &[saham], rules).unwrap();
        let place = read.points[0].point;
        let off = (place.longitude_deg - printed) * 60.0;
        assert!(
            off.abs() < 0.5,
            "{saham:?}: {} against the printed {printed}, {off}′",
            place.longitude_deg
        );
        assert_eq!(place.added_sign, added, "{saham:?}: the added sign");
    }

    /// The source's Example Chart, the forty-first year (Chart X-1), by
    /// day, under its own house mid-points.
    fn chart_x1() -> SahamSky {
        SahamSky::new(
            sky(at(4, 3, 50), at(1, 9, 40), at(6, 17, 13), at(4, 21, 45)),
            at(7, 9, 26),
            at(4, 13, 10),
            true,
        )
    }

    /// A chart whose midheaven the source does not print: any will do for
    /// a saham that reads no house.
    fn chart(sky: AnnualSky, lagna_deg: f64, by_day: bool) -> SahamSky {
        SahamSky::new(sky, lagna_deg, lagna_deg + 270.0, by_day)
    }

    #[test]
    fn sripatis_mid_points_are_the_sources() {
        // Chart X-1's printed mid-points, from its lagna Scorpio 9°26′ and
        // its tenth house Leo 13°10′, the first house first.
        let printed = [
            at(7, 9, 26),
            at(8, 10, 41),
            at(9, 11, 55),
            at(10, 13, 10),
            at(11, 11, 55),
            at(0, 10, 41),
            at(1, 9, 26),
            at(2, 10, 41),
            at(3, 11, 55),
            at(4, 13, 10),
            at(5, 11, 55),
            at(6, 10, 41),
        ];
        let built = super::sripati_mid_points(at(7, 9, 26), at(4, 13, 10));
        for (house, (got, want)) in built.iter().zip(printed).enumerate() {
            assert!(
                (got - want).abs() * 60.0 < 0.5,
                "house {}: {got} against the printed {want}",
                house + 1
            );
        }
    }

    #[test]
    fn the_sources_worked_sahams_come_back() {
        let rules = SahamRules::default();
        // Chart X-1, by day: Punya with the lagna between, Raja without
        // it and so one sign on, Matri.
        let x1 = chart_x1();
        lands(&x1, Saham::Punya, rules, at(4, 15, 16), false);
        lands(&x1, Saham::Raja, rules, at(10, 22, 49), true);
        lands(&x1, Saham::Matri, rules, at(3, 27, 21), false);

        // Chart X-3, the forty-sixth year, by night: Punya takes the sign.
        let x3 = chart(
            sky(at(4, 3, 49), at(11, 21, 17), at(8, 13, 58), 0.0),
            at(10, 23, 36),
            false,
        );
        lands(&x3, Saham::Punya, rules, at(4, 6, 8), true);
        lands(&x3, Saham::Raja, rules, at(6, 13, 27), false);

        // Chart X-9, the forty-seventh year, by night, and a house: its
        // printed eighth mid-point, Capricorn 4°12′, gives Mrityu.
        let x9 = chart(
            sky(at(4, 3, 50), at(4, 8, 2), at(8, 25, 50), 0.0),
            at(2, 7, 15),
            false,
        );
        lands(&x9, Saham::Punya, rules, at(2, 3, 3), false);
        // It prints the eighth mid-point and not the midheaven, so the
        // mid-point is handed in as the chart's own.
        let mut middles = [0.0; 12];
        middles[7] = at(9, 4, 12);
        let x9 = x9.with_chalit(middles);
        let chalit = SahamRules {
            houses: HousePoints::Chalit,
            ..rules
        };
        lands(&x9, Saham::Mrityu, chalit, at(1, 22, 0), false);
        // Equal houses put the eighth 3°03′ further and miss the print.
        let equal = SahamRules {
            houses: HousePoints::Equal,
            ..rules
        };
        let missed = sahams(&x9, &[Saham::Mrityu], equal).unwrap().points[0].point;
        assert!((missed.longitude_deg - at(1, 22, 0) - 3.05).abs() < 0.01);
    }

    #[test]
    fn the_leo_birth_refutes_counting_in_whole_signs() {
        // Chart III-1, the birth, by day: the Sun, the lagna and the Moon
        // all in Leo, at 3°49′, 14°33′ and 17°08′. Between by degrees, and
        // the source prints Leo 27°52′ with no sign added.
        let birth = chart(
            sky(at(4, 3, 49), at(4, 17, 8), 0.0, 0.0),
            at(4, 14, 33),
            true,
        );
        lands(
            &birth,
            Saham::Punya,
            SahamRules::default(),
            at(4, 27, 52),
            false,
        );
        let signs = SahamRules {
            add_sign: AddSign::Signs,
            ..SahamRules::default()
        };
        lands(&birth, Saham::Punya, signs, at(5, 27, 52), true);
        let never = SahamRules {
            add_sign: AddSign::Never,
            ..SahamRules::default()
        };
        lands(&birth, Saham::Punya, never, at(4, 27, 52), false);
    }

    /// Two charts a widely used program was asked about, by day and by
    /// night, with what it answered — values only, the program read as
    /// a black box (`03-design/tajika-sahams.md`).
    #[test]
    fn counting_in_whole_signs_reproduces_the_program() {
        let planets = AnnualSky {
            sun_deg: 54.305_702_612_820_696,
            moon_deg: 234.336_410_294_347_36,
            mars_deg: 26.077_063_200_315_393,
            mercury_deg: 192.917_521_550_408_1,
            jupiter_deg: 131.648_010_088_530_78,
            venus_deg: 20.879_612_918_894_452,
            saturn_deg: 182.676_863_948_191_28,
        };
        let lagna = 116.579_795_339_938_46;
        let rules = SahamRules {
            add_sign: AddSign::Signs,
            houses: HousePoints::Equal,
            roga: RogaReading::Saturn,
        };
        let printed: [(Saham, f64, f64); 16] = [
            (Saham::Punya, 296.610_503_021_465_15, 326.549_087_658_411_8),
            (Saham::Vidya, 326.549_087_658_411_8, 296.610_503_021_465_15),
            (Saham::Yasha, 311.617_302_407_004_1, 341.480_872_909_819_5),
            (Saham::Mahatmya, 27.113_235_161_088_2, 206.107_770_881_842_1),
            (Saham::Bhratri, 65.550_941_480_277_96, 65.550_941_480_277_96),
            (Saham::Pitri, 244.950_956_675_309_04, 18.208_634_004_567_884),
            (Saham::Matri, 330.036_592_715_391_35, 293.122_997_964_485_53),
            (Saham::Jeeva, 197.608_649_199_598_97, 65.550_941_480_277_96),
            (Saham::Roga, 64.920_248_993_782_38, 198.239_341_686_094_52),
            (Saham::Karma, 339.739_336_989_845_75, 283.420_253_690_031_2),
            (Saham::Kali, 222.150_742_228_153_83, 41.008_848_451_723_07),
            (Saham::Shastra, 171.888_667_690_747_6, 243.946_375_410_068_6),
            (Saham::Jadya, 66.317_720_802_532_22, 349.517_322_298_284),
            (
                Saham::Shatru,
                349.979_994_592_062_55,
                273.179_596_087_814_33,
            ),
            (
                Saham::Jalapatha,
                38.902_931_391_747_174,
                224.256_659_288_129_75,
            ),
            (
                Saham::Bandhana,
                260.513_434_413_212_34,
                332.707_571_629_717_96,
            ),
        ];
        for by_day in [true, false] {
            let chart = chart(planets, lagna, by_day);
            let asked: Vec<Saham> = printed.iter().map(|row| row.0).collect();
            let read = sahams(&chart, &asked, rules).unwrap();
            for ((saham, day, night), got) in printed.iter().zip(&read.points) {
                let want = if by_day { *day } else { *night };
                assert!(
                    (got.point.longitude_deg - want).abs() < 1e-9,
                    "{saham:?} by {}: {} against {want}",
                    if by_day { "day" } else { "night" },
                    got.point.longitude_deg
                );
            }
            // Its Asha is Saturn − Mars + lagna where the source has
            // Saturn − Venus: another reading, and a formula away.
            let asha = SahamFormula::reversed_by_night(
                SahamTerm::Graha(Graha::Saturn),
                SahamTerm::Graha(Graha::Mars),
                SahamTerm::Lagna,
            );
            let want = if by_day {
                273.179_596_087_814_33
            } else {
                349.979_994_592_062_55
            };
            let got = saham_point(&chart, &asha, rules).unwrap().longitude_deg;
            assert!((got - want).abs() < 1e-9, "Asha: {got} against {want}");
        }
    }

    #[test]
    fn the_table_is_the_sources_order_and_reads_only_what_came_before() {
        for (index, saham) in Saham::ALL.iter().enumerate() {
            assert_eq!(saham.index(), index, "{saham:?}");
            for roga in [RogaReading::Lagna, RogaReading::Saturn] {
                let formula = saham.formula(SahamRules {
                    roga,
                    ..SahamRules::default()
                });
                for triple in [formula.day, formula.night] {
                    for term in [triple.a, triple.b, triple.c] {
                        if let SahamTerm::Saham(read) = term {
                            // A saham reading only earlier ones cannot
                            // loop, so evaluation always ends.
                            assert!(read.index() < index, "{saham:?} reads {read:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn the_shared_formulas_and_the_ones_the_same_by_night() {
        let rules = SahamRules::default();
        for (one, other) in [
            (Saham::Vidya, Saham::Guru),
            (Saham::Raja, Saham::Pitri),
            (Saham::Kshama, Saham::Kali),
        ] {
            assert_eq!(one.formula(rules), other.formula(rules));
        }
        // The source says "same for day as well as night" of exactly
        // these; every other but Karya-siddhi reverses a and b by night.
        let same = [
            Saham::Bhratri,
            Saham::Putra,
            Saham::Roga,
            Saham::Bandhu,
            Saham::Mrityu,
            Saham::Deshantara,
            Saham::Artha,
            Saham::Paradara,
            Saham::Vanika,
            Saham::Vivaha,
            Saham::Santaapa,
            Saham::Shraddha,
            Saham::Preeti,
            Saham::Vyapara,
            Saham::Labha,
        ];
        for saham in Saham::ALL {
            let formula = saham.formula(rules);
            if same.contains(&saham) {
                assert_eq!(formula.day, formula.night, "{saham:?}");
            } else if saham != Saham::KaryaSiddhi {
                let reversed =
                    SahamFormula::reversed_by_night(formula.day.a, formula.day.b, formula.day.c);
                assert_eq!(formula, reversed, "{saham:?}");
            }
        }
        let roga = Saham::Roga.formula(SahamRules {
            roga: RogaReading::Saturn,
            ..rules
        });
        assert_eq!(roga.day.a, SahamTerm::Graha(Graha::Saturn));
        assert_eq!(roga.night.a, SahamTerm::Graha(Graha::Moon));
    }

    #[test]
    fn roga_by_the_lagna_never_takes_a_sign() {
        // Its c is its a, so it stands on an end of its own arc.
        for step in 0..72 {
            let moon = f64::from(step) * 5.0 + 0.3;
            let chart = chart(sky(0.0, moon, 0.0, 0.0), 131.7, step % 2 == 0);
            let read = sahams(&chart, &[Saham::Roga], SahamRules::default()).unwrap();
            let place = read.points[0].point;
            assert!(!place.added_sign, "the Moon at {moon}");
            let raw = (2.0 * 131.7 - moon).rem_euclid(360.0);
            assert!((place.longitude_deg - raw).abs() < 1e-9);
        }
    }

    #[test]
    fn many_at_once_answer_as_each_alone() {
        let chart = chart_x1();
        let rules = SahamRules::default();
        let every = sahams(&chart, &Saham::ALL, rules).unwrap();
        assert_eq!(every.points.len(), 41);
        assert!(every.by_day);
        assert_eq!(every.rules, rules);
        for asked in &every.points {
            let alone = sahams(&chart, &[asked.saham], rules).unwrap();
            assert_eq!(alone.points[0], *asked);
            // And each is a formula like any caller's.
            let own = saham_point(&chart, &asked.saham.formula(rules), rules).unwrap();
            assert_eq!(own, asked.point);
        }
        // The caller's order, repeats and all; and none asked, none given.
        let twice = sahams(&chart, &[Saham::Raja, Saham::Punya, Saham::Raja], rules).unwrap();
        assert_eq!(twice.points[0], twice.points[2]);
        assert_eq!(twice.points[1].saham, Saham::Punya);
        assert!(sahams(&chart, &[], rules).unwrap().points.is_empty());
    }

    #[test]
    fn a_place_says_its_sign_lord_and_house() {
        // Punya in Chart X-1: Leo 15°16′, the Sun's, the tenth from a
        // Scorpio lagna — "in the tenth house", as the source reads it.
        let read = sahams(&chart_x1(), &[Saham::Punya], SahamRules::default()).unwrap();
        let place = read.points[0].point;
        assert_eq!(place.sign, Rashi::Leo);
        assert_eq!(place.lord, Graha::Sun);
        assert_eq!(place.house, House::try_new(10).unwrap());
    }

    #[test]
    fn what_cannot_be_read_is_refused_by_name() {
        let mut chart = chart_x1();
        let rules = SahamRules::default();
        let node = SahamFormula::same(
            SahamTerm::Graha(Graha::Rahu),
            SahamTerm::Graha(Graha::Sun),
            SahamTerm::Lagna,
        );
        let why = saham_point(&chart, &node, rules).unwrap_err();
        assert_eq!(why.status, Status::InvalidArg);
        assert_eq!(why.field(), Some("formula.day.a"));
        assert!(why.message.contains("Rahu"), "{}", why.message);

        let fixed = SahamFormula::same(
            SahamTerm::Lagna,
            SahamTerm::Degrees(f64::NAN),
            SahamTerm::Lagna,
        );
        let why = saham_point(&chart, &fixed, rules).unwrap_err();
        assert_eq!(why.field(), Some("formula.day.b"));

        let own = SahamRules {
            houses: HousePoints::Chalit,
            ..rules
        };
        let why = sahams(&chart, &[Saham::Punya], own).unwrap_err();
        assert_eq!(why.field(), Some("chalit_deg"));
        let mut middles = [0.0; 12];
        middles[8] = f64::INFINITY;
        let why = sahams(&chart.with_chalit(middles), &[Saham::Punya], own).unwrap_err();
        assert_eq!(why.field(), Some("chalit_deg[8]"));
        chart.midheaven_deg = f64::NAN;
        let why = sahams(&chart, &[Saham::Punya], rules).unwrap_err();
        assert_eq!(why.field(), Some("midheaven_deg"));
        chart.lagna_deg = f64::NAN;
        let why = sahams(&chart, &[Saham::Punya], rules).unwrap_err();
        assert_eq!(why.field(), Some("lagna_deg"));
    }

    #[test]
    fn the_rules_read_camel_case_and_fill_the_rest() {
        let json = serde_json::to_value(SahamRules::default()).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"addSign": "DEGREES", "houses": "SRIPATI", "roga": "LAGNA"})
        );
        let read: SahamRules =
            serde_json::from_value(serde_json::json!({"addSign": "SIGNS"})).unwrap();
        assert_eq!(read.add_sign, AddSign::Signs);
        assert_eq!(read.houses, HousePoints::Sripati);
    }
}

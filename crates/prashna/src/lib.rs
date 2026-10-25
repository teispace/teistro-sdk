//! Prashna: the query chart read as *Shatpanchashika* prints it
//! (`03-design/prashna.md`, C335 to C342).
//!
//! A prashna is a chart cast for the moment a question is asked. This
//! kernel reads such a chart, and only reads it: [`PrashnaSky`] carries
//! the placements, the navāṁśas and the Shadbala totals the SDK took
//! from the chart the caller cast. [`read`] answers four things, each
//! from the verse that says it:
//!
//! - **the verdict** (I.3, I.4, II.2): the clauses that hold, each for
//!   or against, and three outcomes over them, never a score;
//! - **change** (II.1–2): whether the matter stays as it is;
//! - **timing** (II.14–15 by default, V.5 and II.17 as named rules);
//! - **the unspoken question** (VII.7–8): whom it is about, and the
//!   class of the thing thought of (I.7).
//!
//! ```
//! use teistro_core::catalogue::Rashi;
//! use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, Unit, read};
//!
//! // Shatpanchashika V.5's own example: Taurus rising, Virgo the first
//! // sign a graha stands in, five signs on: sixty days, or five if that
//! // graha is retrograde.
//! let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
//! let sky = PrashnaSky {
//!     lagna_deg: 40.0,
//!     // The lagna's six vargas, D1 to D30; its navāṁśa Leo.
//!     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
//!     // Sun to Ketu: every graha from Virgo on.
//!     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
//!     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
//! };
//! let rules = PrashnaRules { timing: teistro_prashna::TimingRule::FirstOccupied, ..PrashnaRules::default() };
//! let answer = read(&sky, Question::default(), rules)?;
//! assert_eq!(answer.timing.count, 5);
//! assert_eq!((answer.timing.amount, answer.timing.unit), (Some(60), Unit::Days));
//! # Ok::<(), teistro_core::Error>(())
//! ```

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![doc(html_no_source)]

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use teistro_aspect::drishti;
use teistro_core::Error;
use teistro_core::catalogue::{Graha, Modality, Parity, Rashi, Rising, Varga};
use teistro_core::house::House;
use teistro_tajika::{
    AnnualSky, AnnualStates, GrahaWeakness, MoonRules, MoonWeakness, OfficeRules, WeaknessRules,
    YearYogas, YogaRules, moon_weakness, weaknesses, year_yogas_with_states,
};

mod baseline;
mod graded;
mod link;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests;

pub use baseline::{Answer, Factor, FactorKind, Score, ScoreRule, number_sign};
pub use graded::{Grade, Graded, GradedRule, LordsSee};
pub use link::{Link, LinkKind, Meaning, MoonScope, SamarasimhaClause, Tense, TriKala};

/// The nine grahas a prashna places, in the order [`PrashnaSky`] holds
/// them.
pub const GRAHAS: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// The seven grahas Shadbala measures, the only ones "the strongest"
/// can be (C338).
const SEVEN: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Where one graha stands in the query chart.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_prashna::Placed;
///
/// let mars = Placed { longitude_deg: 200.0, navamsha: Rashi::Aries, retrograde: true, combust: false };
/// assert_eq!(mars.sign(), Rashi::Libra);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Placed {
    /// Its sidereal longitude, in degrees.
    pub longitude_deg: f64,
    /// The sign of its navāṁśa.
    pub navamsha: Rashi,
    /// Whether it is retrograde.
    pub retrograde: bool,
    /// Whether the Sun burns it.
    pub combust: bool,
}

impl Placed {
    /// The sign it stands in.
    ///
    /// ```
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_prashna::Placed;
    ///
    /// let at = Placed { longitude_deg: -10.0, navamsha: Rashi::Aries, retrograde: false, combust: false };
    /// // Folded into the circle: 350°.
    /// assert_eq!(at.sign(), Rashi::Pisces);
    /// ```
    #[must_use]
    pub fn sign(&self) -> Rashi {
        sign_at(self.longitude_deg)
    }
}

/// The query chart, as the kernel reads it.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// assert_eq!(sky.lagna(), Rashi::Taurus);
/// # let _ = (Graha::Sun, Question::default(), PrashnaRules::default(), read);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrashnaSky {
    /// The lagna's sidereal longitude, in degrees.
    pub lagna_deg: f64,
    /// The lagna's sign in each of the [`SHADVARGA`], the rasi first and
    /// the navāṁśa fourth.
    pub lagna_vargas: [Rashi; 6],
    /// The nine grahas, in the order of [`GRAHAS`].
    pub grahas: [Placed; 9],
    /// The seven grahas' Shadbala totals in rupas, the Sun first.
    pub strength: [f64; 7],
}

impl PrashnaSky {
    /// Where `graha` stands, or `None` for a graha a prashna does not
    /// place.
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Rashi};
    /// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
    ///
    /// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
    /// // one graha a sign from there on; Saturn the strongest.
    /// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
    /// let sky = PrashnaSky {
    ///     lagna_deg: 40.0,
    ///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
    ///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
    ///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
    /// };
    /// assert_eq!(sky.of(Graha::Sun).map(Placed::sign), Some(Rashi::Virgo));
    /// # let _ = (Question::default(), PrashnaRules::default(), read);
    /// ```
    #[must_use]
    pub fn of(&self, graha: Graha) -> Option<&Placed> {
        GRAHAS
            .iter()
            .position(|&placed| placed == graha)
            .and_then(|at| self.grahas.get(at))
    }

    /// The lagna's sign.
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Rashi};
    /// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
    ///
    /// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
    /// // one graha a sign from there on; Saturn the strongest.
    /// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
    /// let sky = PrashnaSky {
    ///     lagna_deg: 40.0,
    ///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
    ///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
    ///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
    /// };
    /// assert_eq!(sky.lagna(), Rashi::Taurus);
    /// # let _ = (Graha::Sun, Question::default(), PrashnaRules::default(), read);
    /// ```
    #[must_use]
    pub fn lagna(&self) -> Rashi {
        sign_at(self.lagna_deg)
    }

    /// The sign of the lagna's navāṁśa, the rising varga by default.
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Rashi};
    /// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
    ///
    /// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
    /// // one graha a sign from there on; Saturn the strongest.
    /// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
    /// let sky = PrashnaSky {
    ///     lagna_deg: 40.0,
    ///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
    ///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
    ///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
    /// };
    /// assert_eq!(sky.lagna_navamsha(), Rashi::Leo);
    /// # let _ = (Graha::Sun, Question::default(), PrashnaRules::default(), read);
    /// ```
    #[must_use]
    pub const fn lagna_navamsha(&self) -> Rashi {
        let [_, _, _, navamsha, _, _] = self.lagna_vargas;
        navamsha
    }

    /// The house `graha` stands in, counted from the lagna by whole
    /// signs.
    fn house_of(&self, graha: Graha) -> Option<u8> {
        self.of(graha)
            .map(|placed| House::between(self.lagna(), placed.sign()).get())
    }

    /// The grahas standing in `sign`, in the order of [`GRAHAS`].
    fn in_sign(&self, sign: Rashi) -> impl Iterator<Item = Graha> + '_ {
        GRAHAS
            .into_iter()
            .zip(self.grahas)
            .filter(move |(_, placed)| placed.sign() == sign)
            .map(|(graha, _)| graha)
    }

    /// A graha's Shadbala total, or `None` for the nodes.
    fn strength_of(&self, graha: Graha) -> Option<f64> {
        SEVEN
            .iter()
            .position(|&one| one == graha)
            .and_then(|at| self.strength.get(at).copied())
    }

    /// The strongest of `among` by Shadbala, and whether another equals
    /// it (C338). A tie goes to the one earlier in weekday order, which
    /// is the order of [`GRAHAS`].
    fn strongest(&self, among: impl IntoIterator<Item = Graha>) -> Option<(Graha, bool)> {
        let mut best: Option<(Graha, f64, bool)> = None;
        for graha in among {
            let Some(rupas) = self.strength_of(graha) else {
                continue;
            };
            // A tie is two equal totals, so the comparison is exact.
            best = match best.map(|(held, top, tie)| (held, top, tie, rupas.total_cmp(&top))) {
                Some((_, _, _, Ordering::Greater)) | None => Some((graha, rupas, false)),
                Some((held, top, _, Ordering::Equal)) => Some((held, top, true)),
                Some((held, top, tie, Ordering::Less)) => Some((held, top, tie)),
            };
        }
        best.map(|(graha, _, tie)| (graha, tie))
    }

    /// Every input is a finite number.
    fn check(&self) -> Result<(), Error> {
        let finite = |value: f64, field: &str| {
            if value.is_finite() {
                Ok(())
            } else {
                Err(
                    Error::invalid_arg(format!("{field} is {value}, not a finite number"))
                        .with_field(field),
                )
            }
        };
        finite(self.lagna_deg, "lagnaDeg")?;
        for placed in &self.grahas {
            finite(placed.longitude_deg, "grahas")?;
        }
        for rupas in self.strength {
            finite(rupas, "strength")?;
        }
        Ok(())
    }
}

/// The sign a sidereal longitude falls in.
fn sign_at(longitude_deg: f64) -> Rashi {
    let index = (longitude_deg.rem_euclid(360.0) / 30.0).floor();
    // `rem_euclid` keeps the quotient in 0..12; the cast cannot truncate.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a sign index in 0..12"
    )]
    let index = index as u16;
    Rashi::from_id(index % 12).unwrap_or(Rashi::Aries)
}

/// Whether a graha counts as a benefic or a malefic in a prashna.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// use teistro_prashna::{Disposition, disposition};
///
/// // The Moon 10° past the Sun waxes.
/// assert_eq!(disposition(&sky, Graha::Moon), Some(Disposition::Benefic));
/// assert_eq!(disposition(&sky, Graha::Saturn), Some(Disposition::Malefic));
/// # let _ = (Question::default(), PrashnaRules::default(), read);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Disposition {
    /// Jupiter, Venus, a Mercury no malefic joins, a waxing Moon.
    Benefic,
    /// The Sun, Mars, Saturn, a waning Moon, a Mercury a malefic joins.
    Malefic,
}

/// Whether `graha` is a benefic or a malefic as the gloss to *Shatpanchashika*
/// I.3 lists them, or `None` for the nodes, which the list leaves out.
///
/// The Moon is a benefic while waxing, from new to full, and Mercury is
/// a malefic in a sign a malefic shares with him.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// use teistro_prashna::{Disposition, disposition};
///
/// // Mercury alone in Scorpio: no malefic joins him.
/// assert_eq!(disposition(&sky, Graha::Mercury), Some(Disposition::Benefic));
/// // The list leaves out the nodes.
/// assert_eq!(disposition(&sky, Graha::Rahu), None);
/// # let _ = (Question::default(), PrashnaRules::default(), read);
/// ```
#[must_use]
pub fn disposition(sky: &PrashnaSky, graha: Graha) -> Option<Disposition> {
    let waxing = || {
        let (Some(sun), Some(moon)) = (sky.of(Graha::Sun), sky.of(Graha::Moon)) else {
            return true;
        };
        (moon.longitude_deg - sun.longitude_deg).rem_euclid(360.0) < 180.0
    };
    let malefic = |graha: Graha| match graha {
        Graha::Sun | Graha::Mars | Graha::Saturn => true,
        Graha::Moon => !waxing(),
        _ => false,
    };
    match graha {
        Graha::Jupiter | Graha::Venus => Some(Disposition::Benefic),
        Graha::Sun | Graha::Mars | Graha::Saturn | Graha::Moon => Some(if malefic(graha) {
            Disposition::Malefic
        } else {
            Disposition::Benefic
        }),
        Graha::Mercury => {
            let sign = sky.of(Graha::Mercury)?.sign();
            let joined = sky.in_sign(sign).any(malefic);
            Some(if joined {
                Disposition::Malefic
            } else {
                Disposition::Benefic
            })
        }
        _ => None,
    }
}

/// How Pisces rises (C336).
///
/// ```
/// use teistro_prashna::{PiscesRising, PrashnaRules};
///
/// assert_eq!(PrashnaRules::default().pisces, PiscesRising::BothWays);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PiscesRising {
    /// Both ways, as the verse the 1941 gloss quotes says ("ubhayato
    /// mīnaḥ"): a clause on both sides.
    #[default]
    BothWays,
    /// By the head, as *Brihat Jataka* I.10 leaves it.
    Shirshodaya,
}

/// The six vargas the 1941 gloss to *Shatpanchashika* I.4 names for the
/// rising varga, in the order a [`PrashnaSky`] holds the lagna's signs.
pub const SHADVARGA: [Varga; 6] = [
    Varga::D1,
    Varga::D2,
    Varga::D3,
    Varga::D9,
    Varga::D12,
    Varga::D30,
];

/// Whose the rising varga is (C336).
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::{ClauseKind, RisingVarga};
///
/// // By the navamsha alone, unless the six vargas are asked for.
/// assert!(answer.verdict.clauses.iter().any(|clause| clause.kind == ClauseKind::RisingNavamsha));
/// let six = PrashnaRules { rising_varga: RisingVarga::Shadvarga, ..PrashnaRules::default() };
/// let wider = read(&sky, Question::default(), six)?;
/// assert!(wider.verdict.clauses.iter().any(|clause| clause.kind == ClauseKind::RisingShadvarga));
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RisingVarga {
    /// The rising navāṁśa's lord, as the verse's *aṁśa* reads alone.
    #[default]
    Navamsha,
    /// The [`SHADVARGA`]: the gloss names six vargas and not how many
    /// must be a benefic's, so a clause for when four or more of the
    /// six lords are benefics, against when four or more are malefics,
    /// and both ways otherwise.
    Shadvarga,
}

/// Which rule times the matter.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::{TimingRule, Unit};
///
/// // II.14–15: Saturn, the strongest, in the 10th from Taurus.
/// assert_eq!((answer.timing.rule, answer.timing.graha, answer.timing.count), (TimingRule::StrongestGraha, Graha::Saturn, 10));
/// assert_eq!(answer.timing.unit, Unit::Months);
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimingRule {
    /// *Shatpanchashika* II.14–15: the strongest graha's house from the
    /// lagna, in months, doubled in a fixed navāṁśa and trebled in a
    /// dual one.
    #[default]
    StrongestGraha,
    /// V.5: the first sign from the lagna a graha stands in, its count
    /// times twelve days, or the count in days if that graha is
    /// retrograde.
    FirstOccupied,
    /// II.17: the Moon's sign from the lagna, in days, and no time at all
    /// when a graha stands between them.
    MoonDays,
    /// The baseline engine's, which no text read gives (C335): the
    /// lagna lord's house from the lagna, in days under a movable lagna,
    /// months under a fixed one and years under a dual one.
    Baseline,
}

/// Which rule reads an unspoken question.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::{MookRule, Person};
///
/// // VII.7–8: the strongest graha in the 10th, a question about a guru.
/// assert_eq!((answer.mook.rule, answer.mook.house), (MookRule::Shatpanchashika, 10));
/// assert_eq!(answer.mook.person, Some(Person::Guru));
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MookRule {
    /// *Shatpanchashika* VII.7–8: the person, from the strongest graha's
    /// house; houses 2, 8, 11 and 12 are not listed (C339).
    #[default]
    Shatpanchashika,
    /// *Jnanaprakasha*, quoted in *Tajika Nilakanthi*: the house the Moon
    /// stands in, or the lagna lord's when he is the stronger.
    MoonHouse,
    /// `BASELINE`: the baseline engine's strongest graha, 2 points for a
    /// kendra and 1 for a trikona, the first in its order taking a tie;
    /// its house is the topic. Unsourced (C339).
    Baseline,
}

/// The readings a prashna is given under.
///
/// ```
/// use teistro_prashna::{MookRule, PrashnaRules, TimingRule};
///
/// let rules = PrashnaRules { timing: TimingRule::MoonDays, ..PrashnaRules::default() };
/// assert_eq!(rules.mook, MookRule::Shatpanchashika);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PrashnaRules {
    /// How Pisces rises.
    pub pisces: PiscesRising,
    /// Whose the rising varga is.
    pub rising_varga: RisingVarga,
    /// Which rule times the matter.
    pub timing: TimingRule,
    /// Which rule reads an unspoken question.
    pub mook: MookRule,
    /// How the Moon's weaknesses are read (Tajika Nilakanthi, Samjna
    /// Tantra vv. 73–74).
    pub moon: MoonRules,
    /// How a Kamboola's offices are read (Samjna Tantra vv. 36–46).
    pub offices: OfficeRules,
    /// How v. 72's weaknesses are read.
    pub weakness: WeaknessRules,
    /// Which questions v. 96's Moon governs.
    pub moon_scope: MoonScope,
    /// Whether the Bhuvanapradīpa's graded yogas are read, and by which
    /// aspect (C373, open: off by default).
    pub graded: GradedRule,
    /// Whether the baseline engine's points come beside the clauses.
    pub score: ScoreRule,
}

impl PrashnaRules {
    /// Every reading the baseline engine takes.
    ///
    /// ```
    /// use teistro_prashna::{MookRule, PrashnaRules, TimingRule};
    ///
    /// let baseline = PrashnaRules::baseline();
    /// assert_eq!((baseline.timing, baseline.mook), (TimingRule::Baseline, MookRule::Baseline));
    /// ```
    #[must_use]
    pub const fn baseline() -> PrashnaRules {
        PrashnaRules {
            pisces: PiscesRising::BothWays,
            rising_varga: RisingVarga::Navamsha,
            timing: TimingRule::Baseline,
            mook: MookRule::Baseline,
            score: ScoreRule::Baseline,
            moon: MoonRules {
                kshina: teistro_tajika::KshinaRule::DarkEighthToBrightEighth,
            },
            offices: OfficeRules {
                order: teistro_tajika::OfficeOrder::Gloss,
                drekkana: teistro_tajika::DrekkanaReading::Tajika,
                enemy: teistro_tajika::EnemySign::Tajika,
                hudda: teistro_tajika::HuddaTable::Nilakanthi,
            },
            weakness: WeaknessRules {
                setting: teistro_tajika::SettingPart::NavamshaSign,
            },
            moon_scope: MoonScope::Every,
            graded: GradedRule::Off,
        }
    }
}

/// What was asked.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// // A question of the 7th, Scorpio: Venus the lagnesha, Mars the karyesha.
/// let answer = read(&sky, Question::about(7), PrashnaRules::default())?;
/// use teistro_prashna::Tense;
///
/// assert_eq!(Question::default().tense, Tense::Unspecified);
/// // A house asked about brings the lords' Tajika link.
/// assert!(answer.links.is_some() && answer.link.is_some());
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Question {
    /// The house the matter belongs to, 1 to 12, when the question names
    /// one (the kārya bhāva).
    pub house: Option<u8>,
    /// The number the querent chose, 1 to 108, read only by the
    /// unsourced [`number_sign`] (C340).
    pub number: Option<u8>,
    /// When the matter falls, which decides what a separation means
    /// (C370).
    pub tense: Tense,
}

impl Question {
    /// A question about the matter of `house`, 1 to 12 (checked by
    /// [`read`]).
    ///
    /// ```
    /// use teistro_prashna::Question;
    ///
    /// assert_eq!(Question::about(7).house, Some(7));
    /// ```
    #[must_use]
    pub const fn about(house: u8) -> Question {
        Question {
            house: Some(house),
            number: None,
            tense: Tense::Unspecified,
        }
    }
}

/// Which way a clause tells.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::{ClauseKind, Favour};
///
/// // Taurus rises by its back, which tells against the matter (I.4).
/// let rising = answer.verdict.clauses.iter().find(|clause| clause.kind == ClauseKind::LagnaRising);
/// assert_eq!(rising.map(|clause| clause.favour), Some(Favour::Against));
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Favour {
    /// Toward success.
    For,
    /// Toward failure.
    Against,
    /// Both at once: Pisces rising both ways.
    Both,
}

/// What a clause of the verdict says. Each names its verse.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::ClauseKind;
///
/// // Without a house asked about there is no karya house to read.
/// assert!(answer.verdict.clauses.iter().all(|clause| clause.kind != ClauseKind::KaryaHouse));
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClauseKind {
    /// I.4: how the lagna rises, by the head, by the back or both ways.
    LagnaRising,
    /// I.4: a benefic or a malefic stands in the lagna.
    InLagna,
    /// I.4: the rising navāṁśa belongs to a benefic or a malefic.
    RisingNavamsha,
    /// I.4 under the gloss's six vargas: most of the rising degree's six
    /// varga lords are benefics or malefics, or neither has four.
    RisingShadvarga,
    /// II.2: a benefic or a malefic aspects the lagna fully.
    AspectsLagna,
    /// II.2: a benefic or a malefic aspects the Moon fully.
    AspectsMoon,
    /// I.3: the house asked about holds, or is fully aspected by, its own
    /// lord, a benefic or a malefic.
    KaryaHouse,
}

impl ClauseKind {
    /// The verse a clause reads.
    ///
    /// ```
    /// use teistro_prashna::ClauseKind;
    ///
    /// assert!(ClauseKind::LagnaRising.source().contains("I.4"));
    /// assert!(ClauseKind::KaryaHouse.source().contains("I.3"));
    /// ```
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            ClauseKind::LagnaRising | ClauseKind::InLagna | ClauseKind::RisingNavamsha => {
                "Shatpanchashika I.4"
            }
            ClauseKind::RisingShadvarga => "Shatpanchashika I.4, the 1941 gloss",
            ClauseKind::AspectsLagna | ClauseKind::AspectsMoon => "Shatpanchashika II.2",
            ClauseKind::KaryaHouse => "Shatpanchashika I.3",
        }
    }
}

/// One clause of the verdict.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::ClauseKind;
///
/// let rising = answer.verdict.clauses.iter().find(|clause| clause.kind == ClauseKind::LagnaRising);
/// // How the lagna rises is about no graha.
/// assert_eq!(rising.map(|clause| clause.graha), Some(None));
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Clause {
    /// What it says.
    pub kind: ClauseKind,
    /// The graha it is about, `None` for how the lagna rises.
    pub graha: Option<Graha>,
    /// Which way it tells.
    pub favour: Favour,
}

/// The three outcomes of *Shatpanchashika* I.4.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::{Favour, Outcome};
///
/// let against = answer.verdict.clauses.iter().filter(|clause| clause.favour != Favour::For).count();
/// let expected = match against {
///     0 => Outcome::Succeeds,
///     n if n == answer.verdict.clauses.len() && answer.verdict.clauses.iter().all(|clause| clause.favour == Favour::Against) => Outcome::Fails,
///     _ => Outcome::WithDifficulty,
/// };
/// assert_eq!(answer.verdict.outcome, expected);
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PrashnaOutcome"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Outcome {
    /// Every clause is for it.
    Succeeds,
    /// The clauses are mixed: "success with difficulty".
    WithDifficulty,
    /// Every clause is against it.
    Fails,
}

/// Whether the matter succeeds.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// // The clauses are the answer, and never a score.
/// assert!(!answer.verdict.clauses.is_empty());
/// assert!(answer.score.is_none());
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PrashnaVerdict"))]
#[serde(rename_all = "camelCase")]
pub struct Verdict {
    /// Every clause that holds.
    pub clauses: Vec<Clause>,
    /// The outcome over them.
    pub outcome: Outcome,
}

/// Whether the matter stays as it is (*Shatpanchashika* II.1–2).
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::Change;
///
/// // A fixed lagna: the state continues (II.1).
/// assert_eq!(answer.change, Change::Stays);
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Change {
    /// A fixed lagna, or the first half of a dual one: no going or
    /// coming, the state continues.
    Stays,
    /// A movable lagna, or the second half of a dual one: the reverse.
    Changes,
}

/// The unit a time is counted in.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::{TimingRule, Unit};
///
/// let days = PrashnaRules { timing: TimingRule::FirstOccupied, ..PrashnaRules::default() };
/// assert_eq!(read(&sky, Question::default(), days)?.timing.unit, Unit::Days);
/// # let _ = (answer, Graha::Sun);
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PrashnaUnit"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Unit {
    /// Days.
    Days,
    /// Months.
    Months,
    /// Years.
    Years,
}

/// When the matter comes to pass.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// // Saturn in a movable navamsha: the count once.
/// assert_eq!(answer.timing.multiplier, 1);
/// assert_eq!(answer.timing.amount, Some(10));
/// assert!(!answer.timing.tie);
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Timing {
    /// The rule that timed it.
    pub rule: TimingRule,
    /// The graha the count was taken from.
    pub graha: Graha,
    /// Whether that graha won a tie in strength (C338).
    pub tie: bool,
    /// The count, 1 to 12.
    pub count: u8,
    /// What the count is multiplied by.
    pub multiplier: u8,
    /// The time, `None` when the rule gives none: under II.17, a graha
    /// standing between the lagna and the Moon.
    pub amount: Option<u16>,
    /// What it is counted in.
    pub unit: Unit,
    /// The grahas standing between the lagna and the Moon, under II.17.
    pub between: Vec<Graha>,
}

/// Whom *Shatpanchashika* VII.7–8 says an unspoken question is about.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::{MookRule, Person};
///
/// assert_eq!(answer.mook.person, Some(Person::Guru));
/// // The other rules name a house, not a person.
/// let moon = PrashnaRules { mook: MookRule::MoonHouse, ..PrashnaRules::default() };
/// assert_eq!(read(&sky, Question::default(), moon)?.mook.person, None);
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Person {
    /// House 1: the querent, or one like them.
    Querent,
    /// House 1, the strong graha a friend of the lagna navāṁśa's lord.
    Friend,
    /// House 6, or house 1 with the strong graha an enemy of the lagna
    /// navāṁśa's lord.
    Enemy,
    /// House 3.
    Brother,
    /// House 4.
    MotherOrSister,
    /// House 5.
    Son,
    /// House 7.
    Wife,
    /// House 9: one who keeps the dharma.
    Religious,
    /// House 10.
    Guru,
    /// Houses 2, 8, 11 and 12, which the verse does not list (C339).
    Unspecified,
}

/// What the thing thought of is (*Shatpanchashika* I.6–7).
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// use teistro_prashna::Thought;
///
/// assert!(matches!(answer.mook.thought, Thought::Mineral | Thought::Root | Thought::Living));
/// # let _ = Graha::Sun;
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Thought {
    /// *Dhātu*: a mineral.
    Mineral,
    /// *Mūla*: a root or a plant.
    Root,
    /// *Jīva*: a living being.
    Living,
}

/// What an unspoken question is about.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// assert_eq!((answer.mook.graha, answer.mook.tie), (Graha::Saturn, false));
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Mook {
    /// The rule that read it.
    pub rule: MookRule,
    /// The graha whose house was read.
    pub graha: Graha,
    /// Whether that graha won a tie in strength.
    pub tie: bool,
    /// The house read, 1 to 12.
    pub house: u8,
    /// The person, under [`MookRule::Shatpanchashika`].
    pub person: Option<Person>,
    /// The class of the thing thought of, from the lagna's navāṁśa
    /// (I.7).
    pub thought: Thought,
}

/// A prashna read.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// // A question of the 7th, Scorpio: Venus the lagnesha, Mars the karyesha.
/// let answer = read(&sky, Question::about(7), PrashnaRules::default())?;
/// assert_eq!(answer.tri_kala.lagnesha, Graha::Venus);
/// assert_eq!(answer.weak.len(), 7);
/// assert!(answer.graded.is_none() && answer.number_sign.is_none());
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Prashna {
    /// The readings it was given under.
    pub rules: PrashnaRules,
    /// Whether the matter succeeds.
    pub verdict: Verdict,
    /// Whether it stays as it is.
    pub change: Change,
    /// When.
    pub timing: Timing,
    /// What an unspoken question is about.
    pub mook: Mook,
    /// How the lagna lord and the lord of the house asked about stand
    /// to each other, as Tajika's sixteen yogas judge them (Tajika
    /// Nilakanthi's Prashna Tantra, vv. 9–21): present only when a
    /// house is asked.
    pub links: Option<YearYogas>,
    /// What those yogas say of the matter (Prashna Tantra vv. 14–21),
    /// with a Kamboola's grade: present when [`Prashna::links`] is.
    pub link: Option<Link>,
    /// The lagnesha's own yogas read as past, present and future
    /// (Prashna Tantra, the general section, n179).
    pub tri_kala: TriKala,
    /// Each of the seven's weaknesses of v. 72, the Sun first: a graha
    /// "unable to bring about the matter".
    pub weak: Vec<GrahaWeakness>,
    /// The Bhuvanapradīpa's graded yogas (n178), under
    /// [`PrashnaRules::graded`] only.
    pub graded: Option<Graded>,
    /// The Moon's weaknesses, which Tajika Nilakanthi's Samjna Tantra
    /// v. 74 reads "at birth or in a query": each clause of vv. 73–74
    /// that holds, none weighed.
    pub moon: MoonWeakness,
    /// The baseline engine's points, under [`ScoreRule::Baseline`] only.
    pub score: Option<Score>,
    /// The sign of the querent's number, when one is given: the
    /// baseline engine's rule, unsourced (C340).
    pub number_sign: Option<Rashi>,
}

/// Reads a query chart.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_prashna::{Placed, PrashnaRules, PrashnaSky, Question, read};
///
/// // Taurus rising, its navamsha Leo; the Sun, the Moon and Ketu in Virgo and
/// // one graha a sign from there on; Saturn the strongest.
/// let at = |longitude_deg: f64| Placed { longitude_deg, navamsha: Rashi::Aries, retrograde: false, combust: false };
/// let sky = PrashnaSky {
///     lagna_deg: 40.0,
///     lagna_vargas: [Rashi::Taurus, Rashi::Cancer, Rashi::Taurus, Rashi::Leo, Rashi::Virgo, Rashi::Virgo],
///     grahas: [at(160.0), at(170.0), at(200.0), at(230.0), at(260.0), at(290.0), at(320.0), at(350.0), at(170.0)],
///     strength: [5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
/// };
/// let answer = read(&sky, Question::default(), PrashnaRules::default())?;
/// // A house outside 1 to 12 is refused.
/// assert!(read(&sky, Question::about(13), PrashnaRules::default()).is_err());
/// # let _ = (answer, Graha::Sun);
/// # Ok::<(), teistro_core::Error>(())
/// ```
///
/// # Errors
///
/// A house outside 1 to 12 (`question.house`), a number outside 1 to
/// 108 (`question.number`), or an input that is not a finite
/// number, naming its field.
pub fn read(sky: &PrashnaSky, question: Question, rules: PrashnaRules) -> Result<Prashna, Error> {
    sky.check()?;
    if let Some(house) = question.house
        && !(1..=12).contains(&house)
    {
        return Err(Error::invalid_arg(format!("house {house} is not 1 to 12"))
            .with_field("question.house"));
    }
    let seven = annual(sky)?;
    let links = links(sky, question)?;
    let link = links
        .as_ref()
        .map(|links| {
            let rules = link::LinkRules {
                offices: rules.offices,
                moon_scope: rules.moon_scope,
            };
            link::link(links, &seven, sky.lagna(), question.tense, rules)
        })
        .transpose()?;
    let graded = graded::graded(
        sky,
        links.as_ref().map(|links| (links.karyesha, links.sign)),
        rules.graded,
    );
    Ok(Prashna {
        rules,
        verdict: verdict(sky, question, rules),
        change: change(sky),
        timing: timing(sky, rules.timing)?,
        mook: mook(sky, rules.mook)?,
        links,
        link,
        tri_kala: link::tri_kala(sky.lagna().attributes().lord, &seven)?,
        weak: weaknesses(sky.lagna_deg, &seven, rules.weakness)?,
        graded,
        moon: moon_weakness(&seven, rules.moon)?,
        score: match rules.score {
            ScoreRule::Off => None,
            ScoreRule::Baseline => Some(baseline::score(sky)?),
        },
        number_sign: question
            .number
            .map(number_sign)
            .transpose()
            .map_err(|error| error.under("question"))?,
    })
}

/// The Tajika yogas between the lagna lord and the kāryesha.
///
/// The Prashna Tantra judges a question by the same sixteen yogas, under
/// the same orbs (v. 13), that the annual chart does, so the year's
/// judgement is read over the query chart's sky.
fn links(sky: &PrashnaSky, question: Question) -> Result<Option<YearYogas>, Error> {
    let Some(house) = question.house.and_then(|house| House::try_from(house).ok()) else {
        return Ok(None);
    };
    let those = |state: fn(&Placed) -> bool| -> Vec<Graha> {
        GRAHAS
            .into_iter()
            .zip(sky.grahas)
            .filter(|(graha, placed)| SEVEN.contains(graha) && state(placed))
            .map(|(graha, _)| graha)
            .collect()
    };
    let states = AnnualStates {
        retrograde: those(|placed| placed.retrograde),
        combust: those(|placed| placed.combust),
    };
    year_yogas_with_states(
        sky.lagna_deg,
        house,
        &annual(sky)?,
        &states,
        YogaRules::default(),
    )
    .map(Some)
}

/// The seven's longitudes, as Tajika reads a sky.
fn annual(sky: &PrashnaSky) -> Result<AnnualSky, Error> {
    let at = |graha: Graha| {
        sky.of(graha)
            .map(|placed| placed.longitude_deg)
            .ok_or_else(unplaced)
    };
    Ok(AnnualSky {
        sun_deg: at(Graha::Sun)?,
        moon_deg: at(Graha::Moon)?,
        mars_deg: at(Graha::Mars)?,
        mercury_deg: at(Graha::Mercury)?,
        jupiter_deg: at(Graha::Jupiter)?,
        venus_deg: at(Graha::Venus)?,
        saturn_deg: at(Graha::Saturn)?,
    })
}

/// The clause a graha makes by its disposition, if it has one.
fn by(sky: &PrashnaSky, kind: ClauseKind, graha: Graha) -> Option<Clause> {
    disposition(sky, graha).map(|disposition| Clause {
        kind,
        graha: Some(graha),
        favour: match disposition {
            Disposition::Benefic => Favour::For,
            Disposition::Malefic => Favour::Against,
        },
    })
}

/// The rising varga read over the [`SHADVARGA`]: four of the six lords
/// make a majority.
fn shadvarga(sky: &PrashnaSky) -> Clause {
    let count = |wanted: Disposition| {
        sky.lagna_vargas
            .iter()
            .filter(|sign| disposition(sky, sign.attributes().lord) == Some(wanted))
            .count()
    };
    Clause {
        kind: ClauseKind::RisingShadvarga,
        graha: None,
        favour: if count(Disposition::Benefic) >= 4 {
            Favour::For
        } else if count(Disposition::Malefic) >= 4 {
            Favour::Against
        } else {
            Favour::Both
        },
    }
}

/// Every graha that aspects `sign` fully, in the order of [`GRAHAS`].
fn aspecting(sky: &PrashnaSky, sign: Rashi) -> impl Iterator<Item = Graha> + '_ {
    GRAHAS
        .into_iter()
        .zip(sky.grahas)
        .filter(move |(graha, placed)| drishti::between(*graha, placed.sign(), sign).is_full())
        .map(|(graha, _)| graha)
}

/// The verdict's clauses and its outcome.
fn verdict(sky: &PrashnaSky, question: Question, rules: PrashnaRules) -> Verdict {
    let lagna = sky.lagna();
    let mut clauses = vec![Clause {
        kind: ClauseKind::LagnaRising,
        graha: None,
        favour: match (lagna.attributes().rising, rules.pisces) {
            (Rising::Sirshodaya, _) | (Rising::Ubhayodaya, PiscesRising::Shirshodaya) => {
                Favour::For
            }
            (Rising::Prishtodaya, _) => Favour::Against,
            _ => Favour::Both,
        },
    }];
    clauses.extend(
        sky.in_sign(lagna)
            .filter_map(|graha| by(sky, ClauseKind::InLagna, graha)),
    );
    clauses.extend(match rules.rising_varga {
        RisingVarga::Navamsha => by(
            sky,
            ClauseKind::RisingNavamsha,
            sky.lagna_navamsha().attributes().lord,
        ),
        RisingVarga::Shadvarga => Some(shadvarga(sky)),
    });
    clauses
        .extend(aspecting(sky, lagna).filter_map(|graha| by(sky, ClauseKind::AspectsLagna, graha)));
    if let Some(moon) = sky.of(Graha::Moon) {
        clauses.extend(
            aspecting(sky, moon.sign()).filter_map(|graha| by(sky, ClauseKind::AspectsMoon, graha)),
        );
    }
    if let Some(house) = question.house.and_then(|house| House::try_from(house).ok()) {
        let sign = house.sign_from(lagna);
        let lord = sign.attributes().lord;
        let held: Vec<Graha> = sky.in_sign(sign).collect();
        let aspects: Vec<Graha> = aspecting(sky, sign)
            .filter(|graha| !held.contains(graha))
            .collect();
        let mut touching = held;
        touching.extend(aspects);
        clauses.extend(touching.into_iter().filter_map(|graha| {
            if graha == lord {
                Some(Clause {
                    kind: ClauseKind::KaryaHouse,
                    graha: Some(graha),
                    favour: Favour::For,
                })
            } else {
                by(sky, ClauseKind::KaryaHouse, graha)
            }
        }));
    }
    let all = |favour: Favour| clauses.iter().all(|clause| clause.favour == favour);
    let outcome = if all(Favour::For) {
        Outcome::Succeeds
    } else if all(Favour::Against) {
        Outcome::Fails
    } else {
        Outcome::WithDifficulty
    };
    Verdict { clauses, outcome }
}

/// *Shatpanchashika* II.1–2 on the lagna's modality.
fn change(sky: &PrashnaSky) -> Change {
    match sky.lagna().attributes().modality {
        Modality::Sthira => Change::Stays,
        Modality::Chara => Change::Changes,
        // A dual sign is fixed in its first half and movable in its
        // second, as the 1941 gloss and Chintamani vv. 17–18 say.
        _ if sky.lagna_deg.rem_euclid(30.0) < 15.0 => Change::Stays,
        _ => Change::Changes,
    }
}

/// The multiplier II.15 gives a navāṁśa's modality.
fn by_modality(sign: Rashi) -> u8 {
    match sign.attributes().modality {
        Modality::Chara => 1,
        Modality::Sthira => 2,
        _ => 3,
    }
}

/// An internal error for a sky that places none of the seven, which
/// [`PrashnaSky`]'s fixed arrays make impossible.
fn unplaced() -> Error {
    Error::internal("a prashna sky places the seven grahas")
}

/// When the matter comes to pass, under `rule`.
fn timing(sky: &PrashnaSky, rule: TimingRule) -> Result<Timing, Error> {
    let lagna = sky.lagna();
    let timed = |graha: Graha, tie: bool, count: u8, multiplier: u8, unit: Unit| Timing {
        rule,
        graha,
        tie,
        count,
        multiplier,
        amount: Some(u16::from(count) * u16::from(multiplier)),
        unit,
        between: Vec::new(),
    };
    match rule {
        TimingRule::StrongestGraha => {
            let (graha, tie) = sky.strongest(SEVEN).ok_or_else(unplaced)?;
            let placed = sky.of(graha).ok_or_else(unplaced)?;
            let count = sky.house_of(graha).ok_or_else(unplaced)?;
            Ok(timed(
                graha,
                tie,
                count,
                by_modality(placed.navamsha),
                Unit::Months,
            ))
        }
        TimingRule::FirstOccupied => {
            let (count, graha, tie) = House::ALL
                .into_iter()
                .find_map(|house| {
                    let sign = house.sign_from(lagna);
                    sky.strongest(sky.in_sign(sign))
                        .map(|(graha, tie)| (house.get(), graha, tie))
                })
                .ok_or_else(unplaced)?;
            let retrograde = sky.of(graha).is_some_and(|placed| placed.retrograde);
            Ok(timed(
                graha,
                tie,
                count,
                if retrograde { 1 } else { 12 },
                Unit::Days,
            ))
        }
        TimingRule::MoonDays => {
            let count = sky.house_of(Graha::Moon).ok_or_else(unplaced)?;
            let between: Vec<Graha> = (2..count)
                .filter_map(|house| House::try_from(house).ok())
                .flat_map(|house| sky.in_sign(house.sign_from(lagna)).collect::<Vec<_>>())
                .collect();
            let mut answer = timed(Graha::Moon, false, count, 1, Unit::Days);
            if !between.is_empty() {
                answer.amount = None;
            }
            answer.between = between;
            Ok(answer)
        }
        TimingRule::Baseline => {
            let lord = lagna.attributes().lord;
            let count = sky.house_of(lord).ok_or_else(unplaced)?;
            let unit = match lagna.attributes().modality {
                Modality::Chara => Unit::Days,
                Modality::Sthira => Unit::Months,
                _ => Unit::Years,
            };
            Ok(timed(lord, false, count, 1, unit))
        }
    }
}

/// The person VII.7–8 names for a house.
fn person(sky: &PrashnaSky, graha: Graha, house: u8) -> Person {
    match house {
        1 => {
            let lord = sky.lagna_navamsha().attributes().lord;
            let held = lord.attributes();
            if graha == lord {
                Person::Querent
            } else if held.friends.contains(&graha) {
                Person::Friend
            } else if held.enemies.contains(&graha) {
                Person::Enemy
            } else {
                Person::Querent
            }
        }
        3 => Person::Brother,
        4 => Person::MotherOrSister,
        5 => Person::Son,
        6 => Person::Enemy,
        7 => Person::Wife,
        9 => Person::Religious,
        10 => Person::Guru,
        _ => Person::Unspecified,
    }
}

/// The class I.7 gives the lagna's navāṁśa: mineral, root and living in
/// turn from the start of an odd sign, the reverse in an even one.
fn thought(sky: &PrashnaSky) -> Thought {
    let within = sky.lagna_deg.rem_euclid(30.0) / (30.0 / 9.0);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a navamsha index in 0..9"
    )]
    let part = (within.floor() as u8) % 3;
    let odd = sky.lagna().attributes().parity == Parity::Odd;
    match (part, odd) {
        (0, true) | (2, false) => Thought::Mineral,
        (1, _) => Thought::Root,
        _ => Thought::Living,
    }
}

/// What an unspoken question is about, under `rule`.
fn mook(sky: &PrashnaSky, rule: MookRule) -> Result<Mook, Error> {
    let (graha, tie) = match rule {
        MookRule::Shatpanchashika => sky.strongest(SEVEN).ok_or_else(unplaced)?,
        MookRule::MoonHouse => {
            let lord = sky.lagna().attributes().lord;
            let stronger = sky.strength_of(lord) > sky.strength_of(Graha::Moon);
            (if stronger { lord } else { Graha::Moon }, false)
        }
        MookRule::Baseline => baseline::topic(sky)?,
    };
    let house = sky.house_of(graha).ok_or_else(unplaced)?;
    Ok(Mook {
        rule,
        graha,
        tie,
        house,
        person: (rule == MookRule::Shatpanchashika).then(|| person(sky, graha, house)),
        thought: thought(sky),
    })
}

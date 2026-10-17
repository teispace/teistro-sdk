//! The tithi pravesha: each year's instant the Moon's elongation from the
//! Sun returns to the birth's while the Sun is back in its natal sign
//! (`03-design/tithi-pravesha.md`).
//!
//! A phase return chosen by a rule. The returns are the hit list's own
//! (`Founder::elongation_events`, C420); the year they are chosen for is
//! the Varsha Pravesha's (`03-design/annual-chart.md`); what this adds is
//! the choice, which is later practice and not a verse (C443).

use serde::{Deserialize, Serialize};
use teistro_astro::events::Lattice;
use teistro_chart::foundation::TransitEvent;
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_port_ephemeris::EphemerisProvider as _;
use teistro_serial::Document;
use teistro_tajika::Reading;

use crate::area::ChartArea;
use crate::ephemeris::no_ephemeris;
use crate::hit_request::natal_elongation_deg;

/// How far either side of a year's solar return the phase returns are
/// searched, days.
///
/// The Sun stays in a sign at most about 31.5 days, so a window this wide
/// holds the whole sojourn the solar return falls in, whichever end of it
/// the return is near; and a synodic month is at most about 29.9 days, so
/// it always holds a phase return on either side of the solar return.
pub const TITHI_WINDOW_DAYS: f64 = 33.0;

/// Which phase return of the year is its tithi pravesha (C444, C445).
///
/// ```
/// use teistro::TithiRule;
///
/// assert_eq!(TithiRule::default(), TithiRule::NatalSign);
/// assert_eq!(TithiRule::NatalSign.key(), "NATAL_SIGN");
/// assert_eq!(TithiRule::ALL.len(), 2);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TithiRule {
    /// The phase return while the Sun is in the sign it held at birth,
    /// the sojourn the year's solar return falls in: the practice's own
    /// words, "when the Sun returns back to the same sign … and the exact
    /// amount of the birth tithi has elapsed" (C443), and the default.
    ///
    /// The sojourn is longer than a synodic month in some signs and
    /// shorter in others, so it can hold two phase returns or none. Of
    /// two, the one nearer the solar return is taken (C445); with none,
    /// the year falls back to [`TithiRule::NearestReturn`] and says so in
    /// [`TithiPravesha::picked`] (C446).
    #[default]
    NatalSign,
    /// The phase return nearest the year's solar return, whatever sign
    /// the Sun is in: always one, always within about fifteen days of the
    /// birthday by the Sun.
    NearestReturn,
}

impl TithiRule {
    /// Every rule, the practice's first.
    pub const ALL: [TithiRule; 2] = [TithiRule::NatalSign, TithiRule::NearestReturn];

    /// The member's key, as serde writes it: `NATAL_SIGN`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            TithiRule::NatalSign => "NATAL_SIGN",
            TithiRule::NearestReturn => "NEAREST_RETURN",
        }
    }
}

/// One year's tithi pravesha: the instant, the solar return it was chosen
/// for, and which rule chose it.
///
/// The chart is the caller's to found at [`TithiPravesha::at`], where
/// the native is, as a year's chart is (`03-design/annual-chart.md`).
///
/// ```
/// use teistro::{TithiPravesha, TithiRule};
/// use teistro::quantity::{JulianDay, Utc};
///
/// let year = TithiPravesha {
///     year: 1,
///     at: JulianDay::<Utc>::literal(2_448_355.0),
///     solar_return: JulianDay::<Utc>::literal(2_448_360.7),
///     elongation_deg: 251.4,
///     rule: TithiRule::NatalSign,
///     picked: TithiRule::NatalSign,
///     in_sign: 1,
/// };
/// // Under six days before the birthday by the Sun.
/// assert!(year.at.get() < year.solar_return.get());
/// assert!(!year.fell_back());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TithiPravesha {
    /// The solar return this year is chosen around, counted as
    /// [`Pravesha::year`](crate::Pravesha::year) counts it: `1` is the first
    /// return, a year after birth, and opens the native's second year of
    /// life.
    pub year: u16,
    /// The instant the elongation returns, UTC: the chart is founded here.
    pub at: JulianDay<Utc>,
    /// The year's solar return, UTC, in the chart's own zodiac
    /// ([`Reading::Sidereal`]): what the rule measures "nearest" from.
    pub solar_return: JulianDay<Utc>,
    /// The elongation returned to, degrees in `[0, 360)`: the Moon's
    /// longitude less the Sun's at birth, the directed angle (C420).
    pub elongation_deg: f64,
    /// The rule asked for.
    pub rule: TithiRule,
    /// The rule that chose this instant: the one asked for, or
    /// [`TithiRule::NearestReturn`] where [`TithiRule::NatalSign`] found
    /// no phase return inside the sign (C446).
    pub picked: TithiRule,
    /// How many phase returns fell while the Sun was in its natal sign
    /// around this solar return: 0, 1 or 2. Counted under either rule, so
    /// a caller can tell an ordinary year from one the rule had to judge.
    pub in_sign: u8,
}

impl TithiPravesha {
    /// Whether [`TithiRule::NatalSign`] found nothing in the sign and the
    /// year fell back to the nearest return (C446).
    ///
    /// ```
    /// use teistro::{TithiPravesha, TithiRule};
    /// use teistro::quantity::{JulianDay, Utc};
    ///
    /// let empty = TithiPravesha {
    ///     year: 7,
    ///     at: JulianDay::<Utc>::literal(2_450_550.2),
    ///     solar_return: JulianDay::<Utc>::literal(2_450_557.1),
    ///     elongation_deg: 12.0,
    ///     rule: TithiRule::NatalSign,
    ///     picked: TithiRule::NearestReturn,
    ///     in_sign: 0,
    /// };
    /// assert!(empty.fell_back());
    /// ```
    #[must_use]
    pub fn fell_back(&self) -> bool {
        self.rule != self.picked
    }
}

impl ChartArea<'_> {
    /// Each year's **tithi pravesha**, `1` through `through`: the instant
    /// the Moon's elongation from the Sun returns to the birth's, chosen
    /// by `rule` around each year's solar return
    /// (`03-design/tithi-pravesha.md`).
    ///
    /// The phase returns are the hit list's (`PHASE_RETURN`, C420): the
    /// directed elongation, seen from the birth place under a topocentric
    /// profile, as the birth's own was (C447). The solar returns are
    /// [`ChartArea::praveshas`]' under [`Reading::Sidereal`], the chart's
    /// own zodiac, and so is the Sun's sign. The chart is the caller's to
    /// found at each instant, where the native is.
    ///
    /// Fewer years than asked for is the answer, as it is for the solar
    /// returns: a year whose window passes the end of the ephemeris is
    /// left out rather than refused with an instant nobody asked about.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Document, Ephemeris, TithiRule};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (birth, request): (Document, ChartRequest) = todo!();
    /// let years = sdk.chart().tithi_praveshas(&birth, TithiRule::NatalSign, 40)?;
    /// for year in &years {
    ///     // The year's chart, founded where the native is.
    ///     let _chart = sdk.chart().reading(year.at, &request)?.value;
    ///     if year.fell_back() {
    ///         println!("year {}: no return in the sign, the nearest taken", year.year);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A `through` outside one to two hundred, `INVALID_ARG` naming
    /// `through`; no ephemeris; a birth chart without the Sun or the Moon,
    /// naming `birth`; whatever the provider refuses while searching.
    pub fn tithi_praveshas(
        self,
        birth: &Document,
        rule: TithiRule,
        through: u16,
    ) -> Result<Vec<TithiPravesha>, Error> {
        let returns = self.praveshas(birth, Reading::Sidereal, through)?;
        let foundation = &birth.foundation;
        let at = |graha: Graha| {
            foundation
                .graha(graha)
                .map(|place| place.longitude_deg)
                .ok_or_else(|| {
                    Error::invalid_arg(format!(
                        "the birth chart places no {}, which a tithi is read from",
                        graha.key()
                    ))
                    .with_field("birth")
                    .with_hint("found the birth chart with the seven grahas, as a reading does")
                })
        };
        let elongation_deg = natal_elongation_deg(at(Graha::Moon)?, at(Graha::Sun)?);
        // A year whose window runs off the ephemeris is left out, as a
        // solar return past it is: the provider's own refusal would name
        // a day the caller never mentioned.
        let covered = self
            .context()
            .ephemeris()
            .ok_or_else(no_ephemeris)?
            .capabilities()
            .jd_range
            .1;
        let place = foundation.place;
        let phase = [Lattice::single(elongation_deg)];
        self.founding(UtcOffset::UTC, |founder| {
            returns
                .iter()
                .take_while(|one| one.at.get() + TITHI_WINDOW_DAYS < covered)
                .map(|one| {
                    let window = (
                        JulianDay::<Utc>::literal(one.at.get() - TITHI_WINDOW_DAYS),
                        JulianDay::<Utc>::literal(one.at.get() + TITHI_WINDOW_DAYS),
                    );
                    let phases: Vec<f64> = founder
                        .elongation_events(&place, &phase, window)?
                        .value
                        .iter()
                        .map(|event| event.instant.get())
                        .collect();
                    let ingresses = founder
                        .transit_events(&place, &[Graha::Sun], &[Lattice::SIGNS], false, window)?
                        .value;
                    let sojourn = sojourn(&ingresses, one.at.get(), window);
                    choose(
                        &phases,
                        sojourn,
                        rule,
                        Year {
                            year: one.year,
                            solar_return: one.at,
                            elongation_deg,
                        },
                    )
                    .ok_or_else(|| {
                        Error::internal(format!(
                            "no phase return within {TITHI_WINDOW_DAYS} days of year {}'s solar \
                             return",
                            one.year
                        ))
                    })
                })
                .collect::<Result<Vec<_>, Error>>()
        })
    }
}

/// The Sun's sojourn in the sign it stands in at `solar_return`, which
/// is its natal sign, a solar return being the Sun on its natal
/// longitude: from the last sign ingress before it to the first after
/// it, Julian days. The window is wider than any sojourn, so both ends
/// are inside it; were one not, the window's own end stands in for it.
fn sojourn(
    ingresses: &[TransitEvent],
    solar_return: f64,
    window: (JulianDay<Utc>, JulianDay<Utc>),
) -> (f64, f64) {
    let entered = ingresses
        .iter()
        .map(|event| event.instant.get())
        .filter(|instant| *instant <= solar_return)
        .fold(window.0.get(), f64::max);
    let left = ingresses
        .iter()
        .map(|event| event.instant.get())
        .filter(|instant| *instant > solar_return)
        .fold(window.1.get(), f64::min);
    (entered, left)
}

/// What a year's choice is made around.
struct Year {
    year: u16,
    solar_return: JulianDay<Utc>,
    elongation_deg: f64,
}

/// The instant of `instants` nearest `to`; of two equally near, the
/// earlier, `min_by` keeping the first of a tie.
fn nearest(instants: &[f64], to: f64) -> Option<f64> {
    instants
        .iter()
        .copied()
        .min_by(|a, b| (a - to).abs().total_cmp(&(b - to).abs()))
}

/// The year's tithi pravesha among its phase returns, `None` only when
/// the window held none, which a window wider than a synodic month on
/// either side of the solar return cannot.
fn choose(
    phases: &[f64],
    (entered, left): (f64, f64),
    rule: TithiRule,
    year: Year,
) -> Option<TithiPravesha> {
    let solar = year.solar_return.get();
    let in_sign: Vec<f64> = phases
        .iter()
        .copied()
        .filter(|instant| (entered..left).contains(instant))
        .collect();
    let (at, picked) = match rule {
        TithiRule::NatalSign => match nearest(&in_sign, solar) {
            Some(at) => (at, TithiRule::NatalSign),
            None => (nearest(phases, solar)?, TithiRule::NearestReturn),
        },
        TithiRule::NearestReturn => (nearest(phases, solar)?, TithiRule::NearestReturn),
    };
    Some(TithiPravesha {
        year: year.year,
        at: JulianDay::<Utc>::literal(at),
        solar_return: year.solar_return,
        elongation_deg: year.elongation_deg,
        rule,
        picked,
        // At most two: the longest sojourn is shorter than two synodic
        // months.
        in_sign: u8::try_from(in_sign.len()).unwrap_or(u8::MAX),
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "tests fail by panicking and compare instants they wrote"
    )]

    use super::*;

    fn year() -> Year {
        Year {
            year: 3,
            solar_return: JulianDay::<Utc>::literal(100.0),
            elongation_deg: 40.0,
        }
    }

    #[test]
    fn one_return_in_the_sign_is_the_year_s() {
        // The Sun entered the sign a day before its solar return at 100;
        // the one return inside the sojourn is taken even though the one
        // before the ingress is nearer the solar return.
        let phases = [98.5, 127.8];
        let sign = (99.0, 129.0);
        let found = choose(&phases, sign, TithiRule::NatalSign, year()).unwrap();
        assert_eq!(found.at.get(), 127.8);
        assert_eq!(found.picked, TithiRule::NatalSign);
        assert_eq!(found.in_sign, 1);
        assert!(!found.fell_back());
        // The rival ignores the sign and takes the nearer.
        let nearest = choose(&phases, sign, TithiRule::NearestReturn, year()).unwrap();
        assert_eq!(nearest.at.get(), 98.5);
        assert_eq!(nearest.picked, TithiRule::NearestReturn);
        assert_eq!(nearest.in_sign, 1, "counted under either rule");
        assert!(!nearest.fell_back());
    }

    #[test]
    fn of_two_in_the_sign_the_nearer_the_solar_return_is_taken() {
        let found = choose(
            &[71.0, 98.0, 127.0],
            (70.0, 101.5),
            TithiRule::NatalSign,
            year(),
        )
        .unwrap();
        assert_eq!(found.at.get(), 98.0);
        assert_eq!(found.in_sign, 2);
        let found = choose(&[70.5, 99.9], (70.0, 101.5), TithiRule::NatalSign, year()).unwrap();
        assert_eq!(found.at.get(), 99.9);
    }

    #[test]
    fn none_in_the_sign_falls_back_to_the_nearest_and_says_so() {
        let found = choose(&[85.0, 114.6], (86.0, 114.5), TithiRule::NatalSign, year()).unwrap();
        assert_eq!(found.at.get(), 114.6);
        assert_eq!(found.picked, TithiRule::NearestReturn);
        assert_eq!(found.in_sign, 0);
        assert!(found.fell_back());
    }

    #[test]
    fn a_window_with_no_return_answers_nothing() {
        assert!(choose(&[], (90.0, 120.0), TithiRule::NearestReturn, year()).is_none());
    }

    #[test]
    fn a_rule_is_spelt_as_serde_writes_it() {
        for rule in TithiRule::ALL {
            let json = serde_json::to_string(&rule).unwrap();
            assert_eq!(json, format!("\"{}\"", rule.key()));
        }
    }
}

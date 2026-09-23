//! What an annual chart's longitudes cannot say: which of the seven are
//! **retrograde** and which **combust** (`03-design/tajika-yogas.md`,
//! "The input shape this decides").
//!
//! Three of the sixteen yogas turn on them — Rudda, Duhphali-kuttha and
//! Durapha — and nothing else in the module does, so they travel as an
//! input of their own rather than widening [`crate::AnnualSky`], which
//! `panchavargiya` and the aspects read as well and which would otherwise
//! make every caller supply data for a question it is not asking.
//!
//! They are **sets** that name their planets, not arrays of seven flags
//! in some order: an entry cannot then be read against the wrong planet,
//! and one that could not be true is refused by name.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::drishti::speed_rank;

/// Which of the seven are retrograde and which combust, in one annual
/// chart.
///
/// The façade fills it from the founded chart's own graha states, under
/// the context's combustion table; a caller of the low-level functions
/// supplies it, or leaves the three yogas that need it unanswered.
///
/// ```
/// use teistro_tajika::AnnualStates;
/// use teistro_core::catalogue::Graha;
///
/// let states = AnnualStates {
///     retrograde: vec![Graha::Saturn],
///     combust: vec![Graha::Mercury],
/// }
/// .check()?;
/// assert!(states.is_retrograde(Graha::Saturn));
/// assert!(!states.is_combust(Graha::Venus));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default)]
pub struct AnnualStates {
    /// The planets going backwards through the zodiac.
    pub retrograde: Vec<Graha>,
    /// The planets the Sun burns.
    pub combust: Vec<Graha>,
}

impl AnnualStates {
    /// Whether `graha` is retrograde.
    #[must_use]
    pub fn is_retrograde(&self, graha: Graha) -> bool {
        self.retrograde.contains(&graha)
    }

    /// Whether `graha` is combust.
    #[must_use]
    pub fn is_combust(&self, graha: Graha) -> bool {
        self.combust.contains(&graha)
    }

    /// Refuses what no chart can hold, naming the field to correct.
    ///
    /// # Errors
    ///
    /// A body outside the seven, or a luminary retrograde, named
    /// `retrograde`; a body outside the seven, or the Sun combust, named
    /// `combust`.
    pub fn check(self) -> Result<AnnualStates, Error> {
        self.validate()?;
        Ok(self)
    }

    /// [`AnnualStates::check`] on a borrowed value, for a judgement that
    /// reads states it does not own.
    pub(crate) fn validate(&self) -> Result<(), Error> {
        let refuse =
            |field: &str, message: String| Err(Error::invalid_arg(message).with_field(field));
        for graha in &self.retrograde {
            if speed_rank(*graha).is_none() {
                return refuse("retrograde", format!("{graha:?} is not one of the seven"));
            }
            if matches!(graha, Graha::Sun | Graha::Moon) {
                return refuse(
                    "retrograde",
                    format!("{graha:?} is never retrograde: the luminaries only go forwards"),
                );
            }
        }
        for graha in &self.combust {
            if speed_rank(*graha).is_none() {
                return refuse("combust", format!("{graha:?} is not one of the seven"));
            }
            if *graha == Graha::Sun {
                return refuse("combust", String::from("the Sun cannot burn itself"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::AnnualStates;
    use teistro_core::catalogue::Graha;

    #[test]
    fn a_state_is_asked_by_name() {
        let states = AnnualStates {
            retrograde: vec![Graha::Mars, Graha::Saturn],
            combust: vec![Graha::Mercury],
        }
        .check()
        .unwrap();
        assert!(states.is_retrograde(Graha::Mars));
        assert!(!states.is_retrograde(Graha::Jupiter));
        assert!(states.is_combust(Graha::Mercury));
        assert!(!states.is_combust(Graha::Mars));
        assert_eq!(
            AnnualStates::default().check().unwrap(),
            AnnualStates::default()
        );
    }

    #[test]
    fn what_no_chart_can_hold_is_refused_by_field() {
        let refused = |states: AnnualStates| states.check().unwrap_err().field().map(String::from);
        let retrograde = |graha| AnnualStates {
            retrograde: vec![graha],
            ..AnnualStates::default()
        };
        let combust = |graha| AnnualStates {
            combust: vec![graha],
            ..AnnualStates::default()
        };
        assert_eq!(
            refused(retrograde(Graha::Sun)).as_deref(),
            Some("retrograde")
        );
        assert_eq!(
            refused(retrograde(Graha::Moon)).as_deref(),
            Some("retrograde")
        );
        assert_eq!(
            refused(retrograde(Graha::Rahu)).as_deref(),
            Some("retrograde")
        );
        assert_eq!(refused(combust(Graha::Sun)).as_deref(), Some("combust"));
        assert_eq!(refused(combust(Graha::Ketu)).as_deref(), Some("combust"));
        // The Moon can be burnt, and every planet but the luminaries can
        // turn back.
        assert!(combust(Graha::Moon).check().is_ok());
        for graha in [
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
        ] {
            assert!(retrograde(graha).check().is_ok(), "{graha:?}");
        }
    }
}

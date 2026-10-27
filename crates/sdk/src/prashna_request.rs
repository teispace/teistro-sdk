//! Prashna through the façade: the chart cast for the moment of a question,
//! read as *Shatpanchashika* and Tajika Nilakanthi print it
//! (`03-design/prashna.md`).
//!
//! The chart is the caller's: a prashna is about the sky that was, at the
//! place it was asked, so nothing is re-cast. What the kernel reads off it
//! is its lagna and that lagna's navāṁśa, the nine grahas with their
//! motion and combustion, and the seven's Shadbala, which the document
//! carries only when the request asked for it.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_prashna::{
    GRAHAS, Placed, Prashna, PrashnaRules, PrashnaSky, Question, SHADVARGA, read,
};
use teistro_serial::Document;
use teistro_state::state;

use crate::ChartArea;
use crate::rules_bridge::{navamsha_of, varga_sign};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const PRASHNA: &str = "prashna";

/// The seven whose Shadbala the kernel weighs, in the order
/// [`PrashnaSky::strength`] holds them.
const SEVEN: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// A question put to the chart of its moment, and the readings it is
/// answered under.
///
/// ```
/// use teistro::PrashnaRequest;
/// use teistro::prashna::{MookRule, PrashnaRules, Question};
///
/// let asked = PrashnaRequest::new(Question::about(7), PrashnaRules::default());
/// assert_eq!(asked.question.house, Some(7));
/// let read = PrashnaRequest::from_json(r#"{"question": {"house": 7}, "rules": {"mook": "MOON_HOUSE"}}"#)?;
/// assert_eq!(read.rules.mook, MookRule::MoonHouse);
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PrashnaRequest {
    /// What was asked: the house the matter belongs to and the querent's
    /// number, each when given.
    pub question: Question,
    /// The readings; the texts' own when left out, and `"BASELINE"` names
    /// [`PrashnaRules::baseline`], every reading the baseline engine takes.
    #[serde(deserialize_with = "teistro_core::preset::read")]
    #[cfg_attr(
        feature = "schema",
        schemars(schema_with = "teistro_core::preset::schema::<PrashnaRules>")
    )]
    pub rules: PrashnaRules,
}

impl PrashnaRequest {
    /// A question under the given readings.
    #[must_use]
    pub const fn new(question: Question, rules: PrashnaRules) -> PrashnaRequest {
        PrashnaRequest { question, rules }
    }

    /// The record a binding sends, as JSON: `question` (`house`,
    /// `number`) and `rules`, every member optional, or `"BASELINE"` for
    /// [`PrashnaRules::baseline`].
    ///
    /// ```
    /// use teistro::PrashnaRequest;
    /// use teistro::prashna::PrashnaRules;
    ///
    /// let baseline = PrashnaRequest::from_json(r#"{"question": {"house": 7}, "rules": "BASELINE"}"#)?;
    /// assert_eq!(baseline.rules, PrashnaRules::baseline());
    /// let typo = PrashnaRequest::from_json(r#"{"rules": {"timeing": "BASELINE"}}"#).unwrap_err();
    /// assert_eq!(typo.field(), Some("prashna.rules.timeing"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, or a key it does not
    /// read, named under `prashna`.
    pub fn from_json(text: &str) -> Result<PrashnaRequest, Error> {
        teistro_core::strict::read(text, PRASHNA)
    }
}

impl ChartArea<'_> {
    /// The query chart as the prashna kernel reads it: the lagna and its
    /// navāṁśa, the nine grahas with their motion and combustion under
    /// the context's settings, and the seven's Shadbala in rupas.
    ///
    /// # Errors
    ///
    /// A document without its Shadbala, naming the section to ask for
    /// (`shadbala`): the strongest graha times the matter and reads the
    /// unspoken question, so a prashna cannot be read without it.
    pub fn prashna_sky(self, document: &Document) -> Result<PrashnaSky, Error> {
        let foundation = &document.foundation;
        let shadbala = document.shadbala.as_ref().ok_or_else(|| {
            Error::invalid_arg(
                "the document carries no Shadbala, which a prashna reads for its strongest \
                 graha; ask for it with `ChartRequest::with_shadbala`",
            )
            .with_field("shadbala")
        })?;
        let states = state(foundation, self.context().settings())?;
        let mut grahas = [Placed {
            longitude_deg: 0.0,
            navamsha: teistro_core::catalogue::Rashi::Aries,
            retrograde: false,
            combust: false,
        }; 9];
        for (slot, graha) in grahas.iter_mut().zip(GRAHAS) {
            let at = foundation.graha(graha).ok_or_else(|| {
                Error::internal(format!("a founded chart places {}", graha.key()))
            })?;
            let held = states.iter().find(|one| one.graha == graha);
            *slot = Placed {
                longitude_deg: at.longitude_deg,
                navamsha: navamsha_of(at.longitude_deg)?,
                retrograde: held.is_some_and(|one| one.motion.retrograde),
                combust: held.is_some_and(|one| one.combustion.is_combust()),
            };
        }
        let mut strength = [0.0; 7];
        for (slot, graha) in strength.iter_mut().zip(SEVEN) {
            *slot = shadbala
                .grahas
                .iter()
                .find(|one| one.graha == graha)
                .map(|one| one.rupas)
                .ok_or_else(|| {
                    Error::internal(format!("a Shadbala reading weighs {}", graha.key()))
                })?;
        }
        Ok(PrashnaSky {
            lagna_deg: foundation.lagna_deg,
            lagna_vargas: lagna_vargas(foundation.lagna_deg)?,
            grahas,
            strength,
        })
    }

    /// A question read off the chart of its moment
    /// (`03-design/prashna.md`): the verdict's clauses, whether the matter
    /// stays, when, what an unspoken question is about, the Tajika links
    /// when a house is asked, and the Moon's weaknesses.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::prashna_sky`]; a house outside 1 to 12 or a number
    /// outside 1 to 108, named under `prashna`.
    pub fn prashna(self, document: &Document, asked: &PrashnaRequest) -> Result<Prashna, Error> {
        read(&self.prashna_sky(document)?, asked.question, asked.rules)
            .map_err(|error| error.under(PRASHNA))
    }
}

/// The lagna's sign in each of the [`SHADVARGA`], under the SDK's classical
/// scheme for each, as the grahas' navāṁśas are read.
fn lagna_vargas(lagna_deg: f64) -> Result<[Rashi; 6], Error> {
    let mut signs = [Rashi::Aries; 6];
    for (slot, varga) in signs.iter_mut().zip(SHADVARGA) {
        *slot = varga_sign(varga, lagna_deg)?;
    }
    Ok(signs)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

    use super::*;

    #[test]
    fn baseline_named_reads_as_the_rules_spelt_out() {
        let named = PrashnaRequest::from_json(r#"{"question": {"house": 7}, "rules": "BASELINE"}"#)
            .unwrap();
        let spelt = serde_json::to_string(&PrashnaRules::baseline()).unwrap();
        let written = PrashnaRequest::from_json(&format!(
            r#"{{"question": {{"house": 7}}, "rules": {spelt}}}"#
        ))
        .unwrap();
        assert_eq!(named, written);
        assert_eq!(
            named,
            PrashnaRequest::new(Question::about(7), PrashnaRules::baseline())
        );
    }

    #[test]
    fn a_name_that_is_no_preset_is_refused_by_the_field() {
        let wrong = PrashnaRequest::from_json(r#"{"rules": "baseline"}"#).unwrap_err();
        assert_eq!(wrong.field(), Some("prashna.rules"));
        assert!(wrong.message.contains("BASELINE"), "{}", wrong.message);
        let typo = PrashnaRequest::from_json(r#"{"rules": {"timeing": "BASELINE"}}"#).unwrap_err();
        assert_eq!(typo.field(), Some("prashna.rules.timeing"));
    }
}

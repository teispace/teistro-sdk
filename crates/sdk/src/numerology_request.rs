//! Numerology as a binding asks it (`03-design/numerology.md`, C320 to
//! C328).

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_numerology::{BirthDate, NumerologyRules, Profile, profile};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const NUMEROLOGY: &str = "numerology";

/// A name and a birth date, and the readings they are taken under.
/// Numerology reads no sky, so this is the whole request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NumerologyRequest {
    /// The name, in the 26 Latin letters.
    pub name: String,
    /// The Gregorian birth date, `{"year", "month", "day"}`.
    pub date: BirthDate,
    /// The readings; the sources' own when left out, and `"BASELINE"`
    /// names [`NumerologyRules::baseline`], every reading the baseline
    /// engine takes.
    #[serde(default, deserialize_with = "teistro_core::preset::read")]
    #[cfg_attr(
        feature = "schema",
        schemars(schema_with = "teistro_core::preset::schema::<NumerologyRules>")
    )]
    pub rules: NumerologyRules,
}

impl NumerologyRequest {
    /// The record a binding sends, as JSON: `name`, `date` and `rules`,
    /// the [`NumerologyRules`] with every field optional, or `"BASELINE"`
    /// for [`NumerologyRules::baseline`].
    ///
    /// ```
    /// use teistro::NumerologyRequest;
    /// use teistro::numerology::{Masters, NumerologyRules};
    ///
    /// let baseline = NumerologyRequest::from_json(
    ///     r#"{"name": "Henry Elder", "date": {"year": 1872, "month": 1, "day": 17}, "rules": "BASELINE"}"#,
    /// )?;
    /// assert_eq!(baseline.rules, NumerologyRules::baseline());
    ///
    /// let asked = NumerologyRequest::from_json(
    ///     r#"{"name": "Henry Elder", "date": {"year": 1872, "month": 1, "day": 17},
    ///         "rules": {"masters": "NONE"}}"#,
    /// )?;
    /// assert_eq!(asked.rules.masters, Masters::None);
    /// let typo = NumerologyRequest::from_json(
    ///     r#"{"name": "Henry", "date": {"year": 1872, "month": 1, "day": 17}, "rules": {"master": "NONE"}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(typo.field(), Some("numerology.rules.master"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or a date the Gregorian calendar does not have, named under
    /// `numerology`.
    pub fn from_json(text: &str) -> Result<NumerologyRequest, Error> {
        teistro_core::strict::read(text, NUMEROLOGY)
    }

    /// Everything numerology says of the name and the date.
    ///
    /// ```
    /// use teistro::NumerologyRequest;
    ///
    /// let asked = NumerologyRequest::from_json(
    ///     r#"{"name": "Henry Elder", "date": {"year": 1872, "month": 1, "day": 17}}"#,
    /// )?;
    /// let read = asked.answer()?;
    /// assert_eq!(read.pythagorean_name.reduction.number, 6);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A name refused by [`teistro_numerology::name_number`], named under
    /// `numerology.name`.
    pub fn answer(&self) -> Result<Profile, Error> {
        profile(&self.name, self.date, &self.rules).map_err(|error| error.under(NUMEROLOGY))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

    use super::*;

    const DATE: &str = r#""date": {"year": 1872, "month": 1, "day": 17}"#;

    #[test]
    fn baseline_named_reads_and_answers_as_the_rules_spelt_out() {
        let named = NumerologyRequest::from_json(&format!(
            r#"{{"name": "Henry Elder", {DATE}, "rules": "BASELINE"}}"#
        ))
        .unwrap();
        let spelt = serde_json::to_string(&NumerologyRules::baseline()).unwrap();
        let written = NumerologyRequest::from_json(&format!(
            r#"{{"name": "Henry Elder", {DATE}, "rules": {spelt}}}"#
        ))
        .unwrap();
        assert_eq!(named, written);
        assert_eq!(named.rules, NumerologyRules::baseline());
        assert_eq!(named.answer().unwrap(), written.answer().unwrap());
        assert!(
            named.answer().unwrap().baseline.is_some(),
            "the baseline's own numbers"
        );
    }

    #[test]
    fn a_name_that_is_no_preset_is_refused_by_the_field_naming_the_presets() {
        let wrong = NumerologyRequest::from_json(&format!(
            r#"{{"name": "Henry", {DATE}, "rules": "BASELIN"}}"#
        ))
        .unwrap_err();
        assert_eq!(wrong.field(), Some("numerology.rules"));
        assert!(wrong.message.contains("BASELINE"), "{}", wrong.message);
        let typo = NumerologyRequest::from_json(&format!(
            r#"{{"name": "Henry", {DATE}, "rules": {{"master": "NONE"}}}}"#
        ))
        .unwrap_err();
        assert_eq!(typo.field(), Some("numerology.rules.master"));
    }
}

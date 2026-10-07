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
#[serde(rename_all = "camelCase")]
pub struct NumerologyRequest {
    /// The name, in the 26 Latin letters.
    pub name: String,
    /// The Gregorian birth date, `{"year", "month", "day"}`.
    pub date: BirthDate,
    /// The readings; the sources' own when left out.
    #[serde(default)]
    pub rules: NumerologyRules,
}

impl NumerologyRequest {
    /// The record a binding sends, as JSON: `name`, `date` and `rules`,
    /// the [`NumerologyRules`] with every field optional.
    ///
    /// ```
    /// use teistro::NumerologyRequest;
    /// use teistro::numerology::Masters;
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

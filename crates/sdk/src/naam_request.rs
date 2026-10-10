//! Two names matched as a binding asks it (`03-design/matching.md`, C291
//! to C296).

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_matching::{NaamMilan, NaamRules, naam_milan};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const NAAM: &str = "naam";

/// Two names to match star to star, and the readings they are matched
/// under. Naam milan needs no chart, so this is the whole request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NaamRequest {
    /// The bride's name, in Devanagari or, when `rules.name.latin` says
    /// so, IAST.
    pub bride: String,
    /// The groom's.
    pub groom: String,
    /// How the names are read and matched; the sources' own when left
    /// out.
    #[serde(default)]
    pub rules: NaamRules,
}

impl NaamRequest {
    /// The record a binding sends, as JSON: `bride` and `groom`, the two
    /// names, and `rules`, the [`NaamRules`] with every field optional.
    ///
    /// ```
    /// use teistro::NaamRequest;
    /// use teistro::matching::LatinName;
    ///
    /// let asked = NaamRequest::from_json(
    ///     r#"{"bride": "sītā", "groom": "rāma", "rules": {"name": {"latin": "IAST"}}}"#,
    /// )?;
    /// assert_eq!(asked.rules.name.latin, LatinName::Iast);
    /// let typo = NaamRequest::from_json(r#"{"bride": "सीता", "groom": "राम", "rules": {"nam": {}}}"#)
    ///     .unwrap_err();
    /// assert_eq!(typo.field(), Some("naam.rules.nam"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `naam`.
    pub fn from_json(text: &str) -> Result<NaamRequest, Error> {
        teistro_core::strict::read(text, NAAM)
    }

    /// The two names matched.
    ///
    /// ```
    /// use teistro::NaamRequest;
    ///
    /// let asked = NaamRequest::from_json(r#"{"bride": "सीता", "groom": "राम"}"#)?;
    /// let read = asked.answer()?;
    /// assert_eq!(read.ashta.kootas.len(), 8);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A name refused by [`teistro_matching::name_syllable`], or one in
    /// Abhijit's row while `rules.name.abhijit` refuses, named under
    /// `naam.bride` or `naam.groom`.
    pub fn answer(&self) -> Result<NaamMilan, Error> {
        naam_milan(&self.bride, &self.groom, self.rules).map_err(|error| error.under(NAAM))
    }
}

//! The navatara chakra as a binding asks it
//! (`03-design/navatara-chakra.md`).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Nakshatra;
use teistro_core::error::Error;
use teistro_muhurta::tara::{NavataraRow, navatara};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const NAVATARA: &str = "navatara";

/// A birth star, whose navatara chakra is asked. The chakra reads no sky,
/// so this is the whole request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavataraRequest {
    /// The birth star, the Moon's nakshatra at birth.
    pub birth: Nakshatra,
}

impl NavataraRequest {
    /// The record a binding sends, as JSON: `{"birth"}`, a nakshatra key.
    ///
    /// ```
    /// use teistro::NavataraRequest;
    /// use teistro::catalogue::Nakshatra;
    ///
    /// let asked = NavataraRequest::from_json(r#"{"birth": "ASHWINI"}"#)?;
    /// assert_eq!(asked.birth, Nakshatra::Ashwini);
    /// let typo = NavataraRequest::from_json(r#"{"birth": "ASHWINI", "birthStar": "MAGHA"}"#).unwrap_err();
    /// assert_eq!(typo.field(), Some("navatara.birthStar"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or a star that is not a nakshatra key, named under
    /// `navatara`.
    pub fn from_json(text: &str) -> Result<NavataraRequest, Error> {
        teistro_core::strict::read(text, NAVATARA)
    }

    /// The chakra: the 27 stars in the order the count from the birth star
    /// reaches them, each with its count, tara, round and special star, as
    /// [`AlmanacArea::navatara`](crate::AlmanacArea::navatara) answers.
    ///
    /// ```
    /// use teistro::NavataraRequest;
    /// use teistro::muhurta::tara::SpecialStar;
    ///
    /// let chakra = NavataraRequest::from_json(r#"{"birth": "ASHWINI"}"#)?.answer();
    /// assert_eq!(chakra.len(), 27);
    /// assert_eq!(chakra.iter().filter(|row| row.special.is_some()).count(), SpecialStar::ALL.len());
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn answer(&self) -> Vec<NavataraRow> {
        navatara(self.birth).to_vec()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

    use super::*;

    #[test]
    fn the_record_answers_the_kernels_chakra() {
        let asked = NavataraRequest::from_json(r#"{"birth": "REVATI"}"#).unwrap();
        assert_eq!(asked.answer(), navatara(Nakshatra::Revati).to_vec());
    }

    #[test]
    fn a_star_that_is_not_a_key_and_a_missing_birth_are_refused_by_name() {
        let unknown = NavataraRequest::from_json(r#"{"birth": "ABHIJIT"}"#).unwrap_err();
        assert_eq!(unknown.field(), Some("navatara.birth"));
        let missing = NavataraRequest::from_json("{}").unwrap_err();
        assert!(
            missing
                .field()
                .is_some_and(|field| field.starts_with("navatara")),
            "{missing:?}"
        );
        assert!(NavataraRequest::from_json(r#""ASHWINI""#).is_err());
    }
}

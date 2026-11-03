//! Aayadi as a binding asks it (`03-design/aayadi.md`).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Nakshatra;
use teistro_core::error::Error;
use teistro_muhurta::aayadi::{Aayadi, AayadiSystem, ManasaraMeasure, Plot, manasara};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const AAYADI: &str = "aayadi";

/// A building's two sides read under one system. Aayadi reads no sky, so
/// this is the whole request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AayadiRequest {
    /// The text whose reckoning is read.
    pub system: AayadiSystem,
    /// The length, 1 to 1 000 000 whole units of the caller's choosing
    /// (hasta, kol, danda or angula).
    pub length: u32,
    /// The breadth, in the same unit.
    pub breadth: u32,
    /// The owner's star, for the clauses that read it (*Samarangana*'s tara
    /// and gana, *Muhurta Chintamani*'s own star and tara); without it those
    /// clauses are not judged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<Nakshatra>,
    /// For `MANASARA` only: the measure its six reckonings take, the split
    /// by default (crux C1194).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measure: Option<ManasaraMeasure>,
}

impl AayadiRequest {
    /// The record a binding sends, as JSON: `{"system", "length",
    /// "breadth", "owner"?, "measure"?}`.
    ///
    /// ```
    /// use teistro::AayadiRequest;
    /// use teistro::muhurta::aayadi::AayadiSystem;
    ///
    /// let asked = AayadiRequest::from_json(r#"{"system": "CHINTAMANI", "length": 29, "breadth": 7}"#)?;
    /// assert_eq!((asked.system, asked.length, asked.owner), (AayadiSystem::Chintamani, 29, None));
    /// let typo = AayadiRequest::from_json(r#"{"system": "CHINTAMANI", "length": 29, "width": 7}"#).unwrap_err();
    /// assert!(typo.field().is_some_and(|field| field.starts_with("aayadi")));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or a value of the wrong kind, named under `aayadi`.
    pub fn from_json(text: &str) -> Result<AayadiRequest, Error> {
        teistro_core::strict::read(text, AAYADI)
    }

    /// The reading: each quantity the system reckons, the vyaya's kind and
    /// the tara where it reads them, and each clause it judges, as
    /// [`AlmanacArea::aayadi`](crate::AlmanacArea::aayadi) answers.
    ///
    /// ```
    /// use teistro::AayadiRequest;
    /// use teistro::catalogue::{Nakshatra, VastuYoni};
    ///
    /// let asked = AayadiRequest::from_json(r#"{"system": "CHINTAMANI", "length": 29, "breadth": 7, "owner": "ANURADHA"}"#)?;
    /// let reading = asked.answer()?;
    /// assert_eq!((reading.yoni(), reading.nakshatra()), (Some(VastuYoni::Simha), Some(Nakshatra::Rohini)));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on a side of 0 or above 1 000 000, named
    /// `aayadi.length` or `aayadi.breadth`, and on a `measure` asked of a
    /// system other than `MANASARA`, named `aayadi.measure`.
    pub fn answer(&self) -> Result<Aayadi, Error> {
        let plot = Plot::new(self.length, self.breadth).map_err(|error| error.under(AAYADI))?;
        match (self.system, self.measure) {
            (AayadiSystem::Manasara, measure) => {
                Ok(manasara(plot, self.owner, measure.unwrap_or_default()))
            }
            (system, None) => Ok(teistro_muhurta::aayadi::aayadi(system, plot, self.owner)),
            (_, Some(_)) => Err(Error::invalid_arg(
                "only MANASARA takes a measure; the others name their own",
            )
            .with_field("aayadi.measure")
            .with_hint(String::from("leave `measure` out, or ask MANASARA"))),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

    use super::*;
    use teistro_muhurta::aayadi::{ManasaraMeasure, Measure};

    #[test]
    fn the_record_answers_the_kernels_reading() {
        let asked = AayadiRequest::from_json(
            r#"{"system": "SAMARANGANA", "length": 29, "breadth": 7, "owner": "BHARANI"}"#,
        )
        .unwrap();
        let plot = Plot::new(29, 7).unwrap();
        assert_eq!(
            asked.answer().unwrap(),
            teistro_muhurta::aayadi::aayadi(
                AayadiSystem::Samarangana,
                plot,
                Some(Nakshatra::Bharani)
            )
        );
    }

    #[test]
    fn manasara_takes_a_measure_and_the_others_refuse_one() {
        let asked = AayadiRequest::from_json(
            r#"{"system": "MANASARA", "length": 29, "breadth": 7, "measure": "SUM"}"#,
        )
        .unwrap();
        let reading = asked.answer().unwrap();
        assert!(
            reading
                .quantities
                .iter()
                .all(|reckoned| reckoned.measure == Measure::Sum)
        );
        assert_eq!(asked.measure, Some(ManasaraMeasure::Sum));
        let split =
            AayadiRequest::from_json(r#"{"system": "MANASARA", "length": 29, "breadth": 7}"#)
                .unwrap();
        assert_eq!(
            split
                .answer()
                .unwrap()
                .quantities
                .first()
                .map(|reckoned| reckoned.measure),
            Some(Measure::Length)
        );
        let refused = AayadiRequest::from_json(
            r#"{"system": "MAYAMATA", "length": 29, "breadth": 7, "measure": "SUM"}"#,
        )
        .unwrap()
        .answer()
        .unwrap_err();
        assert_eq!(refused.field(), Some("aayadi.measure"));
    }

    #[test]
    fn a_side_out_of_range_and_a_wrong_system_are_refused_by_name() {
        let zero = AayadiRequest::from_json(r#"{"system": "MAYAMATA", "length": 0, "breadth": 7}"#)
            .unwrap()
            .answer()
            .unwrap_err();
        assert_eq!(zero.field(), Some("aayadi.length"));
        let wide =
            AayadiRequest::from_json(r#"{"system": "MAYAMATA", "length": 7, "breadth": 1000001}"#)
                .unwrap()
                .answer()
                .unwrap_err();
        assert_eq!(wide.field(), Some("aayadi.breadth"));
        let unknown =
            AayadiRequest::from_json(r#"{"system": "BRIHAT_SAMHITA", "length": 7, "breadth": 7}"#)
                .unwrap_err();
        assert!(
            unknown
                .field()
                .is_some_and(|field| field.starts_with("aayadi")),
            "{unknown:?}"
        );
        assert!(
            AayadiRequest::from_json(r#"{"system": "MAYAMATA", "length": -1, "breadth": 7}"#)
                .is_err()
        );
        assert!(AayadiRequest::from_json("[]").is_err());
    }
}

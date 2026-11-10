//! A guna total graded into bands and said in words, as a binding asks it
//! (`03-design/matching.md`, "Grading a total into bands", SR-39).

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_intl::{Value, params};
use teistro_matching::{GunaBand, GunaBands};

use crate::MatchingArea;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const BAND: &str = "band";

/// A total to grade, the table to grade it by and the locale to say it in.
///
/// ```
/// use teistro::BandRequest;
///
/// let asked = BandRequest::from_json(r#"{"total": 17, "dushkuta": false, "shipped": "NEPALI_ALMANAC"}"#)?;
/// assert_eq!(asked.shipped.as_deref(), Some("NEPALI_ALMANAC"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BandRequest {
    /// The total, out of 36.
    pub total: f64,
    /// Whether the two Moons' signs stand 2/12, 6/8 or 9/5 (*dushkuta*),
    /// lifted or not; `false` when left out.
    #[serde(default)]
    pub dushkuta: bool,
    /// A shipped table by name (`MUHURTA_MARTANDA`, `NEPALI_ALMANAC` or
    /// `ARYAVARTA`); or `bands`, not both. `MUHURTA_MARTANDA` when both
    /// are left out.
    #[serde(default)]
    pub shipped: Option<String>,
    /// A consumer's own table; or `shipped`, not both.
    #[serde(default)]
    pub bands: Option<GunaBands>,
    /// The locale the words are in; the context's when left out.
    #[serde(default)]
    pub locale: Option<String>,
}

/// A total's band, and the words for it.
///
/// ```
/// use teistro::BandRequest;
/// use teistro::matching::GunaBand;
///
/// let sdk = teistro::Context::builder().build()?;
/// let asked = BandRequest::from_json(r#"{"total": 28, "shipped": "NEPALI_ALMANAC", "locale": "en-Latn"}"#)?;
/// let said = sdk.matching().band_words(&asked)?;
/// assert_eq!((said.band, said.text.as_str()), (GunaBand::Good, "28 of 36 gunas: a good match"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BandWords {
    /// The total, out of 36.
    pub total: f64,
    /// Whether the signs were taken to stand 2/12, 6/8 or 9/5.
    pub dushkuta: bool,
    /// The band.
    pub band: GunaBand,
    /// The table's name.
    pub bands: String,
    /// The row of the table that gave the band, from 0.
    pub row: u32,
    /// The locale the words are in.
    pub locale: String,
    /// The band in words: the `sdk.matching.band` message.
    pub words: String,
    /// The total with its band: the `sdk.matching.graded` message.
    pub text: String,
}

impl BandRequest {
    /// The record a binding sends, as JSON: `total`; optionally
    /// `dushkuta`, `shipped` or `bands` (a [`GunaBands`] as
    /// [`GunaBands::from_json`] reads it), and `locale`.
    ///
    /// ```
    /// use teistro::BandRequest;
    ///
    /// let both = BandRequest::from_json(
    ///     r#"{"total": 20, "shipped": "ARYAVARTA", "bands": {"name": "X", "source": "", "rows": [{"band": "UNFIT", "from": 0}]}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(both.field(), Some("band.bands"));
    /// let gapped = BandRequest::from_json(
    ///     r#"{"total": 20, "bands": {"name": "X", "source": "", "rows": [{"band": "FIT", "from": 18}]}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(gapped.field(), Some("band.bands.rows[0]"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, named under `band`, on text that is not the record,
    /// a key it does not read, a total that is not a number from 0 to 36,
    /// a `shipped` name the SDK does not ship, both `shipped` and
    /// `bands`, or a table [`GunaBands::check`] refuses.
    pub fn from_json(text: &str) -> Result<BandRequest, Error> {
        let asked: BandRequest = teistro_core::strict::read(text, BAND)?;
        asked.table().map_err(|why| why.under(BAND))?;
        Ok(asked)
    }

    /// The table asked for, checked.
    fn table(&self) -> Result<GunaBands, Error> {
        if !(self.total.is_finite() && (0.0..=36.0).contains(&self.total)) {
            return Err(Error::invalid_arg(format!(
                "a total of {} is not a number of gunas from 0 to 36",
                self.total
            ))
            .with_field("total"));
        }
        match (&self.shipped, &self.bands) {
            (Some(_), Some(_)) => Err(Error::invalid_arg(
                "`shipped` and `bands` are both given; a total is graded by one table",
            )
            .with_field("bands")),
            (Some(name), None) => GunaBands::shipped(name).ok_or_else(|| {
                Error::invalid_arg(format!("no shipped table is named `{name}`"))
                    .with_field("shipped")
                    .with_hint(format!(
                        "the shipped tables are {}",
                        GunaBands::SHIPPED.join(", ")
                    ))
            }),
            (None, Some(bands)) => {
                bands.check()?;
                Ok(bands.clone())
            }
            (None, None) => Ok(GunaBands::default()),
        }
    }
}

impl MatchingArea<'_> {
    /// **A total graded and said in words**: the band `asked.total` takes
    /// in the table asked for, with `sdk.matching.band`'s words for it and
    /// `sdk.matching.graded`'s sentence, in `asked.locale`.
    ///
    /// ```
    /// use teistro::BandRequest;
    /// use teistro::matching::GunaBand;
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let asked = BandRequest::from_json(r#"{"total": 17, "dushkuta": true, "shipped": "NEPALI_ALMANAC"}"#)?;
    /// assert_eq!(sdk.matching().band_words(&asked)?.band, GunaBand::Unfit);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`BandRequest::from_json`] for a request built by hand;
    /// `UNSUPPORTED` on `band.locale` for a locale not loaded.
    pub fn band_words(self, asked: &BandRequest) -> Result<BandWords, Error> {
        let table = asked.table().map_err(|why| why.under(BAND))?;
        let graded = table.grade_total(asked.total, asked.dushkuta);
        let intl = self.context().locale_engine();
        let tag = asked
            .locale
            .clone()
            .unwrap_or_else(|| intl.locale().to_owned());
        if !intl.locales().any(|loaded| loaded.tag == tag) {
            let known: Vec<&str> = intl.locales().map(|l| l.tag.as_str()).collect();
            return Err(
                Error::unsupported(format!("the locale `{tag}` is not loaded"))
                    .with_field(format!("{BAND}.locale"))
                    .with_hint(format!("the locales are {}", known.join(", "))),
            );
        }
        let spelled = serde_json::to_value(graded.band)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_default();
        let said = params([
            ("total", Value::Num(graded.total)),
            ("band", Value::Str(spelled)),
        ]);
        let words = intl.render_in(&tag, "sdk.matching.band", &said);
        let text = intl.render_in(&tag, "sdk.matching.graded", &said);
        Ok(BandWords {
            total: graded.total,
            dushkuta: graded.dushkuta,
            band: graded.band,
            bands: graded.bands,
            row: u32::try_from(graded.row).unwrap_or(u32::MAX),
            locale: text.resolved_from.unwrap_or(tag),
            words: words.text,
            text: text.text,
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn a_request_is_refused_by_its_field() {
        let field = |text: &str| {
            BandRequest::from_json(text)
                .unwrap_err()
                .field()
                .map(str::to_owned)
        };
        assert_eq!(field(r#"{"total": 37}"#).as_deref(), Some("band.total"));
        assert_eq!(
            field(r#"{"total": 20, "shipped": "LAHIRI"}"#).as_deref(),
            Some("band.shipped")
        );
        assert_eq!(
            field(r#"{"total": 20, "dushkut": true}"#).as_deref(),
            Some("band.dushkut")
        );
    }

    #[test]
    fn every_band_is_said_in_every_locale_the_message_reaches() {
        let sdk = crate::Context::builder().build().unwrap();
        for tag in ["en-Latn", "ne-Deva-NP", "hi-Deva-IN", "sa-Deva", "sa-Latn"] {
            for (total, dushkuta) in [
                (10.0, false),
                (17.0, false),
                (19.0, true),
                (25.0, false),
                (33.0, false),
            ] {
                let asked = BandRequest {
                    total,
                    dushkuta,
                    shipped: Some("NEPALI_ALMANAC".into()),
                    bands: None,
                    locale: Some(tag.into()),
                };
                let said = sdk.matching().band_words(&asked).unwrap();
                assert_eq!(said.locale, tag);
                assert!(
                    !said.words.is_empty() && said.text.contains(&said.words),
                    "{tag}: {said:?}"
                );
            }
        }
        let nowhere = BandRequest {
            total: 20.0,
            dushkuta: false,
            shipped: None,
            bands: None,
            locale: Some("xx-Latn".into()),
        };
        assert_eq!(
            sdk.matching().band_words(&nowhere).unwrap_err().field(),
            Some("band.locale")
        );
        let own = BandRequest::from_json(
            r#"{"total": 22, "bands": {"name": "MINE", "source": "a consumer", "rows": [{"band": "UNFIT", "from": 0}, {"band": "EXCELLENT", "from": 22}]}}"#,
        )
        .unwrap();
        let said = sdk.matching().band_words(&own).unwrap();
        assert_eq!(
            (said.band, said.bands.as_str(), said.row),
            (GunaBand::Excellent, "MINE", 1)
        );
    }
}

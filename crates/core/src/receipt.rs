//! A receipt (`03-design/receipts-and-datasets.md` §1): what a record was
//! asked and under what, and the digest of what it answered, so that a
//! verifier can answer the same request again and compare (SR-51).
//!
//! Every digest here is [`content_hash`] over the canonical form the
//! envelope already writes; the receipt adds no writer of its own. The
//! seal, [`Receipt::digest`], is a digest and not a signature: it shows a
//! field changed after issue, and the re-run shows a change sealed again.

use serde::{Deserialize, Serialize};

use crate::envelope::{Hash, PackStamp, Version, content_hash};
use crate::settings::SettingsSnapshot;

/// The format a receipt names, and the only one this build reads.
///
/// ```
/// assert_eq!(teistro_core::receipt::FORMAT, "teistro.receipt/1");
/// ```
pub const FORMAT: &str = "teistro.receipt/1";

/// The ephemeris an answer was computed with, as a receipt names it.
///
/// ```
/// use teistro_core::receipt::ReceiptProvider;
///
/// let provider: ReceiptProvider =
///     serde_json::from_str(r#"{"name": "teistro-builtin", "version": "1", "dataVersion": "de440"}"#)?;
/// assert_eq!(provider.data_version, "de440");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ReceiptProvider"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReceiptProvider {
    /// The provider's name.
    pub name: String,
    /// Its version.
    pub version: String,
    /// The version of the data it read.
    pub data_version: String,
}

/// What a record was asked, under what, and the digest of what it
/// answered.
///
/// ```
/// use teistro_core::envelope::{Hash, Version, content_hash};
/// use teistro_core::receipt::Receipt;
/// use teistro_core::settings::{ProfileId, SettingsSnapshot, root};
///
/// let snapshot = SettingsSnapshot::of(&root().validated(ProfileId::new("mine")).unwrap());
/// let request = serde_json::json!({"date": {"year": 2026, "month": 10, "day": 10}});
/// let mut receipt = Receipt::new("calendar.convert", request, snapshot, Hash::of(b"value"));
/// receipt.sdk_version = Version::new(1, 0, 0);
/// let receipt = receipt.sealed();
/// assert!(receipt.broken().is_empty());
/// assert_eq!(receipt.request_digest, content_hash(&receipt.request));
/// // An edit after sealing breaks the seal.
/// let mut edited = receipt.clone();
/// edited.output_digest = Hash::of(b"another value");
/// assert_eq!(edited.broken(), ["digest"]);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Receipt {
    /// [`FORMAT`].
    pub format: String,
    /// The record's name, `area.operation`.
    pub record: String,
    /// The request as the record read it.
    pub request: serde_json::Value,
    /// The hash of the request's canonical JSON.
    pub request_digest: Hash,
    /// The settings, whole, with their profile and hash.
    pub settings: SettingsSnapshot,
    /// The locale the context answered in.
    pub locale: String,
    /// The runtime packs the context had loaded.
    pub packs: Vec<PackStamp>,
    /// The ephemeris the answer was computed with; none where it read
    /// none.
    pub provider: Option<ReceiptProvider>,
    /// The zone database's version.
    pub tzdb: String,
    /// The SDK's version.
    pub sdk_version: Version,
    /// The calculation version.
    pub calculation_version: u32,
    /// The catalogue's version.
    pub catalogue_version: u32,
    /// The hash of the answer's value, canonical JSON.
    pub output_digest: Hash,
    /// The seal: the hash of this receipt's canonical JSON with `digest`
    /// left out.
    pub digest: Hash,
}

impl Receipt {
    /// A receipt of `record` asked `request` under `settings` and
    /// answering a value whose digest is `output_digest`, unsealed: the
    /// versions are this build's, the locale, packs, provider and zone
    /// database empty, for the issuer to fill before [`Receipt::sealed`].
    ///
    /// ```
    /// # use teistro_core::envelope::Hash;
    /// # use teistro_core::receipt::Receipt;
    /// # use teistro_core::settings::{ProfileId, SettingsSnapshot, root};
    /// # let snapshot = SettingsSnapshot::of(&root().validated(ProfileId::new("mine")).unwrap());
    /// let receipt = Receipt::new("calendar.convert", serde_json::json!({}), snapshot, Hash::of(b"value"));
    /// assert_eq!(receipt.format, teistro_core::receipt::FORMAT);
    /// // Unsealed: the seal is not yet its fields'.
    /// assert_eq!(receipt.broken(), ["digest"]);
    /// ```
    #[must_use]
    pub fn new(
        record: &str,
        request: serde_json::Value,
        settings: SettingsSnapshot,
        output_digest: Hash,
    ) -> Receipt {
        Receipt {
            format: FORMAT.to_owned(),
            record: record.to_owned(),
            request_digest: content_hash(&request),
            request,
            settings,
            locale: String::new(),
            packs: Vec::new(),
            provider: None,
            tzdb: String::new(),
            sdk_version: Version::new(0, 0, 0),
            calculation_version: crate::envelope::CALCULATION_VERSION,
            catalogue_version: crate::catalogue::SCHEMA_VERSION,
            output_digest,
            digest: Hash::of(&[]),
        }
    }

    /// The receipt with its request digest and seal taken from what it
    /// holds now.
    ///
    /// ```
    /// # use teistro_core::envelope::Hash;
    /// # use teistro_core::receipt::Receipt;
    /// # use teistro_core::settings::{ProfileId, SettingsSnapshot, root};
    /// # let snapshot = SettingsSnapshot::of(&root().validated(ProfileId::new("mine")).unwrap());
    /// let mut receipt = Receipt::new("calendar.convert", serde_json::json!({}), snapshot, Hash::of(b"value"));
    /// receipt.request = serde_json::json!({"days": 3});
    /// let receipt = receipt.sealed();
    /// assert!(receipt.broken().is_empty());
    /// ```
    #[must_use]
    pub fn sealed(mut self) -> Receipt {
        self.request_digest = content_hash(&self.request);
        self.digest = self.seal();
        self
    }

    /// The seal this receipt's fields give: the hash of its canonical
    /// JSON without `digest`.
    ///
    /// ```
    /// # use teistro_core::envelope::Hash;
    /// # use teistro_core::receipt::Receipt;
    /// # use teistro_core::settings::{ProfileId, SettingsSnapshot, root};
    /// # let snapshot = SettingsSnapshot::of(&root().validated(ProfileId::new("mine")).unwrap());
    /// let receipt = Receipt::new("calendar.convert", serde_json::json!({}), snapshot, Hash::of(b"value")).sealed();
    /// assert_eq!(receipt.seal(), receipt.digest);
    /// ```
    #[must_use]
    pub fn seal(&self) -> Hash {
        let value = serde_json::to_value(self).unwrap_or_default();
        let fields: serde_json::Map<String, serde_json::Value> = value
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(name, _)| *name != "digest")
            .map(|(name, field)| (name.clone(), field.clone()))
            .collect();
        content_hash(&fields)
    }

    /// Each digest the receipt does not hold to its own fields, by the
    /// field: `requestDigest`, `settings.hash`, `digest`. Empty for a
    /// receipt as it was issued.
    ///
    /// ```
    /// # use teistro_core::envelope::Hash;
    /// # use teistro_core::receipt::Receipt;
    /// # use teistro_core::settings::{ProfileId, SettingsSnapshot, root};
    /// # let snapshot = SettingsSnapshot::of(&root().validated(ProfileId::new("mine")).unwrap());
    /// let mut receipt = Receipt::new("calendar.convert", serde_json::json!({}), snapshot, Hash::of(b"value")).sealed();
    /// receipt.request = serde_json::json!({"days": 4});
    /// assert_eq!(receipt.broken(), ["requestDigest", "digest"]);
    /// ```
    #[must_use]
    pub fn broken(&self) -> Vec<&'static str> {
        let mut broken = Vec::new();
        if self.request_digest != content_hash(&self.request) {
            broken.push("requestDigest");
        }
        if !self.settings.holds() {
            broken.push("settings.hash");
        }
        if self.digest != self.seal() {
            broken.push("digest");
        }
        broken
    }

    /// A receipt read strictly from its JSON, refused by the field under
    /// `receipt`.
    ///
    /// ```
    /// use teistro_core::receipt::Receipt;
    ///
    /// // Another format is refused by it before anything else is read.
    /// let refused = Receipt::from_json(r#"{"format": "teistro.receipt/9"}"#).unwrap_err();
    /// assert_eq!(refused.field(), Some("receipt.format"));
    /// let refused = Receipt::from_json(r#"{"format": "teistro.receipt/1", "recrd": "x"}"#).unwrap_err();
    /// assert_eq!(refused.field(), Some("receipt.recrd"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not a receipt or a key it does not
    /// have; `SCHEMA_VERSION` at `receipt.format` for another format.
    pub fn from_json(text: &str) -> Result<Receipt, crate::Error> {
        Receipt::from_value(&crate::strict::parse(text, RECEIPT)?, RECEIPT)
    }

    /// As [`Receipt::from_json`], from a value, refused under `root`.
    ///
    /// ```
    /// # use teistro_core::envelope::Hash;
    /// # use teistro_core::receipt::Receipt;
    /// # use teistro_core::settings::{ProfileId, SettingsSnapshot, root};
    /// # let snapshot = SettingsSnapshot::of(&root().validated(ProfileId::new("mine")).unwrap());
    /// let receipt = Receipt::new("calendar.convert", serde_json::json!({}), snapshot, Hash::of(b"value")).sealed();
    /// let value = serde_json::to_value(&receipt)?;
    /// assert_eq!(Receipt::from_value(&value, "issued.receipt")?, receipt);
    /// let mut other = value.clone();
    /// other["format"] = "teistro.receipt/2".into();
    /// assert_eq!(Receipt::from_value(&other, "issued.receipt").unwrap_err().field(), Some("issued.receipt.format"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Receipt::from_json`].
    pub fn from_value(value: &serde_json::Value, root: &str) -> Result<Receipt, crate::Error> {
        // The format first: a receipt of another format is not read as
        // this one and refused for a field it never had.
        if let Some(format) = value
            .get("format")
            .and_then(serde_json::Value::as_str)
            .filter(|format| *format != FORMAT)
        {
            return Err(crate::Error::new(
                crate::Status::SchemaVersion,
                format!("a receipt in `{format}`; this build reads `{FORMAT}`"),
            )
            .with_field(format!("{root}.format")));
        }
        // A format that is no string is refused by the read, so what is
        // read is this format.
        crate::strict::read_value(value, root)
    }
}

/// What a receipt read alone is named under.
const RECEIPT: &str = "receipt";

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::settings::{ProfileId, root};

    fn receipt() -> Receipt {
        let snapshot = SettingsSnapshot::of(&root().validated(ProfileId::new("mine")).unwrap());
        Receipt::new(
            "calendar.convert",
            serde_json::json!({"into": "calendar.BIKRAM_SAMBAT"}),
            snapshot,
            Hash::of(b"value"),
        )
        .sealed()
    }

    #[test]
    fn a_receipt_reads_back_from_its_canonical_json() {
        let written = crate::envelope::canonical_json(&receipt());
        let read = Receipt::from_json(&written).unwrap();
        assert_eq!(read, receipt());
        assert_eq!(read.broken(), Vec::<&str>::new());
    }

    #[test]
    fn another_format_is_refused_by_its_field() {
        let mut other = serde_json::to_value(receipt()).unwrap();
        other["format"] = "teistro.receipt/2".into();
        let refused = Receipt::from_value(&other, "verify").unwrap_err();
        assert_eq!(refused.status, crate::Status::SchemaVersion);
        assert_eq!(refused.field(), Some("verify.format"));
    }

    #[test]
    fn each_digest_names_its_own_field() {
        let mut asked = receipt();
        asked.request = serde_json::json!({"into": "calendar.GREGORIAN"});
        assert_eq!(asked.broken(), ["requestDigest", "digest"]);
        let mut settings = receipt();
        settings.settings.hash = Hash::of(b"other settings");
        assert_eq!(settings.broken(), ["settings.hash", "digest"]);
    }
}

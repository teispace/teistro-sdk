//! `sdk.verify`'s receipts (`03-design/receipts-and-datasets.md` §1,
//! SR-51): a record answered with a receipt naming what it was asked and
//! under what, and a receipt answered again and compared.
//!
//! The executor and the attester of an OKF *Attested Computation*:
//! [`VerifyArea::issue`] runs a record and returns its receipt, and
//! [`VerifyArea::receipt`] takes a receipt and returns a verdict, with no
//! judgement in it but digests compared.

use serde::{Deserialize, Serialize};
use teistro_core::envelope::{Version, canonical_json, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::receipt::{Receipt, ReceiptProvider};
use teistro_core::settings::SettingsSnapshot;

use super::verify::VerifyArea;
use crate::context::Context;

/// What an issue request is named under.
const ISSUE: &str = "issue";

/// A record to answer with a receipt: its name and its request.
///
/// ```
/// use teistro::verify::IssueRequest;
///
/// let asked = IssueRequest::from_json(r#"{"record": "calendar.convert", "request": {"date": {"year": 2026, "month": 10, "day": 10}, "into": "calendar.BIKRAM_SAMBAT"}}"#)?;
/// assert_eq!(asked.record, "calendar.convert");
/// let stray = IssueRequest::from_json(r#"{"record": "calendar.convert", "request": {}, "when": 1}"#).unwrap_err();
/// assert_eq!(stray.field(), Some("issue.when"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct IssueRequest {
    /// The record's name, `area.operation`, as `records` lists it.
    pub record: String,
    /// The record's request, as that record reads it.
    pub request: serde_json::Value,
}

impl IssueRequest {
    /// The request a binding sends, as JSON: `{record, request}`.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `issue`.
    pub fn from_json(text: &str) -> Result<IssueRequest, Error> {
        teistro_core::strict::read(text, ISSUE)
    }
}

/// A record's answer and its receipt.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Issued {
    /// The record's answer exactly as the record answers it: `{value,
    /// provenance}` where it is sealed, the value alone where it is not.
    pub answer: serde_json::Value,
    /// What it was asked, under what, and the digest of its value.
    pub receipt: Receipt,
}

/// What a receipt was found to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ReceiptVerdict"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReceiptVerdict {
    /// Answered again, the value's digest is the receipt's.
    Held,
    /// Answered again, the value's digest is another.
    Differs,
    /// Not answered again, for the `reason` given.
    NotRerun,
    /// A digest the receipt carries is not its own fields': it was
    /// changed after it was issued. Nothing is answered again.
    Broken,
}

/// Why a receipt was not answered again.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "NotRerunCause"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotRerunCause {
    /// This build carries no record by the receipt's name.
    Record,
    /// The receipt names an ephemeris and the context has none, or
    /// another one.
    Provider,
    /// The receipt's settings cannot be put under the context's
    /// ephemeris (a classical one opened for other settings).
    Settings,
    /// The receipt's locale is not loaded.
    Locale,
    /// The receipt names a runtime pack the context has not loaded.
    Packs,
    /// The request asks past a range this context covers: the
    /// ephemeris's data, a table's years, a count's bound; `detail` says
    /// which.
    OutOfRange,
    /// The record refused the request for another reason.
    Refused,
}

/// Why a receipt was not answered again, with what said so.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "NotRerunReason"))]
pub struct NotRerun {
    /// The cause.
    pub cause: NotRerunCause,
    /// What was missing or what refused, in words.
    pub detail: String,
}

/// One side of a receipt's check: what issued it, or what checked it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ReceiptSide"))]
#[serde(rename_all = "camelCase")]
pub struct Side {
    /// The SDK's version.
    pub sdk_version: Version,
    /// The calculation version.
    pub calculation_version: u32,
    /// The catalogue's version.
    pub catalogue_version: u32,
    /// The ephemeris; none where none was read, or the context has none.
    pub provider: Option<ReceiptProvider>,
    /// The zone database's version.
    pub tzdb: String,
}

/// One field that moved between the receipt and its check, with both
/// values as text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ReceiptMove"))]
pub struct Moved {
    /// `output`, `sdkVersion`, `calculationVersion`, `catalogueVersion`,
    /// `provider` or `tzdb`.
    pub field: String,
    /// The receipt's.
    pub issued: String,
    /// The check's.
    pub now: String,
}

/// What a receipt was found to be, and what moved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ReceiptCheck {
    /// What the receipt was found to be.
    pub verdict: ReceiptVerdict,
    /// The record it names.
    pub record: String,
    /// Every field that moved, on a receipt answered again: `output`
    /// first when the value did.
    pub moved: Vec<Moved>,
    /// Each digest that is not its fields' own, on a `BROKEN` receipt:
    /// `requestDigest`, `settings.hash`, `digest`.
    pub broken: Vec<String>,
    /// What issued the receipt, as it says.
    pub issued: Side,
    /// What checked it.
    pub now: Side,
    /// Why it was not answered again, on `NOT_RERUN`.
    pub reason: Option<NotRerun>,
}

impl Side {
    /// The side a receipt says issued it.
    fn issued(receipt: &Receipt) -> Side {
        Side {
            sdk_version: receipt.sdk_version.clone(),
            calculation_version: receipt.calculation_version,
            catalogue_version: receipt.catalogue_version,
            provider: receipt.provider.clone(),
            tzdb: receipt.tzdb.clone(),
        }
    }

    /// This build and `context`.
    fn now(context: &Context) -> Side {
        Side {
            sdk_version: sdk_version(),
            calculation_version: teistro_core::envelope::CALCULATION_VERSION,
            catalogue_version: teistro_core::catalogue::SCHEMA_VERSION,
            provider: context.ephemeris().map(|provider| {
                let identity = provider.capabilities().identity;
                ReceiptProvider {
                    name: identity.name,
                    version: identity.version,
                    data_version: identity.data_version,
                }
            }),
            tzdb: context.zones().version().to_string(),
        }
    }

    /// Each field that differs from `now`, both written as text.
    fn moves(&self, now: &Side) -> Vec<Moved> {
        let provider = |side: &Side| {
            side.provider.as_ref().map_or_else(String::new, |p| {
                format!("{} {} ({})", p.name, p.version, p.data_version)
            })
        };
        [
            (
                "sdkVersion",
                self.sdk_version.to_string(),
                now.sdk_version.to_string(),
            ),
            (
                "calculationVersion",
                self.calculation_version.to_string(),
                now.calculation_version.to_string(),
            ),
            (
                "catalogueVersion",
                self.catalogue_version.to_string(),
                now.catalogue_version.to_string(),
            ),
            ("provider", provider(self), provider(now)),
            ("tzdb", self.tzdb.clone(), now.tzdb.clone()),
        ]
        .into_iter()
        // A receipt of an answer that read no ephemeris names none, and a
        // verifier that has one has not moved it.
        .filter(|(field, issued, now)| {
            issued != now && !(*field == "provider" && issued.is_empty())
        })
        .map(|(field, issued, now)| Moved {
            field: field.to_owned(),
            issued,
            now,
        })
        .collect()
    }
}

/// This build's version.
fn sdk_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version::new(0, 0, 0))
}

impl VerifyArea<'_> {
    /// A record answered as [`crate::records::Record::answer`] answers it,
    /// and its receipt: the record's name, the request, the settings
    /// whole, the locale, the packs, the ephemeris, the zone database, the
    /// versions and the digest of the value, sealed
    /// (`03-design/receipts-and-datasets.md` §1.2).
    ///
    /// ```
    /// use teistro::verify::{IssueRequest, ReceiptVerdict};
    ///
    /// let sdk = teistro::Context::builder().build()?;
    /// let asked = IssueRequest::from_json(r#"{"record": "calendar.convert", "request": {"date": {"year": 2026, "month": 10, "day": 10}, "into": "calendar.BIKRAM_SAMBAT"}}"#)?;
    /// let issued = sdk.verify().issue(&asked)?;
    /// assert_eq!(issued.answer["date"]["year"], 2083);
    /// assert_eq!(sdk.verify().receipt(&issued.receipt).verdict, ReceiptVerdict::Held);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` at `issue.record` for a record this build does not
    /// carry; `INVALID_ARG` at `issue.request` for a request that is not
    /// an object; otherwise whatever the record refuses.
    pub fn issue(self, asked: &IssueRequest) -> Result<Issued, Error> {
        let context = self.context();
        let record = crate::records::record(&asked.record).ok_or_else(|| {
            Error::unsupported(format!("this build carries no record `{}`", asked.record))
                .with_field("issue.record")
                .with_hint("`records` lists the records this build carries")
        })?;
        if !asked.request.is_object() {
            return Err(Error::invalid_arg("a record's request is a JSON object")
                .with_field("issue.request"));
        }
        // The record reads the very text the receipt keeps, so what is
        // answered again is what was answered.
        let answered = record.answer(context, &canonical_json(&asked.request))?;
        let answer = serde_json::from_str(&answered.json())
            .map_err(|why| Error::internal(format!("an answer that does not read back: {why}")))?;
        let mut receipt = Receipt::new(
            record.name,
            asked.request.clone(),
            SettingsSnapshot::of(context.resolved()),
            content_hash(&answered.value),
        );
        receipt.locale = context.intl().locale();
        receipt.packs = context.pack_stamps(receipt.catalogue_version);
        receipt.provider = answered
            .provenance
            .as_ref()
            .map(|provenance| &provenance.provider)
            .filter(|stamp| !stamp.name.is_empty())
            .map(|stamp| ReceiptProvider {
                name: stamp.name.clone(),
                version: stamp.version.clone(),
                data_version: stamp.data_version.clone(),
            });
        receipt.tzdb = context.zones().version().to_string();
        receipt.sdk_version = sdk_version();
        Ok(Issued {
            answer,
            receipt: receipt.sealed(),
        })
    }

    /// A receipt checked: held together, then answered again on this
    /// context under the receipt's own settings, locale and request, and
    /// the value's digest compared (`03-design/receipts-and-datasets.md`
    /// §1.3). Never refused: what could not be checked is the verdict
    /// `NOT_RERUN` with why, and a receipt edited after issue is `BROKEN`.
    #[must_use]
    pub fn receipt(self, receipt: &Receipt) -> ReceiptCheck {
        let context = self.context();
        let mut check = ReceiptCheck {
            verdict: ReceiptVerdict::NotRerun,
            record: receipt.record.clone(),
            moved: Vec::new(),
            broken: Vec::new(),
            issued: Side::issued(receipt),
            now: Side::now(context),
            reason: None,
        };
        let broken = receipt.broken();
        if !broken.is_empty() {
            check.verdict = ReceiptVerdict::Broken;
            check.broken = broken.into_iter().map(str::to_owned).collect();
            return check;
        }
        match rerun(context, receipt) {
            Err(reason) => check.reason = Some(reason),
            Ok(output) => {
                let mut moved = check.issued.moves(&check.now);
                if output == receipt.output_digest {
                    check.verdict = ReceiptVerdict::Held;
                } else {
                    check.verdict = ReceiptVerdict::Differs;
                    moved.insert(
                        0,
                        Moved {
                            field: String::from("output"),
                            issued: receipt.output_digest.to_string(),
                            now: output.to_string(),
                        },
                    );
                }
                check.moved = moved;
            }
        }
        check
    }
}

/// A reason not to re-run.
fn not_rerun(cause: NotRerunCause, detail: impl Into<String>) -> NotRerun {
    NotRerun {
        cause,
        detail: detail.into(),
    }
}

/// The receipt's record answered again on a context derived from
/// `context` under its settings, and the digest of the value; or why it
/// could not be.
fn rerun(context: &Context, receipt: &Receipt) -> Result<teistro_core::envelope::Hash, NotRerun> {
    use NotRerunCause as Cause;
    let record = crate::records::record(&receipt.record).ok_or_else(|| {
        not_rerun(
            Cause::Record,
            format!("this build carries no record `{}`", receipt.record),
        )
    })?;
    if let Some(named) = &receipt.provider {
        let Some(provider) = context.ephemeris() else {
            return Err(not_rerun(
                Cause::Provider,
                format!(
                    "the receipt was computed with {} and this context has no ephemeris",
                    named.name
                ),
            ));
        };
        let here = provider.capabilities().identity.name;
        if here != named.name {
            return Err(not_rerun(
                Cause::Provider,
                format!(
                    "the receipt was computed with {} and this context computes with {here}",
                    named.name
                ),
            ));
        }
    }
    let loaded = context.pack_stamps(receipt.catalogue_version);
    if let Some(missing) = receipt.packs.iter().find(|pack| !loaded.contains(pack)) {
        return Err(not_rerun(
            Cause::Packs,
            format!(
                "the receipt names the pack {} ({}), which this context has not loaded",
                missing.id, missing.hash
            ),
        ));
    }
    let locale = Some(receipt.locale.as_str()).filter(|tag| !tag.is_empty());
    let derived = context
        .with_settings(&receipt.settings, locale)
        .map_err(|why| {
            let cause = if why.field() == Some("locale") {
                Cause::Locale
            } else {
                Cause::Settings
            };
            // Its words with its field, which names the knob.
            not_rerun(cause, why.to_string())
        })?;
    let answered = record
        .answer(&derived, &canonical_json(&receipt.request))
        .map_err(|why| {
            let cause = if why.status == Status::OutOfRange {
                Cause::OutOfRange
            } else {
                Cause::Refused
            };
            not_rerun(cause, why.to_string())
        })?;
    Ok(content_hash(&answered.value))
}

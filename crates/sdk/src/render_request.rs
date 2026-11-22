//! A plan said in the context's locale, as one record (`intl.render`):
//! what a binding does item by item with `ts_intl_render`, for a caller
//! that holds only JSON, an agent over the MCP server among them
//! (`03-design/plans-at-the-boundary.md` §7).
//!
//! A composer's answer stays keys and slots, so one crossing serves every
//! locale; this record is the render, taken as a batch, never a locale on
//! the chart request.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

use crate::Item;
use crate::area::IntlArea;

/// The root a refusal names.
const RENDER: &str = "render";

/// The items to say, each a message key and its slots as a composer's
/// plan writes them: a rashifal's plans, an interpretation's, a day's
/// sentences.
///
/// ```
/// use teistro::RenderRequest;
///
/// let asked = RenderRequest::from_json(
///     r#"{"items": [{"key": "sdk.rashifal.saturn",
///         "params": {"house": 12, "phase": "RISING", "spell": "no"}}]}"#,
/// )?;
/// assert_eq!(asked.items[0].key, "sdk.rashifal.saturn");
/// // No item is no request.
/// let none = RenderRequest::from_json(r#"{"items": []}"#).unwrap_err();
/// assert_eq!(none.field(), Some("render.items"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderRequest {
    /// The items, said in their order.
    pub items: Vec<Item>,
}

impl RenderRequest {
    /// The request a binding or an agent writes.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or no item, each named under `render`.
    pub fn from_json(text: &str) -> Result<RenderRequest, Error> {
        let asked: RenderRequest = teistro_core::strict::read(text, RENDER)?;
        if asked.items.is_empty() {
            return Err(Error::invalid_arg("no item to say").with_field("render.items"));
        }
        Ok(asked)
    }
}

/// One item said: its text and where the text came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Said {
    /// The message key the item named.
    pub key: String,
    /// The plain text, its markup stripped.
    pub text: String,
    /// The locale whose message answered; absent when none had it, and
    /// the text is then the key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_from: Option<String>,
    /// Whether a fallback locale answered.
    pub fallback: bool,
    /// Whether any of the text is from a namespace its locale marks
    /// unreviewed.
    pub draft: bool,
    /// Every problem met: a missing slot, a missing entity.
    pub warnings: Vec<String>,
}

impl IntlArea<'_> {
    /// Each item of `request` said in the context's locale, in order: a
    /// missing message is said as its key with a warning, never refused.
    ///
    /// ```
    /// use teistro::{Context, Ephemeris, RenderRequest};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Test]).locale("ne-Deva-NP").build()?;
    /// let asked = RenderRequest::from_json(
    ///     r#"{"items": [{"key": "sdk.rashifal.saturn",
    ///         "params": {"house": 12, "phase": "RISING", "spell": "no"}}]}"#,
    /// )?;
    /// let said = sdk.intl().say(&asked);
    /// assert_eq!(said[0].resolved_from.as_deref(), Some("ne-Deva-NP"));
    /// assert!(said[0].text.contains("साढेसाती"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn say(self, request: &RenderRequest) -> Vec<Said> {
        request
            .items
            .iter()
            .map(|item| {
                let rendered = self.render(&item.key, &item.params);
                Said {
                    key: item.key.clone(),
                    text: rendered.text,
                    resolved_from: rendered.resolved_from,
                    fallback: rendered.is_fallback,
                    draft: rendered.is_draft,
                    warnings: rendered.warnings,
                }
            })
            .collect()
    }
}

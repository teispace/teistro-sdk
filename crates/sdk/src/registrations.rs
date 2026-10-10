//! A consumer's own layouts and dasha systems read from JSON, as every
//! surface that takes them as text reads them: the C boundary's
//! `layouts_json` and `dashas_json`, and the agent server's `--layouts`
//! and `--dashas` (`03-design/mcp-server.md` §7).

use teistro_core::error::Error;
use teistro_core::strict;

use crate::Layout;
#[cfg(feature = "chart")]
use crate::dasha::DashaDefinition;

/// A JSON array of layout rows, each read strictly and checked by the
/// rules a shipped row passes, refused by its place under `root`
/// (`{root}[2].shape.rings`; `03-design/chart-geometry.md` §7f).
///
/// ```
/// let refused = teistro::registrations::layouts_from_json("[{}]", "layouts").unwrap_err();
/// assert!(refused.field().is_some_and(|field| field.starts_with("layouts[0]")));
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for text that is not an array, or a row that does not
/// read or does not pass the checks, naming it under `root`.
pub fn layouts_from_json(json: &str, root: &str) -> Result<Vec<Layout>, Error> {
    let rows: Vec<serde_json::Value> = strict::read(json, root)?;
    rows.into_iter()
        .enumerate()
        .map(|(index, row)| {
            let at = format!("{root}[{index}]");
            let layout: Layout = strict::read_value(&row, &at)?;
            layout.validate().map_err(|error| error.under(&at))?;
            Ok(layout)
        })
        .collect()
}

/// A JSON array of dasha system definitions, each read strictly and
/// refused by its place under `root`; the context's registry checks each
/// by the rules a shipped row passes when the context is built.
///
/// ```
/// let refused = teistro::registrations::dashas_from_json("[1]", "dashas").unwrap_err();
/// assert!(refused.field().is_some_and(|field| field.starts_with("dashas[0]")));
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for text that is not an array, or a definition that
/// does not read, naming it under `root`.
#[cfg(feature = "chart")]
pub fn dashas_from_json(json: &str, root: &str) -> Result<Vec<DashaDefinition>, Error> {
    let rows: Vec<serde_json::Value> = strict::read(json, root)?;
    rows.into_iter()
        .enumerate()
        .map(|(index, row)| strict::read_value(&row, &format!("{root}[{index}]")))
        .collect()
}

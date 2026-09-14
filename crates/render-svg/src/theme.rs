//! The theme record: how a drawing looks ([`Style`]) and what it says
//! ([`Content`]), serialised as one value (`03-design/render-svg.md` §3).
//!
//! Every field has a default, so a consumer's theme names only what it
//! changes: `{"style": {"ink": "#333333"}}` is a whole theme. An unknown
//! field is refused, because a misspelt colour that silently does nothing
//! is the mistake a theme record is most likely to hold.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

/// How a drawing looks: the renderer's only input besides the geometry and
/// the labels.
///
/// Sizes are fractions of [`Style::size`], so one theme draws the same
/// chart at any size.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Style {
    /// The drawing's width and height, in SVG user units.
    pub size: f64,
    /// The page behind the chart, as `#rrggbb`.
    pub background: String,
    /// Lines and text, as `#rrggbb`.
    pub ink: String,
    /// A cell's fill, as `#rrggbb`.
    pub cell: String,
    /// The fill of the cell the lagna stands in, as `#rrggbb`.
    pub lagna_cell: String,
    /// The lagna's own label and mark, as `#rrggbb`.
    pub accent: String,
    /// Line width, as a fraction of the size.
    pub stroke: f64,
    /// The font family every text asks for.
    pub font_family: String,
    /// The largest a body's label is drawn, as a fraction of the size; a
    /// crowded cell draws smaller (§6).
    pub body_size: f64,
    /// A cell's label, as a fraction of the size.
    pub label_size: f64,
    /// A body drawn at its degree on a wheel, as a fraction of the size.
    pub mark_size: f64,
    /// The width one character is estimated at, in ems (§6).
    pub advance: f64,
    /// The distance between two lines of a stack, in ems.
    pub line_height: f64,
    /// How far below a line's centre its baseline sits, in ems: what
    /// `dominant-baseline` would do, done where every renderer honours it.
    pub baseline_shift: f64,
}

/// What a drawing says: read when the labels are composed, never by the
/// renderer itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Content {
    /// The form a body is written in.
    pub body_form: BodyForm,
    /// What a cell's label shows.
    pub cell_label: CellLabel,
    /// Whether the lagna is written first in the cell it stands in.
    pub lagna_mark: bool,
    /// What is written after a retrograde graha's name, or nothing. The
    /// nodes are never marked: their mean motion is always backwards.
    pub retrograde_mark: Option<String>,
    /// Whether a graha's degree in its sign follows its name, on a drawing
    /// of the founded chart.
    pub degrees: bool,
}

/// The locale form a body is written in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum BodyForm {
    /// The locale's abbreviation: `Su`, `सू`.
    #[default]
    Short,
    /// The symbol: `☉`.
    Glyph,
}

/// What a cell's label shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CellLabel {
    /// The sign's number in a square chart or the chakra; on a wheel, the
    /// house in the ring its bodies are marked in and the sign's glyph in
    /// the others.
    #[default]
    Auto,
    /// The sign's number, 1 for Aries.
    SignNumber,
    /// The sign's abbreviation.
    SignShort,
    /// The sign's symbol.
    SignGlyph,
    /// The house's number.
    House,
    /// Nothing.
    Nothing,
}

/// A whole theme: one [`Style`] and one [`Content`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    /// How the drawing looks.
    pub style: Style,
    /// What it says.
    pub content: Content,
}

impl Default for Style {
    fn default() -> Style {
        Style::light()
    }
}

impl Default for Content {
    fn default() -> Content {
        Content {
            body_form: BodyForm::Short,
            cell_label: CellLabel::Auto,
            lagna_mark: true,
            retrograde_mark: Some(String::from("(R)")),
            degrees: false,
        }
    }
}

impl Style {
    /// Dark ink on white, as a printed patrika.
    #[must_use]
    pub fn light() -> Style {
        Style {
            size: 1000.0,
            background: String::from("#ffffff"),
            ink: String::from("#1f1f1f"),
            cell: String::from("#ffffff"),
            lagna_cell: String::from("#fff3dc"),
            accent: String::from("#b3261e"),
            stroke: 0.002,
            font_family: String::from("sans-serif"),
            body_size: 0.036,
            label_size: 0.026,
            mark_size: 0.03,
            advance: 0.62,
            line_height: 1.2,
            baseline_shift: 0.35,
        }
    }

    /// Light ink on a dark page.
    #[must_use]
    pub fn dark() -> Style {
        Style {
            background: String::from("#121212"),
            ink: String::from("#e6e1e5"),
            cell: String::from("#121212"),
            lagna_cell: String::from("#2e2618"),
            accent: String::from("#ffb4ab"),
            ..Style::light()
        }
    }

    /// Refuses a style that cannot be drawn, naming the field.
    ///
    /// # Errors
    ///
    /// A colour that is not `#rrggbb`, an empty font family, or a size or
    /// fraction that is not a positive finite number in its range.
    pub fn validate(&self) -> Result<(), Error> {
        for (field, colour) in [
            ("style.background", &self.background),
            ("style.ink", &self.ink),
            ("style.cell", &self.cell),
            ("style.lagna_cell", &self.lagna_cell),
            ("style.accent", &self.accent),
        ] {
            if !is_colour(colour) {
                return Err(refused(
                    field,
                    format!("{colour:?} is not a colour"),
                    "write it as #rrggbb, for example #1f1f1f",
                ));
            }
        }
        if self.font_family.trim().is_empty() {
            return Err(refused(
                "style.font_family",
                "the font family is empty",
                "name a family, or a generic one such as sans-serif",
            ));
        }
        within("style.size", self.size, 1.0, MAX_SIZE)?;
        for (field, value) in [
            ("style.stroke", self.stroke),
            ("style.body_size", self.body_size),
            ("style.label_size", self.label_size),
            ("style.mark_size", self.mark_size),
        ] {
            within(field, value, f64::MIN_POSITIVE, 1.0)?;
        }
        for (field, value) in [
            ("style.advance", self.advance),
            ("style.line_height", self.line_height),
        ] {
            within(field, value, f64::MIN_POSITIVE, 4.0)?;
        }
        within("style.baseline_shift", self.baseline_shift, -1.0, 1.0)
    }
}

impl Theme {
    /// The light style with the default content.
    #[must_use]
    pub fn light() -> Theme {
        Theme::default()
    }

    /// The dark style with the default content.
    #[must_use]
    pub fn dark() -> Theme {
        Theme {
            style: Style::dark(),
            content: Content::default(),
        }
    }

    /// Reads a theme from JSON, refusing an unknown field or a style that
    /// cannot be drawn.
    ///
    /// # Errors
    ///
    /// JSON that is not a theme, or a style [`Style::validate`] refuses.
    ///
    /// ```
    /// use teistro_render_svg::Theme;
    ///
    /// let theme = Theme::from_json(r##"{"style": {"ink": "#333333"}}"##)?;
    /// assert_eq!(theme.style.ink, "#333333");
    /// assert_eq!(theme.style.background, Theme::light().style.background);
    ///
    /// let wrong = Theme::from_json(r##"{"style": {"colour": "#333333"}}"##).unwrap_err();
    /// assert!(wrong.message.contains("colour"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    pub fn from_json(text: &str) -> Result<Theme, Error> {
        let theme: Theme = serde_json::from_str(text).map_err(|err| {
            Error::invalid_arg(format!("the theme is not one: {err}"))
                .with_field(String::from("theme"))
        })?;
        theme.style.validate()?;
        Ok(theme)
    }
}

/// The largest drawing, in user units: far past any page, and small enough
/// that every coordinate in hundredths fits an `i64` exactly.
pub const MAX_SIZE: f64 = 1_000_000.0;

fn is_colour(text: &str) -> bool {
    text.strip_prefix('#')
        .is_some_and(|hex| hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn within(field: &str, value: f64, low: f64, high: f64) -> Result<(), Error> {
    if value.is_finite() && (low..=high).contains(&value) {
        Ok(())
    } else {
        Err(refused(
            field,
            format!("{value} is outside {low} to {high}"),
            "sizes are fractions of the drawing's size",
        ))
    }
}

pub(crate) fn refused(field: &str, message: impl Into<String>, hint: &str) -> Error {
    Error::invalid_arg(message.into())
        .with_field(field.to_owned())
        .with_hint(hint.to_owned())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, index their own fixtures and compare exact values"
    )]
    use super::*;

    #[test]
    fn the_shipped_styles_are_valid() {
        Style::light().validate().unwrap();
        Style::dark().validate().unwrap();
    }

    #[test]
    fn a_style_is_refused_by_the_field_it_gets_wrong() {
        type Break = fn(&mut Style);
        let cases: [(&str, Break); 6] = [
            ("style.ink", |s| s.ink = String::from("black")),
            ("style.accent", |s| s.accent = String::from("#12345")),
            ("style.font_family", |s| s.font_family = String::from(" ")),
            ("style.size", |s| s.size = f64::NAN),
            ("style.body_size", |s| s.body_size = 0.0),
            ("style.baseline_shift", |s| s.baseline_shift = 2.0),
        ];
        for (field, break_it) in cases {
            let mut style = Style::light();
            break_it(&mut style);
            let err = style.validate().unwrap_err();
            assert_eq!(err.field(), Some(field), "{err:?}");
        }
    }

    #[test]
    fn a_theme_round_trips_and_names_only_what_it_changes() {
        let dark = Theme::dark();
        let text = serde_json::to_string(&dark).unwrap();
        assert_eq!(Theme::from_json(&text).unwrap(), dark);
        assert_eq!(Theme::from_json("{}").unwrap(), Theme::light());
        let partial = Theme::from_json(r#"{"content": {"body_form": "glyph"}}"#).unwrap();
        assert_eq!(partial.content.body_form, BodyForm::Glyph);
        assert_eq!(partial.style, Style::light());
    }

    #[test]
    fn a_theme_that_cannot_be_drawn_is_refused_when_read() {
        let err = Theme::from_json(r#"{"style": {"ink": "red"}}"#).unwrap_err();
        assert_eq!(err.field(), Some("style.ink"));
        let err = Theme::from_json(r#"{"content": {"body": "short"}}"#).unwrap_err();
        assert_eq!(err.field(), Some("theme"));
    }
}

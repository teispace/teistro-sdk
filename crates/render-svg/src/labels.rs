//! The strings a drawing prints, already in the reader's script
//! (`03-design/render-svg.md` §3).
//!
//! The renderer is told what to write and never decides it, so it holds no
//! glyph table and no astrology. The façade composes these from a chart
//! document, the context's locale and a theme's [`Content`](crate::Content).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::key::KeyId;
use teistro_geometry::Placed;

use crate::svg::is_xml_char;
use crate::theme::refused;

/// What a drawing writes: a title, each cell's label, each body's name,
/// and the lagna's.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Labels {
    /// The drawing's accessible name, written as its `<title>`.
    pub title: Option<String>,
    /// One label per cell, in the placed chart's order; an empty one is not
    /// drawn.
    pub cells: Vec<String>,
    /// Each body's name, by its key.
    pub bodies: BTreeMap<KeyId, String>,
    /// The lagna's name, written first in the cell it stands in; nothing
    /// when a theme does not mark it.
    pub lagna: Option<String>,
}

impl Labels {
    /// Refuses labels that do not fit the chart: a cell count that differs,
    /// a body with no name, or a character XML cannot carry.
    ///
    /// # Errors
    ///
    /// The first mismatch, naming its field.
    pub fn check(&self, placed: &Placed) -> Result<(), Error> {
        if self.cells.len() != placed.cells.len() {
            return Err(refused(
                "labels.cells",
                format!(
                    "{} cell labels for a chart of {} cells",
                    self.cells.len(),
                    placed.cells.len()
                ),
                "give one label per cell, an empty one where nothing is written",
            ));
        }
        let drawn = placed
            .cells
            .iter()
            .flat_map(|cell| &cell.bodies)
            .chain(placed.marks.iter().map(|mark| &mark.body));
        for body in drawn {
            if !self.bodies.contains_key(body) {
                return Err(refused(
                    "labels.bodies",
                    format!("the chart draws {body} and the labels do not name it"),
                    "name every body the chart places",
                ));
            }
        }
        let strings = self
            .title
            .iter()
            .map(|title| ("labels.title", title))
            .chain(self.cells.iter().map(|cell| ("labels.cells", cell)))
            .chain(self.bodies.values().map(|body| ("labels.bodies", body)))
            .chain(self.lagna.iter().map(|lagna| ("labels.lagna", lagna)));
        for (field, text) in strings {
            if let Some(bad) = text.chars().find(|&c| !is_xml_char(c)) {
                return Err(refused(
                    field,
                    format!(
                        "{text:?} holds U+{:04X}, which SVG cannot carry",
                        u32::from(bad)
                    ),
                    "remove control characters from the label",
                ));
            }
        }
        Ok(())
    }
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
    use teistro_core::catalogue::{Graha, Rashi};
    use teistro_geometry::{Body, Placements, place, rows};

    use super::*;

    fn placed() -> Placed {
        let chart =
            Placements::new(Rashi::Leo).with(Body::in_sign(Graha::Sun.key_id(), Rashi::Aries));
        place(&rows::north_indian(), &chart).unwrap()
    }

    fn named(placed: &Placed) -> Labels {
        Labels {
            title: Some(String::from("chart")),
            cells: placed
                .cells
                .iter()
                .map(|cell| cell.sign.id().to_string())
                .collect(),
            bodies: BTreeMap::from([(Graha::Sun.key_id(), String::from("Su"))]),
            lagna: Some(String::from("As")),
        }
    }

    #[test]
    fn labels_that_fit_the_chart_pass() {
        let placed = placed();
        named(&placed).check(&placed).unwrap();
    }

    #[test]
    fn labels_are_refused_by_what_they_miss() {
        let placed = placed();

        let mut short = named(&placed);
        short.cells.pop();
        assert_eq!(
            short.check(&placed).unwrap_err().field(),
            Some("labels.cells")
        );

        let mut nameless = named(&placed);
        nameless.bodies.clear();
        let err = nameless.check(&placed).unwrap_err();
        assert_eq!(err.field(), Some("labels.bodies"));
        assert!(err.message.contains("graha.SUN"), "{}", err.message);

        let mut control = named(&placed);
        control.lagna = Some(String::from("A\u{7}s"));
        assert_eq!(
            control.check(&placed).unwrap_err().field(),
            Some("labels.lagna")
        );
    }
}

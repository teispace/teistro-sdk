//! A placed chart and a theme to a byte-stable SVG (ADR-0026,
//! `03-design/render-svg.md`).
//!
//! The crate sits above the core and reads only what a third party could:
//! a [`Placed`] chart from `teistro-geometry`, the [`Labels`] to write, and
//! a [`Style`]. It knows no astrology and no locale; what a drawing says is
//! composed before it gets here.
//!
//! - [`theme`]: [`Style`], [`Content`] and the [`Theme`] that carries both;
//! - [`labels`]: the strings a drawing prints;
//! - [`render`]: the drawing.
//!
//! Three things the research found (§2):
//!
//! - **Presentation attributes only.** Flutter's SVG renderer does not fully
//!   support CSS and does not honour `dominant-baseline`, so the output has
//!   no `<style>` and places each baseline itself.
//! - **Signs are text, not emoji.** Unicode draws the zodiac signs as emoji
//!   unless they are followed by U+FE0E, which the writer adds after exactly
//!   the characters that define that sequence.
//! - **No glyph table.** Every label comes from a locale's vetted `short` or
//!   `glyph` form.
//!
//! ```
//! use std::collections::BTreeMap;
//!
//! use teistro_core::catalogue::{Graha, Rashi};
//! use teistro_geometry::{Body, Placements, place, rows};
//! use teistro_render_svg::{Labels, Style, render};
//!
//! let chart = Placements::new(Rashi::Leo).with(Body::in_sign(Graha::Sun.key_id(), Rashi::Leo));
//! let placed = place(&rows::north_indian(), &chart)?;
//! let labels = Labels {
//!     title: Some(String::from("A Leo lagna")),
//!     cells: placed.cells.iter().map(|cell| cell.sign.id().to_string()).collect(),
//!     bodies: BTreeMap::from([(Graha::Sun.key_id(), String::from("Su"))]),
//!     lagna: Some(String::from("As")),
//! };
//! let svg = render(&placed, &labels, &Style::light())?;
//! assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
//! assert!(svg.contains("data-body=\"graha.SUN\""));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

pub mod labels;
pub mod theme;

mod fit;
mod marks;
mod svg;

pub use labels::Labels;
pub use theme::{BodyForm, CellLabel, Content, Style, Theme};

use teistro_core::error::Error;
use teistro_geometry::{Placed, PlacedCell, Point};

use crate::fit::{Clear, Metrics, stack, to_unit};
use crate::svg::{number, path_data, text, width_in_chars};

/// Draws a placed chart as SVG.
///
/// The same chart, labels and style give the same bytes on every platform
/// (§5).
///
/// # Errors
///
/// A style [`Style::validate`] refuses, or labels [`Labels::check`] refuses.
pub fn render(placed: &Placed, labels: &Labels, style: &Style) -> Result<String, Error> {
    style.validate()?;
    labels.check(placed)?;
    let mut out = String::with_capacity(4096 + placed.cells.len() * 512);
    let scale = style.size;

    out.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 ");
    number(&mut out, scale);
    out.push(' ');
    number(&mut out, scale);
    out.push_str("\" width=\"");
    number(&mut out, scale);
    out.push_str("\" height=\"");
    number(&mut out, scale);
    out.push_str("\" role=\"img\">");
    if let Some(title) = &labels.title {
        out.push_str("<title>");
        text(&mut out, title);
        out.push_str("</title>");
    }
    out.push_str("<rect width=\"");
    number(&mut out, scale);
    out.push_str("\" height=\"");
    number(&mut out, scale);
    out.push('"');
    attribute(&mut out, "fill", &style.background);
    out.push_str("/>");

    let stacked = placed.marks.is_empty();
    for (cell, label) in placed.cells.iter().zip(&labels.cells) {
        draw_cell(&mut out, cell, label, labels, style, stacked);
    }
    for frame in &placed.frame {
        out.push_str("<path d=\"");
        path_data(&mut out, frame, scale);
        out.push_str("\" fill=\"none\" stroke=\"");
        text(&mut out, &style.ink);
        out.push_str("\" stroke-width=\"");
        number(&mut out, style.stroke * scale);
        out.push_str("\"/>");
    }
    draw_marks(&mut out, placed, labels, style);
    out.push_str("</svg>");
    Ok(out)
}

fn draw_cell(
    out: &mut String,
    cell: &PlacedCell,
    label: &str,
    labels: &Labels,
    style: &Style,
    stacked: bool,
) {
    let scale = style.size;
    out.push_str("<g data-sign=\"");
    text(out, cell.sign.full_key());
    out.push_str("\" data-house=\"");
    out.push_str(&cell.house.to_string());
    out.push('"');
    if cell.lagna {
        out.push_str(" data-lagna=\"true\"");
    }
    out.push_str("><path d=\"");
    path_data(out, &cell.outline, scale);
    out.push('"');
    let fill = if cell.lagna {
        &style.lagna_cell
    } else {
        &style.cell
    };
    attribute(out, "fill", fill);
    attribute(out, "stroke", &style.ink);
    out.push_str(" stroke-width=\"");
    number(out, style.stroke * scale);
    out.push_str("\"/>");

    if !label.is_empty() {
        let fill = if cell.lagna {
            &style.accent
        } else {
            &style.ink
        };
        write_text(
            out,
            &Text {
                at: cell.label,
                font: style.label_size,
                fill,
                body: None,
                says: label,
            },
            style,
        );
    }
    if stacked {
        draw_stack(out, cell, label, labels, style);
    }
    out.push_str("</g>");
}

/// A cell's bodies, and the lagna first where it stands, fitted into the
/// cell and kept off its label (§6).
fn draw_stack(out: &mut String, cell: &PlacedCell, label: &str, labels: &Labels, style: &Style) {
    // The lagna is written in the innermost ring's cell it stands in: in an
    // outer ring of the chakra the same sign means something else.
    let lagna = labels
        .lagna
        .as_deref()
        .filter(|_| cell.lagna && cell.ring == 0);
    let names: Vec<&str> = cell
        .bodies
        .iter()
        .map(|body| labels.bodies.get(body).map_or("", String::as_str))
        .collect();
    let widest = lagna
        .into_iter()
        .chain(names.iter().copied())
        .map(width_in_chars)
        .max()
        .unwrap_or(0);
    let clear = (!label.is_empty()).then(|| Clear {
        centre: cell.label,
        half_width: to_unit(width_in_chars(label)) * style.advance * style.label_size / 2.0,
        half_height: style.line_height * style.label_size / 2.0,
    });
    let arranged = stack(
        &cell.outline,
        cell.anchor,
        clear,
        names.len() + usize::from(lagna.is_some()),
        widest,
        Metrics {
            largest: style.body_size,
            advance: style.advance,
            line_height: style.line_height,
        },
    );
    let mut lines = arranged.lines.iter();
    if let Some(lagna) = lagna
        && let Some(line) = lines.next()
    {
        let lagna = Text {
            at: line.at,
            font: arranged.font,
            fill: &style.accent,
            body: Some(String::from("point.LAGNA")),
            says: lagna,
        };
        write_text(out, &lagna, style);
    }
    for ((body, name), line) in cell.bodies.iter().zip(names).zip(lines) {
        let body = Text {
            at: line.at,
            font: arranged.font,
            fill: &style.ink,
            body: Some(body.to_string()),
            says: name,
        };
        write_text(out, &body, style);
    }
}

/// A wheel's bodies at their degrees: each written at the ring's anchor
/// radius, spread round the ring where they would overlap, with a tick at
/// its true degree inside the ring's outer edge (§6).
fn draw_marks(out: &mut String, placed: &Placed, labels: &Labels, style: &Style) {
    let Some(radii) = placed
        .marks
        .first()
        .and_then(|mark| marks::Radii::of(placed, mark.ring))
    else {
        return;
    };
    let scale = style.size;
    let names: Vec<&str> = placed
        .marks
        .iter()
        .map(|mark| labels.bodies.get(&mark.body).map_or("", String::as_str))
        .collect();
    let widest = names.iter().copied().map(width_in_chars).max().unwrap_or(0);
    // The length of ring a mark's words need: the wider of its estimated
    // width and its line, with a little air.
    let room =
        (to_unit(widest) * style.advance).max(style.line_height) * style.mark_size * MARK_AIR;
    let tick_length = style.mark_size / 2.0;
    let spread = marks::spread(&placed.marks, radii, room);
    for ((mark, at), name) in placed.marks.iter().zip(spread).zip(names) {
        let (from, to) = marks::tick(mark, radii, tick_length);
        out.push_str("<line x1=\"");
        number(out, from.x * scale);
        out.push_str("\" y1=\"");
        number(out, from.y * scale);
        out.push_str("\" x2=\"");
        number(out, to.x * scale);
        out.push_str("\" y2=\"");
        number(out, to.y * scale);
        out.push('"');
        attribute(out, "stroke", &style.ink);
        out.push_str(" stroke-width=\"");
        number(out, style.stroke * scale);
        out.push('"');
        attribute(out, "data-body", &mark.body.to_string());
        out.push_str("/>");
        let words = Text {
            at,
            font: style.mark_size,
            fill: &style.ink,
            body: Some(mark.body.to_string()),
            says: name,
        };
        write_text(out, &words, style);
    }
}

/// How much more ring than its words a mark is given.
const MARK_AIR: f64 = 1.15;

/// One piece of text: where its line's centre is, its size, and what it
/// says, in the unit square.
struct Text<'a> {
    at: Point,
    font: f64,
    fill: &'a str,
    /// The body it names, for a consumer that makes the drawing interactive.
    body: Option<String>,
    says: &'a str,
}

fn write_text(out: &mut String, piece: &Text<'_>, style: &Style) {
    let scale = style.size;
    out.push_str("<text x=\"");
    number(out, piece.at.x * scale);
    out.push_str("\" y=\"");
    number(
        out,
        (piece.at.y + piece.font * style.baseline_shift) * scale,
    );
    out.push_str("\" font-size=\"");
    number(out, piece.font * scale);
    out.push('"');
    attribute(out, "font-family", &style.font_family);
    attribute(out, "fill", piece.fill);
    out.push_str(" text-anchor=\"middle\"");
    if let Some(body) = &piece.body {
        attribute(out, "data-body", body);
    }
    out.push('>');
    text(out, piece.says);
    out.push_str("</text>");
}

fn attribute(out: &mut String, name: &str, value: &str) {
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    text(out, value);
    out.push('"');
}

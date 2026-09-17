//! The bytes of an SVG: numbers, text and outlines written the same way on
//! every platform (`03-design/render-svg.md` §4, §5).

use std::fmt::Write as _;

use teistro_geometry::{Path, Point, Segment};

/// Writes a number of user units rounded to hundredths, through an integer,
/// with trailing zeros removed and `-0` written `0`.
///
/// Rust formats floats itself rather than through the platform's C
/// library, and the rounding here is one multiplication and `round`, so the
/// same value prints the same bytes everywhere.
pub(crate) fn number(out: &mut String, value: f64) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a coordinate is at most MAX_SIZE, whose hundredths fit an i64 exactly"
    )]
    let hundredths = (value * 100.0).round() as i64;
    if hundredths < 0 {
        out.push('-');
    }
    let magnitude = hundredths.unsigned_abs();
    let (whole, part) = (magnitude / 100, magnitude % 100);
    let _ = write!(out, "{whole}");
    match part {
        0 => {}
        p if p % 10 == 0 => {
            let _ = write!(out, ".{}", p / 10);
        }
        p => {
            let _ = write!(out, ".{p:02}");
        }
    }
}

/// The characters Unicode draws as emoji unless asked for text, and for
/// which `emoji-variation-sequences.txt` (17.0) defines a text-style
/// sequence: ♀, ♂ and the twelve zodiac signs. ☉ ☽ ☿ ♃ ♄ ☊ ☋ have none and
/// are not listed (§2).
fn has_text_style(c: char) -> bool {
    matches!(c, '\u{2640}' | '\u{2642}' | '\u{2648}'..='\u{2653}')
}

/// The text-style variation selector.
const TEXT_STYLE: char = '\u{FE0E}';

/// Whether a character may appear in XML 1.0 content.
pub(crate) fn is_xml_char(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..)
}

/// Writes text escaped for an attribute or element content, asking for the
/// text presentation of every character that has one. The caller has
/// already refused a character XML cannot carry.
pub(crate) fn text(out: &mut String, value: &str) {
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
        if has_text_style(c) && chars.peek() != Some(&TEXT_STYLE) {
            out.push(TEXT_STYLE);
        }
    }
}

/// How wide a label is estimated at, in characters: its Unicode scalar
/// values, not counting a variation selector (§6).
pub(crate) fn width_in_chars(label: &str) -> usize {
    label.chars().filter(|&c| c != TEXT_STYLE).count()
}

/// Writes an outline as path data, scaled from the unit square.
pub(crate) fn path_data(out: &mut String, path: &Path, scale: f64) {
    out.push('M');
    point(out, path.start, scale);
    let mut from = path.start;
    for segment in &path.segments {
        match *segment {
            Segment::Line { to } => {
                out.push('L');
                point(out, to, scale);
            }
            Segment::Quad { control, to } => {
                out.push('Q');
                point(out, control, scale);
                out.push(' ');
                point(out, to, scale);
            }
            Segment::Arc {
                centre,
                clockwise,
                to,
            } => {
                // `sqrt` and not `hypot`: IEEE 754 requires the first to be
                // correctly rounded, and leaves the second to the platform.
                let (dx, dy) = (from.x - centre.x, from.y - centre.y);
                let radius = (dx * dx + dy * dy).sqrt();
                out.push('A');
                number(out, radius * scale);
                out.push(' ');
                number(out, radius * scale);
                let large = long_way(from, centre, to, clockwise);
                let _ = write!(out, " 0 {} {} ", u8::from(large), u8::from(clockwise));
                point(out, to, scale);
            }
        }
        from = segment.end();
    }
    out.push('Z');
}

/// Whether an arc from `from` to `to` about `centre`, running the way
/// `clockwise` says, sweeps more than half a turn: SVG's large-arc flag.
///
/// With y downwards, a short clockwise turn has a positive cross product,
/// so the arc is the long way round exactly when the product's sign
/// disagrees with the direction. A half turn has a product of zero, and
/// either flag draws it.
fn long_way(from: Point, centre: Point, to: Point, clockwise: bool) -> bool {
    let cross = (from.x - centre.x) * (to.y - centre.y) - (from.y - centre.y) * (to.x - centre.x);
    if clockwise { cross < 0.0 } else { cross > 0.0 }
}

fn point(out: &mut String, at: Point, scale: f64) {
    number(out, at.x * scale);
    out.push(',');
    number(out, at.y * scale);
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

    fn printed(value: f64) -> String {
        let mut out = String::new();
        number(&mut out, value);
        out
    }

    #[test]
    fn a_number_is_hundredths_with_no_trailing_zeros() {
        assert_eq!(printed(0.0), "0");
        assert_eq!(printed(-0.0), "0");
        assert_eq!(printed(-0.001), "0");
        assert_eq!(printed(1000.0), "1000");
        assert_eq!(printed(333.333_333_333), "333.33");
        assert_eq!(printed(666.666_666_666), "666.67");
        assert_eq!(printed(12.5), "12.5");
        assert_eq!(printed(12.05), "12.05");
        assert_eq!(printed(-3.1), "-3.1");
        assert_eq!(printed(0.07), "0.07");
    }

    #[test]
    fn text_is_escaped_and_asks_for_text_presentation() {
        let mut out = String::new();
        text(&mut out, "a<b>&\"c'");
        assert_eq!(out, "a&lt;b&gt;&amp;&quot;c&#39;");

        let mut out = String::new();
        text(&mut out, "♈☉♂\u{FE0E}");
        assert_eq!(out, "♈\u{FE0E}☉♂\u{FE0E}", "one selector, never two");
        assert_eq!(width_in_chars("♈\u{FE0E}"), 1);
        assert_eq!(width_in_chars("सू"), 2);
    }

    #[test]
    fn xml_refuses_control_characters_and_accepts_every_script() {
        assert!(!is_xml_char('\u{0}'));
        assert!(!is_xml_char('\u{1B}'));
        assert!(!is_xml_char('\u{FFFE}'));
        assert!("सू ☉ Asc\t".chars().all(is_xml_char));
    }

    #[test]
    fn an_arc_writes_its_radius_and_the_way_it_runs() {
        let centre = Point::new(0.5, 0.5);
        let quarter = Path {
            start: Point::new(0.5, 0.0),
            segments: vec![
                Segment::Arc {
                    centre,
                    clockwise: true,
                    to: Point::new(1.0, 0.5),
                },
                Segment::Line { to: centre },
            ],
        };
        let mut out = String::new();
        path_data(&mut out, &quarter, 1000.0);
        assert_eq!(out, "M500,0A500 500 0 0 1 1000,500L500,500Z");

        // The same ends the other way round is three quarters of a turn.
        assert!(long_way(quarter.start, centre, Point::new(1.0, 0.5), false));
        assert!(!long_way(quarter.start, centre, Point::new(1.0, 0.5), true));
        // A half turn is drawn by either flag, and says short.
        assert!(!long_way(quarter.start, centre, Point::new(0.5, 1.0), true));
    }

    #[test]
    fn a_curve_writes_its_control_point() {
        let petal = Path {
            start: Point::new(0.0, 0.0),
            segments: vec![Segment::Quad {
                control: Point::new(0.25, 0.125),
                to: Point::new(0.5, 0.0),
            }],
        };
        let mut out = String::new();
        path_data(&mut out, &petal, 100.0);
        assert_eq!(out, "M0,0Q25,12.5 50,0Z");
    }
}

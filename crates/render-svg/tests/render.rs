//! Every shipped layout drawn, read back as XML and checked against the chart
//! it drew (`03-design/render-svg.md` §4, §5).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own fixtures"
)]

use std::collections::BTreeMap;

use roxmltree::{Document, Node};
use teistro_chart::bhava::{Bhavas, Chalit};
use teistro_core::catalogue::{Graha, HouseSystem, Rashi};
use teistro_geometry::{Body, Layout, Placed, Placements, place, rows};
use teistro_render_svg::{Labels, Style, render};

const GRAHAS: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// A chart that crowds one cell: a Cancer lagna, five grahas in Cancer and
/// the rest spread out, at degrees and cusps such as a quadrant system
/// gives, so the wheel can draw it too.
fn chart() -> Placements {
    let cusps = [
        100.0, 128.0, 157.0, 190.0, 224.0, 253.0, 280.0, 308.0, 337.0, 10.0, 44.0, 73.0,
    ];
    let degrees = [101.0, 103.5, 104.0, 118.0, 119.9, 200.5, 12.0, 330.0, 150.0];
    Placements {
        lagna_deg: Some(100.0),
        houses: Some(Bhavas::of(Chalit::of(HouseSystem::Placidus), &cusps)),
        bodies: GRAHAS
            .iter()
            .zip(degrees)
            .map(|(graha, degree)| Body::at(graha.key_id(), degree))
            .collect(),
        ..Placements::new(Rashi::Cancer)
    }
}

fn labels(placed: &Placed, script: &[&str; 9]) -> Labels {
    Labels {
        title: Some(format!("{} <chart> & more", placed.layout)),
        cells: placed
            .cells
            .iter()
            .map(|cell| (cell.sign.id() + 1).to_string())
            .collect(),
        bodies: GRAHAS
            .iter()
            .zip(script)
            .map(|(graha, name)| (graha.key_id(), (*name).to_owned()))
            .collect::<BTreeMap<_, _>>(),
        lagna: Some(String::from("As")),
    }
}

const LATIN: [&str; 9] = ["Su", "Mo", "Ma", "Me", "Ju", "Ve", "Sa", "Ra", "Ke"];
const DEVANAGARI: [&str; 9] = ["सू", "च", "मं", "बु", "गु", "शु", "शं", "रा", "के"];

fn drawn(layout: &Layout) -> Placed {
    place(layout, &chart()).unwrap()
}

/// The elements named `name`, in document order.
fn all<'a, 'i>(document: &'a Document<'i>, name: &str) -> Vec<Node<'a, 'i>> {
    document
        .descendants()
        .filter(|node| node.tag_name().name() == name)
        .collect()
}

#[test]
fn every_shipped_layout_draws_well_formed_svg_in_two_scripts_and_both_styles() {
    for layout in rows::shipped() {
        let placed = drawn(&layout);
        for (script, style) in [(&LATIN, Style::light()), (&DEVANAGARI, Style::dark())] {
            let labels = labels(&placed, script);
            let svg = render(&placed, &labels, &style).unwrap();
            let document = Document::parse(&svg)
                .unwrap_or_else(|err| panic!("{}: not XML: {err}", layout.key));
            let root = document.root_element();
            assert_eq!(root.tag_name().name(), "svg");
            assert_eq!(
                root.tag_name().namespace(),
                Some("http://www.w3.org/2000/svg")
            );
            assert_eq!(root.attribute("viewBox"), Some("0 0 1000 1000"));

            // The title reads back exactly, escaping and all.
            let title = all(&document, "title");
            assert_eq!(title[0].text(), labels.title.as_deref());

            // One group per cell, each saying which sign and house it is.
            let groups = all(&document, "g");
            assert_eq!(groups.len(), placed.cells.len(), "{}", layout.key);
            for (group, cell) in groups.iter().zip(&placed.cells) {
                assert_eq!(group.attribute("data-sign"), Some(cell.sign.full_key()));
                assert_eq!(
                    group.attribute("data-house"),
                    Some(cell.house.to_string().as_str())
                );
                assert_eq!(group.attribute("data-lagna").is_some(), cell.lagna);
            }

            // Every body is written once per ring, in its own name: stacked
            // in its cell in each of the chakra's rings, which count their
            // houses from different places, or marked once at its degree
            // on the wheel.
            let rings = if placed.marks.is_empty() {
                placed
                    .cells
                    .iter()
                    .map(|cell| usize::from(cell.ring))
                    .max()
                    .unwrap()
                    + 1
            } else {
                1
            };
            let bodies: Vec<_> = all(&document, "text")
                .into_iter()
                .filter_map(|text| Some((text.attribute("data-body")?, text.text()?)))
                .filter(|(body, _)| !body.starts_with("point."))
                .collect();
            assert_eq!(
                bodies.len(),
                GRAHAS.len() * rings,
                "{}: {bodies:?}",
                layout.key
            );
            for (graha, name) in GRAHAS.iter().zip(script) {
                let key = graha.key_id().to_string();
                assert!(
                    bodies
                        .iter()
                        .any(|(body, text)| *body == key && text == name),
                    "{}: {key} is not written as {name}",
                    layout.key
                );
            }

            // No stylesheet, and no attribute a renderer may ignore.
            assert!(all(&document, "style").is_empty());
            assert!(!svg.contains("dominant-baseline"));
        }
    }
}

#[test]
fn the_same_chart_draws_the_same_bytes() {
    for layout in rows::shipped() {
        let placed = drawn(&layout);
        let labels = labels(&placed, &DEVANAGARI);
        let first = render(&placed, &labels, &Style::light()).unwrap();
        let again = render(&drawn(&layout), &labels, &Style::light()).unwrap();
        assert_eq!(first, again, "{}", layout.key);
    }
}

#[test]
fn the_lagna_is_written_first_in_its_own_cell_and_only_in_the_innermost_ring() {
    for layout in rows::shipped() {
        let placed = drawn(&layout);
        let svg = render(&placed, &labels(&placed, &LATIN), &Style::light()).unwrap();
        let document = Document::parse(&svg).unwrap();
        let lagnas: Vec<_> = all(&document, "text")
            .into_iter()
            .filter(|text| text.attribute("data-body") == Some("point.LAGNA"))
            .collect();
        if placed.marks.is_empty() {
            assert_eq!(lagnas.len(), 1, "{}", layout.key);
            let group = lagnas[0].parent_element().unwrap();
            assert_eq!(group.attribute("data-lagna"), Some("true"));
            let first = group
                .children()
                .find(|child| child.attribute("data-body").is_some())
                .unwrap();
            assert_eq!(first, lagnas[0], "{}: the lagna is not first", layout.key);
            assert_eq!(
                first.attribute("fill"),
                Some(Style::light().accent.as_str())
            );
        } else {
            assert!(lagnas.is_empty(), "{}: a wheel marks no lagna", layout.key);
        }
    }
}

/// A number attribute of an element.
fn number(node: &Node<'_, '_>, name: &str) -> f64 {
    node.attribute(name).unwrap().parse().unwrap()
}

#[test]
fn a_crowded_cell_keeps_every_line_inside_it_and_off_its_label() {
    // Every graha and the lagna in one sign: ten lines in one cell.
    let crowded = Placements {
        bodies: GRAHAS
            .iter()
            .map(|graha| Body::in_sign(graha.key_id(), Rashi::Cancer))
            .collect(),
        ..Placements::new(Rashi::Cancer)
    };
    let style = Style::light();
    let scale = style.size;
    for layout in rows::shipped()
        .iter()
        .filter(|layout| layout.shape.as_grid().is_some())
    {
        let placed = place(layout, &crowded).unwrap();
        let svg = render(&placed, &labels(&placed, &DEVANAGARI), &style).unwrap();
        let document = Document::parse(&svg).unwrap();
        for (group, cell) in all(&document, "g").iter().zip(&placed.cells) {
            let texts: Vec<_> = group
                .children()
                .filter(|child| child.tag_name().name() == "text")
                .collect();
            let Some((label, lines)) = texts.split_first() else {
                continue;
            };
            // Each text's estimated box, in the unit square: centre and half
            // extents, undoing the baseline shift.
            let boxed = |node: &Node<'_, '_>| {
                let size = number(node, "font-size") / scale;
                let chars = node.text().unwrap().chars().count();
                (
                    number(node, "x") / scale,
                    number(node, "y") / scale - size * style.baseline_shift,
                    to_f64(chars) * style.advance * size / 2.0,
                    style.line_height * size / 2.0,
                )
            };
            let (lx, ly, lw, lh) = boxed(label);
            // The printed hundredths of a unit are the only slack.
            let slack = 0.01 / scale;
            for line in lines {
                let (x, y, w, h) = boxed(line);
                let apart = (x - lx).abs() + slack >= w + lw || (y - ly).abs() + slack >= h + lh;
                assert!(
                    apart,
                    "{}: {:?} covers {}'s label",
                    layout.key,
                    line.text(),
                    cell.sign.full_key()
                );
                for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                    let corner =
                        teistro_geometry::Point::new(x + dx * (w - slack), y + dy * (h - slack));
                    assert!(
                        cell.outline.contains(corner),
                        "{}: {:?} leaves {}",
                        layout.key,
                        line.text(),
                        cell.sign.full_key()
                    );
                }
            }
        }
    }
}

#[expect(clippy::cast_precision_loss, reason = "a label is a few characters")]
const fn to_f64(count: usize) -> f64 {
    count as f64
}

#[test]
fn zodiac_glyphs_are_drawn_as_text_not_emoji() {
    let placed = drawn(&rows::south_indian());
    let mut labels = labels(&placed, &LATIN);
    labels.cells = placed
        .cells
        .iter()
        .map(|cell| {
            char::from_u32(0x2648 + u32::from(cell.sign.id()))
                .unwrap()
                .to_string()
        })
        .collect();
    let svg = render(&placed, &labels, &Style::light()).unwrap();
    assert_eq!(
        svg.matches('\u{FE0E}').count(),
        12,
        "one selector after each sign"
    );
}

#[test]
fn the_renderer_refuses_what_it_cannot_draw_by_field() {
    let placed = drawn(&rows::east_indian());
    let good = labels(&placed, &LATIN);

    let mut style = Style::light();
    style.ink = String::from("ink");
    let err = render(&placed, &good, &style).unwrap_err();
    assert_eq!(err.field(), Some("style.ink"));

    let mut nameless = good.clone();
    nameless.bodies.remove(&Graha::Ketu.key_id());
    let err = render(&placed, &nameless, &Style::light()).unwrap_err();
    assert_eq!(err.field(), Some("labels.bodies"));
    assert!(err.message.contains("graha.KETU"), "{}", err.message);
}

//! A document's drawing written as SVG through the façade: the words come
//! from the context's locale and the theme, and the bytes from the renderer
//! (`03-design/render-svg.md` §3).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{ChartLayout, Graha, Varga};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::render_svg::{BodyForm, CellLabel, Content, Theme, render};
use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};

fn context(locale: &str) -> Context {
    Context::builder()
        .profile("nepali-default")
        .locale(locale)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a shipped locale and the built-in ephemeris")
}

/// A Kathmandu chart on 15 August 2024, inside Mercury's retrograde (5 to 28
/// August) and Saturn's (from 30 June), drawn three ways.
fn reading(sdk: &Context) -> Document {
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20700).unwrap())
        .with_vargas([Varga::D9])
        .with_drawings([
            (ChartLayout::NorthIndian, Varga::D1),
            (ChartLayout::SouthIndian, Varga::D9),
            (ChartLayout::WesternWheel, Varga::D1),
        ]);
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(2_460_537.5), &request)
        .expect("a reading")
        .value
}

/// The text content of every `<text>` the drawing writes for a body.
fn written(svg: &str, body: &str) -> Vec<String> {
    let opening = format!("data-body=\"{body}\">");
    svg.match_indices(&opening)
        .map(|(at, _)| {
            let rest = &svg[at + opening.len()..];
            rest[..rest.find("</text>").unwrap()].to_owned()
        })
        .collect()
}

#[test]
fn a_nepali_chart_is_written_in_devanagari_and_an_english_one_in_latin() {
    for (locale, sun, digits) in [("ne-Deva-NP", "सू", '०'..='९'), ("en-Latn", "Su", '0'..='9')]
    {
        let sdk = context(locale);
        let document = reading(&sdk);
        let svg = sdk.chart().svg(&document, 0, &Theme::light()).unwrap();
        assert!(
            written(&svg, "graha.SUN")[0].starts_with(sun),
            "{locale}: {:?}",
            written(&svg, "graha.SUN")
        );
        // Every cell's label is the sign's number in the locale's digits.
        let labels = sdk
            .chart()
            .labels(&document, 0, &Content::default())
            .unwrap();
        for (label, cell) in labels.cells.iter().zip(&document.drawings[0].placed.cells) {
            assert!(
                label.chars().all(|c| digits.contains(&c)),
                "{locale}: {label}"
            );
            let number: String = label
                .chars()
                .map(|c| char::from_digit(u32::from(c) - u32::from(*digits.start()), 10).unwrap())
                .collect();
            assert_eq!(number, (cell.sign.id() + 1).to_string());
        }
    }
}

#[test]
fn the_svg_is_the_labels_rendered_and_is_the_same_bytes_twice() {
    let sdk = context("ne-Deva-NP");
    let document = reading(&sdk);
    let theme = Theme::dark();
    for drawing in 0..document.drawings.len() {
        let labels = sdk
            .chart()
            .labels(&document, drawing, &theme.content)
            .unwrap();
        let by_hand = render(&document.drawings[drawing].placed, &labels, &theme.style).unwrap();
        let svg = sdk.chart().svg(&document, drawing, &theme).unwrap();
        assert_eq!(svg, by_hand);
        assert_eq!(svg, sdk.chart().svg(&document, drawing, &theme).unwrap());
    }
}

#[test]
fn a_retrograde_graha_is_marked_in_every_chart_and_a_node_never() {
    let sdk = context("en-Latn");
    let document = reading(&sdk);
    let retrograde: Vec<Graha> = document
        .foundation
        .grahas
        .iter()
        .filter(|position| position.is_retrograde())
        .map(|position| position.graha)
        .collect();
    assert!(
        retrograde
            .iter()
            .any(|graha| !matches!(graha, Graha::Rahu | Graha::Ketu)),
        "the fixture needs a retrograde graha: {retrograde:?}"
    );
    for drawing in 0..document.drawings.len() {
        let labels = sdk
            .chart()
            .labels(&document, drawing, &Content::default())
            .unwrap();
        for position in &document.foundation.grahas {
            let name = &labels.bodies[&position.graha.key_id()];
            let node = matches!(position.graha, Graha::Rahu | Graha::Ketu);
            assert_eq!(
                name.ends_with("(R)"),
                position.is_retrograde() && !node,
                "drawing {drawing}: {:?} is written {name}",
                position.graha
            );
        }
    }
}

#[test]
fn degrees_are_written_on_the_founded_chart_only() {
    let sdk = context("en-Latn");
    let document = reading(&sdk);
    let content = Content {
        degrees: true,
        retrograde_mark: None,
        ..Content::default()
    };
    let founded = sdk.chart().labels(&document, 0, &content).unwrap();
    let navamsha = sdk.chart().labels(&document, 1, &content).unwrap();
    for position in &document.foundation.grahas {
        let key = position.graha.key_id();
        let degree = position.longitude_deg.rem_euclid(30.0).floor();
        assert!(
            founded.bodies[&key].ends_with(&format!(" {degree}°")),
            "{}",
            founded.bodies[&key]
        );
        assert!(
            !navamsha.bodies[&key].contains('°'),
            "{}",
            navamsha.bodies[&key]
        );
    }
}

#[test]
fn a_wheel_labels_its_houses_and_its_signs_by_glyph_as_text() {
    let sdk = context("en-Latn");
    let document = reading(&sdk);
    let wheel = &document.drawings[2].placed;
    let marked = wheel.marks[0].ring;
    let labels = sdk
        .chart()
        .labels(&document, 2, &Content::default())
        .unwrap();
    for (label, cell) in labels.cells.iter().zip(&wheel.cells) {
        if cell.ring == marked {
            assert_eq!(label, &cell.house.to_string());
        } else {
            let glyph = char::from_u32(0x2648 + u32::from(cell.sign.id())).unwrap();
            assert_eq!(label, &glyph.to_string());
        }
    }
    let svg = sdk.chart().svg(&document, 2, &Theme::light()).unwrap();
    assert_eq!(
        svg.matches('\u{FE0E}').count(),
        12,
        "each sign asks for text"
    );

    // The glyph form writes the grahas as symbols, and a chosen label
    // overrides the wheel's own.
    let content = Content {
        body_form: BodyForm::Glyph,
        cell_label: CellLabel::Nothing,
        ..Content::default()
    };
    let labels = sdk.chart().labels(&document, 2, &content).unwrap();
    assert!(labels.bodies[&Graha::Sun.key_id()].starts_with('☉'));
    assert!(labels.cells.iter().all(String::is_empty));
}

#[test]
fn a_drawing_the_document_does_not_have_is_refused() {
    let sdk = context("en-Latn");
    let document = reading(&sdk);
    let error = sdk.chart().svg(&document, 3, &Theme::light()).unwrap_err();
    assert_eq!(error.field(), Some("drawing"));
    assert!(error.message.contains("3 drawings"), "{error}");
}

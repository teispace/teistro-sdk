//! `render` and `check-render`: the golden drawings, a real chart drawn in
//! every shipped layout, in two locales and both themes, compared byte for
//! byte (`docs/03-design/render-svg.md` §5).
//!
//! The drawings are made through the façade, so the gate holds the whole
//! path a consumer takes: the chart founded, placed in each layout, its
//! words composed from the locale and the theme, and the SVG written. A
//! change to any of them shows as a changed drawing in the pull request
//! that made it, which a reviewer can open and look at.

use std::path::Path;

use teistro::catalogue::{ChartLayout, Varga};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::render_svg::Theme;
use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};

use crate::generated::{Output, check, prune, strays, write};

/// Where the drawings live: beside the renderer they gate.
const DIR: &str = "crates/render-svg/golden";

/// Every drawing: a layout, the chart it draws, and its name's part for it.
const DRAWINGS: [(ChartLayout, Varga); 7] = [
    (ChartLayout::NorthIndian, Varga::D1),
    (ChartLayout::SouthIndian, Varga::D1),
    (ChartLayout::EastIndian, Varga::D1),
    (ChartLayout::NepaliLotus, Varga::D1),
    (ChartLayout::SudarshanChakra, Varga::D1),
    (ChartLayout::WesternWheel, Varga::D1),
    (ChartLayout::SouthIndian, Varga::D9),
];

/// The locales and themes each drawing is made in: a Nepali patrika on
/// paper, and an English chart on a dark page.
fn looks() -> [(&'static str, &'static str, Theme); 2] {
    [
        ("ne-Deva-NP", "light", Theme::light()),
        ("en-Latn", "dark", Theme::dark()),
    ]
}

fn outputs() -> Result<Vec<Output>, String> {
    // Kathmandu, 15 August 2024 at 05:45 local: Mercury and Saturn
    // retrograde, so the marks are drawn.
    let place = Place::new(
        Latitude::try_new(27.7172).map_err(|e| e.to_string())?,
        Longitude::try_new(85.324).map_err(|e| e.to_string())?,
        Altitude::try_new(1400.0).map_err(|e| e.to_string())?,
    );
    let offset = UtcOffset::try_from_seconds(20_700).map_err(|e| e.to_string())?;
    let request = ChartRequest::at(place, offset)
        .with_vargas([Varga::D9])
        .with_drawings(DRAWINGS);
    let mut outputs = Vec::new();
    for (locale, look, theme) in looks() {
        let sdk = Context::builder()
            .profile("nepali-default")
            .locale(locale)
            .ephemeris([Ephemeris::Builtin])
            .build()
            .map_err(|e| e.to_string())?;
        let document = sdk
            .chart()
            .reading(JulianDay::<Utc>::literal(2_460_537.5), &request)
            .map_err(|e| e.to_string())?
            .value;
        for (index, (layout, varga)) in DRAWINGS.iter().enumerate() {
            let svg = sdk
                .chart()
                .svg(&document, index, &theme)
                .map_err(|e| e.to_string())?;
            let name = format!(
                "{DIR}/{}-{}-{locale}-{look}.svg",
                layout.key().to_lowercase().replace('_', "-"),
                varga.key().to_lowercase(),
            );
            outputs.push(Output::new(name, svg));
        }
    }
    Ok(outputs)
}

pub(crate) fn generate(root: &Path) -> i32 {
    match outputs() {
        Ok(outputs) => {
            let written = write(root, &outputs);
            prune(root, DIR, &outputs);
            written
        }
        Err(err) => refused(&err),
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs() {
        Ok(outputs) => {
            let mut failures = check(root, &outputs, "cargo xtask render");
            for stray in strays(root, DIR, &outputs) {
                println!(
                    "FAIL  {stray} is a drawing no golden claims; `cargo xtask render` removes it"
                );
                failures += 1;
            }
            i32::from(failures != 0)
        }
        Err(err) => refused(&err),
    }
}

fn refused(err: &str) -> i32 {
    println!("FAIL  {err}");
    1
}

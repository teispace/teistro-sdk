//! A chart document's drawing, written as SVG in the context's locale
//! (`03-design/render-svg.md` §3).
//!
//! The renderer knows no astrology and no locale; this is where a drawing's
//! words come from. Every label is a form the locale vets (`short`, `glyph`)
//! or a number in the locale's own digits, so a Nepali chart is written
//! entirely in Devanagari and an English one in Latin.

use std::collections::BTreeMap;

use teistro_core::catalogue::{Graha, Varga};
use teistro_core::error::Error;
use teistro_core::key::KeyId;
use teistro_geometry::{Drawing, Placed, PlacedCell};
use teistro_intl::render::NumberStyle;
use teistro_render_svg::{BodyForm, CellLabel, Content, Labels, Theme, render};
use teistro_serial::Document;

use crate::area::ChartArea;

impl ChartArea<'_> {
    /// The words a document's drawing is written with, in the context's
    /// locale: for a consumer who adjusts them before rendering, or draws
    /// with a renderer of their own.
    ///
    /// # Errors
    ///
    /// A drawing index the document does not have.
    pub fn labels(
        self,
        document: &Document,
        drawing: usize,
        content: &Content,
    ) -> Result<Labels, Error> {
        let drawn = drawing_at(document, drawing)?;
        Ok(Composer::of(self, content).labels(document, drawn))
    }

    /// A document's drawing as SVG, in the context's locale and the
    /// theme's style: the same bytes for the same chart on every platform.
    ///
    /// ```
    /// use teistro::catalogue::{ChartLayout, Varga};
    /// use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// use teistro::render_svg::Theme;
    /// use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
    ///
    /// let sdk = Context::builder().locale("ne-Deva-NP").ephemeris([Ephemeris::Test]).build()?;
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?)
    ///     .with_drawings([(ChartLayout::NorthIndian, Varga::D1)]);
    /// let document = sdk.chart().reading(JulianDay::<Utc>::literal(2_451_545.0), &request)?.value;
    ///
    /// let svg = sdk.chart().svg(&document, 0, &Theme::light())?;
    /// assert!(svg.contains("data-body=\"graha.SUN\">सू")); // as a Nepali patrika writes it
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A drawing index the document does not have, or a style the renderer
    /// refuses.
    pub fn svg(self, document: &Document, drawing: usize, theme: &Theme) -> Result<String, Error> {
        let drawn = drawing_at(document, drawing)?;
        let labels = Composer::of(self, &theme.content).labels(document, drawn);
        render(&drawn.placed, &labels, &theme.style)
    }
}

fn drawing_at(document: &Document, index: usize) -> Result<&Drawing, Error> {
    document.drawings.get(index).ok_or_else(|| {
        Error::invalid_arg(format!(
            "the document has {} drawings, and {index} is not one",
            document.drawings.len()
        ))
        .with_field(String::from("drawing"))
        .with_hint(String::from(
            "ask for drawings with ChartRequest::with_drawings, and count from 0",
        ))
    })
}

/// What one set of labels is composed from: the context's locale and the
/// theme's content.
struct Composer<'c> {
    area: ChartArea<'c>,
    content: &'c Content,
    /// The locale's digits; ASCII where the locale is not loaded.
    numbers: Option<NumberStyle>,
}

impl<'c> Composer<'c> {
    fn of(area: ChartArea<'c>, content: &'c Content) -> Composer<'c> {
        let numbers = {
            let engine = area.context().locale_engine();
            engine
                .chain_from(engine.locale())
                .first()
                .map(|locale| NumberStyle::of(&locale.meta))
        };
        Composer {
            area,
            content,
            numbers,
        }
    }

    fn labels(&self, document: &Document, drawing: &Drawing) -> Labels {
        let placed = &drawing.placed;
        Labels {
            title: None,
            cells: placed
                .cells
                .iter()
                .map(|cell| self.cell_label(placed, cell))
                .collect(),
            bodies: self.bodies(document, drawing),
            lagna: self
                .content
                .lagna_mark
                .then(|| self.form("point.LAGNA", "short")),
        }
    }

    fn cell_label(&self, placed: &Placed, cell: &PlacedCell) -> String {
        let marked_ring = placed.marks.first().map(|mark| mark.ring);
        let wanted = match (self.content.cell_label, marked_ring) {
            (CellLabel::Auto, None) => CellLabel::SignNumber,
            (CellLabel::Auto, Some(ring)) if ring == cell.ring => CellLabel::House,
            (CellLabel::Auto, Some(_)) => CellLabel::SignGlyph,
            (chosen, _) => chosen,
        };
        match wanted {
            CellLabel::SignNumber => self.number(u32::from(cell.sign.id()) + 1),
            CellLabel::House => self.number(u32::from(cell.house)),
            CellLabel::SignShort => self.form(cell.sign.full_key(), "short"),
            CellLabel::SignGlyph => self.form(cell.sign.full_key(), "glyph"),
            CellLabel::Nothing | CellLabel::Auto => String::new(),
        }
    }

    /// Every body the drawing places, by its key: its form, then its degree
    /// on the founded chart and its retrograde mark on any, when the theme
    /// asks. A degree belongs to the founded chart's longitude, so a
    /// divisional drawing writes none; being retrograde is the graha's, so
    /// it is marked in every chart.
    fn bodies(&self, document: &Document, drawing: &Drawing) -> BTreeMap<KeyId, String> {
        let form = match self.content.body_form {
            BodyForm::Short => "short",
            BodyForm::Glyph => "glyph",
        };
        let placed = &drawing.placed;
        let keys = placed
            .cells
            .iter()
            .flat_map(|cell| cell.bodies.iter().copied())
            .chain(placed.marks.iter().map(|mark| mark.body));
        let mut bodies = BTreeMap::new();
        for key in keys {
            bodies.entry(key).or_insert_with(|| {
                let mut words = self.form(&key.to_string(), form);
                if let Some(position) = document
                    .foundation
                    .grahas
                    .iter()
                    .find(|position| position.graha.key_id() == key)
                {
                    if self.content.degrees && drawing.varga == Varga::D1 {
                        #[expect(
                            clippy::cast_possible_truncation,
                            clippy::cast_sign_loss,
                            reason = "a normalised longitude's degree in its sign is 0 to 29"
                        )]
                        let degree = (position.longitude_deg.rem_euclid(30.0)).floor() as u32;
                        words.push(' ');
                        words.push_str(&self.number(degree));
                        words.push('°');
                    }
                    // The nodes' mean motion is always backwards, so a mark
                    // on them would say nothing.
                    let node = matches!(position.graha, Graha::Rahu | Graha::Ketu);
                    if let Some(mark) = &self.content.retrograde_mark
                        && position.is_retrograde()
                        && !node
                    {
                        words.push_str(mark);
                    }
                }
                words
            });
        }
        bodies
    }

    /// An entity's form in the locale, walking the locale's fallbacks form by
    /// form; the `name` form where no locale carries the one asked for, and
    /// the bare key where none carries the entity, so a gap is visible
    /// rather than blank.
    fn form(&self, key: &str, form: &str) -> String {
        let engine = self.area.context().locale_engine();
        let chain = engine.chain_from(engine.locale());
        chain
            .iter()
            .find_map(|locale| locale.entity(key)?.form(form))
            .or_else(|| {
                chain
                    .iter()
                    .find_map(|locale| Some(locale.entity(key)?.name()))
            })
            .map_or_else(
                || key.rsplit('.').next().unwrap_or(key).to_owned(),
                str::to_owned,
            )
    }

    fn number(&self, value: u32) -> String {
        let ascii = value.to_string();
        match &self.numbers {
            Some(style) => style.localise_with(&ascii, false),
            None => ascii,
        }
    }
}

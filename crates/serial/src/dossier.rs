//! The **dossier**: a chart document as plain text for a language model
//! (`03-design/dossier.md`).
//!
//! Every section the document carries, one banner each, rows of
//! `|`-separated fields under a header line, and a legend written from
//! the same column tables the headers are, so a column without a meaning
//! cannot be written. It renders what was computed and nothing else: no
//! clock, no ephemeris, no locale pack, so the same document under the
//! same options is the same bytes everywhere.
//!
//! ```
//! use teistro_serial::dossier::{DossierOptions, dossier};
//! # fn chart() -> teistro_serial::document::Document { todo!() }
//! # fn main() -> Result<(), teistro_core::error::Error> {
//! # if false {
//! let text = dossier(&chart(), &DossierOptions::default(), &[])?;
//! assert!(text.starts_with("# teistro dossier 1\n"));
//! # }
//! # Ok(())
//! # }
//! ```

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use teistro_calendar::FixedDay;
use teistro_calendar::gregorian::gregorian_from_fixed;
use teistro_core::catalogue::{Graha, Nakshatra, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::AyanamshaChoice;

use crate::document::Document;

/// The format's version, written on the first line. Raised by any change
/// to a byte the dossier writes: a column, a legend line, a number's
/// decimals.
pub const DOSSIER_FORMAT: u32 = 1;

/// One column of a section: its name in the header line and what it is,
/// in the legend. The one table both are written from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Column {
    /// The name in the header line.
    pub name: &'static str,
    /// What the field holds, for the legend.
    pub meaning: &'static str,
}

/// A column, as the section tables write one.
#[must_use]
pub const fn column(name: &'static str, meaning: &'static str) -> Column {
    Column { name, meaning }
}

/// What every section is written under: the "now" of a dated dossier, the
/// clock instants are written in and how deep a dasha goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// The instant a dated dossier is read at; `None`, date-free.
    pub at: Option<JulianDay<Utc>>,
    /// The clock instants are written in, seconds east of UTC.
    pub utc_offset_seconds: i32,
    /// How many levels a dasha timeline is written to; every level
    /// computed when `None`.
    pub dasha_depth: Option<u8>,
}

/// One section of a dossier: the built-in ones and a consumer's alike.
///
/// A section declares its columns once, and the header line and the
/// legend are both written from them. `rows` answers `None` when the
/// document does not carry what the section reads, so a preset leaves it
/// out and an explicit request for it is refused.
pub trait DossierSection: Sync {
    /// The banner and the key a request names it by: `GRAHAS`.
    fn key(&self) -> &str;
    /// The columns, in order.
    fn columns(&self) -> &[Column];
    /// Lines the legend adds after the columns' own.
    fn notes(&self) -> &[&str] {
        &[]
    }
    /// What carries it, for the refusal's hint: `` `shadbala: true` on
    /// the chart request``.
    fn asked_by(&self) -> &str;
    /// The rows, each a field a column; `None` when the document does not
    /// carry the section.
    fn rows(&self, document: &Document, view: &View) -> Option<Vec<Vec<String>>>;
}

/// A named list of sections; presets choose sections and never thin them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DossierPreset {
    /// Every section the document carries.
    #[default]
    Full,
    /// `CHART`, `GRAHAS`, `DASHAS` and `PANCHANGA`, those the document
    /// carries: one person among several in a prompt.
    Family,
}

impl DossierPreset {
    /// The built-in sections it names; every one for [`DossierPreset::Full`].
    #[must_use]
    pub fn sections(self) -> &'static [&'static str] {
        match self {
            DossierPreset::Full => &BUILT_IN_KEYS,
            DossierPreset::Family => &["CHART", "GRAHAS", "DASHAS", "PANCHANGA"],
        }
    }
}

/// What a dossier writes and how.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DossierOptions {
    /// The sections to write, by key, in the dossier's own order; the
    /// preset's when left out. Every one named must be carried.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<String>>,
    /// The sections when `sections` is left out.
    #[serde(default)]
    pub preset: DossierPreset,
    /// The "now" a dated dossier is read at, a UTC Julian day: the running
    /// dasha periods are marked and their branch expanded. Date-free when
    /// left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<f64>,
    /// How many levels a dasha timeline is written to, one up; every level
    /// computed when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dasha_depth: Option<u8>,
    /// The clock instants are written in, seconds east of UTC; UTC when
    /// left out, and a chart request's own clock for its `dossier`
    /// record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utc_offset_seconds: Option<i32>,
}

impl DossierOptions {
    /// The options checked, as the view a section is written under.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on an `at` that is no instant, a `dashaDepth` of
    /// zero, or a clock more than a day from UTC.
    pub fn view(&self) -> Result<View, Error> {
        let at = self
            .at
            .map(|jd| {
                JulianDay::<Utc>::try_new(jd).map_err(|why| Error::from(why).with_field("at"))
            })
            .transpose()?;
        if self.dasha_depth == Some(0) {
            return Err(
                Error::invalid_arg("a timeline is written to one level at least")
                    .with_field("dashaDepth"),
            );
        }
        let offset = self.utc_offset_seconds.unwrap_or(0);
        if offset.unsigned_abs() >= 86_400 {
            return Err(Error::invalid_arg(format!(
                "a clock is less than a day from UTC, not {offset} seconds"
            ))
            .with_field("utcOffsetSeconds"));
        }
        Ok(View {
            at,
            utc_offset_seconds: offset,
            dasha_depth: self.dasha_depth,
        })
    }

    /// The options a binding writes as a chart request's `dossier` record,
    /// read and checked; a refusal names the field the caller wrote,
    /// `dossier.sections[0]`. A section is known when it is built in or one
    /// of `extra`, the sections the reader writes of its own: a chart
    /// request's `RULES`.
    ///
    /// ```
    /// use teistro_serial::dossier::{DossierOptions, DossierPreset};
    ///
    /// let asked = DossierOptions::from_json(r#"{"preset": "FAMILY", "dashaDepth": 2}"#, &[])?;
    /// assert_eq!(asked.preset, DossierPreset::Family);
    /// let typo = DossierOptions::from_json(r#"{"section": ["GRAHAS"]}"#, &[]).unwrap_err();
    /// assert!(typo.field().is_some_and(|field| field.starts_with("dossier")));
    /// let unknown = DossierOptions::from_json(r#"{"sections": ["RULES"]}"#, &[]).unwrap_err();
    /// assert_eq!(unknown.field(), Some("dossier.sections[0]"));
    /// let own = DossierOptions::from_json(r#"{"sections": ["GRAHAS", "RULES"]}"#, &["RULES"])?;
    /// assert_eq!(own.sections.as_deref().map(<[String]>::len), Some(2));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a section neither built in nor in `extra`, or what
    /// [`DossierOptions::view`] refuses.
    pub fn from_json(text: &str, extra: &[&str]) -> Result<DossierOptions, Error> {
        let asked: DossierOptions = teistro_core::strict::read(text, DOSSIER)?;
        asked.view().map_err(|why| why.under(DOSSIER))?;
        for (at, key) in asked.sections.iter().flatten().enumerate() {
            if !BUILT_IN_KEYS.contains(&key.as_str()) && !extra.contains(&key.as_str()) {
                return Err(Error::invalid_arg(format!(
                    "`{key}` is not a dossier section; there are {}",
                    BUILT_IN_KEYS
                        .iter()
                        .chain(extra)
                        .copied()
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .with_field(format!("{DOSSIER}.sections[{at}]")));
            }
        }
        Ok(asked)
    }
}

/// The record every binding writes the options as.
const DOSSIER: &str = "dossier";

/// The dossier of `document`: the format line, the legend of the sections
/// written, and each section, built-in and `extra`, in order.
///
/// # Errors
///
/// `INVALID_ARG` naming `sections[i]` for a key no section has, or one
/// the document does not carry (with what would carry it as the hint),
/// and whatever [`DossierOptions::view`] refuses.
pub fn dossier(
    document: &Document,
    options: &DossierOptions,
    extra: &[&dyn DossierSection],
) -> Result<String, Error> {
    let view = options.view()?;
    let all: Vec<&dyn DossierSection> = BUILT_IN
        .iter()
        .copied()
        .chain(extra.iter().copied())
        .collect();
    let mut written: Vec<(&dyn DossierSection, Vec<Vec<String>>)> = Vec::new();
    if let Some(asked) = &options.sections {
        for (at, key) in asked.iter().enumerate() {
            let Some(section) = all.iter().find(|one| one.key() == key) else {
                return Err(Error::invalid_arg(format!(
                    "`{key}` is not a dossier section; there are {}",
                    all.iter()
                        .map(|one| one.key())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .with_field(format!("sections[{at}]")));
            };
            let Some(rows) = section.rows(document, &view) else {
                return Err(Error::invalid_arg(format!(
                    "the chart does not carry `{key}`, so its dossier cannot write it"
                ))
                .with_field(format!("sections[{at}]"))
                .with_hint(format!("ask with {}", section.asked_by())));
            };
            written.push((*section, rows));
        }
        // The dossier's own order, whatever order they were named in.
        written.sort_by_key(|(section, _)| {
            all.iter()
                .position(|one| one.key() == section.key())
                .unwrap_or(usize::MAX)
        });
        written.dedup_by(|a, b| a.0.key() == b.0.key());
    } else {
        let named = options.preset.sections();
        for section in &all {
            let builtin = BUILT_IN_KEYS.contains(&section.key());
            if builtin && !named.contains(&section.key()) {
                continue;
            }
            if let Some(rows) = section.rows(document, &view) {
                written.push((*section, rows));
            }
        }
    }
    let mut out = format!("# teistro dossier {DOSSIER_FORMAT}\n");
    out.push_str(&legend(written.iter().map(|(section, _)| *section)));
    for (section, rows) in &written {
        let _ = writeln!(out, "== {} ==", section.key());
        let columns = section.columns();
        if !columns.is_empty() {
            out.push_str("# ");
            out.push_str(
                &columns
                    .iter()
                    .map(|one| one.name)
                    .collect::<Vec<_>>()
                    .join("|"),
            );
            out.push('\n');
        }
        for row in rows {
            debug_assert_eq!(
                row.len(),
                columns.len(),
                "{}: a field a column",
                section.key()
            );
            out.push_str(
                &row.iter()
                    .map(|field| clean(field))
                    .collect::<Vec<_>>()
                    .join("|"),
            );
            out.push('\n');
        }
    }
    Ok(out)
}

/// The legend of `sections` alone, for a prompt that holds several
/// people's dossiers under one legend.
#[must_use]
pub fn dossier_legend(sections: &[&dyn DossierSection]) -> String {
    legend(sections.iter().copied())
}

/// The built-in sections, in the dossier's order.
#[must_use]
pub fn built_in() -> &'static [&'static dyn DossierSection] {
    &BUILT_IN
}

fn legend<'a>(sections: impl Iterator<Item = &'a dyn DossierSection>) -> String {
    let mut out = String::from(
        "== LEGEND ==\n\
         # Each section opens `== NAME ==`; a `#` line under it names its columns, `|`-separated.\n\
         # `-` is a value the chart does not have. Members are catalogue keys (JUPITER, ARIES).\n",
    );
    for section in sections {
        let columns = section.columns();
        let _ = writeln!(
            out,
            "# {}: {}",
            section.key(),
            columns
                .iter()
                .map(|one| one.name)
                .collect::<Vec<_>>()
                .join("|")
        );
        for one in columns {
            let _ = writeln!(out, "#   {}: {}", one.name, one.meaning);
        }
        for note in section.notes() {
            let _ = writeln!(out, "#   {note}");
        }
    }
    out
}

/// A field as a row may hold it: no separator and no line break.
fn clean(field: &str) -> String {
    if field.is_empty() {
        return String::from("-");
    }
    field.replace(['|', '\n', '\r'], " ")
}

/// A number to `decimals`, with no sign on a zero.
fn fixed(value: f64, decimals: usize) -> String {
    let written = format!("{value:.decimals$}");
    if written
        .trim_start_matches('-')
        .chars()
        .all(|c| c == '0' || c == '.')
    {
        written.trim_start_matches('-').to_owned()
    } else {
        written
    }
}

/// A value as serde spells it, quotes off: `GREAT_FRIEND`.
fn word<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(text)) => text,
        Ok(other) => other.to_string(),
        Err(_) => String::from("-"),
    }
}

/// The civil minute of `jd` on a clock `offset` seconds east:
/// `1990-04-12 13:20`.
fn civil(jd: JulianDay<Utc>, offset: i32) -> String {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the minutes of a Julian day fit an i64"
    )]
    let minutes =
        ((jd.get() - FixedDay::JD_EPOCH) * 1440.0 + f64::from(offset) / 60.0).round() as i64;
    let day = minutes.div_euclid(1440);
    let minute = minutes.rem_euclid(1440);
    let (year, month, date) = gregorian_from_fixed(FixedDay::new(day));
    format!(
        "{year:04}-{month:02}-{date:02} {:02}:{:02}",
        minute / 60,
        minute % 60
    )
}

/// The civil date of `jd` on the clock: `1990-04-12`.
fn date(jd: JulianDay<Utc>, offset: i32) -> String {
    civil(jd, offset)[..10].to_owned()
}

/// The clock as written: `+05:45`.
fn clock(offset: i32) -> String {
    let sign = if offset < 0 { '-' } else { '+' };
    let minutes = offset.unsigned_abs() / 60;
    format!("{sign}{:02}:{:02}", minutes / 60, minutes % 60)
}

/// Sign, degrees in it, nakshatra and pada of a sidereal longitude.
fn place(longitude_deg: f64) -> [String; 4] {
    let longitude = longitude_deg.rem_euclid(360.0);
    let sign = Rashi::of_longitude(longitude);
    let span = 360.0 / 27.0;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude in [0, 360) over a star's span is 0 to 26"
    )]
    let star = (longitude / span).floor() as u16;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a quarter of a star's span is 0 to 3"
    )]
    let pada = ((longitude % span) / (span / 4.0)).floor() as u8 + 1;
    [
        sign.key().to_owned(),
        fixed(longitude - sign.start_deg(), 2),
        Nakshatra::from_id(star.min(26))
            .map_or_else(|| String::from("-"), |one| one.key().to_owned()),
        pada.min(4).to_string(),
    ]
}

fn dash() -> String {
    String::from("-")
}

struct ChartSection;

impl DossierSection for ChartSection {
    fn key(&self) -> &'static str {
        "CHART"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 2] = [
            column("field", "what the row says"),
            column("value", "its value"),
        ];
        &COLUMNS
    }
    fn notes(&self) -> &[&str] {
        &[
            "fields: kind; instant (civil, the clock, then the UTC Julian day); place (latitude, longitude, metres);",
            "  zodiac (TROPICAL, or SIDEREAL with the ayanamsha and its degrees); day (the civil date the chart",
            "  belongs to, and its part); lagna (sign, degrees, nakshatra, pada).",
        ]
    }
    fn asked_by(&self) -> &'static str {
        "any request"
    }
    fn rows(&self, document: &Document, view: &View) -> Option<Vec<Vec<String>>> {
        let f = &document.foundation;
        let zodiac = match &f.zodiac.ayanamsha {
            None => String::from("TROPICAL"),
            Some(AyanamshaChoice::Catalogued { id }) => {
                format!("SIDEREAL {} {}", id.key(), fixed(f.zodiac.offset_deg, 6))
            }
            Some(_) => format!("SIDEREAL CUSTOM {}", fixed(f.zodiac.offset_deg, 6)),
        };
        let lagna = place(f.lagna_deg);
        let row = |field: &str, value: String| vec![field.to_owned(), value];
        Some(vec![
            row("kind", f.kind.key().to_owned()),
            row(
                "instant",
                format!(
                    "{} {} {}",
                    civil(f.instant, view.utc_offset_seconds),
                    clock(view.utc_offset_seconds),
                    fixed(f.instant.get(), 6)
                ),
            ),
            row(
                "place",
                format!(
                    "{} {} {}",
                    fixed(f.place.latitude.get(), 4),
                    fixed(f.place.longitude.get(), 4),
                    fixed(f.place.altitude.get(), 0)
                ),
            ),
            row("zodiac", zodiac),
            row(
                "day",
                format!(
                    "{:04}-{:02}-{:02} {}",
                    f.day.day.date.year,
                    f.day.day.date.month,
                    f.day.day.date.day,
                    word(&f.day.part)
                ),
            ),
            row("lagna", lagna.join(" ")),
        ])
    }
}

struct NowSection;

impl DossierSection for NowSection {
    fn key(&self) -> &'static str {
        "NOW"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 3] = [
            column(
                "instant",
                "the \"now\" the dossier is read at, civil on the chart's clock",
            ),
            column("jd", "the same, a UTC Julian day"),
            column("age", "the native's age then, years of 365.25 days"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        "`at` on the dossier"
    }
    fn rows(&self, document: &Document, view: &View) -> Option<Vec<Vec<String>>> {
        let at = view.at?;
        Some(vec![vec![
            civil(at, view.utc_offset_seconds),
            fixed(at.get(), 6),
            fixed((at.get() - document.foundation.instant.get()) / 365.25, 2),
        ]])
    }
}

struct GrahasSection;

impl DossierSection for GrahasSection {
    fn key(&self) -> &'static str {
        "GRAHAS"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 12] = [
            column(
                "graha",
                "LAGNA, the nine, and the outer planets when placed",
            ),
            column("sign", "the sign, in the chart's zodiac"),
            column("deg", "degrees in the sign, two decimals"),
            column("nakshatra", "the star of the 27"),
            column("pada", "its quarter, 1 to 4"),
            column("house", "the house under the chart's placement system"),
            column("speed", "degrees a day"),
            column("motion", "DIRECT or RETROGRADE"),
            column("dignity", "the dignity, with the state"),
            column(
                "friendship",
                "the compound friendship with the sign's lord, with the state",
            ),
            column("combustion", "how near the Sun, with the state"),
            column("age", "the Baladi age, with the state"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        "any request"
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        let f = &document.foundation;
        let mut rows = Vec::new();
        let [sign, deg, star, pada] = place(f.lagna_deg);
        rows.push(vec![
            String::from("LAGNA"),
            sign,
            deg,
            star,
            pada,
            String::from("1"),
            dash(),
            dash(),
            dash(),
            dash(),
            dash(),
            dash(),
        ]);
        for g in f.grahas.iter().chain(&f.outer) {
            let [sign, deg, star, pada] = place(g.longitude_deg);
            let state = document
                .state
                .as_ref()
                .and_then(|all| all.iter().find(|one| one.graha == g.graha));
            let of =
                |read: &dyn Fn(&teistro_state::GrahaState) -> String| state.map_or_else(dash, read);
            rows.push(vec![
                g.graha.key().to_owned(),
                sign,
                deg,
                star,
                pada,
                g.house.bhava.to_string(),
                fixed(g.speed_deg_per_day, 4),
                String::from(if g.is_retrograde() {
                    "RETROGRADE"
                } else {
                    "DIRECT"
                }),
                of(&|s| word(&s.dignity)),
                of(&|s| word(&s.friendship.compound)),
                of(&|s| word(&s.combustion.burning)),
                of(&|s| word(&s.age)),
            ]);
        }
        Some(rows)
    }
}

struct VargasSection;

/// The nine, in the catalogue's order, a column each.
const NINE: [Graha; 9] = [
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

impl DossierSection for VargasSection {
    fn key(&self) -> &'static str {
        "VARGAS"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 11] = [
            column(
                "varga",
                "the divisional chart: D9, or grahas/lagna when they differ",
            ),
            column("LAGNA", "the lagna's sign in it"),
            column("SUN", "the Sun's sign"),
            column("MOON", "the Moon's"),
            column("MARS", "Mars's"),
            column("MERCURY", "Mercury's"),
            column("JUPITER", "Jupiter's"),
            column("VENUS", "Venus's"),
            column("SATURN", "Saturn's"),
            column("RAHU", "Rahu's"),
            column("KETU", "Ketu's"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        "`vargas` on the chart request"
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        if document.vargas.is_empty() {
            return None;
        }
        Some(
            document
                .vargas
                .iter()
                .map(|chart| {
                    let mut row = vec![chart.axis.key(), chart.lagna.sign.key().to_owned()];
                    row.extend(NINE.iter().map(|graha| {
                        chart
                            .grahas
                            .iter()
                            .find(|one| one.graha == *graha)
                            .map_or_else(dash, |one| one.at.sign.key().to_owned())
                    }));
                    row
                })
                .collect(),
        )
    }
}

struct DashasSection;

impl DossierSection for DashasSection {
    fn key(&self) -> &'static str {
        "DASHAS"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 7] = [
            column("system", "the dasha system"),
            column(
                "path",
                "where it sits: its place at each level from 0, `/`-joined (2/5)",
            ),
            column("lord", "its lord"),
            column(
                "sign",
                "the sign it is the period of, in a sign-based system",
            ),
            column("from", "the civil date it begins, on the chart's clock"),
            column("to", "the date it ends, the next period's start"),
            column("now", "`*` on a period running at the dossier's `at`"),
        ];
        &COLUMNS
    }
    fn notes(&self) -> &[&str] {
        &[
            "Date-free (no `at`): the complete timeline to the depth written.",
            "Dated: the first level whole, and below it only the running period's branch.",
            "A boundary day belongs to both periods it joins.",
        ]
    }
    fn asked_by(&self) -> &'static str {
        "`dashas` on the chart request"
    }
    fn rows(&self, document: &Document, view: &View) -> Option<Vec<Vec<String>>> {
        if document.dashas.is_empty() {
            return None;
        }
        let mut rows = Vec::new();
        for reading in &document.dashas {
            let running = |row: &teistro_dasha::PeriodRow| {
                view.at.is_some_and(|at| {
                    row.interval.from.get() <= at.get() && at.get() < row.interval.to.get()
                })
            };
            let running_paths: Vec<&str> = reading
                .periods
                .iter()
                .filter(|row| running(row))
                .map(|row| row.path.as_str())
                .collect();
            for row in &reading.periods {
                let level = row.path.split('/').count();
                if view
                    .dasha_depth
                    .is_some_and(|depth| level > usize::from(depth))
                {
                    continue;
                }
                if view.at.is_some() && level > 1 {
                    // Below the first level, only the running branch.
                    let parent = &row.path[..row.path.rfind('/').unwrap_or(0)];
                    if !running_paths.contains(&parent) {
                        continue;
                    }
                }
                rows.push(vec![
                    reading.system.key().to_owned(),
                    row.path.clone(),
                    row.lord.key().to_owned(),
                    row.sign.map_or_else(dash, |sign| sign.key().to_owned()),
                    date(row.interval.from, view.utc_offset_seconds),
                    date(row.interval.to, view.utc_offset_seconds),
                    String::from(if running(row) { "*" } else { "-" }),
                ]);
            }
        }
        Some(rows)
    }
}

struct PanchangaSection;

impl DossierSection for PanchangaSection {
    fn key(&self) -> &'static str {
        "PANCHANGA"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 2] = [
            column(
                "limb",
                "tithi, nakshatra, yoga, karana, vara, sunrise or sunset",
            ),
            column(
                "value",
                "at the chart's instant; sunrise and sunset civil on the chart's clock",
            ),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        "`panchanga: true` on the chart request"
    }
    fn rows(&self, document: &Document, view: &View) -> Option<Vec<Vec<String>>> {
        let p = document.panchanga.as_ref()?;
        let at = document.foundation.instant;
        let row = |limb: &str, value: String| vec![limb.to_owned(), value];
        Some(vec![
            row(
                "tithi",
                p.tithi_at(at)
                    .map_or_else(dash, |one| one.member.key().to_owned()),
            ),
            row(
                "nakshatra",
                p.nakshatra_at(at)
                    .map_or_else(dash, |one| one.member.key().to_owned()),
            ),
            row(
                "yoga",
                p.yoga_at(at)
                    .map_or_else(dash, |one| one.member.key().to_owned()),
            ),
            row(
                "karana",
                p.karana_at(at)
                    .map_or_else(dash, |one| one.member.key().to_owned()),
            ),
            row("vara", p.day.vara.key().to_owned()),
            row("sunrise", civil(p.day.sunrise, view.utc_offset_seconds)),
            row("sunset", civil(p.day.sunset, view.utc_offset_seconds)),
        ])
    }
}

/// A section a chart request asks for by one flag, on the chart request.
macro_rules! flag {
    ($flag:literal) => {
        concat!("`", $flag, ": true` on the chart request")
    };
}

struct HousesSection;

impl DossierSection for HousesSection {
    fn key(&self) -> &'static str {
        "HOUSES"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 5] = [
            column("house", "1 to 12"),
            column("sign", "the sign the house is read as"),
            column("lord", "its lord"),
            column(
                "madhya",
                "the house's middle, a longitude 0 to 360, two decimals",
            ),
            column(
                "sandhi",
                "where it begins, the junction with the house before",
            ),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        flag!("houses")
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        let houses = document.houses.as_ref()?;
        Some(
            (1..=12)
                .filter_map(|number| houses.bhava(number))
                .map(|bhava| {
                    vec![
                        bhava.number.to_string(),
                        bhava.sign.key().to_owned(),
                        bhava.lord.key().to_owned(),
                        fixed(bhava.madhya_deg, 2),
                        fixed(bhava.sandhi_deg, 2),
                    ]
                })
                .collect(),
        )
    }
}

/// The twelve signs, a column each, then the total.
const SIGN_COLUMNS: [Column; 14] = [
    column(
        "of",
        "SARVA, the sum of the seven, or the graha whose bindus the row counts",
    ),
    column("ARIES", "bindus in Aries"),
    column("TAURUS", "in Taurus"),
    column("GEMINI", "in Gemini"),
    column("CANCER", "in Cancer"),
    column("LEO", "in Leo"),
    column("VIRGO", "in Virgo"),
    column("LIBRA", "in Libra"),
    column("SCORPIO", "in Scorpio"),
    column("SAGITTARIUS", "in Sagittarius"),
    column("CAPRICORN", "in Capricorn"),
    column("AQUARIUS", "in Aquarius"),
    column("PISCES", "in Pisces"),
    column("total", "the row's sum"),
];

struct AshtakavargaSection;

impl DossierSection for AshtakavargaSection {
    fn key(&self) -> &'static str {
        "ASHTAKAVARGA"
    }
    fn columns(&self) -> &[Column] {
        &SIGN_COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        flag!("ashtakavarga")
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        let reading = document.ashtakavarga.as_ref()?;
        let row = |of: &str, bindus: Vec<u32>| {
            let mut row = vec![of.to_owned()];
            row.extend(bindus.iter().map(ToString::to_string));
            row.push(bindus.iter().sum::<u32>().to_string());
            row
        };
        let mut rows = vec![row(
            "SARVA",
            reading.sarva.iter().map(|one| u32::from(*one)).collect(),
        )];
        rows.extend(reading.grahas.iter().map(|one| {
            row(
                one.graha.key(),
                one.bindus.iter().map(|b| u32::from(*b)).collect(),
            )
        }));
        Some(rows)
    }
}

struct ShadbalaSection;

impl DossierSection for ShadbalaSection {
    fn key(&self) -> &'static str {
        "SHADBALA"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 12] = [
            column("graha", "the seven"),
            column("sthana", "positional strength, virupas (60 a rupa)"),
            column("dig", "directional, virupas"),
            column("kaala", "temporal, virupas"),
            column("cheshta", "motional, virupas"),
            column("naisargika", "natural, virupas"),
            column("drik", "aspectual, virupas, negative when malefics look"),
            column("rupas", "the six together, rupas"),
            column("required", "the rupas the graha needs to be strong"),
            column("strong", "YES when rupas reach required"),
            column("ishta", "the good it can give, 0 to 60"),
            column("kashta", "the harm, 0 to 60"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        flag!("shadbala")
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        let reading = document.shadbala.as_ref()?;
        Some(
            reading
                .grahas
                .iter()
                .map(|one| {
                    vec![
                        one.graha.key().to_owned(),
                        fixed(one.sthana.total(), 2),
                        fixed(one.dig, 2),
                        fixed(one.kaala.total(), 2),
                        fixed(one.cheshta, 2),
                        fixed(one.naisargika, 2),
                        fixed(one.drik, 2),
                        fixed(one.rupas, 2),
                        fixed(one.required_rupas, 2),
                        String::from(if one.strong { "YES" } else { "NO" }),
                        fixed(one.ishta, 2),
                        fixed(one.kashta, 2),
                    ]
                })
                .collect(),
        )
    }
}

struct BhavaBalaSection;

impl DossierSection for BhavaBalaSection {
    fn key(&self) -> &'static str {
        "BHAVA_BALA"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 7] = [
            column("house", "1 to 12"),
            column("lord", "its lord"),
            column("adhipati", "the lord's own strength, virupas"),
            column("dig", "directional, virupas"),
            column("drishti", "what looks at it, virupas"),
            column("special", "the occupants' and the day's additions, virupas"),
            column("virupas", "the house's strength, the four together"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        flag!("bhavaBala")
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        let reading = document.bhava_bala.as_ref()?;
        Some(
            reading
                .bhavas
                .iter()
                .map(|one| {
                    vec![
                        one.bhava.to_string(),
                        one.lord.key().to_owned(),
                        fixed(one.adhipati, 2),
                        fixed(one.dig, 2),
                        fixed(one.drishti, 2),
                        fixed(one.special, 2),
                        fixed(one.virupas, 2),
                    ]
                })
                .collect(),
        )
    }
}

struct VimshopakaSection;

impl DossierSection for VimshopakaSection {
    fn key(&self) -> &'static str {
        "VIMSHOPAKA"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 5] = [
            column("graha", "the graha"),
            column("shadvarga", "its dignity over the six vargas, 0 to 20"),
            column("saptavarga", "over the seven, 0 to 20"),
            column("dashavarga", "over the ten, 0 to 20"),
            column("shodashavarga", "over the sixteen, 0 to 20"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        flag!("vimshopaka")
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        let reading = document.vimshopaka.as_ref()?;
        Some(
            reading
                .grahas
                .iter()
                .map(|one| {
                    vec![
                        one.graha.key().to_owned(),
                        fixed(one.shadvarga, 2),
                        fixed(one.saptavarga, 2),
                        fixed(one.dashavarga, 2),
                        fixed(one.shodashavarga, 2),
                    ]
                })
                .collect(),
        )
    }
}

struct JaiminiSection;

impl DossierSection for JaiminiSection {
    fn key(&self) -> &'static str {
        "JAIMINI"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 2] = [
            column(
                "field",
                "atmakaraka, karakamsha, or a graha's arudha (arudha.SUN)",
            ),
            column("value", "the graha or the sign"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        flag!("jaimini")
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        let reading = document.jaimini.as_ref()?;
        let mut rows = vec![
            vec![
                String::from("atmakaraka"),
                reading.karakamsha.atmakaraka.key().to_owned(),
            ],
            vec![
                String::from("karakamsha"),
                reading.karakamsha.sign.key().to_owned(),
            ],
        ];
        rows.extend(
            NINE.iter()
                .zip(&reading.graha_arudhas)
                .map(|(graha, sign)| {
                    vec![
                        format!("arudha.{}", graha.key()),
                        sign.map_or_else(dash, |one| one.key().to_owned()),
                    ]
                }),
        );
        Some(rows)
    }
}

struct PointsSection;

impl DossierSection for PointsSection {
    fn key(&self) -> &'static str {
        "POINTS"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 5] = [
            column("point", "an upagraha or a special lagna"),
            column("sign", "its sign"),
            column("deg", "degrees in the sign, two decimals"),
            column("nakshatra", "the star of the 27"),
            column("pada", "its quarter, 1 to 4"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        flag!("points")
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        let points = document.points.as_ref()?;
        Some(
            points
                .all()
                .iter()
                .map(|one| {
                    let [sign, deg, star, pada] = place(one.longitude_deg);
                    vec![one.point.key().to_owned(), sign, deg, star, pada]
                })
                .collect(),
        )
    }
}

struct AspectsSection;

impl DossierSection for AspectsSection {
    fn key(&self) -> &'static str {
        "ASPECTS"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 4] = [
            column("from", "the graha that looks"),
            column("to", "the graha looked at"),
            column(
                "houses",
                "the house it stands in, counted from the one that looks",
            ),
            column("strength", "QUARTER, HALF, THREE_QUARTERS or FULL"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        flag!("aspects")
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        let aspects = document.aspects.as_ref()?;
        Some(
            aspects
                .all()
                .iter()
                .map(|one| {
                    vec![
                        one.from.key().to_owned(),
                        one.to.key().to_owned(),
                        one.houses.to_string(),
                        word(&one.strength),
                    ]
                })
                .collect(),
        )
    }
}

struct AvakahadaSection;

impl DossierSection for AvakahadaSection {
    fn key(&self) -> &'static str {
        "AVAKAHADA"
    }
    fn columns(&self) -> &[Column] {
        const COLUMNS: [Column; 2] = [
            column(
                "field",
                "nakshatra, pada, rashi, their lords, varna, yoni, gana, nadi: the Moon's",
            ),
            column("value", "its value"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        flag!("avakahada")
    }
    fn rows(&self, document: &Document, _: &View) -> Option<Vec<Vec<String>>> {
        let a = document.avakahada.as_ref()?;
        let row = |field: &str, value: String| vec![field.to_owned(), value];
        Some(vec![
            row("nakshatra", a.nakshatra.key().to_owned()),
            row("pada", a.pada.to_string()),
            row("rashi", a.rashi.key().to_owned()),
            row("nakshatra_lord", a.nakshatra_lord.key().to_owned()),
            row("rashi_lord", a.rashi_lord.key().to_owned()),
            row("varna", word(&a.varna)),
            row("yoni", word(&a.yoni)),
            row("gana", word(&a.gana)),
            row("nadi", word(&a.nadi)),
        ])
    }
}

/// The built-in sections, in the dossier's order.
static BUILT_IN: [&dyn DossierSection; 15] = [
    &ChartSection,
    &NowSection,
    &GrahasSection,
    &HousesSection,
    &VargasSection,
    &DashasSection,
    &PanchangaSection,
    &AshtakavargaSection,
    &ShadbalaSection,
    &BhavaBalaSection,
    &VimshopakaSection,
    &JaiminiSection,
    &PointsSection,
    &AspectsSection,
    &AvakahadaSection,
];

/// Their keys, in the same order.
const BUILT_IN_KEYS: [&str; 15] = [
    "CHART",
    "NOW",
    "GRAHAS",
    "HOUSES",
    "VARGAS",
    "DASHAS",
    "PANCHANGA",
    "ASHTAKAVARGA",
    "SHADBALA",
    "BHAVA_BALA",
    "VIMSHOPAKA",
    "JAIMINI",
    "POINTS",
    "ASPECTS",
    "AVAKAHADA",
];

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn the_keys_are_the_sections_in_order() {
        let keys: Vec<&str> = BUILT_IN.iter().map(|one| one.key()).collect();
        assert_eq!(keys, BUILT_IN_KEYS);
    }

    #[test]
    fn every_column_has_a_meaning_and_a_clean_name() {
        for section in BUILT_IN {
            for one in section.columns() {
                assert!(!one.meaning.is_empty(), "{}.{}", section.key(), one.name);
                assert!(
                    !one.name.contains(['|', ' ']),
                    "{}.{}",
                    section.key(),
                    one.name
                );
            }
        }
    }

    #[test]
    fn a_number_and_an_instant_are_written_the_same_everywhere() {
        assert_eq!(fixed(-0.0001, 2), "0.00");
        assert_eq!(fixed(-1.234, 2), "-1.23");
        // J2000.0 is noon UTC on 2000-01-01; at +05:45, 17:45.
        let j2000 = JulianDay::<Utc>::literal(2_451_545.0);
        assert_eq!(civil(j2000, 0), "2000-01-01 12:00");
        assert_eq!(civil(j2000, 20_700), "2000-01-01 17:45");
        assert_eq!(civil(j2000, -46_800), "1999-12-31 23:00");
        assert_eq!(clock(20_700), "+05:45");
        assert_eq!(clock(-12_600), "-03:30");
    }

    #[test]
    fn a_longitude_is_its_sign_star_and_pada() {
        assert_eq!(
            place(0.0),
            ["ARIES", "0.00", "ASHWINI", "1"].map(String::from)
        );
        // 13°20′ opens Bharani; 3°20′ before it closes Ashwini's fourth pada.
        assert_eq!(place(13.0 + 1.0 / 3.0 + 1e-9)[2], "BHARANI");
        assert_eq!(place(13.0)[3], "4");
        assert_eq!(place(359.99)[2], "REVATI");
        assert_eq!(place(-0.5)[0], "PISCES");
    }

    #[test]
    fn a_field_never_breaks_a_row() {
        assert_eq!(clean("a|b\nc"), "a b c");
        assert_eq!(clean(""), "-");
    }
}

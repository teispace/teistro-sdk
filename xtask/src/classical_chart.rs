//! The measurement pass over a chart **founded on a classical
//! astronomy**: which parts of it are the text's.
//!
//! The Surya Siddhanta provider passes the port's kit and declares three
//! overrides — its obliquity, its ayanamsha and its sunrise — beside its
//! positions (`03-design/siddhanta.md` §5). The crate also computes the
//! text's own Lagna (III.46 to 49) and its own day arc. A chart founded
//! over the provider has four parts that could be the text's: the
//! grahas, the zodiac they are measured in, the Lagna, and the day the
//! hours and the ishtakaal are counted through. This pass founds every
//! recorded birth **through the SDK** over the provider, as a consumer
//! would, and holds each part against the text's own answer at the same
//! instant and place. It measures the built thing, so it turns over when
//! the chart layer changes, and its claims are written to fail both ways.
//!
//! Nothing here is gated against a recording: the corpus records a
//! modern chart, not a classical one. The text's answers come from
//! `crates/siddhanta`, which reproduces Burgess's worked 1860 computation
//! (`crates/siddhanta/tests`).
//!
//! `cargo xtask classical-chart` writes the page; `check-classical-chart`
//! regenerates it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};
use teistro_astro::DeltaTModel;
use teistro_astro::scale::tt_of;
use teistro_astro::sky;
use teistro_calendar::gregorian::Gregorian;
use teistro_chart::day::chart_day;
use teistro_siddhanta::SuryaSiddhanta;
use teistro_time::hora::hora_at;

use crate::births::CHARTS;
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, capitalised, fill, spelled, table, verdict_of};
use crate::rules_corpus::read_json;

const PAGE: &str = "docs/03-design/classical-chart-measured.md";

/// The settings a consumer asking for the text's chart would name: the
/// text's own ayanamsha by its catalogue member. The root profile's
/// sunrise is already the text's (the centre on the geometric horizon).
const SETTINGS: &str = r#"{"frame":{"ayanamsha":{"kind":"CATALOGUED","id":"SURYASIDDHANTA"}}}"#;

/// Seconds in a day.
const SECONDS_PER_DAY: f64 = 86_400.0;

/// One recorded birth, founded over the text, beside the text's answers.
struct Reading {
    name: String,
    /// The chart the SDK founded, or its refusal.
    chart: Result<Document, String>,
    /// The text's Lagna, or why it has none (a polar day).
    text: Option<TextChart>,
}

/// What the text itself answers at the birth.
struct TextChart {
    lagna_sidereal_deg: f64,
    /// The text's point on the meridian (III.49), sidereal.
    meridian_deg: f64,
    ayanamsha_deg: f64,
    /// The nine grahas the text places, sidereal in its own zodiac.
    grahas: Vec<(Graha, f64)>,
    /// The day the text's own sunrise opens, and the hour in it.
    sunrise: f64,
    hora: u8,
    /// The ascendant by spherical astronomy on the text's obliquity, in
    /// the text's zodiac: what the chart layer would give if only the
    /// obliquity were the text's.
    spherical_on_text_obliquity_deg: f64,
}

pub(crate) fn generate(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => i32::from(
            check(
                root,
                &[Output::new(PAGE, text)],
                "cargo xtask classical-chart",
            ) != 0,
        ),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

// ── the measurement ────────────────────────────────────────────────────────

/// A context over the text, as a consumer opens one.
fn over_the_text() -> Result<Context, String> {
    Context::builder()
        .settings_json(SETTINGS)
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .map_err(|why| format!("a context over the text: {why}"))
}

/// Every recorded birth, founded over the text.
fn readings(root: &Path, sdk: &Context) -> Result<Vec<Reading>, String> {
    let directory = root.join(CHARTS);
    let mut paths: Vec<_> = std::fs::read_dir(&directory)
        .map_err(|why| format!("{CHARTS}: {why}"))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    let model = SuryaSiddhanta::text();
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let file = read_json(&path)?;
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let number = |value: &serde_json::Value, what: &str| {
            value.as_f64().ok_or_else(|| format!("{name}: no {what}"))
        };
        let at = &file["input"]["place"];
        let place = Place::new(
            Latitude::try_new(number(&at["latitude"], "latitude")?)
                .map_err(|why| format!("{name}: {why}"))?,
            Longitude::try_new(number(&at["longitude"], "longitude")?)
                .map_err(|why| format!("{name}: {why}"))?,
            Altitude::try_new(at["altitude_m"].as_f64().unwrap_or(0.0))
                .map_err(|why| format!("{name}: {why}"))?,
        );
        let jd = number(&file["input"]["resolved"]["jd_ut"], "resolved instant")?;
        let minutes = file["input"]["resolved"]["tz_offset_min"]
            .as_i64()
            .unwrap_or(0);
        let offset = UtcOffset::try_from_seconds(
            i32::try_from(minutes * 60).map_err(|why| format!("{name}: {why}"))?,
        )
        .map_err(|why| format!("{name}: {why}"))?;
        let instant = JulianDay::<Utc>::literal(jd);
        let chart = sdk
            .chart()
            .reading(instant, &ChartRequest::at(place, offset))
            .map(|envelope| envelope.value)
            .map_err(|why| why.to_string());
        let text = text_chart(&model, sdk, &place, offset, instant)?;
        out.push(Reading { name, chart, text });
    }
    Ok(out)
}

/// The text's own answers at an instant and a place, or `None` where
/// the text has no sunrise to count from.
fn text_chart(
    model: &SuryaSiddhanta,
    sdk: &Context,
    place: &Place,
    offset: UtcOffset,
    instant: JulianDay<Utc>,
) -> Result<Option<TextChart>, String> {
    let ut1 = JulianDay::<Ut1>::literal(instant.get());
    let Ok(lagna) = model.lagna(ut1, place.latitude, place.longitude) else {
        return Ok(None);
    };
    let day = &sdk.settings().day;
    let text_day = chart_day(
        model,
        &Gregorian,
        &offset,
        place,
        instant,
        day.polar_day_policy,
    )
    .map_err(|why| format!("the text's day: {why}"))?;
    let hora = hora_at(
        &text_day.day,
        instant,
        day.hora_reckoning
            .try_into()
            .map_err(|why| format!("the hora reckoning: {why}"))?,
    )
    .map_err(|why| format!("the text's hora: {why}"))?;
    let (tt, _) = tt_of(ut1, DeltaTModel::TableThenModel).map_err(|why| why.to_string())?;
    let ayanamsha_deg = model.ayanamsha_deg(ut1);
    let ramc = sky::sidereal_time_deg(ut1, tt, place.longitude);
    let text_obliquity = f64::from(model.parameters().obliquity_sine);
    let obliquity = model.trig().arc(text_obliquity);
    Ok(Some(TextChart {
        lagna_sidereal_deg: lagna.sidereal_deg,
        meridian_deg: lagna.meridian_sidereal_deg,
        ayanamsha_deg,
        grahas: model
            .all(ut1)
            .iter()
            .map(|(graha, at)| (*graha, at.longitude.get()))
            .collect(),
        sunrise: text_day.day.sunrise.get(),
        hora: hora.number,
        spherical_on_text_obliquity_deg: (ascendant_deg(ramc, place.latitude.get(), obliquity)
            - ayanamsha_deg)
            .rem_euclid(360.0),
    }))
}

/// The ascendant by spherical astronomy: the ecliptic's intersection
/// with the eastern horizon for a sidereal time, a latitude and an
/// obliquity, tropical degrees.
fn ascendant_deg(ramc_deg: f64, latitude_deg: f64, obliquity_deg: f64) -> f64 {
    let (ramc, latitude, obliquity) = (
        ramc_deg.to_radians(),
        latitude_deg.to_radians(),
        obliquity_deg.to_radians(),
    );
    let x = -(ramc.sin() * obliquity.cos() + latitude.tan() * obliquity.sin());
    ramc.cos().atan2(x).to_degrees().rem_euclid(360.0)
}

/// The signed difference of two longitudes, degrees in `(-180, 180]`.
fn apart(a: f64, b: f64) -> f64 {
    (a - b + 540.0).rem_euclid(360.0) - 180.0
}

/// The sign of a longitude, 0 for Aries.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a normalised longitude over thirty is 0 to 11"
)]
fn sign_of(longitude: f64) -> u8 {
    ((longitude.rem_euclid(360.0) / 30.0) as u8).min(11)
}

/// Births by name, or `none`.
fn named(names: &[&str]) -> String {
    if names.is_empty() {
        return String::from("none");
    }
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(" and ")
}

/// The median and the worst of a set of magnitudes.
fn spread(mut values: Vec<f64>) -> (f64, f64) {
    values.sort_by(f64::total_cmp);
    let median = values.get(values.len() / 2).copied().unwrap_or(f64::NAN);
    let worst = values.last().copied().unwrap_or(f64::NAN);
    (median, worst)
}

/// How close a part must stand to the text's to be the text's: the
/// same computation carried through the same arithmetic, so a rounding
/// and not a model.
const SAME_DEG: f64 = 1e-9;
/// The same, for an instant, seconds.
const SAME_SECONDS: f64 = 1e-3;

/// One part of a chart, measured against the text's.
struct Part {
    /// What it is, in a sentence and a claim.
    short: &'static str,
    /// What it is, in the table.
    name: &'static str,
    /// Where it came from while it is not the text's, which the page
    /// says of it; a part that is the text's needs no explanation.
    otherwise: &'static str,
    unit: &'static str,
    median: f64,
    worst: f64,
    /// Whether it is the text's, to within a rounding.
    is_text: bool,
    /// What the difference changes, where it changes a reading.
    note: String,
}

impl Part {
    fn of(
        (short, name): (&'static str, &'static str),
        otherwise: &'static str,
        unit: &'static str,
        gaps: &[f64],
        tolerance: f64,
    ) -> Part {
        let (median, worst) = spread(gaps.to_vec());
        Part {
            short,
            name,
            otherwise,
            unit,
            median,
            worst,
            is_text: worst <= tolerance,
            note: String::new(),
        }
    }

    /// The verb a sentence about it takes: the five parts' names are
    /// plural exactly when they end in an `s`.
    fn verb(&self) -> &'static str {
        if self.short.ends_with('s') {
            "are"
        } else {
            "is"
        }
    }

    fn noting(mut self, note: String) -> Part {
        self.note = note;
        self
    }
}

/// Every graha's distance from the text's, per chart: the worst of the
/// nine, and which graha it was.
fn graha_gaps(compared: &[(&Document, &TextChart)]) -> Result<(Vec<f64>, Graha), String> {
    let mut gaps = Vec::with_capacity(compared.len());
    let mut worst = (0.0_f64, Graha::Sun);
    for (chart, text) in compared {
        let mut chart_worst = 0.0_f64;
        for (graha, text_deg) in &text.grahas {
            let founded = chart
                .foundation
                .graha(*graha)
                .ok_or_else(|| format!("a founded chart without {}", graha.key()))?;
            let gap = apart(founded.longitude_deg, *text_deg).abs();
            chart_worst = chart_worst.max(gap);
            if gap > worst.0 {
                worst = (gap, *graha);
            }
        }
        gaps.push(chart_worst);
    }
    Ok((gaps, worst.1))
}

#[expect(
    clippy::too_many_lines,
    reason = "a page reads top to bottom, and splitting it would scatter its prose"
)]
fn page(root: &Path) -> Result<String, String> {
    let sdk = over_the_text()?;
    let readings = readings(root, &sdk)?;
    let compared: Vec<(&Document, &TextChart)> = readings
        .iter()
        .filter_map(|reading| Some((reading.chart.as_ref().ok()?, reading.text.as_ref()?)))
        .collect();
    let text_refused: Vec<&str> = readings
        .iter()
        .filter(|reading| reading.text.is_none())
        .map(|reading| reading.name.as_str())
        .collect();
    let sdk_refused: Vec<&str> = readings
        .iter()
        .filter(|reading| reading.chart.is_err())
        .map(|reading| reading.name.as_str())
        .collect();
    let steps = compared
        .first()
        .map(|(chart, _)| chart.foundation.steps.join(", "))
        .unwrap_or_default();
    let of = |count: usize| format!("{count} of {}", compared.len());

    let zodiac: Vec<f64> = compared
        .iter()
        .map(|(chart, text)| (chart.foundation.zodiac.offset_deg - text.ayanamsha_deg).abs())
        .collect();
    let (grahas, worst_graha) = graha_gaps(&compared)?;
    let lagna: Vec<f64> = compared
        .iter()
        .map(|(chart, text)| apart(chart.foundation.lagna_deg, text.lagna_sidereal_deg).abs())
        .collect();
    let lagna_signs = compared
        .iter()
        .filter(|(chart, text)| {
            sign_of(chart.foundation.lagna_deg) != sign_of(text.lagna_sidereal_deg)
        })
        .count();
    let midheaven: Vec<f64> = compared
        .iter()
        .map(|(chart, text)| {
            sdk.chart()
                .angles(chart)
                .map(|angles| apart(angles.midheaven_deg, text.meridian_deg).abs())
                .map_err(|why| format!("a founded chart's angles: {why}"))
        })
        .collect::<Result<_, _>>()?;
    let obliquity_only: Vec<f64> = compared
        .iter()
        .map(|(_, text)| {
            apart(
                text.spherical_on_text_obliquity_deg,
                text.lagna_sidereal_deg,
            )
            .abs()
        })
        .collect();
    let obliquity_signs = compared
        .iter()
        .filter(|(_, text)| {
            sign_of(text.spherical_on_text_obliquity_deg) != sign_of(text.lagna_sidereal_deg)
        })
        .count();
    let sunrise: Vec<f64> = compared
        .iter()
        .map(|(chart, text)| {
            (chart.foundation.day.day.sunrise.get() - text.sunrise).abs() * SECONDS_PER_DAY
        })
        .collect();
    let horas = compared
        .iter()
        .filter(|(chart, text)| chart.foundation.timing.hora.number != text.hora)
        .count();

    let parts = [
        Part::of(
            (
                "the zodiac",
                "the zodiac: the chart's ayanamsha against the text's",
            ),
            "it is the catalogue's `SURYASIDDHANTA` member, zero in the year 499 and carried forward by \
             modern precession, where the provider answers the text's own (III.9 to 12)",
            "°",
            &zodiac,
            SAME_DEG,
        ),
        Part::of(
            ("the grahas", "the grahas, all nine, in the chart's zodiac"),
            "they are the text's places moved by the SDK's apparent-place corrections or carried \
             into another zodiac",
            "°",
            &grahas,
            SAME_DEG,
        )
        .noting(if grahas.iter().all(|gap| *gap <= SAME_DEG) {
            String::new()
        } else {
            format!("worst {}", worst_graha.key())
        }),
        Part::of(
            ("the Lagna", "the Lagna, in the chart's zodiac"),
            "it is the SDK's spherical ascendant, on the IAU obliquity and modern sidereal time",
            "°",
            &lagna,
            SAME_DEG,
        )
        .noting(format!("{} in another sign", of(lagna_signs))),
        Part::of(
            (
                "the midheaven",
                "the midheaven, as `sdk.chart().angles` answers it",
            ),
            "it is the SDK's spherical midheaven, recomputed from the chart's instant and place",
            "°",
            &midheaven,
            SAME_DEG,
        ),
        Part::of(
            ("the sunrise", "the day's sunrise"),
            "it is the SDK's rise and set solver run over the text's Sun, not the text's day arc, \
             although the provider declares its sunrise as an override",
            " s",
            &sunrise,
            SAME_SECONDS,
        )
        .noting(format!("{} in another hora", of(horas))),
    ];

    let mut out = String::new();
    let _ = writeln!(
        out,
        "# A chart founded on a classical astronomy, measured\n\n\
         Status: `generated` by `cargo xtask classical-chart` over the\n\
         Surya Siddhanta provider. Do not edit: `check-classical-chart`\n\
         regenerates this page and fails on any difference.\n\n\
         The question `siddhanta.md` §5 leaves to the chart layer, and\n\
         `classical-chart.md` answers: when a chart is founded over the\n\
         text's provider, which of its parts are the text's?\n"
    );
    let _ = writeln!(
        out,
        "## 1. What was founded\n\n\
         Every recorded birth of the corpus ({} of them) was founded\n\
         **through the SDK** over `Ephemeris::SuryaSiddhanta`, as a Rust\n\
         consumer opens it, under the root profile with the text's\n\
         own ayanamsha named (`SURYASIDDHANTA`); the root profile's\n\
         sunrise is already the text's, the centre on the geometric\n\
         horizon. Each chart was then held against what `crates/siddhanta`\n\
         answers at the same instant and place: its Lagna (III.46 to 49),\n\
         its ayanamsha, its nine grahas and its own day arc, through which\n\
         the same hora reckoning was counted. The text refuses {}, because\n\
         on the day it has no sunrise to count the Lagna from; the SDK\n\
         refuses {} under the root profile's polar-day policy, which\n\
         synthesises no day.\n\n\
         The steps the provenance stamps on each chart: `{steps}`.\n",
        spelled(readings.len()),
        named(&text_refused),
        named(&sdk_refused),
    );
    let mut rows =
        String::from("| part | the text's | median | worst | note |\n|---|---|---:|---:|---|\n");
    for part in &parts {
        let _ = writeln!(
            rows,
            "| {} | {} | {:.3}{} | {:.3}{} | {} |",
            part.name,
            if part.is_text { "yes" } else { "**no**" },
            part.median,
            part.unit,
            part.worst,
            part.unit,
            part.note
        );
    }
    let _ = writeln!(
        out,
        "## 2. Measured: which parts are the text's\n\n\
         Over the {} births both answer, each part of the chart the SDK\n\
         founded against the text's own, to within a rounding ({SAME_DEG:e}°\n\
         or {SAME_SECONDS} s):\n\n{rows}",
        spelled(compared.len())
    );
    for part in parts.iter().filter(|part| !part.is_text) {
        let _ = writeln!(
            out,
            "**{}** {} not the text's: {}.\n",
            capitalised(part.short),
            part.verb(),
            part.otherwise
        );
    }
    let (obliquity_median, obliquity_worst) = spread(obliquity_only);
    let _ = writeln!(
        out,
        "**{}** The spherical ascendant on the text's own 24° stands a\n\
         median {obliquity_median:.3}° and at worst {obliquity_worst:.3}°\n\
         from the text's Lagna, in another sign on {} births. The text\n\
         reckons the Lagna from the Sun at its own sunrise, carried through\n\
         the signs' rising times in proportion within each sign (Burgess's\n\
         note under III.46 to 49 calls this the text's own approximation),\n\
         on a clock with no equation of time, so only the text reproduces\n\
         it.\n",
        if obliquity_signs > 0 {
            "The obliquity is not the difference."
        } else {
            "The obliquity alone keeps the Lagna's sign."
        },
        of(obliquity_signs)
    );

    let mut claims = Vec::new();
    for part in &parts {
        claims.push(Claim::stated(
            format!("{} {} the text's", part.short, part.verb()),
            verdict_of(part.is_text),
            format!("worst {:.3}{}", part.worst, part.unit),
        ));
    }
    claims.push(
        Claim::counted(
            "the Lagna is in the text's sign",
            lagna_signs,
            compared.len(),
        )
        .with_note(format!("worst {:.3}°", spread(lagna).1)),
    );
    claims.push(Claim::counted(
        "the hora is the text's",
        horas,
        compared.len(),
    ));
    claims.push(
        Claim::counted(
            "the text's obliquity alone would give the text's Lagna, in sign",
            obliquity_signs,
            compared.len(),
        )
        .with_note(format!("worst {obliquity_worst:.3}°")),
    );
    claims.push(Claim::counted(
        "the SDK founds every recorded birth over the text",
        sdk_refused.len(),
        readings.len(),
    ));
    claims.push(Claim::counted(
        "the text answers a Lagna for every recorded birth",
        text_refused.len(),
        readings.len(),
    ));
    let _ = writeln!(out, "## 3. What this pass decides\n");
    let _ = writeln!(out, "{}", table(&claims));
    let falsified = claims
        .iter()
        .filter(|claim| claim.verdict == Verdict::Falsified)
        .count();
    let listed = |is_text: bool| {
        let names: Vec<&str> = parts
            .iter()
            .filter(|part| part.is_text == is_text)
            .map(|part| part.short)
            .collect();
        if names.is_empty() {
            String::from("none")
        } else {
            names.join("; ")
        }
    };
    let _ = writeln!(
        out,
        "{} of the {} claims {} falsified. The parts the text gives the\n\
         chart: {}. The parts the chart still computes itself: {}.\n",
        capitalised(&spelled(falsified)),
        spelled(claims.len()),
        if falsified == 1 { "is" } else { "are" },
        listed(true),
        listed(false),
    );
    Ok(fill(&out))
}

//! The measurement pass over **eclipses** (`03-design/eclipses.md`,
//! `03-design/eclipses-measured.md`).
//!
//! Every solar and lunar eclipse of 1900 to 2100 in NASA's Five Millennium
//! Canon (Espenak and Meeus, public domain) is held against
//! `astro::eclipse` over the built-in ephemeris, both ways: every
//! catalogued eclipse is found and nothing else is. Each answer is then
//! held to the catalogue's kind, greatest moment, gamma, magnitudes, a
//! lunar eclipse's phase durations and a solar eclipse's point of greatest
//! eclipse, and the two placings the design rejected are measured beside
//! the one it ships.
//!
//! `cargo xtask eclipses` writes the page; `check-eclipses` regenerates it
//! in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro_astro::eclipse::{
    Eclipses, LunarEclipse, LunarEclipseKind, Seen, ShadowRule, ShadowSource, SolarEclipse,
    SolarEclipseKind,
};
use teistro_astro::sky::Apparent;
use teistro_astro::{Completion, DeltaTModel, tt_of};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_port_ephemeris::{Body, Horizon};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, median, table, verdict_of};

const PAGE: &str = "docs/03-design/eclipses-measured.md";

/// The lunar catalogue, 1900 to 2100, an eclipse a line: the date and
/// the TD of greatest eclipse, the kind (`N` penumbral, `P` partial, `T`
/// total), gamma, the penumbral and umbral magnitudes, and the penumbral,
/// partial and total phases' durations in minutes, `-` for a phase the
/// eclipse lacks. Read from NASA's `5MKLEcatalog.txt` on 1 October 2026.
const LUNAR: &str = include_str!("eclipses/lunar.txt");

/// The solar catalogue the same way: the date and the TD of greatest
/// eclipse, the kind (`P` partial, `A` annular, `T` total, `H` hybrid),
/// gamma, the magnitude, and the latitude and longitude of greatest
/// eclipse. Read from NASA's `5MCSEcatalog.txt` on 1 October 2026.
const SOLAR: &str = include_str!("eclipses/solar.txt");

/// Local circumstances, a city a line: the date of the eclipse, the
/// city (spaces as underscores), its latitude and longitude in degrees
/// east and north, its elevation in metres (0 where the bulletin prints
/// none), the four contacts' and the maximum's UT, `-` for a contact the
/// bulletin does not print, and the magnitude at the maximum. Transcribed
/// on 1 October 2026 from NASA's eclipse bulletins, in the public
/// domain (`eclipse.gsfc.nasa.gov/SEmono/`): the total solar eclipse of
/// 2009 July 22 (`TSE2009`, tables 9, 10 and 12: India, China and Asia)
/// and the annular solar eclipse of 2010 January 15 (`ASE2010`, tables
/// 2.9 and 2.13: Europe and South Asia). Cities whose maximum or a
/// contact falls at sunrise or sunset, which the bulletin prints to the
/// minute as `Rise` or `Set`, are left out.
const LOCAL: &str = include_str!("eclipses/local.txt");

/// The window both catalogues are read over, as Gregorian years.
const FIRST_YEAR: i32 = 1900;
const LAST_YEAR: i32 = 2100;

/// How far apart an answer and a record may be and still be one eclipse,
/// days: eclipses of a kind are a fortnight apart at the least.
const SAME_ECLIPSE_DAYS: f64 = 0.1;

/// The bounds the page holds each quantity to.
const GREATEST_BOUND_SECONDS: f64 = 5.0;
const GAMMA_BOUND: f64 = 0.0005;
const MAGNITUDE_BOUND: f64 = 0.001;
const DURATION_BOUND_MINUTES: f64 = 1.0;
const POINT_BOUND_DEG: f64 = 0.5;
const LOCAL_BOUND_SECONDS: f64 = 5.0;
/// The maximum's own bound: a shallow eclipse's magnitude is flat for
/// seconds either side of it, where a contact is a sharp crossing.
const LOCAL_MAXIMUM_BOUND_SECONDS: f64 = 10.0;
const LOCAL_MAGNITUDE_BOUND: f64 = 0.002;

/// The Delta T model the search runs under. The catalogue speaks TT and
/// the search UT1; an answer is taken back to TT under the same model,
/// so the model cancels.
const DELTA_T: DeltaTModel = DeltaTModel::TableThenModel;

struct LunarRecord {
    tt: f64,
    date: String,
    kind: LunarEclipseKind,
    gamma: f64,
    penumbral: f64,
    umbral: f64,
    /// The total, partial and penumbral phases, minutes.
    durations: [Option<f64>; 3],
}

struct SolarRecord {
    tt: f64,
    date: String,
    kind: SolarEclipseKind,
    gamma: f64,
    magnitude: f64,
    latitude: f64,
    longitude: f64,
}

struct LocalRecord {
    date: String,
    city: String,
    place: Place,
    /// The four contacts and the maximum, seconds past the date's UT
    /// midnight, as printed: a contact before midnight is a day early.
    moments: [Option<f64>; 5],
    magnitude: f64,
}

fn local_records() -> Result<Vec<LocalRecord>, String> {
    LOCAL
        .lines()
        .map(|line| {
            let f: Vec<&str> = line.split_whitespace().collect();
            let [date, city, lat, lon, elev, c1, c2, c3, c4, max, magnitude] = f.as_slice() else {
                return Err(format!("'{line}' is no local record"));
            };
            let place = Place::try_from_degrees(number(lat)?, number(lon)?, number(elev)?)
                .map_err(|e| format!("{city}: {e}"))?;
            let clock = |text: &str| -> Result<Option<f64>, String> {
                if text == "-" {
                    return Ok(None);
                }
                let parts: Vec<f64> = text.split(':').map(number).collect::<Result<_, _>>()?;
                let [h, m, s] = parts.as_slice() else {
                    return Err(format!("{city}: '{text}' is no time"));
                };
                Ok(Some(h * 3600.0 + m * 60.0 + s))
            };
            Ok(LocalRecord {
                date: (*date).to_string(),
                city: city.replace('_', " "),
                place,
                moments: [clock(c1)?, clock(c2)?, clock(c3)?, clock(c4)?, clock(max)?],
                magnitude: number(magnitude)?,
            })
        })
        .collect()
}

fn number(text: &str) -> Result<f64, String> {
    text.parse::<f64>()
        .map_err(|e| format!("'{text}' is no number: {e}"))
}

/// The TT Julian day of a record's date and time.
fn instant(date: &str, time: &str) -> Result<f64, String> {
    let parts: Vec<i32> = date.split('-').filter_map(|p| p.parse().ok()).collect();
    let [year, month, day] = parts.as_slice() else {
        return Err(format!("'{date}' is no date"));
    };
    let midnight = teistro_calendar::gregorian::fixed_from_gregorian(
        *year,
        u8::try_from(*month).map_err(|e| e.to_string())?,
        u8::try_from(*day).map_err(|e| e.to_string())?,
    )
    .jd_at_midnight()
    .map_err(|e| e.to_string())?
    .get();
    let clock: Vec<f64> = time.split(':').map(number).collect::<Result<_, _>>()?;
    let [hours, minutes, seconds] = clock.as_slice() else {
        return Err(format!("'{time}' is no time"));
    };
    Ok(midnight + (hours * 3600.0 + minutes * 60.0 + seconds) / 86_400.0)
}

/// A signed angle written `12.3N` or `45.6W`.
fn hemisphere(text: &str) -> Result<f64, String> {
    let (value, side) = text.split_at(text.len().saturating_sub(1));
    let value = number(value)?;
    match side {
        "N" | "E" => Ok(value),
        "S" | "W" => Ok(-value),
        _ => Err(format!("'{text}' names no hemisphere")),
    }
}

fn lunar_records() -> Result<Vec<LunarRecord>, String> {
    LUNAR
        .lines()
        .map(|line| {
            let f: Vec<&str> = line.split_whitespace().collect();
            let [date, time, kind, gamma, penumbral, umbral, pen, par, tot] = f.as_slice() else {
                return Err(format!("'{line}' is no lunar record"));
            };
            let kind = match *kind {
                "N" => LunarEclipseKind::Penumbral,
                "P" => LunarEclipseKind::Partial,
                "T" => LunarEclipseKind::Total,
                other => return Err(format!("{date}: '{other}' is no lunar kind")),
            };
            let minutes = |text: &str| (text != "-").then(|| number(text)).transpose();
            Ok(LunarRecord {
                tt: instant(date, time)?,
                date: (*date).to_string(),
                kind,
                gamma: number(gamma)?,
                penumbral: number(penumbral)?,
                umbral: number(umbral)?,
                durations: [minutes(tot)?, minutes(par)?, minutes(pen)?],
            })
        })
        .collect()
}

fn solar_records() -> Result<Vec<SolarRecord>, String> {
    SOLAR
        .lines()
        .map(|line| {
            let f: Vec<&str> = line.split_whitespace().collect();
            let [date, time, kind, gamma, magnitude, latitude, longitude] = f.as_slice() else {
                return Err(format!("'{line}' is no solar record"));
            };
            let kind = match *kind {
                "P" => SolarEclipseKind::Partial,
                "A" => SolarEclipseKind::Annular,
                "T" => SolarEclipseKind::Total,
                "H" => SolarEclipseKind::Hybrid,
                other => return Err(format!("{date}: '{other}' is no solar kind")),
            };
            Ok(SolarRecord {
                tt: instant(date, time)?,
                date: (*date).to_string(),
                kind,
                gamma: number(gamma)?,
                magnitude: number(magnitude)?,
                latitude: hemisphere(latitude)?,
                longitude: hemisphere(longitude)?,
            })
        })
        .collect()
}

/// A source that places both bodies as one placing, whatever the eclipse
/// asks: the rejected placings, measured beside the shipped one.
struct Placed<'s, S: ShadowSource> {
    inner: &'s S,
    sun: Seen,
    moon: Seen,
}

impl<S: ShadowSource> ShadowSource for Placed<'_, S> {
    fn place(&self, body: Body, ut1: JulianDay<Ut1>, _: Seen) -> Result<Apparent, Error> {
        let seen = if body == Body::Sun {
            self.sun
        } else {
            self.moon
        };
        self.inner.place(body, ut1, seen)
    }

    fn describe(&self) -> String {
        self.inner.describe()
    }
}

/// The least, the median and the greatest of a set of differences, and
/// the greatest in size.
struct Spread {
    least: f64,
    median: f64,
    most: f64,
    worst: f64,
    of: usize,
}

impl Spread {
    fn of(values: &[f64]) -> Spread {
        Spread {
            least: values.iter().copied().fold(f64::INFINITY, f64::min),
            median: median(values.iter().copied()),
            most: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            worst: values.iter().map(|v| v.abs()).fold(0.0, f64::max),
            of: values.len(),
        }
    }

    fn row(&self, quantity: &str, unit: &str, places: usize, bound: f64) -> String {
        format!(
            "| {quantity} | {} | {:+.places$} | {:+.places$} | {:+.places$} | {} |\n",
            count(self.of),
            self.least,
            self.median,
            self.most,
            with_unit(&bound.to_string(), unit),
        )
    }
}

/// A figure with its unit, or alone when it has none.
fn with_unit(figure: &str, unit: &str) -> String {
    if unit.is_empty() {
        figure.to_owned()
    } else {
        format!("{figure} {unit}")
    }
}

fn spreads_header() -> &'static str {
    "| quantity, ours less the catalogue's | eclipses | least | median | greatest | bound |\n\
     |---|---|---|---|---|---|\n"
}

/// The record each answer is, by its greatest moment in TT.
fn tt(at: JulianDay<Ut1>) -> Result<f64, String> {
    Ok(tt_of(at, DELTA_T).map_err(|e| e.to_string())?.0.get())
}

/// Answers held against records both ways.
struct Pairing<'r, A, R> {
    /// Each record with the answer that found it.
    pairs: Vec<(&'r A, &'r R)>,
    /// The records nothing found.
    missed: Vec<&'r R>,
    /// The answers no record holds.
    extra: Vec<&'r A>,
}

/// Pairs answers with records both ways.
fn paired<'r, A, R>(
    answers: &'r [A],
    records: &'r [R],
    answer_tt: impl Fn(&A) -> Result<f64, String>,
    record_tt: impl Fn(&R) -> f64,
) -> Result<Pairing<'r, A, R>, String> {
    let times: Vec<f64> = answers.iter().map(&answer_tt).collect::<Result<_, _>>()?;
    let near = |t: f64, r: &R| (t - record_tt(r)).abs() < SAME_ECLIPSE_DAYS;
    let mut pairs = Vec::new();
    let mut missed = Vec::new();
    for record in records {
        match answers.iter().zip(&times).find(|(_, t)| near(**t, record)) {
            Some((answer, _)) => pairs.push((answer, record)),
            None => missed.push(record),
        }
    }
    let extra = answers
        .iter()
        .zip(&times)
        .filter(|(_, t)| !records.iter().any(|r| near(**t, r)))
        .map(|(answer, _)| answer)
        .collect();
    Ok(Pairing {
        pairs,
        missed,
        extra,
    })
}

fn window() -> Result<(JulianDay<Ut1>, JulianDay<Ut1>), String> {
    let at = |year| {
        teistro_calendar::gregorian::fixed_from_gregorian(year, 1, 1)
            .jd_at_midnight()
            .map(|jd| JulianDay::<Ut1>::literal(jd.get()))
            .map_err(|e| e.to_string())
    };
    Ok((at(FIRST_YEAR)?, at(LAST_YEAR + 1)?))
}

/// The lunar eclipses near each record under a source and a rule: the
/// search seeded where the catalogue says, for the comparisons that need
/// no full scan.
fn lunar_near<S: ShadowSource>(
    source: &S,
    rule: ShadowRule,
    records: &[LunarRecord],
) -> Result<Vec<LunarEclipse>, String> {
    let eclipses = Eclipses::new(source, DELTA_T).with_shadow(rule);
    let mut found = Vec::new();
    for record in records {
        let from = JulianDay::<Ut1>::literal(record.tt - 1.0);
        let to = JulianDay::<Ut1>::literal(record.tt + 1.0);
        found.extend(
            eclipses
                .lunar_between(from, to)
                .map_err(|e| format!("{}: {e}", record.date))?,
        );
    }
    Ok(found)
}

/// The difference of two Julian days, seconds.
fn seconds_of(ours: f64, theirs: f64) -> f64 {
    (ours - theirs) * 86_400.0
}

/// What the bulletins' cities measure: the claims, and the section's
/// table.
struct Local {
    claims: Vec<Claim>,
    rows: String,
    disagree: Vec<String>,
}

/// The moments a bulletin prints for a city, in its order.
const LOCAL_MOMENTS: [&str; 5] = [
    "first contact",
    "second contact",
    "third contact",
    "fourth contact",
    "maximum",
];

/// What the bulletin cities measure, before it is judged.
struct LocalOffsets {
    /// Ours less the bulletin's, seconds, per moment of [`LOCAL_MOMENTS`].
    offsets: [Vec<f64>; 5],
    /// Ours less the bulletin's magnitude at the maximum.
    magnitudes: Vec<f64>,
    /// Each moment one side has and the other does not, but for a
    /// contact below the horizon, which the bulletin leaves out.
    disagree: Vec<String>,
    /// The contacts that happened below the horizon.
    below: usize,
}

/// Every bulletin city's eclipse as the SDK sees it from there, less the
/// bulletin's.
fn local_offsets<S: ShadowSource>(
    eclipses: &Eclipses<'_, S>,
    solar: &[SolarEclipse],
    records: &[LocalRecord],
) -> Result<LocalOffsets, String> {
    let mut offsets: [Vec<f64>; 5] = Default::default();
    let mut magnitudes = Vec::new();
    let mut disagree = Vec::new();
    let mut below = 0;
    for record in records {
        let midnight = instant(&record.date, "0:0:0")?;
        let Some(eclipse) = solar
            .iter()
            .find(|e| (e.greatest.get() - midnight - 0.5).abs() < 1.0)
        else {
            return Err(format!("{}: no solar eclipse found that day", record.date));
        };
        let greatest = eclipse.greatest.get();
        let view = eclipses
            .solar_seen(eclipse, record.place, &Horizon::UPPER_LIMB_REFRACTION)
            .map_err(|e| format!("{} {}: {e}", record.date, record.city))?;
        let Some(view) = view else {
            disagree.push(format!("{} {}: not eclipsed", record.date, record.city));
            continue;
        };
        let ours = [
            Some(view.first),
            view.second,
            view.third,
            Some(view.fourth),
            Some(view.maximum),
        ];
        for (n, (printed, found)) in record.moments.iter().zip(ours).enumerate() {
            match (printed, found) {
                (Some(seconds), Some(found)) => {
                    let mut at = midnight + seconds / 86_400.0;
                    if at - greatest > 0.5 {
                        at -= 1.0;
                    } else if greatest - at > 0.5 {
                        at += 1.0;
                    }
                    offsets[n].push(seconds_of(found.at.get(), at));
                }
                (None, Some(found)) if found.altitude_deg < 0.0 => below += 1,
                (None, None) => {}
                (printed, found) => disagree.push(format!(
                    "{} {}: the {} is {} in the bulletin and {} here",
                    record.date,
                    record.city,
                    LOCAL_MOMENTS[n],
                    if printed.is_some() {
                        "printed"
                    } else {
                        "absent"
                    },
                    found.map_or(String::from("absent"), |f| format!(
                        "at altitude {:.2}°",
                        f.altitude_deg
                    )),
                )),
            }
        }
        magnitudes.push(view.magnitude - record.magnitude);
    }
    Ok(LocalOffsets {
        offsets,
        magnitudes,
        disagree,
        below,
    })
}

/// Every bulletin city's eclipse as the SDK sees it from there, against
/// the bulletin.
fn local<S: ShadowSource>(
    eclipses: &Eclipses<'_, S>,
    solar: &[SolarEclipse],
) -> Result<Local, String> {
    let records = local_records()?;
    let LocalOffsets {
        offsets,
        magnitudes,
        disagree,
        below,
    } = local_offsets(eclipses, solar, &records)?;
    let spreads: Vec<Spread> = offsets.iter().map(|o| Spread::of(o)).collect();
    let magnitude = Spread::of(&magnitudes);
    let worst = spreads.iter().take(4).map(|s| s.worst).fold(0.0, f64::max);
    let maximum = spreads.get(4).map_or(f64::NAN, |s| s.worst);
    let claims = vec![
        Claim::counted(
            "every bulletin city sees the contacts the bulletin prints, and any other only below the horizon",
            disagree.len(),
            records.len(),
        )
        .with_note(format!("{below} contacts below the horizon")),
        Claim::stated(
            "every local contact is the bulletin's to the seconds it prints",
            verdict_of(worst.is_finite() && worst <= LOCAL_BOUND_SECONDS),
            format!("worst {worst:.2} s, bound {LOCAL_BOUND_SECONDS} s"),
        ),
        Claim::stated(
            "every local maximum is the bulletin's, as flat as a shallow eclipse's is",
            verdict_of(maximum.is_finite() && maximum <= LOCAL_MAXIMUM_BOUND_SECONDS),
            format!("worst {maximum:.2} s, bound {LOCAL_MAXIMUM_BOUND_SECONDS} s"),
        ),
        Claim::stated(
            "every local magnitude is the bulletin's",
            verdict_of(magnitude.worst <= LOCAL_MAGNITUDE_BOUND),
            format!("worst {:.4}, bound {LOCAL_MAGNITUDE_BOUND}", magnitude.worst),
        ),
    ];
    let mut rows = String::from(
        "| moment, ours less the bulletin's | cities | least | median | greatest | bound |\n\
         |---|---|---|---|---|---|\n",
    );
    for (n, (spread, name)) in spreads.iter().zip(LOCAL_MOMENTS).enumerate() {
        let bound = if n == 4 {
            LOCAL_MAXIMUM_BOUND_SECONDS
        } else {
            LOCAL_BOUND_SECONDS
        };
        rows.push_str(&spread.row(name, "s", 2, bound));
    }
    rows.push_str(&magnitude.row("magnitude", "", 4, LOCAL_MAGNITUDE_BOUND));
    Ok(Local {
        claims,
        rows,
        disagree,
    })
}

fn named<T>(items: &[&T], date: impl Fn(&T) -> String) -> String {
    if items.is_empty() {
        String::from("none")
    } else {
        items
            .iter()
            .map(|item| date(item))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "a page is its sections in order, each a few lines"
)]
fn page() -> Result<String, String> {
    let lunar_records = lunar_records()?;
    let solar_records = solar_records()?;
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let eclipses = Eclipses::new(&sky, DELTA_T);
    let (from, to) = window()?;
    // The two scans and the three comparisons are independent, and each
    // is a few seconds of searching: one thread each.
    let placed = |sun, moon| Placed {
        inner: &sky,
        sun,
        moon,
    };
    let (lunar, solar, moon_apparent, astrometric, chauvenet) = std::thread::scope(|scope| {
        let lunar = scope.spawn(|| eclipses.lunar_between(from, to).map_err(|e| e.to_string()));
        let solar = scope.spawn(|| eclipses.solar_between(from, to).map_err(|e| e.to_string()));
        let moon_apparent = scope.spawn(|| {
            lunar_near(
                &placed(Seen::Astrometric, Seen::Apparent),
                ShadowRule::Danjon,
                &lunar_records,
            )
        });
        let astrometric = scope.spawn(|| {
            lunar_near(
                &placed(Seen::Astrometric, Seen::Astrometric),
                ShadowRule::Danjon,
                &lunar_records,
            )
        });
        let chauvenet = scope.spawn(|| lunar_near(&sky, ShadowRule::Chauvenet, &lunar_records));
        let joined = |name: &str| format!("the {name} thread panicked");
        Ok::<_, String>((
            lunar.join().map_err(|_| joined("lunar"))??,
            solar.join().map_err(|_| joined("solar"))??,
            moon_apparent.join().map_err(|_| joined("placing"))??,
            astrometric.join().map_err(|_| joined("placing"))??,
            chauvenet.join().map_err(|_| joined("Chauvenet"))??,
        ))
    })?;
    let lunar_tt = |e: &LunarEclipse| tt(e.greatest);
    let solar_tt = |e: &SolarEclipse| tt(e.greatest);
    let Pairing {
        pairs: lunar_pairs,
        missed: lunar_missed,
        extra: lunar_extra,
    } = paired(&lunar, &lunar_records, lunar_tt, |r| r.tt)?;
    let Pairing {
        pairs: solar_pairs,
        missed: solar_missed,
        extra: solar_extra,
    } = paired(&solar, &solar_records, solar_tt, |r| r.tt)?;

    let mut claims = Vec::new();
    let mut problems = Vec::new();
    let local = local(&eclipses, &solar)?;
    let both_ways = |kind: &str, found: usize, missed: usize, extra: usize, of: usize| {
        Claim::counted(
            format!("every catalogued {kind} eclipse of {FIRST_YEAR} to {LAST_YEAR} is found, and no other"),
            missed + extra,
            of,
        )
        .with_note(format!("{} found, {missed} missed, {extra} not catalogued", count(found)))
    };
    claims.push(both_ways(
        "lunar",
        lunar.len(),
        lunar_missed.len(),
        lunar_extra.len(),
        lunar_records.len(),
    ));
    claims.push(both_ways(
        "solar",
        solar.len(),
        solar_missed.len(),
        solar_extra.len(),
        solar_records.len(),
    ));

    // The lunar comparisons.
    let mut l_greatest = Vec::new();
    let mut l_gamma = Vec::new();
    let mut l_umbral = Vec::new();
    let mut l_penumbral = Vec::new();
    let mut l_durations: [Vec<f64>; 3] = Default::default();
    let mut l_kinds = Vec::new();
    let mut l_phases_differ = Vec::new();
    for (ours, record) in &lunar_pairs {
        l_greatest.push(seconds_of(tt(ours.greatest)?, record.tt));
        l_gamma.push(ours.gamma - record.gamma);
        l_umbral.push(ours.umbral_magnitude - record.umbral);
        l_penumbral.push(ours.penumbral_magnitude - record.penumbral);
        if ours.kind != record.kind {
            l_kinds.push(record.date.clone());
        }
        for ((phase, theirs), spread) in ours
            .durations()
            .iter()
            .zip(record.durations)
            .zip(l_durations.iter_mut())
        {
            match (phase, theirs) {
                (Some(days), Some(minutes)) => spread.push(days * 1440.0 - minutes),
                (None, None) => {}
                _ => l_phases_differ.push(record.date.clone()),
            }
        }
    }
    claims.push(Claim::counted(
        "a lunar eclipse's kind is the catalogue's",
        l_kinds.len(),
        lunar_pairs.len(),
    ));
    claims.push(Claim::counted(
        "a lunar eclipse has the phases the catalogue times, and no others",
        l_phases_differ.len(),
        lunar_pairs.len(),
    ));
    let l_greatest_spread = Spread::of(&l_greatest);
    let l_gamma_spread = Spread::of(&l_gamma);
    let l_umbral_spread = Spread::of(&l_umbral);
    let l_penumbral_spread = Spread::of(&l_penumbral);
    let l_duration_spreads = l_durations.each_ref().map(|d| Spread::of(d));

    // The solar comparisons.
    let mut s_greatest = Vec::new();
    let mut s_gamma = Vec::new();
    let mut s_magnitude = Vec::new();
    let mut s_latitude = Vec::new();
    let mut s_longitude = Vec::new();
    let mut s_kinds = Vec::new();
    for (ours, record) in &solar_pairs {
        s_greatest.push(seconds_of(tt(ours.greatest)?, record.tt));
        s_gamma.push(ours.gamma - record.gamma);
        s_magnitude.push(ours.magnitude - record.magnitude);
        s_latitude.push(ours.point.latitude.get() - record.latitude);
        s_longitude.push(
            (ours.point.longitude.get() - record.longitude + 540.0).rem_euclid(360.0) - 180.0,
        );
        if ours.kind != record.kind {
            s_kinds.push(record.date.clone());
        }
    }
    claims.push(Claim::counted(
        "a solar eclipse's kind is the catalogue's, hybrids read at the greatest eclipse",
        s_kinds.len(),
        solar_pairs.len(),
    ));
    let s_greatest_spread = Spread::of(&s_greatest);
    let s_gamma_spread = Spread::of(&s_gamma);
    let s_magnitude_spread = Spread::of(&s_magnitude);
    let s_latitude_spread = Spread::of(&s_latitude);
    let s_longitude_spread = Spread::of(&s_longitude);

    let bounded = |rule: &str, spreads: &[&Spread], bound: f64, unit: &str| {
        let worst = spreads.iter().map(|s| s.worst).fold(0.0, f64::max);
        Claim::stated(
            rule,
            verdict_of(worst.is_finite() && worst <= bound),
            format!(
                "worst {}, bound {}",
                with_unit(&format!("{worst:.4}"), unit),
                with_unit(&bound.to_string(), unit)
            ),
        )
    };
    claims.push(bounded(
        "the greatest moment is the catalogue's to the seconds it prints",
        &[&l_greatest_spread, &s_greatest_spread],
        GREATEST_BOUND_SECONDS,
        "s",
    ));
    claims.push(bounded(
        "gamma is the catalogue's",
        &[&l_gamma_spread, &s_gamma_spread],
        GAMMA_BOUND,
        "Earth radii",
    ));
    claims.push(bounded(
        "every magnitude is the catalogue's, the umbra's under Danjon's rule",
        &[&l_umbral_spread, &l_penumbral_spread, &s_magnitude_spread],
        MAGNITUDE_BOUND,
        "",
    ));
    claims.push(bounded(
        "a lunar eclipse's phases last as long as the catalogue says",
        &l_duration_spreads.iter().collect::<Vec<_>>(),
        DURATION_BOUND_MINUTES,
        "min",
    ));
    claims.push(bounded(
        "a solar eclipse is greatest where the catalogue says",
        &[&s_latitude_spread, &s_longitude_spread],
        POINT_BOUND_DEG,
        "°",
    ));

    // The rejected placings and the other shadow rule, searched where the
    // catalogue puts each lunar eclipse; the shipped one is the full scan.
    let placing = |found: &[LunarEclipse]| -> Result<f64, String> {
        let pairs = paired(found, &lunar_records, lunar_tt, |r| r.tt)?.pairs;
        let offsets: Vec<f64> = pairs
            .iter()
            .map(|(ours, record)| Ok(seconds_of(tt(ours.greatest)?, record.tt)))
            .collect::<Result<_, String>>()?;
        Ok(median(offsets))
    };
    let shipped = placing(&lunar)?;
    let placings = [
        ("both apparent (shipped)", shipped),
        (
            "the Moon apparent, the Sun astrometric",
            placing(&moon_apparent)?,
        ),
        ("both astrometric", placing(&astrometric)?),
    ];
    let mut placing_rows = String::from(
        "| the lunar eclipse's placing | greatest moment, median offset |\n|---|---|\n",
    );
    let mut shipped_is_best = true;
    for (name, offset) in placings {
        shipped_is_best &= shipped.abs() <= offset.abs();
        let _ = writeln!(placing_rows, "| {name} | {offset:+.1} s |");
    }
    claims.push(Claim::stated(
        "a lunar eclipse reads both bodies apparent, nearer the catalogue than either other placing",
        verdict_of(shipped_is_best),
        format!("median {shipped:+.2} s"),
    ));
    let chauvenet_pairs = paired(&chauvenet, &lunar_records, lunar_tt, |r| r.tt)?.pairs;
    let chauvenet_umbral = median(
        chauvenet_pairs
            .iter()
            .map(|(ours, record)| ours.umbral_magnitude - record.umbral),
    );
    let chauvenet_penumbral = median(
        chauvenet_pairs
            .iter()
            .map(|(ours, record)| ours.penumbral_magnitude - record.penumbral),
    );
    claims.push(Claim::stated(
        "the catalogue enlarges the shadow by Danjon's rule and not Chauvenet's",
        verdict_of(
            chauvenet_umbral.abs() > l_umbral_spread.median.abs()
                && chauvenet_penumbral.abs() > l_penumbral_spread.median.abs(),
        ),
        format!(
            "Chauvenet's median {chauvenet_umbral:+.4} umbral, {chauvenet_penumbral:+.4} penumbral"
        ),
    ));

    for claim in &claims {
        if claim.verdict != Verdict::Holds {
            problems.push(format!("{}: {}", claim.rule, claim.measured));
        }
    }

    let mut out = String::new();
    out.push_str("# Eclipses, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask eclipses`. Do not edit:\n\
         `check-eclipses` regenerates this page and fails on any difference.\n\n\
         It holds `astro::eclipse` (`eclipses.md`) over the built-in\n\
         ephemeris against every eclipse of NASA's Five Millennium Canon\n\
         between the years it reads, both ways, and measures the placings\n\
         and the shadow rule the design chose between.\n",
    );
    claims.extend(local.claims);
    out.push_str("\n## 1. The claims\n\n");
    out.push_str(&table(&claims));
    let _ = write!(
        out,
        "\n## 2. The records\n\n\
         Fred Espenak and Jean Meeus, *Five Millennium Canon of Solar\n\
         Eclipses* (NASA TP-2006-214141) and *of Lunar Eclipses* (NASA\n\
         TP-2009-214172), NASA Goddard Space Flight Center, in the public\n\
         domain: the catalogues `5MCSEcatalog.txt` and `5MKLEcatalog.txt`,\n\
         {lunar} lunar and {solar} solar eclipses from {FIRST_YEAR} to\n\
         {LAST_YEAR}, held in `xtask/src/eclipses/` a line an eclipse. The\n\
         catalogue's instants are TT; ours are UT1 and taken back to TT\n\
         under the Delta T model the search ran with, so the model cancels.\n\
         An answer and a record are one eclipse when their greatest moments\n\
         are within {SAME_ECLIPSE_DAYS} days.\n\n\
         Missed lunar: {lm}. Not catalogued lunar: {le}. Missed solar: {sm}.\n\
         Not catalogued solar: {se}.\n",
        lunar = count(lunar_records.len()),
        solar = count(solar_records.len()),
        lm = named(&lunar_missed, |r| r.date.clone()),
        le = lunar_extra.len(),
        sm = named(&solar_missed, |r| r.date.clone()),
        se = solar_extra.len(),
    );
    out.push_str("\n## 3. Lunar eclipses\n\n");
    out.push_str(spreads_header());
    out.push_str(&l_greatest_spread.row("greatest moment", "s", 2, GREATEST_BOUND_SECONDS));
    out.push_str(&l_gamma_spread.row("gamma", "", 5, GAMMA_BOUND));
    out.push_str(&l_umbral_spread.row("umbral magnitude", "", 5, MAGNITUDE_BOUND));
    out.push_str(&l_penumbral_spread.row("penumbral magnitude", "", 5, MAGNITUDE_BOUND));
    for (spread, phase) in l_duration_spreads
        .iter()
        .zip(["total", "partial", "penumbral"])
    {
        out.push_str(&spread.row(
            &format!("{phase} phase, minutes"),
            "min",
            2,
            DURATION_BOUND_MINUTES,
        ));
    }
    out.push_str(
        "\nThe catalogue prints a phase to a tenth of a minute, and a phase\n\
         grazing its shadow's edge is long for a small error in gamma: the\n\
         greatest differences are the shallowest eclipses.\n",
    );
    out.push_str("\n## 4. Solar eclipses\n\n");
    out.push_str(spreads_header());
    out.push_str(&s_greatest_spread.row("greatest moment", "s", 2, GREATEST_BOUND_SECONDS));
    out.push_str(&s_gamma_spread.row("gamma", "", 5, GAMMA_BOUND));
    out.push_str(&s_magnitude_spread.row("magnitude", "", 5, MAGNITUDE_BOUND));
    out.push_str(&s_latitude_spread.row("latitude of greatest eclipse", "°", 2, POINT_BOUND_DEG));
    out.push_str(&s_longitude_spread.row("longitude of greatest eclipse", "°", 2, POINT_BOUND_DEG));
    out.push_str(
        "\nThe catalogue prints the point to a tenth of a degree. Its gamma\n\
         is the sphere's and is read so here; the kind and the magnitude\n\
         are the flattened Earth's (`eclipses.md` §4.3).\n",
    );
    let _ = write!(
        out,
        "\n## 5. The placing and the shadow\n\n\
         Which place of each body the shadow is read from decides the\n\
         greatest moment; the shipped placing is the light's path\n\
         (`eclipses.md` §4.1), and the other two are measured here over\n\
         the same records.\n\n{placing_rows}\n\
         The shadow enlarged by Chauvenet's rule rather than Danjon's\n\
         reads the umbral magnitudes {chauvenet_umbral:+.4} and the\n\
         penumbral {chauvenet_penumbral:+.4} from the catalogue at the\n\
         median, where Danjon's reads {du:+.5} and {dp:+.5}: the catalogue\n\
         is Danjon's, which is the default, and Chauvenet's stays a knob\n\
         for the almanacs that used it.\n",
        du = l_umbral_spread.median,
        dp = l_penumbral_spread.median,
    );
    let _ = write!(
        out,
        "\n## 6. Local circumstances\n\n\
         `Eclipses::solar_seen` reads a solar eclipse from a place: the\n\
         eclipse's own Sun and Moon less the station's geocentric position,\n\
         the maximum where the magnitude is greatest and the contacts where\n\
         the discs touch, outside for the first and fourth and inside for\n\
         the second and third (`eclipses.md` §4.5). It is held to the {cities}\n\
         cities of NASA's bulletins for the total eclipse of 2009 July 22\n\
         and the annular one of 2010 January 15, Kathmandu among them,\n\
         each at the latitude, longitude and elevation the bulletin prints.\n\
         A contact the bulletin leaves out happened below the horizon; the\n\
         SDK reports it with the Sun's altitude, and the claim holds it\n\
         there. The bulletins print UT to a tenth of a second.\n\n{rows}\n\
         The contacts are crossings and agree to a few seconds. The maximum\n\
         is the top of a curve, and the greatest differences are the\n\
         shallowest eclipses, Europe's in 2010 at magnitudes under 0.2 and\n\
         the Sun near the horizon, where the curve is flattest; the\n\
         magnitudes themselves agree.\n\n\
         Disagreements: {disagree}.\n",
        cities = count(LOCAL.lines().count()),
        rows = local.rows,
        disagree = if local.disagree.is_empty() {
            String::from("none")
        } else {
            local.disagree.join("; ")
        },
    );
    out.push_str(
        "\n## 7. What the records decide\n\n\
         The search finds the canon's eclipses and no others over two\n\
         centuries, of every kind, and agrees with each to the precision\n\
         the canon prints. So `astro::eclipse` is the SDK's eclipse,\n\
         ready for the almanac's day and the muhurta's blackouts: the\n\
         eclipse's nakshatra (grahanotpatha) and its sutak.\n",
    );
    if problems.is_empty() {
        Ok(fill(&out))
    } else {
        Err(problems.join("\n      "))
    }
}

pub(crate) fn generate(root: &Path) -> i32 {
    match page() {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page() {
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask eclipses") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

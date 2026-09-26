//! The measurement pass over the **transit hit list**
//! (`03-design/transit-hit-list.md`).
//!
//! Nothing records a hit list — the corpus has no transit search and no
//! aspect at all — so this pass holds the list to what the sky and a
//! founded chart say, over the recorded births and one year, asked for in
//! one batch the way a consumer with many charts would ask (the Moon's
//! aspects for the first birth alone: at twenty-seven crossings of every
//! line a year they are most of a batch's price, and one chart's show
//! everything the rest would):
//!
//! - every ingress and every exact aspect is read back through a chart
//!   founded by the SDK, and stands where the list says;
//! - every line is crossed alternately forward and back exactly when the
//!   graha stood still an odd number of times between, which ties the
//!   crossings to the stations the list reports beside them;
//! - every orb's window opens, holds its exact hits and closes, in that
//!   order;
//! - Ketu's every ingress is Rahu's, six signs on, at the same instant.
//!
//! `cargo xtask hits` writes the page; `check-hits` regenerates it in
//! memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{ChartKind, Graha};
use teistro::gochar::GRAHAS;
use teistro::gochar::hits::{AspectPhase, HitEvent, Motion};
use teistro::quantity::{JulianDay, Utc};
use teistro::{
    ChartFoundation, Context, Document, Ephemeris, Hit, HitKind, HitRequest, NatalPoint, UtcOffset,
};

use crate::births::{Birth, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict as Decided, capitalised, count, spelled, table};

const PAGE: &str = "docs/03-design/transit-hit-list-measured.md";

/// The profile the corpus was recorded under, which the births are
/// founded in.
const PROFILE: &str = "conformance-baseline";

/// The year searched: 2026, from its first instant to its last.
const FROM: f64 = 2_461_041.5;
const TO: f64 = 2_461_406.5;

/// The orb asked for, degrees: wide enough that a slow graha's station
/// falls inside a window now and then, which is what the window's grammar
/// has to survive.
const ORB_DEG: f64 = 3.0;

/// How far either side of an ingress a chart is founded, days: a second.
const SECOND: f64 = 1.0 / 86_400.0;

/// The patch the read-back charts are founded under, over the profile:
/// the profile is **topocentric**, so a chart must be founded at the
/// birth's own place to see the sky the list was searched in, and a polar
/// place's winter has no sunrise for the profile's day to start from
/// (Tromsø's January). Civil midnight starts it instead; a day's bounds are
/// no part of a longitude.
const READ_BACK: &str = r#"{"day":{"polar_day_policy":"CIVIL_MIDNIGHT"}}"#;

/// How near an exact aspect's founded separation must stand to its angle,
/// degrees: a tenth of an arcsecond, which the Moon crosses in under a
/// fifth of a second.
const EXACT_DEG: f64 = 1.0 / 36_000.0;

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask hits") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn name(graha: Graha) -> String {
    capitalised(&graha.key().to_lowercase())
}

/// The kind of an event, as the index of a column: the sky's three, then
/// the aspects.
const fn kind_column(event: HitEvent) -> usize {
    match event {
        HitEvent::SignIngress { .. } => 0,
        HitEvent::NakshatraIngress { .. } => 1,
        HitEvent::Station { .. } => 2,
        HitEvent::Aspect { .. } => 3,
    }
}

/// A line one graha crosses, for the stations' parity: the sky's sign and
/// nakshatra boundaries, and each birth's exact conjunction and
/// opposition to each natal point (one line apiece, unlike a square's
/// two).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Line {
    Sign(u16),
    Nakshatra(u16),
    Aspect(usize, u8, u16),
}

fn point_index(point: NatalPoint) -> u8 {
    match point {
        NatalPoint::Graha { graha } => graha as u8,
        NatalPoint::Lagna => u8::MAX,
    }
}

/// The boundary an ingress crossed: the entered division's start moving
/// forward, its end moving back.
fn boundary(into: usize, motion: Motion, of: u16) -> u16 {
    let index = match motion {
        Motion::Direct => into,
        Motion::Retrograde => into + 1,
    };
    u16::try_from(index % usize::from(of)).unwrap_or_default()
}

/// Where a founded chart places a graha, and each natal point of a birth.
fn longitude_of(chart: &ChartFoundation, graha: Graha) -> Result<f64, String> {
    chart
        .graha(graha)
        .map(|at| at.longitude_deg)
        .ok_or_else(|| format!("a founded chart has no {}", graha.key()))
}

fn natal_deg(document: &Document, point: NatalPoint) -> Result<f64, String> {
    match point {
        NatalPoint::Graha { graha } => longitude_of(&document.foundation, graha),
        NatalPoint::Lagna => Ok(document.foundation.lagna_deg),
    }
}

/// The distance from a separation to an aspect's angle, from either side.
fn off_angle(separation: f64, angle: u16) -> f64 {
    let angle = f64::from(angle);
    [angle, 360.0 - angle]
        .into_iter()
        .map(|target| {
            let apart = (separation - target).rem_euclid(360.0);
            apart.min(360.0 - apart)
        })
        .fold(f64::INFINITY, f64::min)
}

#[derive(Default)]
struct Measured {
    /// The sky's events of the year by graha and kind.
    by_graha: [[usize; 3]; 9],
    /// Every birth's aspects by phase.
    phases: [usize; 3],
    ingresses: usize,
    ingresses_off: usize,
    exact: usize,
    exact_off: usize,
    worst_exact_deg: f64,
    successions: usize,
    parity_wrong: usize,
    windows: usize,
    grammar_wrong: usize,
    grazes: usize,
    ketu: usize,
    ketu_wrong: usize,
}

/// The sky's ingresses, read back through a chart founded a second either
/// side of each: the graha stands in the division it left, then in the one
/// it entered.
fn read_ingresses(
    reader: &Context,
    birth: &Birth,
    sky: &[&Hit],
    out: &mut Measured,
) -> Result<(), String> {
    let ingresses: Vec<&Hit> = sky
        .iter()
        .copied()
        .filter(|hit| kind_column(hit.event) < 2)
        .collect();
    let either_side: Vec<JulianDay<Utc>> = ingresses
        .iter()
        .flat_map(|hit| {
            [
                JulianDay::<Utc>::literal(hit.instant.get() - SECOND),
                JulianDay::<Utc>::literal(hit.instant.get() + SECOND),
            ]
        })
        .collect();
    let founded = reader
        .chart()
        .found_many(
            &either_side,
            &birth.document.foundation.place,
            UtcOffset::UTC,
            ChartKind::Natal,
        )
        .map_err(|why| format!("founding either side of the ingresses: {why}"))?
        .value;
    for (hit, pair) in ingresses.iter().zip(founded.chunks(2)) {
        let (into, motion, of) = match hit.event {
            HitEvent::SignIngress { into, motion } => (into as usize, motion, 12_u16),
            HitEvent::NakshatraIngress { into, motion } => (into as usize, motion, 27),
            HitEvent::Station { .. } | HitEvent::Aspect { .. } => continue,
        };
        let width = 360.0 / f64::from(of);
        let division = |chart: &ChartFoundation| -> Result<usize, String> {
            let deg = longitude_of(chart, hit.graha)?.rem_euclid(360.0);
            // A longitude over a division's width is a small whole number.
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a division index is 0 to 26"
            )]
            Ok((deg / width) as usize % usize::from(of))
        };
        let left = match motion {
            Motion::Direct => (into + usize::from(of) - 1) % usize::from(of),
            Motion::Retrograde => (into + 1) % usize::from(of),
        };
        let (Some(before), Some(after)) = (pair.first(), pair.get(1)) else {
            return Err(String::from("a founded pair came back short"));
        };
        out.ingresses += 1;
        if (division(before)?, division(after)?) != (left, into) {
            out.ingresses_off += 1;
        }
    }
    Ok(())
}

/// Each birth's exact aspects, read back through a chart founded at each:
/// the separation stands at the aspect's angle.
fn read_aspects(
    reader: &Context,
    birth: &Birth,
    hits: &[Hit],
    out: &mut Measured,
) -> Result<(), String> {
    let exact: Vec<(&Hit, NatalPoint, u16)> = hits
        .iter()
        .filter_map(|hit| match hit.event {
            HitEvent::Aspect {
                to,
                angle,
                phase: AspectPhase::Exact,
                ..
            } => Some((hit, to, angle)),
            _ => None,
        })
        .collect();
    let instants: Vec<JulianDay<Utc>> = exact.iter().map(|(hit, ..)| hit.instant).collect();
    let founded = reader
        .chart()
        .found_many(
            &instants,
            &birth.document.foundation.place,
            UtcOffset::UTC,
            ChartKind::Natal,
        )
        .map_err(|why| format!("{}: founding at the aspects: {why}", birth.name))?
        .value;
    for ((hit, to, angle), chart) in exact.iter().zip(&founded) {
        let separation = longitude_of(chart, hit.graha)? - natal_deg(&birth.document, *to)?;
        let off = off_angle(separation, *angle);
        out.exact += 1;
        out.worst_exact_deg = out.worst_exact_deg.max(off);
        if off > EXACT_DEG {
            out.exact_off += 1;
        }
    }
    Ok(())
}

/// The lines each graha crossed, with the way it crossed them, and its
/// stations, keyed so the parity can be read line by line.
type Crossings = BTreeMap<(u8, Line), Vec<(f64, Motion)>>;

/// The sky's lines from its ingresses, or a birth's from its exact
/// aspects.
fn crossings(birth: usize, hits: &[Hit], sky: bool, lines: &mut Crossings) {
    for hit in hits {
        let line = match hit.event {
            HitEvent::SignIngress { into, motion } if sky => {
                (Line::Sign(boundary(into as usize, motion, 12)), motion)
            }
            HitEvent::NakshatraIngress { into, motion } if sky => {
                (Line::Nakshatra(boundary(into as usize, motion, 27)), motion)
            }
            HitEvent::Aspect {
                to,
                angle,
                phase: AspectPhase::Exact,
                motion,
            } if !sky => (Line::Aspect(birth, point_index(to), angle), motion),
            _ => continue,
        };
        lines
            .entry((hit.graha as u8, line.0))
            .or_default()
            .push((hit.instant.get(), line.1));
    }
}

/// Successive crossings of one line: the direction changes exactly when
/// the graha stood still an odd number of times between them.
fn parity(lines: &Crossings, stations: &[Vec<f64>; 9], out: &mut Measured) {
    for ((graha, _), crossed) in lines {
        let stood = stations
            .get(usize::from(*graha))
            .map_or(&[][..], Vec::as_slice);
        for pair in crossed.windows(2) {
            let [(a, from), (b, to)] = pair else { continue };
            let between = stood.iter().filter(|at| **at > *a && **at < *b).count();
            out.successions += 1;
            if (between % 2 == 1) != (from != to) {
                out.parity_wrong += 1;
            }
        }
    }
}

/// Each window's phases in order: it opens, holds its exact hits and
/// closes, a window open when the year begins closing unopened and one
/// open when it ends never closing. A window closed with no exact hit
/// inside is a graze: the graha turned inside the orb.
fn grammar(birth: usize, hits: &[Hit], out: &mut Measured) {
    let mut windows: BTreeMap<(usize, u8, u8, u16), Vec<AspectPhase>> = BTreeMap::new();
    for hit in hits {
        if let HitEvent::Aspect {
            to, angle, phase, ..
        } = hit.event
        {
            windows
                .entry((birth, hit.graha as u8, point_index(to), angle))
                .or_default()
                .push(phase);
        }
    }
    for phases in windows.values() {
        // Whether a window is open, and whether it has held an exact hit;
        // before the first event, unknown.
        let mut open: Option<bool> = None;
        let mut exact_inside = false;
        for phase in phases {
            match phase {
                AspectPhase::Entering => {
                    if open == Some(true) {
                        out.grammar_wrong += 1;
                    }
                    open = Some(true);
                    exact_inside = false;
                }
                AspectPhase::Exact => {
                    if open == Some(false) {
                        out.grammar_wrong += 1;
                    }
                    exact_inside = true;
                }
                AspectPhase::Leaving => {
                    if open == Some(false) {
                        out.grammar_wrong += 1;
                    }
                    if open == Some(true) {
                        out.windows += 1;
                        if !exact_inside {
                            out.grazes += 1;
                        }
                    }
                    open = Some(false);
                }
            }
        }
    }
}

/// Ketu's ingresses against Rahu's: the same instants, six signs on.
fn ketu(sky: &[&Hit], out: &mut Measured) {
    let of = |graha: Graha| -> Vec<(f64, usize)> {
        sky.iter()
            .filter(|hit| hit.graha == graha)
            .filter_map(|hit| match hit.event {
                HitEvent::SignIngress { into, .. } => Some((hit.instant.get(), into as usize)),
                _ => None,
            })
            .collect()
    };
    let (rahu, ketu) = (of(Graha::Rahu), of(Graha::Ketu));
    out.ketu = rahu.len().max(ketu.len());
    out.ketu_wrong = out.ketu
        - rahu
            .iter()
            .zip(&ketu)
            .filter(|((a, r), (b, k))| (a - b).abs() < 1e-6 && (r + 6) % 12 == *k)
            .count();
}

fn measure(sdk: &Context, reader: &Context, born: &[Birth]) -> Result<Measured, String> {
    let year =
        HitRequest::between(JulianDay::literal(FROM), JulianDay::literal(TO)).with_orb(ORB_DEG);
    let Some(first_birth) = born.first() else {
        return Err(String::from("no recorded birth to search against"));
    };
    // The sky's events and the first chart's every aspect, every graha.
    let first = sdk
        .chart()
        .hits(&first_birth.document, &year)
        .map_err(|why| format!("{}: {why}", first_birth.name))?
        .value;
    // Every chart's aspects in one batch, the Moon's excepted.
    let slower = year
        .with_grahas(GRAHAS.into_iter().filter(|graha| *graha != Graha::Moon))
        .with_kinds([HitKind::Aspect]);
    let mut lists = sdk
        .chart()
        .hits_many(born.iter().map(|birth| &birth.document), &slower)
        .map_err(|why| format!("the batch: {why}"))?
        .value;
    if let Some(list) = lists.first_mut() {
        list.extend(
            first
                .iter()
                .filter(|hit| hit.graha == Graha::Moon && kind_column(hit.event) == 3)
                .copied(),
        );
        teistro::gochar::hits::sort(list);
    }
    let mut out = Measured::default();
    let sky: Vec<&Hit> = first
        .iter()
        .filter(|hit| kind_column(hit.event) < 3)
        .collect();
    let mut stations: [Vec<f64>; 9] = Default::default();
    for hit in &sky {
        let column = kind_column(hit.event);
        if let Some(cell) = out
            .by_graha
            .get_mut(hit.graha as usize)
            .and_then(|row| row.get_mut(column))
        {
            *cell += 1;
        }
        if column == 2
            && let Some(times) = stations.get_mut(hit.graha as usize)
        {
            times.push(hit.instant.get());
        }
    }
    read_ingresses(reader, first_birth, &sky, &mut out)?;
    ketu(&sky, &mut out);
    let mut lines = Crossings::new();
    crossings(0, &first, true, &mut lines);
    for (index, (birth, hits)) in born.iter().zip(&lists).enumerate() {
        for hit in hits {
            if let HitEvent::Aspect { phase, .. } = hit.event
                && let Some(cell) = out.phases.get_mut(phase as usize)
            {
                *cell += 1;
            }
        }
        read_aspects(reader, birth, hits, &mut out)?;
        crossings(index, hits, false, &mut lines);
        grammar(index, hits, &mut out);
    }
    parity(&lines, &stations, &mut out);
    Ok(out)
}

/// What the pass decides, each claim counted over what it read.
fn claims(measured: &Measured) -> Vec<Claim> {
    vec![
        Claim::counted(
            "every sign and nakshatra ingress stands between the division it left and the one it entered in charts founded a second either side",
            measured.ingresses_off,
            measured.ingresses,
        ),
        Claim::counted(
            "every exact aspect stands at its angle, to a tenth of an arcsecond, in a chart founded at its instant",
            measured.exact_off,
            measured.exact,
        ),
        Claim::counted(
            "successive crossings of one line run opposite ways exactly when the graha stood still an odd number of times between them",
            measured.parity_wrong,
            measured.successions,
        ),
        Claim::counted(
            "every orb's window opens, holds its exact hits and closes, in that order",
            measured.grammar_wrong,
            measured.windows,
        ),
        Claim::counted(
            "Ketu's every ingress is Rahu's at the same instant, six signs on",
            measured.ketu_wrong,
            measured.ketu,
        ),
    ]
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = Context::builder()
        .profile(PROFILE)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("{PROFILE}: {why}"))?;
    let born = births(root, &sdk)?;
    let reader = Context::builder()
        .profile(PROFILE)
        .settings_json(READ_BACK)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("{PROFILE} with {READ_BACK}: {why}"))?;
    let measured = measure(&sdk, &reader, &born)?;

    let mut out = String::new();
    let _ = writeln!(
        out,
        "# The transit hit list, measured\n\n\
         Status: `generated` by `cargo xtask hits`. Do not edit:\n\
         `check-hits` regenerates this page and fails on any difference.\n\n\
         The design this measures is `transit-hit-list.md`. **Nothing\n\
         records a hit list** — the corpus has no transit search and no\n\
         aspect at all — so this page holds the list to what the sky and a\n\
         founded chart say, over 2026. The first recorded birth is asked for\n\
         everything: every graha, every kind of event, and the conjunction\n\
         and the opposition to each natal graha and the lagna (C145), with a\n\
         {ORB_DEG}° orb (C146). All {} births, founded under `{PROFILE}`, are\n\
         then asked for their aspects in **one batch**\n\
         (`sdk.chart().hits_many`), the Moon's excepted: it crosses each\n\
         line twenty-seven times a year, which would be most of the batch's\n\
         price, and the first chart's Moon shows everything the rest would.\n\
         The profile is topocentric, so every read-back chart is founded at\n\
         the birth's own place, under the profile with a polar day reckoned\n\
         from civil midnight (Tromsø has no January sunrise to start one).\n",
        spelled(born.len())
    );
    let mut rows = String::from(
        "| graha | sign ingresses | nakshatra ingresses | stations |\n|---|---:|---:|---:|\n",
    );
    for graha in GRAHAS {
        if let Some(row) = measured.by_graha.get(graha as usize) {
            let [signs, nakshatras, stations] = row;
            let _ = writeln!(
                rows,
                "| {} | {signs} | {nakshatras} | {stations} |",
                name(graha)
            );
        }
    }
    let [entering, exact, leaving] = measured.phases;
    let _ = writeln!(
        out,
        "## 1. What a year holds\n\n\
         The sky's events, the same for every chart, which the batch refines\n\
         once and hands to each:\n\n{rows}\n\
         Against the births' own points: {} exact aspects, {} windows\n\
         entered and {} left, of which {} closed windows held no exact\n\
         hit at all — a graha turning inside the orb and going back the way\n\
         it came.\n",
        count(exact),
        count(entering),
        count(leaving),
        count(measured.grazes)
    );

    let claims = claims(&measured);
    let _ = writeln!(out, "## 2. What this pass decides\n");
    let _ = writeln!(out, "{}", table(&claims));
    let falsified = claims
        .iter()
        .filter(|claim| claim.verdict == Decided::Falsified)
        .count();
    let _ = writeln!(
        out,
        "{} The worst exact aspect stood {:.4}″ from its angle. The parity\n\
         reads every line the list crosses — the sign and nakshatra\n\
         boundaries and each birth's conjunction and opposition lines — so a\n\
         station missing from the list, or one reported where the graha did\n\
         not turn, breaks it at the next crossing.",
        match falsified {
            0 => format!("None of the {} claims is falsified.", spelled(claims.len())),
            1 => format!("One of the {} claims is falsified.", spelled(claims.len())),
            _ => format!(
                "{} of the {} claims are falsified.",
                capitalised(&spelled(falsified)),
                spelled(claims.len())
            ),
        },
        measured.worst_exact_deg * 3_600.0
    );
    Ok(crate::measure::fill(&out))
}

//! The measurement pass over **Sade Sati** (`03-design/sade-sati.md`).
//!
//! Nothing records a Sade Sati — no classical text names it (C147) and the
//! corpus has no transit search — so this pass holds the SDK's periods to
//! what the sky says, over the recorded births and a century, asked for in
//! one batch per reckoning the way a consumer with many charts would ask:
//!
//! - every bound of every visit is read back through the gochar a second
//!   either side of it, and Saturn stands in the visit's house inside it
//!   and in a neighbour outside;
//! - every Sade Sati is its three phases in order;
//! - the degree reckoning begins earlier than the sign reckoning exactly
//!   when the natal Moon stands in the first half of its sign, which is
//!   the geometry of the two lattices and is tested both ways;
//! - a period is the same bits however the window is drawn around it, and
//!   a batch answers as each chart alone;
//! - the baseline engine's rule, joining two visits to one sign under 270
//!   days apart (C148), is measured against the circuit that replaces it.
//!
//! `cargo xtask sade-sati` writes the page; `check-sade-sati` regenerates it
//! in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::quantity::{JulianDay, Utc};
use teistro::sade_sati::{Phase, Reckoning, Report, SadeSati, Spell};
use teistro::{Context, Document, Ephemeris, GocharRequest, SadeSatiRequest};

use crate::births::{Birth, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict as Decided, capitalised, count, spelled, table, verdict_of};

const PAGE: &str = "docs/03-design/sade-sati-measured.md";

/// The century searched: 1950 to 2050, from the first instant of each.
const FROM: f64 = 2_433_282.5;
const TO: f64 = 2_469_807.5;

/// How far either side of a bound the gochar is read, days: a second.
const SECOND: f64 = 1.0 / 86_400.0;

/// The patch over the default profile: a polar place's summer has no
/// sunrise to start the day from (Tromsø's June), so civil midnight starts
/// it. A day's bounds are no part of Saturn's longitude.
const POLAR: &str = r#"{"day":{"polar_day_policy":"CIVIL_MIDNIGHT"}}"#;

/// The baseline engine's gap under which two visits to one sign are one
/// period, days (C148): the rule the circuit replaces.
const BASELINE_MERGE_DAYS: f64 = 270.0;

/// The births whose periods are asked for again one by one, for the batch
/// claim: a batch of these is the batch the rest share.
const ALONE: usize = 3;

/// The window the batch claim asks over, 2020 to 2040: sharing a scan is a
/// property of the batch and not of the window's length, and a century per
/// chart would be most of this pass's price.
const ALONE_WINDOW: (f64, f64) = (2_458_849.5, 2_466_154.5);

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
        Ok(text) => {
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask sade-sati") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// What one reckoning's batch holds.
#[derive(Default)]
struct Tally {
    sade_sati: usize,
    phases_re_entered: usize,
    most_visits: usize,
    fourth: usize,
    eighth: usize,
    shortest_years: f64,
    longest_years: f64,
}

#[derive(Default)]
struct Measured {
    tallies: [Tally; 2],
    bounds: usize,
    bounds_off: usize,
    sade_satis: usize,
    out_of_order: usize,
    earlier: usize,
    earlier_wrong: usize,
    widest_shift_days: f64,
    whole: usize,
    whole_wrong: usize,
    alone: usize,
    alone_wrong: usize,
    longest_pause_days: f64,
    nearest_return_days: f64,
}

fn century() -> SadeSatiRequest {
    SadeSatiRequest::between(JulianDay::literal(FROM), JulianDay::literal(TO))
}

/// Every spell of a report, the Sade Satis' phases and the smaller ones.
fn spells_of(report: &Report) -> impl Iterator<Item = &Spell> {
    report
        .sade_sati
        .iter()
        .flat_map(|one| one.phases.iter())
        .chain(&report.spells)
}

/// The natal Moon's sidereal longitude.
fn moon_deg(document: &Document) -> Result<f64, String> {
    document
        .foundation
        .graha(Graha::Moon)
        .map(|moon| moon.longitude_deg)
        .ok_or_else(|| String::from("a founded chart places the Moon"))
}

/// Reads every bound of `report` back through the gochar a second either
/// side: inside, Saturn stands in the spell's house as the reckoning counts
/// it; outside, it does not.
fn read_back(
    sdk: &Context,
    birth: &Birth,
    report: &Report,
    out: &mut Measured,
) -> Result<(), String> {
    // The first house's start, reckoned here rather than by the SDK's
    // `origin_deg`, so a fault there is a fault the read-back sees: the
    // Moon's sign's start, or 15° before the Moon.
    let moon = moon_deg(&birth.document)?;
    let origin = match report.reckoning {
        Reckoning::Sign => moon - moon.rem_euclid(30.0),
        _ => moon - 15.0,
    };
    let mut probes = Vec::new();
    for spell in spells_of(report) {
        for visit in &spell.visits {
            for (bound, inward) in [(visit.from, SECOND), (visit.to, -SECOND)] {
                let Some(bound) = bound else {
                    return Err(format!(
                        "{}: a bound past the builtin ephemeris inside {FROM} to {TO}",
                        birth.name
                    ));
                };
                probes.push((spell.house, bound.get() + inward, true));
                probes.push((spell.house, bound.get() - inward, false));
            }
        }
    }
    let instants: Vec<JulianDay<Utc>> = probes
        .iter()
        .map(|(_, jd, _)| JulianDay::literal(*jd))
        .collect();
    let readings = sdk
        .chart()
        .gochar(&birth.document, &GocharRequest::over(instants))
        .map_err(|why| format!("{}: {why}", birth.name))?
        .value;
    for ((house, _, inside), reading) in probes.iter().zip(&readings) {
        let saturn = reading
            .grahas
            .get(Graha::Saturn as usize)
            .ok_or_else(|| String::from("a reading places Saturn"))?;
        let longitude = f64::from(saturn.transit.sign as u8) * 30.0 + saturn.transit.degrees;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a longitude over 30 is a house index 0 to 11"
        )]
        let stands = ((longitude - origin).rem_euclid(360.0) / 30.0) as u8 % 12 + 1;
        out.bounds += 1;
        if (stands == *house) != *inside {
            out.bounds_off += 1;
        }
    }
    Ok(())
}

fn years(one: &SadeSati) -> Option<f64> {
    Some((one.ends()?.get() - one.begins()?.get()) / 365.25)
}

/// The counts one reckoning's reports give, and the pauses and returns the
/// 270-day rule is measured against.
fn tally(reports: &[Report], tally: &mut Tally, out: &mut Measured) {
    tally.shortest_years = f64::INFINITY;
    for report in reports {
        for one in &report.sade_sati {
            tally.sade_sati += 1;
            out.sade_satis += 1;
            let houses: Vec<u8> = one.phases.iter().map(|spell| spell.house).collect();
            if houses != [12, 1, 2] {
                out.out_of_order += 1;
            }
            if let Some(length) = years(one) {
                tally.shortest_years = tally.shortest_years.min(length);
                tally.longest_years = tally.longest_years.max(length);
            }
        }
        for spell in spells_of(report) {
            if spell.phase().is_some() && spell.visits.len() > 1 {
                tally.phases_re_entered += 1;
            }
            tally.most_visits = tally.most_visits.max(spell.visits.len());
            match spell.house {
                4 => tally.fourth += 1,
                8 => tally.eighth += 1,
                _ => {}
            }
            for pair in spell.visits.windows(2) {
                if let [a, b] = pair
                    && let (Some(left), Some(back)) = (a.to, b.from)
                {
                    out.longest_pause_days = out.longest_pause_days.max(back.get() - left.get());
                }
            }
        }
        // Two periods of one house on successive circuits: the nearest they
        // come is the gap a merge rule must never close.
        let mut by_house: Vec<(u8, f64, f64)> = spells_of(report)
            .filter_map(|spell| Some((spell.house, spell.begins()?.get(), spell.ends()?.get())))
            .collect();
        by_house.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
        for pair in by_house.windows(2) {
            if let [a, b] = pair
                && a.0 == b.0
            {
                out.nearest_return_days = out.nearest_return_days.min(b.1 - a.2);
            }
        }
    }
}

/// The degree reckoning begins earlier than the sign reckoning exactly
/// when the natal Moon stands in the first half of its sign: its first line
/// is the Moon less 45°, the sign's is the sign's start less 30°.
fn earlier(
    born: &[Birth],
    by_sign: &[Report],
    by_degree: &[Report],
    out: &mut Measured,
) -> Result<(), String> {
    for ((birth, sign), degree) in born.iter().zip(by_sign).zip(by_degree) {
        let within = moon_deg(&birth.document)?.rem_euclid(30.0);
        for one in &sign.sade_sati {
            let Some(begins) = one.begins() else { continue };
            // The same Sade Sati under the other reckoning: the one whose
            // start is within a sign's passage.
            let Some(other) = degree
                .sade_sati
                .iter()
                .filter_map(SadeSati::begins)
                .find(|other| (other.get() - begins.get()).abs() < 1_000.0)
            else {
                continue;
            };
            out.earlier += 1;
            let shift = other.get() - begins.get();
            out.widest_shift_days = out.widest_shift_days.max(shift.abs());
            if (shift < 0.0) != (within < 15.0) {
                out.earlier_wrong += 1;
            }
        }
    }
    Ok(())
}

/// Asked at the middle of each phase of the first birth's, the period
/// comes back to the bit as the century's search found it.
fn whole(sdk: &Context, birth: &Birth, report: &Report, out: &mut Measured) -> Result<(), String> {
    for one in &report.sade_sati {
        for phase in Phase::ALL {
            let Some((from, to)) = one
                .phase(phase)
                .and_then(|spell| Some((spell.begins()?, spell.ends()?)))
            else {
                continue;
            };
            let mid = JulianDay::<Utc>::literal(f64::midpoint(from.get(), to.get()));
            let now = SadeSatiRequest::at(mid).reckoned(report.reckoning);
            let found = sdk
                .chart()
                .sade_sati(&birth.document, &now)
                .map_err(|why| format!("{}: {why}", birth.name))?
                .value;
            out.whole += 1;
            if found.sade_sati.as_slice() != std::slice::from_ref(one) {
                out.whole_wrong += 1;
            }
        }
    }
    Ok(())
}

fn measure(sdk: &Context, born: &[Birth]) -> Result<Measured, String> {
    let mut out = Measured {
        nearest_return_days: f64::INFINITY,
        ..Measured::default()
    };
    let documents: Vec<&Document> = born.iter().map(|birth| &birth.document).collect();
    let mut by_reckoning = Vec::with_capacity(Reckoning::ALL.len());
    for (index, reckoning) in Reckoning::ALL.iter().enumerate() {
        let asked = century().reckoned(*reckoning);
        let reports = sdk
            .chart()
            .sade_sati_many(documents.iter().copied(), &asked)
            .map_err(|why| format!("the batch, {reckoning:?}: {why}"))?
            .value;
        for (birth, report) in born.iter().zip(&reports) {
            read_back(sdk, birth, report, &mut out)?;
        }
        if let Some(tallied) = out.tallies.get_mut(index) {
            let mut counted = std::mem::take(tallied);
            tally(&reports, &mut counted, &mut out);
            if let Some(slot) = out.tallies.get_mut(index) {
                *slot = counted;
            }
        }
        if let (Some(first), Some(report)) = (born.first(), reports.first()) {
            whole(sdk, first, report, &mut out)?;
        }
        let shorter = SadeSatiRequest::between(
            JulianDay::literal(ALONE_WINDOW.0),
            JulianDay::literal(ALONE_WINDOW.1),
        )
        .reckoned(*reckoning);
        let few: Vec<&Document> = documents.iter().copied().take(ALONE).collect();
        let together = sdk
            .chart()
            .sade_sati_many(few.iter().copied(), &shorter)
            .map_err(|why| format!("the batch of {ALONE}, {reckoning:?}: {why}"))?
            .value;
        for (document, together) in few.iter().zip(&together) {
            let alone = sdk
                .chart()
                .sade_sati(document, &shorter)
                .map_err(|why| format!("{reckoning:?} alone: {why}"))?
                .value;
            out.alone += 1;
            if &alone != together {
                out.alone_wrong += 1;
            }
        }
        by_reckoning.push(reports);
    }
    if let [by_sign, by_degree] = by_reckoning.as_slice() {
        earlier(born, by_sign, by_degree, &mut out)?;
    }
    Ok(out)
}

/// What the pass decides, each claim counted over what it read.
fn claims(measured: &Measured) -> Vec<Claim> {
    let merge_holds = measured.longest_pause_days < BASELINE_MERGE_DAYS
        && measured.nearest_return_days > BASELINE_MERGE_DAYS;
    vec![
        Claim::counted(
            "every bound of every visit is where the gochar a second either side moves Saturn into or out of the visit's house",
            measured.bounds_off,
            measured.bounds,
        ),
        Claim::counted(
            "every Sade Sati is the rising, peak and setting phases, in that order",
            measured.out_of_order,
            measured.sade_satis,
        ),
        Claim::counted(
            "the degree reckoning begins earlier than the sign reckoning exactly when the natal Moon stands in the first half of its sign",
            measured.earlier_wrong,
            measured.earlier,
        ),
        Claim::counted(
            "a Sade Sati asked about at one instant inside it is, to the bit, the one a century's search finds",
            measured.whole_wrong,
            measured.whole,
        ),
        Claim::counted(
            "a batch of charts answers each as that chart alone",
            measured.alone_wrong,
            measured.alone,
        ),
        Claim::stated(
            "joining visits to one house under 270 days apart, the baseline engine's rule, groups them as the circuit does",
            verdict_of(merge_holds),
            format!(
                "the longest pause inside a period is {} days, and the nearest two periods of one house come {} days apart",
                days(measured.longest_pause_days),
                days(measured.nearest_return_days)
            ),
        ),
    ]
}

fn days(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.0}")
    } else {
        String::from("no")
    }
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = Context::builder()
        .settings_json(POLAR)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the default profile with {POLAR}: {why}"))?;
    let born = births(root, &sdk)?;
    let measured = measure(&sdk, &born)?;

    let mut out = String::new();
    let _ = writeln!(
        out,
        "# Sade Sati, measured\n\n\
         Status: `generated` by `cargo xtask sade-sati`. Do not edit:\n\
         `check-sade-sati` regenerates this page and fails on any difference.\n\n\
         The design this measures is `sade-sati.md`. **Nothing records a\n\
         Sade Sati** — no classical text names it (C147) and the corpus has\n\
         no transit search — so this page holds the periods to what the sky\n\
         says. All {} recorded births, founded under the default profile\n\
         (with a polar day reckoned from civil midnight, since Tromsø has no\n\
         June sunrise to start one),\n\
         are asked for every period of 1950 to 2050 in **one batch** for\n\
         each reckoning (`sdk.chart().sade_sati_many`), from the natal Moon,\n\
         with the 4th and the 8th as the smaller spells (C149). The profile\n\
         is geocentric, so every chart reads one scan of Saturn.\n",
        spelled(born.len())
    );
    let mut rows = String::from(
        "| reckoning | Sade Satis | phases re-entered | most visits to one house | 4th | 8th | shortest | longest |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    for (reckoning, tally) in Reckoning::ALL.iter().zip(&measured.tallies) {
        let _ = writeln!(
            rows,
            "| {reckoning:?} | {} | {} | {} | {} | {} | {:.2} years | {:.2} years |",
            count(tally.sade_sati),
            count(tally.phases_re_entered),
            tally.most_visits,
            count(tally.fourth),
            count(tally.eighth),
            tally.shortest_years,
            tally.longest_years
        );
    }
    let _ = writeln!(
        out,
        "## 1. What a century holds\n\n\
         Every period reaching into the century, whole, per reckoning. A\n\
         phase is **re-entered** when Saturn turned retrograde across its\n\
         edge and came back, so its spell is more than one visit; a Sade\n\
         Sati's length runs from its first entry into the 12th to its last\n\
         exit from the 2nd.\n\n{rows}\n\
         The two reckonings move a Sade Sati's start by up to {} days\n\
         either way: the degree reading's first line is the Moon less 45°,\n\
         the sign reading's the start of the sign before the Moon's, and the\n\
         two differ by as much as the Moon's distance from the middle of its\n\
         sign.\n",
        days(measured.widest_shift_days)
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
        "{} The read-back reads the gochar, a second consumer of the same\n\
         longitudes, rather than the search that found the bounds, and the\n\
         batch and window claims hold the answer to the bit rather than to a\n\
         tolerance: the search samples an anchored grid and unwraps each step\n\
         from its own sample (`astro-events-and-crossings.md` §4), so a\n\
         window's start cannot move a bound.",
        match falsified {
            0 => format!("None of the {} claims is falsified.", spelled(claims.len())),
            1 => format!("One of the {} claims is falsified.", spelled(claims.len())),
            _ => format!(
                "{} of the {} claims are falsified.",
                capitalised(&spelled(falsified)),
                spelled(claims.len())
            ),
        },
    );
    Ok(crate::measure::fill(&out))
}

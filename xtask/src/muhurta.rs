//! The measurement pass over **muhurta** (`03-design/muhurta.md` §5).
//!
//! Over the built-in ephemeris at Kathmandu, the place the baseline engine
//! and the published almanac both read:
//!
//! - the regression the baseline engine was rebuilt for, through the
//!   search under its own rules and ranking;
//! - Guru and Shukra asta against the windows the Nepal Panchanga
//!   Nirnayak Samiti published for BS 2083, per visibility criterion;
//! - every window of two searches held constant — the grahas' signs, the
//!   lagna's and the Moon's navamsas and the instant's clauses the same a
//!   moment inside either end, and every clause covering its window whole;
//! - where the engine's sunrise sampling parts from the SDK's windows, and
//!   how often Raman's panchaka remainder names the kind the star does
//!   (crux C159).
//!
//! `cargo xtask muhurta` writes the page; `check-muhurta` regenerates it in
//! memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::MuhurtaRequest;
use teistro_astro::Completion;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::visibility::{Criterion, Heliacal};
use teistro_calendar::solar::drik::DrikSun;
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay, Gregorian};
use teistro_chart::foundation::Founder;
use teistro_core::catalogue::{Ayanamsha, Calendar, Choghadiya};
use teistro_core::interval::Interval;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{OverridePolicy, Profile, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_muhurta::instant::clauses as instant_clauses;
use teistro_muhurta::season::{Blackout, BlackoutKind, asta_over};
use teistro_muhurta::sources::Over;
use teistro_muhurta::{
    ActivityRules, Answer, ClauseKind, Judgement, ProviderSources, Ranking, Request, Sources,
    search,
};
use teistro_panchanga::Almanac;
use teistro_panchanga::span::at as span_at;
use teistro_port_ephemeris::{Body, CountingProvider, Horizon};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, table};

const PAGE: &str = "docs/03-design/muhurta-measured.md";

/// Kathmandu, and its clock.
const LATITUDE: f64 = 27.7172;
const LONGITUDE: f64 = 85.324;
const ALTITUDE: f64 = 1400.0;
const OFFSET_DAYS: f64 = 5.75 / 24.0;

/// The regression's range, 2026-09-01 to 2026-11-30.
const FROM: (u8, u8) = (9, 1);
const TO: (u8, u8) = (11, 30);

/// Every open day of the range is cut, so every window is measured.
const DAYS: usize = 91;

/// The days the regression names: vetoed, and kept.
const VETOED: [(u8, u8); 3] = [(9, 21), (10, 19), (11, 5)];
const KEPT: (u8, u8) = (11, 25);
const DEVUTHANI: (u8, u8) = (11, 20);

/// The asta windows published for BS 2083, as local dates: Guru Asar 32
/// to Shrawan 24, Shukra Ashwin 28 to Kartik 11.
/// A published window: the body, its name, and its first and last
/// local dates.
type Published = (Body, &'static str, (u8, u8), (u8, u8));

const PUBLISHED: [Published; 2] = [
    (Body::Jupiter, "Guru", (7, 16), (8, 9)),
    (Body::Venus, "Shukra", (10, 14), (10, 28)),
];

/// The half-year the asta is searched over, 2026-06-01 to 2026-12-31.
const ASTA_RANGE: (f64, f64) = (2_461_192.5, 2_461_406.5);

/// The shortest window the engine offers, days: six minutes.
const ENGINE_SHORTEST_DAYS: f64 = 6.0 / 1440.0;

/// How far inside a window's ends it is read, days: a second, or a
/// quarter of a window shorter than four.
const SECOND: f64 = 1.0 / 86_400.0;

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask muhurta") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn date(month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, 2026, month, day)
}

/// An instant's local date at Kathmandu, as (month, day).
fn local(jd: JulianDay<Utc>) -> Result<(u8, u8), String> {
    let (fixed, _) = FixedDay::from_jd(JulianDay::literal(jd.get() + OFFSET_DAYS));
    let date = Gregorian.date_of(fixed).map_err(|e| e.to_string())?;
    Ok((date.month, date.day))
}

fn spell((month, day): (u8, u8)) -> String {
    format!("2026-{month:02}-{day:02}")
}

/// Kathmandu's clock.
fn clock() -> UtcOffset {
    UtcOffset::literal(5, 45, 0)
}

fn place() -> Place {
    Place::new(
        Latitude::literal(LATITUDE),
        Longitude::literal(LONGITUDE),
        Altitude::literal(ALTITUDE),
    )
}

/// What the page measures.
struct Measured {
    regression: Regression,
    asta: Vec<AstaRow>,
    constancy: Vec<Constancy>,
    partings: Partings,
    remainder: Remainder,
    shipping: Shipping,
}

fn page() -> Result<String, String> {
    // What ships first: the façade's answers, what they cost the provider,
    // and the instant the façade takes the chart zodiac at, which the
    // hand-wired sources below take too so the two are the same search.
    let (shipping, baseline_shipped, raman) = shipping()?;
    let zodiac_at = zodiac_at(&baseline_shipped.provenance)?;

    let provider = Builtin::new();
    let resolved = Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .ok_or("the default profile is not shipped")?
        .resolve(&SettingsPatch::default())
        .map_err(|e| e.to_string())?;
    let delta_t = DeltaTModel::TableThenModel;
    let precession = PrecessionModel::Vondrak2011;
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        delta_t,
    );
    let clock = clock();
    let almanac = Almanac::new(
        &provider, &resolved, &model, &Gregorian, &clock, precession, delta_t,
    );
    let founder = Founder::new(
        &provider, &resolved, &model, &Gregorian, &clock, precession, delta_t,
    );
    let completion = Completion::new(&provider, resolved.settings.provider.overrides, delta_t);
    let heliacal_of = |criterion| {
        Heliacal::new(
            &completion,
            place(),
            criterion,
            Horizon::CENTRE_NO_REFRACTION,
            delta_t,
        )
    };
    let heliacal = heliacal_of(Criterion::SURYA_SIDDHANTA);
    let over = Over {
        almanac: &almanac,
        founder: &founder,
        completion: &completion,
        heliacal: &heliacal,
        calendar: &Gregorian,
        clock: &clock,
        settings: &resolved.settings,
        precession,
        delta_t,
    };
    let sources = ProviderSources::new(&over, place(), zodiac_at).map_err(|e| e.to_string())?;

    let request = |rules: ActivityRules, ranking| Request {
        rules,
        from: date(FROM.0, FROM.1),
        to: date(TO.0, TO.1),
        native: None,
        ranking,
        days_with_windows: DAYS,
        most: usize::MAX,
    };
    let baseline_rules = ActivityRules::baseline_marriage();
    let baseline = search(
        &sources,
        &request(baseline_rules.clone(), Ranking::Baseline),
    )
    .map_err(|e| format!("the baseline search: {e}"))?;
    let raman_rules = ActivityRules::raman_marriage();
    let shipping = Shipping {
        baseline_agrees: baseline_shipped.value == baseline,
        ..shipping
    };

    let mut asta = Vec::new();
    for (name, criterion) in [
        ("Surya Siddhanta", Criterion::SURYA_SIDDHANTA),
        ("the combustion orbs", Criterion::COMBUSTION_ORB),
        ("Ptolemy", Criterion::PTOLEMY),
    ] {
        asta.extend(asta_rows(&heliacal_of(criterion), name)?);
    }
    let measured = Measured {
        regression: regression(&baseline)?,
        asta,
        constancy: vec![
            constancy(&sources, "the baseline's", &baseline_rules, &baseline)?,
            constancy(&sources, "Raman's", &raman_rules, &raman)?,
        ],
        partings: partings(&sources, &baseline)?,
        remainder: remainder(&sources, &raman)?,
        shipping,
    };
    Ok(render(&measured))
}

/// A search through the façade over the page's range, asked as the page
/// asks the crate.
fn asked(rules: ActivityRules, ranking: Ranking) -> MuhurtaRequest {
    MuhurtaRequest::new(rules)
        .ranked(ranking)
        .with_windows_on(DAYS)
        .at_most(usize::MAX)
}

/// The façade's two searches, and what the search and the almanac cost
/// the provider asked apart and together — each counted on a fresh
/// context, behind its cache, since that is what the provider is asked.
/// The baseline's agreement with the crate is the page's to fill.
fn shipping() -> Result<(Shipping, teistro::Envelope<Answer>, Answer), String> {
    // A context owns its provider, and the count is read after it is
    // dropped, so the counter lives as long as the process: this is a
    // generator run once.
    let counted: &'static CountingProvider<Builtin> =
        Box::leak(Box::new(CountingProvider::new(Builtin::new())));
    let context = || {
        teistro::Context::builder()
            .ephemeris([teistro::Ephemeris::Provider(Box::new(counted))])
            .build()
            .map_err(|e| e.to_string())
    };
    let (from, to) = (date(FROM.0, FROM.1), date(TO.0, TO.1));
    let raman = asked(ActivityRules::raman_marriage(), Ranking::Texts);
    let failed = |what: &'static str| move |e: teistro::Error| format!("{what}: {e}");

    counted.reset();
    let alone = context()?
        .almanac()
        .muhurta(&from, &to, &place(), clock(), &raman)
        .map_err(failed("the façade's search"))?;
    let search_alone = counted.calls().total();
    counted.reset();
    context()?
        .almanac()
        .of_each(&from, &to, &place(), clock())
        .map_err(failed("the façade's almanac"))?;
    let almanac_alone = counted.calls().total();
    counted.reset();
    let together = context()?
        .almanac()
        .muhurta_with_days(&from, &to, &place(), clock(), &raman)
        .map_err(failed("the façade's search with its days"))?;
    let both = counted.calls().total();

    let baseline = context()?
        .almanac()
        .muhurta(
            &from,
            &to,
            &place(),
            clock(),
            &asked(ActivityRules::baseline_marriage(), Ranking::Baseline),
        )
        .map_err(failed("the façade's baseline search"))?;
    let shipping = Shipping {
        baseline_agrees: false,
        together_agrees: together.answer == alone,
        search_alone,
        almanac_alone,
        both,
    };
    Ok((shipping, baseline, alone.value))
}

/// The instant the façade took the chart zodiac at, from the convention
/// its provenance names it by.
fn zodiac_at(provenance: &teistro::Provenance) -> Result<JulianDay<Utc>, String> {
    let applied = provenance
        .applied_conventions
        .iter()
        .find(|c| c.knob == "muhurta.zodiacAt")
        .ok_or("the façade's answer names no muhurta.zodiacAt")?;
    let jd: f64 = applied
        .value
        .parse()
        .map_err(|e| format!("muhurta.zodiacAt: {e}"))?;
    JulianDay::try_new(jd).map_err(|e| e.to_string())
}

/// What ships against the crate wired by hand, and what the provider is
/// asked for the search and the almanac apart and together.
struct Shipping {
    baseline_agrees: bool,
    together_agrees: bool,
    /// Provider calls of Raman's search asked alone.
    search_alone: u64,
    /// Provider calls of the almanac of the range asked alone.
    almanac_alone: u64,
    /// Provider calls of the two asked together.
    both: u64,
}

/// The regression through the search.
struct Regression {
    closed: Vec<(u8, u8)>,
    closed_by_chaturmas: usize,
    judged: usize,
    windows: usize,
    open: usize,
    offered: usize,
    scored_unoffered: usize,
    blacked_out: usize,
    best: Option<(String, u8)>,
}

fn regression(answer: &Answer) -> Result<Regression, String> {
    let open: Vec<&Judgement> = answer.windows.iter().filter(|w| w.open()).collect();
    let is_offered = |w: &Judgement| {
        w.clauses.iter().any(|c| {
            matches!(
                c.kind,
                ClauseKind::Choghadiya {
                    choghadiya: Choghadiya::Amrit | Choghadiya::Shubha | Choghadiya::Laabh
                }
            )
        })
    };
    let best = match open.first() {
        Some(w) => Some((
            spell(local(w.at.from)?),
            w.score.as_ref().map_or(0, |s| s.value),
        )),
        None => None,
    };
    Ok(Regression {
        closed: answer
            .closed
            .iter()
            .map(|c| (c.date.month, c.date.day))
            .collect(),
        closed_by_chaturmas: answer
            .closed
            .iter()
            .filter(|c| c.by.contains(&BlackoutKind::Chaturmas))
            .count(),
        judged: answer.days_judged,
        windows: answer.windows.len(),
        open: open.len(),
        offered: open.iter().filter(|w| is_offered(w)).count(),
        scored_unoffered: open
            .iter()
            .filter(|w| w.score.is_some() != is_offered(w))
            .count(),
        blacked_out: answer.windows_blacked_out,
        best,
    })
}

/// One asta window against the published one.
struct AstaRow {
    criterion: &'static str,
    body: &'static str,
    found: Option<((u8, u8), (u8, u8))>,
    published: ((u8, u8), (u8, u8)),
    early_days: i64,
    late_days: i64,
}

fn day_of_year((month, day): (u8, u8)) -> i64 {
    const BEFORE: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    BEFORE
        .get(usize::from(month.saturating_sub(1)))
        .copied()
        .unwrap_or(0)
        + i64::from(day)
}

fn asta_rows(
    heliacal: &Heliacal<'_, Builtin>,
    criterion: &'static str,
) -> Result<Vec<AstaRow>, String> {
    let range = Interval::literal(ASTA_RANGE.0, ASTA_RANGE.1);
    let mut rows = Vec::new();
    for (body, name, from, to) in PUBLISHED {
        let found: Vec<Blackout> = asta_over(heliacal, body, range).map_err(|e| e.to_string())?;
        // The window that holds the published one's middle.
        let middle = (day_of_year(from) + day_of_year(to)) / 2;
        let mut hit = None;
        for b in &found {
            let (start, end) = (local(b.at.from)?, local(b.at.to)?);
            if day_of_year(start) <= middle && middle <= day_of_year(end) {
                hit = Some((start, end));
            }
        }
        let (early_days, late_days) = hit.map_or((0, 0), |(start, end)| {
            (
                day_of_year(from) - day_of_year(start),
                day_of_year(end) - day_of_year(to),
            )
        });
        rows.push(AstaRow {
            criterion,
            body: name,
            found: hit,
            published: (from, to),
            early_days,
            late_days,
        });
    }
    Ok(rows)
}

/// Every window of one search, held constant.
struct Constancy {
    search: &'static str,
    windows: usize,
    unsteady: usize,
    partial_clauses: usize,
    clauses: usize,
    alike: usize,
    shortest_seconds: f64,
    under_six_minutes: usize,
}

fn constancy(
    sources: &dyn Sources,
    search: &'static str,
    rules: &ActivityRules,
    answer: &Answer,
) -> Result<Constancy, String> {
    let mut unsteady = 0;
    let mut partial_clauses = 0;
    let clauses = answer.windows.iter().map(|w| w.clauses.len()).sum();
    let mut shortest = f64::INFINITY;
    let mut under_six_minutes = 0;
    for w in &answer.windows {
        let length = w.at.days();
        shortest = shortest.min(length);
        if length < ENGINE_SHORTEST_DAYS {
            under_six_minutes += 1;
        }
        partial_clauses += w
            .clauses
            .iter()
            .filter(|c| c.at.from.get() > w.at.from.get() || c.at.to.get() < w.at.to.get())
            .count();
        let inset = SECOND.min(length / 4.0);
        let read = |at: f64| -> Result<_, String> {
            let sky = sources
                .sky_at(JulianDay::literal(at))
                .map_err(|e| e.to_string())?;
            let navamsa = |deg: f64| (deg / (30.0 / 9.0)).floor();
            let mut kinds: Vec<String> = instant_clauses(&sky, None, None, w.at)
                .into_iter()
                .chain(rules.instant_clauses(&sky, w.at))
                .map(|c| format!("{:?}", c.kind))
                .collect();
            kinds.sort();
            Ok((
                sky.grahas.map(|g| (g / 30.0).floor()),
                navamsa(sky.lagna_deg),
                navamsa(sky.grahas[1]),
                kinds,
            ))
        };
        if read(w.at.from.get() + inset)? != read(w.at.to.get() - inset)? {
            unsteady += 1;
        }
    }
    // Adjacent windows judged alike: a cut at which no clause the rules
    // read changed, such as a hora's edge.
    let mut by_time: Vec<&Judgement> = answer.windows.iter().collect();
    by_time.sort_by(|a, b| a.at.from.get().total_cmp(&b.at.from.get()));
    let kinds = |w: &Judgement| {
        let mut k: Vec<String> = w.clauses.iter().map(|c| format!("{:?}", c.kind)).collect();
        k.sort();
        k
    };
    let alike = by_time
        .windows(2)
        .filter(|p| {
            p[0].at.to.get().to_bits() == p[1].at.from.get().to_bits() && kinds(p[0]) == kinds(p[1])
        })
        .count();
    Ok(Constancy {
        search,
        windows: answer.windows.len(),
        unsteady,
        partial_clauses,
        clauses,
        alike,
        shortest_seconds: shortest * 86_400.0,
        under_six_minutes,
    })
}

/// Where the engine's sunrise sampling parts from the SDK's windows, over
/// the days the baseline search judged.
struct Partings {
    days: usize,
    star_kept_by_sdk_only: usize,
    offered_under_six_minutes: usize,
}

fn partings(sources: &dyn Sources, answer: &Answer) -> Result<Partings, String> {
    let stars = ActivityRules::baseline_marriage()
        .baseline
        .map(|b| b.stars)
        .unwrap_or_default();
    let closed: Vec<(u8, u8)> = answer
        .closed
        .iter()
        .map(|c| (c.date.month, c.date.day))
        .collect();
    let mut out = Partings {
        days: 0,
        star_kept_by_sdk_only: 0,
        offered_under_six_minutes: answer
            .windows
            .iter()
            .filter(|w| w.open() && w.score.is_some() && w.at.days() < ENGINE_SHORTEST_DAYS)
            .count(),
    };
    let mut on = date(FROM.0, FROM.1);
    loop {
        if !closed.contains(&(on.month, on.day)) {
            let day = sources.day(&on).map_err(|e| e.to_string())?;
            out.days += 1;
            let engine = span_at(&day.limbs.nakshatra, day.day.sunrise)
                .is_some_and(|s| stars.contains(&s.member));
            // The SDK keeps a day for a star when an allowed star runs in
            // it at all; its windows in the others are barred.
            let sdk = day
                .limbs
                .nakshatra
                .iter()
                .any(|s| stars.contains(&s.member) && s.inside.overlaps(day.window));
            // The engine's keeping implies the SDK's, so only this way
            // can part; the other is refused as a defect.
            if engine && !sdk {
                return Err(format!("{on}: the engine keeps a day the SDK drops"));
            }
            if sdk && !engine {
                out.star_kept_by_sdk_only += 1;
            }
        }
        if (on.month, on.day) == TO {
            break;
        }
        on = sources.next_date(&on).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// Raman's panchaka remainder against the kind the star names (C159),
/// over the windows of Raman's search that lie in a panchaka star.
struct Remainder {
    in_panchaka: usize,
    named: usize,
    same: usize,
}

fn remainder(sources: &dyn Sources, answer: &Answer) -> Result<Remainder, String> {
    let mut out = Remainder {
        in_panchaka: 0,
        named: 0,
        same: 0,
    };
    let mut days = std::collections::BTreeMap::new();
    for w in &answer.windows {
        let key = local(w.at.from)?;
        let panchanga = match days.entry(key) {
            std::collections::btree_map::Entry::Occupied(held) => held.into_mut(),
            std::collections::btree_map::Entry::Vacant(slot) => slot.insert(
                sources
                    .day(&date(key.0, key.1))
                    .map_err(|e| e.to_string())?,
            ),
        };
        let middle = JulianDay::literal(f64::midpoint(w.at.from.get(), w.at.to.get()));
        // The window may belong to the almanac day before its civil date.
        let star_kind = span_at(&panchanga.omens.panchaka, middle)
            .filter(|s| s.contains(middle))
            .map(|s| s.member);
        let Some(kind) = star_kind else { continue };
        out.in_panchaka += 1;
        if let Some(named) = w.clauses.iter().find_map(|c| match c.kind {
            ClauseKind::PanchakaRemainder { panchaka } => Some(panchaka),
            _ => None,
        }) {
            out.named += 1;
            if named == kind {
                out.same += 1;
            }
        }
    }
    Ok(out)
}

fn render(m: &Measured) -> String {
    let mut out = String::new();
    intro(&mut out);
    regression_section(&mut out, &m.regression);
    asta_section(&mut out, &m.asta);
    constancy_section(&mut out, &m.constancy);
    partings_section(&mut out, &m.partings);
    remainder_section(&mut out, &m.remainder);
    shipping_section(&mut out, &m.shipping);
    out
}

fn intro(out: &mut String) {
    let _ = writeln!(
        *out,
        "# Muhurta, measured\n\n\
         Status: `generated` by `cargo xtask muhurta`. Do not edit:\n\
         `check-muhurta` regenerates this page and fails on any difference.\n\n\
         The design this measures is `muhurta.md` (§5). Everything is read\n\
         over the built-in ephemeris at Kathmandu (27.7172° N, 85.324° E,\n\
         1400 m, UTC+5:45), under the default profile, the place both the\n\
         baseline engine and the published almanac read. The searches run\n\
         over {} to {} and cut **every** open day, so every window is\n\
         measured.\n",
        spell(FROM),
        spell(TO)
    );
}

fn regression_section(out: &mut String, r: &Regression) {
    let before = day_of_year(DEVUTHANI) - day_of_year(FROM);
    let before = usize::try_from(before).unwrap_or(0);
    let claims = vec![
        Claim::counted(
            "a marriage search under the baseline's rules vetoes 2026-09-21, 2026-10-19 and 2026-11-05",
            VETOED.iter().filter(|d| !r.closed.contains(d)).count(),
            VETOED.len(),
        ),
        Claim::counted(
            "nothing is open before Devuthani Ekadashi, 2026-11-20",
            before - r.closed.iter().filter(|d| **d < DEVUTHANI).count(),
            before,
        ),
        Claim::counted(
            "every day closed is closed by Chaturmas, and says so",
            r.closed.len() - r.closed_by_chaturmas,
            r.closed.len(),
        ),
        Claim::counted(
            "2026-11-25 is kept",
            usize::from(r.closed.contains(&KEPT)),
            1,
        ),
        Claim::counted(
            "an open window is scored exactly when the engine would offer it (Amrita, Shubha or Labha)",
            r.scored_unoffered,
            r.open,
        ),
    ];
    let _ = writeln!(
        *out,
        "## 1. The regression\n\n\
         The baseline engine's marriage (`ActivityRules::baseline_marriage`)\n\
         under its own ranking (`Ranking::Baseline`). {} days are closed and\n\
         {} judged; the judged days are cut into {} windows, {} of them open\n\
         and {} of those in a choghadiya the engine offers. {} windows fell\n\
         inside a blackout that did not cover their day, and were left out.\n\
         The best window opens on {}, scored {}.\n\n{}",
        count(r.closed.len()),
        count(r.judged),
        count(r.windows),
        count(r.open),
        count(r.offered),
        count(r.blacked_out),
        r.best.as_ref().map_or("no day", |(d, _)| d.as_str()),
        r.best.as_ref().map_or(0, |(_, v)| *v),
        table(&claims)
    );
}

fn asta_section(out: &mut String, found: &[AstaRow]) {
    let mut rows = String::from(
        "| criterion | asta | found | published | begins earlier by | ends later by |\n\
         |---|---|---|---|---:|---:|\n",
    );
    let mut asta_claims = Vec::new();
    for row in found {
        let found = row.found.map_or_else(
            || String::from("none"),
            |(a, b)| format!("{} to {}", spell(a), spell(b)),
        );
        let _ = writeln!(
            rows,
            "| {} | {} | {} | {} to {} | {} | {} |",
            row.criterion,
            row.body,
            found,
            spell(row.published.0),
            spell(row.published.1),
            signed_days(row.early_days),
            signed_days(row.late_days)
        );
    }
    for criterion in ["Surya Siddhanta", "the combustion orbs", "Ptolemy"] {
        let of: Vec<&AstaRow> = found.iter().filter(|r| r.criterion == criterion).collect();
        asta_claims.push(Claim::counted(
            format!("under {criterion}, each asta holds the published window whole"),
            of.iter()
                .filter(|r| r.found.is_none() || r.early_days < 0 || r.late_days < 0)
                .count(),
            of.len(),
        ));
    }
    let _ = writeln!(
        *out,
        "## 2. The asta against the published almanac\n\n\
         The Nepal Panchanga Nirnayak Samiti's BS 2083 windows, which an\n\
         Indian almanac published independently: Guru asta Asar 32 to\n\
         Shrawan 24, Shukra asta Ashwin 28 to Kartik 11. Each is set\n\
         against the SDK's asta at Kathmandu, from the last sighting's\n\
         evening or morning to the next first one, by local date. A veto\n\
         a day wider than the almanac blocks a day nobody would use; one a\n\
         day narrower offers a day the country treats as closed, so\n\
         **holding the published window whole** is the property claimed. A\n\
         negative count is a window that begins later or ends earlier than\n\
         the almanac's.\n\n\
         {rows}\n{}",
        table(&asta_claims)
    );
}

fn constancy_section(out: &mut String, measured: &[Constancy]) {
    let mut rows = String::from(
        "| search | windows | unsteady | clauses over part of a window | neighbours judged alike | shortest | under six minutes |\n\
         |---|---:|---:|---:|---:|---:|---:|\n",
    );
    let mut steady_claims = Vec::new();
    for c in measured {
        let _ = writeln!(
            rows,
            "| {} | {} | {} | {} | {} | {:.2} s | {} |",
            c.search,
            count(c.windows),
            c.unsteady,
            c.partial_clauses,
            count(c.alike),
            c.shortest_seconds,
            count(c.under_six_minutes)
        );
        steady_claims.push(Claim::counted(
            format!(
                "every window of {} search reads the same sky a moment inside either end",
                c.search
            ),
            c.unsteady,
            c.windows,
        ));
        steady_claims.push(Claim::counted(
            format!(
                "every clause of {} search's windows covers its window whole",
                c.search
            ),
            c.partial_clauses,
            c.clauses,
        ));
    }
    let _ = writeln!(
        *out,
        "## 3. A window is constant\n\n\
         A window is cut wherever a clause can change, so what is judged at\n\
         its middle holds over all of it. Each window is read a second\n\
         inside either end (a quarter of the way in when it is shorter than\n\
         four): the nine grahas' signs, the lagna's and the Moon's navamsas,\n\
         and every instant clause the rules read. Windows are as short as the\n\
         sky makes them — two cuts a second apart are two real changes — and\n\
         the engine, which offers nothing under six minutes, would not offer\n\
         the shortest. Neighbours **judged alike** meet at a cut no clause\n\
         the rules read changed at — a hora's edge, a graha's ingress into a\n\
         sign nothing weighs — so the answer holds more windows than\n\
         judgements; they are counted rather than merged, because which cuts\n\
         matter is the rules' to say.\n\n{rows}\n{}",
        table(&steady_claims)
    );
}

fn partings_section(out: &mut String, p: &Partings) {
    let _ = writeln!(
        *out,
        "## 4. Where the engine's sampling parts from the SDK\n\n\
         The engine keeps or drops a whole day by the Moon's star **at\n\
         sunrise**; the SDK bars the windows in a star the rite does not\n\
         take and keeps the rest. A day the engine keeps the SDK keeps too,\n\
         since the star at sunrise runs in the day; the parting is one way.\n\
         Of the {} the baseline search judged, the engine drops {} that the\n\
         SDK keeps for an allowed star rising after sunrise. And {} open\n\
         windows the SDK scores are shorter than the six minutes under which\n\
         the engine drops a fragment.\n",
        spelled_days(p.days),
        spelled_days(p.star_kept_by_sdk_only),
        count(p.offered_under_six_minutes)
    );
}

fn remainder_section(out: &mut String, q: &Remainder) {
    let _ = writeln!(
        *out,
        "## 5. The panchaka remainder against the star (C159)\n\n\
         Raman makes a panchaka's kind the remainder by nine of the tithi,\n\
         vara, star and lagna numbers; the baseline engine makes it one kind\n\
         per star of the last five. Of Raman's search's windows, {} lie in a\n\
         panchaka star; the remainder names a panchaka in {} of them, and\n\
         the star's kind in {}. The remainder is the SDK's reading and this\n\
         is how far the engine's would part from it.\n\n\
         C158, where panchaka begins, is not counted: it is closed at rank 1\n\
         (Dhanishtha's latter half), and the rival start parts from it at\n\
         every panchaka by construction, so a count would be the number of\n\
         lunar months.",
        count(q.in_panchaka),
        count(q.named),
        count(q.same)
    );
}

fn shipping_section(out: &mut String, s: &Shipping) {
    let claims = vec![
        Claim::counted(
            "the façade answers the baseline's search as the crate wired by hand does",
            usize::from(!s.baseline_agrees),
            1,
        ),
        Claim::counted(
            "asked beside its days, Raman's search answers as it does alone",
            usize::from(!s.together_agrees),
            1,
        ),
    ];
    let apart = s.search_alone + s.almanac_alone;
    let spared = apart.saturating_sub(s.both);
    let share = |part: u64, of: u64| {
        let as_f64 = |n: u64| f64::from(u32::try_from(n).unwrap_or(u32::MAX));
        if of == 0 {
            0.0
        } else {
            100.0 * as_f64(part) / as_f64(of)
        }
    };
    let _ = writeln!(
        *out,
        "\n## 6. What ships\n\n\
         The searches above are the façade's (`sdk.almanac().muhurta`),\n\
         read back through the muhurta crate wired by hand, because the\n\
         re-reads need its sources. The façade takes the chart zodiac at the\n\
         range's middle and names that instant in its provenance, and the\n\
         hand-wired sources take the instant it names, so the two are one\n\
         search and are held to one answer.\n\n\
         A consumer electing a time shows the almanac beside the windows.\n\
         `muhurta_with_days` founds the range once and serves the search\n\
         its days. Counted in calls that reach the provider behind a fresh\n\
         context's cache, over Raman's search:\n\n\
         | asked | provider calls |\n|---|---:|\n\
         | the search alone | {} |\n\
         | the almanac alone | {} |\n\
         | the two apart | {} |\n\
         | the two together | {} |\n\n\
         Together they are spared {} calls, {:.1}% of the two apart: the\n\
         almanac beside the search adds {:.1}% to the search's own.\n\n{}",
        count_u64(s.search_alone),
        count_u64(s.almanac_alone),
        count_u64(apart),
        count_u64(s.both),
        count_u64(spared),
        share(spared, apart),
        share(s.both.saturating_sub(s.search_alone), s.search_alone),
        table(&claims)
    );
}

fn count_u64(n: u64) -> String {
    count(usize::try_from(n).unwrap_or(usize::MAX))
}

fn spelled_days(n: usize) -> String {
    crate::measure::plural(n, "day")
}

fn signed_days(n: i64) -> String {
    if n.abs() == 1 {
        format!("{n} day")
    } else {
        format!("{n} days")
    }
}

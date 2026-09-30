//! The measurement pass over **KP** (`03-design/kp.md` §6 step 7).
//!
//! Nothing in the corpus records a KP reading, and the Readers' worked
//! examples are already the crate's tests. What no example can say is how
//! far a reading can be trusted, and KP stakes everything on the sub lord,
//! so this pass measures that over the recorded births:
//!
//! - each cusp's and the Moon's **sub-lord margin in minutes of birth
//!   time**: the arc to the nearer edge of the sub, over the point's own
//!   rate, found by founding the chart half a minute either side;
//! - the minutes **read back** through the founder: moved 0.9 of the way to
//!   the lagna's nearer edge, its sub lord stands, and 1.1 of the way it
//!   has changed;
//! - how many lords the **Krishnamurti and VP291** ayanamshas part on, the
//!   two C157 accepts;
//! - how many of the **249 numbers** each place can raise (C156), which the
//!   geometry settles number by number: a start is refused exactly when
//!   its declination keeps it from ever crossing the horizon;
//! - how many charts' cusps a **polar stand-in** gave.
//!
//! `cargo xtask kp` writes the page; `check-kp` regenerates it in memory
//! and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{Graha, HouseSystem};
use teistro::kp::{Lords, Span};
use teistro::quantity::{JulianDay, Tt, Utc};
use teistro::{Context, Document, Ephemeris, KpChart, KpNumber, KpRequest, Nas};

use crate::births::{Birth, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, capitalised, count, spelled, table};

const PAGE: &str = "docs/03-design/kp-measured.md";

/// The patch over KP's own profile: a polar place's summer has no sunrise
/// to start the day from (Tromsø's June), so civil midnight starts it. A
/// day's bounds are no part of a cusp.
const POLAR: &str = r#"{"day":{"polar_day_policy":"CIVIL_MIDNIGHT"}}"#;

/// The same, under the VP291 variant of Krishnamurti's ayanamsha.
const VP291: &str = r#"{"day":{"polar_day_policy":"CIVIL_MIDNIGHT"},"frame":{"ayanamsha":{"kind":"CATALOGUED","id":"KRISHNAMURTI_VP291"}}}"#;

/// A minute, in days.
const MINUTE: f64 = 1.0 / 1440.0;

/// The largest the obliquity of the ecliptic has been or will be in the
/// corpus's centuries, degrees, rounded up: every ecliptic point rises at
/// a latitude under 90° less it.
const OBLIQUITY_MAX_DEG: f64 = 23.45;

/// The birth-time errors a table row counts the margins within, minutes.
const WITHIN_MINUTES: [f64; 3] = [1.0, 2.0, 5.0];

/// The levels whose lords the two ayanamshas are compared on.
const LEVELS: [&str; 3] = ["star", "sub", "sub-sub"];

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask kp") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// How long a point stays inside one of its spans, either way in time.
#[derive(Clone, Copy)]
struct Margin {
    /// Minutes until the nearer edge, whichever way the birth time errs.
    minutes: f64,
    /// Which way that edge lies: `1.0` later, `-1.0` earlier.
    toward: f64,
}

impl Margin {
    /// A point at `at` moving `rate` nanoarcseconds a minute, inside `span`.
    #[allow(clippy::cast_precision_loss, reason = "arcs below 2^53")]
    fn of(span: Span, at: Nas, rate: f64) -> Margin {
        let ahead = at.arc_to(span.end).get() as f64 / rate.abs();
        let behind = span.start.arc_to(at).get() as f64 / rate.abs();
        // A point moving backwards meets its span's start going forwards
        // in time.
        let (later, earlier) = if rate >= 0.0 {
            (ahead, behind)
        } else {
            (behind, ahead)
        };
        if later <= earlier {
            Margin {
                minutes: later,
                toward: 1.0,
            }
        } else {
            Margin {
                minutes: earlier,
                toward: -1.0,
            }
        }
    }
}

/// What one birth measures.
struct Measured {
    name: String,
    latitude_deg: f64,
    /// Each cusp's sub-lord margin, the lagna's first.
    cusp_subs: [Margin; 12],
    /// The lagna's sub-sub-lord margin.
    lagna_sub_sub: Margin,
    /// The Moon's sub-lord margin.
    moon_sub: Margin,
    /// Whether the chart's cusps came from a stand-in for the system asked.
    stood_in: bool,
    /// The lagna's sub lord at 0.9 and 1.1 of the way to its nearer edge.
    read_back: [Graha; 2],
    lagna_sub_lord: Graha,
    /// VP291's sidereal longitudes less Krishnamurti's, arcseconds.
    vp291_arcsec: f64,
    /// How many points' lords VP291 moves, per level of [`LEVELS`].
    vp291_parted: [usize; 3],
    /// How many points were compared.
    points: usize,
    /// The numbers the place cannot raise, and those raised whose lagna is
    /// not the number's start to the nanoarcsecond.
    horary_refused: usize,
    horary_off: usize,
    /// The numbers refused that the sky would raise, or raised that it
    /// would not.
    horary_misjudged: usize,
}

/// Whether a number's start never crosses the horizon at a latitude: its
/// declination is at least the colatitude, so it circles without rising
/// or setting. The geometry alone, independent of the search.
#[allow(clippy::cast_precision_loss, reason = "arcs below 2^53")]
fn never_crosses(
    document: &Document,
    number: KpNumber,
    latitude_deg: f64,
    obliquity_deg: f64,
) -> bool {
    let start_deg = number.start().get() as f64 / Nas::PER_DEGREE as f64;
    let tropical = document
        .foundation
        .zodiac
        .to_tropical(start_deg)
        .to_radians();
    let declination = (obliquity_deg.to_radians().sin() * tropical.sin())
        .asin()
        .to_degrees();
    declination.abs() >= 90.0 - latitude_deg.abs()
}

fn kp_of(sdk: &Context, document: &Document) -> Result<KpChart, String> {
    sdk.chart()
        .kp(document, &KpRequest::new())
        .map_err(|why| format!("reading it as KP: {why}"))
}

/// The same birth founded `days` later, read as KP.
fn kp_at(sdk: &Context, birth: &Birth, days: f64) -> Result<KpChart, String> {
    let moved = sdk
        .chart()
        .reading(
            JulianDay::<Utc>::literal(birth.at() + days),
            &birth.request(),
        )
        .map_err(|why| format!("founding it {days} days on: {why}"))?
        .value;
    kp_of(sdk, &moved)
}

/// Each longitude of `points` taken from `before` to `after`, a minute
/// apart, as nanoarcseconds a minute.
#[allow(clippy::cast_precision_loss, reason = "arcs below 2^53")]
fn rate(before: Nas, after: Nas) -> f64 {
    before.signed_difference(after) as f64
}

fn moon(chart: &KpChart) -> Result<(Nas, Lords), String> {
    chart
        .planet(Graha::Moon)
        .map(|moon| (moon.longitude, moon.lords))
        .ok_or_else(|| String::from("a KP chart places the Moon"))
}

/// Every point's lords, cusps then planets.
fn every_lords(chart: &KpChart) -> Vec<Lords> {
    chart
        .cusps
        .iter()
        .map(|cusp| cusp.lords)
        .chain(chart.planets.iter().map(|planet| planet.lords))
        .collect()
}

fn measure(
    sdk: &Context,
    vp291: &Context,
    birth: &Birth,
    other: &Birth,
) -> Result<Measured, String> {
    let name = &birth.name;
    let at = |why: String| format!("{name}: {why}");
    let chart = kp_of(sdk, &birth.document).map_err(at)?;
    let before = kp_at(sdk, birth, -MINUTE / 2.0).map_err(at)?;
    let after = kp_at(sdk, birth, MINUTE / 2.0).map_err(at)?;

    let cusp_subs: [Margin; 12] = std::array::from_fn(|house| {
        let (cusp, early, late) = (
            &chart.cusps[house],
            &before.cusps[house],
            &after.cusps[house],
        );
        Margin::of(
            cusp.lords.sub.span,
            cusp.longitude,
            rate(early.longitude, late.longitude),
        )
    });
    let [lagna, ..] = &chart.cusps;
    let lagna_rate = rate(before.cusps[0].longitude, after.cusps[0].longitude);
    let lagna_sub_sub = Margin::of(lagna.lords.sub_sub.span, lagna.longitude, lagna_rate);
    let (moon_at, moon_lords) = moon(&chart).map_err(at)?;
    let moon_rate = rate(moon(&before).map_err(at)?.0, moon(&after).map_err(at)?.0);
    let moon_sub = Margin::of(moon_lords.sub.span, moon_at, moon_rate);

    let [lagna_sub, ..] = cusp_subs;
    let read_back = [0.9, 1.1].map(|share| {
        kp_at(
            sdk,
            birth,
            lagna_sub.toward * share * lagna_sub.minutes * MINUTE,
        )
        .map(|moved| moved.cusps[0].lords.sub.lord)
    });
    let [within, beyond] = read_back;
    let read_back = [within.map_err(at)?, beyond.map_err(at)?];

    let under_vp291 = kp_of(vp291, &other.document).map_err(at)?;
    let (vp291_moon, _) = moon(&under_vp291).map_err(at)?;
    #[allow(clippy::cast_precision_loss, reason = "arcs below 2^53")]
    let vp291_arcsec = moon_at.signed_difference(vp291_moon) as f64 / Nas::PER_ARCSECOND as f64;
    let (ours, theirs) = (every_lords(&chart), every_lords(&under_vp291));
    let mut vp291_parted = [0; 3];
    for (a, b) in ours.iter().zip(&theirs) {
        let pairs = [
            (a.star.lord, b.star.lord),
            (a.sub.lord, b.sub.lord),
            (a.sub_sub.lord, b.sub_sub.lord),
        ];
        for (parted, (x, y)) in vp291_parted.iter_mut().zip(pairs) {
            *parted += usize::from(x != y);
        }
    }

    let (mut horary_refused, mut horary_off, mut horary_misjudged) = (0, 0, 0);
    let latitude_deg = birth.document.foundation.place.latitude.get();
    // The obliquity at UT for TT: a minute moves it by a microarcsecond.
    let obliquity_deg =
        teistro_astro::sky::obliquity(JulianDay::<Tt>::literal(birth.at())).true_deg;
    for number in KpNumber::all() {
        let refused = match sdk
            .chart()
            .kp_horary(&birth.document, number, &KpRequest::new())
        {
            Ok(horary) => {
                horary_off += usize::from(horary.cusps[0].longitude != number.start());
                false
            }
            Err(why) if why.field() == Some("place.latitude") => true,
            Err(why) => return Err(at(format!("number {}: {why}", number.get()))),
        };
        horary_refused += usize::from(refused);
        horary_misjudged += usize::from(
            refused != never_crosses(&birth.document, number, latitude_deg, obliquity_deg),
        );
    }

    Ok(Measured {
        name: name.clone(),
        latitude_deg: birth.document.foundation.place.latitude.get(),
        cusp_subs,
        lagna_sub_sub,
        moon_sub,
        stood_in: chart.system != HouseSystem::Placidus,
        read_back,
        lagna_sub_lord: lagna.lords.sub.lord,
        vp291_arcsec,
        vp291_parted,
        points: ours.len().min(theirs.len()),
        horary_refused,
        horary_off,
        horary_misjudged,
    })
}

/// The middle value, the mean of the two middle ones for an even count.
fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let n = values.len();
    match n {
        0 => f64::NAN,
        _ if n % 2 == 1 => values[n / 2],
        _ => f64::midpoint(values[n / 2 - 1], values[n / 2]),
    }
}

/// One row of the margin table: the median and shortest in minutes, and how
/// many fall within each of [`WITHIN_MINUTES`].
fn margin_row(label: &str, minutes: &[f64]) -> String {
    let shortest = minutes.iter().copied().fold(f64::INFINITY, f64::min);
    let within = WITHIN_MINUTES.map(|limit| {
        let inside = minutes.iter().filter(|m| **m < limit).count();
        format!("{} of {}", count(inside), count(minutes.len()))
    });
    format!(
        "| {label} | {:.2} | {:.2} | {} |",
        median(minutes.to_vec()),
        shortest,
        within.join(" | ")
    )
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = Context::builder()
        .profile("kp-default")
        .settings_json(POLAR)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("kp-default with {POLAR}: {why}"))?;
    let vp291 = Context::builder()
        .profile("kp-default")
        .settings_json(VP291)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("kp-default with {VP291}: {why}"))?;
    let born = births(root, &sdk)?;
    let born_vp291 = births(root, &vp291)?;
    let measured = born
        .iter()
        .zip(&born_vp291)
        .map(|(birth, other)| measure(&sdk, &vp291, birth, other))
        .collect::<Result<Vec<_>, _>>()?;

    let mut out = String::new();
    let _ = writeln!(
        out,
        "# KP, measured\n\n\
         Status: `generated` by `cargo xtask kp`. Do not edit: `check-kp`\n\
         regenerates this page and fails on any difference.\n\n\
         The design this measures is `kp.md`. The corpus records no KP\n\
         reading, and the Readers' worked examples are the crate's own\n\
         tests; what no example says is **how far a reading can be\n\
         trusted**, and KP stakes everything on the sub lord. All {}\n\
         recorded births are founded under `kp-default` (Krishnamurti's\n\
         ayanamsha, Placidus, the true node), with a polar day reckoned\n\
         from civil midnight since Tromsø has no June sunrise to start one.\n",
        spelled(born.len())
    );
    margins(&mut out, &measured);
    vp291_parts(&mut out, &measured);
    horary(&mut out, &measured);
    decided(&mut out, &measured);
    Ok(crate::measure::fill(&out))
}

/// §1: each sub lord's margin in minutes of birth time.
fn margins(out: &mut String, measured: &[Measured]) {
    let lagna: Vec<f64> = measured.iter().map(|m| m.cusp_subs[0].minutes).collect();
    let cusps: Vec<f64> = measured
        .iter()
        .flat_map(|m| m.cusp_subs.iter().map(|margin| margin.minutes))
        .collect();
    let limits = WITHIN_MINUTES.map(|limit| format!("under {limit:.0} min"));
    let _ = writeln!(
        out,
        "## 1. How long a sub lord holds\n\n\
         A point's sub lord changes when the point crosses an edge of its\n\
         sub. Each margin is the arc to the nearer edge over the point's own\n\
         rate, taken from the chart founded half a minute either side of the\n\
         birth, so it reads as **the birth-time error that changes the\n\
         lord**, whichever way the time errs.\n\n\
         | sub lord of | median, minutes | shortest, minutes | {} |\n\
         |---|---:|---:|---:|---:|---:|\n\
         {}\n{}\n\n\
         One birth in {} would read its lagna's sub lord differently if\n\
         its time were two minutes out. The lagna's **sub-sub** lord holds\n\
         a median of {:.0} seconds, which no recorded time settles. The\n\
         Moon's sub lord holds a median of {:.1} hours.\n",
        limits.join(" | "),
        margin_row("the lagna", &lagna),
        margin_row("any cusp", &cusps),
        one_in(lagna.iter().filter(|m| **m < 2.0).count(), lagna.len()),
        median(
            measured
                .iter()
                .map(|m| m.lagna_sub_sub.minutes * 60.0)
                .collect()
        ),
        median(measured.iter().map(|m| m.moon_sub.minutes / 60.0).collect()),
    );
}

/// §2: the lords Krishnamurti's and VP291's ayanamshas part on.
fn vp291_parts(out: &mut String, measured: &[Measured]) {
    let points: usize = measured.iter().map(|m| m.points).sum();
    let parted: Vec<usize> = (0..LEVELS.len())
        .map(|level| measured.iter().map(|m| m.vp291_parted[level]).sum())
        .collect();
    let apart = measured.iter().map(|m| m.vp291_arcsec);
    let (least, most) = apart.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
        (lo.min(v), hi.max(v))
    });
    let mut rows = String::new();
    for (level, n) in LEVELS.iter().zip(&parted) {
        let _ = writeln!(rows, "| {level} | {} of {} |", count(*n), count(points));
    }
    let _ = writeln!(
        out,
        "## 2. Krishnamurti against VP291\n\n\
         C157 reads a chart under either of the two ayanamshas KP names.\n\
         Over these births VP291's sidereal longitudes lie {least:.1}″ to {most:.1}″\n\
         from Krishnamurti's, the same for every point of a chart since the\n\
         tropical sky is one. The lords they part on, over every cusp and\n\
         planet:\n\n\
         | level | lords that differ |\n|---|---:|\n{rows}"
    );
}

/// §3: the horary numbers each place cannot raise, and the stand-ins.
fn horary(out: &mut String, measured: &[Measured]) {
    let polar_limit = 90.0 - OBLIQUITY_MAX_DEG;
    let refused: Vec<&Measured> = measured.iter().filter(|m| m.horary_refused > 0).collect();
    let stood_in: Vec<&str> = measured
        .iter()
        .filter(|m| m.stood_in)
        .map(|m| m.name.as_str())
        .collect();
    let mut rows = String::new();
    for m in &refused {
        let _ = writeln!(
            rows,
            "| {} | {:.2}° | {} of 249 |",
            m.name,
            m.latitude_deg,
            count(m.horary_refused)
        );
    }
    let _ = writeln!(
        out,
        "## 3. Horary numbers a place cannot raise\n\n\
         A number's chart puts its start on the ascendant (C156), which a\n\
         place can do only where that ecliptic point rises: everywhere under\n\
         {polar_limit:.2}° of latitude, and not every point beyond it. Each\n\
         birth's place is asked for all 249 numbers at its own moment.\n\n\
         {}\n\
         {} of the charts' cusps came from a stand-in for Placidus, which\n\
         the profile's polar policy puts in where Placidus has no answer{}.\n",
        if refused.is_empty() {
            String::from("Every place raises every number.\n")
        } else {
            format!("| birth | latitude | numbers refused |\n|---|---:|---:|\n{rows}")
        },
        capitalised(&spelled(stood_in.len())),
        if stood_in.is_empty() {
            String::new()
        } else {
            format!(" ({})", stood_in.join(", "))
        },
    );
}

/// §4: the claims.
fn decided(out: &mut String, measured: &[Measured]) {
    let claims = claims(measured);
    let _ = writeln!(out, "## 4. What this pass decides\n");
    let _ = writeln!(out, "{}", table(&claims));
    let falsified = claims
        .iter()
        .filter(|claim| claim.verdict == Verdict::Falsified)
        .count();
    let _ = writeln!(
        out,
        "{} The read-back founds the chart again rather than trusting the\n\
         rate, so a margin that assumed a steady lagna where it is not\n\
         steady would fail it.",
        match falsified {
            0 => format!("None of the {} claims is falsified.", spelled(claims.len())),
            1 => format!("One of the {} claims is falsified.", spelled(claims.len())),
            _ => format!(
                "{} of the {} claims are falsified.",
                capitalised(&spelled(falsified)),
                spelled(claims.len())
            ),
        }
    );
}

/// "one birth in N", rounded, for a share that is not zero.
fn one_in(part: usize, whole: usize) -> String {
    if part == 0 {
        return String::from("none");
    }
    #[allow(clippy::cast_precision_loss, reason = "counts of births")]
    let every = (whole as f64 / part as f64).round();
    format!("{every:.0}")
}

fn claims(measured: &[Measured]) -> Vec<Claim> {
    let read_back_wrong = measured
        .iter()
        .filter(|m| m.read_back[0] != m.lagna_sub_lord || m.read_back[1] == m.lagna_sub_lord)
        .count();
    let raised: usize = measured.iter().map(|m| 249 - m.horary_refused).sum();
    let off: usize = measured.iter().map(|m| m.horary_off).sum();
    let misjudged: usize = measured.iter().map(|m| m.horary_misjudged).sum();
    vec![
        Claim::counted(
            "the lagna's sub lord stands 0.9 of the way to its nearer edge and has changed 1.1 of the way, founded again each time",
            read_back_wrong,
            measured.len(),
        ),
        Claim::counted(
            "a place refuses a number exactly when the number's start never crosses its horizon, judged by the start's declination",
            misjudged,
            measured.len() * 249,
        ),
        Claim::counted(
            "every horary chart raised has the number's start as its lagna, to the nanoarcsecond",
            off,
            raised,
        ),
    ]
}

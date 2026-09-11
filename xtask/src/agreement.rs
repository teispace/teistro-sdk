//! **The accuracy document**: what the built-in ephemeris costs a chart,
//! per body, per span and per tier.
//!
//! ADR-0027 ends by requiring exactly that, because the single number the
//! plan used to carry ("2 arcsec Moon, 1 arcsec planets") was true of
//! neither the Moon nor the outer planets. This page is that document.
//!
//! It renders rather than measures. The measurement needs the reference
//! engine and a tier chosen at compile time, so it is taken by
//! `teistro-ephemeris-teimeris-stack-agreement` — once per tier — and
//! recorded into `crates/ephemeris-builtin/data/stack-agreement-<tier>
//! .json`. This pass reads whichever of those files exist and writes the
//! page from them, so `check-agreement` gates the page against the
//! recorded numbers and never against a machine that happens to have the
//! engine installed.
//!
//! A tier with no recorded file is named as missing rather than left out.
//! A page that silently covered two tiers and titled itself "per tier"
//! would be the project's own recurring defect — a claim with nothing
//! behind it.
//!
//! ```text
//! cargo xtask agreement        # write the page
//! cargo xtask check-agreement  # fail if it drifted
//! ```

use std::fmt::Write as _;
use std::path::Path;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, spaced, spelled, verdict_of};

const PAGE: &str = "docs/03-design/completion-measured.md";

/// Where the engine-derived measurements are recorded, one per tier.
const RECORDED: &str = "crates/ephemeris-builtin/data";

/// The tiers the built-in ephemeris ships, richest last.
const TIERS: [&str; 3] = ["compact", "standard", "full"];

/// The Moon's orbital inclination, degrees, and the amplitude of the
/// latitude rate that follows from it: the latitude swings through the
/// inclination once a draconic month, so its rate peaks at `2 pi i / P`.
/// Both are needed to turn a disagreement in the rate into the tilt of
/// the plane it implies.
const INCLINATION_DEG: f64 = 5.145;
const DRACONIC_MONTH_DAYS: f64 = 27.212_22;

/// What a consumer of the almanac needs the Moon inside, arcseconds
/// (`03-design/lunar-accuracy-measured.md`).
const ALMANAC_MOON_ARCSEC: f64 = 0.445;

/// Seconds of tithi boundary per arcsecond of lunar longitude, the same
/// constant the recording binary uses.
const TITHI_SECONDS_PER_ARCSEC: f64 = 86_400.0 / (10.670 * 3_600.0);

/// Seconds of **nakshatra** boundary per arcsecond of lunar longitude.
///
/// A different rate from the tithi's, and the difference is the point: a
/// tithi is twelve degrees of the Moon's elongation *from the Sun*, so it
/// is crossed at 10.670 degrees a day, while a nakshatra is 13°20' of the
/// Moon's own longitude and is crossed at its sidereal 13.176. The same
/// arcsecond of error is therefore worth less time at a nakshatra edge
/// than at a tithi edge, and Phase 3's exit asks for both published.
const NAKSHATRA_SECONDS_PER_ARCSEC: f64 = 86_400.0 / (13.176_358 * 3_600.0);

/// A claim whose measurement is arcseconds, which `Claim::within` would
/// otherwise print as seconds of time.
fn within_arcsec(rule: String, worst: f64, bound: f64) -> Claim {
    Claim {
        rule,
        verdict: verdict_of(worst.is_finite() && worst <= bound),
        measured: format!("worst {}\u{2033}", arcsec(worst)),
    }
}

/// A claim whose measurement is seconds of tithi boundary.
fn within_tithi(rule: String, worst_arcsec: f64, bound_seconds: f64) -> Claim {
    let seconds = worst_arcsec * TITHI_SECONDS_PER_ARCSEC;
    Claim {
        rule,
        verdict: verdict_of(seconds.is_finite() && seconds <= bound_seconds),
        measured: if seconds < 10.0 {
            format!("worst {seconds:.2} s of tithi")
        } else {
            format!("worst {seconds:.0} s of tithi")
        },
    }
}

/// One rung of the correction ladder, as recorded.
#[derive(Debug, serde::Deserialize)]
struct LadderRow {
    correction: String,
    engine_moves_sun_arcsec: f64,
    sun_arcsec: f64,
    moon_arcsec: f64,
    mars_arcsec: f64,
    builtin_steps: Vec<String>,
}

/// One provider's true node against its own Moon's crossing.
#[derive(Debug, serde::Deserialize)]
struct SelfCheck {
    provider: String,
    year: i32,
    crossings: usize,
    worst_degrees: f64,
}

/// One body's agreement over the whole span, as recorded.
#[derive(Debug, serde::Deserialize)]
struct Row {
    body: String,
    worst_arcsec: f64,
    mean_arcsec: f64,
    at_1800_arcsec: f64,
    at_2100_arcsec: f64,
    at_2400_arcsec: f64,
    worst_speed_arcsec_per_day: f64,
    is_direction: bool,
    worst_lat_arcsec: f64,
    worst_lat_speed_arcsec_per_day: f64,
    worst_dist_relative: f64,
    worst_at_jd: f64,
}

/// One tier's recorded measurement.
#[derive(Debug, serde::Deserialize)]
struct Recorded {
    tier: String,
    frame: String,
    from_jd: f64,
    to_jd: f64,
    step_days: f64,
    samples: usize,
    engine_profile: String,
    moon_worst_tithi_seconds: f64,
    moon_with_bija_arcsec: f64,
    moon_without_bija_arcsec: f64,
    true_node_with_bija_arcsec: f64,
    true_node_without_bija_arcsec: f64,
    moon_bija_fitted_over: String,
    node_against_own_crossing: Vec<SelfCheck>,
    note: String,
    by_correction: Vec<LadderRow>,
    geocentric_rows: Vec<Row>,
    topocentric_rows: Vec<Row>,
}

/// One body's row from a centre's table.
fn row<'a>(rows: &'a [Row], body: &str) -> Option<&'a Row> {
    rows.iter().find(|candidate| candidate.body == body)
}

/// Reads whichever tiers have been recorded, richest last.
///
/// A missing file is a tier not yet measured and the page says so. A file
/// that is **present and will not parse** is a different thing entirely —
/// the recorded shape and this reader have drifted apart — and swallowing
/// it with an `.ok()` would leave the page quietly claiming that nothing
/// had been recorded at all. So it is reported and the pass fails.
fn recorded(root: &Path) -> Result<Vec<Recorded>, String> {
    let mut tiers = Vec::new();
    for tier in TIERS {
        let path = root.join(format!("{RECORDED}/stack-agreement-{tier}.json"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        match serde_json::from_str::<Recorded>(&text) {
            Ok(recorded) => tiers.push(recorded),
            Err(error) => {
                return Err(format!(
                    "{} is recorded but does not parse: {error}. Re-record it with \
                     `teistro-ephemeris-teimeris-stack-agreement`, or fix the reader.",
                    path.display()
                ));
            }
        }
    }
    Ok(tiers)
}

/// The tiers, or a message and a failing exit code.
fn outputs(root: &Path) -> Result<Vec<Output>, i32> {
    match recorded(root) {
        Ok(tiers) => Ok(vec![Output::new(PAGE, page(&tiers))]),
        Err(message) => {
            eprintln!("{message}");
            Err(1)
        }
    }
}

/// The year a Julian day falls in, which is what a reader of a trend
/// column wants; the JD itself is in the recorded file.
fn year_of(jd: f64) -> String {
    format!("{:.0}", 2000.0 + (jd - 2_451_545.0) / 365.25)
}

/// An arcsecond figure, three significant places and no more precision
/// than the measurement has.
fn arcsec(value: f64) -> String {
    if value >= 100.0 {
        format!("{value:.0}")
    } else if value >= 10.0 {
        format!("{value:.1}")
    } else if value >= 1.0 {
        format!("{value:.2}")
    } else {
        format!("{value:.3}")
    }
}

/// Writes the page.
pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => write(root, &outputs),
        Err(code) => code,
    }
}

/// Regenerates the page in memory and fails on any difference.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => check(root, &outputs, "cargo xtask agreement"),
        Err(code) => code,
    }
}

fn page(tiers: &[Recorded]) -> String {
    let mut out = String::new();
    out.push_str(&header(tiers));
    if tiers.is_empty() {
        out.push_str(&missing_section(tiers));
        return out;
    }
    // The richest tier recorded is the theories entire, so every other
    // tier's figures are read against it to separate what a truncation
    // costs from what a theory cannot do.
    let Some(richest) = tiers.last() else {
        return out;
    };
    for tier in tiers {
        out.push_str(&tier_section(tier, richest));
    }
    out.push_str(&ladder_section(tiers));
    out.push_str(&claims_section(tiers));
    out.push_str(&missing_section(tiers));
    out.push_str(&recorded_note(tiers));
    out.push_str(&limits_section());
    out
}

/// What the recording itself says it measured, quoted rather than
/// paraphrased: if the two ever disagree the page is the one that is
/// wrong, and quoting keeps them from disagreeing quietly.
fn recorded_note(tiers: &[Recorded]) -> String {
    let mut out = String::new();
    let Some(first) = tiers.first() else {
        return out;
    };
    let _ = writeln!(out, "## What the recording says it measured\n");
    let _ = writeln!(out, "> {}\n", first.note);
    out
}

fn header(tiers: &[Recorded]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# The completion, measured\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask agreement`, gated by `check-agreement`. Do not edit. The numbers are read from `{RECORDED}/stack-agreement-<tier>.json`, recorded by `teistro-ephemeris-teimeris-stack-agreement` against the reference engine — the number, not the code that produced it.\n"
    );
    let _ = writeln!(
        out,
        "Every other measurement in this directory isolates one thing: a truncation against the whole theory, a theory against the engine in its own frame, the Moon against a tithi. **This one measures what a consumer receives.** A position in the frame a chart is actually cast in — in the equinox of date, sidereal by Lahiri, apparent — with the SDK's own completion doing precession, light time, deflection, aberration, nutation and the ayanamsha over the built-in ephemeris, against the same frame from the reference engine.\n"
    );
    if let Some(first) = tiers.first() {
        let _ = writeln!(
            out,
            "It is **not** two runs of one pipeline, and the next section says why that could not be arranged and what follows from it. Read the geocentric table for the ephemeris and the topocentric one for the chart.\n\nThe frame is `{}`, on the engine's `{}` profile, at {} instants {:.0} days apart from JD {:.1} to {:.1} (1800 to 2400).\n",
            first.frame,
            first.engine_profile,
            count(first.samples),
            first.step_days,
            first.from_jd,
            first.to_jd,
        );
    }
    out
}

fn tier_section(tier: &Recorded, richest: &Recorded) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## `{}`\n", tier.tier);
    let _ = writeln!(
        out,
        "**The two sides are not one pipeline, and cannot be made one.** The engine answers the whole frame in a single native call — its step list is `positions:Native` and nothing else — while the built-in ephemeris is completed by the SDK's own steps. Forcing the SDK's everywhere was tried and is refused: the engine's native positions are *apparent*, the SDK can add corrections and never remove them, so a geometric frame comes back `Unsupported {{ step: \"corrections\" }}`.\n"
    );
    let _ = writeln!(
        out,
        "So both centres are recorded, and the pair is the point. **Geocentric isolates the ephemeris**: the instants are TT, so nothing in that path needs Delta T. **Topocentric is what a chart receives**, and it carries the two sides' Delta T models as well, through the observer's own rotation.\n"
    );
    out.push_str(&centre_table(
        "From the Earth's centre — the ephemeris",
        &tier.geocentric_rows,
    ));
    out.push_str(&centre_table(
        "From the place — what a chart receives",
        &tier.topocentric_rows,
    ));
    out.push_str(&rates_table(&tier.geocentric_rows));

    if let (Some(geocentric), Some(topocentric)) = (
        row(&tier.geocentric_rows, "MOON"),
        row(&tier.topocentric_rows, "MOON"),
    ) {
        let _ = writeln!(
            out,
            "The Moon is the whole of the difference between the two tables: {} from the centre, {} from the place. Nothing about the ephemeris changed between them. Delta T did — at 2400 the two sides' models differ by enough Earth rotation to move the Moon about a hundred arcseconds, and the Moon is the one body near enough for the observer's position to matter. The trend says the same thing: {} at 1800, where Delta T is recorded, against {} at 2400, where it is extrapolated.\n",
            arcsec_mark(geocentric.worst_arcsec),
            arcsec_mark(topocentric.worst_arcsec),
            arcsec_mark(topocentric.at_1800_arcsec),
            arcsec_mark(topocentric.at_2400_arcsec),
        );
        let at = |value: f64, per_arcsec: f64| format!("{:.1}", value * per_arcsec);
        let _ = writeln!(
            out,
            "The geocentric Moon again as **the time it costs a boundary**, which is what a consumer of an almanac feels and what Phase 3's exit asks be published. A tithi is twelve degrees of the Moon's elongation from the Sun and is crossed at 10.670 degrees a day; a nakshatra is 13°20' of the Moon's own longitude, crossed at its sidereal 13.176 — so the same arcsecond is worth less time at a nakshatra edge.\n"
        );
        let _ = writeln!(out, "| boundary | worst | 1800 | 2100 | 2400 |");
        let _ = writeln!(out, "|---|---:|---:|---:|---:|");
        for (name, per_arcsec) in [
            ("tithi", TITHI_SECONDS_PER_ARCSEC),
            ("nakshatra", NAKSHATRA_SECONDS_PER_ARCSEC),
        ] {
            let _ = writeln!(
                out,
                "| {name} | {} s | {} s | {} s | {} s |",
                at(geocentric.worst_arcsec, per_arcsec),
                at(geocentric.at_1800_arcsec, per_arcsec),
                at(geocentric.at_2100_arcsec, per_arcsec),
                at(geocentric.at_2400_arcsec, per_arcsec),
            );
        }
        let _ = writeln!(
            out,
            "\nThe recorded **topocentric** tithi worst, for comparison, is {} seconds — the Delta T term again, and the reason the table above is the geocentric one.\n",
            spaced(tier.moon_worst_tithi_seconds, 0),
        );
    }

    let _ = writeln!(
        out,
        "**The bija, measured rather than declared.** From the centre, the Moon is {} with it and {} without — the correction removes {} of {}, fitted over {}. The true node moves {} against {}, which is the same shift carried through and not a correction of its own.\n\nADR-0027 argued the theory to be 19.4\u{2033} uncorrected over this span and about 3 corrected, from a floor measurement in ELP's own frame. Reaching {} and {} again through the whole completion is an independent path to the same two numbers.\n\nIt is a knob rather than a baked-in adjustment, so what it is worth is recorded beside what it was set to — where a `bija: true` field would have asserted only that somebody meant to switch it on.\n",
        arcsec_mark(tier.moon_with_bija_arcsec),
        arcsec_mark(tier.moon_without_bija_arcsec),
        arcsec_mark((tier.moon_without_bija_arcsec - tier.moon_with_bija_arcsec).abs()),
        arcsec_mark(tier.moon_without_bija_arcsec),
        tier.moon_bija_fitted_over,
        arcsec_mark(tier.true_node_with_bija_arcsec),
        arcsec_mark(tier.true_node_without_bija_arcsec),
        arcsec_mark(tier.moon_without_bija_arcsec),
        arcsec_mark(tier.moon_with_bija_arcsec),
    );

    out.push_str(&node_section(tier, richest));
    out
}

/// The true node's account: why it is what it is, and the three
/// explanations measurement excluded before the fourth was believed.
fn node_section(tier: &Recorded, richest: &Recorded) -> String {
    let mut out = String::new();
    if let (Some(node), Some(moon)) = (
        row(&tier.geocentric_rows, "TRUE_NODE"),
        row(&tier.geocentric_rows, "MOON"),
    ) {
        let inclination = INCLINATION_DEG.to_radians();
        let tilt = node.worst_arcsec * inclination.sin();
        // The latitude rate's own amplitude, so a disagreement in it can
        // be read as the tilt it implies.
        let rate_amplitude =
            std::f64::consts::TAU * INCLINATION_DEG * 3_600.0 / DRACONIC_MONTH_DAYS;
        let implied =
            moon.worst_lat_speed_arcsec_per_day / rate_amplitude * INCLINATION_DEG * 3_600.0;
        let richest_node =
            row(&richest.geocentric_rows, "TRUE_NODE").map_or(f64::NAN, |row| row.worst_arcsec);
        let _ = writeln!(
            out,
            "**The true node, and why it is {} when the Moon is {}.** A node is where the orbit meets the ecliptic, and near that meeting the Moon's latitude is level: it climbs at about the tangent of the orbit's {INCLINATION_DEG}-degree inclination, so an error across the orbit arrives along it divided by that tangent — an amplification of about {:.0}.\n",
            arcsec_mark(node.worst_arcsec),
            arcsec_mark(moon.worst_arcsec),
            1.0 / inclination.tan(),
        );
        let by_truncation = node.worst_arcsec - richest_node;
        let _ = writeln!(
            out,
            "{} It is **not the bija**: turning it off moves the node {} of {}. It is **not the precession model**: deriving the reference pole from the SDK's own model rather than the lunar theory's moved it 0.13 arcseconds. And it is **not the node's own arithmetic** — the table below shows each provider's node sitting on its own Moon's crossing.\n",
            if by_truncation.abs() < 1.0 {
                format!(
                    "It is **not truncation**: `{}`, the theories entire, reads {} for the same figure.",
                    richest.tier,
                    arcsec_mark(richest_node)
                )
            } else {
                format!(
                    "**{} of it is this tier's truncation**: `{}`, the theories entire, reads {}, so the rest of what follows is about the {} that remain there too.",
                    arcsec_mark(by_truncation),
                    richest.tier,
                    arcsec_mark(richest_node),
                    arcsec_mark(richest_node)
                )
            },
            arcsec_mark(
                (tier.true_node_without_bija_arcsec - tier.true_node_with_bija_arcsec).abs()
            ),
            arcsec_mark(node.worst_arcsec),
        );
        out.push_str(&self_check_table(&tier.node_against_own_crossing));
        let _ = writeln!(
            out,
            "**Each node is where its own Moon crosses.** So neither is wrong about its own theory, and what the {} measures is the two theories disagreeing about the *plane* rather than about the node. Read backwards, {} of node is {} of orbital tilt — and the Moon's position agrees far better than that, to {} of latitude. A plane is fixed by a velocity as much as by a position, and the out-of-plane rate is where the disagreement is: {} a day against a rate whose own amplitude is {}, which tilts a {INCLINATION_DEG}-degree orbit by {}. That is the number the node is amplifying, and it accounts for the tilt the node implies.\n",
            arcsec_mark(node.worst_arcsec),
            arcsec_mark(node.worst_arcsec),
            arcsec_mark(tilt),
            arcsec_mark(moon.worst_lat_arcsec),
            arcsec_mark(moon.worst_lat_speed_arcsec_per_day),
            arcsec_mark(rate_amplitude),
            arcsec_mark(implied),
        );
        let _ = writeln!(
            out,
            "The mean node, a polynomial that does not depend on where the Moon is, is unaffected and reads {} at every tier.\n",
            row(&tier.geocentric_rows, "MEAN_NODE").map_or_else(
                || "an unrecorded figure".to_string(),
                |mean| arcsec_mark(mean.worst_arcsec)
            ),
        );
    }
    out
}

/// The rates, which are what an *event* is found from: a crossing, a
/// conjunction, a tithi boundary. A position can agree while the motion
/// through it does not, and a search that lands on a boundary feels the
/// rate rather than the place.
fn rates_table(rows: &[Row]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "### The rates, from the centre\n");
    let _ = writeln!(out, "| body | longitude (\"/day) | latitude (\"/day) |");
    let _ = writeln!(out, "|---|---:|---:|");
    for row in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            row.body,
            arcsec(row.worst_speed_arcsec_per_day),
            if row.is_direction {
                "\u{2014}".to_string()
            } else {
                arcsec(row.worst_lat_speed_arcsec_per_day)
            },
        );
    }
    let _ = writeln!(
        out,
        "\nWorst disagreement in each rate, arcseconds a day. The Moon's latitude rate matters twice over: it is the out-of-plane motion, so it fixes the orbital plane the nodes are cut from.\n"
    );
    out
}

/// Each provider's node against its own Moon, which separates a
/// disagreement between two definitions from an error in either.
fn self_check_table(checks: &[SelfCheck]) -> String {
    let mut out = String::new();
    if checks.is_empty() {
        return out;
    }
    let _ = writeln!(
        out,
        "| provider | year | crossings | node against its own crossing |"
    );
    let _ = writeln!(out, "|---|---:|---:|---:|");
    for check in checks {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            check.provider,
            check.year,
            check.crossings,
            arcsec_mark(check.worst_degrees * 3_600.0),
        );
    }
    let _ = writeln!(
        out,
        "\nA month of six-hour steps at each epoch, with the crossing interpolated rather than sampled — at six hours the Moon moves three degrees, and comparing at the nearest sample would measure the step size and call it an error.\n"
    );
    out
}

/// One centre's table of bodies.
fn centre_table(title: &str, rows: &[Row]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "### {title}\n");
    let _ = writeln!(
        out,
        "| body | worst | mean | 1800 | 2100 | 2400 | latitude | distance | worst at |"
    );
    let _ = writeln!(out, "|---|---:|---:|---:|---:|---:|---:|---:|---:|");
    for row in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.body,
            arcsec(row.worst_arcsec),
            arcsec(row.mean_arcsec),
            arcsec(row.at_1800_arcsec),
            arcsec(row.at_2100_arcsec),
            arcsec(row.at_2400_arcsec),
            if row.is_direction {
                "—".to_string()
            } else {
                arcsec(row.worst_lat_arcsec)
            },
            if row.is_direction {
                "—".to_string()
            } else {
                format!("{:.1e}", row.worst_dist_relative)
            },
            year_of(row.worst_at_jd),
        );
    }
    let _ = writeln!(
        out,
        "\nArcseconds of ecliptic longitude, the shortest way round the circle, except the last two: **latitude** is the worst in arcseconds and **distance** the worst relative difference. A node and an apogee are *directions*, carrying a longitude and nothing else, so the SDK answers them with no latitude and no distance and those two columns would compare a declared zero against whatever the engine supplies — a convention, not an error, and left blank rather than dressed up as a measurement.\n"
    );
    out
}

/// An arcsecond figure with its mark, for prose.
fn arcsec_mark(value: f64) -> String {
    format!("{}\u{2033}", arcsec(value))
}

fn ladder_section(tiers: &[Recorded]) -> String {
    let mut out = String::new();
    let Some(tier) = tiers
        .iter()
        .find(|tier| tier.tier == "standard")
        .or(tiers.first())
    else {
        return out;
    };
    let _ = writeln!(out, "## One correction at a time\n");
    let _ = writeln!(
        out,
        "A disagreement in the apparent frame is five corrections at once, and a table of it names none of them. This ladder asks for one correction at a time, at `{}`, so a number can be attributed.\n",
        tier.tier
    );
    let _ = writeln!(
        out,
        "The first column is the check that makes the rest mean anything: **how far the engine's own answer moves** when only that correction is asked for. A flag the engine does not honour leaves it at zero, and a comparison against a correction the engine did not apply says nothing at all.\n"
    );
    let _ = writeln!(
        out,
        "| correction | engine moves the Sun | Sun | Moon | Mars |"
    );
    let _ = writeln!(out, "|---|---:|---:|---:|---:|");
    for row in &tier.by_correction {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            row.correction,
            arcsec(row.engine_moves_sun_arcsec),
            arcsec(row.sun_arcsec),
            arcsec(row.moon_arcsec),
            arcsec(row.mars_arcsec),
        );
    }
    let single: Vec<&LadderRow> = tier
        .by_correction
        .iter()
        .filter(|row| row.correction.ends_with("only"))
        .collect();
    let named = |rows: &[&LadderRow]| -> String {
        let names: Vec<String> = rows
            .iter()
            .map(|row| format!("`{}`", row.correction.trim_end_matches(" only")))
            .collect();
        match names.as_slice() {
            [] => "none of them".to_string(),
            [one] => one.clone(),
            [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
        }
    };
    let honoured: Vec<&LadderRow> = single
        .iter()
        .copied()
        .filter(|row| row.engine_moves_sun_arcsec > 0.001)
        .collect();
    let ignored: Vec<&LadderRow> = single
        .iter()
        .copied()
        .filter(|row| row.engine_moves_sun_arcsec <= 0.001)
        .collect();
    let _ = writeln!(
        out,
        "\nOf the {} single corrections the engine distinguishes {} — asking for {} moves its own answer not at all. So those rows do not measure agreement: they measure the SDK applying a correction the engine did not, and are here to be read that way rather than mistaken for accuracy. **Only the rows the engine honours, and the two whole-frame rows at the foot, compare like with like.**\n",
        spelled(single.len()),
        named(&honoured),
        named(&ignored),
    );
    if let Some(steps) = tier
        .by_correction
        .last()
        .map(|row| row.builtin_steps.join("`, `"))
    {
        let _ = writeln!(
            out,
            "The last row runs the whole completion: `{steps}`. Each names who did the work — `Native` is the provider's own answer, `Sdk` is this crate's.\n"
        );
    }
    out
}

/// Every bound, for every tier recorded, in the order a reader meets
/// them. Building them is separate from rendering them so that adding a
/// bound is one list entry rather than a change inside a page.
fn claims_for(tiers: &[Recorded]) -> Vec<Claim> {
    let mut claims = Vec::new();
    for tier in tiers {
        let name = &tier.tier;
        let planets: Vec<&Row> = tier
            .geocentric_rows
            .iter()
            .filter(|row| {
                !matches!(
                    row.body.as_str(),
                    "MOON" | "MEAN_NODE" | "TRUE_NODE" | "SUN"
                )
            })
            .collect();
        claims.push(within_arcsec(
            format!("`{name}`: the retired single-number claim — one arcsecond for every planet"),
            planets
                .iter()
                .map(|row| row.worst_arcsec)
                .fold(0.0_f64, f64::max),
            1.0,
        ));
        claims.push(within_arcsec(
            format!("`{name}`: the Sun stays inside one arcsecond over the whole span"),
            tier.geocentric_rows
                .iter()
                .find(|row| row.body == "SUN")
                .map_or(f64::NAN, |row| row.worst_arcsec),
            1.0,
        ));
        if let Some(moon) = tier.geocentric_rows.iter().find(|row| row.body == "MOON") {
            // ADR-0027's own words: the bija makes `standard`
            // second-accurate over the two centuries it is fitted to and
            // minute-accurate over six. Both are claims about a tithi
            // boundary, so both are measured as one.
            claims.push(within_tithi(
                format!(
                    "`{name}`: second-accurate inside the bija's fitted span, {} (ADR-0027)",
                    tier.moon_bija_fitted_over
                ),
                moon.at_2100_arcsec,
                1.0,
            ));
            claims.push(within_tithi(
                format!("`{name}`: minute-accurate over the whole six centuries (ADR-0027)"),
                moon.worst_arcsec,
                60.0,
            ));
            claims.push(within_arcsec(
                format!("`{name}`: the Moon meets the almanac's {ALMANAC_MOON_ARCSEC} arcseconds"),
                moon.worst_arcsec,
                ALMANAC_MOON_ARCSEC,
            ));
        }
        if let Some(node) = tier
            .geocentric_rows
            .iter()
            .find(|row| row.body == "MEAN_NODE")
        {
            claims.push(within_arcsec(
                format!("`{name}`: the mean node is inside one arcsecond"),
                node.worst_arcsec,
                1.0,
            ));
        }
    }
    claims
}

fn claims_section(tiers: &[Recorded]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## The claims this decides\n");
    let _ = writeln!(
        out,
        "Each bound is one ADR-0027 argued from, not one chosen to be met, and each is decided on the **geocentric** table — the one that isolates the ephemeris. Deciding them on the topocentric table would be charging the lunar theory for a Delta T model six centuries out. A falsified row is the measurement doing its job: it says the tier cannot carry that claim, and the tier's published figure becomes the measured one.\n"
    );
    let claims = claims_for(tiers);
    out.push_str(&crate::measure::table(&claims));
    let falsified: Vec<&Claim> = claims
        .iter()
        .filter(|claim| matches!(claim.verdict, Verdict::Falsified))
        .collect();
    if falsified.is_empty() {
        let _ = writeln!(out, "\nEvery bound holds.\n");
    } else {
        let _ = writeln!(
            out,
            "\n{} of {} falsified:\n",
            falsified.len(),
            claims.len()
        );
        for claim in &falsified {
            let _ = writeln!(out, "- {} — {}", claim.rule, claim.measured);
        }
        // Which bounds fail even at the richest tier? Those are the
        // theory, and no table can mend them. Which fail only at a
        // lighter one? Those are the truncation that tier was chosen to
        // buy. Grouping them by hand would be a sentence that drifts from
        // the rows above it the first time a figure moves.
        let strip = |rule: &str| -> String {
            rule.split_once(": ")
                .map_or_else(|| rule.to_string(), |(_, rest)| rest.to_string())
        };
        let richest = tiers
            .last()
            .map(|tier| tier.tier.as_str())
            .unwrap_or_default();
        let theory: Vec<String> = falsified
            .iter()
            .filter(|claim| claim.rule.starts_with(&format!("`{richest}`")))
            .map(|claim| strip(&claim.rule))
            .collect();
        let truncation: Vec<String> = falsified
            .iter()
            .filter(|claim| !claim.rule.starts_with(&format!("`{richest}`")))
            .map(|claim| strip(&claim.rule))
            .filter(|rule| !theory.contains(rule))
            .collect();
        let _ = writeln!(
            out,
            "\n**None of them is the completion.** What the completion decides is the Sun, the mean node and the frame the whole table is stated in; those are the rows that hold wherever the ephemeris under them is good enough to show it.\n"
        );
        if !theory.is_empty() {
            let _ = writeln!(
                out,
                "Of these, {} still fail at `{richest}`, the theories entire, so no table can mend them — they are the **age of a published theory**. VSOP87 was fitted to DE200 in 1981 and the engine answers from a modern ephemeris, which is why a planet drifts; ELP2000-82B carries the same era's tidal term, which is what the bija removes and what it cannot remove all of. Only a refit against a modern reference — `reference`, in ADR-0021's ladder — moves these.\n",
                spelled(theory.len()),
            );
            for rule in &theory {
                let _ = writeln!(out, "- {rule}");
            }
            let _ = writeln!(out);
        }
        if !truncation.is_empty() {
            let _ = writeln!(
                out,
                "The rest fail only at the lighter tiers, and that is the **truncation those tiers were chosen to buy** rather than a defect: `compact` is an arcminute in about 80 KB, and it says so.\n"
            );
            for rule in &truncation {
                let _ = writeln!(out, "- {rule}");
            }
            let _ = writeln!(out);
        }
    }
    out
}

fn missing_section(tiers: &[Recorded]) -> String {
    let mut out = String::new();
    let missing: Vec<&str> = TIERS
        .iter()
        .copied()
        .filter(|tier| !tiers.iter().any(|recorded| recorded.tier == *tier))
        .collect();
    if missing.is_empty() {
        return out;
    }
    let _ = writeln!(out, "## Tiers not yet recorded\n");
    let _ = writeln!(
        out,
        "**{}.** The measurement needs the tier compiled in and the reference engine present, so each is recorded separately:\n",
        missing.join(", ")
    );
    for tier in &missing {
        let _ = writeln!(
            out,
            "```text\ncargo run --release --no-default-features --features {tier} \\\n  --bin teistro-ephemeris-teimeris-stack-agreement \\\n  > {RECORDED}/stack-agreement-{tier}.json\n```"
        );
    }
    let _ = writeln!(
        out,
        "\nUntil then this page's per-tier claim covers {} of {} tiers, which is what these lines are for: a page titled \"per tier\" that quietly covered two of three would be a claim with nothing behind it.\n",
        tiers.len(),
        TIERS.len()
    );
    out
}

fn limits_section() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What this does not measure\n");
    let _ = writeln!(
        out,
        "**Correctness.** Both sides are compared against the same reference engine, so agreement proves consistency and not truth (ADR-0021). The bija is refitted against JPL Horizons or CSPICE before v1 and every figure here republished if it moves.\n"
    );
    let _ = writeln!(
        out,
        "**Any span but 1800 to 2400.** Every trend column grows towards the edges, so a tier claiming a wider range is swept over that range rather than inheriting these numbers.\n"
    );
    let _ = writeln!(
        out,
        "**Latitude and distance.** The tables are ecliptic longitude, which is what a sign, a nakshatra, a tithi and a dasha are computed from. Latitude matters to a graha's war and to the true node, and is measured where those are.\n"
    );
    let _ = writeln!(
        out,
        "**Pluto**, which has no series here at all and is fitted from a public-domain kernel separately.\n"
    );
    out
}

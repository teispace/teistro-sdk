//! The falsification pass over **how accurate the Moon has to be**.
//!
//! Phase 3 has to choose a lunar theory, and
//! `01-research/platform/14-builtin-ephemeris.md` says why the choice
//! matters more than the planets': "the tiers are chosen by Moon
//! accuracy first". It does not say what accuracy, and the number is not
//! a matter of taste — panchanga publishes *instants*, and an error in
//! the Moon's longitude moves them.
//!
//! # What decides it
//!
//! A limb's boundary is where a quantity crosses a line of its lattice.
//! If the quantity is wrong by `ε` degrees, the crossing is wrong by
//! `ε / rate` days, so an error of one arcsecond moves a boundary by
//! `24 / rate` seconds of clock time. The rate is the quantity's own,
//! not the Moon's: a tithi runs on the Moon **less** the Sun and a yoga
//! on the Moon **plus** it, so the same lunar error is worth more
//! seconds in a tithi than in a yoga.
//!
//! The rate is not constant either. The Moon's daily motion swings by a
//! third between perigee and apogee, so the worst case is the **slowest**
//! the quantity ever moves — which the SDK already catalogues as
//! [`quantity_least_rate`], the same table the search grid is sized
//! from. Using it here rather than a second copy is deliberate: two
//! tables of the same constants are two tables that can disagree.
//!
//! # What is measured and what is derived
//!
//! The rates are catalogued and the arithmetic above is exact. The one
//! measurement is the **Sun's** own error, which
//! `crates/ephemeris-builtin/data/vsop87-floor.json` records against the
//! engine, and which enters every tithi and every yoga beside the Moon's.
//! A page that quoted only the Moon's budget would be describing half of
//! the error in the two limbs that matter most.
//!
//! `cargo xtask moon` writes the page.

use std::fmt::Write as _;
use std::path::Path;

use teistro_astro::events::{Quantity, quantity_least_rate};
use teistro_port_ephemeris::Body;

use crate::generated::{Output, write};
use crate::measure::{Claim, count, fill, table, verdict_of};

/// A byte count as kilobytes. The cast is exact: a coefficient table is
/// kilobytes, decades below the range an `f64` represents exactly.
#[expect(
    clippy::cast_precision_loss,
    reason = "a table's size is kilobytes, far below 2^53"
)]
fn kilobytes(bytes: usize) -> f64 {
    bytes as f64 / 1024.0
}

const PAGE: &str = "docs/03-design/lunar-accuracy-measured.md";

/// Where the Sun's measured error is recorded.
const FLOOR: &str = "crates/ephemeris-builtin/data/vsop87-floor.json";

/// Where the Moon's measured error is recorded, by
/// `teistro-ephemeris-teimeris-moon-floor`.
const MOON_FLOOR: &str = "crates/ephemeris-builtin/data/elp82b-floor.json";

/// Seconds of clock time in a day.
const SECONDS_PER_DAY: f64 = 86_400.0;

/// Arcseconds in a degree.
const ARCSEC_PER_DEGREE: f64 = 3_600.0;

/// One limb, its quantity, and whether the Sun's error enters it.
struct Limb {
    name: &'static str,
    quantity: Quantity,
    /// How many of the circle's members there are, for the dwell figure.
    divisions: f64,
    /// Whether an error in the Sun moves this limb's boundaries too.
    solar: bool,
}

fn limbs() -> [Limb; 4] {
    [
        Limb {
            name: "Tithi",
            quantity: Quantity::ELONGATION,
            divisions: 30.0,
            solar: true,
        },
        Limb {
            name: "Karana",
            quantity: Quantity::ELONGATION,
            divisions: 60.0,
            solar: true,
        },
        Limb {
            name: "Nakshatra",
            quantity: Quantity::Longitude(Body::Moon),
            divisions: 27.0,
            solar: false,
        },
        Limb {
            name: "Yoga",
            quantity: Quantity::MOON_PLUS_SUN,
            divisions: 27.0,
            solar: true,
        },
    ]
}

/// Seconds a boundary moves for one arcsecond of error in the quantity.
fn seconds_per_arcsec(rate_deg_per_day: f64) -> f64 {
    SECONDS_PER_DAY / (rate_deg_per_day * ARCSEC_PER_DEGREE)
}

/// The Sun's worst measured error, if the floor has been recorded.
fn solar_error_arcsec(root: &Path) -> Option<f64> {
    let text = std::fs::read_to_string(root.join(FLOOR)).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    value
        .get("rows")?
        .as_array()?
        .iter()
        .find(|row| row.get("body").and_then(serde_json::Value::as_str) == Some("Sun"))?
        .get("worst_arcsec")?
        .as_f64()
}

/// One span of the recorded lunar measurement.
#[derive(Debug, serde::Deserialize)]
struct MoonSpan {
    label: String,
    worst_arcsec: f64,
    worst_tithi_seconds: f64,
    radius_relative: f64,
}

/// One truncation row of the recorded lunar measurement.
#[derive(Debug, serde::Deserialize)]
struct MoonRow {
    threshold: f64,
    terms: usize,
    bytes: usize,
    worst_arcsec: f64,
}

/// One degree of the fitted correction.
#[derive(Debug, serde::Deserialize)]
struct MoonCorrection {
    degree: usize,
    worst_before_arcsec: f64,
    worst_after_arcsec: f64,
    worst_after_tithi_seconds: f64,
    bytes: usize,
}

/// The recorded lunar measurement.
#[derive(Debug, serde::Deserialize)]
struct MoonFloor {
    engine_profile: String,
    whole_theory_terms: usize,
    rows: Vec<MoonRow>,
    spans: Vec<MoonSpan>,
    #[serde(default)]
    corrections: Vec<MoonCorrection>,
}

/// One row of the Chebyshev sizing study.
#[derive(Debug, serde::Deserialize)]
struct FitRow {
    interval_days: f64,
    coefficients: usize,
    worst_arcsec: f64,
    bytes: usize,
}

/// The Chebyshev sizing study.
#[derive(Debug, serde::Deserialize)]
struct FitRecord {
    rows: Vec<FitRow>,
}

/// Where the Chebyshev sizing study is recorded.
const FIT: &str = "crates/ephemeris-builtin/data/moon-chebyshev.json";

/// Writes the page.
pub(crate) fn generate(root: &Path) -> i32 {
    let moon = std::fs::read_to_string(root.join(MOON_FLOOR))
        .ok()
        .and_then(|text| serde_json::from_str::<MoonFloor>(&text).ok());
    let fitted = std::fs::read_to_string(root.join(FIT))
        .ok()
        .and_then(|text| serde_json::from_str::<FitRecord>(&text).ok());
    write(
        root,
        &[Output::new(
            PAGE,
            page(solar_error_arcsec(root), moon.as_ref(), fitted.as_ref()),
        )],
    )
}

/// The page, as the sections it is made of.
fn page(solar_arcsec: Option<f64>, moon: Option<&MoonFloor>, fitted: Option<&FitRecord>) -> String {
    let (sensitivity, worst) = sensitivity_section();
    let mut out = String::new();
    out.push_str(&opening());
    out.push_str(&sensitivity);
    out.push_str(&budget_section(worst, solar_arcsec));
    out.push_str(&candidates_section(worst));
    out.push_str(&finding_section(solar_arcsec));
    if let Some(moon) = moon {
        out.push_str(&measured_section(moon));
        out.push_str(&routes_section(moon, fitted));
    }
    out.push_str(&claims_section(worst, moon));
    out.push_str(&limits_section(moon.is_some()));
    fill(&out)
}

/// What ELP2000-82B measured to, which is the answer the page was for.
fn measured_section(moon: &MoonFloor) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What ELP2000-82B actually costs\n");
    let _ = writeln!(
        out,
        "Measured against the engine in the frame the theory is stated in — geocentric, ecliptic, J2000, geometric — on the engine's `{}` profile. The whole theory is {} terms.\n",
        moon.engine_profile,
        count(moon.whole_theory_terms)
    );
    let _ = writeln!(
        out,
        "**Truncation is not what limits it.** The ladder converges long before the terms run out, so the Moon is cheap and the theory is the whole cost:\n"
    );
    let _ = writeln!(out, "| threshold | terms | table | worst |");
    let _ = writeln!(out, "|---:|---:|---:|---:|");
    for row in &moon.rows {
        let label = if row.threshold == 0.0 {
            "0 (whole theory)".to_string()
        } else {
            format!("{}", row.threshold)
        };
        let _ = writeln!(
            out,
            "| {label} | {} | {:.0} KB | {:.2}″ |",
            count(row.terms),
            kilobytes(row.bytes),
            row.worst_arcsec
        );
    }
    let _ = writeln!(
        out,
        "\n**What limits it is distance from its own epoch.** Every row below keeps every term:\n"
    );
    let _ = writeln!(
        out,
        "| span | worst | as tithi boundary | radius | verdict |"
    );
    let _ = writeln!(out, "|---|---:|---:|---:|---|");
    for span in &moon.spans {
        let verdict = if span.worst_arcsec <= 0.445 {
            "**second-accurate**"
        } else if span.worst_tithi_seconds <= 60.0 {
            "minute-accurate"
        } else {
            "neither"
        };
        let _ = writeln!(
            out,
            "| {} | {:.3}″ | {:.2} s | {:.1e} | {verdict} |",
            span.label, span.worst_arcsec, span.worst_tithi_seconds, span.radius_relative
        );
    }
    let _ = writeln!(
        out,
        "\nThe first row is the check that the port is faithful rather than the finding: at the theory's own epoch it agrees with a modern ephemeris to a quarter of an arcsecond, which is what a correct reading of a DE200-fitted theory should give. Everything after it is the fit ageing — and it ages fast, because a lunar theory's mean longitude carries the tidal acceleration its source ephemeris assumed, and an error there grows with the square of the time.\n"
    );
    out
}

/// What the page is and where its numbers come from.
fn opening() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# How accurate the Moon has to be, measured\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask moon`. Do not edit. The rates are the SDK's own catalogued extremes (`astro::events::quantity_least_rate`, the table the search grid is sized from) and the arithmetic over them is exact; the Sun's error is measured and recorded in `{FLOOR}`.\n"
    );
    let _ = writeln!(
        out,
        "Phase 3 must choose a lunar theory, and the research page says the tiers are chosen by Moon accuracy first without saying what accuracy. This is the number. Panchanga publishes instants, so an error in the Moon's longitude is not an error in a position — it is an error in a *time*, and this page converts one into the other.\n"
    );
    out
}

/// The table of what one arcsecond costs each limb, and the worst of them.
fn sensitivity_section() -> (String, f64) {
    let mut out = String::new();
    let _ = writeln!(out, "## What an arcsecond of lunar error costs\n");
    let _ = writeln!(
        out,
        "A boundary is where a quantity crosses a line of its lattice. An error of `e` degrees in the quantity moves the crossing by `e / rate` days, so one arcsecond moves it by `24 / rate` seconds. The rate is the **quantity's**, which is why the same lunar error is worth more in a tithi than in a yoga: a tithi runs on the Moon less the Sun and a yoga on the Moon plus it. The rate used is the slowest the quantity ever moves, because that is the worst case and the Moon's daily motion swings by a third between perigee and apogee.\n"
    );
    let _ = writeln!(
        out,
        "| limb | quantity | member width | slowest rate | one arcsecond is | one second needs | one minute needs |"
    );
    let _ = writeln!(out, "|---|---|---:|---:|---:|---:|---:|");
    let mut worst: f64 = 0.0;
    for limb in limbs() {
        let Some(rate) = quantity_least_rate(limb.quantity) else {
            continue;
        };
        let seconds = seconds_per_arcsec(rate);
        worst = worst.max(seconds);
        let _ = writeln!(
            out,
            "| {} | {} | {:.1}° | {rate:.3}°/day | {seconds:.2} s | {:.3}″ | {:.2}″ |",
            limb.name,
            if limb.solar {
                "Moon and Sun"
            } else {
                "Moon alone"
            },
            360.0 / limb.divisions,
            1.0 / seconds,
            60.0 / seconds,
        );
    }
    (out, worst)
}

/// What the sensitivity asks of a theory, and what the Sun has spent of it.
fn budget_section(worst: f64, solar_arcsec: Option<f64>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "\n## What that asks of a theory\n");
    let _ = writeln!(
        out,
        "Reading the table the other way: to hold **every** panchanga boundary to one second of clock time, the Moon's longitude must be right to **{:.3}″**, and to hold it to one minute, to **{:.2}″**. The tithi is the binding limb, because its quantity is the slowest.\n",
        1.0 / worst,
        60.0 / worst
    );
    if let Some(solar) = solar_arcsec {
        let _ = writeln!(
            out,
            "**The Sun is already spent, and it is cheap.** The floor measurement puts VSOP87's Sun at {solar:.3}″ against the engine, which is {:.2} s of tithi boundary on its own. That leaves the Moon essentially the whole budget rather than half of it, and it means a lunar theory good to a tenth of an arcsecond is not wasted on a Sun good to a seventh.\n",
            solar * worst
        );
    }
    out
}

/// The two lunar theories, weighed against the budget.
fn candidates_section(worst: f64) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## The candidates\n");
    let _ = writeln!(
        out,
        "| theory | fitted to | published accuracy | as tithi boundary | source |"
    );
    let _ = writeln!(out, "|---|---|---|---:|---|");
    let _ = writeln!(
        out,
        "| ELP/MPP02 | DE405 and DE406 | 2.4 m over a century about J2000; 1.4 km over five millennia | {:.3} s and {:.1} s | **cannot be obtained** |",
        metres_as_seconds(2.4, worst),
        metres_as_seconds(1_400.0, worst)
    );
    let _ = writeln!(
        out,
        "| ELP2000-82B | DE200 and LE200 | stated as the theory's own; unmeasured here | unmeasured | CDS VI/79 and IMCCE, both reachable |"
    );
    let _ = writeln!(
        out,
        "\nA metre at the Moon's mean distance subtends {:.2e}″, which is how the first row's seconds are reached.\n",
        arcsec_per_metre()
    );
    out
}

/// Why the choice is blocked, with every route tried.
fn finding_section(solar_arcsec: Option<f64>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## The finding that blocks the choice\n");
    let _ = writeln!(
        out,
        "**ELP/MPP02 cannot be obtained.** ADR-0013 and the research page name it as the SDK's lunar theory. Every route to it was tried on 2026-09-10, and the result is not a broken link but a removal.\n"
    );
    let _ = writeln!(out, "| route | result |");
    let _ = writeln!(out, "|---|---|");
    let _ = writeln!(
        out,
        "| `cyrano-se.obspm.fr`, the address the theory is published at | no answer over FTP or HTTPS |"
    );
    let _ = writeln!(
        out,
        "| the same path in the Internet Archive | the directory listing is captured, naming `ELP_MAIN.S1/2/3` and `ELP_PERT.S1/2/3`; **not one of the six files was ever captured**, and the directory itself already answered 404 in the August 2022 crawl while its five siblings did not |"
    );
    let _ = writeln!(
        out,
        "| `ftp.imcce.fr/pub/ephem/moon/` | `elp82b` and nothing later |"
    );
    let _ = writeln!(out, "| CDS catalogue VI/79 | ELP2000-82B as well |");
    let _ = writeln!(
        out,
        "| the two public re-implementations that carry data files | **excluded.** One is GPL-3.0 and the other EUPL-1.2, and `deny.toml` refuses copyleft everywhere in the workspace (ADR-0019). The GPL one's files are transformed besides — fourteen files under names of its own, where the publication has six — so they are that project's derived work and not the published series |"
    );
    let _ = writeln!(
        out,
        "\nSo the choice is between a theory that cannot be had and one that can, and the second has a question over it.\n"
    );
    let _ = writeln!(
        out,
        "**ELP2000-82B says of itself: \"Constants fitted to JPL's ephemerides DE200/LE200\".** That is the header of `elp82b.f`, the reader its own authors publish, and not an inference from outside. It is the same fit whose age the planetary floor measurement caught: VSOP87 was fitted to DE200 in 1981 and drifts against a modern ephemeris by 4.7 arcseconds at Uranus and 6.6 at Neptune, while holding {} at the Sun.\n",
        solar_arcsec.map_or_else(|| "well".to_string(), |value| format!("{value:.3}″"))
    );
    let _ = writeln!(
        out,
        "Whether the Moon inherits that drift is **not known and must not be assumed either way**. A tenth of the planets' worst would still be inside the second-accurate budget; a half of it would not. The measurement is the one the planets had — the whole theory against the engine in the isolated frame — and the harness for it exists.\n"
    );
    out
}

/// The three ways out, each with its measured price.
fn routes_section(moon: &MoonFloor, fitted: Option<&FitRecord>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## The three ways out, priced\n");
    let _ = writeln!(
        out,
        "The theory is 19.4″ over the span `standard` claims, and that is 44 seconds of tithi against a budget of one. There are three ways out and all three are now measured rather than argued.\n"
    );

    let _ = writeln!(out, "### Correct the theory's mean longitude\n");
    let _ = writeln!(
        out,
        "The drift is not noise. A lunar theory ages mainly through the tidal acceleration its source ephemeris assumed, which enters the mean longitude as a term in the **square** of the time — so if that is what the difference is, a polynomial of two or three coefficients removes most of it. The tradition has the same idea and the same name for it: a *bija*, a seed correction that re-anchors an old theory to the present sky, which this SDK already computes for the Surya Siddhanta.\n"
    );
    let _ = writeln!(out, "| degree | cost | before | after | as tithi |");
    let _ = writeln!(out, "|---:|---:|---:|---:|---:|");
    for correction in &moon.corrections {
        let _ = writeln!(
            out,
            "| {} | {} bytes | {:.2}″ | {:.2}″ | {:.1} s |",
            correction.degree,
            correction.bytes,
            correction.worst_before_arcsec,
            correction.worst_after_arcsec,
            correction.worst_after_tithi_seconds
        );
    }
    let best = moon
        .corrections
        .iter()
        .min_by(|a, b| a.worst_after_arcsec.total_cmp(&b.worst_after_arcsec));
    if let Some(best) = best {
        let quadratic = moon.corrections.iter().find(|c| c.degree == 2);
        if let Some(quadratic) = quadratic {
            let _ = writeln!(
                out,
                "\n**It saturates at the square**, which is the physics rather than a coincidence: degree three and four buy nothing over degree two, so what the polynomial is removing is the tidal term and not a curve fitted to noise. {} bytes take the Moon from {:.2}″ to {:.2}″ — {:.0} seconds of tithi to {:.1} — over six centuries. What remains is the periodic part of the difference, which no polynomial can reach.\n",
                quadratic.bytes,
                quadratic.worst_before_arcsec,
                quadratic.worst_after_arcsec,
                quadratic.worst_before_arcsec * 2.25,
                quadratic.worst_after_tithi_seconds
            );
        }
        let _ = writeln!(
            out,
            "The best of them is degree {} at {:.2}″.\n",
            best.degree, best.worst_after_arcsec
        );
    }

    out.push_str(&fitted_route(fitted));

    let _ = writeln!(out, "### Narrow what `standard` claims\n");
    let _ = writeln!(
        out,
        "The span table above is this route's price list. The theory holds under four seconds of tithi over 1900 to 2100 and under eight over 1850 to 2150, so a tier that claims three centuries rather than six needs nothing built at all — it needs the claim to say so.\n"
    );
    out
}

/// The route that replaces the theory with a fitted table.
fn fitted_route(fitted: Option<&FitRecord>) -> String {
    let mut out = String::new();
    let Some(fitted) = fitted else {
        return out;
    };
    let _ = writeln!(out, "### Replace the theory with a fitted table\n");
    let _ = writeln!(
        out,
        "ADR-0021 names a Chebyshev refit from a modern kernel as the `reference` tier. It has an arithmetic problem the ADR does not price: **an analytic theory costs the same whatever span it is asked for, and a fitted table costs one block per interval.** The Moon circles in 27 days where Jupiter takes twelve years, so the Moon is where that bites.\n"
    );
    let _ = writeln!(
        out,
        "| interval | coefficients | worst | size over 1800 to 2400 |"
    );
    let _ = writeln!(out, "|---:|---:|---:|---:|");
    for row in &fitted.rows {
        let _ = writeln!(
            out,
            "| {:.0} days | {} | {} | {} |",
            row.interval_days,
            row.coefficients,
            if row.worst_arcsec < 0.0001 {
                "under 0.0001″".to_string()
            } else {
                format!("{:.4}″", row.worst_arcsec)
            },
            if row.bytes >= 1024 * 1024 {
                format!("{:.2} MB", kilobytes(row.bytes) / 1024.0)
            } else {
                format!("{:.0} KB", kilobytes(row.bytes))
            }
        );
    }
    let cheapest = fitted
        .rows
        .iter()
        .filter(|row| row.worst_arcsec <= 0.445)
        .min_by_key(|row| row.bytes);
    if let Some(row) = cheapest {
        let _ = writeln!(
            out,
            "\n**The cheapest fit that reaches the second-accurate budget is {:.2} MB** — {:.0}-day blocks of {} coefficients at {:.3}″. ADR-0021 budgets about 1 MB for *every* body at this tier; the Moon alone is several times that, so it is the ADR's ladder that has to move and not the measurement.\n",
            kilobytes(row.bytes) / 1024.0,
            row.interval_days,
            row.coefficients,
            row.worst_arcsec
        );
    }
    let _ = writeln!(
        out,
        "The error is the fit's **representation** error against the theory it is fitted to, which is what a table designer chooses. It is deliberately not accuracy against an ephemeris: how smooth the Moon is, and so how well a polynomial catches it, is the same question whichever modern source it comes from.\n"
    );
    out
}

/// The claims, and what each measures to.
fn claims_section(worst: f64, moon: Option<&MoonFloor>) -> String {
    let mut claims = vec![
        Claim::stated(
            "the Moon decides the tiers, not the planets",
            verdict_of(true),
            format!("one arcsecond of Moon is {worst:.2} s of tithi; one arcsecond of a planet is a position nobody times"),
        ),
        Claim::stated(
            "a second-accurate panchanga is reachable with an analytic theory",
            verdict_of(metres_as_seconds(2.4, worst) < 1.0),
            format!("ELP/MPP02's published 2.4 m is {:.3} s", metres_as_seconds(2.4, worst)),
        ),
        Claim::stated(
            "the chosen theory can be obtained",
            verdict_of(false),
            "ELP/MPP02 is published at an address that no longer serves it, was never captured by the archive, and reaches the public only through copyleft re-implementations that ADR-0019 refuses".to_string(),
        ),
    ];
    if let Some(moon) = moon {
        let standard = moon.spans.iter().find(|span| span.label == "1800 to 2400");
        if let Some(span) = standard {
            claims.push(Claim::stated(
                "`standard` holds 2 arcseconds for the Moon over 1800 to 2400",
                verdict_of(span.worst_arcsec <= 2.0),
                format!(
                    "ELP2000-82B, every term kept, is {:.2}″ — {:.0} s of tithi",
                    span.worst_arcsec, span.worst_tithi_seconds
                ),
            ));
        }
        let cheapest = moon
            .rows
            .iter()
            .filter(|row| {
                row.worst_arcsec
                    <= moon.rows.last().map_or(f64::MAX, |last| last.worst_arcsec) * 1.01
            })
            .min_by_key(|row| row.bytes);
        if let Some(row) = cheapest {
            claims.push(Claim::stated(
                "the Moon's table is a cost worth optimising",
                verdict_of(false),
                format!(
                    "the theory's own accuracy is reached at {:.0} KB; the remaining {} terms buy nothing",
                    kilobytes(row.bytes),
                    count(moon.whole_theory_terms - row.terms)
                ),
            ));
        }
    }
    let mut out = String::new();
    let _ = writeln!(out, "## What the claims measure to\n");
    let _ = writeln!(out, "{}", table(&claims));
    out
}

/// What the page cannot see.
fn limits_section(measured: bool) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "\n## What this does not measure\n");
    if measured {
        let _ = writeln!(
            out,
            "**What a better lunar source would cost.** ELP/MPP02 cannot be had; what remains is the `reference` tier's refit from a modern kernel, or `ephemeris-de` reading one directly, and neither is sized here. That is the next decision, and it is an ADR rather than a build step.\n"
        );
    } else {
        let _ = writeln!(
            out,
            "**What ELP2000-82B actually costs.** Its accuracy against a modern ephemeris is the next measurement, and the floor harness already exists: it is the same comparison the planets had, in the same isolated frame.\n"
        );
    }
    let _ = writeln!(
        out,
        "**The rate distribution.** The table uses the slowest the quantity ever moves, which is the worst case and is what a bound needs. A typical boundary moves less, and a page that wanted the typical figure would have to sweep real rates rather than read the catalogued extreme.\n"
    );
    let _ = writeln!(
        out,
        "**Latitude and distance.** Only longitude moves a panchanga boundary. The Moon's latitude decides eclipses and its distance decides the parallax that moves a rising, and both have budgets of their own that this page does not set.\n"
    );
    out
}

/// One metre at the Moon's mean distance, in arcseconds.
fn arcsec_per_metre() -> f64 {
    const MEAN_DISTANCE_M: f64 = 384_400_000.0;
    const ARCSEC_PER_RAD: f64 = 206_264.806_247_096_36;
    ARCSEC_PER_RAD / MEAN_DISTANCE_M
}

/// A distance error at the Moon, as seconds of boundary movement.
fn metres_as_seconds(metres: f64, seconds_per_arcsec: f64) -> f64 {
    metres * arcsec_per_metre() * seconds_per_arcsec
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The conversion is the whole page, so it is checked against a
    /// figure worked by hand: the Moon alone moves about 13.176°/day at
    /// its mean rate, and 24/13.176 is about 1.82 seconds per arcsecond.
    #[test]
    fn an_arcsecond_is_a_day_over_the_rate() {
        let seconds = seconds_per_arcsec(13.176);
        assert!(
            (seconds - 1.821).abs() < 0.002,
            "13.176 degrees a day gives {seconds} seconds an arcsecond"
        );
    }

    /// A slower quantity costs more seconds for the same error, which is
    /// why the tithi and not the nakshatra sets the budget.
    #[test]
    fn a_slower_quantity_costs_more_seconds() {
        let nakshatra = quantity_least_rate(Quantity::Longitude(Body::Moon)).expect("catalogued");
        let tithi = quantity_least_rate(Quantity::ELONGATION).expect("catalogued");
        assert!(tithi < nakshatra, "the elongation is the slower quantity");
        assert!(
            seconds_per_arcsec(tithi) > seconds_per_arcsec(nakshatra),
            "so it turns the same arcsecond into more seconds"
        );
    }

    /// A metre at the Moon is a small angle, and the page's whole
    /// comparison of theories rests on it.
    #[test]
    fn a_metre_at_the_moon_is_half_a_milliarcsecond() {
        let value = arcsec_per_metre();
        assert!(
            (value - 5.365e-4).abs() < 1e-6,
            "one metre subtends {value} arcseconds"
        );
    }
}

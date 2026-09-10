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
use crate::measure::{Claim, fill, table, verdict_of};

const PAGE: &str = "docs/03-design/lunar-accuracy-measured.md";

/// Where the Sun's measured error is recorded.
const FLOOR: &str = "crates/ephemeris-builtin/data/vsop87-floor.json";

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

/// Writes the page.
pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &[Output::new(PAGE, page(solar_error_arcsec(root)))])
}

/// The page, as the sections it is made of.
fn page(solar_arcsec: Option<f64>) -> String {
    let (sensitivity, worst) = sensitivity_section();
    let mut out = String::new();
    out.push_str(&opening());
    out.push_str(&sensitivity);
    out.push_str(&budget_section(worst, solar_arcsec));
    out.push_str(&candidates_section(worst));
    out.push_str(&finding_section(solar_arcsec));
    out.push_str(&claims_section(worst));
    out.push_str(&limits_section());
    fill(&out)
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

/// The claims, and what each measures to.
fn claims_section(worst: f64) -> String {
    let claims = vec![
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
    let mut out = String::new();
    let _ = writeln!(out, "## What the claims measure to\n");
    let _ = writeln!(out, "{}", table(&claims));
    out
}

/// What the page cannot see.
fn limits_section() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "\n## What this does not measure\n");
    let _ = writeln!(
        out,
        "**What ELP2000-82B actually costs.** Its accuracy against a modern ephemeris is the next measurement, and the floor harness already exists: it is the same comparison the planets had, in the same isolated frame.\n"
    );
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

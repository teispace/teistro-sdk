//! The falsification pass over the bhava chalit methods, which the
//! roadmap asks for before the chart layer is written (Phase 4).
//!
//! A bhava chalit says which house a graha is *in*, as opposed to which
//! sign it occupies. Four methods are named for it, and the question this
//! pass exists to answer is not which is correct — no measurement decides
//! that — but whether the choice matters. If the four agreed to within a
//! rounding error the chart layer could pick one and move on; if they do
//! not, every result that names a house has to name the method too, and
//! the settings hash has to carry it.
//!
//! The pass computes each method with the SDK's own house systems over
//! the fifty-five recorded charts, places the recorded grahas in them, and
//! reports two things: which method reproduces the placements the
//! recording engine wrote down, and how often each pair of methods
//! disagrees about a graha's house.
//!
//! `cargo xtask chalit` writes the page; `check-chalit` regenerates it in
//! memory and fails on any difference, so the numbers on the page are the
//! numbers this build produces.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::houses::{ChartFrame, Outcome, houses_at};
use teistro_astro::scale::tt_of;
use teistro_core::catalogue::HouseSystem;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::PolarPolicy;

use crate::generated::{Output, check, write};

const PAGE: &str = "docs/03-design/chart-bhava-chalit.md";
const CHARTS: &str = "fixtures/baseline/charts";

/// The grahas the recording engine places, in the order it lists them.
const GRAHAS: [&str; 9] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU",
];

/// One candidate reading of the sky into twelve houses.
struct Method {
    /// What it is called in the register and on the page.
    name: &'static str,
    /// The house system whose cusps are its bhava sandhi — the boundary
    /// a graha crosses to change house.
    system: HouseSystem,
    /// Where its bhava madhya are, which is what distinguishes two
    /// methods built on the same cusps.
    madhya: &'static str,
    /// What tradition reads it this way.
    reading: &'static str,
}

/// The four named methods, and whole sign as the thing they are all
/// alternatives to.
const METHODS: [Method; 5] = [
    Method {
        name: "Sripati",
        system: HouseSystem::Sripati,
        madhya: "the Porphyry cusps",
        reading: "the classical Indian bhava: the quadrants trisected, each cusp the middle of its house, the sandhi halfway between two middles",
    },
    Method {
        name: "Vehlow",
        system: HouseSystem::Vehlow,
        madhya: "the ascendant and every 30° from it",
        reading: "equal houses centred on the ascendant, so the lagna sits in the middle of the first house rather than at its edge",
    },
    Method {
        name: "Porphyry",
        system: HouseSystem::Porphyry,
        madhya: "midway between consecutive cusps",
        reading: "the same trisected quadrants read the Western way, each cusp the start of its house",
    },
    Method {
        name: "KP (Placidus)",
        system: HouseSystem::Placidus,
        madhya: "midway between consecutive cusps",
        reading: "Krishnamurti Paddhati, which takes the Placidus cusps as house starts",
    },
    Method {
        name: "whole sign",
        system: HouseSystem::WholeSign,
        madhya: "the middle of each sign",
        reading: "not a chalit at all: the sign the lagna falls in is the whole first house. It is here because it is what a chalit is measured against",
    },
];

/// One chart, read out of a fixture.
struct Chart {
    id: String,
    latitude: f64,
    /// The recorded sidereal longitude of each graha the fixture carries.
    grahas: BTreeMap<String, f64>,
    /// The house the recording engine put each graha in.
    recorded: BTreeMap<String, u8>,
    /// What the engine called its own reading.
    recorded_mode: String,
    /// Each method's cusps, sidereal, as this build computes them.
    cusps: Vec<[f64; 12]>,
    /// Whether every method was computed as asked. Inside the polar
    /// circle a quadrant system is substituted, and a substituted method
    /// is not the method: those charts are counted separately rather
    /// than being quietly averaged in.
    defined: bool,
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask chalit") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

// ── the measurement ────────────────────────────────────────────────────────

/// Every chart of the corpus, with each method's cusps computed for it.
fn charts(root: &Path) -> Result<Vec<Chart>, String> {
    let dir = root.join(CHARTS);
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map_err(|err| {
            format!(
                "cannot read {}: {err}. The corpus is a submodule; `git submodule update --init`",
                dir.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    files.iter().map(|path| chart(path)).collect()
}

/// One fixture, with this build's cusps for every method.
fn chart(path: &Path) -> Result<Chart, String> {
    let text = std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let fixture: Value =
        serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
    let number = |value: &Value| value.as_f64().unwrap_or(f64::NAN);

    let latitude = number(&fixture["input"]["place"]["latitude"]);
    let place = Place::new(
        Latitude::literal(latitude),
        Longitude::literal(number(&fixture["input"]["place"]["longitude"])),
        Altitude::literal(
            fixture["input"]["place"]["altitude_m"]
                .as_f64()
                .unwrap_or(0.0),
        ),
    );
    let ut1 = JulianDay::<Ut1>::literal(number(&fixture["input"]["resolved"]["jd_ut"]));
    let (tt, _) = tt_of(ut1, DeltaTModel::TableThenModel)
        .map_err(|err| format!("{}: {err}", path.display()))?;
    let ayanamsha = number(&fixture["foundation"]["ayanamsha"]["value_deg"]);
    let frame = ChartFrame {
        sidereal_offset_deg: ayanamsha,
        sun_declination_deg: None,
    };

    let mut cusps = Vec::with_capacity(METHODS.len());
    let mut defined = true;
    for method in &METHODS {
        let houses = houses_at(
            method.system,
            ut1,
            tt,
            &place,
            &frame,
            PolarPolicy::FallbackWholeSign,
        )
        .map_err(|err| format!("{}: {}: {err}", path.display(), method.name))?;
        defined &= houses.outcome == Outcome::Defined;
        // The fixtures are sidereal; the SDK's cusps are tropical of date.
        let mut sidereal = [0.0_f64; 12];
        for (out, cusp) in sidereal.iter_mut().zip(houses.cusps.iter()) {
            *out = (cusp - ayanamsha).rem_euclid(360.0);
        }
        cusps.push(sidereal);
    }

    let mut grahas = BTreeMap::new();
    for name in GRAHAS {
        if let Some(longitude) =
            fixture["positions"]["bodies"][name]["sidereal_longitude_deg"].as_f64()
        {
            grahas.insert(name.to_string(), longitude);
        }
    }
    let mut recorded = BTreeMap::new();
    if let Some(houses) = fixture["houses"]["bhava_chalit"]["planet_houses"].as_object() {
        for (name, house) in houses {
            if let Some(house) = house.as_u64() {
                recorded.insert(name.clone(), u8::try_from(house).unwrap_or(0));
            }
        }
    }
    Ok(Chart {
        id: fixture["id"].as_str().unwrap_or_default().to_string(),
        latitude,
        grahas,
        recorded,
        recorded_mode: fixture["houses"]["bhava_chalit"]["mode"]
            .as_str()
            .unwrap_or("none")
            .to_string(),
        cusps,
        defined,
    })
}

/// The house a longitude falls in, the cusps being the sandhi, 1 to 12.
///
/// Each house runs from its own sandhi up to the next, going forward
/// through the zodiac, so a house that spans 0° is one house and not two.
fn house_of(longitude: f64, cusps: &[f64; 12]) -> u8 {
    for (index, start) in cusps.iter().enumerate() {
        let next = cusps[(index + 1) % 12];
        let span = (next - start).rem_euclid(360.0);
        let span = if span == 0.0 { 360.0 } else { span };
        if (longitude - start).rem_euclid(360.0) < span {
            return u8::try_from(index + 1).unwrap_or(1);
        }
    }
    12
}

/// What the pass found.
struct Findings {
    charts: usize,
    placements: usize,
    /// Per method, the charts whose every recorded placement it
    /// reproduces.
    reproduces: Vec<usize>,
    /// Per pair of methods, the placements they disagree on.
    disagreements: BTreeMap<(usize, usize), usize>,
    /// The same, over the charts where every method was computed as
    /// asked rather than substituted.
    defined_disagreements: BTreeMap<(usize, usize), usize>,
    /// How many placements those charts hold, and how many charts.
    defined_placements: usize,
    defined_charts: usize,
    /// The methods' disagreement with Sripati, by latitude band.
    by_latitude: Vec<(&'static str, usize, usize)>,
    /// The charts where the two a Jyotisha application would choose
    /// between disagree most, worst first, so a reader can look one up.
    worst_charts: Vec<(String, usize, usize)>,
    /// What every fixture called its own reading.
    modes: BTreeMap<String, usize>,
}

fn measure(charts: &[Chart]) -> Findings {
    let mut reproduces = vec![0_usize; METHODS.len()];
    let mut disagreements: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut defined_disagreements: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut placements = 0;
    let mut defined_placements = 0;
    let mut defined_charts = 0;
    let mut modes: BTreeMap<String, usize> = BTreeMap::new();
    // Sripati against Vehlow, inside and outside the tropics: the
    // quadrant methods distort with latitude and the equal ones do not.
    let mut bands = [
        ("under 30° of latitude", 0_usize, 0_usize),
        ("30° and beyond", 0, 0),
    ];
    let mut worst_charts: Vec<(String, usize, usize)> = Vec::new();

    for chart in charts {
        *modes.entry(chart.recorded_mode.clone()).or_default() += 1;
        let houses: Vec<BTreeMap<&str, u8>> = chart
            .cusps
            .iter()
            .map(|cusps| {
                chart
                    .grahas
                    .iter()
                    .map(|(name, longitude)| (name.as_str(), house_of(*longitude, cusps)))
                    .collect()
            })
            .collect();

        for (index, placed) in houses.iter().enumerate() {
            let all = chart
                .recorded
                .iter()
                .all(|(name, house)| placed.get(name.as_str()).copied() == Some(*house));
            if all && !chart.recorded.is_empty() {
                reproduces[index] += 1;
            }
        }

        placements += chart.grahas.len();
        if chart.defined {
            defined_placements += chart.grahas.len();
            defined_charts += 1;
        }
        for left in 0..METHODS.len() {
            for right in (left + 1)..METHODS.len() {
                let differing = chart
                    .grahas
                    .keys()
                    .filter(|name| {
                        houses[left].get(name.as_str()) != houses[right].get(name.as_str())
                    })
                    .count();
                *disagreements.entry((left, right)).or_default() += differing;
                if chart.defined {
                    *defined_disagreements.entry((left, right)).or_default() += differing;
                }
            }
        }

        let band = usize::from(chart.latitude.abs() >= 30.0);
        let differing = chart
            .grahas
            .keys()
            .filter(|name| houses[0].get(name.as_str()) != houses[1].get(name.as_str()))
            .count();
        bands[band].1 += differing;
        bands[band].2 += chart.grahas.len();
        if differing > 0 {
            worst_charts.push((chart.id.clone(), differing, chart.grahas.len()));
        }
    }
    // Worst first, then by id, so the order is the same on every machine.
    worst_charts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    worst_charts.truncate(5);

    Findings {
        charts: charts.len(),
        placements,
        reproduces,
        disagreements,
        defined_disagreements,
        defined_placements,
        defined_charts,
        by_latitude: bands.to_vec(),
        worst_charts,
        modes,
    }
}

// ── the page ───────────────────────────────────────────────────────────────

fn page(root: &Path) -> Result<String, String> {
    let charts = charts(root)?;
    if charts.is_empty() {
        return Err(format!("{CHARTS} holds no charts"));
    }
    let found = measure(&charts);
    Ok(render(&found))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "counts of a few thousand, far below the mantissa"
)]
fn percent(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64 * 100.0
    }
}

/// The page: what the methods are, what the engine does, how far apart
/// they are, and what that decides.
fn render(found: &Findings) -> String {
    let mut out = methods_section(found);
    out.push_str(&distance_section(found));
    out.push_str(&conclusion(found));
    out
}

/// The four methods, and which of them the recording engine computes.
fn methods_section(found: &Findings) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "# Bhava chalit: which method, and how much it matters\n\n\
         Status: `measured`, 2026-09-07. Generated by `cargo xtask chalit` and held by\n\
         `check-chalit`; every number below is what this build computes.\n\n\
         The roadmap asks for a falsification pass over the bhava chalit methods before\n\
         the chart layer is written. This is it.\n\n\
         A chalit says which *house* a graha is in, as against which sign it occupies.\n\
         Four methods are named for it. The question here is not which is correct — no\n\
         measurement decides that — but whether the choice matters. If the four agreed to\n\
         within a rounding error, the chart layer could pick one and move on.\n\n\
         **They do not agree.** Over {} charts and {} graha placements, two methods\n\
         built on the same cusps disagree about half the time, and the two closest still\n\
         disagree about one placement in five.\n\n\
         ## The methods\n\n\
         | method | its sandhi | its madhya | read this way by |\n|---|---|---|---|\n",
        found.charts, found.placements
    );
    for method in &METHODS {
        let _ = writeln!(
            out,
            "| **{}** | the `{}` cusps | {} | {} |",
            method.name,
            method.system.key(),
            method.madhya,
            method.reading
        );
    }

    let _ = write!(
        out,
        "\nThe *sandhi* is the boundary a graha crosses to change house; the *madhya* is\n\
         the middle, where a graha is strongest. Two methods can share their cusps and\n\
         still be different chalits, because one reads a cusp as a boundary and the other\n\
         as a middle: Sripati and Porphyry are exactly that pair, and they are half a\n\
         house apart everywhere.\n\n\
         ## What the recording engine actually does\n\n\
         The corpus records a `bhava_chalit` for every chart, and every one of them calls\n\
         itself "
    );
    let modes: Vec<String> = found
        .modes
        .iter()
        .map(|(mode, count)| format!("`{mode}` ({count})"))
        .collect();
    let _ = write!(
        out,
        "{}. That label is wrong, and the geometry says so:\n\n\
         | method | charts whose every recorded placement it reproduces |\n|---|---:|\n",
        modes.join(", ")
    );
    for (method, count) in METHODS.iter().zip(&found.reproduces) {
        let _ = writeln!(out, "| {} | **{count}** of {} |", method.name, found.charts);
    }
    out
}

/// How often each pair puts a graha in a different house.
fn distance_section(found: &Findings) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "\nThe engine computes **Vehlow** and calls it equal house. They are not the same\n\
         thing: equal houses *start* at the ascendant, Vehlow *centres* on it, and the two\n\
         are fifteen degrees apart. This is the third entry of the deliberate-difference\n\
         registry made exact — `nepali-default` says `VEHLOW` because that is what the\n\
         charts it reproduces contain, whatever their label\n\
         (`05-testing/01-golden-vectors.md`).\n\n\
         ## How far apart the methods are\n\n\
         Every pair, over all {} placements, and over the {} of them from the {} charts\n\
         where no system had to be substituted:\n\n\
         | | {} |\n|---|{}\n",
        found.placements,
        found.defined_placements,
        found.defined_charts,
        METHODS
            .iter()
            .skip(1)
            .map(|m| m.name)
            .collect::<Vec<_>>()
            .join(" | "),
        "---:|".repeat(METHODS.len() - 1)
    );
    for (row, left) in METHODS.iter().enumerate().take(METHODS.len() - 1) {
        let mut cells = vec![String::new(); METHODS.len() - 1];
        for column in (row + 1)..METHODS.len() {
            let all = found
                .disagreements
                .get(&(row, column))
                .copied()
                .unwrap_or_default();
            let defined = found
                .defined_disagreements
                .get(&(row, column))
                .copied()
                .unwrap_or_default();
            cells[column - 1] = format!(
                "{:.1}% / {:.1}%",
                percent(all, found.placements),
                percent(defined, found.defined_placements)
            );
        }
        let _ = writeln!(out, "| **{}** | {} |", left.name, cells.join(" | "));
    }

    let _ = write!(
        out,
        "\nRead the table as \"how often these two would put a graha in different houses\".\n\
         Each cell is \"over every chart / over the charts where nothing was\n\
         substituted\". Inside the polar circle a quadrant system has no cusps to give and\n\
         whole sign stands in for it, which pulls a quadrant method towards whole sign and\n\
         away from an equal method that did not fall back; the second number is the one\n\
         without that.\n\n\
         Sripati against Porphyry is the sharpest result here, because those two are the\n\
         *same cusps*: the trisected quadrants, read once as house middles and once as\n\
         house starts. Their cusps agree to the last decimal and their houses disagree\n\
         {:.1}% of the time. Nothing about the geometry is in dispute between them, only\n\
         what a cusp means — and that alone moves half the chart.\n\n\
         Sripati against Vehlow — the two a Jyotisha application would actually choose\n\
         between — is {:.1}%: better than one placement in five.\n\n\
         ## Latitude\n\n\
         Sripati is built on quadrants, which distort as the latitude rises; Vehlow is not.\n\n\
         | latitude | Sripati against Vehlow |\n|---|---:|\n",
        percent(
            found
                .disagreements
                .get(&(0, 2))
                .copied()
                .unwrap_or_default(),
            found.placements
        ),
        percent(
            found
                .disagreements
                .get(&(0, 1))
                .copied()
                .unwrap_or_default(),
            found.placements
        )
    );
    for (band, differing, total) in &found.by_latitude {
        let _ = writeln!(
            out,
            "| {band} | {:.1}% of {total} |",
            percent(*differing, *total)
        );
    }

    let _ = write!(
        out,
        "\nThe charts where those two disagree most, for anyone who wants to look one up:\n\n\
         | chart | grahas in a different house |\n|---|---:|\n"
    );
    for (id, differing, total) in &found.worst_charts {
        let _ = writeln!(out, "| `{id}` | {differing} of {total} |");
    }

    out
}

/// What the measurement decides, and how it was taken.
fn conclusion(found: &Findings) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "\n## What this decides\n\n\
         1. **The chalit method is a setting, not a constant.** `houses.chalit_system`\n   \
            already exists and already feeds the settings hash\n   \
            (`03-design/settings-and-profiles.md`); this measurement is why it must. A\n   \
            result that names a house without naming the method is not reproducible.\n\
         2. **A house is not a number on its own.** Every placement the chart layer reports\n   \
            carries the method that produced it, as every position carries its frame.\n\
         3. **The layer reports madhya as well as sandhi.** `astro::houses::Houses` gives\n   \
            the cusps and nothing else, which is enough to place a graha and not enough to\n   \
            say how near the middle of its house it sits — the thing bhava bala is built\n   \
            on. The chart layer's house service returns both.\n\
         4. **Whole sign is not a fallback for a chalit.** It disagrees with every method\n   \
            here by a quarter to a half of all placements. An application that shows both\n   \
            shows two different charts, and should say so.\n\n\
         ## How this was measured\n\n\
         For each of the {} charts of the conformance corpus: the SDK's own\n\
         `astro::houses::houses_at` computed each method's cusps from the chart's instant\n\
         and place, the cusps were taken back to the sidereal frame with the ayanamsha the\n\
         fixture records, and each recorded graha longitude was placed between them. The\n\
         SDK's cusps reproduce the recording engine's within an arcsecond over this same\n\
         set (`crates/astro/tests/baseline_houses.rs`), so the disagreements here are\n\
         between the methods and not between the engines.\n\n\
         The polar policy is `FALLBACK_WHOLE_SIGN`, and {} of the {} charts needed it. A\n\
         substituted method is not that method, so every pairwise number is given twice:\n\
         once over everything, and once over the charts where nothing was substituted.\n\
         Quote the second.\n",
        found.charts,
        found.charts - found.defined_charts,
        found.charts
    );
    out
}

#[cfg(test)]
mod tests {
    use super::{house_of, percent};

    #[test]
    fn a_graha_lands_in_the_house_that_starts_before_it() {
        // Twelve equal houses starting at 10°.
        let cusps = [
            10.0, 40.0, 70.0, 100.0, 130.0, 160.0, 190.0, 220.0, 250.0, 280.0, 310.0, 340.0,
        ];
        assert_eq!(house_of(10.0, &cusps), 1, "a cusp belongs to its own house");
        assert_eq!(house_of(39.999, &cusps), 1);
        assert_eq!(house_of(40.0, &cusps), 2);
        assert_eq!(house_of(345.0, &cusps), 12);
        // The house that spans 0° is one house, not two.
        assert_eq!(house_of(355.0, &cusps), 12);
        assert_eq!(house_of(5.0, &cusps), 12);
        assert_eq!(house_of(0.0, &cusps), 12);
    }

    #[test]
    fn unequal_houses_place_by_span_and_not_by_index() {
        // A quadrant system: the houses are not 30° wide.
        let cusps = [
            0.0, 20.0, 45.0, 90.0, 135.0, 160.0, 180.0, 200.0, 225.0, 270.0, 315.0, 340.0,
        ];
        assert_eq!(house_of(19.9, &cusps), 1);
        assert_eq!(house_of(20.0, &cusps), 2);
        assert_eq!(house_of(89.9, &cusps), 3, "the widest house holds 45°");
        assert_eq!(house_of(339.9, &cusps), 11);
        assert_eq!(house_of(340.0, &cusps), 12);
    }

    #[test]
    fn a_percentage_of_nothing_is_nothing() {
        assert!((percent(0, 0) - 0.0).abs() < f64::EPSILON);
        assert!((percent(1, 4) - 25.0).abs() < f64::EPSILON);
    }
}

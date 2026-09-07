//! The falsification pass over the recorded houses, which the houses
//! module is designed from (Phase 4).
//!
//! Most of `houses.*` already has a reader. `astro`'s own baseline test
//! compares all twenty-two systems' cusps, `chart`'s compares the
//! chalit's madhya, sandhi and every placement, and `cargo xtask chalit`
//! measured how far the four chalit methods stand apart. What this pass
//! is for is the part **nothing has read**, which turned out to be the
//! part that matters for a service:
//!
//! 1. `selected.is_degenerate` — the engine's own flag for a system
//!    with no solution at the place. Never compared with anything.
//! 2. `selected.cusp_sign_index` and `selected.mc` — never compared.
//! 3. The **completeness** of `bhava_chalit.shifted`. `chart`'s test
//!    checks every body the engine listed; nothing checks that the SDK
//!    lists no others.
//! 4. `houses.module_overrides`, a settings knob with no reader at all —
//!    the same shape of gap that made a chart founded on the SDK's own
//!    default profile fail when `state` was built (registry entry 23).
//!
//! `cargo xtask houses` writes the page; `check-houses` regenerates it
//! in memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::houses::{ChartFrame, Outcome, houses_at};
use teistro_astro::scale::tt_of;
use teistro_core::catalogue::{Degeneracy, HouseSystem};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::{PolarPolicy, Profile, SHIPPED_PROFILES, SettingsPatch};

use crate::classical::{GRAHAS, sign_of};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table, verdict_of};

const PAGE: &str = "docs/03-design/houses-measured.md";
const CHARTS: &str = "fixtures/baseline/charts";
const VARIANTS: &str = "fixtures/baseline/variants";

/// The obliquity the polar test is taken at, degrees. The SDK's own
/// `is_polar` takes the obliquity of the date; this is the round figure
/// a reader checks a latitude against.
const OBLIQUITY_DEG: f64 = 23.4393;

/// How near two longitudes must agree to be the same, degrees. The
/// engine and the SDK compute an angle over different libraries, and
/// `astro`'s own baseline test bounds the cusps at this scale.
const SAME_DEG: f64 = 0.01;

/// One fixture's houses, in the shapes this pass measures.
struct Reading {
    fixture: String,
    system: HouseSystem,
    cusps: Vec<f64>,
    cusp_signs: Vec<u8>,
    ascendant: f64,
    mc: f64,
    degenerate: bool,
    latitude: f64,
    place: Place,
    jd: f64,
    /// The ayanamsha the fixture recorded, where it recorded one. A
    /// variant that carries only its houses does not, and the frame's
    /// offset changes no outcome — only the longitudes it is read in.
    ayanamsha: Option<f64>,
    /// The bodies the engine says the chalit moves, and where to.
    shifted: Vec<(String, u8, u8)>,
    /// Every graha's sidereal longitude, for the shift's completeness.
    longitudes: BTreeMap<String, f64>,
    /// The chalit's own sandhi, which the shift is measured over.
    sandhi: Vec<f64>,
}

fn readings(root: &Path) -> Result<Vec<Reading>, String> {
    let mut found = Vec::new();
    for directory in [CHARTS, VARIANTS] {
        let dir = root.join(directory);
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
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
        paths.sort();
        for path in &paths {
            let text = std::fs::read_to_string(path)
                .map_err(|err| format!("{}: {err}", path.display()))?;
            let value: Value =
                serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
            let name = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Some(reading) = read(&name, &value) {
                found.push(reading);
            }
        }
    }
    if found.is_empty() {
        return Err(String::from("no fixture carries a selected house system"));
    }
    Ok(found)
}

/// The house system the corpus's own name for it stands for. The engine
/// writes the catalogue's key in lower case with hyphens.
fn system_of(name: &str) -> Option<HouseSystem> {
    HouseSystem::ALL
        .into_iter()
        .find(|system| system.key().to_lowercase().replace('_', "-") == name)
}

fn read(name: &str, fixture: &Value) -> Option<Reading> {
    let selected = fixture["houses"]["selected"].as_object()?;
    // A variant may carry its houses and nothing else, which is exactly
    // where the two degenerate charts of the corpus live: the instant
    // and the place come from the input every fixture has.
    let foundation = &fixture["foundation"];
    let place = &fixture["input"]["place"];
    let latitude = place["latitude"].as_f64()?;
    let chalit = &fixture["houses"]["bhava_chalit"];
    // The same two variants carry no positions either, so the shift of
    // §4 simply has nothing to say about them.
    let bodies = fixture["positions"]["bodies"].as_object();
    Some(Reading {
        fixture: name.to_string(),
        system: system_of(selected.get("system")?.as_str()?)?,
        cusps: numbers(selected.get("cusps")?),
        cusp_signs: selected
            .get("cusp_sign_index")?
            .as_array()?
            .iter()
            .filter_map(|value| u8::try_from(value.as_u64()?).ok())
            .collect(),
        ascendant: selected.get("ascendant")?.as_f64()?,
        mc: selected.get("mc")?.as_f64()?,
        degenerate: selected
            .get("is_degenerate")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        latitude,
        place: Place::new(
            Latitude::try_new(latitude).ok()?,
            Longitude::try_new(place["longitude"].as_f64()?).ok()?,
            Altitude::try_new(place["altitude_m"].as_f64().unwrap_or(0.0)).ok()?,
        ),
        jd: foundation["jd_ut"]
            .as_f64()
            .or_else(|| fixture["input"]["resolved"]["jd_ut"].as_f64())?,
        ayanamsha: foundation["ayanamsha"]["value_deg"].as_f64(),
        shifted: chalit["shifted"]
            .as_array()
            .map(|list| {
                list.iter()
                    .filter_map(|entry| {
                        Some((
                            entry["planet"].as_str()?.to_string(),
                            u8::try_from(entry["chalit_house"].as_u64()?).ok()?,
                            u8::try_from(entry["whole_sign_house"].as_u64()?).ok()?,
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        longitudes: bodies
            .map(|bodies| {
                GRAHAS
                    .into_iter()
                    .filter_map(|body| {
                        Some((
                            body.to_string(),
                            bodies.get(body)?["sidereal_longitude_deg"].as_f64()?,
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default(),
        sandhi: numbers(&chalit["bhava_sandhi"]),
    })
}

fn numbers(value: &Value) -> Vec<f64> {
    value
        .as_array()
        .map(|list| list.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default()
}

/// The angle between two longitudes, degrees, the shorter way round.
fn apart(first: f64, second: f64) -> f64 {
    let gap = (first - second).abs().rem_euclid(360.0);
    gap.min(360.0 - gap)
}

/// What the SDK makes of a fixture's own selected system: the outcome,
/// the system whose cusps came back (a substitute is not the one asked
/// for) and the midheaven in the fixture's own frame.
fn outcome_of(
    reading: &Reading,
    policy: PolarPolicy,
) -> Result<(Outcome, HouseSystem, f64), String> {
    let ut1 = JulianDay::<Ut1>::literal(reading.jd);
    let (tt, _) = tt_of(ut1, DeltaTModel::TableThenModel)
        .map_err(|err| format!("{}: {err}", reading.fixture))?;
    let ayanamsha = reading.ayanamsha.unwrap_or(0.0);
    let frame = ChartFrame {
        sidereal_offset_deg: ayanamsha,
        sun_declination_deg: None,
    };
    let houses = houses_at(reading.system, ut1, tt, &reading.place, &frame, policy)
        .map_err(|err| format!("{}: {err}", reading.fixture))?;
    let mc = (houses.angles.midheaven_deg - ayanamsha).rem_euclid(360.0);
    Ok((houses.outcome, houses.system, mc))
}

// ── the page ───────────────────────────────────────────────────────────────

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask houses") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn page(root: &Path) -> Result<String, String> {
    let readings = readings(root)?;
    let sections = [
        header(&readings),
        cusps(&readings)?,
        degeneracy(&readings)?,
        shifts(&readings),
        overrides(),
        decides(&readings)?,
    ];
    Ok(fill(&sections.concat()))
}

// ── 1. what nothing has read ───────────────────────────────────────────────

fn header(readings: &[Reading]) -> String {
    let mut chosen: BTreeMap<&str, usize> = BTreeMap::new();
    for reading in readings {
        *chosen.entry(reading.system.key()).or_default() += 1;
    }
    let shown: Vec<String> = chosen
        .iter()
        .map(|(system, seen)| format!("`{system}` on {seen}"))
        .collect();
    format!(
        "# The houses, measured\n\n\
         Status: `generated` by `cargo xtask houses` over the conformance\n\
         corpus's `houses` sections, 2026-09-07. Do not edit:\n\
         `check-houses` regenerates this page and fails on any difference.\n\
         The design written from it is\n\
         [`houses-service.md`](houses-service.md).\n\n\
         ## 1. What nothing has read\n\n\
         Most of this section already has a reader. `astro`'s own baseline\n\
         test compares all twenty-two systems' cusps, `chart`'s compares the\n\
         chalit's madhya, its sandhi and every placement, and `cargo xtask\n\
         chalit` measured how far the four chalit methods stand apart. So\n\
         this pass is not a second look at those: it is the first look at\n\
         the four things nothing has read, which turn out to be the ones a\n\
         **service** over the geometry has to get right.\n\n\
         {} fixtures carry a selected system: {}.\n\n",
        count(readings.len()),
        shown.join(", "),
    )
}

// ── 2. the cusps' signs and the midheaven ──────────────────────────────────

fn cusps(readings: &[Reading]) -> Result<String, String> {
    let mut wrong_signs = 0;
    let mut cells = 0;
    let mut mc_worst = 0.0_f64;
    let mut mc_seen = 0;
    let mut ascendant_worst = 0.0_f64;
    for reading in readings {
        for (cusp, recorded) in reading.cusps.iter().zip(&reading.cusp_signs) {
            cells += 1;
            wrong_signs += usize::from(sign_of(*cusp) != *recorded);
        }
        // The midheaven, which nothing has compared: the ascendant beside
        // it is checked by `chart`, and is recomputed here as the control.
        // A fixture that records no ayanamsha cannot be compared in the
        // sidereal frame, and there is nothing to gain by guessing one.
        if reading.ayanamsha.is_some() {
            let (_, _, mc) = outcome_of(reading, PolarPolicy::FallbackWholeSign)?;
            mc_worst = mc_worst.max(apart(mc, reading.mc));
            mc_seen += 1;
        }
        ascendant_worst = ascendant_worst.max(apart(
            reading.cusps.first().copied().unwrap_or(0.0),
            reading.ascendant,
        ));
    }
    let claims = [
        Claim::counted(
            "a cusp's recorded sign is the sign its own longitude falls in",
            wrong_signs,
            cells,
        ),
        Claim::stated(
            "the recorded midheaven is the one the SDK computes",
            verdict_of(mc_worst <= SAME_DEG),
            format!("worst {mc_worst:.4}° over {mc_seen}"),
        ),
    ];
    Ok(format!(
        "## 2. The cusps' signs, and the midheaven\n\n\
         Two fields with no reader, and both hold. The sign index beside\n\
         each cusp is the sign that cusp's own longitude falls in — {} of\n\
         them, exactly, so it is a rendering and not a second reading. And\n\
         the midheaven the engine records for the selected system is the\n\
         one the SDK computes from the same instant and place, within\n\
         {mc_worst:.4}°, which is the scale `astro`'s own cusp comparison\n\
         works at.\n\n{}\n\
         The first cusp and the recorded ascendant stand as much as\n\
         {ascendant_worst:.4}° apart, which is not an error: under a\n\
         whole-sign chart the first cusp is the **start of the ascendant's\n\
         sign** and the ascendant is wherever inside it the ecliptic\n\
         actually rises, so the two differ by however far into its sign the\n\
         ascendant stands. A service that reports \"the first house begins\n\
         here\" and \"the ascendant is here\" is reporting two different\n\
         numbers, and the design keeps them apart for that reason.\n\n",
        count(cells),
        table(&claims),
    ))
}

// ── 3. the degeneracy flag ─────────────────────────────────────────────────

/// What §3 finds about the engine's own degeneracy flag.
struct Degenerate {
    /// Fixtures the engine flags, with their latitude and system.
    flagged: Vec<(String, f64, &'static str)>,
    /// Fixtures the engine flags and the SDK computes without trouble.
    flagged_but_defined: Vec<(String, f64, &'static str)>,
    /// Fixtures the engine leaves clear and the SDK could not compute
    /// as asked, with what it did instead.
    clear_but_not_defined: Vec<(String, f64, &'static str, String)>,
    seen: usize,
    /// Fixtures whose flagged system the catalogue does not call polar.
    unexpected: usize,
    /// The highest latitude the engine leaves unflagged, and the lowest
    /// it flags, on a system the catalogue calls polar.
    highest_clear: f64,
    lowest_flagged: f64,
}

fn measure_degeneracy(readings: &[Reading]) -> Result<Degenerate, String> {
    let mut found = Degenerate {
        flagged: Vec::new(),
        flagged_but_defined: Vec::new(),
        clear_but_not_defined: Vec::new(),
        seen: 0,
        unexpected: 0,
        highest_clear: 0.0,
        lowest_flagged: f64::INFINITY,
    };
    for reading in readings {
        found.seen += 1;
        let polar_system = reading.system.attributes().degeneracy == Degeneracy::PolarUndefined;
        // The policy that leaves the outcome visible rather than
        // refusing: what is measured is whether the SDK *noticed*.
        let (outcome, gave, _) = outcome_of(reading, PolarPolicy::FallbackWholeSign)?;
        let ours = outcome != Outcome::Defined;
        match (reading.degenerate, ours) {
            (true, false) => found.flagged_but_defined.push((
                reading.fixture.clone(),
                reading.latitude,
                reading.system.key(),
            )),
            (false, true) => found.clear_but_not_defined.push((
                reading.fixture.clone(),
                reading.latitude,
                reading.system.key(),
                match outcome {
                    Outcome::Substituted { asked } => {
                        format!("`{}` stood in for `{}`", gave.key(), asked.key())
                    }
                    Outcome::Clamped { asked_latitude_deg } => {
                        format!("computed at a latitude clamped from {asked_latitude_deg:.4}°")
                    }
                    Outcome::Defined => String::new(),
                },
            )),
            _ => {}
        }
        if reading.degenerate {
            found.flagged.push((
                reading.fixture.clone(),
                reading.latitude,
                reading.system.key(),
            ));
            found.unexpected += usize::from(!polar_system);
            found.lowest_flagged = found.lowest_flagged.min(reading.latitude.abs());
        } else if polar_system {
            found.highest_clear = found.highest_clear.max(reading.latitude.abs());
        }
    }
    Ok(found)
}

fn degeneracy(readings: &[Reading]) -> Result<String, String> {
    let found = measure_degeneracy(readings)?;
    let polar_at = 90.0 - OBLIQUITY_DEG;
    let claims = [
        Claim::counted(
            "a flagged chart's system is one the catalogue calls polar",
            found.unexpected,
            found.flagged.len(),
        ),
        Claim::counted(
            "the engine's flag is the SDK's own outcome",
            found.flagged_but_defined.len() + found.clear_but_not_defined.len(),
            found.seen,
        ),
        Claim::stated(
            format!("a chart is flagged only inside the polar circle ({polar_at:.2}°)"),
            verdict_of(found.lowest_flagged.is_finite() && found.lowest_flagged >= polar_at),
            if found.lowest_flagged.is_finite() {
                format!("the lowest flagged is {:.4}°", found.lowest_flagged)
            } else {
                String::from("nothing tests it")
            },
        ),
    ];
    let mut out = format!(
        "## 3. The degeneracy flag, which nothing had compared\n\n\
         `selected.is_degenerate` says the chosen system had no solution at\n\
         the place. It is the field this pass exists for: nothing in the SDK\n\
         had ever read it, and it is what a service has to report.\n\n{}\n\
         The engine flags {} of {} charts, and both are on a system the\n\
         catalogue calls `POLAR_UNDEFINED`, so the first claim holds and the\n\
         two agree that far.\n\n\
         | fixture | latitude | system |\n|---|---|---|\n",
        table(&claims),
        found.flagged.len(),
        count(found.seen),
    );
    for (fixture, latitude, system) in &found.flagged {
        let _ = writeln!(out, "| {fixture} | {latitude:.4}° | `{system}` |");
    }
    let _ = write!(
        out,
        "\nThe second and third claims are where they part, and they part in\n\
         **both directions**, which is the finding.\n\n\
         The polar circle for an obliquity of {OBLIQUITY_DEG:.4}° is\n\
         {polar_at:.2}°. The engine flags charts at {:.4}° — *below* it,\n\
         where the SDK computes Placidus without trouble — and leaves\n\
         charts clear at up to {:.4}°, *above* it, where the SDK cannot\n\
         compute the system asked for at all. So the two are not the same\n\
         quantity read to different precision; they disagree about which\n\
         charts are the difficult ones.\n\n\
         | the engine flags, the SDK computes | latitude | system |\n|---|---|---|\n",
        found.lowest_flagged, found.highest_clear,
    );
    for (fixture, latitude, system) in &found.flagged_but_defined {
        let _ = writeln!(out, "| {fixture} | {latitude:.4}° | `{system}` |");
    }
    out.push_str("\n| the engine leaves clear, the SDK cannot | latitude | what the SDK did |\n|---|---|---|\n");
    for (fixture, latitude, _, what) in &found.clear_but_not_defined {
        let _ = writeln!(out, "| {fixture} | {latitude:.4}° | {what} |");
    }
    out.push_str(
        "\nThat is a difference to carry rather than reconcile, and it is\n\
         the argument for the shape the module takes. A **boolean cannot\n\
         say what happened**: the SDK's outcome distinguishes the system\n\
         computed as asked, another standing in for it, and one computed at\n\
         a clamped latitude, and which of the three occurred is exactly\n\
         what a caller needs in order to decide whether to trust the chart.\n\
         The module reports the outcome and the policy that produced it; a\n\
         harness comparing against this corpus compares the flag with \"the\n\
         outcome was not `DEFINED`\" and allows the charts above, which the\n\
         deliberate-difference registry names.\n\n",
    );
    Ok(out)
}

// ── 4. the shift, counted both ways ────────────────────────────────────────

fn shifts(readings: &[Reading]) -> String {
    let mut listed = 0;
    let mut ours = 0;
    let mut missed = 0;
    let mut invented = 0;
    let mut seen = 0;
    let mut skipped = 0;
    let mut skipped_listed = 0;
    for reading in readings {
        if reading.sandhi.len() != 12 {
            continue;
        }
        // A fixture that carries its houses and no positions has bodies
        // to name and none to place, so it is counted as skipped rather
        // than quietly swelling the denominator.
        if reading.longitudes.is_empty() {
            skipped += 1;
            skipped_listed += reading.shifted.len();
            continue;
        }
        seen += 1;
        listed += reading.shifted.len();
        for (body, longitude) in &reading.longitudes {
            let chalit = house_of(*longitude, &reading.sandhi);
            let whole = whole_sign_house(*longitude, reading.ascendant);
            let shifts = chalit != whole;
            let engine_says = reading.shifted.iter().any(|(who, _, _)| who == body);
            ours += usize::from(shifts);
            missed += usize::from(engine_says && !shifts);
            invented += usize::from(shifts && !engine_says);
        }
    }
    let claims = [
        Claim::counted(
            "every body the engine lists as shifted does shift",
            missed,
            listed,
        ),
        Claim::counted("and no other body does", invented, ours),
        Claim::stated(
            "the two counts are the same set",
            verdict_of(listed == ours && missed == 0 && invented == 0),
            format!("{listed} listed, {ours} found"),
        ),
    ];
    format!(
        "## 4. The shift, counted the other way as well\n\n\
         The engine records which bodies the chalit moves out of their\n\
         whole-sign house. `chart`'s own test checks every body it lists;\n\
         what nothing checked is the other direction — that the SDK finds\n\
         **no others**. A rule that shifted one body too many would pass the\n\
         first check and fail a chart.\n\n{}\n\
         Over {} fixtures the engine lists {} shifted bodies and reading the\n\
         same sandhi the same way finds the same {}. Both directions are\n\
         counted because they are different mistakes: a missed shift hides a\n\
         placement, and an invented one moves a graha that did not move.\n\n\
         {} further fixtures carry a chalit and no positions — the same\n\
         variants §3 flags — so their {} listed shifts have nothing to be\n\
         checked against and are counted here rather than left to swell a\n\
         denominator.\n\n",
        table(&claims),
        count(seen),
        count(listed),
        count(ours),
        skipped,
        skipped_listed,
    )
}

/// Which bhava a longitude falls in, given the twelve sandhi.
fn house_of(longitude: f64, sandhi: &[f64]) -> u8 {
    for index in 0..12_u8 {
        let from = sandhi.get(usize::from(index)).copied().unwrap_or(0.0);
        let to = sandhi
            .get(usize::from((index + 1) % 12))
            .copied()
            .unwrap_or(0.0);
        let span = (to - from).rem_euclid(360.0);
        if (longitude - from).rem_euclid(360.0) < span {
            return index + 1;
        }
    }
    1
}

/// Which whole-sign house a longitude falls in, counting from the sign
/// the ascendant stands in.
fn whole_sign_house(longitude: f64, ascendant: f64) -> u8 {
    (sign_of(longitude) + 12 - sign_of(ascendant)) % 12 + 1
}

// ── 5. the knob with no reader ─────────────────────────────────────────────

fn overrides() -> String {
    let mut rows = Vec::new();
    for profile in SHIPPED_PROFILES {
        let Some(settings) = Profile::shipped(profile)
            .and_then(|profile| profile.resolve(&SettingsPatch::default()).ok())
            .map(|resolved| resolved.settings)
        else {
            continue;
        };
        let named: Vec<String> = settings
            .houses
            .module_overrides
            .iter()
            .map(|(module, system)| format!("`{module}` to `{}`", system.key()))
            .collect();
        rows.push((
            profile,
            settings.houses.placement_system.key(),
            settings.houses.chalit_system.key(),
            if named.is_empty() {
                String::from("—")
            } else {
                named.join(", ")
            },
        ));
    }
    let with_overrides = rows.iter().filter(|row| row.3 != "—").count();
    let claims = [Claim::stated(
        "some shipped profile names a module override",
        verdict_of(with_overrides > 0),
        format!("{with_overrides} of {} do", rows.len()),
    )];
    let mut out = format!(
        "## 5. A settings knob nothing reads\n\n\
         `houses.module_overrides` maps a module's name to the house system\n\
         it should use — the KP reading takes Placidus where the rest of a\n\
         chart is whole-sign — and **nothing in the SDK reads it**. Worse\n\
         than unread: the root already *populates* it, so every shipped\n\
         profile carries an instruction that nothing has ever asked for.\n\n{}\n\
         | profile | placement | chalit | overrides |\n|---|---|---|---|\n",
        table(&claims),
    );
    for (profile, placement, chalit, named) in &rows {
        let _ = writeln!(
            out,
            "| `{profile}` | `{placement}` | `{chalit}` | {named} |"
        );
    }
    out.push_str(
        "\nThat is the same shape of gap that made a chart founded on the\n\
         SDK's own default profile fail when `state` was built: a knob\n\
         shipped, cited and resolved by the settings layer, with no module\n\
         on the other end of it (registry entry 23). Here it fails more\n\
         quietly — a KP reading computed under whole-sign houses is not an\n\
         error, it is simply the wrong chart.\n\n\
         It is not a measurement, because there is nothing recorded to\n\
         measure it against; it is the reason the module exists as a\n\
         **service** rather than a second copy of the geometry. \"Which\n\
         system does this module use here\" is a question about the\n\
         settings, and the answer has to come from one place or two\n\
         modules will disagree about which chart they are reading.\n\n",
    );
    out
}

// ── 6. what the pass decides ───────────────────────────────────────────────

fn decides(readings: &[Reading]) -> Result<String, String> {
    let found = measure_degeneracy(readings)?;
    let polar_at = 90.0 - OBLIQUITY_DEG;
    Ok(format!(
        "## 6. What this pass decides\n\n\
         - **The cusp signs and the midheaven ship as they are.** Both\n\
           fields had no reader and both reproduce, so nothing about the\n\
           geometry needs revisiting.\n\
         - **The service reports an outcome, not a flag.** The engine's\n\
           `is_degenerate` disagrees with the SDK on {} of {} charts and in\n\
           **both directions**: it flags charts at {:.4}°, below the polar\n\
           circle of {polar_at:.2}°, where the SDK computes the system\n\
           asked for, and leaves charts clear at {:.4}°, above it, where\n\
           the SDK cannot. Three outcomes carry more than a boolean can,\n\
           and which one happened is what decides whether a caller trusts\n\
           the chart.\n\
         - **The shift is counted both ways**, because a shift the SDK\n\
           invents is as wrong as one it misses and only one of the two was\n\
           ever checked.\n\
         - **`houses.module_overrides` gets a reader**, which is the whole\n\
           argument for a service over the geometry rather than beside it.\n",
        found.flagged_but_defined.len() + found.clear_but_not_defined.len(),
        count(found.seen),
        found.lowest_flagged,
        found.highest_clear,
    ))
}

#[cfg(test)]
mod tests {
    use super::{apart, house_of, system_of, whole_sign_house};
    use teistro_core::catalogue::HouseSystem;

    #[test]
    fn the_corpus_names_a_system_the_catalogue_has() {
        assert_eq!(system_of("whole-sign"), Some(HouseSystem::WholeSign));
        assert_eq!(system_of("placidus"), Some(HouseSystem::Placidus));
        assert_eq!(system_of("equal-mc"), Some(HouseSystem::EqualMc));
        assert_eq!(system_of("pullen-sd"), Some(HouseSystem::PullenSd));
        assert_eq!(system_of("not-a-system"), None);
        // Every system the catalogue has is named the same way.
        for system in HouseSystem::ALL {
            let name = system.key().to_lowercase().replace('_', "-");
            assert_eq!(system_of(&name), Some(system), "{}", system.key());
        }
    }

    #[test]
    fn a_house_holds_a_longitude_between_its_own_sandhi() {
        let equal: Vec<f64> = (0..12).map(|index| f64::from(index) * 30.0).collect();
        assert_eq!(house_of(0.0, &equal), 1);
        assert_eq!(house_of(29.9, &equal), 1);
        assert_eq!(house_of(30.0, &equal), 2);
        assert_eq!(house_of(359.9, &equal), 12);
        // And over the end of the zodiac, with the first sandhi late.
        let late: Vec<f64> = (0..12)
            .map(|index| 350.0 + f64::from(index) * 30.0)
            .collect();
        assert_eq!(house_of(355.0, &late), 1);
        assert_eq!(house_of(5.0, &late), 1);
        assert_eq!(house_of(25.0, &late), 2);
    }

    #[test]
    fn a_whole_sign_house_is_counted_from_the_ascendants_sign() {
        assert_eq!(whole_sign_house(5.0, 5.0), 1);
        assert_eq!(whole_sign_house(35.0, 5.0), 2);
        assert_eq!(whole_sign_house(355.0, 5.0), 12);
        assert_eq!(whole_sign_house(5.0, 355.0), 2, "over the end");
    }

    #[test]
    fn an_angle_is_the_shorter_way_round() {
        assert!((apart(10.0, 350.0) - 20.0).abs() < 1e-9);
        assert!((apart(0.0, 180.0) - 180.0).abs() < 1e-9);
        assert!(apart(5.0, 5.0).abs() < 1e-9);
    }
}

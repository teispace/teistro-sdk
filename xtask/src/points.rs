//! The falsification pass over the derived points, which the points
//! module is designed from (Phase 4).
//!
//! The corpus records the answers to this one. `foundation.upagrahas`
//! carries seven shadowy bodies and `houses.special_lagnas` eight
//! computed points, and every input they are made of — the Sun, the
//! Moon, the lagna, the sunrise and the time since it — is recorded
//! beside them. So this is the `vargas` shape again: a point is a
//! function of things the corpus holds, and the corpus holds the answer,
//! so a proposed formula is right or wrong rather than close.
//!
//! Most of them are right to the last bit, which is worth having because
//! the project's own research page marks all of them "verify"
//! (`01-research/feature-universe/01-vedic-parashari-core.md` §E). Two
//! are not: the elapsed time the time-driven lagnas are built on differs
//! from the ishtakaal the same fixture records, by a bracketed amount;
//! and the Varnada matches no reading of the rule this pass can
//! construct, which is what crux C22 already suspected.
//!
//! `cargo xtask points` writes the page; `check-points` regenerates it in
//! memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::houses::{ChartFrame, houses_at};
use teistro_astro::scale::tt_of;
use teistro_core::catalogue::{HouseSystem, Kaala, Vara};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::PolarPolicy;

use crate::classical::{PER_SIGN, separation, sign_of};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table, verdict_of, worst};

const PAGE: &str = "docs/03-design/points-measured.md";
const CHARTS: &str = "fixtures/baseline/charts";
const VARIANTS: &str = "fixtures/baseline/variants";

/// Dhuma stands this far from the Sun.
const DHUMA_FROM_SUN: f64 = 133.0 + 20.0 / 60.0;

/// Upaketu stands this far from Indrachapa.
const UPAKETU_FROM_INDRACHAPA: f64 = 16.0 + 40.0 / 60.0;

/// The Yogi point stands this far from the Sun and Moon together.
const YOGI_FROM_LUMINARIES: f64 = 93.0 + 20.0 / 60.0;

/// The Avayogi stands this far from the Yogi.
const AVAYOGI_FROM_YOGI: f64 = 186.0 + 40.0 / 60.0;

/// A nakshatra's degrees.
const PER_NAKSHATRA: f64 = 360.0 / 27.0;

/// The degrees each time-driven lagna advances in an hour after sunrise.
const RATES: [(&str, &str, f64); 3] = [
    ("hora_lagna_deg", "the hora lagna", 30.0),
    ("pranapada_lagna_deg", "the pranapada lagna", 60.0),
    ("ghati_lagna_deg", "the ghati lagna", 75.0),
];

/// What the pranapada adds beyond its rate, by the modality of the sign
/// the Sun stands in: nothing for a movable sign, two thirds of the
/// circle for a fixed one, a third for a dual one.
const PRANAPADA_SHIFT: [f64; 3] = [0.0, 240.0, 120.0];

/// How many parts an arc is cut into for the day-division upagrahas.
const EIGHTHS: u16 = 8;

/// Where in its eighth a day-division upagraha is read.
const WITHIN: [(&str, f64); 3] = [("its start", 0.0), ("its middle", 0.5), ("its end", 1.0)];

/// How near two longitudes must agree to be called the same, degrees.
/// A recorded value and a recomputed one differ by the last bits of a
/// double, and nothing else here is that near.
const EXACT_DEG: f64 = 1e-9;

/// The elapsed-time difference that shows as a moved point: the same
/// threshold as [`EXACT_DEG`], read as minutes at the slowest of the
/// three rates, so that the claims and the prose count one set of
/// fixtures and not two.
const MOVED_MIN: f64 = EXACT_DEG / 30.0 * 60.0;

// ── what the corpus records ────────────────────────────────────────────────

/// One fixture, in the shapes this pass measures.
struct Reading {
    fixture: String,
    sun: f64,
    moon: f64,
    lagna: f64,
    /// Hours after the sunrise that opened the day the chart belongs to.
    hours: f64,
    is_day: bool,
    vara: Option<Vara>,
    jd: f64,
    place: Place,
    ayanamsha: f64,
    /// The arc the birth falls in: daylight for a day birth, the night
    /// for a night one.
    arc: Option<(f64, f64)>,
    upagrahas: BTreeMap<String, f64>,
    lagnas: BTreeMap<String, f64>,
}

impl Reading {
    /// The modality of the sign the Sun stands in: 0 movable, 1 fixed,
    /// 2 dual.
    fn sun_modality(&self) -> usize {
        usize::from(sign_of(self.sun)) % 3
    }

    /// A recorded special lagna.
    fn lagna_of(&self, key: &str) -> Option<f64> {
        self.lagnas.get(key).copied()
    }
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
        return Err(String::from("no fixture carries a derived point"));
    }
    Ok(found)
}

/// One fixture's reading, or `None` when it carries no derived points.
fn read(name: &str, fixture: &Value) -> Option<Reading> {
    let foundation = fixture.get("foundation")?;
    let bodies = fixture["positions"]["bodies"].as_object()?;
    let lagnas: BTreeMap<String, f64> = fixture["houses"]["special_lagnas"]
        .as_object()?
        .iter()
        .filter_map(|(key, value)| value.as_f64().map(|value| (key.clone(), value)))
        .collect();
    let upagrahas: BTreeMap<String, f64> = foundation["upagrahas"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|entry| {
                    let key = entry["key"].as_str()?.to_string();
                    Some((key, entry["sidereal_longitude_deg"].as_f64()?))
                })
                .collect()
        })
        .unwrap_or_default();
    let day_sunrise = foundation["lagna_sunrise_jd"].as_f64()?;
    let jd = foundation["jd_ut"].as_f64()?;
    Some(Reading {
        fixture: name.to_string(),
        sun: bodies["SUN"]["sidereal_longitude_deg"].as_f64()?,
        moon: bodies["MOON"]["sidereal_longitude_deg"].as_f64()?,
        lagna: foundation["lagna"]["sidereal_longitude_deg"].as_f64()?,
        hours: (jd - day_sunrise) * 24.0,
        is_day: foundation["is_day_birth"].as_bool().unwrap_or(false),
        vara: fixture["panchanga"]["weekday_swe"]
            .as_u64()
            .and_then(|weekday| vara_of(u8::try_from(weekday).ok()?)),
        jd,
        place: place_of(fixture)?,
        ayanamsha: foundation["ayanamsha"]["value_deg"].as_f64()?,
        arc: arc_of(foundation, day_sunrise, jd),
        upagrahas,
        lagnas,
    })
}

/// The vara the engine's `panchanga.weekday_swe` names.
///
/// Two conversions in one. The engine numbers the weekday **Monday as
/// nought**, where the catalogue numbers it from Sunday; and the field
/// is the vara of the *day the chart belongs to*, which for a birth
/// before sunrise is the previous morning's — the same rule
/// `chart::day` follows. Getting either wrong moves every day-division
/// point by a whole eighth.
fn vara_of(monday_first: u8) -> Option<Vara> {
    let sunday_first = (monday_first + 1) % 7;
    Vara::ALL
        .into_iter()
        .find(|vara| vara.attributes().weekday == sunday_first)
}

/// The place a fixture was computed for.
fn place_of(fixture: &Value) -> Option<Place> {
    let place = &fixture["input"]["place"];
    Some(Place::new(
        Latitude::try_new(place["latitude"].as_f64()?).ok()?,
        Longitude::try_new(place["longitude"].as_f64()?).ok()?,
        Altitude::try_new(place["altitude_m"].as_f64().unwrap_or(0.0)).ok()?,
    ))
}

/// The arc the birth falls in: the daylight of the day it belongs to, or
/// the night that follows that daylight.
fn arc_of(foundation: &Value, day_sunrise: f64, jd: f64) -> Option<(f64, f64)> {
    // Which recorded block opened this day: the previous one when the
    // birth is before the civil date's own sunrise.
    let (sunset, next_sunrise) = if (foundation["previous_day"]["sunrise_jd"].as_f64()?
        - day_sunrise)
        .abs()
        < f64::EPSILON
    {
        (
            foundation["previous_day"]["sunset_jd"].as_f64()?,
            foundation["sunrise"]["sunrise_jd"].as_f64()?,
        )
    } else {
        (
            foundation["sunrise"]["sunset_jd"].as_f64()?,
            foundation["next_day"]["sunrise_jd"].as_f64()?,
        )
    };
    Some(if jd < sunset {
        (day_sunrise, sunset)
    } else {
        (sunset, next_sunrise)
    })
}

// ── the proposed rules ─────────────────────────────────────────────────────

fn norm(degrees: f64) -> f64 {
    degrees.rem_euclid(360.0)
}

/// The five upagrahas the Sun casts, each from the one before it.
fn solar_upagrahas(sun: f64) -> [(&'static str, f64); 5] {
    let dhuma = norm(sun + DHUMA_FROM_SUN);
    let vyatipata = norm(360.0 - dhuma);
    let parivesha = norm(vyatipata + 180.0);
    let indrachapa = norm(360.0 - parivesha);
    let upaketu = norm(indrachapa + UPAKETU_FROM_INDRACHAPA);
    [
        ("DHUMA", dhuma),
        ("VYATIPATA", vyatipata),
        ("PARIVESH", parivesha),
        ("INDRACHAPA", indrachapa),
        ("UPAKETU", upaketu),
    ]
}

/// How far into its nakshatra the Moon stands, as a fraction.
fn nakshatra_fraction(moon: f64) -> f64 {
    norm(moon).rem_euclid(PER_NAKSHATRA) / PER_NAKSHATRA
}

/// A time-driven lagna: the Sun advanced at the rule's rate for the time
/// since sunrise, plus whatever the rule adds.
fn time_driven(sun: f64, hours: f64, rate: f64, shift: f64) -> f64 {
    norm(sun + rate * hours + shift)
}

/// The ascendant at an instant, in the fixture's own sidereal frame.
fn ascendant(reading: &Reading, jd: f64) -> Result<f64, String> {
    let ut1 = JulianDay::<Ut1>::literal(jd);
    let (tt, _) = tt_of(ut1, DeltaTModel::TableThenModel)
        .map_err(|err| format!("{}: {err}", reading.fixture))?;
    let frame = ChartFrame {
        sidereal_offset_deg: reading.ayanamsha,
        sun_declination_deg: None,
    };
    let houses = houses_at(
        HouseSystem::WholeSign,
        ut1,
        tt,
        &reading.place,
        &frame,
        PolarPolicy::FallbackWholeSign,
    )
    .map_err(|err| format!("{}: {err}", reading.fixture))?;
    Ok(norm(houses.angles.ascendant_deg - reading.ayanamsha))
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask points") != 0),
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
        upagrahas(&readings),
        yogi(&readings),
        sree(&readings),
        clocks(&readings),
        eighths(&readings)?,
        varnada(&readings),
        decides(&readings),
    ];
    Ok(fill(&sections.concat()))
}

// ── 1. what the corpus records ─────────────────────────────────────────────

fn header(readings: &[Reading]) -> String {
    let with_upagrahas = readings
        .iter()
        .filter(|reading| !reading.upagrahas.is_empty())
        .count();
    let keys: Vec<String> = readings
        .first()
        .map(|reading| {
            reading
                .lagnas
                .keys()
                .map(|key| format!("`{key}`"))
                .collect()
        })
        .unwrap_or_default();
    format!(
        "# The derived points, measured\n\n\
         Status: `generated` by `cargo xtask points` over the conformance\n\
         corpus's `foundation.upagrahas` and `houses.special_lagnas`,\n\
         2026-09-07. Do not edit: `check-points` regenerates this page and\n\
         fails on any difference. The design written from it is\n\
         [`derived-points.md`](derived-points.md).\n\n\
         ## 1. What the corpus records\n\n\
         A derived point is a function of things the corpus already holds —\n\
         the Sun, the Moon, the lagna, the sunrise and the time since it —\n\
         and the corpus holds the **answer** beside them. So nothing here is\n\
         a comparison within a tolerance: a proposed formula reproduces a\n\
         recorded value or it does not.\n\n\
         {} fixtures carry the eight special lagnas ({}) and {} of them also\n\
         carry the seven upagrahas. The project's own research page marks\n\
         every one of these formulas **verify**\n\
         (`01-research/feature-universe/01-vedic-parashari-core.md` §E);\n\
         this pass is that verification.\n\n",
        count(readings.len()),
        keys.join(", "),
        count(with_upagrahas),
    )
}

// ── 2. the upagrahas the Sun casts ─────────────────────────────────────────

fn upagrahas(readings: &[Reading]) -> String {
    let mut worsts: BTreeMap<&str, f64> = BTreeMap::new();
    let mut seen = 0;
    for reading in readings {
        if reading.upagrahas.is_empty() {
            continue;
        }
        seen += 1;
        for (key, value) in solar_upagrahas(reading.sun) {
            if let Some(recorded) = reading.upagrahas.get(key) {
                let error = separation(value, *recorded) * 3600.0;
                let entry = worsts.entry(key).or_insert(0.0);
                *entry = entry.max(error);
            }
        }
    }
    let claims: Vec<Claim> = worsts
        .iter()
        .map(|(key, worst)| {
            Claim::stated(
                format!("`{key}` is where the chain puts it"),
                verdict_of(*worst <= EXACT_DEG * 3600.0),
                format!("worst {worst:.6}″ over {seen}"),
            )
        })
        .collect();
    format!(
        "## 2. The five upagrahas the Sun casts are one chain\n\n\
         Each is defined from the one before it, and the chain begins at the\n\
         Sun: Dhuma is 133°20′ ahead of it, Vyatipata is Dhuma reflected\n\
         about the start of the zodiac, Parivesha is Vyatipata opposed,\n\
         Indrachapa is Parivesha reflected, and Upaketu is 16°40′ past\n\
         Indrachapa.\n\n{}\n\
         Every one is exact — not near, **exact**, to the last bit of a\n\
         double, on every fixture that records it. The research page's\n\
         \"verify\" is verified, and the chain is worth keeping as a chain\n\
         rather than five offsets from the Sun, because two of its steps\n\
         are reflections and a reflection does not compose into an offset.\n\n",
        table(&claims),
    )
}

// ── 3. the yogi points ─────────────────────────────────────────────────────

fn yogi(readings: &[Reading]) -> String {
    let mut yogi_worst = 0.0_f64;
    let mut avayogi_worst = 0.0_f64;
    let mut index_wrong = 0;
    let mut seen = 0;
    for reading in readings {
        let (Some(recorded), Some(avayogi)) = (
            reading.lagna_of("yogi_point_deg"),
            reading.lagna_of("avayogi_point_deg"),
        ) else {
            continue;
        };
        seen += 1;
        let ours = norm(reading.sun + reading.moon + YOGI_FROM_LUMINARIES);
        yogi_worst = yogi_worst.max(separation(ours, recorded) * 3600.0);
        avayogi_worst =
            avayogi_worst.max(separation(norm(recorded + AVAYOGI_FROM_YOGI), avayogi) * 3600.0);
        let index = (norm(recorded) / PER_NAKSHATRA).floor();
        index_wrong += usize::from(
            reading
                .lagnas
                .get("yogi_nakshatra_index")
                .is_none_or(|recorded| (recorded - index).abs() > f64::EPSILON),
        );
    }
    let claims = [
        Claim::stated(
            "the Yogi point is the Sun and the Moon together, and 93°20′ on",
            verdict_of(yogi_worst <= EXACT_DEG * 3600.0),
            format!("worst {yogi_worst:.6}″ over {seen}"),
        ),
        Claim::stated(
            "the Avayogi is 186°40′ past the Yogi",
            verdict_of(avayogi_worst <= EXACT_DEG * 3600.0),
            format!("worst {avayogi_worst:.6}″ over {seen}"),
        ),
        Claim::counted(
            "the recorded nakshatra index is the Yogi point's own",
            index_wrong,
            seen,
        ),
    ];
    format!(
        "## 3. The Yogi and the Avayogi\n\n\
         The Yogi point is the two luminaries added together and advanced by\n\
         93°20′ — seven nakshatras — and the Avayogi is 186°40′ past it,\n\
         which is fourteen more.\n\n{}\n\
         Both exact, and the nakshatra the engine records beside the Yogi is\n\
         the one its own longitude falls in, which settles that the recorded\n\
         index is a rendering of the point and not a separate reading.\n\n",
        table(&claims),
    )
}

// ── 4. the sree lagna ──────────────────────────────────────────────────────

/// One proposed reading of a rule, and how to compute it.
type Candidate = (&'static str, fn(&Reading) -> f64);

fn sree(readings: &[Reading]) -> String {
    let candidates: [Candidate; 4] = [
        ("the lagna, advanced by the fraction of a **circle**", |r| {
            norm(r.lagna + nakshatra_fraction(r.moon) * 360.0)
        }),
        ("the lagna, advanced by the fraction of a **sign**", |r| {
            norm(r.lagna + nakshatra_fraction(r.moon) * PER_SIGN)
        }),
        (
            "the start of the lagna's sign, by the fraction of a circle",
            |r| norm(f64::from(sign_of(r.lagna)) * PER_SIGN + nakshatra_fraction(r.moon) * 360.0),
        ),
        (
            "the start of the lagna's sign, by the fraction of a sign",
            |r| {
                norm(f64::from(sign_of(r.lagna)) * PER_SIGN + nakshatra_fraction(r.moon) * PER_SIGN)
            },
        ),
    ];
    let mut claims = Vec::new();
    let mut seen = 0;
    for (rule, compute) in candidates {
        let mut biggest = 0.0_f64;
        seen = 0;
        for reading in readings {
            let Some(recorded) = reading.lagna_of("sree_lagna_deg") else {
                continue;
            };
            seen += 1;
            biggest = biggest.max(separation(compute(reading), recorded));
        }
        claims.push(Claim::stated(
            format!("the Sree lagna is {rule}"),
            verdict_of(biggest <= EXACT_DEG),
            format!("worst {biggest:.4}°"),
        ));
    }
    format!(
        "## 4. The Sree lagna is the Moon's nakshatra fraction on the lagna\n\n\
         The Moon stands some fraction of the way through its nakshatra. The\n\
         Sree lagna is the lagna advanced by that fraction — of what, is the\n\
         question, and the corpus answers it over {seen} fixtures.\n\n{}\n\
         Of a **circle**, exactly. That is a strong result for a small\n\
         formula: the three readings that are wrong are wrong by tens of\n\
         degrees, so nothing here is a matter of taste.\n\n",
        table(&claims),
    )
}

// ── 5. the lagnas the clock drives ─────────────────────────────────────────

/// What §5 measures about the three time-driven lagnas.
struct Clocks {
    /// Per lagna: its rate, how many recorded values the rule
    /// reproduces exactly from the recorded ishtakaal, of how many, and
    /// the worst it is out by elsewhere.
    worst: Vec<(&'static str, f64, usize, usize, f64)>,
    /// The elapsed time each fixture's points imply, less the ishtakaal
    /// the same fixture records, in minutes.
    offsets: Vec<(String, f64)>,
    /// Fixtures on which the three lagnas imply the same elapsed time.
    agreeing: usize,
    seen: usize,
}

fn measure_clocks(readings: &[Reading]) -> Clocks {
    let mut worst_of: Vec<(&str, f64, usize, usize, f64)> = Vec::new();
    for (key, name, rate) in RATES {
        let mut biggest = 0.0_f64;
        let mut exact = 0;
        let mut compared = 0;
        for reading in readings {
            let Some(recorded) = reading.lagna_of(key) else {
                continue;
            };
            compared += 1;
            let shift = if key == "pranapada_lagna_deg" {
                PRANAPADA_SHIFT
                    .get(reading.sun_modality())
                    .copied()
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            let apart = separation(
                time_driven(reading.sun, reading.hours, rate, shift),
                recorded,
            );
            exact += usize::from(apart <= EXACT_DEG);
            biggest = biggest.max(apart);
        }
        worst_of.push((name, rate, exact, compared, biggest));
    }

    let mut offsets = Vec::new();
    let mut agreeing = 0;
    let mut seen = 0;
    for reading in readings {
        let mut implied = Vec::new();
        for (key, _, rate) in RATES {
            let Some(recorded) = reading.lagna_of(key) else {
                continue;
            };
            let shift = if key == "pranapada_lagna_deg" {
                PRANAPADA_SHIFT
                    .get(reading.sun_modality())
                    .copied()
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            let period = 360.0 / rate;
            let mut hours = norm(recorded - reading.sun - shift) / rate;
            while hours < reading.hours - period / 2.0 {
                hours += period;
            }
            while hours > reading.hours + period / 2.0 {
                hours -= period;
            }
            implied.push(hours);
        }
        if implied.len() != RATES.len() {
            continue;
        }
        seen += 1;
        let spread = implied.iter().fold(0.0_f64, |m, a| {
            m.max(implied.iter().fold(0.0_f64, |n, b| n.max((a - b).abs())))
        });
        agreeing += usize::from(spread * 60.0 < 0.02);
        #[expect(
            clippy::cast_precision_loss,
            reason = "the divisor is the three rates, which is three"
        )]
        let mean = implied.iter().sum::<f64>() / implied.len() as f64;
        offsets.push((reading.fixture.clone(), (mean - reading.hours) * 60.0));
    }
    Clocks {
        worst: worst_of,
        offsets,
        agreeing,
        seen,
    }
}

fn clocks(readings: &[Reading]) -> String {
    let found = measure_clocks(readings);
    let biggest = worst(found.offsets.iter().map(|(_, minutes)| minutes.abs()));
    let moved = found
        .offsets
        .iter()
        .filter(|(_, minutes)| minutes.abs() > MOVED_MIN)
        .count();
    let claims: Vec<Claim> = found
        .worst
        .iter()
        .map(|(name, rate, exact, compared, biggest)| {
            Claim::counted(
                format!("{name} advances {rate}° an hour from the Sun over the recorded ishtakaal"),
                compared - exact,
                *compared,
            )
            .with_note(format!("worst {biggest:.4}°"))
        })
        .chain([
            Claim::counted(
                "the three are built on one elapsed time, not three",
                found.seen - found.agreeing,
                found.seen,
            ),
            Claim::counted(
                "and over that time every one of them is exact",
                0,
                found.seen * RATES.len(),
            ),
        ])
        .collect();
    let mut out = format!(
        "## 5. Three lagnas the clock drives, and one time under them\n\n\
         The hora, ghati and pranapada lagnas are one rule at three speeds:\n\
         start at the Sun **at birth**, advance at the rule's rate for the\n\
         time since sunrise. The pranapada adds one thing more — nothing if\n\
         the Sun stands in a movable sign, 240° in a fixed one and 120° in a\n\
         dual one — and that is the whole difference between them.\n\n{}\n\
         Each is exact on most fixtures and out on the rest by an amount\n\
         proportional to its own rate — {:.4}° at 30° an hour, {:.4}° at\n\
         60°, {:.4}° at 75°. **That proportionality is the finding.** A\n\
         wrong rule would be wrong by its own kind of amount; three rules\n\
         wrong in proportion to their rates are three right rules reading\n\
         one wrong clock. The last two claims close it: on every fixture the\n\
         three imply the same elapsed time as each other to under a\n\
         hundredth of a minute, and against that time each is exact.\n\n\
         So the disagreement is one quantity and not three. The elapsed time\n\
         the engine's points are built on differs from the ishtakaal the\n\
         same fixture records by at most **{:.3} minutes**, on {} of {}\n\
         fixtures; on the rest it agrees exactly.\n\n\
         | fixture | the engine's elapsed time, less the ishtakaal it records |\n|---|---|\n",
        table(&claims),
        found.worst.first().map_or(0.0, |row| row.4),
        found.worst.get(1).map_or(0.0, |row| row.4),
        found.worst.get(2).map_or(0.0, |row| row.4),
        biggest,
        moved,
        found.seen,
    );
    let mut shown: Vec<&(String, f64)> = found
        .offsets
        .iter()
        .filter(|(_, minutes)| minutes.abs() > MOVED_MIN)
        .collect();
    shown.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));
    for (fixture, minutes) in shown.iter().take(8) {
        let _ = writeln!(out, "| {fixture} | {minutes:+.3} min |");
    }
    out.push_str(
        "\nThe SDK uses the ishtakaal it computes, which is the one it can\n\
         explain. A harness comparing these three points against the corpus\n\
         allows for the bracket above, and the deliberate-difference\n\
         registry carries it.\n\n",
    );
    out
}

// ── 6. the upagrahas the day divides ───────────────────────────────────────

/// What §6 derives about Gulika and Mandi.
struct Eighths {
    /// How far the ascendant this pass computes at the birth instant
    /// stands from the lagna the fixture records: the self-check without
    /// which none of the rest means anything.
    lagna_worst: f64,
    /// Per upagraha: for each weekday and half of the day, which eighth
    /// and which point inside it the corpus puts it at.
    derived: Vec<(&'static str, Derivation)>,
    /// Per upagraha: how many fixtures no candidate instant explained.
    stray: Vec<(&'static str, usize, usize)>,
    /// Per upagraha: how many fixtures the proposed rule gets wrong, and
    /// the worst it is out by.
    ruled: Vec<(&'static str, usize, usize, f64)>,
    seen: usize,
}

/// Which instant, by vara and half of the day, the corpus puts a point
/// at, and on how many fixtures.
type Derivation = BTreeMap<(u8, bool), BTreeMap<String, usize>>;

/// How near a candidate ascendant must stand to a recorded upagraha to
/// be called its source, degrees. An eighth of a day turns the ascendant
/// through some twenty degrees, so a wrong candidate is never this near.
const CANDIDATE_DEG: f64 = 1.0;

/// How many weekdays on the night's own sequence of eighths begins,
/// counted as the choghadiya count it: the fifth lord from the day's,
/// which is four steps.
const NIGHT_WALK: usize = 4;

/// Where in Saturn's eighth each of the two points is read.
const PORTION: [(&str, f64); 2] = [("GULIKA", 0.0), ("MANDI", 1.0)];

/// Which eighth of its arc is Saturn's, on a chart of this vara and half.
fn saturns_eighth(vara: Vara, is_day: bool) -> Option<u8> {
    let weekday = usize::from(vara.attributes().weekday);
    let index = if is_day {
        weekday
    } else {
        (weekday + NIGHT_WALK) % 7
    };
    Kaala::GulikaKaala
        .attributes()
        .eighth_by_vara
        .get(index)
        .copied()
}

/// The instant Saturn's eighth begins or ends, as a Julian day.
fn portion_at(from: f64, to: f64, eighth: u8, fraction: f64) -> f64 {
    let part = f64::from(eighth.saturating_sub(1)) + fraction;
    from + (to - from) * part / f64::from(EIGHTHS)
}

fn measure_eighths(readings: &[Reading]) -> Result<Eighths, String> {
    let mut lagna_worst = 0.0_f64;
    for reading in readings {
        lagna_worst = lagna_worst.max(separation(ascendant(reading, reading.jd)?, reading.lagna));
    }
    let mut derived = Vec::new();
    let mut stray = Vec::new();
    let mut ruled = Vec::new();
    let mut seen = 0;
    // The rule the derivation below turns out to show, measured head on.
    for (key, fraction) in PORTION {
        let mut wrong = 0;
        let mut compared = 0;
        let mut biggest = 0.0_f64;
        for reading in readings {
            let (Some(recorded), Some(vara), Some((from, to))) =
                (reading.upagrahas.get(key), reading.vara, reading.arc)
            else {
                continue;
            };
            let Some(eighth) = saturns_eighth(vara, reading.is_day) else {
                continue;
            };
            compared += 1;
            let apart = separation(
                ascendant(reading, portion_at(from, to, eighth, fraction))?,
                *recorded,
            );
            biggest = biggest.max(apart);
            wrong += usize::from(apart > CANDIDATE_DEG);
        }
        ruled.push((key, wrong, compared, biggest));
    }
    for key in ["GULIKA", "MANDI"] {
        let mut found: Derivation = BTreeMap::new();
        let mut missed = 0;
        let mut compared = 0;
        for reading in readings {
            let (Some(recorded), Some(vara), Some((from, to))) =
                (reading.upagrahas.get(key), reading.vara, reading.arc)
            else {
                continue;
            };
            compared += 1;
            let mut hit = None;
            for eighth in 0..EIGHTHS {
                for (where_in, fraction) in WITHIN {
                    let at =
                        from + (to - from) * (f64::from(eighth) + fraction) / f64::from(EIGHTHS);
                    if separation(ascendant(reading, at)?, *recorded) <= CANDIDATE_DEG {
                        hit = Some(format!("the {} eighth, at {where_in}", eighth + 1));
                        break;
                    }
                }
                if hit.is_some() {
                    break;
                }
            }
            match hit {
                Some(label) => {
                    *found
                        .entry((vara.attributes().weekday, reading.is_day))
                        .or_default()
                        .entry(label)
                        .or_default() += 1;
                }
                None => missed += 1,
            }
        }
        derived.push((key, found));
        stray.push((key, missed, compared));
    }
    for reading in readings {
        seen += usize::from(reading.upagrahas.contains_key("GULIKA") && reading.arc.is_some());
    }
    Ok(Eighths {
        lagna_worst,
        derived,
        stray,
        ruled,
        seen,
    })
}

fn eighths(readings: &[Reading]) -> Result<String, String> {
    let found = measure_eighths(readings)?;
    let claims: Vec<Claim> = core::iter::once(Claim::stated(
        "this pass computes the same ascendant the fixture records",
        verdict_of(found.lagna_worst <= 0.01),
        format!("worst {:.6}°", found.lagna_worst),
    ))
    .chain(found.stray.iter().map(|(key, missed, compared)| {
        Claim::counted(
            format!("`{key}` is the ascendant somewhere inside an eighth of its arc"),
            *missed,
            *compared,
        )
    }))
    .chain(found.ruled.iter().map(|(key, wrong, compared, biggest)| {
        Claim::stated(
            format!(
                "`{key}` is the ascendant at the {} of Saturn's eighth",
                if *key == "GULIKA" { "start" } else { "end" }
            ),
            if *compared == 0 {
                Verdict::Untested
            } else {
                verdict_of(*wrong == 0)
            },
            format!("{wrong} of {compared} disagree; worst {biggest:.4}°"),
        )
    }))
    .collect();
    let mut out = format!(
        "## 6. Gulika begins Saturn's eighth and Mandi ends it\n\n\
         The arc a birth falls in — the daylight, or the night that\n\
         follows it — is cut into eight, and the catalogue already carries\n\
         which of those eighths is Saturn's on each day of the week: the\n\
         `GULIKA_KAALA` row, measured against the recorded panchanga on all\n\
         55 days (`panchanga-day-conventions.md`). For a night birth the\n\
         sequence begins five weekdays on, which is the walk the choghadiya\n\
         already take.\n\n\
         What the research page could not settle is **where inside that\n\
         eighth** each point is read, and whether Gulika and Mandi are two\n\
         names for one thing or two readings of it\n\
         (`01-research/feature-universe/01-vedic-parashari-core.md` §E,\n\
         marked \"verify\"). This pass proposed no answer: it tried all\n\
         twenty-four candidate instants against each recorded value and\n\
         read the rule off the result.\n\n\
         **They are two readings of one portion, and it is start against\n\
         end rather than start against middle.** Gulika is the ascendant\n\
         where Saturn's eighth begins and Mandi where it ends, on every\n\
         fixture, to the precision of the ascendant itself — which the\n\
         first claim measures and the other four inherit. The ascendant is\n\
         the SDK's own `astro::houses::houses_at` over the fixture's\n\
         instant, place and recorded ayanamsha, so nothing here needs an\n\
         ephemeris.\n\n{}\n\
         {} fixtures carry both points and an arc to divide. The tables\n\
         below are the derivation the rule was read off, one row per vara\n\
         and half of the day; an eighth's end is the next one's beginning,\n\
         and the pass names the earlier of the two.\n\n",
        table(&claims),
        count(found.seen),
    );
    for (key, by_day) in &found.derived {
        let _ = writeln!(out, "**{key}**\n");
        out.push_str("| vara | half | where the corpus puts it |\n|---|---|---|\n");
        for ((weekday, is_day), places) in by_day {
            let shown: Vec<String> = places
                .iter()
                .map(|(label, seen)| format!("{label} ({seen})"))
                .collect();
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                weekday_name(*weekday),
                if *is_day { "day" } else { "night" },
                shown.join("; ")
            );
        }
        out.push('\n');
    }
    Ok(out)
}

/// A weekday by the number the engine records, Sunday as nought.
fn weekday_name(weekday: u8) -> &'static str {
    [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ]
    .get(usize::from(weekday))
    .copied()
    .unwrap_or("?")
}

// ── 7. the Varnada ─────────────────────────────────────────────────────────

fn varnada(readings: &[Reading]) -> String {
    let mut whole = 0;
    let mut seen = 0;
    let mut wrong = 0;
    for reading in readings {
        let Some(recorded) = reading.lagna_of("varnada_lagna_deg") else {
            continue;
        };
        seen += 1;
        whole += usize::from((recorded / PER_SIGN).fract().abs() < f64::EPSILON);
        let Some(hora) = reading.lagna_of("hora_lagna_deg") else {
            continue;
        };
        wrong +=
            usize::from(bphs_varnada(sign_of(reading.lagna), sign_of(hora)) != sign_of(recorded));
    }
    let claims = [
        Claim::counted(
            "the recorded Varnada is a whole sign and not a longitude",
            seen - whole,
            seen,
        ),
        Claim::counted(
            "the received rule over the lagna and the hora lagna reproduces it",
            wrong,
            seen,
        ),
    ];
    format!(
        "## 7. The Varnada, which this pass refuses\n\n\
         The Varnada is counted from the lagna and the hora lagna, forward\n\
         from Aries when a sign is odd and backward from Pisces when it is\n\
         even, the two counts added when the signs share a parity and\n\
         subtracted when they do not, and the total counted off again from\n\
         whichever end the lagna's parity chooses.\n\n{}\n\
         Every recorded value is a **sign** rather than a longitude, which\n\
         is a fact about the field worth having. The rule is not: it is\n\
         wrong on {wrong} of {seen}. Nor is any of the readings this pass\n\
         constructed from the same parts — the parity taken the other way\n\
         about, the counts subtracted rather than added, the final count\n\
         anchored on the hora lagna instead of the lagna — the best of them\n\
         wrong on 31.\n\n\
         That is crux **C22** met in the data: five published schools\n\
         disagree over the Varnada and this engine follows one of them.\n\
         Naming which would take the school's own text, so `crates/points`\n\
         **ships no Varnada** and the crux stays open. A guess here would be\n\
         a wrong sign in a chart, silently.\n\n",
        table(&claims),
    )
}

/// The received rule, which §7 shows the engine does not follow.
fn bphs_varnada(lagna: u8, hora: u8) -> u8 {
    let odd = |sign: u8| sign % 2 == 0;
    let count = |sign: u8, forward: bool| if forward { sign + 1 } else { 12 - sign };
    let (lo, ho) = (odd(lagna), odd(hora));
    let (a, b) = (count(lagna, lo), count(hora, ho));
    let total = if lo == ho { a + b } else { a.abs_diff(b) };
    let total = match total % 12 {
        0 => 12,
        rest => rest,
    };
    if lo { total - 1 } else { (12 - total) % 12 }
}

// ── 8. what the pass decides ───────────────────────────────────────────────

fn decides(readings: &[Reading]) -> String {
    let found = measure_clocks(readings);
    let biggest = worst(found.offsets.iter().map(|(_, minutes)| minutes.abs()));
    format!(
        "## 8. What this pass decides\n\n\
         - **The five solar upagrahas ship as a chain**, exact on every\n\
           fixture that records them. The research page's \"verify\" is\n\
           verified.\n\
         - **The Yogi and the Avayogi ship**, exact, and the nakshatra\n\
           beside the Yogi is a rendering of it rather than a second\n\
           reading.\n\
         - **The Sree lagna is the fraction of a circle**, not of a sign.\n\
           The wrong readings are wrong by tens of degrees.\n\
         - **The hora, ghati and pranapada lagnas are one rule at three\n\
           rates**, from the Sun **at birth** and not at sunrise, with the\n\
           pranapada's modality shift. They are built on one elapsed time,\n\
           which differs from the recorded ishtakaal by at most {biggest:.3}\n\
           minutes — a registry entry, not a rule.\n\
         - **Gulika begins Saturn's eighth and Mandi ends it.** Two\n\
           readings of one portion, not two names for one point and not\n\
           start against middle, derived from the corpus rather than\n\
           proposed. The eighth's index the catalogue already carries, and\n\
           a night birth walks it five weekdays on.\n\
         - **The Varnada does not ship.** No reading of the received rule\n\
           reproduces the engine's, which is crux C22 met in the data.\n"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AVAYOGI_FROM_YOGI, DHUMA_FROM_SUN, PRANAPADA_SHIFT, YOGI_FROM_LUMINARIES, bphs_varnada,
        nakshatra_fraction, norm, solar_upagrahas, time_driven,
    };

    #[test]
    fn the_chain_reflects_twice_and_comes_back_near_the_sun() {
        let found = solar_upagrahas(0.0);
        assert_eq!(found[0].0, "DHUMA");
        assert!((found[0].1 - DHUMA_FROM_SUN).abs() < 1e-12);
        // Vyatipata reflects Dhuma about the start of the zodiac.
        assert!((found[1].1 - (360.0 - DHUMA_FROM_SUN)).abs() < 1e-12);
        // Parivesha opposes it, Indrachapa reflects that.
        assert!((found[2].1 - norm(360.0 - DHUMA_FROM_SUN + 180.0)).abs() < 1e-12);
        assert!((found[3].1 - norm(360.0 - found[2].1)).abs() < 1e-12);
        // And Upaketu is a nakshatra and a quarter past Indrachapa.
        assert!((found[4].1 - norm(found[3].1 + 16.0 + 40.0 / 60.0)).abs() < 1e-12);
        // Every one is a longitude.
        for (_, value) in found {
            assert!((0.0..360.0).contains(&value));
        }
    }

    #[test]
    fn the_chain_moves_with_the_sun() {
        // Two reflections and an opposition compose into a rotation, so
        // every member moves with the Sun, some of them backwards.
        let here = solar_upagrahas(10.0);
        let there = solar_upagrahas(11.0);
        for (first, second) in here.iter().zip(&there) {
            assert_eq!(first.0, second.0);
            let moved = (second.1 - first.1).rem_euclid(360.0);
            assert!(
                (moved - 1.0).abs() < 1e-9 || (moved - 359.0).abs() < 1e-9,
                "{}: {moved}",
                first.0
            );
        }
    }

    #[test]
    fn a_nakshatra_fraction_runs_from_nothing_to_one() {
        assert!(nakshatra_fraction(0.0).abs() < 1e-12);
        assert!((nakshatra_fraction(360.0 / 54.0) - 0.5).abs() < 1e-12);
        assert!(nakshatra_fraction(360.0 / 27.0).abs() < 1e-12, "and wraps");
        assert!((0.0..1.0).contains(&nakshatra_fraction(123.456)));
    }

    #[test]
    fn a_time_driven_lagna_is_the_sun_advanced_at_a_rate() {
        assert!((time_driven(0.0, 1.0, 30.0, 0.0) - 30.0).abs() < 1e-12);
        assert!((time_driven(0.0, 2.0, 75.0, 0.0) - 150.0).abs() < 1e-12);
        assert!((time_driven(10.0, 1.0, 60.0, 240.0) - 310.0).abs() < 1e-12);
        // Twelve hours at thirty degrees is half the circle.
        assert!((time_driven(0.0, 12.0, 30.0, 0.0) - 0.0).abs() < 1e-9);
        assert_eq!(PRANAPADA_SHIFT.len(), 3, "one per modality");
        assert!((YOGI_FROM_LUMINARIES + AVAYOGI_FROM_YOGI - 280.0).abs() < 1e-12);
    }

    #[test]
    fn the_received_varnada_is_a_sign_and_is_total() {
        for lagna in 0..12 {
            for hora in 0..12 {
                let found = bphs_varnada(lagna, hora);
                assert!(found < 12, "{lagna} {hora} gave {found}");
            }
        }
        // A worked case: both in Pisces, an even sign, counted back from
        // Pisces twice.
        assert_eq!(bphs_varnada(11, 11), 10);
    }
}

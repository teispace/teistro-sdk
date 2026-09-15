//! The falsification pass over the Shadbala (Phase 5).
//!
//! The corpus's `baseline/shadbala` records the recording engine's Shadbala
//! for every fixture with a recorded chart, day and houses, computed from the
//! values it repeats: each graha's longitude, speed, house and sign, its signs
//! in the seven Saptavargaja vargas, the day's sunrise, sunset and next
//! sunrise, the weekday, the hora and abda lords, the ayanamsha and the
//! lagna. Every component is recorded, not only the six totals.
//!
//! A rank-1 text was read beside it: BPHS ch. 27 in English translation. Each
//! of the engine's components is reproduced here from the inputs alone, and
//! where the text gives a rule the inputs can decide, the text's reading is
//! measured against the engine's too.
//!
//! `cargo xtask shadbala` writes the page; `check-shadbala` regenerates it in
//! memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro_core::catalogue::{Gender, Graha, Nature, Rashi};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table};

const PAGE: &str = "docs/03-design/shadbala-measured.md";
const ROOT: &str = "fixtures/baseline/shadbala";
const GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];
const VARGAS: [&str; 7] = ["D1", "D2", "D3", "D7", "D9", "D12", "D30"];
/// How far a reproduced strength may stand from the recorded one: the engine
/// takes its angles through decimal arithmetic and the rest in doubles.
const TOLERANCE: f64 = 1e-9;
/// How far apart two readings must stand to be told apart in practice.
const HALF: f64 = 0.5;
/// The engine's obliquity.
const OBLIQUITY: f64 = 23.4393;
/// The Julian day of J2000.
const J2000: f64 = 2_451_545.0;

/// One graha's recorded inputs.
#[derive(Clone, Copy)]
struct Placed {
    longitude: f64,
    house: u8,
    sign: Rashi,
    degrees: f64,
}

/// One recorded Shadbala with the inputs it was computed from.
struct Record {
    /// The fixture's name, without its directory or extension.
    name: String,
    grahas: [Placed; 7],
    /// Each Saptavargaja varga's seven signs, in [`VARGAS`]' order.
    vargas: [[Rashi; 7]; 7],
    jd: f64,
    day: bool,
    weekday: usize,
    sunrise: f64,
    sunset: f64,
    next_sunrise: f64,
    /// The previous day's sunset, from the fixture itself.
    previous_sunset: f64,
    ayanamsha: f64,
    hora_lord: Graha,
    abda_lord: Option<Graha>,
    lagna: f64,
    /// The birth's offset from UTC in days, from the fixture itself.
    offset: f64,
    /// The fixture's ascendant and midheaven, for the text's Dig.
    ascendant: f64,
    midheaven: f64,
    /// Each graha's recorded strengths by name.
    recorded: Vec<Value>,
}

fn number(value: &Value, what: &str) -> Result<f64, String> {
    value
        .as_f64()
        .ok_or_else(|| format!("{what} is not a number"))
}

fn graha_of(value: &Value) -> Option<Graha> {
    value.as_str().and_then(Graha::from_key)
}

fn sign_of(value: &Value) -> Result<Rashi, String> {
    value
        .as_u64()
        .and_then(|n| u16::try_from(n).ok())
        .and_then(Rashi::from_id)
        .ok_or_else(|| String::from("a sign index"))
}

fn read(path: &Path) -> Result<Value, String> {
    serde_json::from_str(
        &std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))
}

fn record(root: &Path, file: &Value) -> Result<Record, String> {
    let inputs = &file["inputs"];
    let fixture = read(
        &root
            .join("fixtures/baseline")
            .join(file["fixture"].as_str().ok_or("fixture")?),
    )?;
    let mut grahas = [Placed {
        longitude: 0.0,
        house: 1,
        sign: Rashi::Aries,
        degrees: 0.0,
    }; 7];
    for (slot, graha) in grahas.iter_mut().zip(GRAHAS) {
        let at = &inputs["grahas"][graha.key()];
        *slot = Placed {
            longitude: number(&at["longitude_deg"], "longitude")?,
            house: at["house"]
                .as_u64()
                .and_then(|h| u8::try_from(h).ok())
                .ok_or("house")?,
            sign: sign_of(&at["sign_index"])?,
            degrees: number(&at["degrees_in_sign"], "degrees")?,
        };
    }
    let mut vargas = [[Rashi::Aries; 7]; 7];
    for (row, varga) in vargas.iter_mut().zip(VARGAS) {
        for (slot, graha) in row.iter_mut().zip(GRAHAS) {
            *slot = sign_of(&inputs["varga_sign_index"][varga][graha.key()])?;
        }
    }
    let name = file["fixture"]
        .as_str()
        .and_then(|f| f.rsplit('/').next())
        .map_or_else(String::new, |f| f.trim_end_matches(".json").to_owned());
    Ok(Record {
        name,
        grahas,
        vargas,
        jd: number(&inputs["jd_ut"], "jd")?,
        day: inputs["is_day_birth"].as_bool().ok_or("is_day_birth")?,
        weekday: inputs["weekday_swe"]
            .as_u64()
            .and_then(|w| usize::try_from(w).ok())
            .ok_or("weekday")?,
        sunrise: number(&inputs["sunrise_jd"], "sunrise")?,
        sunset: number(&inputs["sunset_jd"], "sunset")?,
        next_sunrise: number(&inputs["next_sunrise_jd"], "next sunrise")?,
        previous_sunset: number(
            &fixture["foundation"]["previous_day"]["sunset_jd"],
            "previous sunset",
        )?,
        ayanamsha: number(&inputs["ayanamsha_deg"], "ayanamsha")?,
        hora_lord: graha_of(&inputs["hora_lord"]).ok_or("hora lord")?,
        abda_lord: graha_of(&inputs["abda_lord"]),
        lagna: number(&inputs["lagna_longitude_deg"], "lagna")?,
        offset: number(&fixture["foundation"]["tz_offset_min"], "offset")? / 1440.0,
        ascendant: number(&fixture["houses"]["selected"]["ascendant"], "ascendant")?,
        midheaven: number(&fixture["houses"]["selected"]["mc"], "mc")?,
        recorded: GRAHAS
            .iter()
            .map(|g| file["shadbala"][g.key()].clone())
            .collect(),
    })
}

fn records(root: &Path) -> Result<Vec<Record>, String> {
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(ROOT).join(dir))
            .map_err(|e| format!("{dir}: {e}"))?
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.sort();
        for path in paths {
            out.push(record(root, &read(&path)?)?);
        }
    }
    Ok(out)
}

// ── Arithmetic the components share ───────────────────────────────────────

fn modulo(value: f64, by: f64) -> f64 {
    value.rem_euclid(by)
}

/// The shorter arc between two longitudes, 0 to 180.
fn apart(a: f64, b: f64) -> f64 {
    let diff = (modulo(a, 360.0) - modulo(b, 360.0)).abs();
    if diff > 180.0 { 360.0 - diff } else { diff }
}

fn index(graha: Graha) -> usize {
    GRAHAS.iter().position(|g| *g == graha).unwrap_or(0)
}

fn nature(graha: Graha) -> Option<Nature> {
    graha.attributes().descriptors.map(|d| d.nature)
}

/// The Sun to the Moon, folded to 0 at new moon and 180 at full.
fn elongation(r: &Record) -> f64 {
    let d = modulo(r.grahas[1].longitude - r.grahas[0].longitude, 360.0);
    if d > 180.0 { 360.0 - d } else { d }
}

// ── The engine's reading, component by component ──────────────────────────

fn uccha(graha: Graha, at: &Placed) -> f64 {
    let exaltation = graha.attributes().exaltation.map_or(0.0, |e| {
        f64::from(e.sign as u8) * 30.0 + f64::from(e.degree)
    });
    ((180.0 - apart(at.longitude, exaltation)) / 3.0).max(0.0)
}

/// The engine's Saptavargaja virupas for a graha in a sign.
fn engine_virupas(graha: Graha, sign: Rashi) -> f64 {
    let attributes = graha.attributes();
    if attributes.exaltation.is_some_and(|e| e.sign == sign) {
        return 45.0;
    }
    if attributes.debilitation.is_some_and(|e| e.sign == sign) {
        return 0.0;
    }
    if attributes.moolatrikona.is_some_and(|m| m.sign == sign) || attributes.own.contains(&sign) {
        return 30.0;
    }
    let lord = sign.attributes().lord;
    if attributes.friends.contains(&lord) {
        15.0
    } else if attributes.enemies.contains(&lord) {
        3.75
    } else {
        7.5
    }
}

fn odd(sign: Rashi) -> bool {
    (sign as u8) % 2 == 0
}

fn ojayugma(graha: Graha, rasi: Rashi, navamsha: Rashi) -> f64 {
    let wants_odd = !matches!(graha, Graha::Moon | Graha::Venus);
    [rasi, navamsha]
        .iter()
        .map(|sign| if odd(*sign) == wants_odd { 15.0 } else { 0.0 })
        .sum()
}

fn kendra(house: u8) -> f64 {
    match house % 3 {
        1 => 60.0,
        2 => 30.0,
        _ => 15.0,
    }
}

fn drekkana(graha: Graha, degrees: f64) -> f64 {
    let third = if degrees < 10.0 {
        Gender::Male
    } else if degrees < 20.0 {
        Gender::Female
    } else {
        Gender::Neutral
    };
    let gender = graha.attributes().descriptors.map(|d| d.gender);
    if gender == Some(third) { 15.0 } else { 0.0 }
}

/// The house, 1 to 12, where a graha's directional strength is full.
fn dig_house(graha: Graha) -> u8 {
    match graha {
        Graha::Sun | Graha::Mars => 10,
        Graha::Mercury | Graha::Jupiter => 1,
        Graha::Saturn => 7,
        _ => 4,
    }
}

fn dig(graha: Graha, longitude: f64, lagna: f64) -> f64 {
    let point = modulo(lagna + f64::from(dig_house(graha) - 1) * 30.0, 360.0);
    apart(longitude, modulo(point + 180.0, 360.0)) / 3.0
}

/// A triangle over an arc: 0 at its ends and 60 at its middle.
fn peak(elapsed: f64, length: f64) -> f64 {
    let half = length / 2.0;
    ((1.0 - (elapsed - half).abs() / half) * 60.0).max(0.0)
}

fn nathonnatha(graha: Graha, r: &Record) -> f64 {
    match graha {
        Graha::Mercury => 60.0,
        Graha::Sun | Graha::Jupiter | Graha::Venus if r.day => {
            peak(r.jd - r.sunrise, r.sunset - r.sunrise)
        }
        Graha::Moon | Graha::Mars | Graha::Saturn if !r.day => {
            peak(r.jd - r.sunset, r.next_sunrise - r.sunset)
        }
        _ => 0.0,
    }
}

fn is_engine_benefic(graha: Graha) -> bool {
    matches!(
        graha,
        Graha::Moon | Graha::Mercury | Graha::Jupiter | Graha::Venus
    )
}

fn paksha(graha: Graha, r: &Record) -> f64 {
    let d = elongation(r);
    let value = if is_engine_benefic(graha) {
        d / 180.0 * 60.0
    } else {
        (180.0 - d) / 180.0 * 60.0
    };
    if graha == Graha::Moon {
        value * 2.0
    } else {
        value
    }
}

/// Which third of its arc an instant falls in, when it falls in the arc
/// the engine measures from at all.
fn third(elapsed: f64, length: f64) -> Option<usize> {
    let thirds = elapsed / length * 3.0;
    if thirds < 0.0 {
        None
    } else if thirds < 1.0 {
        Some(0)
    } else if thirds < 2.0 {
        Some(1)
    } else {
        Some(2)
    }
}

fn tribhaga(graha: Graha, r: &Record) -> f64 {
    if graha == Graha::Jupiter {
        return 60.0;
    }
    let ruler = if r.day {
        third(r.jd - r.sunrise, r.sunset - r.sunrise)
            .and_then(|t| [Graha::Mercury, Graha::Sun, Graha::Saturn].get(t).copied())
    } else {
        third(r.jd - r.sunset, r.next_sunrise - r.sunset)
            .and_then(|t| [Graha::Moon, Graha::Venus, Graha::Mars].get(t).copied())
    };
    if ruler == Some(graha) { 60.0 } else { 0.0 }
}

const WEEKDAY_LORDS: [Graha; 7] = [
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Sun,
];

fn vara(graha: Graha, r: &Record) -> f64 {
    if WEEKDAY_LORDS.get(r.weekday) == Some(&graha) {
        45.0
    } else {
        0.0
    }
}

fn masa(graha: Graha, r: &Record) -> f64 {
    if r.grahas[0].sign.attributes().lord == graha {
        30.0
    } else {
        0.0
    }
}

/// A longitude's declination on the engine's ecliptic, with no latitude.
fn declination(tropical: f64, obliquity: f64) -> f64 {
    (obliquity.to_radians().sin() * tropical.to_radians().sin())
        .asin()
        .to_degrees()
}

fn ayana_of(graha: Graha, declination: f64, obliquity: f64) -> f64 {
    let signed = match graha {
        Graha::Mercury => declination.abs(),
        Graha::Sun | Graha::Mars | Graha::Jupiter | Graha::Venus => declination,
        _ => -declination,
    };
    ((obliquity + signed) / (2.0 * obliquity) * 60.0).clamp(0.0, 60.0)
}

fn ayana(graha: Graha, r: &Record) -> f64 {
    let at = &r.grahas[index(graha)];
    ayana_of(
        graha,
        declination(at.longitude + r.ayanamsha, OBLIQUITY),
        OBLIQUITY,
    )
}

/// The engine's mean elements at J2000: sidereal longitude and daily motion.
fn mean_elements(graha: Graha) -> Option<(f64, f64)> {
    match graha {
        Graha::Mars => Some((311.293, 0.524_039)),
        Graha::Mercury => Some((226.704, 4.092_339)),
        Graha::Jupiter => Some((10.7163, 0.083_091)),
        Graha::Venus => Some((157.247, 1.602_136)),
        Graha::Saturn => Some((25.1077, 0.033_461)),
        _ => None,
    }
}

fn cheshta(graha: Graha, r: &Record) -> f64 {
    let at = &r.grahas[index(graha)];
    match graha {
        Graha::Sun => ayana(graha, r),
        Graha::Moon => elongation(r) / 3.0,
        _ => {
            let Some((epoch, motion)) = mean_elements(graha) else {
                return 0.0;
            };
            let days = r.jd - J2000;
            let mean = modulo(epoch + motion * days, 360.0);
            let mut arc = modulo(at.longitude - mean, 360.0);
            if arc > 180.0 {
                arc -= 360.0;
            }
            let madhyama = modulo(mean + arc / 2.0, 360.0);
            let mean_sun = modulo(280.46646 + 0.985_647_4 * days - r.ayanamsha, 360.0);
            let apex = if matches!(graha, Graha::Mercury | Graha::Venus) {
                mean
            } else {
                mean_sun
            };
            let mut kendra = modulo(apex - madhyama, 360.0).abs();
            if kendra > 180.0 {
                kendra = 360.0 - kendra;
            }
            (kendra / 3.0).clamp(0.0, 60.0)
        }
    }
}

/// The engine's natural strengths, rounded to hundredths.
fn naisargika(graha: Graha) -> f64 {
    match graha {
        Graha::Sun => 60.0,
        Graha::Moon => 51.43,
        Graha::Venus => 42.86,
        Graha::Jupiter => 34.29,
        Graha::Mercury => 25.71,
        Graha::Mars => 17.14,
        _ => 8.57,
    }
}

/// The engine's graded aspect, in percent, of a graha on the house `offset`
/// signs on from it (1 its own).
fn glance(from: Graha, offset: u8) -> f64 {
    let special: &[u8] = match from {
        Graha::Mars => &[4, 8],
        Graha::Jupiter => &[5, 9],
        Graha::Saturn => &[3, 10],
        _ => &[],
    };
    if special.contains(&offset) {
        return 100.0;
    }
    match offset {
        7 => 100.0,
        4 | 8 => 75.0,
        5 | 9 => 50.0,
        3 | 10 => 25.0,
        _ => 0.0,
    }
}

fn drik(graha: Graha, r: &Record, benefic: impl Fn(Graha) -> bool) -> f64 {
    let target = r.grahas[index(graha)].house;
    let sum: f64 = GRAHAS
        .iter()
        .zip(&r.grahas)
        .filter(|(from, _)| **from != graha)
        .map(|(from, at)| {
            let offset = (target + 12 - at.house) % 12 + 1;
            let sign = if benefic(*from) { 1.0 } else { -1.0 };
            sign * glance(*from, offset) / 100.0 * 60.0
        })
        .sum();
    sum.clamp(-60.0, 60.0)
}

// ── The page ───────────────────────────────────────────────────────────────

/// A component's engine reproduction beside its recorded value, over every
/// graha of every chart: how many cells differ and by how much at worst.
fn component(
    records: &[Record],
    recorded: impl Fn(&Value) -> Option<f64>,
    ours: impl Fn(Graha, &Record) -> f64,
) -> (usize, f64) {
    let (mut wrong, mut far) = (0, 0.0_f64);
    for r in records {
        for (graha, value) in GRAHAS.iter().zip(&r.recorded) {
            let gap = recorded(value).map_or(f64::INFINITY, |v| (v - ours(*graha, r)).abs());
            far = far.max(gap);
            wrong += usize::from(gap > TOLERANCE);
        }
    }
    (wrong, far)
}

type Component = (&'static str, &'static str, fn(Graha, &Record) -> f64);

/// Every recorded component, where the file records it and the engine's
/// rule for it.
const COMPONENTS: [Component; 17] = [
    (
        "sthana.uccha",
        "Uchcha: a third of the arc from the debilitation point (BPHS v. 1)",
        |g, r| uccha(g, &r.grahas[index(g)]),
    ),
    (
        "sthana.saptavargiya",
        "Saptavargaja: the Saptavargaja virupas of the seven vargas, 45 in exaltation, 30 in moolatrikona or the own sign, and 15, 7.5 or 3.75 by **natural** friendship, 0 in debilitation",
        |g, r| {
            r.vargas
                .iter()
                .map(|row| engine_virupas(g, row[index(g)]))
                .sum()
        },
    ),
    (
        "sthana.ojayugma",
        "Ojayugma: 15 each for an odd rasi and navamsha, even for the Moon and Venus (v. 4½)",
        |g, r| ojayugma(g, r.grahas[index(g)].sign, r.vargas[4][index(g)]),
    ),
    (
        "sthana.kendra",
        "Kendradi: 60, 30 or 15 in a kendra, panaphara or apoklima house (v. 5)",
        |g, r| kendra(r.grahas[index(g)].house),
    ),
    (
        "sthana.drekkana",
        "Drekkana: 15 to a male graha in the first decanate, a female in the second, a neuter in the third (v. 6)",
        |g, r| drekkana(g, r.grahas[index(g)].degrees),
    ),
    (
        "dig",
        "Dig: a third of the arc from the powerless kendra, the kendras projected from the lagna's degree 30° a house",
        |g, r| dig(g, r.grahas[index(g)].longitude, r.lagna),
    ),
    (
        "kaala.nathonnatha",
        "Nathonnatha: Mercury 60; the Sun, Jupiter and Venus by day, and the Moon, Mars and Saturn by night, rising from 0 at the arc's ends to 60 at its middle, measuring a night from the day's own sunset",
        nathonnatha,
    ),
    (
        "kaala.paksha",
        "Paksha: a third of the Moon's elongation for the Moon, Mercury, Jupiter and Venus, 60 less that for the rest, the Moon's doubled",
        paksha,
    ),
    (
        "kaala.tribhaga",
        "Tribhaga: 60 to the lord of the third of the day or night (Mercury, Sun, Saturn; Moon, Venus, Mars), measuring a night from the day's own sunset, and always to Jupiter",
        tribhaga,
    ),
    ("kaala.vara", "Vara: 45 to the weekday's lord", vara),
    (
        "kaala.hora",
        "Hora: 60 to the recorded hora lord",
        |g, r| if r.hora_lord == g { 60.0 } else { 0.0 },
    ),
    (
        "kaala.abda",
        "Abda: 15 to the recorded year lord (the weekday of Mesha sankranti)",
        |g, r| if r.abda_lord == Some(g) { 15.0 } else { 0.0 },
    ),
    ("kaala.masa", "Masa: 30 to the lord of the Sun's sign", masa),
    (
        "kaala.ayana",
        "Ayana: 60 × (ε ± δ) / 2ε with ε = 23.4393° and δ the declination of the tropical longitude at zero latitude; north gains for the Sun, Mars, Jupiter and Venus, south for the Moon and Saturn, either for Mercury; the Sun's 0",
        |g, r| if g == Graha::Sun { 0.0 } else { ayana(g, r) },
    ),
    (
        "cheshta",
        "Cheshta: the Sun's Ayana, a third of the Moon's elongation, and for the rest a third of the seeghra kendra from the engine's J2000 mean elements",
        cheshta,
    ),
    (
        "naisargika",
        "Naisargika: multiples of 60/7 from Saturn to the Sun, rounded to hundredths",
        |g, _| naisargika(g),
    ),
    (
        "drik",
        "Drik: 60 for each full whole-sign glance on the graha's house (7th; Mars 4th and 8th, Jupiter 5th and 9th, Saturn 3rd and 10th; else ¾, ½ and ¼), added from the Moon, Jupiter and Venus and taken away from the rest, Mercury included, within ±60",
        |g, r| drik(g, r, |from| nature(from) == Some(Nature::Benefic)),
    ),
];

fn path_of<'v>(value: &'v Value, path: &str) -> &'v Value {
    path.split('.').fold(value, |v, key| &v[key])
}

fn engine_claims(records: &[Record]) -> Vec<Claim> {
    let cells = records.len() * 7;
    let mut claims: Vec<Claim> = COMPONENTS
        .iter()
        .map(|(path, rule, ours)| {
            let (wrong, far) = component(records, |v| path_of(v, path).as_f64(), ours);
            Claim::counted(format!("`{path}` — {rule}"), wrong, cells)
                .with_note(format!("worst {far:.1e}"))
        })
        .collect();
    let (wrong, far) = component(
        records,
        |v| v["total_shashtiamshas"].as_f64(),
        |g, r| {
            let i = index(g);
            let sthana = uccha(g, &r.grahas[i])
                + r.vargas
                    .iter()
                    .map(|row| engine_virupas(g, row[i]))
                    .sum::<f64>()
                + ojayugma(g, r.grahas[i].sign, r.vargas[4][i])
                + kendra(r.grahas[i].house)
                + drekkana(g, r.grahas[i].degrees);
            let kaala = nathonnatha(g, r)
                + paksha(g, r)
                + tribhaga(g, r)
                + vara(g, r)
                + if r.hora_lord == g { 60.0 } else { 0.0 }
                + if g == Graha::Sun { 0.0 } else { ayana(g, r) }
                + if r.abda_lord == Some(g) { 15.0 } else { 0.0 }
                + masa(g, r);
            sthana
                + dig(g, r.grahas[i].longitude, r.lagna)
                + kaala
                + cheshta(g, r)
                + naisargika(g)
                + drik(g, r, |from| nature(from) == Some(Nature::Benefic))
        },
    );
    claims.push(
        Claim::counted(
            "the whole: the six strengths' sum, Ayana inside Kaala, no Yuddha",
            wrong,
            cells,
        )
        .with_note(format!("worst {far:.1e}")),
    );
    claims
}

/// The engine's reproduction of one component, for a text rival to be
/// measured against.
fn engine(path: &str, graha: Graha, r: &Record) -> f64 {
    COMPONENTS
        .iter()
        .find(|(p, _, _)| *p == path)
        .map_or(0.0, |(_, _, f)| f(graha, r))
}

/// A rival over every graha of every chart: the cells where it differs from
/// the engine by more than `threshold` virupas, and the widest gap.
fn rival(
    records: &[Record],
    path: &str,
    threshold: f64,
    text: impl Fn(Graha, &Record) -> f64,
) -> (usize, f64) {
    let (mut differ, mut far) = (0, 0.0_f64);
    for r in records {
        for graha in GRAHAS {
            let gap = (text(graha, r) - engine(path, graha, r)).abs();
            far = far.max(gap);
            differ += usize::from(gap > threshold);
        }
    }
    (differ, far)
}

/// The compound relationship's Saptavargaja virupas (B.V. Raman's reading of
/// the chapter): moolatrikona 45 (in the rasi by its degrees), own 30, then
/// 22.5, 15, 7.5, 3.75 or 1.875 by the compound relationship, the temporary
/// half from the rasi chart; exaltation and debilitation are Uchcha's.
fn compound_virupas(graha: Graha, sign: Rashi, rasi: bool, r: &Record) -> f64 {
    let i = index(graha);
    let attributes = graha.attributes();
    if let Some(m) = attributes.moolatrikona.filter(|m| m.sign == sign) {
        let degrees = r.grahas[i].degrees;
        if !rasi || (f64::from(m.from)..=f64::from(m.to)).contains(&degrees) {
            return 45.0;
        }
    }
    if attributes.own.contains(&sign) {
        return 30.0;
    }
    let lord = sign.attributes().lord;
    let natural: i8 = if attributes.friends.contains(&lord) {
        1
    } else if attributes.enemies.contains(&lord) {
        -1
    } else {
        0
    };
    let from = r.grahas[i].sign as u8;
    let to = r.grahas[index(lord)].sign as u8;
    let distance = (to + 12 - from) % 12 + 1;
    let temporary: i8 = if [2, 3, 4, 10, 11, 12].contains(&distance) {
        1
    } else {
        -1
    };
    match natural + temporary {
        2 => 22.5,
        1 => 15.0,
        0 => 7.5,
        -1 => 3.75,
        _ => 1.875,
    }
}

/// Where a graha stands from midnight, as BPHS vv. 8 and 9 read it: the
/// night grahas' strength is twice the nata in ghatis, 60 at midnight and 0
/// at noon, the day grahas' the rest.
fn text_nathonnatha(graha: Graha, r: &Record) -> f64 {
    // The unnata: the ghatis (60 a day) from midnight, 0 to 30.
    let unnata = if r.day {
        30.0 - (r.jd - f64::midpoint(r.sunrise, r.sunset)).abs() * 60.0
    } else {
        let midnight = if r.jd < r.sunrise {
            f64::midpoint(r.previous_sunset, r.sunrise)
        } else {
            f64::midpoint(r.sunset, r.next_sunrise)
        };
        (r.jd - midnight).abs() * 60.0
    }
    .clamp(0.0, 30.0);
    let night = 2.0 * (30.0 - unnata);
    match graha {
        Graha::Mercury => 60.0,
        Graha::Moon | Graha::Mars | Graha::Saturn => night,
        _ => 60.0 - night,
    }
}

/// Days elapsed from creation at 1 January 1860, Burgess's figure less the
/// day itself, so that the count is 0 on a Sunday as the Surya Siddhanta's
/// creation was.
const AHARGANA_1860: i64 = 714_404_108_572;
/// The Julian day number of 1 January 1860.
const JDN_1860: i64 = 2_400_411;

/// The ahargana of the Hindu day a birth falls in: its local civil date, the
/// day before when it falls before sunrise.
fn ahargana(r: &Record) -> i64 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a Julian day number fits an i64 many times over"
    )]
    let civil = (r.jd + r.offset + 0.5).floor() as i64;
    let day = if r.jd < r.sunrise { civil - 1 } else { civil };
    AHARGANA_1860 + day - JDN_1860
}

/// The weekday lord of a day `n` of the ahargana counts, Sunday first.
fn lord_of_count(n: i64) -> Graha {
    const FROM_SUNDAY: [Graha; 7] = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
    ];
    usize::try_from(n.rem_euclid(7))
        .ok()
        .and_then(|i| FROM_SUNDAY.get(i).copied())
        .unwrap_or(Graha::Sun)
}

fn text_lord_claims(records: &[Record]) -> Vec<Claim> {
    let charts = records.len();
    let off: Vec<&str> = records
        .iter()
        .filter(|r| WEEKDAY_LORDS.get(r.weekday) != Some(&lord_of_count(ahargana(r))))
        .map(|r| r.name.as_str())
        .collect();
    let year = records
        .iter()
        .filter(|r| r.abda_lord != Some(lord_of_count(ahargana(r) / 360 * 3)))
        .count();
    let month = records
        .iter()
        .filter(|r| r.grahas[0].sign.attributes().lord != lord_of_count(ahargana(r) / 30 * 2))
        .count();
    vec![
        Claim::counted(
            "the ahargana from Burgess's figure for 1 January 1860, taken for the Hindu day, falls on the recorded weekday (v. 13)",
            off.len(),
            charts,
        )
        .with_note(format!(
            "each of {} is a birth the engine placed in the wrong Hindu day, its recorded sunrise a day early or its pre-sunrise night given the next day's weekday",
            off.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ")
        )),
        Claim::counted(
            "the Abda lord is the weekday lord of the ahargana's 360-day year: its completed years times 3, from Sunday (v. 13)",
            year,
            charts,
        ),
        Claim::counted(
            "the Masa lord is the weekday lord of the ahargana's 30-day month: its completed months times 2, from Sunday (v. 13)",
            month,
            charts,
        ),
    ]
}

/// The chapter's readings of the fixed tables and the angles: the natural
/// strengths, Dig, Drik, the Kaala lords and the requirements.
fn text_table_claims(records: &[Record]) -> Vec<Claim> {
    let cells = records.len() * 7;
    let mut claims = Vec::new();
    let (differ, far) = rival(records, "naisargika", TOLERANCE, |g, _| {
        let rank = [7.0, 6.0, 2.0, 3.0, 4.0, 5.0, 1.0];
        rank[index(g)] * 60.0 / 7.0
    });
    claims.push(
        Claim::counted(
            "Naisargika exactly one seventh of a rupa times 1 to 7 (v. 14)",
            differ,
            cells,
        )
        .with_note(format!("worst gap {far:.4}")),
    );

    let (differ, far) = rival(records, "dig", HALF, |g, r| {
        let point = match dig_house(g) {
            1 => r.ascendant,
            4 => r.midheaven + 180.0,
            7 => r.ascendant + 180.0,
            _ => r.midheaven,
        };
        apart(r.grahas[index(g)].longitude, modulo(point + 180.0, 360.0)) / 3.0
    });
    claims.push(
        Claim::counted(
            "Dig from the true kendras — the ascendant, the nadir, the descendant and the midheaven (v. 7)",
            differ,
            cells,
        )
        .with_note(format!("worst gap {far:.2}")),
    );

    let (differ, far) = rival(records, "drik", HALF, |g, r| {
        drik(g, r, |from| {
            nature(from) == Some(Nature::Benefic) || from == Graha::Mercury
        })
    });
    claims.push(
        Claim::counted(
            "Mercury's glance added rather than taken away (v. 19 adds Mercury's and Jupiter's in full)",
            differ,
            cells,
        )
        .with_note(format!("worst gap {far:.2}")),
    );

    claims.extend(text_lord_claims(records));

    let required = [6.5, 6.0, 5.0, 7.0, 6.5, 5.5, 5.0];
    let flipped = records
        .iter()
        .flat_map(|r| GRAHAS.iter().zip(&r.recorded))
        .filter(|(g, v)| {
            let rupas = v["total_rupas"].as_f64().unwrap_or(0.0);
            let engine = v["is_sufficient"].as_bool().unwrap_or(false);
            engine != (rupas >= required[index(**g)])
        })
        .count();
    claims.push(Claim::counted(
        "BPHS vv. 32 and 33's requirements, 390, 360, 300, 420, 390, 330 and 300 virupas, the Sun's 6.5 rupas where the engine asks 5",
        flipped,
        cells,
    ));
    claims
}

fn text_claims(records: &[Record]) -> Vec<Claim> {
    let cells = records.len() * 7;
    let mut claims = Vec::new();

    let (differ, far) = rival(records, "sthana.saptavargiya", HALF, |g, r| {
        r.vargas
            .iter()
            .enumerate()
            .map(|(k, row)| compound_virupas(g, row[index(g)], k == 0, r))
            .sum()
    });
    claims.push(
        Claim::counted(
            "Saptavargaja by the compound relationship (moolatrikona 45, own 30, 22.5, 15, 7.5, 3.75, 1.875), exaltation not counted",
            differ,
            cells,
        )
        .with_note(format!("worst gap {far:.2}")),
    );

    let (differ, far) = rival(records, "kaala.nathonnatha", HALF, text_nathonnatha);
    claims.push(
        Claim::counted(
            "Nathonnatha from midnight at any hour: the night grahas twice the nata in ghatis, the day grahas 60 less that (vv. 8 and 9)",
            differ,
            cells,
        )
        .with_note(format!("worst gap {far:.2}")),
    );

    let before = records
        .iter()
        .filter(|r| !r.day && r.jd < r.sunrise)
        .count();
    claims.push(Claim::stated(
        "a birth before sunrise is measured in the night it falls in",
        if before == 0 {
            crate::measure::Verdict::Holds
        } else {
            crate::measure::Verdict::Falsified
        },
        format!(
            "the engine measures {before} of {} charts' nights from the day's own sunset, after the birth: every night graha's Nathonnatha and the Tribhaga lord are lost",
            count(records.len())
        ),
    ));

    let (differ, far) = rival(records, "kaala.ayana", HALF, |g, r| {
        if g == Graha::Sun {
            2.0 * ayana(g, r)
        } else {
            ayana(g, r)
        }
    });
    claims.push(
        Claim::counted(
            "the Sun's Ayana counted in Kaala, doubled (v. 17)",
            differ,
            cells,
        )
        .with_note(format!("worst gap {far:.2}")),
    );

    let (differ, far) = rival(records, "cheshta", HALF, |g, r| {
        if g == Graha::Moon {
            paksha(g, r)
        } else {
            cheshta(g, r)
        }
    });
    claims.push(
        Claim::counted("the Moon's Cheshta is her Paksha (v. 18)", differ, cells)
            .with_note(format!("worst gap {far:.2}")),
    );

    claims.extend(text_table_claims(records));
    claims
}

fn page(root: &Path) -> Result<String, String> {
    let records = records(root)?;
    if records.is_empty() {
        return Err(String::from("the corpus records no Shadbala"));
    }
    let mut out = String::new();
    let _ = write!(
        out,
        "# The Shadbala, measured\n\n\
         Status: `generated` by `cargo xtask shadbala` over the conformance corpus's \
         `baseline/shadbala`, 2026-09-15. Do not edit: `check-shadbala` regenerates this page and \
         fails on any difference. The design it measures is \
         [`strength-schemes.md`](strength-schemes.md).\n\n\
         The corpus records the recording engine's Shadbala for {charts} charts, every component of \
         each of the seven grahas, computed from the recorded chart, day and houses the files repeat. \
         BPHS ch. 27 was read beside it.\n\n\
         ## The engine's reading, component by component\n\n\
         Each rule is reproduced from the inputs alone and compared with every recorded cell, within \
         1e-9 of a virupa.\n\n{engine}\n\
         ## The text's reading where the inputs decide it\n\n\
         Each is compared with the engine's reproduction; a cell differs when the two stand more than \
         half a virupa apart, the natural strengths when they differ at all.\n\n{text}\n",
        charts = count(records.len()),
        engine = table(&engine_claims(&records)),
        text = table(&text_claims(&records)),
    );
    let _ = write!(
        out,
        "## What it means for the module\n\n\
         **The engine's arithmetic is settled**: every component reproduces from the recorded inputs, \
         so the conformance profile can take it whole.\n\n\
         **The text is not the engine**, and a rank-1 text corrects a rank-2 value, so the module's \
         defaults follow BPHS wherever the chapter decides, each fork a setting with the engine's \
         reading as its other value: the Saptavargaja by the compound relationship (C64), Nathonnatha \
         from midnight and a pre-dawn birth's night from the previous evening (C65), the Sun's Ayana \
         doubled, the Moon's Cheshta her Paksha and the kranti from the ephemeris (C66), the Abda and \
         Masa lords from the ahargana (C67), Dig from the true angles (C68), Drik by the chapter's \
         quarters (C69), and the chapter's requirements and exact natural strengths (C71).\n\n\
         **What neither settles**: ch. 26's sphuta drishti, which the translation read garbles, so \
         Drik reads the graded whole-sign drishti under both; the seeghra kendra's mean elements, \
         which only the engine gives; and the Yuddha bala, which is not built (C69, C70).\n",
    );
    Ok(fill(&out))
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask shadbala") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

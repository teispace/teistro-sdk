//! Accidental fortitudes, measured (cruxes C211–C216): what each of the
//! shipped readings decides over the corpus's recorded births, with the
//! premise each crux's decision rests on counted rather than believed.
//!
//! The printed figures that are the acceptance test live in
//! `crates/hellenistic` as unit tests; this page asks how often the
//! choices those figures could not decide fall on real skies.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::hellenistic::house_of;
use teistro::{
    Accident, AccidentalRules, Context, Ephemeris, FortitudeRequest, Fortitudes, Partile,
};

use crate::births::{CHARTS, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table};

const PAGE: &str = "docs/03-design/fortitudes-measured.md";

/// Lilly's tropical zodiac over the conformance profile's sky.
fn tropical() -> Result<Context, String> {
    Context::builder()
        .profile("conformance-baseline")
        .settings_json(r#"{"frame": {"zodiac": "TROPICAL"}}"#)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the tropical conformance profile: {why}"))
}

/// The four relations to the Sun, one of which every planet but the Sun
/// holds.
const SOLAR: [Accident; 4] = [
    Accident::Cazimi,
    Accident::Combust,
    Accident::UnderBeams,
    Accident::FreeFromCombustion,
];

/// One birth read under the shipped rules and under each rival a crux
/// weighed.
struct Read {
    lilly: Fortitudes,
    /// Partile within a degree of the exact aspect (C216).
    within_a_degree: Fortitudes,
    /// Combust whatever the sign (C211).
    any_sign: Fortitudes,
}

/// Each planet of a read, its longitude beside it.
fn planets(read: &Fortitudes) -> impl Iterator<Item = (f64, &teistro::PlanetAccidents)> {
    read.dignities
        .planets
        .iter()
        .map(|at| at.longitude_deg)
        .zip(&read.planets)
}

/// How many planets meet a line, over every read.
fn meeting(reads: &[&Fortitudes], line: Accident) -> usize {
    reads
        .iter()
        .flat_map(|read| &read.planets)
        .filter(|at| at.accidents.contains(&line))
        .count()
}

/// The house the later practice gives: a planet within the orb **before** a
/// cusp is in its house, and otherwise the house whose cusp it passed.
fn before_only(longitude: f64, cusps: &[f64; 12], orb: f64) -> u8 {
    let plain = house_of(longitude, cusps, 0.0);
    let next = usize::from(plain.get() % 12);
    let ahead = cusps
        .get(next)
        .map_or(f64::INFINITY, |cusp| (cusp - longitude).rem_euclid(360.0));
    if ahead <= orb {
        u8::try_from(next).map_or(plain.get(), |next| next + 1)
    } else {
        plain.get()
    }
}

fn structural_claims(reads: &[Read]) -> Vec<Claim> {
    let lilly: Vec<&Fortitudes> = reads.iter().map(|read| &read.lilly).collect();
    let all = reads.len() * 7;
    let solar_wrong = lilly
        .iter()
        .flat_map(|read| &read.planets)
        .filter(|at| {
            let held = SOLAR
                .iter()
                .filter(|line| at.accidents.contains(line))
                .count();
            held != usize::from(at.planet != Graha::Sun)
        })
        .count();
    let rules = AccidentalRules::LILLY;
    let mut moved = 0;
    let mut beyond_next = 0;
    let mut rival_differs = 0;
    for read in &lilly {
        let cusps = &read.sky.cusps_deg;
        for (longitude, at) in planets(read) {
            let plain = house_of(longitude, cusps, 0.0).get();
            if at.house.get() != plain {
                moved += 1;
                if at.house.get() != plain % 12 + 1 {
                    beyond_next += 1;
                }
            }
            if before_only(longitude, cusps, rules.cusp_orb_deg) != at.house.get() {
                rival_differs += 1;
            }
        }
    }
    vec![
        Claim::counted(
            "every planet but the Sun stands in exactly one relation to him (cazimi, combust, under the beams or free), and the Sun in none",
            solar_wrong,
            all,
        ),
        Claim::counted(
            "the five-degree rule moves a planet into the next house or leaves it (p. 33)",
            beyond_next,
            all,
        )
        .with_note(format!("{} planets moved by it", count(moved))),
        Claim::counted(
            "C214: the text's nearest cusp, before or after it, places every planet as \"five degrees before the cusp\" does",
            rival_differs,
            all,
        )
        .with_note("they part only in a house narrower than the two orbs together"),
    ]
}

fn rival_claims(reads: &[Read]) -> Vec<Claim> {
    let lilly: Vec<&Fortitudes> = reads.iter().map(|read| &read.lilly).collect();
    let near: Vec<&Fortitudes> = reads.iter().map(|read| &read.within_a_degree).collect();
    let anywhere: Vec<&Fortitudes> = reads.iter().map(|read| &read.any_sign).collect();
    let partile_lines = [
        Accident::ConjunctBenefic,
        Accident::TrineBenefic,
        Accident::SextileBenefic,
        Accident::ConjunctMalefic,
        Accident::OpposedMalefic,
        Accident::SquareMalefic,
        Accident::ConjunctNorthNode,
        Accident::ConjunctSouthNode,
    ];
    let same: usize = partile_lines
        .iter()
        .map(|&line| meeting(&lilly, line))
        .sum();
    let degree: usize = partile_lines.iter().map(|&line| meeting(&near, line)).sum();
    let combust = meeting(&lilly, Accident::Combust);
    let combust_anywhere = meeting(&anywhere, Accident::Combust);
    let beams = meeting(&lilly, Accident::UnderBeams);
    vec![
        Claim::stated(
            "C211: what the sign clause decides — planets within 8°30′ of the Sun but in another sign, read under the beams rather than combust",
            Verdict::Holds,
            format!(
                "{} combust in the Sun's sign; {} more would be combust whatever the sign",
                count(combust),
                count(combust_anywhere - combust)
            ),
        ),
        Claim::stated(
            "C212: how many planets the stated beams (17°) take, which the Chapter XXVIII tally never scores",
            Verdict::Holds,
            format!("{} planets under the beams", count(beams)),
        ),
        Claim::stated(
            "C216: partile lines read by the same degree, against within a degree of the exact aspect",
            Verdict::Holds,
            format!(
                "{} by the same degree; {} within a degree",
                count(same),
                count(degree)
            ),
        ),
    ]
}

/// How often each line holds, over every planet of every birth.
fn line_counts(reads: &[Read]) -> String {
    let lilly: Vec<&Fortitudes> = reads.iter().map(|read| &read.lilly).collect();
    let lines = [
        Accident::Direct,
        Accident::Retrograde,
        Accident::Swift,
        Accident::Slow,
        Accident::Oriental,
        Accident::Occidental,
        Accident::Increasing,
        Accident::Decreasing,
        Accident::FreeFromCombustion,
        Accident::Cazimi,
        Accident::Combust,
        Accident::UnderBeams,
        Accident::ConjunctBenefic,
        Accident::ConjunctNorthNode,
        Accident::TrineBenefic,
        Accident::SextileBenefic,
        Accident::ConjunctMalefic,
        Accident::ConjunctSouthNode,
        Accident::OpposedMalefic,
        Accident::SquareMalefic,
        Accident::Besieged,
        Accident::Regulus,
        Accident::Spica,
        Accident::Algol,
    ];
    let mut out = String::from("| line | planets |\n|---|---|\n");
    for line in lines {
        let _ = writeln!(out, "| `{line:?}` | {} |", count(meeting(&lilly, line)));
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = tropical()?;
    let births = births(root, &sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    let lilly = FortitudeRequest::default();
    let within_a_degree =
        lilly.with_rules(AccidentalRules::LILLY.with_partile(Partile::Within { orb_deg: 1.0 }));
    let mut any_sign_rules = AccidentalRules::LILLY;
    any_sign_rules.combustion_in_sign = false;
    let any_sign = lilly.with_rules(any_sign_rules);
    let reads = births
        .iter()
        .map(|birth| {
            let read = |request: &FortitudeRequest| {
                sdk.chart()
                    .fortitudes(&birth.document, request)
                    .map_err(|why| format!("{}: {why}", birth.name))
            };
            Ok(Read {
                lilly: read(&lilly)?,
                within_a_degree: read(&within_a_degree)?,
                any_sign: read(&any_sign)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let claims: Vec<Claim> = structural_claims(&reads)
        .into_iter()
        .chain(rival_claims(&reads))
        .collect();
    let mut out = String::from(
        "# Accidental fortitudes, measured\n\n\
         Status: `generated` by `cargo xtask fortitudes` from the corpus's \
         recorded births, 2026-10-02. Do not edit: `check-fortitudes` \
         regenerates this page and fails on any difference.\n\n\
         Lilly's accidental fortitudes (`essential-dignities.md` \
         §Accidental fortitudes) are held to two of his printed figures by \
         the unit tests of `crates/hellenistic`. What those figures cannot \
         decide is how the shipped readings fall on other skies, so this \
         page counts it. ",
    );
    let _ = write!(
        out,
        "It reads each of the corpus's {} births in the tropical zodiac, \
         Lilly's, through `ChartArea::fortitudes` under the default request \
         (Regiomontanus houses, Lilly's orbs and scores), and again under \
         the rival each crux weighed.\n\n",
        count(births.len())
    );
    out.push_str(&table(&claims));
    out.push_str("\n## How often each line holds\n\n");
    out.push_str(&line_counts(&reads));
    out.push_str(
        "\n## What it means\n\n\
         The first three rows hold the shipped rules to the shape the text \
         gives them. The last three weigh each crux's rival on these skies: \
         their counts are how many lines the choice moves, and each rival \
         is a knob of `AccidentalRules` for a reader who decides the other \
         way.\n",
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
        Ok(text) => {
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask fortitudes") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

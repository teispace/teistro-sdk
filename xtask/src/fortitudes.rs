//! Accidental fortitudes and the almuten, measured (cruxes C211–C220):
//! what each of the shipped readings decides over the corpus's recorded
//! births, with the premise each crux's decision rests on counted rather
//! than believed.
//!
//! The printed figures that are the acceptance test live in
//! `crates/hellenistic` as unit tests; this page asks how often the
//! choices those figures could not decide fall on real skies.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::hellenistic::{almuten_of, almuten_of_places, house_of, part_of_fortune};
use teistro::{
    Accident, AccidentalRules, Almuten, Context, Ephemeris, FortitudeRequest, Fortitudes,
    FortuneRule, Partile, PlaceReading, Sect,
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
    /// The chart's ascendant and midheaven, from its angles.
    angles: (f64, f64),
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

/// Chapter CV's places almuten of a read: its ascendant, midheaven, Sun,
/// Moon and Part of Fortune, Fortune taken by `fortune`.
fn places_almuten(read: &Read, fortune: FortuneRule) -> Result<Almuten, String> {
    let lilly = &read.lilly;
    let dignities = &lilly.dignities;
    let (ascendant, midheaven) = read.angles;
    let at = |planet: Graha| {
        dignities
            .planets
            .iter()
            .find(|each| each.planet == planet)
            .map(|each| each.longitude_deg)
            .ok_or_else(|| format!("a read without {planet:?}"))
    };
    let (sun, moon) = (at(Graha::Sun)?, at(Graha::Moon)?);
    let places = [
        ascendant,
        midheaven,
        sun,
        moon,
        part_of_fortune(ascendant, sun, moon, dignities.sect, fortune),
    ];
    almuten_of_places(
        &places,
        dignities.sect,
        &dignities.rules,
        &dignities.scores,
        PlaceReading::Degree,
    )
    .map_err(|why| why.to_string())
}

/// The three almutens over the corpus (C218–C220): how often each ties,
/// how often Lilly's and Chapter CV's agree, and what each rival moves.
fn almuten_claims(reads: &[Read]) -> Result<Vec<Claim>, String> {
    let (mut figure_tied, mut places_tied, mut agree) = (0, 0, 0);
    let (mut houses, mut sign_moves) = (0, 0);
    let (mut nights, mut reversal_moves) = (0, 0);
    for read in reads {
        let figure = read.lilly.almuten().almutens();
        let places = places_almuten(read, FortuneRule::DayAndNight)?.almutens();
        figure_tied += usize::from(figure.len() > 1);
        places_tied += usize::from(places.len() > 1);
        agree += usize::from(figure == places);
        let dignities = &read.lilly.dignities;
        for &cusp in &read.lilly.sky.cusps_deg {
            let by = |reading| {
                almuten_of(
                    cusp,
                    dignities.sect,
                    &dignities.rules,
                    &dignities.scores,
                    reading,
                )
                .map(|at| at.almutens())
                .map_err(|why| why.to_string())
            };
            houses += 1;
            sign_moves += usize::from(by(PlaceReading::Degree)? != by(PlaceReading::Sign)?);
        }
        if dignities.sect == Sect::Night {
            nights += 1;
            let reversed = places_almuten(read, FortuneRule::ReversedByNight)?.almutens();
            reversal_moves += usize::from(reversed != places);
        }
    }
    let births = reads.len();
    Ok(vec![
        Claim::stated(
            "C219: births whose almuten is tied, which the shipped reading reports rather than breaks",
            Verdict::Holds,
            format!(
                "the figure's in {} of {}; the five places' in {}",
                count(figure_tied),
                count(births),
                count(places_tied)
            ),
        ),
        Claim::stated(
            "Lilly's almuten of the figure (the greatest net) and Chapter CV's (the most essential dignities over the ascendant, midheaven, Sun, Moon and Fortune) name the same planets",
            Verdict::Holds,
            format!("in {} of {} births", count(agree), count(births)),
        ),
        Claim::stated(
            "C218: houses whose almuten changes when the cusp's sign is read instead of its degree",
            Verdict::Holds,
            format!("{} of {} houses", count(sign_moves), count(houses)),
        ),
        Claim::stated(
            "C220: night births whose places' almuten moves when Fortune is reversed by night",
            Verdict::Holds,
            format!(
                "{} of {} night births",
                count(reversal_moves),
                count(nights)
            ),
        ),
    ])
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
                angles: sdk
                    .chart()
                    .angles(&birth.document)
                    .map(|at| (at.ascendant_deg, at.midheaven_deg))
                    .map_err(|why| format!("{}: {why}", birth.name))?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let claims: Vec<Claim> = structural_claims(&reads)
        .into_iter()
        .chain(rival_claims(&reads))
        .chain(almuten_claims(&reads)?)
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
         gives them. The next three weigh each crux's rival on these skies: \
         their counts are how many lines the choice moves, and each rival \
         is a knob of `AccidentalRules` for a reader who decides the other \
         way. The last four read the almuten (§The almuten) three ways, \
         Lilly's of the figure, Chapter CV's over five places, and each \
         house's of its cusp, and count what C218 and C220's rivals move; \
         those rivals are `PlaceReading::Sign` and \
         `FortuneRule::ReversedByNight`.\n",
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

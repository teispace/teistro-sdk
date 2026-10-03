//! A Western chart's houses, measured (`western-houses.md`, C249, C250):
//! every recorded birth's planets counted in Placidus and in the other
//! divisions a Western reader might choose, how often the ascendant's
//! reach of one sidereal hour holds a planet of the twelfth house, and
//! how wide that hour is in longitude by latitude.
//!
//! The births are founded under the conformance profile with the outer
//! planets, which is sidereal; each is founded again under
//! `western-tropical-default` to hold the claim that the zodiac does not
//! move a house.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{Graha, HouseSystem};
use teistro::{Context, Ephemeris, HouseRequest, Status, WesternHouses};
use teistro_core::house::house_of;

use crate::births::{CHARTS, births, conformance};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, median, share, table, worst};

const PAGE: &str = "docs/03-design/western-houses-measured.md";

/// The divisions read beside Placidus: Lilly's, the two modern quadrant
/// systems most offered, and the three that divide the ecliptic.
const RIVALS: [HouseSystem; 6] = [
    HouseSystem::Regiomontanus,
    HouseSystem::Koch,
    HouseSystem::Campanus,
    HouseSystem::Porphyry,
    HouseSystem::Equal,
    HouseSystem::WholeSign,
];

/// How far apart two foundings must place a planet for its own place, and
/// not the zodiac, to have moved its house, degrees.
const PLACE_DEG: f64 = 0.01;

/// The latitude bands the hour's width is reported in, degrees from the
/// equator, each up to and excluding its end.
const BANDS: [(f64, f64); 4] = [(0.0, 20.0), (20.0, 40.0), (40.0, 55.0), (55.0, 90.0)];

/// One birth read.
struct Read {
    /// Its latitude, degrees.
    latitude_deg: f64,
    /// Its houses in Placidus, founded sidereal.
    placidus: WesternHouses,
    /// Its houses in each rival division, in [`RIVALS`]' order.
    rivals: Vec<WesternHouses>,
    /// The planets the tropical founding counts in another house.
    moved: Vec<Moved>,
}

/// A planet the two foundings of a birth count differently.
struct Moved {
    birth: String,
    graha: Graha,
    /// How far apart the two foundings place it, tropical, degrees.
    arc_deg: f64,
}

/// The shorter arc between two longitudes, degrees.
fn arc(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

impl Read {
    /// Whether the degree that rose an hour before stands outside the
    /// twelfth house, which Leo takes it to be in.
    fn reaches_past_twelfth(&self) -> bool {
        let frame = &self.placidus.frame;
        house_of(frame.reach_deg, &frame.cusps_deg, 0.0).get() != 12
    }

    /// How much longitude the hour before the birth raised, degrees.
    fn hour_deg(&self) -> f64 {
        let frame = &self.placidus.frame;
        (frame.ascendant_deg - frame.reach_deg).rem_euclid(360.0)
    }
}

fn tropical() -> Result<Context, String> {
    Context::builder()
        .profile("western-tropical-default")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the tropical profile: {why}"))
}

/// Every birth read, and the names of those Placidus is undefined at under
/// the conformance profile's polar policy, which refuses rather than falls
/// back.
fn measure(root: &Path) -> Result<(Vec<Read>, Vec<String>), String> {
    let sdk = conformance()?;
    let western = tropical()?;
    let mut reads = Vec::new();
    let mut polar = Vec::new();
    for birth in births(root, &sdk)? {
        let named = |why: teistro::Error| format!("{}: {why}", birth.name);
        let chart = birth.with_outer_planets(&sdk)?;
        let asked = |system: Option<HouseSystem>| HouseRequest { system };
        let placidus = match sdk.chart().western_houses(&chart, &asked(None)) {
            Ok(houses) => houses,
            Err(why) if why.status == Status::Unsupported => {
                polar.push(birth.name.clone());
                continue;
            }
            Err(why) => return Err(named(why)),
        };
        let turned = birth.with_outer_planets(&western)?;
        let tropical = western
            .chart()
            .western_houses(&turned, &asked(None))
            .map_err(named)?;
        let rivals = RIVALS
            .iter()
            .map(|&system| {
                sdk.chart()
                    .western_houses(&chart, &asked(Some(system)))
                    .map_err(named)
            })
            .collect::<Result<_, _>>()?;
        let moved = placidus
            .planets
            .iter()
            .zip(&tropical.planets)
            .filter(|(ours, theirs)| ours != theirs)
            .map(|(ours, _)| {
                let there = |document: &teistro::Document| {
                    document
                        .foundation
                        .graha(ours.graha)
                        .map(|at| at.tropical_deg)
                        .ok_or_else(|| format!("{}: no {}", birth.name, ours.graha.key()))
                };
                Ok(Moved {
                    birth: birth.name.clone(),
                    graha: ours.graha,
                    arc_deg: arc(there(&chart)?, there(&turned)?),
                })
            })
            .collect::<Result<_, String>>()?;
        reads.push(Read {
            latitude_deg: chart.foundation.place.latitude.get(),
            placidus,
            rivals,
            moved,
        });
    }
    if reads.is_empty() {
        return Err(format!("{CHARTS} records no charts Placidus is defined at"));
    }
    Ok((reads, polar))
}

fn claims(reads: &[Read], polar: &[String]) -> Vec<Claim> {
    let moved = moved(reads);
    let in_place = moved.iter().filter(|one| one.arc_deg < PLACE_DEG).count();
    let outside = reads
        .iter()
        .filter(|read| read.reaches_past_twelfth())
        .count();
    vec![
        Claim::counted(
            format!(
                "a planet the sidereal and the tropical founding count in different houses stands {PLACE_DEG}° or more apart in them: the zodiac moves no house"
            ),
            in_place,
            moved.len(),
        ),
        Claim::counted(
            "Placidus is defined at every recorded birthplace",
            polar.len(),
            reads.len() + polar.len(),
        ),
        Claim::counted(
            "the degree that rose an hour before the birth stands in the twelfth house under Placidus",
            outside,
            reads.len(),
        ),
    ]
}

/// How many planets each rival counts in another house than Placidus.
fn by_division(reads: &[Read]) -> String {
    let mut out = String::from(
        "| division | planets | in another house | share | births with one moved | share |\n|---|---|---|---|---|---|\n",
    );
    for (at, system) in RIVALS.iter().enumerate() {
        let mut planets = 0;
        let mut moved = 0;
        let mut births = 0;
        for read in reads {
            let Some(rival) = read.rivals.get(at) else {
                continue;
            };
            let differ = read
                .placidus
                .planets
                .iter()
                .zip(&rival.planets)
                .filter(|(ours, theirs)| ours.house != theirs.house)
                .count();
            planets += read.placidus.planets.len();
            moved += differ;
            births += usize::from(differ > 0);
        }
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            system.key().to_lowercase().replace('_', " "),
            count(planets),
            count(moved),
            share(moved, planets),
            count(births),
            share(births, reads.len()),
        );
    }
    out
}

/// How wide the hour before the birth is in longitude, by latitude.
fn by_latitude(reads: &[Read]) -> String {
    let mut out = String::from(
        "| latitude | births | median | narrowest | widest |\n|---|---|---|---|---|\n",
    );
    for (from, to) in BANDS {
        let hours: Vec<f64> = reads
            .iter()
            .filter(|read| (from..to).contains(&read.latitude_deg.abs()))
            .map(Read::hour_deg)
            .collect();
        if hours.is_empty() {
            let _ = writeln!(out, "| {from}° to {to}° | 0 | | | |");
            continue;
        }
        let narrowest = hours.iter().copied().fold(f64::INFINITY, f64::min);
        let _ = writeln!(
            out,
            "| {from}° to {to}° | {} | {:.1}° | {:.1}° | {:.1}° |",
            count(hours.len()),
            median(hours.iter().copied()),
            narrowest,
            worst(hours.iter().copied()),
        );
    }
    out
}

/// The planets the two foundings of a birth count differently, every
/// birth's.
fn moved(reads: &[Read]) -> Vec<&Moved> {
    reads.iter().flat_map(|read| &read.moved).collect()
}

/// The zodiac against the profile: each planet the tropical founding
/// counts in another house, and how far apart the two placed it.
fn by_profile(reads: &[Read]) -> String {
    let mut out = String::new();
    out.push_str(
        "\n## The zodiac and the profile\n\n\
         Each birth is founded again under `western-tropical-default`, and \
         its houses read in that zodiac. The two profiles differ in more \
         than the zodiac, so a planet they place differently can change \
         house where it stands near a cusp; the zodiac alone turns every \
         cusp and every planet together and moves none.\n\n\
         | birth | planet | apart in the two foundings |\n|---|---|---|\n",
    );
    for one in moved(reads) {
        let _ = writeln!(
            out,
            "| {} | {} | {:.3}° |",
            one.birth,
            one.graha.key().to_lowercase(),
            one.arc_deg
        );
    }
    out
}

/// The ascendant's reach: how many planets it holds, how wide the hour
/// is by latitude, and where it passes the twelfth cusp.
fn reach(reads: &[Read]) -> String {
    let twelfth: Vec<bool> = reads
        .iter()
        .flat_map(|read| &read.placidus.planets)
        .filter(|placed| placed.house.get() == 12)
        .map(|placed| placed.with_ascendant)
        .collect();
    let reached = twelfth.iter().filter(|&&with| with).count();
    let first = reads
        .iter()
        .flat_map(|read| &read.placidus.planets)
        .filter(|placed| placed.house.get() == 1)
        .count();
    let mut out = String::new();
    let _ = write!(
        out,
        "\n## The ascendant's reach\n\n\
         Leo counts with the ascendant everything up to the degree that \
         rose one sidereal hour before the birth (C250). Of the {} planets \
         in the twelfth house, {} ({}) stand within that reach; {} more \
         stand in the first house itself. The hour is 15° of right \
         ascension, and the table gives how much longitude it raised, \
         which Leo calls half a sign \"in a great many cases\". It is \
         widest where signs rise quickly, so it depends on the sign rising \
         as much as on the latitude.\n\n",
        count(twelfth.len()),
        count(reached),
        share(reached, twelfth.len()),
        count(first),
    );
    out.push_str(&by_latitude(reads));
    let past = reads
        .iter()
        .filter(|read| read.reaches_past_twelfth())
        .count();
    let _ = write!(
        out,
        "\nLeo takes the degree to \"be found in the twelfth house, of \
         course\" (p. 90). It is not at {} of the {} births: where the \
         signs about the ascendant rise quickly, Placidus's twelfth house \
         is narrower than the hour, and the reach runs past its cusp into \
         the eleventh. The flag still follows the degree, so a planet of \
         the eleventh within it is read with the ascendant.\n",
        count(past),
        count(reads.len()),
    );
    out
}

fn page(root: &Path) -> Result<String, String> {
    let (reads, polar) = measure(root)?;
    let planets: usize = reads.iter().map(|read| read.placidus.planets.len()).sum();
    let mut out = String::from(
        "# A Western chart's houses, measured\n\n\
         Status: `generated` by `cargo xtask western-houses` from the \
         corpus's recorded births, 2026-10-03. Do not edit: \
         `check-western-houses` regenerates this page and fails on any \
         difference.\n\n",
    );
    let _ = write!(
        out,
        "Each of the corpus's {} births Placidus is defined at is founded under \
         `conformance-baseline` with the outer planets and its {} planets \
         counted in Placidus, the division Leo's figures are cast in \
         (`western-houses.md`, C249), and again in six other divisions \
         and in the tropical zodiac.\n\n",
        count(reads.len()),
        count(planets),
    );
    out.push_str(&table(&claims(&reads, &polar)));
    if !polar.is_empty() {
        let _ = write!(
            out,
            "\nPlacidus divides the semi-arcs, which do not exist inside the \
             polar circles, and the conformance profile refuses rather than \
             falls back, so {} left out of everything below: {}. A profile \
             that falls back, as `western-tropical-default` does to \
             Porphyry, reads them, and the answer names the division used.\n",
            if polar.len() == 1 {
                "one birth is"
            } else {
                "these births are"
            },
            polar.join(", "),
        );
    }
    out.push_str(&by_profile(&reads));
    out.push_str(
        "\n## The division a planet is counted in\n\n\
         How many planets each division counts in another house than \
         Placidus, over every birth. This is the size of C249: a reader \
         who takes Lilly's Regiomontanus, or a modern default, for Leo's \
         houses moves this many planets.\n\n",
    );
    out.push_str(&by_division(&reads));
    out.push_str(&reach(&reads));
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
        Ok(text) => i32::from(
            check(
                root,
                &[Output::new(PAGE, text)],
                "cargo xtask western-houses",
            ) != 0,
        ),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

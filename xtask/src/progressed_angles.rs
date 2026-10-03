//! The progressed angles, measured (crux C237): each way of moving the
//! birth's meridian held against Leo's own map over the corpus's births,
//! age by age, so the size of the choice is stated in degrees rather than
//! argued.
//!
//! The acceptance tests that hold each method to Leo's worked figures
//! live in `crates/sdk/tests/progressions.rs`; this page asks how far the
//! methods part on real skies, and holds the bounds the arithmetic gives.

use std::fmt::Write as _;
use std::path::Path;

use teistro::western::AngleMethod;
use teistro::{ChartAngles, ProgressionRequest, Status};

use crate::births::{Birth, CHARTS, births};
use crate::fortitudes::tropical;
use crate::generated::{Output, check, write};
use crate::measure::{
    Claim, Verdict, capitalised, count, fill, listed, median, plural, spelled, table, worst,
};

const PAGE: &str = "docs/03-design/progressed-angles-measured.md";

/// The ages read, in tropical years of life: whole years, so a day of sky
/// is a whole day under the default measure.
const AGES: [u32; 8] = [10, 20, 30, 40, 50, 60, 70, 80];

/// The tropical year the default measure counts a life in, days.
const YEAR_DAYS: f64 = teistro::western::TROPICAL_YEAR_DAYS;

/// How far two angles may stand apart and still be one: the methods all
/// return the radical meridian for no sky at all, but for rounding.
const SAME_DEG: f64 = 1e-6;

/// Twice the Sun's greatest equation of the centre (1.915°, eccentricity
/// 0.0167): the most a true arc and a mean arc along the ecliptic can part
/// over any span, whatever its length.
const CENTRE_BOUND_DEG: f64 = 2.0 * 1.915;

/// The equation of time's whole range, from about −14.2 minutes in
/// February to about +16.4 in November, as degrees of right ascension:
/// the most a true arc and a mean arc along the equator can part.
const TIME_BOUND_DEG: f64 = (14.2 + 16.4) / 4.0;

/// How near the quotidian meridian stands to Leo's at a birthday: the
/// sidereal day's excess is what both add, so they part only by the
/// sidereal clock's slower terms.
const BIRTHDAY_DEG: f64 = 0.05;

/// The angles a method moves the meridian to, at each age.
struct Read {
    /// The birth's own angles, which every method must return at birth.
    radical: ChartAngles,
    /// Each method's angles for no sky at all.
    at_birth: Vec<ChartAngles>,
    /// Each age's angles, a row an age and a column a method, in
    /// `AngleMethod::ALL`'s order.
    by_age: Vec<Vec<ChartAngles>>,
    /// The quotidian and Leo's angles half a year past each age.
    half_years: Vec<(ChartAngles, ChartAngles)>,
}

/// The shorter way round between two longitudes, degrees.
fn apart(one: f64, other: f64) -> f64 {
    let gap = (one - other).rem_euclid(360.0);
    gap.min(360.0 - gap)
}

fn column(method: AngleMethod) -> usize {
    AngleMethod::ALL
        .iter()
        .position(|one| *one == method)
        .unwrap_or_default()
}

/// A birth's angles `years` into its life by one method, under Leo's
/// default measure.
fn progressed_at(
    sdk: &teistro::Context,
    birth: &Birth,
    years: f64,
    method: AngleMethod,
) -> Result<ChartAngles, teistro::Error> {
    let life = teistro::quantity::JulianDay::literal(birth.at() + years * YEAR_DAYS);
    sdk.chart()
        .progressed(
            &birth.document,
            life,
            &ProgressionRequest::default().with_angles(method),
            &birth.request(),
        )
        .map(|progressed| progressed.angles)
}

fn read(sdk: &teistro::Context, birth: &Birth) -> Result<Read, teistro::Error> {
    let at = |years: f64, method: AngleMethod| progressed_at(sdk, birth, years, method);
    let every = |years: f64| {
        AngleMethod::ALL
            .iter()
            .map(|method| at(years, *method))
            .collect::<Result<Vec<_>, teistro::Error>>()
    };
    let radical = sdk.chart().angles(&birth.document)?;
    let by_age = AGES
        .iter()
        .map(|age| every(f64::from(*age)))
        .collect::<Result<Vec<_>, teistro::Error>>()?;
    let half_years = AGES
        .iter()
        .map(|age| {
            let years = f64::from(*age) + 0.5;
            Ok((
                at(years, AngleMethod::Quotidian)?,
                at(years, AngleMethod::NaibodRightAscension)?,
            ))
        })
        .collect::<Result<Vec<_>, teistro::Error>>()?;
    Ok(Read {
        radical,
        at_birth: every(0.0)?,
        by_age,
        half_years,
    })
}

/// Every pair of one birth's angles at every age under two methods.
fn pairs(reads: &[Read], one: AngleMethod, other: AngleMethod) -> Vec<(ChartAngles, ChartAngles)> {
    reads
        .iter()
        .flat_map(|read| read.by_age.iter())
        .map(|row| (row[column(one)], row[column(other)]))
        .collect()
}

fn midheavens_apart(pairs: &[(ChartAngles, ChartAngles)]) -> impl Iterator<Item = f64> + '_ {
    pairs
        .iter()
        .map(|(one, other)| apart(one.midheaven_deg, other.midheaven_deg))
}

/// The western profile, the one progressions are made for: tropical, and
/// its day reckoned from midnight, so a polar day refuses nothing.
fn western() -> Result<teistro::Context, String> {
    teistro::Context::builder()
        .profile("western-tropical-default")
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the western profile: {why}"))
}

/// How many of the births' progressed charts, one an age by Leo's map, a
/// context refuses, and at how many births.
fn refused<'a>(
    sdk: &teistro::Context,
    births: impl Iterator<Item = &'a Birth>,
) -> (usize, usize, usize) {
    let (mut refused, mut asked, mut at) = (0, 0, 0);
    for birth in births {
        let before = refused;
        for age in AGES {
            asked += 1;
            refused += usize::from(
                progressed_at(sdk, birth, f64::from(age), AngleMethod::default()).is_err(),
            );
        }
        at += usize::from(refused > before);
    }
    (refused, asked, at)
}

fn claims(reads: &[Read]) -> Vec<Claim> {
    let born_wrong = reads
        .iter()
        .filter(|read| {
            read.at_birth.iter().any(|angles| {
                apart(angles.midheaven_deg, read.radical.midheaven_deg) > SAME_DEG
                    || apart(angles.ascendant_deg, read.radical.ascendant_deg) > SAME_DEG
            })
        })
        .count();
    let by_longitude = pairs(
        reads,
        AngleMethod::SolarArcLongitude,
        AngleMethod::NaibodLongitude,
    );
    let by_equator = pairs(
        reads,
        AngleMethod::SolarArcRightAscension,
        AngleMethod::NaibodRightAscension,
    );
    let birthdays = pairs(
        reads,
        AngleMethod::Quotidian,
        AngleMethod::NaibodRightAscension,
    );
    let half_years: Vec<f64> = reads
        .iter()
        .flat_map(|read| read.half_years.iter())
        .map(|(quotidian, leo)| apart(quotidian.midheaven_deg, leo.midheaven_deg))
        .collect();
    let beyond = |values: &mut dyn Iterator<Item = f64>, bound: f64| {
        values.filter(|gap| *gap > bound).count()
    };
    vec![
        Claim::counted(
            "every method returns the radical midheaven and ascendant for no sky at all",
            born_wrong,
            reads.len(),
        ),
        Claim::counted(
            format!(
                "the solar arc and Naibod's along the ecliptic part by no more than twice the equation of the centre ({CENTRE_BOUND_DEG:.2}°)"
            ),
            beyond(&mut midheavens_apart(&by_longitude), CENTRE_BOUND_DEG),
            by_longitude.len(),
        ),
        Claim::counted(
            format!(
                "the solar arc and Leo's mean Sun in right ascension part by no more than the equation of time's range ({TIME_BOUND_DEG:.2}°)"
            ),
            beyond(&mut midheavens_apart(&by_equator), TIME_BOUND_DEG),
            by_equator.len(),
        ),
        Claim::counted(
            format!(
                "the quotidian meridian is Leo's at every birthday, within {BIRTHDAY_DEG}°"
            ),
            beyond(&mut midheavens_apart(&birthdays), BIRTHDAY_DEG),
            birthdays.len(),
        ),
        Claim::counted(
            "the quotidian meridian stands half the circle from Leo's half a year after a birthday, within 1°",
            half_years.iter().filter(|gap| (180.0 - **gap) > 1.0).count(),
            half_years.len(),
        )
        .with_note("a day of sky turns it a full circle, so between birthdays it reads any sign"),
    ]
}

fn degrees(value: f64) -> String {
    format!("{value:.2}°")
}

/// The methods the tables hold against Leo's: all but his own and the
/// quotidian, which is his at every whole-year age the tables read and
/// is held by the claims instead.
fn compared() -> Vec<AngleMethod> {
    AngleMethod::ALL
        .into_iter()
        .filter(|method| {
            !matches!(
                method,
                AngleMethod::NaibodRightAscension | AngleMethod::Quotidian
            )
        })
        .collect()
}

/// The worst distance from Leo's of any compared method at any age, for
/// one angle.
fn farthest(reads: &[Read], angle: fn(&ChartAngles) -> f64) -> f64 {
    let leo = column(AngleMethod::default());
    worst(
        reads
            .iter()
            .flat_map(|read| read.by_age.iter())
            .flat_map(|row| {
                compared()
                    .into_iter()
                    .map(move |method| apart(angle(&row[column(method)]), angle(&row[leo])))
            }),
    )
}

/// How far each method parts from Leo's map, age by age: the median and
/// the worst over the births, for one angle.
fn spread(reads: &[Read], angle: fn(&ChartAngles) -> f64) -> String {
    let others = compared();
    let mut out = String::from("| age |");
    for method in &others {
        let _ = write!(out, " `{method:?}` |");
    }
    out.push_str("\n|---|");
    out.push_str(&"---|".repeat(others.len()));
    out.push('\n');
    let leo = column(AngleMethod::default());
    for (row, age) in AGES.iter().enumerate() {
        let _ = write!(out, "| {age} |");
        for method in &others {
            let gaps: Vec<f64> = reads
                .iter()
                .map(|read| {
                    let angles = &read.by_age[row];
                    apart(angle(&angles[column(*method)]), angle(&angles[leo]))
                })
                .collect();
            let _ = write!(
                out,
                " {} / {} |",
                degrees(median(gaps.iter().copied())),
                degrees(worst(gaps))
            );
        }
        out.push('\n');
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = western()?;
    let founded = births(root, &sdk)?;
    if founded.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    // A birth whose progressed sky runs past the ephemeris's years is set
    // aside by name; any other refusal fails the page.
    let (mut reads, mut beyond) = (Vec::new(), Vec::new());
    for birth in &founded {
        match read(&sdk, birth) {
            Ok(read) => reads.push(read),
            Err(why) if why.status == Status::OutOfRange => beyond.push(birth.name.clone()),
            Err(why) => return Err(format!("{}: {why}", birth.name)),
        }
    }
    let conformance = tropical()?;
    let in_range = births(root, &conformance)?;
    let (refused, asked, refused_at) = refused(
        &conformance,
        in_range
            .iter()
            .filter(|birth| !beyond.contains(&birth.name)),
    );
    let mut out = String::from(
        "# The progressed angles, measured\n\n\
         Status: `generated` by `cargo xtask progressed-angles` from the \
         corpus's recorded births, 2026-10-03. Do not edit: \
         `check-progressed-angles` regenerates this page and fails on any \
         difference.\n\n\
         C237 (`western-progressions.md`) is how the progressed midheaven \
         moves: Leo's own map advances it by the mean Sun in right \
         ascension, and the twentieth century's habit is the solar arc. No \
         text weighs one against the other, so every method ships as a \
         knob. This page states the choice's size. ",
    );
    let aside = match beyond.len() {
        0 => String::new(),
        1 => format!(
            " {} is left out, as its progressed sky runs past the built-in ephemeris's years.",
            listed(&beyond)
        ),
        n => format!(
            " {} are left out, as their progressed sky runs past the built-in ephemeris's years: {}.",
            capitalised(&spelled(n)),
            listed(&beyond)
        ),
    };
    let _ = write!(
        out,
        "It reads {} of the corpus's births under \
         `western-tropical-default` through `ChartArea::progressed` at the \
         secondary rate and the tropical year, under each `AngleMethod`, at \
         {} ages from {} to {} years.{aside}\n\n",
        count(reads.len()),
        count(AGES.len()),
        AGES[0],
        AGES[AGES.len() - 1],
    );
    let mut measured = claims(&reads);
    measured.push(Claim::stated(
        "under the conformance profile's NEAREST_EVENT polar day, a progressed chart is founded wherever the birth was",
        if refused == 0 { Verdict::Holds } else { Verdict::Falsified },
        format!(
            "{} of {} refused, at {}",
            count(refused),
            count(asked),
            plural(refused_at, "birth")
        ),
    )
    .with_note("a polar day's nearest event can lie weeks from the sky a later age reads; CIVIL_MIDNIGHT holds them all"));
    out.push_str(&table(&measured));
    out.push_str(
        "\n## The midheaven, against Leo's map\n\n\
         Each cell is the median and the worst distance, over the births, \
         between a method's progressed midheaven and Leo's.\n\n",
    );
    out.push_str(&spread(&reads, |angles| angles.midheaven_deg));
    out.push_str(
        "\n## The ascendant, against Leo's map\n\n\
         The ascendant is the one the turned meridian rises with at the \
         birthplace, so a small gap in the meridian is a larger one in \
         the ascendant far from the equator.\n\n",
    );
    out.push_str(&spread(&reads, |angles| angles.ascendant_deg));
    let _ = write!(
        out,
        "\n## What it means\n\n\
         The first row holds that the methods agree where they must. The \
         next two hold the bounds the arithmetic gives: a true arc and a \
         mean one part by the change in the equation of the centre along \
         the ecliptic, and by the change in the equation of time along the \
         equator, whatever the age. The next two say what the quotidian \
         is: Leo's meridian at each birthday, because the sidereal day's \
         excess is what both add, and any meridian at all between \
         birthdays, so the tables leave it out. For the other three the \
         tables are C237's size: up to 80 a method's midheaven stands at \
         most {} from Leo's, and its ascendant at most {}.\n",
        degrees(farthest(&reads, |angles| angles.midheaven_deg)),
        degrees(farthest(&reads, |angles| angles.ascendant_deg)),
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
        Ok(text) => i32::from(
            check(
                root,
                &[Output::new(PAGE, text)],
                "cargo xtask progressed-angles",
            ) != 0,
        ),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

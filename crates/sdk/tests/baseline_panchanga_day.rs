//! The daily panchanga the SDK computes over its own sky, against the 55
//! days the conformance corpus records (`panchanga_day`), under the profile
//! the corpus was recorded with (`conformance-baseline`).
//!
//! `crates/panchanga/tests/baseline.rs` holds what is arithmetic over the
//! corpus's own numbers (the periods over its arcs, the classification of
//! its positions, the tables). What needs a sky is here: the instants the
//! four moving limbs change at, the Moon's rise and set, and the muhurta
//! yogas, which are read off the limbs the sky gives
//! (`03-design/panchanga-day-conventions.md` §§2, 8, 9).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::panic,
    reason = "tests fail by panicking, index the corpus's JSON by key and print the measurement under --nocapture; the shared accuracy recorder panics on a file it cannot write"
)]

#[path = "../../astro/tests/common/mod.rs"]
mod accuracy;

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;
use teistro::catalogue::{Calendar, Karana, MuhurtaYoga, Nakshatra, Tithi, Yoga};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro::{CalendarDate, Context, Ephemeris, Error, Panchanga, Span, Status, UtcOffset};

/// Seconds in a day.
const SECONDS: f64 = 86_400.0;

/// The bound on a limb's change, in terrestrial time: the worst of the 271
/// the corpus records outside the range-edge days measured 7.2 s over the
/// built-in standard tier, median 3.2 s, a figure the two ephemerides'
/// Moons set (an arc-second of the Moon is about two seconds of time, and
/// the tier is a few arc-seconds). Ten seconds keeps a margin over the
/// measurement and is a ninth of the corpus's own band for the standard
/// tier (`fixtures/tolerances.json`, `panchanga_day.*.end_jd`: 0.001 day).
const TRANSITION_SECONDS: f64 = 10.0;

/// The bound on a moonrise or moonset: measured worst 28.0 s, at Ushuaia
/// (54.8°S), from -26.7 to 28.0 s, the rises' and the sets' means each
/// under a third of a second. Where the Sun's rise holds within 2.5 s
/// below 60° (`ACCURACY.md`, rise and set), the Moon's scatters by tens of
/// seconds either way even at the mid latitudes; a horizon convention
/// would move the rises one way and the sets the other, and the tier's
/// Moon (a few arc-seconds, about a second of time at the horizon) is too
/// close to explain it, so the cause is not named yet. The bound is the
/// measurement's, rounded up, so that a change in either sky shows.
const MOON_EVENT_SECONDS: f64 = 30.0;

/// The profile the corpus was recorded under, over the built-in ephemeris.
fn context() -> Context {
    Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

/// The SDK's ΔT less the recording's at a chart's birth, in seconds: the
/// two models part by up to minutes far from the present (the recording's
/// 395.7 s at 2350, c046), and every instant in UT moves by the difference
/// while the sky in terrestrial time does not.
fn delta_t_apart(sdk: &Context, chart: &Value) -> f64 {
    let birth = chart["input"]["resolved"]["jd_ut"].as_f64().unwrap();
    let ours = teistro_astro::delta_t::delta_t(JulianDay::<Ut1>::literal(birth), sdk.delta_t())
        .unwrap()
        .seconds;
    ours - chart["foundation"]["delta_t_days"].as_f64().unwrap() * SECONDS
}

/// The muhurta yogas the corpus flags, by its key, with the catalogue
/// member each is.
const YOGAS: [(&str, MuhurtaYoga); 5] = [
    ("amrit_siddhi", MuhurtaYoga::AmritSiddhi),
    ("sarvartha_siddhi", MuhurtaYoga::SarvarthaSiddhi),
    ("siddha", MuhurtaYoga::Siddha),
    ("dwipushkar", MuhurtaYoga::Dwipushkar),
    ("tripushkar", MuhurtaYoga::Tripushkar),
];

/// The births at the ends of the data's years (tagged `range-start` and
/// `range-end`), whose days the SDK refuses: a day's window and the Moon's
/// search around it reach past the first or the last day the built-in
/// ephemeris holds, where the recording engine's files ran on. The A/B
/// excuses the same two (`05-testing/ab-measured.md`).
const RANGE_EDGES: [&str; 2] = ["c047", "c048"];

/// Every recorded chart, in name order, with the days the SDK computes for
/// its local civil date at its place and `around` days either side of it;
/// the days at [`RANGE_EDGES`] are held to be refused for being out of
/// range, and left out.
fn days(around: i64) -> Vec<(Value, Vec<Panchanga>)> {
    let sdk = context();
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/charts");
    let mut names: Vec<_> = std::fs::read_dir(&directory)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|path| {
            let chart: Value =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            let day = days_of(&sdk, &chart, around);
            (chart, day)
        })
        .filter_map(|(chart, day)| {
            let edge = RANGE_EDGES.contains(&id(&chart));
            match day {
                Ok(day) => {
                    assert!(!edge, "{}: answered at a range edge", id(&chart));
                    Some((chart, day))
                }
                Err(error) => {
                    assert!(
                        edge && error.status == Status::OutOfRange,
                        "{}: {error}",
                        id(&chart)
                    );
                    None
                }
            }
        })
        .collect()
}

/// The day of the proleptic Gregorian calendar `by` days from a date
/// written `YYYY-MM-DD` (Hinnant's civil-day algorithms, round trip).
fn gregorian(date: &str, by: i64) -> CalendarDate {
    let parts: Vec<i64> = date.split('-').map(|part| part.parse().unwrap()).collect();
    let (year, month, day) = (parts[0], parts[1], parts[2]);
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe + by;
    let era = days.div_euclid(146_097);
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    CalendarDate::defined(
        Calendar::Gregorian,
        i32::try_from(year).unwrap(),
        u8::try_from(month).unwrap(),
        u8::try_from(day).unwrap(),
    )
}

/// The SDK's days for a recorded chart: its `panchanga_day`'s local date
/// and `around` days either side, on the clock the section was written on,
/// at the birth's place.
fn days_of(sdk: &Context, chart: &Value, around: i64) -> Result<Vec<Panchanga>, Error> {
    let section = &chart["panchanga_day"];
    let date = section["local_date"].as_str().unwrap();
    let place = &chart["input"]["place"];
    let number = |value: &Value| value.as_f64().unwrap();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a zone offset in seconds is under a day"
    )]
    let offset = (number(&section["tz_offset_hours"]) * 3600.0).round() as i32;
    sdk.almanac()
        .of(
            &gregorian(date, -around),
            &gregorian(date, around),
            &Place::new(
                Latitude::try_new(number(&place["latitude"])).unwrap(),
                Longitude::try_new(number(&place["longitude"])).unwrap(),
                Altitude::try_new(number(&place["altitude_m"])).unwrap(),
            ),
            UtcOffset::try_from_seconds(offset).unwrap(),
        )
        .map(|day| day.value)
}

fn id(chart: &Value) -> &str {
    chart["id"].as_str().unwrap_or_default()
}

/// The instants a limb's members end at over the SDK's days, by member id.
/// A movable karana recurs in three and a half days, so a member can end
/// more than once in three; the comparison takes the one nearest.
fn ends<T: Copy>(
    days: &[Panchanga],
    spans: impl Fn(&Panchanga) -> &[Span<T>],
    id_of: impl Fn(T) -> u16,
) -> BTreeMap<u16, Vec<f64>> {
    let mut ends: BTreeMap<u16, Vec<f64>> = BTreeMap::new();
    for span in days.iter().flat_map(|day| spans(day).iter()) {
        ends.entry(id_of(span.member))
            .or_default()
            .push(span.whole.to.get());
    }
    ends
}

/// Every change of a moving limb the corpus records — each span's end but
/// the window's last, which is the next sunrise and not a change — held to
/// the instant the SDK's own search finds the same member ending at.
///
/// The SDK's days either side are read too: a change is a fact of the sky
/// and not of a window, and on the two polar days the recording's
/// synthesised arcs (entry 3 of the deliberate-difference registry) hold
/// members the SDK's own polar window does not.
///
/// The instants are compared in terrestrial time, the sky's: each side's
/// UT is its TT less its own ΔT, so the SDK's is moved by
/// [`delta_t_apart`] before it is compared. That difference is a model's
/// and is held apart: it is largest at 2350 (c046), where it is minutes
/// and is nearly the whole of that day's UT difference.
#[test]
fn the_limbs_change_when_the_recording_says() {
    let sdk = context();
    let mut errors: Vec<f64> = Vec::new();
    let mut worst = (0.0_f64, String::new());
    let mut worst_ut_apart = (0.0_f64, String::new());
    for (chart, days) in &days(1) {
        let apart = delta_t_apart(&sdk, chart);
        if apart.abs() > 1.0 {
            println!("{}: the ΔT models part by {apart:.2} s", id(chart));
        }
        if apart.abs() > worst_ut_apart.0.abs() {
            worst_ut_apart = (apart, id(chart).to_owned());
        }
        let ours = [
            ("tithi", ends(days, |day| &day.limbs.tithi, Tithi::id)),
            (
                "nakshatra",
                ends(days, |day| &day.limbs.nakshatra, Nakshatra::id),
            ),
            ("yoga", ends(days, |day| &day.limbs.yoga, Yoga::id)),
            ("karana", ends(days, |day| &day.limbs.karana, Karana::id)),
        ];
        for (limb, ours) in ours {
            let recorded = chart["panchanga_day"][limb].as_array().unwrap();
            let changes = recorded.len().saturating_sub(1);
            for span in recorded.iter().take(changes) {
                let member = u16::try_from(span["index"].as_u64().unwrap()).unwrap();
                let recorded_end = span["end_jd"].as_f64().unwrap();
                let end = ours.get(&member).and_then(|ends| {
                    ends.iter().copied().min_by(|a, b| {
                        (a - recorded_end)
                            .abs()
                            .total_cmp(&(b - recorded_end).abs())
                    })
                });
                assert!(
                    end.is_some(),
                    "{} {limb}: the SDK's days hold no member {member}",
                    id(chart)
                );
                let error = ((end.unwrap_or_default() - recorded_end) * SECONDS + apart).abs();
                if error > worst.0 {
                    worst = (error, format!("{} {limb} {member}", id(chart)));
                }
                errors.push(error);
            }
        }
    }
    errors.sort_by(f64::total_cmp);
    let median = errors[errors.len() / 2];
    println!(
        "{} limb changes in TT: worst {:.2} s ({}), median {median:.2} s; the ΔT models \
         part by at most {:.1} s ({})",
        errors.len(),
        worst.0,
        worst.1,
        worst_ut_apart.0,
        worst_ut_apart.1
    );
    // The models part the most at 2350, by minutes.
    assert_eq!(worst_ut_apart.1, "c046");
    assert!(worst_ut_apart.0.abs() > 60.0, "{worst_ut_apart:?}");
    assert_eq!(
        errors.len(),
        271,
        "every change the corpus records, less the nine on the two range-edge days"
    );
    accuracy::record(
        "crossings",
        "the baseline's panchanga limb changes, geocentric, in TT",
        worst.0,
        " s",
        TRANSITION_SECONDS,
        errors.len(),
    );
    assert!(
        worst.0 <= TRANSITION_SECONDS,
        "worst {:.2} s at {}",
        worst.0,
        worst.1
    );
}

/// The Moon's first rise and first set of the civil day
/// (`panchanga.moon_events`, `CIVIL_DAY` under the profile), against the
/// recorded ones: present where the recording has one, absent where it has
/// none, and within the band where both do.
#[test]
fn the_moon_rises_and_sets_when_the_recording_says() {
    let mut compared = 0;
    let mut absent = 0;
    let mut worst = (0.0_f64, String::new());
    // The SDK's instant less the recording's, by kind, signed: a horizon
    // convention moves rises one way and sets the other.
    let mut signed: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    for (chart, days) in &days(0) {
        let day = &days[0];
        for (key, ours) in [
            ("moonrise_jd", day.moon.rises.first()),
            ("moonset_jd", day.moon.sets.first()),
        ] {
            let recorded = chart["panchanga_day"][key].as_f64();
            assert_eq!(
                ours.is_some(),
                recorded.is_some(),
                "{} {key}: ours {ours:?}, recorded {recorded:?}",
                id(chart)
            );
            let (Some(ours), Some(recorded)) = (ours, recorded) else {
                absent += 1;
                continue;
            };
            let offset = (ours.get() - recorded) * SECONDS;
            signed.entry(key).or_default().push(offset);
            if offset.abs() > worst.0 {
                worst = (offset.abs(), format!("{} {key}", id(chart)));
            }
            compared += 1;
        }
    }
    println!(
        "{compared} moon events, worst {:.2} s ({}); {absent} absent from both",
        worst.0, worst.1
    );
    for (key, offsets) in &mut signed {
        offsets.sort_by(f64::total_cmp);
        #[allow(clippy::cast_precision_loss, reason = "a count of tens")]
        let mean = offsets.iter().sum::<f64>() / offsets.len() as f64;
        println!(
            "  {key}: the SDK's less the recording's, mean {mean:.2} s, from {:.2} to {:.2} s",
            offsets[0],
            offsets[offsets.len() - 1]
        );
    }
    assert_eq!(
        (compared, absent),
        (104, 2),
        "the 108 recorded less the four on the range-edge days; c029's polar night has neither"
    );
    accuracy::record(
        "rise_set",
        "the baseline's moonrises and moonsets, the first of each from the civil midnight",
        worst.0,
        " s",
        MOON_EVENT_SECONDS,
        compared,
    );
    assert!(
        worst.0 <= MOON_EVENT_SECONDS,
        "worst {:.2} s at {}",
        worst.0,
        worst.1
    );
}

/// Why a muhurta yoga's recorded flag and the SDK's part: the
/// recording's vara-and-nakshatra tables are not the published ones the SDK
/// ships (`panchanga.muhurta_tables`, `CLASSICAL`), and no rotation of
/// either index brings them together
/// (`03-design/panchanga-day-conventions.md` §8).
const OTHER_TABLE: &str = "the recording's vara-and-nakshatra table, not the published one (§8)";

/// The flags that part, each with its day's vara and sunrise nakshatra (as
/// the catalogue names them, and checked), all
/// for [`OTHER_TABLE`]: the recording sets Sarvartha Siddhi on four
/// pairs the published table lacks and Siddha on seven, and the published
/// Sarvartha Siddhi table names Wednesday's Rohini, which the recording
/// leaves clear. Tripushkar, the one yoga with a rule rather than a table,
/// agrees on all fifty-three.
const KNOWN_YOGAS: [(&str, &str, &str); 12] = [
    ("c002", "sarvartha_siddhi", "Budhavara, Rohini"),
    ("c005", "siddha", "Somavara, Pushya"),
    ("c015", "sarvartha_siddhi", "Guruvara, Magha"),
    ("c018", "sarvartha_siddhi", "Shukravara, UttaraBhadrapada"),
    ("c019", "sarvartha_siddhi", "Mangalavara, Dhanishtha"),
    ("c020", "sarvartha_siddhi", "Shanivara, PurvaBhadrapada"),
    ("c039", "siddha", "Guruvara, Anuradha"),
    ("c040", "siddha", "Ravivara, Mula"),
    ("c044", "siddha", "Shukravara, Mula"),
    ("c051", "siddha", "Somavara, Mula"),
    ("c052", "siddha", "Somavara, Mula"),
    ("c055", "siddha", "Mangalavara, Revati"),
];

/// The five muhurta yogas the corpus flags, against the yogas the SDK's day
/// finds: a recorded flag is a yoga holding at sunrise, which is every yoga
/// the SDK finds, since each is cut from the sunrise nakshatra's span. The
/// flags agree but for the table the recording reads ([`KNOWN_YOGAS`]).
#[test]
fn the_muhurta_yogas_are_the_recorded_flags_but_for_the_tables() {
    let mut agree = 0;
    let mut differ: Vec<(String, &str, String)> = Vec::new();
    for (chart, days) in &days(0) {
        let day = &days[0];
        let flags = &chart["panchanga_day"]["muhurta_yogas"];
        for (key, yoga) in YOGAS {
            let recorded = flags[key].as_bool().unwrap();
            let ours = day.omens.yogas.iter().any(|found| found.yoga == yoga);
            if ours == recorded {
                agree += 1;
            } else {
                println!(
                    "{} {key}: recorded {recorded}, ours {ours} ({:?}, {:?}): {OTHER_TABLE}",
                    id(chart),
                    day.day.vara,
                    day.limbs.nakshatra.first().map(|span| span.member)
                );
                let at_sunrise = day.limbs.nakshatra.first().map(|span| span.member);
                differ.push((
                    id(chart).to_owned(),
                    key,
                    format!("{:?}, {:?}", day.day.vara, at_sunrise.unwrap()),
                ));
            }
        }
    }
    println!("{agree} muhurta yoga flags agree; {} differ", differ.len());
    assert_eq!(agree + differ.len(), 53 * YOGAS.len());
    let known: Vec<(String, &str, String)> = KNOWN_YOGAS
        .iter()
        .map(|(id, key, day)| ((*id).to_owned(), *key, (*day).to_owned()))
        .collect();
    assert_eq!(differ, known, "the flags that part are the explained ones");
}

//! The shape of a batch of almanacs, written as JSON for the pass that
//! measures it (`cargo xtask almanac`).
//!
//! A panchanga blob has a problem the chart blob did not: **its lists are
//! ragged**. A chart holds one lagna, twelve bhavas and a graha count
//! fixed by its kind, so every per-chart section has a stride the blob
//! can state once. A day holds two tithis or three, sixteen choghadiya or
//! none, and a Moon that may rise twice or not at all — so a section's
//! stride is not a property of the batch, and a layout that assumes one
//! is either wrong or wasteful.
//!
//! This writes what a real batch of days actually holds, so the design is
//! measured rather than supposed. It emits counts and not documents: the
//! measurement's subject is how many rows each list has, and a run of
//! whole almanacs would be megabytes of numbers nothing reads.
//!
//! Two places, because latitude is what makes a day ragged. Kathmandu has
//! every arc every day; Tromsø at 69.65°N has weeks with no sunrise and
//! weeks with no night, which is the case `panchanga-day.md` §15 says is
//! reported as **absence** rather than as a period of no length.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    reason = "an example fails by panicking and reports what it measured"
)]

use std::collections::BTreeMap;

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::solar::drik::DrikSun;
use teistro_calendar::{CalendarDate, CalendarSystem, Gregorian};
use teistro_core::catalogue::{Ayanamsha, Calendar};
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::settings::{
    DEFAULT_PROFILE, OverridePolicy, PolarDayPolicy, Profile, SettingsPatch, Sunrise,
};
use teistro_core::time::UtcOffset;
use teistro_panchanga::{Almanac, Panchanga};
use teistro_port_ephemeris::test_provider::TestProvider;
use teistro_time::local_day::DayState;

/// One place a batch is measured at.
struct Site {
    /// What the page calls it.
    name: &'static str,
    /// Where.
    place: Place,
    /// The clock the day's date is read in, seconds east of UTC.
    offset_seconds: i32,
    /// The ranges to measure, each `(label, year, month, day, days)`.
    ranges: &'static [(&'static str, i32, u8, u8, usize)],
    /// Whether the site needs a polar-day policy.
    ///
    /// The default profile leaves `day.polar_day_policy` at `UNDEFINED`,
    /// which refuses a day with no sunrise outright rather than
    /// synthesising bounds nobody asked for — `panchanga-day.md` §15's
    /// first row, met in practice. A place inside the Arctic Circle
    /// therefore has to say which policy it wants before it can be
    /// measured at all, and that is a finding rather than a nuisance.
    polar: bool,
}

fn main() {
    let provider = TestProvider;
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );

    let sites = [
        Site {
            name: "kathmandu",
            place: Place::new(
                Latitude::literal(27.7172),
                Longitude::literal(85.3240),
                Altitude::literal(1400.0),
            ),
            offset_seconds: 5 * 3600 + 45 * 60,
            // A whole year, so every season and every lunar month is in
            // it, and the maximum of every list is a real maximum.
            ranges: &[("a year", 2024, 1, 1, 366)],
            polar: false,
        },
        Site {
            // High enough for a four-hour winter day and a twenty-hour
            // summer one, and low enough that the Sun still rises: the
            // arcs are extreme rather than absent.
            name: "reykjavik",
            place: Place::new(
                Latitude::literal(64.1466),
                Longitude::literal(-21.9426),
                Altitude::literal(50.0),
            ),
            offset_seconds: 0,
            ranges: &[
                ("midsummer", 2024, 6, 14, 21),
                ("midwinter", 2024, 12, 1, 21),
            ],
            polar: true,
        },
        Site {
            name: "tromso",
            place: Place::new(
                Latitude::literal(69.6492),
                Longitude::literal(18.9553),
                Altitude::literal(10.0),
            ),
            offset_seconds: 3600,
            // The two solstices, where the Sun does not set and does not
            // rise: the polar case, which is the one that empties a list.
            ranges: &[
                ("midsummer", 2024, 6, 14, 21),
                ("midwinter", 2024, 12, 1, 21),
            ],
            polar: true,
        },
    ];

    let mut days = Vec::new();
    let mut refusals = Vec::new();
    for site in &sites {
        let mut patch = SettingsPatch::default();
        if site.polar {
            patch.day.polar_day_policy = Some(PolarDayPolicy::NearestEvent);
        }
        let resolved = Profile::shipped(DEFAULT_PROFILE)
            .expect("the default profile")
            .resolve(&patch)
            .expect("it resolves");
        let clock = UtcOffset::try_from_seconds(site.offset_seconds).expect("a clock in range");
        let almanac = Almanac::new(
            &provider,
            &resolved,
            &model,
            &Gregorian,
            &clock,
            PrecessionModel::default(),
            DeltaTModel::TableThenModel,
        );
        for (label, year, month, day, count) in site.ranges {
            let from = CalendarDate::defined(Calendar::Gregorian, *year, *month, *day);
            let last = Gregorian
                .fixed_of(&from)
                .expect("a date the calendar has")
                .plus_days(i64::try_from(*count).unwrap_or(1) - 1);
            let to = Gregorian.date_of(last).expect("a date the calendar has");
            // A refused range is a result, not a crash: what a place
            // and a policy cannot produce is as much of a measurement as
            // what they can, and the page reports both.
            match almanac.between(&from, &to, &site.place) {
                Ok(batch) => {
                    for panchanga in &batch.value {
                        days.push(shape(site.name, label, panchanga));
                    }
                }
                Err(error) => refusals.push(serde_json::json!({
                    "site": site.name,
                    "range": label,
                    "status": format!("{:?}", error.status),
                    "message": error.to_string(),
                })),
            }
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "days": days,
            "refusals": refusals,
        }))
        .expect("the shapes serialise")
    );
}

/// One day's shape: how many rows each of its lists would put in a blob.
fn shape(site: &str, range: &str, day: &Panchanga) -> serde_json::Value {
    let lists: BTreeMap<&str, usize> = BTreeMap::from([
        ("limbs.tithi", day.limbs.tithi.len()),
        ("limbs.nakshatra", day.limbs.nakshatra.len()),
        ("limbs.yoga", day.limbs.yoga.len()),
        ("limbs.karana", day.limbs.karana.len()),
        ("kaalas", day.kaalas.len()),
        ("choghadiya", day.choghadiya.len()),
        ("horas", day.horas.len()),
        ("muhurtas.daylight", day.muhurtas.daylight.len()),
        ("muhurtas.night", day.muhurtas.night.len()),
        ("moon.rises", day.moon.rises.len()),
        ("moon.sets", day.moon.sets.len()),
        ("moon.signs", day.moon.signs.len()),
        ("sun.signs", day.sun.signs.len()),
        ("omens.panchaka", day.omens.panchaka.len()),
        ("omens.yogas", day.omens.yogas.len()),
    ]);
    let present: BTreeMap<&str, bool> = BTreeMap::from([
        ("muhurtas.abhijit", day.muhurtas.abhijit.is_some()),
        ("muhurtas.brahma", day.muhurtas.brahma.is_some()),
        ("sun.sankranti", day.sun.sankranti.is_some()),
    ]);
    serde_json::json!({
        "site": site,
        "range": range,
        "polar": !matches!(day.day.state, DayState::Normal),
        "lists": lists,
        "present": present,
    })
}

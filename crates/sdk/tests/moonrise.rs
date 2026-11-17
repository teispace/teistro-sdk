//! The moonrise convention (SR-102, SR-113,
//! `docs/03-design/moonrise-convention.md`): the Moon rises by
//! `day.moonrise`, the almanacs' upper limb with standard refraction by
//! default, whatever the day's sunrise convention; and the one horizon
//! model, `day.horizon`, lowers both bodies' horizons by its dip.
//!
//! The acceptance values are a published almanac's: timeanddate.com's
//! moonrise and moonset table for Kathmandu, January 2026 (rank 3), which
//! **truncates** to the minute: every one of its 62 events that month is
//! the DE440 moon oracle's (`oracle/moonrises.json`) with the seconds
//! dropped. The second-level check is `limb_oracle.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, Panchanga, UtcOffset};

/// Seconds in a day.
const SECONDS: f64 = 86_400.0;

/// Nepal's clock, +05:45, as seconds.
const NEPAL: i32 = 20_700;

/// timeanddate.com, "Moonrise, Moonset, and Moon Phase in Kathmandu,
/// Nepal", January 2026: (day, hour, minute) on Nepal's clock, rises.
const PRINTED_RISES: [(u8, u8, u8); 8] = [
    (1, 14, 58),
    (5, 19, 32),
    (9, 23, 29),
    (13, 2, 12),
    (17, 5, 51),
    (22, 9, 3),
    (27, 11, 56),
    (31, 16, 0),
];

/// The same table's sets.
const PRINTED_SETS: [(u8, u8, u8); 4] = [(5, 8, 34), (13, 12, 53), (22, 20, 59), (30, 4, 32)];

fn kathmandu() -> Place {
    Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    )
}

/// January 2026 at Kathmandu under `profile` and a settings patch.
fn january(profile: &str, patch: &str) -> Vec<Panchanga> {
    let sdk = Context::builder()
        .profile(profile)
        .ephemeris([Ephemeris::Builtin])
        .settings_json(patch)
        .build()
        .unwrap();
    sdk.almanac()
        .of(
            &CalendarDate::defined(Calendar::Gregorian, 2026, 1, 1),
            &CalendarDate::defined(Calendar::Gregorian, 2026, 1, 31),
            &kathmandu(),
            UtcOffset::try_from_seconds(NEPAL).unwrap(),
        )
        .unwrap()
        .value
}

/// A printed minute on Nepal's clock as a UTC Julian day.
fn printed((day, hour, minute): (u8, u8, u8)) -> f64 {
    // 0h UT of 1 January 2026 is JD 2461041.5.
    2_461_041.5 + f64::from(day - 1) + (f64::from(hour) * 60.0 + f64::from(minute)) / 1_440.0
        - f64::from(NEPAL) / SECONDS
}

/// Each printed event's offset, seconds, from the nearest of `ours`.
fn after_print(printed_events: &[(u8, u8, u8)], ours: &[f64]) -> Vec<f64> {
    printed_events
        .iter()
        .map(|event| {
            let at = printed(*event);
            ours.iter()
                .map(|jd| (jd - at) * SECONDS)
                .min_by(|a, b| a.abs().total_cmp(&b.abs()))
                .unwrap()
        })
        .collect()
}

fn rises(days: &[Panchanga]) -> Vec<f64> {
    days.iter()
        .flat_map(|day| day.moon.rises.iter().map(|at| at.get()))
        .collect()
}

fn sets(days: &[Panchanga]) -> Vec<f64> {
    days.iter()
        .flat_map(|day| day.moon.sets.iter().map(|at| at.get()))
        .collect()
}

#[test]
fn nepals_moon_rises_by_the_upper_limb_as_the_almanac_prints_it() {
    // Nepal's drik profile, whose day opens at the disc's centre, and the
    // root's drik ones.
    for profile in ["nepali-drik", "parashari-classical", "kp-default"] {
        let days = january(profile, "{}");
        // A truncated minute: the event is within the printed minute.
        for (what, printed_events, ours) in [
            ("rise", &PRINTED_RISES[..], rises(&days)),
            ("set", &PRINTED_SETS[..], sets(&days)),
        ] {
            for (event, after) in printed_events
                .iter()
                .zip(after_print(printed_events, &ours))
            {
                assert!(
                    (-1.0..61.0).contains(&after),
                    "{profile} {what} {event:?}: {after:.1} s after the printed minute"
                );
            }
        }
    }
    // The committee's settings name the same convention; its sky needs
    // the text's provider, whose Moon is not the modern one printed here.
    let committee = teistro::settings::Profile::shipped("nepali-committee")
        .unwrap()
        .resolve(&teistro::settings::SettingsPatch::default())
        .unwrap();
    assert_eq!(
        committee.settings.day.moonrise,
        teistro::settings::Sunrise::UpperLimbRefraction.into()
    );
}

#[test]
fn the_moon_no_longer_follows_the_days_sunrise() {
    // Before SR-102 the Moon rose by `day.sunrise`, which Nepal's
    // profiles set to the disc's centre with refraction: some 80 s late,
    // so the printed minute is missed on most days.
    let own = january("nepali-drik", "{}");
    let centre = january(
        "nepali-drik",
        r#"{"day":{"moonrise":{"kind":"NAMED","which":"CENTRE_REFRACTION"}}}"#,
    );
    let late: Vec<f64> = after_print(&PRINTED_RISES, &rises(&centre));
    assert!(
        late.iter().filter(|after| **after >= 61.0).count() >= PRINTED_RISES.len() / 2,
        "{late:?}"
    );
    // The days themselves are the same: a moonrise convention moves no
    // sunrise.
    for (a, b) in own.iter().zip(&centre) {
        assert_eq!(a.day.sunrise, b.day.sunrise);
    }
    for (a, b) in rises(&own).iter().zip(rises(&centre)) {
        let later = (b - a) * SECONDS;
        assert!((55.0..110.0).contains(&later), "{later} s");
    }
}

#[test]
fn the_horizon_models_dip_lowers_the_sun_and_the_moon_alike() {
    // SR-113: the dip is `day.horizon`'s, one model for every horizon a
    // context reads. Seen from Kathmandu's 1400 m through the air near the
    // ground the horizon lies some 1.05 degrees down (Thom's coefficient,
    // as the Swiss Ephemeris reckons it), which the January Sun, climbing
    // 0.2 degrees a minute, clears 306 to 320 s sooner over the month; the
    // Moon keeps her own convention and is lowered by the same dip, 295 to
    // 359 s sooner by her declination and motion.
    let level = january("nepali-drik", "{}");
    let dipped = january("nepali-drik", r#"{"day":{"horizon":{"dip":"REFRACTED"}}}"#);
    for (l, d) in level.iter().zip(&dipped) {
        let sooner = (l.day.sunrise.get() - d.day.sunrise.get()) * SECONDS;
        assert!((280.0..350.0).contains(&sooner), "{sooner} s");
    }
    // Each level moonrise beside the dipped one nearest it: a rise near
    // a sunrise may fall in the day before once the dip moves both.
    let dipped = rises(&dipped);
    for l in rises(&level) {
        let sooner = dipped
            .iter()
            .map(|d| (l - d) * SECONDS)
            .min_by(|a, b| a.abs().total_cmp(&b.abs()))
            .unwrap();
        assert!((240.0..420.0).contains(&sooner), "{sooner} s");
    }
}

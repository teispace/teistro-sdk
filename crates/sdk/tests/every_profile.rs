//! Every shipped profile through every almanac call, at places that test
//! the edges: Kathmandu, Chennai in the tropics, Sydney south of the
//! equator, Honolulu far west, and Tromsø above the Arctic circle in its
//! polar night. A call may answer or refuse, but a refusal must be one a
//! caller can act on: never `INTERNAL`, never a panic, always a message.
//! Each profile must answer the almanac's day at Kathmandu, so a profile
//! that opens and refuses everything cannot pass as "refused by name".
//!
//! The attempts are counted from the lists, not from what answered, so a
//! skipped combination shows in the count (`count-attempts-not-reports`).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "a test fails by panicking and names the combination at fault"
)]

use teistro::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation};
use teistro::catalogue::Calendar;
use teistro::first_year::FirstYear;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::settings::SHIPPED_PROFILES;
use teistro::{
    AyanaReckoning, CalendarDate, Context, Ephemeris, Error, FestivalPack, FestivalRequest, Status,
    UtcOffset,
};

/// A place, its clock and its name.
struct Site {
    name: &'static str,
    place: Place,
    offset: UtcOffset,
}

fn site(name: &'static str, lat: f64, lon: f64, alt: f64, offset: UtcOffset) -> Site {
    Site {
        name,
        place: Place::new(
            Latitude::literal(lat),
            Longitude::literal(lon),
            Altitude::literal(alt),
        ),
        offset,
    }
}

fn sites() -> [Site; 5] {
    [
        site(
            "Kathmandu",
            27.7172,
            85.324,
            1337.0,
            UtcOffset::literal(5, 45, 0),
        ),
        site(
            "Chennai",
            13.0827,
            80.2707,
            6.0,
            UtcOffset::literal(5, 30, 0),
        ),
        site(
            "Sydney",
            -33.8688,
            151.2093,
            58.0,
            UtcOffset::literal(10, 0, 0),
        ),
        site(
            "Honolulu",
            21.3069,
            -157.8583,
            5.0,
            UtcOffset::literal(-10, 0, 0),
        ),
        site(
            "Tromso",
            69.6492,
            18.9553,
            10.0,
            UtcOffset::literal(1, 0, 0),
        ),
    ]
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// 2026-12-20 06:00 UTC: inside Tromsø's polar night.
fn instant() -> JulianDay<Utc> {
    JulianDay::literal(2_461_394.75)
}

fn context(profile: &str) -> Context {
    Context::builder()
        .profile(profile)
        .ephemeris([Ephemeris::SuryaSiddhanta, Ephemeris::Builtin])
        .build()
        .unwrap_or_else(|error| panic!("{profile} does not open: {error}"))
}

/// An answer; or, under the polar night alone, a refusal saying the Sun
/// did not rise (`UNSUPPORTED`, the profile's polar-day policy undefined)
/// or that a night without a sunset cannot be divided (`OUT_OF_RANGE`).
fn acted_on<T>(what: &str, polar: bool, found: Result<T, Error>) -> bool {
    match found {
        Ok(_) => true,
        Err(error) => {
            assert!(polar, "{what}: {error}");
            assert!(
                matches!(error.status, Status::Unsupported | Status::OutOfRange),
                "{what}: {error}"
            );
            assert!(
                error.message.contains("polar") || error.message.contains("neither rose nor set"),
                "{what}: {error}"
            );
            false
        }
    }
}

#[test]
fn every_profile_answers_every_almanac_call_or_refuses_by_name() {
    let (from, to) = (date(2026, 12, 15), date(2026, 12, 21));
    let mut attempts = 0;
    let mut answered = 0;
    for profile in SHIPPED_PROFILES {
        let sdk = context(profile);
        let almanac = sdk.almanac();
        let kathmandu = &sites()[0];
        almanac
            .day(&from, &kathmandu.place, kathmandu.offset)
            .unwrap_or_else(|error| panic!("{profile}: the day at Kathmandu: {error}"));
        for site in sites() {
            let what = |call: &str| format!("{profile} at {}: {call}", site.name);
            let polar = site.name == "Tromso";
            let acted_on =
                |call: &str, found: Result<(), Error>| acted_on(&what(call), polar, found);
            let (place, offset) = (&site.place, site.offset);
            let death = AshauchRequest {
                key: "k".to_owned(),
                event: AshauchEvent::Death {
                    at: instant(),
                    cremated_at: None,
                    ahitagni: false,
                    deceased: Deceased::default(),
                },
                relation: Relation::Sapinda,
                heard_at: None,
                abroad: false,
                nearness: Nearness::SameVillage,
                full_days: None,
                during: Vec::new(),
                fire_giver: false,
            };
            let calls = [
                acted_on("day", almanac.day(&from, place, offset).map(drop)),
                acted_on("of", almanac.of(&from, &to, place, offset).map(drop)),
                acted_on("years", almanac.years(&from, &to, place, offset).map(drop)),
                acted_on(
                    "sankalpa",
                    almanac
                        .sankalpa(instant(), place, offset, AyanaReckoning::Sidereal)
                        .map(drop),
                ),
                acted_on(
                    "ashauch",
                    almanac
                        .ashauch(&AshauchTable::dharmasindhu(), &death, place, offset)
                        .map(drop),
                ),
                acted_on(
                    "ashauch, Mitakshara",
                    almanac
                        .ashauch(&AshauchTable::mitakshara(), &death, place, offset)
                        .map(drop),
                ),
                acted_on(
                    "first year",
                    almanac
                        .first_year(&FirstYear::nepal("k", instant()), place, offset)
                        .map(drop),
                ),
            ];
            attempts += calls.len();
            answered += calls.iter().filter(|ok| **ok).count();
            for pack in FestivalPack::ALL {
                attempts += 1;
                let found = almanac.festivals(
                    &date(2026, 12, 1),
                    &date(2026, 12, 31),
                    place,
                    offset,
                    &FestivalRequest::from(pack),
                );
                answered += usize::from(acted_on(pack.key(), found.map(drop)));
            }
        }
    }
    let expected = SHIPPED_PROFILES.len() * sites().len() * (7 + FestivalPack::ALL.len());
    assert_eq!(attempts, expected, "every combination was attempted");
    // Every combination away from the polar night answered; a refusal there
    // is the rule a profile states, not a failure.
    let polar = SHIPPED_PROFILES.len() * (7 + FestivalPack::ALL.len());
    assert!(
        answered >= attempts - polar,
        "{answered} of {attempts} answered"
    );
}

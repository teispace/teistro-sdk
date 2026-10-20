//! **A year of festivals is the almanac's year, read lean.** The festival
//! call founds only what its rules read of each day (`festival_days`);
//! this holds that answer to the one founded from the full panchanga
//! days (`festivals_with_days`), value and stamp, for every year from
//! 2000 to 2100 at Kathmandu, Delhi and Sydney, under both setups over
//! Teimeris, with every shipped pack's rules at once.
//!
//! It prints one line per place and setup, and fails on the first year
//! the two answers differ, naming it.
//!
//! ```text
//! TM_EMBED_TIER=MODERN cargo run --release --features app-speed \
//!   --bin teistro-ephemeris-teimeris-festival-agreement
//! ```

#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    clippy::panic,
    reason = "a tooling binary prints its answer and stops on a refused request"
)]

use std::time::{Duration, Instant};

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};
use teistro_ephemeris_teimeris::TeimerisProvider;

/// A setup's ephemeris chain, built fresh for its context.
type Chain = fn() -> Vec<Ephemeris>;

/// The first year and the last.
const YEARS: (i32, i32) = (2000, 2100);

fn teimeris() -> Ephemeris {
    Ephemeris::Provider(Box::new(
        TeimerisProvider::embedded().expect("Teimeris built with TM_EMBED_TIER=MODERN"),
    ))
}

fn place(latitude: f64, longitude: f64, altitude: f64) -> Place {
    Place::new(
        Latitude::try_new(latitude).expect("a latitude"),
        Longitude::try_new(longitude).expect("a longitude"),
        Altitude::try_new(altitude).expect("an altitude"),
    )
}

fn main() {
    let places = [
        (
            "kathmandu",
            place(27.7172, 85.324, 1400.0),
            UtcOffset::literal(5, 45, 0),
        ),
        (
            "delhi",
            place(28.6139, 77.209, 216.0),
            UtcOffset::literal(5, 30, 0),
        ),
        (
            "sydney",
            place(-33.8688, 151.2093, 58.0),
            UtcOffset::literal(10, 0, 0),
        ),
    ];
    let request = FestivalPack::ALL
        .into_iter()
        .fold(FestivalRequest::new(Vec::new()), FestivalRequest::with_pack);
    let setups: [(&str, &str, Chain); 2] = [
        ("drik-teimeris", "parashari-classical", || vec![teimeris()]),
        ("committee-teimeris", "nepali-committee", || {
            vec![Ephemeris::SuryaSiddhanta, teimeris()]
        }),
    ];
    for (key, profile, chain) in setups {
        let context = Context::builder()
            .profile(profile)
            .ephemeris(chain())
            .build()
            .expect("the setup opens");
        for (name, place, offset) in &places {
            let (mut lean, mut full) = (Duration::ZERO, Duration::ZERO);
            let mut observances = 0;
            for year in YEARS.0..=YEARS.1 {
                let from = CalendarDate::defined(Calendar::Gregorian, year, 1, 1);
                let to = CalendarDate::defined(Calendar::Gregorian, year, 12, 31);
                let started = Instant::now();
                let found = context
                    .almanac()
                    .festivals(&from, &to, place, *offset, &request)
                    .expect("a year of festivals");
                lean += started.elapsed();
                let started = Instant::now();
                let founded = context
                    .almanac()
                    .festivals_with_days(&from, &to, place, *offset, &request)
                    .expect("a year of festivals with its days");
                full += started.elapsed();
                assert!(
                    found == founded.answer,
                    "{key} at {name}: {year}'s festivals differ from the ones its full days give"
                );
                observances += found.value.observances.len() + found.value.ekadashis.len();
            }
            println!(
                "{key} {name}: {}..={} agree, {observances} observances; {:.1} ms a year lean, {:.1} full",
                YEARS.0,
                YEARS.1,
                lean.as_secs_f64() * 1000.0 / f64::from(YEARS.1 - YEARS.0 + 1),
                full.as_secs_f64() * 1000.0 / f64::from(YEARS.1 - YEARS.0 + 1),
            );
        }
    }
}

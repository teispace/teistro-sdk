//! **The answer a phone must give**: the content hash of one month's
//! almanac at Kathmandu under each setup over Teimeris, opened on its
//! embedded data, the way the Flutter package opens it on a phone.
//!
//! A Flutter app built on `teistro_flutter` and
//! `teistro_ephemeris_teimeris_flutter` asks for the same month and prints
//! the same lines (`adapters/ephemeris-teimeris/flutter/example`); the two
//! printouts are equal byte for byte when the phone's answer is the
//! desktop's.
//!
//! ```text
//! TM_EMBED_TIER=MODERN cargo run --release --features app-speed \
//!   --bin teistro-ephemeris-teimeris-day-hash
//! ```

#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    reason = "a tooling binary prints its answer and stops on a refused request"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, UtcOffset};
use teistro_ephemeris_teimeris::TeimerisProvider;

fn teimeris() -> Ephemeris {
    Ephemeris::Provider(Box::new(
        TeimerisProvider::embedded().expect("Teimeris built with TM_EMBED_TIER=MODERN"),
    ))
}

fn main() {
    let place = Place::new(
        Latitude::try_new(27.7172).expect("a latitude"),
        Longitude::try_new(85.324).expect("a longitude"),
        Altitude::try_new(1400.0).expect("an altitude"),
    );
    let from = CalendarDate::defined(Calendar::Gregorian, 2025, 11, 1);
    let to = CalendarDate::defined(Calendar::Gregorian, 2025, 11, 30);
    let setups: [(&str, &str, Vec<Ephemeris>); 2] = [
        ("drik-teimeris", "parashari-classical", vec![teimeris()]),
        (
            "committee-teimeris",
            "nepali-committee",
            vec![Ephemeris::SuryaSiddhanta, teimeris()],
        ),
    ];
    for (key, profile, chain) in setups {
        let context = Context::builder()
            .profile(profile)
            .ephemeris(chain)
            .build()
            .expect("the setup opens");
        let month = context
            .almanac()
            .of(&from, &to, &place, UtcOffset::literal(5, 45, 0))
            .expect("the month");
        println!(
            "{key} settings {} content {}",
            month.provenance.settings_hash, month.provenance.content_hash
        );
    }
}

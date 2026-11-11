//! Sreekrishna Jayanthi in `INDIA_KERALA` against the six notifications,
//! 2021 to 2026, at Thiruvananthapuram under the modern sky: the dark 8th
//! by *Dharmasindhu*'s midnight rule, the one in Chingam (C1320,
//! `03-design/kerala-holidays.md`). 2025 is the year the month decides:
//! Shravana's 8th fell in Karkidakam, before Chingam 1 on 17 August.

#![cfg(feature = "builtin-ephemeris")]
#![allow(
    clippy::unwrap_used,
    missing_docs,
    reason = "a test fails by panicking"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalRequest, UtcOffset};

/// Each notification's Sreekrishna Jayanthi, as (year, month, day).
const PRINTED: [(i32, u8, u8); 6] = [
    (2021, 8, 30),
    (2022, 8, 18),
    (2023, 9, 6),
    (2024, 8, 26),
    (2025, 9, 14),
    (2026, 9, 4),
];

#[test]
fn sreekrishna_jayanthi_is_the_dark_eighth_in_chingam_every_year_printed() {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let thiruvananthapuram = Place::new(
        Latitude::literal(8.5241),
        Longitude::literal(76.9366),
        Altitude::literal(0.0),
    );
    // The state by its ISO 3166-2 code names its pack (SR-89).
    let request = FestivalRequest::from_json(r#"{"rules": ["IN-KL"]}"#).unwrap();
    for (year, month, day) in PRINTED {
        let answer = sdk
            .almanac()
            .festivals(
                &CalendarDate::defined(Calendar::Gregorian, year, 1, 1),
                &CalendarDate::defined(Calendar::Gregorian, year, 12, 31),
                &thiruvananthapuram,
                UtcOffset::literal(5, 30, 0),
                &request,
            )
            .unwrap()
            .value;
        let kept: Vec<(u8, u8)> = answer
            .observances
            .iter()
            .filter(|one| one.rule == "JANMASHTAMI")
            .map(|one| (one.day.month, one.day.day))
            .collect();
        assert_eq!(kept, [(month, day)], "{year}");
    }
}

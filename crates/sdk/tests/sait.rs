//! Festival saits (`03-design/festival-sait.md`): one printed time on each
//! sky. Every printed sait is held by `festival-measured.md` §8.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    missing_docs,
    reason = "a test fails by panicking and indexes what it built"
)]

use teistro::catalogue::{Calendar, Rashi};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

/// Ghatasthapana 2025 at Kathmandu: the committee printed 09:13, a Pisces
/// navamsa in a Libra lagna inside the first ten ghatis, after an earlier
/// dual navamsa with the Sun in the lagna.
#[test]
fn ghatasthapana_2025_is_elected_where_the_committee_printed_it() {
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let day = CalendarDate::defined(Calendar::Gregorian, 2025, 9, 22);
    for (profile, ephemeris) in [
        ("nepali-committee", Ephemeris::SuryaSiddhanta),
        ("nepali-default", Ephemeris::Builtin),
    ] {
        let sdk = Context::builder()
            .profile(profile)
            .ephemeris([ephemeris])
            .build()
            .unwrap();
        let found = sdk
            .almanac()
            .festivals(
                &day,
                &day,
                &kathmandu,
                UtcOffset::literal(5, 45, 0),
                &FestivalRequest::from(FestivalPack::Nepal),
            )
            .unwrap()
            .value;
        let observance = found
            .observances
            .iter()
            .find(|o| o.rule == "NAVARATRA_ARAMBHA")
            .unwrap();
        let sait = observance.sait.as_ref().unwrap();
        let chosen = &sait.candidates[sait.chosen.unwrap()];
        assert_eq!(
            (chosen.lagna, chosen.navamsa, chosen.whole, sait.window),
            (Some(Rashi::Libra), Some(Rashi::Pisces), true, 0),
            "{profile}"
        );
        let minutes = (sait.at.unwrap().get() + 0.5 + 5.75 / 24.0).rem_euclid(1.0) * 1440.0;
        assert!(
            (minutes - f64::from(9 * 60 + 13)).abs() < 2.0,
            "{profile}: {minutes}"
        );
        // A rule the pack gives no sait carries none.
        assert!(
            found
                .observances
                .iter()
                .filter(|o| o.rule != "NAVARATRA_ARAMBHA")
                .all(|o| o.sait.is_none())
        );
    }
}

//! The `tamil-vakya` profile through the almanac: the vakya sky's tithi
//! ends at Chennai are the *Vakyakarana*'s to the minute, and Tamil
//! Nadu's Vinayakar Chathurthi of 2023 falls where G.O. (Ms.) No. 528 of
//! 31 August 2023 moved it, 18 September, where the modern sky gives the
//! 19th (`docs/calendars/vakya.md`, C1296).

#![cfg(feature = "builtin-ephemeris")]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    missing_docs,
    reason = "a test fails by panicking and indexes what it built"
)]

use teistro::catalogue::{Calendar, Tithi};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};
use teistro_siddhanta::vakya;

fn chennai() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(13.0827),
            Longitude::literal(80.2707),
            Altitude::literal(0.0),
        ),
        UtcOffset::literal(5, 30, 0),
    )
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

fn vakya_sky() -> Context {
    Context::builder()
        .profile("tamil-vakya")
        .ephemeris([Ephemeris::SuryaSiddhanta, Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn modern_sky() -> Context {
    Context::builder()
        .profile("parashari-classical")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

/// The days the answer gives `key` between two September 2023 dates.
fn ganesha(context: &Context) -> Vec<(i32, u8, u8)> {
    let (place, offset) = chennai();
    let request = FestivalRequest::from(FestivalPack::IndiaMaharashtra);
    let answer = context
        .almanac()
        .festivals(
            &date(2023, 9, 10),
            &date(2023, 9, 25),
            &place,
            offset,
            &request,
        )
        .unwrap()
        .value;
    answer
        .observances
        .iter()
        .filter(|o| o.rule.as_str() == "GANESHA_CHATURTHI")
        .map(|o| (o.day.year, o.day.month, o.day.day))
        .collect()
}

#[test]
fn vinayakar_chathurthi_2023_is_the_day_tamil_nadu_declared() {
    assert_eq!(ganesha(&vakya_sky()), [(2023, 9, 18)]);
    assert_eq!(ganesha(&modern_sky()), [(2023, 9, 19)]);
}

#[test]
fn the_almanacs_tritiya_ends_where_the_vakyakarana_puts_it() {
    // 18 September 2023, Kali day 1,871,741: the tritiya ends 13 nadis
    // 42 vinadis after sunrise (24 minutes a nadi).
    let (place, offset) = chennai();
    let day = vakya_sky()
        .almanac()
        .day(&date(2023, 9, 18), &place, offset)
        .unwrap()
        .value;
    let tritiya = day
        .limbs
        .tithi
        .iter()
        .find(|span| span.member == Tithi::ShuklaTritiya)
        .unwrap();
    let length = day.day.next_sunrise.get() - day.day.sunrise.get();
    let found = (tritiya.whole.to.get() - day.day.sunrise.get()) / length * 3_600.0;
    let end = vakya::tithi_ends(1_871_741)[0];
    let printed = (end.at.parts() - 1_871_740 * vakya::PARTS_PER_DAY) as f64 / 4.0;
    assert!(
        (found - printed).abs() < 1.0,
        "{found} against {printed} vinadis"
    );
}

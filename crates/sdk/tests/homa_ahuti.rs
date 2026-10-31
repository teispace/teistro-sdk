//! The homa ahuti through the façade (crux C751): each day asked for it
//! carries, for every stretch the Moon's and the Sun's nakshatras both
//! held, the graha the count of the one from the other gives, and a day
//! not asked carries none and hashes as before.
//!
//! The Sun's star is read back from a chart at each share's middle, not
//! from the almanac's own search, so a share placed in the wrong stretch
//! or counted the wrong way fails here. The Moon's is the day's own
//! nakshatra, which the count is of.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, Graha, Nakshatra};
use teistro::omen::ahuti;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{AlmanacRequest, CalendarDate, ChartRequest, Context, Ephemeris, UtcOffset};

/// A nakshatra's span, degrees.
const STAR: f64 = 360.0 / 27.0;

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn context() -> Context {
    Context::builder()
        .profile("nepali-drik")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

/// The star a sidereal longitude falls in, or none within `margin`
/// degrees of a boundary, where a chart's nutated ayanamsha and the
/// almanac's own reading may place it either side.
fn star(longitude: f64, margin: f64) -> Option<Nakshatra> {
    let into = longitude.rem_euclid(360.0) / STAR;
    let edge = (into - into.round()).abs() * STAR;
    // The first of the 27 whose end lies past the longitude.
    let id = (1..=27u16).find(|k| f64::from(*k) > into).unwrap_or(27) - 1;
    (edge > margin).then(|| Nakshatra::from_id(id)).flatten()
}

#[test]
fn each_share_is_the_moons_star_counted_from_the_suns() {
    let sdk = context();
    let place = kathmandu();
    let offset = UtcOffset::literal(5, 45, 0);
    let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    let answer = sdk
        .almanac()
        .asked(
            &date(1, 1),
            &date(3, 31),
            &place,
            offset,
            &AlmanacRequest::new().with_homa_ahuti(),
        )
        .unwrap();
    let charts = ChartRequest::at(place, offset);
    let (mut read, mut shares) = (0, 0);
    let mut seen = [false; 9];
    for day in &answer.days.value {
        let homa = &day.homa_ahuti;
        assert!(
            homa.len() >= day.limbs.nakshatra.len(),
            "{:?}: a share for each star the Moon held",
            day.day.date
        );
        // The shares tile the day, in order and without a gap.
        assert_eq!(homa[0].inside.from, day.window.from, "{:?}", day.day.date);
        assert_eq!(homa[homa.len() - 1].inside.to, day.window.to);
        for pair in homa.windows(2) {
            assert_eq!(pair[0].whole.to, pair[1].whole.from, "{:?}", day.day.date);
        }
        for share in homa {
            shares += 1;
            let middle = share.inside.from.get().midpoint(share.inside.to.get());
            let chart = sdk
                .chart()
                .reading(JulianDay::<Utc>::literal(middle), &charts)
                .unwrap()
                .value;
            let sun = chart.foundation.graha(Graha::Sun).unwrap().longitude_deg;
            let Some(sun) = star(sun, 0.02) else {
                continue;
            };
            // The Moon's star is the day's own: a chart may read her from
            // another centre, half a degree apart by parallax.
            let moon = day
                .limbs
                .nakshatra
                .iter()
                .find(|stay| stay.whole.from.get() <= middle && middle < stay.whole.to.get())
                .unwrap()
                .member;
            assert_eq!(
                share.member,
                ahuti(sun, moon),
                "{:?} at {middle}",
                day.day.date
            );
            seen[omen_index(share.member)] = true;
            read += 1;
        }
    }
    // Hardly a share falls within a fiftieth of a degree of a boundary,
    // and a quarter of a year's Moon passes every count from the Sun.
    assert!(read * 50 > shares * 49, "{read} of {shares} read back");
    assert_eq!(seen, [true; 9], "every graha takes a share");
}

/// Where a graha stands in the verse's order.
fn omen_index(graha: Graha) -> usize {
    teistro::omen::HOMA_AHUTI
        .iter()
        .position(|g| *g == graha)
        .unwrap()
}

#[test]
fn a_day_not_asked_carries_none_and_hashes_as_before() {
    let sdk = context();
    let place = kathmandu();
    let offset = UtcOffset::literal(5, 45, 0);
    let date = CalendarDate::defined(Calendar::Gregorian, 2026, 6, 15);
    let plain = sdk
        .almanac()
        .asked(&date, &date, &place, offset, &AlmanacRequest::new())
        .unwrap();
    let asked = sdk
        .almanac()
        .asked(
            &date,
            &date,
            &place,
            offset,
            &AlmanacRequest::new().with_homa_ahuti(),
        )
        .unwrap();
    let (plain_day, asked_day) = (&plain.days.value[0], &asked.days.value[0]);
    assert_eq!(plain_day.homa_ahuti.len(), 0);
    assert_ne!(asked_day.homa_ahuti.len(), 0);
    // The day not asked writes no `homa_ahuti` at all, so its document,
    // and every hash over it, is what it was before the field existed.
    let document = serde_json::to_value(plain_day).unwrap();
    assert!(document.get("homa_ahuti").is_none());
    // Asking adds the shares and changes nothing else.
    let mut stripped = asked_day.clone();
    stripped.homa_ahuti.clear();
    assert_eq!(&stripped, plain_day);
    assert_ne!(plain.day_hashes, asked.day_hashes, "an asked day says so");
}

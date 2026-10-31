//! The year's office-bearers through the façade
//! (`03-design/year-rulers.md`): the rulers printed for VS 2080, 2082 and
//! 2083, each asserted under the rule that printed them, and the readings
//! the other rules give where they part.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, Graha};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::rulers::{RulerDayRule, RulerText, YearOffice, YearRuler};
use teistro::{CalendarDate, Context, Ephemeris, LunarYear, UtcOffset};

fn context() -> Context {
    Context::builder()
        .profile("nepali-drik")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn kathmandu() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        ),
        UtcOffset::literal(5, 45, 0),
    )
}

fn hyderabad() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(17.385),
            Longitude::literal(78.4867),
            Altitude::literal(505.0),
        ),
        UtcOffset::literal(5, 30, 0),
    )
}

/// The one year holding a day.
fn year_of(
    sdk: &Context,
    (place, clock): (Place, UtcOffset),
    (y, m, d): (i32, u8, u8),
) -> LunarYear {
    let date = CalendarDate::defined(Calendar::Gregorian, y, m, d);
    let years = sdk.almanac().years(&date, &date, &place, clock).unwrap();
    assert_eq!(years.value.len(), 1, "a day falls in one year");
    years.value[0].clone()
}

fn office(year: &LunarYear, office: YearOffice) -> &YearRuler {
    year.rulers.iter().find(|r| r.office == office).unwrap()
}

/// An office's lord under one rule.
fn under(year: &LunarYear, which: YearOffice, rule: RulerDayRule) -> Graha {
    office(year, which)
        .readings
        .iter()
        .find(|reading| reading.rule == rule)
        .unwrap()
        .lord
}

#[test]
fn every_office_is_answered_with_its_texts_and_rules() {
    let year = year_of(&context(), kathmandu(), (2026, 8, 1));
    let offices: Vec<YearOffice> = year.rulers.iter().map(|r| r.office).collect();
    assert_eq!(offices, YearOffice::ALL);
    for ruler in &year.rulers {
        assert_eq!(ruler.texts, ruler.office.texts());
        let rules: Vec<RulerDayRule> = ruler.readings.iter().map(|r| r.rule).collect();
        assert_eq!(rules, ruler.office.rules());
        assert!(year.opened.get() <= ruler.event.get() && ruler.event.get() < year.ended.get());
        for reading in &ruler.readings {
            assert_eq!(reading.lord, reading.vara.attributes().lord);
        }
    }
    assert!(
        office(&year, YearOffice::Nirasadhipati)
            .texts
            .contains(&RulerText::NaradaSamhita)
    );
    assert!(
        !office(&year, YearOffice::Nirasadhipati)
            .texts
            .contains(&RulerText::Jyotirnibandha)
    );
}

/// VS 2080, Pingala: Chaitra Shukla Pratipada on Wednesday 22 March 2023
/// and Mesha's ingress on Friday 14 April, so Mercury is raja and Venus
/// mantri, as a patro printed in Nepal gave them (e-Himalaya Times,
/// March 2023, citing *Jyotirnibandha*'s sunrise verse).
#[test]
fn vs_2080_mercury_raja_and_venus_mantri_at_kathmandu() {
    let year = year_of(&context(), kathmandu(), (2023, 8, 1));
    assert_eq!(year.vikrama, 2080);
    assert_eq!(
        under(&year, YearOffice::Raja, RulerDayRule::Sunrise),
        Graha::Mercury
    );
    // The new moon fell on Tuesday evening: Ratnavali's reading is
    // Tuesday's, and Garga's, past midday, the next day's.
    assert_eq!(
        under(&year, YearOffice::Raja, RulerDayRule::NewMoon),
        Graha::Mars
    );
    assert_eq!(
        under(&year, YearOffice::Raja, RulerDayRule::Garga),
        Graha::Mercury
    );
    assert_eq!(office(&year, YearOffice::Mantri).lord(), Some(Graha::Venus));
    assert_eq!(
        under(&year, YearOffice::Mantri, RulerDayRule::Vara),
        Graha::Venus
    );
}

/// VS 2083 (2026–27, Parabhava in the south): the nine a southern
/// panchanga printed for Hyderabad (onlinejyotish.com's Ugadi
/// panchangam): Jupiter raja, Mars mantri, the Moon commander, Venus lord
/// of crops, Mercury of grain, the Moon of prices and of the clouds, the
/// Sun of juices and Venus of dry goods. Karka's ingress (16 July, 23:44)
/// and Tula's (17 October, 19:57) and Makara's (14 January 2027) fell at
/// night and were printed on the next day: the daylight reading.
#[test]
fn vs_2083_the_nine_printed_for_hyderabad() {
    let year = year_of(&context(), hyderabad(), (2026, 8, 1));
    assert_eq!(year.vikrama, 2083);
    let printed = [
        (YearOffice::Raja, Graha::Jupiter),
        (YearOffice::Mantri, Graha::Mars),
        (YearOffice::Senadhipati, Graha::Moon),
        (YearOffice::Sasyadhipati, Graha::Venus),
        (YearOffice::Dhanyadhipati, Graha::Mercury),
        (YearOffice::Arghadhipati, Graha::Moon),
        (YearOffice::Meghadhipati, Graha::Moon),
        (YearOffice::Rasadhipati, Graha::Sun),
        (YearOffice::Nirasadhipati, Graha::Venus),
    ];
    for (which, lord) in printed {
        assert_eq!(office(&year, which).lord(), Some(lord), "{which:?}");
    }
    // The weekday holding each night ingress is the day before.
    assert_eq!(
        under(&year, YearOffice::Sasyadhipati, RulerDayRule::Vara),
        Graha::Jupiter
    );
    assert_eq!(
        under(&year, YearOffice::Rasadhipati, RulerDayRule::Vara),
        Graha::Saturn
    );
    assert_eq!(
        under(&year, YearOffice::Nirasadhipati, RulerDayRule::Vara),
        Graha::Jupiter
    );
}

/// VS 2083's pratipada began on Thursday 19 March 2026 after sunrise and
/// ended before Friday's: it ran at no sunrise, so *Jyotirnibandha* v. 3
/// takes the earlier day, Thursday, and Jupiter is raja, as the press
/// printed in the north and the south, though the lunar year's first
/// sunrise is Friday's.
#[test]
fn a_pratipada_at_no_sunrise_gives_the_earlier_day_its_raja() {
    let year = year_of(&context(), kathmandu(), (2026, 8, 1));
    let raja = office(&year, YearOffice::Raja);
    let sunrise = raja
        .readings
        .iter()
        .find(|r| r.rule == RulerDayRule::Sunrise)
        .unwrap();
    assert_eq!(sunrise.lord, Graha::Jupiter);
    assert!(sunrise.day.get() < year.began.get());
    assert_eq!(
        under(&year, YearOffice::Raja, RulerDayRule::NewMoon),
        Graha::Jupiter
    );
}

/// VS 2082 (Vishwavasu): the Sun raja, Chaitra Shukla Pratipada being
/// Sunday 30 March 2025; Mesha's ingress at 03:30 on Monday 14 April was
/// printed as the Moon's, the next daylight's (hindupad.com's
/// navanayaka).
#[test]
fn vs_2082_the_sun_raja_and_the_moon_mantri() {
    let year = year_of(&context(), hyderabad(), (2025, 8, 1));
    assert_eq!(year.vikrama, 2082);
    assert_eq!(office(&year, YearOffice::Raja).lord(), Some(Graha::Sun));
    assert_eq!(office(&year, YearOffice::Mantri).lord(), Some(Graha::Moon));
    assert_eq!(
        under(&year, YearOffice::Mantri, RulerDayRule::Vara),
        Graha::Sun
    );
}

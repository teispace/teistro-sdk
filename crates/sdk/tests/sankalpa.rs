//! The time a sankalpa recites (`03-design/sankalpa.md`): the parts read
//! at the day's sunrise are that day's, the parts read at the rite are the
//! spans holding it, a rite before dawn is the day before's, and the two
//! ayana reckonings part between the solstice and Makara sankranti.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use std::collections::BTreeSet;

use teistro::catalogue::{Ayana, Calendar, Rashi};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{AyanaReckoning, CalendarDate, Context, Ephemeris, Sankalpa, Span, UtcOffset};

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

fn day(y: i32, m: u8, d: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, y, m, d)
}

/// An instant given as a Nepal clock reading on a Gregorian day.
fn nepal(y: i32, m: u8, d: u8, hour: f64) -> JulianDay<Utc> {
    let midnight = jd_at_midnight(y, m, d);
    JulianDay::try_new(midnight + (hour - 5.75) / 24.0).unwrap()
}

/// The JD of a Gregorian day's 00:00 UTC (Meeus 7.1).
fn jd_at_midnight(year: i32, month: u8, date: u8) -> f64 {
    let (year, month) = if month <= 2 {
        (year - 1, i32::from(month) + 12)
    } else {
        (year, i32::from(month))
    };
    let century = year.div_euclid(100);
    let gregorian = 2 - century + century.div_euclid(4);
    (365.25 * f64::from(year + 4716)).floor()
        + (30.6001 * f64::from(month + 1)).floor()
        + f64::from(date)
        + f64::from(gregorian)
        - 1524.5
}

fn holding<T: Copy>(spans: &[Span<T>], at: JulianDay<Utc>) -> T {
    spans
        .iter()
        .find(|span| span.whole.from.get() <= at.get() && at.get() < span.whole.to.get())
        .unwrap()
        .member
}

#[test]
fn a_morning_rite_reads_its_day_at_sunrise_and_its_limbs_at_the_rite() {
    let sdk = context();
    let (place, clock) = kathmandu();
    for (date, at) in [
        (day(2026, 1, 15), nepal(2026, 1, 15, 8.0)),
        (day(2026, 4, 20), nepal(2026, 4, 20, 14.5)),
        (day(2026, 8, 28), nepal(2026, 8, 28, 22.25)),
    ] {
        let sankalpa = sdk
            .almanac()
            .sankalpa(at, &place, clock, AyanaReckoning::Sidereal)
            .unwrap()
            .value;
        let panchanga = sdk.almanac().day(&date, &place, clock).unwrap().value;
        let mut year = sdk
            .almanac()
            .years(&date, &date, &place, clock)
            .unwrap()
            .value;
        // A sankalpa recites the year's name and numbers, not its rulers.
        assert!(sankalpa.year.rulers.is_empty() && !year[0].rulers.is_empty());
        year[0].rulers.clear();
        assert_eq!(sankalpa.date, panchanga.day.date, "{date}");
        assert_eq!(sankalpa.vara, panchanga.day.vara, "{date}");
        assert_eq!(sankalpa.sunrise, panchanga.day.sunrise, "{date}");
        assert_eq!(sankalpa.next_sunrise, panchanga.day.next_sunrise, "{date}");
        assert_eq!(sankalpa.year, year[0], "{date}");
        assert_eq!(sankalpa.month, panchanga.month, "{date}");
        assert_eq!(sankalpa.ritu, panchanga.sun.ritu, "{date}");
        assert_eq!(sankalpa.ayana, panchanga.sun.ayana, "{date}");
        let limbs = &panchanga.limbs;
        assert_eq!(sankalpa.tithi, holding(&limbs.tithi, at), "{date}");
        assert_eq!(
            sankalpa.paksha,
            sankalpa.tithi.attributes().paksha,
            "{date}"
        );
        assert_eq!(sankalpa.nakshatra, holding(&limbs.nakshatra, at), "{date}");
        assert_eq!(sankalpa.yoga, holding(&limbs.yoga, at), "{date}");
        assert_eq!(sankalpa.karana, holding(&limbs.karana, at), "{date}");
    }
}

#[test]
fn a_rite_before_dawn_is_the_day_befores() {
    let sdk = context();
    let (place, clock) = kathmandu();
    let at = nepal(2026, 1, 15, 4.0);
    let sankalpa = sdk
        .almanac()
        .sankalpa(at, &place, clock, AyanaReckoning::Sidereal)
        .unwrap()
        .value;
    let date = &sankalpa.date;
    assert_eq!((date.year, date.month, date.day), (2026, 1, 14));
    assert!(sankalpa.sunrise.get() < at.get() && at.get() < sankalpa.next_sunrise.get());
}

#[test]
fn the_reckonings_part_between_the_solstice_and_makara_sankranti() {
    let sdk = context();
    let (place, clock) = kathmandu();
    let ask = |at, ayana| {
        sdk.almanac()
            .sankalpa(at, &place, clock, ayana)
            .unwrap()
            .value
    };
    // 2026-01-05: the tropical Sun is past the winter solstice, the
    // sidereal one not yet in Makara.
    let between = nepal(2026, 1, 5, 9.0);
    assert_eq!(
        ask(between, AyanaReckoning::Sidereal).ayana,
        Ayana::Dakshinayana
    );
    let tropical = ask(between, AyanaReckoning::Tropical);
    assert_eq!(tropical.ayana, Ayana::Uttarayana);
    assert_eq!(tropical.ayana_reckoning, AyanaReckoning::Tropical);
    // 2026-03-01: both are past their turn.
    let after = nepal(2026, 3, 1, 9.0);
    assert_eq!(
        ask(after, AyanaReckoning::Sidereal).ayana,
        Ayana::Uttarayana
    );
    assert_eq!(
        ask(after, AyanaReckoning::Tropical).ayana,
        Ayana::Uttarayana
    );
}

#[test]
fn the_answer_is_hashed_over_what_was_asked() {
    let sdk = context();
    let (place, clock) = kathmandu();
    let at = nepal(2026, 1, 15, 8.0);
    let hash = |ayana| {
        sdk.almanac()
            .sankalpa(at, &place, clock, ayana)
            .unwrap()
            .provenance
            .input_hash
    };
    assert_eq!(
        hash(AyanaReckoning::Sidereal),
        hash(AyanaReckoning::Sidereal)
    );
    assert_ne!(
        hash(AyanaReckoning::Sidereal),
        hash(AyanaReckoning::Tropical)
    );
}

#[test]
fn the_signs_are_the_bodies_at_the_rite() {
    let sdk = context();
    let (place, clock) = kathmandu();
    // The morning after Makara sankranti 2026 (14 January): the Sun has
    // entered Capricorn, and Jupiter, retrograde, stands in Gemini.
    let sankalpa = sdk
        .almanac()
        .sankalpa(
            nepal(2026, 1, 15, 8.0),
            &place,
            clock,
            AyanaReckoning::Sidereal,
        )
        .unwrap()
        .value;
    assert_eq!(sankalpa.sun, Rashi::Capricorn);
    assert_eq!(sankalpa.jupiter, Rashi::Gemini);
    assert_eq!(sankalpa.ayana, Ayana::Uttarayana);
}

/// Every string path in a JSON value, its list indices dropped.
fn string_paths(value: &serde_json::Value, at: &str, found: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::String(_) => {
            found.insert(at.to_owned());
        }
        serde_json::Value::Array(items) => {
            for item in items {
                string_paths(item, at, found);
            }
        }
        serde_json::Value::Object(fields) => {
            for (key, inner) in fields {
                let path = if at.is_empty() {
                    key.clone()
                } else {
                    format!("{at}.{key}")
                };
                string_paths(inner, &path, found);
            }
        }
        _ => {}
    }
}

/// `Sankalpa::MEMBERS` names every catalogue member a sankalpa holds and
/// nothing else, and what it writes in full reads back.
#[test]
fn a_sankalpa_names_its_catalogue_members_where_its_table_says() {
    // A setting's value, a state or a description: not a member.
    const NOT_MEMBERS: [&str; 6] = [
        "ayanaReckoning",
        "rituReckoning",
        "month.convention",
        "month.kind",
        "year.count",
        "date.resolution.kind",
    ];
    let sdk = context();
    let (place, clock) = kathmandu();
    // A day of an expunged year, so `lupta` holds a string: Ananda began
    // and ended inside VS 2077 (`samvatsara-measured.md`).
    let mut found = BTreeSet::new();
    for at in [nepal(2026, 1, 15, 8.0), nepal(2020, 8, 1, 8.0)] {
        let sankalpa = sdk
            .almanac()
            .sankalpa(at, &place, clock, AyanaReckoning::Sidereal)
            .unwrap()
            .value;
        string_paths(&serde_json::to_value(&sankalpa).unwrap(), "", &mut found);
        let full = sankalpa.in_full().unwrap();
        assert_eq!(full["tithi"], format!("tithi.{}", sankalpa.tithi.key()));
        let back: Sankalpa = serde_json::from_value(full).unwrap();
        assert_eq!(back, sankalpa);
    }
    let members: BTreeSet<String> = Sankalpa::MEMBERS
        .iter()
        .map(|(path, _)| (*path).to_owned())
        .collect();
    let plain: BTreeSet<String> = NOT_MEMBERS.iter().map(|path| (*path).to_owned()).collect();
    assert!(members.is_disjoint(&plain));
    let listed: BTreeSet<String> = members.union(&plain).cloned().collect();
    assert_eq!(
        found, listed,
        "a string path is unlisted, or a listed one is gone"
    );
}

/// A binding's record reads what the façade answers, its members written
/// in full, and a key the reader does not take is refused by name.
#[test]
fn the_record_answers_the_facade_in_full() {
    let sdk = context();
    let (place, clock) = kathmandu();
    let at = nepal(2026, 1, 15, 8.0);
    let record = teistro::records::record("almanac.sankalpa").unwrap();
    let answered = record
        .answer(
            &sdk,
            &format!(
                r#"{{"at": {}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
                    "altitudeM": 1400, "utcOffsetSeconds": 20700}}"#,
                at.get()
            ),
        )
        .unwrap();
    let facade = sdk
        .almanac()
        .sankalpa(at, &place, clock, AyanaReckoning::Sidereal)
        .unwrap()
        .value;
    assert_eq!(answered.value, facade.in_full().unwrap());
    assert_eq!(answered.value["sun"], "rashi.CAPRICORN");
    let refused = record
        .answer(&sdk, r#"{"at": 2461055.6, "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20700, "when": 1}"#)
        .unwrap_err();
    assert_eq!(refused.field(), Some("sankalpa.when"));
}

//! The N-day digest (`03-design/digest-format.md`): the bytes read back
//! through the reference reader against the panchanga they came from, its
//! seal and version, and its size.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, Context, DIGEST_MAX_DAYS, DIGEST_NO_NAME, DayClock, DigestRequest, DigestSpan,
    FestivalPack, FestivalRequest, Hash, Status, UtcOffset, ZoneSpec, digest_schema,
};
use teistro_idl::blob::{Reader, ScalarValue};

/// What 60 days may weigh; the consumer's ceiling is a few hundred KB.
const SIXTY_DAYS_BYTES: usize = 200 * 1024;

/// The sections of the week `a_digest_is_the_same_bytes_on_every_platform`
/// writes, hashed.
const PINNED: &str = "3d64b337209fb8186faeed9ea4469d51048f52983c362fcab237afbe64d4947d";

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

fn sdk() -> Context {
    Context::builder().nepal_official([]).build().unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn asked(days: u32) -> DigestRequest {
    DigestRequest {
        first: CalendarDate::defined(Calendar::Gregorian, 2025, 9, 20),
        days,
        place: kathmandu(),
        clock: DayClock::Offset(UtcOffset::literal(5, 45, 0)),
        festivals: Some(FestivalRequest::from(FestivalPack::NepalPanchanga)),
    }
}

fn int(value: &ScalarValue) -> i64 {
    value.as_i64()
}

fn ints(reader: &Reader<'_, '_>, section: &str, column: &str) -> Vec<i64> {
    reader
        .column(section, column)
        .unwrap()
        .iter()
        .map(int)
        .collect()
}

#[test]
fn the_days_read_back_as_the_panchanga_they_came_from() {
    let sdk = sdk();
    let request = asked(10);
    let bytes = sdk.almanac().digest(&request).unwrap().value;
    let schema = digest_schema();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    let last = CalendarDate::defined(Calendar::Gregorian, 2025, 9, 29);
    let days = sdk
        .almanac()
        .of(
            &request.first,
            &last,
            &request.place,
            UtcOffset::literal(5, 45, 0),
        )
        .unwrap()
        .value;
    let base = int(&reader.field("summary", "base_unix_seconds").unwrap());
    // 2025-09-20T00:00 at +05:45.
    assert_eq!(base, 1_758_326_400 - 20_700);
    assert_eq!(int(&reader.field("summary", "day_count").unwrap()), 10);
    assert_eq!(ints(&reader, "days", "day")[0], 20);
    let seconds = |at: teistro::quantity::JulianDay<teistro::quantity::Utc>| {
        let unix = (at.get()
            - teistro::quantity::JulianDay::<teistro::quantity::Utc>::UNIX_EPOCH.get())
            * 86_400.0;
        #[allow(clippy::cast_possible_truncation, reason = "a day in 2025")]
        let whole = unix.round() as i64;
        whole - base
    };
    let sunrise = ints(&reader, "days", "sunrise");
    let first_span = ints(&reader, "days", "span_first");
    let span_count = ints(&reader, "days", "span_count");
    let (kinds, names, from, to) = (
        ints(&reader, "spans", "kind"),
        ints(&reader, "spans", "name"),
        ints(&reader, "spans", "from"),
        ints(&reader, "spans", "to"),
    );
    let text = reader.bytes("text").unwrap();
    let (key_at, key_length) = (
        ints(&reader, "names", "key_at"),
        ints(&reader, "names", "key_length"),
    );
    let key_of = |row: usize| {
        let at = usize::try_from(key_at[row]).unwrap();
        let length = usize::try_from(key_length[row]).unwrap();
        std::str::from_utf8(&text[at..at + length]).unwrap()
    };
    for (index, day) in days.iter().enumerate() {
        assert_eq!(sunrise[index], seconds(day.day.sunrise));
        let start = usize::try_from(first_span[index]).unwrap();
        let rows = start..start + usize::try_from(span_count[index]).unwrap();
        let of_kind = |kind: DigestSpan| {
            rows.clone()
                .filter(|&row| kinds[row] == kind as i64)
                .collect::<Vec<_>>()
        };
        let tithis = of_kind(DigestSpan::Tithi);
        assert_eq!(tithis.len(), day.limbs.tithi.len());
        for (row, span) in tithis.iter().zip(&day.limbs.tithi) {
            assert_eq!(from[*row], seconds(span.whole.from));
            assert_eq!(to[*row], seconds(span.whole.to));
            let name = usize::try_from(names[*row]).unwrap();
            assert!(key_of(name).starts_with("tithi."), "{}", key_of(name));
        }
        assert_eq!(of_kind(DigestSpan::Hora).len(), day.horas.len());
        assert_eq!(of_kind(DigestSpan::Choghadiya).len(), day.choghadiya.len());
        assert_eq!(
            of_kind(DigestSpan::Abhijit).len(),
            usize::from(day.muhurtas.abhijit.is_some())
        );
        for row in of_kind(DigestSpan::Abhijit) {
            assert_eq!(names[row], i64::from(DIGEST_NO_NAME));
        }
    }
}

#[test]
fn observances_are_named_on_their_days_and_a_fast_is_flagged() {
    let bytes = sdk().almanac().digest(&asked(20)).unwrap().value;
    let schema = digest_schema();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    let text = std::str::from_utf8(reader.bytes("text").unwrap()).unwrap();
    assert!(text.contains("VIJAYA_DASHAMI") && text.contains("Vijaya Dashami"));
    let flags = ints(&reader, "observances", "flags");
    assert!(flags.contains(&1), "an Ekadashi falls in the range");
    let counts: i64 = ints(&reader, "days", "observance_count").iter().sum();
    assert_eq!(counts, i64::try_from(flags.len()).unwrap());
}

#[test]
fn the_same_request_writes_the_same_bytes_and_the_seal_covers_them() {
    let sdk = sdk();
    let answer = sdk.almanac().digest(&asked(5)).unwrap();
    assert_eq!(answer.value, sdk.almanac().digest(&asked(5)).unwrap().value);
    assert_eq!(answer.provenance.content_hash, Hash::of(&answer.value));
    let sealed = answer.value.len() - 64;
    assert_eq!(
        hex(Hash::of(&answer.value[..sealed]).bytes()),
        std::str::from_utf8(&answer.value[sealed..]).unwrap()
    );
}

/// The determinism claim, pinned where every platform's tests run it: a
/// week's sections, provenance left out since it names the build.
#[test]
fn a_digest_is_the_same_bytes_on_every_platform() {
    let sdk = sdk();
    let answer = sdk.almanac().digest(&asked(7)).unwrap();
    let schema = digest_schema();
    let reader = Reader::parse(&answer.value, &schema).unwrap();
    let mut read = Vec::new();
    for section in [
        "summary",
        "days",
        "spans",
        "observances",
        "names",
        "text",
        "locale",
        "clock",
    ] {
        let entry = reader.entry(section).unwrap();
        read.extend_from_slice(&answer.value[entry.offset..entry.offset + entry.length]);
    }
    assert_eq!(hex(Hash::of(&read).bytes()), PINNED);
}

#[test]
fn no_festivals_writes_no_observances() {
    let mut request = asked(3);
    request.festivals = None;
    let bytes = sdk().almanac().digest(&request).unwrap().value;
    let schema = digest_schema();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    assert_eq!(reader.count("observances"), Some(0));
}

#[test]
fn a_count_out_of_range_is_refused_by_its_field() {
    let sdk = sdk();
    for days in [0, DIGEST_MAX_DAYS + 1] {
        let refused = sdk.almanac().digest(&asked(days)).unwrap_err();
        assert_eq!(refused.status, Status::OutOfRange);
        assert_eq!(refused.field(), Some("days"));
    }
}

#[test]
fn a_60_day_digest_keeps_its_size() {
    let bytes = sdk().almanac().digest(&asked(60)).unwrap().value;
    assert!(
        bytes.len() <= SIXTY_DAYS_BYTES,
        "{} bytes for 60 days",
        bytes.len()
    );
}

#[test]
fn the_record_reads_its_defaults_and_names_a_refusal() {
    let base = r#""first": {"year": 2082, "month": 1, "day": 1}, "calendar": "BIKRAM_SAMBAT",
        "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700"#;
    let asked = DigestRequest::from_json(&format!("{{{base}}}")).unwrap();
    assert_eq!(asked.days, 60);
    assert!(asked.festivals.is_none());
    assert_eq!(asked.first.calendar, Calendar::BikramSambat);
    let refused =
        DigestRequest::from_json(&format!(r#"{{{base}, "festivals": {{"rules": "NOPE"}}}}"#))
            .unwrap_err();
    assert!(refused.field().unwrap().starts_with("digest"));
    let refused = DigestRequest::from_json(&format!(r#"{{{base}, "utcOffsetSeconds": 999999}}"#))
        .unwrap_err();
    assert!(refused.field().unwrap().starts_with("digest"));
    let out = DigestRequest::from_json(r#"{"first": {"year": 2026, "month": 1, "day": 1}, "latitudeDeg": 99.0, "longitudeDeg": 0.0, "utcOffsetSeconds": 0}"#).unwrap_err();
    assert_eq!(out.field(), Some("digest.latitudeDeg"));
}

#[test]
fn a_bikram_sambat_digest_dates_its_days_there() {
    let sdk = sdk();
    let request = DigestRequest {
        first: CalendarDate::defined(Calendar::BikramSambat, 2082, 1, 1),
        ..asked(2)
    };
    let bytes = sdk.almanac().digest(&request).unwrap().value;
    let schema = digest_schema();
    let reader = Reader::parse(&bytes, &schema).unwrap();
    assert_eq!(ints(&reader, "days", "year"), [2082, 2082]);
    assert_eq!(ints(&reader, "days", "day"), [1, 2]);
    assert_eq!(
        int(&reader.field("summary", "calendar").unwrap()),
        i64::from(teistro::catalogue::Catalogued::id(Calendar::BikramSambat))
    );
}

#[test]
fn a_polar_day_says_so() {
    let sdk = sdk();
    let request = DigestRequest {
        first: CalendarDate::defined(Calendar::Gregorian, 2026, 6, 21),
        days: 1,
        place: Place::new(
            Latitude::literal(78.2),
            Longitude::literal(15.6),
            Altitude::literal(0.0),
        ),
        clock: DayClock::Offset(UtcOffset::literal(1, 0, 0)),
        festivals: None,
    };
    let answer = sdk.almanac().digest(&request).unwrap();
    let schema = digest_schema();
    let reader = Reader::parse(&answer.value, &schema).unwrap();
    assert_eq!(ints(&reader, "days", "state"), [2]);
}

#[test]
fn a_zone_writes_each_day_on_its_own_offset() {
    let sdk = sdk();
    let new_york = Place::new(
        Latitude::literal(40.7128),
        Longitude::literal(-74.006),
        Altitude::literal(10.0),
    );
    let on = |clock: DayClock| DigestRequest {
        first: CalendarDate::defined(Calendar::Gregorian, 2026, 10, 31),
        days: 3,
        place: new_york,
        clock,
        festivals: None,
    };
    let zoned = sdk
        .almanac()
        .digest(&on(DayClock::Zone(ZoneSpec::iana("America/New_York"))))
        .unwrap();
    let schema = digest_schema();
    let reader = Reader::parse(&zoned.value, &schema).unwrap();
    assert_eq!(
        ints(&reader, "clock", "utc_offset_seconds"),
        [-14_400, -18_000, -18_000]
    );
    // The summary and the base are the first day's.
    assert_eq!(
        int(&reader.field("summary", "utc_offset_seconds").unwrap()),
        -14_400
    );
    assert_eq!(
        int(&reader.field("summary", "base_unix_seconds").unwrap()),
        // 2026-10-31 is 20,757 days from 1970-01-01.
        20_757 * 86_400 + 14_400
    );
    assert_eq!(int(&reader.field("summary", "format_minor").unwrap()), 1);
    assert_eq!(
        zoned.provenance.time.clock.unwrap().offset.seconds(),
        -14_400
    );

    // A stated offset is every day's.
    let stated = sdk
        .almanac()
        .digest(&on(DayClock::Offset(UtcOffset::literal(-4, 0, 0))))
        .unwrap();
    let reader = Reader::parse(&stated.value, &schema).unwrap();
    assert_eq!(
        ints(&reader, "clock", "utc_offset_seconds"),
        [-14_400, -14_400, -14_400]
    );

    // A zone the database does not have is the zone's own refusal.
    let unknown = sdk
        .almanac()
        .digest(&on(DayClock::Zone(ZoneSpec::iana("America/Nowhere"))))
        .unwrap_err();
    assert_eq!(unknown.field(), Some("zone"));
}

#[test]
fn a_request_names_one_clock() {
    let text = |clock: &str| {
        format!(
            r#"{{"first": {{"year": 2026, "month": 1, "day": 1}}, "latitudeDeg": 27.7, "longitudeDeg": 85.3{clock}}}"#
        )
    };
    let both =
        text(r#", "utcOffsetSeconds": 20700, "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}"#);
    assert_eq!(
        DigestRequest::from_json(&both).unwrap_err().field(),
        Some("digest.zone")
    );
    assert_eq!(
        DigestRequest::from_json(&text("")).unwrap_err().field(),
        Some("digest.utcOffsetSeconds")
    );
    let asked = DigestRequest::from_json(&text(
        r#", "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}"#,
    ))
    .unwrap();
    assert_eq!(
        asked.clock,
        DayClock::Zone(ZoneSpec::iana("Asia/Kathmandu"))
    );
}

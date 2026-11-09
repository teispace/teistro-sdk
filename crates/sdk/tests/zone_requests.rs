//! Every request beside the chart and the days asked on a zone's clock
//! (`docs/03-design/time-and-timezone.md` §3.4, SR-10): `zone` in place of
//! `utcOffsetSeconds`, each instant read at the offset the zone had then,
//! both together refused naming `zone`, and a bare offset answered byte for
//! byte as before. The instants are real Kathmandu ones: Nepal kept +05:30
//! until 1986 and +05:45 since.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they found"
)]

use serde_json::{Value, json};
use teistro::catalogue::Calendar;
use teistro::finders::{ElectRequest, ObservanceRequest, PeopleRequest};
use teistro::first_year::FirstYear;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rectification::{Rules, Window};
use teistro::verify::ClaimsRequest;
use teistro::{
    AnniversariesRequest, AshauchQuery, AyanaReckoning, CalendarDate, ChartRequest, Context,
    DayClock, DeathTimeQuery, DisagreementRequest, Envelope, Ephemeris, FestivalCalendarRequest,
    FestivalRequest, FirstYearQuery, HouseholdQuery, MuhurtaRequest, MundaneRequest, NextRequest,
    PakshiRequest, PartnerMatching, PartnerSynastry, PublishedReasonsRequest, RashifalBatch,
    RectificationRequest, ResearchAnswer, ResearchRequest, SankalpaRequest, SentencesRequest,
    UtcOffset, ZoneSource, ZoneSpec, content_hash,
};

fn context() -> Context {
    Context::builder()
        .profile("nepali-drik")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

/// The place as every record here writes it.
const PLACE: &str = r#""latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400"#;
/// The zone as every record here writes it.
const ZONE: &str = r#""zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}"#;

/// Noon in Kathmandu on 1 June 1984 (+05:30), 06:30 UTC: a first year
/// from it ends before the change.
const NOON_1984: f64 = 2_445_852.770_833;
/// Noon in Kathmandu on 1 June 1985 (+05:30), 06:30 UTC.
const NOON_1985: f64 = 2_446_217.770_833;
/// Noon in Kathmandu on 1 June 2026 (+05:45), 06:15 UTC.
const NOON_2026: f64 = 2_461_192.760_417;

/// Each instant with the offset Nepal kept at it.
const BOTH: [(f64, i32); 2] = [(NOON_1985, 19_800), (NOON_2026, 20_700)];

fn at(jd: f64) -> JulianDay<Utc> {
    JulianDay::<Utc>::literal(jd)
}

fn offset(seconds: i32) -> UtcOffset {
    UtcOffset::try_from_seconds(seconds).unwrap()
}

fn nepal() -> ZoneSpec {
    ZoneSpec::iana("Asia/Kathmandu")
}

/// The clock field a record is sent with: an offset's, or the zone's.
fn clock(seconds: Option<i32>) -> String {
    seconds.map_or_else(
        || ZONE.to_owned(),
        |seconds| format!(r#""utcOffsetSeconds": {seconds}"#),
    )
}

/// The offset a provenance states, and whether the zone database gave it.
fn stated<T>(answer: &Envelope<T>) -> (i32, bool) {
    let clock = answer.provenance.time.clock.as_ref().unwrap();
    (clock.offset.seconds(), clock.source == ZoneSource::Iana)
}

/// A JSON answer and the façade's own answer, sealed over the same value,
/// are one answer byte for byte.
/// That a request by zone and one by the offset the zone kept answer the
/// same and seal apart: the zone is part of what was asked (C1252).
fn sealed_apart<T: PartialEq + std::fmt::Debug>(
    zoned: &Envelope<T>,
    stated: &Envelope<T>,
    what: &str,
) {
    assert_eq!(zoned.value, stated.value, "{what}");
    assert_ne!(
        zoned.provenance.input_hash, stated.provenance.input_hash,
        "{what}: the zone is sealed"
    );
}

fn same_bytes(sent: &Envelope<Value>, direct: Value) {
    let direct = Envelope::sealing(direct, sent.provenance.clone());
    assert_eq!(
        teistro::canonical_json(&sent.value),
        teistro::canonical_json(&direct.value)
    );
    assert_eq!(sent.provenance.content_hash, direct.provenance.content_hash);
}

/// Both fields together are refused, named `zone` under the record.
fn refuses_both<T: std::fmt::Debug>(
    read: impl Fn(&str) -> Result<T, teistro::Error>,
    record: &str,
    root: &str,
) {
    let both = format!(r#""utcOffsetSeconds": 19800, {ZONE}"#);
    let refused = read(&record.replace("CLOCK", &both)).unwrap_err();
    assert_eq!(
        refused.field(),
        Some(format!("{root}.zone").as_str()),
        "{refused}"
    );
}

// ---- sankalpa and the claims judged at one -------------------------------

fn sankalpa_record(jd: f64, seconds: Option<i32>) -> String {
    format!(r#"{{"at": {jd}, {PLACE}, {}}}"#, clock(seconds))
}

#[test]
fn a_sankalpa_on_an_offset_answers_as_before() {
    let sdk = context();
    let asked = SankalpaRequest::from_json(&sankalpa_record(NOON_2026, Some(20_700))).unwrap();
    assert_eq!(asked.clock, DayClock::Offset(offset(20_700)));
    let sent = sdk.almanac().sankalpa_request(&asked).unwrap();
    let direct = sdk
        .almanac()
        .sankalpa(
            at(NOON_2026),
            &kathmandu(),
            offset(20_700),
            AyanaReckoning::Sidereal,
        )
        .unwrap();
    same_bytes(&sent, direct.value.in_full().unwrap());
    assert_eq!(stated(&sent), (20_700, false));
}

#[test]
fn a_sankalpa_on_a_zone_reads_each_instant_on_its_own_offset() {
    let sdk = context();
    for (jd, seconds) in BOTH {
        let zoned = SankalpaRequest::from_json(&sankalpa_record(jd, None)).unwrap();
        let zoned = sdk.almanac().sankalpa_request(&zoned).unwrap();
        let stated_offset =
            SankalpaRequest::from_json(&sankalpa_record(jd, Some(seconds))).unwrap();
        let stated_offset = sdk.almanac().sankalpa_request(&stated_offset).unwrap();
        sealed_apart(&zoned, &stated_offset, &format!("sankalpa {jd}"));
        assert_eq!(stated(&zoned), (seconds, true));
    }
}

#[test]
fn a_sankalpa_and_its_claims_refuse_both_clocks() {
    let record = format!(r#"{{"at": {NOON_2026}, {PLACE}, CLOCK}}"#);
    refuses_both(SankalpaRequest::from_json, &record, "sankalpa");
    let claims = format!(r#"{{"text": "x", "at": {NOON_2026}, {PLACE}, CLOCK}}"#);
    refuses_both(ClaimsRequest::from_json, &claims, "claims");
    let neither = SankalpaRequest::from_json(&format!(r#"{{"at": {NOON_2026}, {PLACE}}}"#));
    assert_eq!(
        neither.unwrap_err().field(),
        Some("sankalpa.utcOffsetSeconds")
    );
}

// ---- a death's first year ------------------------------------------------

fn first_year_record(jd: f64, seconds: Option<i32>) -> String {
    format!(
        r#"{{{PLACE}, {}, "request": {{"key": "GRANDFATHER", "at": {jd}}}}}"#,
        clock(seconds)
    )
}

#[test]
fn a_first_year_on_an_offset_answers_as_before() {
    let sdk = context();
    let asked = FirstYearQuery::from_json(&first_year_record(NOON_2026, Some(20_700))).unwrap();
    let sent = sdk.almanac().first_year_query(&asked).unwrap();
    let direct = sdk
        .almanac()
        .first_year(
            &FirstYear::dharmasindhu("GRANDFATHER", at(NOON_2026)),
            &kathmandu(),
            offset(20_700),
        )
        .unwrap();
    same_bytes(&sent, direct.value.in_full().unwrap());
}

#[test]
fn a_first_year_on_a_zone_reads_each_death_on_its_own_offset() {
    let sdk = context();
    // A year from 1984 ends before Nepal's change, so it is the year on
    // +05:30 throughout.
    for (jd, seconds) in [(NOON_1984, 19_800), (NOON_2026, 20_700)] {
        let read = |seconds: Option<i32>| {
            let asked = FirstYearQuery::from_json(&first_year_record(jd, seconds)).unwrap();
            sdk.almanac().first_year_query(&asked).unwrap()
        };
        let zoned = read(None);
        sealed_apart(&zoned, &read(Some(seconds)), &format!("first year {jd}"));
        assert_eq!(stated(&zoned).0, seconds);
    }
}

#[test]
fn a_first_year_refuses_both_clocks() {
    refuses_both(
        FirstYearQuery::from_json,
        &format!(r#"{{{PLACE}, CLOCK, "request": {{"key": "K", "at": {NOON_2026}}}}}"#),
        "firstYear",
    );
}

// ---- ashauch: a period, a household and a death's time --------------------

fn ashauch_record(jd: f64, seconds: Option<i32>) -> String {
    format!(
        r#"{{{PLACE}, {}, "request": {{"key": "grandfather", "relation": "SAPINDA",
            "event": {{"event": "DEATH", "at": {jd}}}}}}}"#,
        clock(seconds)
    )
}

fn household_record(jd: f64, seconds: Option<i32>) -> String {
    format!(
        r#"{{{PLACE}, {}, "household": {{
            "events": [{{"key": "grandfather", "event": {{"event": "DEATH", "at": {jd}}}}}],
            "observers": [{{"key": "son", "ties": [{{"event": "grandfather", "relation": "CHILD"}}]}}]}}}}"#,
        clock(seconds)
    )
}

fn death_time_record(jd: f64, seconds: Option<i32>) -> String {
    format!(r#"{{{PLACE}, {}, "at": {jd}}}"#, clock(seconds))
}

#[test]
fn an_ashauch_on_an_offset_answers_as_before() {
    let sdk = context();
    let asked = AshauchQuery::from_json(&ashauch_record(NOON_2026, Some(20_700))).unwrap();
    let sent = sdk.almanac().ashauch_query(&asked).unwrap();
    let direct = sdk
        .almanac()
        .ashauch(&asked.table, &asked.request, &kathmandu(), offset(20_700))
        .unwrap();
    same_bytes(&sent, direct.value.in_full().unwrap());
    let asked = DeathTimeQuery::from_json(&death_time_record(NOON_2026, Some(20_700))).unwrap();
    let sent = sdk.almanac().death_time_query(&asked).unwrap();
    let direct = sdk
        .almanac()
        .death_time(
            &asked.table,
            at(NOON_2026),
            None,
            &kathmandu(),
            offset(20_700),
        )
        .unwrap();
    same_bytes(&sent, direct.value.in_full().unwrap());
}

#[test]
fn an_ashauch_on_a_zone_reads_each_death_on_its_own_offset() {
    let sdk = context();
    let almanac = sdk.almanac();
    for (jd, seconds) in BOTH {
        let ashauch = |seconds| {
            let asked = AshauchQuery::from_json(&ashauch_record(jd, seconds)).unwrap();
            almanac.ashauch_query(&asked).unwrap()
        };
        let zoned = ashauch(None);
        sealed_apart(&zoned, &ashauch(Some(seconds)), &format!("ashauch {jd}"));
        assert_eq!(stated(&zoned), (seconds, true));
        let household = |seconds| {
            let asked = HouseholdQuery::from_json(&household_record(jd, seconds)).unwrap();
            almanac.household_query(&asked).unwrap()
        };
        sealed_apart(
            &household(None),
            &household(Some(seconds)),
            &format!("household {jd}"),
        );
        let death_time = |seconds| {
            let asked = DeathTimeQuery::from_json(&death_time_record(jd, seconds)).unwrap();
            almanac.death_time_query(&asked).unwrap()
        };
        sealed_apart(
            &death_time(None),
            &death_time(Some(seconds)),
            &format!("death time {jd}"),
        );
    }
}

#[test]
fn an_ashauch_refuses_both_clocks() {
    let event = format!(r#"{{"event": "DEATH", "at": {NOON_2026}}}"#);
    refuses_both(
        AshauchQuery::from_json,
        &format!(
            r#"{{{PLACE}, CLOCK, "request": {{"key": "k", "relation": "SAPINDA", "event": {event}}}}}"#
        ),
        "ashauch",
    );
    refuses_both(
        HouseholdQuery::from_json,
        &format!(
            r#"{{{PLACE}, CLOCK, "household": {{"events": [{{"key": "k", "event": {event}}}],
                "observers": [{{"key": "o", "ties": [{{"event": "k", "relation": "CHILD"}}]}}]}}}}"#
        ),
        "household",
    );
    refuses_both(
        DeathTimeQuery::from_json,
        &format!(r#"{{{PLACE}, CLOCK, "at": {NOON_2026}}}"#),
        "deathTime",
    );
}

// ---- Pancha Pakshi -------------------------------------------------------

fn pakshi_record(year: i32, seconds: Option<i32>) -> String {
    format!(
        r#"{{"first": {{"year": {year}, "month": 6, "day": 1}}, {PLACE}, {},
            "native": {{"bird": "OWL"}}}}"#,
        clock(seconds)
    )
}

#[test]
fn a_pakshi_on_an_offset_answers_as_before() {
    let sdk = context();
    let asked = PakshiRequest::from_json(&pakshi_record(2026, Some(20_700))).unwrap();
    let sent = sdk.almanac().pakshi_request(&asked).unwrap();
    let day = CalendarDate::defined(Calendar::Gregorian, 2026, 6, 1);
    let direct = sdk
        .almanac()
        .pakshi_days(
            &day,
            &day,
            &kathmandu(),
            offset(20_700),
            asked.native,
            &asked.rules,
        )
        .unwrap();
    assert_eq!(sent, direct);
}

#[test]
fn a_pakshi_on_a_zone_reads_each_day_on_its_own_offset() {
    let sdk = context();
    for (year, seconds) in [(1985, 19_800), (2026, 20_700)] {
        let read = |seconds| {
            let asked = PakshiRequest::from_json(&pakshi_record(year, seconds)).unwrap();
            sdk.almanac().pakshi_request(&asked).unwrap()
        };
        let zoned = read(None);
        sealed_apart(&zoned, &read(Some(seconds)), &format!("pakshi {year}"));
        assert_eq!(stated(&zoned), (seconds, true));
    }
    refuses_both(
        PakshiRequest::from_json,
        &format!(
            r#"{{"first": {{"year": 2026, "month": 6, "day": 1}}, {PLACE}, CLOCK, "native": {{"bird": "OWL"}}}}"#
        ),
        "pakshi",
    );
}

// ---- rashifal ------------------------------------------------------------

fn rashifal_record(year: i32, seconds: Option<i32>) -> String {
    format!(
        r#"{{"periods": [{{"first": {{"year": {year}, "month": 6, "day": 1}}, {PLACE}, {},
            "snapshot": {{"at": "CLOCK", "hour": 6, "minute": 0}}}}]}}"#,
        clock(seconds)
    )
}

#[test]
fn a_rashifal_on_an_offset_answers_as_before() {
    let sdk = context();
    let asked = RashifalBatch::from_json(&rashifal_record(2026, Some(20_700))).unwrap();
    let sent = sdk.chart().rashifal_answers(&asked).unwrap();
    let day = CalendarDate::defined(Calendar::Gregorian, 2026, 6, 1);
    let direct = sdk
        .chart()
        .rashifal(
            &teistro::RashifalRequest::day(day, kathmandu(), offset(20_700))
                .at(teistro::Snapshot::Clock { hour: 6, minute: 0 }),
        )
        .unwrap();
    assert_eq!(sent.value[0].period, direct.value);
}

#[test]
fn a_rashifal_on_a_zone_reads_each_day_on_its_own_offset() {
    let sdk = context();
    for (year, seconds) in [(1985, 19_800), (2026, 20_700)] {
        let read = |seconds| {
            let asked = RashifalBatch::from_json(&rashifal_record(year, seconds)).unwrap();
            sdk.chart().rashifal_answers(&asked).unwrap()
        };
        let zoned = read(None);
        assert_eq!(zoned.value, read(Some(seconds)).value, "{year}");
        assert_eq!(stated(&zoned), (seconds, true));
    }
    refuses_both(
        RashifalBatch::from_json,
        &format!(
            r#"{{"periods": [{{"first": {{"year": 2026, "month": 6, "day": 1}}, {PLACE}, CLOCK}}]}}"#
        ),
        "rashifal.periods[0]",
    );
}

// ---- mundane charts ------------------------------------------------------

fn mundane_record(jd: f64, clock: &str) -> String {
    format!(
        r#"{{"kind": "EVENT", "event": {{"name": "A treaty", "instant": {jd}, "source": "a gazette"}},
            {PLACE}{clock}}}"#
    )
}

#[test]
fn a_mundane_chart_on_an_offset_answers_as_before() {
    let sdk = context();
    let read = |clock: &str| {
        let asked = MundaneRequest::from_json(&mundane_record(NOON_2026, clock)).unwrap();
        sdk.chart().mundane(&asked).unwrap()
    };
    let stated_offset = read(r#", "utcOffsetSeconds": 20700"#);
    // The input hash seals the request as it always wrote it.
    let written = json!({"kind": "EVENT",
        "event": {"name": "A treaty", "instant": NOON_2026, "source": "a gazette"},
        "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400.0,
        "utcOffsetSeconds": 20_700});
    assert_eq!(stated_offset.provenance.input_hash, content_hash(&written));
    assert_eq!(stated_offset.value.moment.utc_offset_seconds, 20_700);
    // An offset left out is UTC, sealed as it always was.
    let utc = read("");
    let explicit = read(r#", "utcOffsetSeconds": 0"#);
    assert_eq!(utc, explicit);
}

#[test]
fn a_mundane_chart_on_a_zone_is_cast_on_the_moment_s_offset() {
    let sdk = context();
    for (jd, seconds) in BOTH {
        let read = |clock: &str| {
            let asked = MundaneRequest::from_json(&mundane_record(jd, clock)).unwrap();
            sdk.chart().mundane(&asked).unwrap()
        };
        let mut zoned = read(&format!(", {ZONE}"));
        let stated_offset = read(&format!(r#", "utcOffsetSeconds": {seconds}"#));
        assert_eq!(zoned.value.moment.utc_offset_seconds, seconds);
        let carried = zoned.value.chart.clock.take().unwrap();
        assert_eq!(carried.offset.seconds(), seconds);
        assert_eq!(zoned.value, stated_offset.value, "{jd}");
        assert_eq!(stated(&zoned), (seconds, true));
    }
    refuses_both(
        MundaneRequest::from_json,
        &mundane_record(NOON_2026, ", CLOCK"),
        "mundane",
    );
}

// ---- rectification -------------------------------------------------------

/// Asks the purifier and the Svarodaya, both of which read the day.
const RECTIFICATION: &str = r#"{"purify": {"minutes": 30}, "svarodaya": {"minutes": 30}}"#;

#[test]
fn a_rectification_on_an_offset_seals_what_it_did() {
    let sdk = context();
    let window = Window::between(at(NOON_2026 - 0.01), at(NOON_2026 + 0.01)).unwrap();
    let rules = Rules::default();
    let read = sdk
        .chart()
        .rectify(window, &kathmandu(), offset(20_700), &rules)
        .unwrap();
    // The input hash names no zone for a stated offset, as before.
    let written = json!({"window": window, "place": kathmandu(),
        "utcOffsetSeconds": 20_700, "rules": rules});
    assert_eq!(read.provenance.input_hash, content_hash(&written));
    assert_eq!(stated(&read), (20_700, false));
}

#[test]
fn a_rectification_on_a_zone_reads_each_window_on_its_own_offset() {
    let sdk = context();
    let asked = RectificationRequest::from_json(RECTIFICATION).unwrap();
    for (jd, seconds) in BOTH {
        let birth = sdk
            .chart()
            .reading(at(jd), &ChartRequest::at(kathmandu(), offset(seconds)))
            .unwrap()
            .value;
        let zoned = sdk
            .chart()
            .rectification(&birth, DayClock::Zone(nepal()), &asked)
            .unwrap();
        let stated_offset = sdk
            .chart()
            .rectification(&birth, offset(seconds), &asked)
            .unwrap();
        assert_eq!(zoned, stated_offset, "{jd}");
        let svarodaya = sdk
            .chart()
            .svarodaya(at(jd), &kathmandu(), DayClock::Zone(nepal()))
            .unwrap();
        assert_eq!(stated(&svarodaya), (seconds, true));
    }
}

// ---- research ------------------------------------------------------------

fn study(births: &[(f64, Option<i32>)]) -> ResearchAnswer {
    let births: Vec<Value> = births
        .iter()
        .map(|&(jd, seconds)| {
            let mut birth = json!({"instant": jd, "latitudeDeg": 27.7172, "longitudeDeg": 85.324});
            match seconds {
                Some(seconds) => birth["utcOffsetSeconds"] = json!(seconds),
                None => birth["zone"] = json!({"kind": "IANA", "zone": "Asia/Kathmandu"}),
            }
            birth
        })
        .collect();
    let request = json!({"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}, "holds": "FORMED",
        "births": births, "design": {"groups": [0, 1]}});
    context()
        .research()
        .request(&ResearchRequest::from_json(&request.to_string()).unwrap())
        .unwrap()
}

#[test]
fn a_study_of_zoned_births_reads_each_on_its_own_offset() {
    let ResearchAnswer::Counts(zoned) = study(&[(NOON_1985, None), (NOON_2026, None)]) else {
        panic!("a count study answers counts");
    };
    let ResearchAnswer::Counts(stated_offsets) =
        study(&[(NOON_1985, Some(19_800)), (NOON_2026, Some(20_700))])
    else {
        panic!("a count study answers counts");
    };
    assert_eq!(zoned.value, stated_offsets.value);
    // The zone is sealed into what the study was asked.
    assert_ne!(
        zoned.provenance.input_hash,
        stated_offsets.provenance.input_hash
    );
    let both = json!({"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}, "design": {"groups": [0]},
        "births": [{"instant": NOON_2026, "latitudeDeg": 27.7, "longitudeDeg": 85.3,
                    "utcOffsetSeconds": 20_700, "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}}]});
    let refused = ResearchRequest::from_json(&both.to_string()).unwrap_err();
    assert_eq!(refused.field(), Some("research.births[0].zone"));
}

// ---- the partner a match and a synastry read ------------------------------

fn partner(clock: &str) -> String {
    format!(
        r#"{{"instant": {NOON_1985}, "place": {{"latitude": 27.7172, "longitude": 85.324, "altitude": 1400}}{clock}}}"#
    )
}

#[test]
fn a_partner_on_a_zone_is_read_on_the_birth_s_offset() {
    let sdk = context();
    let charts = [sdk
        .chart()
        .reading(
            at(NOON_2026),
            &ChartRequest::at(kathmandu(), offset(20_700)),
        )
        .unwrap()
        .value];
    let matched = |clock: &str| {
        let asked = PartnerMatching::from_json(&format!(
            r#"{{"partner": {}, "partnerRole": "BRIDE"}}"#,
            partner(clock)
        ))
        .unwrap();
        sdk.chart().matching_with(&charts, &asked).unwrap()
    };
    assert_eq!(
        matched(&format!(", {ZONE}")),
        matched(r#", "utcOffsetSeconds": 19800"#)
    );
    let both = partner(&format!(r#", "utcOffsetSeconds": 19800, {ZONE}"#));
    let refused =
        PartnerMatching::from_json(&format!(r#"{{"partner": {both}, "partnerRole": "BRIDE"}}"#))
            .unwrap_err();
    assert_eq!(refused.field(), Some("matching.partner.zone"));
    let refused = PartnerSynastry::from_json(&format!(r#"{{"partner": {both}}}"#)).unwrap_err();
    assert_eq!(refused.field(), Some("synastry.partner.zone"));
    // A Davison birth means two stated clocks, so a zone's partner is fixed
    // to the offset it had first.
    let zoned = teistro::Partner::in_zone(at(NOON_1985), kathmandu(), nepal());
    let fixed = zoned.fixed(&sdk).unwrap();
    assert_eq!(fixed.utc_offset, Some(offset(19_800)));
    assert_eq!(zoned.davison(&fixed).unwrap_err().field(), Some("zone"));
}

// ---- the almanac's range requests: the days on the zone ---------------------

/// Each year asked and the offset Nepal kept through the months read.
const YEARS: [(i32, i32); 2] = [(1985, 19_800), (2026, 20_700)];

/// A request on a zone answers what the offset Nepal kept then answers,
/// and seals the zone into what it was asked.
fn twins<T: PartialEq + std::fmt::Debug>(read: impl Fn(i32, Option<i32>) -> Envelope<T>) {
    for (year, seconds) in YEARS {
        let zoned = read(year, None);
        let stated_offset = read(year, Some(seconds));
        assert_eq!(zoned.value, stated_offset.value, "{year}");
        assert_ne!(
            zoned.provenance.input_hash, stated_offset.provenance.input_hash,
            "{year}: the zone is sealed"
        );
    }
}

/// A day of `year` as a record writes it.
fn day(year: i32, month: u8, day: u8) -> String {
    format!(r#"{{"year": {year}, "month": {month}, "day": {day}}}"#)
}

/// The same request with each clock field a record carries replaced.
fn clocked(record: &str, seconds: Option<i32>) -> String {
    record.replace("CLOCK", &clock(seconds))
}

fn sentences_record(year: i32) -> String {
    format!(r#"{{"date": {}, {PLACE}, CLOCK}}"#, day(year, 6, 1))
}

#[test]
fn sentences_on_a_zone_read_the_day_on_its_own_offset() {
    let sdk = context();
    twins(|year, seconds| {
        let asked = SentencesRequest::from_json(&clocked(&sentences_record(year), seconds));
        sdk.almanac().sentences(&asked.unwrap()).unwrap()
    });
    refuses_both(
        SentencesRequest::from_json,
        &sentences_record(2026),
        "sentences",
    );
}

fn next_record(jd: f64) -> String {
    format!(r#"{{"after": {jd}, {PLACE}, CLOCK, "kind": "EKADASHI"}}"#)
}

#[test]
fn the_next_occurrence_on_a_zone_is_read_on_its_own_offset() {
    let sdk = context();
    twins(|year, seconds| {
        let after = if year == 1985 { NOON_1985 } else { NOON_2026 };
        let asked = NextRequest::from_json(&clocked(&next_record(after), seconds));
        sdk.almanac().next(&asked.unwrap()).unwrap()
    });
    refuses_both(NextRequest::from_json, &next_record(NOON_2026), "next");
}

fn elect_record(first: &str, last: &str) -> String {
    format!(
        r#"{{"first": {first}, "last": {last}, {PLACE}, CLOCK,
            "muhurta": {{"rules": "RAMAN_MARRIAGE", "most": 5}}}}"#
    )
}

#[test]
fn an_election_on_a_zone_searches_on_the_range_s_offset() {
    let sdk = context();
    twins(|year, seconds| {
        let record = elect_record(&day(year, 11, 20), &day(year, 11, 30));
        let asked = ElectRequest::from_json(&clocked(&record, seconds)).unwrap();
        sdk.almanac().elect(&asked).unwrap()
    });
    // A search reckons on one offset, so a range across Nepal's change
    // is refused rather than searched on either (C1252).
    let across = elect_record(&day(1985, 12, 25), &day(1986, 1, 5));
    let asked = ElectRequest::from_json(&clocked(&across, None)).unwrap();
    assert!(sdk.almanac().elect(&asked).is_err());
    refuses_both(
        ElectRequest::from_json,
        &elect_record(&day(2026, 11, 20), &day(2026, 11, 30)),
        "elect",
    );
}

fn people_record(year: i32) -> String {
    format!(
        r#"{{"day": {}, {PLACE}, CLOCK,
            "people": [{{"name": "Asha", "instant": {NOON_1985}, {PLACE}, BIRTH}}]}}"#,
        day(year, 6, 1)
    )
}

#[test]
fn a_people_digest_on_a_zone_reads_the_day_and_each_birth_on_its_own_offset() {
    let sdk = context();
    twins(|year, seconds| {
        // The birth is in 1985 either way: +05:30 stated, or the zone.
        let birth = clock(seconds.map(|_| 19_800));
        let record = clocked(&people_record(year), seconds).replace("BIRTH", &birth);
        sdk.almanac()
            .people(&PeopleRequest::from_json(&record).unwrap())
            .unwrap()
    });
    let day_both = people_record(2026).replace("BIRTH", &clock(Some(20_700)));
    refuses_both(PeopleRequest::from_json, &day_both, "people");
    let birth_both = people_record(2026)
        .replacen("CLOCK", &clock(Some(20_700)), 1)
        .replace("BIRTH", "CLOCK");
    refuses_both(PeopleRequest::from_json, &birth_both, "people.people[0]");
}

fn anniversaries_record(year: i32) -> String {
    format!(
        r#"{{"fromYear": {year}, "years": 1, {PLACE}, CLOCK, "anniversaries": [{{"key": "GRANDFATHER",
            "origin": {{"by": "LUNAR", "month": "KARTIKA", "tithi": "SHUKLA_DASHAMI"}}}}]}}"#
    )
}

#[test]
fn anniversaries_on_a_zone_read_each_year_on_its_own_offset() {
    let sdk = context();
    twins(|year, seconds| {
        let asked = AnniversariesRequest::from_json(&clocked(&anniversaries_record(year), seconds));
        sdk.almanac().anniversaries(&asked.unwrap()).unwrap()
    });
    refuses_both(
        AnniversariesRequest::from_json,
        &anniversaries_record(2026),
        "anniversaries",
    );
}

fn observance_record(year: i32) -> String {
    format!(
        r#"{{"first": {}, "last": {}, {PLACE}, CLOCK, "traditions": [
            {{"name": "smarta", "festivals": {{"rules": "DHARMASINDHU"}}, "rule": "EKADASHI_SMARTA"}}]}}"#,
        day(year, 11, 1),
        day(year, 11, 30)
    )
}

#[test]
fn an_observance_on_a_zone_reads_the_days_on_their_own_offset() {
    let sdk = context();
    twins(|year, seconds| {
        let asked = ObservanceRequest::from_json(&clocked(&observance_record(year), seconds));
        sdk.almanac().observance(&asked.unwrap(), None).unwrap()
    });
    refuses_both(
        ObservanceRequest::from_json,
        &observance_record(2026),
        "observance",
    );
}

fn reasons_record(year: i32) -> String {
    format!(
        r#"{{"first": {}, "last": {}, {PLACE}, CLOCK, "festivals": {{"rules": "NEPAL"}},
            "places": [{{"name": "Kathmandu", {PLACE}, CLOCK}}]}}"#,
        day(year, 10, 1),
        day(year, 10, 31)
    )
}

#[test]
fn published_reasons_on_an_offset_answer_as_before() {
    let sdk = context();
    let asked = PublishedReasonsRequest::from_json(&clocked(&reasons_record(2026), Some(20_700)));
    let asked = asked.unwrap();
    let sent = sdk
        .almanac()
        .published_reasons_request(&[], &asked)
        .unwrap();
    let direct = sdk
        .almanac()
        .published_reasons(
            (&asked.from, &asked.to),
            (&kathmandu(), offset(20_700)),
            &asked.festivals,
            &asked.places,
            &[],
        )
        .unwrap();
    same_bytes(&sent, direct.value.in_full().unwrap());
}

#[test]
fn published_reasons_on_a_zone_read_the_days_on_their_own_offset() {
    let sdk = context();
    twins(|year, seconds| {
        let asked = PublishedReasonsRequest::from_json(&clocked(&reasons_record(year), seconds));
        sdk.almanac()
            .published_reasons_request(&[], &asked.unwrap())
            .unwrap()
    });
    let record = reasons_record(2026);
    refuses_both(
        PublishedReasonsRequest::from_json,
        &record.replacen("CLOCK", &clock(Some(20_700)), 1),
        "publishedReasons.places[0]",
    );
    let main = record
        .replacen("CLOCK", "MAIN", 1)
        .replace("CLOCK", &clock(Some(20_700)));
    refuses_both(
        PublishedReasonsRequest::from_json,
        &main.replace("MAIN", "CLOCK"),
        "publishedReasons",
    );
}

/// The feed's stamp: noon UTC on 1 June of `year`, in the year read
/// so a zone and the offset of then stamp it alike.
const fn stamp(year: i32) -> i64 {
    if year == 1985 {
        486_475_200
    } else {
        1_780_315_200
    }
}

fn feed_record(year: i32) -> String {
    format!(
        r#"{{"first": {}, "last": {}, {PLACE}, CLOCK, "festivals": {{"rules": "NEPAL"}},
            "stampUnixSeconds": {}, "name": "Nepal", "refreshMinutes": 0}}"#,
        day(year, 10, 1),
        day(year, 10, 31),
        stamp(year)
    )
}

#[test]
fn a_festival_calendar_on_an_offset_answers_as_before() {
    let sdk = context();
    let asked = FestivalCalendarRequest::from_json(&clocked(&feed_record(2026), Some(20_700)));
    let asked = asked.unwrap();
    let sent = sdk.almanac().festival_calendar_request(&asked).unwrap();
    let direct = sdk
        .almanac()
        .festival_calendar(
            &asked.from,
            &asked.to,
            &kathmandu(),
            offset(20_700),
            &asked.festivals,
            &asked.feed,
        )
        .unwrap();
    assert_eq!(sent, direct);
}

#[test]
fn a_festival_calendar_on_a_zone_reads_the_days_on_their_own_offset() {
    let sdk = context();
    twins(|year, seconds| {
        let asked = FestivalCalendarRequest::from_json(&clocked(&feed_record(year), seconds));
        sdk.almanac()
            .festival_calendar_request(&asked.unwrap())
            .unwrap()
    });
    refuses_both(
        FestivalCalendarRequest::from_json,
        &feed_record(2026),
        "festivalCalendar",
    );
}

fn disagreement_record(year: i32) -> String {
    format!(
        r#"{{"first": {}, "last": {}, {PLACE}, CLOCK, "days": true, "festivals": {{"rules": "NEPAL"}}}}"#,
        day(year, 10, 1),
        day(year, 10, 31)
    )
}

/// The other almanac a comparison reads beside Nepal's: the default one.
fn other() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

#[test]
fn disagreements_on_an_offset_answer_as_before() {
    let (sdk, other) = (context(), other());
    let asked = DisagreementRequest::from_json(&clocked(&disagreement_record(2026), Some(20_700)));
    let asked = asked.unwrap();
    let sent = sdk.almanac().disagreements_request(&other, &asked).unwrap();
    let direct = sdk
        .almanac()
        .disagreements(&other, &asked.from, &asked.to, &kathmandu(), offset(20_700))
        .unwrap();
    same_bytes(&sent.days.unwrap(), direct.value.in_full().unwrap());
}

#[test]
fn disagreements_on_a_zone_read_both_almanacs_on_their_own_offset() {
    let (sdk, other) = (context(), other());
    let read = |year: i32, seconds: Option<i32>| {
        let asked = DisagreementRequest::from_json(&clocked(&disagreement_record(year), seconds));
        sdk.almanac()
            .disagreements_request(&other, &asked.unwrap())
            .unwrap()
    };
    twins(|year, seconds| read(year, seconds).days.unwrap());
    twins(|year, seconds| read(year, seconds).festivals.unwrap());
    refuses_both(
        DisagreementRequest::from_json,
        &disagreement_record(2026),
        "disagreements",
    );
}

/// A zone's days with the clock each says set aside, each that says one
/// having said `seconds`: a day asked by a bare offset leaves its clock out
/// (§3.4), and a search on one offset reads its days on that offset.
fn unclocked(days: &[teistro::Panchanga], seconds: i32) -> Vec<teistro::Panchanga> {
    days.iter()
        .map(|day| {
            let mut day = day.clone();
            if let Some(clock) = day.clock.take() {
                assert_eq!(clock.offset.seconds(), seconds);
            }
            day
        })
        .collect()
}

/// October of `year` as a range.
fn october(year: i32) -> (CalendarDate, CalendarDate) {
    let gregorian =
        |day| CalendarDate::defined(teistro::catalogue::Calendar::Gregorian, year, 10, day);
    (gregorian(1), gregorian(31))
}

#[test]
fn festival_and_muhurta_days_on_a_zone_read_each_day_on_its_own_offset() {
    let sdk = context();
    let festivals = FestivalRequest::from_json(r#"{"rules": "NEPAL"}"#).unwrap();
    let muhurta = MuhurtaRequest::from_json(r#"{"rules": "RAMAN_MARRIAGE", "most": 5}"#).unwrap();
    for (year, seconds) in YEARS {
        let (from, to) = october(year);
        let read = |clock: DayClock| {
            let almanac = sdk.almanac();
            (
                almanac
                    .festivals_with_days(&from, &to, &kathmandu(), clock.clone(), &festivals)
                    .unwrap(),
                almanac
                    .muhurta_with_days(&from, &to, &kathmandu(), clock, &muhurta)
                    .unwrap(),
            )
        };
        let (zoned, stated_offset) = (read(DayClock::Zone(nepal())), read(offset(seconds).into()));
        assert_eq!(
            unclocked(&zoned.0.days.value, seconds),
            stated_offset.0.days.value,
            "{year}"
        );
        assert_eq!(zoned.0.answer.value, stated_offset.0.answer.value, "{year}");
        assert_eq!(
            unclocked(&zoned.1.days.value, seconds),
            stated_offset.1.days.value,
            "{year}"
        );
        assert_eq!(zoned.1.answer.value, stated_offset.1.answer.value, "{year}");
    }
    // The search reckons on one offset: a range across the change is refused.
    let from = CalendarDate::defined(teistro::catalogue::Calendar::Gregorian, 1985, 12, 25);
    let to = CalendarDate::defined(teistro::catalogue::Calendar::Gregorian, 1986, 1, 5);
    let across = sdk.almanac().muhurta_with_days(
        &from,
        &to,
        &kathmandu(),
        DayClock::Zone(nepal()),
        &muhurta,
    );
    assert!(across.is_err());
}

#[test]
fn festival_days_on_an_offset_answer_as_the_festivals_do() {
    let sdk = context();
    let festivals = FestivalRequest::from_json(r#"{"rules": "NEPAL"}"#).unwrap();
    let (from, to) = october(2026);
    let with_days = sdk
        .almanac()
        .festivals_with_days(&from, &to, &kathmandu(), offset(20_700), &festivals)
        .unwrap();
    let alone = sdk
        .almanac()
        .festivals(&from, &to, &kathmandu(), offset(20_700), &festivals)
        .unwrap();
    assert_eq!(with_days.answer, alone);
}

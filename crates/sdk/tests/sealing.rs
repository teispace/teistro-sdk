//! Every envelope the façade publishes carries the hash of its own value,
//! and a batch's members can be stamped with theirs.
//!
//! A batch's provenance hashes the **list**, so a member handed out alone —
//! a binding's `found(one)`, one day of an almanac — carries a hash that is
//! not its value's unless it is re-stamped. `interpreted` and
//! `almanac().of_each` answer each member's own hash from the serialisation
//! that seals the batch; these hold them to the hash the one-of call seals,
//! and the batch's to the list's (`03-design/serial-and-the-envelope.md` §3).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, ChartKind};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    CalendarDate, ChartRequest, Context, Ephemeris, PlanRequest, RuleRequest, ShippedRules,
    UtcOffset, content_hash,
};

fn context() -> Context {
    Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a context")
}

fn place() -> Place {
    Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    )
}

fn offset() -> UtcOffset {
    UtcOffset::try_from_seconds(20_700).unwrap()
}

const INSTANTS: [f64; 3] = [2_447_995.489_583_333_5, 2_448_000.5, 2_451_545.0];

fn instants() -> Vec<JulianDay<Utc>> {
    INSTANTS.map(JulianDay::<Utc>::literal).to_vec()
}

/// A batch of charts: the list's hash on the batch, each chart's own on
/// the chart — the hash `reading(one)` seals over that chart alone —
/// with and without rules, whose answers are part of the value hashed.
#[test]
fn an_interpreted_chart_carries_its_own_hash_and_the_batch_the_lists() {
    let sdk = context();
    let request = ChartRequest::at(place(), offset());
    let read = sdk
        .chart()
        .interpreted(&instants(), &request, None, PlanRequest::default())
        .unwrap();
    let documents: Vec<_> = read.value.iter().map(|one| &one.document).collect();
    assert_eq!(read.provenance.content_hash, content_hash(&documents));
    for (at, chart) in instants().into_iter().zip(&read.value) {
        assert_eq!(chart.content_hash, content_hash(&chart.document));
        let alone = sdk.chart().reading(at, &request).unwrap();
        assert_eq!(alone.provenance.content_hash, chart.content_hash);
    }

    let set = RuleRequest::shipped([ShippedRules::Nabhasas])
        .rule_set()
        .unwrap();
    let ruled = sdk
        .chart()
        .interpreted(&instants(), &request, Some(&set), PlanRequest::default())
        .unwrap();
    let pairs: Vec<_> = ruled
        .value
        .iter()
        .map(|one| (&one.document, one.reading.as_ref().unwrap()))
        .collect();
    assert_eq!(ruled.provenance.content_hash, content_hash(&pairs));
    for (chart, pair) in ruled.value.iter().zip(&pairs) {
        assert_eq!(chart.content_hash, content_hash(pair));
    }
    // The same batch, sealed by the call that returns it whole.
    let whole = sdk
        .chart()
        .readings_with_rules(
            &instants(),
            &request.clone().with_rule_inputs(set.rules()),
            &set,
        )
        .unwrap();
    assert_eq!(whole.provenance.content_hash, content_hash(&whole.value));
}

/// A founded chart and a batch of them: each sealed over what it holds.
#[test]
fn a_founded_chart_is_sealed_over_itself_and_a_batch_over_its_list() {
    let sdk = context();
    let many = sdk
        .chart()
        .found_many(&instants(), &place(), offset(), ChartKind::Natal)
        .unwrap();
    assert_eq!(many.provenance.content_hash, content_hash(&many.value));
    let one = sdk
        .chart()
        .found(instants()[0], &place(), offset(), ChartKind::Natal)
        .unwrap();
    assert_eq!(one.provenance.content_hash, content_hash(&one.value));
    assert_eq!(one.value, many.value[0]);
    assert_ne!(one.provenance.content_hash, many.provenance.content_hash);
}

/// A range of days: the list's hash on the range, each day's own beside
/// it — the hash `day` seals over that day alone.
#[test]
fn a_day_of_a_range_carries_its_own_hash() {
    let sdk = context();
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 4, 13);
    let to = CalendarDate::defined(Calendar::Gregorian, 2024, 4, 15);
    let (range, each) = sdk
        .almanac()
        .of_each(&from, &to, &place(), offset())
        .unwrap();
    let whole = sdk.almanac().of(&from, &to, &place(), offset()).unwrap();
    assert_eq!(range.provenance.content_hash, whole.provenance.content_hash);
    assert_eq!(range.provenance.content_hash, content_hash(&range.value));
    assert_eq!(each.len(), 3);
    let second = CalendarDate::defined(Calendar::Gregorian, 2024, 4, 14);
    let day = sdk.almanac().day(&second, &place(), offset()).unwrap();
    assert_eq!(day.provenance.content_hash, each[1]);
    assert_eq!(each[1], content_hash(&range.value[1]));
}

/// The widest document the SDK produces — every section, every varga,
/// and the inputs of every shipped rule, which ask for varga schemes of
/// their own — serialises, so its hash is of its bytes and not of nothing.
///
/// Two varga schemes serde could not write (`Map::Listed` and
/// `Spans::Degrees`, tagged newtypes holding a list) sealed every document
/// carrying them with the hash of the empty string, and a batch of charts
/// read with rules carried that hash: the canonical writer swallowed the
/// error. It no longer does in a debug build, and this reaches every
/// scheme so that a type added the same way stops here.
#[test]
fn the_widest_document_serialises_whole() {
    let sdk = context();
    let set = RuleRequest::shipped(ShippedRules::ALL).rule_set().unwrap();
    let request = ChartRequest::at(place(), offset())
        .with_everything()
        .with_rule_inputs(set.rules());
    let read = sdk
        .chart()
        .readings_with_rules(&instants(), &request, &set)
        .unwrap();
    for (document, reading) in &read.value {
        serde_json::to_value(document).expect("the document serialises");
        serde_json::to_value(reading).expect("its rules' answer serialises");
    }
    let empty = teistro::Hash::of(b"");
    assert_ne!(read.provenance.content_hash, empty);
}

//! The Rust surface against the facts every other binding asserts.
//!
//! The point of these is not that the crates work — each has its own
//! tests — but that **this composition** gives the same answers as the
//! one at the C boundary. Where a fact is already asserted by
//! `bindings/c/tests/smoke.c`, it is asserted here in the same words, so
//! a difference between the two compositions is a failing test rather
//! than something a reader has to notice.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::{Calendar, ChartKind, Era};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    Body, CalendarDate, Context, Ephemeris, Frame, PositionRequest, Scale, TimeScale, UtcOffset,
};
use teistro_core::envelope::CalendarResolution;

/// A context on the profile every binding's quickstart names.
fn context() -> Context {
    Context::builder()
        .profile("nepali-default")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the shipped profile, locale and built-in ephemeris")
}

#[test]
fn a_date_converts_into_bikram_sambat() {
    let sdk = context();
    // The C smoke test's own fact, in its own words: 14 April 2015 is
    // 1 Baisakh 2072 BS, in the Vikrama era, inside the official table.
    let gregorian = CalendarDate::defined(Calendar::Gregorian, 2015, 4, 14);
    let bs = sdk
        .calendar()
        .convert(&gregorian, Calendar::BikramSambat)
        .expect("a date inside the table");
    assert_eq!((bs.year, bs.month, bs.day), (2072, 1, 1));
    let era = bs.era.expect("Bikram Sambat has an era");
    assert_eq!((era.era, era.year), (Era::Vikrama, 2072));
    assert!(
        matches!(bs.resolution, CalendarResolution::Tabular { .. }),
        "inside the official table, not computed: {:?}",
        bs.resolution
    );
}

#[test]
fn the_calendar_area_is_a_value_that_can_be_held() {
    let sdk = context();
    // What `const cal = ctx.calendar` is in Node: read once, used many
    // times, and the context still reachable from it.
    let calendar = sdk.calendar();
    let day = CalendarDate::defined(Calendar::Gregorian, 2015, 4, 14);
    let fixed = calendar.fixed_of(&day).expect("a Gregorian date");
    let back = calendar
        .date_of(Calendar::Gregorian, fixed)
        .expect("back again");
    assert_eq!(
        (back.year, back.month, back.day),
        (day.year, day.month, day.day)
    );
    // A date **read** from a fixed day carries the era the calendar
    // knows; one a consumer wrote does not, because they did not say.
    // The round trip therefore enriches rather than round-trips, which
    // is worth asserting rather than papering over.
    assert!(day.era.is_none());
    assert_eq!(
        back.era.map(|era| (era.era, era.year)),
        Some((Era::CommonEra, 2015))
    );
    assert_eq!(calendar.month_length(Calendar::Gregorian, 2024, 2), Ok(29));
    assert_eq!(calendar.is_leap(Calendar::Gregorian, 2024), Ok(true));
    assert_eq!(calendar.context().profile(), sdk.profile());
}

#[test]
fn a_calendar_the_sdk_does_not_ship_yet_is_refused_by_name() {
    let sdk = context();
    let refusal = sdk
        .calendar()
        .month_length(Calendar::IndianLunisolar, 1946, 1)
        .expect_err("a calendar the SDK does not ship");
    assert_eq!(refusal.field(), Some("calendar"));
    assert!(refusal.to_string().contains("does not ship"), "{refusal}");
}

#[test]
fn the_default_profile_needs_no_naming() {
    let sdk = Context::builder().build().expect("every default");
    assert!(!sdk.profile().is_empty());
    // No ephemeris named is no ephemeris, which is a decision and not a
    // default: ADR-0029 refuses a context quietly given one.
    assert!(sdk.ephemeris().is_none());
}

#[test]
fn an_unknown_profile_is_refused_with_the_shipped_ones() {
    let refusal = Context::builder()
        .profile("no-such-profile")
        .build()
        .expect_err("no shipped profile");
    assert_eq!(refusal.field(), Some("profile"));
    assert!(
        refusal
            .hint()
            .is_some_and(|hint| hint.contains("nepali-default")),
        "{refusal:?}"
    );
}

#[test]
fn an_unknown_locale_is_refused_with_the_loaded_ones() {
    let refusal = Context::builder()
        .locale("xx-Zzzz-ZZ")
        .build()
        .expect_err("no such locale");
    assert_eq!(refusal.field(), Some("locale"));
    assert!(
        refusal
            .hint()
            .is_some_and(|hint| hint.contains("ne-Deva-NP")),
        "{refusal:?}"
    );
}

#[test]
fn two_contexts_with_the_same_settings_share_a_hash() {
    let one = context();
    let two = context();
    assert_eq!(one.settings_hash(), two.settings_hash());
    let other = Context::builder()
        .profile("parashari-classical")
        .build()
        .expect("a shipped profile");
    assert_ne!(one.settings_hash(), other.settings_hash());
}

#[test]
fn a_chain_of_one_keeps_its_own_refusal() {
    // With one entry there is no next entry, so a refusal is the
    // refusal rather than "nothing could be opened". Proved on the
    // profile rather than the ephemeris, because a bad profile is not
    // an ephemeris failure and is what caught this in Dart.
    let refusal = Context::builder()
        .profile("no-such-profile")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect_err("no shipped profile");
    assert_eq!(refusal.field(), Some("profile"));
}

#[test]
fn a_chain_falls_back_in_order_and_says_what_it_opened() {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Test, Ephemeris::Builtin])
        .build()
        .expect("the first entry opens");
    let opened = sdk.ephemeris().expect("an ephemeris");
    assert_eq!(opened.capabilities().identity.name, "test-provider");
}

#[test]
fn a_chain_falls_back_from_an_entry_that_could_not_open() {
    // What a chain is *for*, and what it could not do until
    // `Ephemeris::opening` existed: an entry that fails, and a later one
    // that answers. Without a recipe an entry is an already-built
    // provider, which cannot fail, so every chain succeeded on its first
    // entry and the ordering was decoration.
    let sdk = Context::builder()
        .ephemeris([
            Ephemeris::opening("an-engine-that-is-not-there", || {
                Err(teistro::Error::unsupported(
                    "the ephemeris data is not where the index says",
                ))
            }),
            Ephemeris::Test,
        ])
        .build()
        .expect("the second entry opens");
    assert_eq!(
        sdk.ephemeris()
            .expect("an ephemeris")
            .capabilities()
            .identity
            .name,
        "test-provider"
    );
}

#[test]
fn a_chain_that_opens_nothing_names_every_entry_that_failed() {
    let refusal = Context::builder()
        .ephemeris([
            Ephemeris::opening("first", || Err(teistro::Error::unsupported("no data"))),
            Ephemeris::opening("second", || Err(teistro::Error::unsupported("no library"))),
        ])
        .build()
        .expect_err("nothing opens");
    assert_eq!(refusal.field(), Some("ephemeris"));
    let said = refusal.to_string();
    assert!(said.contains("first: "), "{said}");
    assert!(said.contains("second: "), "{said}");
}

/// A worker builds its own context, which is the pattern a `!Send`
/// context leaves and the one every other binding states.
///
/// Asserted rather than left to a doc comment, because a Rust consumer
/// putting one in an `axum` app state finds out the hard way otherwise.
/// The reason is the locale engine and not the ephemeris: the port
/// requires `Send + Sync` of a provider, while `teistro_intl`'s plural
/// rules hold an `Rc`-backed icu4x payload.
#[test]
fn a_worker_builds_its_own_context() {
    let answers: Vec<(i32, u8, u8)> = (0..4)
        .map(|_| {
            std::thread::spawn(|| {
                let sdk = context();
                let day = CalendarDate::defined(Calendar::Gregorian, 2015, 4, 14);
                let bs = sdk
                    .calendar()
                    .convert(&day, Calendar::BikramSambat)
                    .expect("a date inside the table");
                (bs.year, bs.month, bs.day)
            })
        })
        .map(|worker| worker.join().expect("the worker finished"))
        .collect();
    assert_eq!(answers, vec![(2072, 1, 1); 4]);
}

#[test]
fn a_nepali_birth_time_resolves() {
    let sdk = context();
    // The C smoke test's own fact: 00:20 on 1 January 1986 in Kathmandu
    // is +05:45, the offset that began that midnight, under the zone's
    // current rules and with no warning.
    let civil = teistro::CivilDateTime::at(
        CalendarDate::defined(Calendar::Gregorian, 1986, 1, 1),
        teistro::CivilTime::new(0, 20, 0).expect("a time of day"),
    );
    let zone = teistro::ZoneSpec::Iana {
        zone: String::from("Asia/Kathmandu"),
    };
    let resolved = sdk.time().resolve(&civil, &zone).expect("a known zone");
    assert_eq!(resolved.zone.offset.seconds(), 20700);
    assert!(
        resolved.zone.warnings.is_empty(),
        "{:?}",
        resolved.zone.warnings
    );
    assert!(!resolved.zone.tzdb_version.is_empty());

    // And back: the instant read as the civil clock in that zone.
    let (back, _) = sdk
        .time()
        .civil_of(resolved.instant, &zone, Calendar::Gregorian)
        .expect("a known zone");
    assert_eq!(
        (back.date.year, back.date.month, back.date.day),
        (1986, 1, 1)
    );
    let time = back.time.expect("the time is known");
    assert_eq!((time.hour(), time.minute()), (0, 20));
}

#[test]
fn a_scale_conversion_reports_what_it_applied() {
    let sdk = context();
    let at_j2000 = sdk
        .time()
        .convert(2_451_545.0, Scale::Ut1, Scale::Tt)
        .expect("inside the model's range");
    // Never only the number: ΔT at J2000 is about 64 seconds, and which
    // model said so is what a cache key and an audit trail are made of.
    let applied = at_j2000.delta_t.expect("UT1 to TT needs a ΔT");
    assert!(
        (60.0..70.0).contains(&applied.seconds),
        "ΔT at J2000 is about 64 s, not {}",
        applied.seconds
    );
    assert!(at_j2000.jd > 2_451_545.0);

    // A scale to itself applies nothing and says so, rather than making
    // a caller who converts by a run-time value special-case it.
    let same = sdk
        .time()
        .convert(2_451_545.0, Scale::Tt, Scale::Tt)
        .expect("the identity");
    // Bit for bit, which is what "applies nothing" has to mean: a
    // conversion that rounded the identity would be a conversion.
    assert_eq!(same.jd.to_bits(), 2_451_545.0_f64.to_bits());
    assert!(same.delta_t.is_none());

    // And the round trip comes back.
    let there = sdk
        .time()
        .convert(2_451_545.0, Scale::Utc, Scale::Tt)
        .expect("forward");
    let back = sdk
        .time()
        .convert(there.jd, Scale::Tt, Scale::Utc)
        .expect("back");
    assert!(
        (back.jd - 2_451_545.0).abs() < 1e-9,
        "{} != 2451545",
        back.jd
    );
}

#[test]
fn delta_t_carries_its_source_and_its_model() {
    let sdk = context();
    let value = sdk
        .time()
        .delta_t(teistro::quantity::JulianDay::try_new(2_451_545.0).expect("a Julian day"))
        .expect("inside the model's range");
    assert!((60.0..70.0).contains(&value.seconds), "{}", value.seconds);
    assert!(!value.model.key().is_empty());
}

#[test]
fn the_locale_renders_a_message_by_its_typed_accessor() {
    let sdk = context();
    // What the C smoke test prints, and what every binding's quickstart
    // shows: the reason a graha is in a bhava, in Nepali.
    assert_eq!(sdk.intl().locale(), "ne-Deva-NP");
    let rendered = sdk
        .intl()
        .render_typed(&teistro::messages::sdk::reason::GrahaInBhava {
            graha: teistro::catalogue::Graha::Jupiter,
            bhava: 7,
        });
    assert!(
        rendered.text.contains("गुरु"),
        "the Nepali for Jupiter: {}",
        rendered.text
    );
    assert!(sdk.intl().has("sdk.reason.grahaInBhava"));
}

#[test]
fn an_entity_carries_its_forms_in_the_locale() {
    let sdk = context();
    let sun = sdk.intl().entity("graha.SUN").expect("a catalogued graha");
    assert!(!sun.forms.is_empty(), "{sun:?}");

    let refusal = sdk
        .intl()
        .entity("graha.NOT_A_GRAHA")
        .expect_err("no such entity");
    assert_eq!(refusal.field(), Some("key"));
}

#[test]
fn a_key_resolves_to_its_id_and_back() {
    let sdk = context();
    let id = sdk.keys().id("graha.SUN").expect("a catalogued key");
    assert_eq!(sdk.keys().name(id).expect("a live id"), "graha.SUN");

    let refusal = sdk.keys().id("graha.SUNN").expect_err("no such key");
    assert!(
        refusal.hint().is_some_and(|hint| hint.contains("SUN")),
        "{refusal:?}"
    );
}

#[test]
fn the_canonical_frame_packs_and_unpacks() {
    let sdk = context();
    let canonical = sdk.frame().canonical();
    let bits = sdk.frame().pack(canonical);
    assert_eq!(sdk.frame().unpack(bits).expect("its own bits"), canonical);
}

#[test]
fn positions_answer_in_the_frame_asked_for() {
    let sdk = context();
    let jds = [2_451_545.0];
    let bodies = [Body::Sun, Body::Moon];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
    let sky = sdk.positions(&request).expect("the built-in ephemeris");

    // The answer is the astronomy crate's own type: a Rust consumer
    // reads a `Longitude` off it where every other binding decodes a
    // result blob to get the same number back as a double.
    assert_eq!((sky.columns.jd_count, sky.columns.body_count), (1, 2));
    let sun = sky.columns.at(0, 0).expect("a cell");
    // The Sun at J2000 is in Capricorn, tropical, near 280°.
    assert!(
        (279.0..282.0).contains(&sun.lon),
        "the Sun at J2000 is near 280°, not {}",
        sun.lon
    );
    // And what was applied to get there, which is half the answer.
    assert!(!sky.steps.is_empty());
    assert!(
        sky.step_keys()
            .iter()
            .any(|step| step.starts_with("positions:"))
    );
}

#[test]
fn positions_without_an_ephemeris_refuse_by_capability() {
    let sdk = Context::builder().build().expect("every default");
    let jds = [2_451_545.0];
    let bodies = [Body::Sun];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
    let refusal = sdk.positions(&request).expect_err("no ephemeris");
    // The same field and the same hint the C boundary gives, because a
    // consumer who forgot the option wants the same sentence in
    // whichever language they forgot it in.
    assert_eq!(refusal.field(), Some("ephemeris"));
    assert!(
        refusal.hint().is_some_and(|hint| hint.contains("builtin")),
        "{refusal:?}"
    );
}

#[test]
fn the_engine_area_refuses_where_there_is_nothing_to_ask() {
    // The built-in describes no operations of its own -- it is the SDK's
    // own ephemeris, not an engine with a surface -- so `engine` refuses
    // by name rather than answering an empty manifest. A consumer on a
    // real engine gets the manifest; a consumer on the fallback is told
    // which they have.
    let sdk = context();
    let refusal = sdk.engine().manifest().expect_err("the built-in has none");
    assert!(
        refusal.to_string().contains("describes no operations"),
        "{refusal}"
    );

    // And with no ephemeris at all it is the capability refusal, naming
    // the option, not a confusing one about operations.
    let bare = Context::builder().build().expect("every default");
    let refusal = bare.engine().manifest().expect_err("no ephemeris");
    assert_eq!(refusal.field(), Some("ephemeris"));
}

/// A place and an offset the chart and the almanac tests share.
fn kathmandu() -> (teistro::quantity::Place, teistro::UtcOffset) {
    use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    (
        Place::new(
            Latitude::try_new(27.7172).expect("a latitude"),
            Longitude::try_new(85.324).expect("a longitude"),
            Altitude::try_new(1400.0).expect("an altitude"),
        ),
        teistro::UtcOffset::try_from_seconds(20700).expect("+05:45"),
    )
}

#[test]
fn one_chart_is_the_batch_of_one_unwrapped() {
    use teistro::catalogue::ChartKind;
    use teistro::quantity::{JulianDay, Utc};

    let sdk = Context::builder()
        .profile("parashari-classical")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a shipped profile");
    let (place, offset) = kathmandu();
    let instants = [
        JulianDay::<Utc>::literal(2_460_482.5),
        JulianDay::<Utc>::literal(2_460_600.25),
    ];
    let many = sdk
        .chart()
        .found_many(&instants, &place, offset, ChartKind::Natal)
        .expect("the built-in ephemeris");
    assert_eq!(many.value.len(), 2);
    // A lagna is a longitude, and the two instants are months apart, so
    // the second is not the first -- which is what a batch that ran
    // instants the wrong way round would produce.
    let lagnas: Vec<f64> = many.value.iter().map(|chart| chart.lagna_deg).collect();
    let [first, second] = lagnas.as_slice() else {
        panic!("two instants founded two charts, not {}", lagnas.len());
    };
    // Bit for bit, because the assertion is that they are *different*
    // computations rather than that they are far apart.
    assert_ne!(first.to_bits(), second.to_bits());

    let one = sdk
        .chart()
        .found(instants[0], &place, offset, ChartKind::Natal)
        .expect("the built-in ephemeris");
    assert_eq!(
        one.value.lagna_deg.to_bits(),
        first.to_bits(),
        "the batch of one takes the same path as the batch"
    );
    // The envelope carries a content hash rather than the founder's
    // placeholder, as a binding's blob does.
    // ...and it is not the placeholder the founder leaves.
    assert_ne!(
        many.provenance.content_hash,
        teistro::Hash::of(&[]),
        "the founder's placeholder was not replaced"
    );
}

#[test]
fn an_almanac_answers_a_run_of_days_in_one_crossing() {
    use teistro::catalogue::Calendar;

    let sdk = Context::builder()
        .profile("parashari-classical")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a shipped profile");
    let (place, offset) = kathmandu();
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 17);
    let to = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 19);
    let week = sdk
        .almanac()
        .of(&from, &to, &place, offset)
        .expect("the built-in ephemeris");
    assert_eq!(week.value.len(), 3);
    // Consecutive days share a boundary: day n's next sunrise is day
    // n+1's sunrise, which is the reason a run costs less than the days
    // asked for separately.
    for pair in week.value.windows(2) {
        let [earlier, later] = pair else {
            unreachable!("`windows(2)` yields pairs");
        };
        assert_eq!(
            earlier.day.next_sunrise.get().to_bits(),
            later.day.sunrise.get().to_bits()
        );
    }

    let one = sdk
        .almanac()
        .day(&from, &place, offset)
        .expect("the built-in ephemeris");
    assert_eq!(
        one.value.day.sunrise.get().to_bits(),
        week.value
            .first()
            .expect("three days")
            .day
            .sunrise
            .get()
            .to_bits()
    );
}

/// **An envelope carries the hash of its own value.**
///
/// `Provenance::new` leaves the content hash `Hash::of(&[])`, and
/// `03-design/serial-and-the-envelope.md` §2 found why that is a shape
/// problem rather than a bug in a producer: a value and its stamp are
/// built separately and joined at the end, so the one field that cannot
/// be filled until the value exists is the one everybody forgets. §8
/// left "whether the producers should seal" open; they do now, and this
/// is the property that says so.
///
/// It is asserted here rather than in `chart` and `panchanga` because
/// this is the surface a consumer holds, and because all four of the
/// envelopes it can obtain are reachable from one context — including
/// the two **unwrapped** ones, whose hash is of the single value and not
/// of the batch it came from. A `found(one)` claiming the hash of a list
/// of one would be a stamp that describes something else.
#[test]
fn every_envelope_is_sealed_with_the_hash_of_its_own_value() {
    let sdk = Context::builder()
        .profile("parashari-classical")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a shipped profile");
    let place = Place::new(
        Latitude::try_new(27.7172).expect("a latitude"),
        Longitude::try_new(85.324).expect("a longitude"),
        Altitude::try_new(1400.0).expect("an altitude"),
    );
    let offset = UtcOffset::try_from_seconds(20700).expect("+05:45");
    let instants = [
        JulianDay::<Utc>::literal(2_460_482.5),
        JulianDay::<Utc>::literal(2_460_600.25),
    ];
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 17);
    let to = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 18);

    let charts = sdk
        .chart()
        .found_many(&instants, &place, offset, ChartKind::Natal)
        .expect("the built-in ephemeris");
    let chart = sdk
        .chart()
        .found(instants[0], &place, offset, ChartKind::Natal)
        .expect("the built-in ephemeris");
    let week = sdk
        .almanac()
        .of(&from, &to, &place, offset)
        .expect("the built-in ephemeris");
    let day = sdk
        .almanac()
        .day(&from, &place, offset)
        .expect("the built-in ephemeris");

    assert_eq!(
        charts.provenance.content_hash,
        teistro::content_hash(&charts.value)
    );
    assert_eq!(
        chart.provenance.content_hash,
        teistro::content_hash(&chart.value)
    );
    assert_eq!(
        week.provenance.content_hash,
        teistro::content_hash(&week.value)
    );
    assert_eq!(
        day.provenance.content_hash,
        teistro::content_hash(&day.value)
    );

    // And the placeholder is what it is no longer: a hash of nothing.
    assert_ne!(
        charts.provenance.content_hash,
        teistro::Hash::of(&[]),
        "the placeholder `Provenance::new` leaves"
    );
    // The unwrapped one is **not** the batch's, because it is not the
    // batch's value.
    assert_ne!(
        chart.provenance.content_hash,
        charts.provenance.content_hash
    );
    assert_ne!(day.provenance.content_hash, week.provenance.content_hash);
}

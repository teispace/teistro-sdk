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
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, Catalogued, ChartKind, Era, Point, Varga};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    Body, CalendarDate, ChartRequest, Context, Ephemeris, Frame, PositionRequest, Scale, TimeScale,
    UtcOffset,
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

/// **A reading is a founding plus arithmetic**, and every section it
/// asks for is there.
///
/// The assembly `03-design/chart-reading.md` exists for: the document
/// type has been built since `crates/serial` was written and the only
/// code that put one together was an example of that crate, so a
/// consumer could not ask for a divisional chart in any language,
/// including this one.
///
/// What is asserted is the shape rather than the numbers: every
/// section's own values are held by its own crate's baseline tests
/// against the corpus, and repeating them here would be a second copy of
/// a fixture. What no other test can say is that the *façade* asks each
/// producer for the right thing and puts the answer in the right field.
#[test]
fn a_reading_carries_the_sections_it_was_asked_for() {
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
    let instant = JulianDay::<Utc>::literal(2_460_482.5);

    // Nothing but the foundation, which is the default and the reason
    // the default is what it is: a caller who wants a birth chart does
    // not pay for twenty-one divisional charts.
    let bare = sdk
        .chart()
        .reading(instant, &ChartRequest::at(place, offset))
        .expect("the built-in ephemeris");
    assert_eq!(bare.value.sections(), vec!["foundation"]);

    // And everything, which is what a consumer storing a chart wants.
    let whole = sdk
        .chart()
        .reading(instant, &ChartRequest::at(place, offset).with_everything())
        .expect("the built-in ephemeris");
    assert_eq!(
        whole.value.sections(),
        vec![
            "foundation",
            "panchanga",
            "vargas",
            "state",
            "aspects",
            "points",
            "houses",
            "ashtakavarga",
            "vimshopaka",
            "shadbala",
            "dashas"
        ],
        "every section `teistro-serial`'s document declares but the drawings, which are named pairs"
    );

    // The foundation is the same chart either way: the sections are
    // derived from it and none of them can move it.
    assert_eq!(
        bare.value.foundation.lagna_deg.to_bits(),
        whole.value.foundation.lagna_deg.to_bits()
    );

    // Each section, asked of the producer the design page names.
    assert_eq!(whole.value.vargas.len(), Varga::all().len());
    assert_eq!(
        whole.value.state.as_ref().map(Vec::len),
        Some(whole.value.foundation.grahas.len()),
        "one state per graha the foundation carries"
    );
    assert!(
        whole
            .value
            .houses
            .as_ref()
            .is_some_and(|houses| houses.bhava(1).is_some() && houses.bhava(12).is_some()),
        "twelve bhavas, which is why zero rows cannot mean `asked for and empty`"
    );
    assert!(
        whole
            .value
            .points
            .as_ref()
            .is_some_and(|points| !points.all().is_empty()),
        "the upagrahas and the special lagnas"
    );
    assert!(
        whole
            .value
            .aspects
            .as_ref()
            .is_some_and(|aspects| !aspects.all().is_empty()),
        "a chart of nine grahas has drishti in it"
    );
    assert_eq!(
        whole
            .value
            .panchanga
            .as_ref()
            .map(|day| day.day.date.clone()),
        Some(whole.value.foundation.day.day.date.clone()),
        "the almanac of the day the *chart* belongs to, which before sunrise is not the instant's civil date"
    );

    // **Gulika and Mandi are what say the new primitive was threaded
    // through.** They are Saturn's eighth of the day's arc, and the
    // eighth asks for the ascendant at each division — so
    // `Points::from_longitudes`, which is what `crates/serial`'s own
    // sample uses and all that was reachable before this, gives every
    // other point and not these two. Naming them is a sharper assertion
    // than a count: a count would pass on eleven.
    let named: Vec<Point> = whole
        .value
        .points
        .as_ref()
        .map(|points| points.all().iter().map(|found| found.point).collect())
        .unwrap_or_default();
    assert!(named.contains(&Point::Gulika), "{named:?}");
    assert!(named.contains(&Point::Mandi), "{named:?}");

    // A batch of one agrees with the one, bit for bit, as the founding
    // does — and the seal is of *this* document rather than of the list.
    let batch = sdk
        .chart()
        .readings(&[instant], &ChartRequest::at(place, offset).with_houses())
        .expect("the built-in ephemeris");
    assert_eq!(batch.value.len(), 1);
    assert_ne!(batch.provenance.content_hash, whole.provenance.content_hash);
    assert_eq!(
        batch.provenance.content_hash,
        teistro::content_hash(&batch.value)
    );
}

/// A drawing is the founded chart, or one of its divisional charts, placed in
/// a layout; this is what proves the geometry crate's placement is fed the
/// right chart, which its own tests cannot.
#[test]
fn a_reading_draws_the_charts_it_was_asked_for() {
    use teistro::catalogue::ChartLayout;

    let sdk = context();
    let (place, offset) = kathmandu();
    let instant = JulianDay::<Utc>::literal(2_460_482.5);
    let request = ChartRequest::at(place, offset)
        .with_vargas([Varga::D9])
        .with_drawings([
            (ChartLayout::NorthIndian, Varga::D1),
            (ChartLayout::SouthIndian, Varga::D9),
            (ChartLayout::WesternWheel, Varga::D1),
        ]);
    let document = sdk
        .chart()
        .reading(instant, &request)
        .expect("a reading")
        .value;
    assert!(document.sections().contains(&"drawings"));
    assert_eq!(document.drawings.len(), 3);

    // D1 in North Indian: house 1 is the top diamond and shows the lagna's
    // sign; every graha is in the cell of the sign the foundation puts it in.
    let north = &document.drawings[0];
    let foundation = &document.foundation;
    assert_eq!(
        (north.varga, north.placed.layout.as_str()),
        (Varga::D1, "NORTH_INDIAN")
    );
    let first = &north.placed.cells[0];
    assert_eq!(
        (first.house, u16::from(foundation.lagna_sign_index())),
        (1, first.sign.id())
    );
    for position in &foundation.grahas {
        let cell = north
            .placed
            .cells
            .iter()
            .find(|cell| cell.bodies.contains(&position.graha.key_id()))
            .expect("every graha is drawn");
        assert_eq!(
            cell.sign.id(),
            u16::from(position.sign_index()),
            "{:?}",
            position.graha
        );
    }

    // D9 in South Indian: the navamsha's own lagna, and its grahas by their
    // navamsha signs, which is the divisional chart the document carries.
    let navamsha = &document.vargas[0];
    let south = &document.drawings[1];
    assert_eq!(south.varga, Varga::D9);
    let lagna_cell = south
        .placed
        .cells
        .iter()
        .find(|cell| cell.lagna)
        .expect("a lagna");
    assert_eq!(
        (lagna_cell.sign, lagna_cell.house),
        (navamsha.lagna.sign, 1)
    );
    for placed in &navamsha.grahas {
        let cell = south
            .placed
            .cells
            .iter()
            .find(|cell| cell.bodies.contains(&placed.graha.key_id()))
            .expect("every graha is drawn");
        assert_eq!(cell.sign, placed.at.sign, "{:?}", placed.graha);
    }

    // The wheel: each graha in the house the foundation's own bhavas give it,
    // and a mark for each at its degree.
    let wheel = &document.drawings[2];
    assert_eq!(wheel.placed.marks.len(), foundation.grahas.len());
    for position in &foundation.grahas {
        let cell = wheel
            .placed
            .cells
            .iter()
            .find(|cell| cell.ring == 0 && cell.bodies.contains(&position.graha.key_id()))
            .expect("every graha is in a house");
        assert_eq!(cell.house, position.house.bhava, "{:?}", position.graha);
    }
}

#[test]
fn a_drawing_that_cannot_be_drawn_is_refused_by_its_place_in_the_request() {
    use teistro::catalogue::ChartLayout;

    let sdk = context();
    let (place, offset) = kathmandu();
    let instant = JulianDay::<Utc>::literal(2_460_482.5);
    // A Western wheel of the navamsha, which has signs and no degrees.
    let request = ChartRequest::at(place, offset).with_drawings([
        (ChartLayout::NorthIndian, Varga::D1),
        (ChartLayout::WesternWheel, Varga::D9),
    ]);
    let error = sdk
        .chart()
        .reading(instant, &request)
        .expect_err("a wheel of the D9");
    assert_eq!(error.field(), Some("drawings[1].varga"));
    assert!(
        error.message.contains("WESTERN_WHEEL") && error.message.contains("D9"),
        "{error}"
    );
}

#[test]
fn a_consumer_s_own_layout_is_drawn_and_a_shipped_one_is_not_replaced() {
    let mut odia = teistro::geometry::rows::east_indian();
    odia.key = String::from("ACME_ODIA");
    let sdk = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .layout(odia)
        .build()
        .expect("a valid layout of the consumer's own");
    // The key resolves through the context that registered it, to an id
    // the layouts give back, and the id names the key again.
    let id = sdk.keys().id("chart_layout.ACME_ODIA").expect("registered");
    assert!(id.is_registered());
    assert_eq!(sdk.layouts().id("ACME_ODIA"), Some(id));
    assert_eq!(
        sdk.keys().name(id).expect("named"),
        "chart_layout.ACME_ODIA"
    );
    assert!(
        Context::builder()
            .ephemeris([Ephemeris::Builtin])
            .build()
            .expect("a context")
            .keys()
            .id("chart_layout.ACME_ODIA")
            .is_err(),
        "another context registered nothing"
    );
    assert!(sdk.layouts().get("NORTH_INDIAN").is_some());

    let mut takeover = teistro::geometry::rows::east_indian();
    takeover.key = String::from("NORTH_INDIAN");
    let refused = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .layout(takeover)
        .build()
        .expect_err("a shipped key");
    assert_eq!(refused.field(), Some("layouts[0].key"));
}

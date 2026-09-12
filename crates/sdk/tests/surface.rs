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

use teistro::catalogue::{Calendar, Era};
use teistro::{CalendarDate, Context, Ephemeris};
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

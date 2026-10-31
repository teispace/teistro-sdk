//! A context across threads (SR-71, `03-design/rust-consumer-surface.md`,
//! "A context across threads"): a `Context` moves to another thread, a fork
//! shares the core and the ephemeris's cache and keeps its locale state
//! its own, and a `SharedContext` serves many threads at once.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use std::sync::{Arc, OnceLock};

use teistro::catalogue::{Calendar, Nakshatra};
use teistro::matching::{Birth, Book, BookName, NamingRules, SuggestFilter};
use teistro::{
    Body, CalendarDate, Context, Ephemeris, Frame, PositionRequest, SharedContext, TimeScale,
};
use teistro_port_ephemeris::{CountingProvider, EphemerisProvider};

/// A context on the profile every binding's quickstart names.
fn context() -> Context {
    Context::builder()
        .profile("nepali-drik")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the shipped profile, locale and built-in ephemeris")
}

/// 1 Baisakh 2072, the C smoke test's own fact, as a context answers it.
fn new_year(sdk: &Context) -> (i32, u8, u8) {
    let day = CalendarDate::defined(Calendar::Gregorian, 2015, 4, 14);
    let bs = sdk
        .calendar()
        .convert(&day, Calendar::BikramSambat)
        .expect("a date inside the table");
    (bs.year, bs.month, bs.day)
}

#[test]
fn a_context_moves_to_another_thread() {
    let sdk = context();
    let hash = sdk.settings_hash();
    let (answer, moved_hash) = std::thread::spawn(move || (new_year(&sdk), sdk.settings_hash()))
        .join()
        .expect("the worker finished");
    assert_eq!(answer, (2072, 1, 1));
    assert_eq!(moved_hash, hash);
}

#[test]
fn a_shared_context_serves_every_worker_the_same_answers() {
    let sdk = context();
    let shared: SharedContext = sdk.share();
    let answers: Vec<((i32, u8, u8), String)> = (0..4)
        .map(|_| {
            let shared = shared.clone();
            std::thread::spawn(move || {
                let mine = shared.context();
                (new_year(&mine), mine.intl().locale())
            })
        })
        .map(|worker| worker.join().expect("the worker finished"))
        .collect();
    assert_eq!(answers, vec![((2072, 1, 1), "ne-Deva-NP".to_owned()); 4]);
}

#[test]
fn a_fork_answers_as_its_source_does() {
    let sdk = context();
    let fork = sdk.fork();
    assert_eq!(fork.settings_hash(), sdk.settings_hash());
    assert_eq!(fork.profile(), sdk.profile());
    assert_eq!(fork.intl().locale(), sdk.intl().locale());
    assert_eq!(new_year(&fork), new_year(&sdk));
}

/// The point of a fork over a second build: the ephemeris and its cache
/// are the source's, so a grid one asked for is not asked of the engine
/// again by the other.
#[test]
fn a_fork_reads_the_cache_its_source_filled() {
    let slot: Arc<OnceLock<&'static CountingProvider<Box<dyn EphemerisProvider>>>> = Arc::default();
    let seen = Arc::clone(&slot);
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .wrapping(move |opened| -> Box<dyn EphemerisProvider> {
            // Leaked so the test can read the count the context's own
            // provider keeps; a test process ends soon after.
            let counter: &'static CountingProvider<Box<dyn EphemerisProvider>> =
                Box::leak(Box::new(CountingProvider::new(opened)));
            assert!(seen.set(counter).is_ok(), "wrapped once");
            Box::new(counter)
        })
        .build()
        .expect("the built-in ephemeris");
    let counter = slot.get().expect("the context wrapped its provider");
    let jds = [2_451_545.0, 2_451_546.0];
    let bodies = [Body::Sun, Body::Moon];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);

    let first = sdk.positions(&request).expect("the built-in ephemeris");
    let asked = counter.calls().positions;
    assert!(asked > 0, "the first grid reaches the engine");

    let fork = sdk.fork();
    let again = std::thread::scope(|scope| {
        scope
            .spawn(move || fork.positions(&request).map(|answer| answer.value))
            .join()
            .expect("the worker finished")
    })
    .expect("the built-in ephemeris");
    assert_eq!(
        counter.calls().positions,
        asked,
        "the fork answered from the cache its source filled"
    );
    assert_eq!(again.columns.at(1, 1), first.value.columns.at(1, 1));
}

#[test]
fn a_forks_locale_is_its_own() {
    let sdk = context();
    let fork = sdk.fork();
    assert_eq!(
        fork.intl().set_locale("en").expect("a shipped locale"),
        "en-Latn"
    );
    assert_eq!(fork.intl().locale(), "en-Latn");
    assert_eq!(
        sdk.intl().locale(),
        "ne-Deva-NP",
        "the source keeps its locale"
    );
}

#[test]
fn a_shared_context_keeps_the_locale_it_was_shared_with() {
    let sdk = context();
    let shared = sdk.share();
    sdk.intl().set_locale("en-Latn").expect("a shipped locale");
    let worker = shared.context();
    worker.intl().set_locale("en").expect("a shipped locale");
    // Neither the source's change after sharing nor a worker's own
    // reaches the next context it makes.
    assert_eq!(shared.context().intl().locale(), "ne-Deva-NP");
    assert_eq!(worker.intl().locale(), "en-Latn");
}

/// A name book is locale state: one loaded into a fork is that fork's,
/// and its source refuses it by the field a binding names.
#[test]
fn a_book_loaded_in_a_fork_is_refused_by_its_source() {
    let sdk = context();
    let fork = sdk.fork();
    let names = ["चूडामणि", "राम"].map(|name| BookName {
        devanagari: name.to_owned(),
        ..BookName::default()
    });
    let book = fork
        .naming()
        .load(Book {
            names: names.to_vec(),
        })
        .expect("a book of Devanagari names")
        .book;
    let birth = Birth {
        nakshatra: Nakshatra::Ashwini,
        pada: 1,
    };
    let found = fork
        .naming()
        .suggest(
            book,
            birth,
            &SuggestFilter::default(),
            NamingRules::default(),
        )
        .expect("the fork holds the book");
    assert_eq!(found.names[0].index, 0);
    let refused = sdk
        .naming()
        .suggest(
            book,
            birth,
            &SuggestFilter::default(),
            NamingRules::default(),
        )
        .expect_err("the source never loaded it");
    assert_eq!(refused.field(), Some("bookId"));
    // And a fork taken after the book was loaded carries it.
    assert!(
        fork.fork()
            .naming()
            .suggest(
                book,
                birth,
                &SuggestFilter::default(),
                NamingRules::default()
            )
            .is_ok()
    );
}

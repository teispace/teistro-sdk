//! The limb kernel's own properties, over a sky that is plausible rather
//! than real.
//!
//! `crates/port-ephemeris`'s test provider moves the Sun and the Moon at
//! the rates the real ones move at, with one periodic term each, so it is
//! the wrong sky at the right speed. That is exactly what a kernel test
//! needs: the *instants* it produces mean nothing and every structural
//! property does. The instants are held to the corpus by the conformance
//! harness over a real adapter, which Phase 1 deferred; `baseline.rs`
//! beside this file holds everything the corpus can decide without one.
//!
//! What is asserted here is what a caller can rely on whatever provider
//! answers: the spans cover the window, they are in order, they share
//! their boundaries, each carries its own bounds as well as the clipped
//! ones, a tithi is two karanas, and every member is the one the
//! integer path names.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking and print their measurements under --nocapture"
)]

use teistro_astro::ayanamsha::Basis;
use teistro_astro::completion::Completion;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::events::Longitudes;
use teistro_astro::precession::PrecessionModel;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::interval::Interval;
use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
use teistro_panchanga::limb::{self, LONGEST_SPAN_DAYS, Zodiac};
use teistro_panchanga::span::Span;
use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};

/// A day of the plausible sky: J2000.0 and the twenty-four hours after.
fn window() -> Interval {
    Interval::literal(2_451_545.0, 2_451_546.0)
}

fn zodiac() -> Zodiac {
    Zodiac {
        ayanamsha: Some(AyanamshaChoice::Catalogued {
            id: Ayanamsha::Lahiri,
        }),
        basis: Basis::Mean,
        precession: PrecessionModel::Iau2006,
        delta_t: DeltaTModel::TableThenModel,
    }
}

/// The test provider's longitudes, tropical as the kernel asks for them.
fn source<'a>(completion: &'a Completion<'a, TestProvider>) -> impl Longitudes + 'a {
    completion.longitudes(Frame {
        zodiac: FrameZodiac::Tropical,
        ..Frame::CANONICAL
    })
}

/// The tolerance the boundary solver stops at, days: two searches of the
/// same crossing agree to this and not to the last bit of a double.
const SOLVER_TOLERANCE_DAYS: f64 = 1e-7;

/// Whatever the members are, a list of spans has to behave like one.
fn holds_together<T: core::fmt::Debug>(
    spans: &[Span<T>],
    window: Interval,
    longest_days: f64,
    what: &str,
) {
    assert!(!spans.is_empty(), "{what}: a day has at least one");
    assert_eq!(
        spans[0].inside.from, window.from,
        "{what}: the first span opens the window"
    );
    assert_eq!(
        spans[spans.len() - 1].inside.to,
        window.to,
        "{what}: the last closes it"
    );
    for pair in spans.windows(2) {
        assert_eq!(
            pair[0].whole.to, pair[1].whole.from,
            "{what}: consecutive members share one instant"
        );
        assert_eq!(
            pair[0].inside.to, pair[1].inside.from,
            "{what}: and so do the clipped views"
        );
        assert!(
            pair[0].whole.from.get() < pair[1].whole.from.get(),
            "{what}: they are in order"
        );
    }
    for span in spans {
        assert!(
            span.whole.from.get() <= span.inside.from.get()
                && span.whole.to.get() >= span.inside.to.get(),
            "{what}: the clipped view is inside the member's own bounds"
        );
        assert!(
            !span.inside.is_empty(),
            "{what}: an empty span is not a span"
        );
        assert!(
            span.whole.days() < longest_days,
            "{what}: no member outruns the search's widening"
        );
    }
}

#[test]
fn a_day_of_limbs_covers_its_window_exactly() {
    let provider = TestProvider::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let source = source(&completion);
    let limbs = limb::limbs(&source, window(), zodiac()).expect("the test provider answers");

    holds_together(&limbs.tithi, window(), LONGEST_SPAN_DAYS, "tithi");
    holds_together(&limbs.nakshatra, window(), LONGEST_SPAN_DAYS, "nakshatra");
    holds_together(&limbs.yoga, window(), LONGEST_SPAN_DAYS, "yoga");
    holds_together(&limbs.karana, window(), LONGEST_SPAN_DAYS, "karana");

    println!(
        "a day holds {} tithis, {} nakshatras, {} yogas and {} karanas",
        limbs.tithi.len(),
        limbs.nakshatra.len(),
        limbs.yoga.len(),
        limbs.karana.len()
    );
    // A day is one to three of the slow limbs and two to four karanas.
    assert!((1..=3).contains(&limbs.tithi.len()));
    assert!((1..=3).contains(&limbs.nakshatra.len()));
    assert!((1..=3).contains(&limbs.yoga.len()));
    assert!((2..=4).contains(&limbs.karana.len()));
}

#[test]
fn a_tithi_is_two_karanas() {
    let provider = TestProvider::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let source = source(&completion);
    // A week, so that no span of either list is cut away by the window
    // and the two can be compared whole.
    let week = Interval::literal(2_451_545.0, 2_451_552.0);
    let limbs = limb::limbs(&source, week, zodiac()).expect("the test provider answers");

    // Every tithi boundary is a karana boundary: the two lists come from
    // one search at six degrees, and the halving is what makes it one.
    let mut shared = 0;
    for tithi in limbs.tithi.iter().skip(1) {
        assert!(
            limbs.karana.iter().any(|karana| {
                (karana.whole.from.get() - tithi.whole.from.get()).abs() < SOLVER_TOLERANCE_DAYS
            }),
            "a tithi beginning at {} is not a karana boundary",
            tithi.whole.from
        );
        shared += 1;
    }
    println!("{shared} tithi boundaries are karana boundaries");
    assert!(shared >= 5, "a week holds at least six tithis");
    // Two karanas make each tithi, and a karana is about half of one.
    assert!(
        limbs.karana.len() >= 2 * limbs.tithi.len() - 2,
        "{} karanas for {} tithis",
        limbs.karana.len(),
        limbs.tithi.len()
    );
    let tithi_days = limbs.tithi[1].whole.days();
    for karana in limbs.karana.iter().skip(1).take(limbs.karana.len() - 2) {
        let ratio = karana.whole.days() / tithi_days;
        assert!(
            (0.4..0.6).contains(&ratio),
            "a karana is half a tithi, not {ratio}"
        );
    }
}

#[test]
fn a_wider_window_holds_the_same_members_at_the_same_instants() {
    // The kernel must not depend on where the window was cut: a day
    // asked for alone and the same day inside a longer stretch have to
    // agree, or a range would answer differently from its own days.
    let provider = TestProvider::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let source = source(&completion);
    let day = limb::limbs(&source, window(), zodiac()).expect("the test provider answers");
    let wide = Interval::literal(window().from.get() - 3.0, window().to.get() + 3.0);
    let stretch = limb::limbs(&source, wide, zodiac()).expect("the test provider answers");

    let mut matched = 0;
    for span in &day.tithi {
        let same = stretch
            .tithi
            .iter()
            .find(|other| other.member == span.member && other.whole.overlaps(span.whole))
            .unwrap_or_else(|| panic!("{:?} is not in the stretch", span.member));
        assert!(
            (same.whole.from.get() - span.whole.from.get()).abs() < SOLVER_TOLERANCE_DAYS,
            "{:?} begins at a different instant: {} against {}",
            span.member,
            same.whole.from,
            span.whole.from
        );
        matched += 1;
    }
    println!("{matched} tithis agree between a day and a week");
    assert!(matched > 0);
}

#[test]
fn the_members_are_the_ones_the_integer_path_names() {
    let provider = TestProvider::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let source = source(&completion);
    let limbs = limb::limbs(&source, window(), zodiac()).expect("the test provider answers");

    // Read the sky in the middle of each span and classify it by hand;
    // the kernel must have named the same member.
    let ayanamsha = teistro_astro::ayanamsha::value_deg(
        &AyanamshaChoice::Catalogued {
            id: Ayanamsha::Lahiri,
        },
        teistro_core::quantity::JulianDay::literal(2_451_545.5),
        Basis::Mean,
        PrecessionModel::Iau2006,
        DeltaTModel::TableThenModel,
    )
    .expect("Lahiri at J2000");

    for span in &limbs.nakshatra {
        let middle = teistro_core::quantity::JulianDay::literal(
            span.whole.days().mul_add(0.5, span.whole.from.get()),
        );
        let (tropical, _) = source
            .longitude_and_speed(Body::Moon, middle)
            .expect("the test provider answers");
        let sidereal = (tropical - ayanamsha).rem_euclid(360.0);
        let index = teistro_core::angle::Nas::try_from_degrees(sidereal)
            .expect("finite")
            .nakshatra_index()
            .get();
        // The ayanamsha used here is the one at the middle of the day
        // rather than at each instant, so a member within an arcsecond of
        // a boundary may differ; nothing in this window is.
        assert_eq!(
            u32::from(index),
            u32::from(span.member.id()),
            "the nakshatra in the middle of its own span"
        );
    }
}

#[test]
fn the_signs_a_body_stood_in_cover_the_window() {
    let provider = TestProvider::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let source = source(&completion);
    for body in [Body::Sun, Body::Moon] {
        let signs = limb::signs(&source, body, window(), zodiac()).expect("the provider answers");
        // A solar sign is a month long, so the bound is the sign search's
        // own widening rather than a limb's.
        holds_together(&signs, window(), 40.0, "sign");
        // The Sun crosses a sign in a month and the Moon in two and a
        // half days, so neither manages more than two in a day.
        assert!((1..=2).contains(&signs.len()), "{body:?}: {}", signs.len());
    }
}

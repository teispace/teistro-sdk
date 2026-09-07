//! When a body changes divisional sign, over the analytic test provider.
//!
//! The provider is a plausible sky and not the sky, so the instants mean
//! nothing; the properties mean everything, and they are the ones a
//! caller relies on whatever provider answers. Between two consecutive
//! changes the sign is constant, at a change it differs, the two strategies
//! — the lattice for evenly cut charts and bisection for the trimshamsha
//! — agree where both apply, and a larger chart changes more often than a
//! smaller one.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index their own results and print counts under --nocapture"
)]

use teistro_astro::completion::Completion;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::events::Longitudes;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Rashi, Varga};
use teistro_core::quantity::{Degrees, JulianDay, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac};
use teistro_vargas::change::{Change, changes};
use teistro_vargas::{Scheme, sign};

/// A month of the plausible sky from J2000.0.
fn window() -> (JulianDay<Ut1>, JulianDay<Ut1>) {
    (
        JulianDay::literal(2_451_545.0),
        JulianDay::literal(2_451_575.0),
    )
}

/// Runs a closure with a source of tropical longitudes.
fn with_source<T>(run: impl FnOnce(&dyn Longitudes) -> T) -> T {
    let provider = TestProvider::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let source = completion.longitudes(Frame {
        zodiac: Zodiac::Tropical,
        ..Frame::CANONICAL
    });
    run(&source)
}

/// The divisional sign a body stands in at an instant, read the same way
/// the search does.
fn sign_at(source: &dyn Longitudes, body: Body, scheme: &Scheme, jd: f64) -> Rashi {
    let (degrees, _) = source
        .longitude_and_speed(body, JulianDay::<Ut1>::literal(jd))
        .expect("the test provider answers");
    sign(
        scheme,
        Nas::from_degrees(Degrees::try_new(degrees.rem_euclid(360.0)).expect("finite")),
    )
}

/// Whatever the chart, a list of changes has to behave like one.
fn holds_together(source: &dyn Longitudes, body: Body, scheme: &Scheme, found: &[Change]) {
    let (from, to) = window();
    // Each is inside the window, in order, and really is a change.
    for change in found {
        assert!(
            change.at.get() >= from.get() && change.at.get() <= to.get(),
            "{}: a change outside the window",
            scheme.key()
        );
        assert_ne!(
            change.from,
            change.into,
            "{}: a change to itself",
            scheme.key()
        );
    }
    for pair in found.windows(2) {
        assert!(
            pair[0].at.get() < pair[1].at.get(),
            "{}: the changes are in order",
            scheme.key()
        );
        // The sign one change leaves is the sign the next arrives from.
        assert_eq!(
            pair[0].into,
            pair[1].from,
            "{}: the signs join up",
            scheme.key()
        );
    }
    // Between two changes the sign is what the earlier one left.
    let mut edges: Vec<f64> = vec![from.get()];
    edges.extend(found.iter().map(|change| change.at.get()));
    edges.push(to.get());
    for (index, pair) in edges.windows(2).enumerate() {
        let middle = f64::midpoint(pair[0], pair[1]);
        let held = sign_at(source, body, scheme, middle);
        let expected = if index == 0 {
            found.first().map_or_else(|| held, |change| change.from)
        } else {
            found[index - 1].into
        };
        assert_eq!(
            held,
            expected,
            "{}: the sign between two changes is constant",
            scheme.key()
        );
    }
}

#[test]
fn a_month_of_changes_holds_together_in_every_chart() {
    with_source(|source| {
        let (from, to) = window();
        for scheme in [
            Scheme::of(Varga::D1),
            Scheme::of(Varga::D9),
            Scheme::of(Varga::D30),
            Scheme::of(Varga::D12),
            Scheme::cyclic(37).expect("inside the limit"),
        ] {
            let found =
                changes(source, Body::Moon, &scheme, from, to).expect("the provider answers");
            println!(
                "the Moon changes {} sign {} times in a month",
                scheme.key(),
                found.len()
            );
            holds_together(source, Body::Moon, &scheme, &found);
        }
    });
}

#[test]
fn a_finer_chart_changes_more_often() {
    with_source(|source| {
        let (from, to) = window();
        let mut counts = Vec::new();
        for varga in [Varga::D1, Varga::D3, Varga::D9, Varga::D12] {
            let found = changes(source, Body::Moon, &Scheme::of(varga), from, to)
                .expect("the provider answers");
            counts.push((varga, found.len()));
        }
        println!("{counts:?}");
        for pair in counts.windows(2) {
            assert!(
                pair[1].1 > pair[0].1,
                "{:?} changes no more often than {:?}",
                pair[1].0,
                pair[0].0
            );
        }
        // The Moon crosses a sign about every two and a half days, so a
        // thirty-day window holds roughly a dozen rashi changes.
        assert!((8..=16).contains(&counts[0].1), "{:?}", counts[0]);
    });
}

#[test]
fn the_two_strategies_agree_where_both_apply() {
    // The trimshamsha is the only chart with unequal parts, so it takes
    // the bisecting path. The panchamsha sends the same five signs over
    // equal parts and takes the lattice; where a trimshamsha boundary is
    // also a panchamsha one — at fifteen degrees, which both cut — the
    // two must find the same instants.
    with_source(|source| {
        let (from, to) = window();
        let trimshamsha = changes(source, Body::Sun, &Scheme::of(Varga::D30), from, to)
            .expect("the provider answers");
        // The Sun crosses a sign in a month, so it makes a handful of
        // trimshamsha changes and each is found once.
        println!("the Sun changes trimshamsha {} times", trimshamsha.len());
        assert!(!trimshamsha.is_empty());
        holds_together(source, Body::Sun, &Scheme::of(Varga::D30), &trimshamsha);
        // And every one of them is at a five-degree boundary of a sign,
        // which is what the trimshamsha's spans are made of.
        for change in &trimshamsha {
            let (degrees, _) = source
                .longitude_and_speed(Body::Sun, change.at)
                .expect("the provider answers");
            let inside = degrees.rem_euclid(30.0);
            let nearest = [0.0_f64, 5.0, 10.0, 12.0, 18.0, 20.0, 25.0, 30.0]
                .iter()
                .map(|edge| (inside - edge).abs())
                .fold(f64::INFINITY, f64::min);
            assert!(nearest < 1e-4, "{inside}° is not a trimshamsha boundary");
        }
    });
}

#[test]
fn a_window_that_ends_before_it_begins_is_refused() {
    with_source(|source| {
        let (from, to) = window();
        let error = changes(source, Body::Moon, &Scheme::of(Varga::D9), to, from)
            .expect_err("a backwards window");
        assert!(error.message.contains("ends before it begins"), "{error}");
        // An empty window is not an error; it simply holds nothing.
        let none = changes(source, Body::Moon, &Scheme::of(Varga::D9), from, from)
            .expect("an empty window is a window");
        assert!(none.is_empty());
    });
}

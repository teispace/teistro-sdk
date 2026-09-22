//! The annual chart through the façade: the Sun's returns to where it
//! stood at birth, the two rivals beside them, and the chart the year
//! opens (`docs/03-design/annual-chart.md`).
//!
//! What these hold is the property the measured page measures and a
//! consumer relies on — that the return is the **chart's own** sidereal
//! reading and not a frame's, which is 18 arcseconds and about two
//! degrees of lagna apart.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::tajika::{Reading, SIDEREAL_YEAR_DAYS};
use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};

const BIRTH: f64 = 2_447_995.489_583_333_5;

fn context() -> Context {
    Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a built context")
}

fn request() -> ChartRequest {
    ChartRequest::at(
        Place::new(
            Latitude::try_new(27.7172).unwrap(),
            Longitude::try_new(85.324).unwrap(),
            Altitude::try_new(1400.0).unwrap(),
        ),
        UtcOffset::try_from_seconds(20_700).unwrap(),
    )
}

fn natal(sdk: &Context) -> Document {
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(BIRTH), &request())
        .unwrap()
        .value
}

/// The property the whole module rests on: at a return the Sun stands
/// where it stood at birth, read the way a founded chart reads it.
///
/// A frame's own sidereal zodiac applies the **mean** ayanamsha where a
/// chart applies the nutated one, 18.46 arcseconds apart, so a return
/// computed that way would fail this by about seven minutes of the Sun's
/// time. A tenth of an arcsecond is a thousand times tighter than that
/// gap and still far looser than the search's own tolerance, so this
/// cannot pass by luck.
#[test]
fn at_every_return_the_sun_stands_where_it_stood() {
    let sdk = context();
    let document = natal(&sdk);
    let was = document.foundation.graha(Graha::Sun).unwrap().longitude_deg;
    let years = sdk
        .chart()
        .praveshas(&document, Reading::Sidereal, 40)
        .unwrap();
    assert_eq!(years.len(), 40, "forty returns of a 1990 birth");
    for one in &years {
        let annual = sdk.chart().reading(one.at, &request()).unwrap().value;
        let now = annual.foundation.graha(Graha::Sun).unwrap().longitude_deg;
        let apart = (now - was).rem_euclid(360.0);
        let apart = apart.min(360.0 - apart) * 3600.0;
        assert!(apart < 0.1, "year {}: {apart} arcseconds", one.year);
    }
}

/// The returns are in order, one a year, and each says which reading made
/// it. The interval is a **sidereal** year and not a tropical one, which
/// is the measurement that says which zodiac was read.
#[test]
fn the_returns_run_a_sidereal_year_apart_and_say_what_they_are() {
    let sdk = context();
    let document = natal(&sdk);
    let years = sdk
        .chart()
        .praveshas(&document, Reading::Sidereal, 12)
        .unwrap();
    assert!(years.iter().map(|one| one.year).eq(1..=12));
    assert!(years.iter().all(|one| one.reading == Reading::Sidereal));
    let mut last = BIRTH;
    for one in &years {
        let gap = one.at.get() - last;
        // The Earth's orbit is not a circle, so the interval moves about
        // a fifth of a day either way; a tropical year would be 0.014
        // short of this every single time.
        assert!(
            (gap - SIDEREAL_YEAR_DAYS).abs() < 0.02,
            "year {}: {gap} days",
            one.year
        );
        last = one.at.get();
    }
}

/// The two rivals are reachable by name and are not the same instant.
///
/// A consumer building a Western tool asks for the tropical return and
/// gets it; nobody gets it by accident, which is what naming the reading
/// on every answer buys.
#[test]
fn the_rivals_are_asked_for_by_name_and_differ() {
    let sdk = context();
    let document = natal(&sdk);
    let sidereal = sdk
        .chart()
        .praveshas(&document, Reading::Sidereal, 40)
        .unwrap();
    let tropical = sdk
        .chart()
        .praveshas(&document, Reading::Tropical, 40)
        .unwrap();
    let mean = sdk.chart().praveshas(&document, Reading::Mean, 40).unwrap();
    assert!(tropical.iter().all(|one| one.reading == Reading::Tropical));
    assert!(mean.iter().all(|one| one.reading == Reading::Mean));

    // At forty years the tropical reading is hours away and the mean one
    // minutes; both are the wrong chart, by different amounts.
    let hours = |a: f64, b: f64| (a - b).abs() * 24.0;
    let tropical_apart = hours(tropical[39].at.get(), sidereal[39].at.get());
    let mean_apart = hours(mean[39].at.get(), sidereal[39].at.get());
    assert!(tropical_apart > 1.0, "{tropical_apart} hours");
    assert!(mean_apart > 0.05 && mean_apart < 1.0, "{mean_apart} hours");
    assert!(tropical_apart > mean_apart);

    // The mean reading needs no ephemeris, so it answers from a context
    // that has none at all.
    let bare = Context::builder()
        .build()
        .expect("a context with no ephemeris");
    let without = bare
        .chart()
        .praveshas(&document, Reading::Mean, 40)
        .expect("the mean reading is arithmetic");
    assert_eq!(without, mean);
    assert!(
        bare.chart()
            .praveshas(&document, Reading::Sidereal, 1)
            .is_err(),
        "the true reading needs one"
    );
}

/// A year is founded as a chart, and it is the chart of that instant.
#[test]
fn a_year_founds_as_a_chart_of_its_own_instant() {
    let sdk = context();
    let document = natal(&sdk);
    let thirtieth = sdk
        .chart()
        .praveshas(&document, Reading::Sidereal, 30)
        .unwrap()[29];
    let annual = sdk
        .chart()
        .annual(&document, Reading::Sidereal, 30, &request())
        .unwrap()
        .value;
    assert_eq!(annual.foundation.instant, thirtieth.at);
    assert_eq!(annual.foundation.place, document.foundation.place);
    // Thirty years on, the lagna is its own: a chart, not a copy. The
    // return falls hours from the birth's clock time, so the lagna has
    // moved a good part of the circle rather than a rounding.
    let apart = (annual.foundation.lagna_deg - document.foundation.lagna_deg).rem_euclid(360.0);
    assert!(apart.min(360.0 - apart) > 1.0, "{apart}°");
}

/// A year outside the cap is refused by the field that names it, and a
/// year past the ephemeris by the year with what it does reach.
#[test]
fn a_year_that_cannot_be_reached_is_refused_by_name() {
    let sdk = context();
    let document = natal(&sdk);
    let wide = sdk
        .chart()
        .praveshas(&document, Reading::Sidereal, 0)
        .unwrap_err();
    assert_eq!(wide.field(), Some("through"));
    assert!(wide.to_string().contains("1 to 200"), "{wide}");

    // A nonsense year is reported as one whichever reading was asked for
    // and whether or not a provider is attached: the argument is refused
    // before anything is looked up. Without that order, a context with no
    // ephemeris would answer "no ephemeris" to a question about the year,
    // and capping would turn the zero into an empty list.
    let bare = Context::builder()
        .build()
        .expect("a context with no ephemeris");
    for reading in [Reading::Sidereal, Reading::Tropical, Reading::Mean] {
        for ctx in [&sdk, &bare] {
            let why = ctx.chart().praveshas(&document, reading, 0).unwrap_err();
            assert_eq!(why.field(), Some("through"), "{reading:?}");
        }
    }

    // A birth the built-in ephemeris cannot follow for two centuries.
    let late = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(2_597_641.0 - 200.0), &request())
        .unwrap()
        .value;
    let refused = sdk
        .chart()
        .annual(&late, Reading::Sidereal, 200, &request())
        .unwrap_err();
    assert_eq!(refused.field(), Some("year"));
    assert!(
        refused.hint().unwrap_or_default().contains("year"),
        "{refused:?}"
    );
}

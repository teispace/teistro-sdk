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
use teistro::{ChartRequest, Context, Document, Ephemeris, House, UtcOffset};

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

/// The source works its chart on a **geocentric** frame, which the default
/// profile is and the conformance one is not: under the latter its Moon is
/// 58 arcminutes out, which is the Moon's parallax at Bombay.
fn source_context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the default profile, which is geocentric as the source is")
}

/// Bombay, where the source's example was born and casts its years.
fn source_place() -> ChartRequest {
    ChartRequest::at(
        Place::new(
            Latitude::try_new(18.0 + 58.0 / 60.0).unwrap(),
            Longitude::try_new(72.0 + 50.0 / 60.0).unwrap(),
            Altitude::try_new(11.0).unwrap(),
        ),
        UtcOffset::try_from_seconds(19_800).unwrap(),
    )
}

/// The source's birth chart and the annual chart of its forty-first year,
/// under the **mean** return its Dhruvanka computes. 07:11 IST is 01:41 UTC.
fn source_birth_and_year(sdk: &Context) -> (Document, Document) {
    let bombay = source_place();
    let birth = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(2_431_322.570_138_889), &bombay)
        .unwrap()
        .value;
    let mean = sdk.chart().praveshas(&birth, Reading::Mean, 40).unwrap()[39];
    let annual = sdk.chart().reading(mean.at, &bombay).unwrap().value;
    (birth, annual)
}

/// The source's own worked year, end to end through the façade: birth,
/// return, annual chart and its five office-bearers (K.S. Charak, *A
/// Textbook of Varshaphala*, Chart III-1, a birth at Bombay on 20 August
/// 1944 at 07:11 IST, and its annual chart for the forty-first year).
///
/// The source steps the years by the **mean** sidereal year — its
/// Dhruvanka of 1d 6h 6m 29s for forty years is forty mean years modulo
/// a week — so it is [`Reading::Mean`] that must land on its 13:17:29
/// IST, and the true return is the one it says "may differ by a few
/// minutes". Its positions are **geocentric**: under the topocentric
/// conformance profile its Moon is 58′ out, which is the Moon's
/// parallax, and under the text-cited default it is 3′. The bounds are
/// what it prints — whole arcminutes and seconds — with that margin.
#[test]
fn the_sources_worked_year_reproduces_end_to_end() {
    let sdk = source_context();
    let bombay = source_place();
    let arcmin = |deg: f64, sign: f64, degrees: f64, minutes: f64| {
        (deg - (sign * 30.0 + degrees + minutes / 60.0)) * 60.0
    };
    // 07:11 IST is 01:41 UTC.
    let birth = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(2_431_322.570_138_889), &bombay)
        .unwrap()
        .value;
    assert!(
        arcmin(birth.foundation.lagna_deg, 4.0, 14.0, 36.0).abs() < 2.0,
        "Leo 14°36′"
    );

    let mean = sdk.chart().praveshas(&birth, Reading::Mean, 40).unwrap()[39];
    let ist_hours = (mean.at.get() + 0.5 + 5.5 / 24.0).fract() * 24.0;
    let printed = 13.0 + 17.0 / 60.0 + 29.0 / 3600.0;
    assert!(
        (ist_hours - printed).abs() * 3600.0 < 3.0,
        "13:17:29 IST, got {ist_hours}h"
    );

    let annual = sdk.chart().reading(mean.at, &bombay).unwrap().value;
    let year = &annual.foundation;
    let placed = |graha| year.graha(graha).unwrap().longitude_deg;
    assert!(
        arcmin(year.lagna_deg, 7.0, 9.0, 26.0).abs() < 3.0,
        "Scorpio 9°26′"
    );
    assert!(
        arcmin(placed(Graha::Sun), 4.0, 3.0, 50.0).abs() < 2.0,
        "Leo 3°50′"
    );
    assert!(
        arcmin(placed(Graha::Moon), 1.0, 9.0, 40.0).abs() < 5.0,
        "Taurus 9°40′"
    );
    assert!(year.day.part.is_daylight());

    // The true return is the "few minutes" the source sets aside — the
    // Sun's own perturbations, which it names — and here it moves none of
    // the five. Measured: 0.83 minutes on this profile's **mean**
    // ayanamsha; about 4.8 on the conformance profile's **nutated** one,
    // the difference being nutation, which the source does not apply. So
    // the bound is the source's claim, a nonzero gap of minutes, and not
    // a figure of its own.
    let true_return = sdk
        .chart()
        .praveshas(&birth, Reading::Sidereal, 40)
        .unwrap()[39];
    let minutes = (true_return.at.get() - mean.at.get()) * 1440.0;
    assert!(
        minutes.abs() > 0.0 && minutes.abs() < 15.0,
        "{minutes} minutes"
    );
    let on_the_true = sdk.chart().reading(true_return.at, &bombay).unwrap().value;
    assert_eq!(
        sdk.chart()
            .office_bearers(&birth, &on_the_true, 40)
            .unwrap(),
        sdk.chart().office_bearers(&birth, &annual, 40).unwrap(),
        "the few minutes move none of the five"
    );
}

/// What the source reads **from** that year's chart: its five
/// office-bearers, the five-fold strength of its seven (Table VI-10), and
/// the lord of the year — every figure as the source prints it.
#[test]
fn the_sources_worked_year_is_read_as_the_source_reads_it() {
    let sdk = source_context();
    let (birth, annual) = source_birth_and_year(&sdk);

    let five = sdk.chart().office_bearers(&birth, &annual, 40).unwrap();
    assert_eq!(
        [
            five.muntha,
            five.janma_lagna,
            five.varsha_lagna,
            five.tri_rashi,
            five.dina_ratri
        ],
        [
            Graha::Jupiter,
            Graha::Sun,
            Graha::Mars,
            Graha::Mars,
            Graha::Sun
        ],
        "the source's five: Muntha, Janma Lagna, Varsha Lagna, Tri-Rashi, Dina-Ratri"
    );

    // The five-fold strength of its seven, through the façade and on the
    // chart the façade founded.
    let bala = sdk.chart().panchavargiya(&annual).unwrap();
    let of = |graha| {
        bala.iter()
            .find(|one| one.graha == graha)
            .expect("one of the seven")
    };
    assert_eq!(of(Graha::Jupiter).vishwa.to_string(), "14:46:00");
    assert_eq!(of(Graha::Saturn).vishwa.to_string(), "16:47:45");
    assert_eq!(of(Graha::Sun).total.to_string(), "57:21:00");
    let strongest = bala.iter().max_by_key(|one| one.vishwa).unwrap();
    assert_eq!(strongest.graha, Graha::Saturn, "of all seven");

    // The Tajika aspects of that chart, and the source's own worked
    // Ithasala in it: the Sun at Leo 3°50′ and Mars at Scorpio 7°42′,
    // orb 11°30′ (the mean of 15 and 8), 3°52′ apart within their signs,
    // the Sun faster and behind — so they are coming together.
    let pairs = sdk.chart().drishtis(&annual).unwrap();
    assert_eq!(pairs.len(), 21, "every pair of the seven");
    let sun_mars = pairs
        .iter()
        .find(|pair| {
            [pair.faster, pair.slower].contains(&Graha::Sun)
                && [pair.faster, pair.slower].contains(&Graha::Mars)
        })
        .expect("the Sun and Mars");
    assert_eq!(sun_mars.faster, Graha::Sun);
    assert!((sun_mars.orb_deg - 11.5).abs() < 1e-12);
    assert!(
        (sun_mars.apart_deg - (3.0 + 52.0 / 60.0)).abs() < 2.0 / 60.0,
        "3°52′, got {}",
        sun_mars.apart_deg
    );
    // Behind by more than a single degree, so of Table X-3's three kinds
    // it is the **Vartamana**: coming, and not yet arrived.
    assert_eq!(sun_mars.yoga, Some(teistro::TajikaYoga::IthasalaVartamana));
    assert!(sun_mars.yoga.is_some_and(teistro::TajikaYoga::is_ithasala));
    assert!(!sun_mars.disputed(), "well clear of the contested band");

    // The readings the source leaves open reach this chart too, and
    // change nothing in it: no pair of this sky stands in the band where
    // its two accounts differ, so all three answer alike.
    for sub_degree in [
        teistro::SubDegree::Poorna,
        teistro::SubDegree::Ishrafa,
        teistro::SubDegree::None,
    ] {
        let under = sdk
            .chart()
            .drishtis_with_rules(&annual, teistro::DrishtiRules { sub_degree })
            .unwrap();
        assert_eq!(under.len(), 21);
        assert!(
            !under.iter().any(teistro::Between::disputed),
            "no pair of this chart is contested"
        );
        assert_eq!(
            under.iter().filter(|pair| pair.yoga.is_some()).count(),
            pairs.iter().filter(|pair| pair.yoga.is_some()).count(),
            "the reading moves nothing here"
        );
    }

    // And the lord of the year the source names: **the Sun**, not the
    // strongest office-bearer. Jupiter leads on strength and stands in the
    // second from the lagna, which gives no Tajika aspect, so the source
    // disqualifies it in as many words and the Sun takes the year.
    let year = sdk
        .chart()
        .varshesha(&birth, &annual, 40, teistro::VarsheshaRules::default())
        .unwrap();
    assert_eq!(year.graha, Graha::Sun);
    assert_eq!(year.vishwa.to_string(), "14:20:15");
    assert_eq!(year.claims[0].graha, Graha::Jupiter);
    assert!(
        !year.claims[0].aspects_lagna,
        "the second house aspects nothing"
    );
}

/// The sixteen Tajika yogas, asked of the source's own worked year through
/// the façade — and asked as the sources define them, about a **matter**.
///
/// Fourteen of the sixteen are judgements about the lagnesha and the
/// karyesha, so the same chart answers differently for each of the twelve
/// houses. What this holds is that the question reaches the answer: the
/// pair is read from the chart, the yoga from the pair, and a yoga this
/// build cannot compute says so rather than reading as absent.
#[test]
fn the_years_yogas_answer_a_matter_and_name_what_they_cannot_answer() {
    let sdk = source_context();
    let (_birth, annual) = source_birth_and_year(&sdk);
    // The **first** house is the lagna, so asking about it names the
    // lagna's own sign: the test takes it from the answer rather than
    // recomputing a sign from a longitude, which would be a second copy
    // of the arithmetic under test.
    let first = sdk
        .chart()
        .tajika_yogas(&annual, House::try_new(1).unwrap())
        .unwrap();
    let lagna_sign = first.sign;
    assert!(first.same_lord, "the first house's lord is the lagnesha");

    // Every house is answerable, and each names its own karyesha.
    for number in 1..=12u8 {
        let house = House::try_new(number).unwrap();
        let found = sdk.chart().tajika_yogas(&annual, house).unwrap();
        assert_eq!(found.house, house);
        assert_eq!(found.sign, house.sign_from(lagna_sign));
        assert_eq!(found.lagnesha, lagna_sign.attributes().lord);
        assert_eq!(found.karyesha, found.sign.attributes().lord);

        // Twelve of the sixteen are not built, and each carries its
        // reason. A consumer asking about one gets `None` -- not `false`,
        // which would be a claim this build has no right to make.
        assert_eq!(found.unanswered.len(), 9);
        for yoga in &found.unanswered {
            assert!(yoga.awaiting().is_some(), "{yoga:?} says what it needs");
            assert_eq!(found.holds(*yoga), None);
        }
        // The seven that are built always answer, true or false.
        for yoga in [
            teistro::YearYoga::Ithasala,
            teistro::YearYoga::Ishrafa,
            teistro::YearYoga::Nakta,
            teistro::YearYoga::Yamaya,
            teistro::YearYoga::Manau,
            teistro::YearYoga::Kamboola,
            teistro::YearYoga::Khallasara,
        ] {
            assert!(found.holds(yoga).is_some(), "{yoga:?} is built");
        }
        // Manau, Kamboola and Khallasara are judgements **about** an
        // Ithasala, so none of them can hold without one.
        if found.holds(teistro::YearYoga::Ithasala) != Some(true) {
            for yoga in [
                teistro::YearYoga::Manau,
                teistro::YearYoga::Kamboola,
                teistro::YearYoga::Khallasara,
            ] {
                assert_eq!(found.holds(yoga), Some(false), "no Ithasala to judge");
            }
        }
        // The first house is the lagna, so its lord is the lagnesha and
        // there is no pair to judge.
        assert_eq!(found.same_lord, found.karyesha == found.lagnesha);
        if found.same_lord {
            assert!(found.between.is_none() && found.held.is_empty());
        }
        // Nothing is ever carried across a pair that already aspects.
        if found.between.is_some_and(|pair| pair.drishti.is_aspect()) {
            assert!(found.held.iter().all(|one| one.through.is_none()));
        }
    }

    // The source's own definition of "unqualified", through the façade:
    // every clause carried, and the Hudda one structurally false because
    // the Egyptian terms give the luminaries no degrees at all.
    let how = sdk.chart().qualification(&annual, Graha::Moon).unwrap();
    assert_eq!(how.graha, Graha::Moon);
    assert!(!how.own_hudda, "the Hudda gives the luminaries nothing");
    assert_eq!(
        how.is_unqualified(),
        !how.exalted
            && !how.debilitated
            && !how.aspected
            && !how.own_hudda
            && !how.own_drekkana
            && !how.own_navamsha
    );

    // The readings the source leaves open reach here too, through the
    // pair every one of the fourteen is built on.
    let tenth = House::try_new(10).unwrap();
    for sub_degree in [
        teistro::SubDegree::Poorna,
        teistro::SubDegree::Ishrafa,
        teistro::SubDegree::None,
    ] {
        let under = sdk
            .chart()
            .tajika_yogas_with_rules(&annual, tenth, teistro::DrishtiRules { sub_degree })
            .unwrap();
        assert_eq!(under.house, tenth);
        assert_eq!(under.unanswered.len(), 9);
    }
}

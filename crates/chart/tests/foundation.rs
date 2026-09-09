//! A foundation founded end to end, over the analytic test provider.
//!
//! The provider's numbers are not an ephemeris and are not compared with
//! one; `crates/astro` measures the astronomy and the corpus tests beside
//! this one measure the day, the bhavas and the zodiac against recorded
//! charts. What is checked here is what only the assembled value can
//! show: that the parts agree with each other.
//!
//! Those agreements are the design's own claims. Every graha stands in the
//! same zodiac as the cusps it is placed against; a placement's bhava is
//! the bhava its `through` and `from_madhya` describe; the lagna is the
//! ascendant of the chart's own division; Ketu is Rahu's opposite point;
//! the day the chart belongs to holds its instant; and founding one chart
//! twice, or founding it in a batch, gives the same value.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::print_stdout,
    reason = "tests fail by panicking, index fixed arrays and print the measurement under --nocapture"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::Gregorian;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::bhava::Chalit;
use teistro_chart::foundation::{ChartFoundation, Founder, bodies_of};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::{Ayanamsha, ChartKind, Graha, HouseSystem};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{OverridePolicy, Profile, Resolved, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_port_ephemeris::test_provider::TestProvider;

/// Kathmandu, where the corpus's own charts are.
fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

fn resolved() -> Resolved {
    Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .unwrap_or_else(|| panic!("the default profile"))
        .resolve(&SettingsPatch::default())
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Founds a chart at an instant, with everything the founder needs.
fn found(instant: f64) -> ChartFoundation {
    let provider = TestProvider;
    let resolved = resolved();
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let founder = Founder::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    );
    founder
        .found_one(
            JulianDay::<Utc>::literal(instant),
            &place(),
            ChartKind::Natal,
        )
        .unwrap_or_else(|e| panic!("{e}"))
        .value
}

/// A spread of instants across a day and across the centuries, so the
/// agreements are checked somewhere other than one convenient moment.
const INSTANTS: [f64; 6] = [
    2_447_995.489_583, // the corpus's c001, a birth before sunrise
    2_447_995.750_000, // the same day, by daylight
    2_451_545.000_000, // J2000
    2_415_020.500_000, // 1900
    2_469_807.500_000, // 2050
    2_400_000.500_000, // 1858
];

#[test]
fn every_graha_stands_in_the_same_zodiac_as_the_cusps() {
    for instant in INSTANTS {
        let chart = found(instant);
        // The chart's zodiac is one shift from the tropical one, and the
        // grahas carry both readings of it.
        for graha in &chart.grahas {
            assert!(
                difference_deg(
                    chart.zodiac.of_tropical(graha.tropical_deg),
                    graha.longitude_deg
                )
                .abs()
                    < 1e-9,
                "{:?}: the two readings disagree",
                graha.graha
            );
        }
        // The lagna is in the first bhava of the placement division,
        // whatever that division is: that is what "first house" means.
        let first = chart.houses.place(chart.lagna_deg);
        assert_eq!(
            first.bhava, 1,
            "the lagna is in the first bhava of its own division"
        );
        // And under whole sign it is the sign the lagna falls in.
        assert_eq!(
            chart.lagna_sign_index(),
            (chart.houses.sandhi[0].rem_euclid(360.0) / 30.0) as u8,
            "the first whole-sign house is the lagna's sign"
        );
    }
}

#[test]
fn a_placement_describes_the_bhava_it_names() {
    for instant in INSTANTS {
        let chart = found(instant);
        for graha in &chart.grahas {
            for (which, placed, bhavas) in [
                ("chalit", graha.placement, &chart.chalit),
                ("house", graha.house, &chart.houses),
            ] {
                let index = usize::from(placed.bhava - 1);
                assert!(
                    (1..=12).contains(&placed.bhava),
                    "{which}: a bhava is 1 to 12"
                );
                assert!(
                    (0.0..1.0).contains(&placed.through),
                    "{:?} {which}: through {} is outside its bhava",
                    graha.graha,
                    placed.through
                );
                // The graha is past the sandhi that opens its bhava and
                // before the one that closes it.
                let opened = (graha.longitude_deg - bhavas.sandhi[index]).rem_euclid(360.0);
                let width = bhavas.widths()[index];
                assert!(
                    opened < width,
                    "{:?} {which}: {opened}° into a bhava {width}° wide",
                    graha.graha
                );
                // And `from_madhya` is measured from that bhava's middle.
                assert!(
                    (placed.from_madhya_deg
                        - difference_deg(graha.longitude_deg, bhavas.madhya[index]))
                    .abs()
                        < 1e-12,
                    "{:?} {which}: from_madhya is another bhava's",
                    graha.graha
                );
                assert_eq!(placed.method, bhavas.chalit.method);
            }
        }
    }
}

#[test]
fn ketu_is_rahus_opposite_point_in_the_same_frame() {
    for instant in INSTANTS {
        let chart = found(instant);
        let rahu = chart.graha(Graha::Rahu).expect("Rahu");
        let ketu = chart.graha(Graha::Ketu).expect("Ketu");
        assert!(
            (difference_deg(ketu.longitude_deg, rahu.longitude_deg).abs() - 180.0).abs() < 1e-9,
            "the nodes are half a circle apart"
        );
        assert!(
            (ketu.speed_deg_per_day - rahu.speed_deg_per_day).abs() < 1e-12,
            "the nodes move together"
        );
        assert!(
            ketu.distance_au.abs() < f64::EPSILON,
            "a node has no distance of its own (entry 6)"
        );
        // And it is placed like any other graha, not copied from Rahu.
        assert_eq!(ketu.placement.method, chart.chalit.chalit.method);
    }
}

#[test]
fn the_day_the_chart_belongs_to_holds_its_instant() {
    for instant in INSTANTS {
        let chart = found(instant);
        let (from, to) = chart.day.part_bounds();
        assert!(
            (from.get()..=to.get()).contains(&chart.instant.get()),
            "the instant {} is outside its own part [{}, {}]",
            chart.instant.get(),
            from.get(),
            to.get()
        );
        assert!((0.0..=1.0).contains(&chart.day.elapsed));
        // The lagna's anchor is the sunrise that opened that day, which
        // is at or before the instant.
        assert!(chart.day.lagna_sunrise().get() <= chart.instant.get());
    }

    // The corpus's c001 is the case the module exists for: a birth
    // before sunrise belongs to the previous day's night.
    let before_dawn = found(INSTANTS[0]);
    assert!(!before_dawn.day.part.is_daylight());
    let after = found(INSTANTS[1]);
    assert!(after.day.part.is_daylight());
    assert!(
        after.day.lagna_sunrise().get() > before_dawn.day.lagna_sunrise().get(),
        "the two are anchored on different mornings"
    );
}

#[test]
fn founding_the_same_moment_twice_gives_the_same_value() {
    // The determinism contract, at this layer: a foundation is a plain
    // value and two of the same inputs are equal field for field.
    for instant in INSTANTS {
        assert_eq!(found(instant), found(instant));
    }

    // And a batch is its members, in order.
    let provider = TestProvider;
    let resolved = resolved();
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let founder = Founder::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    );
    let instants: Vec<JulianDay<Utc>> = INSTANTS.iter().map(|jd| JulianDay::literal(*jd)).collect();
    let stamped = founder
        .found(&instants, &place(), ChartKind::Natal)
        .unwrap_or_else(|e| panic!("{e}"));
    let batch = &stamped.value;
    assert_eq!(batch.len(), INSTANTS.len());
    // The stamp says what produced them.
    assert_eq!(stamped.provenance.profile, resolved.profile.as_str());
    assert_eq!(
        stamped.provenance.settings_hash,
        resolved.settings.hash(),
        "a change of settings is a change of hash"
    );
    assert!(!stamped.provenance.provider.name.is_empty());
    for (one, instant) in batch.iter().zip(INSTANTS) {
        assert_eq!(*one, found(instant));
    }
}

#[test]
fn the_chart_carries_both_divisions_and_says_which_is_which() {
    let chart = found(INSTANTS[2]);
    let settings = resolved().settings;
    // The default profile places by whole sign and reads its chalit as
    // Sripati (ADR-0024), which are different divisions and different
    // answers.
    assert_eq!(chart.houses.chalit.method, settings.houses.placement_system);
    assert_eq!(chart.chalit.chalit.method, settings.houses.chalit_system);
    assert_eq!(chart.chalit.chalit.method, HouseSystem::Sripati);
    assert_eq!(
        chart.chalit.chalit.source,
        Chalit::of(HouseSystem::Sripati).source,
        "Sripati reads Porphyry's cusps"
    );
    let apart = chart
        .houses
        .sandhi
        .iter()
        .zip(chart.chalit.sandhi.iter())
        .filter(|(a, b)| difference_deg(**a, **b).abs() > 1e-9)
        .count();
    assert!(apart > 0, "the two divisions are not the same boundaries");

    // Every graha carries both, and they may disagree — that is the
    // whole of the falsification pass.
    let differing = chart
        .grahas
        .iter()
        .filter(|g| g.placement.bhava != g.house.bhava)
        .count();
    println!(
        "{} of {} grahas fall in a different bhava under the chalit",
        differing,
        chart.grahas.len()
    );

    // And the bodies asked for are the eight the settings name, with
    // Ketu derived rather than requested.
    assert_eq!(bodies_of(&settings).len(), 8);
    assert_eq!(chart.grahas.len(), 9);
    assert!(chart.graha(Graha::Ketu).is_some());
}

#[test]
fn the_timing_is_counted_from_the_day_the_chart_belongs_to() {
    for instant in INSTANTS {
        let chart = found(instant);
        let timing = &chart.timing;
        // The ishtakaal is a count within a day: sixty ghatis, or sixty
        // and a little where a sunrise-to-sunrise day runs past
        // twenty-four hours under the civil reckoning.
        assert!(timing.ishtakaal.ghati <= 60, "{}", timing.ishtakaal.ghati);
        assert!(timing.ishtakaal.pala < 60);
        assert!(timing.ishtakaal.vipala < 60);
        assert_eq!(timing.ghati_reckoning, settings_of().day.ghati_reckoning);

        // The hora holds the instant and belongs to the same day.
        assert!((1..=24).contains(&timing.hora.number));
        assert!(
            timing.hora.start.get() <= chart.instant.get()
                && chart.instant.get() <= timing.hora.end.get(),
            "the hora does not hold the instant it was asked for"
        );
        assert_eq!(
            timing.hora.is_daytime(),
            chart.day.part.is_daylight(),
            "a daytime hora is a daytime birth"
        );
    }

    // The case the whole day module exists for: a birth before dawn is
    // counted from the previous morning, so its ishtakaal is late in the
    // day rather than early.
    let before_dawn = found(INSTANTS[0]);
    assert!(
        before_dawn.timing.ishtakaal.ghati > 30,
        "a birth in the small hours is late in its own day, not early: {} ghati",
        before_dawn.timing.ishtakaal.ghati
    );
    let by_day = found(INSTANTS[1]);
    assert!(
        by_day.timing.ishtakaal.ghati < 30,
        "and a daylight birth is early"
    );
}

#[test]
fn a_foundation_is_stamped_with_what_produced_it() {
    let provider = TestProvider;
    let resolved = resolved();
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let founder = Founder::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    );
    let stamped = founder
        .found_one(
            JulianDay::<Utc>::literal(INSTANTS[2]),
            &place(),
            ChartKind::Natal,
        )
        .unwrap_or_else(|e| panic!("{e}"));

    let stamp = &stamped.provenance;
    assert_eq!(stamp.profile, resolved.profile.as_str());
    assert_eq!(stamp.settings_hash, resolved.settings.hash());
    assert_eq!(
        stamp.calculation_version,
        teistro_core::envelope::CALCULATION_VERSION
    );
    assert_eq!(
        stamp.catalogue_version,
        teistro_core::catalogue::SCHEMA_VERSION
    );
    assert!(
        !stamp.provider.name.is_empty(),
        "the ephemeris that answered"
    );
    assert!(!stamp.provider.frame.is_empty(), "the frame it answered in");
    assert!(!stamp.time.delta_t_model.is_empty());
    assert!(!stamp.time.leap_table.is_empty());

    // The input hash tells two questions apart and two askings of one
    // question together.
    let again = founder
        .found_one(
            JulianDay::<Utc>::literal(INSTANTS[2]),
            &place(),
            ChartKind::Natal,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(again.provenance.input_hash, stamp.input_hash);
    let elsewhere = founder
        .found_one(
            JulianDay::<Utc>::literal(INSTANTS[3]),
            &place(),
            ChartKind::Natal,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(elsewhere.provenance.input_hash, stamp.input_hash);
}

/// The settings the tests found charts under.
fn settings_of() -> teistro_core::settings::Settings {
    resolved().settings
}

#[test]
fn every_shipped_profile_founds_a_chart_from_a_place() {
    // Two of the five shipped profiles are topocentric, which is what
    // `nepali-default` inherited from the baseline engine. Until the
    // completion's centre step they could not found a chart at all: the
    // provider was asked for a frame it does not answer, and the refusal
    // reached the consumer as a capability error on a profile the SDK
    // ships (`03-design/topocentric-measured.md`).
    let provider = TestProvider;
    let mut made = 0_usize;
    let mut from_a_place = 0_usize;
    for id in teistro_core::settings::SHIPPED_PROFILES {
        let resolved = Profile::shipped(id)
            .unwrap_or_else(|| panic!("the shipped profile {id}"))
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{id}: {e}"));
        let placed = resolved.settings.frame.centre == teistro_core::settings::Centre::Topocentric;
        let model = DrikSun::new(
            &provider,
            Ayanamsha::Lahiri,
            Sunrise::CentreNoRefraction.into(),
            OverridePolicy::PreferNative,
            DeltaTModel::TableThenModel,
        );
        let clock = UtcOffset::literal(5, 45, 0);
        let founder = Founder::new(
            &provider,
            &resolved,
            &model,
            &Gregorian,
            &clock,
            PrecessionModel::Vondrak2011,
            DeltaTModel::TableThenModel,
        );
        let outcome = match founder.found_one(
            JulianDay::<Utc>::literal(INSTANTS[2]),
            &place(),
            ChartKind::Natal,
        ) {
            Ok(chart) => chart,
            // The analytic test provider answers eight bodies and no true
            // node, so `kp-default` cannot be founded over it. That is a
            // provider's coverage and not the centre's doing, and the
            // difference is the point: the refusal must name a body.
            Err(error) => {
                let said = error.to_string();
                assert!(
                    said.contains("NODE") || said.contains("does not support"),
                    "{id}: {said}"
                );
                assert!(
                    !said.contains("centre") && !said.contains("TOPOCENTRIC"),
                    "{id} was refused for its centre: {said}"
                );
                continue;
            }
        };
        made += 1;
        let chart = &outcome.value;
        // Every body the settings ask the provider for, and Ketu, which
        // no provider answers because it is Rahu's opposite point.
        assert_eq!(
            chart.grahas.len(),
            bodies_of(&resolved.settings).len() + 1,
            "{id}"
        );
        // A chart cast from a place says so in the frame it stamps, and
        // the step that put it there is one of the steps it lists.
        let frame = &outcome.provenance.provider.frame;
        assert_eq!(
            frame.contains("TOPOCENTRIC"),
            placed,
            "{id}: the frame is {frame}"
        );
        assert_eq!(
            chart.steps.iter().any(|step| step.starts_with("centre:")),
            placed,
            "{id}: the steps are {:?}",
            chart.steps
        );
        from_a_place += usize::from(placed);
    }
    // Three of the five shipped profiles found a chart over this
    // provider and two of the three are cast from a place; the other two
    // ask for the true node, which the analytic provider does not answer.
    // Before the centre step none of the placed ones could be founded at
    // all.
    assert!(
        made >= 3 && from_a_place >= 2,
        "{made} profiles founded, {from_a_place} of them from a place"
    );
}

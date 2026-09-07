//! The houses of a founded chart, end to end over the analytic test
//! provider.
//!
//! The rules are held to the corpus in `baseline.rs`. What only the
//! assembled value can show is that the parts agree with each other —
//! that a bhava's middle lies inside it, that both readings place every
//! body, and that a degenerate chart is reported rather than refused.
//!
//! And, as `aspect` and `points` do, that **every shipped profile
//! answers**, with the module override actually taking effect: that knob
//! had no reader at all until this crate.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index their own results and print counts under --nocapture"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::houses::Outcome;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::Gregorian;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::foundation::{ChartFoundation, Founder};
use teistro_core::catalogue::{Ayanamsha, ChartKind, Graha, HouseSystem};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{
    OverridePolicy, Profile, SHIPPED_PROFILES, Settings, SettingsPatch, Sunrise,
};
use teistro_core::time::UtcOffset;
use teistro_houses::Houses;
use teistro_houses::classify::{HOUSES, Quadrant};
use teistro_houses::system::{Purpose, system_for};

fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

fn founded_on(profile: &str) -> Result<(ChartFoundation, Settings), Error> {
    let provider = teistro_port_ephemeris::test_provider::TestProvider;
    let resolved = Profile::shipped(profile)
        .unwrap_or_else(|| panic!("the profile `{profile}`"))
        .resolve(&SettingsPatch::default())
        .unwrap_or_else(|e| panic!("{profile}: {e}"));
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let foundation = Founder::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    )
    .found_one(
        JulianDay::<Utc>::literal(2_460_482.5),
        &place(),
        ChartKind::Natal,
    )?
    .value;
    Ok((foundation, resolved.settings))
}

fn founded() -> (ChartFoundation, Settings) {
    founded_on(teistro_core::settings::DEFAULT_PROFILE).expect("the default profile founds")
}

#[test]
fn every_shipped_profile_answers_which_system_it_uses() {
    // No provider and no chart: a profile and a settings lookup are the
    // whole of it, which is the cheapest guard against the failure
    // `state` met (registry entry 23).
    for profile in SHIPPED_PROFILES {
        let settings = Profile::shipped(profile)
            .expect("a shipped profile")
            .resolve(&SettingsPatch::default())
            .expect("it resolves")
            .settings;
        for purpose in [Purpose::Placement, Purpose::Chalit] {
            let system = system_for(&settings, purpose, None);
            assert!(
                HouseSystem::ALL.contains(&system),
                "{profile} {purpose:?}: {system:?}"
            );
        }
        println!(
            "{profile}: placement {:?}, chalit {:?}",
            system_for(&settings, Purpose::Placement, None),
            system_for(&settings, Purpose::Chalit, None)
        );
    }
}

#[test]
fn the_shipped_override_for_kp_is_read_and_no_other_module_moves() {
    // The root *populates* this knob — every shipped profile carries
    // `kp` to Placidus — and nothing had ever asked for it. Under the
    // default profile the rest of the chart is whole-sign, so the
    // override is a real difference and not a restatement.
    let (_, mut settings) = founded();
    assert_eq!(
        system_for(&settings, Purpose::Placement, Some("kp")),
        HouseSystem::Placidus,
        "the shipped override"
    );
    assert_eq!(
        system_for(&settings, Purpose::Placement, None),
        HouseSystem::WholeSign,
        "and it differs from the chart's own"
    );
    // A module with no override of its own takes the chart's.
    assert_eq!(
        system_for(&settings, Purpose::Placement, Some("jaimini")),
        system_for(&settings, Purpose::Placement, None)
    );
    // A new override takes effect and moves nothing else.
    settings
        .houses
        .module_overrides
        .insert(String::from("tajika"), HouseSystem::Campanus);
    assert_eq!(
        system_for(&settings, Purpose::Placement, Some("tajika")),
        HouseSystem::Campanus
    );
    assert_eq!(
        system_for(&settings, Purpose::Placement, Some("kp")),
        HouseSystem::Placidus
    );
    assert_eq!(
        system_for(&settings, Purpose::Chalit, None),
        settings.houses.chalit_system
    );
}

#[test]
fn every_bhava_has_a_middle_inside_it_and_a_lord() {
    let (foundation, _) = founded();
    let houses = Houses::of(&foundation).expect("a founded chart");
    assert_eq!(houses.all().len(), usize::from(HOUSES));
    for (index, bhava) in houses.all().iter().enumerate() {
        assert_eq!(bhava.number, u8::try_from(index + 1).unwrap());
        assert_eq!(houses.bhava(bhava.number), Some(bhava));
        assert!((0.0..360.0).contains(&bhava.madhya_deg));
        assert!((0.0..360.0).contains(&bhava.sandhi_deg));
        // The lord is the lord of the sign the middle falls in.
        assert_eq!(bhava.lord, bhava.sign.attributes().lord);
        assert!(Graha::ALL.contains(&bhava.lord));
    }
    // Nothing outside the twelve is a bhava.
    assert_eq!(houses.bhava(0), None);
    assert_eq!(houses.bhava(13), None);
    println!(
        "twelve bhavas under {:?}, chalit {:?}",
        houses.placement_system(),
        houses.chalit_system()
    );
}

#[test]
fn the_quadrants_partition_the_twelve_bhavas_of_a_chart() {
    let (foundation, _) = founded();
    let houses = Houses::of(&foundation).expect("a founded chart");
    let mut counted = 0;
    for quadrant in Quadrant::ALL {
        let found: Vec<u8> = houses
            .quadrant(quadrant)
            .map(|bhava| bhava.number)
            .collect();
        assert_eq!(found.len(), 4, "{quadrant:?}");
        assert_eq!(found, quadrant.houses().to_vec(), "{quadrant:?}");
        counted += found.len();
    }
    assert_eq!(counted, usize::from(HOUSES), "every bhava in one quadrant");
    // And the overlapping kinds cut across them.
    let trines: Vec<u8> = houses
        .all()
        .iter()
        .filter(|bhava| bhava.is_trikona())
        .map(|bhava| bhava.number)
        .collect();
    assert_eq!(trines, vec![1, 5, 9]);
}

#[test]
fn both_readings_place_every_body_and_the_moved_are_the_ones_that_differ() {
    let (foundation, _) = founded();
    let houses = Houses::of(&foundation).expect("a founded chart");
    assert_eq!(houses.bodies().len(), foundation.grahas.len());
    for (placed, position) in houses.bodies().iter().zip(&foundation.grahas) {
        assert_eq!(placed.graha, position.graha);
        assert_eq!(placed.placement, position.house.bhava);
        assert_eq!(placed.chalit, position.placement.bhava);
        assert!((1..=HOUSES).contains(&placed.placement));
        assert!((1..=HOUSES).contains(&placed.chalit));
        assert_eq!(placed.shifted(), placed.placement != placed.chalit);
        assert_eq!(houses.placed(placed.graha), Some(placed));
    }
    let moved: Vec<Graha> = houses.moved().map(|placed| placed.graha).collect();
    for graha in &moved {
        let placed = houses.placed(*graha).expect("a placed body");
        assert!(placed.shifted(), "{graha:?}");
    }
    assert_eq!(
        moved.len(),
        houses.bodies().iter().filter(|p| p.shifted()).count()
    );
    println!(
        "{} bodies, {} moved by the chalit",
        houses.bodies().len(),
        moved.len()
    );
}

#[test]
fn a_degenerate_chart_is_reported_and_not_refused() {
    let (foundation, _) = founded();
    let plain = Houses::of(&foundation).expect("a founded chart");
    assert!(plain.is_defined());
    assert_eq!(plain.outcome(), Outcome::Defined);
    // A chart whose system had to be substituted still computes; the
    // outcome says so, and the caller decides.
    let substituted = Houses::with_outcome(
        &foundation,
        Outcome::Substituted {
            asked: HouseSystem::Placidus,
        },
    )
    .expect("a substituted chart is still a chart");
    assert!(!substituted.is_defined());
    assert_eq!(
        substituted.outcome(),
        Outcome::Substituted {
            asked: HouseSystem::Placidus
        }
    );
    // And the bhavas are the same ones: the outcome is a report, not a
    // different computation.
    assert_eq!(substituted.all(), plain.all());
}

#[test]
fn a_chart_the_provider_cannot_found_fails_for_the_provider_s_reason() {
    let mut computed = 0;
    for profile in SHIPPED_PROFILES {
        match founded_on(profile) {
            Ok((foundation, _)) => {
                Houses::of(&foundation).unwrap_or_else(|e| panic!("{profile}: {e}"));
                computed += 1;
            }
            Err(error) => assert_eq!(error.status, Status::Unsupported, "{profile}: {error}"),
        }
    }
    assert!(computed >= 1, "the default profile founds and answers");
}

#[test]
fn two_runs_of_the_same_chart_are_the_same_value() {
    let (foundation, _) = founded();
    let once = Houses::of(&foundation).expect("a founded chart");
    let twice = Houses::of(&foundation).expect("a founded chart");
    assert_eq!(once, twice, "the determinism contract");
    let json = serde_json::to_string(&once).expect("a serialisable value");
    assert!(json.contains("KENDRA"), "{json:.200}");
}

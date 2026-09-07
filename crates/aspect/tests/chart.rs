//! Every relation of a founded chart, end to end over the analytic test
//! provider.
//!
//! The kernel's rules are exhausted in `exhaustive.rs` and their counts
//! held to the corpus in `corpus.rs`. What only the assembled value can
//! show is that the parts agree with each other — that a relation is
//! the relation of the position the foundation gave it, that mutuality
//! is symmetric across a whole chart, and that the entry point refuses
//! what it cannot answer.
//!
//! And one thing no other test can show: that **every shipped profile
//! founds a chart and computes its aspects**. `crates/state` shipped
//! with the SDK's own default profile naming a combustion table nothing
//! resolved (registry entry 23), which failed only when a chart was
//! founded on it. `aspect.drishti_table` is the same shape of knob, so
//! the same failure is tested for here rather than waited for.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index their own results and print counts under --nocapture"
)]

use teistro_aspect::drishti::{PARASHARA, Strength, house_count};
use teistro_aspect::{Aspects, rashi};
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::Gregorian;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::foundation::{ChartFoundation, Founder};
use teistro_core::catalogue::{Ayanamsha, ChartKind, Graha, Rashi};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{
    OverridePolicy, Profile, SHIPPED_PROFILES, Settings, SettingsPatch, Sunrise, root,
};
use teistro_core::time::UtcOffset;
use teistro_port_ephemeris::test_provider::TestProvider;

fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

fn resolved(profile: &str) -> Settings {
    Profile::shipped(profile)
        .unwrap_or_else(|| panic!("the profile `{profile}`"))
        .resolve(&SettingsPatch::default())
        .unwrap_or_else(|e| panic!("{profile}: {e}"))
        .settings
}

fn founded_on(profile: &str) -> Result<(ChartFoundation, Settings), Error> {
    let provider = TestProvider;
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

/// Every shipped profile names a drishti table this crate resolves.
///
/// This is the `state` lesson as a test. `parashari-classical` had named
/// a combustion table nothing shipped since ADR-0024, and nothing caught
/// it until a chart was founded on the default profile
/// (`05-testing/01-golden-vectors.md`, entry 23). It needs no provider
/// and no chart: a profile and a table lookup are the whole of it.
#[test]
fn every_shipped_profile_names_a_drishti_table_that_resolves() {
    for profile in SHIPPED_PROFILES {
        let settings = resolved(profile);
        let table = teistro_aspect::drishti::table(&settings.aspect.drishti_table)
            .unwrap_or_else(|e| panic!("{profile} names a drishti table nothing resolves: {e}"));
        assert_eq!(table, PARASHARA, "{profile}");
    }
    // And the root's own value, which every profile inherits unless it
    // patches over it.
    assert_eq!(
        teistro_aspect::drishti::table(&root().aspect.drishti_table).expect("the root's"),
        PARASHARA
    );
}

/// And every profile the test provider can found a chart under actually
/// computes its aspects.
///
/// Some shipped profiles ask the provider for more than the analytic
/// test provider has — a topocentric frame, whose completion step is
/// Phase 3's to write, or the true node, which it does not carry. Those
/// are reported rather than skipped silently, and the test fails if a
/// profile fails for any reason that is not the provider's limits.
#[test]
fn every_profile_the_provider_can_found_computes_its_aspects() {
    let mut computed = Vec::new();
    let mut deferred = Vec::new();
    for profile in SHIPPED_PROFILES {
        match founded_on(profile) {
            Ok((foundation, settings)) => {
                let aspects = Aspects::of(&foundation, &settings)
                    .unwrap_or_else(|e| panic!("{profile}: {e}"));
                assert_eq!(aspects.table(), PARASHARA, "{profile}");
                assert!(!aspects.all().is_empty(), "{profile}");
                println!("{profile}: {} relations", aspects.all().len());
                computed.push(profile);
            }
            Err(error) => {
                assert_eq!(
                    error.status,
                    Status::Unsupported,
                    "{profile} failed for a reason that is not a provider's limit: {error}"
                );
                assert!(
                    error.message.contains("provider") || error.message.contains("completion"),
                    "{profile}: {error}"
                );
                // Whatever the provider cannot do, the profile's own
                // drishti table still has to resolve.
                let settings = resolved(profile);
                assert!(
                    teistro_aspect::drishti::table(&settings.aspect.drishti_table).is_ok(),
                    "{profile}"
                );
                deferred.push(profile);
            }
        }
    }
    println!("computed under {computed:?}; {deferred:?} ask the provider for more than it has");
    assert!(
        computed.contains(&teistro_core::settings::DEFAULT_PROFILE),
        "the default profile founds and computes, whatever the others need"
    );
    assert_eq!(
        computed.len() + deferred.len(),
        SHIPPED_PROFILES.len(),
        "every profile was tried"
    );
}

#[test]
fn a_relation_is_the_relation_of_the_positions_the_foundation_gave() {
    let (foundation, settings) = founded();
    let aspects = Aspects::of(&foundation, &settings).expect("a founded chart");
    for relation in aspects.all() {
        let from = foundation.graha(relation.from).expect("a placed body");
        let to = foundation.graha(relation.to).expect("a placed body");
        assert_eq!(
            aspects.sign(relation.from),
            Rashi::from_id(u16::from(from.sign_index())),
            "{:?}",
            relation.from
        );
        assert_eq!(
            relation.houses,
            house_count(
                aspects.sign(relation.from).expect("placed"),
                aspects.sign(relation.to).expect("placed")
            )
        );
        assert!(relation.strength.is_any(), "only real relations are kept");
        assert_ne!(relation.from, relation.to, "nothing aspects itself");
        // The edges are facts about those same longitudes.
        assert!(relation.from_edge.sign_deg >= 0.0);
        assert!(relation.to_edge.sign_deg <= 15.0);
        assert!(relation.nearest_edge_deg() <= relation.from_edge.sign_deg);
        // A relation this chart holds is firm at a hair and not at half
        // a sign.
        assert!(relation.is_firm(1e-9) || relation.nearest_edge_deg() <= 1e-9);
        assert!(!relation.is_firm(15.0));
        let _ = to;
    }
    println!(
        "{} relations, {} of them full",
        aspects.all().len(),
        aspects
            .all()
            .iter()
            .filter(|r| r.strength.is_full())
            .count()
    );
}

#[test]
fn a_body_casts_and_receives_the_relations_the_accessors_find() {
    let (foundation, settings) = founded();
    let aspects = Aspects::of(&foundation, &settings).expect("a founded chart");
    let mut cast = 0;
    let mut received = 0;
    for position in &foundation.grahas {
        let graha = position.graha;
        for relation in aspects.cast_by(graha) {
            assert_eq!(relation.from, graha);
            assert!(aspects.aspects(graha, relation.to));
            assert_eq!(aspects.between(graha, relation.to), Some(relation));
            cast += 1;
        }
        for relation in aspects.on(graha) {
            assert_eq!(relation.to, graha);
            received += 1;
        }
        // The strongest is at least as strong as any other on it.
        if let Some(strongest) = aspects.strongest_on(graha) {
            for relation in aspects.on(graha) {
                assert!(strongest.strength >= relation.strength, "{graha:?}");
            }
        } else {
            assert_eq!(aspects.on(graha).count(), 0, "{graha:?}");
        }
    }
    assert_eq!(cast, aspects.all().len(), "every relation is cast by one");
    assert_eq!(received, aspects.all().len(), "and received by one");
    assert!(!aspects.aspects(Graha::Sun, Graha::Sun), "not itself");
}

#[test]
fn a_mutual_relation_is_listed_once_and_agrees_from_both_ends() {
    let (foundation, settings) = founded();
    let aspects = Aspects::of(&foundation, &settings).expect("a founded chart");
    let mut seen = Vec::new();
    for mutual in aspects.mutual() {
        assert_ne!(mutual.first, mutual.second);
        let pair = (mutual.first, mutual.second);
        let flipped = (mutual.second, mutual.first);
        assert!(
            !seen.contains(&pair) && !seen.contains(&flipped),
            "{pair:?}"
        );
        seen.push(pair);
        // Both directions exist and carry what the pair says.
        let outward = aspects
            .between(mutual.first, mutual.second)
            .expect("the outward relation");
        let back = aspects
            .between(mutual.second, mutual.first)
            .expect("and the one back");
        assert_eq!(outward.strength, mutual.outward);
        assert_eq!(back.strength, mutual.back);
        assert_eq!(outward.houses, mutual.houses);
        assert_eq!(
            mutual.is_full(),
            outward.strength.is_full() && back.strength.is_full()
        );
        // A mutual **full** aspect is the seventh, or Mars and Saturn.
        if mutual.is_full() && mutual.houses != 7 {
            let pair = [mutual.first, mutual.second];
            assert!(
                pair.contains(&Graha::Mars) && pair.contains(&Graha::Saturn),
                "{pair:?} across the {}th",
                mutual.houses
            );
        }
    }
    println!("{} mutual relations", seen.len());
}

#[test]
fn a_conjunction_is_symmetric_and_never_a_drishti() {
    let (foundation, settings) = founded();
    let aspects = Aspects::of(&foundation, &settings).expect("a founded chart");
    for position in &foundation.grahas {
        let graha = position.graha;
        for other in aspects.conjunct(graha) {
            assert_ne!(other, graha);
            assert!(
                aspects.conjunct(other).any(|back| back == graha),
                "{graha:?}"
            );
            assert_eq!(aspects.sign(graha), aspects.sign(other));
            // Sharing a sign is never a drishti, either way about.
            assert!(!aspects.aspects(graha, other), "{graha:?} and {other:?}");
            assert!(!aspects.aspects(other, graha));
        }
    }
}

#[test]
fn the_rashi_reading_is_the_signs_and_is_always_mutual() {
    let (foundation, settings) = founded();
    let aspects = Aspects::of(&foundation, &settings).expect("a founded chart");
    for first in &foundation.grahas {
        for second in &foundation.grahas {
            let there = aspects.rashi_aspects(first.graha, second.graha);
            assert_eq!(
                there,
                aspects.rashi_aspects(second.graha, first.graha),
                "{:?} and {:?}",
                first.graha,
                second.graha
            );
            let (Some(from), Some(to)) = (aspects.sign(first.graha), aspects.sign(second.graha))
            else {
                panic!("both are placed");
            };
            assert_eq!(there, rashi::aspects(from, to));
        }
    }
}

#[test]
fn a_drishti_table_the_sdk_does_not_ship_is_refused_by_name() {
    let (foundation, mut settings) = founded();
    settings.aspect.drishti_table = String::from("SOMEONE_ELSES");
    let error = Aspects::of(&foundation, &settings).expect_err("no such table");
    assert!(error.message.contains("SOMEONE_ELSES"), "{error}");
    assert!(error.message.contains(PARASHARA), "{error}");
    assert_eq!(error.field(), Some("aspect.drishti_table"));
}

#[test]
fn two_runs_of_the_same_chart_are_the_same_value() {
    let (foundation, settings) = founded();
    let once = Aspects::of(&foundation, &settings).expect("a founded chart");
    let twice = Aspects::of(&foundation, &settings).expect("a founded chart");
    assert_eq!(once, twice, "the determinism contract");
    // And the value serialises, because everything above it stores one.
    let json = serde_json::to_string(&once).expect("a serialisable value");
    assert!(
        json.contains("FULL") || json.contains("HALF"),
        "{json:.200}"
    );
}

#[test]
fn a_chart_with_one_body_has_no_relations_and_is_not_an_error() {
    let (mut foundation, settings) = founded();
    foundation.grahas.truncate(1);
    let aspects = Aspects::of(&foundation, &settings).expect("one body is a chart");
    assert!(aspects.all().is_empty(), "nothing to look at");
    assert_eq!(aspects.mutual().count(), 0);
    assert_eq!(aspects.strongest_on(Graha::Sun), None);
    foundation.grahas.clear();
    let empty = Aspects::of(&foundation, &settings).expect("and none is too");
    assert!(empty.all().is_empty());
    assert_eq!(empty.sign(Graha::Sun), None);
    assert!(!empty.rashi_aspects(Graha::Sun, Graha::Moon));
    assert_eq!(Strength::None.quarters(), 0);
}

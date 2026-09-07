//! Every recorded state reading of the conformance corpus.
//!
//! The whole of this crate is arithmetic over a founded chart, so the
//! corpus decides all of it without a provider: the recorded longitudes,
//! latitudes, speeds and houses are the input, and the recorded dignity,
//! friendships, combustion, age, wakefulness, war and lajjitadi are the
//! answer. 837 readings of nine grahas over 93 fixtures.
//!
//! Two differences are asserted **as** differences rather than skipped:
//! the deeply debilitated body the engine records as dreaming rather than
//! asleep, and the lagna, which the engine gives placeholder friendships
//! to and this crate does not treat as a graha at all.
//!
//! `cargo xtask state` measures the same rules against the same corpus
//! without using this crate, and writes what it found to
//! `03-design/state-tables-measured.md`. That pass falsifies the design
//! before the code exists; this test holds the code to it afterwards.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index JSON by key and print the measurement under --nocapture"
)]

use std::path::Path;

use serde_json::Value;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{
    AvasthaBaladi, AvasthaJagradadi, AvasthaLajjitadi, Dignity, Graha, Rashi, Relationship,
};
use teistro_core::quantity::Degrees;
use teistro_state::avastha::{self, AtWar, Placement};
use teistro_state::boundary::Boundaries;
use teistro_state::burn::{self, BPHS, Burning, SURYA_SIDDHANTA};
use teistro_state::dignity::{self, Friendship};

/// The nine grahas the engine gives a state to.
const GRAHAS: [&str; 9] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU",
];

/// One fixture, read into the shapes the crate takes.
struct Chart {
    name: String,
    bodies: Value,
    signs: Vec<(Graha, Rashi)>,
    placements: Vec<Placement>,
    fighters: Vec<AtWar>,
}

fn at(degrees: f64) -> Nas {
    Nas::from_degrees(Degrees::try_new(degrees.rem_euclid(360.0)).expect("a finite longitude"))
}

fn charts() -> Vec<Chart> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline");
    let mut charts = Vec::new();
    for directory in ["charts", "variants"] {
        let dir = root.join(directory);
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| {
                panic!(
                    "{}: {e}. The corpus is a submodule; `git submodule update --init`",
                    dir.display()
                )
            })
            .map(|entry| entry.expect("an entry").path())
            .filter(|path| path.extension().is_some_and(|e| e == "json"))
            .collect();
        paths.sort();
        for path in &paths {
            let value: Value =
                serde_json::from_str(&std::fs::read_to_string(path).expect("a fixture"))
                    .expect("valid JSON");
            if !value["positions"]["bodies"].is_object() {
                continue;
            }
            let name = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_default();
            charts.push(build(name, &value["positions"]["bodies"]));
        }
    }
    assert!(!charts.is_empty(), "the corpus carries positions");
    charts
}

/// One fixture's signs, placements and fighters, in the crate's terms.
fn build(name: String, bodies: &Value) -> Chart {
    let mut signs = Vec::new();
    let mut fighters = Vec::new();
    for body in GRAHAS {
        let Some(graha) = Graha::from_key(body) else {
            continue;
        };
        let longitude = bodies[body]["sidereal_longitude_deg"]
            .as_f64()
            .unwrap_or_else(|| panic!("{name} {body}: no longitude"));
        signs.push((graha, at(longitude).sign()));
        fighters.push(AtWar {
            graha,
            longitude_deg: longitude.rem_euclid(360.0),
            latitude_deg: bodies[body]["latitude_deg"].as_f64().unwrap_or(0.0),
        });
    }
    let sign_of = |graha: Graha| {
        signs
            .iter()
            .find(|(who, _)| *who == graha)
            .map(|(_, sign)| *sign)
    };
    let mut placements = Vec::new();
    for body in GRAHAS {
        let Some(graha) = Graha::from_key(body) else {
            continue;
        };
        let longitude = at(bodies[body]["sidereal_longitude_deg"].as_f64().unwrap());
        let sign = longitude.sign();
        let friendship = dignity::friendship(graha, sign, sign_of);
        placements.push(Placement {
            graha,
            sign,
            house: u8::try_from(bodies[body]["house"].as_u64().unwrap_or(0)).unwrap_or(0),
            dignity: dignity::dignity(
                graha,
                sign,
                longitude.in_sign().to_degrees(),
                friendship.compound,
            ),
        });
    }
    Chart {
        name,
        bodies: bodies.clone(),
        signs,
        placements,
        fighters,
    }
}

impl Chart {
    fn friendship(&self, graha: Graha, sign: Rashi) -> Friendship {
        let signs = &self.signs;
        dignity::friendship(graha, sign, |who| {
            signs
                .iter()
                .find(|(other, _)| *other == who)
                .map(|(_, sign)| *sign)
        })
    }

    fn placement(&self, graha: Graha) -> Placement {
        *self
            .placements
            .iter()
            .find(|placement| placement.graha == graha)
            .unwrap_or_else(|| panic!("{}: no {graha:?}", self.name))
    }
}

/// The key a catalogue member is recorded under, case-insensitively.
fn same(recorded: &Value, key: &str) -> bool {
    recorded
        .as_str()
        .is_some_and(|text| text.eq_ignore_ascii_case(key))
}

#[test]
fn every_recorded_dignity_and_friendship_is_the_one_the_ladder_gives() {
    let mut checked = 0;
    let mut charts_seen = 0;
    for chart in charts() {
        charts_seen += 1;
        for body in GRAHAS {
            let graha = Graha::from_key(body).expect("a catalogued graha");
            let placement = chart.placement(graha);
            let friendship = chart.friendship(graha, placement.sign);
            let recorded = &chart.bodies[body];
            assert!(
                same(&recorded["dignity"], placement.dignity.key()),
                "{} {body}: dignity {:?} against {}",
                chart.name,
                placement.dignity,
                recorded["dignity"]
            );
            assert!(
                same(&recorded["natural_friendship"], friendship.natural.key()),
                "{} {body}: natural friendship",
                chart.name
            );
            assert!(
                same(
                    &recorded["temporary_friendship"],
                    friendship.temporary.key()
                ),
                "{} {body}: temporary friendship",
                chart.name
            );
            assert!(
                same(&recorded["panchadha_maitri"], friendship.compound.key()),
                "{} {body}: the compound",
                chart.name
            );
            checked += 1;
        }
    }
    println!("{checked} dignities and friendships over {charts_seen} fixtures");
    assert_eq!(charts_seen, 93);
    assert_eq!(checked, 837);
}

#[test]
fn every_recorded_combustion_is_the_one_the_orbs_give() {
    let orbs = burn::table(BPHS).expect("the shipped table");
    let mut checked = 0;
    let mut burnt = 0;
    for chart in charts() {
        let sun = chart
            .fighters
            .iter()
            .find(|body| body.graha == Graha::Sun)
            .map(|body| body.longitude_deg);
        for body in GRAHAS {
            let graha = Graha::from_key(body).expect("a catalogued graha");
            let recorded = &chart.bodies[body];
            let longitude = recorded["sidereal_longitude_deg"].as_f64().unwrap();
            let from_sun = (graha != Graha::Sun)
                .then(|| sun.map(|sun| avastha::separation(longitude, sun)))
                .flatten();
            // The corpus records the same distance, so a units slip is
            // caught before the classification is.
            if let (Some(ours), Some(theirs)) = (from_sun, recorded["degrees_from_sun"].as_f64()) {
                assert!(
                    (ours - theirs).abs() < 1e-9,
                    "{} {body}: {ours} against {theirs}",
                    chart.name
                );
            }
            let found = burn::combustion(
                graha,
                from_sun,
                recorded["is_retrograde"].as_bool().unwrap_or(false),
                orbs,
            );
            let expected = match recorded["combust"].as_str() {
                Some("combust") => Burning::Combust,
                Some("deep-combust") => Burning::Deep,
                _ => Burning::None,
            };
            assert_eq!(found.burning, expected, "{} {body}", chart.name);
            burnt += usize::from(found.is_combust());
            checked += 1;
        }
    }
    println!("{checked} combustion readings, {burnt} of them burnt");
    assert_eq!(checked, 837);
    assert_eq!(burnt, 66, "the corpus records sixty-six");
}

/// The default profile names the Surya Siddhanta's table, which gives no
/// orb inside its orb. Over the whole corpus that changes one thing and
/// only in the safe direction: the same sixty-six bodies burn, and the
/// thirty-six the engine calls deeply combust come back merely combust.
#[test]
fn the_texts_table_burns_the_same_bodies_and_never_deeply() {
    let text = burn::table(SURYA_SIDDHANTA).expect("the shipped table");
    let parashari = burn::table(BPHS).expect("the shipped table");
    let mut burnt = 0;
    let mut shallower = 0;
    for chart in charts() {
        let sun = chart
            .fighters
            .iter()
            .find(|body| body.graha == Graha::Sun)
            .map(|body| body.longitude_deg);
        for body in GRAHAS {
            let graha = Graha::from_key(body).expect("a catalogued graha");
            let recorded = &chart.bodies[body];
            let longitude = recorded["sidereal_longitude_deg"].as_f64().unwrap();
            let from_sun = (graha != Graha::Sun)
                .then(|| sun.map(|sun| avastha::separation(longitude, sun)))
                .flatten();
            let retrograde = recorded["is_retrograde"].as_bool().unwrap_or(false);
            let found = burn::combustion(graha, from_sun, retrograde, text);
            let deeper = burn::combustion(graha, from_sun, retrograde, parashari);
            assert_ne!(found.burning, Burning::Deep, "{} {body}", chart.name);
            assert_eq!(
                found.is_combust(),
                deeper.is_combust(),
                "{} {body}: the outer orb is the same in both",
                chart.name
            );
            burnt += usize::from(found.is_combust());
            shallower += usize::from(found.burning != deeper.burning);
        }
    }
    println!("{burnt} burnt, {shallower} of them deeply under the other table");
    assert_eq!(burnt, 66, "the same sixty-six the engine records");
    assert_eq!(shallower, 36, "and thirty-six the engine calls deep");
}

#[test]
fn every_recorded_age_wakefulness_and_lajjitadi_is_the_one_the_rules_give() {
    let mut ages = 0;
    let mut wakefulness = 0;
    let mut lajjitadi = 0;
    let mut deep_debilitated = 0;
    for chart in charts() {
        for body in GRAHAS {
            let graha = Graha::from_key(body).expect("a catalogued graha");
            let placement = chart.placement(graha);
            let recorded = &chart.bodies[body];
            let longitude = at(recorded["sidereal_longitude_deg"].as_f64().unwrap());

            let age = avastha::age(placement.sign, longitude.in_sign().to_degrees());
            assert!(
                same(&recorded["avastha_baladi"], age.key()),
                "{} {body}: age {age:?} against {}",
                chart.name,
                recorded["avastha_baladi"]
            );
            ages += 1;
            let _: AvasthaBaladi = age;

            let avasthas = &recorded["avasthas"];
            if avasthas.is_null() {
                continue;
            }
            let ours = avastha::wakefulness(placement.dignity);
            if placement.dignity == Dignity::DeepDebilitated {
                // Registry: the engine lets the deeper state fall through
                // to the milder one. Asserted as a difference.
                deep_debilitated += 1;
                assert_eq!(ours, AvasthaJagradadi::Sushupti);
                assert!(
                    same(&avasthas["jagradadi"], AvasthaJagradadi::Swapna.key()),
                    "{} {body}: the engine records it dreaming",
                    chart.name
                );
            } else {
                assert!(
                    same(&avasthas["jagradadi"], ours.key()),
                    "{} {body}: wakefulness",
                    chart.name
                );
                wakefulness += 1;
            }

            // The three lajjitadi the chart decides, each way round.
            let found = avastha::lajjitadi(placement, &chart.placements);
            let theirs: Vec<String> = avasthas["lajjitadi"]
                .as_array()
                .map(|list| {
                    list.iter()
                        .filter_map(|name| name.as_str().map(ToString::to_string))
                        .collect()
                })
                .unwrap_or_default();
            for state in [
                AvasthaLajjitadi::Garvita,
                AvasthaLajjitadi::Lajjita,
                AvasthaLajjitadi::Kshobhita,
            ] {
                assert_eq!(
                    found.holding.contains(&state),
                    theirs.iter().any(|name| name == state.key()),
                    "{} {body}: {state:?}",
                    chart.name
                );
            }
            lajjitadi += 1;
        }
    }
    println!(
        "{ages} ages, {wakefulness} wakefulness readings, {lajjitadi} lajjitadi, and {deep_debilitated} deep debilitations recorded dreaming"
    );
    assert_eq!(ages, 837);
    assert_eq!(lajjitadi, 651, "the nodes carry no avasthas");
    assert_eq!(deep_debilitated, 3, "the registry's own count");
}

#[test]
fn every_recorded_war_is_found_with_the_same_victor() {
    let mut wars = 0;
    let mut found_wars = 0;
    for chart in charts() {
        for body in GRAHAS {
            let graha = Graha::from_key(body).expect("a catalogued graha");
            let recorded = &chart.bodies[body]["avasthas"]["planetary_war"];
            let ours = avastha::war(graha, &chart.fighters);
            if let Some(war) = recorded.as_object() {
                wars += 1;
                let ours = ours.unwrap_or_else(|| panic!("{} {body}: no war found", chart.name));
                assert!(
                    same(&war["opponent"], ours.opponent.key()),
                    "{} {body}: the opponent",
                    chart.name
                );
                assert_eq!(
                    ours.is_winner,
                    war["is_winner"].as_bool().unwrap_or(false),
                    "{} {body}: the victor",
                    chart.name
                );
                assert!(ours.apart_deg < avastha::WAR_ORB_DEG);
            } else if ours.is_some() {
                panic!("{} {body}: a war the corpus does not record", chart.name);
            }
            found_wars += usize::from(ours.is_some());
        }
    }
    println!("{wars} recorded wars, {found_wars} found");
    assert_eq!(wars, 14, "seven pairs, from both sides");
    assert_eq!(found_wars, wars);
}

#[test]
fn the_boundary_flags_fall_inside_the_bracket_the_corpus_gives() {
    // The engine flags on one threshold for all three divisions, which
    // the corpus brackets between about 21 and 43 arcseconds. The SDK
    // ships no threshold; what is asserted is that its distances put
    // every flagged reading below the bracket and every unflagged one
    // above it, which is what makes the corpus's flag reproducible from
    // a distance.
    let mut flagged = 0.0_f64;
    let mut clear = f64::INFINITY;
    let mut checked = 0;
    for chart in charts() {
        for body in GRAHAS {
            let recorded = &chart.bodies[body];
            let found = Boundaries::of(at(recorded["sidereal_longitude_deg"].as_f64().unwrap()));
            for (field, distance) in [
                ("near_sign_boundary", found.sign_deg),
                ("near_nakshatra_boundary", found.nakshatra_deg),
                ("near_pada_boundary", found.pada_deg),
            ] {
                if recorded[field].as_bool().unwrap_or(false) {
                    flagged = flagged.max(distance);
                } else {
                    clear = clear.min(distance);
                }
                checked += 1;
            }
        }
    }
    println!(
        "{checked} distances: flagged out to {flagged:.5}°, unflagged from {clear:.5}° — a bracket of {:.0}\" to {:.0}\"",
        flagged * 3600.0,
        clear * 3600.0
    );
    assert!(flagged < clear, "the two never overlap");
    assert!((0.005..0.013).contains(&flagged.max(clear).min(clear)));
    assert_eq!(checked, 837 * 3);
}

#[test]
fn the_lagna_is_not_a_graha_and_the_sdk_does_not_pretend_it_is() {
    // Registry: the engine records a dignity of neutral and placeholder
    // friendships for the lagna — neutral naturally, friend temporarily,
    // which do not even compound to what it reports. The SDK computes no
    // state for it, because it has no dispositor relationship to have.
    let mut lagnas = 0;
    for chart in charts() {
        let recorded = &chart.bodies["LAGNA"];
        if recorded.is_null() {
            continue;
        }
        lagnas += 1;
        assert!(same(&recorded["dignity"], Dignity::Neutral.key()));
        assert!(same(
            &recorded["natural_friendship"],
            Relationship::Neutral.key()
        ));
        assert!(same(
            &recorded["temporary_friendship"],
            Relationship::Friend.key()
        ));
        // Which is the difference: neutral and friend compound to friend.
        assert_eq!(
            dignity::compound(Relationship::Neutral, Relationship::Friend),
            Relationship::Friend
        );
        assert!(
            same(&recorded["panchadha_maitri"], Relationship::Neutral.key()),
            "and the engine reports neutral anyway"
        );
        assert!(Graha::from_key("LAGNA").is_none(), "it is not a graha");
    }
    println!("{lagnas} lagnas, none of them a graha");
    assert_eq!(lagnas, 93);
}

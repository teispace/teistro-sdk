//! What a house says when everything bearing on it is gathered, over the
//! conformance corpus's recorded charts (`03-design/rules-engine.md`, "What a
//! house says when several grahas share it").

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use std::collections::BTreeMap;

use common::{chart_at, files_in};
use teistro_rules::{Body, Evaluator, House, Kind, Readings, Rule, shipped};

/// Everything the SDK writes from a text, which is what a consumer would ask a
/// house of.
fn rules() -> Vec<Rule> {
    shipped::arishtas()
        .iter()
        .chain(shipped::gandantas())
        .chain(shipped::nabhasas())
        .chain(shipped::readings())
        .cloned()
        .collect()
}

#[test]
fn a_house_gathers_every_rule_whose_grahas_stand_in_it() {
    let rules = rules();
    let mut occupied: BTreeMap<usize, usize> = BTreeMap::new();
    let (mut charts, mut houses, mut held, mut composed) = (0, 0, 0, 0);
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::RECORDING_ENGINE).with_rules(&rules);
        charts += 1;
        let readings = evaluator.house_readings(&rules);
        assert_eq!(readings.len(), 12);
        for reading in &readings {
            houses += 1;
            held += reading.held.len();
            composed += reading.composition.len();
            *occupied.entry(reading.occupants.len()).or_default() += 1;
            // Its sign is the one the chart gives that house, and its
            // occupants are exactly the grahas standing in that sign.
            assert_eq!(reading.sign, evaluator.house_sign(reading.house));
            let standing: Vec<Body> = Body::nine()
                .into_iter()
                .filter(|body| chart.placement(*body).sign == reading.sign)
                .collect();
            assert_eq!(reading.occupants, standing);
            for gathered in &reading.held {
                // Nothing is gathered under a house its grahas do not stand
                // in, and nothing gathered is absent.
                assert!(gathered.result.present, "{}", gathered.rule.key);
                assert!(
                    gathered.result.houses.contains(&reading.house),
                    "{} under house {}",
                    gathered.rule.key,
                    reading.house.get()
                );
            }
            // A composition bears only where its own conditions are met.
            for composition in &reading.composition {
                assert!(
                    composition.houses.is_empty()
                        || composition.houses.contains(&reading.house.get())
                );
                if !composition.strong {
                    assert!(reading.occupants.len() >= usize::from(composition.at_least));
                }
                assert_eq!(composition.source().rank.map(teistro_rules::EvidenceRank::get), Some(1));
            }
        }
        // Every graha stands in exactly one house, so the twelve readings of a
        // chart account for the nine between them.
        let counted: usize = readings.iter().map(|reading| reading.occupants.len()).sum();
        assert_eq!(counted, 9);
    }
    assert_eq!((charts, houses), (93, 93 * 12));
    // How crowded the recorded houses are, and what the corpus therefore
    // exercises: the composition rules bear only where two or more share a
    // house, and the refusal only where three or more share an angle.
    let crowded: Vec<(usize, usize)> = occupied.into_iter().collect();
    assert_eq!(crowded, OCCUPANTS);
    // Forty-eight of the 1116 houses hold three grahas or more — the case a
    // reading of one graha at a time cannot answer, and the case the texts
    // decline to write.
    let crowded_houses: usize = crowded
        .iter()
        .filter(|(occupants, _)| *occupants >= 3)
        .map(|(_, houses)| *houses)
        .sum();
    assert_eq!(crowded_houses, 48);
    // What a consumer receives: 4913 rule results gathered under a house over
    // the 93 charts, and 455 statements of how to read them together.
    assert_eq!((held, composed), (4913, 455));
}

/// How many houses of the 1116 hold each number of grahas.
const OCCUPANTS: [(usize, usize); 6] = [
    (0, 547),
    (1, 365),
    (2, 156),
    (3, 35),
    (4, 10),
    (5, 3),
];

/// The texts say four things about reading a crowded house together and refuse
/// a fifth; each is a verse, and the refusal is as much a result as the rest.
#[test]
fn the_compositions_are_what_the_texts_say_and_one_of_them_is_a_refusal() {
    let all = teistro_rules::COMPOSITIONS;
    assert_eq!(all.len(), 6);
    let kinds: Vec<Kind> = all.iter().map(|composition| composition.kind).collect();
    assert_eq!(
        kinds,
        [
            Kind::Compose,
            Kind::Arbitrate,
            Kind::Arbitrate,
            Kind::Modulate,
            Kind::Arbitrate,
            Kind::Refuse
        ]
    );
    for composition in &all {
        let source = composition.source();
        assert_eq!(source.rank.map(teistro_rules::EvidenceRank::get), Some(1));
        assert!(source.chapter.is_some() && source.verse.is_some());
        assert!(!composition.says.is_empty());
        assert!(composition.houses.iter().all(|house| (1..=12).contains(house)));
    }
    // The one refusal is Saravali's, and it bears only on the four angles it
    // declined to write.
    let refusal = all
        .iter()
        .find(|composition| composition.kind == Kind::Refuse)
        .unwrap();
    assert_eq!(refusal.houses, &[1, 4, 7, 10]);
    assert_eq!(refusal.at_least, 3);
    assert!(!refusal.bears_on(House::try_new(4).unwrap(), 2, 0));
    assert!(refusal.bears_on(House::try_new(4).unwrap(), 3, 0));
    assert!(!refusal.bears_on(House::try_new(5).unwrap(), 5, 0));
    // The ascetic order's is the only one that asks for strength, so a house
    // of four grahas none of them strong does not reach it.
    let ascetic = all
        .iter()
        .find(|composition| composition.key == "ARBITRATE_THE_ASCETIC_ORDER_BY_THE_STRONGEST")
        .unwrap();
    assert!(!ascetic.bears_on(House::try_new(1).unwrap(), 4, 3));
    assert!(ascetic.bears_on(House::try_new(1).unwrap(), 4, 4));
}

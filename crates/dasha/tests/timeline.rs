//! The contract every kernel's [`Timeline`] keeps, held over each one: no
//! period before birth and the first at it, each mahadasha found at its
//! start and up to its end and the next one at its end, every parent
//! partitioned by its children, and the chain at an instant made of the
//! periods beginning there.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test fails by panicking and indexes what it built"
)]

use teistro_core::angle::Nas;
use teistro_core::catalogue::{DashaSystem, Dignity, Graha, Rashi};
use teistro_core::interval::Interval;
use teistro_core::quantity::{Degrees, Depth, JulianDay, Utc};
use teistro_core::settings::{
    AfterCycle, AshtottariGrouping, BirthPeriod, DecennialDivision, FirdariaNodes, GrahaDivision,
    YearLength, root,
};
use teistro_dasha::{
    Birth, Clock, Dasha, DecennialDasha, FirdariaDasha, GrahaDasha, KalachakraDasha,
    KalachakraRules, Period, ProfectionDasha, RashiChart, RashiDasha, RashiRules, ReleasingDasha,
    Rules, Share, Timeline, YearDasha, YearRing, decennial_order, rashi_row, row,
};

const BIRTH: f64 = 2_451_545.0;

/// More mahadashas than any firdaria order has.
const FIRDARIA_SPAN: usize = 16;

/// The float just below a positive one: `f64::next_down`, which the
/// workspace's MSRV predates.
fn below(x: f64) -> f64 {
    f64::from_bits(x.to_bits() - 1)
}

fn birth() -> JulianDay<Utc> {
    JulianDay::literal(BIRTH)
}

/// A gap the contract tolerates between two instants it computes two
/// ways, in days: well inside a millisecond.
const SLACK: f64 = 1e-7;

/// The contract, over the first `cycles` cycles and to `levels` levels.
fn holds(name: &str, timeline: &impl Timeline, cycles: u32, levels: usize) {
    let first = timeline.mahadasha(0, 0).unwrap();
    let born = first.interval.from.get();
    assert!(
        timeline.mahadasha_at(born - 1e-6).is_none(),
        "{name}: before birth"
    );
    assert_eq!(
        timeline.mahadasha_at(born).map(|p| p.path),
        Some(first.path),
        "{name}"
    );
    let mut mahadashas = Vec::new();
    for cycle in 0..cycles {
        for index in 0..=u8::MAX {
            match timeline.mahadasha(cycle, usize::from(index)) {
                Some(period) => mahadashas.push(period),
                None => break,
            }
        }
    }
    for pair in mahadashas.windows(2) {
        let (this, next) = (pair[0], pair[1]);
        assert!(
            (this.interval.to.get() - next.interval.from.get()).abs() < SLACK,
            "{name}: {} ends where {} begins",
            this.path,
            next.path
        );
        // The last bit before a period begins: in the period before while
        // that one holds it, and in one or the other where the two ends,
        // summed their own ways, leave the bit between them.
        let before = below(next.interval.from.get());
        let found = timeline.mahadasha_at(before).map(|p| p.path);
        if before < this.interval.to.get() {
            assert_eq!(
                found,
                Some(this.path),
                "{name}: the bit before {}",
                next.path
            );
        } else {
            assert!(
                found == Some(this.path) || found == Some(next.path),
                "{name}: the bit before {} is in neither: {found:?}",
                next.path
            );
        }
    }
    for period in &mahadashas {
        let (from, to) = (period.interval.from.get(), period.interval.to.get());
        if to - from < 1.0 {
            continue;
        }
        let at = |instant: f64| timeline.mahadasha_at(instant).map(|p| p.path);
        let label = format!("{name} {}", period.path);
        assert_eq!(at(from), Some(period.path), "{label}: exactly its start");
        assert_eq!(at(from + SLACK), Some(period.path), "{label}: its start");
        assert_eq!(
            at(from.midpoint(to)),
            Some(period.path),
            "{label}: its middle"
        );
        assert_eq!(at(to - 1e-3), Some(period.path), "{label}: its end");
        partitions(name, timeline, period, levels);
    }
}

/// Under `END` the birth cycle is the whole timeline: no mahadasha of a
/// later cycle, and none found after the last one ends.
fn ends(name: &str, timeline: &impl Timeline) {
    assert!(
        timeline.mahadasha(1, 0).is_none(),
        "{name}: a second cycle under END"
    );
    let last = (0..=usize::from(u8::MAX))
        .map_while(|index| timeline.mahadasha(0, index))
        .last()
        .unwrap();
    assert!(
        timeline
            .mahadasha_at(last.interval.to.get() + 1.0)
            .is_none(),
        "{name}: a day past the end under END"
    );
}

/// Every child of `parent` follows the last, the first from the parent's
/// start and the last to its end, each found by the chain at its start,
/// and so on down to `levels`.
fn partitions(name: &str, timeline: &impl Timeline, parent: &Period, levels: usize) {
    if parent.path.depth() >= levels {
        return;
    }
    let children: Vec<Period> = timeline.children(parent).collect();
    if children.is_empty() {
        return;
    }
    let label = format!("{name} {}", parent.path);
    let first = children.first().unwrap();
    let last = children.last().unwrap();
    assert!(
        (first.interval.from.get() - parent.interval.from.get()).abs() < SLACK,
        "{label}"
    );
    assert!(
        (last.interval.to.get() - parent.interval.to.get()).abs() < SLACK,
        "{label}"
    );
    for pair in children.windows(2) {
        assert!(
            (pair[0].interval.to.get() - pair[1].interval.from.get()).abs() < SLACK,
            "{label}"
        );
    }
    let depth = Depth::try_new(u8::try_from(parent.path.depth() + 1).unwrap()).unwrap();
    for child in children.iter().take(3) {
        if child.interval.days() < 1e-3 {
            continue;
        }
        let start = JulianDay::literal(child.interval.from.get() + SLACK);
        let chain = timeline.at(start, depth);
        assert_eq!(chain.deepest().map(|p| p.path), Some(child.path), "{label}");
        partitions(name, timeline, child, levels);
    }
}

fn moon(degrees: f64) -> Birth {
    Birth {
        instant: birth(),
        moon: Nas::from_degrees(Degrees::try_new(degrees).unwrap()),
        moon_span: None,
    }
}

fn rules(after_cycle: AfterCycle) -> Rules {
    Rules {
        after_cycle,
        ..Rules::of(&root().dasha, DashaSystem::Vimshottari)
    }
}

#[test]
fn every_nakshatra_dasha_keeps_the_contract() {
    for system in [
        DashaSystem::Vimshottari,
        DashaSystem::Yogini,
        DashaSystem::Ashtottari,
    ] {
        let row = row(system, AshtottariGrouping::ThreeEach).unwrap();
        for after_cycle in [AfterCycle::End, AfterCycle::Repeat] {
            for birth_period in [BirthPeriod::Compressed, BirthPeriod::Elapsed] {
                let rules = Rules {
                    birth_period,
                    ..rules(after_cycle)
                };
                let dasha = Dasha::new(row, &moon(123.456), rules).unwrap();
                let name = format!("{system:?} {after_cycle:?} {birth_period:?}");
                holds(&name, &dasha, 3, 3);
                if after_cycle == AfterCycle::End {
                    ends(&name, &dasha);
                }
            }
        }
    }
}

#[test]
fn every_sign_dasha_keeps_the_contract() {
    let chart = RashiChart {
        lagna: Rashi::Aquarius,
        arudha_lagna: Rashi::Leo,
        navamsa_lagna: Rashi::Taurus,
        signs: [
            Rashi::Capricorn,
            Rashi::Gemini,
            Rashi::Leo,
            Rashi::Virgo,
            Rashi::Pisces,
            Rashi::Libra,
            Rashi::Aries,
            Rashi::Scorpio,
            Rashi::Taurus,
        ],
        dignities: [Dignity::Neutral; 9],
        brahma: None,
    };
    for system in [
        DashaSystem::Chara,
        DashaSystem::Narayana,
        DashaSystem::Trikona,
        DashaSystem::SudarshanaChakra,
    ] {
        for after_cycle in [AfterCycle::End, AfterCycle::Repeat] {
            let dasha = RashiDasha::new(
                rashi_row(system).unwrap(),
                &chart,
                birth(),
                YearLength::Julian36525,
                after_cycle,
                RashiRules::RECORDING_ENGINE,
            )
            .unwrap();
            let name = format!("{system:?} {after_cycle:?}");
            holds(&name, &dasha, 3, 3);
            // A row whose text runs it again for life (the Sudarshana
            // chakra, BPHS ch. 74 v. 26) does so under END too.
            match (after_cycle, rashi_row(system).unwrap().recurs) {
                (AfterCycle::End, false) => ends(&name, &dasha),
                (_, true) => assert!(dasha.mahadasha(1, 0).is_some(), "{name} recurs"),
                _ => {}
            }
        }
    }
}

#[test]
fn the_kalachakra_keeps_the_contract() {
    // Under `REPEAT` too, so the second cycle is there and an instant at
    // a cycle's boundary is put in the right one.
    for after_cycle in [AfterCycle::End, AfterCycle::Repeat] {
        for degrees in [3.0, 123.456, 250.0, 359.0] {
            let rules = KalachakraRules {
                after_cycle,
                ..KalachakraRules::of(&root())
            };
            let dasha = KalachakraDasha::new(&moon(degrees), rules).unwrap();
            let name = format!("Kalachakra at {degrees} {after_cycle:?}");
            holds(&name, &dasha, 3, 2);
            if after_cycle == AfterCycle::End {
                ends(&name, &dasha);
            }
        }
    }
}

#[test]
fn the_hellenistic_time_lords_keep_the_contract() {
    for start in [Rashi::Aries, Rashi::Gemini, Rashi::Capricorn] {
        let releasing = ReleasingDasha::new(start, birth(), YearLength::Savana360);
        holds(&format!("releasing from {start:?}"), &releasing, 1, 3);
        let profection = ProfectionDasha::new(start, birth(), YearLength::Julian36525);
        holds(&format!("profection from {start:?}"), &profection, 2, 1);
    }
    for first in [Graha::Sun, Graha::Moon] {
        for nodes in [FirdariaNodes::AfterMars, FirdariaNodes::End] {
            let dasha = FirdariaDasha::new(first, birth(), YearLength::Julian36525, nodes).unwrap();
            let name = format!("firdaria from {first:?} {nodes:?}");
            holds(&name, &dasha, 3, 2);
            // A seventh is the deepest level al-Biruni gives (§439).
            for maha in (0..FIRDARIA_SPAN).map_while(|index| dasha.mahadasha(0, index)) {
                for seventh in dasha.children(&maha) {
                    assert_eq!(
                        dasha.children(&seventh).count(),
                        0,
                        "{name} {}",
                        seventh.path
                    );
                }
            }
        }
    }
    let places = [
        (Graha::Sun, 280.0),
        (Graha::Moon, 45.0),
        (Graha::Mars, 120.0),
        (Graha::Mercury, 300.0),
        (Graha::Jupiter, 200.0),
        (Graha::Venus, 330.0),
        (Graha::Saturn, 10.0),
    ];
    let order = decennial_order(Graha::Sun, &places).unwrap();
    for division in [DecennialDivision::Cycles, DecennialDivision::Proportional] {
        let dasha = DecennialDasha::new(order, birth(), YearLength::Julian36525, division).unwrap();
        holds(&format!("decennials {division:?}"), &dasha, 3, 3);
    }
}

#[test]
fn every_graha_dasha_keeps_the_contract() {
    let lords = vec![
        (Graha::Sun, 11.0),
        (Graha::Venus, 12.0),
        (Graha::Rahu, 4.0),
        (Graha::Mars, 3.5),
    ];
    for division in [GrahaDivision::Equal, GrahaDivision::Proportional] {
        for after_cycle in [AfterCycle::End, AfterCycle::Repeat] {
            let dasha = GrahaDasha::new(
                lords.clone(),
                birth(),
                YearLength::Julian36525,
                division,
                after_cycle,
            )
            .unwrap();
            holds(&format!("graha {division:?} {after_cycle:?}"), &dasha, 3, 3);
        }
    }
}

#[test]
fn a_year_s_dasha_keeps_the_contract() {
    let ring = |remaining| YearRing {
        system: teistro_dasha::DashaName::Catalogued(DashaSystem::Mudda),
        ring: [
            (Graha::Sun, 6.0),
            (Graha::Moon, 10.0),
            (Graha::Mars, 7.0),
            (Graha::Rahu, 18.0),
        ]
        .into_iter()
        .map(|(lord, weight)| Share {
            lord,
            sign: None,
            weight,
        })
        .collect(),
        first: 2,
        remaining,
    };
    for remaining in [None, Some(0.4)] {
        for birth_period in [BirthPeriod::Compressed, BirthPeriod::Elapsed] {
            let clock = Clock::even(Interval::literal(BIRTH, BIRTH + 365.25)).unwrap();
            let dasha = YearDasha::new(ring(remaining), clock, birth_period).unwrap();
            holds(
                &format!("year {remaining:?} {birth_period:?}"),
                &dasha,
                1,
                3,
            );
        }
    }
}

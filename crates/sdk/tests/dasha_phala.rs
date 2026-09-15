//! A chart document's dasha phala, computed end to end on the corpus's first
//! chart founded with the built-in ephemeris. The corpus records none, so this
//! holds the reading to the verses against the state computed beside it: each
//! Subhanka within its varga's share, the rasi chart's the state's dignity's,
//! the totals complementary, the phase the decanate's and the flags the
//! placement's.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "tests fail by panicking, and the points are exact halves"
)]

mod common;

use common::reading;
use teistro::catalogue::{Dignity, Graha};
use teistro::strength::DashaPhase;

#[test]
fn a_reading_carries_a_dasha_phala_consistent_with_the_state() {
    let (_, document) = reading("{}", |request| request.with_dasha_phala().with_state());
    assert!(document.sections().contains(&"dasha_phala"));
    let reading = document.dasha_phala.expect("the section asked for");
    let states = document.state.expect("asked for beside it");
    let order: Vec<Graha> = reading.grahas.iter().map(|g| g.graha).collect();
    assert_eq!(order, teistro::strength::bhava_bala::NINE);
    for graha in &reading.grahas {
        let state = states.iter().find(|s| s.graha == graha.graha).unwrap();
        let position = document
            .foundation
            .grahas
            .iter()
            .find(|p| p.graha == graha.graha)
            .unwrap();
        for (k, points) in graha.subhankas.iter().enumerate() {
            let out_of = if k == 0 { 60.0 } else { 30.0 };
            assert!(
                (0.0..=out_of).contains(points),
                "{:?} varga {k}",
                graha.graha
            );
        }
        assert_eq!(graha.subhanka + graha.asubhanka, 240.0);
        assert_eq!(graha.subhanka, graha.subhankas.iter().sum::<f64>());
        let exalted = matches!(state.dignity, Dignity::Exalted | Dignity::DeepExalted);
        assert_eq!(exalted, graha.subhankas[0] == 60.0, "{:?}", graha.graha);
        if exalted || state.house == 1 {
            assert!(graha.favourable, "{:?}", graha.graha);
        }
        if matches!(state.house, 6 | 8 | 12) {
            assert!(graha.unfavourable, "{:?}", graha.graha);
        }
        // Worked from the degrees here rather than from the kernel's arc.
        let in_sign = position.longitude_deg.rem_euclid(30.0);
        let forward = if in_sign < 10.0 {
            DashaPhase::Commencement
        } else if in_sign < 20.0 {
            DashaPhase::Middle
        } else {
            DashaPhase::End
        };
        let reversed = state.motion.retrograde || matches!(graha.graha, Graha::Rahu | Graha::Ketu);
        let expected = match (reversed, forward) {
            (true, DashaPhase::Commencement) => DashaPhase::End,
            (true, DashaPhase::End) => DashaPhase::Commencement,
            (_, phase) => phase,
        };
        assert_eq!(graha.phase, expected, "{:?}", graha.graha);
    }
}

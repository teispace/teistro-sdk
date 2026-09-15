//! A chart document's Vaiseshikamsa, computed end to end on the corpus's
//! first chart founded with the built-in ephemeris. The corpus records no
//! Vaiseshikamsa, so this holds the reading to its own rules: each count within
//! its scheme's vargas, each name the one the count earns, the saptavarga's
//! count never below the shadvarga's, and the impaired flag the state's
//! combustion and war.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

mod common;

use common::reading;

#[test]
fn a_reading_carries_a_vaiseshikamsa_consistent_with_its_rules() {
    let (_, document) = reading("{}", |request| request.with_vaiseshikamsa().with_state());
    assert!(document.sections().contains(&"vaiseshikamsa"));
    let reading = document.vaiseshikamsa.expect("the section asked for");
    let states = document.state.expect("asked for beside it");
    assert_eq!(reading.grahas.len(), 7);
    for graha in &reading.grahas {
        let schemes = [
            (graha.shadvarga, 6),
            (graha.saptavarga, 7),
            (graha.dashavarga, 10),
            (graha.shodashavarga, 16),
        ];
        for (standing, vargas) in schemes {
            assert!(standing.good_vargas <= vargas, "{:?}", graha.graha);
            assert_eq!(
                standing.name.map(|name| name.attributes().good_vargas),
                (standing.good_vargas >= 2).then_some(standing.good_vargas),
                "{:?}: a name for every count from two, and only that count's",
                graha.graha
            );
        }
        assert!(graha.saptavarga.good_vargas >= graha.shadvarga.good_vargas);
        let state = states.iter().find(|s| s.graha == graha.graha).unwrap();
        // Combust, defeated, or in Shayana, the bad avastha ch. 6 v. 53 names.
        assert_eq!(
            graha.impaired,
            state.is_combust() || state.lost_its_war() || state.is_shayana()
        );
        assert!(
            state.sayanadi.is_some(),
            "{:?}: the nine have a Sayanadi",
            graha.graha
        );
    }
}

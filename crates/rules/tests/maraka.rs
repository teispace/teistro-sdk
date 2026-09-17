//! BPHS ch. 44's marakas over the 93 recorded charts: what the verses make
//! true of every chart.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use common::{chart_at, files_in};
use teistro_core::catalogue::Graha;
use teistro_rules::longevity::{Brings, Reason};
use teistro_rules::{Body, BodyRef, Class, Condition, Evaluator, Participants, Readings};

const NINE: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

#[test]
fn every_chart_s_marakas_are_what_the_verses_make_them() {
    let (mut charts, mut death, mut saturn, mut nodes) = (0, 0, 0, 0);
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS);
        let marakas = evaluator.marakas();
        for graha in NINE {
            let reasons = marakas.of(graha);
            // The prime reasons are the kernel's own maraka class.
            let class = evaluator.holds(
                &Condition::PlanetIs {
                    planet: BodyRef::Body(Body::Graha(graha)),
                    class: Class::Maraka,
                },
                &mut Participants::default(),
            );
            assert_eq!(reasons.is_prime(), class, "{} {graha:?}", path.display());
            death += usize::from(reasons.brings() == Some(Brings::Death));
            saturn += usize::from(reasons.contains(Reason::SaturnIllDisposed));
            nodes += usize::from(reasons.contains(Reason::NodePlaced));
        }
        // One star has one lord, and one decanate one: each of these reasons
        // falls on exactly one graha of every chart.
        for reason in [
            Reason::VipatStarLord,
            Reason::PratyakStarLord,
            Reason::VadhaStarLord,
            Reason::TwentyThirdStarLord,
            Reason::TwentySecondDecanateLord,
            Reason::LordOfSecond,
            Reason::LordOfSeventh,
            Reason::LordOfSixth,
            Reason::LordOfEighth,
        ] {
            let holders = NINE
                .iter()
                .filter(|graha| marakas.of(**graha).contains(reason))
                .count();
            assert_eq!(holders, 1, "{} {reason:?}", path.display());
        }
        charts += 1;
    }
    // 673 of the 837 grahas carry a reason that brings death — seven of nine a
    // chart. That breadth is the verses', and it is why a period read alone is
    // only a vulnerability, to be read with the span of life. Saturn is
    // ill-disposed and related to a prime maraka on 13 charts, and a node is
    // placed to kill 93 times.
    assert_eq!((charts, death, saturn, nodes), (93, 673, 13, 93));
}

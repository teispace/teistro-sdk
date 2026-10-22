//! The chara karakas' ranking on the Karaka dasha's worked example (BPHS
//! ch. 46 v. 178, crux C739).
//!
//! Santhanam's example chart (vol. 2, p. 558) prints each graha's sign,
//! degrees, minutes and seconds, and his karaka table (pp. 559 and 564)
//! ranks them Sun, Venus, Rahu, Jupiter, Saturn, Mercury, Moon, Mars. The
//! page prints the Moon at 2s 22° 6′ 58″, which would make her the
//! Amatyakaraka; his own ranking puts her seventh, where 2° 6′ 58″ would.
//! The test holds the ranking to both readings, so the conflict stays
//! visible rather than settled by a digit.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test fails by panicking and indexes the ten placements it builds"
)]

use teistro_core::catalogue::{Dignity, Graha, Rashi};
use teistro_rules::{Body, House, Placement, RuleChart};

/// A longitude from the page's sign (Aries 0), degrees, minutes, seconds.
fn longitude(sign: u8, degrees: u8, minutes: u8, seconds: u8) -> f64 {
    f64::from(sign) * 30.0
        + f64::from(degrees)
        + f64::from(minutes) / 60.0
        + f64::from(seconds) / 3600.0
}

fn chart(moon_degrees: u8) -> RuleChart {
    let at = |longitude| Placement {
        longitude,
        sign: Rashi::of_longitude(longitude),
        house: House::try_new(1).unwrap(),
        dignity: Dignity::Neutral,
        retrograde: false,
        combust: false,
        karaka7: None,
        karaka8: None,
        navamsha: Rashi::Aries,
    };
    let mut placements = [at(0.0); 10];
    let mut place = |graha: Graha, longitude: f64| {
        placements[Body::Graha(graha).index()] = at(longitude);
    };
    place(Graha::Sun, longitude(9, 29, 36, 53));
    place(Graha::Moon, longitude(2, moon_degrees, 6, 58));
    place(Graha::Mars, longitude(1, 0, 56, 21));
    place(Graha::Mercury, longitude(10, 13, 6, 26));
    place(Graha::Jupiter, longitude(10, 13, 41, 58));
    place(Graha::Venus, longitude(10, 20, 4, 2));
    place(Graha::Saturn, longitude(7, 13, 24, 27));
    place(Graha::Rahu, longitude(2, 13, 55, 34));
    placements[Body::Lagna.index()] = at(longitude(10, 16, 28, 5));
    RuleChart {
        placements,
        panchanga: None,
        strengths: None,
    }
}

#[test]
fn the_karakas_rank_as_santhanams_table_does_with_the_moon_at_two_degrees() {
    assert_eq!(
        chart(2).chara_karaka_ranking(true),
        [
            Graha::Sun,
            Graha::Venus,
            Graha::Rahu,
            Graha::Jupiter,
            Graha::Saturn,
            Graha::Mercury,
            Graha::Moon,
            Graha::Mars,
        ]
    );
    // Without Rahu the seven keep their order.
    assert_eq!(
        chart(2).chara_karaka_ranking(false),
        [
            Graha::Sun,
            Graha::Venus,
            Graha::Jupiter,
            Graha::Saturn,
            Graha::Mercury,
            Graha::Moon,
            Graha::Mars,
        ]
    );
}

#[test]
fn the_printed_twenty_two_degrees_would_make_the_moon_amatyakaraka() {
    let ranking = chart(22).chara_karaka_ranking(true);
    assert_eq!(&ranking[..3], &[Graha::Sun, Graha::Moon, Graha::Venus]);
}

//! The classical rules a falsification pass needs before the crate that
//! would provide them exists.
//!
//! A pass measures a proposed rule against the corpus **without** using
//! the module it is designing, which is the whole point: a pass that
//! called into `crates/state` would only prove that crate agrees with
//! itself. But two passes proposing rules about the same twelve signs
//! need the same sign arithmetic, the same lordships and the same
//! friendship table, and a second copy of those is a second thing to get
//! wrong. They live here.
//!
//! Nothing in this module reads the corpus or writes a page. It is the
//! zodiac, the catalogue's own tables, and the arithmetic over them.

use std::collections::BTreeMap;

use teistro_core::catalogue::{Graha, Rashi};

/// The nine grahas a recorded chart gives a state to, in the recording
/// engine's own order.
pub(crate) const GRAHAS: [&str; 9] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU",
];

/// The two shadow grahas.
pub(crate) const NODES: [&str; 2] = ["RAHU", "KETU"];

/// A sign's degrees.
pub(crate) const PER_SIGN: f64 = 30.0;

/// The signs of the zodiac.
pub(crate) const SIGNS: u8 = 12;

/// The sign a longitude falls in, 0 for Aries.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a normalised longitude over thirty is 0 to 11"
)]
pub(crate) fn sign_of(longitude: f64) -> u8 {
    ((longitude.rem_euclid(360.0) / PER_SIGN) as u8).min(11)
}

/// The lord of a sign.
pub(crate) fn lord_of(sign: u8) -> Option<Graha> {
    Rashi::from_id(u16::from(sign)).map(|rashi| rashi.attributes().lord)
}

/// Which house of the first sign the second is, counting inclusively
/// from one: the count the whole tradition reckons a relation by.
pub(crate) const fn house_count(from: u8, to: u8) -> u8 {
    (to + SIGNS - from) % SIGNS + 1
}

/// Whether a body stands in its own sign, counting the lordship the
/// catalogue gives.
pub(crate) fn in_own_sign(graha: Graha, sign: u8) -> bool {
    Rashi::from_id(u16::from(sign)).is_some_and(|rashi| {
        graha.attributes().own.contains(&rashi) || lord_of(sign) == Some(graha)
    })
}

/// The natural friendship a body has with the lord of a sign.
///
/// A body in its own sign is its own friend — the one addition the corpus
/// forces on the catalogue's own table.
pub(crate) fn natural(graha: Graha, sign: u8) -> &'static str {
    let Some(lord) = lord_of(sign) else {
        return "neutral";
    };
    if lord == graha {
        return "friend";
    }
    let attributes = graha.attributes();
    if attributes.friends.contains(&lord) {
        "friend"
    } else if attributes.enemies.contains(&lord) {
        "enemy"
    } else {
        "neutral"
    }
}

/// The houses from a body in which its dispositor is a temporary friend.
pub(crate) const FRIENDLY_HOUSES: [u8; 6] = [2, 3, 4, 10, 11, 12];

/// The temporary friendship a body has with its dispositor.
pub(crate) fn temporary(
    graha: Graha,
    sign: u8,
    signs: &BTreeMap<String, u8>,
) -> Option<&'static str> {
    let lord = lord_of(sign)?;
    if lord == graha {
        return Some("friend");
    }
    let at = signs.get(lord.key())?;
    let distance = house_count(sign, *at);
    Some(if FRIENDLY_HOUSES.contains(&distance) {
        "friend"
    } else {
        "enemy"
    })
}

/// The five-fold compound of the two friendships.
pub(crate) fn panchadha(natural: &str, temporary: &str) -> &'static str {
    match (natural, temporary) {
        ("friend", "friend") => "GREAT_FRIEND",
        ("neutral", "friend") => "FRIEND",
        ("neutral", "enemy") => "ENEMY",
        ("enemy", "enemy") => "GREAT_ENEMY",
        _ => "NEUTRAL",
    }
}

/// The angle between two longitudes, degrees.
pub(crate) fn separation(first: f64, second: f64) -> f64 {
    let apart = (first - second).abs().rem_euclid(360.0);
    apart.min(360.0 - apart)
}

/// How far a longitude is from the nearest boundary of a division.
pub(crate) fn edge(longitude: f64, width: f64) -> f64 {
    let inside = longitude.rem_euclid(width);
    inside.min(width - inside)
}

#[cfg(test)]
mod tests {
    use super::{
        GRAHAS, NODES, edge, house_count, in_own_sign, lord_of, natural, panchadha, separation,
        sign_of,
    };
    use teistro_core::catalogue::Graha;

    #[test]
    fn a_longitude_falls_in_the_sign_that_holds_it() {
        assert_eq!(sign_of(0.0), 0);
        assert_eq!(sign_of(29.999), 0);
        assert_eq!(sign_of(30.0), 1);
        assert_eq!(sign_of(359.999), 11);
        assert_eq!(sign_of(360.0), 0, "the circle wraps");
        assert_eq!(sign_of(-1.0), 11);
    }

    #[test]
    fn a_house_is_counted_inclusively_from_one() {
        assert_eq!(house_count(0, 0), 1, "a sign is its own first");
        assert_eq!(house_count(0, 6), 7, "and the seventh is opposite");
        assert_eq!(house_count(6, 0), 7, "either way round");
        assert_eq!(house_count(11, 0), 2, "over the end of the zodiac");
        assert_eq!(house_count(0, 11), 12);
    }

    #[test]
    fn the_lords_are_the_catalogue_s() {
        assert_eq!(lord_of(0), Some(Graha::Mars), "Aries is Mars's");
        assert_eq!(lord_of(4), Some(Graha::Sun), "Leo is the Sun's");
        assert_eq!(lord_of(12), None);
    }

    #[test]
    fn a_body_in_its_own_sign_is_its_own_friend() {
        assert!(in_own_sign(Graha::Sun, 4), "the Sun in Leo");
        assert!(!in_own_sign(Graha::Sun, 3));
        assert_eq!(natural(Graha::Sun, 4), "friend", "and so its own friend");
        assert_eq!(natural(Graha::Sun, 3), "friend", "Cancer is the Moon's");
        assert_eq!(natural(Graha::Sun, 1), "enemy", "Taurus is Venus's");
        assert_eq!(natural(Graha::Sun, 2), "neutral", "Gemini is Mercury's");
    }

    #[test]
    fn the_compound_is_the_classical_one() {
        assert_eq!(panchadha("friend", "friend"), "GREAT_FRIEND");
        assert_eq!(panchadha("friend", "enemy"), "NEUTRAL");
        assert_eq!(panchadha("neutral", "friend"), "FRIEND");
        assert_eq!(panchadha("neutral", "enemy"), "ENEMY");
        assert_eq!(panchadha("enemy", "friend"), "NEUTRAL");
        assert_eq!(panchadha("enemy", "enemy"), "GREAT_ENEMY");
    }

    #[test]
    fn a_separation_is_the_shorter_way_round() {
        assert!((separation(10.0, 350.0) - 20.0).abs() < 1e-9);
        assert!((separation(350.0, 10.0) - 20.0).abs() < 1e-9);
        assert!((separation(0.0, 180.0) - 180.0).abs() < 1e-9);
        assert!(separation(5.0, 5.0).abs() < 1e-9);
    }

    #[test]
    fn a_distance_to_a_boundary_is_to_the_nearer_one() {
        assert!(edge(0.0, 30.0).abs() < 1e-9);
        assert!((edge(1.0, 30.0) - 1.0).abs() < 1e-9);
        assert!((edge(29.0, 30.0) - 1.0).abs() < 1e-9, "the near side");
        assert!((edge(15.0, 30.0) - 15.0).abs() < 1e-9);
    }

    #[test]
    fn the_bodies_are_the_ones_a_recorded_chart_carries() {
        assert_eq!(GRAHAS.len(), 9);
        assert!(NODES.iter().all(|node| GRAHAS.contains(node)));
        assert!(GRAHAS.iter().all(|body| Graha::from_key(body).is_some()));
    }
}

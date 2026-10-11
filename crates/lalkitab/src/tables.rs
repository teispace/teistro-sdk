//! The 1952 edition's tables, each cited to its page and each, where the
//! book follows a rule, written as the rule (`03-design/lalkitab.md`).
//!
//! What ships is method: house numbers, planet lists and conditions, in
//! the SDK's own words. No sentence of the book is reproduced.

use teistro_core::catalogue::{Graha, Rashi};

/// The nine planets Lal Kitab reads, in its own order (1952 p. 33):
/// Jupiter, Sun, Moon, Venus, Mars, Mercury, Saturn, Rahu, Ketu.
pub const PLANETS: [Graha; 9] = [
    Graha::Jupiter,
    Graha::Sun,
    Graha::Moon,
    Graha::Venus,
    Graha::Mars,
    Graha::Mercury,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// Whether `graha` is one of the nine.
#[must_use]
pub fn reads(graha: Graha) -> bool {
    PLANETS.contains(&graha)
}

/// The houses a planet is permanent in, its *pakka ghar* (1952 p. 29,
/// the kundali diagram): the first is the one a single-house reading
/// names.
#[must_use]
pub fn pakka_ghar(graha: Graha) -> &'static [u8] {
    match graha {
        Graha::Sun => &[1],
        Graha::Moon => &[4],
        Graha::Mars => &[3, 8],
        Graha::Mercury | Graha::Venus => &[7],
        Graha::Jupiter => &[2, 5, 9, 11],
        Graha::Saturn => &[10, 8],
        Graha::Rahu => &[12],
        Graha::Ketu => &[6],
        _ => &[],
    }
}

/// The house a sign stands for: house `n` is rashi `n`, Aries 1
/// (1952 pp. ~6–7).
#[must_use]
pub fn house_of_sign(sign: Rashi) -> u8 {
    Rashi::ALL
        .iter()
        .position(|each| *each == sign)
        .and_then(|index| u8::try_from(index + 1).ok())
        .unwrap_or(1)
}

/// The seventh house from `house`.
#[must_use]
pub const fn seventh_from(house: u8) -> u8 {
    (house + 5) % 12 + 1
}

/// The houses a planet owns: house `n` is read as sign `n`, so ownership
/// is the classical rulership by number, read off the catalogue (1952
/// pp. ~6–7). The nodes own none: houses 6 and 12 are held jointly, Mercury
/// with Ketu and Jupiter with Rahu (p. ~46), which [`joint_owners`] gives.
#[must_use]
pub fn own(graha: Graha) -> Vec<u8> {
    if matches!(graha, Graha::Rahu | Graha::Ketu) || !reads(graha) {
        return Vec::new();
    }
    graha
        .attributes()
        .own
        .iter()
        .map(|sign| house_of_sign(*sign))
        .collect()
}

/// The planets that hold a house jointly when it is empty (1952 p. ~46).
#[must_use]
pub const fn joint_owners(house: u8) -> Option<[Graha; 2]> {
    match house {
        6 => Some([Graha::Mercury, Graha::Ketu]),
        12 => Some([Graha::Jupiter, Graha::Rahu]),
        _ => None,
    }
}

/// The houses a planet is exalted in (1952 pp. ~44–46). The book states a
/// rule and its examples are the classical exaltation signs read as house
/// numbers, so the seven are read off the catalogue; the nodes take the
/// book's own houses, Rahu Mercury's (3, 6) and Ketu Jupiter's (9, 12).
#[must_use]
pub fn exalted(graha: Graha) -> Vec<u8> {
    match graha {
        Graha::Rahu => vec![3, 6],
        Graha::Ketu => vec![9, 12],
        _ if reads(graha) => graha
            .attributes()
            .exaltation
            .map(|at| vec![house_of_sign(at.sign)])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// The houses a planet is debilitated in: the seventh from each exalted
/// house (1952 pp. ~44–46), which for the nodes is the other's exaltation.
#[must_use]
pub fn debilitated(graha: Graha) -> Vec<u8> {
    exalted(graha).into_iter().map(seventh_from).collect()
}

/// How one planet regards another (1952 p. 31). The relation is
/// directed: the book says so, and the table is not symmetric.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Regard {
    /// A friend.
    Friend,
    /// Equal (*barabar*).
    Equal,
    /// An enemy.
    Enemy,
}

/// How `of` regards `to` (1952 p. 31); a planet is its own equal.
#[must_use]
pub fn regard(of: Graha, to: Graha) -> Option<Regard> {
    use Graha::{Jupiter, Ketu, Mars, Mercury, Moon, Rahu, Saturn, Sun, Venus};
    if !reads(of) || !reads(to) {
        return None;
    }
    if of == to {
        return Some(Regard::Equal);
    }
    let (friends, enemies): (&[Graha], &[Graha]) = match of {
        Jupiter => (&[Sun, Mars, Moon], &[Venus, Mercury]),
        Sun => (&[Jupiter, Mars, Moon], &[Venus, Saturn, Rahu, Ketu]),
        Moon => (&[Sun, Mercury], &[Ketu, Rahu]),
        Venus => (&[Saturn, Mercury, Ketu], &[Sun, Moon, Rahu]),
        Mars => (&[Sun, Moon, Jupiter], &[Mercury, Ketu]),
        Mercury => (&[Sun, Venus, Rahu], &[Moon]),
        Saturn => (&[Mercury, Venus, Rahu], &[Sun, Moon, Mars]),
        Rahu => (&[Mercury, Saturn, Ketu], &[Sun, Venus, Mars]),
        Ketu => (&[Venus, Rahu], &[Moon, Mars]),
        _ => return None,
    };
    Some(if friends.contains(&to) {
        Regard::Friend
    } else if enemies.contains(&to) {
        Regard::Enemy
    } else {
        Regard::Equal
    })
}

/// A house's waker: the planet whose presence wakes it when it sleeps
/// (1952 p. 98).
#[must_use]
pub const fn waker(house: u8) -> Graha {
    match house {
        1 => Graha::Mars,
        2 | 4 | 8 => Graha::Moon,
        3 => Graha::Mercury,
        5 => Graha::Sun,
        6 => Graha::Rahu,
        7 => Graha::Venus,
        9 | 11 => Graha::Jupiter,
        10 => Graha::Saturn,
        _ => Graha::Ketu,
    }
}

/// The age a planet matures and wakes by itself, its *aam saal* (1952
/// pp. 33, 99); also where its second round of the 35-year cycle starts.
#[must_use]
pub const fn matures_at(graha: Graha) -> Option<u8> {
    match graha {
        Graha::Jupiter => Some(16),
        Graha::Sun => Some(22),
        Graha::Moon => Some(24),
        Graha::Venus => Some(25),
        Graha::Mars => Some(28),
        Graha::Mercury => Some(34),
        Graha::Saturn => Some(36),
        Graha::Rahu => Some(42),
        Graha::Ketu => Some(48),
        _ => None,
    }
}

/// A planet's years in the 35-year cycle (1952 p. 33): Jupiter 6, Sun 2,
/// Moon 1, Venus 3, Mars 6, Mercury 2, Saturn 6, Rahu 6, Ketu 3.
#[must_use]
pub const fn cycle_years(graha: Graha) -> u8 {
    match graha {
        Graha::Sun | Graha::Mercury => 2,
        Graha::Moon => 1,
        Graha::Venus | Graha::Ketu => 3,
        Graha::Jupiter | Graha::Mars | Graha::Saturn | Graha::Rahu => 6,
        _ => 0,
    }
}

/// A planet's mahadasha years (1952 p. 33): Vimshottari's values, in Lal
/// Kitab's order, 120 in all.
#[must_use]
pub const fn mahadasha_years(graha: Graha) -> u8 {
    match graha {
        Graha::Jupiter => 16,
        Graha::Sun => 6,
        Graha::Moon => 10,
        Graha::Venus => 20,
        Graha::Mars | Graha::Ketu => 7,
        Graha::Mercury => 17,
        Graha::Saturn => 19,
        Graha::Rahu => 18,
        _ => 0,
    }
}

/// The planets ruling the thirds of a year whose planet is `graha`,
/// months 1–4, 5–8 and 9–12 (1952 p. 34).
#[must_use]
pub const fn thirds(graha: Graha) -> Option<[Graha; 3]> {
    use Graha::{Jupiter, Ketu, Mars, Mercury, Moon, Rahu, Saturn, Sun, Venus};
    Some(match graha {
        Jupiter => [Ketu, Jupiter, Sun],
        Sun => [Sun, Moon, Mars],
        Moon => [Jupiter, Sun, Moon],
        Venus => [Mars, Venus, Mercury],
        Mars => [Mars, Saturn, Venus],
        Mercury => [Moon, Mars, Jupiter],
        Saturn => [Rahu, Mercury, Saturn],
        Rahu => [Mars, Ketu, Rahu],
        Ketu => [Saturn, Rahu, Ketu],
        _ => return None,
    })
}

/// The artificial (*masnui*) planets two planets in one house make
/// (1952 p. 27): the pair, and what it counts as.
pub const MASNUI: [(Graha, Graha, Masnui); 13] = [
    (Graha::Sun, Graha::Venus, Masnui::Jupiter),
    (Graha::Mercury, Graha::Venus, Masnui::Sun),
    (Graha::Sun, Graha::Jupiter, Masnui::Moon),
    (Graha::Rahu, Graha::Ketu, Masnui::Venus),
    (Graha::Sun, Graha::Mercury, Masnui::MarsBenefic),
    (Graha::Sun, Graha::Saturn, Masnui::MarsMalefic),
    (Graha::Jupiter, Graha::Rahu, Masnui::Mercury),
    (Graha::Venus, Graha::Jupiter, Masnui::SaturnLikeKetu),
    (Graha::Mars, Graha::Mercury, Masnui::SaturnLikeRahu),
    (Graha::Mars, Graha::Saturn, Masnui::RahuExalted),
    (Graha::Sun, Graha::Saturn, Masnui::RahuDebilitated),
    (Graha::Venus, Graha::Saturn, Masnui::KetuExalted),
    (Graha::Moon, Graha::Saturn, Masnui::KetuDebilitated),
];

/// What a pair of planets in one house counts as (1952 p. 27).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Masnui {
    /// An artificial Jupiter.
    Jupiter,
    /// An artificial Sun.
    Sun,
    /// An artificial Moon.
    Moon,
    /// An artificial Venus.
    Venus,
    /// An artificial benefic (*nek*) Mars.
    MarsBenefic,
    /// An artificial malefic (*bad*) Mars.
    MarsMalefic,
    /// An artificial Mercury.
    Mercury,
    /// An artificial Saturn of Ketu's nature.
    SaturnLikeKetu,
    /// An artificial Saturn of Rahu's nature.
    SaturnLikeRahu,
    /// An artificial Rahu in its exalted state.
    RahuExalted,
    /// An artificial Rahu in its debilitated state.
    RahuDebilitated,
    /// An artificial Ketu in its exalted state.
    KetuExalted,
    /// An artificial Ketu in its debilitated state.
    KetuDebilitated,
}

/// The nine debts (*rin*), each arising when one of a planet's enemies
/// sits in one of its houses (1952 p. 125).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Rin {
    /// The ancestors' debt (Jupiter's houses).
    Pitri,
    /// One's own debt (the Sun's).
    Swa,
    /// The mother's debt (the Moon's).
    Matri,
    /// The wife's debt (Venus's).
    Stri,
    /// The relatives' debt (Mars's).
    Rishtedari,
    /// The sister's or daughter's debt (Mercury's).
    Bhagini,
    /// The debt of cruelty (Saturn's).
    Zalimana,
    /// The debt of the unborn (Rahu's).
    Ajanma,
    /// The divine debt (Ketu's).
    Daivi,
}

/// Each debt: the planet, its houses, and the enemies whose seat there
/// raises it (1952 p. 125). The Sun's "papi" are Saturn, Rahu and Ketu.
pub const RINAS: [(Rin, Graha, &[u8], &[Graha]); 9] = [
    (
        Rin::Pitri,
        Graha::Jupiter,
        &[2, 5, 9, 12],
        &[Graha::Venus, Graha::Mercury, Graha::Rahu],
    ),
    (
        Rin::Swa,
        Graha::Sun,
        &[5],
        &[Graha::Venus, Graha::Saturn, Graha::Rahu, Graha::Ketu],
    ),
    (Rin::Matri, Graha::Moon, &[4], &[Graha::Ketu]),
    (
        Rin::Stri,
        Graha::Venus,
        &[2, 7],
        &[Graha::Sun, Graha::Rahu, Graha::Moon],
    ),
    (
        Rin::Rishtedari,
        Graha::Mars,
        &[1, 8],
        &[Graha::Mercury, Graha::Ketu],
    ),
    (Rin::Bhagini, Graha::Mercury, &[3, 6], &[Graha::Moon]),
    (
        Rin::Zalimana,
        Graha::Saturn,
        &[10, 11],
        &[Graha::Sun, Graha::Moon, Graha::Mars],
    ),
    (
        Rin::Ajanma,
        Graha::Rahu,
        &[12],
        &[Graha::Venus, Graha::Sun, Graha::Mars],
    ),
    (Rin::Daivi, Graha::Ketu, &[6], &[Graha::Moon, Graha::Mars]),
];

/// The first state of the ancestors' debt (1952 p. 128): a planet in the
/// ninth house while Mercury sits in that planet's root house; here, each
/// planet and its root houses.
#[must_use]
pub fn root(graha: Graha) -> &'static [u8] {
    match graha {
        Graha::Jupiter | Graha::Rahu => &[12],
        Graha::Sun => &[5],
        Graha::Moon => &[4],
        Graha::Venus => &[2, 7],
        Graha::Mars => &[1, 8],
        Graha::Saturn => &[10, 11],
        Graha::Ketu => &[6],
        _ => &[],
    }
}

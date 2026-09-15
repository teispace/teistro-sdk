//! The Sthana bala: a graha's strength by where it stands — its distance from
//! debilitation, its dignity in the seven vargas, the parity of its rasi and
//! navamsha, its house and its decanate.

use teistro_core::catalogue::{Gender, Graha, Rashi, Relationship};
use teistro_core::settings::{Drekkana, Saptavargaja};
use teistro_state::dignity::{compound, natural, temporary_from};

use super::{ShadbalaChart, ShadbalaGraha, ShadbalaRules, SthanaBala, apart, seat, sign_of};

/// A graha's Sthana bala.
pub(crate) fn of(graha: Graha, chart: &ShadbalaChart, rules: &ShadbalaRules) -> SthanaBala {
    let at = seat(&chart.grahas, graha);
    let [_, _, _, _, navamsha, ..] = chart.vargas;
    SthanaBala {
        uchcha: uchcha(graha, &at),
        saptavargaja: saptavargaja(graha, chart, rules.saptavargaja),
        ojayugma: ojayugma(graha, sign_of(at.longitude), seat(&navamsha, graha)),
        kendradi: kendradi(at.house),
        drekkana: drekkana(graha, at.longitude, rules.drekkana),
    }
}

/// A third of the arc from the graha's debilitation point (v. 1).
fn uchcha(graha: Graha, at: &ShadbalaGraha) -> f64 {
    let exaltation = graha.attributes().exaltation.map_or(0.0, |e| {
        f64::from(e.sign as u8) * 30.0 + f64::from(e.degree)
    });
    ((180.0 - apart(at.longitude, exaltation)) / 3.0).max(0.0)
}

/// The dignity in the seven vargas, summed.
fn saptavargaja(graha: Graha, chart: &ShadbalaChart, rule: Saptavargaja) -> f64 {
    let [rasi, ..] = chart.vargas;
    chart
        .vargas
        .iter()
        .enumerate()
        .map(|(k, row)| {
            let sign = seat(row, graha);
            if rule == Saptavargaja::Natural {
                natural_virupas(graha, sign)
            } else {
                compound_virupas(graha, sign, k == 0, &rasi)
            }
        })
        .sum()
}

/// The recording engine's virupas for a graha in a varga sign.
pub(crate) fn natural_virupas(graha: Graha, sign: Rashi) -> f64 {
    let attributes = graha.attributes();
    if attributes.exaltation.is_some_and(|e| e.sign == sign) {
        45.0
    } else if attributes.debilitation.is_some_and(|e| e.sign == sign) {
        0.0
    } else if attributes.moolatrikona.is_some_and(|m| m.sign == sign)
        || attributes.own.contains(&sign)
    {
        30.0
    } else {
        match natural(graha, sign) {
            Relationship::Friend => 15.0,
            Relationship::Enemy => 3.75,
            _ => 7.5,
        }
    }
}

/// The compound relationship's virupas: 45 for the moolatrikona rasi in the
/// rasi chart alone, 30 for the own sign, then 22.5, 15, 7.5, 3.75 or 1.875
/// by the compound relationship with the sign's lord, its temporary half from
/// the rasi chart (B.V. Raman, Art. 30).
pub(crate) fn compound_virupas(graha: Graha, sign: Rashi, in_rasi: bool, rasi: &[Rashi; 7]) -> f64 {
    let attributes = graha.attributes();
    if in_rasi && attributes.moolatrikona.is_some_and(|m| m.sign == sign) {
        return 45.0;
    }
    if attributes.own.contains(&sign) {
        return 30.0;
    }
    let temporary = temporary_from(graha, sign.attributes().lord, seat(rasi, graha), |g| {
        Some(seat(rasi, g))
    })
    .unwrap_or(Relationship::Neutral);
    match compound(natural(graha, sign), temporary) {
        Relationship::GreatFriend => 22.5,
        Relationship::Friend => 15.0,
        Relationship::Enemy => 3.75,
        Relationship::GreatEnemy => 1.875,
        _ => 7.5,
    }
}

/// 15 each for a rasi and a navamsha of the graha's parity: odd for all but
/// the Moon and Venus (v. 4½).
fn ojayugma(graha: Graha, rasi: Rashi, navamsha: Rashi) -> f64 {
    let wants_odd = !matches!(graha, Graha::Moon | Graha::Venus);
    let odd = |sign: Rashi| (sign as u8) % 2 == 0;
    [rasi, navamsha]
        .into_iter()
        .map(|sign| if odd(sign) == wants_odd { 15.0 } else { 0.0 })
        .sum()
}

/// 60, 30 or 15 for a kendra, panaphara or apoklima house (v. 5).
const fn kendradi(house: u8) -> f64 {
    match house % 3 {
        1 => 60.0,
        2 => 30.0,
        _ => 15.0,
    }
}

/// 15 for a graha in the decanate its gender takes under `rule` (v. 6).
fn drekkana(graha: Graha, longitude: f64, rule: Drekkana) -> f64 {
    let degrees = longitude.rem_euclid(30.0);
    let (second, third) = if rule == Drekkana::MaleNeuterFemale {
        (Gender::Neutral, Gender::Female)
    } else {
        (Gender::Female, Gender::Neutral)
    };
    let decanate = if degrees < 10.0 {
        Gender::Male
    } else if degrees < 20.0 {
        second
    } else {
        third
    };
    if graha.attributes().descriptors.map(|d| d.gender) == Some(decanate) {
        15.0
    } else {
        0.0
    }
}

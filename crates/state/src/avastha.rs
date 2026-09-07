//! The avasthas: the ages, the wakefulness, the war, and the ones the
//! corpus cannot settle.
//!
//! Four families, and the module's stance on them is the point of it:
//!
//! - **Baladi**, the five ages, is a division of the sign that reverses
//!   in an even one. Exact over the corpus.
//! - **Jagradadi**, the wakefulness, follows the dignity. Exact.
//! - **Lajjitadi** has six members. Three of them are decided by facts
//!   the chart already carries; three are not.
//! - **Deeptadi** has nine. Three are decided; six are not.
//!
//! The six that are not are precisely the ones whose classical
//! definitions read "or aspected by", and the SDK has no aspect model
//! yet. So this module **reports nothing** where it cannot decide, rather
//! than a plausible guess: a caller can tell an absent answer from a
//! wrong one, and a wrong rule that reproduces a plausible-looking value
//! is never discovered (`03-design/state-and-avasthas.md` §7).

use serde::Serialize;
use teistro_core::catalogue::{
    AvasthaBaladi, AvasthaDeeptadi, AvasthaJagradadi, AvasthaLajjitadi, Dignity, Graha, Rashi,
    Relationship, Tatwa,
};

/// How near two planets must be to be at war, degrees.
///
/// The corpus brackets it between 0.96°, the widest war it records, and
/// 1.30°, the nearest pair that is not one.
pub const WAR_ORB_DEG: f64 = 1.0;

/// The five ages, from infant to dead.
const AGES: [AvasthaBaladi; 5] = [
    AvasthaBaladi::Bala,
    AvasthaBaladi::Kumara,
    AvasthaBaladi::Yuva,
    AvasthaBaladi::Vriddha,
    AvasthaBaladi::Mrita,
];

/// The degrees each age holds: a sign in five.
const AGE_SPAN_DEG: f64 = 6.0;

/// The house whose company shames a body.
const HOUSE_OF_SHAME: u8 = 5;

/// The bodies whose company shames another in that house.
const SHAMERS: [Graha; 5] = [
    Graha::Rahu,
    Graha::Ketu,
    Graha::Sun,
    Graha::Saturn,
    Graha::Mars,
];

/// The bodies that can go to war: not the luminaries, not the shadows.
pub const FIGHTERS: [Graha; 5] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// A body's age: which fifth of its sign it stands in, counted forward
/// in an odd sign and backward in an even one.
///
/// The alternation is most of the answer — reading it forward everywhere
/// is wrong on 359 of the corpus's 837 readings.
#[must_use]
pub fn age(sign: Rashi, degrees_in_sign: f64) -> AvasthaBaladi {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a degree of a sign over six is 0 to 4"
    )]
    let part = ((degrees_in_sign.max(0.0) / AGE_SPAN_DEG) as usize).min(4);
    // Aries is odd and its index is even, so an even index runs forward.
    let index = if (sign as u8) % 2 == 0 {
        part
    } else {
        4 - part
    };
    AGES.get(index).copied().unwrap_or(AvasthaBaladi::Bala)
}

/// A body's wakefulness, which follows its dignity.
///
/// The recording engine reports a **deeply** debilitated body dreaming
/// rather than asleep, which is what a switch that lists the plain
/// values and defaults for the rest does; the SDK treats the deeper
/// state as at least as bad, and the difference is registered.
#[must_use]
pub const fn wakefulness(dignity: Dignity) -> AvasthaJagradadi {
    match dignity {
        Dignity::Exalted | Dignity::DeepExalted | Dignity::Mooltrikona | Dignity::OwnSign => {
            AvasthaJagradadi::Jagrat
        }
        Dignity::Enemy | Dignity::GreatEnemy | Dignity::Debilitated | Dignity::DeepDebilitated => {
            AvasthaJagradadi::Sushupti
        }
        _ => AvasthaJagradadi::Swapna,
    }
}

/// What the chart says about one body, for the avasthas that read it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// Which body.
    pub graha: Graha,
    /// The sign it stands in.
    pub sign: Rashi,
    /// The bhava it falls in, 1 to 12.
    pub house: u8,
    /// Its dignity.
    pub dignity: Dignity,
    /// How it stands to the lord of the sign it is in, compounded: what
    /// "an enemy's sign" and "a friend's sign" mean.
    pub compound: Relationship,
}

/// Whether a state holds, certainly does not, or cannot be decided.
///
/// The third is not a failure and not a "no". A caller that treats an
/// undecided state as absent will be wrong about it; one that shows it
/// as unknown will not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Holds {
    /// It holds.
    Yes,
    /// It does not, and the chart is enough to say so.
    No,
    /// Nothing the SDK can compute decides it.
    Undecided,
}

/// The lajjitadi a chart decides, the ones it rules out, and the ones it
/// cannot decide either way.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Lajjitadi {
    /// The states that hold.
    pub holding: Vec<AvasthaLajjitadi>,
    /// The states that **certainly do not** hold, because the
    /// tradition's own necessary condition fails.
    pub ruled_out: Vec<AvasthaLajjitadi>,
    /// The states nothing decides: the necessary condition holds and
    /// what narrows it further is not in the chart. Named so a caller
    /// knows the list is short rather than empty.
    pub undecided: Vec<AvasthaLajjitadi>,
}

impl Lajjitadi {
    /// What the chart says about one state.
    #[must_use]
    pub fn state(&self, which: AvasthaLajjitadi) -> Holds {
        if self.holding.contains(&which) {
            Holds::Yes
        } else if self.undecided.contains(&which) {
            Holds::Undecided
        } else {
            Holds::No
        }
    }
}

/// The three whose classical definitions read "or aspected by", and
/// which no reading the corpus carries separates.
///
/// They are not simply unknown. Each has a **necessary** condition the
/// tradition states plainly — an enemy's sign, a watery sign, a friend's
/// sign — and `cargo xtask aspect` measured every one of them against
/// all 651 recorded readings: not one recorded state falls outside its
/// condition, and each condition holds on a good many readings the
/// engine does not record. So a body outside the condition certainly
/// does not hold the state, and a body inside it is undecided
/// (`03-design/aspect-drishti-measured.md` §7).
///
/// Adding the aspect clause the definitions also carry made every rule
/// *worse*, which is evidence the recording engine does not compute
/// these from a drishti at all — so this needs no aspect model and does
/// not wait on one.
pub const NARROWED_LAJJITADI: [AvasthaLajjitadi; 3] = [
    AvasthaLajjitadi::Kshudha,
    AvasthaLajjitadi::Trishita,
    AvasthaLajjitadi::Mudita,
];

/// Whether the tradition's necessary condition for one of the three
/// holds, which is what makes it undecided rather than ruled out.
#[must_use]
pub fn may_hold(state: AvasthaLajjitadi, placement: Placement) -> bool {
    match state {
        // The hungry: in a sign whose lord it is at odds with.
        AvasthaLajjitadi::Kshudha => matches!(
            placement.compound,
            Relationship::Enemy | Relationship::GreatEnemy
        ),
        // The thirsty: in a watery sign.
        AvasthaLajjitadi::Trishita => placement.sign.attributes().element == Tatwa::Jala,
        // The delighted: in a sign whose lord it is at ease with.
        AvasthaLajjitadi::Mudita => matches!(
            placement.compound,
            Relationship::Friend | Relationship::GreatFriend
        ),
        // Everything else is decided outright, so nothing is withheld.
        _ => false,
    }
}

/// Which lajjitadi hold for a body, which certainly do not, and which
/// the chart cannot decide.
///
/// Three are decided outright:
///
/// - **Garvita**, the proud: exalted or in its moolatrikona.
/// - **Lajjita**, the ashamed: in the fifth house sharing its sign with
///   the Sun, Mars, Saturn, Rahu or Ketu.
/// - **Kshobhita**, the agitated: sharing its sign with the Sun.
///
/// The other three carry a necessary condition and no sufficient one
/// ([`NARROWED_LAJJITADI`]), so each is either ruled out or undecided
/// and never asserted.
#[must_use]
pub fn lajjitadi(placement: Placement, chart: &[Placement]) -> Lajjitadi {
    let mut holding = Vec::new();
    if matches!(placement.dignity, Dignity::Exalted | Dignity::Mooltrikona) {
        holding.push(AvasthaLajjitadi::Garvita);
    }
    let shares_with = |who: Graha| {
        who != placement.graha
            && chart
                .iter()
                .any(|other| other.graha == who && other.sign == placement.sign)
    };
    if placement.house == HOUSE_OF_SHAME && SHAMERS.iter().copied().any(shares_with) {
        holding.push(AvasthaLajjitadi::Lajjita);
    }
    if shares_with(Graha::Sun) {
        holding.push(AvasthaLajjitadi::Kshobhita);
    }
    let (undecided, ruled_out) = NARROWED_LAJJITADI
        .into_iter()
        .partition(|state| may_hold(*state, placement));
    Lajjitadi {
        holding,
        ruled_out,
        undecided,
    }
}

/// The deeptadi a body's dignity decides, or `None`.
///
/// Only the top of the ladder is decidable: exaltation is Deepta, an own
/// sign or a moolatrikona is Swastha, a great friend's sign is Mudita.
/// Below that six states split on something the corpus does not record
/// and the SDK cannot yet compute, so the answer is nothing.
#[must_use]
pub const fn deeptadi(dignity: Dignity) -> Option<AvasthaDeeptadi> {
    match dignity {
        Dignity::Exalted | Dignity::DeepExalted => Some(AvasthaDeeptadi::Deepta),
        Dignity::OwnSign | Dignity::Mooltrikona => Some(AvasthaDeeptadi::Swastha),
        Dignity::GreatFriend => Some(AvasthaDeeptadi::Mudita),
        _ => None,
    }
}

/// Two planets at war, from the point of view of one of them.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct War {
    /// The other body.
    pub opponent: Graha,
    /// Whether this body won it.
    pub is_winner: bool,
    /// How far apart they stand, degrees.
    pub apart_deg: f64,
}

/// Where a body stands, for the war.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtWar {
    /// Which body.
    pub graha: Graha,
    /// Its longitude, degrees.
    pub longitude_deg: f64,
    /// Its ecliptic latitude, degrees: the victor is the northern body.
    pub latitude_deg: f64,
}

/// The war a body is in, if it is in one.
///
/// Two of the five planets within a degree of each other are at war, and
/// the **northern** body wins — true of all fourteen readings the corpus
/// carries. A body already at war with one neighbour and near another
/// takes the nearer of them, so the answer is one war and not a list.
#[must_use]
pub fn war(graha: Graha, chart: &[AtWar]) -> Option<War> {
    if !FIGHTERS.contains(&graha) {
        return None;
    }
    let me = chart.iter().find(|body| body.graha == graha)?;
    chart
        .iter()
        .filter(|other| other.graha != graha && FIGHTERS.contains(&other.graha))
        .map(|other| (separation(me.longitude_deg, other.longitude_deg), other))
        .filter(|(apart, _)| *apart < WAR_ORB_DEG)
        .min_by(|(first, _), (second, _)| first.total_cmp(second))
        .map(|(apart, other)| War {
            opponent: other.graha,
            is_winner: me.latitude_deg > other.latitude_deg,
            apart_deg: apart,
        })
}

/// The angle between two longitudes, degrees, the shorter way round.
#[must_use]
pub fn separation(first: f64, second: f64) -> f64 {
    let apart = (first - second).abs().rem_euclid(360.0);
    apart.min(360.0 - apart)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::{
        AtWar, FIGHTERS, Holds, NARROWED_LAJJITADI, Placement, age, deeptadi, lajjitadi, may_hold,
        separation, wakefulness, war,
    };
    use teistro_core::catalogue::{
        AvasthaBaladi, AvasthaDeeptadi, AvasthaJagradadi, AvasthaLajjitadi, Dignity, Graha, Rashi,
        Relationship,
    };

    #[test]
    fn the_ages_reverse_in_an_even_sign() {
        // Aries is odd: the first six degrees are infancy.
        assert_eq!(age(Rashi::Aries, 1.0), AvasthaBaladi::Bala);
        assert_eq!(age(Rashi::Aries, 29.0), AvasthaBaladi::Mrita);
        // Taurus is even: they run the other way.
        assert_eq!(age(Rashi::Taurus, 1.0), AvasthaBaladi::Mrita);
        assert_eq!(age(Rashi::Taurus, 29.0), AvasthaBaladi::Bala);
        // And they partition the sign.
        for sign in Rashi::ALL {
            let seen: Vec<AvasthaBaladi> = (0..5)
                .map(|part| age(sign, f64::from(part) * 6.0 + 3.0))
                .collect();
            assert_eq!(seen.len(), 5);
            for state in [
                AvasthaBaladi::Bala,
                AvasthaBaladi::Kumara,
                AvasthaBaladi::Yuva,
                AvasthaBaladi::Vriddha,
                AvasthaBaladi::Mrita,
            ] {
                assert!(seen.contains(&state), "{sign:?} misses {state:?}");
            }
        }
        // A degree past the sign's end still lands in the last part.
        assert_eq!(age(Rashi::Aries, 30.0), AvasthaBaladi::Mrita);
        assert_eq!(age(Rashi::Aries, -1.0), AvasthaBaladi::Bala);
    }

    #[test]
    fn the_deeper_state_is_never_the_milder_one() {
        assert_eq!(
            wakefulness(Dignity::Debilitated),
            AvasthaJagradadi::Sushupti
        );
        assert_eq!(
            wakefulness(Dignity::DeepDebilitated),
            AvasthaJagradadi::Sushupti,
            "the engine reports this dreaming; the SDK does not"
        );
        assert_eq!(wakefulness(Dignity::Exalted), AvasthaJagradadi::Jagrat);
        assert_eq!(wakefulness(Dignity::OwnSign), AvasthaJagradadi::Jagrat);
        assert_eq!(wakefulness(Dignity::Neutral), AvasthaJagradadi::Swapna);
        assert_eq!(wakefulness(Dignity::GreatFriend), AvasthaJagradadi::Swapna);
    }

    #[test]
    fn the_deeptadi_answers_only_where_it_can() {
        assert_eq!(deeptadi(Dignity::Exalted), Some(AvasthaDeeptadi::Deepta));
        assert_eq!(deeptadi(Dignity::OwnSign), Some(AvasthaDeeptadi::Swastha));
        assert_eq!(
            deeptadi(Dignity::Mooltrikona),
            Some(AvasthaDeeptadi::Swastha)
        );
        assert_eq!(
            deeptadi(Dignity::GreatFriend),
            Some(AvasthaDeeptadi::Mudita)
        );
        // And nothing below it, rather than a plausible guess.
        for dignity in [
            Dignity::Friend,
            Dignity::Neutral,
            Dignity::Enemy,
            Dignity::GreatEnemy,
            Dignity::Debilitated,
            Dignity::DeepDebilitated,
        ] {
            assert_eq!(deeptadi(dignity), None, "{dignity:?}");
        }
    }

    #[test]
    fn the_three_lajjitadi_the_chart_decides() {
        let sun = Placement {
            graha: Graha::Sun,
            sign: Rashi::Leo,
            house: 5,
            dignity: Dignity::OwnSign,
            compound: Relationship::GreatFriend,
        };
        let mars = Placement {
            graha: Graha::Mars,
            sign: Rashi::Leo,
            house: 5,
            dignity: Dignity::Neutral,
            compound: Relationship::Neutral,
        };
        let chart = [sun, mars];
        // Mars shares the Sun's sign in the fifth: ashamed and agitated.
        let found = lajjitadi(mars, &chart);
        assert!(found.holding.contains(&AvasthaLajjitadi::Kshobhita));
        assert!(found.holding.contains(&AvasthaLajjitadi::Lajjita));
        assert!(!found.holding.contains(&AvasthaLajjitadi::Garvita));
        // Mars in Leo is neutral to the Sun and Leo is not watery, so
        // all three of the narrowed states are ruled out and none is
        // left undecided.
        assert!(found.undecided.is_empty(), "{found:?}");
        assert_eq!(found.ruled_out, NARROWED_LAJJITADI.to_vec());
        assert_eq!(found.state(AvasthaLajjitadi::Kshudha), Holds::No);
        assert_eq!(found.state(AvasthaLajjitadi::Kshobhita), Holds::Yes);
        // The Sun does not agitate itself.
        assert!(
            !lajjitadi(sun, &chart)
                .holding
                .contains(&AvasthaLajjitadi::Kshobhita)
        );
        // An exalted body elsewhere is proud and nothing else.
        let jupiter = Placement {
            graha: Graha::Jupiter,
            sign: Rashi::Cancer,
            house: 4,
            dignity: Dignity::Exalted,
            compound: Relationship::GreatFriend,
        };
        let proud = lajjitadi(jupiter, &[jupiter, sun]);
        assert_eq!(proud.holding, vec![AvasthaLajjitadi::Garvita]);
        // Cancer is watery and its lord is Jupiter's great friend, so
        // two of the three are undecided and the hungry one is not.
        assert_eq!(
            proud.undecided,
            vec![AvasthaLajjitadi::Trishita, AvasthaLajjitadi::Mudita]
        );
        assert_eq!(proud.ruled_out, vec![AvasthaLajjitadi::Kshudha]);
        assert_eq!(proud.state(AvasthaLajjitadi::Trishita), Holds::Undecided);
        assert_eq!(proud.state(AvasthaLajjitadi::Kshudha), Holds::No);
    }

    #[test]
    fn a_necessary_condition_rules_a_state_out_and_never_asserts_one() {
        let watery = Placement {
            graha: Graha::Moon,
            sign: Rashi::Scorpio,
            house: 1,
            dignity: Dignity::Debilitated,
            compound: Relationship::Enemy,
        };
        assert!(
            may_hold(AvasthaLajjitadi::Trishita, watery),
            "a watery sign"
        );
        assert!(may_hold(AvasthaLajjitadi::Kshudha, watery), "an enemy's");
        assert!(!may_hold(AvasthaLajjitadi::Mudita, watery));
        // A state the chart decides outright is never withheld.
        for state in [
            AvasthaLajjitadi::Garvita,
            AvasthaLajjitadi::Lajjita,
            AvasthaLajjitadi::Kshobhita,
        ] {
            assert!(!may_hold(state, watery), "{state:?}");
        }
        // The three lists partition the family: every member appears
        // exactly once.
        let found = lajjitadi(watery, &[watery]);
        let mut all = found.holding.clone();
        all.extend(found.ruled_out.iter().copied());
        all.extend(found.undecided.iter().copied());
        for state in NARROWED_LAJJITADI {
            assert_eq!(
                all.iter().filter(|seen| **seen == state).count(),
                1,
                "{state:?}"
            );
        }
    }

    #[test]
    fn a_war_is_symmetric_and_exactly_one_side_wins() {
        let chart = [
            AtWar {
                graha: Graha::Mars,
                longitude_deg: 100.0,
                latitude_deg: 1.0,
            },
            AtWar {
                graha: Graha::Venus,
                longitude_deg: 100.5,
                latitude_deg: -1.0,
            },
            AtWar {
                graha: Graha::Saturn,
                longitude_deg: 200.0,
                latitude_deg: 0.0,
            },
        ];
        let mars = war(Graha::Mars, &chart).expect("within a degree");
        let venus = war(Graha::Venus, &chart).expect("within a degree");
        assert_eq!(mars.opponent, Graha::Venus);
        assert_eq!(venus.opponent, Graha::Mars);
        assert!(mars.is_winner, "the northern body wins");
        assert!(!venus.is_winner);
        assert!((mars.apart_deg - venus.apart_deg).abs() < 1e-12);
        // Saturn is far away and fights nobody.
        assert!(war(Graha::Saturn, &chart).is_none());
        // Neither the luminaries nor the shadows fight at all.
        for graha in [Graha::Sun, Graha::Moon, Graha::Rahu, Graha::Ketu] {
            assert!(!FIGHTERS.contains(&graha), "{graha:?}");
            assert!(war(graha, &chart).is_none(), "{graha:?}");
        }
    }

    #[test]
    fn a_war_takes_the_nearer_opponent() {
        let chart = [
            AtWar {
                graha: Graha::Mars,
                longitude_deg: 100.0,
                latitude_deg: 0.0,
            },
            AtWar {
                graha: Graha::Venus,
                longitude_deg: 100.9,
                latitude_deg: 0.0,
            },
            AtWar {
                graha: Graha::Mercury,
                longitude_deg: 100.2,
                latitude_deg: 0.0,
            },
        ];
        assert_eq!(
            war(Graha::Mars, &chart).expect("two are near").opponent,
            Graha::Mercury
        );
    }

    #[test]
    fn a_separation_is_the_shorter_way_round() {
        assert!((separation(10.0, 350.0) - 20.0).abs() < 1e-9);
        assert!((separation(0.0, 180.0) - 180.0).abs() < 1e-9);
        assert!(separation(5.0, 5.0).abs() < 1e-9);
    }
}

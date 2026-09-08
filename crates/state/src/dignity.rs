//! The dignity ladder and the three friendship readings.
//!
//! Two things in here are not where a reader would put them, and the
//! corpus put them there (`03-design/state-tables-measured.md` §§1–2):
//!
//! - **Moolatrikona is checked before exaltation.** Three grahas have a
//!   moolatrikona span inside their exaltation sign, and where the two
//!   overlap the answer is the moolatrikona.
//! - **A body in its own sign is its own friend**, in both friendship
//!   readings. The catalogue structurally cannot say it — a graha is in
//!   none of its own three lists — so the rule lives here.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Dignity, Graha, Rashi, Relationship};

/// The signs from a body in which its dispositor is a temporary friend:
/// the 2nd, 3rd, 4th, 10th, 11th and 12th.
pub const FRIENDLY_HOUSES: [u8; 6] = [2, 3, 4, 10, 11, 12];

/// How near the exact debilitation degree a debilitation must be to be a
/// deep one, degrees.
///
/// The corpus brackets it between 0.75° and 1.51° for the seven grahas,
/// so a whole degree sits inside. It cannot separate that from a smaller
/// orb applied to every body including the shadow grahas; the SDK takes
/// the first, because the nodes are already outside the rest of the
/// ladder (`03-design/state-and-avasthas.md` §12).
pub const DEEP_DEBILITATION_ORB_DEG: f64 = 1.0;

/// Whether a graha is one of the two shadow grahas, which own no sign
/// and whose dignity ends at neutral.
#[must_use]
pub const fn is_shadow(graha: Graha) -> bool {
    matches!(graha, Graha::Rahu | Graha::Ketu)
}

/// How a body stands to the lord of the sign it is in, three ways.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Friendship {
    /// The table's own reading.
    pub natural: Relationship,
    /// Where the dispositor stands.
    pub temporary: Relationship,
    /// The five-fold compound of the two.
    pub compound: Relationship,
    /// The lord of the sign, which is what all three are with. `None`
    /// only for a body the catalogue gives no sign.
    pub dispositor: Option<Graha>,
}

/// Whether a graha stands in one of its own signs.
///
/// Both readings the catalogue offers: the graha's own list, and the
/// sign's lordship, which agree for every shipped member and would catch
/// a future one where they did not.
#[must_use]
pub fn in_own_sign(graha: Graha, sign: Rashi) -> bool {
    graha.attributes().own.contains(&sign) || sign.attributes().lord == graha
}

/// The natural friendship a graha has with the lord of a sign.
#[must_use]
pub fn natural(graha: Graha, sign: Rashi) -> Relationship {
    let lord = sign.attributes().lord;
    if lord == graha {
        // A body in its own sign is its own friend. Not in the
        // catalogue, because a graha is in none of its own lists.
        return Relationship::Friend;
    }
    let attributes = graha.attributes();
    if attributes.friends.contains(&lord) {
        Relationship::Friend
    } else if attributes.enemies.contains(&lord) {
        Relationship::Enemy
    } else {
        Relationship::Neutral
    }
}

/// The temporary friendship a graha has with its dispositor, given where
/// every graha stands.
///
/// `None` when the chart does not carry the dispositor, which a partial
/// chart may not.
#[must_use]
pub fn temporary(
    graha: Graha,
    sign: Rashi,
    sign_of: impl Fn(Graha) -> Option<Rashi>,
) -> Option<Relationship> {
    let lord = sign.attributes().lord;
    if lord == graha {
        return Some(Relationship::Friend);
    }
    let at = sign_of(lord)?;
    let distance = (at as u8 + 12 - sign as u8) % 12 + 1;
    Some(if FRIENDLY_HOUSES.contains(&distance) {
        Relationship::Friend
    } else {
        Relationship::Enemy
    })
}

/// The five-fold compound of the two friendship readings.
///
/// The classical table: two friendships make a great friend, two
/// enmities a great enemy, and a disagreement is neutral whichever way
/// round it falls.
#[must_use]
pub const fn compound(natural: Relationship, temporary: Relationship) -> Relationship {
    match (natural, temporary) {
        (Relationship::Friend, Relationship::Friend) => Relationship::GreatFriend,
        (Relationship::Neutral, Relationship::Friend) => Relationship::Friend,
        (Relationship::Neutral, Relationship::Enemy) => Relationship::Enemy,
        (Relationship::Enemy, Relationship::Enemy) => Relationship::GreatEnemy,
        _ => Relationship::Neutral,
    }
}

/// Every friendship reading for a graha in a chart.
#[must_use]
pub fn friendship(
    graha: Graha,
    sign: Rashi,
    sign_of: impl Fn(Graha) -> Option<Rashi>,
) -> Friendship {
    let natural = natural(graha, sign);
    let temporary = temporary(graha, sign, sign_of).unwrap_or(Relationship::Neutral);
    Friendship {
        natural,
        temporary,
        compound: compound(natural, temporary),
        dispositor: Some(sign.attributes().lord),
    }
}

/// The dignity of a graha standing at so many degrees of a sign.
///
/// `compound` is its five-fold friendship with the dispositor, which the
/// ladder falls through to and the shadow grahas never reach.
#[must_use]
pub fn dignity(graha: Graha, sign: Rashi, degrees_in_sign: f64, compound: Relationship) -> Dignity {
    let attributes = graha.attributes();
    // Moolatrikona first: three grahas have a span inside their own
    // exaltation sign, and there the moolatrikona is the answer.
    if let Some(span) = attributes.moolatrikona
        && span.sign == sign
        && span.to > span.from
        && degrees_in_sign >= f64::from(span.from)
        && degrees_in_sign < f64::from(span.to)
    {
        return Dignity::Mooltrikona;
    }
    if attributes.exaltation.is_some_and(|at| at.sign == sign) {
        return Dignity::Exalted;
    }
    if let Some(at) = attributes.debilitation
        && at.sign == sign
    {
        let deep = !is_shadow(graha)
            && (degrees_in_sign - f64::from(at.degree)).abs() <= DEEP_DEBILITATION_ORB_DEG;
        return if deep {
            Dignity::DeepDebilitated
        } else {
            Dignity::Debilitated
        };
    }
    if in_own_sign(graha, sign) {
        return Dignity::OwnSign;
    }
    if is_shadow(graha) {
        // The shadows own no sign and take no friendship dignity.
        return Dignity::Neutral;
    }
    match compound {
        Relationship::GreatFriend => Dignity::GreatFriend,
        Relationship::Friend => Dignity::Friend,
        Relationship::Enemy => Dignity::Enemy,
        Relationship::GreatEnemy => Dignity::GreatEnemy,
        _ => Dignity::Neutral,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::{compound, dignity, friendship, in_own_sign, natural, temporary};
    use teistro_core::catalogue::{Dignity, Graha, Rashi, Relationship};

    /// A chart in which only the named grahas stand anywhere.
    fn placed(pairs: &'static [(Graha, Rashi)]) -> impl Fn(Graha) -> Option<Rashi> {
        move |graha| {
            pairs
                .iter()
                .find(|(who, _)| *who == graha)
                .map(|(_, sign)| *sign)
        }
    }

    #[test]
    fn a_body_in_its_own_sign_is_its_own_friend() {
        assert!(in_own_sign(Graha::Sun, Rashi::Leo));
        assert!(!in_own_sign(Graha::Sun, Rashi::Cancer));
        assert_eq!(natural(Graha::Sun, Rashi::Leo), Relationship::Friend);
        assert_eq!(
            temporary(Graha::Sun, Rashi::Leo, placed(&[])),
            Some(Relationship::Friend),
            "and without needing to know where anything stands"
        );
        // The catalogue's table otherwise.
        assert_eq!(natural(Graha::Sun, Rashi::Cancer), Relationship::Friend);
        assert_eq!(natural(Graha::Sun, Rashi::Taurus), Relationship::Enemy);
        assert_eq!(natural(Graha::Sun, Rashi::Gemini), Relationship::Neutral);
    }

    #[test]
    fn the_temporary_friendship_is_where_the_dispositor_stands() {
        // The Sun in Aries; Mars, its dispositor, in Taurus — the second
        // from it, which is friendly.
        let chart = placed(&[(Graha::Mars, Rashi::Taurus)]);
        assert_eq!(
            temporary(Graha::Sun, Rashi::Aries, &chart),
            Some(Relationship::Friend)
        );
        // Mars in Leo, the fifth from Aries, which is not.
        let chart = placed(&[(Graha::Mars, Rashi::Leo)]);
        assert_eq!(
            temporary(Graha::Sun, Rashi::Aries, &chart),
            Some(Relationship::Enemy)
        );
        // Mars in Aries itself: the first, and an enemy — sharing a sign
        // with the dispositor is not the same as being it.
        let chart = placed(&[(Graha::Mars, Rashi::Aries)]);
        assert_eq!(
            temporary(Graha::Moon, Rashi::Aries, &chart),
            Some(Relationship::Enemy)
        );
        // A chart without the dispositor cannot answer.
        assert_eq!(temporary(Graha::Sun, Rashi::Aries, placed(&[])), None);
    }

    #[test]
    fn the_compound_is_the_classical_table() {
        use Relationship::{Enemy, Friend, GreatEnemy, GreatFriend, Neutral};
        assert_eq!(compound(Friend, Friend), GreatFriend);
        assert_eq!(compound(Friend, Enemy), Neutral);
        assert_eq!(compound(Neutral, Friend), Friend);
        assert_eq!(compound(Neutral, Enemy), Enemy);
        assert_eq!(compound(Enemy, Friend), Neutral);
        assert_eq!(compound(Enemy, Enemy), GreatEnemy);
    }

    #[test]
    fn moolatrikona_is_checked_before_exaltation() {
        // The Moon's exaltation is Taurus and its moolatrikona is Taurus
        // 3° to 30°, so the two overlap for most of the sign.
        assert_eq!(
            dignity(Graha::Moon, Rashi::Taurus, 1.0, Relationship::Neutral),
            Dignity::Exalted,
            "below the span it is exalted"
        );
        assert_eq!(
            dignity(Graha::Moon, Rashi::Taurus, 18.0, Relationship::Neutral),
            Dignity::Mooltrikona,
            "and inside it the moolatrikona wins"
        );
    }

    #[test]
    fn a_debilitation_is_deep_within_a_degree_and_never_for_a_shadow() {
        // Saturn's debilitation is Aries 20°.
        assert_eq!(
            dignity(Graha::Saturn, Rashi::Aries, 20.5, Relationship::Neutral),
            Dignity::DeepDebilitated
        );
        assert_eq!(
            dignity(Graha::Saturn, Rashi::Aries, 22.0, Relationship::Neutral),
            Dignity::Debilitated
        );
        // Rahu's is Scorpio 20°, and it never takes the deep form.
        assert_eq!(
            dignity(Graha::Rahu, Rashi::Scorpio, 20.0, Relationship::Neutral),
            Dignity::Debilitated
        );
    }

    #[test]
    fn a_shadow_graha_falls_to_neutral_where_a_graha_takes_its_friendship() {
        // Neither owns a sign, so a friendship dignity never applies.
        for shadow in [Graha::Rahu, Graha::Ketu] {
            assert_eq!(
                dignity(shadow, Rashi::Leo, 5.0, Relationship::GreatEnemy),
                Dignity::Neutral,
                "{shadow:?}"
            );
        }
        // A graha in the same place takes the friendship.
        assert_eq!(
            dignity(Graha::Saturn, Rashi::Leo, 5.0, Relationship::GreatEnemy),
            Dignity::GreatEnemy
        );
    }

    #[test]
    fn the_ladder_is_total() {
        // Every graha at every degree of every sign gets an answer, and
        // an own sign is never reported as a friendship.
        for graha in [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
            Graha::Rahu,
            Graha::Ketu,
        ] {
            for sign in Rashi::ALL {
                for tenth in 0..300 {
                    let degrees = f64::from(tenth) / 10.0;
                    let found = dignity(graha, sign, degrees, Relationship::Neutral);
                    if in_own_sign(graha, sign) {
                        assert!(
                            matches!(
                                found,
                                Dignity::OwnSign
                                    | Dignity::Mooltrikona
                                    | Dignity::Exalted
                                    | Dignity::Debilitated
                                    | Dignity::DeepDebilitated
                            ),
                            "{graha:?} in {sign:?} at {degrees}: {found:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_friendship_names_the_dispositor_it_is_with() {
        let found = friendship(
            Graha::Sun,
            Rashi::Taurus,
            placed(&[(Graha::Venus, Rashi::Gemini)]),
        );
        assert_eq!(found.dispositor, Some(Graha::Venus));
        assert_eq!(found.natural, Relationship::Enemy);
        assert_eq!(found.temporary, Relationship::Friend, "the second from it");
        assert_eq!(found.compound, Relationship::Neutral);
    }
}

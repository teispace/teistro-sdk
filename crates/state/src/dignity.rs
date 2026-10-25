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

/// How near the exact exaltation degree an exaltation must be to be a deep
/// one, degrees: the band crux C18 names, the same degree as the
/// debilitation's. The corpus holds no exalted graha within it, so it
/// cannot bracket this orb as it does the other; without it no graha was
/// ever deeply exalted, and a rule naming that dignity alone, Jataka
/// Parijata's Lakshmi, could never hold.
pub const DEEP_EXALTATION_ORB_DEG: f64 = 1.0;

/// Whether a graha is one of the two shadow grahas, which own no sign
/// and whose dignity ends at neutral.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_state::dignity::is_shadow;
///
/// assert!(is_shadow(Graha::Ketu));
/// assert!(!is_shadow(Graha::Saturn));
/// ```
#[must_use]
pub const fn is_shadow(graha: Graha) -> bool {
    matches!(graha, Graha::Rahu | Graha::Ketu)
}

/// How a body stands to the lord of the sign it is in, three ways.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi, Relationship};
/// use teistro_state::dignity::friendship;
///
/// // The Sun in Gemini, Mercury in Leo: Mercury is neutral to the Sun by
/// // nature and a friend from the third, so a friend in all.
/// let sign_of = |graha| (graha == Graha::Mercury).then_some(Rashi::Leo);
/// let reading = friendship(Graha::Sun, Rashi::Gemini, sign_of);
/// assert_eq!((reading.natural, reading.temporary, reading.compound), (Relationship::Neutral, Relationship::Friend, Relationship::Friend));
/// assert_eq!(reading.dispositor, Some(Graha::Mercury));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_state::dignity::in_own_sign;
///
/// assert!(in_own_sign(Graha::Mars, Rashi::Scorpio));
/// assert!(!in_own_sign(Graha::Mars, Rashi::Leo));
/// ```
#[must_use]
pub fn in_own_sign(graha: Graha, sign: Rashi) -> bool {
    graha.attributes().own.contains(&sign) || sign.attributes().lord == graha
}

/// The natural friendship a graha has with the lord of a sign.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi, Relationship};
/// use teistro_state::dignity::natural;
///
/// // The Sun's friends are the Moon, Mars and Jupiter; Venus and Saturn its
/// // enemies; Mercury neutral.
/// assert_eq!(natural(Graha::Sun, Rashi::Aries), Relationship::Friend);
/// assert_eq!(natural(Graha::Sun, Rashi::Taurus), Relationship::Enemy);
/// assert_eq!(natural(Graha::Sun, Rashi::Gemini), Relationship::Neutral);
/// assert_eq!(natural(Graha::Sun, Rashi::Leo), Relationship::Friend);
/// ```
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
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi, Relationship};
/// use teistro_state::dignity::temporary;
///
/// // The Sun in Aries: Mars in Gemini, the third, is a friend; in Libra, the
/// // seventh, an enemy.
/// assert_eq!(temporary(Graha::Sun, Rashi::Aries, |_| Some(Rashi::Gemini)), Some(Relationship::Friend));
/// assert_eq!(temporary(Graha::Sun, Rashi::Aries, |_| Some(Rashi::Libra)), Some(Relationship::Enemy));
/// // A chart that does not carry Mars cannot say.
/// assert_eq!(temporary(Graha::Sun, Rashi::Aries, |_| None), None);
/// ```
#[must_use]
pub fn temporary(
    graha: Graha,
    sign: Rashi,
    sign_of: impl Fn(Graha) -> Option<Rashi>,
) -> Option<Relationship> {
    temporary_from(graha, sign.attributes().lord, sign, sign_of)
}

/// The temporary friendship of a graha seated in `seat` with `lord`, wherever
/// `sign_of` places it: a friend from the second, third, fourth, tenth,
/// eleventh or twelfth sign, else an enemy, and its own friend.
///
/// [`temporary`] measures from the sign the graha stands in; a divisional
/// chart's dignity measures from its rasi seat instead.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi, Relationship};
/// use teistro_state::dignity::temporary_from;
///
/// // Venus seated in Pisces, Jupiter in Capricorn: the eleventh from it.
/// let friend = temporary_from(Graha::Venus, Graha::Jupiter, Rashi::Pisces, |_| Some(Rashi::Capricorn));
/// assert_eq!(friend, Some(Relationship::Friend));
/// ```
#[must_use]
pub fn temporary_from(
    graha: Graha,
    lord: Graha,
    seat: Rashi,
    sign_of: impl Fn(Graha) -> Option<Rashi>,
) -> Option<Relationship> {
    if lord == graha {
        return Some(Relationship::Friend);
    }
    let at = sign_of(lord)?;
    let distance = (at as u8 + 12 - seat as u8) % 12 + 1;
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
///
/// ```
/// use teistro_core::catalogue::Relationship::{Enemy, Friend, GreatEnemy, GreatFriend, Neutral};
/// use teistro_state::dignity::compound;
///
/// assert_eq!(compound(Friend, Friend), GreatFriend);
/// assert_eq!(compound(Enemy, Enemy), GreatEnemy);
/// assert_eq!(compound(Neutral, Friend), Friend);
/// // A disagreement is neutral whichever way round.
/// assert_eq!(compound(Friend, Enemy), Neutral);
/// assert_eq!(compound(Enemy, Friend), Neutral);
/// ```
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

/// The dignity of a graha in a divisional chart's sign, which carries no
/// degrees.
///
/// By the sign alone, in the order BPHS ch. 28 vv. 7 to 9 rank them:
/// exaltation, debilitation, moolatrikona, the own sign; then the compound
/// friendship with the sign's lord, its temporary half read between the
/// graha's and the lord's rasi seats as `rasi_sign_of` gives them (B.V.
/// Raman, Art. 30). The shadow grahas stop at neutral, as in [`dignity`].
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// use teistro_state::dignity::varga_dignity;
///
/// // By the sign alone: the Sun's navamsha in Libra is its debilitation.
/// assert_eq!(varga_dignity(Graha::Sun, Rashi::Libra, |_| None), Dignity::Debilitated);
/// // In Gemini, Mercury neutral by nature and a friend from the Sun's rasi
/// // seat in Aries to its own in Pisces, the twelfth.
/// let seats = |graha| Some(if graha == Graha::Sun { Rashi::Aries } else { Rashi::Pisces });
/// assert_eq!(varga_dignity(Graha::Sun, Rashi::Gemini, seats), Dignity::Friend);
/// ```
#[must_use]
pub fn varga_dignity(
    graha: Graha,
    sign: Rashi,
    rasi_sign_of: impl Fn(Graha) -> Option<Rashi>,
) -> Dignity {
    let attributes = graha.attributes();
    if attributes.exaltation.is_some_and(|at| at.sign == sign) {
        return Dignity::Exalted;
    }
    if attributes.debilitation.is_some_and(|at| at.sign == sign) {
        return Dignity::Debilitated;
    }
    if attributes
        .moolatrikona
        .is_some_and(|span| span.sign == sign)
    {
        return Dignity::Mooltrikona;
    }
    if in_own_sign(graha, sign) {
        return Dignity::OwnSign;
    }
    if is_shadow(graha) {
        return Dignity::Neutral;
    }
    let temporary = rasi_sign_of(graha)
        .and_then(|seat| temporary_from(graha, sign.attributes().lord, seat, &rasi_sign_of))
        .unwrap_or(Relationship::Neutral);
    match compound(natural(graha, sign), temporary) {
        Relationship::GreatFriend => Dignity::GreatFriend,
        Relationship::Friend => Dignity::Friend,
        Relationship::Enemy => Dignity::Enemy,
        Relationship::GreatEnemy => Dignity::GreatEnemy,
        _ => Dignity::Neutral,
    }
}

/// Every friendship reading for a graha in a chart.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi, Relationship};
/// use teistro_state::dignity::friendship;
///
/// // A body in its own sign is its own friend, twice over.
/// let own = friendship(Graha::Sun, Rashi::Leo, |_| Some(Rashi::Leo));
/// assert_eq!(own.compound, Relationship::GreatFriend);
/// ```
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
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi, Relationship};
/// use teistro_state::dignity::dignity;
///
/// let neutral = Relationship::Neutral;
/// // The Sun is exalted in Aries, deeply within a degree of 10°.
/// assert_eq!(dignity(Graha::Sun, Rashi::Aries, 10.5, neutral), Dignity::DeepExalted);
/// assert_eq!(dignity(Graha::Sun, Rashi::Aries, 25.0, neutral), Dignity::Exalted);
/// // Its moolatrikona is the first twenty degrees of Leo, its own sign after.
/// assert_eq!(dignity(Graha::Sun, Rashi::Leo, 10.0, neutral), Dignity::Mooltrikona);
/// assert_eq!(dignity(Graha::Sun, Rashi::Leo, 25.0, neutral), Dignity::OwnSign);
/// assert_eq!(dignity(Graha::Sun, Rashi::Libra, 10.0, neutral), Dignity::DeepDebilitated);
/// // Elsewhere the friendship decides.
/// assert_eq!(dignity(Graha::Sun, Rashi::Gemini, 5.0, Relationship::Friend), Dignity::Friend);
/// ```
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
    if let Some(at) = attributes.exaltation
        && at.sign == sign
    {
        let deep = !is_shadow(graha)
            && (degrees_in_sign - f64::from(at.degree)).abs() <= DEEP_EXALTATION_ORB_DEG;
        return if deep {
            Dignity::DeepExalted
        } else {
            Dignity::Exalted
        };
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn an_exaltation_is_deep_within_a_degree_and_never_for_a_shadow() {
        // Saturn's exaltation is Libra 20°.
        assert_eq!(
            dignity(Graha::Saturn, Rashi::Libra, 19.5, Relationship::Neutral),
            Dignity::DeepExalted
        );
        assert_eq!(
            dignity(Graha::Saturn, Rashi::Libra, 18.0, Relationship::Neutral),
            Dignity::Exalted
        );
        // Rahu's is Taurus 20°, and it never takes the deep form.
        assert_eq!(
            dignity(Graha::Rahu, Rashi::Taurus, 20.0, Relationship::Neutral),
            Dignity::Exalted
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
                                    | Dignity::DeepExalted
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

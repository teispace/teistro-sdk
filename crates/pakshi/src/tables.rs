//! The birds, the activities and the printed rules, each cited to its
//! page (`docs/03-design/pakshi.md` §2). Short names: AG is Agastya's
//! *Pañcapaṭci Sāstiram* (Kanchipuram, 1880, by PDF page), AG07 its 1907
//! print with Santhalinga Swamigal's commentary, AY is Ayyar's *South
//! Indian Customs* (1925), PUL is Pulippani's *Biorhythms of Natal Moon*
//! (1993, in copyright: facts only).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Nakshatra, Paksha, Vara};

/// The five birds, in the tradition's order (AY p. 102; AG p4 v. 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Bird {
    /// *Vallūṟu*: Ayyar's hawk, the modern books' vulture.
    Vulture,
    /// *Āntai*, the owl.
    Owl,
    /// *Kākam*, the crow.
    Crow,
    /// *Kōḻi*, the cock.
    Cock,
    /// *Mayil*, the peacock.
    Peacock,
}

impl Bird {
    /// Every bird, in the tradition's order.
    pub const ALL: [Bird; 5] = [
        Bird::Vulture,
        Bird::Owl,
        Bird::Crow,
        Bird::Cock,
        Bird::Peacock,
    ];

    /// The bird's place in the order, from 0.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The bird at place `index` round the order.
    #[must_use]
    pub const fn at(index: usize) -> Bird {
        match index % 5 {
            0 => Bird::Vulture,
            1 => Bird::Owl,
            2 => Bird::Crow,
            3 => Bird::Cock,
            _ => Bird::Peacock,
        }
    }

    /// The bird's natural strength in eighths (AG p40, the *mūkku pakṣi*
    /// verse): the crow whole, the vulture three quarters, the owl a half,
    /// the cock a quarter, the peacock an eighth.
    #[must_use]
    pub const fn strength_eighths(self) -> u8 {
        match self {
            Bird::Crow => 8,
            Bird::Vulture => 6,
            Bird::Owl => 4,
            Bird::Cock => 2,
            Bird::Peacock => 1,
        }
    }
}

/// What a bird does in a yama or a sub-period (AG p5 v. 6; AY p. 102).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Activity {
    /// *Ūṇ*, eating.
    Eating,
    /// *Naṭai*, walking.
    Walking,
    /// *Aracu*, ruling.
    Ruling,
    /// *Tuyil*, sleeping.
    Sleeping,
    /// *Cāvu*, dying.
    Dying,
}

/// How an activity is judged for an undertaking (AY p. 102; PUL pp. 18–19).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Quality {
    /// Ruling or eating: a time to begin.
    Good,
    /// Walking: middling.
    Middling,
    /// Sleeping or dying: a time to begin nothing.
    Bad,
}

impl Activity {
    /// How the activity is judged.
    #[must_use]
    pub const fn quality(self) -> Quality {
        match self {
            Activity::Ruling | Activity::Eating => Quality::Good,
            Activity::Walking => Quality::Middling,
            Activity::Sleeping | Activity::Dying => Quality::Bad,
        }
    }
}

/// The two halves of a day: sunrise to sunset, and sunset to the next
/// sunrise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Half {
    /// Sunrise to sunset.
    Day,
    /// Sunset to the next sunrise.
    Night,
}

/// Whether the paksha is the bright one. The catalogue's `Paksha` may
/// gain members; anything but Shukla reads as the dark half.
const fn bright(paksha: Paksha) -> bool {
    matches!(paksha, Paksha::Shukla)
}

/// The order a half's activities run in, each bird stepping one place a
/// yama (AG p5 v. 6, p15 v. 3, p25 v. 3; AY pp. 106–109).
#[must_use]
pub const fn sequence(paksha: Paksha, half: Half) -> [Activity; 5] {
    use Activity::{Dying as D, Eating as E, Ruling as R, Sleeping as S, Walking as W};
    match (bright(paksha), half) {
        (true, Half::Day) => [E, W, R, S, D],
        (true, Half::Night) => [E, R, D, W, S],
        (false, Half::Day) => [E, D, S, R, W],
        (false, Half::Night) => [E, S, W, D, R],
    }
}

/// How far apart in the sequence two neighbouring birds stand: the rule
/// the four printed tables follow (`pakshi.md` §3).
const fn step(paksha: Paksha, half: Half) -> usize {
    match (bright(paksha), half) {
        (true, Half::Day) => 1,
        (false, Half::Day) => 2,
        (_, Half::Night) => 4,
    }
}

/// The bird that eats in the first yama of a weekday's half (AG p15 v. 1,
/// p20 v. 1, p25 v. 1; AY pp. 106–109).
#[must_use]
pub fn first_eater(paksha: Paksha, half: Half, vara: Vara) -> Bird {
    use Bird::{Cock as K, Crow as C, Owl as O, Peacock as P, Vulture as V};
    let row: [Bird; 7] = match (bright(paksha), half) {
        (true, Half::Day) => [V, O, V, O, C, K, P],
        (true, Half::Night) => [C, K, C, K, P, V, O],
        (false, Half::Day) => [K, P, K, C, O, V, P],
        (false, Half::Night) => [V, K, V, O, C, P, K],
    };
    row.get(usize::from(vara.id())).copied().unwrap_or(V)
}

/// A bird's activity in yama `yama` (0 to 4) of a weekday's half: its
/// place behind the first yama's eater times the half's step, moved on a
/// place a yama, read in the half's sequence. It reproduces every cell of
/// Ayyar's four tables but one, his printing slip (crux P7).
#[must_use]
pub fn activity(bird: Bird, paksha: Paksha, half: Half, vara: Vara, yama: usize) -> Activity {
    let behind = (bird.index() + 5 - first_eater(paksha, half, vara).index()) % 5;
    let place = (behind * step(paksha, half) + yama) % 5;
    sequence(paksha, half)
        .get(place)
        .copied()
        .unwrap_or(Activity::Eating)
}

/// The bird doing `activity` in yama `yama` of a weekday's half. Each
/// yama the five birds hold the five activities, so there is always one.
#[must_use]
pub fn doing(activity_done: Activity, paksha: Paksha, half: Half, vara: Vara, yama: usize) -> Bird {
    Bird::ALL
        .into_iter()
        .find(|bird| activity(*bird, paksha, half, vara, yama) == activity_done)
        .unwrap_or(Bird::Vulture)
}

/// The bird that is dead the whole day and night of a weekday (AG p15
/// v. 2, p25 v. 2; PUL p. 57).
#[must_use]
pub fn death_bird(paksha: Paksha, vara: Vara) -> Bird {
    use Bird::{Cock as K, Crow as C, Owl as O, Peacock as P, Vulture as V};
    let row: [Bird; 7] = if bright(paksha) {
        [O, C, K, P, V, O, V]
    } else {
        [C, O, V, P, K, P, K]
    };
    row.get(usize::from(vara.id())).copied().unwrap_or(O)
}

/// How the native's bird is found from the birth star (crux P1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum BirthBird {
    /// The bright half's groups for a bright-half birth and the birds
    /// reversed over the same groups for a dark-half one (PUL p. vi), as
    /// modern practice reads it.
    #[default]
    ByPaksha,
    /// One table whatever the paksha (AY pp. 104–105; AG07 p. 60 vv. 22,
    /// 24): the only assignment the public-domain texts give.
    Single,
}

/// The native's bird from the birth nakshatra and paksha: the stars in
/// groups of 5, 6, 5, 5 and 6 from Ashvini, the vulture's to the
/// peacock's, the dark half reversing the birds under [`BirthBird::ByPaksha`].
#[must_use]
pub fn birth_bird(nakshatra: Nakshatra, paksha: Paksha, rule: BirthBird) -> Bird {
    let group = match nakshatra.id() {
        0..=4 => 0,
        5..=10 => 1,
        11..=15 => 2,
        16..=20 => 3,
        _ => 4,
    };
    match rule {
        BirthBird::ByPaksha if !bright(paksha) => Bird::at(4 - group),
        _ => Bird::at(group),
    }
}

/// How long each sub-period of a yama runs (crux P4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum SubLengths {
    /// Agastya's nazhigai, one length an activity "for the waxing and the
    /// waning" alike (AG p5 vv. 7–9, p17).
    #[default]
    Agastya,
    /// Pulippani's figures, which differ by half (PUL p. 46).
    Pulippani,
}

/// The share of a yama a sub-period of `activity` runs, in 144ths (the
/// minutes of a 144-minute yama).
#[must_use]
pub const fn sub_share(activity: Activity, paksha: Paksha, half: Half, lengths: SubLengths) -> u8 {
    use Activity::{Dying, Eating, Ruling, Sleeping, Walking};
    match (lengths, bright(paksha), half) {
        (SubLengths::Pulippani, true, Half::Night) => match activity {
            Eating | Walking => 30,
            Ruling | Sleeping => 24,
            Dying => 36,
        },
        (SubLengths::Pulippani, false, Half::Day) => match activity {
            Eating => 48,
            Dying => 30,
            Sleeping => 12,
            Ruling => 18,
            Walking => 36,
        },
        (SubLengths::Pulippani, false, Half::Night) => match activity {
            Eating | Walking => 42,
            Sleeping | Ruling => 18,
            Dying => 24,
        },
        _ => match activity {
            Eating => 30,
            Walking => 36,
            Ruling => 48,
            Sleeping => 18,
            Dying => 12,
        },
    }
}

/// One sub-period of a yama for a native: what is done, by whose main
/// activity it is owned, and its share of the yama.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Sub {
    /// The activity.
    pub activity: Activity,
    /// The bird whose main activity in the yama it is.
    pub owner: Bird,
    /// Its share of the yama, in 144ths.
    pub share: u8,
}

/// A native's sub-periods in a yama, in order: its own main activity
/// first, then the half's sequence round (PUL p. 37; crux P5), each owned
/// by the bird whose main activity it is.
#[must_use]
pub fn subs(
    bird: Bird,
    paksha: Paksha,
    half: Half,
    vara: Vara,
    yama: usize,
    lengths: SubLengths,
) -> [Sub; 5] {
    let order = sequence(paksha, half);
    let main = activity(bird, paksha, half, vara, yama);
    let first = order.iter().position(|each| *each == main).unwrap_or(0);
    core::array::from_fn(|k| {
        let done = order.get((first + k) % 5).copied().unwrap_or(main);
        Sub {
            activity: done,
            owner: doing(done, paksha, half, vara, yama),
            share: sub_share(done, paksha, half, lengths),
        }
    })
}

/// How one bird regards another (crux P6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Relation {
    /// A friend.
    Friend,
    /// An enemy.
    Enemy,
    /// Neither.
    Neutral,
}

/// Whose friends and enemies (crux P6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Relations {
    /// Agastya's directed lists (AG p51, p40 v. 13), the same in both
    /// halves; a bird neither lists is neutral.
    #[default]
    Agastya,
    /// Pulippani's: each bird the friend of its two neighbours round a
    /// cycle and the enemy of the other two, the cycle V–O–C–K–P in the
    /// bright half (p. 48) and V–C–O–K–P in the dark (p. 121).
    Pulippani,
}

/// How `of` regards `to`; a bird is its own friend.
#[must_use]
pub fn relation(of: Bird, to: Bird, paksha: Paksha, scheme: Relations) -> Relation {
    use Bird::{Cock as K, Crow as C, Owl as O, Peacock as P, Vulture as V};
    if of == to {
        return Relation::Friend;
    }
    match scheme {
        Relations::Agastya => {
            let (friends, enemies): (&[Bird], &[Bird]) = match of {
                P => (&[K, C], &[V, O]),
                C => (&[P, K, V], &[O]),
                V => (&[C], &[O, K, P]),
                K => (&[P, C], &[V, O]),
                O => (&[K, P, V], &[C]),
            };
            if friends.contains(&to) {
                Relation::Friend
            } else if enemies.contains(&to) {
                Relation::Enemy
            } else {
                Relation::Neutral
            }
        }
        Relations::Pulippani => {
            let cycle = if bright(paksha) {
                [V, O, C, K, P]
            } else {
                [V, C, O, K, P]
            };
            let place = |bird: Bird| cycle.iter().position(|each| *each == bird).unwrap_or(0);
            let apart = (place(to) + 5 - place(of)) % 5;
            if apart == 1 || apart == 4 {
                Relation::Friend
            } else {
                Relation::Enemy
            }
        }
    }
}

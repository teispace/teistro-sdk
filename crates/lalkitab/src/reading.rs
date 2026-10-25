//! What a teva says, clause by clause (1952 pp. 27–128).
//!
//! The book judges in words; this reports the conditions it names and
//! leaves the judgement to the reader (`report-clauses-not-a-verdict`):
//! a planet's dignities and the regard it has for its house's owners, not
//! a "good" or "bad"; a debt's seated enemies, not a severity.

use serde::Serialize;
use teistro_core::catalogue::Graha;

use crate::aspects::{Strength, looked_at_by, looks_at};
use crate::tables::{
    MASNUI, Masnui, PLANETS, RINAS, Regard, Rin, debilitated, exalted, joint_owners, own,
    pakka_ghar, regard, root, waker,
};
use crate::teva::Teva;

/// A dignity a planet has in its house.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// use teistro_lalkitab::Dignity;
///
/// let rahu = reading.planets.iter().find(|planet| planet.graha == Graha::Rahu).ok_or(teistro_core::error::Error::internal("nine planets"))?;
/// // Rahu in 12: its pakka ghar, and the seventh from its exaltation in 6.
/// assert_eq!(rahu.dignities, [Dignity::Pakka, Dignity::Debilitated]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "LalKitabDignity"))]
pub enum Dignity {
    /// Its permanent house (*pakka ghar*, p. 29).
    Pakka,
    /// Exalted (pp. ~44–46).
    Exalted,
    /// Debilitated (pp. ~44–46).
    Debilitated,
    /// A house it owns.
    Own,
}

/// How a planet regards one owner of the house it sits in.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// use teistro_lalkitab::{OwnerRegard, Regard};
///
/// let mercury = reading.planets.iter().find(|planet| planet.graha == Graha::Mercury).ok_or(teistro_core::error::Error::internal("nine planets"))?;
/// // House 4 is the Moon's, whom Mercury takes as an enemy.
/// assert_eq!(mercury.owners, [OwnerRegard { owner: Graha::Moon, regard: Regard::Enemy }]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OwnerRegard {
    /// The owner: a planet that owns the house or has it as its pakka ghar.
    pub owner: Graha,
    /// How the sitting planet regards it (p. 31, directed).
    pub regard: Regard,
}

/// One house a planet's aspect falls on.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// use teistro_lalkitab::aspects::Strength;
///
/// let mars = reading.planets.iter().find(|planet| planet.graha == Graha::Mars).ok_or(teistro_core::error::Error::internal("nine planets"))?;
/// let casts: Vec<_> = mars.casts.iter().map(|cast| (cast.to, cast.strength, cast.onto.clone())).collect();
/// assert_eq!(casts, [(9, Strength::Half, vec![Graha::Moon]), (11, Strength::Half, vec![])]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Cast {
    /// The house.
    pub to: u8,
    /// How hard.
    pub strength: Strength,
    /// The planets there, which the aspect reaches.
    pub onto: Vec<Graha>,
}

/// One planet of the teva.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// let sun = reading.planets.iter().find(|planet| planet.graha == Graha::Sun).ok_or(teistro_core::error::Error::internal("nine planets"))?;
/// // The Sun in 4 looks only at 10, which is empty: it sleeps.
/// assert!(!sun.awake && sun.dignities.is_empty());
/// let mars = reading.planets.iter().find(|planet| planet.graha == Graha::Mars).ok_or(teistro_core::error::Error::internal("nine planets"))?;
/// // Mars alone in its pakka ghar, and no occupied house looks at 3.
/// assert!(mars.awake && mars.kayam);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PlanetReading {
    /// The planet.
    pub graha: Graha,
    /// Its house.
    pub house: u8,
    /// Its dignities there, in the order this lists them.
    pub dignities: Vec<Dignity>,
    /// Its regard for each owner of its house, itself excluded.
    pub owners: Vec<OwnerRegard>,
    /// Whether it is awake (pp. 96–99).
    pub awake: bool,
    /// Whether it is *kayam*, established: in a dignity, alone in its
    /// house, and looked at by no occupied house (p. 34).
    pub kayam: bool,
    /// The houses its aspect falls on (pp. 102–104).
    pub casts: Vec<Cast>,
}

/// An occupied house looking at another.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// use teistro_lalkitab::Look;
/// use teistro_lalkitab::aspects::Strength;
///
/// // House 10 is empty and awake: 4, occupied, looks at it in full.
/// let tenth = &reading.houses[9];
/// assert_eq!(tenth.looked_at_by, [Look { from: 4, strength: Strength::Full }]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Look {
    /// The house looking.
    pub from: u8,
    /// How hard.
    pub strength: Strength,
}

/// An enemy seated in one of a planet's houses.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// use teistro_lalkitab::{Rin, Seated};
///
/// assert_eq!(reading.rinas[0].rin, Rin::Pitri);
/// assert_eq!(reading.rinas[0].seated, [Seated { enemy: Graha::Rahu, house: 12 }]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Seated {
    /// The enemy.
    pub enemy: Graha,
    /// The house it sits in.
    pub house: u8,
}

/// One house of the teva.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// let first = &reading.houses[0];
/// // Empty and looked at by no house: house 1 sleeps, and Mars wakes it.
/// assert!(first.occupants.is_empty() && !first.awake);
/// assert_eq!(first.waker, Graha::Mars);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "LalKitabHouse"))]
pub struct HouseReading {
    /// The house, 1 to 12.
    pub house: u8,
    /// Its planets.
    pub occupants: Vec<Graha>,
    /// The occupied houses that look at it, with how hard.
    pub looked_at_by: Vec<Look>,
    /// Whether it is awake: occupied, or looked at by an occupied house
    /// (pp. 96–97; crux LK-C11).
    pub awake: bool,
    /// The planet that wakes it when it sleeps (p. 98).
    pub waker: Graha,
}

/// A pair of planets in one house counting as a third (p. 27).
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// use teistro_lalkitab::{Formed, Masnui};
///
/// assert_eq!(reading.masnui, [
///     Formed { pair: [Graha::Sun, Graha::Mercury], house: 4, counts_as: Masnui::MarsBenefic },
///     Formed { pair: [Graha::Venus, Graha::Saturn], house: 7, counts_as: Masnui::KetuExalted },
/// ]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Formed {
    /// The pair.
    pub pair: [Graha; 2],
    /// Their house.
    pub house: u8,
    /// What they count as.
    pub counts_as: Masnui,
}

/// A debt the teva carries (p. 125): an enemy of a planet seated in one of
/// its houses.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// // Rahu in 12, a house of Jupiter's: the one debt this teva carries.
/// assert_eq!(reading.rinas.len(), 1);
/// assert_eq!(reading.rinas[0].of, Graha::Jupiter);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Debt {
    /// The debt.
    pub rin: Rin,
    /// The planet whose houses hold the enemy.
    pub of: Graha,
    /// Each enemy seated there, with the house.
    pub seated: Vec<Seated>,
}

/// The ancestors' debt in its first state (p. 128): a planet in house 9
/// while Mercury sits in that planet's root house.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// use teistro_lalkitab::PitriState;
///
/// assert_eq!(reading.pitri, [PitriState { ninth: Graha::Moon, mercury: 4 }]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PitriState {
    /// The planet in house 9.
    pub ninth: Graha,
    /// Mercury's house, the planet's root.
    pub mercury: u8,
}

/// The teva's own conditions.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// // Houses 4 and 7 hold two planets each: not a minor's chart.
/// assert!(reading.flags.ratandha && !reading.flags.nabalig);
/// assert!(reading.flags.dharmi.is_empty());
/// assert_eq!(reading.flags.sathi, [[Graha::Jupiter, Graha::Moon], [Graha::Rahu, Graha::Ketu]]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Flags {
    /// Night-blind (*ratandha*): the Sun in 4 and Saturn in 7 (p. ~43).
    pub ratandha: bool,
    /// A minor's chart (*nabalig*): houses 1, 4, 7 and 10 empty, or holding
    /// only one of Saturn, Rahu, Ketu or Mercury between them (p. ~48).
    pub nabalig: bool,
    /// The *dharmi* papis: Rahu or Ketu in 4 or with the Moon, Saturn in 11
    /// or with Jupiter (p. ~43).
    pub dharmi: Vec<Graha>,
    /// Companions (*sathi*): two planets each in a house of the other's,
    /// own, exalted or pakka (p. ~43).
    pub sathi: Vec<[Graha; 2]>,
}

/// Everything a teva says.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// assert_eq!((reading.planets.len(), reading.houses.len()), (9, 12));
/// assert_eq!(reading.planets[0].graha, Graha::Jupiter);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "LalKitabReading"))]
pub struct Reading {
    /// The nine planets, in Lal Kitab's order.
    pub planets: Vec<PlanetReading>,
    /// The twelve houses.
    pub houses: Vec<HouseReading>,
    /// The artificial planets formed.
    pub masnui: Vec<Formed>,
    /// The debts it carries.
    pub rinas: Vec<Debt>,
    /// The ancestors' debt's first state, where it holds.
    pub pitri: Vec<PitriState>,
    /// The chart's own conditions.
    pub flags: Flags,
}

/// The houses a planet holds a claim on: own, exalted and pakka.
fn claims(graha: Graha) -> Vec<u8> {
    let mut houses = own(graha);
    houses.extend(exalted(graha));
    houses.extend(pakka_ghar(graha));
    houses
}

/// The owners of a house: the planets that own it or have it as pakka,
/// and its joint owners.
fn owners_of(house: u8) -> Vec<Graha> {
    PLANETS
        .iter()
        .copied()
        .filter(|graha| own(*graha).contains(&house) || pakka_ghar(*graha).contains(&house))
        .chain(joint_owners(house).into_iter().flatten())
        .fold(Vec::new(), |mut owners, graha| {
            if !owners.contains(&graha) {
                owners.push(graha);
            }
            owners
        })
}

/// The occupied houses that look at `house`.
fn looked_at_from_occupied(teva: &Teva, house: u8) -> Vec<Look> {
    looked_at_by(house)
        .filter(|(from, _)| teva.occupied(*from))
        .map(|(from, strength)| Look { from, strength })
        .collect()
}

/// Whether a planet in `house` is awake (pp. 96–99): in its pakka ghar it
/// always is; the specific pairs of p. 98 put it to sleep; a planet of the
/// first side sleeps when every house it looks at is empty, and one of the
/// later side when the first side is empty, or, from 8, when 2 is.
fn awake(teva: &Teva, graha: Graha, house: u8) -> bool {
    if pakka_ghar(graha).contains(&house) {
        return true;
    }
    let empty = |h: u8| !teva.occupied(h);
    let paired = match house {
        2 => empty(10),
        9 | 10 => empty(2),
        _ => false,
    };
    if paired {
        return false;
    }
    if house <= 6 {
        return looks_at(house).iter().any(|(to, _)| !empty(*to));
    }
    let first_side = (1..=6).any(|h| !empty(h));
    first_side && (house != 8 || !empty(2))
}

/// One planet's reading.
fn planet(teva: &Teva, graha: Graha, house: u8) -> PlanetReading {
    let mut dignities = Vec::new();
    for (dignity, houses) in [
        (Dignity::Pakka, pakka_ghar(graha).to_vec()),
        (Dignity::Exalted, exalted(graha)),
        (Dignity::Debilitated, debilitated(graha)),
        (Dignity::Own, own(graha)),
    ] {
        if houses.contains(&house) {
            dignities.push(dignity);
        }
    }
    let owners = owners_of(house)
        .into_iter()
        .filter(|owner| *owner != graha)
        .filter_map(|owner| regard(graha, owner).map(|regard| OwnerRegard { owner, regard }))
        .collect();
    let alone = teva.occupants(house).len() == 1;
    let kayam = !dignities.is_empty() && alone && looked_at_from_occupied(teva, house).is_empty();
    let casts = looks_at(house)
        .iter()
        .map(|(to, strength)| Cast {
            to: *to,
            strength: *strength,
            onto: teva.occupants(*to),
        })
        .collect();
    PlanetReading {
        graha,
        house,
        dignities,
        owners,
        awake: awake(teva, graha, house),
        kayam,
        casts,
    }
}

/// The debts the teva carries, and the ancestors' debt's first state.
fn debts(teva: &Teva) -> (Vec<Debt>, Vec<PitriState>) {
    let rinas = RINAS
        .iter()
        .filter_map(|(rin, of, houses, enemies)| {
            let seated: Vec<Seated> = enemies
                .iter()
                .filter_map(|enemy| {
                    let house = teva.house_of(*enemy)?;
                    houses.contains(&house).then_some(Seated {
                        enemy: *enemy,
                        house,
                    })
                })
                .collect();
            (!seated.is_empty()).then_some(Debt {
                rin: *rin,
                of: *of,
                seated,
            })
        })
        .collect();
    let mercury = teva.house_of(Graha::Mercury).unwrap_or(0);
    let pitri = teva
        .occupants(9)
        .into_iter()
        .filter(|ninth| *ninth != Graha::Mercury && root(*ninth).contains(&mercury))
        .map(|ninth| PitriState { ninth, mercury })
        .collect();
    (rinas, pitri)
}

/// The teva's own conditions.
fn flags(teva: &Teva) -> Flags {
    let at = |graha: Graha| teva.house_of(graha).unwrap_or(0);
    let kendras: Vec<Graha> = [1, 4, 7, 10]
        .into_iter()
        .flat_map(|house| teva.occupants(house))
        .collect();
    let lone = matches!(
        kendras.as_slice(),
        [] | [Graha::Saturn | Graha::Rahu | Graha::Ketu | Graha::Mercury]
    );
    let with = |a: Graha, b: Graha| at(a) == at(b);
    let mut dharmi = Vec::new();
    for node in [Graha::Rahu, Graha::Ketu] {
        if at(node) == 4 || with(node, Graha::Moon) {
            dharmi.push(node);
        }
    }
    if at(Graha::Saturn) == 11 || with(Graha::Saturn, Graha::Jupiter) {
        dharmi.push(Graha::Saturn);
    }
    let mut sathi = Vec::new();
    for (index, a) in PLANETS.iter().enumerate() {
        for b in PLANETS.iter().skip(index + 1) {
            let (in_a, in_b) = (at(*a), at(*b));
            if in_a != in_b && claims(*b).contains(&in_a) && claims(*a).contains(&in_b) {
                sathi.push([*a, *b]);
            }
        }
    }
    Flags {
        ratandha: at(Graha::Sun) == 4 && at(Graha::Saturn) == 7,
        nabalig: lone,
        dharmi,
        sathi,
    }
}

/// Everything a teva says.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_lalkitab::{Teva, read};
///
/// let teva = Teva::from_houses([
///     (Graha::Jupiter, 2), (Graha::Sun, 4), (Graha::Moon, 9),
///     (Graha::Venus, 7), (Graha::Mars, 3), (Graha::Mercury, 4),
///     (Graha::Saturn, 7), (Graha::Rahu, 12), (Graha::Ketu, 6),
/// ])?;
/// let reading = read(&teva);
/// let jupiter = reading.planets.iter().find(|planet| planet.graha == Graha::Jupiter).ok_or(teistro_core::error::Error::internal("nine planets"))?;
/// // Jupiter alone in 2, its pakka ghar, which only an empty 8 looks at.
/// assert!(jupiter.kayam);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn read(teva: &Teva) -> Reading {
    let planets = teva
        .placements()
        .map(|(graha, house)| planet(teva, graha, house))
        .collect();
    let houses = (1..=12_u8)
        .map(|house| {
            let looked = looked_at_from_occupied(teva, house);
            HouseReading {
                house,
                occupants: teva.occupants(house),
                awake: teva.occupied(house) || !looked.is_empty(),
                looked_at_by: looked,
                waker: waker(house),
            }
        })
        .collect();
    let masnui = MASNUI
        .iter()
        .filter_map(|(a, b, counts_as)| {
            let house = teva.house_of(*a)?;
            (teva.house_of(*b) == Some(house)).then_some(Formed {
                pair: [*a, *b],
                house,
                counts_as: *counts_as,
            })
        })
        .collect();
    let (rinas, pitri) = debts(teva);
    Reading {
        planets,
        houses,
        masnui,
        rinas,
        pitri,
        flags: flags(teva),
    }
}

//! The marriage doshas of a match as one list: what the Ashta Koota, the
//! ten considerations and the Kuja dosha already report, gathered without
//! a new judgment or a severity (`03-design/matching.md`, C289, C290).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Koota;

use crate::{AshtaKoota, KootaReading, Kuja, MaitriRelation, MatchRole, Porutham};

/// Which reading a marriage dosha comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DoshaSystem {
    /// The Ashta Koota of *Muhurta Chintamani* VI.21–36.
    AshtaKoota,
    /// The ten considerations of *Kalaprakasika* XIII.
    Porutham,
    /// The Kuja dosha of *Manasagari*'s jāyābhāva v. 4.
    Kuja,
}

/// One marriage dosha a match carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MarriageDosha {
    /// The reading it comes from.
    pub system: DoshaSystem,
    /// The koota or consideration it is, or `None` for the Kuja dosha,
    /// which is no koota.
    pub koota: Option<Koota>,
    /// The side carrying it, for the Kuja dosha; `None` for the rest,
    /// which are read between the two.
    pub side: Option<MatchRole>,
    /// Whether an exception the source names lifts it.
    pub lifted: bool,
}

impl MarriageDosha {
    const fn between(system: DoshaSystem, koota: Koota, lifted: bool) -> MarriageDosha {
        MarriageDosha {
            system,
            koota: Some(koota),
            side: None,
            lifted,
        }
    }
}

/// Every marriage dosha a match carries, in the answers' own order: the
/// Ashta Koota's Bhakoot, Nadi, Gana and the lords' enmity in the verse's
/// order; each consideration of the ten that disagrees or agrees only by
/// the p. 76 exception, in the chapter's; and each side's Kuja dosha, the
/// bride's first.
///
/// ```
/// use teistro_matching::{
///     KootaRules, KujaNative, KujaRules, Native, PoruthamRules, ashta_koota, kuja,
///     marriage_doshas, porutham,
/// };
///
/// // One Moon and one Mars on both sides: one nadi, unlifted in one pada.
/// let moon = Native::of_moon(45.0)?;
/// let mars = KujaNative::of_longitudes(5.0, 45.0, 60.0, 190.0)?;
/// let doshas = marriage_doshas(
///     &ashta_koota(moon, moon, KootaRules::default()),
///     &porutham(moon, moon, PoruthamRules::default()),
///     &kuja(mars, mars, KujaRules::default()),
/// );
/// assert!(doshas.iter().any(|d| d.koota == Some(teistro_core::catalogue::Koota::Nadi) && !d.lifted));
/// assert_eq!(doshas.iter().filter(|d| d.side.is_some()).count(), 2);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn marriage_doshas(ashta: &AshtaKoota, ten: &Porutham, mars: &Kuja) -> Vec<MarriageDosha> {
    let eight = ashta.kootas.iter().filter_map(|row| {
        let (koota, lifted) = match row.reading {
            KootaReading::Bhakoot {
                dosha: Some(_),
                lifted,
                ..
            } => (Koota::Bhakoot, lifted),
            KootaReading::Nadi {
                dosha: true,
                lifted,
                ..
            } => (Koota::Nadi, lifted),
            KootaReading::Gana {
                dosha: true,
                lifted,
                ..
            } => (Koota::Gana, lifted),
            KootaReading::GrahaMaitri {
                relation:
                    MaitriRelation::FriendEnemy
                    | MaitriRelation::NeutralEnemy
                    | MaitriRelation::MutualEnemies,
                lifted,
                ..
            } => (Koota::GrahaMaitri, lifted),
            _ => return None,
        };
        Some(MarriageDosha::between(
            DoshaSystem::AshtaKoota,
            koota,
            lifted,
        ))
    });
    let considerations = ten
        .considerations
        .iter()
        .filter(|row| !row.agrees || row.lifted)
        .map(|row| MarriageDosha::between(DoshaSystem::Porutham, row.reading.koota(), row.lifted));
    let sides = [
        (MatchRole::Bride, mars.bride),
        (MatchRole::Groom, mars.groom),
    ]
    .into_iter()
    .filter(|(_, side)| side.dosha)
    .map(|(role, _)| MarriageDosha {
        system: DoshaSystem::Kuja,
        koota: None,
        side: Some(role),
        lifted: false,
    });
    eight.chain(considerations).chain(sides).collect()
}

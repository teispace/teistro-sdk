//! Naam milan: the match of two names, star to star, with the varga
//! koota of their first letters (`03-design/matching.md`, C295, C296).

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

use crate::{
    AshtaKoota, KootaRules, MatchRole, NameRules, NameSyllable, NameVarga, Porutham, PoruthamRules,
    ashta_koota, name_syllable, porutham,
};

/// How two vargas stand (VI.35).
///
/// ```
/// use teistro_matching::{NameVarga, VargaRelation, varga_koota};
///
/// // The cat eats the rat, the 5th from it.
/// assert_eq!(varga_koota(NameVarga::Cat, NameVarga::Rat).relation, VargaRelation::Enemy);
/// assert_eq!(varga_koota(NameVarga::Cat, NameVarga::Lion).relation, VargaRelation::Neutral);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VargaRelation {
    /// One varga: good.
    Same,
    /// Each the 5th from the other, eater and eaten: bad.
    Enemy,
    /// Neither: good.
    Neutral,
}

/// The varga koota of two names.
///
/// ```
/// use teistro_matching::{NameVarga, VargaRelation, varga_koota};
///
/// let koota = varga_koota(NameVarga::Deer, NameVarga::Deer);
/// assert_eq!((koota.bride, koota.relation), (NameVarga::Deer, VargaRelation::Same));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct VargaKoota {
    /// The bride's varga.
    pub bride: NameVarga,
    /// The groom's.
    pub groom: NameVarga,
    /// How they stand.
    pub relation: VargaRelation,
}

/// The varga koota of two vargas.
///
/// ```
/// use teistro_matching::{NameVarga, VargaRelation, varga_koota};
///
/// // Garuda and the serpent, either way round.
/// assert_eq!(varga_koota(NameVarga::Garuda, NameVarga::Serpent).relation, VargaRelation::Enemy);
/// assert_eq!(varga_koota(NameVarga::Serpent, NameVarga::Garuda).relation, VargaRelation::Enemy);
/// ```
#[must_use]
pub fn varga_koota(bride: NameVarga, groom: NameVarga) -> VargaKoota {
    let apart = (bride.place() + 8 - groom.place()) % 8;
    let relation = match apart {
        0 => VargaRelation::Same,
        4 => VargaRelation::Enemy,
        _ => VargaRelation::Neutral,
    };
    VargaKoota {
        bride,
        groom,
        relation,
    }
}

/// The readings two names are matched under.
///
/// ```
/// use teistro_matching::{EqualVarna, KootaRules, NaamRules, NameRules, PoruthamRules};
///
/// let rules = NaamRules { koota: KootaRules { equal_varna: EqualVarna::Half, ..KootaRules::default() }, ..NaamRules::default() };
/// assert_eq!((rules.name, rules.porutham), (NameRules::default(), PoruthamRules::default()));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NaamRules {
    /// How each name is read.
    pub name: NameRules,
    /// The Ashta Koota's.
    pub koota: KootaRules,
    /// The ten considerations'.
    pub porutham: PoruthamRules,
}

/// The match of two names (C296).
///
/// ```
/// use teistro_matching::{NaamRules, VargaRelation, naam_milan};
///
/// let milan = naam_milan("राधा", "रमेश", NaamRules::default())?;
/// // Both names begin with र: one varga, one star.
/// assert_eq!(milan.varga.relation, VargaRelation::Same);
/// assert_eq!(milan.bride.nakshatra, milan.groom.nakshatra);
/// assert!(milan.ashta.total <= 36.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NaamMilan {
    /// The bride's name's first syllable.
    pub bride: NameSyllable,
    /// The groom's.
    pub groom: NameSyllable,
    /// Their vargas (VI.35).
    pub varga: VargaKoota,
    /// The Ashta Koota of the two name stars.
    pub ashta: AshtaKoota,
    /// The ten considerations of the two name stars.
    pub porutham: Porutham,
}

/// Two names matched name star to name star, as Vasishtha asks when a
/// birth is unknown (C296).
///
/// ```
/// use teistro_matching::{AbhijitPada, LatinName, NaamRules, NameRules, naam_milan};
///
/// // A bride's name in Abhijit's row is refused unless a pada is named.
/// let refused = naam_milan("जूही", "राम", NaamRules::default()).unwrap_err();
/// assert_eq!(refused.field(), Some("bride.abhijit"));
/// let rules = NaamRules { name: NameRules { abhijit: AbhijitPada::UttaraAshadha, ..NameRules::default() }, ..NaamRules::default() };
/// assert!(naam_milan("जूही", "राम", rules).is_ok());
///
/// // A match needs one star: a CASUAL spelling read two ways is refused.
/// let casual = NaamRules { name: NameRules { latin: LatinName::Casual, ..NameRules::default() }, ..NaamRules::default() };
/// assert_eq!(naam_milan("Shiva", "Krishna", casual).unwrap_err().field(), Some("bride.name"));
/// assert!(naam_milan("Sita", "Krishna", casual).is_ok());
/// ```
///
/// # Errors
///
/// Either name refused by [`name_syllable`] or [`NameSyllable::native`],
/// named under `bride` or `groom`: among them a `CASUAL` spelling that
/// reaches more than one syllable, whose hint lists the spellings (C772).
pub fn naam_milan(bride: &str, groom: &str, rules: NaamRules) -> Result<NaamMilan, Error> {
    let read = |name: &str, role: MatchRole| {
        name_syllable(name, rules.name)
            .and_then(|syllable| Ok((syllable, syllable.native(rules.name.abhijit)?)))
            .map_err(|error| error.under(role.name()))
    };
    let (bride, bride_star) = read(bride, MatchRole::Bride)?;
    let (groom, groom_star) = read(groom, MatchRole::Groom)?;
    Ok(NaamMilan {
        bride,
        groom,
        varga: varga_koota(bride.varga, groom.varga),
        ashta: ashta_koota(bride_star, groom_star, rules.koota),
        porutham: porutham(bride_star, groom_star, rules.porutham),
    })
}

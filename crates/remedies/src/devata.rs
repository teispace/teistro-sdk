//! The ishṭa-devatā (`03-design/remedies.md`, step 4, C354 to C356).
//!
//! BPHS (the 1923 print) ch. 9 vv. 70–76, ch. 33 vv. 68–74 in the later
//! recension, reads the deity a person is devoted to from the grahas in
//! the 12th sign from the kārakāṁśa. The answer reports each graha there
//! with the deity its verse names and whether Ketu shares the sign, and
//! decides nothing the verses leave open:
//!
//! - **"With Ketu."** vv. 70–72 join the Sun, the Moon and Venus to Ketu,
//!   and the recension's v. 71 joins Mars; the instrumentals after it
//!   (Mercury and Saturn, Jupiter, Rahu) may continue the conjunction or
//!   not, and Rahu never stands with Ketu, so the series cannot be read
//!   as all "with Ketu" (C355). Each devotion says whether Ketu is there.
//! - **The chart.** The verses count "from the kārakāṁśa" and name no
//!   chart (C130); the caller passes the signs of the chart it reads, and
//!   the façade answers both.
//!
//! - **The amātya.** vv. 76–79 read the same from the amātyakāraka, the
//!   graha next below the ātmakāraka in degrees (v. 76): the 12th from it
//!   gives the same devotions (v. 77), a malefic there in a malefic's sign
//!   minor deities (v. 78), and the grahas joined to it in whichever house
//!   it stands the fruits as before (v. 79). [`amatya_devata`] counts from
//!   the sign the caller passes, which the façade takes as the amātya's
//!   navāṁśa, the chapter's analogue of the kārakāṁśa (C357).
//!
//! The baseline engine looks a pair of deities up by the Moon's sign
//! instead, in records written for the kārakāṁśa sign; no text gives
//! such a table, so [`baseline_ishta_devata`] is reached only when asked.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::house::House;

use crate::subjects::NINE;

/// A deity a verse, or the baseline's table, names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Deity {
    /// Śiva.
    Shiva,
    /// Gaurī, worshipped by a Śākta (v. 70 as the recension pairs it).
    Gauri,
    /// Pārvatī, as the baseline names her beside Gaurī.
    Parvati,
    /// Lakṣmī, *samudra-tanayā*, the ocean's daughter (v. 72).
    Lakshmi,
    /// Skanda, Kārttikeya (vv. 73 and 74).
    Skanda,
    /// Viṣṇu (v. 73).
    Vishnu,
    /// Durgā in her tāmasī form, with the service of spirits (v. 74).
    Durga,
    /// Heramba, Gaṇeśa (v. 74).
    Heramba,
    /// Sūrya, the 1899 and 1923 prints' *ravi-bhakti* (C354).
    Surya,
    /// The *kṣudra-devatā*: minor deities (vv. 75 and 76).
    Minor,
    /// Candra, in the baseline's table.
    Chandra,
    /// Sarasvatī, in the baseline's table.
    Saraswati,
    /// Narasiṁha, in the baseline's table.
    Narasimha,
    /// Hanumān, in the baseline's table.
    Hanuman,
    /// Dattātreya, in the baseline's table.
    Dattatreya,
    /// Śani, in the baseline's table.
    Shani,
    /// Kṛṣṇa, in the baseline's table.
    Krishna,
    /// Bṛhaspati, in the baseline's table.
    Brihaspati,
}

/// Whose devotee the Sun with Ketu makes (C354). The 1899 and 1923
/// prints give vv. 70–71 with their fruits exchanged and read
/// *ravi-bhakti*; the recension and Jaimini's *Upadeśa Sūtras* read
/// Śiva for the Sun and Gaurī for the Moon.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SunWithKetu {
    /// Śiva, the recension's *śiva-bhakti*; the default.
    #[default]
    Shiva,
    /// Sūrya, the prints' *ravi-bhakti*.
    Surya,
}

/// The readings the ishṭa-devatā is given under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct DevataRules {
    /// Whose devotee the Sun with Ketu makes.
    pub sun_with_ketu: SunWithKetu,
}

/// One graha in the 12th from the kārakāṁśa, and the devotion its verse
/// names.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Devotion {
    /// The graha.
    pub graha: Graha,
    /// The deities its verse names, either of them where it says *vā*.
    pub deities: Vec<Deity>,
    /// The verse, in the 1923 print's ch. 9.
    pub verse: u8,
    /// Whether Ketu stands in the same sign. The verse joins the graha to
    /// Ketu for the Sun, the Moon, Venus and Mars, and may not for the
    /// rest (C355); always false of Ketu itself and of Rahu.
    pub with_ketu: bool,
}

/// The 12th from the kārakāṁśa in one chart, read as BPHS vv. 70–76
/// read it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct IshtaDevata {
    /// The readings it was given under.
    pub rules: DevataRules,
    /// The 12th sign from the kārakāṁśa.
    pub sign: Rashi,
    /// Each graha in it with its devotion, in catalogue order; empty when
    /// the sign is empty, which the verses give no deity for.
    pub devotions: Vec<Devotion>,
    /// The grahas there that make a devotee of minor deities in a sign a
    /// natural malefic rules (Leo, Aries, Scorpio, Capricorn or Aquarius),
    /// each named beside its own devotion: Saturn or Venus from the
    /// kārakāṁśa (vv. 75 and 76, in their order), and from the amātya
    /// every natural malefic as well (v. 78, in catalogue order).
    pub minor: Vec<Graha>,
}

/// The signs a natural malefic rules: the Sun's, Mars's and Saturn's.
const MALEFIC_LORDS: [Graha; 3] = [Graha::Sun, Graha::Mars, Graha::Saturn];

/// The grahas vv. 75 and 76 name in the 12th from the kārakāṁśa.
const MINOR_FROM_KARAKA: [Graha; 2] = [Graha::Saturn, Graha::Venus];

/// The grahas the 12th from the amātya names: vv. 75 and 76's, carried
/// over by v. 77, and v. 78's natural malefics.
const MINOR_FROM_AMATYA: [Graha; 6] = [
    Graha::Sun,
    Graha::Mars,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// The deities and verse a graha in the 12th is given.
fn devotion_of(graha: Graha, rules: DevataRules) -> (Vec<Deity>, u8) {
    match graha {
        Graha::Sun => (
            vec![match rules.sun_with_ketu {
                SunWithKetu::Shiva => Deity::Shiva,
                SunWithKetu::Surya => Deity::Surya,
            }],
            70,
        ),
        Graha::Moon => (vec![Deity::Gauri], 71),
        Graha::Venus => (vec![Deity::Lakshmi], 72),
        Graha::Mars => (vec![Deity::Skanda], 73),
        Graha::Mercury | Graha::Saturn => (vec![Deity::Vishnu], 73),
        Graha::Jupiter => (vec![Deity::Shiva], 73),
        Graha::Rahu => (vec![Deity::Durga], 74),
        Graha::Ketu => (vec![Deity::Heramba, Deity::Skanda], 74),
        // The outer planets: no verse names them, and no chart passes them.
        _ => (Vec::new(), 74),
    }
}

/// The ishṭa-devatā from the 12th sign from `karakamsha`, the grahas
/// placed by `signs` in the order of [`NINE`]: the rasi chart's signs or
/// the navāṁśa's, whichever chart the reader counts in (C130).
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_remedies::{Deity, DevataRules, ishta_devata};
///
/// // The kārakāṁśa in Leo: the 12th is Cancer. Ketu stands there with
/// // the Moon, Rahu opposite in Capricorn, and the rest in Aries.
/// let mut signs = [Rashi::Aries; 9];
/// signs[1] = Rashi::Cancer;
/// signs[7] = Rashi::Capricorn;
/// signs[8] = Rashi::Cancer;
/// let read = ishta_devata(Rashi::Leo, &signs, DevataRules::default());
/// assert_eq!(read.sign, Rashi::Cancer);
/// assert_eq!(read.devotions[0].deities, [Deity::Gauri]);
/// assert!(read.devotions[0].with_ketu);
/// ```
#[must_use]
pub fn ishta_devata(karakamsha: Rashi, signs: &[Rashi; 9], rules: DevataRules) -> IshtaDevata {
    twelfth_from(karakamsha, signs, rules, &MINOR_FROM_KARAKA)
}

/// The 12th sign from `from` and its devotions, the grahas `minor` names
/// there counted as minor deities' devotees in a malefic's sign.
fn twelfth_from(
    from: Rashi,
    signs: &[Rashi; 9],
    rules: DevataRules,
    minor: &[Graha],
) -> IshtaDevata {
    let sign = House::VYAYA.sign_from(from);
    let malefic_sign = MALEFIC_LORDS.contains(&sign.attributes().lord);
    IshtaDevata {
        rules,
        sign,
        devotions: devotions_in(sign, signs, rules, None),
        minor: minor
            .iter()
            .copied()
            .filter(|&graha| malefic_sign && placed(graha, sign, signs))
            .collect(),
    }
}

/// Whether `graha` stands in `sign`, the grahas placed by `signs` in the
/// order of [`NINE`].
fn placed(graha: Graha, sign: Rashi, signs: &[Rashi; 9]) -> bool {
    NINE.iter()
        .zip(signs)
        .any(|(&one, &at)| one == graha && at == sign)
}

/// Each graha in `sign` but `except`, in catalogue order, with the
/// devotion its verse names and whether Ketu shares the sign.
fn devotions_in(
    sign: Rashi,
    signs: &[Rashi; 9],
    rules: DevataRules,
    except: Option<Graha>,
) -> Vec<Devotion> {
    let ketu_there = placed(Graha::Ketu, sign, signs);
    NINE.into_iter()
        .filter(|&graha| Some(graha) != except && placed(graha, sign, signs))
        .map(|graha| {
            let (deities, verse) = devotion_of(graha, rules);
            Devotion {
                graha,
                deities,
                verse,
                with_ketu: ketu_there && !matches!(graha, Graha::Rahu | Graha::Ketu),
            }
        })
        .collect()
}

/// The amātya's devotions in one chart, read as BPHS vv. 77–79 read them.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AmatyaDevata {
    /// The 12th from the sign counted from, read as the kārakāṁśa's
    /// (v. 77), with every natural malefic there in a malefic's sign a
    /// devotee of minor deities (v. 78).
    pub twelfth: IshtaDevata,
    /// The amātya's own sign in this chart.
    pub sign: Rashi,
    /// The house it stands in, by sign from this chart's lagna (v. 79's
    /// *tanvādau*), 1 to 12.
    pub house: u8,
    /// Each graha joined to it in that sign, with the devotion its verse
    /// names: v. 79's "the fruit as before".
    pub joined: Vec<Devotion>,
}

/// The amātya's devotions from `from`, the sign the 12th is counted from
/// (the façade passes the amātya's navāṁśa, C357), with `amatya` placed
/// by `signs` in the order of [`NINE`] and its house counted from `lagna`.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_remedies::{DevataRules, amatya_devata};
///
/// // The amātya Mars in Aries with the Sun, its navāṁśa in Leo, a
/// // Cancer lagna; Rahu stands in Cancer, the 12th from Leo, a sign the
/// // Moon rules, so no minor deity.
/// let mut signs = [Rashi::Libra; 9];
/// signs[0] = Rashi::Aries;
/// signs[2] = Rashi::Aries;
/// signs[7] = Rashi::Cancer;
/// signs[8] = Rashi::Capricorn;
/// let read = amatya_devata(Rashi::Leo, Graha::Mars, Rashi::Cancer, &signs, DevataRules::default());
/// assert_eq!(read.house, 10);
/// assert_eq!(read.joined[0].graha, Graha::Sun);
/// assert_eq!(read.twelfth.devotions[0].graha, Graha::Rahu);
/// assert!(read.twelfth.minor.is_empty());
/// ```
#[must_use]
pub fn amatya_devata(
    from: Rashi,
    amatya: Graha,
    lagna: Rashi,
    signs: &[Rashi; 9],
    rules: DevataRules,
) -> AmatyaDevata {
    let sign = NINE
        .iter()
        .zip(signs)
        .find_map(|(&one, &at)| (one == amatya).then_some(at))
        .unwrap_or(from);
    AmatyaDevata {
        twelfth: twelfth_from(from, signs, rules, &MINOR_FROM_AMATYA),
        sign,
        house: House::between(lagna, sign).get(),
        joined: devotions_in(sign, signs, rules, Some(amatya)),
    }
}

/// The baseline engine's ishṭa-devatā: a pair of deities by the Moon's
/// sign, from twelve records written for the kārakāṁśa sign and pending
/// review in its own source. `BASELINE` and unsourced (C356).
#[must_use]
pub const fn baseline_ishta_devata(moon: Rashi) -> [Deity; 2] {
    match moon {
        Rashi::Aries => [Deity::Surya, Deity::Skanda],
        Rashi::Taurus => [Deity::Gauri, Deity::Lakshmi],
        Rashi::Gemini | Rashi::Virgo => [Deity::Vishnu, Deity::Saraswati],
        Rashi::Cancer => [Deity::Parvati, Deity::Chandra],
        Rashi::Leo => [Deity::Shiva, Deity::Surya],
        Rashi::Libra => [Deity::Lakshmi, Deity::Saraswati],
        Rashi::Scorpio => [Deity::Narasimha, Deity::Hanuman],
        Rashi::Sagittarius => [Deity::Vishnu, Deity::Dattatreya],
        Rashi::Capricorn => [Deity::Shani, Deity::Krishna],
        Rashi::Aquarius => [Deity::Shani, Deity::Shiva],
        // Pisces; `Rashi` is non-exhaustive.
        _ => [Deity::Vishnu, Deity::Brihaspati],
    }
}

/// The ishṭa-devatā in both charts the kārakāṁśa's houses are counted
/// in, since the verses name neither (C130).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct IshtaDevatas {
    /// The Ātmakāraka the kārakāṁśa is its navāṁśa sign.
    pub atmakaraka: Graha,
    /// The kārakāṁśa.
    pub karakamsha: Rashi,
    /// The 12th from it, the grahas placed by the rasi chart.
    pub in_rasi: IshtaDevata,
    /// The 12th from it, the grahas placed by the navāṁśa.
    pub in_navamsha: IshtaDevata,
    /// The same read from the amātyakāraka (vv. 76–79).
    pub amatya: AmatyaDevatas,
}

/// The amātya's devotions in both charts, as [`IshtaDevatas`] reads the
/// ātmakāraka's.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AmatyaDevatas {
    /// The amātyakāraka, under the chart's chara kāraka scheme.
    pub graha: Graha,
    /// Its navāṁśa sign, which the 12th is counted from.
    pub amsha: Rashi,
    /// Read with the grahas and the lagna placed by the rasi chart.
    pub in_rasi: AmatyaDevata,
    /// Read with the grahas and the lagna placed by the navāṁśa.
    pub in_navamsha: AmatyaDevata,
}

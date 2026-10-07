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
    /// Saturn or Venus there in a sign a natural malefic rules (Leo,
    /// Aries, Scorpio, Capricorn or Aquarius): devotion to minor deities
    /// (vv. 75 and 76), named beside its own devotion.
    pub minor: Vec<Graha>,
}

/// The signs a natural malefic rules: the Sun's, Mars's and Saturn's.
const MALEFIC_LORDS: [Graha; 3] = [Graha::Sun, Graha::Mars, Graha::Saturn];

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
    let sign = House::VYAYA.sign_from(karakamsha);
    let placed = |graha: Graha| {
        NINE.iter()
            .zip(signs)
            .any(|(&one, &at)| one == graha && at == sign)
    };
    let ketu_there = placed(Graha::Ketu);
    let devotions: Vec<Devotion> = NINE
        .into_iter()
        .filter(|&graha| placed(graha))
        .map(|graha| {
            let (deities, verse) = devotion_of(graha, rules);
            Devotion {
                graha,
                deities,
                verse,
                with_ketu: ketu_there && !matches!(graha, Graha::Rahu | Graha::Ketu),
            }
        })
        .collect();
    let malefic_sign = MALEFIC_LORDS.contains(&sign.attributes().lord);
    let minor = [Graha::Saturn, Graha::Venus]
        .into_iter()
        .filter(|&graha| malefic_sign && placed(graha))
        .collect();
    IshtaDevata {
        rules,
        sign,
        devotions,
        minor,
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
}

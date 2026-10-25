//! The fixed significators of BPHS ch. 32, the karaka chapter (the
//! sanskritdocuments e-text's numbering, which is the chapter the repository
//! cites for the chara karakas; `03-design/jaimini-significators.md` §7):
//!
//! - the **sthira karakas** of vv. 18 to 20, each relative's fixed graha, two
//!   of them "the stronger of" a pair — the father's the Sun or Venus, the
//!   mother's the Moon or Mars — which the chart decides;
//! - the **karaka bhavas** of vv. 22 to 24, the house counted from a graha
//!   that shows a relative: the 9th from the Sun the father, the 4th from the
//!   Moon the mother, and so on to the 8th from Saturn, death;
//! - the **bhava karakas** of v. 34, one natural (*naisargika*) significator a
//!   house, the Sun for the 1st to Saturn for the 12th, which no chart changes.
//!
//! The chara karakas of vv. 1 to 15 rank the grahas by degrees and so change
//! from chart to chart; v. 17 sends a chara karaka two grahas' equal degrees
//! leave unheld to these, स्थिरकारकवशात्तस्य फलं ज्ञेयम्.

use core::cmp::Ordering;

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::house::House;

use crate::rashi::{RashiChart, stronger_sign};

/// What a fixed significator signifies: a relative, or death.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_dasha::jaimini::{KARAKA_BHAVA_HOUSES, Signification};
///
/// // The 7th from Venus shows the wife.
/// let (graha, house, of) = KARAKA_BHAVA_HOUSES[5];
/// assert_eq!((graha, house.get(), of), (Graha::Venus, 7, Signification::Wife));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Signification {
    /// The father, पितृ (v. 18; the 9th from the Sun, v. 22).
    Father,
    /// The mother, मातृ (v. 19; the 4th from the Moon, v. 22).
    Mother,
    /// A sister, भगिनी (v. 19, from Mars).
    Sister,
    /// The wife's brother, श्याल (v. 19, from Mars).
    WifesBrother,
    /// A younger brother, कनीयान् (v. 19, from Mars).
    YoungerBrother,
    /// A brother, भ्राता (the 3rd from Mars, v. 23).
    Brother,
    /// The mother's kin, मातृसजातीयाः (v. 20, from Mercury).
    MaternalRelatives,
    /// The maternal uncle and his kin, मातुलाद्याः (v. 20 from Mercury; the
    /// 6th from Mercury, v. 23).
    MaternalUncle,
    /// The paternal grandfather, पितामह (v. 20, from Jupiter).
    PaternalGrandfather,
    /// The husband, पति, the e-text's पितः read as पतिः (v. 20, from Venus;
    /// crux C410).
    Husband,
    /// A son, पुत्र (v. 20 from Saturn; the 5th from Jupiter, v. 23).
    Son,
    /// The wife, दाराः (the 7th from Venus, v. 23).
    Wife,
    /// Death, मृत्यु, of the father and the others (the 8th from Saturn,
    /// v. 24).
    Death,
}

/// The sthira karakas of BPHS ch. 32 vv. 18 to 20: each signification with
/// the grahas the verse names for it, in the verse's order. A pair is "the
/// stronger of" the two (यो बली रविशुक्रयोः, चन्द्रारयोर्बली); one graha is
/// the karaka outright.
///
/// v. 19 names the mother under Mars as well (जननीत्यपि), which the Moon–Mars
/// pair already holds; v. 21's wife, parents, parents-in-law and maternal
/// grandfather name no graha and are not here (C411).
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_dasha::jaimini::{STHIRA_KARAKAS, Signification};
///
/// let (father, of) = STHIRA_KARAKAS[0];
/// assert_eq!(father, Signification::Father);
/// assert_eq!(of, &[Graha::Sun, Graha::Venus]);
/// ```
pub const STHIRA_KARAKAS: [(Signification, &[Graha]); 10] = [
    (Signification::Father, &[Graha::Sun, Graha::Venus]),
    (Signification::Mother, &[Graha::Moon, Graha::Mars]),
    (Signification::Sister, &[Graha::Mars]),
    (Signification::WifesBrother, &[Graha::Mars]),
    (Signification::YoungerBrother, &[Graha::Mars]),
    (Signification::MaternalRelatives, &[Graha::Mercury]),
    (Signification::MaternalUncle, &[Graha::Mercury]),
    (Signification::PaternalGrandfather, &[Graha::Jupiter]),
    (Signification::Husband, &[Graha::Venus]),
    (Signification::Son, &[Graha::Saturn]),
];

/// The karaka bhavas of BPHS ch. 32 vv. 22 to 24: a graha, the house counted
/// from it, and what that house shows — रवितः पुण्यभे तातः, the 9th from the
/// Sun the father, to मन्दादष्टमतो मृत्युः, the 8th from Saturn death.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_dasha::jaimini::{KARAKA_BHAVA_HOUSES, Signification};
///
/// let (graha, house, of) = KARAKA_BHAVA_HOUSES[0];
/// assert_eq!((graha, house.get(), of), (Graha::Sun, 9, Signification::Father));
/// ```
pub const KARAKA_BHAVA_HOUSES: [(Graha, House, Signification); 7] = [
    (Graha::Sun, House::ALL[8], Signification::Father),
    (Graha::Moon, House::ALL[3], Signification::Mother),
    (Graha::Mars, House::ALL[2], Signification::Brother),
    (Graha::Mercury, House::ALL[5], Signification::MaternalUncle),
    (Graha::Jupiter, House::ALL[4], Signification::Son),
    (Graha::Venus, House::ALL[6], Signification::Wife),
    (Graha::Saturn, House::ALL[7], Signification::Death),
];

/// The bhava karakas of BPHS ch. 32 v. 34, the 1st house to the 12th:
/// सूर्यो गुरुः कुजः सोमो गुरुर्भौमो सितः शनिः । गुरुश्चन्द्रसुतो जीवो मन्दश्च
/// — one graha a house, as the verse gives them (C412).
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_dasha::jaimini::BHAVA_KARAKAS;
///
/// // The 10th's is चन्द्रसुत, the Moon's son, Mercury.
/// assert_eq!(BHAVA_KARAKAS[9], Graha::Mercury);
/// ```
pub const BHAVA_KARAKAS: [Graha; 12] = [
    Graha::Sun,
    Graha::Jupiter,
    Graha::Mars,
    Graha::Moon,
    Graha::Jupiter,
    Graha::Mars,
    Graha::Venus,
    Graha::Saturn,
    Graha::Jupiter,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Saturn,
];

/// A house's bhava karaka (BPHS ch. 32 v. 34), the natural significator no
/// chart changes. A number outside 1 to 12 is refused where a [`House`] is
/// made, naming the field `house`.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_core::house::House;
/// use teistro_dasha::jaimini::bhava_karaka;
///
/// assert_eq!(bhava_karaka(House::try_new(7)?), Graha::Venus);
/// assert!(House::try_new(13).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn bhava_karaka(house: House) -> Graha {
    BHAVA_KARAKAS
        .get(usize::from(house.get()) - 1)
        .copied()
        .unwrap_or(Graha::Sun)
}

/// How a sthira karaka was chosen, so a reader can check the choice and a
/// consumer who weighs strength otherwise knows which ones to recompute.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// # use teistro_dasha::rashi::RashiChart;
/// #
/// # let mut signs = [Rashi::Aries; 9];
/// # signs[Graha::Ketu as usize] = Rashi::Libra;
/// # let chart = RashiChart {
/// #     lagna: Rashi::Aries,
/// #     arudha_lagna: Rashi::Aries,
/// #     navamsa_lagna: Rashi::Aries,
/// #     signs,
/// #     dignities: [Dignity::Neutral; 9],
/// #     brahma: None,
/// # };
///
/// use teistro_dasha::jaimini::{ChosenBy, stronger_karaka};
///
/// // Together and equal, the Sun is named first.
/// assert_eq!(stronger_karaka(&chart, Graha::Sun, Graha::Venus), (Graha::Sun, ChosenBy::FirstNamed));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChosenBy {
    /// The verse names one graha.
    Alone,
    /// The two stand in signs BPHS ch. 46 vv. 158 to 166 part
    /// ([`stronger_sign`]): an exalted occupant, more occupants, then the
    /// modality (C408).
    Sign,
    /// Their signs are equal by that ladder — always so when they share one
    /// — and one's dignity is the higher (C409).
    Dignity,
    /// Nothing parts them, and the graha the verse names first is taken, the
    /// Sun for the father and the Moon for the mother (C409).
    FirstNamed,
    /// The caller's own comparison, through [`sthira_karakas_by`].
    Caller,
}

/// One sthira karaka of a chart.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// # use teistro_dasha::rashi::RashiChart;
/// #
/// # let mut signs = [Rashi::Aries; 9];
/// # signs[Graha::Ketu as usize] = Rashi::Libra;
/// # let chart = RashiChart {
/// #     lagna: Rashi::Aries,
/// #     arudha_lagna: Rashi::Aries,
/// #     navamsa_lagna: Rashi::Aries,
/// #     signs,
/// #     dignities: [Dignity::Neutral; 9],
/// #     brahma: None,
/// # };
///
/// use teistro_dasha::jaimini::{ChosenBy, Signification, sthira_karakas};
///
/// let father = &sthira_karakas(&chart)[0];
/// assert_eq!(father.signification, Signification::Father);
/// assert_eq!(father.candidates, [Graha::Sun, Graha::Venus]);
/// assert_eq!((father.graha, father.chosen_by), (Graha::Sun, ChosenBy::FirstNamed));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SthiraKaraka {
    /// What it signifies.
    pub signification: Signification,
    /// The grahas the verse names for it, in the verse's order: one, or the
    /// pair the stronger is taken of.
    pub candidates: Vec<Graha>,
    /// The karaka.
    pub graha: Graha,
    /// How it was chosen.
    pub chosen_by: ChosenBy,
}

/// A karaka bhava of a chart (BPHS ch. 32 vv. 22 to 24): the sign the house
/// counted from a graha falls in, whole signs in the rasi chart.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// # use teistro_dasha::rashi::RashiChart;
/// #
/// # let mut signs = [Rashi::Aries; 9];
/// # signs[Graha::Ketu as usize] = Rashi::Libra;
/// # let chart = RashiChart {
/// #     lagna: Rashi::Aries,
/// #     arudha_lagna: Rashi::Aries,
/// #     navamsa_lagna: Rashi::Aries,
/// #     signs,
/// #     dignities: [Dignity::Neutral; 9],
/// #     brahma: None,
/// # };
///
/// use teistro_dasha::jaimini::{Signification, karaka_bhavas};
///
/// // The 4th from the Moon in Aries, the mother's, is Cancer.
/// let mother = karaka_bhavas(&chart)[1];
/// assert_eq!((mother.graha, mother.house.get()), (Graha::Moon, 4));
/// assert_eq!((mother.signification, mother.sign), (Signification::Mother, Rashi::Cancer));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct KarakaBhava {
    /// The graha counted from.
    pub graha: Graha,
    /// The house counted from its sign.
    pub house: House,
    /// What that house shows.
    pub signification: Signification,
    /// The sign it falls in.
    pub sign: Rashi,
}

/// A graha's dignity's rank, the catalogue's order: deep exaltation first,
/// deep debility last.
fn dignity_rank(chart: &RashiChart, graha: Graha) -> usize {
    chart
        .dignities
        .get(graha as usize)
        .map_or(usize::MAX, |dignity| *dignity as usize)
}

/// The stronger of two grahas as the sthira karakas weigh them (C408, C409):
/// the one in the stronger sign by BPHS ch. 46 vv. 158 to 166 — the measure
/// this module already weighs two grahas by, v. 173's बलाधिक्य for Brahma —
/// then on equal signs the one of higher dignity, then the one named first.
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// use teistro_dasha::jaimini::{ChosenBy, stronger_karaka};
/// use teistro_dasha::rashi::RashiChart;
///
/// // The Sun and Venus together in Taurus: no sign parts them, and Venus
/// // in its own sign outranks a neutral Sun.
/// let mut dignities = [Dignity::Neutral; 9];
/// dignities[Graha::Venus as usize] = Dignity::OwnSign;
/// let chart = RashiChart {
///     lagna: Rashi::Aries,
///     arudha_lagna: Rashi::Aries,
///     navamsa_lagna: Rashi::Aries,
///     signs: [Rashi::Taurus; 9],
///     dignities,
///     brahma: None,
/// };
/// assert_eq!(
///     stronger_karaka(&chart, Graha::Sun, Graha::Venus),
///     (Graha::Venus, ChosenBy::Dignity)
/// );
/// ```
#[must_use]
pub fn stronger_karaka(chart: &RashiChart, first: Graha, second: Graha) -> (Graha, ChosenBy) {
    let (at_first, at_second) = (chart.sign_of(first), chart.sign_of(second));
    if let Some(stronger) = stronger_sign(chart, at_first, at_second) {
        let graha = if stronger == at_first { first } else { second };
        return (graha, ChosenBy::Sign);
    }
    match dignity_rank(chart, first).cmp(&dignity_rank(chart, second)) {
        Ordering::Less => (first, ChosenBy::Dignity),
        Ordering::Greater => (second, ChosenBy::Dignity),
        Ordering::Equal => (first, ChosenBy::FirstNamed),
    }
}

/// The sthira karakas, each pair decided by `pick`.
fn chosen(pick: impl Fn(Graha, Graha) -> (Graha, ChosenBy)) -> [SthiraKaraka; 10] {
    STHIRA_KARAKAS.map(|(signification, candidates)| {
        let (graha, chosen_by) = match candidates {
            [first, second] => pick(*first, *second),
            _ => (
                candidates.first().copied().unwrap_or(Graha::Sun),
                ChosenBy::Alone,
            ),
        };
        SthiraKaraka {
            signification,
            candidates: candidates.to_vec(),
            graha,
            chosen_by,
        }
    })
}

/// A chart's sthira karakas (BPHS ch. 32 vv. 18 to 20), in
/// [`STHIRA_KARAKAS`]'s order, the father's and the mother's the stronger of
/// their pair by [`stronger_karaka`].
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// use teistro_dasha::jaimini::{ChosenBy, Signification, sthira_karakas};
/// use teistro_dasha::rashi::RashiChart;
///
/// // Venus exalted in Pisces, the Sun alone in Aries: Pisces holds an
/// // exalted graha and Aries does not, so Venus is the father's karaka.
/// let mut signs = [Rashi::Gemini; 9];
/// signs[Graha::Sun as usize] = Rashi::Aries;
/// signs[Graha::Venus as usize] = Rashi::Pisces;
/// let mut dignities = [Dignity::Neutral; 9];
/// dignities[Graha::Venus as usize] = Dignity::Exalted;
/// let chart = RashiChart {
///     lagna: Rashi::Aries,
///     arudha_lagna: Rashi::Aries,
///     navamsa_lagna: Rashi::Aries,
///     signs,
///     dignities,
///     brahma: None,
/// };
/// let karakas = sthira_karakas(&chart);
/// assert_eq!(karakas[0].signification, Signification::Father);
/// assert_eq!((karakas[0].graha, karakas[0].chosen_by), (Graha::Venus, ChosenBy::Sign));
/// assert_eq!(karakas[9].graha, Graha::Saturn, "the son's is Saturn, outright");
/// ```
#[must_use]
pub fn sthira_karakas(chart: &RashiChart) -> [SthiraKaraka; 10] {
    chosen(|first, second| stronger_karaka(chart, first, second))
}

/// A chart's sthira karakas with the pairs weighed by the caller: `stronger`
/// answers `Greater` when the first graha is the stronger, `Less` when the
/// second is, and `Equal` when it cannot part them, where
/// [`stronger_karaka`] decides and says how. A consumer weighing by Shadbala,
/// C408's rival reading, compares the two grahas' rupas here.
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// use teistro_dasha::jaimini::{ChosenBy, sthira_karakas_by};
/// use teistro_dasha::rashi::RashiChart;
///
/// let chart = RashiChart {
///     lagna: Rashi::Aries,
///     arudha_lagna: Rashi::Aries,
///     navamsa_lagna: Rashi::Aries,
///     signs: [Rashi::Aries; 9],
///     dignities: [Dignity::Neutral; 9],
///     brahma: None,
/// };
/// // Rupas, the Sun to Saturn: Venus 7.1 over the Sun's 6.2, Mars 5.0
/// // under the Moon's 5.4.
/// let rupas: [f64; 7] = [6.2, 5.4, 5.0, 6.0, 6.5, 7.1, 5.9];
/// let by_rupas = |a: Graha, b: Graha| {
///     let of = |g: Graha| rupas.get(g as usize).copied().unwrap_or(0.0);
///     of(a).total_cmp(&of(b))
/// };
/// let karakas = sthira_karakas_by(&chart, by_rupas);
/// assert_eq!((karakas[0].graha, karakas[0].chosen_by), (Graha::Venus, ChosenBy::Caller));
/// assert_eq!((karakas[1].graha, karakas[1].chosen_by), (Graha::Moon, ChosenBy::Caller));
/// ```
#[must_use]
pub fn sthira_karakas_by(
    chart: &RashiChart,
    stronger: impl Fn(Graha, Graha) -> Ordering,
) -> [SthiraKaraka; 10] {
    chosen(|first, second| match stronger(first, second) {
        Ordering::Greater => (first, ChosenBy::Caller),
        Ordering::Less => (second, ChosenBy::Caller),
        Ordering::Equal => stronger_karaka(chart, first, second),
    })
}

/// A chart's karaka bhavas (BPHS ch. 32 vv. 22 to 24), in
/// [`KARAKA_BHAVA_HOUSES`]'s order: the sign each house counted from its
/// graha falls in.
///
/// ```
/// use teistro_core::catalogue::{Dignity, Graha, Rashi};
/// use teistro_dasha::jaimini::{Signification, karaka_bhavas};
/// use teistro_dasha::rashi::RashiChart;
///
/// // The Sun in Leo: the 9th from it, the father's, is Aries.
/// let mut signs = [Rashi::Taurus; 9];
/// signs[Graha::Sun as usize] = Rashi::Leo;
/// let chart = RashiChart {
///     lagna: Rashi::Aries,
///     arudha_lagna: Rashi::Aries,
///     navamsa_lagna: Rashi::Aries,
///     signs,
///     dignities: [Dignity::Neutral; 9],
///     brahma: None,
/// };
/// let father = karaka_bhavas(&chart)[0];
/// assert_eq!(father.signification, Signification::Father);
/// assert_eq!(father.sign, Rashi::Aries);
/// ```
#[must_use]
pub fn karaka_bhavas(chart: &RashiChart) -> [KarakaBhava; 7] {
    KARAKA_BHAVA_HOUSES.map(|(graha, house, signification)| KarakaBhava {
        graha,
        house,
        signification,
        sign: house.sign_from(chart.sign_of(graha)),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, reason = "tests")]

    use teistro_core::catalogue::Dignity;

    use super::*;

    /// Every graha in `rest` but those `placed` names, all neutral but
    /// those `dignified` names.
    fn chart(rest: Rashi, placed: &[(Graha, Rashi)], dignified: &[(Graha, Dignity)]) -> RashiChart {
        let mut signs = [rest; 9];
        for (graha, sign) in placed {
            signs[*graha as usize] = *sign;
        }
        let mut dignities = [Dignity::Neutral; 9];
        for (graha, dignity) in dignified {
            dignities[*graha as usize] = *dignity;
        }
        RashiChart {
            lagna: Rashi::Aries,
            arudha_lagna: Rashi::Aries,
            navamsa_lagna: Rashi::Aries,
            signs,
            dignities,
            brahma: None,
        }
    }

    fn of(karakas: &[SthiraKaraka; 10], signification: Signification) -> (Graha, ChosenBy) {
        let found = karakas
            .iter()
            .find(|karaka| karaka.signification == signification);
        found.map_or((Graha::Ketu, ChosenBy::Alone), |karaka| {
            (karaka.graha, karaka.chosen_by)
        })
    }

    /// v. 18, worked by the ladder of ch. 46 vv. 158 to 166: the Sun in Leo
    /// with the Moon and Mercury (three occupants), Venus alone in Virgo
    /// (one). Neither sign holds an exalted graha (Venus is debilitated in
    /// Virgo), so occupancy decides: 3 > 1, the Sun.
    ///
    /// v. 19: the Moon in Leo (three) and Mars alone in Capricorn, exalted.
    /// Capricorn holds an exalted graha and Leo does not, so exaltation
    /// decides before occupancy is counted: Mars.
    #[test]
    fn the_pairs_follow_the_sign_ladder() {
        let chart = chart(
            Rashi::Sagittarius,
            &[
                (Graha::Sun, Rashi::Leo),
                (Graha::Moon, Rashi::Leo),
                (Graha::Mercury, Rashi::Leo),
                (Graha::Venus, Rashi::Virgo),
                (Graha::Mars, Rashi::Capricorn),
            ],
            &[
                (Graha::Venus, Dignity::Debilitated),
                (Graha::Mars, Dignity::Exalted),
            ],
        );
        let karakas = sthira_karakas(&chart);
        assert_eq!(
            of(&karakas, Signification::Father),
            (Graha::Sun, ChosenBy::Sign)
        );
        assert_eq!(
            of(&karakas, Signification::Mother),
            (Graha::Mars, ChosenBy::Sign)
        );
    }

    /// The Sun and Venus conjoined in Aries: one sign, so the ladder ties
    /// and dignity decides — the Sun exalted (rank 1) over Venus neutral
    /// (rank 6). With both neutral nothing parts them and the Sun, named
    /// first, is taken.
    #[test]
    fn a_shared_sign_falls_to_dignity_then_the_first_named() {
        let exalted = chart(
            Rashi::Gemini,
            &[(Graha::Sun, Rashi::Aries), (Graha::Venus, Rashi::Aries)],
            &[(Graha::Sun, Dignity::Exalted)],
        );
        assert_eq!(
            of(&sthira_karakas(&exalted), Signification::Father),
            (Graha::Sun, ChosenBy::Dignity)
        );
        let plain = chart(
            Rashi::Gemini,
            &[(Graha::Sun, Rashi::Aries), (Graha::Venus, Rashi::Aries)],
            &[],
        );
        assert_eq!(
            of(&sthira_karakas(&plain), Signification::Father),
            (Graha::Sun, ChosenBy::FirstNamed)
        );
    }

    /// C408's rival, Shadbala: on the chart of
    /// [`the_pairs_follow_the_sign_ladder`] the ladder gives the Sun, and
    /// rupas giving Venus 7.4 against the Sun's 5.9 give Venus. A caller
    /// that cannot part a pair (equal rupas, the Moon and Mars at 5.5) falls
    /// back to the ladder, which says so: Mars by sign.
    #[test]
    fn shadbala_is_the_rival_reading_and_can_disagree() {
        let chart = chart(
            Rashi::Sagittarius,
            &[
                (Graha::Sun, Rashi::Leo),
                (Graha::Moon, Rashi::Leo),
                (Graha::Mercury, Rashi::Leo),
                (Graha::Venus, Rashi::Virgo),
                (Graha::Mars, Rashi::Capricorn),
            ],
            &[(Graha::Mars, Dignity::Exalted)],
        );
        let rupas: [f64; 7] = [5.9, 5.5, 5.5, 6.0, 6.0, 7.4, 6.0];
        let by_rupas = |a: Graha, b: Graha| rupas[a as usize].total_cmp(&rupas[b as usize]);
        let rival = sthira_karakas_by(&chart, by_rupas);
        let ladder = sthira_karakas(&chart);
        assert_eq!(
            of(&rival, Signification::Father),
            (Graha::Venus, ChosenBy::Caller)
        );
        assert_eq!(
            of(&ladder, Signification::Father),
            (Graha::Sun, ChosenBy::Sign)
        );
        assert_eq!(
            of(&rival, Signification::Mother),
            (Graha::Mars, ChosenBy::Sign)
        );
    }

    /// The single karakas of vv. 19 and 20 hold on every chart, whatever
    /// the pairs do.
    #[test]
    fn the_single_karakas_are_the_verses() {
        let karakas = sthira_karakas(&chart(Rashi::Aries, &[], &[]));
        for (signification, graha) in [
            (Signification::Sister, Graha::Mars),
            (Signification::WifesBrother, Graha::Mars),
            (Signification::YoungerBrother, Graha::Mars),
            (Signification::MaternalRelatives, Graha::Mercury),
            (Signification::MaternalUncle, Graha::Mercury),
            (Signification::PaternalGrandfather, Graha::Jupiter),
            (Signification::Husband, Graha::Venus),
            (Signification::Son, Graha::Saturn),
        ] {
            assert_eq!(
                of(&karakas, signification),
                (graha, ChosenBy::Alone),
                "{signification:?}"
            );
        }
    }

    /// C410's rival reads v. 20's पितः as पिता, Venus the father again: that
    /// adds no karaka to v. 18's pair and leaves no husband. The default
    /// keeps the husband, and Venus is in both rows.
    #[test]
    fn venus_signifies_the_husband_and_is_a_candidate_for_the_father() {
        let karakas = sthira_karakas(&chart(Rashi::Aries, &[], &[]));
        let husband = karakas
            .iter()
            .filter(|karaka| karaka.signification == Signification::Husband)
            .count();
        assert_eq!(husband, 1);
        assert_eq!(karakas[0].candidates, vec![Graha::Sun, Graha::Venus]);
        let fathers = STHIRA_KARAKAS
            .iter()
            .filter(|(signification, _)| *signification == Signification::Father)
            .count();
        assert_eq!(fathers, 1, "the father has one row, the pair");
    }

    /// vv. 22 to 24 counted by hand from a chart with the Sun in Leo (index
    /// 4), the Moon in Cancer (3), Mars in Aries (0), Mercury in Virgo (5),
    /// Jupiter in Sagittarius (8), Venus in Libra (6), Saturn in Capricorn
    /// (9): Leo + 8 = 12 ≡ 0 Aries; Cancer + 3 = 6 Libra; Aries + 2 = 2
    /// Gemini; Virgo + 5 = 10 Aquarius; Sagittarius + 4 = 12 ≡ 0 Aries;
    /// Libra + 6 = 12 ≡ 0 Aries; Capricorn + 7 = 16 ≡ 4 Leo.
    #[test]
    fn the_karaka_bhavas_count_from_their_grahas() {
        let chart = chart(
            Rashi::Taurus,
            &[
                (Graha::Sun, Rashi::Leo),
                (Graha::Moon, Rashi::Cancer),
                (Graha::Mars, Rashi::Aries),
                (Graha::Mercury, Rashi::Virgo),
                (Graha::Jupiter, Rashi::Sagittarius),
                (Graha::Venus, Rashi::Libra),
                (Graha::Saturn, Rashi::Capricorn),
            ],
            &[],
        );
        let signs = karaka_bhavas(&chart).map(|bhava| bhava.sign);
        assert_eq!(
            signs,
            [
                Rashi::Aries,
                Rashi::Libra,
                Rashi::Gemini,
                Rashi::Aquarius,
                Rashi::Aries,
                Rashi::Aries,
                Rashi::Leo,
            ]
        );
        let houses = KARAKA_BHAVA_HOUSES.map(|(_, house, _)| house.get());
        assert_eq!(houses, [9, 4, 3, 6, 5, 7, 8]);
    }

    /// v. 34 one graha a house; the rival lists that give a house several
    /// karakas (the 4th the Moon and Mercury, the 10th four) are not the
    /// verse (C412): the 4th's is the Moon alone, the 10th's Mercury alone.
    #[test]
    fn the_bhava_karakas_are_v_34_one_a_house() {
        let by_house = House::ALL.map(bhava_karaka);
        assert_eq!(by_house, BHAVA_KARAKAS);
        assert_eq!(bhava_karaka(House::ALL[3]), Graha::Moon);
        assert_eq!(bhava_karaka(House::ALL[9]), Graha::Mercury);
        // Jupiter holds four houses, the 2nd, 5th, 9th and 11th; Mars and
        // Saturn two each; the Sun, the Moon, Mercury and Venus one.
        let count = |graha| BHAVA_KARAKAS.iter().filter(|g| **g == graha).count();
        assert_eq!(count(Graha::Jupiter), 4);
        assert_eq!((count(Graha::Mars), count(Graha::Saturn)), (2, 2));
        assert_eq!(count(Graha::Rahu) + count(Graha::Ketu), 0);
    }
}

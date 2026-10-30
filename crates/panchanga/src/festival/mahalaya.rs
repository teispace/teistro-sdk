//! The fortnight of the ancestors as *Dharmasindhu* keeps it
//! (`03-design/shraddha.md` §2.5, §3.2): a mahalaya on each death tithi of
//! Bhadrapada's dark half (amanta), and the fortnight's special days, each
//! on the day whose aparahna holds more of its tithi (DS p. 61: "निर्णयो
//! ऽपराह्णव्याप्त्यादौ"; C663, C664). Pages are the 1888 Nirnaya-sagara
//! edition's.

use teistro_core::catalogue::{Masa, Nakshatra, Paksha, Tithi};
use teistro_core::settings::LunarMonth as Convention;

use super::{APARAHNA, FestivalRule, InAdhika, Occurs, by_share};

/// The mahalaya's tithis: the dark 1st to the new moon but the 14th, which
/// DS p. 61 keeps for the killed by a weapon alone. The keys are the ones
/// `NEPAL_PANCHANGA` gives the same days, so a reader names them once; a
/// request naming both packs keeps the later one's rule of each key.
const MAHALAYAS: [(&str, Tithi); 14] = [
    ("PRATIPADA_SHRADDHA", Tithi::KrishnaPratipada),
    ("DWITIYA_SHRADDHA", Tithi::KrishnaDvitiya),
    ("TRITIYA_SHRADDHA", Tithi::KrishnaTritiya),
    ("CHATURTHI_SHRADDHA", Tithi::KrishnaChaturthi),
    ("PANCHAMI_SHRADDHA", Tithi::KrishnaPanchami),
    ("SHASHTHI_SHRADDHA", Tithi::KrishnaShashthi),
    ("SAPTAMI_SHRADDHA", Tithi::KrishnaSaptami),
    ("ASHTAMI_SHRADDHA", Tithi::KrishnaAshtami),
    ("NAVAMI_SHRADDHA", Tithi::KrishnaNavami),
    ("DASHAMI_SHRADDHA", Tithi::KrishnaDashami),
    ("EKADASHI_SHRADDHA", Tithi::KrishnaEkadashi),
    ("DWADASHI_SHRADDHA", Tithi::KrishnaDwadashi),
    ("TRAYODASHI_SHRADDHA", Tithi::KrishnaTrayodashi),
    ("AMAVASYA_SHRADDHA", Tithi::Amavasya),
];

/// The fortnight's own days kept on a tithi, each with what DS says of it.
const SPECIAL: [(&str, Tithi, &str); 4] = [
    (
        "AVIDHAVA_NAVAMI",
        Tithi::KrishnaNavami,
        "Dharmasindhu p. 62: on the 9th, the shraddha of every woman of the family who died before her husband (अविधवानवमी)",
    ),
    (
        "SANYASI_MAHALAYA",
        Tithi::KrishnaDwadashi,
        "Dharmasindhu p. 62: the ascetics' mahalaya on the 12th holding the aparahna (अपराह्णव्यापिन्यां द्वादश्याम्); the text records Vaishnavas keeping it on the Ekadashi's parana day and prefers the 12th",
    ),
    (
        "MAGHA_SHRADDHA",
        Tithi::KrishnaTrayodashi,
        "Dharmasindhu p. 62: a shraddha on the 13th, joined with Magha or alone (त्रयोदश्यां मघायुतायां केवलायां वा), and on Magha alone too, which is not given here",
    ),
    (
        "CHATURDASHI_SHRADDHA",
        Tithi::KrishnaChaturdashi,
        "Dharmasindhu p. 61: the 14th only for the killed by a weapon (श्राद्धं शस्त्रहतस्यैव चतुर्दश्यां प्रकीर्तितम्); any other death on the 14th or the full moon keeps its mahalaya on the 12th or the new moon",
    ),
];

impl FestivalRule {
    /// *Dharmasindhu*'s fortnight of the ancestors (pp. 61–62): fourteen
    /// mahalayas, the 1st to the new moon of Bhadrapada's dark half but the
    /// 14th, then Avidhava Navami, the ascetics' mahalaya on the 12th, Magha
    /// trayodashi, the 14th of the killed by a weapon and the Bharani
    /// shraddha, each on the day holding more
    /// of its tithi, or of Bharani, in the aparahna.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let fortnight = FestivalRule::pitri_paksha();
    /// assert_eq!(fortnight.len(), 19);
    /// assert!(fortnight.iter().all(|rule| rule.source.starts_with("Dharmasindhu p. 6")));
    /// ```
    #[must_use]
    pub fn pitri_paksha() -> Vec<FestivalRule> {
        let rule = |key: &str, occurs: Occurs, source: String| {
            let (decide, otherwise) = by_share();
            FestivalRule {
                key: key.to_owned(),
                source,
                month: Some(Masa::Bhadrapada),
                convention: Convention::Amanta,
                occurs,
                in_adhika: InAdhika::Nija,
                at: APARAHNA,
                decide,
                otherwise,
                parana: Vec::new(),
                meta: super::FestivalMeta::default(),
            }
        };
        let share = "the day holding more of it in the aparahna, the earlier on a tie and when neither does (C663, C664)";
        let mut rules: Vec<FestivalRule> = MAHALAYAS
            .into_iter()
            .map(|(key, tithi)| {
                rule(key, tithi.into(), format!(
                    "Dharmasindhu p. 61: a mahalaya on the death tithi, Bhadrapada's dark half from the 1st to the new moon (भाद्रपदापरपक्षे प्रतिपदमारभ्य दर्शान्ततिथिषु), decided by the aparahna (निर्णयोऽपराह्णव्याप्त्यादौ); {share}"
                ))
            })
            .collect();
        rules.extend(
            SPECIAL
                .into_iter()
                .map(|(key, tithi, said)| rule(key, tithi.into(), format!("{said}; {share}"))),
        );
        rules.push(rule(
            "BHARANI_SHRADDHA",
            Occurs::Nakshatra {
                nakshatra: Nakshatra::Bharani,
                paksha: Paksha::Krishna,
            },
            format!(
                "Dharmasindhu p. 61: once in the dark half, on Bharani, without pinda and by the sankalpa (अपरपक्षे भरणीश्राद्धम् अपिण्डकम्); {share}"
            ),
        ));
        rules
    }
}

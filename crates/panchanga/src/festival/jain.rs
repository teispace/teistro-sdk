//! The Jain festivals and fasts (`03-design/vira-nirvana-and-jain-festivals.md`
//! §3, SR-59): Mahavira's birth and nirvana, the new year of the Vira
//! Nirvana Samvat, the Shvetambaras' Paryushana and Ayambil Oli, the
//! Digambaras' Das Lakshana, and the monthly parva tithis.
//!
//! The two sects, and the gacchas within the Shvetambaras, keep some of
//! these on different tithis. That is never guessed: each rule names the
//! communities that keep it in [`FestivalMeta::communities`], `SHVETAMBARA`
//! or `DIGAMBARA` and, where the gacchas part, the gaccha, so a consumer
//! keeps one tradition's days with a [`MetaFilter`](super::MetaFilter) on
//! the community. A day both sects keep names both.
//!
//! Every rule reads the tithi at sunrise, as Jain calendars do. A parva
//! tithi is never lost or doubled in them: the rule they attribute to
//! Umasvati moves the loss or the doubling onto the tithi before it, so a
//! lost parva tithi is kept on the day of the one before it and a doubled
//! one on its second day ([`UMASVATI`]). Any other tithi is kept on the
//! first sunrise that holds it, or on the day before when none does
//! ([`SUNRISE`]).

use teistro_core::catalogue::{FestivalKind, Masa, Tithi, Tradition};
use teistro_core::settings::LunarMonth as Convention;

use super::{
    Case, Choice, FestivalMeta, FestivalRule, FollowingRule, Guard, InAdhika, Occurs, Window, case,
};

/// The rule a parva tithi's day is reckoned by.
const UMASVATI: &str = "the tithi at sunrise by Umasvati's rule, kṣaye pūrvā tithiḥ kāryā vṛddhau kāryā tathottarā (Tithidin ane Parvaradhan tatha Arhattithibhaskar, Jain Pravachan Pracharak Trust, 1977, p. 257): a parva tithi's loss or doubling falls on the tithi before it, so a lost parva tithi is kept on the day of the one before it and a doubled one on its second day";

/// The rule any other tithi's day is reckoned by.
const SUNRISE: &str = "the tithi at sunrise: the first sunrise that holds it, or the day before when none does, since Umasvati's rule (Tithidin ane Parvaradhan tatha Arhattithibhaskar, 1977, p. 257) moves only a parva tithi's loss or doubling, and this is not one (C1112)";

/// The parva tithis' numbers in a half, the 2nd, 5th, 8th, 11th, 14th
/// and the 15th, full moon or new (Compendium p. 250).
const PARVA: [u8; 6] = [2, 5, 8, 11, 14, 15];

/// Whether a rule is kept on a parva tithi.
fn is_parva(occurs: Occurs) -> bool {
    match occurs {
        Occurs::Tithi { tithi } => PARVA.contains(&tithi.attributes().number),
        Occurs::DayOfPaksha { day_of_paksha } => PARVA.contains(&day_of_paksha),
        Occurs::Nakshatra { .. } => false,
    }
}

/// The JAINA Education Committee's manual.
const COMPENDIUM: &str = "JAINA Education Committee, Compendium of Jainism (3rd ed.)";

/// Both sects.
const BOTH: &[&str] = &["SHVETAMBARA", "DIGAMBARA"];

/// The Shvetambaras.
const SHVETAMBARA: &[&str] = &["SHVETAMBARA"];

/// The Digambaras.
const DIGAMBARA: &[&str] = &["DIGAMBARA"];

/// The gacchas that keep Paryushana from Shravana's dark 12th to
/// Samvatsari on Bhadrapada's bright 4th.
const ON_THE_FOURTH: &[&str] = &["SHVETAMBARA", "TAPAGACHHA", "KHARATARA_GACHHA"];

/// The Shvetambaras that keep it from the dark 13th to the bright 5th.
const ON_THE_FIFTH: &[&str] = &["SHVETAMBARA", "ACHALGACHHA", "STHANAKVASI", "TERAPANTH"];

/// Where the gacchas' two reckonings of Paryushana are stated.
const GACCHAS: &str = "the gacchas part as the Paryushana article of the English Wikipedia relays: Tapagaccha and Kharataragaccha from Shravana's dark 12th (amanta) to Samvatsari on Bhadrapada's bright 4th, the day Kalakacharya moved it to; Achalagaccha, the Sthanakvasis and the Terapanthis from the dark 13th to the bright 5th, a month and twenty nights into the rains as the Kalpa Sutra's rules for ascetics (§1, Jacobi, SBE 22, 1884) count it; no gaccha's own calendar is in hand (C1112)";

/// One yearly rule's row: key, amanta month, tithi, communities, source.
type Yearly = (
    &'static str,
    Masa,
    Tithi,
    &'static [&'static str],
    &'static str,
);

/// The yearly days.
const YEARLY: [Yearly; 12] = [
    (
        "MAHAVIR_JAYANTI",
        Masa::Chaitra,
        Tithi::ShuklaTrayodashi,
        BOTH,
        "Compendium p. 247: the 13th of Chaitra's bright half",
    ),
    (
        "MAHAVIR_NIRVANA",
        Masa::Ashwina,
        Tithi::Amavasya,
        BOTH,
        "Kalpa Sutra, Life of Mahavira §122 (Jacobi, SBE 22, 1884): the fifteenth of Kartika's dark fortnight, its last night, which is amanta Ashwina's new moon, the illumination of §127; Compendium p. 248: Diwali",
    ),
    (
        "PARYUSHANA",
        Masa::Shravana,
        Tithi::KrishnaDwadashi,
        ON_THE_FOURTH,
        "Compendium p. 246: the Shvetambaras' eight days, ending at Samvatsari",
    ),
    (
        "SAMVATSARI",
        Masa::Bhadrapada,
        Tithi::ShuklaChaturthi,
        ON_THE_FOURTH,
        "Compendium p. 246: Samvatsari, Paryushana's last day; Doniger (ed.), Encyclopedia of World Religions (1999) p. 555: Bhadrapada's bright 4th",
    ),
    (
        "PARYUSHANA_TRAYODASHI",
        Masa::Shravana,
        Tithi::KrishnaTrayodashi,
        ON_THE_FIFTH,
        "Compendium p. 246: the Shvetambaras' eight days, ending at Samvatsari",
    ),
    (
        "SAMVATSARI_PANCHAMI",
        Masa::Bhadrapada,
        Tithi::ShuklaPanchami,
        ON_THE_FIFTH,
        "Compendium p. 246: Samvatsari, Paryushana's last day",
    ),
    (
        "DASA_LAKSHANA",
        Masa::Bhadrapada,
        Tithi::ShuklaPanchami,
        DIGAMBARA,
        "Compendium p. 246: the Digambaras' ten days of Das Lakshana as the Shvetambara festival ends; Sangave, Jaina Community (2nd ed., 1980) p. 243: the 5th to the 14th of Bhadrapada's bright half",
    ),
    (
        "ANANTA_CHATURDASHI",
        Masa::Bhadrapada,
        Tithi::ShuklaChaturdashi,
        DIGAMBARA,
        "Sangave, Jaina Community (2nd ed., 1980) p. 243: Das Lakshana's tenth and last day, Ananta Chaturdashi, Bhadrapada's bright 14th",
    ),
    (
        "CHAITRA_AYAMBIL_OLI",
        Masa::Chaitra,
        Tithi::ShuklaSaptami,
        SHVETAMBARA,
        "Compendium p. 249: the nine days of the Navpad Oli in Chaitra; from the 7th of the bright half to the full moon, as the community calendars print it (C1113)",
    ),
    (
        "CHAITRA_AYAMBIL_OLI_END",
        Masa::Chaitra,
        Tithi::Purnima,
        SHVETAMBARA,
        "Compendium p. 249: the Navpad Oli in Chaitra, its ninth day the full moon (C1113)",
    ),
    (
        "ASHVINA_AYAMBIL_OLI",
        Masa::Ashwina,
        Tithi::ShuklaSaptami,
        SHVETAMBARA,
        "Compendium p. 249: the nine days of the Navpad Oli in Ashvina; from the 7th of the bright half to the full moon, as the community calendars print it (C1113)",
    ),
    (
        "ASHVINA_AYAMBIL_OLI_END",
        Masa::Ashwina,
        Tithi::Purnima,
        SHVETAMBARA,
        "Compendium p. 249: the Navpad Oli in Ashvina, its ninth day the full moon (C1113)",
    ),
];

/// One monthly parva tithi's row: key, what it is kept on, communities,
/// source.
type Monthly = (&'static str, Occurs, &'static [&'static str], &'static str);

/// The twelve Shvetambara parva tithis, the 2nd, 5th, 8th, 11th, 14th
/// and 15th of each half (Compendium p. 250), and the Digambaras' four,
/// the 8th and 14th (Samantabhadra's Ratnakaranda Shravakachara v. 106:
/// the proshadhopavasa on the parva, the 8th and the 14th).
const MONTHLY: [Monthly; 7] = [
    (
        "JAIN_PARVA_DVITIYA",
        Occurs::DayOfPaksha { day_of_paksha: 2 },
        SHVETAMBARA,
        "Compendium p. 250: the 2nd of each half, a parva tithi",
    ),
    (
        "JAIN_PARVA_PANCHAMI",
        Occurs::DayOfPaksha { day_of_paksha: 5 },
        SHVETAMBARA,
        "Compendium p. 250: the 5th of each half, a parva tithi",
    ),
    (
        "JAIN_PARVA_ASHTAMI",
        Occurs::DayOfPaksha { day_of_paksha: 8 },
        BOTH,
        "Compendium p. 250: the 8th of each half, a parva tithi; Ratnakaranda Shravakachara v. 106: the Digambaras' fast on the 8th",
    ),
    (
        "JAIN_PARVA_EKADASHI",
        Occurs::DayOfPaksha { day_of_paksha: 11 },
        SHVETAMBARA,
        "Compendium p. 250: the 11th of each half, a parva tithi",
    ),
    (
        "JAIN_PARVA_CHATURDASHI",
        Occurs::DayOfPaksha { day_of_paksha: 14 },
        BOTH,
        "Compendium p. 250: the 14th of each half, a parva tithi; Ratnakaranda Shravakachara v. 106: the Digambaras' fast on the 14th",
    ),
    (
        "JAIN_PARVA_PURNIMA",
        Occurs::Tithi {
            tithi: Tithi::Purnima,
        },
        SHVETAMBARA,
        "Compendium p. 250: the full moon, a parva tithi",
    ),
    (
        "JAIN_PARVA_AMAVASYA",
        Occurs::Tithi {
            tithi: Tithi::Amavasya,
        },
        SHVETAMBARA,
        "Compendium p. 250: the new moon, a parva tithi",
    ),
];

/// A Jain rule's values: kept by the religion, by `communities`.
fn kept_by(communities: &[&str]) -> FestivalMeta {
    FestivalMeta {
        kind: Some(FestivalKind::ReligiousCultural),
        tradition: Some(Tradition::Jain),
        communities: communities.iter().map(|&key| key.to_owned()).collect(),
        // No fast: the Jain fasts (Ayambil's one plain meal, the parva
        // tithis' upavasa) wait on their own sources, not the Hindu texts'.
        ..FestivalMeta::default()
    }
}

/// A rule on `occurs` in `month` (every month when `None`), at sunrise by
/// [`UMASVATI`] on a parva tithi and by [`SUNRISE`] on any other.
fn at_sunrise(
    key: &str,
    month: Option<Masa>,
    occurs: Occurs,
    communities: &[&str],
    source: &str,
) -> FestivalRule {
    let parva = is_parva(occurs);
    let mut decide = vec![Guard::new([case(Case::LaterOnly)], Choice::Later)];
    if parva {
        decide.push(Guard::new([case(Case::Both)], Choice::Later));
    }
    let reckoned = if parva { UMASVATI } else { SUNRISE };
    FestivalRule {
        key: key.to_owned(),
        source: format!("{}; {reckoned}", cited(source, communities)),
        month,
        convention: Convention::Amanta,
        occurs,
        in_adhika: if month.is_none() {
            InAdhika::Also
        } else {
            InAdhika::Nija
        },
        at: Window::Sunrise,
        decide,
        otherwise: Choice::Earlier,
        parana: Vec::new(),
        meta: kept_by(communities),
    }
}

/// A row's source in full: the manual named where the row cites it by
/// page, and the gacchas' reckonings where the row is one of theirs.
fn cited(source: &str, communities: &[&str]) -> String {
    let named = source.replace("Compendium p.", &format!("{COMPENDIUM} p."));
    if communities == ON_THE_FOURTH || communities == ON_THE_FIFTH {
        format!("{named}; {GACCHAS}")
    } else {
        named
    }
}

impl FestivalRule {
    /// The Jain pack's days by the tithi: the yearly days and the monthly
    /// parva tithis, each naming the communities that keep it.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let rules = FestivalRule::jain();
    /// let samvatsari = rules.iter().find(|rule| rule.key == "SAMVATSARI").ok_or("shipped")?;
    /// assert!(samvatsari.meta.communities.iter().any(|key| key == "TAPAGACHHA"));
    /// let das = rules.iter().find(|rule| rule.key == "DASA_LAKSHANA").ok_or("shipped")?;
    /// assert_eq!(das.meta.communities, ["DIGAMBARA"]);
    /// assert!(rules.iter().all(|rule| rule.source.contains("Umasvati")));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn jain() -> Vec<FestivalRule> {
        let yearly = YEARLY
            .iter()
            .map(|&(key, month, tithi, communities, source)| {
                at_sunrise(key, Some(month), tithi.into(), communities, source)
            });
        let monthly = MONTHLY.iter().map(|&(key, occurs, communities, source)| {
            at_sunrise(key, None, occurs, communities, source)
        });
        yearly.chain(monthly).collect()
    }
}

impl FollowingRule {
    /// The Vira Nirvana Samvat's new year, the day after the nirvana, when
    /// Gautama attained kevala (Compendium p. 248).
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let new_year = &FollowingRule::jain()[0];
    /// assert_eq!((new_year.after.as_str(), new_year.days), ("MAHAVIR_NIRVANA", 1));
    /// ```
    #[must_use]
    pub fn jain() -> Vec<FollowingRule> {
        vec![FollowingRule {
            key: "VIRA_NIRVANA_NEW_YEAR".to_owned(),
            source: format!(
                "{COMPENDIUM} p. 248: the new year is the day after the nirvana, when Gautama attained kevala; the Vira Nirvana Samvat's year turns there (teistro_calendar::vira_nirvana)"
            ),
            after: "MAHAVIR_NIRVANA".to_owned(),
            days: 1,
            meta: kept_by(BOTH),
        }]
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_rule_checks_cites_and_keys_are_unique() {
        let mut keys = HashSet::new();
        for rule in FestivalRule::jain() {
            rule.check().unwrap();
            assert!(
                rule.source.contains("p. ") || rule.source.contains("v. "),
                "{}",
                rule.key
            );
            assert_eq!(rule.meta.tradition, Some(Tradition::Jain), "{}", rule.key);
            assert!(!rule.meta.communities.is_empty(), "{}", rule.key);
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::jain() {
            rule.check().unwrap();
            assert!(
                keys.contains(&rule.after),
                "{} follows {}",
                rule.key,
                rule.after
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
    }

    /// Each sect's days are named, never left to a guess: a sect-specific
    /// rule names its sect, and a gaccha's names the Shvetambaras too.
    #[test]
    fn every_rule_names_its_sect() {
        for rule in FestivalRule::jain() {
            let named = |key: &str| rule.meta.communities.iter().any(|held| held == key);
            assert!(named("SHVETAMBARA") || named("DIGAMBARA"), "{}", rule.key);
        }
        let das = FestivalRule::jain()
            .into_iter()
            .find(|rule| rule.key == "DASA_LAKSHANA")
            .unwrap();
        assert_eq!(das.meta.communities, DIGAMBARA);
    }

    /// A doubled parva tithi is kept on its second day, any other on its
    /// first, and a lost one on the day before: the later day on `BOTH`
    /// for a parva tithi alone, the earlier on `NEITHER` for all.
    #[test]
    fn umasvatis_rule_takes_the_second_of_a_doubled_parva_tithi() {
        let later_only = Guard::new([case(Case::LaterOnly)], Choice::Later);
        let both = Guard::new([case(Case::Both)], Choice::Later);
        let mut parvas = 0;
        for rule in FestivalRule::jain() {
            assert_eq!(rule.at, Window::Sunrise);
            assert_eq!(rule.otherwise, Choice::Earlier);
            if is_parva(rule.occurs) {
                parvas += 1;
                assert_eq!(
                    rule.decide,
                    [later_only.clone(), both.clone()],
                    "{}",
                    rule.key
                );
                assert!(rule.source.contains("kṣaye"), "{}", rule.key);
            } else {
                assert_eq!(
                    rule.decide,
                    std::slice::from_ref(&later_only),
                    "{}",
                    rule.key
                );
            }
        }
        // The seven monthly rules, the nirvana, the two Oli ends, the two
        // bright 5ths and the 14th.
        assert_eq!(parvas, 13);
    }
}

//! The days Mithila's almanac keeps by its own rules
//! (`03-design/makaranda.md` §8, SR-16): the Vishwavidyalaya Panchang of
//! Kameshwar Singh Darbhanga Sanskrit University, read on its 2022-23 and
//! 2009-10 editions' pages (Internet Archive `vishwavidyalaya-panchang`;
//! PDF page numbers, which for pp. 11–18 are the almanac's own).
//!
//! The pack lays over another: a request naming `NEPAL_PANCHANGA` and then
//! `MITHILA` keeps the print's Jitiya in place of the committee's, since a
//! rule replaces the one with its key. Each rule is decided by one of the
//! fits the Nepal rules use, so a reader sees the same words for the same
//! decision:
//!
//! - **Jitiya** is the dark 8th held at pradosha, the earlier day only
//!   when it alone holds it, the later on a tie or when neither does: the
//!   print's own boxed rule, after the *Vishnudharmottara*. The committee's
//!   udaya `JITIYA` gives the day after whenever the 8th begins between
//!   sunrise and pradosha (18 September 2022, where the print has the
//!   17th).
//! - **Othgan**, the women's meal before dawn, is the civil day before it.
//! - **Chaurchan**, the Moon worshipped with Rohini on Bhadrapada's bright
//!   4th, is kept on the evening holding the 4th, the earlier when both do.
//! - **Sama-Chakeva** runs from Chhath's dawn arghya, the day after Chhath,
//!   to the immersion on Kartika's full moon held at pradosha.
//! - **Mauna Panchami**, the start of **Madhushravani**, its end, and
//!   **Vidyapati's day** are the plain udaya rule.
//! - **Jur Sital** is the day after the Sun enters Mesha, which is the
//!   first of Ashvini in the sidereal zodiac; the day is the civil day of
//!   the entry, so a night entry's day waits on C1230.
//!
//! Every month is amanta, as the SDK's [`Masa`] is; the print names them
//! purnimanta, so its Shravana's dark half is the amanta Ashadha's.

use teistro_core::catalogue::{FestivalKind, Masa, Nakshatra, Tithi, Tradition};

use super::nepal_panchanga::Fit;
use super::{Dated, DatedRule, FestivalMeta, FestivalRule, FollowingRule, InAdhika, Window};

/// The print, as its rows cite it.
const PRINT: &str = "Vishwavidyalaya Panchang (KSDSU Darbhanga)";

/// One rule's row: key, amanta month, tithi, fit, and where the print
/// gives it.
type Row = (&'static str, Masa, Tithi, Fit, &'static str);

/// The rules by the tithi.
const RULES: [Row; 7] = [
    (
        "JITIYA",
        Masa::Bhadrapada,
        Tithi::KrishnaAshtami,
        Fit::EarlierAlone(Window::Pradosha),
        "2022-23 PDF p. 15, boxed: जीमूतवाहनव्रतम् (जितिया) on the dark 8th of Ashvina (purnimanta) held at pradosha, after the Vishnudharmottara's पूर्वेद्युरपरेद्युर्वा प्रदोषे यत्र चाष्टमी and the Bhavishya; the later day when both days hold it, to agree with udaya and a forenoon parana, and when neither does; printed on 17 September 2022 and, 2009-10 PDF p. 15, on 11 September 2009",
    ),
    (
        "MAUNA_PANCHAMI",
        Masa::Ashadha,
        Tithi::KrishnaPanchami,
        Fit::Udaya,
        "2022-23 PDF p. 11: मौना ५, Manasa's waking and puja on Shravana's dark 5th (purnimanta), printed on 18 July 2022; no clause is printed",
    ),
    (
        "MADHUSHRAVANI_SAMAPTI",
        Masa::Shravana,
        Tithi::ShuklaTritiya,
        Fit::Udaya,
        "2022-23 PDF p. 12: मधुश्रावणीव्रतं पूजनञ्च, Madhushravani's last day on Shravana's bright 3rd, printed on 31 July 2022; no clause is printed, and an adhika Shravana's is not read (C1231)",
    ),
    (
        "CHAURCHAN",
        Masa::Bhadrapada,
        Tithi::ShuklaChaturthi,
        Fit::EarlierHolds(Window::Pradosha),
        "2022-23 PDF p. 14: चौठचन्द्र, the Moon worshipped with Rohini (सरोहिणीकचन्द्रपूजनं) in the evening of Bhadrapada's bright 4th, printed on 30 August 2022 and, 2009-10 PDF p. 14, on 23 August 2009, each the day the 4th began, after the 3rd ended at midday; the tie when both evenings hold the 4th is not printed, and the earlier is read from the two days",
    ),
    (
        "CHHATH",
        Masa::Kartika,
        Tithi::ShuklaShashthi,
        Fit::Udaya,
        "2022-23 PDF p. 18: प्रतिहारषष्ठी, the evening arghya on Kartika's bright 6th, printed on 30 October 2022",
    ),
    (
        "VIDYAPATI_SMRITI_DIVAS",
        Masa::Kartika,
        Tithi::ShuklaTrayodashi,
        Fit::Udaya,
        "2022-23 PDF p. 18: Vidyapati's day, numbered the 596th, on Kartika's bright 13th, printed on 6 November 2022; one printed day, so the udaya reading is inferred",
    ),
    (
        "SAMA_CHAKEVA_VISARJANA",
        Masa::Kartika,
        Tithi::Purnima,
        Fit::EarlierAlone(Window::Pradosha),
        "2022-23 PDF p. 18: उत्तराभिमुखसामाविसर्जनम्, the immersion on the full-moon vrata's day, printed on 7 November 2022 with the bath on the 8th, as the full moon held the 7th's evening alone; the reading at pradosha is inferred from the vrata's day",
    ),
];

/// The days counted from another rule's: key, the rule, the count, and
/// where.
const COUNTED: [(&str, &str, i8, &str); 3] = [
    (
        "OTHGAN",
        "JITIYA",
        -1,
        "2022-23 PDF p. 15: रात्र्यन्ते स्त्रीणां विशिष्टभोजनम् (ओठगन), the women's meal at the night's end, on the row before Jitiya: 16 September 2022, and 10 September 2009 (2009-10 PDF p. 15)",
    ),
    (
        "MADHUSHRAVANI_ARAMBHA",
        "MAUNA_PANCHAMI",
        0,
        "2022-23 PDF p. 11: मधुश्रावणीपूजाकथारम्भः, Madhushravani's puja and stories begun on Mauna Panchami's day, 18 July 2022",
    ),
    (
        "SAMA_CHAKEVA_ARAMBHA",
        "CHHATH",
        1,
        "2022-23 PDF p. 18: प्रातःकालिकार्घदानं पारणञ्च, सामापूजारम्भः, Sama's puja begun with Chhath's dawn arghya and parana, the day after Chhath: 31 October 2022",
    ),
];

/// A Mithila rule's values: a Hindu observance the Maithils keep, as the
/// Mithila print gives it.
fn maithil() -> FestivalMeta {
    FestivalMeta {
        kind: Some(FestivalKind::ReligiousCultural),
        tradition: Some(Tradition::Hindu),
        communities: vec!["MAITHIL".to_owned()],
        ..FestivalMeta::default()
    }
}

impl FestivalRule {
    /// The Mithila print's days by the tithi, each citing its page: Jitiya
    /// at pradosha, Mauna Panchami, Madhushravani's last day, Chaurchan,
    /// Chhath, Vidyapati's day and Sama's immersion.
    ///
    /// ```
    /// use teistro_panchanga::festival::{FestivalRule, Window};
    ///
    /// let rules = FestivalRule::mithila();
    /// let jitiya = rules.iter().find(|rule| rule.key == "JITIYA").ok_or("shipped")?;
    /// assert_eq!(jitiya.at, Window::Pradosha);
    /// assert!(jitiya.source.contains("PDF p. 15"));
    /// assert!(rules.iter().all(|rule| rule.meta.communities == ["MAITHIL"]));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn mithila() -> Vec<FestivalRule> {
        RULES
            .iter()
            .map(|&(key, month, tithi, fit, printed)| FestivalRule {
                meta: maithil(),
                ..fit.rule(
                    key,
                    Some(month),
                    tithi.into(),
                    InAdhika::Nija,
                    &format!("{PRINT} {printed}; the print's rule"),
                )
            })
            .collect()
    }
}

impl FollowingRule {
    /// The Mithila print's days counted from another of its rules: Othgan
    /// the day before Jitiya, Madhushravani begun on Mauna Panchami, and
    /// Sama's puja begun the day after Chhath.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let othgan = &FollowingRule::mithila()[0];
    /// assert_eq!((othgan.key.as_str(), othgan.after.as_str(), othgan.days), ("OTHGAN", "JITIYA", -1));
    /// ```
    #[must_use]
    pub fn mithila() -> Vec<FollowingRule> {
        COUNTED
            .iter()
            .map(|&(key, after, days, printed)| FollowingRule {
                key: key.to_owned(),
                source: format!("{PRINT} {printed}"),
                after: after.to_owned(),
                days,
                meta: maithil(),
            })
            .collect()
    }
}

impl DatedRule {
    /// The Mithila print's day kept by the Sun: Jur Sital, the day after
    /// the Sun enters Mesha, the first of Ashvini.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_panchanga::festival::{Dated, DatedRule};
    ///
    /// let jur_sital = &DatedRule::mithila()[0];
    /// assert_eq!(jur_sital.on, Dated::SunEnters { nakshatra: Nakshatra::Ashwini, days: 1 });
    /// ```
    #[must_use]
    pub fn mithila() -> Vec<DatedRule> {
        vec![DatedRule {
            key: "JUR_SITAL".to_owned(),
            source: format!(
                "{PRINT} 2022-23 PDF p. 29, read in the OCR: जूडिशीतल with stale food and the trees watered, on the day the print numbers 02 of the solar month and calls मासादि, after the Mesha ingress at 29|20 ghatis on 14 April 2023 (सतुआइन, 2006-07): 15 April 2023; the Sun enters Mesha at the first of Ashvini; an entry at night's civil day is not read (C1230)"
            ),
            on: Dated::SunEnters {
                nakshatra: Nakshatra::Ashwini,
                days: 1,
            },
            meta: maithil(),
        }]
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use std::collections::HashSet;

    use super::super::{Case, Choice, Guard, case};
    use super::*;

    #[test]
    fn every_rule_checks_cites_the_print_and_keys_are_unique() {
        let mut keys = HashSet::new();
        for rule in FestivalRule::mithila() {
            rule.check().unwrap();
            assert!(rule.source.contains("PDF p. "), "{}", rule.key);
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::mithila() {
            rule.check().unwrap();
            assert!(
                keys.contains(&rule.after),
                "{} follows {}",
                rule.key,
                rule.after
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::mithila() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
    }

    /// The print's Jitiya: the earlier day only when it alone holds the
    /// 8th at pradosha.
    #[test]
    fn jitiya_is_the_earlier_alone_at_pradosha() {
        let jitiya = FestivalRule::mithila().remove(0);
        assert_eq!(jitiya.key, "JITIYA");
        assert_eq!(
            (jitiya.at, jitiya.decide, jitiya.otherwise),
            (
                Window::Pradosha,
                vec![Guard::new([case(Case::EarlierOnly)], Choice::Earlier)],
                Choice::Later
            )
        );
        assert!(jitiya.source.contains("pradosha: the earlier day only"));
    }
}

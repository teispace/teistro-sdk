//! Gujarat's public holidays (`03-design/gujarat-holidays.md`, SR-03, the
//! eighth state set): the days the General Administration Department's
//! yearly notification lists in its Part I as public holidays for the
//! State Government's offices, with those its Note (1) names as falling on
//! a Sunday or a second or fourth Saturday.
//!
//! The notification is rank 1 for the list. A day both this list and the
//! central list hold reuses `INDIA_CENTRAL`'s rule, and a day a shipped
//! text or print keeps reuses that rule. Gujarat's lunar months are
//! amanta and its Vikram Samvat year is Kartikadi: the New Year (Bestu
//! Varas) is Kartika's bright 1st, the day after Diwali's new moon or the
//! next, Bhai Bij Kartika's bright 2nd, and Janmashtami "Shravan Vad-8",
//! Shravana's dark 8th. Makar Sankranti (Uttarayan) is 14 January in every
//! year, and Samvatsari the Jain pack's Bhadrapada's bright 4th, the
//! Chaturthi Paksha's. `festival-measured.md` §17 holds every rule to the
//! notifications at Gandhinagar.
//!
//! Ramjan-Eid, Bakri-Eid, Muharram and Eid-e-Meeladunnabi are not
//! reckoned: Note (2) leaves their days to the moon, so they ship as
//! printed, marked [`SubjectTo::MoonSighting`]. The Parsi New Year
//! (Shahenshahi) ships as printed, unmarked (C1243). Part II's optional
//! holidays and Part III's banks' list are not here (C1448).

use teistro_core::catalogue::{Masa, Tithi, Tradition};

use super::india_central::{Civil, Own, Printed, PrintedDays, StateList, as_cited, sighted};
use super::nepal_panchanga::Fit;
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{DatedRule, FestivalMeta, FestivalRule, FollowingRule, PublishedSet};

/// Where the list is kept: India, then the state the notification names.
const REGIONS: [&str; 2] = ["IN", "IN-GJ"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The notification the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of Gujarat, General Administration Department, notification \
     No. GS/22/2025/JSR/2025/509/GH of 27 November 2025 (Gujarat Government Gazette, \
     Extraordinary No. 160, Part IV-A): Part I's public holidays for 2026 and its Note (1)";

/// Each year's notification, and the copy read.
const NOTIFICATIONS: [(i32, &str); 8] = [
    (
        2019,
        "No. GS-32/2018-PHD/102018/1411/GH of 3 November 2018, rank 1 (the Government Central Press print)",
    ),
    (
        2020,
        "No. GS/30/2019/PHD/102019/1111/GH of 19 October 2019, rank 1 as copied (a staff site's copy of its text)",
    ),
    (
        2021,
        "No. GS/33/2020/PHD/102020/500/GH of 28 October 2020, rank 1 as copied (the signed scan)",
    ),
    (
        2022,
        "No. GS/20/2021/PHD/102021/431/GH of 24 November 2021, rank 1 as copied (the signed scan)",
    ),
    (
        2023,
        "No. GS/16/2022/PHD/102022/603/GH of 5 November 2022, rank 1 as copied (the signed scan)",
    ),
    (
        2024,
        "No. GS/27/2023/JSR/102023/930/GH of 20 November 2023, rank 1 as copied (the signed scan)",
    ),
    (
        2025,
        "No. GS/26/2024/JSR/2024/605/GH of 14 November 2024, rank 1 as copied (the signed scan)",
    ),
    (
        2026,
        "No. GS/22/2025/JSR/2025/509/GH of 27 November 2025, rank 1 (the Gujarat Government Gazette, Extraordinary No. 160)",
    ),
];

/// Gujarat's convention for a lunar day, as a new rule cites it.
const ALMANAC: &str = "Gujarat's almanacs, as the state's notifications of 2019 to 2026 keep \
     the day; no almanac's page is read, so the rule is held to the notifications and the \
     citation is open (C1441)";

/// Gujarat's rules by the tithi of an amanta month.
const GUJARAT: [Own; 1] = [(
    "VIKRAM_SAMVAT_NEW_YEAR",
    Masa::Kartika,
    Tithi::ShuklaPratipada,
    Fit::Udaya,
    false,
    "Nutan Varsh Din, the Vikram Samvat New Year, Besatu Varsh (Kartak sud-1), Kartika's bright 1st, kept at sunrise where Dharmasindhu's Bali pratipada keeps the earlier day unless the 1st lasts nine muhurtas past the later sunrise (C1442)",
)];

/// The rules of the central list this list keeps: every lunar one but
/// Buddha Purnima, on every year's list.
const CENTRAL_LEFT: [&str; 1] = ["BUDDHA_JAYANTI"];

/// Nepal's printed panchanga's rules this list keeps, by the key it keeps
/// them under, each with the print's key and whether it is optional:
/// Chetichand is kept on Chaitra's bright 1st at sunrise, the print's
/// Samvatsara arambha, the day after Chaitra's new moon (C1446).
const PRINT_KEPT: [(&str, &str, bool); 2] = [
    ("CHETI_CHAND", "SAMVATSARA_ARAMBHA", false),
    ("PARASHURAMA_JAYANTI", "PARASHURAMA_JAYANTI", false),
];

/// The Jain pack's Samvatsari, the Chaturthi Paksha's, as the notification
/// names it.
const JAIN_KEPT: [(&str, &str, bool); 1] = [("SAMVATSARI", "SAMVATSARI", false)];

/// The state's days on a Gregorian date: Makar Sankranti on 14 January
/// (C1440), Dr Ambedkar's birthday and Sardar Patel's.
const CIVIL: [Civil; 3] = [
    (
        "UTTARAYANA_PUNYAKALA",
        (1, 14),
        Some(Tradition::Hindu),
        &[],
        false,
    ),
    ("AMBEDKAR_JAYANTI", (4, 14), None, &[], false),
    ("SARDAR_PATEL_JAYANTI", (10, 31), None, &[], false),
];

/// The days set by the moon, as (month, day) in the order of
/// [`NOTIFICATIONS`], each named as the central list names it.
const SIGHTED_HERE: [(&str, &[(u8, u8)]); 4] = [
    (
        "EID_AL_FITR",
        &[
            (6, 6),
            (5, 25),
            (5, 14),
            (5, 3),
            (4, 22),
            (4, 11),
            (3, 31),
            (3, 21),
        ],
    ),
    (
        "EID_AL_ADHA",
        &[
            (8, 12),
            (8, 1),
            (7, 21),
            (7, 10),
            (6, 29),
            (6, 17),
            (6, 7),
            (5, 27),
        ],
    ),
    (
        "MUHARRAM",
        &[
            (9, 11),
            (8, 30),
            (8, 19),
            (8, 9),
            (7, 29),
            (7, 17),
            (7, 6),
            (6, 26),
        ],
    ),
    (
        "MAWLID",
        &[
            (11, 10),
            (10, 30),
            (10, 19),
            (10, 9),
            (9, 28),
            (9, 16),
            (9, 5),
            (8, 26),
        ],
    ),
];

/// The Parsi New Year (Shahenshahi), Pateti, as the notifications print
/// it: the catalogue has no Shahenshahi calendar (C1243).
const PARSI_NEW_YEAR: Printed<'static> = (
    "PARSI_NEW_YEAR",
    "Parsi New Year Day - Pateti (Parsi Shahenshahi)",
    "पारसी नयाँ वर्ष (पतेती, शहंशाही)",
    &[
        (8, 17),
        (8, 16),
        (8, 16),
        (8, 16),
        (8, 16),
        (8, 15),
        (8, 15),
        (8, 15),
    ],
);

/// The values of a day of the list.
fn listed(tradition: Option<Tradition>, optional: bool) -> FestivalMeta {
    STATE.listed(tradition, optional)
}

impl FestivalRule {
    /// The lunar days of Gujarat's public holidays: the central list's
    /// rules for every lunar day it holds but Buddha Purnima, Holi's Holika
    /// fire among them; *Dharmasindhu*'s for Raksha Bandhan and Bhai Bij;
    /// Nepal's printed panchanga's for Chetichand (Chaitra's bright 1st)
    /// and Parshuram Jayanti; the Jain pack's Samvatsari; and Gujarat's own
    /// Vikram Samvat New Year.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let rules = FestivalRule::india_gujarat();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let new_year = of("VIKRAM_SAMVAT_NEW_YEAR").ok_or("shipped")?;
    /// assert_eq!(new_year.meta.regions, ["IN", "IN-GJ"]);
    /// // Chetichand is the print's Chaitra shukla 1, Samvatsari the Jain pack's.
    /// assert!(of("CHETI_CHAND").ok_or("shipped")?.source.starts_with("Nepal's national panchanga"));
    /// assert!(of("SAMVATSARI").ok_or("shipped")?.source.contains("Bhadrapada's bright 4th"));
    /// assert!(of("BUDDHA_JAYANTI").is_none());
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_gujarat() -> Vec<FestivalRule> {
        let mut rules = STATE.central(&CENTRAL_LEFT);
        rules.extend(STATE.by_text(&[("RAKSHABANDHAN", false), ("YAMA_DWITIYA", false)]));
        rules.extend(STATE.print_kept(&PRINT_KEPT));
        rules.extend(STATE.taken(&FestivalRule::jain(), Tradition::Jain, &JAIN_KEPT, as_cited));
        rules.extend(STATE.own(&GUJARAT, |_, what| format!("{NOTICE}: {what}; {ALMANAC}")));
        rules
    }
}

impl FollowingRule {
    /// Holi's second day, Dhuleti, the central list's Holi: the day after
    /// the Holika fire.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let dhuleti = &FollowingRule::india_gujarat()[0];
    /// assert_eq!((dhuleti.after.as_str(), dhuleti.days), ("HOLIKA", 1));
    /// assert_eq!(dhuleti.meta.regions, ["IN", "IN-GJ"]);
    /// assert!(!dhuleti.meta.optional);
    /// ```
    #[must_use]
    pub fn india_gujarat() -> Vec<FollowingRule> {
        FollowingRule::india_central()
            .into_iter()
            .map(|rule| FollowingRule {
                meta: STATE.kept(rule.meta),
                ..rule
            })
            .collect()
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, Makar Sankranti on
    /// 14 January, Dr Ambedkar's birthday and Sardar Vallabhbhai Patel's.
    ///
    /// ```
    /// use teistro_panchanga::festival::{Dated, DatedRule};
    ///
    /// let rules = DatedRule::india_gujarat();
    /// assert_eq!(rules.len(), 8);
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key).map(|rule| rule.on);
    /// assert!(matches!(of("UTTARAYANA_PUNYAKALA"), Some(Dated::CivilDate { month: 1, day: 14, .. })));
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-GJ"]));
    /// ```
    #[must_use]
    pub fn india_gujarat() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        rules.extend(STATE.civil(&CIVIL, NOTICE));
        rules
    }
}

impl PublishedSet {
    /// The days the notifications print rather than reckon: the four set by
    /// the moon, each marked subject to the sighting, and the Parsi New
    /// Year (Shahenshahi), unmarked.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_gujarat();
    /// assert_eq!(set.entries.len(), 5 * 8);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2019, 6, 6));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// ```
    #[must_use]
    pub fn india_gujarat() -> PublishedSet {
        let mut entries = PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic), false),
            cite: |notice| {
                format!(
                    "Government of Gujarat, General Administration Department, notification {notice}, its day subject to the moon (Note (2))"
                )
            },
        }
        .entries(&sighted(&SIGHTED_HERE));
        entries.extend(
            PrintedDays {
                notices: &NOTIFICATIONS,
                declared: &[],
                subject_to: None,
                meta: FestivalMeta {
                    communities: vec!["PARSI".to_owned()],
                    ..listed(Some(Tradition::Zoroastrian), false)
                },
                cite: |notice| {
                    format!(
                        "Government of Gujarat, General Administration Department, notification {notice}, as printed: the catalogue has no Shahenshahi calendar (C1243)"
                    )
                },
            }
            .entries(&[PARSI_NEW_YEAR]),
        );
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of Gujarat, General Administration Department".to_owned(),
            id: "in-gj-gad-printed".to_owned(),
            version: 1,
            entries,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use std::collections::HashSet;

    use teistro_core::catalogue::FestivalKind;

    use super::*;
    use crate::festival::PublishedEntry;

    #[test]
    fn every_rule_checks_and_keys_are_unique() {
        let mut keys = HashSet::new();
        for rule in FestivalRule::india_gujarat() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_gujarat() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::india_gujarat() {
            assert!(
                keys.contains(&rule.after),
                "{} counts from nothing",
                rule.key
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for entry in PublishedSet::india_gujarat().entries {
            if let PublishedEntry::Add { key, .. } = entry {
                assert!(!keys.contains(&key), "{key} is reckoned and printed");
            }
        }
    }

    #[test]
    fn every_day_of_the_list_is_gazetted_in_the_state() {
        let lunar = FestivalRule::india_gujarat()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let dated = DatedRule::india_gujarat()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let following = FollowingRule::india_gujarat()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let metas: Vec<(String, FestivalMeta)> = lunar.chain(dated).chain(following).collect();
        assert_eq!(metas.len(), 14 + 8 + 1);
        for (key, meta) in &metas {
            assert_eq!(meta.regions, REGIONS, "{key}");
            assert!(!meta.optional, "{key}");
            if key != "HOLIKA" {
                assert_eq!(meta.kind, Some(FestivalKind::Gazetted), "{key}");
            }
        }
    }

    #[test]
    fn a_day_both_lists_hold_is_the_central_rule() {
        let central = FestivalRule::india_central();
        for rule in FestivalRule::india_gujarat() {
            if let Some(shared) = central.iter().find(|other| other.key == rule.key) {
                assert_eq!((&rule.at, &rule.source), (&shared.at, &shared.source));
            }
        }
    }

    #[test]
    fn every_printed_day_is_in_a_noticed_year() {
        let set = PublishedSet::india_gujarat();
        assert_eq!(
            set.entries.len(),
            (SIGHTED_HERE.len() + 1) * NOTIFICATIONS.len()
        );
        let marked = set
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry,
                    PublishedEntry::Add {
                        subject_to: Some(SubjectTo::MoonSighting),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(marked, SIGHTED_HERE.len() * NOTIFICATIONS.len());
    }
}

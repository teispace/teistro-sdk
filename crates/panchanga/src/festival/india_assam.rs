//! Assam's public holidays (`03-design/assam-holidays.md`, SR-03, the
//! fifth state set): the days the General Administration Department's
//! yearly notification closes the offices of the Government of Assam and
//! its revenue and magisterial courts, the two it makes half holidays, and
//! the annual closing of bank accounts.
//!
//! The notification is rank 1 for the list, and from 2022 prints every
//! day with its Bhaskarabda date beside the Gregorian and the Saka. A day
//! both this list and the central list hold reuses `INDIA_CENTRAL`'s
//! rule, and a day West Bengal's list keeps by Bengal's reckoning reuses
//! `INDIA_WEST_BENGAL`'s. Assam's own days are kept by the Assamese month
//! (`Calendar::AssamSolar`, the Surya Siddhanta's Sun, C1300): the three
//! Bihus on the months' ends, Sati Sadhani Divas on Bohag 7, Gwthar Bathou
//! San on Magh's second Tuesday, and the Vaishnava saints' days and
//! Janmashtami on a tithi of an Assamese month, the form `tithiInMonth`
//! (C1361). `festival-measured.md` §14 holds every rule to eight years of
//! notifications at Guwahati.
//!
//! Id-ul-Fitr, Id-ul-Zuha and Muharram are not reckoned: their days are
//! the moon's, so they ship as printed, marked
//! [`SubjectTo::MoonSighting`], as the central list's are.

use teistro_core::catalogue::{Calendar, Masa, Tithi, Tradition, Vara};

use super::india_central::{
    Named, OnDay, PrintedDays, SIGHTED, StateList, find, on_civil_date, sighted,
};
use super::nepal_panchanga::Fit;
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{
    Dated, DatedRule, FestivalMeta, FestivalRule, FollowingRule, InAdhika, PublishedSet, Which,
    Window,
};

/// Where the list is kept: India, then the state the notice names.
const REGIONS: [&str; 2] = ["IN", "ASSAM"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The central list's lunar rules this list keeps: Buddha Purnima and
/// Guru Nanak's birthday.
const CENTRAL_KEPT: [&str; 2] = ["BUDDHA_JAYANTI", "GURU_NANAK_JAYANTI"];

/// West Bengal's rules this list keeps, the Durga Puja's Navami and the
/// days the two states keep alike: Dol Jatra, and Kali Puja on the new
/// moon held at nishitha.
const BENGAL_KEPT: [&str; 3] = ["DOLA_YATRA", "MAHA_NAVAMI", "KALI_PUJA"];

/// The notification the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of Assam, General Administration Department, notification e-file \
     No. 221561/440 of 17 November 2025: the list of holidays for the year 2026 in the State of \
     Assam";

/// Each year's notification, and how it was read.
const NOTIFICATIONS: [(i32, &str); 8] = [
    (
        2019,
        "No. GAG(A).81/2016/Pt/I/220 of 19 December 2018, Assam Gazette Extraordinary No. 629, rank 1 (the gazette's scan)",
    ),
    (
        2020,
        "No. GAG(A)81/2016/494 of 29 November 2019, Assam Gazette Extraordinary No. 486, rank 1 (the gazette's scan)",
    ),
    (
        2021,
        "No. GAG(A).81/2016/637 of 18 December 2020, Assam Gazette Extraordinary No. 566, rank 1 (the gazette's scan)",
    ),
    (
        2022,
        "No. GAG(A).81/2016/749 of 28 October 2021, with its corrigenda of 30 December 2021 and 12 January 2022, rank 1 (the signed notification's scan)",
    ),
    (
        2023,
        "e-file No. 221561/1 of 28 November 2022, rank 1 (the signed notification's scan)",
    ),
    (
        2024,
        "e-file No. 221561/11 of November 2023, rank 1 as copied (a copy of the e-signed notification)",
    ),
    (
        2025,
        "No. E-221561/347 of 6 December 2024, with its corrigendum of 27 December 2024, rank 1 (the e-signed notification)",
    ),
    (
        2026,
        "e-file No. 221561/440 of 17 November 2025, rank 1 (the e-signed notification)",
    ),
];

/// Assam's convention for a lunar day or a tithi of an Assamese month, as
/// a new rule cites it.
const ALMANAC: &str = "Assam's almanacs, as the state's notifications of 2019 to 2026 keep the \
     day; no almanac's page is read, so the rule is held to the notifications and the citation is \
     open (C1361)";

/// The window the days of a saint's passing are judged at: the whole
/// daylight, its fifteen muhurtas (C1362).
const DAYLIGHT: Window = Window::DayMuhurta { from: 1, to: 15 };

/// Assam's rules by the tithi of a lunar month: key, amanta month, tithi,
/// fit, and what the day is.
const ASSAM: [(&str, Masa, Tithi, Fit, &str); 2] = [
    (
        "BIR_CHILARAI_DIVAS",
        Masa::Magha,
        Tithi::Purnima,
        Fit::Udaya,
        "Bir Chilarai Divas, Magha's full moon in every year listed",
    ),
    (
        "KARAM_PUJA",
        Masa::Bhadrapada,
        Tithi::ShuklaEkadashi,
        Fit::Udaya,
        "Karam Puja, Bhadrapada's bright 11th, listed from 2021",
    ),
];

/// Assam's days on a tithi of an Assamese month: key, month (Bohag the
/// 1st), tithi, the window it is judged at, and what the day is.
const IN_MONTH: [(&str, u8, Tithi, Window, &str); 6] = [
    (
        "DAMODARDEVA_TITHI",
        1,
        Tithi::ShuklaPratipada,
        DAYLIGHT,
        "the Tithi of Sri Sri Damodardeva, the bright 1st in Bohag, the day whose daylight holds more of it",
    ),
    (
        "MADHABDEVA_JANMOTSAV",
        2,
        Tithi::KrishnaPratipada,
        Window::Sunrise,
        "the Janmotsav of Sri Sri Madhabdeva, the dark 1st in Jeth",
    ),
    (
        "MADHABDEVA_TIROBHAV",
        5,
        Tithi::KrishnaPanchami,
        DAYLIGHT,
        "the Tirobhav Tithi of Sri Sri Madhabdeva, the dark 5th in Bhada, the day whose daylight holds more of it",
    ),
    (
        "JANMASHTAMI",
        5,
        Tithi::KrishnaAshtami,
        Window::Sunrise,
        "Janmastomi, the dark 8th in Bhada",
    ),
    (
        "SANKARDEVA_TIROBHAV",
        5,
        Tithi::ShuklaDvitiya,
        DAYLIGHT,
        "the Tirobhav Tithi of Srimanta Sankardeva, the bright 2nd in Bhada, the day whose daylight holds more of it",
    ),
    (
        "SANKARDEVA_JANMOTSAV",
        6,
        Tithi::ShuklaDashami,
        Window::Sunrise,
        "the Janmotsav of Srimanta Sankardeva, the bright 10th in Ahin",
    ),
];

/// The Bihus and Sati Sadhani Divas, on a day of an Assamese month: key,
/// month, day (negative from the month's last), and what the notification
/// prints.
const ON_DATE: [(&str, u8, i8, &str); 7] = [
    (
        "MAGH_BIHU",
        9,
        -1,
        "Magh Bihu & Tusu Puja, the last day of Puh, the Makara sankranti's (C1306)",
    ),
    ("MAGH_BIHU_SECOND", 10, 1, "Magh Bihu's second day, Magh 1"),
    (
        "BOHAG_BIHU",
        12,
        -1,
        "Bohag Bihu's first day, the last day of Chot, the Mesha sankranti's",
    ),
    (
        "BOHAG_BIHU_SECOND",
        1,
        1,
        "Bohag Bihu's second day, Bohag 1, the Bhaskarabda new year",
    ),
    ("BOHAG_BIHU_THIRD", 1, 2, "Bohag Bihu's third day, Bohag 2"),
    (
        "SATI_SADHANI_DIVAS",
        1,
        7,
        "Sati Sadhini Divas, Bohag 7, listed from 2021",
    ),
    (
        "KATI_BIHU",
        6,
        -1,
        "Kati Bihu, the last day of Ahin, the Tula sankranti's (C1306)",
    ),
];

/// The state's days on a Gregorian date: key, (month, day), and the
/// communities a day is for.
const CIVIL: [(&str, (u8, i8), &[&str]); 6] = [
    ("NETAJI_JAYANTI", (1, 23), &[]),
    ("ME_DAM_ME_PHI", (1, 31), &[]),
    ("BANK_ACCOUNTS_CLOSING", (4, 1), &["BANKS"]),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1), &[]),
    ("LACHIT_DIVAS", (11, 24), &[]),
    ("ASOM_DIVAS", (12, 2), &[]),
];

/// The days set by the moon in every year, as (month, day) in the order
/// of [`NOTIFICATIONS`], each named as the central list names it.
const SIGHTED_HERE: [(&str, &[(u8, u8)]); 2] = [
    (
        "EID_AL_FITR",
        &[
            (6, 5),
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
];

/// Muharram, a half holiday, in the years a notification lists it: 2025
/// and 2026 list none.
const MUHARRAM_DAYS: [OnDay<'static>; 6] = [
    (2019, "MUHARRAM", (9, 10)),
    (2020, "MUHARRAM", (8, 30)),
    (2021, "MUHARRAM", (8, 19)),
    (2022, "MUHARRAM", (8, 9)),
    (2023, "MUHARRAM", (7, 29)),
    (2024, "MUHARRAM", (7, 17)),
];

/// The values of a day of the list.
fn listed(tradition: Option<Tradition>) -> FestivalMeta {
    STATE.listed(tradition, false)
}

impl FestivalRule {
    /// The lunar days of Assam's public holidays: the central list's rules
    /// for Buddha Purnima and Guru Nanak's birthday, West Bengal's for Dol
    /// Jatra, the Durga Puja's Navami and Kali Puja, Nepal's printed
    /// panchanga's for the Puja's Saptami and Ashtami, Lakshmi Puja and
    /// Chhath Puja, *Dharmasindhu*'s for Bhatri Dwitiya, and Assam's own
    /// for Bir Chilarai Divas and Karam Puja.
    ///
    /// ```
    /// use teistro_panchanga::festival::{FestivalRule, Window};
    ///
    /// let rules = FestivalRule::india_assam();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let kali = of("KALI_PUJA").ok_or("shipped")?;
    /// assert_eq!(kali.at, Window::Nishitha);
    /// assert_eq!(kali.meta.regions, ["IN", "ASSAM"]);
    /// // Janmastomi is kept by the Assamese month, not the lunar one.
    /// assert!(of("JANMASHTAMI").is_none());
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_assam() -> Vec<FestivalRule> {
        let central = FestivalRule::india_central();
        let mut rules: Vec<FestivalRule> = CENTRAL_KEPT
            .iter()
            .filter_map(|&key| find(&central, key))
            .map(|rule| FestivalRule {
                meta: STATE.kept(rule.meta),
                ..rule
            })
            .collect();
        let bengal = FestivalRule::india_west_bengal();
        rules.extend(
            BENGAL_KEPT
                .iter()
                .filter_map(|&key| find(&bengal, key))
                .map(|rule| FestivalRule {
                    source: format!(
                        "{NOTICE}, kept as West Bengal's list keeps it: {}",
                        rule.source
                    ),
                    meta: listed(Some(Tradition::Hindu)),
                    ..rule
                }),
        );
        rules.extend(STATE.by_print(&["PHULPATI", "MAHA_ASHTAMI", "KOJAGRAT_PURNIMA", "CHHATH"]));
        rules.extend(STATE.by_text(&[("YAMA_DWITIYA", false)]));
        rules.extend(
            ASSAM
                .iter()
                .map(|&(key, month, tithi, fit, what)| FestivalRule {
                    meta: listed(Some(Tradition::Hindu)),
                    ..fit.rule(
                        key,
                        Some(month),
                        tithi.into(),
                        InAdhika::Nija,
                        &format!("{NOTICE}: {what}; {ALMANAC}"),
                    )
                }),
        );
        rules
    }
}

impl FollowingRule {
    /// The Durga Puja's Vijaya Dashami, the day after its Navami, as West
    /// Bengal's list counts it.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let rules = FollowingRule::india_assam();
    /// assert_eq!(rules.len(), 1);
    /// assert_eq!((rules[0].key.as_str(), rules[0].after.as_str(), rules[0].days), ("VIJAYA_DASHAMI", "MAHA_NAVAMI", 1));
    /// ```
    #[must_use]
    pub fn india_assam() -> Vec<FollowingRule> {
        vec![FollowingRule {
            key: "VIJAYA_DASHAMI".to_owned(),
            source: format!(
                "{NOTICE}: Durga Puja & Vijaya Dashomi, the day after the Puja's Navami in every year listed"
            ),
            after: "MAHA_NAVAMI".to_owned(),
            days: 1,
            meta: listed(Some(Tradition::Hindu)),
        }]
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, the state's days
    /// on a Gregorian date, the Bihus and Sati Sadhani Divas on days of
    /// the Assamese month, Gwthar Bathou San on Magh's second Tuesday, and
    /// the Vaishnava saints' days and Janmastomi on a tithi of an Assamese
    /// month.
    ///
    /// ```
    /// use teistro_core::catalogue::{Calendar, Tithi};
    /// use teistro_panchanga::festival::{Dated, DatedRule, Which, Window};
    ///
    /// let rules = DatedRule::india_assam();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key).map(|rule| rule.on);
    /// assert_eq!(
    ///     of("KATI_BIHU"),
    ///     Some(Dated::CivilDate { calendar: Calendar::AssamSolar, month: 6, day: -1, skipped: None })
    /// );
    /// assert_eq!(
    ///     of("SANKARDEVA_JANMOTSAV"),
    ///     Some(Dated::TithiInMonth {
    ///         calendar: Calendar::AssamSolar,
    ///         month: 6,
    ///         tithi: Tithi::ShuklaDashami,
    ///         twice: Which::Earlier,
    ///         at: Window::Sunrise,
    ///         days: 0,
    ///     })
    /// );
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "ASSAM"]));
    /// ```
    #[must_use]
    pub fn india_assam() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        rules.extend(CIVIL.iter().map(|&(key, day, communities)| {
            let meta = FestivalMeta {
                communities: communities.iter().map(|&one| one.to_owned()).collect(),
                ..listed(None)
            };
            on_civil_date(key, day, NOTICE, meta)
        }));
        rules.extend(ON_DATE.iter().map(|&(key, month, day, what)| DatedRule {
            key: key.to_owned(),
            source: format!("{NOTICE}: {what}"),
            on: Dated::CivilDate {
                calendar: Calendar::AssamSolar,
                month,
                day,
                skipped: None,
            },
            meta: listed(Some(Tradition::Hindu)),
        }));
        rules.push(DatedRule {
            key: "GWTHAR_BATHOU_SAN".to_owned(),
            source: format!(
                "{NOTICE}: Gwthar Bathou San (Bathou Puja), a Tuesday in late January; Magh's second Tuesday in every year listed"
            ),
            on: Dated::CivilWeekday {
                calendar: Calendar::AssamSolar,
                month: 10,
                weekday: Vara::Mangalavara,
                nth: 2,
            },
            meta: listed(None),
        });
        rules.extend(
            IN_MONTH
                .iter()
                .map(|&(key, month, tithi, at, what)| DatedRule {
                    key: key.to_owned(),
                    source: format!("{NOTICE}: {what}; {ALMANAC}"),
                    on: Dated::TithiInMonth {
                        calendar: Calendar::AssamSolar,
                        month,
                        tithi,
                        twice: Which::Earlier,
                        at,
                        days: 0,
                    },
                    meta: listed(Some(Tradition::Hindu)),
                }),
        );
        rules
    }
}

impl PublishedSet {
    /// The days the notifications print rather than reckon, for the years
    /// a notification is in hand: Id-ul-Fitr and Id-ul-Zuha, and the half
    /// holiday of Muharram in the years it is listed, each marked subject
    /// to the sighting.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_assam();
    /// assert_eq!(set.entries.len(), 2 * 8 + 6);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2019, 6, 5));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// ```
    #[must_use]
    pub fn india_assam() -> PublishedSet {
        let printed = PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic)),
            cite: |notice| {
                format!(
                    "Government of Assam, General Administration Department, notification {notice}, its day subject to the appearance of the moon"
                )
            },
        };
        let mut entries = printed.entries(&sighted(&SIGHTED_HERE));
        let muharram: Vec<Named<'_>> = SIGHTED
            .iter()
            .filter(|row| row.0 == "MUHARRAM")
            .map(|&(key, en, ne, _)| (key, en, ne))
            .collect();
        entries.extend(printed.on_days(&muharram, &MUHARRAM_DAYS));
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of Assam, General Administration Department".to_owned(),
            id: "in-as-gad-printed".to_owned(),
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
        for rule in FestivalRule::india_assam() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_assam() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::india_assam() {
            assert!(
                keys.contains(&rule.after),
                "{} counts from a rule",
                rule.key
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
    }

    #[test]
    fn every_day_of_the_list_is_gazetted_in_the_state() {
        let lunar = FestivalRule::india_assam()
            .into_iter()
            .map(|rule| rule.meta);
        let dated = DatedRule::india_assam().into_iter().map(|rule| rule.meta);
        let counted = FollowingRule::india_assam()
            .into_iter()
            .map(|rule| rule.meta);
        let metas: Vec<FestivalMeta> = lunar.chain(dated).chain(counted).collect();
        assert_eq!(metas.len(), 12 + 25 + 1);
        for meta in &metas {
            assert_eq!(meta.kind, Some(FestivalKind::Gazetted));
            assert_eq!(meta.regions, REGIONS);
            assert!(!meta.optional);
        }
    }

    #[test]
    fn the_saints_passing_is_judged_by_the_daylight() {
        for rule in DatedRule::india_assam() {
            if let Dated::TithiInMonth { at, .. } = rule.on {
                assert_eq!(
                    at == DAYLIGHT,
                    rule.key.ends_with("TIROBHAV") || rule.key == "DAMODARDEVA_TITHI",
                    "{}",
                    rule.key
                );
            }
        }
    }

    #[test]
    fn every_printed_day_is_in_a_noticed_year() {
        let set = PublishedSet::india_assam();
        assert_eq!(
            set.entries.len(),
            SIGHTED_HERE.len() * NOTIFICATIONS.len() + MUHARRAM_DAYS.len()
        );
        assert!(set.entries.iter().all(|entry| matches!(
            entry,
            PublishedEntry::Add {
                subject_to: Some(SubjectTo::MoonSighting),
                ..
            }
        )));
    }
}

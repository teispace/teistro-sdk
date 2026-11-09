//! Maharashtra's public holidays (`03-design/maharashtra-holidays.md`,
//! SR-03, the first state set): the days the General Administration
//! Department's yearly notification "PUBLIC HOLIDAYS" lists under s. 25 of
//! the Negotiable Instruments Act, the bank-only day it adds, and Bhaubeej,
//! which a second notification adds for the State Government's offices.
//!
//! The notification is rank 1 for the list. Every reckoned day reuses a
//! rule the SDK ships, and a day the central list also holds reuses
//! `INDIA_CENTRAL`'s rule, so the two packs cannot drift apart.
//! `festival-measured.md` §10 holds every rule to seven years of
//! notifications at Mumbai.
//!
//! Ramzan-Id, Bakri-Id, Moharum and Id-e-Milad are not reckoned: they ship
//! as each notification prints them, marked [`SubjectTo::MoonSighting`], as
//! the central list's do. The Parsi New Year (Shahenshahi) also ships as
//! printed, since the catalogue has no Shahenshahi calendar (C1243); its
//! day does not wait on the moon, so it is not marked.

use teistro_core::catalogue::Tradition;

use super::india_central::{Printed, PrintedDays, SIGHTED, gazetted, on_civil_date};
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{DatedRule, FestivalMeta, FestivalRule, FollowingRule, PublishedSet};

/// Where the list is kept: India, then the state the notice names.
const REGIONS: [&str; 2] = ["IN", "MAHARASHTRA"];

/// The notification the list and its fixed days are read from, the first
/// whose gazette page was read whole.
const NOTICE: &str = "Government of Maharashtra, General Administration Department, notification \
     PHD-1122/C.R.117/Desk-29 of 2 December 2022 (Maharashtra Government Gazette, Extraordinary \
     Part I), s. 25 of the Negotiable Instruments Act, 1881";

/// Each year's notification, and how much of it was read: the gazette or
/// a copy giving its number and table (rank 1), or a press list alone
/// (rank 3, C1244).
const NOTIFICATIONS: [(i32, &str); 7] = [
    (
        2020,
        "PHD. 1119/C.R.69/Desk-29 of 7 December 2019, rank 1 (a copy with its number and table)",
    ),
    (
        2021,
        "PHD. 1120/C.R.82/Desk-29 of 1 December 2020, rank 3 (a press list; the gazette not read)",
    ),
    (
        2022,
        "PHD. 1121/C.R.97/Desk-29 of 2 December 2021, rank 1 (a copy with its number and table)",
    ),
    (
        2023,
        "PHD-1122/C.R.117/Desk-29 of 2 December 2022, rank 1 (the gazette page)",
    ),
    (
        2024,
        "सार्वसु-1123/प्र.क्र.148/जपुक (कार्या-29) of 9 November 2023, Gazette Extraordinary No. 118, rank 1 (the Marathi gazette)",
    ),
    (
        2025,
        "of 4 December 2024, rank 3 (a press list; the gazette and its number not read)",
    ),
    (
        2026,
        "PHD-1125/C.R.199/Japuk (29) of December 2025, rank 3 (a press list; the gazette not read)",
    ),
];

/// The days set by the moon, as (month, day) in the order of
/// [`NOTIFICATIONS`]; each is named as the central list names it.
const SIGHTED_HERE: [(&str, &[(u8, u8)]); 4] = [
    (
        "EID_AL_FITR",
        &[(5, 25), (5, 13), (5, 3), (4, 22), (4, 11), (3, 31), (3, 21)],
    ),
    (
        "EID_AL_ADHA",
        &[(8, 1), (7, 21), (7, 10), (6, 28), (6, 17), (6, 7), (5, 28)],
    ),
    (
        "MUHARRAM",
        &[(8, 30), (8, 19), (8, 9), (7, 29), (7, 17), (7, 6), (6, 26)],
    ),
    (
        "MAWLID",
        &[
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

/// The Parsi New Year (Shahenshahi), as the notifications print it, in
/// the order of [`NOTIFICATIONS`]: a 365-day year, a day earlier every
/// four years (C1243).
const PARSI_NEW_YEAR: Printed<'static> = (
    "PARSI_NEW_YEAR",
    "Parsi New Year (Shahenshahi)",
    "पारसी नयाँ वर्ष (शहंशाही)",
    &[
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
    gazetted(tradition, &REGIONS, optional)
}

/// A rule of the central list's, kept in Maharashtra: the same rule and
/// values, the state's region, and on every year's list.
fn kept_here(meta: FestivalMeta) -> FestivalMeta {
    FestivalMeta {
        regions: REGIONS.iter().map(|&region| region.to_owned()).collect(),
        optional: false,
        ..meta
    }
}

impl FestivalRule {
    /// The lunar days of Maharashtra's public holidays: the central list's
    /// rules for the days both lists hold, *Dharmasindhu*'s for Bali
    /// Pratipada and Bhaubeej, and Nepal's printed panchanga's for Gudhi
    /// Padwa and Ganesh Chaturthi. Janmashtami is on the central list
    /// only. The Holika fire is here for Holi to count from; it is on no
    /// list.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let rules = FestivalRule::india_maharashtra();
    /// let central = FestivalRule::india_central();
    /// let of = |rules: &[FestivalRule], key: &str| rules.iter().find(|rule| rule.key == key).cloned();
    /// // A day both lists hold is the central list's rule, kept in the state.
    /// let shivaratri = of(&rules, "SHIVARATRI").ok_or("shipped")?;
    /// assert_eq!(shivaratri.at, of(&central, "SHIVARATRI").ok_or("shipped")?.at);
    /// assert_eq!(shivaratri.meta.regions, ["IN", "MAHARASHTRA"]);
    /// assert!(!shivaratri.meta.optional);
    /// // Bhaubeej is added by its own notification, not every year.
    /// assert!(of(&rules, "YAMA_DWITIYA").ok_or("shipped")?.meta.optional);
    /// assert!(of(&rules, "JANMASHTAMI").is_none());
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_maharashtra() -> Vec<FestivalRule> {
        let of =
            |key: &str, rules: &[FestivalRule]| rules.iter().find(|rule| rule.key == key).cloned();
        let hindu = Some(Tradition::Hindu);
        let central = FestivalRule::india_central()
            .into_iter()
            .filter(|rule| rule.key != "JANMASHTAMI")
            .map(|rule| FestivalRule {
                meta: kept_here(rule.meta),
                ..rule
            });
        let text = FestivalRule::dharmasindhu();
        let by_text = [("BALI_PRATIPADA", false), ("YAMA_DWITIYA", true)]
            .into_iter()
            .filter_map(|(key, optional)| {
                of(key, &text).map(|rule| FestivalRule {
                    meta: listed(hindu, optional).with_fast(key),
                    ..rule
                })
            });
        let print = FestivalRule::nepal_panchanga();
        let printed = ["SAMVATSARA_ARAMBHA", "GANESHA_CHATURTHI"]
            .into_iter()
            .filter_map(|key| {
                of(key, &print).map(|rule| FestivalRule {
                    meta: listed(hindu, false),
                    ..rule
                })
            });
        central.chain(by_text).chain(printed).collect()
    }
}

impl FollowingRule {
    /// Holi's second day, the central list's Holi: the day after the
    /// Holika fire.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let holi = &FollowingRule::india_maharashtra()[0];
    /// assert_eq!((holi.after.as_str(), holi.days), ("HOLIKA", 1));
    /// assert!(!holi.meta.optional && FollowingRule::india_central()[0].meta.optional);
    /// ```
    #[must_use]
    pub fn india_maharashtra() -> Vec<FollowingRule> {
        FollowingRule::india_central()
            .into_iter()
            .map(|rule| FollowingRule {
                meta: kept_here(rule.meta),
                ..rule
            })
            .collect()
    }
}

impl DatedRule {
    /// The holidays on a civil date: the central list's five, the state's
    /// three days (Chhatrapati Shivaji Maharaj's birthday, Dr Ambedkar's
    /// and Maharashtra Day), and the yearly closing of bank accounts, the
    /// notification's day for banks alone.
    ///
    /// ```
    /// use teistro_panchanga::festival::DatedRule;
    ///
    /// let rules = DatedRule::india_maharashtra();
    /// assert_eq!(rules.len(), 9);
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "MAHARASHTRA"]));
    /// let banks = rules.iter().find(|rule| rule.key == "BANK_ACCOUNTS_CLOSING").ok_or("shipped")?;
    /// assert_eq!(banks.meta.communities, ["BANKS"]);
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_maharashtra() -> Vec<DatedRule> {
        let central = DatedRule::india_central()
            .into_iter()
            .map(|rule| DatedRule {
                source: NOTICE.to_owned(),
                meta: kept_here(rule.meta),
                ..rule
            });
        let state = [
            ("CHHATRAPATI_SHIVAJI_JAYANTI", (2, 19), Vec::new()),
            ("AMBEDKAR_JAYANTI", (4, 14), Vec::new()),
            ("MAHARASHTRA_DAY", (5, 1), Vec::new()),
            ("BANK_ACCOUNTS_CLOSING", (4, 1), vec!["BANKS".to_owned()]),
        ]
        .into_iter()
        .map(|(key, day, communities)| {
            let meta = FestivalMeta {
                communities,
                ..listed(None, false)
            };
            on_civil_date(key, day, NOTICE, meta)
        });
        central.chain(state).collect()
    }
}

impl PublishedSet {
    /// The days the notifications print rather than reckon, for the years
    /// a notification is in hand: the four set by the moon, each marked
    /// subject to the sighting, and the Parsi New Year (Shahenshahi).
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_maharashtra();
    /// assert_eq!(set.entries.len(), 35);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2020, 5, 25));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// let PublishedEntry::Add { key, subject_to, .. } = &set.entries[34] else { panic!("an ADD") };
    /// assert_eq!((key.as_str(), *subject_to), ("PARSI_NEW_YEAR", None));
    /// ```
    #[must_use]
    pub fn india_maharashtra() -> PublishedSet {
        let sighted: Vec<Printed<'_>> = SIGHTED_HERE
            .iter()
            .filter_map(|(key, days)| {
                SIGHTED
                    .iter()
                    .find(|row| row.0 == *key)
                    .map(|&(key, en, ne, _)| (key, en, ne, *days))
            })
            .collect();
        let mut entries = PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic), false),
            cite: |notice| {
                format!(
                    "Government of Maharashtra, GAD notification {notice}, its day subject to the sighting of the moon"
                )
            },
        }
        .entries(&sighted);
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
                        "Government of Maharashtra, GAD notification {notice}, as printed: the catalogue has no Shahenshahi calendar (C1243)"
                    )
                },
            }
            .entries(&[PARSI_NEW_YEAR]),
        );
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of Maharashtra, General Administration Department".to_owned(),
            id: "in-mh-gad-printed".to_owned(),
            version: 1,
            entries,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use teistro_core::catalogue::FestivalKind;

    use super::super::PublishedEntry;
    use super::*;

    #[test]
    fn every_day_of_the_list_is_gazetted_in_the_state() {
        let lunar: Vec<FestivalMeta> = FestivalRule::india_maharashtra()
            .into_iter()
            .filter(|rule| rule.key != "HOLIKA")
            .map(|rule| rule.meta)
            .collect();
        let dated = DatedRule::india_maharashtra()
            .into_iter()
            .map(|rule| rule.meta);
        let metas: Vec<FestivalMeta> = lunar.into_iter().chain(dated).collect();
        assert_eq!(metas.len(), 11 + 9);
        for meta in &metas {
            assert_eq!(meta.kind, Some(FestivalKind::Gazetted));
            assert_eq!(meta.regions, REGIONS);
        }
    }

    #[test]
    fn a_day_both_lists_hold_is_the_central_rule() {
        let central = FestivalRule::india_central();
        for rule in FestivalRule::india_maharashtra() {
            if let Some(shared) = central.iter().find(|other| other.key == rule.key) {
                assert_eq!((&rule.at, &rule.source), (&shared.at, &shared.source));
                assert_eq!(rule.meta.fast, shared.meta.fast, "{}", rule.key);
            }
        }
    }

    #[test]
    fn a_day_set_by_the_moon_is_named_as_the_central_list_names_it() {
        let set = PublishedSet::india_maharashtra();
        assert_eq!(
            set.entries.len(),
            (SIGHTED_HERE.len() + 1) * NOTIFICATIONS.len()
        );
        let central = PublishedSet::india_central();
        let names = |set: &PublishedSet, wanted: &str| {
            set.entries.iter().find_map(|entry| match entry {
                PublishedEntry::Add { key, names, .. } if key == wanted => Some(names.clone()),
                _ => None,
            })
        };
        for (key, _) in SIGHTED_HERE {
            assert_eq!(names(&set, key), names(&central, key), "{key}");
        }
    }
}

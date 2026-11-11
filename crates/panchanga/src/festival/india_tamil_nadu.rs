//! Tamil Nadu's public holidays (`03-design/tamil-nadu-holidays.md`,
//! SR-03, the fourth state set): the days the Public (Miscellaneous)
//! Department's yearly order declares public holidays under s. 25 of the
//! Negotiable Instruments Act, its notification published in the Tamil
//! Nadu Government Gazette, with the annual closing of accounts the order
//! keeps for the banks alone.
//!
//! The order is rank 1 for the list. A day both this list and the central
//! list hold reuses `INDIA_CENTRAL`'s rule. Tamil Nadu's solar days are
//! kept by the Tamil month of the vakya school (`Calendar::TamilVakya`):
//! Pongal, Thiruvalluvar Day and Uzhavar Thirunal on Thai 1 to 3, Tamil
//! New Year on Chithirai 1, and Thai Poosam on Pushya in Thai (C1352).
//! The state reckons its lunar days by the vakya almanac too (C1350), so
//! the pack is held to the print under the `tamil-vakya` profile:
//! Vinayakar Chathurthi and Krishna Jayanthi meet every year there and
//! part under the modern sky. Ayutha Pooja is the day before Vijaya
//! Dasami in every year listed (C1353). `festival-measured.md` §13 holds
//! every rule to eight years of orders at Chennai, under both skies.
//!
//! Ramzan, Bakrid, Muharram and Milad-un-Nabi are not reckoned: their days
//! are the moon's, so they ship as each order prints them, marked
//! [`SubjectTo::MoonSighting`], as the central list's do.

use teistro_core::catalogue::{Calendar, Masa, Nakshatra, Tithi, Tradition};

use super::india_central::{PrintedDays, StateList, find, on_civil_date, sighted};
use super::nepal_panchanga::Fit;
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{
    Dated, DatedRule, FestivalMeta, FestivalRule, FollowingRule, InAdhika, PublishedSet, Which,
};

/// Where the list is kept: India, then the state the order names.
const REGIONS: [&str; 2] = ["IN", "IN-TN"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The central list's lunar rule this list keeps: Mahaveer Jayanthi.
const CENTRAL_KEPT: [&str; 1] = ["MAHAVIR_JAYANTI"];

/// The order the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of Tamil Nadu, Public (Miscellaneous) Department, G.O.(Ms.) \
     No. 708 of 11 November 2025, its notification under s. 25 of the Negotiable Instruments Act, \
     1881, published in the Tamil Nadu Government Gazette: public holidays for 2026";

/// Each year's order, and how it was read: the order itself (rank 1), a
/// copy giving its number and table (a staff site's, rank 1 as copied),
/// or a list with the gazette unread (rank 3, C1354).
const ORDERS: [(i32, &str); 8] = [
    (
        2019,
        "G.O.Ms. No. 840 of 29 October 2018, rank 1 as copied (a staff site's copy with its number and table)",
    ),
    (
        2020,
        "G.O.Ms. No. 762 of 22 October 2019, rank 1 as copied (a staff site's copy with its number and table)",
    ),
    (
        2021,
        "G.O.Ms. No. 554 of 27 October 2020, rank 1 as copied (a staff site's copy with its number and table); Thai Poosam added by a later order of January 2021",
    ),
    (
        2022,
        "G.O.(Ms.) No. 935 of 2 November 2021, Gazette Extraordinary Part II(1) No. 525, rank 3 (a list whose first rows match the gazette's; the gazette not read whole)",
    ),
    (
        2023,
        "G.O.(Ms.) No. 682 of 11 October 2022, rank 1 as copied, amended by G.O.(Ms.) No. 528 of 31 August 2023",
    ),
    (
        2024,
        "G.O.(Ms.) No. 692 of 9 November 2023, rank 1 as copied (a staff site's copy with its number and table)",
    ),
    (
        2025,
        "G.O.(Ms.) No. 792 of 22 November 2024, rank 1 (the signed order and its appendix)",
    ),
    (
        2026,
        "G.O.(Ms.) No. 708 of 11 November 2025, rank 1 (the signed order and its appendix)",
    ),
];

/// Tamil Nadu's convention for a lunar day, as a new rule cites it.
const ALMANAC: &str = "the vakya almanacs of Tamil Nadu, as the state's orders of 2019 to 2026 \
     keep the day; no almanac's page is read, so the rule is held to the orders and the citation \
     is open (C1350)";

/// The state's days of the Tamil vakya calendar: key, month (Chithirai
/// the 1st), day, tradition, and what the day is.
const SOLAR: [(&str, u8, i8, Option<Tradition>, &str); 4] = [
    (
        "THAI_PONGAL",
        10,
        1,
        Some(Tradition::Hindu),
        "Pongal, Thai 1",
    ),
    (
        "THIRUVALLUVAR_DAY",
        10,
        2,
        None,
        "Thiruvalluvar Day, Thai 2",
    ),
    ("UZHAVAR_THIRUNAL", 10, 3, None, "Uzhavar Thirunal, Thai 3"),
    (
        "PUTHANDU",
        1,
        1,
        Some(Tradition::Hindu),
        "Tamil New Year's Day, Chithirai 1, printed in one row with Dr. B. R. Ambedkar's birthday",
    ),
];

/// The state's days on a Gregorian date: key, (month, day) and
/// communities.
const CIVIL: [(&str, (u8, i8), &[&str]); 4] = [
    ("NEW_YEAR_S_DAY_AD", (1, 1), &[]),
    ("BANK_ACCOUNTS_CLOSING", (4, 1), &["BANKS"]),
    ("AMBEDKAR_JAYANTI", (4, 14), &[]),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1), &[]),
];

/// The days set by the moon, as (month, day) in the order of [`ORDERS`],
/// each named as the central list names it; Milad-un-Nabi is the central
/// list's Id-e-Milad.
const SIGHTED_HERE: [(&str, &[(u8, u8)]); 4] = [
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
            (5, 28),
        ],
    ),
    (
        "MUHARRAM",
        &[
            (9, 10),
            (8, 30),
            (8, 20),
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

/// The values of a day of the list.
fn listed(tradition: Option<Tradition>) -> FestivalMeta {
    STATE.listed(tradition, false)
}

impl FestivalRule {
    /// The lunar days of Tamil Nadu's public holidays: the central list's
    /// rule for Mahaveer Jayanthi, *Dharmasindhu*'s Janmashtami for Krishna
    /// Jayanthi, Nepal's printed panchanga's for the Telugu New Year,
    /// Vinayakar Chathurthi and Deepavali (Naraka Chaturdashi), and Tamil
    /// Nadu's own for Vijaya Dasami. Under the `tamil-vakya` profile each
    /// reads the vakya almanac's tithi (C1350).
    ///
    /// ```
    /// use teistro_panchanga::festival::{FestivalRule, Window};
    ///
    /// let rules = FestivalRule::india_tamil_nadu();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let dasami = of("VIJAYA_DASHAMI").ok_or("shipped")?;
    /// assert_eq!(dasami.at, Window::Sunrise);
    /// assert_eq!(dasami.meta.regions, ["IN", "IN-TN"]);
    /// // Deepavali is the Naraka Chaturdashi, not the central list's Diwali.
    /// assert!(of("NARAK_CHATURDASHI").is_some() && of("LAKSHMI_PUJA").is_none());
    /// assert!(of("JANMASHTAMI").ok_or("shipped")?.source.starts_with("Dharmasindhu"));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_tamil_nadu() -> Vec<FestivalRule> {
        let mut rules: Vec<FestivalRule> = CENTRAL_KEPT
            .iter()
            .filter_map(|&key| find(&FestivalRule::india_central(), key))
            .map(|rule| FestivalRule {
                meta: STATE.kept(rule.meta),
                ..rule
            })
            .collect();
        rules.extend(STATE.by_text(&[("JANMASHTAMI", false)]));
        rules.extend(STATE.by_print(&[
            "SAMVATSARA_ARAMBHA",
            "GANESHA_CHATURTHI",
            "NARAK_CHATURDASHI",
        ]));
        rules.push(FestivalRule {
            meta: listed(Some(Tradition::Hindu)),
            ..Fit::Udaya.rule(
                "VIJAYA_DASHAMI",
                Some(Masa::Ashwina),
                Tithi::ShuklaDashami.into(),
                InAdhika::Nija,
                &format!("Vijaya Dasami, Ashwina's bright 10th; {ALMANAC}"),
            )
        });
        rules
    }
}

impl FollowingRule {
    /// Ayutha Pooja, the day before Vijaya Dasami in every year listed
    /// (C1353).
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let ayutha = &FollowingRule::india_tamil_nadu()[0];
    /// assert_eq!((ayutha.key.as_str(), ayutha.after.as_str(), ayutha.days), ("AYUDHA_PUJA", "VIJAYA_DASHAMI", -1));
    /// ```
    #[must_use]
    pub fn india_tamil_nadu() -> Vec<FollowingRule> {
        vec![FollowingRule {
            key: "AYUDHA_PUJA".to_owned(),
            source: format!(
                "{NOTICE}: Ayutha Pooja, the day before Vijaya Dasami in every year of the orders of 2019 to 2026, though the navami at sunrise is not always that day (C1353)"
            ),
            after: "VIJAYA_DASHAMI".to_owned(),
            days: -1,
            meta: listed(Some(Tradition::Hindu)),
        }]
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, the state's days on
    /// a Gregorian date with the banks' closing of accounts, Pongal,
    /// Thiruvalluvar Day, Uzhavar Thirunal and Tamil New Year on the Tamil
    /// vakya calendar, and Thai Poosam on Pushya in Thai.
    ///
    /// ```
    /// use teistro_core::catalogue::{Calendar, Nakshatra};
    /// use teistro_panchanga::festival::{Dated, DatedRule, Which};
    ///
    /// let rules = DatedRule::india_tamil_nadu();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key).map(|rule| rule.on);
    /// assert_eq!(
    ///     of("THAI_PONGAL"),
    ///     Some(Dated::CivilDate { calendar: Calendar::TamilVakya, month: 10, day: 1, skipped: None })
    /// );
    /// assert_eq!(
    ///     of("THAI_POOSAM"),
    ///     Some(Dated::NakshatraInMonth {
    ///         calendar: Calendar::TamilVakya,
    ///         month: 10,
    ///         nakshatra: Nakshatra::Pushya,
    ///         twice: Which::Later,
    ///         days: 0,
    ///     })
    /// );
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-TN"]));
    /// ```
    #[must_use]
    pub fn india_tamil_nadu() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        rules.extend(CIVIL.iter().map(|&(key, day, communities)| {
            let meta = FestivalMeta {
                communities: communities.iter().map(|&one| one.to_owned()).collect(),
                ..listed(None)
            };
            on_civil_date(key, day, NOTICE, meta)
        }));
        rules.extend(
            SOLAR
                .iter()
                .map(|&(key, month, day, tradition, what)| DatedRule {
                    key: key.to_owned(),
                    source: format!(
                        "{NOTICE}: {what}, of the Tamil month as the vakya almanacs begin it; the modern sky's month parts in 2026, when Makara sankranti fell before Chennai's sunset on 14 January (C1351)"
                    ),
                    on: Dated::CivilDate {
                        calendar: Calendar::TamilVakya,
                        month,
                        day,
                        skipped: None,
                    },
                    meta: listed(tradition),
                }),
        );
        rules.push(DatedRule {
            key: "THAI_POOSAM".to_owned(),
            source: format!(
                "{NOTICE}: Thai Poosam, the star Poosam (Pushya) in Thai, a holiday from 2021; the star's day is the first whose sunrise it holds, the later of two in one month, the one joined to Thai's full moon, as 11 February 2025 on Thai 29 (C1352)"
            ),
            on: Dated::NakshatraInMonth {
                calendar: Calendar::TamilVakya,
                month: 10,
                nakshatra: Nakshatra::Pushya,
                twice: Which::Later,
                days: 0,
            },
            meta: listed(Some(Tradition::Hindu)),
        });
        rules
    }
}

impl PublishedSet {
    /// The days the orders print rather than reckon, for the years an
    /// order is in hand: the four set by the moon, each marked subject to
    /// the sighting.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_tamil_nadu();
    /// assert_eq!(set.entries.len(), 4 * 8);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2019, 6, 5));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// ```
    #[must_use]
    pub fn india_tamil_nadu() -> PublishedSet {
        let entries = PrintedDays {
            notices: &ORDERS,
            declared: &[],
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic)),
            cite: |order| {
                format!(
                    "Government of Tamil Nadu, Public (Miscellaneous) Department, {order}, as printed; the day is the moon's"
                )
            },
        }
        .entries(&sighted(&SIGHTED_HERE));
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of Tamil Nadu, Public (Miscellaneous) Department".to_owned(),
            id: "in-tn-public-printed".to_owned(),
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

    #[test]
    fn every_rule_checks_and_keys_are_unique() {
        let mut keys = HashSet::new();
        for rule in FestivalRule::india_tamil_nadu() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_tamil_nadu() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::india_tamil_nadu() {
            assert!(
                keys.contains(&rule.after),
                "{} counts from a rule here",
                rule.key
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
    }

    #[test]
    fn every_day_of_the_list_is_gazetted_in_the_state() {
        let lunar = FestivalRule::india_tamil_nadu()
            .into_iter()
            .map(|rule| rule.meta);
        let dated = DatedRule::india_tamil_nadu()
            .into_iter()
            .map(|rule| rule.meta);
        let counted = FollowingRule::india_tamil_nadu()
            .into_iter()
            .map(|rule| rule.meta);
        let metas: Vec<FestivalMeta> = lunar.chain(dated).chain(counted).collect();
        assert_eq!(metas.len(), 6 + 14 + 1);
        for meta in &metas {
            assert_eq!(meta.kind, Some(FestivalKind::Gazetted));
            assert_eq!(meta.regions, REGIONS);
            assert!(!meta.optional);
        }
    }

    #[test]
    fn every_printed_day_is_in_an_ordered_year() {
        let set = PublishedSet::india_tamil_nadu();
        assert_eq!(set.entries.len(), SIGHTED_HERE.len() * ORDERS.len());
        assert!(SIGHTED_HERE.iter().all(|row| row.1.len() == ORDERS.len()));
    }
}

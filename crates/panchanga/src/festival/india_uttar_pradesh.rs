//! Uttar Pradesh's public holidays (`03-design/uttar-pradesh-holidays.md`,
//! SR-03, the tenth state set): the days the General Administration
//! Section's yearly notification (विज्ञप्ति) lists in its list of public
//! holidays, and those of its list of restricted holidays (निर्बन्धित
//! अवकाश), from which an employee chooses, marked
//! [`FestivalMeta::optional`].
//!
//! The notification is rank 1 for the list. A day both this list and the
//! central list hold reuses `INDIA_CENTRAL`'s rule, a day *Dharmasindhu*,
//! Nepal's printed panchanga or another state's list reckons reuses that
//! rule, and the state's one new lunar rule is Guru Ravidas Jayanti. The
//! notification prints each day's Vikram Samvat tithi by the purnimanta
//! month, as "भाद्रपद कृष्ण 08" for Janmashtami; the rules name the amanta
//! month, Shravana's dark 8th, as every shipped rule does. Govardhan Puja is
//! *Dharmasindhu*'s Bali pratipada, whose clause names it, Agrasen Jayanti
//! the Navaratra's first day, Valmiki Jayanti Ashvina's full moon as
//! Karnataka's list keeps it, and Makar Sankranti the first of Makara of
//! `TAMIL_SOLAR`, the day of the ingress when it falls before sunset and the
//! next day otherwise (C1522). `festival-measured.md` §20 holds every rule
//! to the notifications at Lucknow.
//!
//! The days the notes mark with an asterisk, "to be kept by the local
//! sighting of the moon", are not reckoned: they ship as printed, marked
//! [`SubjectTo::MoonSighting`]. The banks' closing of accounts, the
//! holidays under executive orders and a district's local holidays are not
//! here (C1527), nor are the orders that moved or added a day after the
//! notification (C1526).

use teistro_calendar::easter::Computus;
use teistro_core::catalogue::{Calendar, Masa, Tithi, Tradition};

use super::india_central::{Civil, Own, Printed, PrintedDays, StateList, sighted};
use super::nepal_panchanga::Fit;
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{Dated, DatedRule, FestivalMeta, FestivalRule, FollowingRule, PublishedSet};

/// Where the list is kept: India, then the state the notification names.
const REGIONS: [&str; 2] = ["IN", "IN-UP"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The notification the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of Uttar Pradesh, General Administration Section, notification \
     No. 671/तीन-2025-39(2)/2016 of 17 November 2025 (Uttar Pradesh Gazette, Extraordinary, \
     17 November 2025): para 2's lists of public and restricted holidays for 2026";

/// Each year's notification, and the copy read.
const NOTIFICATIONS: [(i32, &str); 8] = [
    (
        2019,
        "No. 12/2018/846/तीन-2018-39(2)/2016 of 27 November 2018, rank 1 as copied (the electronic issue, as the state police's site keeps it)",
    ),
    (
        2020,
        "No. 680/तीन-2019-39(2)/2016 of 4 November 2019, rank 1 as copied (the electronic issue, as the state police's site keeps it)",
    ),
    (
        2021,
        "No. 19/2020/912/तीन-2020-39(2)/2016 of 11 December 2020, rank 1 as copied (the electronic issue, as the state police's site keeps it)",
    ),
    (
        2022,
        "No. 10/2021/514/तीन-2021-39(2)/2016 of 20 October 2021, rank 1 as copied (the electronic issue, as an HR firm republished it)",
    ),
    (
        2023,
        "No. 508/तीन-2022-39(2)/2016 of 15 October 2022, rank 1 as copied (the signed scan)",
    ),
    (
        2024,
        "No. 528/तीन-2023-39(2)/2016 of 4 December 2023, rank 1 as copied (the signed scan)",
    ),
    (
        2025,
        "No. 870/तीन-2024-39(2)/2016 of 17 December 2024, rank 1 as copied (the signed scan)",
    ),
    (
        2026,
        "No. 671/तीन-2025-39(2)/2016 of 17 November 2025, rank 1 as copied (the Uttar Pradesh Gazette, Extraordinary, 17 November 2025)",
    ),
];

/// Uttar Pradesh's convention for a lunar day, as its new rule cites it.
const ALMANAC: &str = "Uttar Pradesh's almanacs, as the state's notifications of 2019 to 2026 \
     keep the day; no almanac's page is read, so the rule is held to the notifications and \
     the citation is open (C1521)";

/// Uttar Pradesh's rules by the tithi of an amanta month.
const UTTAR_PRADESH: [Own; 1] = [(
    "RAVIDAS_JAYANTI",
    Masa::Magha,
    Tithi::Purnima,
    Fit::Udaya,
    true,
    "the restricted list's Sant Ravidas Jayanti, Magha's full moon (माघ शुक्ल 15), kept at sunrise",
)];

/// Nepal's rules this list keeps, each with whether it is restricted:
/// Raksha Bandhan and Bhaiyadooj, which meet every notification where
/// *Dharmasindhu*'s rules part from two and one (C1528).
const NEPAL_KEPT: [(&str, &str, bool); 2] = [
    ("RAKSHABANDHAN", "RAKSHABANDHAN", false),
    ("YAMA_DWITIYA", "YAMA_DWITIYA", false),
];

/// The rules of the central list this list keeps: every lunar one but the
/// Holika fire, which the list holds as a public holiday of its own, and
/// Dussehra and Deepawali, which it keeps as Telangana's list does (C1528).
const CENTRAL_LEFT: [&str; 3] = ["HOLIKA", "VIJAYA_DASHAMI", "LAKSHMI_PUJA"];

/// *Dharmasindhu*'s rules this list keeps, each with whether the list
/// holds it restricted: the Holika fire.
const BY_TEXT: [(&str, bool); 1] = [("HOLIKA", false)];

/// *Dharmasindhu*'s rules this list keeps under a key of its own, each
/// with the text's key, whether it is restricted, and the day as the
/// notification names it.
const TEXT_KEPT: [(&str, &str, bool, &str); 2] = [
    (
        "GOVARDHAN_PUJA",
        "BALI_PRATIPADA",
        false,
        "Govardhan Puja, Kartika's bright 1st, which Dharmasindhu p. 78's Bali pratipada names with the Govardhan puja (C1523)",
    ),
    (
        "AGRASEN_JAYANTI",
        "NAVARATRA_ARAMBHA",
        true,
        "the restricted list's Maharaja Agrasen Jayanti, Ashvina's bright 1st (आश्विन शुक्ल 01), the Navaratra's first day",
    ),
];

/// Nepal's printed panchanga's rules this list keeps, by the key it keeps
/// them under, each with the print's key and whether it is restricted:
/// the Dussehra's Mahanavami, which meets more of the notifications than
/// *Dharmasindhu*'s (C1528), and Chetichand
/// on Chaitra's bright 1st at sunrise, as Gujarat's list keeps it (C1446).
const PRINT_KEPT: [(&str, &str, bool); 7] = [
    ("MAHA_NAVAMI", "MAHA_NAVAMI", false),
    ("BASANTA_PANCHAMI", "BASANTA_PANCHAMI", true),
    ("CHETI_CHAND", "SAMVATSARA_ARAMBHA", true),
    ("PARASHURAMA_JAYANTI", "PARASHURAMA_JAYANTI", true),
    ("ANANTA_CHATURDASHI", "ANANTA_CHATURDASHI", true),
    ("MAHA_ASHTAMI", "MAHA_ASHTAMI", true),
    ("CHHATH", "CHHATH", true),
];

/// The state's days on a Gregorian date: Dr Ambedkar's birthday a public
/// holiday, and the restricted list's New Year's Day, its leaders' and
/// heroes' days, Vishwakarma Puja and Christmas Eve.
const CIVIL: [Civil; 11] = [
    ("AMBEDKAR_JAYANTI", (4, 14), None, &[], false),
    ("NEW_YEAR_S_DAY_AD", (1, 1), None, &[], true),
    ("KARPOORI_THAKUR_JAYANTI", (1, 24), None, &[], true),
    (
        "KASHYAP_NISHADRAJ_JAYANTI",
        (4, 5),
        Some(Tradition::Hindu),
        &[],
        true,
    ),
    ("CHANDRASHEKHAR_JAYANTI", (4, 17), None, &[], true),
    ("MAHARANA_PRATAP_JAYANTI", (5, 9), None, &[], true),
    (
        "VISHWAKARMA_PUJA",
        (9, 17),
        Some(Tradition::Hindu),
        &[],
        true,
    ),
    ("SARDAR_PATEL_JAYANTI", (10, 31), None, &[], true),
    ("UDA_DEVI_SHAHEED_DIWAS", (11, 16), None, &[], true),
    ("CHARAN_SINGH_JAYANTI", (12, 23), None, &[], true),
    (
        "CHRISTMAS_EVE",
        (12, 24),
        Some(Tradition::Christian),
        &[],
        true,
    ),
];

/// The public days set by the moon, as (month, day) in the order of
/// [`NOTIFICATIONS`], each named as the central list names it.
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
            (5, 27),
        ],
    ),
    (
        "MUHARRAM",
        &[
            (9, 10),
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

/// Hazrat Ali's birthday, a public holiday the moon sets.
const HAZRAT_ALI: Printed<'static> = (
    "HAZRAT_ALI_BIRTHDAY",
    "Birthday of Hazrat Ali",
    "हजरत अलीको जन्मदिन",
    &[
        (3, 21),
        (3, 9),
        (2, 26),
        (2, 15),
        (2, 5),
        (1, 25),
        (1, 14),
        (1, 3),
    ],
);

/// The restricted days the moon sets, each in every year's list: the
/// Urs of Khwaja Moinuddin Chishti, Shab-e-Barat, Jamat-ul-Vida, the
/// restricted days beside Id-ul-Fitr, Id-uz-Zuha and Muharram, and
/// Chehlum.
const SIGHTED_RESTRICTED: [Printed<'static>; 7] = [
    (
        "KHWAJA_GARIB_NAWAZ_URS",
        "Urs of Hazrat Khwaja Moinuddin Chishti Ajmeri Garib Nawaz",
        "ख्वाजा मोइनुद्दीन चिश्ती गरीब नवाजको उर्स",
        &[
            (4, 14),
            (3, 3),
            (2, 19),
            (1, 27),
            (1, 29),
            (1, 18),
            (1, 8),
            (12, 16),
        ],
    ),
    (
        "SHAB_E_BARAT",
        "Shab-e-Barat",
        "शब-ए-बरात",
        &[
            (4, 21),
            (4, 9),
            (3, 29),
            (3, 19),
            (3, 8),
            (2, 26),
            (2, 14),
            (2, 4),
        ],
    ),
    (
        "JUMAT_UL_VIDA",
        "Jamat-ul-Vida (Alvida), the last Friday of Ramzan",
        "जुमात-उल-विदा",
        &[
            (5, 31),
            (5, 22),
            (5, 7),
            (4, 29),
            (4, 21),
            (4, 5),
            (3, 28),
            (3, 13),
        ],
    ),
    (
        "EID_AL_FITR_NEXT_DAY",
        "Id-ul-Fitr (restricted)",
        "ईद उल फित्र (निर्बन्धित)",
        &[
            (6, 6),
            (5, 26),
            (5, 15),
            (5, 4),
            (4, 23),
            (4, 12),
            (4, 1),
            (3, 22),
        ],
    ),
    (
        "EID_AL_ADHA_NEXT_DAY",
        "Id-uz-Zuha (Bakrid) (restricted)",
        "बकर ईद (निर्बन्धित)",
        &[
            (8, 13),
            (8, 2),
            (7, 22),
            (7, 11),
            (6, 30),
            (6, 18),
            (6, 8),
            (5, 28),
        ],
    ),
    (
        "MUHARRAM_SECOND_DAY",
        "Muharram (restricted)",
        "मोहर्रम (निर्बन्धित)",
        &[
            (9, 11),
            (8, 31),
            (8, 20),
            (8, 10),
            (7, 30),
            (7, 18),
            (7, 7),
            (6, 25),
        ],
    ),
    (
        "ARBAEEN",
        "Chehlum",
        "चेहल्लुम",
        &[
            (10, 19),
            (10, 8),
            (9, 28),
            (9, 17),
            (9, 6),
            (8, 25),
            (8, 14),
            (8, 4),
        ],
    ),
];

/// The values of a day of the list, restricted or not.
fn listed(tradition: Option<Tradition>, optional: bool) -> FestivalMeta {
    STATE.listed(tradition, optional)
}

impl FestivalRule {
    /// The lunar days of Uttar Pradesh's public and restricted holidays:
    /// the central list's rules for every lunar day it holds but the
    /// Holika fire and Janmashtami; *Dharmasindhu*'s for the Holika fire,
    /// Raksha Bandhan, Janmashtami, Mahanavami, Bhaiyadooj, Govardhan Puja
    /// (its Bali pratipada) and Agrasen Jayanti (the Navaratra's first
    /// day); Nepal's printed panchanga's for Basant Panchami, Chetichand,
    /// Parshuram Jayanti, Anant Chaturdashi, Mahashtami, Narak Chaturdashi
    /// and Chhath; Karnataka's Valmiki Jayanti; and the state's own Guru
    /// Ravidas Jayanti.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let rules = FestivalRule::india_uttar_pradesh();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let holika = of("HOLIKA").ok_or("shipped")?;
    /// assert_eq!(holika.meta.regions, ["IN", "IN-UP"]);
    /// assert!(!holika.meta.optional);
    /// // Govardhan Puja is the text's Bali pratipada, Chhath the print's.
    /// assert!(of("GOVARDHAN_PUJA").ok_or("shipped")?.source.contains("Dharmasindhu p. 78"));
    /// let chhath = of("CHHATH").ok_or("shipped")?;
    /// assert!(chhath.source.starts_with("Nepal's national panchanga") && chhath.meta.optional);
    /// assert!(of("RAVIDAS_JAYANTI").ok_or("shipped")?.source.contains("C1521"));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_uttar_pradesh() -> Vec<FestivalRule> {
        let mut rules = STATE.central(&CENTRAL_LEFT);
        rules.extend(STATE.by_text(&BY_TEXT));
        let kept: Vec<(&str, &str, bool)> = TEXT_KEPT
            .iter()
            .map(|&(key, from, optional, _)| (key, from, optional))
            .collect();
        rules.extend(STATE.taken(
            &FestivalRule::dharmasindhu(),
            Tradition::Hindu,
            &kept,
            |key, rule| {
                let what = TEXT_KEPT
                    .iter()
                    .find(|row| row.0 == key)
                    .map_or("", |row| row.3);
                format!("{NOTICE}: {what}; {}", rule.source)
            },
        ));
        rules.extend(STATE.print_kept(&PRINT_KEPT));
        rules.extend(STATE.taken(
            &FestivalRule::nepal(),
            Tradition::Hindu,
            &NEPAL_KEPT,
            |_, rule| {
                format!(
                    "{NOTICE}: the public list's day; as Nepal's rule reckons it (C1528): {}",
                    rule.source
                )
            },
        ));
        rules.extend(STATE.taken(
            &FestivalRule::india_telangana(),
            Tradition::Hindu,
            &[
                ("VIJAYA_DASHAMI", "VIJAYA_DASHAMI", false),
                ("LAKSHMI_PUJA", "LAKSHMI_PUJA", false),
            ],
            |_, rule| {
                format!(
                    "{NOTICE}: the public list's day; as Telangana's list reckons it (C1528): {}",
                    rule.source
                )
            },
        ));
        rules.extend(STATE.taken(
            &FestivalRule::india_karnataka(),
            Tradition::Hindu,
            &[("VALMIKI_JAYANTI", "VALMIKI_JAYANTI", true)],
            |_, rule| {
                format!(
                    "{NOTICE}: the restricted list's Maharshi Valmiki Jayanti; as Karnataka's list reckons it: {}",
                    rule.source
                )
            },
        ));
        rules.extend(STATE.own(&UTTAR_PRADESH, |_, what| {
            format!("{NOTICE}: {what}; {ALMANAC}")
        }));
        rules
    }
}

impl FollowingRule {
    /// Holi, the day after the Holika fire, as the central list keeps it,
    /// and the restricted list's Holi, the day after that.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let rules = FollowingRule::india_uttar_pradesh();
    /// let counted: Vec<(&str, &str, i8, bool)> = rules
    ///     .iter()
    ///     .map(|rule| (rule.key.as_str(), rule.after.as_str(), rule.days, rule.meta.optional))
    ///     .collect();
    /// assert_eq!(
    ///     counted,
    ///     [
    ///         ("HOLI", "HOLIKA", 1, false),
    ///         ("NARAK_CHATURDASHI", "LAKSHMI_PUJA", -1, true),
    ///         ("HOLI_SECOND_DAY", "HOLIKA", 2, true),
    ///     ]
    /// );
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-UP"]));
    /// ```
    #[must_use]
    pub fn india_uttar_pradesh() -> Vec<FollowingRule> {
        let mut rules: Vec<FollowingRule> = FollowingRule::india_central()
            .into_iter()
            .map(|rule| FollowingRule {
                meta: STATE.kept(rule.meta),
                ..rule
            })
            .collect();
        rules.push(FollowingRule {
            key: "NARAK_CHATURDASHI".to_owned(),
            source: format!(
                "{NOTICE}: the restricted list's Narak Chaturdashi (कार्तिक कृष्ण 14), the day before Deepawali in six of the eight notifications (C1528)"
            ),
            after: "LAKSHMI_PUJA".to_owned(),
            days: -1,
            meta: listed(Some(Tradition::Hindu), true),
        });
        rules.push(FollowingRule {
            key: "HOLI_SECOND_DAY".to_owned(),
            source: format!(
                "{NOTICE}: the restricted list's Holi, the day after the public list's (चैत्र कृष्ण 02), two days after the Holika fire"
            ),
            after: "HOLIKA".to_owned(),
            days: 2,
            meta: listed(Some(Tradition::Hindu), true),
        });
        rules
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, Dr Ambedkar's
    /// birthday, the restricted list's days on a Gregorian date, Easter
    /// Saturday and Easter Monday by the Western computus, and Makar
    /// Sankranti on the first of Makara of `TAMIL_SOLAR`, by its sunset
    /// rule (C1522).
    ///
    /// ```
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_panchanga::festival::{Dated, DatedRule};
    ///
    /// let rules = DatedRule::india_uttar_pradesh();
    /// assert_eq!(rules.len(), 5 + 11 + 3);
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let sankranti = of("UTTARAYANA_PUNYAKALA").ok_or("shipped")?;
    /// assert_eq!(
    ///     sankranti.on,
    ///     Dated::CivilDate { calendar: Calendar::TamilSolar, month: 10, day: 1, skipped: None }
    /// );
    /// assert!(sankranti.meta.optional);
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-UP"]));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_uttar_pradesh() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        rules.extend(STATE.civil(&CIVIL, NOTICE));
        rules.extend(
            [
                (
                    -1,
                    "EASTER_SATURDAY",
                    "Easter Saturday, the day before Easter",
                ),
                (1, "EASTER_MONDAY", "Easter Monday, the day after Easter"),
            ]
            .map(|(days, key, what)| DatedRule {
                key: key.to_owned(),
                source: format!("{NOTICE}: the restricted list's {what}"),
                on: Dated::Easter {
                    computus: Computus::Gregorian,
                    days,
                },
                meta: listed(Some(Tradition::Christian), true),
            }),
        );
        rules.push(STATE.on_solar_day(
            "UTTARAYANA_PUNYAKALA",
            format!(
                "{NOTICE}: the restricted list's Makar Sankranti, the day of the Sun's entry into Makara when it falls before sunset and the next day when after, the first of Makara by TAMIL_SOLAR's sunset rule (C1522)"
            ),
            (Calendar::TamilSolar, 10, 1),
            true,
        ));
        rules
    }
}

impl PublishedSet {
    /// The days the notifications print rather than reckon: the five public
    /// days and the seven restricted ones the notes leave to the local
    /// sighting of the moon, each marked subject to it.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_uttar_pradesh();
    /// assert_eq!(set.entries.len(), 12 * 8);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2019, 3, 21));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// ```
    #[must_use]
    pub fn india_uttar_pradesh() -> PublishedSet {
        let moon = |optional: bool| PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic), optional),
            cite: |notice| {
                format!(
                    "Government of Uttar Pradesh, General Administration Section, notification {notice}, its day kept by the local sighting of the moon (the lists' note)"
                )
            },
        };
        let mut public = vec![HAZRAT_ALI];
        public.extend(sighted(&SIGHTED_HERE));
        let mut entries = moon(false).entries(&public);
        entries.extend(moon(true).entries(&SIGHTED_RESTRICTED));
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of Uttar Pradesh, General Administration Section".to_owned(),
            id: "in-up-gad-printed".to_owned(),
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
        for rule in FestivalRule::india_uttar_pradesh() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_uttar_pradesh() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::india_uttar_pradesh() {
            assert!(
                keys.contains(&rule.after),
                "{} counts from nothing",
                rule.key
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for entry in PublishedSet::india_uttar_pradesh().entries {
            if let PublishedEntry::Add { key, .. } = entry {
                assert!(!keys.contains(&key), "{key} is reckoned and printed");
            }
        }
    }

    #[test]
    fn every_day_of_the_list_is_gazetted_in_the_state() {
        let lunar = FestivalRule::india_uttar_pradesh()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let dated = DatedRule::india_uttar_pradesh()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let following = FollowingRule::india_uttar_pradesh()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let metas: Vec<(String, FestivalMeta)> = lunar.chain(dated).chain(following).collect();
        assert_eq!(metas.len(), 23 + 19 + 2);
        let optional = metas.iter().filter(|(_, meta)| meta.optional).count();
        assert_eq!(optional, 10 + 13 + 1);
        for (key, meta) in &metas {
            assert_eq!(meta.regions, REGIONS, "{key}");
            assert_eq!(meta.kind, Some(FestivalKind::Gazetted), "{key}");
        }
    }

    #[test]
    fn a_day_both_lists_hold_is_the_central_rule() {
        let central = FestivalRule::india_central();
        for rule in FestivalRule::india_uttar_pradesh() {
            if let Some(shared) = central
                .iter()
                .find(|other| other.key == rule.key && !CENTRAL_LEFT.contains(&rule.key.as_str()))
            {
                assert_eq!((&rule.at, &rule.source), (&shared.at, &shared.source));
            }
        }
    }

    #[test]
    fn every_printed_day_is_in_a_noticed_year_and_marked() {
        let set = PublishedSet::india_uttar_pradesh();
        let rows = 1 + SIGHTED_HERE.len() + SIGHTED_RESTRICTED.len();
        assert_eq!(set.entries.len(), rows * NOTIFICATIONS.len());
        let restricted = set
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry,
                    PublishedEntry::Add {
                        subject_to: Some(SubjectTo::MoonSighting),
                        meta,
                        ..
                    } if meta.optional
                )
            })
            .count();
        assert_eq!(restricted, SIGHTED_RESTRICTED.len() * NOTIFICATIONS.len());
    }
}

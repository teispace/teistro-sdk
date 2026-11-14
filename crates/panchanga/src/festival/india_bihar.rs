//! Bihar's public holidays (`03-design/bihar-holidays.md`, SR-03, the
//! twelfth state set): the days the General Administration Department's
//! yearly notification lists in its Schedule-1, the general holidays by
//! executive order, and its Schedule-3, the public holidays under the
//! Negotiable Instruments Act, 1881, and the restricted holidays of its
//! Schedule-2, of which an employee chooses three, marked
//! [`FestivalMeta::optional`].
//!
//! The notification is rank 1 for the list. A day both this list and the
//! central list hold reuses `INDIA_CENTRAL`'s rule, a day *Dharmasindhu*,
//! Nepal's printed panchanga, the Jain pack or another state's list
//! reckons reuses that rule, and the state's one new lunar rule is Janaki
//! Navami (C1561). Every lunar key was measured at Patna against the eight
//! notifications and the rule meeting most was kept (C1562). The months
//! are amanta: Janmashtami is Shravana's dark 8th and Jitiya Bhadrapada's,
//! which the notifications name in Bhadra and Ashvina by the purnimanta
//! reckoning. Holi and its second day count from the Holika fire, Kharna
//! and the morning arghya from Chhath, and the day after Vijaya Dashami
//! from Vijaya Dashami.
//!
//! The days the notes leave to the sighting of the moon, the Islamic days,
//! are not reckoned: they ship as printed, marked
//! [`SubjectTo::MoonSighting`] (C1563). The last Monday of Shravana, a
//! weekday of a lunisolar month no shipped form reckons, ships as printed
//! (C1564), as does Guru Nanak's birthday, listed in 2019 and 2020 only
//! (C1573). Schedule-4's bank closing, the election holidays and the orders
//! that moved a day after the notification are not here (C1569, C1572).

use teistro_core::catalogue::{Calendar, Masa, Tithi, Tradition};

use super::india_central::{Civil, Named, OnDay, Own, Printed, PrintedDays, StateList, sighted};
use super::nepal_panchanga::Fit;
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{DatedRule, FestivalMeta, FestivalRule, FollowingRule, PublishedSet};

/// Where the list is kept: India, then the state the notification names.
const REGIONS: [&str; 2] = ["IN", "IN-BR"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The notification the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of Bihar, General Administration Department, notification \
     No. 3/एच0-01/2025-15401 of 19 August 2025: Schedule-1's general holidays, Schedule-2's \
     restricted holidays and Schedule-3's public holidays under the Negotiable Instruments \
     Act, 1881, for 2026";

/// Each year's notification, and the copy read.
const NOTIFICATIONS: [(i32, &str); 8] = [
    (
        2019,
        "No. 3/एच0-02/2018-सा0प्र0 15543 of 29 November 2018, rank 1 as copied (the signed scan, as a compliance site republished it)",
    ),
    (
        2020,
        "No. 3/एच0-03/2019-13501 of 30 September 2019, rank 1 as copied (the signed scan, as a compliance site republished it)",
    ),
    (
        2021,
        "No. 3/एच0-02/2020-12008 of 16 December 2020, rank 1 as copied (the signed scan, as a compliance site republished it)",
    ),
    (
        2022,
        "No. 3/एच0-01/2021-11265 of 24 September 2021, rank 1 as copied (the signed scan, as a compliance site republished it)",
    ),
    (
        2023,
        "No. 3/एच0-01/2022-15471 of 31 August 2022, rank 1 as copied (the Bihar Gazette, Extraordinary, No. 653, as an HR firm republished it)",
    ),
    (
        2024,
        "No. 3/एच0-02/2023-16953 of 5 September 2023, rank 1 as copied (the Bihar Gazette, Extraordinary, No. 736 of 11 September 2023, as an HR firm republished it)",
    ),
    (
        2025,
        "No. 3/एच0-01/2024-15764 of 1 October 2024, rank 1 as copied (the Bihar Gazette, Extraordinary, No. 956 of 3 October 2024, as an HR firm republished it)",
    ),
    (
        2026,
        "No. 3/एच0-01/2025-15401 of 19 August 2025, rank 1 as copied (the signed scan, as an HR firm republished it)",
    ),
];

/// The almanac the notifications take their Hindu days from, as the new
/// rule cites it.
const ALMANAC: &str = "the list of festival days of the Government Sanskrit College, Kajipur, \
     Patna, from which the department takes the Hindu days (its letter No. 67/25 of \
     12 July 2025, as the amendment of 6 November 2026 cites it); the college's list is not \
     read, so the rule is held to the notifications and the citation is open (C1561)";

/// The rules another pack reckons that this list keeps: the key kept
/// under, the other pack's key, whether it is restricted, and the day as
/// the notification names it.
type Kept = (&'static str, &'static str, bool, &'static str);

/// The rules of the central list this list keeps: every lunar one but the
/// Holika fire, Janmashtami, Dussehra and Deepawali, which other packs'
/// rules meet more often (C1562), and Guru Nanak's birthday, listed in two
/// years only (C1573).
const CENTRAL_LEFT: [&str; 5] = [
    "HOLIKA",
    "JANMASHTAMI",
    "VIJAYA_DASHAMI",
    "LAKSHMI_PUJA",
    "GURU_NANAK_JAYANTI",
];

/// *Dharmasindhu*'s rules this list keeps, each with whether the list
/// holds it restricted.
const BY_TEXT: [(&str, bool); 6] = [
    ("HOLIKA", true),
    ("JANMASHTAMI", false),
    ("RAKSHABANDHAN", true),
    ("HARITALIKA", true),
    ("NAVARATRA_ARAMBHA", true),
    ("YAMA_DWITIYA", false),
];

/// *Dharmasindhu*'s Devi visarjana, kept as Vijaya Dashami.
const TEXT_KEPT: [Kept; 1] = [(
    "VIJAYA_DASHAMI",
    "DEVI_VISARJAN",
    false,
    "Vijaya Dashami, Ashvina's bright 10th, on Dharmasindhu's day of the Devi's visarjana (C1562)",
)];

/// Nepal's printed panchanga's rules this list keeps.
const PRINT_KEPT: [Kept; 7] = [
    (
        "BASANTA_PANCHAMI",
        "BASANTA_PANCHAMI",
        false,
        "Basant Panchami and Saraswati Puja, Magha's bright 5th",
    ),
    (
        "SAMRAT_ASHOK_ASHTAMI",
        "CHAITE_DASHAIN",
        false,
        "Samrat Ashok's birthday, Chaitra's bright 8th (चैत्र शुक्ल अष्टमी), the print's Chaite Dashain",
    ),
    (
        "KABIR_JAYANTI",
        "KABIR_JAYANTI",
        false,
        "Sant Kabir's birthday, Jyeshtha's full moon",
    ),
    (
        "PHULPATI",
        "PHULPATI",
        false,
        "the Durga Puja's Saptami, Ashvina's bright 7th",
    ),
    (
        "MAHA_ASHTAMI",
        "MAHA_ASHTAMI",
        false,
        "the Durga Puja's Maha Ashtami, Ashvina's bright 8th",
    ),
    (
        "JITIYA",
        "JITIYA",
        true,
        "Schedule-2's Jiutiya, the mothers' fast on Ashvina's dark 8th in the purnimanta naming, Bhadrapada's in the amanta (2025 and 2026)",
    ),
    (
        "CHHATH",
        "CHHATH",
        false,
        "Chhath Puja, the evening arghya on Kartika's bright 6th (C1568)",
    ),
];

/// Bihar's own rule by the tithi of an amanta month.
const BIHAR: [Own; 1] = [(
    "SITA_NAVAMI",
    Masa::Vaishakha,
    Tithi::ShuklaNavami,
    Fit::Udaya,
    false,
    "Janaki Navami, Vaishakha's bright 9th, kept at sunrise",
)];

/// The days counted from another: key, the rule, the count, whether it is
/// restricted, and how the notifications give it.
const COUNTED: [(&str, &str, i8, bool, &str); 4] = [
    (
        "HOLI_SECOND_DAY",
        "HOLIKA",
        2,
        false,
        "Holi's second day, two days after the Holika fire in every year listed",
    ),
    (
        "CHHATH_KHARNA",
        "CHHATH",
        -1,
        true,
        "Schedule-2's Kharna, the day before Chhath's evening arghya in every year listed",
    ),
    (
        "CHHATH_SECOND_DAY",
        "CHHATH",
        1,
        false,
        "Chhath's second day, the morning arghya, the day after the evening's in every year listed",
    ),
    (
        "VIJAYA_DASHAMI_NEXT_DAY",
        "VIJAYA_DASHAMI",
        1,
        true,
        "Schedule-2's Durga Puja (Ekadashi), the day after Vijaya Dashami in every year listed",
    ),
];

/// The state's days on a Gregorian date: Bihar Diwas, Dr Ambedkar's and
/// Veer Kunwar Singh's birthdays and May Day public holidays, and the
/// restricted list's New Year's Day, its leaders' birthdays, Vishwakarma
/// Puja and Christmas Eve.
const CIVIL: [Civil; 12] = [
    ("BIHAR_DIWAS", (3, 22), None, &[], false),
    ("AMBEDKAR_JAYANTI", (4, 14), None, &[], false),
    ("VEER_KUNWAR_SINGH_JAYANTI", (4, 23), None, &[], false),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1), None, &[], false),
    ("NEW_YEAR_S_DAY_AD", (1, 1), None, &[], true),
    ("KARPOORI_THAKUR_JAYANTI", (1, 24), None, &[], true),
    ("ANUGRAH_NARAYAN_SINHA_JAYANTI", (6, 18), None, &[], true),
    (
        "VISHWAKARMA_PUJA",
        (9, 17),
        Some(Tradition::Hindu),
        &[],
        true,
    ),
    ("JAYAPRAKASH_NARAYAN_JAYANTI", (10, 11), None, &[], true),
    ("SHRI_KRISHNA_SINHA_JAYANTI", (10, 21), None, &[], true),
    ("RAJENDRA_PRASAD_JAYANTI", (12, 3), None, &[], true),
    (
        "CHRISTMAS_EVE",
        (12, 24),
        Some(Tradition::Christian),
        &[],
        true,
    ),
];

/// The public days set by the moon the central list names, as (month,
/// day) in the order of [`NOTIFICATIONS`]. 2020's prints the Prophet's
/// birthday on "30 November", which its weekday and its Hijri date put on
/// 30 October (C1563).
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

/// The state's own public days set by the moon: Shab-e-Barat and Chehlum.
const SIGHTED_PUBLIC: [Printed<'static>; 2] = [
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
        "ARBAEEN",
        "Chehlum",
        "चेहल्लुम",
        &[
            (10, 20),
            (10, 8),
            (9, 28),
            (9, 17),
            (9, 6),
            (8, 25),
            (8, 15),
            (8, 4),
        ],
    ),
];

/// The restricted days the moon sets, each in every year's list:
/// Jamat-ul-Vida, and the days after Id-ul-Fitr, Id-uz-Zoha and Muharram.
const SIGHTED_RESTRICTED: [Printed<'static>; 4] = [
    (
        "JUMAT_UL_VIDA",
        "Jamat-ul-Vida, the last Friday of Ramzan",
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
        "Id-ul-Fitr (the second day)",
        "ईद उल फित्रको दोस्रो दिन",
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
        "Id-uz-Zoha (Bakrid) (the second day)",
        "बकर ईदको दोस्रो दिन",
        &[
            (8, 13),
            (8, 2),
            (7, 22),
            (7, 11),
            (6, 30),
            (6, 18),
            (6, 8),
            (5, 29),
        ],
    ),
    (
        "MUHARRAM_SECOND_DAY",
        "Muharram (the second day)",
        "मोहर्रमको दोस्रो दिन",
        &[
            (9, 11),
            (8, 31),
            (8, 20),
            (8, 10),
            (7, 30),
            (7, 18),
            (7, 7),
            (6, 27),
        ],
    ),
];

/// Schedule-2's last Monday of Shravana, the last Monday on or before
/// Shravana's full moon, which no shipped form reckons (C1564).
const SHRAVANI: [Printed<'static>; 1] = [(
    "LAST_SHRAVANI_SOMVAR",
    "Last Shravani Somvar",
    "श्रावणको अन्तिम सोमबार",
    &[
        (8, 12),
        (8, 3),
        (8, 16),
        (8, 8),
        (8, 28),
        (8, 19),
        (8, 4),
        (8, 24),
    ],
)];

/// Guru Nanak's birthday, a public holiday on the lists of 2019 and 2020
/// only (C1573).
const GURU_NANAK: [Named<'static>; 1] = [(
    "GURU_NANAK_JAYANTI",
    "Guru Nanak's birthday",
    "गुरु नानक जयन्ती",
)];

/// Each year's Guru Nanak's birthday.
const GURU_NANAK_DAYS: [OnDay<'static>; 2] = [
    (2019, "GURU_NANAK_JAYANTI", (11, 12)),
    (2020, "GURU_NANAK_JAYANTI", (11, 30)),
];

/// The values of a day of the list, restricted or not.
fn listed(tradition: Option<Tradition>, optional: bool) -> FestivalMeta {
    STATE.listed(tradition, optional)
}

/// The rules of `rules` each of `kept` names, kept on the list as days of
/// `tradition` and citing the notification, the day it names, and the
/// other list's source.
fn kept_from(
    rules: &[FestivalRule],
    tradition: Tradition,
    kept: &[Kept],
    whose: &str,
) -> Vec<FestivalRule> {
    let keys: Vec<(&str, &str, bool)> = kept
        .iter()
        .map(|&(key, from, optional, _)| (key, from, optional))
        .collect();
    STATE.taken(rules, tradition, &keys, |key, rule| {
        let what = kept.iter().find(|row| row.0 == key).map_or("", |row| row.3);
        format!("{NOTICE}: {what}; as {whose} reckons it: {}", rule.source)
    })
}

impl FestivalRule {
    /// The lunar days of Bihar's public and restricted holidays: the
    /// central list's rules for Maha Shivaratri, Rama Navami, Mahavir
    /// Jayanti and Buddha Purnima; *Dharmasindhu*'s for the Holika fire,
    /// Janmashtami, Raksha Bandhan, Hartalika Teej, the Navaratra's first
    /// day, Bhai Dooj and, as Vijaya Dashami, the Devi's
    /// visarjana; Nepal's printed panchanga's for Basant Panchami, Samrat
    /// Ashok's birthday, Kabir's birthday, the Durga Puja's Saptami and
    /// Ashtami, Jiutiya and Chhath; Uttar Pradesh's Ravidas Jayanti,
    /// Telangana's Mahanavami and Deepawali, the Jain pack's Anant Chaturdashi and
    /// Odisha's Guru Gobind Singh's birthday; and the state's own Janaki
    /// Navami.
    ///
    /// ```
    /// use teistro_core::catalogue::Tradition;
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let rules = FestivalRule::india_bihar();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let chhath = of("CHHATH").ok_or("shipped")?;
    /// assert_eq!(chhath.meta.regions, ["IN", "IN-BR"]);
    /// assert!(!chhath.meta.optional);
    /// assert!(chhath.source.contains("as Nepal's printed panchanga reckons it"));
    /// // Samrat Ashok's birthday is the print's Chaite Dashain.
    /// assert!(of("SAMRAT_ASHOK_ASHTAMI").ok_or("shipped")?.source.contains("Chaite Dashain"));
    /// assert!(of("HOLIKA").ok_or("shipped")?.meta.optional);
    /// let gurpurab = of("GURU_GOBIND_SINGH_JAYANTI").ok_or("shipped")?;
    /// assert_eq!(gurpurab.meta.tradition, Some(Tradition::Sikh));
    /// assert!(of("SITA_NAVAMI").ok_or("shipped")?.source.contains("C1561"));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_bihar() -> Vec<FestivalRule> {
        let mut rules = STATE.central(&CENTRAL_LEFT);
        rules.extend(STATE.by_text(&BY_TEXT));
        rules.extend(kept_from(
            &FestivalRule::dharmasindhu(),
            Tradition::Hindu,
            &TEXT_KEPT,
            "Dharmasindhu",
        ));
        rules.extend(kept_from(
            &FestivalRule::nepal_panchanga(),
            Tradition::Hindu,
            &PRINT_KEPT,
            "Nepal's printed panchanga",
        ));
        let measured = |key, what| -> [Kept; 1] { [(key, key, false, what)] };
        rules.extend(kept_from(
            &FestivalRule::india_uttar_pradesh(),
            Tradition::Hindu,
            &measured(
                "RAVIDAS_JAYANTI",
                "Sant Ravidas's birthday, Magha's full moon",
            ),
            "Uttar Pradesh's list",
        ));
        rules.extend(kept_from(
            &FestivalRule::india_telangana(),
            Tradition::Hindu,
            &[
                (
                    "MAHA_NAVAMI",
                    "MAHA_NAVAMI",
                    false,
                    "the Durga Puja's Navami, Ashvina's bright 9th, at midday; Dharmasindhu's rule meets as many notifications and breaks its fast after the Devi's visarjana, which this list keeps as Vijaya Dashami (C1562, C1567)",
                ),
                (
                    "LAKSHMI_PUJA",
                    "LAKSHMI_PUJA",
                    false,
                    "Deepawali, Ashvina's new moon (C1569)",
                ),
            ],
            "Telangana's list",
        ));
        rules.extend(kept_from(
            &FestivalRule::jain(),
            Tradition::Hindu,
            &[(
                "ANANTA_CHATURDASHI",
                "ANANTA_CHATURDASHI",
                true,
                "Schedule-2's Anant Chaturdashi, Bhadrapada's bright 14th (C1562)",
            )],
            "the Jain pack",
        ));
        rules.extend(kept_from(
            &FestivalRule::india_odisha(),
            Tradition::Sikh,
            &measured(
                "GURU_GOBIND_SINGH_JAYANTI",
                "Guru Gobind Singh's birthday, Pausha's bright 7th (C1565)",
            ),
            "Odisha's list",
        ));
        rules.extend(STATE.own(&BIHAR, |_, what| format!("{NOTICE}: {what}; {ALMANAC}")));
        rules
    }
}

impl FollowingRule {
    /// Holi, the day after the Holika fire, as the central list keeps it;
    /// Holi's second day; Kharna and Chhath's morning arghya, the days
    /// either side of Chhath's evening; and the restricted day after Vijaya
    /// Dashami.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let rules = FollowingRule::india_bihar();
    /// let counted: Vec<(&str, &str, i8, bool)> = rules
    ///     .iter()
    ///     .map(|rule| (rule.key.as_str(), rule.after.as_str(), rule.days, rule.meta.optional))
    ///     .collect();
    /// assert_eq!(
    ///     counted,
    ///     [
    ///         ("HOLI", "HOLIKA", 1, false),
    ///         ("HOLI_SECOND_DAY", "HOLIKA", 2, false),
    ///         ("CHHATH_KHARNA", "CHHATH", -1, true),
    ///         ("CHHATH_SECOND_DAY", "CHHATH", 1, false),
    ///         ("VIJAYA_DASHAMI_NEXT_DAY", "VIJAYA_DASHAMI", 1, true),
    ///     ]
    /// );
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-BR"]));
    /// ```
    #[must_use]
    pub fn india_bihar() -> Vec<FollowingRule> {
        let mut rules: Vec<FollowingRule> = FollowingRule::india_central()
            .into_iter()
            .map(|rule| FollowingRule {
                meta: STATE.kept(rule.meta),
                ..rule
            })
            .collect();
        rules.extend(
            COUNTED
                .iter()
                .map(|&(key, after, days, optional, how)| FollowingRule {
                    key: key.to_owned(),
                    source: format!("{NOTICE}: {how}"),
                    after: after.to_owned(),
                    days,
                    meta: listed(Some(Tradition::Hindu), optional),
                }),
        );
        rules
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, the state's days on
    /// a Gregorian date, and the restricted list's Makar Sankranti on the
    /// first of Makara of `TAMIL_SOLAR`, by its sunset rule, as Uttar
    /// Pradesh's list keeps it (C1566).
    ///
    /// ```
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_panchanga::festival::{Dated, DatedRule};
    ///
    /// let rules = DatedRule::india_bihar();
    /// assert_eq!(rules.len(), 5 + 12 + 1);
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let sankranti = of("UTTARAYANA_PUNYAKALA").ok_or("shipped")?;
    /// assert_eq!(
    ///     sankranti.on,
    ///     Dated::CivilDate { calendar: Calendar::TamilSolar, month: 10, day: 1, skipped: None }
    /// );
    /// assert!(sankranti.meta.optional);
    /// assert!(!of("BIHAR_DIWAS").ok_or("shipped")?.meta.optional);
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-BR"]));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_bihar() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        rules.extend(STATE.civil(&CIVIL, NOTICE));
        rules.push(STATE.on_solar_day(
            "UTTARAYANA_PUNYAKALA",
            format!(
                "{NOTICE}: the restricted list's Makar Sankranti, the day of the Sun's entry into Makara when it falls before sunset and the next day when after, the first of Makara by TAMIL_SOLAR's sunset rule (C1522, C1566)"
            ),
            (Calendar::TamilSolar, 10, 1),
            true,
        ));
        rules
    }
}

impl PublishedSet {
    /// The days the notifications print rather than reckon: the six public
    /// and four restricted Islamic days the note leaves to the sighting of
    /// the moon, each marked subject to it; and, unmarked, the restricted
    /// last Monday of Shravana and Guru Nanak's birthday of 2019 and 2020.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_bihar();
    /// assert_eq!(set.entries.len(), 10 * 8 + 8 + 2);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2019, 6, 5));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// ```
    #[must_use]
    pub fn india_bihar() -> PublishedSet {
        let printed = |tradition, optional, subject_to| PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to,
            meta: listed(tradition, optional),
            cite: |notice| {
                format!(
                    "Government of Bihar, General Administration Department, notification {notice}, as printed"
                )
            },
        };
        let islamic = Some(Tradition::Islamic);
        let moon = Some(SubjectTo::MoonSighting);
        let mut public = sighted(&SIGHTED_HERE);
        public.extend(SIGHTED_PUBLIC);
        let mut entries = printed(islamic, false, moon).entries(&public);
        entries.extend(printed(islamic, true, moon).entries(&SIGHTED_RESTRICTED));
        entries.extend(printed(Some(Tradition::Hindu), true, None).entries(&SHRAVANI));
        entries.extend(
            printed(Some(Tradition::Sikh), false, None).on_days(&GURU_NANAK, &GURU_NANAK_DAYS),
        );
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of Bihar, General Administration Department".to_owned(),
            id: "in-br-gad-printed".to_owned(),
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
        for rule in FestivalRule::india_bihar() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_bihar() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::india_bihar() {
            assert!(
                keys.contains(&rule.after),
                "{} counts from nothing",
                rule.key
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for entry in PublishedSet::india_bihar().entries {
            if let PublishedEntry::Add { key, .. } = entry {
                assert!(!keys.contains(&key), "{key} is reckoned and printed");
            }
        }
    }

    #[test]
    fn every_day_of_the_list_is_gazetted_in_the_state() {
        let lunar = FestivalRule::india_bihar()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let dated = DatedRule::india_bihar()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let following = FollowingRule::india_bihar()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let metas: Vec<(String, FestivalMeta)> = lunar.chain(dated).chain(following).collect();
        assert_eq!(metas.len(), 24 + 18 + 5);
        let optional = metas.iter().filter(|(_, meta)| meta.optional).count();
        assert_eq!(optional, 6 + 9 + 2);
        for (key, meta) in &metas {
            assert_eq!(meta.regions, REGIONS, "{key}");
            assert_eq!(meta.kind, Some(FestivalKind::Gazetted), "{key}");
        }
    }

    #[test]
    fn a_day_both_lists_hold_is_the_central_rule() {
        let central = FestivalRule::india_central();
        for rule in FestivalRule::india_bihar() {
            if let Some(shared) = central
                .iter()
                .find(|other| other.key == rule.key && !CENTRAL_LEFT.contains(&rule.key.as_str()))
            {
                assert_eq!((&rule.at, &rule.source), (&shared.at, &shared.source));
            }
        }
    }

    #[test]
    fn every_printed_day_is_in_a_noticed_year() {
        let set = PublishedSet::india_bihar();
        let rows = SIGHTED_HERE.len() + SIGHTED_PUBLIC.len() + SIGHTED_RESTRICTED.len();
        assert_eq!(
            set.entries.len(),
            (rows + SHRAVANI.len()) * NOTIFICATIONS.len() + GURU_NANAK_DAYS.len()
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
        assert_eq!(marked, rows * NOTIFICATIONS.len());
        let years: Vec<i32> = NOTIFICATIONS.iter().map(|row| row.0).collect();
        assert!(set.entries.iter().all(|entry| match entry {
            PublishedEntry::Add { day, .. } => years.contains(&day.year),
            _ => false,
        }));
    }
}

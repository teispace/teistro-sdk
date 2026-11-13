//! Odisha's public holidays (`03-design/odisha-holidays.md`, SR-03, the
//! eleventh state set): the days the Revenue and Disaster Management
//! Department's yearly notification lists in its Annexure-I as holidays
//! for the State Government's offices and its Revenue and Magisterial
//! Courts, those its Annexure-III names as falling on a Sunday or a second
//! or fourth Saturday, and the optional holidays of its Annexure-II, an
//! employee's choice of one.
//!
//! The notification is rank 1 for the list. A day both this list and the
//! central list hold reuses `INDIA_CENTRAL`'s rule, a day Nepal's printed
//! panchanga or *Dharmasindhu* keeps reuses that rule, and Dola Purnima
//! and Kali Puja reuse West Bengal's. The months are amanta: Sabitri
//! Amabasya is Vaishakha's new moon and Prathamastami Kartika's dark 8th,
//! which the Odia almanacs name in Jyeshtha and Margashirsha by the
//! purnimanta reckoning. Holi is the day after Dola Purnima, Bahuda Yatra
//! the ninth day of the Ratha Yatra, and Vijaya Dashami the day after the
//! Mahanavami.
//!
//! The sankranti days are kept on the civil day of the Sun's entry (C1481):
//! Maha Vishuva Sankranti on the entry into Ashwini, the first of Mesha;
//! Raja Sankranti and Makar Sankranti on the last day of the Bengal solar
//! calendar's Jyaistha and Poush, the civil day before the month the
//! entry begins; and Pahili Raja on the day before Raja Sankranti.
//!
//! Id-ul-Fitre, Id-ul-Zuha, Muharram, the Prophet's birthday and
//! Shab-e-Qadr are not reckoned: they ship as printed, unmarked, since no
//! notification leaves their day to the moon (C1494). The days of some
//! years only, Panchayati Raj Divas (2019 to 2025), Mahavir Jayanti (2019
//! to 2021) and 2026's day following Nuakhai, ship as printed too (C1493).

use teistro_calendar::easter::Computus;
use teistro_core::catalogue::{Calendar, Masa, Nakshatra, Tithi, Tradition};

use super::india_central::{Civil, Named, OnDay, Own, PrintedDays, StateList, sighted};
use super::india_telangana::SIGHTED_OPTIONAL;
use super::nepal_panchanga::Fit;
use super::published::PUBLISHED_FORMAT;
use super::{
    Dated, DatedRule, DayReckoning, FestivalMeta, FestivalRule, FollowingRule, PublishedSet, Window,
};

/// Where the list is kept: India, then the state the notification names.
const REGIONS: [&str; 2] = ["IN", "IN-OD"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The notification the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of Odisha, Revenue and Disaster Management Department, \
     notification No. RDM-Reg-MISC-0021-2025-42428/R&D.M. of 11 November 2025: Annexure-I's \
     holidays, Annexure-II's optional holidays and Annexure-III's days on a Sunday or a second \
     or fourth Saturday for 2026";

/// Each year's notification, and the copy read.
const NOTIFICATIONS: [(i32, &str); 8] = [
    (
        2019,
        "No. RDM-Reg-MISC-0014-2018-40221/R&D.M. of 29 October 2018, rank 1 (the department's PDF, as archived)",
    ),
    (
        2020,
        "No. RDM-Reg-REGN-0050-2019-32550/R&D.M. of 23 October 2019, rank 1 (the department's PDF, as archived)",
    ),
    (
        2021,
        "No. RDM-Reg-REGN-0072-2020-30754/R&D.M. of 20 October 2020, rank 1 (the department's signed scan, as archived)",
    ),
    (
        2022,
        "No. RDM-Reg-REGN-0080-2021-34561/R&D.M. of 17 November 2021, rank 1 (the department's signed scan, as archived)",
    ),
    (
        2023,
        "No. RDM-Reg-MISC-0014-2022-37021/R&D.M. of 1 November 2022, rank 1 (the department's signed scan, as archived)",
    ),
    (
        2024,
        "No. RDM-Reg-MISC-0014-2023-39304/R&D.M. of 7 November 2023, rank 1 (the department's signed scan, as archived)",
    ),
    (
        2025,
        "No. RDM-Reg-MISC-0013-2024-39077/R&D.M. of 12 November 2024, rank 1 as copied (the signed scan a university enclosed in its notice)",
    ),
    (
        2026,
        "No. RDM-Reg-MISC-0021-2025-42428/R&D.M. of 11 November 2025, rank 1 as copied (the signed text, as a compliance site republished it)",
    ),
];

/// The Odia almanacs, as a new rule cites them.
const ALMANAC: &str = "the Odia panjikas each notification is copied to (the Mukti Mandap \
     Pandita Sabha, Puri; Kohinoor Press; Asali Khadiratna Panjeeka), as the state's \
     notifications of 2019 to 2026 keep the day; no panjika's page is read, so the rule is \
     held to the notifications and the citation is open (C1496)";

/// The central list's lunar rules this list keeps, each with whether
/// Odisha lists it as an optional holiday: Buddha Purnima, Pandit
/// Raghunath Murmu's birthday too, is general from 2022 (C1493).
const CENTRAL_KEPT: [(&str, bool); 3] = [
    ("SHIVARATRI", false),
    ("RAMA_NAVAMI", false),
    ("BUDDHA_JAYANTI", false),
];

/// The rules another pack reckons that this list keeps: the key kept
/// under, the other pack's key, whether it is optional, and the day as
/// the notification names it.
type Kept = (&'static str, &'static str, bool, &'static str);

/// Nepal's printed panchanga's rules this list keeps.
const PRINT_KEPT: [Kept; 10] = [
    (
        "BASANTA_PANCHAMI",
        "BASANTA_PANCHAMI",
        false,
        "Basanta Panchami, Magha's bright 5th (C1486)",
    ),
    (
        "JAGANNATH_RATH_JATRA",
        "JAGANNATH_RATH_JATRA",
        false,
        "Ratha Yatra, Ashadha's bright 2nd",
    ),
    (
        "GANESHA_CHATURTHI",
        "GANESHA_CHATURTHI",
        false,
        "Ganesh Puja, Bhadrapada's bright 4th",
    ),
    (
        "NUAKHAI",
        "RISHI_PANCHAMI",
        false,
        "Nuakhai, the new rice's festival, Bhadrapada's bright 5th, the print's Rishi Panchami (C1499)",
    ),
    (
        "MAHALAYA_AMAVASYA",
        "AMAVASYA_SHRADDHA",
        false,
        "Mahalaya, Bhadrapada's new moon, the print's last shraddha of the pitri paksha",
    ),
    (
        "PHULPATI",
        "PHULPATI",
        false,
        "the Durga Puja's Maha Saptami, Ashvina's bright 7th (C1484)",
    ),
    (
        "MAHA_ASHTAMI",
        "MAHA_ASHTAMI",
        false,
        "the Durga Puja's Maha Astami, Ashvina's bright 8th (C1484)",
    ),
    (
        "MAHA_NAVAMI",
        "MAHA_NAVAMI",
        false,
        "the Durga Puja's Maha Navami, Ashvina's bright 9th (C1484)",
    ),
    (
        "RAHAS_PURNIMA",
        "GURU_NANAK_JAYANTI",
        false,
        "Rahas Purnima, Kartika's full moon, the day the print keeps Guru Nanak's birthday",
    ),
    (
        "ANLA_NAVAMI",
        "KUSHMANDA_NAVAMI",
        true,
        "Annexure-II's Anla Navami, Kartika's bright 9th, the print's Kushmanda Navami (C1490)",
    ),
];

/// West Bengal's own rules this list keeps: Dola Purnima, Krishna's swing
/// festival on Phalguna's full moon, and Kali Puja, which the notifications
/// name with Diwali.
const BENGAL_KEPT: [Kept; 2] = [
    (
        "DOLA_YATRA",
        "DOLA_YATRA",
        false,
        "Dola Purnima, Phalguna's full moon",
    ),
    (
        "KALI_PUJA",
        "KALI_PUJA",
        false,
        "Kali Puja and Diwali, Ashvina's new moon (C1488)",
    ),
];

/// *Dharmasindhu*'s Raksha Bandhan, Shravana's full moon, kept as Jhulana
/// Purnima, the last day of Krishna's swing festival.
const TEXT_KEPT: [Kept; 1] = [(
    "JHULANA_PURNIMA",
    "RAKSHABANDHAN",
    false,
    "Jhulana Purnima, Shravana's full moon, on Dharmasindhu's Raksha Bandhan (C1489)",
)];

/// Odisha's own rules by the tithi of an amanta month.
const ODIA: [Own; 3] = [
    (
        "SAVITRI_AMAVASYA",
        Masa::Vaishakha,
        Tithi::Amavasya,
        Fit::Udaya,
        false,
        "Sabitri Amabasya, the wives' Savitri vrata on the new moon of Jyeshtha's dark half in the purnimanta naming, Vaishakha's in the amanta",
    ),
    (
        "KUMAR_PURNIMA",
        Masa::Ashwina,
        Tithi::Purnima,
        Fit::Share(Window::Moonrise),
        false,
        "Kumar Purnima, Ashvina's full moon, whose fast is broken with the offering to the rising moon (C1487)",
    ),
    (
        "PRATHAMASHTAMI",
        Masa::Kartika,
        Tithi::KrishnaAshtami,
        Fit::Udaya,
        true,
        "Annexure-II's Prathamastami, the first-born's rite on the dark 8th of Margashirsha in the purnimanta naming, Kartika's in the amanta",
    ),
];

/// Guru Gobind Singh's birthday, Pausha's bright 7th, an optional holiday.
const SIKH: [Own; 1] = [(
    "GURU_GOBIND_SINGH_JAYANTI",
    Masa::Pausha,
    Tithi::ShuklaSaptami,
    Fit::Udaya,
    true,
    "Annexure-II's Guru Gobind Singh's birthday, Pausha's bright 7th (C1491)",
)];

/// The days counted from another: key, the rule, the count, whether it is
/// optional, and how the notifications give it.
const COUNTED: [(&str, &str, i8, bool, &str); 3] = [
    (
        "HOLI",
        "DOLA_YATRA",
        1,
        false,
        "Annexure-I's Holi, the day after Dola Purnima in every year listed",
    ),
    (
        "BAHUDA_JATRA",
        "JAGANNATH_RATH_JATRA",
        8,
        true,
        "Annexure-II's Bahuda Yatra, the chariots' return from the Gundicha temple on the ninth day of the Ratha Yatra (C1497)",
    ),
    (
        "VIJAYA_DASHAMI",
        "MAHA_NAVAMI",
        1,
        false,
        "Annexure-I's Vijaya Dashami, the day after the Mahanavami in every year listed (C1484)",
    ),
];

/// The state's days on a Gregorian date. Subhas Chandra Bose's birthday is
/// Veer Surendra Sai's too; the Brahmo Samaj's foundation day moves to 20
/// August in 2026 (C1492).
const CIVIL: [Civil; 6] = [
    ("NETAJI_JAYANTI", (1, 23), None, &[], false),
    ("UTKAL_DIVAS", (4, 1), None, &[], false),
    ("AMBEDKAR_JAYANTI", (4, 14), None, &[], false),
    ("RAM_MOHAN_ROY_JAYANTI", (5, 22), None, &[], true),
    ("BRAHMO_SAMAJ_FOUNDATION_DAY", (8, 20), None, &[], true),
    (
        "CHRISTMAS_EVE",
        (12, 24),
        Some(Tradition::Christian),
        &[],
        true,
    ),
];

/// The sankranti days by the Bengal solar calendar, whose month begins on
/// the civil day after the Sun's entry: key, month, day from the month's
/// end, and the day as the notification names it.
const SANKRANTI: [(&str, u8, i8, &str); 3] = [
    (
        "PAHILI_RAJA",
        2,
        -2,
        "Pahili Raja, the first day of the Raja festival, the day before the Sun's entry into Mithuna (C1481, C1482)",
    ),
    (
        "MITHUNA_SANKRANTI",
        2,
        -1,
        "Raja Sankranti, the civil day of the Sun's entry into Mithuna, Jyaistha's last day in the Bengal solar calendar (C1481, C1482)",
    ),
    (
        "UTTARAYANA_PUNYAKALA",
        9,
        -1,
        "Makar Sankranti, the civil day of the Sun's entry into Makara, Poush's last day in the Bengal solar calendar (C1481)",
    ),
];

/// The days set by the moon, as (month, day) in the order of
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
            (6, 30),
            (6, 17),
            (6, 7),
            (5, 27),
        ],
    ),
    (
        "MUHARRAM",
        &[
            (9, 9),
            (8, 30),
            (8, 20),
            (8, 9),
            (7, 28),
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
            (9, 29),
            (9, 16),
            (9, 5),
            (8, 26),
        ],
    ),
];

/// Annexure-II's Shab-e-Qadr, an optional holiday: 2023's list holds none.
const SHAB_E_QADR_DAYS: [OnDay<'static>; 7] = [
    (2019, "SHAB_E_QADR", (6, 2)),
    (2020, "SHAB_E_QADR", (5, 20)),
    (2021, "SHAB_E_QADR", (5, 10)),
    (2022, "SHAB_E_QADR", (4, 27)),
    (2024, "SHAB_E_QADR", (4, 7)),
    (2025, "SHAB_E_QADR", (3, 28)),
    (2026, "SHAB_E_QADR", (3, 17)),
];

/// The days some years' lists hold and later ones drop or never repeat.
const SOME_YEARS: [Named<'static>; 2] = [
    (
        "PANCHAYATI_RAJ_DIVAS",
        "Panchayati Raj Divas",
        "पञ्चायती राज दिवस",
    ),
    (
        "NUAKHAI_NEXT_DAY",
        "Day following Nuakhai",
        "नुआखाईको भोलिपल्ट",
    ),
];

/// Each year's general days of [`SOME_YEARS`]: Panchayati Raj Divas, Biju
/// Patnaik's birthday, is on every list to 2025 and not on 2026's.
const SOME_YEARS_DAYS: [OnDay<'static>; 8] = [
    (2019, "PANCHAYATI_RAJ_DIVAS", (3, 5)),
    (2020, "PANCHAYATI_RAJ_DIVAS", (3, 5)),
    (2021, "PANCHAYATI_RAJ_DIVAS", (3, 5)),
    (2022, "PANCHAYATI_RAJ_DIVAS", (3, 5)),
    (2023, "PANCHAYATI_RAJ_DIVAS", (3, 5)),
    (2024, "PANCHAYATI_RAJ_DIVAS", (3, 5)),
    (2025, "PANCHAYATI_RAJ_DIVAS", (3, 5)),
    (2026, "NUAKHAI_NEXT_DAY", (9, 16)),
];

/// Mahavir Jayanti, an optional holiday to 2021, as printed: 2021's list
/// prints 14 April, eleven days before the bright 13th (C1493).
const MAHAVIR: [Named<'static>; 1] = [("MAHAVIR_JAYANTI", "Mahaveer Jayanti", "महावीर जयन्ती")];

/// Each year's Mahavir Jayanti.
const MAHAVIR_DAYS: [OnDay<'static>; 3] = [
    (2019, "MAHAVIR_JAYANTI", (4, 17)),
    (2020, "MAHAVIR_JAYANTI", (4, 6)),
    (2021, "MAHAVIR_JAYANTI", (4, 14)),
];

/// The values of a day of the list, optional or not.
fn listed(tradition: Option<Tradition>, optional: bool) -> FestivalMeta {
    STATE.listed(tradition, optional)
}

/// The rules of `rules` each of `kept` names, kept on the list and citing
/// the notification, the day it names, and the other list's source.
fn kept_from(rules: &[FestivalRule], kept: &[Kept], whose: &str) -> Vec<FestivalRule> {
    let keys: Vec<(&str, &str, bool)> = kept
        .iter()
        .map(|&(key, from, optional, _)| (key, from, optional))
        .collect();
    STATE.taken(rules, Tradition::Hindu, &keys, |key, rule| {
        let what = kept.iter().find(|row| row.0 == key).map_or("", |row| row.3);
        format!("{NOTICE}: {what}; as {whose} reckons it: {}", rule.source)
    })
}

impl FestivalRule {
    /// The lunar days of Odisha's public holidays: the central list's
    /// rules for Maha Shivaratri, Rama Navami and Buddha Purnima;
    /// *Dharmasindhu*'s for Janmastami and, as Jhulana Purnima, Raksha
    /// Bandhan; Nepal's printed panchanga's for Basanta Panchami, Ratha
    /// Yatra, Ganesh Puja, Nuakhai, Mahalaya, the Durga Puja's Saptami to
    /// Navami, Rahas Purnima and the optional Anla Navami; West Bengal's
    /// for Dola Purnima and Kali Puja; and Odisha's own for Sabitri
    /// Amabasya, Kumar Purnima, and the optional Prathamastami and Guru
    /// Gobind Singh's birthday.
    ///
    /// ```
    /// use teistro_core::catalogue::Tradition;
    /// use teistro_panchanga::festival::{FestivalRule, Window};
    ///
    /// let rules = FestivalRule::india_odisha();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let kumar = of("KUMAR_PURNIMA").ok_or("shipped")?;
    /// assert_eq!(kumar.at, Window::Moonrise);
    /// assert_eq!(kumar.meta.regions, ["IN", "IN-OD"]);
    /// // Nuakhai is the print's Rishi Panchami, Dola Purnima West Bengal's.
    /// assert!(of("NUAKHAI").ok_or("shipped")?.source.contains("as Nepal's printed panchanga reckons it"));
    /// assert!(of("DOLA_YATRA").ok_or("shipped")?.source.contains("as West Bengal's list reckons it"));
    /// let gurpurab = of("GURU_GOBIND_SINGH_JAYANTI").ok_or("shipped")?;
    /// assert_eq!(gurpurab.meta.tradition, Some(Tradition::Sikh));
    /// assert!(gurpurab.meta.optional);
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_odisha() -> Vec<FestivalRule> {
        let mut rules = STATE.central_kept(&CENTRAL_KEPT);
        rules.extend(STATE.by_text(&[("JANMASHTAMI", false)]));
        rules.extend(kept_from(
            &FestivalRule::dharmasindhu(),
            &TEXT_KEPT,
            "Dharmasindhu",
        ));
        rules.extend(kept_from(
            &FestivalRule::nepal_panchanga(),
            &PRINT_KEPT,
            "Nepal's printed panchanga",
        ));
        rules.extend(kept_from(
            &FestivalRule::india_west_bengal(),
            &BENGAL_KEPT,
            "West Bengal's list",
        ));
        let cite = |_: &str, what: &str| format!("{NOTICE}: {what}; {ALMANAC}");
        rules.extend(STATE.own(&ODIA, cite));
        rules.extend(STATE.own(&SIKH, cite).into_iter().map(|rule| FestivalRule {
            meta: listed(Some(Tradition::Sikh), rule.meta.optional),
            ..rule
        }));
        rules
    }
}

impl FollowingRule {
    /// The days the notifications count from another: Holi the day after
    /// Dola Purnima, the optional Bahuda Yatra the ninth day of the Ratha
    /// Yatra, and Vijaya Dashami the day after the Mahanavami.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let rules = FollowingRule::india_odisha();
    /// let counted: Vec<(&str, &str, i8)> =
    ///     rules.iter().map(|rule| (rule.key.as_str(), rule.after.as_str(), rule.days)).collect();
    /// assert_eq!(
    ///     counted,
    ///     [("HOLI", "DOLA_YATRA", 1), ("BAHUDA_JATRA", "JAGANNATH_RATH_JATRA", 8), ("VIJAYA_DASHAMI", "MAHA_NAVAMI", 1)]
    /// );
    /// assert!(rules[1].meta.optional && !rules[0].meta.optional);
    /// ```
    #[must_use]
    pub fn india_odisha() -> Vec<FollowingRule> {
        COUNTED
            .iter()
            .map(|&(key, after, days, optional, how)| FollowingRule {
                key: key.to_owned(),
                source: format!("{NOTICE}: {how}"),
                after: after.to_owned(),
                days,
                meta: listed(Some(Tradition::Hindu), optional),
            })
            .collect()
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, Easter Saturday,
    /// the state's days on a Gregorian date, Maha Vishuva Sankranti on the
    /// civil day of the Sun's entry into Mesha, and Pahili Raja, Raja
    /// Sankranti and Makar Sankranti by the Bengal solar calendar's month
    /// ends.
    ///
    /// ```
    /// use teistro_core::catalogue::{Calendar, Nakshatra};
    /// use teistro_panchanga::festival::{Dated, DatedRule, DayReckoning};
    ///
    /// let rules = DatedRule::india_odisha();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key).map(|rule| rule.on);
    /// assert_eq!(
    ///     of("MESHA_SANKRANTI"),
    ///     Some(Dated::SunEnters { nakshatra: Nakshatra::Ashwini, days: 0, reckoning: DayReckoning::Civil })
    /// );
    /// assert_eq!(
    ///     of("MITHUNA_SANKRANTI"),
    ///     Some(Dated::CivilDate { calendar: Calendar::BengalSolar, month: 2, day: -1, skipped: None })
    /// );
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-OD"]));
    /// ```
    #[must_use]
    pub fn india_odisha() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        rules.push(DatedRule {
            key: "EASTER_SATURDAY".to_owned(),
            source: format!("{NOTICE}: Annexure-II's Easter Saturday, the day before Easter"),
            on: Dated::Easter {
                computus: Computus::Gregorian,
                days: -1,
            },
            meta: listed(Some(Tradition::Christian), true),
        });
        rules.extend(STATE.civil(&CIVIL, NOTICE));
        rules.push(DatedRule {
            key: "MESHA_SANKRANTI".to_owned(),
            source: format!(
                "{NOTICE}: Maha Vishuva Sankranti (Pana Sankranti), the Odia new year; the civil day of the Sun's entry into Mesha, the first of Ashwini (C1481, C1483)"
            ),
            on: Dated::SunEnters {
                nakshatra: Nakshatra::Ashwini,
                days: 0,
                reckoning: DayReckoning::Civil,
            },
            meta: listed(Some(Tradition::Hindu), false),
        });
        rules.extend(SANKRANTI.iter().map(|&(key, month, day, what)| {
            STATE.on_solar_day(
                key,
                format!("{NOTICE}: {what}"),
                (Calendar::BengalSolar, month, day),
                false,
            )
        }));
        rules
    }
}

impl PublishedSet {
    /// The days the notifications print rather than reckon, unmarked, since
    /// none leaves a day to the moon: the four Islamic general holidays and
    /// Annexure-II's Shab-e-Qadr; Panchayati Raj Divas to 2025; Mahavir
    /// Jayanti to 2021; and 2026's day following Nuakhai.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet};
    ///
    /// let set = PublishedSet::india_odisha();
    /// assert_eq!(set.entries.len(), 4 * 8 + 7 + 8 + 3);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2019, 6, 5));
    /// assert_eq!(*subject_to, None);
    /// ```
    #[must_use]
    pub fn india_odisha() -> PublishedSet {
        let printed = |tradition, optional| PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to: None,
            meta: listed(tradition, optional),
            cite: |notice| {
                format!(
                    "Government of Odisha, Revenue and Disaster Management Department, notification {notice}, as printed"
                )
            },
        };
        let islamic = Some(Tradition::Islamic);
        let mut entries = printed(islamic, false).entries(&sighted(&SIGHTED_HERE));
        entries.extend(printed(islamic, true).on_days(&SIGHTED_OPTIONAL, &SHAB_E_QADR_DAYS));
        entries.extend(printed(None, false).on_days(&SOME_YEARS, &SOME_YEARS_DAYS[..7]));
        entries.extend(
            printed(Some(Tradition::Hindu), false).on_days(&SOME_YEARS, &SOME_YEARS_DAYS[7..]),
        );
        entries.extend(printed(Some(Tradition::Jain), true).on_days(&MAHAVIR, &MAHAVIR_DAYS));
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of Odisha, Revenue and Disaster Management Department"
                .to_owned(),
            id: "in-od-rdm-printed".to_owned(),
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
        for rule in FestivalRule::india_odisha() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_odisha() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::india_odisha() {
            assert!(
                keys.contains(&rule.after),
                "{} counts from nothing",
                rule.key
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for entry in PublishedSet::india_odisha().entries {
            if let PublishedEntry::Add { key, .. } = entry {
                assert!(!keys.contains(&key), "{key} is reckoned and printed");
            }
        }
    }

    #[test]
    fn every_day_of_the_list_is_kept_in_the_state() {
        let lunar = FestivalRule::india_odisha()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let dated = DatedRule::india_odisha()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let following = FollowingRule::india_odisha()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let metas: Vec<(String, FestivalMeta)> = lunar.chain(dated).chain(following).collect();
        assert_eq!(metas.len(), 21 + 16 + 3);
        let optional = metas.iter().filter(|(_, meta)| meta.optional).count();
        assert_eq!(optional, 8);
        for (key, meta) in &metas {
            assert_eq!(meta.regions, REGIONS, "{key}");
            assert_eq!(meta.kind, Some(FestivalKind::Gazetted), "{key}");
        }
    }

    #[test]
    fn every_printed_day_is_in_a_noticed_year() {
        let set = PublishedSet::india_odisha();
        assert_eq!(
            set.entries.len(),
            SIGHTED_HERE.len() * NOTIFICATIONS.len()
                + SHAB_E_QADR_DAYS.len()
                + SOME_YEARS_DAYS.len()
                + MAHAVIR_DAYS.len()
        );
        assert!(set.entries.iter().all(|entry| matches!(
            entry,
            PublishedEntry::Add {
                subject_to: None,
                ..
            }
        )));
        let years: Vec<i32> = NOTIFICATIONS.iter().map(|row| row.0).collect();
        assert!(set.entries.iter().all(|entry| match entry {
            PublishedEntry::Add { day, .. } => years.contains(&day.year),
            _ => false,
        }));
    }

    #[test]
    fn the_sankranti_days_fall_on_a_bengal_month_end() {
        let rules = DatedRule::india_odisha();
        let ends: Vec<(&str, u8, i8)> = rules
            .iter()
            .filter_map(|rule| match rule.on {
                Dated::CivilDate {
                    calendar: Calendar::BengalSolar,
                    month,
                    day,
                    ..
                } => Some((rule.key.as_str(), month, day)),
                _ => None,
            })
            .collect();
        assert_eq!(
            ends,
            [
                ("PAHILI_RAJA", 2, -2),
                ("MITHUNA_SANKRANTI", 2, -1),
                ("UTTARAYANA_PUNYAKALA", 9, -1)
            ]
        );
    }
}

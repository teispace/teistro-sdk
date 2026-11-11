//! West Bengal's public holidays (`03-design/west-bengal-holidays.md`,
//! SR-03, the second state set): the days the Finance (Audit)
//! Department's yearly notification lists, under s. 25 of the Negotiable
//! Instruments Act (List I), under the State Government's order (List II)
//! and as sectional holidays (List III), with the occasions its note names
//! as falling on a Sunday.
//!
//! The notification is rank 1 for the list. A day both this list and the
//! central list hold reuses `INDIA_CENTRAL`'s rule, and every other
//! reckoned day reuses a rule the SDK ships where one exists. Bengal's own
//! days are new rules read through the fits the Nepal and Mithila packs
//! use: Dol on Phalguna's full moon at sunrise, Kali Puja on the new moon
//! held at nishitha, the Durga Puja's Navami, and Harichand Thakur's
//! birthday on the tithi the notice names. The Puja's Chaturthi to
//! Shashthi are the three days before the Saptami and its Dashami the day
//! after the Navami, the days counted as the notices count them. Bengali
//! New Year, Tagore's birthday and Viswakarma Puja are days of the Bengal
//! solar calendar (`Calendar::BengalSolar`). `festival-measured.md` §11
//! holds every rule to eight years of notifications at Kolkata.
//!
//! Id-ul-Fitr, Id-ud-Zoha, Muharram, Fateha-Dwaz-Daham and Shab-e-Barat are
//! not reckoned: the notices leave their days to the sighting of the moon,
//! so they ship as printed, marked [`SubjectTo::MoonSighting`]. The days
//! the notices add around a festival, and the sectional days of one year
//! or a few, ship as printed, unmarked.

use teistro_calendar::easter::Computus;
use teistro_core::catalogue::{Calendar, Masa, Tithi, Tradition};

use super::india_central::{
    Declared, Named, OnDay, Printed, PrintedDays, StateList, on_civil_date, sighted,
};
use super::nepal_panchanga::Fit;
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{
    Dated, DatedRule, FestivalMeta, FestivalRule, FollowingRule, InAdhika, PublishedEntry,
    PublishedSet, Window,
};

/// Where the list is kept: India, then the state the notice names.
const REGIONS: [&str; 2] = ["IN", "IN-WB"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The central list's lunar rules this list keeps by another: Diwali is
/// Kali Puja, Dasara the day after the Navami, Janmastami the Smarta day,
/// and Holi the day after Dol, so the Holika fire is not here.
const CENTRAL_LEFT: [&str; 4] = ["VIJAYA_DASHAMI", "LAKSHMI_PUJA", "JANMASHTAMI", "HOLIKA"];

/// The notification the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of West Bengal, Finance (Audit) Department, notification No. \
     4188-F(P2) of 27 November 2025: List I under s. 25 of the Negotiable Instruments Act, 1881, \
     List II under the State Government's order, List III sectional";

/// Each year's notification, and how it was read: a copy giving its
/// number, its lists and its notes, republished by a staff site (rank 1;
/// the state's own server refused a scripted fetch, C1279).
const NOTIFICATIONS: [(i32, &str); 8] = [
    (
        2019,
        "Nos. 6812-F(P2) to 6815-F(P2) of 5 November 2018, rank 1 (a copy with their numbers and tables)",
    ),
    (
        2020,
        "Nos. 5962-F(P2) to 5965-F(P2) of 1 November 2019, rank 1 (a copy with their numbers and tables)",
    ),
    (
        2021,
        "No. 3100-F(P2) of 5 November 2020, rank 1 (a copy with its number and tables)",
    ),
    (
        2022,
        "No. 3640-F(P2) of 26 November 2021, rank 1 (a copy with its number and tables)",
    ),
    (
        2023,
        "No. 4331-F(P2) of 21 October 2022, rank 1 (a copy with its number and tables)",
    ),
    (
        2024,
        "No. 6112-F(P2) of 9 November 2023, rank 1 (a copy with its number and tables)",
    ),
    (
        2025,
        "No. 4712-F(P2) of 22 November 2024, rank 1 (a copy with its number and tables)",
    ),
    (
        2026,
        "No. 4188-F(P2) of 27 November 2025, rank 1 (a copy with its number and tables)",
    ),
];

/// The Bengal panjikas, as a new rule cites them.
const PANJIKA: &str = "the Bengal panjikas (Gupta Press, Surya Siddhanta; Bisuddha Siddhanta), as \
     West Bengal's notifications of 2019 to 2026 keep the day; the panjika's page is not read, so \
     the rule is held to the notices and the citation is open (C1278)";

/// Bengal's rules by the tithi: key, amanta month, tithi, fit, and what
/// the day is.
const BENGAL: [(&str, Masa, Tithi, Fit, &str); 4] = [
    (
        "DOLA_YATRA",
        Masa::Phalguna,
        Tithi::Purnima,
        Fit::Udaya,
        "Dolyatra, Krishna's swing festival and Chaitanya's birthday, on Phalguna's full moon",
    ),
    (
        "KALI_PUJA",
        Masa::Ashwina,
        Tithi::Amavasya,
        Fit::LaterAlone(Window::Nishitha),
        "Shyama (Kali) Puja, worshipped at midnight on the new moon of Ashwina (amanta; Kartika in the purnimanta naming)",
    ),
    (
        "MAHA_NAVAMI",
        Masa::Ashwina,
        Tithi::ShuklaNavami,
        Fit::Udaya,
        "the Durga Puja's Maha Navami on Ashwina's bright 9th; a navami that begins near midday, as in 2024, the notice keeps with the Ashtami (C1270)",
    ),
    (
        "HARICHAND_THAKUR_JAYANTI",
        Masa::Phalguna,
        Tithi::KrishnaTrayodashi,
        Fit::Udaya,
        "Shri Shri Harichand Thakur's birthday, which the notices print as the 'Madhu Krishna Troyodashi Tithi': Madhu is Chaitra in the purnimanta naming, whose dark half is the amanta Phalguna's; the reading at sunrise is inferred (C1274)",
    ),
];

/// The days the notices count from another: key, the rule, the count, and
/// how the notices give it.
const COUNTED: [(&str, &str, i8, &str); 5] = [
    (
        "HOLI",
        "DOLA_YATRA",
        1,
        "List II: 'Holi (Day after Doljatra)'; in 2024 and 2025 a footnote names Holi on Doljatra's day and the list keeps the day after as 'Day after Doljatra' (C1278)",
    ),
    (
        "DURGA_CHATURTHI",
        "PHULPATI",
        -3,
        "List II: the Durga Puja's Maha Chaturthi, the third day before the Saptami in every year listed",
    ),
    (
        "DURGA_PANCHAMI",
        "PHULPATI",
        -2,
        "List II: the Durga Puja's Maha Panchami, the second day before the Saptami in every year listed",
    ),
    (
        "DURGA_SHASHTHI",
        "PHULPATI",
        -1,
        "List II: the Durga Puja's Maha Shashthi, the day before the Saptami in every year listed, though in 2026 the 6th holds both sunrises",
    ),
    (
        "VIJAYA_DASHAMI",
        "MAHA_NAVAMI",
        1,
        "List I: the Durga Puja's Dasami (Vijaya Dasami), the day after the Navami (C1270)",
    ),
];

/// The state's days on a Gregorian date: key, (month, day), tradition and
/// communities.
type Civil = (
    &'static str,
    (u8, i8),
    Option<Tradition>,
    &'static [&'static str],
);

/// The state's days on a Gregorian date. Bhanu Bhakta's birthday is listed
/// for the Darjeeling and Kalimpong districts only; Hul Diwas is List
/// III's, for the Santhal; the Thakur Panchanan Barma's and Birsa Munda's
/// birthdays are listed from 2022.
const CIVIL: [Civil; 10] = [
    ("NEW_YEAR_S_DAY_AD", (1, 1), None, &[]),
    ("VIVEKANANDA_JAYANTI", (1, 12), None, &[]),
    ("NETAJI_JAYANTI", (1, 23), None, &[]),
    ("THAKUR_PANCHANAN_BARMA_JAYANTI", (2, 14), None, &[]),
    ("BANK_ACCOUNTS_CLOSING", (4, 1), None, &["BANKS"]),
    ("AMBEDKAR_JAYANTI", (4, 14), None, &[]),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1), None, &[]),
    ("HUL_DIWAS", (6, 30), None, &["SANTHAL"]),
    ("BHANU_JAYANTI", (7, 13), None, &[]),
    ("BIRSA_MUNDA_JAYANTI", (11, 15), None, &[]),
];

/// The state's days of the Bengal solar calendar: key, month (Baishakh the
/// 1st), day (from the month's end when negative), tradition, and what the
/// day is.
const SOLAR: [(&str, u8, i8, Option<Tradition>, &str); 3] = [
    (
        "POHELA_BOISHAKH",
        1,
        1,
        None,
        "List I: Bengali New Year's Day (Nababarsha), Baishakh 1, the civil day after the Sun's entry into Mesha, or the day after next when the entry falls after midnight",
    ),
    (
        "RABINDRA_JAYANTI",
        1,
        25,
        None,
        "List I: Rabindranath Tagore's birthday, 25 Baishakh",
    ),
    (
        "VISHWAKARMA_PUJA",
        5,
        -1,
        Some(Tradition::Hindu),
        "List II (2026): Viswakarma Puja, the last day of Bhadra, the day the Sun enters Kanya",
    ),
];

/// The days set by the moon, as (month, day) in the order of
/// [`NOTIFICATIONS`], each named as the central list names it; Fateha-Dwaz-
/// Daham is the central list's Id-e-Milad. The 2021 copy prints Id-ul-Fitr
/// as "4 May, Friday" with its additional day on Saturday 15 May; the
/// Friday was 14 May.
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

/// Shab-e-Barat, set by the moon, in the order of [`NOTIFICATIONS`]:
/// sectional in List III to 2022, then in List II.
const SHAB_E_BARAT: Printed<'static> = (
    "SHAB_E_BARAT",
    "Shab-e-Barat",
    "शब-ए-बरात",
    &[
        (4, 21),
        (4, 9),
        (3, 30),
        (3, 19),
        (3, 8),
        (2, 26),
        (2, 14),
        (2, 4),
    ],
);

/// The day the Department moved once the moon was seen.
const DECLARED: [Declared; 1] = [(
    "MUHARRAM",
    2021,
    (8, 20),
    "Government of West Bengal, Finance (Audit) Department, order No. 2365-F of 2021, moving Muharram from 19 to 20 August",
)];

/// A group of days the notices print once or more in some years: their
/// tradition, communities, names and days.
type Group = (
    Option<Tradition>,
    &'static [&'static str],
    &'static [Named<'static>],
    &'static [OnDay<'static>],
);

/// The days the notices add around a festival, and the sectional days of
/// one year or a few, by tradition. A festival's added days in one year are
/// numbered, first to fourth, since a published day of one key replaces
/// another within a week of it. Karam Puja is "to be notified later"
/// every year, so it has no day; Guru Rabidas's 2026 day is misprinted
/// ("Is February") and is left out.
const ADDED: [Group; 6] = [
    (Some(Tradition::Hindu), &[], &HINDU_NAMES, &HINDU_DAYS),
    (Some(Tradition::Islamic), &[], &ID_NAMES, &ID_DAYS),
    (Some(Tradition::Sikh), &[], &SIKH_NAMES, &SIKH_DAYS),
    (Some(Tradition::Jain), &[], &JAIN_NAMES, &JAIN_DAYS),
    (None, &["RAVIDASSIA"], &RAVIDAS_NAMES, &RAVIDAS_DAYS),
    (None, &[], &OTHER_NAMES, &OTHER_DAYS),
];

/// The days added around a Hindu festival.
const HINDU_NAMES: [Named<'static>; 12] = [
    (
        "SARASWATI_PUJA_ADDITIONAL_DAY",
        "Additional day for Saraswati Puja",
        "सरस्वती पूजाको थप दिन",
    ),
    ("DURGA_TRITIYA", "Durga Puja: Tritiya", "दुर्गा पूजा: तृतीया"),
    (
        "DURGA_PUJA_ADDITIONAL_DAY_1",
        "Durga Puja: first additional day",
        "दुर्गा पूजा: पहिलो थप दिन",
    ),
    (
        "DURGA_PUJA_ADDITIONAL_DAY_2",
        "Durga Puja: second additional day",
        "दुर्गा पूजा: दोस्रो थप दिन",
    ),
    (
        "DURGA_PUJA_ADDITIONAL_DAY_3",
        "Durga Puja: third additional day",
        "दुर्गा पूजा: तेस्रो थप दिन",
    ),
    (
        "DURGA_PUJA_ADDITIONAL_DAY_4",
        "Durga Puja: fourth additional day",
        "दुर्गा पूजा: चौथो थप दिन",
    ),
    (
        "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_1",
        "Lakshmi Puja: first additional day",
        "लक्ष्मी पूजा: पहिलो थप दिन",
    ),
    (
        "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_2",
        "Lakshmi Puja: second additional day",
        "लक्ष्मी पूजा: दोस्रो थप दिन",
    ),
    (
        "KALI_PUJA_ADDITIONAL_DAY_1",
        "Kali Puja: first additional day",
        "काली पूजा: पहिलो थप दिन",
    ),
    (
        "KALI_PUJA_ADDITIONAL_DAY_2",
        "Kali Puja: second additional day",
        "काली पूजा: दोस्रो थप दिन",
    ),
    (
        "YAMA_DWITIYA_ADDITIONAL_DAY",
        "Additional day for Bhratridwitiya",
        "भ्रातृद्वितीयाको थप दिन",
    ),
    (
        "CHHATH_ADDITIONAL_DAY",
        "Additional day for Chhat Puja",
        "छठ पूजाको थप दिन",
    ),
];

/// Each year's days added around a Hindu festival.
const HINDU_DAYS: [OnDay<'static>; 63] = [
    (2019, "SARASWATI_PUJA_ADDITIONAL_DAY", (2, 11)),
    (2019, "DURGA_PUJA_ADDITIONAL_DAY_1", (10, 9)),
    (2019, "DURGA_PUJA_ADDITIONAL_DAY_2", (10, 10)),
    (2019, "DURGA_PUJA_ADDITIONAL_DAY_3", (10, 11)),
    (2019, "DURGA_PUJA_ADDITIONAL_DAY_4", (10, 12)),
    (2019, "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_1", (10, 14)),
    (2019, "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_2", (10, 15)),
    (2019, "KALI_PUJA_ADDITIONAL_DAY_1", (10, 28)),
    (2019, "YAMA_DWITIYA_ADDITIONAL_DAY", (10, 30)),
    (2020, "SARASWATI_PUJA_ADDITIONAL_DAY", (1, 31)),
    (2020, "DURGA_TRITIYA", (10, 19)),
    (2020, "DURGA_PUJA_ADDITIONAL_DAY_1", (10, 27)),
    (2020, "DURGA_PUJA_ADDITIONAL_DAY_2", (10, 28)),
    (2020, "DURGA_PUJA_ADDITIONAL_DAY_3", (10, 29)),
    (2020, "YAMA_DWITIYA_ADDITIONAL_DAY", (11, 17)),
    (2020, "CHHATH_ADDITIONAL_DAY", (11, 19)),
    (2021, "SARASWATI_PUJA_ADDITIONAL_DAY", (2, 17)),
    (2021, "DURGA_PUJA_ADDITIONAL_DAY_1", (10, 18)),
    (2021, "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_1", (10, 21)),
    (2021, "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_2", (10, 22)),
    (2021, "KALI_PUJA_ADDITIONAL_DAY_1", (11, 5)),
    (2021, "CHHATH_ADDITIONAL_DAY", (11, 9)),
    (2022, "SARASWATI_PUJA_ADDITIONAL_DAY", (2, 4)),
    (2022, "DURGA_PUJA_ADDITIONAL_DAY_1", (10, 6)),
    (2022, "DURGA_PUJA_ADDITIONAL_DAY_2", (10, 7)),
    (2022, "DURGA_PUJA_ADDITIONAL_DAY_3", (10, 8)),
    (2022, "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_1", (10, 10)),
    (2022, "KALI_PUJA_ADDITIONAL_DAY_1", (10, 25)),
    (2022, "KALI_PUJA_ADDITIONAL_DAY_2", (10, 26)),
    (2022, "CHHATH_ADDITIONAL_DAY", (10, 31)),
    (2023, "SARASWATI_PUJA_ADDITIONAL_DAY", (1, 25)),
    (2023, "DURGA_PUJA_ADDITIONAL_DAY_1", (10, 25)),
    (2023, "DURGA_PUJA_ADDITIONAL_DAY_2", (10, 26)),
    (2023, "DURGA_PUJA_ADDITIONAL_DAY_3", (10, 27)),
    (2023, "KALI_PUJA_ADDITIONAL_DAY_1", (11, 13)),
    (2023, "KALI_PUJA_ADDITIONAL_DAY_2", (11, 14)),
    (2023, "YAMA_DWITIYA_ADDITIONAL_DAY", (11, 16)),
    (2023, "CHHATH_ADDITIONAL_DAY", (11, 20)),
    (2024, "SARASWATI_PUJA_ADDITIONAL_DAY", (2, 13)),
    (2024, "DURGA_PUJA_ADDITIONAL_DAY_1", (10, 14)),
    (2024, "DURGA_PUJA_ADDITIONAL_DAY_2", (10, 15)),
    (2024, "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_1", (10, 17)),
    (2024, "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_2", (10, 18)),
    (2024, "KALI_PUJA_ADDITIONAL_DAY_1", (11, 1)),
    (2024, "YAMA_DWITIYA_ADDITIONAL_DAY", (11, 4)),
    (2024, "CHHATH_ADDITIONAL_DAY", (11, 8)),
    (2025, "SARASWATI_PUJA_ADDITIONAL_DAY", (2, 3)),
    (2025, "DURGA_PUJA_ADDITIONAL_DAY_1", (10, 3)),
    (2025, "DURGA_PUJA_ADDITIONAL_DAY_2", (10, 4)),
    (2025, "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_1", (10, 7)),
    (2025, "KALI_PUJA_ADDITIONAL_DAY_1", (10, 21)),
    (2025, "KALI_PUJA_ADDITIONAL_DAY_2", (10, 22)),
    (2025, "YAMA_DWITIYA_ADDITIONAL_DAY", (10, 24)),
    (2025, "CHHATH_ADDITIONAL_DAY", (10, 28)),
    (2026, "SARASWATI_PUJA_ADDITIONAL_DAY", (1, 22)),
    (2026, "DURGA_PUJA_ADDITIONAL_DAY_1", (10, 22)),
    (2026, "DURGA_PUJA_ADDITIONAL_DAY_2", (10, 23)),
    (2026, "DURGA_PUJA_ADDITIONAL_DAY_3", (10, 24)),
    (2026, "KOJAGRAT_PURNIMA_ADDITIONAL_DAY_1", (10, 26)),
    (2026, "KALI_PUJA_ADDITIONAL_DAY_1", (11, 9)),
    (2026, "KALI_PUJA_ADDITIONAL_DAY_2", (11, 10)),
    (2026, "YAMA_DWITIYA_ADDITIONAL_DAY", (11, 12)),
    (2026, "CHHATH_ADDITIONAL_DAY", (11, 16)),
];

/// The days added around an Id.
const ID_NAMES: [Named<'static>; 2] = [
    (
        "EID_AL_FITR_ADDITIONAL_DAY",
        "Additional day for Id-ul-Fitr",
        "ईद उल फित्रको थप दिन",
    ),
    (
        "EID_AL_ADHA_ADDITIONAL_DAY",
        "Additional day for Id-ud-Zoha (Bakrid)",
        "बकर ईदको थप दिन",
    ),
];

/// Each year's days added around an Id.
const ID_DAYS: [OnDay<'static>; 9] = [
    (2020, "EID_AL_FITR_ADDITIONAL_DAY", (5, 26)),
    (2021, "EID_AL_FITR_ADDITIONAL_DAY", (5, 15)),
    (2022, "EID_AL_FITR_ADDITIONAL_DAY", (5, 4)),
    (2023, "EID_AL_FITR_ADDITIONAL_DAY", (4, 21)),
    (2024, "EID_AL_FITR_ADDITIONAL_DAY", (4, 10)),
    (2025, "EID_AL_FITR_ADDITIONAL_DAY", (4, 1)),
    (2025, "EID_AL_ADHA_ADDITIONAL_DAY", (6, 6)),
    (2026, "EID_AL_FITR_ADDITIONAL_DAY", (3, 20)),
    (2026, "EID_AL_ADHA_ADDITIONAL_DAY", (5, 26)),
];

/// The Sikh sectional days.
const SIKH_NAMES: [Named<'static>; 2] = [
    ("BAISAKHI", "Baisakhi", "बैसाखी"),
    (
        "GURU_GOBIND_SINGH_JAYANTI",
        "Prakash Purab of Guru Gobind Singh",
        "गुरु गोविन्द सिंह जयन्ती",
    ),
];

/// Each year's Sikh sectional days.
const SIKH_DAYS: [OnDay<'static>; 3] = [
    (2019, "BAISAKHI", (4, 14)),
    (2020, "BAISAKHI", (4, 14)),
    (2025, "GURU_GOBIND_SINGH_JAYANTI", (1, 6)),
];

/// The Jain day a footnote names.
const JAIN_NAMES: [Named<'static>; 1] = [(
    "PARSHWANATH_RATH_YATRA",
    "Parshwanath's Rathajatra",
    "पार्श्वनाथ रथयात्रा",
)];

/// Each year's Jain day.
const JAIN_DAYS: [OnDay<'static>; 3] = [
    (2024, "PARSHWANATH_RATH_YATRA", (11, 15)),
    (2025, "PARSHWANATH_RATH_YATRA", (11, 5)),
    (2026, "PARSHWANATH_RATH_YATRA", (11, 24)),
];

/// Guru Rabidas's birthday, sectional for his followers.
const RAVIDAS_NAMES: [Named<'static>; 1] = [(
    "GURU_RAVIDAS_JAYANTI",
    "Birthday of Guru Rabidas",
    "गुरु रविदास जयन्ती",
)];

/// Each year's Guru Rabidas's birthday.
const RAVIDAS_DAYS: [OnDay<'static>; 4] = [
    (2022, "GURU_RAVIDAS_JAYANTI", (2, 16)),
    (2023, "GURU_RAVIDAS_JAYANTI", (2, 5)),
    (2024, "GURU_RAVIDAS_JAYANTI", (2, 24)),
    (2025, "GURU_RAVIDAS_JAYANTI", (2, 12)),
];

/// The birthdays a footnote names on another holiday's day.
const OTHER_NAMES: [Named<'static>; 2] = [
    (
        "RAGHUNATH_MURMU_JAYANTI",
        "Birthday of Pandit Raghunath Murmu",
        "पण्डित रघुनाथ मुर्मु जयन्ती",
    ),
    (
        "BIR_CHILARAI_JAYANTI",
        "Birthday of Bir Chilarai",
        "वीर चिलाराय जयन्ती",
    ),
];

/// Each year's birthdays a footnote names.
const OTHER_DAYS: [OnDay<'static>; 6] = [
    (2022, "RAGHUNATH_MURMU_JAYANTI", (5, 16)),
    (2023, "RAGHUNATH_MURMU_JAYANTI", (5, 5)),
    (2024, "RAGHUNATH_MURMU_JAYANTI", (5, 23)),
    (2025, "RAGHUNATH_MURMU_JAYANTI", (5, 12)),
    (2026, "RAGHUNATH_MURMU_JAYANTI", (5, 1)),
    (2026, "BIR_CHILARAI_JAYANTI", (2, 1)),
];

/// The values of a day of the list.
fn listed(tradition: Option<Tradition>) -> FestivalMeta {
    STATE.listed(tradition, false)
}

/// A printed day's source, from its year's notification.
fn printed(notice: &str) -> String {
    format!(
        "Government of West Bengal, Finance (Audit) Department, notification {notice}, as printed"
    )
}

impl FestivalRule {
    /// The lunar days of West Bengal's public holidays: the central list's
    /// rules for the days both lists hold, *Dharmasindhu*'s for Rakhi
    /// Bandhan, Janmastami and Bhratridwitiya, Nepal's printed panchanga's
    /// for Saraswati Puja, Rathayatra, Mahalaya, the Puja's Saptami and
    /// Ashtami, Lakshmi Puja and Chhat Puja, and Bengal's own for Dol, Kali
    /// Puja, the Puja's Navami and Harichand Thakur's birthday.
    ///
    /// ```
    /// use teistro_panchanga::festival::{FestivalRule, Window};
    ///
    /// let rules = FestivalRule::india_west_bengal();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let kali = of("KALI_PUJA").ok_or("shipped")?;
    /// assert_eq!(kali.at, Window::Nishitha);
    /// assert_eq!(kali.meta.regions, ["IN", "IN-WB"]);
    /// // The central list's Diwali, Dasara and Holika are not the state's.
    /// assert!(of("LAKSHMI_PUJA").is_none() && of("HOLIKA").is_none());
    /// assert!(of("JANMASHTAMI").ok_or("shipped")?.source.starts_with("Dharmasindhu"));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_west_bengal() -> Vec<FestivalRule> {
        let mut rules = STATE.central(&CENTRAL_LEFT);
        rules.extend(STATE.by_text(&[
            ("RAKSHABANDHAN", false),
            ("JANMASHTAMI", false),
            ("YAMA_DWITIYA", false),
        ]));
        rules.extend(STATE.by_print(&[
            "BASANTA_PANCHAMI",
            "JAGANNATH_RATH_JATRA",
            "AMAVASYA_SHRADDHA",
            "PHULPATI",
            "MAHA_ASHTAMI",
            "KOJAGRAT_PURNIMA",
            "CHHATH",
        ]));
        rules.extend(
            BENGAL
                .iter()
                .map(|&(key, month, tithi, fit, what)| FestivalRule {
                    meta: listed(Some(Tradition::Hindu)),
                    ..fit.rule(
                        key,
                        Some(month),
                        tithi.into(),
                        InAdhika::Nija,
                        &format!("{what}; {PANJIKA}"),
                    )
                }),
        );
        rules
    }
}

impl FollowingRule {
    /// The days the notices count from another: Holi the day after Dol,
    /// the Durga Puja's Chaturthi, Panchami and Shashthi the three days
    /// before its Saptami, and its Dashami the day after its Navami.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let rules = FollowingRule::india_west_bengal();
    /// let counted: Vec<(&str, &str, i8)> =
    ///     rules.iter().map(|rule| (rule.key.as_str(), rule.after.as_str(), rule.days)).collect();
    /// assert_eq!(counted[0], ("HOLI", "DOLA_YATRA", 1));
    /// assert_eq!(counted[1], ("DURGA_CHATURTHI", "PHULPATI", -3));
    /// ```
    #[must_use]
    pub fn india_west_bengal() -> Vec<FollowingRule> {
        COUNTED
            .iter()
            .map(|&(key, after, days, how)| FollowingRule {
                key: key.to_owned(),
                source: format!("{NOTICE}, {how}"),
                after: after.to_owned(),
                days,
                meta: listed(Some(Tradition::Hindu)),
            })
            .collect()
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, Easter Saturday,
    /// the state's days on a Gregorian date, the yearly closing of bank
    /// accounts, and Bengali New Year, Tagore's birthday and Viswakarma
    /// Puja on the Bengal solar calendar.
    ///
    /// ```
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_panchanga::festival::{Dated, DatedRule};
    ///
    /// let rules = DatedRule::india_west_bengal();
    /// let new_year = rules.iter().find(|rule| rule.key == "POHELA_BOISHAKH").ok_or("shipped")?;
    /// assert_eq!(
    ///     new_year.on,
    ///     Dated::CivilDate { calendar: Calendar::BengalSolar, month: 1, day: 1, skipped: None }
    /// );
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-WB"]));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_west_bengal() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        rules.push(DatedRule {
            key: "EASTER_SATURDAY".to_owned(),
            source: format!("{NOTICE}: List III, Easter Saturday, for Christians"),
            on: Dated::Easter {
                computus: Computus::Gregorian,
                days: -1,
            },
            meta: listed(Some(Tradition::Christian)),
        });
        rules.extend(CIVIL.iter().map(|&(key, day, tradition, communities)| {
            let meta = FestivalMeta {
                communities: communities.iter().map(|&one| one.to_owned()).collect(),
                ..listed(tradition)
            };
            on_civil_date(key, day, NOTICE, meta)
        }));
        rules.extend(
            SOLAR
                .iter()
                .map(|&(key, month, day, tradition, what)| DatedRule {
                    key: key.to_owned(),
                    source: format!("{NOTICE}, {what}"),
                    on: Dated::CivilDate {
                        calendar: Calendar::BengalSolar,
                        month,
                        day,
                        skipped: None,
                    },
                    meta: listed(tradition),
                }),
        );
        rules
    }
}

impl PublishedSet {
    /// The days the notifications print rather than reckon, for the years
    /// a notification is in hand: the five set by the moon, each marked
    /// subject to the sighting but Muharram 2021, which an order moved; and
    /// the days added around a festival and the sectional days of a year or
    /// a few, unmarked.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_west_bengal();
    /// assert_eq!(set.entries.len(), 40 + 88);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2019, 6, 5));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// let PublishedEntry::Add { key, subject_to, .. } = &set.entries[40] else { panic!("an ADD") };
    /// assert_eq!((key.as_str(), *subject_to), ("SARASWATI_PUJA_ADDITIONAL_DAY", None));
    /// ```
    #[must_use]
    pub fn india_west_bengal() -> PublishedSet {
        let mut moon = sighted(&SIGHTED_HERE);
        moon.push(SHAB_E_BARAT);
        let mut entries = PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &DECLARED,
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic)),
            cite: |notice| {
                format!(
                    "Government of West Bengal, Finance (Audit) Department, notification {notice}, its day subject to the sighting of the moon"
                )
            },
        }
        .entries(&moon);
        entries.extend(ADDED.iter().flat_map(added));
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of West Bengal, Finance (Audit) Department".to_owned(),
            id: "in-wb-finance-printed".to_owned(),
            version: 1,
            entries,
        }
    }
}

/// A group's entries: each day as its year's notification prints it.
fn added(&(tradition, communities, names, days): &Group) -> Vec<PublishedEntry> {
    PrintedDays {
        notices: &NOTIFICATIONS,
        declared: &[],
        subject_to: None,
        meta: FestivalMeta {
            communities: communities.iter().map(|&one| one.to_owned()).collect(),
            ..listed(tradition)
        },
        cite: printed,
    }
    .on_days(names, days)
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
        for rule in FestivalRule::india_west_bengal() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::india_west_bengal() {
            rule.check().unwrap();
            assert!(
                keys.contains(&rule.after),
                "{} follows {}",
                rule.key,
                rule.after
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_west_bengal() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
    }

    #[test]
    fn every_day_of_the_list_is_gazetted_in_the_state() {
        let lunar = FestivalRule::india_west_bengal()
            .into_iter()
            .map(|rule| rule.meta);
        let counted = FollowingRule::india_west_bengal()
            .into_iter()
            .map(|rule| rule.meta);
        let dated = DatedRule::india_west_bengal()
            .into_iter()
            .map(|rule| rule.meta);
        let metas: Vec<FestivalMeta> = lunar.chain(counted).chain(dated).collect();
        assert_eq!(metas.len(), 19 + 5 + 19);
        for meta in &metas {
            assert_eq!(meta.kind, Some(FestivalKind::Gazetted));
            assert_eq!(meta.regions, REGIONS);
            assert!(!meta.optional);
        }
    }

    #[test]
    fn a_day_both_lists_hold_is_the_central_rule() {
        let central = FestivalRule::india_central();
        for rule in FestivalRule::india_west_bengal() {
            if CENTRAL_LEFT.contains(&rule.key.as_str()) {
                continue;
            }
            if let Some(shared) = central.iter().find(|other| other.key == rule.key) {
                assert_eq!((&rule.at, &rule.source), (&shared.at, &shared.source));
            }
        }
    }

    #[test]
    fn every_added_day_is_named_and_in_a_noticed_year() {
        for (_, _, names, days) in ADDED {
            for (year, key, _) in days {
                assert!(names.iter().any(|named| named.0 == *key), "{key}");
                assert!(
                    NOTIFICATIONS.iter().any(|notice| notice.0 == *year),
                    "{year}"
                );
            }
        }
        let added: usize = ADDED.iter().map(|group| group.3.len()).sum();
        let set = PublishedSet::india_west_bengal();
        assert_eq!(set.entries.len(), 5 * NOTIFICATIONS.len() + added);
    }
}

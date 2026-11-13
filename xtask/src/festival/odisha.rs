//! The `INDIA_ODISHA` pack against the Government of Odisha's holiday
//! notifications (`03-design/odisha-holidays.md`, SR-03), as
//! [`super::andhra_pradesh`] holds Andhra Pradesh's, through the same
//! comparison.
//!
//! Every rule of the pack, at Bhubaneswar on India's clock, against each
//! year's general holidays (Annexure-I and Annexure-III's days on a Sunday
//! or a second or fourth Saturday) and Annexure-II's optional holidays;
//! every parting is named with its cause, which the pass checks both ways.
//! The days the notifications print rather than reckon are held only to
//! stand in their year, and a year with no notification holds none.

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Section, Sky};

/// Each year's notification of the Revenue and Disaster Management
/// Department, and the copy read.
const NOTICES: [(i32, &str); 8] = [
    (
        2019,
        "No. RDM-Reg-MISC-0014-2018-40221, 29 October 2018: rank 1, the department's copy as archived",
    ),
    (
        2020,
        "No. RDM-Reg-REGN-0050-2019-32550, 23 October 2019: rank 1, the department's copy as archived",
    ),
    (
        2021,
        "No. RDM-Reg-REGN-0072-2020-30754, 20 October 2020: rank 1, the signed scan as archived",
    ),
    (
        2022,
        "No. RDM-Reg-REGN-0080-2021-34561, 17 November 2021: rank 1, the signed scan as archived",
    ),
    (
        2023,
        "No. RDM-Reg-MISC-0014-2022-37021, 1 November 2022: rank 1, the signed scan as archived",
    ),
    (
        2024,
        "No. RDM-Reg-MISC-0014-2023-39304, 7 November 2023: rank 1, the signed scan as archived",
    ),
    (
        2025,
        "No. RDM-Reg-MISC-0013-2024-39077, 12 November 2024: rank 1 as copied, the signed scan a university enclosed (C1480)",
    ),
    (
        2026,
        "No. RDM-Reg-MISC-0021-2025-42428, 11 November 2025: rank 1 as copied, the signed text a compliance site carries (C1480)",
    ),
];

/// Each year's reckoned days, general and optional, less the days every
/// notification lists on one date, which [`CIVIL`] holds in every year.
const LISTED: [(i32, &str, MonthDay); 246] = [
    (2019, "BASANTA_PANCHAMI", (2, 10)),
    (2019, "SHIVARATRI", (3, 4)),
    (2019, "DOLA_YATRA", (3, 21)),
    (2019, "HOLI", (3, 22)),
    (2019, "RAMA_NAVAMI", (4, 13)),
    (2019, "GOOD_FRIDAY", (4, 19)),
    (2019, "EASTER_SATURDAY", (4, 20)),
    (2019, "MESHA_SANKRANTI", (4, 14)),
    (2019, "BUDDHA_JAYANTI", (5, 18)),
    (2019, "SAVITRI_AMAVASYA", (6, 3)),
    (2019, "PAHILI_RAJA", (6, 14)),
    (2019, "MITHUNA_SANKRANTI", (6, 15)),
    (2019, "JAGANNATH_RATH_JATRA", (7, 4)),
    (2019, "BAHUDA_JATRA", (7, 12)),
    (2019, "JHULANA_PURNIMA", (8, 15)),
    (2019, "JANMASHTAMI", (8, 23)),
    (2019, "GANESHA_CHATURTHI", (9, 2)),
    (2019, "NUAKHAI", (9, 3)),
    (2019, "MAHALAYA_AMAVASYA", (9, 28)),
    (2019, "PHULPATI", (10, 5)),
    (2019, "MAHA_ASHTAMI", (10, 6)),
    (2019, "MAHA_NAVAMI", (10, 7)),
    (2019, "VIJAYA_DASHAMI", (10, 8)),
    (2019, "KUMAR_PURNIMA", (10, 13)),
    (2019, "KALI_PUJA", (10, 27)),
    (2019, "ANLA_NAVAMI", (11, 5)),
    (2019, "RAHAS_PURNIMA", (11, 12)),
    (2019, "PRATHAMASHTAMI", (11, 20)),
    (2019, "GURU_GOBIND_SINGH_JAYANTI", (1, 13)),
    (2019, "BRAHMO_SAMAJ_FOUNDATION_DAY", (1, 25)),
    (2020, "BASANTA_PANCHAMI", (1, 30)),
    (2020, "SHIVARATRI", (2, 21)),
    (2020, "DOLA_YATRA", (3, 9)),
    (2020, "HOLI", (3, 10)),
    (2020, "RAMA_NAVAMI", (4, 2)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "EASTER_SATURDAY", (4, 11)),
    (2020, "MESHA_SANKRANTI", (4, 13)),
    (2020, "BUDDHA_JAYANTI", (5, 7)),
    (2020, "SAVITRI_AMAVASYA", (5, 22)),
    (2020, "PAHILI_RAJA", (6, 14)),
    (2020, "MITHUNA_SANKRANTI", (6, 15)),
    (2020, "JAGANNATH_RATH_JATRA", (6, 23)),
    (2020, "BAHUDA_JATRA", (7, 1)),
    (2020, "JHULANA_PURNIMA", (8, 3)),
    (2020, "JANMASHTAMI", (8, 11)),
    (2020, "GANESHA_CHATURTHI", (8, 22)),
    (2020, "NUAKHAI", (8, 23)),
    (2020, "MAHALAYA_AMAVASYA", (9, 17)),
    (2020, "PHULPATI", (10, 23)),
    (2020, "MAHA_ASHTAMI", (10, 24)),
    (2020, "MAHA_NAVAMI", (10, 25)),
    (2020, "VIJAYA_DASHAMI", (10, 26)),
    (2020, "KUMAR_PURNIMA", (10, 31)),
    (2020, "KALI_PUJA", (11, 14)),
    (2020, "ANLA_NAVAMI", (11, 23)),
    (2020, "RAHAS_PURNIMA", (11, 30)),
    (2020, "PRATHAMASHTAMI", (12, 8)),
    (2020, "GURU_GOBIND_SINGH_JAYANTI", (1, 2)),
    (2020, "BRAHMO_SAMAJ_FOUNDATION_DAY", (1, 25)),
    (2021, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2021, "BASANTA_PANCHAMI", (2, 16)),
    (2021, "SHIVARATRI", (3, 11)),
    (2021, "DOLA_YATRA", (3, 28)),
    (2021, "HOLI", (3, 29)),
    (2021, "RAMA_NAVAMI", (4, 21)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "EASTER_SATURDAY", (4, 3)),
    (2021, "MESHA_SANKRANTI", (4, 14)),
    (2021, "BUDDHA_JAYANTI", (5, 26)),
    (2021, "SAVITRI_AMAVASYA", (6, 10)),
    (2021, "PAHILI_RAJA", (6, 14)),
    (2021, "MITHUNA_SANKRANTI", (6, 15)),
    (2021, "JAGANNATH_RATH_JATRA", (7, 12)),
    (2021, "BAHUDA_JATRA", (7, 20)),
    (2021, "JHULANA_PURNIMA", (8, 22)),
    (2021, "JANMASHTAMI", (8, 30)),
    (2021, "GANESHA_CHATURTHI", (9, 10)),
    (2021, "NUAKHAI", (9, 11)),
    (2021, "MAHALAYA_AMAVASYA", (10, 6)),
    (2021, "PHULPATI", (10, 12)),
    (2021, "MAHA_ASHTAMI", (10, 13)),
    (2021, "MAHA_NAVAMI", (10, 14)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "KUMAR_PURNIMA", (10, 20)),
    (2021, "KALI_PUJA", (11, 4)),
    (2021, "ANLA_NAVAMI", (11, 12)),
    (2021, "RAHAS_PURNIMA", (11, 19)),
    (2021, "PRATHAMASHTAMI", (11, 27)),
    (2021, "GURU_GOBIND_SINGH_JAYANTI", (1, 20)),
    (2021, "BRAHMO_SAMAJ_FOUNDATION_DAY", (1, 25)),
    (2022, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2022, "BASANTA_PANCHAMI", (2, 5)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "DOLA_YATRA", (3, 18)),
    (2022, "HOLI", (3, 19)),
    (2022, "RAMA_NAVAMI", (4, 10)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "EASTER_SATURDAY", (4, 16)),
    (2022, "MESHA_SANKRANTI", (4, 14)),
    (2022, "BUDDHA_JAYANTI", (5, 16)),
    (2022, "SAVITRI_AMAVASYA", (5, 30)),
    (2022, "PAHILI_RAJA", (6, 14)),
    (2022, "MITHUNA_SANKRANTI", (6, 15)),
    (2022, "JAGANNATH_RATH_JATRA", (7, 1)),
    (2022, "BAHUDA_JATRA", (7, 9)),
    (2022, "JHULANA_PURNIMA", (8, 11)),
    (2022, "JANMASHTAMI", (8, 18)),
    (2022, "GANESHA_CHATURTHI", (8, 31)),
    (2022, "NUAKHAI", (9, 1)),
    (2022, "MAHALAYA_AMAVASYA", (9, 25)),
    (2022, "PHULPATI", (10, 2)),
    (2022, "MAHA_ASHTAMI", (10, 3)),
    (2022, "MAHA_NAVAMI", (10, 4)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "KUMAR_PURNIMA", (10, 9)),
    (2022, "KALI_PUJA", (10, 24)),
    (2022, "ANLA_NAVAMI", (11, 2)),
    (2022, "RAHAS_PURNIMA", (11, 8)),
    (2022, "PRATHAMASHTAMI", (11, 16)),
    (2022, "GURU_GOBIND_SINGH_JAYANTI", (1, 9)),
    (2022, "BRAHMO_SAMAJ_FOUNDATION_DAY", (1, 25)),
    (2023, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2023, "BASANTA_PANCHAMI", (1, 26)),
    (2023, "SHIVARATRI", (2, 18)),
    (2023, "DOLA_YATRA", (3, 7)),
    (2023, "HOLI", (3, 8)),
    (2023, "RAMA_NAVAMI", (3, 30)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "EASTER_SATURDAY", (4, 8)),
    (2023, "MESHA_SANKRANTI", (4, 14)),
    (2023, "BUDDHA_JAYANTI", (5, 5)),
    (2023, "SAVITRI_AMAVASYA", (5, 19)),
    (2023, "PAHILI_RAJA", (6, 14)),
    (2023, "MITHUNA_SANKRANTI", (6, 15)),
    (2023, "JAGANNATH_RATH_JATRA", (6, 20)),
    (2023, "BAHUDA_JATRA", (6, 28)),
    (2023, "JHULANA_PURNIMA", (8, 30)),
    (2023, "JANMASHTAMI", (9, 6)),
    (2023, "GANESHA_CHATURTHI", (9, 19)),
    (2023, "NUAKHAI", (9, 20)),
    (2023, "MAHALAYA_AMAVASYA", (10, 14)),
    (2023, "PHULPATI", (10, 21)),
    (2023, "MAHA_ASHTAMI", (10, 22)),
    (2023, "MAHA_NAVAMI", (10, 23)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "KUMAR_PURNIMA", (10, 28)),
    (2023, "KALI_PUJA", (11, 12)),
    (2023, "ANLA_NAVAMI", (11, 21)),
    (2023, "RAHAS_PURNIMA", (11, 27)),
    (2023, "PRATHAMASHTAMI", (12, 5)),
    (2023, "GURU_GOBIND_SINGH_JAYANTI", (1, 5)),
    (2023, "BRAHMO_SAMAJ_FOUNDATION_DAY", (1, 25)),
    (2024, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2024, "BASANTA_PANCHAMI", (2, 14)),
    (2024, "SHIVARATRI", (3, 8)),
    (2024, "DOLA_YATRA", (3, 25)),
    (2024, "HOLI", (3, 26)),
    (2024, "RAMA_NAVAMI", (4, 17)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "EASTER_SATURDAY", (3, 30)),
    (2024, "MESHA_SANKRANTI", (4, 14)),
    (2024, "BUDDHA_JAYANTI", (5, 23)),
    (2024, "SAVITRI_AMAVASYA", (6, 6)),
    (2024, "PAHILI_RAJA", (6, 14)),
    (2024, "MITHUNA_SANKRANTI", (6, 15)),
    (2024, "JAGANNATH_RATH_JATRA", (7, 7)),
    (2024, "BAHUDA_JATRA", (7, 15)),
    (2024, "JHULANA_PURNIMA", (8, 19)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "GANESHA_CHATURTHI", (9, 7)),
    (2024, "NUAKHAI", (9, 8)),
    (2024, "MAHALAYA_AMAVASYA", (10, 2)),
    (2024, "PHULPATI", (10, 10)),
    (2024, "MAHA_ASHTAMI", (10, 11)),
    (2024, "MAHA_NAVAMI", (10, 12)),
    (2024, "VIJAYA_DASHAMI", (10, 13)),
    (2024, "KUMAR_PURNIMA", (10, 16)),
    (2024, "KALI_PUJA", (10, 31)),
    (2024, "ANLA_NAVAMI", (11, 10)),
    (2024, "RAHAS_PURNIMA", (11, 15)),
    (2024, "PRATHAMASHTAMI", (11, 23)),
    (2024, "GURU_GOBIND_SINGH_JAYANTI", (1, 17)),
    (2024, "BRAHMO_SAMAJ_FOUNDATION_DAY", (1, 25)),
    (2025, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2025, "BASANTA_PANCHAMI", (2, 2)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "DOLA_YATRA", (3, 14)),
    (2025, "HOLI", (3, 15)),
    (2025, "RAMA_NAVAMI", (4, 6)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "EASTER_SATURDAY", (4, 19)),
    (2025, "MESHA_SANKRANTI", (4, 14)),
    (2025, "BUDDHA_JAYANTI", (5, 12)),
    (2025, "SAVITRI_AMAVASYA", (5, 27)),
    (2025, "PAHILI_RAJA", (6, 14)),
    (2025, "MITHUNA_SANKRANTI", (6, 15)),
    (2025, "JAGANNATH_RATH_JATRA", (6, 27)),
    (2025, "BAHUDA_JATRA", (7, 5)),
    (2025, "JHULANA_PURNIMA", (8, 9)),
    (2025, "JANMASHTAMI", (8, 15)),
    (2025, "GANESHA_CHATURTHI", (8, 27)),
    (2025, "NUAKHAI", (8, 28)),
    (2025, "MAHALAYA_AMAVASYA", (9, 21)),
    (2025, "PHULPATI", (9, 29)),
    (2025, "MAHA_ASHTAMI", (9, 30)),
    (2025, "MAHA_NAVAMI", (10, 1)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "KUMAR_PURNIMA", (10, 7)),
    (2025, "KALI_PUJA", (10, 21)),
    (2025, "ANLA_NAVAMI", (10, 31)),
    (2025, "RAHAS_PURNIMA", (11, 5)),
    (2025, "PRATHAMASHTAMI", (11, 12)),
    (2025, "GURU_GOBIND_SINGH_JAYANTI", (1, 6)),
    (2025, "BRAHMO_SAMAJ_FOUNDATION_DAY", (1, 25)),
    (2026, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2026, "BASANTA_PANCHAMI", (1, 23)),
    (2026, "SHIVARATRI", (2, 15)),
    (2026, "DOLA_YATRA", (3, 3)),
    (2026, "HOLI", (3, 4)),
    (2026, "RAMA_NAVAMI", (3, 27)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "EASTER_SATURDAY", (4, 4)),
    (2026, "MESHA_SANKRANTI", (4, 14)),
    (2026, "BUDDHA_JAYANTI", (5, 1)),
    (2026, "SAVITRI_AMAVASYA", (5, 16)),
    (2026, "PAHILI_RAJA", (6, 14)),
    (2026, "MITHUNA_SANKRANTI", (6, 15)),
    (2026, "JAGANNATH_RATH_JATRA", (7, 16)),
    (2026, "BAHUDA_JATRA", (7, 24)),
    (2026, "JHULANA_PURNIMA", (8, 27)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "GANESHA_CHATURTHI", (9, 14)),
    (2026, "NUAKHAI", (9, 15)),
    (2026, "MAHALAYA_AMAVASYA", (10, 10)),
    (2026, "PHULPATI", (10, 17)),
    (2026, "MAHA_ASHTAMI", (10, 18)),
    (2026, "MAHA_NAVAMI", (10, 19)),
    (2026, "VIJAYA_DASHAMI", (10, 20)),
    (2026, "KUMAR_PURNIMA", (10, 25)),
    (2026, "KALI_PUJA", (11, 8)),
    (2026, "ANLA_NAVAMI", (11, 18)),
    (2026, "RAHAS_PURNIMA", (11, 24)),
    (2026, "PRATHAMASHTAMI", (12, 1)),
    (2026, "GURU_GOBIND_SINGH_JAYANTI", (1, 6)),
    (2026, "BRAHMO_SAMAJ_FOUNDATION_DAY", (8, 20)),
];

/// The days every notification lists on one date, Netaji's birthday, Utkal
/// Divas and Raja Rammohan Roy's among them.
const CIVIL: [(&str, MonthDay); 9] = [
    ("NETAJI_JAYANTI", (1, 23)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("UTKAL_DIVAS", (4, 1)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("RAM_MOHAN_ROY_JAYANTI", (5, 22)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("CHRISTMAS_EVE", (12, 24)),
    ("CHRISTMAS", (12, 25)),
];

/// The cause of each year's Brahmo Samaj parting, 2019 to 2025.
const BRAHMO: &str = "the rule keeps 20 August, the day the 2026 notification moves the Brahmo Samaj's foundation day to, the Samaj's first meeting in 1828; the notifications to 2025 list 25 January (C1492)";

/// Why a rule parts from a notification, per (year, rule).
const PARTS: [(i32, &str, &str); 22] = [
    (2019, "BRAHMO_SAMAJ_FOUNDATION_DAY", BRAHMO),
    (
        2020,
        "PAHILI_RAJA",
        "the day before Mithuna Sankranti, which the Bengal table puts on 14 June, so the rule takes the 13th; the notification lists the 14th (C1482)",
    ),
    (
        2020,
        "MITHUNA_SANKRANTI",
        "the Sun enters Gemini at 00:01 on 15 June by the built-in sky, but the generated Bengal table, whose ingress falls before midnight, closes Jyeshtha on the 14th and the rule takes it; the notification lists the 15th (C1482)",
    ),
    (2020, "BRAHMO_SAMAJ_FOUNDATION_DAY", BRAHMO),
    (2021, "BRAHMO_SAMAJ_FOUNDATION_DAY", BRAHMO),
    (2022, "BRAHMO_SAMAJ_FOUNDATION_DAY", BRAHMO),
    (
        2023,
        "GURU_GOBIND_SINGH_JAYANTI",
        "Pausha's bright 7th fell on 29 December 2022, so the rule finds none in 2023; the notification lists 5 January (C1491)",
    ),
    (2023, "BRAHMO_SAMAJ_FOUNDATION_DAY", BRAHMO),
    (
        2024,
        "MESHA_SANKRANTI",
        "the Sun enters Aries at 21:06 on 13 April, and the rule takes the civil day of the ingress; the notification lists the 14th, the first sunrise after it (C1483)",
    ),
    (2024, "BRAHMO_SAMAJ_FOUNDATION_DAY", BRAHMO),
    (
        2025,
        "BASANTA_PANCHAMI",
        "the bright 5th runs from 09:14 on 2 February to 06:53 on the 3rd, holding the 3rd's sunrise, and the rule takes the 3rd; the notification lists the 2nd, whose forenoon the 5th holds (C1486)",
    ),
    (
        2025,
        "KUMAR_PURNIMA",
        "the full moon runs from 12:24 on 6 October to 09:17 on the 7th, holding the 6th's moonrise, and the rule takes the 6th; the notification lists the 7th, whose sunrise the full moon holds (C1487)",
    ),
    (
        2025,
        "KALI_PUJA",
        "the new moon runs from 15:45 on 20 October to 17:55 on the 21st, holding the 20th's midnight alone, and the rule takes the 20th; the notification lists the 21st, though 2024's lists the 31st October on the same reading (C1488)",
    ),
    (2025, "BRAHMO_SAMAJ_FOUNDATION_DAY", BRAHMO),
    (
        2026,
        "RAMA_NAVAMI",
        "the bright 9th runs from 11:49 on 26 March to 10:07 on the 27th, holding the 26th's midday, and the central rule takes the 26th; the notification lists the 27th, whose sunrise the 9th holds (C1485)",
    ),
    (
        2026,
        "JHULANA_PURNIMA",
        "the full moon runs from 09:09 on 27 August to 09:48 on the 28th, and Dharmasindhu's rule takes the 28th; the notification lists the 27th, whose midday the full moon holds (C1489)",
    ),
    (
        2026,
        "PHULPATI",
        "the bright 7th begins at 05:54 on 17 October, and the sunrise reading takes the 18th; the notification lists the 17th, as for each Durga day after it (C1484)",
    ),
    (
        2026,
        "MAHA_ASHTAMI",
        "the bright 8th runs from 08:28 on 18 October, after sunrise, and the sunrise reading takes the 19th; the notification lists the 18th (C1484)",
    ),
    (
        2026,
        "MAHA_NAVAMI",
        "the bright 9th runs from 10:52 on 19 October, after sunrise, and the sunrise reading takes the 20th; the notification lists the 19th (C1484)",
    ),
    (
        2026,
        "VIJAYA_DASHAMI",
        "counted a day after Maha Navami, so the rule takes the 21st; the notification lists the 20th, the day the bright 10th begins on, at 12:50 (C1484)",
    ),
    (
        2026,
        "ANLA_NAVAMI",
        "the bright 9th runs from 06:05 on 18 November to 07:06 on the 19th, holding the 19th's sunrise, and the rule takes the 19th; the notification lists the 18th (C1490)",
    ),
    (
        2026,
        "GURU_GOBIND_SINGH_JAYANTI",
        "Pausha's bright 7th fell on 27 December 2025, so the rule finds none in 2026; the notification lists 6 January (C1491)",
    ),
];

/// The days each year's notification prints rather than reckons: the four
/// Islamic days, which no notification leaves to the moon (C1494), with
/// Shab-e-Qader where listed, Panchayati Raj Divas to 2025, Mahaveer
/// Jayanti to 2021, and 2026's day following Nuakhai.
const PRINTED: [usize; 8] = [7, 7, 7, 6, 5, 6, 6, 6];

/// The page's section on the list.
const SECTION: Section = Section {
    heading: "21. Odisha's public holidays against the notifications",
    against: "`INDIA_ODISHA` at Bhubaneswar on India's clock, against each year's general \
         holidays of the Revenue and Disaster Management Department's notification \
         (Annexure-I, and Annexure-III's days on a Sunday or a second or fourth Saturday) \
         and Annexure-II's optional holidays. The days are reckoned under the modern sky \
         (the built-in ephemeris, the default profile).",
    civil: "civil",
    printed: "The days the notifications print are not reckoned: the four Islamic days \
         (Id-ul-Fitre, Id-ul-Zuha, Muharram and the Prophet's birthday), which no \
         notification leaves to the moon, and Shab-e-Qader where listed, all unmarked; \
         Panchayati Raj Divas, listed to 2025; Mahaveer Jayanti, listed to 2021; and 2026's \
         day following Nuakhai. 2031, with no notification, holds none.",
};

/// Bhubaneswar, the seat the notifications are issued from: the Lok Seva
/// Bhavan.
fn bhubaneswar() -> Place {
    Place::new(
        Latitude::literal(20.2961),
        Longitude::literal(85.8245),
        Altitude::literal(0.0),
    )
}

/// The state's list.
fn odisha() -> Notices {
    Notices {
        pack: FestivalPack::IndiaOdisha,
        sky: Sky::Drik,
        place: bhubaneswar(),
        years: &NOTICES,
        listed: &LISTED,
        civil: &CIVIL,
        parts: &PARTS,
        printed: &PRINTED,
        unnoticed: 2031,
    }
}

/// Every year's rows, and a problem for every parting no entry names.
pub(super) fn compare(problems: &mut Vec<String>) -> Result<Measured, String> {
    odisha().compare(problems)
}

/// §21 of the page.
pub(super) fn render(out: &mut String, measured: &Measured) {
    odisha().render(out, &SECTION, measured);
}

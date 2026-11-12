//! The `INDIA_TELANGANA` pack against the Government of Telangana's G.O.s
//! (`03-design/telangana-holidays.md`, SR-03), as [`super::karnataka`]
//! holds Karnataka's, through the same comparison.
//!
//! Every rule of the pack, at Hyderabad on India's clock, against each
//! year's general holidays (Annexure-I and its Part II of the days on a
//! Sunday or a second Saturday) and Annexure-II's optional holidays; every
//! parting is named with its cause, which the pass checks both ways. The
//! days the G.O.s print rather than reckon are held only to stand in their
//! year, the moon's marked, and a year with no G.O. holds none.

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Section, Sky};

/// Each year's G.O. of the General Administration (Spl.E) Department, and
/// the copy read.
const NOTICES: [(i32, &str); 8] = [
    (
        2019,
        "G.O.Rt.No.2493, 16 November 2018: rank 1 as copied, the G.O.'s text as JNTU Hyderabad republished it",
    ),
    (
        2020,
        "G.O.Rt.No.3022, 21 November 2019: rank 1 as copied, the signed scan in JNTU Hyderabad's circular",
    ),
    (
        2021,
        "G.O.Rt.No.1756, 9 November 2020: rank 1 as copied, the signed scan in JNTU Hyderabad's circular",
    ),
    (
        2022,
        "G.O.Rt.No.2618, 26 November 2021: rank 1 as copied, a staff site's copy",
    ),
    (
        2023,
        "G.O.Rt.No.2140, 15 November 2022: rank 1 as copied, the signed scan",
    ),
    (
        2024,
        "G.O.Rt.No.1633, 12 December 2023: rank 1 as copied, a staff site's copy",
    ),
    (
        2025,
        "G.O.Rt.No.1479, 8 November 2024: rank 1 as copied, the signed scan",
    ),
    (
        2026,
        "G.O.Rt.No.1715, 6 December 2025: rank 1 as copied, the signed scan",
    ),
];

/// Each year's reckoned days, general and optional, less the days every
/// G.O. lists on one date, which [`CIVIL`] holds in every year.
const LISTED: [(i32, &str, MonthDay); 193] = [
    (2019, "BHOGI", (1, 14)),
    (2019, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2019, "KANUMU", (1, 16)),
    (2019, "SHIVARATRI", (3, 4)),
    (2019, "HOLI", (3, 21)),
    (2019, "SAMVATSARA_ARAMBHA", (4, 6)),
    (2019, "RAMA_NAVAMI", (4, 14)),
    (2019, "GOOD_FRIDAY", (4, 19)),
    (2019, "JANMASHTAMI_VAISHNAVA", (8, 24)),
    (2019, "GANESHA_CHATURTHI", (9, 2)),
    (2019, "BATHUKAMMA", (9, 28)),
    (2019, "MAHA_ASHTAMI", (10, 6)),
    (2019, "MAHA_NAVAMI", (10, 7)),
    (2019, "VIJAYA_DASHAMI", (10, 8)),
    (2019, "LAKSHMI_PUJA", (10, 27)),
    (2019, "GURU_NANAK_JAYANTI", (11, 12)),
    (2019, "BASANTA_PANCHAMI", (2, 10)),
    (2019, "JAGANNATH_RATH_JATRA", (7, 4)),
    (2019, "MAHAVIR_JAYANTI", (4, 17)),
    (2019, "BASAVA_JAYANTI", (5, 7)),
    (2019, "BUDDHA_JAYANTI", (5, 18)),
    (2020, "BHOGI", (1, 14)),
    (2020, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2020, "KANUMU", (1, 16)),
    (2020, "SHIVARATRI", (2, 21)),
    (2020, "HOLI", (3, 9)),
    (2020, "SAMVATSARA_ARAMBHA", (3, 25)),
    (2020, "RAMA_NAVAMI", (4, 2)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "JANMASHTAMI_VAISHNAVA", (8, 11)),
    (2020, "GANESHA_CHATURTHI", (8, 22)),
    (2020, "BATHUKAMMA", (10, 17)),
    (2020, "MAHA_ASHTAMI", (10, 24)),
    (2020, "MAHA_NAVAMI", (10, 24)),
    (2020, "VIJAYA_DASHAMI", (10, 25)),
    (2020, "NARAK_CHATURDASHI", (11, 14)),
    (2020, "LAKSHMI_PUJA", (11, 14)),
    (2020, "GURU_NANAK_JAYANTI", (11, 30)),
    (2020, "BASANTA_PANCHAMI", (1, 30)),
    (2020, "JAGANNATH_RATH_JATRA", (6, 23)),
    (2020, "RAKSHABANDHAN", (8, 3)),
    (2020, "MAHAVIR_JAYANTI", (4, 6)),
    (2020, "BASAVA_JAYANTI", (4, 26)),
    (2020, "BUDDHA_JAYANTI", (5, 7)),
    (2021, "BHOGI", (1, 13)),
    (2021, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2021, "KANUMU", (1, 15)),
    (2021, "SHIVARATRI", (3, 11)),
    (2021, "HOLI", (3, 29)),
    (2021, "SAMVATSARA_ARAMBHA", (4, 13)),
    (2021, "RAMA_NAVAMI", (4, 21)),
    (2021, "GOOD_FRIDAY", (4, 2)),
    (2021, "JANMASHTAMI_VAISHNAVA", (8, 31)),
    (2021, "GANESHA_CHATURTHI", (9, 10)),
    (2021, "BATHUKAMMA", (10, 6)),
    (2021, "MAHA_ASHTAMI", (10, 13)),
    (2021, "MAHA_NAVAMI", (10, 14)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "VIJAYA_DASHAMI_NEXT_DAY", (10, 16)),
    (2021, "NARAK_CHATURDASHI", (11, 3)),
    (2021, "LAKSHMI_PUJA", (11, 4)),
    (2021, "GURU_NANAK_JAYANTI", (11, 19)),
    (2021, "BASANTA_PANCHAMI", (2, 16)),
    (2021, "JAGANNATH_RATH_JATRA", (7, 12)),
    (2021, "RAKSHABANDHAN", (8, 22)),
    (2021, "MAHAVIR_JAYANTI", (4, 25)),
    (2021, "BASAVA_JAYANTI", (5, 14)),
    (2021, "BUDDHA_JAYANTI", (5, 26)),
    (2021, "PUTHANDU", (4, 14)),
    (2022, "BHOGI", (1, 14)),
    (2022, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2022, "KANUMU", (1, 16)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "HOLI", (3, 18)),
    (2022, "SAMVATSARA_ARAMBHA", (4, 2)),
    (2022, "RAMA_NAVAMI", (4, 10)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "JANMASHTAMI_VAISHNAVA", (8, 20)),
    (2022, "GANESHA_CHATURTHI", (8, 31)),
    (2022, "BATHUKAMMA", (9, 25)),
    (2022, "MAHA_ASHTAMI", (10, 3)),
    (2022, "MAHA_NAVAMI", (10, 4)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "VIJAYA_DASHAMI_NEXT_DAY", (10, 6)),
    (2022, "NARAK_CHATURDASHI", (10, 24)),
    (2022, "LAKSHMI_PUJA", (10, 25)),
    (2022, "GURU_NANAK_JAYANTI", (11, 8)),
    (2022, "BASANTA_PANCHAMI", (2, 5)),
    (2022, "JAGANNATH_RATH_JATRA", (7, 1)),
    (2022, "RAKSHABANDHAN", (8, 12)),
    (2022, "MAHAVIR_JAYANTI", (4, 14)),
    (2022, "BASAVA_JAYANTI", (5, 3)),
    (2022, "BUDDHA_JAYANTI", (5, 16)),
    (2022, "PUTHANDU", (4, 14)),
    (2023, "BHOGI", (1, 14)),
    (2023, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2023, "KANUMU", (1, 16)),
    (2023, "SHIVARATRI", (2, 18)),
    (2023, "HOLI", (3, 7)),
    (2023, "SAMVATSARA_ARAMBHA", (3, 22)),
    (2023, "RAMA_NAVAMI", (3, 30)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "JANMASHTAMI_VAISHNAVA", (9, 7)),
    (2023, "GANESHA_CHATURTHI", (9, 18)),
    (2023, "BATHUKAMMA", (10, 14)),
    (2023, "MAHA_ASHTAMI", (10, 22)),
    (2023, "MAHA_NAVAMI", (10, 23)),
    (2023, "VIJAYA_DASHAMI", (10, 24)),
    (2023, "VIJAYA_DASHAMI_NEXT_DAY", (10, 25)),
    (2023, "NARAK_CHATURDASHI", (11, 11)),
    (2023, "LAKSHMI_PUJA", (11, 12)),
    (2023, "GURU_NANAK_JAYANTI", (11, 27)),
    (2023, "BASANTA_PANCHAMI", (1, 26)),
    (2023, "JAGANNATH_RATH_JATRA", (6, 20)),
    (2023, "RAKSHABANDHAN", (8, 31)),
    (2023, "MAHAVIR_JAYANTI", (4, 4)),
    (2023, "BASAVA_JAYANTI", (4, 23)),
    (2023, "BUDDHA_JAYANTI", (5, 5)),
    (2023, "PUTHANDU", (4, 14)),
    (2024, "BHOGI", (1, 14)),
    (2024, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2024, "KANUMU", (1, 16)),
    (2024, "SHIVARATRI", (3, 8)),
    (2024, "HOLI", (3, 25)),
    (2024, "SAMVATSARA_ARAMBHA", (4, 9)),
    (2024, "RAMA_NAVAMI", (4, 17)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "JANMASHTAMI_VAISHNAVA", (8, 26)),
    (2024, "GANESHA_CHATURTHI", (9, 7)),
    (2024, "BATHUKAMMA", (10, 2)),
    (2024, "MAHA_ASHTAMI", (10, 10)),
    (2024, "MAHA_NAVAMI", (10, 11)),
    (2024, "VIJAYA_DASHAMI", (10, 12)),
    (2024, "VIJAYA_DASHAMI_NEXT_DAY", (10, 13)),
    (2024, "NARAK_CHATURDASHI", (10, 30)),
    (2024, "LAKSHMI_PUJA", (10, 31)),
    (2024, "GURU_NANAK_JAYANTI", (11, 15)),
    (2024, "BASANTA_PANCHAMI", (2, 14)),
    (2024, "JAGANNATH_RATH_JATRA", (7, 7)),
    (2024, "RAKSHABANDHAN", (8, 19)),
    (2024, "MAHAVIR_JAYANTI", (4, 21)),
    (2024, "BASAVA_JAYANTI", (5, 10)),
    (2024, "BUDDHA_JAYANTI", (5, 23)),
    (2024, "PUTHANDU", (4, 14)),
    (2025, "BHOGI", (1, 13)),
    (2025, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2025, "KANUMU", (1, 15)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "HOLI", (3, 14)),
    (2025, "SAMVATSARA_ARAMBHA", (3, 30)),
    (2025, "RAMA_NAVAMI", (4, 6)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "JANMASHTAMI_VAISHNAVA", (8, 16)),
    (2025, "GANESHA_CHATURTHI", (8, 27)),
    (2025, "BATHUKAMMA", (9, 21)),
    (2025, "MAHA_ASHTAMI", (9, 30)),
    (2025, "MAHA_NAVAMI", (10, 1)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "VIJAYA_DASHAMI_NEXT_DAY", (10, 3)),
    (2025, "NARAK_CHATURDASHI", (10, 19)),
    (2025, "LAKSHMI_PUJA", (10, 20)),
    (2025, "GURU_NANAK_JAYANTI", (11, 5)),
    (2025, "BASANTA_PANCHAMI", (2, 3)),
    (2025, "JAGANNATH_RATH_JATRA", (6, 27)),
    (2025, "RAKSHABANDHAN", (8, 9)),
    (2025, "MAHAVIR_JAYANTI", (4, 10)),
    (2025, "BASAVA_JAYANTI", (4, 30)),
    (2025, "BUDDHA_JAYANTI", (5, 12)),
    (2025, "PUTHANDU", (4, 14)),
    (2026, "BHOGI", (1, 14)),
    (2026, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2026, "KANUMU", (1, 16)),
    (2026, "SHIVARATRI", (2, 15)),
    (2026, "HOLI", (3, 3)),
    (2026, "SAMVATSARA_ARAMBHA", (3, 19)),
    (2026, "RAMA_NAVAMI", (3, 27)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "JANMASHTAMI_VAISHNAVA", (9, 4)),
    (2026, "GANESHA_CHATURTHI", (9, 14)),
    (2026, "SADDULA_BATHUKAMMA", (10, 18)),
    (2026, "MAHA_NAVAMI", (10, 19)),
    (2026, "VIJAYA_DASHAMI", (10, 20)),
    (2026, "VIJAYA_DASHAMI_NEXT_DAY", (10, 21)),
    (2026, "NARAK_CHATURDASHI", (11, 8)),
    (2026, "LAKSHMI_PUJA", (11, 8)),
    (2026, "GURU_NANAK_JAYANTI", (11, 24)),
    (2026, "BASANTA_PANCHAMI", (1, 23)),
    (2026, "JAGANNATH_RATH_JATRA", (7, 16)),
    (2026, "RAKSHABANDHAN", (8, 28)),
    (2026, "MAHAVIR_JAYANTI", (3, 31)),
    (2026, "BASAVA_JAYANTI", (4, 20)),
    (2026, "BUDDHA_JAYANTI", (5, 1)),
    (2026, "PUTHANDU", (4, 14)),
];

/// The days every G.O. lists on one date, New Year's Day and Christmas
/// Eve among them.
const CIVIL: [(&str, MonthDay); 9] = [
    ("NEW_YEAR_S_DAY_AD", (1, 1)),
    ("INDIA_REPUBLIC_DAY", (1, 26)),
    ("JAGJIVAN_RAM_JAYANTI", (4, 5)),
    ("AMBEDKAR_JAYANTI", (4, 14)),
    ("INDIA_INDEPENDENCE_DAY", (8, 15)),
    ("GANDHI_JAYANTI", (10, 2)),
    ("CHRISTMAS_EVE", (12, 24)),
    ("CHRISTMAS", (12, 25)),
    ("BOXING_DAY", (12, 26)),
];

/// Why a rule parts from a G.O., per (year, rule).
const PARTS: [(i32, &str, &str); 11] = [
    (
        2019,
        "MAHA_ASHTAMI",
        "the bright 8th runs from 09:50 on 5 October to 10:54 on the 6th, before the 6th's midday, and the midday reading takes the 5th; the G.O. lists the 6th, whose sunrise the ashtami holds, a Sunday named in Part II (C1393)",
    ),
    (
        2019,
        "MAHA_NAVAMI",
        "the bright 9th runs from 10:54 on 6 October to 12:38 on the 7th, holding both middays, and the midday reading keeps the earlier, the 6th; the G.O. lists the 7th, whose sunrise the navami holds (C1393)",
    ),
    (
        2019,
        "VIJAYA_DASHAMI",
        "the bright 10th runs from 12:38 on 7 October to 14:50 on the 8th, holding both middays, and the midday reading keeps the earlier, the 7th; the G.O. lists the 8th, whose sunrise the dashami holds (C1393)",
    ),
    (
        2020,
        "HOLI",
        "the full moon runs from 03:04 on 9 March to 23:17, so the central list's Holika fire is the 9th and Holi the 10th; the G.O. lists Holi on the 9th, the full moon's day (C1398)",
    ),
    (
        2020,
        "JANMASHTAMI_VAISHNAVA",
        "Rohini of 13 August falls in Adi, before the Sun enters Simha, and Avani's Rohini is 10 September; the G.O. lists 11 August, the dark 8th's day (09:07 on the 11th to 11:16 on the 12th), not yet named for the Srivaishnava Agamam (C1394)",
    ),
    (
        2020,
        "BATHUKAMMA",
        "Bhadrapada's new moon runs from 19:57 on 16 September to 16:30 on the 17th, and the rule takes the 17th; Ashvina is doubled, and the G.O. lists 17 October, the first day of the nija Ashvina, after the adhika month's new moon (C1395)",
    ),
    (
        2020,
        "MAHA_ASHTAMI",
        "the bright 8th runs from 06:57 on 23 October to 06:59 on the 24th, and the midday reading takes the 23rd; the G.O. lists the 24th, whose sunrise the ashtami holds by minutes, on one day with Maharnavami (C1393)",
    ),
    (
        2022,
        "LAKSHMI_PUJA",
        "the new moon runs from 17:27 on 24 October to 16:18 on the 25th, ending before the 25th's sunset, so the evening rule takes the 24th; the G.O. lists the 25th, whose sunrise the new moon holds (C1396)",
    ),
    (
        2024,
        "JANMASHTAMI_VAISHNAVA",
        "Rohini runs from 15:55 on 26 August to 15:38 on the 27th, and the rule takes the 27th, whose sunrise it holds; the G.O. lists the 26th, the dark 8th's day (03:39 on the 26th to 02:20 on the 27th) (C1394)",
    ),
    (
        2025,
        "JANMASHTAMI_VAISHNAVA",
        "Avani holds Rohini twice, the earlier from 04:38 on 17 August to 03:17 on the 18th, and the rule takes the 17th; the G.O. lists the 16th, the dark 8th's day (23:50 on the 15th to 21:35 on the 16th) (C1394)",
    ),
    (
        2026,
        "NARAK_CHATURDASHI",
        "the dark 14th runs from 10:48 on 7 November to 11:28 on the 8th, and the afternoon rule takes the 7th; the G.O. lists the 8th, whose dawn the 14th holds, on one day with Deepavali (C1397)",
    ),
];

/// The days each year's G.O. prints rather than reckons: the five general
/// days set by the moon, Annexure-II's eleven (Hazrath Ali's birthday twice
/// in 2026), and Bonalu, Varalakshmi Vratham and the Parsi New Year.
const PRINTED: [usize; 8] = [19, 19, 19, 19, 19, 19, 19, 20];

/// The page's section on the list.
const SECTION: Section = Section {
    heading: "16. Telangana's public holidays against the G.O.s",
    against: "`INDIA_TELANGANA` at Hyderabad on India's clock, against each year's general \
         holidays of the General Administration (Spl.E) Department's G.O. (Annexure-I and its \
         Part II of the days on a Sunday or a second Saturday) and Annexure-II's optional \
         holidays. The days are reckoned under the modern sky (the built-in ephemeris, the \
         default profile).",
    civil: "civil",
    printed: "The days the G.O.s print are not reckoned: the five general days set by the moon \
         (Ramzan and its following day, Bakrid, the 10th of Moharam and Eid Miladun Nabi) and \
         Annexure-II's eleven, each marked; and Bonalu, Varalakshmi Vratham and the Parsi New \
         Year, as printed. 2031, with no G.O., holds none.",
};

/// Hyderabad, the seat the G.O.s are issued from.
fn hyderabad() -> Place {
    Place::new(
        Latitude::literal(17.385),
        Longitude::literal(78.4867),
        Altitude::literal(0.0),
    )
}

/// The state's list.
fn telangana() -> Notices {
    Notices {
        pack: FestivalPack::IndiaTelangana,
        sky: Sky::Drik,
        place: hyderabad(),
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
    telangana().compare(problems)
}

/// §16 of the page.
pub(super) fn render(out: &mut String, measured: &Measured) {
    telangana().render(out, &SECTION, measured);
}

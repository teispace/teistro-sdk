//! The `INDIA_ANDHRA_PRADESH` pack against the Government of Andhra
//! Pradesh's G.O.s (`03-design/andhra-pradesh-holidays.md`, SR-03), as
//! [`super::telangana`] holds Telangana's, through the same comparison.
//!
//! Every rule of the pack, at Amaravati on India's clock, against each
//! year's general holidays (Annexure-I and its I(A) of the days on a
//! Sunday or a second Saturday) and Annexure-II's optional holidays; every
//! parting is named with its cause, which the pass checks both ways. The
//! days the G.O.s print rather than reckon are held only to stand in their
//! year, the moon's marked, and a year with no G.O. holds none.

use teistro::FestivalPack;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};

use super::MonthDay;
use super::india::{Measured, Notices, Section, Sky};

/// Each year's G.O. of the General Administration (Poll.B) Department,
/// and the copy read.
const NOTICES: [(i32, &str); 7] = [
    (
        2020,
        "G.O.Rt.No.2745, 5 December 2019: rank 1 as copied, its annexures in RGUKT's proceedings",
    ),
    (
        2021,
        "G.O.Rt.No.1963, 15 December 2020: rank 1 as copied, the G.O.'s text",
    ),
    (
        2022,
        "G.O.Rt.No.1997, 26 November 2021: rank 1 as copied, the signed scan",
    ),
    (
        2023,
        "G.O.Rt.No.2692, 15 December 2022: rank 1 as copied, the signed scan in RGUKT's copy",
    ),
    (
        2024,
        "G.O.Rt.No.2318, 30 November 2023: rank 1 as copied, the attested scan",
    ),
    (
        2025,
        "G.O.Rt.No.2115, 6 December 2024: rank 1 as copied, the G.O.'s text",
    ),
    (
        2026,
        "G.O.Rt.No.2276, 4 December 2025: rank 3, its annexures as a teachers' site transcribed them (C1402)",
    ),
];

/// Each year's reckoned days, general and optional, less the days every
/// G.O. lists on one date, which [`CIVIL`] holds in every year.
const LISTED: [(i32, &str, MonthDay); 135] = [
    (2020, "BHOGI", (1, 14)),
    (2020, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2020, "KANUMU", (1, 16)),
    (2020, "SHIVARATRI", (2, 21)),
    (2020, "HOLI", (3, 10)),
    (2020, "SAMVATSARA_ARAMBHA", (3, 25)),
    (2020, "RAMA_NAVAMI", (4, 2)),
    (2020, "GOOD_FRIDAY", (4, 10)),
    (2020, "JANMASHTAMI", (8, 11)),
    (2020, "GANESHA_CHATURTHI", (8, 22)),
    (2020, "MAHALAYA_AMAVASYA", (9, 17)),
    (2020, "MAHA_ASHTAMI", (10, 24)),
    (2020, "VIJAYA_DASHAMI", (10, 25)),
    (2020, "LAKSHMI_PUJA", (11, 14)),
    (2020, "GURU_NANAK_JAYANTI", (11, 30)),
    (2020, "JAGANNATH_RATH_JATRA", (6, 23)),
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
    (2021, "JANMASHTAMI", (8, 30)),
    (2021, "GANESHA_CHATURTHI", (9, 10)),
    (2021, "MAHALAYA_AMAVASYA", (10, 6)),
    (2021, "MAHA_ASHTAMI", (10, 13)),
    (2021, "VIJAYA_DASHAMI", (10, 15)),
    (2021, "NARAK_CHATURDASHI", (11, 3)),
    (2021, "LAKSHMI_PUJA", (11, 4)),
    (2021, "GURU_NANAK_JAYANTI", (11, 19)),
    (2021, "JAGANNATH_RATH_JATRA", (7, 12)),
    (2021, "MAHAVIR_JAYANTI", (4, 25)),
    (2021, "BASAVA_JAYANTI", (5, 14)),
    (2021, "BUDDHA_JAYANTI", (5, 26)),
    (2022, "BHOGI", (1, 14)),
    (2022, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2022, "KANUMU", (1, 16)),
    (2022, "SHIVARATRI", (3, 1)),
    (2022, "HOLI", (3, 18)),
    (2022, "SAMVATSARA_ARAMBHA", (4, 2)),
    (2022, "RAMA_NAVAMI", (4, 10)),
    (2022, "GOOD_FRIDAY", (4, 15)),
    (2022, "JANMASHTAMI", (8, 19)),
    (2022, "GANESHA_CHATURTHI", (8, 31)),
    (2022, "MAHALAYA_AMAVASYA", (9, 25)),
    (2022, "MAHA_ASHTAMI", (10, 3)),
    (2022, "VIJAYA_DASHAMI", (10, 5)),
    (2022, "NARAK_CHATURDASHI", (10, 23)),
    (2022, "LAKSHMI_PUJA", (10, 24)),
    (2022, "GURU_NANAK_JAYANTI", (11, 8)),
    (2022, "JAGANNATH_RATH_JATRA", (7, 1)),
    (2022, "MAHAVIR_JAYANTI", (4, 14)),
    (2022, "BASAVA_JAYANTI", (5, 3)),
    (2022, "BUDDHA_JAYANTI", (5, 16)),
    (2023, "BHOGI", (1, 14)),
    (2023, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2023, "KANUMU", (1, 16)),
    (2023, "SHIVARATRI", (2, 18)),
    (2023, "HOLI", (3, 8)),
    (2023, "SAMVATSARA_ARAMBHA", (3, 22)),
    (2023, "RAMA_NAVAMI", (3, 30)),
    (2023, "GOOD_FRIDAY", (4, 7)),
    (2023, "JANMASHTAMI", (9, 6)),
    (2023, "GANESHA_CHATURTHI", (9, 18)),
    (2023, "MAHALAYA_AMAVASYA", (10, 14)),
    (2023, "MAHA_ASHTAMI", (10, 22)),
    (2023, "VIJAYA_DASHAMI", (10, 23)),
    (2023, "LAKSHMI_PUJA", (11, 12)),
    (2023, "GURU_NANAK_JAYANTI", (11, 27)),
    (2023, "JAGANNATH_RATH_JATRA", (6, 20)),
    (2023, "MAHAVIR_JAYANTI", (4, 4)),
    (2023, "BASAVA_JAYANTI", (4, 23)),
    (2023, "BUDDHA_JAYANTI", (5, 5)),
    (2024, "BHOGI", (1, 14)),
    (2024, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2024, "KANUMU", (1, 16)),
    (2024, "SHIVARATRI", (3, 8)),
    (2024, "HOLI", (3, 25)),
    (2024, "SAMVATSARA_ARAMBHA", (4, 9)),
    (2024, "RAMA_NAVAMI", (4, 17)),
    (2024, "GOOD_FRIDAY", (3, 29)),
    (2024, "JANMASHTAMI", (8, 26)),
    (2024, "GANESHA_CHATURTHI", (9, 7)),
    (2024, "MAHALAYA_AMAVASYA", (10, 2)),
    (2024, "MAHA_ASHTAMI", (10, 11)),
    (2024, "VIJAYA_DASHAMI", (10, 12)),
    (2024, "LAKSHMI_PUJA", (10, 31)),
    (2024, "GURU_NANAK_JAYANTI", (11, 15)),
    (2024, "JAGANNATH_RATH_JATRA", (7, 7)),
    (2024, "MAHAVIR_JAYANTI", (4, 21)),
    (2024, "BASAVA_JAYANTI", (5, 10)),
    (2024, "BUDDHA_JAYANTI", (5, 23)),
    (2025, "BHOGI", (1, 13)),
    (2025, "UTTARAYANA_PUNYAKALA", (1, 14)),
    (2025, "KANUMU", (1, 15)),
    (2025, "SHIVARATRI", (2, 26)),
    (2025, "HOLI", (3, 14)),
    (2025, "SAMVATSARA_ARAMBHA", (3, 30)),
    (2025, "RAMA_NAVAMI", (4, 6)),
    (2025, "GOOD_FRIDAY", (4, 18)),
    (2025, "JANMASHTAMI", (8, 16)),
    (2025, "GANESHA_CHATURTHI", (8, 27)),
    (2025, "MAHALAYA_AMAVASYA", (9, 21)),
    (2025, "MAHA_ASHTAMI", (9, 30)),
    (2025, "VIJAYA_DASHAMI", (10, 2)),
    (2025, "LAKSHMI_PUJA", (10, 20)),
    (2025, "GURU_NANAK_JAYANTI", (11, 5)),
    (2025, "JAGANNATH_RATH_JATRA", (6, 27)),
    (2025, "MAHAVIR_JAYANTI", (4, 10)),
    (2025, "BASAVA_JAYANTI", (4, 30)),
    (2025, "BUDDHA_JAYANTI", (5, 12)),
    (2026, "BHOGI", (1, 14)),
    (2026, "UTTARAYANA_PUNYAKALA", (1, 15)),
    (2026, "KANUMU", (1, 16)),
    (2026, "SHIVARATRI", (2, 15)),
    (2026, "HOLI", (3, 3)),
    (2026, "SAMVATSARA_ARAMBHA", (3, 19)),
    (2026, "RAMA_NAVAMI", (3, 27)),
    (2026, "GOOD_FRIDAY", (4, 3)),
    (2026, "JANMASHTAMI", (9, 4)),
    (2026, "GANESHA_CHATURTHI", (9, 14)),
    (2026, "MAHALAYA_AMAVASYA", (10, 10)),
    (2026, "MAHA_ASHTAMI", (10, 18)),
    (2026, "VIJAYA_DASHAMI", (10, 20)),
    (2026, "LAKSHMI_PUJA", (11, 8)),
    (2026, "GURU_NANAK_JAYANTI", (11, 24)),
    (2026, "JAGANNATH_RATH_JATRA", (7, 16)),
    (2026, "MAHAVIR_JAYANTI", (3, 31)),
    (2026, "BASAVA_JAYANTI", (4, 20)),
    (2026, "BUDDHA_JAYANTI", (5, 1)),
];

/// The days every G.O. lists on one date, New Year's Day, Christmas Eve
/// and Boxing Day among them.
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
const PARTS: [(i32, &str, &str); 8] = [
    (
        2020,
        "MAHA_ASHTAMI",
        "the bright 8th runs from 06:57 on 23 October to 06:59 on the 24th, and the midday reading takes the 23rd; the G.O. lists the 24th, whose sunrise the ashtami holds, as Telangana's does (C1401, C1393)",
    ),
    (
        2021,
        "BASAVA_JAYANTI",
        "the bright 3rd begins at 05:39 on 14 May, a minute and a half after Amaravati's sunrise (05:38) and before Hyderabad's (05:44), so the print's rule takes the 15th here and the 14th there; the G.O. lists the 14th (C1407)",
    ),
    (
        2022,
        "JANMASHTAMI",
        "the dark 8th runs from 21:21 on 18 August to 22:59 on the 19th, holding the 18th's midnight, and the rule takes the 18th; the G.O. lists the 19th, whose sunrise the ashtami holds (C1406)",
    ),
    (
        2022,
        "NARAK_CHATURDASHI",
        "the dark 14th runs from 18:03 on 23 October to 17:27 on the 24th, and the afternoon rule takes the 24th, Deepavali's day; the G.O. lists the 23rd, whose evening the 14th begins in (C1404)",
    ),
    (
        2023,
        "HOLI",
        "the full moon runs from 16:17 on 6 March to 18:10 on the 7th, so the central list's Holika fire is the 6th and Holi the 7th; the G.O. lists Holi on the 8th, the first day whose sunrise the dark 1st holds (C1405)",
    ),
    (
        2023,
        "VIJAYA_DASHAMI",
        "the bright 10th runs from 17:45 on 23 October to 15:14 on the 24th, holding the 24th's midday alone, and the rule takes the 24th; the G.O. lists the 23rd as the general holiday and the 24th as an optional \"Vijayadasami (Tidhi Dwayam)\" (C1401)",
    ),
    (
        2024,
        "MAHA_ASHTAMI",
        "the bright 8th runs from 12:32 on 10 October to 12:07 on the 11th, and the midday reading takes the 10th, as Telangana's G.O. lists it; this G.O. lists the 11th, whose sunrise the ashtami holds (C1401)",
    ),
    (
        2025,
        "JANMASHTAMI",
        "the dark 8th runs from 23:50 on 15 August to 21:35 on the 16th, holding the 15th's midnight, and the rule takes the 15th; the G.O. lists the 16th, whose sunrise the ashtami holds (C1406)",
    ),
];

/// The days each year's G.O. prints rather than reckons: the four general
/// days set by the moon (and 2021's Shab-e-Barath), Annexure-II's Islamic
/// days (2023's Shab-e-Meraj a general holiday), Varalakshmi Vratham, the
/// Parsi New Year, and 2023's second Vijayadasami.
const PRINTED: [usize; 7] = [17, 17, 17, 18, 17, 17, 17];

/// The page's section on the list.
const SECTION: Section = Section {
    heading: "18. Andhra Pradesh's public holidays against the G.O.s",
    against: "`INDIA_ANDHRA_PRADESH` at Amaravati on India's clock, against each year's \
         general holidays of the General Administration (Poll.B) Department's G.O. \
         (Annexure-I and its I(A) of the days on a Sunday or a second Saturday) and \
         Annexure-II's optional holidays with its II(A). The days are reckoned under the \
         modern sky (the built-in ephemeris, the default profile).",
    civil: "civil",
    printed: "The days the G.O.s print are not reckoned: the four general days set by the \
         moon (Ramzan, Bakrid, Moharrum and Eid Miladun Nabi) and 2021's Shab-e-Barath, each \
         marked, as para 5 leaves them to the moon; and, as printed, Annexure-II's Islamic \
         days (2023's Shab-e-Meraj a general holiday), Varalakshmi Vratham, the Parsi New Year \
         and 2023's second Vijayadasami. 2031, with no G.O., holds none.",
};

/// Amaravati, the seat the G.O.s are issued from: the Secretariat at
/// Velagapudi.
fn amaravati() -> Place {
    Place::new(
        Latitude::literal(16.5193),
        Longitude::literal(80.515),
        Altitude::literal(0.0),
    )
}

/// The state's list.
fn andhra_pradesh() -> Notices {
    Notices {
        pack: FestivalPack::IndiaAndhraPradesh,
        sky: Sky::Drik,
        place: amaravati(),
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
    andhra_pradesh().compare(problems)
}

/// §18 of the page.
pub(super) fn render(out: &mut String, measured: &Measured) {
    andhra_pradesh().render(out, &SECTION, measured);
}

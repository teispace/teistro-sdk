//! Telangana's public holidays (`03-design/telangana-holidays.md`, SR-03,
//! the seventh state set): the days the General Administration (Spl.E)
//! Department's yearly G.O. lists in its Annexure-I as general holidays,
//! those its Part II names as falling on a Sunday or a second Saturday,
//! and the optional holidays of its Annexure-II, an employee's choice of
//! five.
//!
//! The G.O. is rank 1 for the list. A day both this list and the central
//! list hold reuses `INDIA_CENTRAL`'s rule, and a day Nepal's printed
//! panchanga keeps reuses that rule. Telangana's lunar months are amanta:
//! Bathukamma begins on Bhadrapada's new moon, the Dasara days are
//! Ashvina's bright 8th to 10th, and Deepavali is Ashvina's new moon. The
//! Sankranti days are the last day of Dhanu and the first two of Makara,
//! Tamil New Year is Tamil Nadu's Puthandu, and Sri Krishna Astami, as
//! per the Srivaishnava Agamam, is Rohini in solar Simha (Avani).
//! `festival-measured.md` §16 holds every rule to the G.O.s at Hyderabad.
//!
//! Ramzan and its following day, Bakrid, Shahadat Imam Hussain (10th
//! Moharam) and Eid Miladun Nabi, and eleven optional days of the Islamic
//! year, are not reckoned: para 5 of each G.O. leaves their days to the
//! moon, so they ship as printed, marked [`SubjectTo::MoonSighting`].
//! Bonalu, Varalakshmi Vratham and the Parsi New Year ship as printed,
//! unmarked. Telangana Formation Day, which no G.O. lists, is not here
//! (C1392).

use teistro_core::catalogue::{Calendar, Masa, Nakshatra, Tithi, Tradition};

use super::india_central::{Civil, Named, OnDay, Own, Printed, PrintedDays, StateList, sighted};
use super::nepal_panchanga::Fit;
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{
    DatedRule, DayPart, FestivalMeta, FestivalRule, FollowingRule, PublishedSet, Which, Window,
};

/// Where the list is kept: India, then the state the G.O. names.
const REGIONS: [&str; 2] = ["IN", "IN-TG"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The central list's lunar rules this list keeps, each with whether
/// Telangana lists it as an optional holiday. The Holika fire is the day
/// before Holi, which the list keeps as the central list does.
const CENTRAL_KEPT: [(&str, bool); 5] = [
    ("SHIVARATRI", false),
    ("HOLIKA", false),
    ("MAHAVIR_JAYANTI", true),
    ("BUDDHA_JAYANTI", true),
    ("GURU_NANAK_JAYANTI", false),
];

/// Nepal's printed panchanga's rules this list keeps, by the key it keeps
/// them under, each with the print's key and whether it is optional.
const PRINT_KEPT: [(&str, &str, bool); 4] = [
    ("SAMVATSARA_ARAMBHA", "SAMVATSARA_ARAMBHA", false),
    ("BASAVA_JAYANTI", "AKSHAYA_TRITIYA", true),
    ("BASANTA_PANCHAMI", "BASANTA_PANCHAMI", true),
    ("JAGANNATH_RATH_JATRA", "JAGANNATH_RATH_JATRA", true),
];

/// The G.O. the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of Telangana, General Administration (Spl.E) Department, \
     G.O.Rt.No.1715 of 6 December 2025: Annexure-I's general holidays and Annexure-II's \
     optional holidays for 2026";

/// Each year's G.O., and the copy read.
const NOTIFICATIONS: [(i32, &str); 8] = [
    (
        2019,
        "G.O.Rt.No.2493 of 16 November 2018, rank 1 as copied (the text of the G.O. as JNTU Hyderabad republished it)",
    ),
    (
        2020,
        "G.O.Rt.No.3022 of 21 November 2019, rank 1 as copied (the signed scan, in JNTU Hyderabad's circular of 23 November 2019)",
    ),
    (
        2021,
        "G.O.Rt.No.1756 of 9 November 2020, rank 1 as copied (the signed scan, in JNTU Hyderabad's circular of 10 November 2020)",
    ),
    (
        2022,
        "G.O.Rt.No.2618 of 26 November 2021, rank 1 as copied (a staff site's copy of the G.O.)",
    ),
    (
        2023,
        "G.O.Rt.No.2140 of 15 November 2022, rank 1 as copied (the signed scan, with G.O.Rt.No.2141)",
    ),
    (
        2024,
        "G.O.Rt.No.1633 of 12 December 2023, rank 1 as copied (a staff site's copy of the G.O.)",
    ),
    (
        2025,
        "G.O.Rt.No.1479 of 8 November 2024, rank 1 as copied (the signed scan)",
    ),
    (
        2026,
        "G.O.Rt.No.1715 of 6 December 2025, rank 1 as copied (the signed scan)",
    ),
];

/// Telangana's convention for a lunar day, as a new rule cites it.
const ALMANAC: &str = "the Telugu almanacs, as the state's G.O.s of 2019 to 2026 keep the \
     day; no almanac's page is read, so the rule is held to the G.O.s and the citation is open \
     (C1389)";

/// Telangana's convention for the Dasara days, as their rules cite it.
const AT_MIDDAY: &str = "kept at midday, the later day only when it alone holds the tithi \
     there, as Vinayaka Chavithi is; the G.O.s of 2021 to 2026 keep every Dasara day so, and \
     2019's and 2020's part (C1393)";

/// The Dasara days, which [`AT_MIDDAY`] cites.
const DASARA: [&str; 4] = [
    "MAHA_ASHTAMI",
    "SADDULA_BATHUKAMMA",
    "MAHA_NAVAMI",
    "VIJAYA_DASHAMI",
];

/// Telangana's rules by the tithi of an amanta month.
const TELANGANA: [Own; 10] = [
    (
        "RAMA_NAVAMI",
        Masa::Chaitra,
        Tithi::ShuklaNavami,
        Fit::Udaya,
        false,
        "Sri Rama Navami, Chaitra's bright 9th, kept at sunrise as Karnataka's list keeps it",
    ),
    (
        "RAKSHABANDHAN",
        Masa::Shravana,
        Tithi::Purnima,
        Fit::Udaya,
        true,
        "Annexure-II's Sravana Purnima / Rakhi Purnima, Shravana's full moon",
    ),
    (
        "GANESHA_CHATURTHI",
        Masa::Bhadrapada,
        Tithi::ShuklaChaturthi,
        Fit::LaterAlone(Window::Part {
            part: DayPart::Madhyahna,
        }),
        false,
        "Vinayaka Chavithi, Bhadrapada's bright 4th, the day of the midday worship",
    ),
    (
        "BATHUKAMMA",
        Masa::Bhadrapada,
        Tithi::Amavasya,
        Fit::Udaya,
        false,
        "Bathukamma starting day (Engili Pula Bathukamma), Bhadrapada's new moon, Mahalaya Amavasya",
    ),
    (
        "MAHA_ASHTAMI",
        Masa::Ashwina,
        Tithi::ShuklaAshtami,
        Fit::LaterAlone(Window::Part {
            part: DayPart::Madhyahna,
        }),
        true,
        "Durgashtami, Ashvina's bright 8th",
    ),
    (
        "SADDULA_BATHUKAMMA",
        Masa::Ashwina,
        Tithi::ShuklaAshtami,
        Fit::LaterAlone(Window::Part {
            part: DayPart::Madhyahna,
        }),
        false,
        "Saddula Bathukamma, the festival's last day, on Durgashtami, Ashvina's bright 8th",
    ),
    (
        "MAHA_NAVAMI",
        Masa::Ashwina,
        Tithi::ShuklaNavami,
        Fit::LaterAlone(Window::Part {
            part: DayPart::Madhyahna,
        }),
        true,
        "Maharnavami, Ashvina's bright 9th",
    ),
    (
        "VIJAYA_DASHAMI",
        Masa::Ashwina,
        Tithi::ShuklaDashami,
        Fit::LaterAlone(Window::Part {
            part: DayPart::Madhyahna,
        }),
        false,
        "Vijaya Dasami (Dussehra), Ashvina's bright 10th",
    ),
    (
        "NARAK_CHATURDASHI",
        Masa::Ashwina,
        Tithi::KrishnaChaturdashi,
        Fit::LaterAlone(Window::Part {
            part: DayPart::Aparahna,
        }),
        true,
        "Naraka Chaturdhi, Ashvina's dark 14th, the afternoon's",
    ),
    (
        "LAKSHMI_PUJA",
        Masa::Ashwina,
        Tithi::Amavasya,
        Fit::LaterAlone(Window::Pradosha),
        false,
        "Deepavali, Ashvina's new moon, the evening's",
    ),
];

/// The state's days on a Gregorian date: New Year's Day is a general
/// holiday to 2025 and optional in 2026.
const CIVIL: [Civil; 5] = [
    ("NEW_YEAR_S_DAY_AD", (1, 1), None, &[], true),
    ("JAGJIVAN_RAM_JAYANTI", (4, 5), None, &[], false),
    ("AMBEDKAR_JAYANTI", (4, 14), None, &[], false),
    (
        "CHRISTMAS_EVE",
        (12, 24),
        Some(Tradition::Christian),
        &[],
        true,
    ),
    (
        "BOXING_DAY",
        (12, 26),
        Some(Tradition::Christian),
        &[],
        false,
    ),
];

/// The general days set by the moon in every year, as (month, day) in
/// the order of [`NOTIFICATIONS`], each named as the central list names
/// it.
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

/// The following day of Ramzan, a general holiday the moon sets with Idu'l
/// Fitr.
const RAMZAN_NEXT_DAY: Printed<'static> = (
    "EID_AL_FITR_NEXT_DAY",
    "Following day of Ramzan",
    "रमजानको भोलिपल्ट",
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
);

/// Annexure-II's optional days set by the moon.
const SIGHTED_OPTIONAL: [Named<'static>; 11] = [
    (
        "HAZRAT_ALI_BIRTHDAY",
        "Birthday of Hazrath Ali (R.A.)",
        "हजरत अलीको जन्मदिन",
    ),
    ("SHAB_E_MERAJ", "Shab-e-Meraj", "शब-ए-मेराज"),
    ("SHAB_E_BARAT", "Shab-e-Barat", "शब-ए-बरात"),
    (
        "SHAHADAT_HAZRAT_ALI",
        "Shahadat Hzt Ali (R.A.)",
        "हजरत अलीको शहादत",
    ),
    ("JUMAT_UL_VIDA", "Jumatul Wida", "जुमात-उल-विदा"),
    ("SHAB_E_QADR", "Shab-e-Qader", "शब-ए-कद्र"),
    ("EID_E_GHADEER", "Eid-e-Ghadeer", "ईद-ए-गदीर"),
    ("MUHARRAM_NINTH", "9th Moharram", "नवौं मोहर्रम"),
    ("ARBAEEN", "Arbayeen", "अरबईन"),
    ("YAZ_DAHUM_SHAREEF", "Yaz Dahum Shareef", "याज दहुम शरीफ"),
    (
        "MAHDI_MAUD_BIRTHDAY",
        "Birthday of Hzt. Syed Mohammed Juvanpuri Mahdi Ma'ud (A.S.)",
        "सैयद मोहम्मद जौनपुरी महदी मौऊदको जन्मदिन",
    ),
];

/// Each year's optional days set by the moon: 2026 prints Hazrath Ali's
/// birthday twice, the Islamic year falling twice in it.
const SIGHTED_OPTIONAL_DAYS: [OnDay<'static>; 89] = [
    (2019, "MAHDI_MAUD_BIRTHDAY", (1, 21)),
    (2019, "HAZRAT_ALI_BIRTHDAY", (3, 21)),
    (2019, "SHAB_E_MERAJ", (4, 4)),
    (2019, "SHAB_E_BARAT", (4, 21)),
    (2019, "SHAHADAT_HAZRAT_ALI", (5, 26)),
    (2019, "JUMAT_UL_VIDA", (5, 31)),
    (2019, "SHAB_E_QADR", (6, 2)),
    (2019, "EID_E_GHADEER", (8, 20)),
    (2019, "MUHARRAM_NINTH", (9, 9)),
    (2019, "ARBAEEN", (10, 20)),
    (2019, "YAZ_DAHUM_SHAREEF", (12, 9)),
    (2020, "MAHDI_MAUD_BIRTHDAY", (1, 10)),
    (2020, "HAZRAT_ALI_BIRTHDAY", (3, 9)),
    (2020, "SHAB_E_MERAJ", (3, 23)),
    (2020, "SHAB_E_BARAT", (4, 9)),
    (2020, "SHAHADAT_HAZRAT_ALI", (5, 14)),
    (2020, "SHAB_E_QADR", (5, 21)),
    (2020, "JUMAT_UL_VIDA", (5, 22)),
    (2020, "EID_E_GHADEER", (8, 9)),
    (2020, "MUHARRAM_NINTH", (8, 29)),
    (2020, "ARBAEEN", (10, 8)),
    (2020, "YAZ_DAHUM_SHAREEF", (11, 27)),
    (2021, "HAZRAT_ALI_BIRTHDAY", (2, 26)),
    (2021, "SHAB_E_MERAJ", (3, 12)),
    (2021, "SHAB_E_BARAT", (3, 29)),
    (2021, "SHAHADAT_HAZRAT_ALI", (5, 3)),
    (2021, "JUMAT_UL_VIDA", (5, 7)),
    (2021, "SHAB_E_QADR", (5, 10)),
    (2021, "EID_E_GHADEER", (7, 29)),
    (2021, "MUHARRAM_NINTH", (8, 18)),
    (2021, "ARBAEEN", (9, 29)),
    (2021, "YAZ_DAHUM_SHAREEF", (11, 16)),
    (2021, "MAHDI_MAUD_BIRTHDAY", (12, 19)),
    (2022, "HAZRAT_ALI_BIRTHDAY", (2, 15)),
    (2022, "SHAB_E_MERAJ", (3, 1)),
    (2022, "SHAB_E_BARAT", (3, 19)),
    (2022, "SHAHADAT_HAZRAT_ALI", (4, 22)),
    (2022, "SHAB_E_QADR", (4, 29)),
    (2022, "JUMAT_UL_VIDA", (4, 29)),
    (2022, "EID_E_GHADEER", (7, 18)),
    (2022, "MUHARRAM_NINTH", (8, 8)),
    (2022, "ARBAEEN", (9, 17)),
    (2022, "YAZ_DAHUM_SHAREEF", (11, 6)),
    (2022, "MAHDI_MAUD_BIRTHDAY", (12, 8)),
    (2023, "HAZRAT_ALI_BIRTHDAY", (2, 5)),
    (2023, "SHAB_E_MERAJ", (2, 19)),
    (2023, "SHAB_E_BARAT", (3, 8)),
    (2023, "SHAHADAT_HAZRAT_ALI", (4, 11)),
    (2023, "JUMAT_UL_VIDA", (4, 14)),
    (2023, "SHAB_E_QADR", (4, 18)),
    (2023, "EID_E_GHADEER", (7, 7)),
    (2023, "MUHARRAM_NINTH", (7, 28)),
    (2023, "ARBAEEN", (9, 6)),
    (2023, "YAZ_DAHUM_SHAREEF", (10, 27)),
    (2023, "MAHDI_MAUD_BIRTHDAY", (11, 29)),
    (2024, "HAZRAT_ALI_BIRTHDAY", (1, 25)),
    (2024, "SHAB_E_MERAJ", (2, 8)),
    (2024, "SHAB_E_BARAT", (2, 26)),
    (2024, "SHAHADAT_HAZRAT_ALI", (3, 31)),
    (2024, "JUMAT_UL_VIDA", (4, 5)),
    (2024, "SHAB_E_QADR", (4, 7)),
    (2024, "EID_E_GHADEER", (6, 25)),
    (2024, "MUHARRAM_NINTH", (7, 16)),
    (2024, "ARBAEEN", (8, 26)),
    (2024, "YAZ_DAHUM_SHAREEF", (10, 15)),
    (2024, "MAHDI_MAUD_BIRTHDAY", (11, 16)),
    (2025, "HAZRAT_ALI_BIRTHDAY", (1, 14)),
    (2025, "SHAB_E_MERAJ", (1, 28)),
    (2025, "SHAB_E_BARAT", (2, 14)),
    (2025, "SHAHADAT_HAZRAT_ALI", (3, 21)),
    (2025, "JUMAT_UL_VIDA", (3, 28)),
    (2025, "SHAB_E_QADR", (3, 28)),
    (2025, "EID_E_GHADEER", (6, 15)),
    (2025, "MUHARRAM_NINTH", (7, 5)),
    (2025, "ARBAEEN", (8, 15)),
    (2025, "YAZ_DAHUM_SHAREEF", (10, 4)),
    (2025, "MAHDI_MAUD_BIRTHDAY", (11, 16)),
    (2026, "HAZRAT_ALI_BIRTHDAY", (1, 3)),
    (2026, "SHAB_E_MERAJ", (1, 17)),
    (2026, "SHAB_E_BARAT", (2, 4)),
    (2026, "SHAHADAT_HAZRAT_ALI", (3, 10)),
    (2026, "JUMAT_UL_VIDA", (3, 13)),
    (2026, "SHAB_E_QADR", (3, 17)),
    (2026, "EID_E_GHADEER", (6, 4)),
    (2026, "MUHARRAM_NINTH", (6, 25)),
    (2026, "ARBAEEN", (8, 4)),
    (2026, "YAZ_DAHUM_SHAREEF", (9, 23)),
    (2026, "MAHDI_MAUD_BIRTHDAY", (10, 26)),
    (2026, "HAZRAT_ALI_BIRTHDAY", (12, 26)),
];

/// Bonalu, a general holiday printed with no rule the language holds: a
/// Monday of Ashadha the G.O. picks (C1390).
const BONALU: Printed<'static> = (
    "BONALU",
    "Bonalu",
    "बोनालु",
    &[
        (7, 29),
        (7, 20),
        (8, 2),
        (7, 25),
        (7, 17),
        (7, 29),
        (7, 21),
        (8, 10),
    ],
);

/// Varalakshmi Vratham, an optional holiday printed with no rule the
/// language holds: a Friday before Shravana's full moon (C1391).
const VARALAKSHMI: Printed<'static> = (
    "VARAMAHALAKSHMI_VRATA",
    "Varalakshmi Vratham",
    "वरलक्ष्मी व्रतम्",
    &[
        (8, 9),
        (7, 31),
        (8, 20),
        (8, 5),
        (8, 25),
        (8, 16),
        (8, 8),
        (8, 21),
    ],
);

/// The Parsi New Year, an optional holiday as printed: the catalogue has
/// no Shahenshahi calendar (C1243).
const PARSI_NEW_YEAR: Printed<'static> = (
    "PARSI_NEW_YEAR",
    "Parsi New Year's Day",
    "पारसी नयाँ वर्ष",
    &[
        (8, 17),
        (8, 16),
        (8, 16),
        (8, 16),
        (8, 16),
        (8, 15),
        (8, 15),
        (8, 15),
    ],
);

/// The values of a day of the list, optional or not.
fn listed(tradition: Option<Tradition>, optional: bool) -> FestivalMeta {
    STATE.listed(tradition, optional)
}

impl FestivalRule {
    /// The lunar days of Telangana's public holidays: the central list's
    /// rules for Maha Shivaratri, the Holika fire before Holi, Kartika
    /// Purnima (Guru Nanak's birthday), and the optional Mahaveer Jayanthi
    /// and Buddha Purnima; Nepal's printed panchanga's for Ugadi and the
    /// optional Basava Jayanthi (Akshaya Tritiya), Sri Panchami and Ratha
    /// Yathra; and Telangana's own, with Bathukamma, the Dasara days kept
    /// at midday, and Deepavali among them.
    ///
    /// ```
    /// use teistro_panchanga::festival::{DayPart, FestivalRule, Window};
    ///
    /// let rules = FestivalRule::india_telangana();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let dasara = of("VIJAYA_DASHAMI").ok_or("shipped")?;
    /// assert_eq!(dasara.at, Window::Part { part: DayPart::Madhyahna });
    /// assert_eq!(dasara.meta.regions, ["IN", "IN-TG"]);
    /// // Basava Jayanthi is the print's Akshaya Tritiya, an optional holiday.
    /// let basava = of("BASAVA_JAYANTI").ok_or("shipped")?;
    /// assert!(basava.source.starts_with("Nepal's national panchanga") && basava.meta.optional);
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_telangana() -> Vec<FestivalRule> {
        let mut rules = STATE.central_kept(&CENTRAL_KEPT);
        rules.extend(STATE.print_kept(&PRINT_KEPT));
        rules.extend(STATE.own(&TELANGANA, |key, what| {
            let why = if DASARA.contains(&key) {
                AT_MIDDAY
            } else {
                ALMANAC
            };
            format!("{NOTICE}: {what}; {why}")
        }));
        rules
    }
}

impl FollowingRule {
    /// Holi, the day after the Holika fire, as the central list keeps it;
    /// and from 2021 the following day of Vijaya Dasami.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let rules = FollowingRule::india_telangana();
    /// let counted: Vec<(&str, &str, i8)> = rules
    ///     .iter()
    ///     .map(|rule| (rule.key.as_str(), rule.after.as_str(), rule.days))
    ///     .collect();
    /// assert_eq!(counted, [("HOLI", "HOLIKA", 1), ("VIJAYA_DASHAMI_NEXT_DAY", "VIJAYA_DASHAMI", 1)]);
    /// assert!(rules.iter().all(|rule| !rule.meta.optional));
    /// ```
    #[must_use]
    pub fn india_telangana() -> Vec<FollowingRule> {
        let mut rules: Vec<FollowingRule> = FollowingRule::india_central()
            .into_iter()
            .map(|rule| FollowingRule {
                meta: STATE.kept(rule.meta),
                ..rule
            })
            .collect();
        rules.push(FollowingRule {
            key: "VIJAYA_DASHAMI_NEXT_DAY".to_owned(),
            source: format!(
                "{NOTICE}: the following day of Vijaya Dasami, a general holiday from 2021"
            ),
            after: "VIJAYA_DASHAMI".to_owned(),
            days: 1,
            meta: listed(Some(Tradition::Hindu), false),
        });
        rules
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, the state's days
    /// on a Gregorian date, Bhogi on Dhanu's last day and Sankranti and
    /// Kanumu on Makara's first two, Tamil New Year on Chithirai's first
    /// as Tamil Nadu's list keeps it, and Sri Krishna Astami on Rohini in
    /// Avani.
    ///
    /// ```
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_panchanga::festival::{Dated, DatedRule};
    ///
    /// let rules = DatedRule::india_telangana();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key).map(|rule| rule.on);
    /// assert_eq!(
    ///     of("BHOGI"),
    ///     Some(Dated::CivilDate { calendar: Calendar::MalayalamSolar, month: 5, day: -1, skipped: None })
    /// );
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-TG"]));
    /// ```
    #[must_use]
    pub fn india_telangana() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        rules.extend(STATE.civil(&CIVIL, NOTICE));
        let sankranti = [
            (
                "BHOGI",
                5,
                -1,
                false,
                "Bhogi, the eve of Sankranti; the last day of Dhanu (C1388)",
            ),
            (
                "UTTARAYANA_PUNYAKALA",
                6,
                1,
                false,
                "Sankranti / Pongal, Makara Sankranti; the first day of Makaram, the sankranti's day when it falls before three fifths of the daylight and the next day after (C1373, C1388)",
            ),
            (
                "KANUMU",
                6,
                2,
                true,
                "Annexure-II's Kanumu, the day after Sankranti; the second day of Makaram (C1388)",
            ),
        ];
        rules.extend(
            sankranti
                .into_iter()
                .map(|(key, month, day, optional, what)| {
                    STATE.on_solar_day(
                        key,
                        format!("{NOTICE}: {what}"),
                        (Calendar::MalayalamSolar, month, day),
                        optional,
                    )
                }),
        );
        rules.extend(
            DatedRule::india_tamil_nadu()
                .into_iter()
                .filter(|rule| rule.key == "PUTHANDU")
                .map(|rule| DatedRule {
                    source: format!(
                        "{NOTICE}: Annexure-II's Tamil New Year's Day; {}",
                        rule.source
                    ),
                    meta: listed(Some(Tradition::Hindu), true),
                    ..rule
                }),
        );
        rules.push(STATE.on_star(
            "JANMASHTAMI_VAISHNAVA",
            format!(
                "{NOTICE}: Sri Krishna Astami, as per the Srivaishnava Agamam; Rohini in Avani (solar Simha), the earlier of two (C1394)"
            ),
            (Calendar::TamilSolar, 5),
            (Nakshatra::Rohini, Which::Earlier),
            false,
        ));
        rules
    }
}

impl PublishedSet {
    /// The days the G.O.s print rather than reckon: the five general days
    /// set by the moon and Annexure-II's eleven, each marked subject to the
    /// sighting; and Bonalu, Varalakshmi Vratham and the Parsi New Year,
    /// unmarked.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_telangana();
    /// assert_eq!(set.entries.len(), 5 * 8 + 89 + 3 * 8);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2019, 6, 5));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// ```
    #[must_use]
    pub fn india_telangana() -> PublishedSet {
        let moon = |optional: bool| PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic), optional),
            cite: |notice| {
                format!(
                    "Government of Telangana, General Administration (Spl.E) Department, {notice}, its day subject to the moon (para 5)"
                )
            },
        };
        let printed = |tradition, communities: &[&str], optional| PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to: None,
            meta: FestivalMeta {
                communities: communities.iter().map(|&one| one.to_owned()).collect(),
                ..listed(Some(tradition), optional)
            },
            cite: |notice| {
                format!(
                    "Government of Telangana, General Administration (Spl.E) Department, {notice}, as printed"
                )
            },
        };
        let mut sighted_here = sighted(&SIGHTED_HERE);
        sighted_here.insert(1, RAMZAN_NEXT_DAY);
        let mut entries = moon(false).entries(&sighted_here);
        entries.extend(moon(true).on_days(&SIGHTED_OPTIONAL, &SIGHTED_OPTIONAL_DAYS));
        entries.extend(printed(Tradition::Hindu, &[], false).entries(&[BONALU]));
        entries.extend(printed(Tradition::Hindu, &[], true).entries(&[VARALAKSHMI]));
        entries
            .extend(printed(Tradition::Zoroastrian, &["PARSI"], true).entries(&[PARSI_NEW_YEAR]));
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of Telangana, General Administration (Spl.E) Department"
                .to_owned(),
            id: "in-tg-gad-printed".to_owned(),
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
        for rule in FestivalRule::india_telangana() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_telangana() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::india_telangana() {
            assert!(
                keys.contains(&rule.after),
                "{} counts from nothing",
                rule.key
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for entry in PublishedSet::india_telangana().entries {
            if let PublishedEntry::Add { key, .. } = entry {
                assert!(!keys.contains(&key), "{key} is reckoned and printed");
            }
        }
    }

    #[test]
    fn every_day_of_the_list_is_kept_in_the_state() {
        let lunar = FestivalRule::india_telangana()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let dated = DatedRule::india_telangana()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let following = FollowingRule::india_telangana()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let metas: Vec<(String, FestivalMeta)> = lunar.chain(dated).chain(following).collect();
        assert_eq!(metas.len(), 19 + 15 + 2);
        let optional = metas.iter().filter(|(_, meta)| meta.optional).count();
        assert_eq!(optional, 13);
        for (key, meta) in &metas {
            assert_eq!(meta.regions, REGIONS, "{key}");
            if key != "HOLIKA" {
                assert_eq!(meta.kind, Some(FestivalKind::Gazetted), "{key}");
            }
        }
    }

    #[test]
    fn every_printed_day_is_in_a_noticed_year() {
        let set = PublishedSet::india_telangana();
        assert_eq!(
            set.entries.len(),
            (SIGHTED_HERE.len() + 1) * NOTIFICATIONS.len() + SIGHTED_OPTIONAL_DAYS.len() + 3 * 8
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
        assert_eq!(marked, 40 + 89);
    }
}

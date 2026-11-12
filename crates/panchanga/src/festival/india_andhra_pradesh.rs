//! Andhra Pradesh's public holidays (`03-design/andhra-pradesh-holidays.md`,
//! SR-03, the ninth state set): the days the General Administration
//! (Poll.B) Department's yearly G.O. lists in its Annexure-I as general
//! holidays, those its Annexure-I(A) names as falling on a Sunday or a
//! second Saturday, and the optional holidays of its Annexure-II and II(A),
//! an employee's choice of five.
//!
//! The G.O. is rank 1 for the list. A day both this list and the central
//! list hold reuses `INDIA_CENTRAL`'s rule, a day Nepal's printed panchanga
//! keeps reuses that rule, and a day of the Telugu calendar Telangana's
//! list reckons reuses Telangana's rule: the two states keep one almanac
//! tradition, and their G.O.s print the same day for every such row but
//! the few `festival-measured.md` §18 names. The months are amanta.
//! Mahalaya Amavasya is Bhadrapada's new moon, the Dasara days are
//! Ashvina's bright 8th and 10th at midday, and Deepavali is Ashvina's new
//! moon; Bhogi, Sankranti and Kanuma are the last day of Dhanu and the
//! first two of Makara. Sri Krishna Astami is *Dharmasindhu*'s Janmashtami,
//! as Tamil Nadu's list keeps it (C1406).
//!
//! Ramzan, Bakrid, Moharrum and Eid Miladun Nabi are not reckoned: para 5
//! of each G.O. leaves their days to the moon, so they ship as printed,
//! marked [`SubjectTo::MoonSighting`], with 2021's Shab-e-Barath, which
//! that year's para 5 names too. Annexure-II's Islamic days, which para 5
//! does not name, ship as printed, unmarked (C1403), as do Varalakshmi
//! Vratham, the Parsi New Year and 2023's second Vijayadasami.

use teistro_core::catalogue::Tradition;

use super::india_central::{Civil, Named, OnDay, Printed, PrintedDays, StateList, sighted};
use super::india_telangana::{PARSI_NEW_YEAR, SIGHTED_OPTIONAL, VARALAKSHMI};
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{DatedRule, FestivalMeta, FestivalRule, FollowingRule, PublishedSet};

/// Where the list is kept: India, then the state the G.O. names.
const REGIONS: [&str; 2] = ["IN", "IN-AP"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The central list's lunar rules this list keeps, each with whether
/// Andhra Pradesh lists it as an optional holiday. The Holika fire is the
/// day before Holi, which the list keeps as the central list does.
const CENTRAL_KEPT: [(&str, bool); 5] = [
    ("SHIVARATRI", false),
    ("HOLIKA", false),
    ("MAHAVIR_JAYANTI", true),
    ("BUDDHA_JAYANTI", true),
    ("GURU_NANAK_JAYANTI", true),
];

/// Nepal's printed panchanga's rules this list keeps, by the key it keeps
/// them under, each with the print's key and whether it is optional.
const PRINT_KEPT: [(&str, &str, bool); 3] = [
    ("SAMVATSARA_ARAMBHA", "SAMVATSARA_ARAMBHA", false),
    ("BASAVA_JAYANTI", "AKSHAYA_TRITIYA", true),
    ("JAGANNATH_RATH_JATRA", "JAGANNATH_RATH_JATRA", true),
];

/// Telangana's lunar rules this list keeps, by the key it keeps them
/// under, each with Telangana's key, whether it is optional, and the day
/// as the G.O. names it. Mahalaya Amavasya is the new moon Telangana's
/// Bathukamma begins on.
const TELUGU_KEPT: [(&str, &str, bool, &str); 7] = [
    (
        "RAMA_NAVAMI",
        "RAMA_NAVAMI",
        false,
        "Sri Rama Navami, Chaitra's bright 9th at sunrise",
    ),
    (
        "GANESHA_CHATURTHI",
        "GANESHA_CHATURTHI",
        false,
        "Vinayaka Chavithi, Bhadrapada's bright 4th, the day of the midday worship",
    ),
    (
        "MAHALAYA_AMAVASYA",
        "BATHUKAMMA",
        true,
        "Annexure-II's Mahalaya Amavasya, Bhadrapada's new moon at sunrise",
    ),
    (
        "MAHA_ASHTAMI",
        "MAHA_ASHTAMI",
        false,
        "Durgastami, Ashvina's bright 8th, kept at midday (C1401)",
    ),
    (
        "VIJAYA_DASHAMI",
        "VIJAYA_DASHAMI",
        false,
        "Vijayadasami, Ashvina's bright 10th, kept at midday (C1401)",
    ),
    (
        "NARAK_CHATURDASHI",
        "NARAK_CHATURDASHI",
        true,
        "Annexure-II's Naraka Chaturdasi, Ashvina's dark 14th, the afternoon's (C1404)",
    ),
    (
        "LAKSHMI_PUJA",
        "LAKSHMI_PUJA",
        false,
        "Deepavali, Ashvina's new moon, the evening's",
    ),
];

/// The Sankranti days, as Telangana's list keeps them, each with the day
/// as the G.O. names it: Kanuma is a general holiday here.
const SANKRANTI: [(&str, &str); 3] = [
    ("BHOGI", "Bhogi"),
    ("UTTARAYANA_PUNYAKALA", "Makara Sankranti"),
    ("KANUMU", "Kanuma"),
];

/// The G.O. the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of Andhra Pradesh, General Administration (Poll.B) \
     Department, G.O.Rt.No.2276 of 4 December 2025: Annexure-I's general holidays and \
     Annexure-II's optional holidays for 2026";

/// Each year's G.O., and the copy read.
const NOTIFICATIONS: [(i32, &str); 7] = [
    (
        2020,
        "G.O.Rt.No.2745 of 5 December 2019, rank 1 as copied (its annexures, in RGUKT's proceedings of 26 December 2019)",
    ),
    (
        2021,
        "G.O.Rt.No.1963 of 15 December 2020, rank 1 as copied (the text of the G.O.)",
    ),
    (
        2022,
        "G.O.Rt.No.1997 of 26 November 2021, rank 1 as copied (the signed scan, with G.O.Rt.No.1998)",
    ),
    (
        2023,
        "G.O.Rt.No.2692 of 15 December 2022, rank 1 as copied (the signed scan, RGUKT's copy)",
    ),
    (
        2024,
        "G.O.Rt.No.2318 of 30 November 2023, rank 1 as copied (the attested scan)",
    ),
    (
        2025,
        "G.O.Rt.No.2115 of 6 December 2024, rank 1 as copied (the text of the G.O.)",
    ),
    (
        2026,
        "G.O.Rt.No.2276 of 4 December 2025, rank 3 (its annexures as a teachers' site transcribed them, C1402)",
    ),
];

/// The state's days on a Gregorian date: New Year's Day, Christmas Eve and
/// Boxing Day are optional in every year.
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
        true,
    ),
];

/// The general days set by the moon in every year, as (month, day) in
/// the order of [`NOTIFICATIONS`], each named as the central list names
/// it.
const SIGHTED_HERE: [(&str, &[(u8, u8)]); 4] = [
    (
        "EID_AL_FITR",
        &[(5, 25), (5, 14), (5, 3), (4, 22), (4, 11), (3, 31), (3, 20)],
    ),
    (
        "EID_AL_ADHA",
        &[(8, 1), (7, 21), (7, 10), (6, 29), (6, 17), (6, 7), (5, 27)],
    ),
    (
        "MUHARRAM",
        &[(8, 30), (8, 19), (8, 9), (7, 29), (7, 17), (7, 6), (6, 25)],
    ),
    (
        "MAWLID",
        &[
            (10, 30),
            (10, 20),
            (10, 9),
            (9, 28),
            (9, 16),
            (9, 5),
            (8, 25),
        ],
    ),
];

/// The Islamic days a year's G.O. lists as general holidays rather than
/// optional: 2021's Shab-e-Barath, with Holi, which that year's para 5
/// leaves to the moon; and 2023's Shab-e-Meraj, with Maha Sivaratri, which
/// it does not.
const SIGHTED_GENERAL: [OnDay<'static>; 1] = [(2021, "SHAB_E_BARAT", (3, 29))];

/// 2023's Shab-e-Meraj, a general holiday para 5 does not name.
const PRINTED_GENERAL: [OnDay<'static>; 1] = [(2023, "SHAB_E_MERAJ", (2, 18))];

/// Each year's optional Islamic days, as Annexure-II and II(A) print them.
const ISLAMIC_OPTIONAL: [OnDay<'static>; 75] = [
    (2020, "MAHDI_MAUD_BIRTHDAY", (1, 10)),
    (2020, "HAZRAT_ALI_BIRTHDAY", (3, 9)),
    (2020, "SHAB_E_MERAJ", (3, 23)),
    (2020, "SHAB_E_BARAT", (4, 9)),
    (2020, "SHAHADAT_HAZRAT_ALI", (5, 14)),
    (2020, "SHAB_E_QADR", (5, 21)),
    (2020, "JUMAT_UL_VIDA", (5, 22)),
    (2020, "EID_E_GHADEER", (8, 7)),
    (2020, "MUHARRAM_NINTH", (8, 29)),
    (2020, "ARBAEEN", (10, 8)),
    (2020, "YAZ_DAHUM_SHAREEF", (11, 27)),
    (2021, "HAZRAT_ALI_BIRTHDAY", (2, 25)),
    (2021, "SHAB_E_MERAJ", (3, 11)),
    (2021, "SHAHADAT_HAZRAT_ALI", (5, 4)),
    (2021, "JUMAT_UL_VIDA", (5, 7)),
    (2021, "SHAB_E_QADR", (5, 9)),
    (2021, "EID_E_GHADEER", (7, 29)),
    (2021, "MUHARRAM_NINTH", (8, 18)),
    (2021, "ARBAEEN", (9, 29)),
    (2021, "YAZ_DAHUM_SHAREEF", (11, 16)),
    (2021, "MAHDI_MAUD_BIRTHDAY", (12, 19)),
    (2022, "HAZRAT_ALI_BIRTHDAY", (2, 15)),
    (2022, "SHAB_E_MERAJ", (3, 1)),
    (2022, "SHAB_E_BARAT", (3, 19)),
    (2022, "SHAHADAT_HAZRAT_ALI", (4, 22)),
    (2022, "JUMAT_UL_VIDA", (4, 29)),
    (2022, "SHAB_E_QADR", (4, 29)),
    (2022, "EID_E_GHADEER", (7, 18)),
    (2022, "MUHARRAM_NINTH", (8, 8)),
    (2022, "ARBAEEN", (9, 17)),
    (2022, "YAZ_DAHUM_SHAREEF", (11, 6)),
    (2022, "MAHDI_MAUD_BIRTHDAY", (12, 8)),
    (2023, "HAZRAT_ALI_BIRTHDAY", (2, 5)),
    (2023, "SHAB_E_BARAT", (3, 7)),
    (2023, "SHAB_E_QADR", (4, 18)),
    (2023, "JUMAT_UL_VIDA", (4, 21)),
    (2023, "SHAHADAT_HAZRAT_ALI", (4, 24)),
    (2023, "EID_E_GHADEER", (7, 6)),
    (2023, "MUHARRAM_NINTH", (7, 28)),
    (2023, "ARBAEEN", (9, 5)),
    (2023, "MAHDI_MAUD_BIRTHDAY", (9, 9)),
    (2023, "YAZ_DAHUM_SHAREEF", (10, 26)),
    (2024, "HAZRAT_ALI_BIRTHDAY", (1, 25)),
    (2024, "SHAB_E_MERAJ", (2, 7)),
    (2024, "SHAB_E_BARAT", (2, 25)),
    (2024, "SHAHADAT_HAZRAT_ALI", (4, 1)),
    (2024, "JUMAT_UL_VIDA", (4, 5)),
    (2024, "SHAB_E_QADR", (4, 7)),
    (2024, "EID_E_GHADEER", (6, 25)),
    (2024, "MUHARRAM_NINTH", (7, 16)),
    (2024, "ARBAEEN", (8, 25)),
    (2024, "YAZ_DAHUM_SHAREEF", (10, 15)),
    (2024, "MAHDI_MAUD_BIRTHDAY", (11, 16)),
    (2025, "HAZRAT_ALI_BIRTHDAY", (1, 13)),
    (2025, "SHAB_E_MERAJ", (1, 27)),
    (2025, "SHAB_E_BARAT", (2, 14)),
    (2025, "SHAHADAT_HAZRAT_ALI", (3, 22)),
    (2025, "SHAB_E_QADR", (3, 27)),
    (2025, "JUMAT_UL_VIDA", (3, 28)),
    (2025, "EID_E_GHADEER", (6, 15)),
    (2025, "MUHARRAM_NINTH", (7, 5)),
    (2025, "ARBAEEN", (8, 15)),
    (2025, "YAZ_DAHUM_SHAREEF", (10, 9)),
    (2025, "MAHDI_MAUD_BIRTHDAY", (11, 6)),
    (2026, "HAZRAT_ALI_BIRTHDAY", (1, 3)),
    (2026, "SHAB_E_MERAJ", (1, 16)),
    (2026, "SHAB_E_BARAT", (2, 3)),
    (2026, "SHAHADAT_HAZRAT_ALI", (3, 11)),
    (2026, "JUMAT_UL_VIDA", (3, 13)),
    (2026, "SHAB_E_QADR", (3, 15)),
    (2026, "EID_E_GHADEER", (6, 3)),
    (2026, "MUHARRAM_NINTH", (6, 16)),
    (2026, "ARBAEEN", (8, 4)),
    (2026, "YAZ_DAHUM_SHAREEF", (9, 22)),
    (2026, "MAHDI_MAUD_BIRTHDAY", (10, 27)),
];

/// Varalakshmi Vratham, the Friday before Shravana's full moon as
/// Telangana's list prints it (C1391): optional to 2024 and a general
/// holiday from 2025.
const VARALAKSHMI_DAYS: [OnDay<'static>; 7] = [
    (2020, VARALAKSHMI.0, (7, 31)),
    (2021, VARALAKSHMI.0, (8, 20)),
    (2022, VARALAKSHMI.0, (8, 5)),
    (2023, VARALAKSHMI.0, (8, 25)),
    (2024, VARALAKSHMI.0, (8, 16)),
    (2025, VARALAKSHMI.0, (8, 8)),
    (2026, VARALAKSHMI.0, (8, 21)),
];

/// The first year Varalakshmi Vratham is a general holiday.
const VARALAKSHMI_GENERAL_FROM: i32 = 2025;

/// The Parsi New Year, an optional holiday as printed: the catalogue has
/// no Shahenshahi calendar (C1243).
const PARSI_DAYS: Printed<'static> = (
    PARSI_NEW_YEAR.0,
    PARSI_NEW_YEAR.1,
    PARSI_NEW_YEAR.2,
    &[
        (8, 20),
        (8, 16),
        (8, 16),
        (8, 16),
        (8, 15),
        (8, 15),
        (8, 15),
    ],
);

/// 2023's second Vijayadasami, an optional holiday the G.O. prints as the
/// tithi's second day (C1401).
const TITHI_DWAYAM: [Named<'static>; 1] = [(
    "VIJAYA_DASHAMI_NEXT_DAY",
    "Vijayadasami (Tidhi Dwayam)",
    "विजया दशमी (तिथि द्वय)",
)];

/// The day [`TITHI_DWAYAM`] names.
const TITHI_DWAYAM_DAY: [OnDay<'static>; 1] = [(2023, "VIJAYA_DASHAMI_NEXT_DAY", (10, 24))];

/// The values of a day of the list, optional or not.
fn listed(tradition: Option<Tradition>, optional: bool) -> FestivalMeta {
    STATE.listed(tradition, optional)
}

/// The name of a printed day.
const fn named((key, en, ne, _): Printed<'_>) -> Named<'_> {
    (key, en, ne)
}

impl FestivalRule {
    /// The lunar days of Andhra Pradesh's public holidays: the central
    /// list's rules for Maha Sivaratri, the Holika fire before Holi, and
    /// the optional Mahaveer Jayanthi, Buddha Purnima and Karthika Purnima
    /// (Guru Nanak's birthday); Nepal's printed panchanga's for Ugadi and
    /// the optional Basava Jayanthi (Akshaya Tritiya) and Ratha Yatra;
    /// *Dharmasindhu*'s Janmashtami for Sri Krishna Astami; and
    /// Telangana's rules for the Telugu days, Mahalaya Amavasya, the Dasara
    /// days kept at midday and Deepavali among them.
    ///
    /// ```
    /// use teistro_panchanga::festival::{DayPart, FestivalRule, Window};
    ///
    /// let rules = FestivalRule::india_andhra_pradesh();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let durgastami = of("MAHA_ASHTAMI").ok_or("shipped")?;
    /// assert_eq!(durgastami.at, Window::Part { part: DayPart::Madhyahna });
    /// assert_eq!(durgastami.meta.regions, ["IN", "IN-AP"]);
    /// assert!(!durgastami.meta.optional);
    /// // Mahalaya Amavasya is the new moon Telangana's Bathukamma begins on.
    /// let mahalaya = of("MAHALAYA_AMAVASYA").ok_or("shipped")?;
    /// assert!(mahalaya.source.contains("as Telangana's list reckons it") && mahalaya.meta.optional);
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_andhra_pradesh() -> Vec<FestivalRule> {
        let mut rules = STATE.central_kept(&CENTRAL_KEPT);
        rules.extend(STATE.print_kept(&PRINT_KEPT));
        rules.extend(STATE.by_text(&[("JANMASHTAMI", false)]));
        let kept: Vec<(&str, &str, bool)> = TELUGU_KEPT
            .iter()
            .map(|&(key, from, optional, _)| (key, from, optional))
            .collect();
        rules.extend(STATE.taken(
            &FestivalRule::india_telangana(),
            Tradition::Hindu,
            &kept,
            |key, rule| {
                let what = TELUGU_KEPT
                    .iter()
                    .find(|row| row.0 == key)
                    .map_or("", |row| row.3);
                format!(
                    "{NOTICE}: {what}; as Telangana's list reckons it: {}",
                    rule.source
                )
            },
        ));
        rules
    }
}

impl FollowingRule {
    /// Holi, the day after the Holika fire, as the central list keeps it.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let rules = FollowingRule::india_andhra_pradesh();
    /// let counted: Vec<(&str, &str, i8)> = rules
    ///     .iter()
    ///     .map(|rule| (rule.key.as_str(), rule.after.as_str(), rule.days))
    ///     .collect();
    /// assert_eq!(counted, [("HOLI", "HOLIKA", 1)]);
    /// assert!(rules.iter().all(|rule| !rule.meta.optional));
    /// ```
    #[must_use]
    pub fn india_andhra_pradesh() -> Vec<FollowingRule> {
        FollowingRule::india_central()
            .into_iter()
            .map(|rule| FollowingRule {
                meta: STATE.kept(rule.meta),
                ..rule
            })
            .collect()
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, the state's days
    /// on a Gregorian date, and Bhogi, Makara Sankranti and Kanuma as
    /// Telangana's list keeps them, on Dhanu's last day and Makara's first
    /// two.
    ///
    /// ```
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_panchanga::festival::{Dated, DatedRule};
    ///
    /// let rules = DatedRule::india_andhra_pradesh();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let kanuma = of("KANUMU").ok_or("shipped")?;
    /// assert_eq!(
    ///     kanuma.on,
    ///     Dated::CivilDate { calendar: Calendar::MalayalamSolar, month: 6, day: 2, skipped: None }
    /// );
    /// assert!(!kanuma.meta.optional);
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-AP"]));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_andhra_pradesh() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        rules.extend(STATE.civil(&CIVIL, NOTICE));
        let telangana = DatedRule::india_telangana();
        rules.extend(SANKRANTI.iter().filter_map(|&(key, what)| {
            telangana
                .iter()
                .find(|rule| rule.key == key)
                .map(|rule| DatedRule {
                    source: format!(
                        "{NOTICE}: {what}; as Telangana's list reckons it: {}",
                        rule.source
                    ),
                    meta: listed(Some(Tradition::Hindu), false),
                    ..rule.clone()
                })
        }));
        rules
    }
}

impl PublishedSet {
    /// The days the G.O.s print rather than reckon: the four general days
    /// set by the moon and 2021's Shab-e-Barath, marked subject to the
    /// sighting as para 5 leaves them; and, unmarked, Annexure-II's eleven
    /// Islamic days, 2023's Shab-e-Meraj, Varalakshmi Vratham, the Parsi
    /// New Year and 2023's second Vijayadasami.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_andhra_pradesh();
    /// assert_eq!(set.entries.len(), 4 * 7 + 1 + 75 + 1 + 2 * 7 + 1);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2020, 5, 25));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// ```
    #[must_use]
    pub fn india_andhra_pradesh() -> PublishedSet {
        let moon = |optional: bool| PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic), optional),
            cite: |notice| {
                format!(
                    "Government of Andhra Pradesh, General Administration (Poll.B) Department, {notice}, its day subject to the moon (para 5)"
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
                    "Government of Andhra Pradesh, General Administration (Poll.B) Department, {notice}, as printed"
                )
            },
        };
        let islamic = |optional| printed(Tradition::Islamic, &[], optional);
        let hindu = |optional| printed(Tradition::Hindu, &[], optional);
        let (optional_years, general_years): (Vec<OnDay<'_>>, Vec<OnDay<'_>>) = VARALAKSHMI_DAYS
            .iter()
            .copied()
            .partition(|row| row.0 < VARALAKSHMI_GENERAL_FROM);
        let varalakshmi = [named(VARALAKSHMI)];
        let mut entries = moon(false).entries(&sighted(&SIGHTED_HERE));
        entries.extend(moon(false).on_days(&SIGHTED_OPTIONAL, &SIGHTED_GENERAL));
        entries.extend(islamic(true).on_days(&SIGHTED_OPTIONAL, &ISLAMIC_OPTIONAL));
        entries.extend(islamic(false).on_days(&SIGHTED_OPTIONAL, &PRINTED_GENERAL));
        entries.extend(hindu(true).on_days(&varalakshmi, &optional_years));
        entries.extend(hindu(false).on_days(&varalakshmi, &general_years));
        entries.extend(printed(Tradition::Zoroastrian, &["PARSI"], true).entries(&[PARSI_DAYS]));
        entries.extend(hindu(true).on_days(&TITHI_DWAYAM, &TITHI_DWAYAM_DAY));
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of Andhra Pradesh, General Administration (Poll.B) Department"
                .to_owned(),
            id: "in-ap-gad-printed".to_owned(),
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
        for rule in FestivalRule::india_andhra_pradesh() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_andhra_pradesh() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in FollowingRule::india_andhra_pradesh() {
            assert!(
                keys.contains(&rule.after),
                "{} counts from nothing",
                rule.key
            );
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for entry in PublishedSet::india_andhra_pradesh().entries {
            if let PublishedEntry::Add { key, .. } = entry {
                assert!(!keys.contains(&key), "{key} is reckoned and printed");
            }
        }
    }

    #[test]
    fn every_day_of_the_list_is_kept_in_the_state() {
        let lunar = FestivalRule::india_andhra_pradesh()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let dated = DatedRule::india_andhra_pradesh()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let following = FollowingRule::india_andhra_pradesh()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let metas: Vec<(String, FestivalMeta)> = lunar.chain(dated).chain(following).collect();
        assert_eq!(metas.len(), 16 + 13 + 1);
        let optional = metas.iter().filter(|(_, meta)| meta.optional).count();
        assert_eq!(optional, 10);
        for (key, meta) in &metas {
            assert_eq!(meta.regions, REGIONS, "{key}");
            if key != "HOLIKA" {
                assert_eq!(meta.kind, Some(FestivalKind::Gazetted), "{key}");
            }
        }
    }

    #[test]
    fn every_printed_day_is_in_a_noticed_year() {
        let set = PublishedSet::india_andhra_pradesh();
        assert_eq!(
            set.entries.len(),
            SIGHTED_HERE.len() * NOTIFICATIONS.len()
                + SIGHTED_GENERAL.len()
                + ISLAMIC_OPTIONAL.len()
                + PRINTED_GENERAL.len()
                + VARALAKSHMI_DAYS.len()
                + NOTIFICATIONS.len()
                + TITHI_DWAYAM_DAY.len()
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
        assert_eq!(marked, 28 + 1);
    }

    #[test]
    fn varalakshmi_vratham_turns_general_in_2025() {
        let set = PublishedSet::india_andhra_pradesh();
        let standing: Vec<(i32, bool)> = set
            .entries
            .iter()
            .filter_map(|entry| match entry {
                PublishedEntry::Add { key, day, meta, .. } if key == VARALAKSHMI.0 => {
                    Some((day.year, meta.optional))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            standing,
            [
                (2020, true),
                (2021, true),
                (2022, true),
                (2023, true),
                (2024, true),
                (2025, false),
                (2026, false)
            ]
        );
    }
}

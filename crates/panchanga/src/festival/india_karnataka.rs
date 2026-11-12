//! Karnataka's public holidays (`03-design/karnataka-holidays.md`, SR-03,
//! the sixth state set): the days the Department of Personnel and
//! Administrative Reforms' yearly Notification-I lists as general
//! holidays, with the days its Note 1 names as falling on a Sunday or a
//! second or fourth Saturday, the restricted holidays of its Annexure, an
//! employee's choice of two, and the banks' closing of accounts its
//! Notification-II adds.
//!
//! The notification is rank 1 for the list. A day both this list and the
//! central list hold reuses `INDIA_CENTRAL`'s rule, and a day a shipped
//! text or print keeps reuses that rule. Karnataka's lunar months are
//! amanta, so Ugadi is Chaitra's bright 1st, Basava Jayanti Vaishakha's
//! bright 3rd, Mahalaya Amavasye Bhadrapada's new moon, Maharshi Valmiki
//! Jayanti Ashvina's full moon and Kanakadasa Jayanti Kartika's dark 3rd.
//! Souramana Ugadi is the civil day of the Sun's entry into Mesha, and
//! Onam and Sree Narayana Guru's birthday keep their stars in Chingam as
//! Kerala's list does. `festival-measured.md` §15 holds every rule to the
//! notifications at Bengaluru.
//!
//! Ramzan, Bakrid, Moharam and Eid-Milad, and the restricted Shab-e-Barat,
//! Shab-e-Qadar and Jumat-ul-Vida, are not reckoned: Note 3 leaves their
//! days to the moon, so they ship as printed, marked
//! [`SubjectTo::MoonSighting`]. Varamahalakshmi Vrata, Rig Upakarma and
//! Huttari ship as printed, unmarked. Kodagu's district holidays are not
//! here (C1379).

use teistro_calendar::easter::Computus;
use teistro_core::catalogue::{Calendar, Masa, Nakshatra, Tithi, Tradition};

use super::india_central::{Civil, Named, OnDay, Own, PrintedDays, StateList, find, sighted};
use super::nepal_panchanga::Fit;
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{
    Dated, DatedRule, DayReckoning, FestivalMeta, FestivalRule, FollowingRule, PublishedSet, Which,
};

/// Where the list is kept: India, then the state the notice names.
const REGIONS: [&str; 2] = ["IN", "IN-KA"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The central list's lunar rules this list keeps, each with whether
/// Karnataka lists it as a restricted holiday.
const CENTRAL_KEPT: [(&str, bool); 4] = [
    ("SHIVARATRI", false),
    ("MAHAVIR_JAYANTI", false),
    ("BUDDHA_JAYANTI", true),
    ("GURU_NANAK_JAYANTI", true),
];

/// Nepal's printed panchanga's rules this list keeps, by the key it keeps
/// them under, each with the print's key and whether it is restricted.
const PRINT_KEPT: [(&str, &str, bool); 6] = [
    ("SAMVATSARA_ARAMBHA", "SAMVATSARA_ARAMBHA", false),
    ("BASAVA_JAYANTI", "AKSHAYA_TRITIYA", false),
    ("GANESHA_CHATURTHI", "GANESHA_CHATURTHI", false),
    ("AMAVASYA_SHRADDHA", "AMAVASYA_SHRADDHA", false),
    ("NARAK_CHATURDASHI", "NARAK_CHATURDASHI", false),
    ("ANANTA_CHATURDASHI", "ANANTA_CHATURDASHI", true),
];

/// *Dharmasindhu*'s rules this list keeps, by the key it keeps them
/// under, each with the text's key and whether it is restricted.
const TEXT_KEPT: [(&str, &str, bool); 5] = [
    ("HOLIKA", "HOLIKA", true),
    ("UPAKARMA_TAITTIRIYA", "UPAKARMA_TAITTIRIYA", true),
    ("RAKSHABANDHAN", "RAKSHABANDHAN", true),
    ("JANMASHTAMI", "JANMASHTAMI", true),
    ("SWARNA_GOWRI_VRATA", "HARITALIKA", true),
];

/// The notification the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of Karnataka, Department of Personnel and Administrative \
     Reforms (State Protocol), notification DPAR 16 HHL 2025 of 17 November 2025: the general \
     holidays, and the Annexure's restricted holidays, for 2026";

/// Each year's notification, and how it was read.
const NOTIFICATIONS: [(i32, &str); 8] = [
    (
        2019,
        "DPAR 13 HHL 2018 of November 2018, rank 3 (a news site's list of the general holidays; the notification not read)",
    ),
    (
        2020,
        "DPAR 14 HHL 2019 of 22 November 2019, rank 1 as copied (the signed English scan)",
    ),
    (
        2021,
        "DPAR 12 HHL 2020 of 21 November 2020, Karnataka Rajyapatra Part IV-A No. 582, rank 1 (the gazette's pages)",
    ),
    (
        2022,
        "DPAR 37 HHL 2021 of 30 November 2021, with its corrigendum of 3 December 2021, rank 1 as copied (the signed scans)",
    ),
    (
        2023,
        "DPAR 16 HHL 2022 of 21 November 2022, rank 1 as copied (the signed Kannada scan)",
    ),
    (
        2024,
        "DPAR 16 HHL 2023 of 28 November 2023, rank 1 as copied (the signed English scan)",
    ),
    (
        2025,
        "DPAR 16 HHL 2024 of 21 November 2024, rank 1 as copied (the signed Kannada and English scans)",
    ),
    (
        2026,
        "DPAR 16 HHL 2025 of 17 November 2025, rank 1 as copied (the signed Kannada scan)",
    ),
];

/// Karnataka's convention for a lunar day, as a new rule cites it.
const ALMANAC: &str = "Karnataka's almanacs, as the state's notifications of 2020 to 2026 keep \
     the day; no almanac's page is read, so the rule is held to the notifications and the \
     citation is open (C1371)";

/// Karnataka's convention for a day another list or text keeps by another
/// window, as the rules held to sunrise cite it.
const AT_SUNRISE: &str = "kept at sunrise, as Karnataka keeps its own days: the central list \
     keeps Sri Ramanavami at midday, Nepal's printed panchanga keeps Shankaracharya Jayanti at \
     midday, and Dharmasindhu keeps Balipadyami on the later day only when its 1st lasts nine \
     muhurtas past sunrise; each of those parts from the notifications, and the sunrise reading \
     meets every year (C1375)";

/// The days another list or text keeps that Karnataka's rules hold to
/// sunrise.
const AT_SUNRISE_KEYS: [&str; 3] = ["RAMA_NAVAMI", "SHANKARACHARYA_JAYANTI", "BALI_PRATIPADA"];

/// Karnataka's rules by the tithi of an amanta month: key, month, tithi,
/// fit, whether restricted, and what the day is.
const KARNATAKA: [Own; 9] = [
    (
        "DEVARA_DASIMAYYA_JAYANTI",
        Masa::Chaitra,
        Tithi::ShuklaPanchami,
        Fit::Udaya,
        true,
        "Devara Dasimayya Jayanti, Chaitra's bright 5th, the fourth day after Ugadi in every year listed",
    ),
    (
        "RAMA_NAVAMI",
        Masa::Chaitra,
        Tithi::ShuklaNavami,
        Fit::Udaya,
        true,
        "Sri Ramanavami, Chaitra's bright 9th",
    ),
    (
        "SHANKARACHARYA_JAYANTI",
        Masa::Vaishakha,
        Tithi::ShuklaPanchami,
        Fit::Udaya,
        true,
        "Sri Shankaracharya Jayanti, Vaishakha's bright 5th",
    ),
    (
        "MADHVA_NAVAMI",
        Masa::Magha,
        Tithi::ShuklaNavami,
        Fit::Udaya,
        true,
        "Sri Madhva Navami, Magha's bright 9th",
    ),
    (
        "MAHA_NAVAMI",
        Masa::Ashwina,
        Tithi::ShuklaNavami,
        Fit::Udaya,
        false,
        "Mahanavami and Ayudhapuje, Ashvina's bright 9th",
    ),
    (
        "VIJAYA_DASHAMI",
        Masa::Ashwina,
        Tithi::ShuklaDashami,
        Fit::Udaya,
        false,
        "Vijayadashami, Ashvina's bright 10th",
    ),
    (
        "VALMIKI_JAYANTI",
        Masa::Ashwina,
        Tithi::Purnima,
        Fit::Udaya,
        false,
        "Maharshi Valmiki Jayanti, Ashvina's full moon",
    ),
    (
        "BALI_PRATIPADA",
        Masa::Kartika,
        Tithi::ShuklaPratipada,
        Fit::Udaya,
        false,
        "Balipadyami, the Deepavali holiday, Kartika's bright 1st",
    ),
    (
        "KANAKADASA_JAYANTI",
        Masa::Kartika,
        Tithi::KrishnaTritiya,
        Fit::Udaya,
        false,
        "Kanakadasa Jayanti, Kartika's dark 3rd (amanta), the third day after Kartika's full moon",
    ),
];

/// The state's days on a Gregorian date. Kanya Mariyamma Jayanti, the
/// Nativity of Mary, and Vishwakarma Jayanti are printed on 8 and 17
/// September in every year listed.
const CIVIL: [Civil; 8] = [
    ("NEW_YEAR_S_DAY_AD", (1, 1), None, &[], true),
    ("BANK_ACCOUNTS_CLOSING", (4, 1), None, &["BANKS"], false),
    ("AMBEDKAR_JAYANTI", (4, 14), None, &[], false),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1), None, &[], false),
    (
        "KANYA_MARIYAMMA_JAYANTI",
        (9, 8),
        Some(Tradition::Christian),
        &[],
        true,
    ),
    (
        "VISHWAKARMA_PUJA",
        (9, 17),
        Some(Tradition::Hindu),
        &[],
        true,
    ),
    ("KANNADA_RAJYOTSAVA", (11, 1), None, &[], false),
    (
        "CHRISTMAS_EVE",
        (12, 24),
        Some(Tradition::Christian),
        &[],
        true,
    ),
];

/// The days set by the moon in every year, as (month, day) in the order
/// of [`NOTIFICATIONS`], each named as the central list names it.
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
            (10, 20),
            (10, 9),
            (9, 28),
            (9, 16),
            (9, 5),
            (8, 26),
        ],
    ),
];

/// The restricted days set by the moon.
const SIGHTED_RESTRICTED: [Named<'static>; 3] = [
    ("SHAB_E_BARAT", "Shab-e-Barat", "शब-ए-बरात"),
    ("SHAB_E_QADR", "Shab-e-Qadar", "शब-ए-कद्र"),
    ("JUMAT_UL_VIDA", "Jumat-ul-Vida", "जुमात-उल-विदा"),
];

/// Each year's restricted days set by the moon: 2019's list is not in
/// hand.
const SIGHTED_RESTRICTED_DAYS: [OnDay<'static>; 21] = [
    (2020, "SHAB_E_BARAT", (4, 9)),
    (2020, "SHAB_E_QADR", (5, 20)),
    (2020, "JUMAT_UL_VIDA", (5, 22)),
    (2021, "SHAB_E_BARAT", (3, 30)),
    (2021, "SHAB_E_QADR", (5, 10)),
    (2021, "JUMAT_UL_VIDA", (5, 7)),
    (2022, "SHAB_E_BARAT", (3, 18)),
    (2022, "SHAB_E_QADR", (4, 28)),
    (2022, "JUMAT_UL_VIDA", (4, 29)),
    (2023, "SHAB_E_BARAT", (3, 7)),
    (2023, "SHAB_E_QADR", (4, 18)),
    (2023, "JUMAT_UL_VIDA", (4, 21)),
    (2024, "SHAB_E_BARAT", (2, 25)),
    (2024, "SHAB_E_QADR", (4, 6)),
    (2024, "JUMAT_UL_VIDA", (4, 5)),
    (2025, "SHAB_E_BARAT", (2, 14)),
    (2025, "SHAB_E_QADR", (3, 27)),
    (2025, "JUMAT_UL_VIDA", (3, 28)),
    (2026, "SHAB_E_BARAT", (2, 4)),
    (2026, "SHAB_E_QADR", (3, 17)),
    (2026, "JUMAT_UL_VIDA", (3, 20)),
];

/// The restricted days printed with no rule a source gives or the
/// language holds: Varamahalakshmi Vrata, a Friday before Shravana's full
/// moon (C1376); Rig Upakarma, which no one rule meets (C1377); and
/// Huttari, Kodagu's harvest, set each year by the temple's astrologers
/// (C1378).
const PRINTED_NAMES: [Named<'static>; 3] = [
    (
        "VARAMAHALAKSHMI_VRATA",
        "Varamahalakshmi Vrata",
        "वरमहालक्ष्मी व्रत",
    ),
    ("RIG_UPAKARMA", "Rig Upakarma", "ऋग् उपाकर्म"),
    ("HUTTARI", "Huttari", "हुत्तरी"),
];

/// Each year's printed restricted days.
const PRINTED_DAYS: [OnDay<'static>; 21] = [
    (2020, "VARAMAHALAKSHMI_VRATA", (7, 31)),
    (2020, "RIG_UPAKARMA", (8, 31)),
    (2020, "HUTTARI", (12, 1)),
    (2021, "VARAMAHALAKSHMI_VRATA", (8, 20)),
    (2021, "RIG_UPAKARMA", (8, 21)),
    (2021, "HUTTARI", (11, 20)),
    (2022, "VARAMAHALAKSHMI_VRATA", (8, 5)),
    (2022, "RIG_UPAKARMA", (8, 2)),
    (2022, "HUTTARI", (12, 8)),
    (2023, "VARAMAHALAKSHMI_VRATA", (8, 25)),
    (2023, "RIG_UPAKARMA", (8, 29)),
    (2023, "HUTTARI", (11, 28)),
    (2024, "VARAMAHALAKSHMI_VRATA", (8, 16)),
    (2024, "RIG_UPAKARMA", (8, 19)),
    (2024, "HUTTARI", (12, 14)),
    (2025, "VARAMAHALAKSHMI_VRATA", (8, 8)),
    (2025, "RIG_UPAKARMA", (8, 9)),
    (2025, "HUTTARI", (12, 5)),
    (2026, "VARAMAHALAKSHMI_VRATA", (8, 21)),
    (2026, "RIG_UPAKARMA", (8, 26)),
    (2026, "HUTTARI", (11, 26)),
];

/// The values of a day of the list, restricted or not.
fn listed(tradition: Option<Tradition>, restricted: bool) -> FestivalMeta {
    STATE.listed(tradition, restricted)
}

impl FestivalRule {
    /// The lunar days of Karnataka's public holidays: the central list's
    /// rules for Maha Shivaratri, Mahavira Jayanti and the restricted
    /// Buddha Purnima and Guru Nanak Jayanti; Nepal's printed panchanga's
    /// for Ugadi, Basava Jayanti (Akshaya Tritiya), Varasiddhi Vinayaka
    /// Vrata, Mahalaya Amavasye, Naraka Chaturdashi, and the restricted
    /// Ananta Padmanabha Vrata; *Dharmasindhu*'s for the restricted Holi,
    /// Yajur Upakarma, Raksha Bandhana, Krishna Janmashtami and
    /// Swarnagowri Vrata; and Karnataka's own, held to sunrise, with
    /// Balipadyami and the restricted Sri Ramanavami and Shankaracharya
    /// Jayanti among them (C1375).
    ///
    /// ```
    /// use teistro_panchanga::festival::{FestivalRule, Window};
    ///
    /// let rules = FestivalRule::india_karnataka();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let kanakadasa = of("KANAKADASA_JAYANTI").ok_or("shipped")?;
    /// assert_eq!(kanakadasa.at, Window::Sunrise);
    /// assert_eq!(kanakadasa.meta.regions, ["IN", "IN-KA"]);
    /// // Swarnagowri is the text's Haritalika, a restricted holiday.
    /// let gowri = of("SWARNA_GOWRI_VRATA").ok_or("shipped")?;
    /// assert!(gowri.source.starts_with("Dharmasindhu") && gowri.meta.optional);
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_karnataka() -> Vec<FestivalRule> {
        let mut rules = STATE.central_kept(&CENTRAL_KEPT);
        rules.extend(STATE.print_kept(&PRINT_KEPT));
        let text = FestivalRule::dharmasindhu();
        rules.extend(TEXT_KEPT.iter().filter_map(|&(key, from, restricted)| {
            find(&text, from).map(|rule| FestivalRule {
                key: key.to_owned(),
                meta: listed(Some(Tradition::Hindu), restricted).with_fast(from),
                ..rule
            })
        }));
        rules.extend(STATE.own(&KARNATAKA, |key, what| {
            let why = if AT_SUNRISE_KEYS.contains(&key) {
                AT_SUNRISE
            } else {
                ALMANAC
            };
            format!("{NOTICE}: {what}; {why}")
        }));
        rules
    }
}

impl FollowingRule {
    /// None: no day the notifications list is counted from another.
    #[must_use]
    pub fn india_karnataka() -> Vec<FollowingRule> {
        Vec::new()
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, Holy Saturday, the
    /// state's days on a Gregorian date, Souramana Ugadi on the civil day
    /// of the Sun's entry into Mesha, Makara Sankranti and Tula Sankramana
    /// on a solar month's first day, and the days kept on a star of a
    /// solar month: Onam and Sree Narayana Guru's birthday in Chingam, Sri
    /// Ramanujacharya's in Chittirai.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_panchanga::festival::{Dated, DatedRule, DayReckoning};
    ///
    /// let rules = DatedRule::india_karnataka();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key).map(|rule| rule.on);
    /// assert_eq!(
    ///     of("SOURAMANA_UGADI"),
    ///     Some(Dated::SunEnters { nakshatra: Nakshatra::Ashwini, days: 0, reckoning: DayReckoning::Civil })
    /// );
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-KA"]));
    /// ```
    #[must_use]
    pub fn india_karnataka() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        rules.push(DatedRule {
            key: "EASTER_SATURDAY".to_owned(),
            source: format!("{NOTICE}: the Annexure's Holy Saturday, the day before Easter"),
            on: Dated::Easter {
                computus: Computus::Gregorian,
                days: -1,
            },
            meta: listed(Some(Tradition::Christian), true),
        });
        rules.extend(STATE.civil(&CIVIL, NOTICE));
        rules.push(DatedRule {
            key: "SOURAMANA_UGADI".to_owned(),
            source: format!(
                "{NOTICE}: the Annexure's Souramana Ugadi, the solar new year; the civil day of the Sun's entry into Mesha, the first of Ashwini (C1372)"
            ),
            on: Dated::SunEnters {
                nakshatra: Nakshatra::Ashwini,
                days: 0,
                reckoning: DayReckoning::Civil,
            },
            meta: listed(Some(Tradition::Hindu), true),
        });
        rules.push(STATE.on_solar_day(
            "UTTARAYANA_PUNYAKALA",
            format!(
                "{NOTICE}: Uttarayana Punyakala, Makara Sankranti; the first day of Makaram, the sankranti's day when it falls before three fifths of the daylight and the next day after (C1373)"
            ),
            (Calendar::MalayalamSolar, 6, 1),
            false,
        ));
        rules.push(STATE.on_solar_day(
            "TULA_SANKRANTI",
            format!(
                "{NOTICE}: the Annexure's Tula Sankramana; the first day of Thulam, the sankranti's day when it falls before three fifths of the daylight and the next day after (C1373)"
            ),
            (Calendar::MalayalamSolar, 3, 1),
            true,
        ));
        let starred = [
            (
                "THIRUVONAM",
                (Calendar::MalayalamSolar, 1),
                Nakshatra::Shravana,
                "the Annexure's Onam, Thiruvonam (Shravana) in Chingam, as Kerala's list keeps it (C1323)",
            ),
            (
                "SREE_NARAYANA_GURU_JAYANTHI",
                (Calendar::MalayalamSolar, 1),
                Nakshatra::Shatabhisha,
                "the Annexure's Brahmashri Narayanaguru Jayanti, Chatayam (Shatabhisha) in Chingam, as Kerala's list keeps it (C1323)",
            ),
            (
                "RAMANUJACHARYA_JAYANTI",
                (Calendar::TamilSolar, 1),
                Nakshatra::Ardra,
                "the Annexure's Sri Ramanujacharya Jayanti, Thiruvathirai (Ardra) in Chittirai, the later of two as Kerala's and Tamil Nadu's lists keep a star (C1323, C1374)",
            ),
        ];
        rules.extend(starred.into_iter().map(|(key, month, nakshatra, what)| {
            STATE.on_star(
                key,
                format!("{NOTICE}: {what}"),
                month,
                (nakshatra, Which::Later),
                true,
            )
        }));
        rules
    }
}

impl PublishedSet {
    /// The days the notifications print rather than reckon, for the years
    /// a notification is in hand: the four general days set by the moon,
    /// and from 2020 the three restricted ones, each marked subject to the
    /// sighting; and from 2020 the restricted Varamahalakshmi Vrata, Rig
    /// Upakarma and Huttari, unmarked.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_karnataka();
    /// assert_eq!(set.entries.len(), 4 * 8 + 3 * 7 + 3 * 7);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2019, 6, 5));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// ```
    #[must_use]
    pub fn india_karnataka() -> PublishedSet {
        let moon = |restricted: bool| PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic), restricted),
            cite: |notice| {
                format!(
                    "Government of Karnataka, Department of Personnel and Administrative Reforms, notification {notice}, its day subject to the moon (Note 3)"
                )
            },
        };
        let mut entries = moon(false).entries(&sighted(&SIGHTED_HERE));
        entries.extend(moon(true).on_days(&SIGHTED_RESTRICTED, &SIGHTED_RESTRICTED_DAYS));
        entries.extend(
            PrintedDays {
                notices: &NOTIFICATIONS,
                declared: &[],
                subject_to: None,
                meta: listed(Some(Tradition::Hindu), true),
                cite: |notice| {
                    format!(
                        "Government of Karnataka, Department of Personnel and Administrative Reforms, notification {notice}, the Annexure's restricted holiday, as printed"
                    )
                },
            }
            .on_days(&PRINTED_NAMES, &PRINTED_DAYS),
        );
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher:
                "Government of Karnataka, Department of Personnel and Administrative Reforms"
                    .to_owned(),
            id: "in-ka-dpar-printed".to_owned(),
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
        for rule in FestivalRule::india_karnataka() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_karnataka() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        assert_eq!(FollowingRule::india_karnataka().len(), 0);
        for entry in PublishedSet::india_karnataka().entries {
            if let PublishedEntry::Add { key, .. } = entry {
                assert!(!keys.contains(&key), "{key} is reckoned and printed");
            }
        }
    }

    #[test]
    fn every_day_of_the_list_is_gazetted_in_the_state() {
        let lunar = FestivalRule::india_karnataka()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let dated = DatedRule::india_karnataka()
            .into_iter()
            .map(|rule| (rule.key, rule.meta));
        let metas: Vec<(String, FestivalMeta)> = lunar.chain(dated).collect();
        assert_eq!(metas.len(), 24 + 20);
        let restricted = metas.iter().filter(|(_, meta)| meta.optional).count();
        assert_eq!(restricted, 22);
        for (key, meta) in &metas {
            assert_eq!(meta.kind, Some(FestivalKind::Gazetted), "{key}");
            assert_eq!(meta.regions, REGIONS, "{key}");
        }
    }

    #[test]
    fn every_printed_day_is_in_a_noticed_year() {
        let set = PublishedSet::india_karnataka();
        assert_eq!(
            set.entries.len(),
            SIGHTED_HERE.len() * NOTIFICATIONS.len()
                + SIGHTED_RESTRICTED_DAYS.len()
                + PRINTED_DAYS.len()
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
        assert_eq!(marked, 32 + 21);
    }
}

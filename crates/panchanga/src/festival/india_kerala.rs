//! Kerala's public holidays (`03-design/kerala-holidays.md`, SR-03, the
//! third state set): the days the General Administration (Co-ordination)
//! Department's yearly notification in the Kerala Gazette lists as public
//! holidays to the public offices, with the occasions it names as falling
//! on a Sunday or a second Saturday, and its restricted holidays, each for
//! the community it names.
//!
//! The notification is rank 1 for the list, and prints every day with its
//! Malayalam Era date. A day both this list and the central list hold
//! reuses `INDIA_CENTRAL`'s rule. Kerala's own days are kept by the
//! Malayalam month (`Calendar::MalayalamSolar`): Vishu on the first civil
//! day whose sunrise follows the Mesha sankranti (C1301), Onam on
//! Thiruvonam in Chingam with the three days around it, Sree Narayana
//! Guru's birthday on Chatayam in Chingam, and his Samadhi on Kanni 5
//! (C1304). Sreekrishna Jayanthi is *Dharmasindhu*'s Janmashtami, the
//! ashtami's day rather than Rohini's (C1320). `festival-measured.md`
//! §12 holds every rule to six years of notifications at
//! Thiruvananthapuram.
//!
//! Id-ul-Fitr, Id-ul-Ad'ha, Muharram and Milad-i-Sherif are not reckoned:
//! the notifications leave their days to the appearance of the moon, so
//! they ship as printed, marked [`SubjectTo::MoonSighting`]. Ayya Vaikunda
//! Swami's birthday, a restricted holiday printed on 12 March to 2024 and
//! on 4 March from 2025, ships as printed, unmarked.

use teistro_calendar::easter::Computus;
use teistro_core::catalogue::{Calendar, Masa, Nakshatra, Tithi, Tradition};

use super::india_central::{Named, OnDay, PrintedDays, StateList, find, on_civil_date, sighted};
use super::nepal_panchanga::Fit;
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{
    Dated, DatedRule, DayReckoning, FestivalMeta, FestivalRule, FollowingRule, InAdhika,
    PublishedSet, Which,
};

/// Where the list is kept: India, then the state the notice names.
const REGIONS: [&str; 2] = ["IN", "IN-KL"];

/// The state's list.
const STATE: StateList = StateList { regions: &REGIONS };

/// The central list's lunar rules this list keeps: Sivarathri, and
/// Deepavali, the central list's Diwali.
const CENTRAL_KEPT: [&str; 2] = ["SHIVARATRI", "LAKSHMI_PUJA"];

/// The notification the list and its fixed days are read from, the latest.
const NOTICE: &str = "Government of Kerala, General Administration (Co-ordination) Department, \
     G.O.(P) No. 14/2025/GAD of 31 October 2025 (Kerala Gazette Extraordinary No. 3845): public \
     holidays to the public offices for 2026";

/// Each year's notification, and how it was read.
const NOTIFICATIONS: [(i32, &str); 6] = [
    (
        2021,
        "G.O.(P) No. 16/2020/GAD of 22 October 2020, rank 1 (a staff site's text of the signed order, with its scan)",
    ),
    (
        2022,
        "G.O.(P) No. 29/2021/GAD of 21 October 2021, Kerala Gazette Extraordinary No. 3060, rank 1 (gad.kerala.gov.in)",
    ),
    (
        2023,
        "G.O.(P) No. 22/2022/GAD of 26 October 2022, Kerala Gazette Extraordinary No. 3657, rank 1 (a copy of the digitally signed gazette)",
    ),
    (
        2024,
        "G.O.(P) No. 24/2023/GAD of 4 October 2023, Kerala Gazette Extraordinary No. 3254, rank 1 (a copy of the digitally signed gazette)",
    ),
    (
        2025,
        "G.O.(P) No. 34/2024/GAD of 14 October 2024, Kerala Gazette Extraordinary, rank 1 (a copy of the digitally signed gazette)",
    ),
    (
        2026,
        "G.O.(P) No. 14/2025/GAD of 31 October 2025, Kerala Gazette Extraordinary No. 3845, rank 1 (gad.kerala.gov.in)",
    ),
];

/// Kerala's convention for a lunar day, as a new rule cites it.
const ALMANAC: &str = "Kerala's almanacs, as the state's notifications of 2021 to 2026 keep the \
     day; no almanac's page is read, so the rule is held to the notifications and the citation is \
     open (C1321)";

/// Kerala's rules by the tithi: key, amanta month, tithi, fit, and what
/// the day is.
const KERALA: [(&str, Masa, Tithi, Fit, &str); 3] = [
    (
        "KARKIDAKA_VAVU",
        Masa::Ashadha,
        Tithi::Amavasya,
        Fit::Udaya,
        "Karkidaka Vavu, the ancestors' bali at dawn on the new moon of Karkidakam, which ends the amanta Ashadha",
    ),
    (
        "MAHA_NAVAMI",
        Masa::Ashwina,
        Tithi::ShuklaNavami,
        Fit::Udaya,
        "Mahanavami, Ashwina's bright 9th, the day of the Ayudha puja",
    ),
    (
        "VIJAYA_DASHAMI",
        Masa::Ashwina,
        Tithi::ShuklaDashami,
        Fit::Udaya,
        "Vijayadasami, Ashwina's bright 10th, the day of the Vidyarambham",
    ),
];

/// Kerala's days on a star of a Malayalam month: key, month (Chingam the
/// 1st), star, the days counted from its day, and what the day is.
const STARRED: [(&str, u8, Nakshatra, i8, &str); 5] = [
    (
        "THIRUVONAM",
        1,
        Nakshatra::Shravana,
        0,
        "Thiruvonam, the star Thiruvonam (Shravana) in Chingam",
    ),
    (
        "FIRST_ONAM",
        1,
        Nakshatra::Shravana,
        -1,
        "First Onam (Uthradom), the day before Thiruvonam in every year listed",
    ),
    (
        "THIRD_ONAM",
        1,
        Nakshatra::Shravana,
        1,
        "Third Onam, the day after Thiruvonam in every year listed",
    ),
    (
        "FOURTH_ONAM",
        1,
        Nakshatra::Shravana,
        2,
        "Fourth Onam, the second day after Thiruvonam in every year listed, Kanni 1 in 2024",
    ),
    (
        "SREE_NARAYANA_GURU_JAYANTHI",
        1,
        Nakshatra::Shatabhisha,
        0,
        "Sree Narayana Guru Jayanthi, the star Chatayam (Shatabhisha) in Chingam",
    ),
];

/// The state's days on a Gregorian date: key, (month, day), tradition,
/// communities, and whether it is a restricted holiday. Ayyankali's
/// birthday, 28 August, is 12 Chingam in every year listed.
type Civil = (
    &'static str,
    (u8, i8),
    Option<Tradition>,
    &'static [&'static str],
    bool,
);

/// The state's days on a Gregorian date.
const CIVIL: [Civil; 5] = [
    ("MANNAM_JAYANTHI", (1, 2), None, &[], false),
    ("AMBEDKAR_JAYANTI", (4, 14), None, &[], false),
    ("INTERNATIONAL_LABOUR_DAY", (5, 1), None, &[], false),
    ("AYYANKALI_JAYANTHI", (8, 28), None, &[], false),
    (
        "VISHWAKARMA_PUJA",
        (9, 17),
        Some(Tradition::Hindu),
        &["VISHWAKARMA"],
        true,
    ),
];

/// The days set by the moon, as (month, day) in the order of
/// [`NOTIFICATIONS`], each named as the central list names it;
/// Milad-i-Sherif is the central list's Id-e-Milad.
const SIGHTED_HERE: [(&str, &[(u8, u8)]); 4] = [
    (
        "EID_AL_FITR",
        &[(5, 13), (5, 2), (4, 21), (4, 10), (3, 31), (3, 20)],
    ),
    (
        "EID_AL_ADHA",
        &[(7, 20), (7, 9), (6, 28), (6, 17), (6, 6), (5, 27)],
    ),
    (
        "MUHARRAM",
        &[(8, 19), (8, 8), (7, 28), (7, 16), (7, 6), (6, 25)],
    ),
    (
        "MAWLID",
        &[(10, 19), (10, 8), (9, 27), (9, 16), (9, 5), (8, 25)],
    ),
];

/// Ayya Vaikunda Swami's birthday, restricted to the Nadar community.
const VAIKUNDA_NAMES: [Named<'static>; 1] = [(
    "AYYA_VAIKUNDA_SWAMI_JAYANTHI",
    "Ayya Vaikunda Swami Jayanthi",
    "अय्या वैकुण्ड स्वामी जयन्ती",
)];

/// Each year's Ayya Vaikunda Swami's birthday.
const VAIKUNDA_DAYS: [OnDay<'static>; 6] = [
    (2021, "AYYA_VAIKUNDA_SWAMI_JAYANTHI", (3, 12)),
    (2022, "AYYA_VAIKUNDA_SWAMI_JAYANTHI", (3, 12)),
    (2023, "AYYA_VAIKUNDA_SWAMI_JAYANTHI", (3, 12)),
    (2024, "AYYA_VAIKUNDA_SWAMI_JAYANTHI", (3, 12)),
    (2025, "AYYA_VAIKUNDA_SWAMI_JAYANTHI", (3, 4)),
    (2026, "AYYA_VAIKUNDA_SWAMI_JAYANTHI", (3, 4)),
];

/// The values of a day of the list.
fn listed(tradition: Option<Tradition>) -> FestivalMeta {
    STATE.listed(tradition, false)
}

/// The values of a restricted holiday, for `communities`.
fn restricted(tradition: Option<Tradition>, communities: &[&str]) -> FestivalMeta {
    FestivalMeta {
        communities: communities.iter().map(|&one| one.to_owned()).collect(),
        ..STATE.listed(tradition, true)
    }
}

impl FestivalRule {
    /// The lunar days of Kerala's public holidays: the central list's
    /// rules for Sivarathri and Deepavali, *Dharmasindhu*'s Janmashtami
    /// for Sreekrishna Jayanthi and its Taittiriya upakarma for Avani
    /// Avittom, and Kerala's own for Karkidaka Vavu,
    /// Mahanavami and Vijayadasami.
    ///
    /// ```
    /// use teistro_panchanga::festival::{FestivalRule, Window};
    ///
    /// let rules = FestivalRule::india_kerala();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key);
    /// let vavu = of("KARKIDAKA_VAVU").ok_or("shipped")?;
    /// assert_eq!(vavu.at, Window::Sunrise);
    /// assert_eq!(vavu.meta.regions, ["IN", "IN-KL"]);
    /// // Sreekrishna Jayanthi is the ashtami's day, not Rohini's (C1320).
    /// assert!(of("JANMASHTAMI").ok_or("shipped")?.source.starts_with("Dharmasindhu"));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_kerala() -> Vec<FestivalRule> {
        let mut rules: Vec<FestivalRule> = CENTRAL_KEPT
            .iter()
            .filter_map(|&key| find(&FestivalRule::india_central(), key))
            .map(|rule| FestivalRule {
                meta: STATE.kept(rule.meta),
                ..rule
            })
            .collect();
        rules.extend(STATE.by_text(&[("JANMASHTAMI", false)]));
        rules.extend(
            find(&FestivalRule::dharmasindhu(), "UPAKARMA_TAITTIRIYA").map(|rule| FestivalRule {
                key: "AVANI_AVITTOM".to_owned(),
                source: format!(
                    "{NOTICE}, restricted holiday for Brahmin employees: Avani Avittom, the Yajurvedis' upakarma; {}",
                    rule.source
                ),
                meta: restricted(Some(Tradition::Hindu), &["BRAHMIN"]),
                ..rule
            }),
        );
        rules.extend(
            KERALA
                .iter()
                .map(|&(key, month, tithi, fit, what)| FestivalRule {
                    meta: listed(Some(Tradition::Hindu)),
                    ..fit.rule(
                        key,
                        Some(month),
                        tithi.into(),
                        InAdhika::Nija,
                        &format!("{what}; {ALMANAC}"),
                    )
                }),
        );
        rules
    }
}

impl FollowingRule {
    /// None: the days the notifications count from another are counted
    /// from Thiruvonam's star ([`DatedRule::india_kerala`]), which no
    /// karmakala rule keeps.
    #[must_use]
    pub fn india_kerala() -> Vec<FollowingRule> {
        Vec::new()
    }
}

impl DatedRule {
    /// The holidays on a date: the central list's five, Maundy Thursday
    /// and Easter, the state's days on a Gregorian date, Vishu on the
    /// first sunrise after the Sun enters Mesha, the days kept on a star
    /// of Chingam, and Sree Narayana Guru's Samadhi on Kanni 5.
    ///
    /// ```
    /// use teistro_core::catalogue::{Calendar, Nakshatra};
    /// use teistro_panchanga::festival::{Dated, DatedRule, DayReckoning, Which};
    ///
    /// let rules = DatedRule::india_kerala();
    /// let of = |key: &str| rules.iter().find(|rule| rule.key == key).map(|rule| rule.on);
    /// assert_eq!(
    ///     of("VISHU"),
    ///     Some(Dated::SunEnters { nakshatra: Nakshatra::Ashwini, days: 1, reckoning: DayReckoning::Sunrise })
    /// );
    /// assert_eq!(
    ///     of("THIRUVONAM"),
    ///     Some(Dated::NakshatraInMonth {
    ///         calendar: Calendar::MalayalamSolar,
    ///         month: 1,
    ///         nakshatra: Nakshatra::Shravana,
    ///         twice: Which::Later,
    ///         days: 0,
    ///     })
    /// );
    /// assert!(rules.iter().all(|rule| rule.meta.regions == ["IN", "IN-KL"]));
    /// ```
    #[must_use]
    pub fn india_kerala() -> Vec<DatedRule> {
        let mut rules = STATE.central_dated(NOTICE);
        let easter = |key: &str, days: i16, what: &str| DatedRule {
            key: key.to_owned(),
            source: format!("{NOTICE}: {what}"),
            on: Dated::Easter {
                computus: Computus::Gregorian,
                days,
            },
            meta: listed(Some(Tradition::Christian)),
        };
        rules.push(easter(
            "MAUNDY_THURSDAY",
            -3,
            "Maundy Thursday, three days before Easter",
        ));
        rules.push(easter(
            "EASTER",
            0,
            "Easter, among the occasions falling on a Sunday",
        ));
        rules.extend(
            CIVIL
                .iter()
                .map(|&(key, day, tradition, communities, optional)| {
                    let meta = if optional {
                        restricted(tradition, communities)
                    } else {
                        listed(tradition)
                    };
                    on_civil_date(key, day, NOTICE, meta)
                }),
        );
        rules.push(DatedRule {
            key: "VISHU".to_owned(),
            source: format!(
                "{NOTICE}: Vishu, printed on Medam 1 or 2; the first civil day whose sunrise follows the Sun's entry into Mesha, the first of Ashwini, at Thiruvananthapuram (C1301)"
            ),
            on: Dated::SunEnters {
                nakshatra: Nakshatra::Ashwini,
                days: 1,
                reckoning: DayReckoning::Sunrise,
            },
            meta: listed(Some(Tradition::Hindu)),
        });
        rules.extend(
            STARRED
                .iter()
                .map(|&(key, month, nakshatra, days, what)| DatedRule {
                    key: key.to_owned(),
                    source: format!(
                        "{NOTICE}: {what}; the star's day is the first whose sunrise it holds, the later of two in one month as 2024's Thiruvonam on Chingam 30 (C1323)"
                    ),
                    on: Dated::NakshatraInMonth {
                        calendar: Calendar::MalayalamSolar,
                        month,
                        nakshatra,
                        twice: Which::Later,
                        days,
                    },
                    meta: listed(Some(Tradition::Hindu)),
                }),
        );
        rules.push(DatedRule {
            key: "SREE_NARAYANA_GURU_SAMADHI".to_owned(),
            source: format!(
                "{NOTICE}: Sree Narayana Guru Samadhi Day, printed on Kanni 5 (5-2 of the Malayalam Era) in every year"
            ),
            on: Dated::CivilDate {
                calendar: Calendar::MalayalamSolar,
                month: 2,
                day: 5,
                skipped: None,
            },
            meta: listed(Some(Tradition::Hindu)),
        });
        rules
    }
}

impl PublishedSet {
    /// The days the notifications print rather than reckon, for the years
    /// a notification is in hand: the four set by the moon, each marked
    /// subject to the sighting, and Ayya Vaikunda Swami's birthday,
    /// unmarked.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_kerala();
    /// assert_eq!(set.entries.len(), 4 * 6 + 6);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2021, 5, 13));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// ```
    #[must_use]
    pub fn india_kerala() -> PublishedSet {
        let mut entries = PrintedDays {
            notices: &NOTIFICATIONS,
            declared: &[],
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic)),
            cite: |notice| {
                format!(
                    "Government of Kerala, General Administration (Co-ordination) Department, {notice}, its day subject to change with the appearance of the moon"
                )
            },
        }
        .entries(&sighted(&SIGHTED_HERE));
        entries.extend(
            PrintedDays {
                notices: &NOTIFICATIONS,
                declared: &[],
                subject_to: None,
                meta: restricted(None, &["NADAR"]),
                cite: |notice| {
                    format!(
                        "Government of Kerala, General Administration (Co-ordination) Department, {notice}, a restricted holiday for the Nadar community, as printed"
                    )
                },
            }
            .on_days(&VAIKUNDA_NAMES, &VAIKUNDA_DAYS),
        );
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of Kerala, General Administration (Co-ordination) Department"
                .to_owned(),
            id: "in-kl-gad-printed".to_owned(),
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

    /// Whether `entries` hold an entry of `key`.
    fn holds(entries: &[PublishedEntry], key: &str) -> bool {
        entries
            .iter()
            .any(|entry| matches!(entry, PublishedEntry::Add { key: held, .. } if held == key))
    }

    #[test]
    fn every_rule_checks_and_keys_are_unique() {
        let mut keys = HashSet::new();
        for rule in FestivalRule::india_kerala() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        for rule in DatedRule::india_kerala() {
            rule.check().unwrap();
            assert!(keys.insert(rule.key.clone()), "{} twice", rule.key);
        }
        assert_eq!(FollowingRule::india_kerala().len(), 0);
    }

    #[test]
    fn every_day_of_the_list_is_gazetted_in_the_state() {
        let lunar = FestivalRule::india_kerala()
            .into_iter()
            .map(|rule| rule.meta);
        let dated = DatedRule::india_kerala().into_iter().map(|rule| rule.meta);
        let metas: Vec<FestivalMeta> = lunar.chain(dated).collect();
        assert_eq!(metas.len(), 7 + 19);
        for meta in &metas {
            assert_eq!(meta.kind, Some(FestivalKind::Gazetted));
            assert_eq!(meta.regions, REGIONS);
            // A restricted holiday is a community's, and only those are
            // optional.
            assert_eq!(meta.optional, !meta.communities.is_empty());
        }
    }

    #[test]
    fn every_printed_day_is_in_a_noticed_year() {
        let set = PublishedSet::india_kerala();
        assert_eq!(
            set.entries.len(),
            SIGHTED_HERE.len() * NOTIFICATIONS.len() + VAIKUNDA_DAYS.len()
        );
        assert!(holds(&set.entries, "AYYA_VAIKUNDA_SWAMI_JAYANTHI"));
        assert!(holds(&set.entries, "MAWLID"));
    }
}

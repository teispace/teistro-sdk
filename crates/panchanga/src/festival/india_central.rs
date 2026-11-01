//! The Government of India's central holidays (`03-design/india-central-holidays.md`,
//! SR-03): the rules of the days its Department of Personnel and Training
//! lists for its administrative offices in Delhi, and the four days it
//! sets by the moon.
//!
//! The notice is rank 1 for the list. A day's rule is cited to a text or a
//! printed panchanga, and `festival-measured.md` §9 holds every rule to
//! seven years of notices. Each year's list is the fourteen compulsory
//! holidays of para 2 and three chosen from the twelve occasions of para
//! 3.1, so a chosen occasion is marked [`FestivalMeta::optional`].
//!
//! Idu'l Fitr, Idu'l Zuha, Muharram and Id-e-Milad are not reckoned. Para
//! 5.1 leaves their days to the sighting of the moon, so they ship as what
//! the notices print, each marked [`SubjectTo::MoonSighting`], and a year
//! with no notice holds none of them.

use teistro_calendar::CalendarDay;
use teistro_core::catalogue::{Calendar, FestivalKind, Tradition};

use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{
    Dated, DatedRule, FestivalMeta, FestivalRule, FollowingRule, PublishedEntry, PublishedSet,
};

/// The notice the list comes from.
const NOTICE: &str = "Government of India, DoPT O.M. F.No.12/2/2023-JCA of 3 July 2025, para 2";

/// A chosen occasion's notice.
const OPTIONAL: &str = "Government of India, DoPT O.M. F.No.12/2/2023-JCA of 3 July 2025, para 3.1";

/// The office memorandum printing each year's days.
const MEMORANDA: [(i32, &str); 7] = [
    (2021, "F.No.12/9/2020-JCA-2 of 10 June 2020"),
    (2022, "F.No.12/5/2021-JCA-2 of 8 June 2021"),
    (2023, "F.No.12/5/2022-JCA of 16 June 2022"),
    (2024, "F.No.12/2/2023-JCA of 3 July 2023"),
    (2025, "F.No.12/2/2023-JCA of 9 July 2024"),
    (2026, "F.No.12/2/2023-JCA of 3 July 2025"),
    (2027, "F.No.12/2/2023-JCA of 16 July 2026"),
];

/// A day set by the moon, as [`SIGHTED`] holds it.
type Sighted = (&'static str, &'static str, &'static str, [(u8, u8); 7]);

/// The days set by the moon: key, English name, Nepali name, and the days
/// the notices print, as (month, day) in the order of [`MEMORANDA`].
const SIGHTED: [Sighted; 4] = [
    (
        "EID_AL_FITR",
        "Id-ul-Fitr",
        "ईद उल फित्र",
        [(5, 14), (5, 3), (4, 22), (4, 11), (3, 31), (3, 21), (3, 10)],
    ),
    (
        "EID_AL_ADHA",
        "Id-ul-Zuha (Bakrid)",
        "बकर ईद",
        [(7, 21), (7, 10), (6, 29), (6, 17), (6, 7), (5, 27), (5, 17)],
    ),
    (
        "MUHARRAM",
        "Muharram",
        "मोहर्रम",
        [(8, 19), (8, 9), (7, 29), (7, 17), (7, 6), (6, 26), (6, 16)],
    ),
    (
        "MAWLID",
        "Milad-un-Nabi or Id-e-Milad",
        "मोहम्मद जयन्ती",
        [
            (10, 19),
            (10, 9),
            (9, 28),
            (9, 16),
            (9, 5),
            (8, 26),
            (8, 15),
        ],
    ),
];

/// The days the Department moved once the moon was seen, each a day it
/// declared: key, year, the day, and the memorandum.
const DECLARED: [(&str, i32, (u8, u8), &str); 2] = [
    (
        "MUHARRAM",
        2021,
        (8, 20),
        "DoPT O.M. No. 12/13/2016-JCA-2 of 11 August 2021",
    ),
    (
        "EID_AL_ADHA",
        2026,
        (5, 28),
        "DoPT O.M. F.No.12/3/2023-JCA of 22 May 2026",
    ),
];

/// The Gregorian day `(month, day)` of `year`.
fn gregorian(year: i32, (month, day): (u8, u8)) -> CalendarDay {
    CalendarDay {
        calendar: Calendar::Gregorian,
        year,
        month,
        day,
        era: None,
        leap: None,
    }
}

/// The values of a rule of the list: `tradition` as the rule's own text
/// or the notice's name says, in India. The list names no fast; a rule
/// taken from a text gains that text's with [`FestivalMeta::with_fast`].
fn listed(tradition: Option<Tradition>, optional: bool) -> FestivalMeta {
    FestivalMeta {
        kind: Some(FestivalKind::Gazetted),
        tradition,
        communities: Vec::new(),
        regions: vec!["IN".to_owned()],
        optional,
        fast: Vec::new(),
    }
}

impl FestivalRule {
    /// The lunar days of India's central holidays. The Holika fire is here
    /// for Holi to count from; it is on no list.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalRule;
    ///
    /// let rules = FestivalRule::india_central();
    /// let janmashtami = rules.iter().find(|rule| rule.key == "JANMASHTAMI").ok_or("shipped")?;
    /// assert!(janmashtami.source.contains("C851") && janmashtami.meta.optional);
    /// let diwali = rules.iter().find(|rule| rule.key == "LAKSHMI_PUJA").ok_or("shipped")?;
    /// assert!(diwali.source.starts_with("Dharmasindhu") && !diwali.meta.optional);
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_central() -> Vec<FestivalRule> {
        let of =
            |key: &str, rules: &[FestivalRule]| rules.iter().find(|rule| rule.key == key).cloned();
        let text = FestivalRule::dharmasindhu();
        let print = FestivalRule::nepal_panchanga();
        let hindu = Some(Tradition::Hindu);
        let by_text = [
            ("RAMA_NAVAMI", true),
            ("SHIVARATRI", true),
            ("VIJAYA_DASHAMI", false),
            ("LAKSHMI_PUJA", false),
        ]
        .into_iter()
        .filter_map(|(key, optional)| {
            of(key, &text).map(|rule| FestivalRule {
                meta: listed(hindu, optional).with_fast(key),
                ..rule
            })
        });
        let printed = [
            ("BUDDHA_JAYANTI", Tradition::Buddhist),
            ("MAHAVIR_JAYANTI", Tradition::Jain),
            ("GURU_NANAK_JAYANTI", Tradition::Sikh),
        ]
        .into_iter()
        .filter_map(|(key, tradition)| {
            of(key, &print).map(|rule| FestivalRule {
                meta: listed(Some(tradition), false),
                ..rule
            })
        });
        let janmashtami = of("JANMASHTAMI", &text).map(|rule| {
            let udaya = rule.udaya();
            FestivalRule {
                source: format!(
                    "fitted to the notices' Janmashtami, which they mark Vaishnava; no Vaishnava text in hand states it (C175, C851). {}",
                    udaya.source
                ),
                meta: listed(hindu, true).with_fast("JANMASHTAMI"),
                ..udaya
            }
        });
        let holika = of("HOLIKA", &text).map(|rule| FestivalRule {
            meta: FestivalMeta {
                regions: vec!["IN".to_owned()],
                ..FestivalMeta::hindu_text()
            },
            ..rule
        });
        by_text
            .chain(printed)
            .chain(janmashtami)
            .chain(holika)
            .collect()
    }
}

impl FollowingRule {
    /// Holi, the day after the Holika fire, as *Dharmasindhu* p. 95 keeps
    /// the honouring of its ash and the notices list the day.
    ///
    /// ```
    /// use teistro_panchanga::festival::FollowingRule;
    ///
    /// let holi = &FollowingRule::india_central()[0];
    /// assert_eq!((holi.after.as_str(), holi.days), ("HOLIKA", 1));
    /// ```
    #[must_use]
    pub fn india_central() -> Vec<FollowingRule> {
        vec![FollowingRule {
            key: "HOLI".to_owned(),
            source: format!(
                "{OPTIONAL}: Holi; Dharmasindhu p. 95: the morning after the fire, on the pratipada"
            ),
            after: "HOLIKA".to_owned(),
            days: 1,
            meta: listed(Some(Tradition::Hindu), true),
        }]
    }
}

impl DatedRule {
    /// The holidays on a civil date: the three national days, Christmas,
    /// and Good Friday two days before Easter by the Western computus.
    ///
    /// ```
    /// use teistro_panchanga::festival::DatedRule;
    ///
    /// let rules = DatedRule::india_central();
    /// assert_eq!(rules.len(), 5);
    /// assert!(rules.iter().all(|rule| rule.meta.kind.is_some() && !rule.meta.optional));
    /// ```
    #[must_use]
    pub fn india_central() -> Vec<DatedRule> {
        let civil = |key: &str, month: u8, day: i8, tradition: Option<Tradition>| DatedRule {
            key: key.to_owned(),
            source: NOTICE.to_owned(),
            on: Dated::CivilDate {
                calendar: Calendar::Gregorian,
                month,
                day,
                skipped: None,
            },
            meta: listed(tradition, false),
        };
        vec![
            civil("INDIA_REPUBLIC_DAY", 1, 26, None),
            civil("INDIA_INDEPENDENCE_DAY", 8, 15, None),
            civil("GANDHI_JAYANTI", 10, 2, None),
            civil("CHRISTMAS", 12, 25, Some(Tradition::Christian)),
            DatedRule {
                key: "GOOD_FRIDAY".to_owned(),
                source: NOTICE.to_owned(),
                on: Dated::Easter {
                    computus: teistro_calendar::easter::Computus::Gregorian,
                    days: -2,
                },
                meta: listed(Some(Tradition::Christian), false),
            },
        ]
    }
}

impl PublishedSet {
    /// The days the notices set by the moon, for the years a notice is in
    /// hand. Each stands on the printed day marked subject to the sighting,
    /// or on the day the Department later declared, unmarked.
    ///
    /// ```
    /// use teistro_panchanga::festival::{PublishedEntry, PublishedSet, SubjectTo};
    ///
    /// let set = PublishedSet::india_central();
    /// assert_eq!(set.entries.len(), 28);
    /// assert!(set.check().is_ok());
    /// let PublishedEntry::Add { day, subject_to, .. } = &set.entries[0] else { panic!("an ADD") };
    /// assert_eq!((day.year, day.month, day.day), (2021, 5, 14));
    /// assert_eq!(*subject_to, Some(SubjectTo::MoonSighting));
    /// ```
    #[must_use]
    pub fn india_central() -> PublishedSet {
        let entry = |key: &str, en: &str, ne: &str, year: i32, memorandum: &str, day| {
            let declared = DECLARED.iter().find(|row| row.0 == key && row.1 == year);
            PublishedEntry::Add {
                key: key.to_owned(),
                day: gregorian(year, declared.map_or(day, |row| row.2)),
                names: [
                    ("en-Latn".to_owned(), en.to_owned()),
                    ("ne-Deva-NP".to_owned(), ne.to_owned()),
                ]
                .into(),
                window: None,
                source: declared.map_or_else(
                    || format!("DoPT O.M. {memorandum}, Annexure I, its day subject to para 5.1"),
                    |row| row.3.to_owned(),
                ),
                subject_to: declared.is_none().then_some(SubjectTo::MoonSighting),
                meta: listed(Some(Tradition::Islamic), false),
            }
        };
        let entries = SIGHTED
            .iter()
            .flat_map(|&(key, en, ne, days)| {
                MEMORANDA
                    .iter()
                    .zip(days)
                    .map(move |(&(year, memorandum), day)| (key, en, ne, year, memorandum, day))
            })
            .map(|(key, en, ne, year, memorandum, day)| entry(key, en, ne, year, memorandum, day))
            .collect();
        PublishedSet {
            format: PUBLISHED_FORMAT.to_owned(),
            publisher: "Government of India, Department of Personnel and Training".to_owned(),
            id: "in-dopt-moon-sighted".to_owned(),
            version: 1,
            entries,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn a_declared_day_replaces_the_printed_one_and_drops_the_mark() {
        let set = PublishedSet::india_central();
        let moved: Vec<_> = set
            .entries
            .iter()
            .filter_map(|entry| match entry {
                PublishedEntry::Add {
                    key,
                    day,
                    subject_to: None,
                    source,
                    ..
                } => Some((key.as_str(), day.year, day.month, day.day, source.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(moved.len(), DECLARED.len());
        assert!(
            moved
                .iter()
                .any(|row| matches!(row, ("MUHARRAM", 2021, 8, 20, _)))
        );
        assert!(
            moved
                .iter()
                .any(|row| matches!(row, ("EID_AL_ADHA", 2026, 5, 28, _)))
        );
    }

    #[test]
    fn every_rule_of_the_list_stands_in_india_and_is_gazetted() {
        for meta in FestivalRule::india_central()
            .iter()
            .filter(|rule| rule.key != "HOLIKA")
            .map(|rule| &rule.meta)
        {
            assert_eq!(meta.kind, Some(FestivalKind::Gazetted));
            assert_eq!(meta.regions, ["IN"]);
        }
    }
}

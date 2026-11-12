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
use teistro_core::catalogue::{Calendar, FestivalKind, Masa, Nakshatra, Tithi, Tradition};

use super::nepal_panchanga::Fit;
use super::published::{PUBLISHED_FORMAT, SubjectTo};
use super::{
    Dated, DatedRule, FestivalMeta, FestivalRule, FollowingRule, InAdhika, PublishedEntry,
    PublishedSet, Which,
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

/// A day a list prints rather than reckons: its key, English name, Nepali
/// name, and the days the notices print, as (month, day), one for each
/// notice in order.
pub(super) type Printed<'a> = (&'a str, &'a str, &'a str, &'a [(u8, u8)]);

/// A printed day's key, English name and Nepali name.
pub(super) type Named<'a> = (&'a str, &'a str, &'a str);

/// A day one year's notice prints: its year, key and (month, day).
pub(super) type OnDay<'a> = (i32, &'a str, (u8, u8));

/// A state's own lunar day: key, amanta month, tithi, how the rule fits
/// its two days, whether the list holds it optional, and what the day is.
pub(super) type Own = (&'static str, Masa, Tithi, Fit, bool, &'static str);

/// A state's day on a Gregorian date: key, (month, day), tradition,
/// communities, and whether the list holds it optional.
pub(super) type Civil = (
    &'static str,
    (u8, i8),
    Option<Tradition>,
    &'static [&'static str],
    bool,
);

/// A day moved after its notice: key, year, the day, and the order.
pub(super) type Declared = (&'static str, i32, (u8, u8), &'static str);

/// The days set by the moon: key, English name, Nepali name, and the days
/// the notices print, in the order of [`MEMORANDA`].
pub(super) const SIGHTED: [Printed<'static>; 4] = [
    (
        "EID_AL_FITR",
        "Id-ul-Fitr",
        "ईद उल फित्र",
        &[(5, 14), (5, 3), (4, 22), (4, 11), (3, 31), (3, 21), (3, 10)],
    ),
    (
        "EID_AL_ADHA",
        "Id-ul-Zuha (Bakrid)",
        "बकर ईद",
        &[(7, 21), (7, 10), (6, 29), (6, 17), (6, 7), (5, 27), (5, 17)],
    ),
    (
        "MUHARRAM",
        "Muharram",
        "मोहर्रम",
        &[(8, 19), (8, 9), (7, 29), (7, 17), (7, 6), (6, 26), (6, 16)],
    ),
    (
        "MAWLID",
        "Milad-un-Nabi or Id-e-Milad",
        "मोहम्मद जयन्ती",
        &[
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
const DECLARED: [Declared; 2] = [
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

/// A list's day on a Gregorian date `(month, day)`, citing `source`.
pub(super) fn on_civil_date(
    key: &str,
    (month, day): (u8, i8),
    source: &str,
    meta: FestivalMeta,
) -> DatedRule {
    DatedRule {
        key: key.to_owned(),
        source: source.to_owned(),
        on: Dated::CivilDate {
            calendar: Calendar::Gregorian,
            month,
            day,
            skipped: None,
        },
        meta,
    }
}

/// The values of a rule of a government's list kept in `regions`:
/// `tradition` as the rule's own text or the notice's name says. A list
/// names no fast; a rule taken from a text gains that text's with
/// [`FestivalMeta::with_fast`].
pub(super) fn gazetted(
    tradition: Option<Tradition>,
    regions: &[&str],
    optional: bool,
) -> FestivalMeta {
    FestivalMeta {
        kind: Some(FestivalKind::Gazetted),
        tradition,
        communities: Vec::new(),
        regions: regions.iter().map(|&region| region.to_owned()).collect(),
        optional,
        fast: Vec::new(),
    }
}

/// The values of a rule of the central list, kept in India.
fn listed(tradition: Option<Tradition>, optional: bool) -> FestivalMeta {
    gazetted(tradition, &["IN"], optional)
}

/// How a list's printed days stand: the notices they come from, the days
/// moved after them, what a printed day is subject to, and the values and
/// source each entry carries.
pub(super) struct PrintedDays<'a> {
    /// Each year's notice, in the order of a row's days.
    pub(super) notices: &'a [(i32, &'a str)],
    /// The days moved after the notice, each standing unmarked on the day
    /// declared and citing its order.
    pub(super) declared: &'a [Declared],
    /// What a printed day is subject to.
    pub(super) subject_to: Option<SubjectTo>,
    /// Every entry's values.
    pub(super) meta: FestivalMeta,
    /// A printed day's source, from its year's notice.
    pub(super) cite: fn(&str) -> String,
}

impl PrintedDays<'_> {
    /// An `ADD` entry for each day of each row, a year of
    /// [`PrintedDays::notices`] apiece.
    pub(super) fn entries(&self, rows: &[Printed<'_>]) -> Vec<PublishedEntry> {
        rows.iter()
            .flat_map(|&(key, en, ne, days)| {
                self.notices
                    .iter()
                    .zip(days)
                    .map(move |(&(year, notice), &day)| {
                        self.entry((key, en, ne), year, notice, day)
                    })
            })
            .collect()
    }

    /// An `ADD` entry for each of `days`, a day a year's notice prints
    /// once or more and other years not at all: its year, key and day,
    /// named from `names` by its key. A day whose year has no notice or
    /// whose key no name is left out.
    pub(super) fn on_days(&self, names: &[Named<'_>], days: &[OnDay<'_>]) -> Vec<PublishedEntry> {
        days.iter()
            .filter_map(|&(year, key, day)| {
                let found = names.iter().find(|row| row.0 == key)?;
                let notice = self.notices.iter().find(|row| row.0 == year)?;
                Some(self.entry(*found, year, notice.1, day))
            })
            .collect()
    }

    /// The entry of `key` on `day` of `year`, from its `notice` or the
    /// order that moved it.
    fn entry(
        &self,
        (key, en, ne): Named<'_>,
        year: i32,
        notice: &str,
        day: (u8, u8),
    ) -> PublishedEntry {
        let declared = self
            .declared
            .iter()
            .find(|row| row.0 == key && row.1 == year);
        PublishedEntry::Add {
            key: key.to_owned(),
            day: gregorian(year, declared.map_or(day, |row| row.2)),
            names: [
                ("en-Latn".to_owned(), en.to_owned()),
                ("ne-Deva-NP".to_owned(), ne.to_owned()),
            ]
            .into(),
            window: None,
            source: declared.map_or_else(|| (self.cite)(notice), |row| row.3.to_owned()),
            subject_to: self.subject_to.filter(|_| declared.is_none()),
            meta: self.meta.clone(),
        }
    }
}

/// The first of `rules` with `key`.
/// The citation a taken rule keeps when the list that keeps it adds
/// nothing of its own: the rule's own source.
pub(super) fn as_cited(_key: &str, rule: &FestivalRule) -> String {
    rule.source.clone()
}

pub(super) fn find(rules: &[FestivalRule], key: &str) -> Option<FestivalRule> {
    rules.iter().find(|rule| rule.key == key).cloned()
}

/// A state's list of public holidays, kept in its regions: the values of
/// its days, and the rules it takes from the central list and the shipped
/// texts, so a state pack and the packs it draws on cannot drift apart.
pub(super) struct StateList {
    /// Where the list is kept: India, then the state.
    pub(super) regions: &'static [&'static str],
}

impl StateList {
    /// The values of a day of the list.
    pub(super) fn listed(&self, tradition: Option<Tradition>, optional: bool) -> FestivalMeta {
        gazetted(tradition, self.regions, optional)
    }

    /// A rule of the central list, kept in the state: the same rule and
    /// values, the state's regions, and on every year's list.
    pub(super) fn kept(&self, meta: FestivalMeta) -> FestivalMeta {
        FestivalMeta {
            regions: self
                .regions
                .iter()
                .map(|&region| region.to_owned())
                .collect(),
            optional: false,
            ..meta
        }
    }

    /// The central list's lunar rules but those keyed in `left`, kept in
    /// the state.
    pub(super) fn central(&self, left: &[&str]) -> Vec<FestivalRule> {
        FestivalRule::india_central()
            .into_iter()
            .filter(|rule| !left.contains(&rule.key.as_str()))
            .map(|rule| FestivalRule {
                meta: self.kept(rule.meta),
                ..rule
            })
            .collect()
    }

    /// The central list's civil days, kept in the state and citing its
    /// `notice`.
    pub(super) fn central_dated(&self, notice: &str) -> Vec<DatedRule> {
        DatedRule::india_central()
            .into_iter()
            .map(|rule| DatedRule {
                source: notice.to_owned(),
                meta: self.kept(rule.meta),
                ..rule
            })
            .collect()
    }

    /// *Dharmasindhu*'s rule of each `(key, optional)`, on the list with
    /// the fast the text gives it.
    pub(super) fn by_text(&self, keys: &[(&str, bool)]) -> Vec<FestivalRule> {
        let text = FestivalRule::dharmasindhu();
        keys.iter()
            .filter_map(|&(key, optional)| {
                find(&text, key).map(|rule| FestivalRule {
                    meta: self.listed(Some(Tradition::Hindu), optional).with_fast(key),
                    ..rule
                })
            })
            .collect()
    }

    /// Nepal's printed panchanga's rule of each key, on the list.
    pub(super) fn by_print(&self, keys: &[&str]) -> Vec<FestivalRule> {
        let kept: Vec<(&str, &str, bool)> = keys.iter().map(|&key| (key, key, false)).collect();
        self.print_kept(&kept)
    }

    /// Nepal's printed panchanga's rule of each `(key, from, optional)`,
    /// the print's `from` kept on the list under `key`.
    pub(super) fn print_kept(&self, keys: &[(&str, &str, bool)]) -> Vec<FestivalRule> {
        self.taken(
            &FestivalRule::nepal_panchanga(),
            Tradition::Hindu,
            keys,
            as_cited,
        )
    }

    /// The rule of `rules` keyed `from` of each `(key, from, optional)`,
    /// kept on the list under `key`, of `tradition`, and citing
    /// `cite(key, rule)`: a day another list reckons, which this one keeps
    /// as that list does ([`as_cited`] keeps that list's citation).
    pub(super) fn taken(
        &self,
        rules: &[FestivalRule],
        tradition: Tradition,
        keys: &[(&str, &str, bool)],
        cite: impl Fn(&str, &FestivalRule) -> String,
    ) -> Vec<FestivalRule> {
        keys.iter()
            .filter_map(|&(key, from, optional)| {
                find(rules, from).map(|rule| FestivalRule {
                    key: key.to_owned(),
                    source: cite(key, &rule),
                    meta: self.listed(Some(tradition), optional),
                    ..rule
                })
            })
            .collect()
    }

    /// The central list's lunar rule of each `(key, optional)`, kept in
    /// the state: the same rule and values, the state's regions, and
    /// optional as the state's list holds it.
    pub(super) fn central_kept(&self, keys: &[(&str, bool)]) -> Vec<FestivalRule> {
        let central = FestivalRule::india_central();
        keys.iter()
            .filter_map(|&(key, optional)| {
                find(&central, key).map(|rule| FestivalRule {
                    meta: FestivalMeta {
                        optional,
                        ..self.kept(rule.meta)
                    },
                    ..rule
                })
            })
            .collect()
    }

    /// The state's own lunar rules, each citing `cite(key, what)`.
    pub(super) fn own(
        &self,
        days: &[Own],
        cite: impl Fn(&str, &str) -> String,
    ) -> Vec<FestivalRule> {
        days.iter()
            .map(|&(key, month, tithi, fit, optional, what)| FestivalRule {
                meta: self.listed(Some(Tradition::Hindu), optional),
                ..fit.rule(
                    key,
                    Some(month),
                    tithi.into(),
                    InAdhika::Nija,
                    &cite(key, what),
                )
            })
            .collect()
    }

    /// The state's days on a Gregorian date, citing its `notice`.
    pub(super) fn civil(&self, days: &[Civil], notice: &str) -> Vec<DatedRule> {
        days.iter()
            .map(|&(key, day, tradition, communities, optional)| {
                let meta = FestivalMeta {
                    communities: communities.iter().map(|&one| one.to_owned()).collect(),
                    ..self.listed(tradition, optional)
                };
                on_civil_date(key, day, notice, meta)
            })
            .collect()
    }

    /// A Hindu day of the list on `day` of a solar calendar's `month`, a
    /// negative day counting from the month's end.
    pub(super) fn on_solar_day(
        &self,
        key: &str,
        source: String,
        (calendar, month, day): (Calendar, u8, i8),
        optional: bool,
    ) -> DatedRule {
        DatedRule {
            key: key.to_owned(),
            source,
            on: Dated::CivilDate {
                calendar,
                month,
                day,
                skipped: None,
            },
            meta: self.listed(Some(Tradition::Hindu), optional),
        }
    }

    /// A Hindu day of the list on the day a solar calendar's `month` holds
    /// `nakshatra`, `twice` choosing when the month holds it twice.
    pub(super) fn on_star(
        &self,
        key: &str,
        source: String,
        (calendar, month): (Calendar, u8),
        (nakshatra, twice): (Nakshatra, Which),
        optional: bool,
    ) -> DatedRule {
        DatedRule {
            key: key.to_owned(),
            source,
            on: Dated::NakshatraInMonth {
                calendar,
                month,
                nakshatra,
                twice,
                days: 0,
            },
            meta: self.listed(Some(Tradition::Hindu), optional),
        }
    }
}

/// The central list's day set by the moon of each key, with the days a
/// state's notices print, in their order.
pub(super) fn sighted<'a>(rows: &[(&str, &'a [(u8, u8)])]) -> Vec<Printed<'a>> {
    rows.iter()
        .filter_map(|&(key, days)| {
            SIGHTED
                .iter()
                .find(|row| row.0 == key)
                .map(|&(key, en, ne, _)| (key, en, ne, days))
        })
        .collect()
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
    /// assert!(janmashtami.source.contains("Hari Bhakti Vilasa 15.358"));
    /// let diwali = rules.iter().find(|rule| rule.key == "LAKSHMI_PUJA").ok_or("shipped")?;
    /// assert!(diwali.source.starts_with("Dharmasindhu") && !diwali.meta.optional);
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn india_central() -> Vec<FestivalRule> {
        let of = |key: &str, rules: &[FestivalRule]| find(rules, key);
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
        let vaishnava = FestivalRule::hari_bhakti_vilasa();
        let janmashtami = of("JANMASHTAMI_VAISHNAVA", &vaishnava).map(|rule| FestivalRule {
            key: "JANMASHTAMI".to_owned(),
            source: format!(
                "the notices mark their Janmashtami Vaishnava (C851): {}",
                rule.source
            ),
            meta: listed(hindu, true).with_fast("JANMASHTAMI_VAISHNAVA"),
            ..rule
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
        let civil = |key: &str, month: u8, day: i8, tradition: Option<Tradition>| {
            on_civil_date(key, (month, day), NOTICE, listed(tradition, false))
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
        let entries = PrintedDays {
            notices: &MEMORANDA,
            declared: &DECLARED,
            subject_to: Some(SubjectTo::MoonSighting),
            meta: listed(Some(Tradition::Islamic), false),
            cite: |memorandum| {
                format!("DoPT O.M. {memorandum}, Annexure I, its day subject to para 5.1")
            },
        }
        .entries(&SIGHTED);
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

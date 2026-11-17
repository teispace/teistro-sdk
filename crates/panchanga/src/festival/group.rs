//! Festival groups (`03-design/festival-groups.md`, SR-125): a festival
//! kept over several days, each day its own rule, named as one.
//!
//! A pack names its groups as data: a key (`TIHAR`), the member rules in
//! their order, the main day among them and the source that says which
//! rules make the festival. The reckoning places each member as it places
//! any rule, and [`group_spans`] gathers each occurrence of the main day
//! with the members' days around it into a span: its first and last days,
//! each member's day, and every day between, a day no member falls on
//! marked as a gap. Who keeps a member is the member's own rule's meta
//! (`MHA_PUJA` is `NEWAR`'s), so a group never says it twice.

use serde::{Deserialize, Serialize};
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay};
use teistro_core::error::Error;

use super::{Observances, PublishedStatus};

/// A festival kept over several days: its member rules, its main day, and
/// the source that makes them one festival.
///
/// ```
/// use teistro_panchanga::festival::FestivalGroup;
///
/// let tihar = FestivalGroup::nepal_panchanga().into_iter().find(|g| g.key == "TIHAR").ok_or("shipped")?;
/// assert_eq!(tihar.main, "LAKSHMI_PUJA");
/// assert_eq!(tihar.members.first().map(String::as_str), Some("KAG_TIHAR"));
/// tihar.check()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FestivalGroup {
    /// The group's key, `TIHAR`; a consumer's own names and descriptions
    /// hang on it.
    pub key: String,
    /// The member rules' keys, in the festival's order; each a key the
    /// request carries.
    pub members: Vec<String>,
    /// The member whose day the group is reckoned around: one of
    /// `members`. A span is found for each of its days.
    pub main: String,
    /// The source that says which rules make the festival.
    pub source: String,
}

/// One member of a group's span, in the group's order.
///
/// ```
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_panchanga::festival::{GroupDay, GroupMember, GroupSpan};
///
/// let date = |day| CalendarDate::defined(Calendar::Gregorian, 2026, 11, day);
/// let member = |rule: &str, day| GroupMember { rule: rule.to_owned(), day: Some(date(day)) };
/// let on = |day, members: &[&str]| GroupDay {
///     day: date(day),
///     members: members.iter().map(|&m| m.to_owned()).collect(),
///     gap: members.is_empty(),
/// };
/// let span = GroupSpan {
///     group: "GAI_AND_GOVARDHAN".to_owned(),
///     main: "GOVARDHAN_PUJA".to_owned(),
///     first: date(8),
///     last: date(10),
///     members: vec![member("LAKSHMI_PUJA", 8), member("GOVARDHAN_PUJA", 10)],
///     days: vec![on(8, &["LAKSHMI_PUJA"]), on(9, &[]), on(10, &["GOVARDHAN_PUJA"])],
/// };
/// assert_eq!(span.members[1].rule, "GOVARDHAN_PUJA");
/// assert!(span.members.iter().all(|m| m.day.is_some()));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GroupMember {
    /// The member rule's key.
    pub rule: String,
    /// Its day in this span; absent when it falls on none within
    /// [`FestivalGroup::MOST_REACH_DAYS`] of the main day, or a published
    /// set withdrew it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day: Option<CalendarDate>,
}

/// One day of a group's span.
///
/// ```
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_panchanga::festival::{GroupDay, GroupMember, GroupSpan};
///
/// let date = |day| CalendarDate::defined(Calendar::Gregorian, 2026, 11, day);
/// let member = |rule: &str, day| GroupMember { rule: rule.to_owned(), day: Some(date(day)) };
/// let on = |day, members: &[&str]| GroupDay {
///     day: date(day),
///     members: members.iter().map(|&m| m.to_owned()).collect(),
///     gap: members.is_empty(),
/// };
/// let span = GroupSpan {
///     group: "GAI_AND_GOVARDHAN".to_owned(),
///     main: "GOVARDHAN_PUJA".to_owned(),
///     first: date(8),
///     last: date(10),
///     members: vec![member("LAKSHMI_PUJA", 8), member("GOVARDHAN_PUJA", 10)],
///     days: vec![on(8, &["LAKSHMI_PUJA"]), on(9, &[]), on(10, &["GOVARDHAN_PUJA"])],
/// };
/// assert!(span.days[1].gap && span.days[1].members.is_empty());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GroupDay {
    /// The day.
    pub day: CalendarDate,
    /// The members that fall on it, in the group's order; empty on a gap.
    pub members: Vec<String>,
    /// Whether no member falls on it: a day inside the span all the same,
    /// as when a tithi holds two sunrises between two members' days.
    pub gap: bool,
}

/// A group's occurrence: the days from its first member's to its last,
/// each member's day, and every day between.
///
/// ```
/// use teistro_calendar::CalendarDate;
/// use teistro_core::catalogue::Calendar;
/// use teistro_panchanga::festival::{GroupDay, GroupMember, GroupSpan};
///
/// let date = |day| CalendarDate::defined(Calendar::Gregorian, 2026, 11, day);
/// let member = |rule: &str, day| GroupMember { rule: rule.to_owned(), day: Some(date(day)) };
/// let on = |day, members: &[&str]| GroupDay {
///     day: date(day),
///     members: members.iter().map(|&m| m.to_owned()).collect(),
///     gap: members.is_empty(),
/// };
/// let span = GroupSpan {
///     group: "GAI_AND_GOVARDHAN".to_owned(),
///     main: "GOVARDHAN_PUJA".to_owned(),
///     first: date(8),
///     last: date(10),
///     members: vec![member("LAKSHMI_PUJA", 8), member("GOVARDHAN_PUJA", 10)],
///     days: vec![on(8, &["LAKSHMI_PUJA"]), on(9, &[]), on(10, &["GOVARDHAN_PUJA"])],
/// };
/// assert_eq!(span.days.len(), 3);
/// assert_eq!(span.days.iter().filter(|d| d.gap).count(), 1);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GroupSpan {
    /// The group's key.
    pub group: String,
    /// The main member's key.
    pub main: String,
    /// The span's first day.
    pub first: CalendarDate,
    /// The span's last day, inclusive.
    pub last: CalendarDate,
    /// Each member and its day, in the group's order.
    pub members: Vec<GroupMember>,
    /// Every day from `first` to `last`, in order.
    pub days: Vec<GroupDay>,
}

impl GroupSpan {
    /// The main member's day.
    ///
    /// ```
    /// use teistro_calendar::CalendarDate;
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_panchanga::festival::{GroupDay, GroupMember, GroupSpan};
    ///
    /// let date = |day| CalendarDate::defined(Calendar::Gregorian, 2026, 11, day);
    /// let member = |rule: &str, day| GroupMember { rule: rule.to_owned(), day: Some(date(day)) };
    /// let on = |day, members: &[&str]| GroupDay {
    ///     day: date(day),
    ///     members: members.iter().map(|&m| m.to_owned()).collect(),
    ///     gap: members.is_empty(),
    /// };
    /// let span = GroupSpan {
    ///     group: "GAI_AND_GOVARDHAN".to_owned(),
    ///     main: "GOVARDHAN_PUJA".to_owned(),
    ///     first: date(8),
    ///     last: date(10),
    ///     members: vec![member("LAKSHMI_PUJA", 8), member("GOVARDHAN_PUJA", 10)],
    ///     days: vec![on(8, &["LAKSHMI_PUJA"]), on(9, &[]), on(10, &["GOVARDHAN_PUJA"])],
    /// };
    /// assert_eq!(span.main_day(), Some(&date(10)));
    /// ```
    #[must_use]
    pub fn main_day(&self) -> Option<&CalendarDate> {
        self.members
            .iter()
            .find(|member| member.rule == self.main)
            .and_then(|member| member.day.as_ref())
    }

    /// Whether every member has a day in it.
    ///
    /// ```
    /// use teistro_calendar::CalendarDate;
    /// use teistro_core::catalogue::Calendar;
    /// use teistro_panchanga::festival::{GroupDay, GroupMember, GroupSpan};
    ///
    /// let date = |day| CalendarDate::defined(Calendar::Gregorian, 2026, 11, day);
    /// let member = |rule: &str, day| GroupMember { rule: rule.to_owned(), day: Some(date(day)) };
    /// let on = |day, members: &[&str]| GroupDay {
    ///     day: date(day),
    ///     members: members.iter().map(|&m| m.to_owned()).collect(),
    ///     gap: members.is_empty(),
    /// };
    /// let mut span = GroupSpan {
    ///     group: "GAI_AND_GOVARDHAN".to_owned(),
    ///     main: "GOVARDHAN_PUJA".to_owned(),
    ///     first: date(8),
    ///     last: date(10),
    ///     members: vec![member("LAKSHMI_PUJA", 8), member("GOVARDHAN_PUJA", 10)],
    ///     days: vec![on(8, &["LAKSHMI_PUJA"]), on(9, &[]), on(10, &["GOVARDHAN_PUJA"])],
    /// };
    /// assert!(span.is_whole());
    /// span.members[0].day = None;
    /// assert!(!span.is_whole());
    /// ```
    #[must_use]
    pub fn is_whole(&self) -> bool {
        self.members.iter().all(|member| member.day.is_some())
    }
}

impl FestivalGroup {
    /// How far a member's day may lie from the main day, in days: a
    /// member further off is another occurrence's. Dashain's first day is
    /// nine or ten days before Vijaya Dashami, the furthest any shipped
    /// group reaches.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalGroup;
    ///
    /// assert_eq!(FestivalGroup::MOST_REACH_DAYS, 20);
    /// ```
    pub const MOST_REACH_DAYS: i64 = 20;

    /// The most members a group may have.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalGroup;
    ///
    /// let group = FestivalGroup {
    ///     key: "LONG".to_owned(),
    ///     members: (0..=FestivalGroup::MOST_MEMBERS).map(|n| format!("R{n}")).collect(),
    ///     main: "R0".to_owned(),
    ///     source: "an example".to_owned(),
    /// };
    /// assert_eq!(group.check().unwrap_err().field(), Some("members"));
    /// ```
    pub const MOST_MEMBERS: usize = 32;

    /// Refuses a group no reckoning could answer: no key, no members, a
    /// member given twice, or a main day that is not a member. Whether
    /// each member is a rule of the request is the request's to check.
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalGroup;
    ///
    /// let group = FestivalGroup {
    ///     key: "ACME".to_owned(),
    ///     members: vec!["A".to_owned(), "B".to_owned()],
    ///     main: "C".to_owned(),
    ///     source: "an example".to_owned(),
    /// };
    /// assert_eq!(group.check().unwrap_err().field(), Some("main"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `key`, `members` or `main`; `LIMIT` naming
    /// `members` past [`FestivalGroup::MOST_MEMBERS`].
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg("a festival group needs a key").with_field("key"));
        }
        if self.members.is_empty() {
            return Err(Error::invalid_arg(format!(
                "{}: a group names the rules that make it",
                self.key
            ))
            .with_field("members"));
        }
        if self.members.len() > FestivalGroup::MOST_MEMBERS {
            return Err(Error::limit(format!(
                "{}: a group has at most {} members, not {}",
                self.key,
                FestivalGroup::MOST_MEMBERS,
                self.members.len()
            ))
            .with_field("members"));
        }
        for (index, member) in self.members.iter().enumerate() {
            if self.members.iter().take(index).any(|held| held == member) {
                return Err(Error::invalid_arg(format!(
                    "{}: `{member}` is a member twice",
                    self.key
                ))
                .with_field(format!("members[{index}]")));
            }
        }
        if !self.members.contains(&self.main) {
            return Err(Error::invalid_arg(format!(
                "{}: the main day `{}` is none of the members",
                self.key, self.main
            ))
            .with_field("main"));
        }
        Ok(())
    }
}

/// A group shipped in a pack: key, members, main day and source.
type Shipped = (
    &'static str,
    &'static [&'static str],
    &'static str,
    &'static str,
);

/// The groups as a pack's table writes them.
fn shipped(table: &[Shipped]) -> Vec<FestivalGroup> {
    table
        .iter()
        .map(|&(key, members, main, source)| FestivalGroup {
            key: key.to_owned(),
            members: members.iter().map(|&member| member.to_owned()).collect(),
            main: main.to_owned(),
            source: source.to_owned(),
        })
        .collect()
}

/// Dashain as Nepal keeps it, from the jar's setting to the full moon.
const DASHAIN: Shipped = (
    "DASHAIN",
    &[
        "NAVARATRA_ARAMBHA",
        "PHULPATI",
        "MAHA_ASHTAMI",
        "MAHA_NAVAMI",
        "VIJAYA_DASHAMI",
        "KOJAGRAT_PURNIMA",
    ],
    "VIJAYA_DASHAMI",
    "Dharmasindhu p. 65 (the Navaratra begun on Ashvina's bright 1st), p. 69 (the Mahanavami), pp. 70-71 (the visarjana and the Vijaya Dashami) and p. 74 (the Kojagara on the full moon); the national panchanga, VS 2082 p. 13 and VS 2083 p. 15, prints each with the Saptami's फूलपाती and the Mahashtami; the Ministry of Home Affairs' VS 2083 notice gives Ghatasthapana and \"दशैं\" from Phulpati to the 12th; the tika is given from the Dashami to the full moon, where Dashain ends (C2101)",
);

/// Tihar, the yamapanchaka as Nepal keeps it.
const TIHAR: Shipped = (
    "TIHAR",
    &[
        "KAG_TIHAR",
        "KUKUR_TIHAR",
        "LAKSHMI_PUJA",
        "GOVARDHAN_PUJA",
        "MHA_PUJA",
        "YAMA_DWITIYA",
    ],
    "LAKSHMI_PUJA",
    "Dharmasindhu p. 77 (Lakshmi's worship on the new moon), p. 78 (the Bali and Govardhan puja) and p. 79 (the Yama dwitiya), the days from Ashvina's dark 13th to Kartika's bright 2nd Nepal keeps as the yamapanchaka (C613); the national panchanga, VS 2082 pp. 14-15 and VS 2083 pp. 16-17, prints the crows' and dogs' days, Lakshmi puja, Govardhan and Mha puja and Bhai Tika; the Ministry of Home Affairs' VS 2083 notice gives \"तिहार\" from Lakshmi puja to the day after Bhai Tika; Mha puja is the Newar community's (its meta), Dhanteras outside the five (C2102, C2103)",
);

/// The Durga Puja as the eastern states' notices give it, from its
/// Saptami to its Dashami.
const DURGA_PUJA: Shipped = (
    "DURGA_PUJA",
    &["PHULPATI", "MAHA_ASHTAMI", "MAHA_NAVAMI", "VIJAYA_DASHAMI"],
    "MAHA_ASHTAMI",
    "Dharmasindhu p. 69 (the Mahanavami joined to the 8th) and p. 70 (the visarjana on the 10th); the state's notification lists the Durga Puja's Saptami, Ashtami, Navami and Dashami, the Saptami its patrika day (C2104)",
);

impl FestivalGroup {
    /// The groups of Nepal's national panchanga: Dashain and Tihar
    /// (`festival-groups.md` §2).
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalGroup;
    ///
    /// let keys: Vec<String> = FestivalGroup::nepal_panchanga().into_iter().map(|g| g.key).collect();
    /// assert_eq!(keys, ["DASHAIN", "TIHAR"]);
    /// ```
    #[must_use]
    pub fn nepal_panchanga() -> Vec<FestivalGroup> {
        shipped(&[DASHAIN, TIHAR])
    }

    /// Bihar's groups: the Durga Puja the notification names, the
    /// Navaratra from its first day, and Chhath from Kharna to the
    /// morning arghya (`festival-groups.md` §3).
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalGroup;
    ///
    /// let keys: Vec<String> = FestivalGroup::india_bihar().into_iter().map(|g| g.key).collect();
    /// assert_eq!(keys, ["DURGA_PUJA", "NAVARATRI", "CHHATH"]);
    /// ```
    #[must_use]
    pub fn india_bihar() -> Vec<FestivalGroup> {
        shipped(&[
            DURGA_PUJA,
            (
                "NAVARATRI",
                &[
                    "NAVARATRA_ARAMBHA",
                    "PHULPATI",
                    "MAHA_ASHTAMI",
                    "MAHA_NAVAMI",
                ],
                "MAHA_ASHTAMI",
                "Dharmasindhu p. 65 (the Navaratra begun on Ashvina's bright 1st) and p. 69 (its 9th, joined to the 8th); Bihar's notification lists the Navaratra's first day and the Puja's Saptami and Ashtami (C2105)",
            ),
            (
                "CHHATH",
                &["CHHATH_KHARNA", "CHHATH", "CHHATH_SECOND_DAY"],
                "CHHATH",
                "Bihar's notification, Schedule-2: Kharna the day before Chhath's evening arghya and the morning arghya the day after, Chhath itself on Kartika's bright 6th; Nahay-Khay, the day before Kharna, no rule of the pack gives (C2106)",
            ),
        ])
    }

    /// Uttar Pradesh's group: Deepawali's days from Narak Chaturdashi to
    /// the Yama dwitiya (`festival-groups.md` §3).
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalGroup;
    ///
    /// let keys: Vec<String> = FestivalGroup::india_uttar_pradesh().into_iter().map(|g| g.key).collect();
    /// assert_eq!(keys, ["DEEPAWALI"]);
    /// ```
    #[must_use]
    pub fn india_uttar_pradesh() -> Vec<FestivalGroup> {
        shipped(&[(
            "DEEPAWALI",
            &[
                "NARAK_CHATURDASHI",
                "LAKSHMI_PUJA",
                "GOVARDHAN_PUJA",
                "YAMA_DWITIYA",
            ],
            "LAKSHMI_PUJA",
            "Dharmasindhu p. 77 (Lakshmi's worship on the new moon), p. 78 (the Bali and Govardhan puja) and p. 79 (the Yama dwitiya); the state's lists give each with Narak Chaturdashi the day before Deepawali (C2102)",
        )])
    }

    /// West Bengal's Durga Puja, from its Shashthi (the bodhana) to its
    /// Dashami (`festival-groups.md` §3).
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalGroup;
    ///
    /// let keys: Vec<String> = FestivalGroup::india_west_bengal().into_iter().map(|g| g.key).collect();
    /// assert_eq!(keys, ["DURGA_PUJA"]);
    /// ```
    #[must_use]
    pub fn india_west_bengal() -> Vec<FestivalGroup> {
        shipped(&[(
            "DURGA_PUJA",
            &[
                "DURGA_SHASHTHI",
                "PHULPATI",
                "MAHA_ASHTAMI",
                "MAHA_NAVAMI",
                "VIJAYA_DASHAMI",
            ],
            "MAHA_ASHTAMI",
            "Dharmasindhu p. 69 (the Mahanavami) and p. 70 (the visarjana on the 10th); the Finance (Audit) Department's lists give the Shashthi to the Dashami, the rite's days from its bodhana to its visarjana, and List II's Chaturthi and Panchami, which it also calls the Puja's, are holidays before the rite (C2104)",
        )])
    }

    /// Assam's Durga Puja (`festival-groups.md` §3).
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalGroup;
    ///
    /// let keys: Vec<String> = FestivalGroup::india_assam().into_iter().map(|g| g.key).collect();
    /// assert_eq!(keys, ["DURGA_PUJA"]);
    /// ```
    #[must_use]
    pub fn india_assam() -> Vec<FestivalGroup> {
        shipped(&[DURGA_PUJA])
    }

    /// Odisha's Durga Puja (`festival-groups.md` §3).
    ///
    /// ```
    /// use teistro_panchanga::festival::FestivalGroup;
    ///
    /// let keys: Vec<String> = FestivalGroup::india_odisha().into_iter().map(|g| g.key).collect();
    /// assert_eq!(keys, ["DURGA_PUJA"]);
    /// ```
    #[must_use]
    pub fn india_odisha() -> Vec<FestivalGroup> {
        shipped(&[DURGA_PUJA])
    }
}

/// Whether a published set withdrew the day.
fn standing(published: Option<&super::Published>) -> bool {
    published.is_none_or(|p| p.status != PublishedStatus::Withdrawn)
}

/// Each rule's standing days in an answer, by key: the observances, the
/// fasts, the dated days, the anniversaries and the days a set added.
fn days_by_key(found: &Observances) -> Vec<(&str, &CalendarDate)> {
    found
        .observances
        .iter()
        .filter(|o| standing(o.published.as_ref()))
        .map(|o| (o.rule.as_str(), &o.day))
        .chain(
            found
                .ekadashis
                .iter()
                .filter(|f| standing(f.published.as_ref()))
                .map(|f| (f.rule.as_str(), &f.day)),
        )
        .chain(
            found
                .dated
                .iter()
                .filter(|d| standing(d.published.as_ref()))
                .map(|d| (d.rule.as_str(), &d.day)),
        )
        .chain(
            found
                .anniversaries
                .iter()
                .filter(|a| standing(a.observance.published.as_ref()))
                .map(|a| (a.key.as_str(), &a.observance.day)),
        )
        .chain(found.added.iter().map(|a| (a.key.as_str(), &a.day)))
        .collect()
}

/// Every group's spans in an answer, in the order of their first days
/// (the groups' order on a tie), and the member days no span took: a
/// member whose main day lies outside the days reckoned, which a caller
/// reckons further for (`festival-groups.md` §4).
///
/// A span is found for each day of a group's main member. Each other
/// member takes its day nearest the main day, the earlier on a tie,
/// within [`FestivalGroup::MOST_REACH_DAYS`]. The span runs from the
/// earliest day taken to the latest, and lists every day between.
///
/// ```
/// use teistro_calendar::Gregorian;
/// use teistro_panchanga::festival::{FestivalGroup, Observances, group_spans};
///
/// let (spans, loose) = group_spans(&FestivalGroup::nepal_panchanga(), &Observances::default(), &Gregorian)?;
/// assert!(spans.is_empty() && loose.is_empty());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A date the calendar cannot read, which a reckoning in it never gives.
pub fn group_spans(
    groups: &[FestivalGroup],
    found: &Observances,
    calendar: &dyn CalendarSystem,
) -> Result<(Vec<GroupSpan>, Vec<FixedDay>), Error> {
    let mut held: Vec<(&str, FixedDay)> = Vec::new();
    for (key, date) in days_by_key(found) {
        let fixed = calendar.fixed_of(date)?;
        if !held.contains(&(key, fixed)) {
            held.push((key, fixed));
        }
    }
    held.sort_by_key(|&(_, fixed)| fixed);
    let mut spans: Vec<(FixedDay, usize, GroupSpan)> = Vec::new();
    let mut loose: Vec<FixedDay> = Vec::new();
    for (order, group) in groups.iter().enumerate() {
        let days_of = |member: &str| -> Vec<FixedDay> {
            held.iter()
                .filter(|(key, _)| *key == member)
                .map(|&(_, fixed)| fixed)
                .collect()
        };
        let mut taken: Vec<FixedDay> = Vec::new();
        for main in days_of(&group.main) {
            let picked: Vec<Option<FixedDay>> = group
                .members
                .iter()
                .map(|member| {
                    if *member == group.main {
                        return Some(main);
                    }
                    days_of(member)
                        .into_iter()
                        .filter(|day| main.days_until(*day).abs() <= FestivalGroup::MOST_REACH_DAYS)
                        .min_by_key(|day| (main.days_until(*day).abs(), *day))
                })
                .collect();
            let (Some(first), Some(last)) = (
                picked.iter().flatten().min().copied(),
                picked.iter().flatten().max().copied(),
            ) else {
                continue;
            };
            taken.extend(picked.iter().flatten().copied());
            let members = group
                .members
                .iter()
                .zip(&picked)
                .map(|(rule, day)| {
                    Ok(GroupMember {
                        rule: rule.clone(),
                        day: day.map(|day| calendar.date_of(day)).transpose()?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let mut days = Vec::new();
            let mut day = first;
            while day <= last {
                let on: Vec<String> = group
                    .members
                    .iter()
                    .zip(&picked)
                    .filter(|(_, picked)| **picked == Some(day))
                    .map(|(rule, _)| rule.clone())
                    .collect();
                days.push(GroupDay {
                    day: calendar.date_of(day)?,
                    gap: on.is_empty(),
                    members: on,
                });
                day = day.plus_days(1);
            }
            spans.push((
                first,
                order,
                GroupSpan {
                    group: group.key.clone(),
                    main: group.main.clone(),
                    first: calendar.date_of(first)?,
                    last: calendar.date_of(last)?,
                    members,
                    days,
                },
            ));
        }
        for member in group.members.iter().filter(|member| **member != group.main) {
            loose.extend(
                days_of(member)
                    .into_iter()
                    .filter(|day| !taken.contains(day)),
            );
        }
    }
    spans.sort_by_key(|&(first, order, _)| (first, order));
    loose.sort();
    loose.dedup();
    Ok((spans.into_iter().map(|(_, _, span)| span).collect(), loose))
}

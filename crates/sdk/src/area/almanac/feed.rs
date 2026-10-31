//! A year of observances as an iCalendar feed a calendar app subscribes
//! to (`03-design/icalendar.md`, P2-2).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::ical::{self, Alarm, Color, Event, Feed, Hints, When};
use teistro_calendar::{CalendarDate, FixedDay};
use teistro_core::catalogue::Calendar;
use teistro_core::envelope::{Envelope, Hash, canonical_json};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_intl::messages::sdk::calendar::Parana;
use teistro_panchanga::festival::{Published, PublishedStatus};

use super::AlmanacArea;
use crate::area::system_of;
use crate::asked::{DayAsked, offset_of, place_of};
use crate::festival_request::FestivalRequest;

/// The field a feed request's refusals are named under.
const FESTIVAL_CALENDAR: &str = "festivalCalendar";

/// The fixed day of 1970-01-01.
const UNIX_EPOCH_DAY: i64 = 719_163;

/// What a feed says of itself: its name, the product that wrote it, how
/// often a subscriber fetches it again, and the instant every event is
/// stamped with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedOptions {
    /// The calendar's name, as an app lists it.
    pub name: String,
    /// The `PRODID`, as `-//Owner//Product//EN`.
    pub product: String,
    /// How often a subscriber fetches the feed again, in minutes; none
    /// leaves it to the app.
    pub refresh_minutes: Option<u32>,
    /// Every event's `DTSTAMP`, in seconds since 1970-01-01T00:00:00Z.
    /// The caller gives it, so the same request writes the same bytes.
    pub stamp_unix_seconds: i64,
    /// The feed's own categories.
    pub categories: Vec<String>,
    /// The feed's colour.
    pub color: Option<Color>,
    /// The alarm on every event no [`FeedHint`] or event of its own
    /// gives another; none writes none.
    pub alarm: Option<Alarm>,
    /// Which events the feed keeps: those any selector selects, every
    /// event when empty.
    pub only: Vec<FeedSelector>,
    /// Categories, a colour and an alarm by what an event is; the first
    /// that selects an event gives it its own.
    pub hints: Vec<FeedHint>,
    /// The caller's own events, merged into the day's order. One whose
    /// `UID` is a computed event's stands in its place.
    pub events: Vec<SuppliedEvent>,
}

/// How a feed's event came to be, which a [`FeedSelector`] can name.
///
/// ```
/// use teistro::FeedForm;
///
/// assert_eq!(serde_json::to_string(&FeedForm::Parana)?, "\"PARANA\"");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeedForm {
    /// A yearly rule's observance.
    Observance,
    /// An Ekadashi fast.
    Ekadashi,
    /// A dated rule's day.
    Dated,
    /// An anniversary.
    Anniversary,
    /// A day a published set added.
    Added,
    /// The timed window a fast is broken in.
    Parana,
    /// An event the caller supplied.
    Supplied,
}

/// What selects an event: a list left empty asks nothing, a list given
/// holds the event's value, and every list given must. `kinds` and
/// `traditions` read a rule's catalogue fields (`03-design/icalendar.md`
/// §6); `rules` holds rule keys, a parana answering to its fast's and a
/// supplied event to its `UID`.
///
/// ```
/// use teistro::{FeedForm, FeedSelector};
///
/// let fasts: FeedSelector = serde_json::from_str(r#"{"forms": ["EKADASHI", "PARANA"]}"#)?;
/// assert_eq!(fasts.forms, [FeedForm::Ekadashi, FeedForm::Parana]);
/// assert!(fasts.rules.is_empty());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct FeedSelector {
    /// How the event came to be.
    pub forms: Vec<FeedForm>,
    /// A rule's catalogue kind, or a supplied event's own.
    pub kinds: Vec<String>,
    /// A rule's tradition, or a supplied event's own.
    pub traditions: Vec<String>,
    /// A rule's key.
    pub rules: Vec<String>,
}

/// What the events a selector selects say: categories, a colour and an
/// alarm, each optional.
///
/// ```
/// use teistro::{FeedHint, FeedSelector};
///
/// let hint = FeedHint { select: FeedSelector::default(), hints: Default::default() };
/// assert!(hint.hints.categories.is_empty());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FeedHint {
    /// The events it speaks of.
    pub select: FeedSelector,
    /// What they say.
    pub hints: Hints,
}

/// An event the caller supplies, a holiday from its own data, written
/// among the computed ones.
///
/// ```
/// use teistro::{SuppliedEvent, FixedDay};
/// use teistro::calendar_feed::When;
///
/// let holiday = SuppliedEvent::new("Republic Day", When::Day(FixedDay::new(739_637)));
/// assert_eq!(holiday.uid, None);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuppliedEvent {
    /// The `UID`. None hashes the summary, the time and the place as a
    /// computed event's is hashed, so it stays stable while they do.
    pub uid: Option<String>,
    /// `SUMMARY`.
    pub summary: String,
    /// `DESCRIPTION`.
    pub description: Option<String>,
    /// When it is.
    pub when: When,
    /// Its kind, for a selector, and a category.
    pub kind: Option<String>,
    /// Its tradition, for a selector, and a category.
    pub tradition: Option<String>,
    /// Its own categories, colour and alarm, ahead of any [`FeedHint`].
    pub hints: Hints,
}

impl SuppliedEvent {
    /// An event of this summary and time, and nothing else.
    ///
    /// ```
    /// use teistro::{SuppliedEvent, FixedDay};
    /// use teistro::calendar_feed::When;
    ///
    /// let event = SuppliedEvent::new("Office closed", When::Day(FixedDay::new(739_637)));
    /// assert_eq!(event.summary, "Office closed");
    /// ```
    #[must_use]
    pub fn new(summary: impl Into<String>, when: When) -> SuppliedEvent {
        SuppliedEvent {
            uid: None,
            summary: summary.into(),
            description: None,
            when,
            kind: None,
            tradition: None,
            hints: Hints::default(),
        }
    }
}

impl FeedOptions {
    /// A feed named "Festivals", written by the SDK, refreshed daily and
    /// stamped at `stamp_unix_seconds`.
    #[must_use]
    pub fn new(stamp_unix_seconds: i64) -> FeedOptions {
        FeedOptions {
            name: "Festivals".to_owned(),
            product: format!("-//Teispace//Teistro SDK {}//EN", env!("CARGO_PKG_VERSION")),
            refresh_minutes: Some(1_440),
            stamp_unix_seconds,
            categories: Vec::new(),
            color: None,
            alarm: None,
            only: Vec::new(),
            hints: Vec::new(),
            events: Vec::new(),
        }
    }

    /// With another name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> FeedOptions {
        self.name = name.into();
        self
    }

    /// With another `PRODID`.
    #[must_use]
    pub fn with_product(mut self, product: impl Into<String>) -> FeedOptions {
        self.product = product.into();
        self
    }

    /// With another refresh interval, or none (`None`, or zero).
    #[must_use]
    pub const fn with_refresh_minutes(mut self, minutes: Option<u32>) -> FeedOptions {
        self.refresh_minutes = minutes;
        self
    }
}

/// A feed a binding asks for: the festivals over a range at one place on
/// one clock, and what the feed says of itself.
#[derive(Clone, Debug, PartialEq)]
pub struct FestivalCalendarRequest {
    /// The first day.
    pub from: CalendarDate,
    /// The last day, inclusive.
    pub to: CalendarDate,
    /// The place.
    pub place: Place,
    /// The offset the civil days run at.
    pub offset: UtcOffset,
    /// The festivals.
    pub festivals: FestivalRequest,
    /// What the feed says of itself.
    pub feed: FeedOptions,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "FestivalCalendarRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FestivalCalendarAsked {
    #[serde(default)]
    calendar: Option<Calendar>,
    first: DayAsked,
    #[serde(default)]
    last: Option<DayAsked>,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    utc_offset_seconds: i32,
    /// The festivals, as a festival request.
    #[cfg_attr(feature = "schema", schemars(with = "crate::festival_request::Asked"))]
    festivals: Value,
    stamp_unix_seconds: i64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    product: Option<String>,
    /// A day when left out; zero writes none.
    #[serde(default)]
    refresh_minutes: Option<u32>,
    /// The feed's own categories.
    #[serde(default)]
    categories: Vec<String>,
    /// The feed's colour: a CSS3 colour name, which RFC 7986 requires.
    #[serde(default)]
    color: Option<String>,
    /// The alarm on every event no hint or event of its own gives another.
    #[serde(default)]
    alarm: Option<AlarmAsked>,
    /// Which events the feed keeps: those any selector selects, every
    /// event when left out.
    #[serde(default)]
    only: Vec<FeedSelector>,
    /// Categories, a colour and an alarm by what an event is; the first
    /// that selects an event gives it its own.
    #[serde(default)]
    hints: Vec<FeedHintAsked>,
    /// The caller's own events, merged in.
    #[serde(default)]
    events: Vec<SuppliedEventAsked>,
}

/// A display alarm as a binding writes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "FeedAlarm"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AlarmAsked {
    /// Minutes from the event's start: negative before it, positive after.
    trigger_minutes: i32,
    /// What the alarm says; the event's summary when left out.
    #[serde(default)]
    description: Option<String>,
}

impl From<AlarmAsked> for Alarm {
    fn from(asked: AlarmAsked) -> Alarm {
        Alarm {
            trigger_minutes: asked.trigger_minutes,
            description: asked.description,
        }
    }
}

/// A hint as a binding writes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "FeedHint"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FeedHintAsked {
    /// The events it speaks of; every event when left out.
    #[serde(default)]
    select: FeedSelector,
    #[serde(default)]
    categories: Vec<String>,
    /// A CSS3 colour name.
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    alarm: Option<AlarmAsked>,
}

/// A supplied event as a binding writes it: `day`, or `fromUnixSeconds`
/// and `toUnixSeconds`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SuppliedEvent"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SuppliedEventAsked {
    /// Hashed from the summary, the time and the place when left out.
    #[serde(default)]
    uid: Option<String>,
    summary: String,
    #[serde(default)]
    description: Option<String>,
    /// The request's calendar when left out.
    #[serde(default)]
    calendar: Option<Calendar>,
    /// A whole day.
    #[serde(default)]
    day: Option<DayAsked>,
    /// A span's start, in seconds since 1970.
    #[serde(default)]
    from_unix_seconds: Option<i64>,
    /// A span's end, exclusive.
    #[serde(default)]
    to_unix_seconds: Option<i64>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    tradition: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
    /// A CSS3 colour name.
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    alarm: Option<AlarmAsked>,
}

impl FestivalCalendarRequest {
    /// The record a binding sends, as JSON: `first` (and `last`, the
    /// first when left out) as `{"year", "month", "day"}` in `calendar`
    /// (Gregorian when left out), `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` and `utcOffsetSeconds`; `festivals`, a festival
    /// request; `stampUnixSeconds`; and optionally `name`, `product`,
    /// `refreshMinutes` (a day when left out, none when zero),
    /// `categories`, `color`, `alarm`, `only`, `hints` and `events`
    /// (`03-design/icalendar.md` §6).
    ///
    /// ```
    /// use teistro::FestivalCalendarRequest;
    ///
    /// let asked = FestivalCalendarRequest::from_json(
    ///     r#"{"first": {"year": 2026, "month": 1, "day": 1},
    ///         "last": {"year": 2026, "month": 12, "day": 31},
    ///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "festivals": {"rules": "NEPAL"}, "stampUnixSeconds": 1767225600,
    ///         "name": "Nepal", "refreshMinutes": 0}"#,
    /// )?;
    /// assert_eq!((asked.feed.name.as_str(), asked.feed.refresh_minutes), ("Nepal", Some(0)));
    /// let stampless = FestivalCalendarRequest::from_json(
    ///     r#"{"first": {"year": 2026, "month": 1, "day": 1},
    ///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "festivals": {"rules": "NEPAL"}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(stampless.field(), Some("festivalCalendar"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read or one it needs left out, a place or offset out of range, or a
    /// festival request its own reader refuses, each named under
    /// `festivalCalendar`.
    pub fn from_json(text: &str) -> Result<FestivalCalendarRequest, Error> {
        let asked: FestivalCalendarAsked = teistro_core::strict::read(text, FESTIVAL_CALENDAR)?;
        asked.request().map_err(|why| why.under(FESTIVAL_CALENDAR))
    }
}

impl FestivalCalendarAsked {
    fn request(self) -> Result<FestivalCalendarRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        let festivals = FestivalRequest::from_json(&self.festivals.to_string())?;
        let mut feed = FeedOptions::new(self.stamp_unix_seconds);
        if let Some(minutes) = self.refresh_minutes {
            feed = feed.with_refresh_minutes(Some(minutes));
        }
        if let Some(name) = self.name {
            feed = feed.with_name(name);
        }
        if let Some(product) = self.product {
            feed = feed.with_product(product);
        }
        feed.categories = self.categories;
        feed.color = color_of(self.color.as_deref(), "color")?;
        feed.alarm = self.alarm.map(Alarm::from);
        feed.only = self.only;
        feed.hints = self
            .hints
            .into_iter()
            .enumerate()
            .map(|(at, hint)| {
                Ok(FeedHint {
                    select: hint.select,
                    hints: Hints {
                        categories: hint.categories,
                        color: color_of(hint.color.as_deref(), "color")
                            .map_err(|why| why.under(&format!("hints[{at}]")))?,
                        alarm: hint.alarm.map(Alarm::from),
                    },
                })
            })
            .collect::<Result<_, Error>>()?;
        feed.events = self
            .events
            .into_iter()
            .enumerate()
            .map(|(at, event)| {
                event
                    .supplied(calendar)
                    .map_err(|why| why.under(&format!("events[{at}]")))
            })
            .collect::<Result<_, Error>>()?;
        Ok(FestivalCalendarRequest {
            from: self.first.in_calendar(calendar),
            to: self.last.unwrap_or(self.first).in_calendar(calendar),
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            offset: offset_of(self.utc_offset_seconds)?,
            festivals,
            feed,
        })
    }
}

impl SuppliedEventAsked {
    fn supplied(self, calendar: Calendar) -> Result<SuppliedEvent, Error> {
        let when = match (self.day, self.from_unix_seconds, self.to_unix_seconds) {
            (Some(day), None, None) => {
                let date = day.in_calendar(self.calendar.unwrap_or(calendar));
                When::Day(system_of(date.calendar)?.fixed_of(&date)?)
            }
            (None, Some(from_unix_seconds), Some(to_unix_seconds)) => When::Span {
                from_unix_seconds,
                to_unix_seconds,
            },
            _ => {
                return Err(Error::invalid_arg(
                    "an event is a `day`, or `fromUnixSeconds` with `toUnixSeconds`",
                ));
            }
        };
        Ok(SuppliedEvent {
            uid: self.uid,
            summary: self.summary,
            description: self.description,
            when,
            kind: self.kind,
            tradition: self.tradition,
            hints: Hints {
                categories: self.categories,
                color: color_of(self.color.as_deref(), "color")?,
                alarm: self.alarm.map(Alarm::from),
            },
        })
    }
}

/// A colour a request names, refused by its field.
fn color_of(name: Option<&str>, field: &str) -> Result<Option<Color>, Error> {
    name.map(|name| Color::named(name).map_err(|why| why.with_field(field)))
        .transpose()
}

/// An event on its way into the feed, with what orders and selects it.
struct Draft {
    /// The civil day.
    day: FixedDay,
    /// The start of a span; a whole day's comes first.
    start: Option<i64>,
    /// The rule's key, or a supplied event's `UID`.
    rule: String,
    form: FeedForm,
    classes: Classes,
    event: Event,
}

/// What the catalogue says a rule is. Supplied events give their own; a
/// computed event reads [`classes_of`].
#[derive(Default)]
struct Classes {
    kind: Option<String>,
    tradition: Option<String>,
}

/// A rule's catalogue fields, the one place the feed reads them.
///
/// SR-02 adds them to the rules: this is where they arrive. Until a rule
/// carries them a selector naming them is refused by
/// [`refuse_unclassified`] rather than answered with an empty feed.
fn classes_of(_festivals: &FestivalRequest, _rule: &str) -> Classes {
    Classes::default()
}

impl FeedSelector {
    /// Whether every list given holds this event's value.
    fn selects(&self, draft: &Draft) -> bool {
        let holds = |list: &[String], value: Option<&str>| {
            list.is_empty() || value.is_some_and(|value| list.iter().any(|item| item == value))
        };
        (self.forms.is_empty() || self.forms.contains(&draft.form))
            && holds(&self.kinds, draft.classes.kind.as_deref())
            && holds(&self.traditions, draft.classes.tradition.as_deref())
            && holds(&self.rules, Some(&draft.rule))
    }
}

/// `UNSUPPORTED` when a selector names a field no event of this feed
/// carries: an empty feed would read as "nothing is of that tradition".
fn refuse_unclassified(drafts: &[Draft], options: &FeedOptions) -> Result<(), Error> {
    if drafts.is_empty() {
        return Ok(());
    }
    let selectors = || {
        options
            .only
            .iter()
            .chain(options.hints.iter().map(|hint| &hint.select))
    };
    if selectors().any(|selector| !selector.kinds.is_empty())
        && drafts.iter().all(|draft| draft.classes.kind.is_none())
    {
        return Err(
            Error::unsupported("no event of this feed carries a kind to select by")
                .with_field("kinds"),
        );
    }
    if selectors().any(|selector| !selector.traditions.is_empty())
        && drafts.iter().all(|draft| draft.classes.tradition.is_none())
    {
        return Err(
            Error::unsupported("no event of this feed carries a tradition to select by")
                .with_field("traditions"),
        );
    }
    Ok(())
}

/// A supplied event as a draft: a span on the civil day its start falls
/// on at `offset`.
fn supplied_draft(supplied: &SuppliedEvent, place: &Place, offset: UtcOffset) -> Draft {
    let (day, start, hashed) = match supplied.when {
        When::Day(day) => (day, None, format!("SUPPLIED|{}", supplied.summary)),
        When::Span {
            from_unix_seconds,
            to_unix_seconds,
        } => {
            let local = from_unix_seconds.saturating_add(i64::from(offset.seconds()));
            (
                FixedDay::new(UNIX_EPOCH_DAY + local.div_euclid(86_400)),
                Some(from_unix_seconds),
                format!(
                    "SUPPLIED|{}|{from_unix_seconds}|{to_unix_seconds}",
                    supplied.summary
                ),
            )
        }
    };
    let uid = supplied
        .uid
        .clone()
        .unwrap_or_else(|| uid(&hashed, day, place));
    Draft {
        day,
        start,
        rule: uid.clone(),
        form: FeedForm::Supplied,
        classes: Classes {
            kind: supplied.kind.clone(),
            tradition: supplied.tradition.clone(),
        },
        event: Event {
            uid,
            summary: supplied.summary.clone(),
            description: supplied.description.clone(),
            when: supplied.when,
            hints: supplied.hints.clone(),
        },
    }
}

/// The feed's events: the supplied merged over the computed, those the
/// options keep, each with its hints, in the feed's order.
fn merged(
    mut drafts: Vec<Draft>,
    options: &FeedOptions,
    place: &Place,
    offset: UtcOffset,
) -> Result<Vec<Event>, Error> {
    let supplied: Vec<Draft> = options
        .events
        .iter()
        .map(|event| supplied_draft(event, place, offset))
        .collect();
    drafts.retain(|draft| {
        !supplied
            .iter()
            .any(|event| event.event.uid == draft.event.uid)
    });
    drafts.extend(supplied);
    refuse_unclassified(&drafts, options)?;
    if !options.only.is_empty() {
        drafts.retain(|draft| options.only.iter().any(|selector| selector.selects(draft)));
    }
    for draft in &mut drafts {
        let hint = options.hints.iter().find(|hint| hint.select.selects(draft));
        let hints = &mut draft.event.hints;
        let classes = [draft.classes.kind.take(), draft.classes.tradition.take()];
        let mut categories: Vec<String> = classes.into_iter().flatten().collect();
        categories.append(&mut hints.categories);
        if let Some(hint) = hint {
            categories.extend(hint.hints.categories.iter().cloned());
            hints.color = hints.color.or(hint.hints.color);
            hints.alarm = hints.alarm.take().or_else(|| hint.hints.alarm.clone());
        }
        hints.alarm = hints.alarm.take().or_else(|| options.alarm.clone());
        let mut seen = std::collections::BTreeSet::new();
        categories.retain(|category| seen.insert(category.clone()));
        hints.categories = categories;
    }
    drafts.sort_by(|a, b| {
        a.day
            .cmp(&b.day)
            .then(a.start.cmp(&b.start))
            .then_with(|| a.rule.cmp(&b.rule))
    });
    Ok(drafts.into_iter().map(|draft| draft.event).collect())
}

impl AlmanacArea<'_> {
    /// The observances [`AlmanacArea::festivals`] gives over a range, each
    /// Ekadashi fast, each dated rule's day and each anniversary's, as an
    /// iCalendar feed: one whole-day event each, on its civil day, ordered
    /// by day and then rule. An anniversary's `SUMMARY` is its own key,
    /// which the consumer names it by.
    ///
    /// A fast whose rule keeps a parana adds a timed event, the window
    /// its meal is kept in, in UTC (`sdk.calendar.parana`, "Parana:
    /// Smarta Ekadashi"). Its start is rounded up to the minute and its
    /// end down, since a calendar app shows minutes, so the shown window
    /// lies inside the computed one; a window under a minute writes no
    /// event. A rule without `parana` writes none. A vrata's parana
    /// (Shivaratri's, and the Navaratri's when it is kept inside the 9th
    /// left) is written the same way from its rule's first reading, when
    /// that reading ends the window; an open one writes none.
    ///
    /// - `SUMMARY` is the festival's name in the context's locale, or its
    ///   rule's key when the locale and its fallbacks name it not.
    /// - `DESCRIPTION` gives the rule's source and the answer's
    ///   provenance: the SDK's version, the profile, the ephemeris and the
    ///   festival answer's content hash. A day a published set decides
    ///   (`03-design/published-overrides.md` §3) says "Published by" its
    ///   publisher, set, version and source, and the rule's own day when
    ///   it differs, in place of "Computed by"; a withdrawn one writes no
    ///   event, and an added one is an event like any other, named in the
    ///   context's locale from the set's names (its language's, then the
    ///   first given, then its key).
    /// - `UID` is the first 32 hex digits of SHA-256 over the rule, the
    ///   day's fixed number and the place's three numbers in the canonical
    ///   grammar, then `@teistro`: a feed published again updates an
    ///   event, and an observance a new setup moves to another day is a
    ///   new event there.
    ///
    /// The feed then takes what [`FeedOptions`] asks: the caller's own
    /// events merged in, one whose `UID` is a computed event's standing
    /// in its place; only the events [`FeedOptions::only`] selects, every
    /// one when empty; and each event's categories (its kind and
    /// tradition, its own, then the hint's), colour and alarm, its own
    /// ahead of the first [`FeedHint`] that selects it, ahead of
    /// [`FeedOptions::alarm`].
    ///
    /// The text is sealed with the festival answer's provenance, its
    /// content hash taken over the text.
    ///
    /// ```no_run
    /// use teistro::{CalendarDate, Context, FeedOptions, FestivalPack, FestivalRequest, UtcOffset};
    /// use teistro::catalogue::Calendar;
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    ///
    /// let sdk = Context::builder().profile("nepali-committee").build()?;
    /// let kathmandu = Place::new(Latitude::try_new(27.7172)?, Longitude::try_new(85.324)?, Altitude::try_new(1400.0)?);
    /// let feed = sdk.almanac().festival_calendar(
    ///     &CalendarDate::defined(Calendar::Gregorian, 2026, 1, 1),
    ///     &CalendarDate::defined(Calendar::Gregorian, 2026, 12, 31),
    ///     &kathmandu,
    ///     UtcOffset::literal(5, 45, 0),
    ///     &FestivalRequest::from(FestivalPack::NepalPanchanga),
    ///     &FeedOptions::new(1_767_225_600).with_name("Nepal 2026"),
    /// )?;
    /// std::fs::write("nepal-2026.ics", feed.value)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::festivals`]; `OUT_OF_RANGE` for a day outside the
    /// years 1 to 9999; `INVALID_ARG` for a `UID` two supplied events
    /// share or an empty category; `UNSUPPORTED`, on `kinds` or
    /// `traditions`, when a selector names a field no event of the feed
    /// carries.
    pub fn festival_calendar(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        festivals: &FestivalRequest,
        options: &FeedOptions,
    ) -> Result<Envelope<String>, Error> {
        let Envelope { value, provenance } = self.festivals(from, to, place, offset, festivals)?;
        let answered = format!(
            "the Teistro SDK {} under the {} profile with the {} ephemeris; answer {}.",
            provenance.sdk_version,
            provenance.profile,
            provenance.provider.name,
            provenance.content_hash,
        );
        let about = format!("Computed by {answered}");
        let occurrences = kept_days(&value);
        let describe = |rule: &str| match source_of(festivals, rule) {
            Some(source) => format!("{source}\n{about}"),
            None => about.clone(),
        };
        let mut drafts: Vec<Draft> = Vec::new();
        for (rule, day, published, form) in occurrences {
            let fixed = system_of(day.calendar)?.fixed_of(day)?;
            let event = Event {
                uid: uid(rule, fixed, place),
                summary: self.festival_name(rule),
                description: Some(match published {
                    Some(published) => published_by(published, &answered)?,
                    None => describe(rule),
                }),
                when: When::Day(fixed),
                hints: Hints::default(),
            };
            drafts.push(Draft {
                day: fixed,
                start: None,
                rule: rule.clone(),
                form,
                classes: classes_of(festivals, rule),
                event,
            });
        }
        for added in &value.added {
            let fixed = system_of(added.day.calendar)?.fixed_of(&added.day)?;
            let event = Event {
                uid: uid(&added.key, fixed, place),
                summary: self.added_name(&added.key, &added.names),
                description: Some(published_by(&added.published, &answered)?),
                when: When::Day(fixed),
                hints: Hints::default(),
            };
            drafts.push(Draft {
                day: fixed,
                start: None,
                rule: added.key.clone(),
                form: FeedForm::Added,
                classes: Classes::default(),
                event,
            });
        }
        drafts.extend(self.parana_drafts(&value, festivals, place, &describe)?);
        let feed = Feed {
            product: options.product.clone(),
            name: options.name.clone(),
            refresh_minutes: options.refresh_minutes,
            stamp_unix_seconds: options.stamp_unix_seconds,
            categories: options.categories.clone(),
            color: options.color,
            events: merged(drafts, options, place, offset)?,
        };
        Ok(Envelope::sealing(ical::write(&feed)?, provenance))
    }

    /// The timed events: an Ekadashi's kept window, and a vrata's first
    /// reading when it has an end.
    fn parana_drafts(
        self,
        value: &teistro_panchanga::festival::Observances,
        festivals: &FestivalRequest,
        place: &Place,
        describe: &dyn Fn(&str) -> String,
    ) -> Result<Vec<Draft>, Error> {
        let mut drafts = Vec::new();
        // An Ekadashi's kept window, and a vrata's first reading when it
        // has an end: an open reading ("from the 8th's end") has no window
        // to write.
        let fasts = value.ekadashis.iter().filter_map(|fast| {
            if withdrawn(fast.published.as_ref()) {
                return None;
            }
            let parana = fast.parana.as_ref()?;
            let kept = parana.kept?;
            Some((&fast.rule, &parana.day, kept.span.from, kept.span.to))
        });
        let vratas = value.observances.iter().filter_map(|observance| {
            if withdrawn(observance.published.as_ref()) {
                return None;
            }
            let first = observance.parana.first()?;
            Some((&observance.rule, &first.day, first.from, first.to?))
        });
        for (rule, day, from, to) in fasts.chain(vratas) {
            let (from, to) = (minute_after(from), minute_before(to));
            if to <= from {
                continue;
            }
            let fixed = system_of(day.calendar)?.fixed_of(day)?;
            let summary = self
                .context
                .locale_engine()
                .render_typed(&Parana {
                    fast: self.festival_name(rule),
                })
                .text;
            let event = Event {
                uid: uid(&format!("{rule}|PARANA"), fixed, place),
                summary,
                description: Some(describe(rule)),
                when: When::Span {
                    from_unix_seconds: from,
                    to_unix_seconds: to,
                },
                hints: Hints::default(),
            };
            drafts.push(Draft {
                day: fixed,
                start: Some(from),
                rule: rule.clone(),
                form: FeedForm::Parana,
                classes: classes_of(festivals, rule),
                event,
            });
        }
        Ok(drafts)
    }

    /// As [`AlmanacArea::festival_calendar`], for a request a binding
    /// sent.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::festival_calendar`].
    pub fn festival_calendar_request(
        self,
        asked: &FestivalCalendarRequest,
    ) -> Result<Envelope<String>, Error> {
        self.festival_calendar(
            &asked.from,
            &asked.to,
            &asked.place,
            asked.offset,
            &asked.festivals,
            &asked.feed,
        )
    }

    /// An added day's name in the context's locale: the locale's own,
    /// then its language's, then the first the set gives, then its key.
    pub(super) fn added_name(
        self,
        key: &str,
        names: &std::collections::BTreeMap<String, String>,
    ) -> String {
        let engine = self.context.locale_engine();
        let locale = engine.locale();
        let language = locale.split('-').next().unwrap_or(locale);
        names
            .get(locale)
            .or_else(|| {
                names
                    .iter()
                    .find(|(tag, _)| tag.split('-').next() == Some(language))
                    .map(|(_, name)| name)
            })
            .or_else(|| names.values().next())
            .map_or_else(|| key.to_owned(), String::clone)
    }

    /// A rule's name in the context's locale, or its key.
    pub(super) fn festival_name(self, rule: &str) -> String {
        let engine = self.context.locale_engine();
        engine
            .entity_from(engine.locale(), &format!("festival.{rule}"))
            .and_then(|entity| entity.form("name"))
            .map_or_else(|| rule.to_owned(), str::to_owned)
    }

    /// A catalogue member's name in the context's locale, or its full key.
    pub(super) fn entity_name(self, key: &str) -> String {
        let engine = self.context.locale_engine();
        engine
            .entity_from(engine.locale(), key)
            .and_then(|entity| entity.form("name"))
            .map_or_else(|| key.to_owned(), str::to_owned)
    }
}

/// Every whole day the answer keeps, with what published it and its
/// form: the observances, the Ekadashi fasts, the dated rules' days and
/// the anniversaries. A withdrawn day is not kept, so a subscriber's
/// calendar drops it.
pub(super) fn kept_days(
    value: &teistro_panchanga::festival::Observances,
) -> impl Iterator<Item = (&String, &CalendarDate, &Option<Published>, FeedForm)> {
    value
        .observances
        .iter()
        .map(|observance| {
            (
                &observance.rule,
                &observance.day,
                &observance.published,
                FeedForm::Observance,
            )
        })
        .chain(
            value
                .ekadashis
                .iter()
                .map(|fast| (&fast.rule, &fast.day, &fast.published, FeedForm::Ekadashi)),
        )
        .chain(
            value
                .dated
                .iter()
                .map(|kept| (&kept.rule, &kept.day, &kept.published, FeedForm::Dated)),
        )
        .chain(value.anniversaries.iter().map(|kept| {
            (
                &kept.key,
                &kept.observance.day,
                &kept.observance.published,
                FeedForm::Anniversary,
            )
        }))
        .filter(|(_, _, published, _)| !withdrawn(published.as_ref()))
}

/// Whether a published set withdrew the day, which the feed then leaves
/// out, its parana with it.
fn withdrawn(published: Option<&Published>) -> bool {
    published.is_some_and(|p| p.status == PublishedStatus::Withdrawn)
}

/// A published day's `DESCRIPTION`: who published it, in which set and
/// version, its source, the rule's own day when it differs, and the
/// answer's provenance after "Answered by".
fn published_by(published: &Published, answered: &str) -> Result<String, Error> {
    use std::fmt::Write as _;
    let mut said = format!(
        "Published by {} in {} version {}: {}.",
        published.publisher, published.set, published.version, published.source
    );
    if let Some(computed) = &published.computed {
        let fixed = system_of(computed.calendar)?.fixed_of(computed)?;
        let rules = system_of(Calendar::Gregorian)?.date_of(fixed)?;
        let _ = write!(
            said,
            " The rule's own day is {:04}-{:02}-{:02}.",
            rules.year, rules.month, rules.day
        );
    }
    Ok(format!("{said}\nAnswered by {answered}"))
}

/// The source a request's rule of this key cites, of whichever kind.
fn source_of<'a>(festivals: &'a FestivalRequest, rule: &str) -> Option<&'a str> {
    let yearly = festivals.rules().iter().map(|r| (&r.key, &r.source));
    let ekadashis = festivals.ekadashis().iter().map(|r| (&r.key, &r.source));
    let following = festivals.following().iter().map(|r| (&r.key, &r.source));
    let dated = festivals.dated().iter().map(|r| (&r.key, &r.source));
    yearly
        .chain(ekadashis)
        .chain(following)
        .chain(dated)
        .find(|(key, _)| key.as_str() == rule)
        .map(|(_, source)| source.as_str())
}

/// Seconds since the Unix epoch at the first whole minute at or after
/// `at`.
fn minute_after(at: JulianDay<Utc>) -> i64 {
    seconds_of(unix_minutes(at).ceil())
}

/// Seconds since the Unix epoch at the last whole minute at or before
/// `at`.
fn minute_before(at: JulianDay<Utc>) -> i64 {
    seconds_of(unix_minutes(at).floor())
}

/// A whole number of minutes as seconds.
fn seconds_of(minutes: f64) -> i64 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a whole number of minutes in the years 1 to 9999 is far inside the range"
    )]
    let minutes = minutes as i64;
    minutes * 60
}

/// Minutes since the Unix epoch, a millisecond's rounding below a whole
/// minute counted as on it: a Julian day near 2.46e6 carries about 40
/// microseconds, so a window computed on the minute is not moved a
/// minute by its last bit.
fn unix_minutes(at: JulianDay<Utc>) -> f64 {
    let minutes = (at.get() - JulianDay::<Utc>::UNIX_EPOCH.get()) * 1_440.0;
    let nearest = minutes.round();
    if (minutes - nearest).abs() < 1.0 / 60_000.0 {
        nearest
    } else {
        minutes
    }
}

/// An occurrence's `UID`: the rule, the day and the place, hashed.
fn uid(rule: &str, day: FixedDay, place: &Place) -> String {
    let at = format!(
        "{rule}|{}|{}|{}|{}",
        day.get(),
        canonical_json(&place.latitude.get()),
        canonical_json(&place.longitude.get()),
        canonical_json(&place.altitude.get()),
    );
    let mut hex = Hash::of(at.as_bytes()).to_string();
    hex.truncate(32);
    format!("{hex}@teistro")
}

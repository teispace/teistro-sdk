//! A year of observances as an iCalendar feed a calendar app subscribes
//! to (`03-design/icalendar.md`, P2-2).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::ical::{self, Event, Feed, When};
use teistro_calendar::{CalendarDate, FixedDay};
use teistro_core::catalogue::Calendar;
use teistro_core::envelope::{Envelope, Hash, canonical_json};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_intl::messages::sdk::calendar::Parana;

use super::AlmanacArea;
use crate::area::system_of;
use crate::asked::{DayAsked, offset_of, place_of};
use crate::festival_request::FestivalRequest;

/// The field a feed request's refusals are named under.
const FESTIVAL_CALENDAR: &str = "festivalCalendar";

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
}

impl FestivalCalendarRequest {
    /// The record a binding sends, as JSON: `first` (and `last`, the
    /// first when left out) as `{"year", "month", "day"}` in `calendar`
    /// (Gregorian when left out), `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` and `utcOffsetSeconds`; `festivals`, a festival
    /// request; `stampUnixSeconds`; and optionally `name`, `product` and
    /// `refreshMinutes` (a day when left out, none when zero).
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
    /// (Shivaratri's) is written the same way from its rule's first
    /// reading, when that reading ends the window; an open one writes
    /// none.
    ///
    /// - `SUMMARY` is the festival's name in the context's locale, or its
    ///   rule's key when the locale and its fallbacks name it not.
    /// - `DESCRIPTION` gives the rule's source and the answer's
    ///   provenance: the SDK's version, the profile, the ephemeris and the
    ///   festival answer's content hash.
    /// - `UID` is the first 32 hex digits of SHA-256 over the rule, the
    ///   day's fixed number and the place's three numbers in the canonical
    ///   grammar, then `@teistro`: a feed published again updates an
    ///   event, and an observance a new setup moves to another day is a
    ///   new event there.
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
    /// years 1 to 9999.
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
        let about = format!(
            "Computed by the Teistro SDK {} under the {} profile with the {} ephemeris; answer {}.",
            provenance.sdk_version,
            provenance.profile,
            provenance.provider.name,
            provenance.content_hash,
        );
        let occurrences = value
            .observances
            .iter()
            .map(|observance| (&observance.rule, &observance.day))
            .chain(value.ekadashis.iter().map(|fast| (&fast.rule, &fast.day)))
            .chain(value.dated.iter().map(|kept| (&kept.rule, &kept.day)))
            .chain(
                value
                    .anniversaries
                    .iter()
                    .map(|kept| (&kept.key, &kept.observance.day)),
            );
        let describe = |rule: &str| match source_of(festivals, rule) {
            Some(source) => format!("{source}\n{about}"),
            None => about.clone(),
        };
        // Each event, keyed by its day, its start (a whole day's first)
        // and its rule, the order the feed lists them in.
        let mut events: Vec<(FixedDay, Option<i64>, &str, Event)> = Vec::new();
        for (rule, day) in occurrences {
            let fixed = system_of(day.calendar)?.fixed_of(day)?;
            let event = Event {
                uid: uid(rule, fixed, place),
                summary: self.festival_name(rule),
                description: Some(describe(rule)),
                when: When::Day(fixed),
            };
            events.push((fixed, None, rule, event));
        }
        // An Ekadashi's kept window, and a vrata's first reading when it
        // has an end: an open reading ("from the 8th's end") has no window
        // to write.
        let fasts = value.ekadashis.iter().filter_map(|fast| {
            let parana = fast.parana.as_ref()?;
            let kept = parana.kept?;
            Some((&fast.rule, &parana.day, kept.span.from, kept.span.to))
        });
        let vratas = value.observances.iter().filter_map(|observance| {
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
            };
            events.push((fixed, Some(from), rule, event));
        }
        events.sort_by(|(a, s, x, _), (b, t, y, _)| a.cmp(b).then(s.cmp(t)).then_with(|| x.cmp(y)));
        let feed = Feed {
            product: options.product.clone(),
            name: options.name.clone(),
            refresh_minutes: options.refresh_minutes,
            stamp_unix_seconds: options.stamp_unix_seconds,
            events: events.into_iter().map(|(_, _, _, event)| event).collect(),
        };
        Ok(Envelope::sealing(ical::write(&feed)?, provenance))
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

    /// A rule's name in the context's locale, or its key.
    fn festival_name(self, rule: &str) -> String {
        let engine = self.context.locale_engine();
        engine
            .entity_from(engine.locale(), &format!("festival.{rule}"))
            .and_then(|entity| entity.form("name"))
            .map_or_else(|| rule.to_owned(), str::to_owned)
    }
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

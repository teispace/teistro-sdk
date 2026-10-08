//! A rashifal through the façade: one period of civil days at a place,
//! read for each of the twelve signs (`03-design/rashifal.md`).

use serde::{Deserialize, Serialize};
use teistro_calendar::{CalendarDate, FixedDay};
use teistro_core::catalogue::{Calendar, Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_gochar::Transit;
use teistro_rashifal::RashiReading;
use teistro_rashifal::baseline::{BaselineScore, Panchanga, Period};

/// The instant a period's sky is read at (C358).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "at", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Snapshot {
    /// Sunrise at the place on the reference day, as the context reckons
    /// it.
    #[default]
    Sunrise,
    /// A clock time on the reference day at the request's offset: the
    /// baseline engine's is 06:00.
    Clock {
        /// The hour, 0 to 23.
        hour: u8,
        /// The minute, 0 to 59.
        minute: u8,
    },
}

/// Every graha a period reports the ingresses and stations of unless asked
/// otherwise: all but the Moon, whose ingresses come every two and a half
/// days (C360).
pub const EVENT_GRAHAS: [Graha; 8] = [
    Graha::Sun,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// A period of civil days at a place, to read for each of the twelve signs.
///
/// ```
/// use teistro::CalendarDate;
/// use teistro::catalogue::Calendar;
/// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
/// use teistro::{RashifalRequest, Snapshot, UtcOffset};
///
/// let kathmandu = Place::new(
///     Latitude::literal(27.7172),
///     Longitude::literal(85.324),
///     Altitude::literal(1400.0),
/// );
/// let week = RashifalRequest::between(
///     CalendarDate::defined(Calendar::Gregorian, 2026, 10, 4),
///     CalendarDate::defined(Calendar::Gregorian, 2026, 10, 10),
///     kathmandu,
///     UtcOffset::literal(5, 45, 0),
/// );
/// assert_eq!(week.snapshot(), Snapshot::Sunrise);
/// let as_the_baseline = week.at(Snapshot::Clock { hour: 6, minute: 0 });
/// assert_eq!(as_the_baseline.snapshot(), Snapshot::Clock { hour: 6, minute: 0 });
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct RashifalRequest {
    first: CalendarDate,
    last: CalendarDate,
    place: Place,
    offset: UtcOffset,
    snapshot: Snapshot,
    events: Vec<Graha>,
    spells: Vec<u8>,
}

impl RashifalRequest {
    /// The days from `first` to `last`, inclusive, at `place`, whose civil
    /// days run at `offset`: read at sunrise on the middle day, with every
    /// graha's events but the Moon's and C149's spells.
    #[must_use]
    pub fn between(
        first: CalendarDate,
        last: CalendarDate,
        place: Place,
        offset: UtcOffset,
    ) -> RashifalRequest {
        RashifalRequest {
            first,
            last,
            place,
            offset,
            snapshot: Snapshot::Sunrise,
            events: EVENT_GRAHAS.to_vec(),
            spells: teistro_gochar::sade_sati::DEFAULT_SPELLS.to_vec(),
        }
    }

    /// One day.
    #[must_use]
    pub fn day(date: CalendarDate, place: Place, offset: UtcOffset) -> RashifalRequest {
        RashifalRequest::between(date.clone(), date, place, offset)
    }

    /// Read at another instant than sunrise.
    #[must_use]
    pub const fn at(mut self, snapshot: Snapshot) -> RashifalRequest {
        self.snapshot = snapshot;
        self
    }

    /// The grahas whose ingresses and stations the period reports.
    #[must_use]
    pub fn with_events(mut self, grahas: impl IntoIterator<Item = Graha>) -> RashifalRequest {
        self.events = grahas.into_iter().collect();
        self
    }

    /// The houses from a sign counted as Saturn's smaller spells (C149).
    #[must_use]
    pub fn with_spells(mut self, houses: impl IntoIterator<Item = u8>) -> RashifalRequest {
        self.spells = houses.into_iter().collect();
        self
    }

    /// The first day.
    #[must_use]
    pub const fn first(&self) -> &CalendarDate {
        &self.first
    }

    /// The last day.
    #[must_use]
    pub const fn last(&self) -> &CalendarDate {
        &self.last
    }

    /// The place.
    #[must_use]
    pub const fn place(&self) -> &Place {
        &self.place
    }

    /// The offset the civil days run at.
    #[must_use]
    pub const fn offset(&self) -> UtcOffset {
        self.offset
    }

    /// The instant the sky is read at.
    #[must_use]
    pub const fn snapshot(&self) -> Snapshot {
        self.snapshot
    }

    /// The grahas whose events are reported.
    #[must_use]
    pub fn events(&self) -> &[Graha] {
        &self.events
    }

    /// The houses counted as Saturn's smaller spells.
    #[must_use]
    pub fn spells(&self) -> &[u8] {
        &self.spells
    }

    /// Refuses what no period can be: a day its calendar does not have, a
    /// last day before the first, a clock time off the clock, a spell
    /// outside 1 to 12 or one of Sade Sati's own, or a graha asked twice.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the field.
    pub fn check(&self) -> Result<(), Error> {
        if let Snapshot::Clock { hour, minute } = self.snapshot
            && (hour > 23 || minute > 59)
        {
            return Err(
                Error::invalid_arg(format!("{hour:02}:{minute:02} is not a time of day"))
                    .with_field("snapshot"),
            );
        }
        self.days()?;
        teistro_gochar::sade_sati::check_houses(&self.spells)
            .map_err(|refused| refused.with_field("spells"))?;
        for (at, graha) in self.events.iter().enumerate() {
            if self.events.iter().skip(at + 1).any(|other| other == graha) {
                return Err(
                    Error::invalid_arg(format!("{} is asked for twice", graha.key()))
                        .with_field("events"),
                );
            }
        }
        Ok(())
    }
}

impl RashifalRequest {
    /// The first and last days as fixed days, and how many days the period
    /// holds, at least one.
    pub(crate) fn days(&self) -> Result<(FixedDay, FixedDay, u32), Error> {
        let calendar = crate::area::system_of(self.first.calendar)?;
        let first = calendar
            .fixed_of(&self.first)
            .map_err(|why| why.with_field("first"))?;
        let last = calendar
            .fixed_of(&self.last)
            .map_err(|why| why.with_field("last"))?;
        let days = u32::try_from(first.days_until(last) + 1)
            .ok()
            .filter(|days| *days > 0)
            .ok_or_else(|| {
                Error::invalid_arg("a period's last day comes before its first").with_field("last")
            })?;
        Ok((first, last, days))
    }
}

/// One period read for each of the twelve signs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RashifalPeriod {
    /// The period's first day.
    pub first: CalendarDate,
    /// Its last day.
    pub last: CalendarDate,
    /// The day it is read at, the middle one (C359).
    pub reference: CalendarDate,
    /// The instant it is read at (C358).
    pub instant: JulianDay<Utc>,
    /// Each graha's place then, the Sun to Ketu.
    pub transits: [Transit; 9],
    /// Whether each was moving backwards then, the Sun to Ketu.
    pub retrograde: [bool; 9],
    /// What the baseline's score reads of the reference day's panchanga.
    pub panchanga: Panchanga,
    /// Each sign's reading, Aries to Pisces.
    pub readings: [RashiReading; 12],
}

impl RashifalPeriod {
    /// One sign's reading.
    #[must_use]
    pub fn of(&self, rashi: Rashi) -> &RashiReading {
        let [aries, ..] = &self.readings;
        self.readings
            .iter()
            .find(|one| one.rashi == rashi)
            .unwrap_or(aries)
    }

    /// The baseline engine's score of one sign's reading for `period`,
    /// `BASELINE` and unsourced (C361).
    #[must_use]
    pub fn baseline_score(&self, rashi: Rashi, period: Period) -> BaselineScore {
        teistro_rashifal::baseline::baseline_score(
            self.of(rashi),
            &self.retrograde,
            self.panchanga,
            period,
        )
    }
}

/// The record a binding writes a rashifal as, which a refusal is named
/// under.
const RASHIFAL: &str = "rashifal";

/// A civil day as a request names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DayAsked {
    year: i32,
    month: u8,
    day: u8,
}

/// One period as a binding writes it, camel-cased as every request record
/// is: the days, the place and the offset, and optionally the snapshot,
/// the grahas whose events are reported and Saturn's spells.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PeriodAsked {
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
    #[serde(default)]
    snapshot: Option<Snapshot>,
    #[serde(default)]
    events: Option<Vec<Graha>>,
    #[serde(default)]
    spells: Option<Vec<u8>>,
}

/// Many periods as a binding writes them, and the period the baseline
/// engine's score is read for, when it is asked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchAsked {
    periods: Vec<PeriodAsked>,
    #[serde(default)]
    baseline: Option<Period>,
}

/// Periods to read, as a binding asks them: each a [`RashifalRequest`],
/// and the baseline engine's score for each sign when `baseline` names
/// the period it is read for.
#[derive(Clone, Debug, PartialEq)]
pub struct RashifalBatch {
    /// The periods, in the order they are answered.
    pub periods: Vec<RashifalRequest>,
    /// The period the baseline's score is read for, or none.
    pub baseline: Option<Period>,
}

impl RashifalBatch {
    /// The record a binding sends, as JSON: `periods`, each with `first`
    /// (and `last`, the first when left out) as `{"year", "month",
    /// "day"}` in `calendar` (Gregorian when left out), `latitudeDeg`,
    /// `longitudeDeg`, `altitudeM` and `utcOffsetSeconds`, and optionally
    /// `snapshot`, `events` and `spells`; and `baseline`, a
    /// [`Period`](teistro_rashifal::baseline::Period), when the baseline
    /// engine's score is wanted.
    ///
    /// ```
    /// use teistro::{RashifalBatch, Snapshot};
    ///
    /// let asked = RashifalBatch::from_json(
    ///     r#"{"periods": [{"first": {"year": 2026, "month": 10, "day": 4},
    ///         "last": {"year": 2026, "month": 10, "day": 10},
    ///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
    ///         "utcOffsetSeconds": 20700, "snapshot": {"at": "CLOCK", "hour": 6, "minute": 0},
    ///         "events": ["SATURN", "graha.JUPITER"]}], "baseline": "WEEKLY"}"#,
    /// )?;
    /// assert_eq!(asked.periods[0].snapshot(), Snapshot::Clock { hour: 6, minute: 0 });
    /// assert_eq!(asked.periods[0].events().len(), 2);
    /// let typo = RashifalBatch::from_json(
    ///     r#"{"periods": [{"first": {"year": 2026, "month": 10, "day": 4},
    ///         "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20700, "spell": [4]}]}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(typo.field(), Some("rashifal.periods[0].spell"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, no period at all, a place or offset out of range, and
    /// whatever [`RashifalRequest::check`] refuses, each named under
    /// `rashifal`.
    pub fn from_json(text: &str) -> Result<RashifalBatch, Error> {
        let asked: BatchAsked = teistro_core::strict::read(text, RASHIFAL)?;
        if asked.periods.is_empty() {
            return Err(Error::invalid_arg("no period to read").with_field("rashifal.periods"));
        }
        let periods = asked
            .periods
            .into_iter()
            .enumerate()
            .map(|(at, one)| {
                let under = format!("{RASHIFAL}.periods[{at}]");
                one.request()
                    .and_then(|request| request.check().map(|()| request))
                    .map_err(|why| why.under(&under))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(RashifalBatch {
            periods,
            baseline: asked.baseline,
        })
    }
}

impl PeriodAsked {
    fn request(self) -> Result<RashifalRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        let date = |day: DayAsked| CalendarDate::defined(calendar, day.year, day.month, day.day);
        let place = Place::new(
            Latitude::try_new(self.latitude_deg)
                .map_err(|why| Error::from(why).with_field("latitudeDeg"))?,
            Longitude::try_new(self.longitude_deg)
                .map_err(|why| Error::from(why).with_field("longitudeDeg"))?,
            Altitude::try_new(self.altitude_m)
                .map_err(|why| Error::from(why).with_field("altitudeM"))?,
        );
        let offset = UtcOffset::try_from_seconds(self.utc_offset_seconds)
            .map_err(|why| Error::from(why).with_field("utcOffsetSeconds"))?;
        let mut request = RashifalRequest::between(
            date(self.first),
            date(self.last.unwrap_or(self.first)),
            place,
            offset,
        );
        if let Some(snapshot) = self.snapshot {
            request = request.at(snapshot);
        }
        if let Some(events) = self.events {
            request = request.with_events(events);
        }
        if let Some(spells) = self.spells {
            request = request.with_spells(spells);
        }
        Ok(request)
    }
}

/// One period's answer as a binding reads it: the period, and the
/// baseline engine's score of each sign, Aries to Pisces, when it was
/// asked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RashifalAnswer {
    /// The period read.
    pub period: RashifalPeriod,
    /// The baseline engine's score of each sign, `BASELINE` (C361).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<Vec<BaselineScore>>,
}

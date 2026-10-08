//! A rashifal through the façade: one period of civil days at a place,
//! read for each of the twelve signs (`03-design/rashifal.md`).

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_gochar::Transit;
use teistro_rashifal::RashiReading;
use teistro_rashifal::baseline::Panchanga;

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

    /// Refuses what no period can be: a clock time off the clock, a spell
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
    pub fn baseline_score(
        &self,
        rashi: Rashi,
        period: teistro_rashifal::baseline::Period,
    ) -> teistro_rashifal::baseline::BaselineScore {
        teistro_rashifal::baseline::baseline_score(
            self.of(rashi),
            &self.retrograde,
            self.panchanga,
            period,
        )
    }
}

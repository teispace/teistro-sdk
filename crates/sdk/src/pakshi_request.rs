//! Pancha Pakshi through the façade (`03-design/pakshi.md`): days of a
//! native's bird at a place, read over the almanac's sunrise, sunset and
//! tithi, and a native's bird from a chart's Moon.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Calendar, Graha, Nakshatra, Paksha};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;
use teistro_pakshi::{Bird, BirthBird, Day, Reading, Rules, birth_bird, read_day};
use teistro_panchanga::almanac::Panchanga;
use teistro_serial::Document;
use teistro_time::DayState;

use crate::asked::{DayAsked, offset_of, place_of};
use crate::{AlmanacArea, ChartArea};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const PAKSHI: &str = "pakshi";

/// Whose bird a reading follows: a bird named outright, or the one a
/// birth star and paksha give under a rule (`03-design/pakshi.md` §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PakshiNative {
    /// The bird itself.
    Bird(Bird),
    /// The bird of a birth star in a paksha, under a rule (crux P1).
    Star {
        /// The Moon's nakshatra at birth.
        nakshatra: Nakshatra,
        /// The paksha at birth.
        paksha: Paksha,
        /// How the dark half assigns the birds.
        rule: BirthBird,
    },
}

impl PakshiNative {
    /// The native's bird.
    #[must_use]
    pub fn bird(self) -> Bird {
        match self {
            PakshiNative::Bird(bird) => bird,
            PakshiNative::Star {
                nakshatra,
                paksha,
                rule,
            } => birth_bird(nakshatra, paksha, rule),
        }
    }
}

impl From<Bird> for PakshiNative {
    fn from(bird: Bird) -> PakshiNative {
        PakshiNative::Bird(bird)
    }
}

/// One civil day's reading: `None` on a day the Sun does not both rise
/// and set at the place, which has no yamas.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PakshiDay {
    /// The civil day.
    pub date: CalendarDate,
    /// Its reading, when the day has yamas.
    pub reading: Option<Reading>,
}

/// Days of a native's bird at a place, as a binding asks them.
#[derive(Clone, Debug, PartialEq)]
pub struct PakshiRequest {
    /// The first day.
    pub first: CalendarDate,
    /// The last day, inclusive.
    pub last: CalendarDate,
    /// The place.
    pub place: Place,
    /// The offset its civil days run at.
    pub offset: UtcOffset,
    /// Whose bird.
    pub native: PakshiNative,
    /// What the days are read under.
    pub rules: Rules,
}

/// Whose bird, as a binding writes it: `bird`, or `nakshatra` and `paksha`
/// with an optional `rule`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NativeAsked {
    bird: Option<Bird>,
    nakshatra: Option<Nakshatra>,
    paksha: Option<Paksha>,
    rule: Option<BirthBird>,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RequestAsked {
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
    native: NativeAsked,
    #[serde(default)]
    rules: Rules,
}

impl NativeAsked {
    fn native(self) -> Result<PakshiNative, Error> {
        match self {
            NativeAsked {
                bird: Some(bird),
                nakshatra: None,
                paksha: None,
                rule: None,
            } => Ok(PakshiNative::Bird(bird)),
            NativeAsked {
                bird: None,
                nakshatra: Some(nakshatra),
                paksha: Some(paksha),
                rule,
            } => Ok(PakshiNative::Star {
                nakshatra,
                paksha,
                rule: rule.unwrap_or_default(),
            }),
            NativeAsked { bird: Some(_), .. } => Err(Error::invalid_arg(
                "a native is a bird or a birth star and paksha, not both",
            )
            .with_field("bird")),
            NativeAsked {
                nakshatra: None, ..
            } => Err(
                Error::invalid_arg("a native names a bird, or a birth star and paksha")
                    .with_field("nakshatra"),
            ),
            NativeAsked { .. } => Err(Error::invalid_arg(
                "a birth star is read with the paksha of the birth",
            )
            .with_field("paksha")),
        }
    }
}

impl PakshiRequest {
    /// The record a binding sends, as JSON: `first` (and `last`, the first
    /// when left out) as `{"year", "month", "day"}` in `calendar`
    /// (Gregorian when left out), `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` and `utcOffsetSeconds`; `native`, either `{"bird"}` or
    /// `{"nakshatra", "paksha", "rule"}` with `rule` optional; and `rules`,
    /// optional.
    ///
    /// ```
    /// use teistro::PakshiRequest;
    /// use teistro::pakshi::Bird;
    ///
    /// let asked = PakshiRequest::from_json(
    ///     r#"{"first": {"year": 1984, "month": 10, "day": 31},
    ///         "latitudeDeg": 13.08, "longitudeDeg": 80.27, "utcOffsetSeconds": 19800,
    ///         "native": {"nakshatra": "nakshatra.UTTARA_ASHADHA", "paksha": "SHUKLA"}}"#,
    /// )?;
    /// assert_eq!(asked.native.bird(), Bird::Cock);
    /// let both = PakshiRequest::from_json(
    ///     r#"{"first": {"year": 1984, "month": 10, "day": 31},
    ///         "latitudeDeg": 13.08, "longitudeDeg": 80.27, "utcOffsetSeconds": 19800,
    ///         "native": {"bird": "OWL", "nakshatra": "BHARANI", "paksha": "SHUKLA"}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(both.field(), Some("pakshi.native.bird"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place or offset out of range, or a native that is neither a
    /// bird nor a birth star and paksha, each named under `pakshi`.
    pub fn from_json(text: &str) -> Result<PakshiRequest, Error> {
        let asked: RequestAsked = teistro_core::strict::read(text, PAKSHI)?;
        asked.request().map_err(|why| why.under(PAKSHI))
    }
}

impl RequestAsked {
    fn request(self) -> Result<PakshiRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        Ok(PakshiRequest {
            first: self.first.in_calendar(calendar),
            last: self.last.unwrap_or(self.first).in_calendar(calendar),
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            offset: offset_of(self.utc_offset_seconds)?,
            native: self.native.native().map_err(|why| why.under("native"))?,
            rules: self.rules,
        })
    }
}

/// A day's reading over the almanac's own day, or `None` when the Sun
/// does not both rise and set.
fn reading_of(day: &Panchanga, bird: Bird, rules: Rules) -> Result<Option<Reading>, Error> {
    let local = &day.day;
    if local.state != DayState::Normal {
        return Ok(None);
    }
    let paksha = day
        .tithi_at(local.sunrise)
        .map(|span| span.member.attributes().paksha)
        .ok_or_else(|| Error::internal("no tithi runs at the day's sunrise"))?;
    let read = Day::new(
        local.sunrise.get(),
        local.sunset.get(),
        local.next_sunrise.get(),
        local.vara,
        paksha,
    )?;
    Ok(Some(read_day(&read, bird, &rules)))
}

impl AlmanacArea<'_> {
    /// A native's bird read over each day from `first` to `last` at a
    /// place (`03-design/pakshi.md`): the ten yamas from the almanac's
    /// sunrise, sunset and next sunrise, each with the bird's activity and
    /// its timed sub-periods, the day's weekday that of its sunrise and
    /// its paksha the one running at its sunrise (cruxes P2, P9), and the
    /// day's death bird beside them. A day the Sun does not both rise and
    /// set has no reading.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::of`].
    pub fn pakshi_days(
        self,
        first: &CalendarDate,
        last: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        native: PakshiNative,
        rules: &Rules,
    ) -> Result<Envelope<Vec<PakshiDay>>, Error> {
        let Envelope { value, provenance } = self.of(first, last, place, offset)?;
        let bird = native.bird();
        let days = value
            .iter()
            .map(|day| {
                Ok(PakshiDay {
                    date: day.day.date.clone(),
                    reading: reading_of(day, bird, *rules)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Envelope::sealing(days, provenance))
    }

    /// As [`AlmanacArea::pakshi_days`] for a request a binding sent.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::pakshi_days`].
    pub fn pakshi_request(self, asked: &PakshiRequest) -> Result<Envelope<Vec<PakshiDay>>, Error> {
        self.pakshi_days(
            &asked.first,
            &asked.last,
            &asked.place,
            asked.offset,
            asked.native,
            &asked.rules,
        )
    }

    /// One day of [`AlmanacArea::pakshi_days`].
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::day`]; `UNSUPPORTED` on a day the Sun does not
    /// both rise and set at the place, which has no yamas.
    pub fn pakshi(
        self,
        date: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        native: impl Into<PakshiNative>,
        rules: &Rules,
    ) -> Result<Envelope<Reading>, Error> {
        let Envelope { value, provenance } = self.day(date, place, offset)?;
        let reading = reading_of(&value, native.into().bird(), *rules)?.ok_or_else(|| {
            Error::unsupported(
                "the Sun does not both rise and set at this place on this day, so it has no yamas",
            )
            .with_hint(
                "Pancha Pakshi divides a day at its sunrise and sunset; read a day the Sun crosses the horizon",
            )
        })?;
        Ok(Envelope::sealing(reading, provenance))
    }
}

impl ChartArea<'_> {
    /// A native from a chart (`03-design/pakshi.md` §2): the Moon's
    /// nakshatra, and the paksha by the Moon's elongation from the Sun at
    /// birth, under `rule` (crux P1). Its [`PakshiNative::bird`] is the bird.
    ///
    /// # Errors
    ///
    /// `INTERNAL` for a document that places no Moon or Sun, or places one
    /// at a longitude that is not finite.
    pub fn pakshi_native(
        self,
        document: &Document,
        rule: BirthBird,
    ) -> Result<PakshiNative, Error> {
        let foundation = &document.foundation;
        let longitude = |graha: Graha| {
            foundation
                .graha(graha)
                .map(|at| at.longitude_deg)
                .ok_or_else(|| Error::internal(format!("a founded chart places {}", graha.key())))
        };
        let moon = longitude(Graha::Moon)?;
        let elongation = (moon - longitude(Graha::Sun)?).rem_euclid(360.0);
        Ok(PakshiNative::Star {
            nakshatra: Nas::try_from_degrees(moon)?.nakshatra(),
            paksha: if elongation < 180.0 {
                Paksha::Shukla
            } else {
                Paksha::Krishna
            },
            rule,
        })
    }
}

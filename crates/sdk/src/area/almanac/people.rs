//! The people digest (`03-design/people-digest.md`, A2-1): for a list of
//! births and one day at one place, each person's running dasha, every
//! graha's transit from their Moon, and the day's events that touch them —
//! the transit hits on their natal points, a dasha period opening, the
//! day's star counted from theirs and the Moon's house from their Moon —
//! in one call. Each part is the SDK's own: a natal reading founds the
//! chart and its dasha, [`ChartArea::gochar`](crate::ChartArea::gochar)
//! reads the transits, [`ChartArea::hits_many`](crate::ChartArea::hits_many)
//! searches the day's sky once for every chart, and the day is the
//! almanac's.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Calendar, DashaSystem, Graha, Nakshatra, Rashi};
use teistro_core::envelope::{Envelope, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::{Depth, JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_dasha::Timeline;
use teistro_gochar::GrahaGochar;
use teistro_gochar::hits::Hit;
use teistro_muhurta::tara::{TaraReading, chandra_house, tara};
use teistro_panchanga::span::Span;
use teistro_serial::Document;

use super::AlmanacArea;
use crate::asked::{DayAsked, instant, offset_of, place_of};
use crate::{ChartRequest, GocharRequest, HitRequest};

/// The field a people digest's refusals are named under.
const PEOPLE: &str = "people";

/// The most births one digest reads.
pub const MOST_PEOPLE: usize = 50;

/// How many levels of the dasha a digest names when a request does not
/// say: the mahadasha, the antardasha and the pratyantardasha.
pub const DASHA_DEPTH: u8 = 3;

/// One person: a name the answer calls them by, and their birth.
#[derive(Clone, Debug, PartialEq)]
pub struct Person {
    /// What the answer calls them.
    pub name: String,
    /// The birth, a UTC Julian day.
    pub instant: JulianDay<Utc>,
    /// Where they were born.
    pub place: Place,
    /// The clock of their birth.
    pub offset: UtcOffset,
}

/// A people digest: the day and the place it is reckoned at, the births,
/// and the dasha read.
///
/// ```
/// use teistro::finders::PeopleRequest;
///
/// let asked = PeopleRequest::from_json(
///     r#"{"day": {"year": 2026, "month": 11, "day": 1}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
///         "utcOffsetSeconds": 20700,
///         "people": [{"name": "Asha", "instant": 2447000.25, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
///                     "utcOffsetSeconds": 20700}]}"#,
/// )?;
/// assert_eq!((asked.people.len(), asked.depth), (1, 3));
/// let none = PeopleRequest::from_json(
///     r#"{"day": {"year": 2026, "month": 11, "day": 1}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
///         "utcOffsetSeconds": 20700, "people": []}"#,
/// )
/// .unwrap_err();
/// assert_eq!(none.field(), Some("people.people"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct PeopleRequest {
    /// The day.
    pub day: CalendarDate,
    /// Where it is reckoned: its sunrise, its star and its sky.
    pub place: Place,
    /// The offset its civil day runs at.
    pub offset: UtcOffset,
    /// The births, in the order the answer lists them.
    pub people: Vec<Person>,
    /// The dasha system read.
    pub dasha: DashaSystem,
    /// How many of its levels are named, 1 to 6.
    pub depth: u8,
}

/// One person as a binding writes them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PeoplePerson"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PersonAsked {
    /// What the answer calls them; each name once.
    name: String,
    /// The birth, a UTC Julian day.
    instant: f64,
    /// The birthplace's latitude, degrees north positive.
    latitude_deg: f64,
    /// The birthplace's longitude, degrees east positive.
    longitude_deg: f64,
    /// The birthplace's height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
    /// The offset from UTC of the birth's clock, in seconds, east positive.
    utc_offset_seconds: i32,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PeopleRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PeopleAsked {
    /// The calendar `day` is in; Gregorian when left out.
    #[serde(default)]
    calendar: Option<Calendar>,
    /// The day the digest is of.
    day: DayAsked,
    /// The latitude of the place the day is reckoned at, degrees north positive.
    latitude_deg: f64,
    /// Its longitude, degrees east positive.
    longitude_deg: f64,
    /// Its height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
    /// The offset from UTC its civil day runs at, in seconds, east positive.
    utc_offset_seconds: i32,
    /// The births, 1 to 50, in the order the answer lists them.
    people: Vec<PersonAsked>,
    /// The dasha system read; `VIMSHOTTARI` when left out.
    #[serde(default)]
    dasha: Option<DashaSystem>,
    /// How many of its levels are named, 1 to 6; 3 when left out.
    #[serde(default)]
    depth: Option<u8>,
}

impl PeopleRequest {
    /// The record a binding sends, as JSON: `day` as `{year, month, day}`
    /// in `calendar` (Gregorian when left out); `latitudeDeg`,
    /// `longitudeDeg`, `altitudeM` and `utcOffsetSeconds` of the place the
    /// day is reckoned at; `people`, each `{name, instant, latitudeDeg,
    /// longitudeDeg, altitudeM, utcOffsetSeconds}`; `dasha`, `VIMSHOTTARI`
    /// when left out; and `depth`, 3 when left out.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place, offset or instant out of range, no person or a name
    /// given twice, or a depth past 1 to 6; `LIMIT` for more than
    /// [`MOST_PEOPLE`]. Each is named under `people`.
    pub fn from_json(text: &str) -> Result<PeopleRequest, Error> {
        let asked: PeopleAsked = teistro_core::strict::read(text, PEOPLE)?;
        asked.request().map_err(|why| why.under(PEOPLE))
    }
}

impl PeopleAsked {
    fn request(self) -> Result<PeopleRequest, Error> {
        if self.people.is_empty() {
            return Err(
                Error::invalid_arg("a digest reads one birth at least").with_field("people")
            );
        }
        if self.people.len() > MOST_PEOPLE {
            return Err(Error::limit(format!(
                "a digest reads at most {MOST_PEOPLE} births, not {}",
                self.people.len()
            ))
            .with_field("people"));
        }
        let depth = self.depth.unwrap_or(DASHA_DEPTH);
        Depth::try_new(depth).map_err(|why| Error::from(why).with_field("depth"))?;
        let mut people: Vec<Person> = Vec::with_capacity(self.people.len());
        for (at, one) in self.people.into_iter().enumerate() {
            let read = || -> Result<Person, Error> {
                if people.iter().any(|held| held.name == one.name) {
                    return Err(Error::invalid_arg(format!(
                        "`{}` is named twice; each name is one row of the answer",
                        one.name
                    ))
                    .with_field("name"));
                }
                Ok(Person {
                    name: one.name.clone(),
                    instant: instant(one.instant, "instant")?,
                    place: place_of(one.latitude_deg, one.longitude_deg, one.altitude_m)?,
                    offset: offset_of(one.utc_offset_seconds)?,
                })
            };
            let person = read().map_err(|why| why.under(&format!("people[{at}]")))?;
            people.push(person);
        }
        Ok(PeopleRequest {
            day: self
                .day
                .in_calendar(self.calendar.unwrap_or(Calendar::Gregorian)),
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            offset: offset_of(self.utc_offset_seconds)?,
            people,
            dasha: self.dasha.unwrap_or(DashaSystem::Vimshottari),
            depth,
        })
    }
}

/// One period of the dasha running at the day's sunrise.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RunningPeriod {
    /// Its level, 1 the mahadasha.
    pub level: u8,
    /// Its lord.
    pub lord: Graha,
    /// Its sign, for a sign-based dasha.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sign: Option<Rashi>,
    /// When it began, a UTC Julian day.
    pub from: JulianDay<Utc>,
    /// When it ends.
    pub to: JulianDay<Utc>,
}

/// One person's day.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PersonDay {
    /// Their name.
    pub name: String,
    /// Their natal Moon's nakshatra, the birth star.
    pub birth_star: Nakshatra,
    /// Their natal Moon's sign, which the transits are counted from.
    pub moon_sign: Rashi,
    /// Their lagna's sign.
    pub lagna: Rashi,
    /// The dasha running at the day's sunrise, one period a level.
    pub dasha: Vec<RunningPeriod>,
    /// The periods that open during the day, from sunrise to the next.
    pub dasha_opens: Vec<RunningPeriod>,
    /// Every graha's transit at the day's sunrise, counted from their
    /// Moon, with the verdict *Phaladeepika* ch. 26 gives it.
    pub transits: Vec<GrahaGochar>,
    /// The day's transit events on their chart, from sunrise to the next:
    /// a graha entering a sign or nakshatra, a station, an aspect
    /// perfecting to a natal point.
    pub hits: Vec<Hit>,
    /// The day's star counted from their birth star.
    pub tara: TaraReading,
    /// The Moon's house from their Moon at sunrise, which Chandrabala reads.
    pub chandra_house: u8,
    /// Whether the Moon stands in the eighth from theirs (chandrashtama).
    pub chandrashtama: bool,
    /// Whether Saturn transits the twelfth, the first or the second from
    /// their Moon (sade sati).
    pub sade_sati: bool,
}

/// What a people digest answers.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PeopleDigest {
    /// The day.
    pub day: CalendarDate,
    /// Its sunrise at the place, a UTC Julian day.
    pub sunrise: JulianDay<Utc>,
    /// The next sunrise, where the day's events end.
    pub next_sunrise: JulianDay<Utc>,
    /// The day's star at sunrise.
    pub nakshatra: Nakshatra,
    /// The Moon's sign at sunrise.
    pub moon_sign: Rashi,
    /// The dasha system read.
    pub dasha: DashaSystem,
    /// Each person's day, in the request's order.
    pub people: Vec<PersonDay>,
}

/// What the day gives every person's reading: its star and the Moon's
/// sign at sunrise, its sunrises, the transit request at sunrise, and the
/// dasha asked.
struct DaySky {
    nakshatra: Nakshatra,
    moon_sign: Rashi,
    sunrise: JulianDay<Utc>,
    next_sunrise: JulianDay<Utc>,
    at_sunrise: GocharRequest,
    depth: Depth,
    system: DashaSystem,
}

/// The member of the span holding `at`, else the first.
fn member_at<T: Copy>(spans: &[Span<T>], at: f64) -> Option<T> {
    spans
        .iter()
        .find(|span| span.whole.from.get() <= at && at < span.whole.to.get())
        .or_else(|| spans.first())
        .map(|span| span.member)
}

/// A chain's periods as the digest names them.
fn periods<'p>(chain: impl Iterator<Item = &'p teistro_dasha::Period>) -> Vec<RunningPeriod> {
    chain
        .zip(1..)
        .map(|(period, level)| RunningPeriod {
            level,
            lord: period.lord,
            sign: period.sign,
            from: period.interval.from,
            to: period.interval.to,
        })
        .collect()
}

/// The periods of `then` that open within `[from, to)`.
fn opening(then: &[RunningPeriod], from: f64, to: f64) -> Vec<RunningPeriod> {
    then.iter()
        .filter(|period| (from..to).contains(&period.from.get()))
        .copied()
        .collect()
}

impl AlmanacArea<'_> {
    /// **A day for many people at once**: for each birth, the dasha
    /// running at the day's sunrise and the periods opening in it, every
    /// graha's transit from their Moon, the day's transit hits on their
    /// chart, the day's star counted from theirs and the Moon's house from
    /// their Moon (`03-design/people-digest.md`). The day and its sky are
    /// reckoned once at the request's place; each chart is founded at its
    /// own birthplace; the day's sky is searched once for every chart.
    ///
    /// ```no_run
    /// use teistro::Context;
    /// use teistro::finders::PeopleRequest;
    ///
    /// let sdk = Context::builder().build()?;
    /// let asked = PeopleRequest::from_json(
    ///     r#"{"day": {"year": 2026, "month": 11, "day": 1}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
    ///         "utcOffsetSeconds": 20700,
    ///         "people": [{"name": "Asha", "instant": 2447000.25, "latitudeDeg": 27.7172,
    ///                     "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}]}"#,
    /// )?;
    /// for person in sdk.almanac().people(&asked)?.value.people {
    ///     println!("{}: {:?}, {} hits", person.name, person.dasha.first().map(|p| p.lord), person.hits.len());
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A context with no ephemeris; a birth or a day the ephemeris does
    /// not reach; a dasha system whose reading needs more than a natal
    /// chart (named `people.dasha`).
    pub fn people(self, request: &PeopleRequest) -> Result<Envelope<PeopleDigest>, Error> {
        let Envelope {
            value: days,
            mut provenance,
        } = self.of(&request.day, &request.day, &request.place, request.offset)?;
        let Some(today) = days.first() else {
            return Err(Error::internal("a day asked of the almanac answered none"));
        };
        let (sunrise, next_sunrise) = (today.day.sunrise, today.day.next_sunrise);
        let at = sunrise.get();
        let (Some(nakshatra), Some(moon_sign)) = (
            member_at(&today.limbs.nakshatra, at),
            member_at(&today.moon.signs, at),
        ) else {
            return Err(Error::internal("a reckoned day has a star and a Moon sign"));
        };
        let chart = self.context.chart();
        let depth = Depth::try_new(request.depth)
            .map_err(|why| Error::from(why).with_field("people.depth"))?;
        let natals: Vec<Document> = request
            .people
            .iter()
            .enumerate()
            .map(|(at, person)| {
                let asked =
                    ChartRequest::at(person.place, person.offset).with_dashas([request.dasha]);
                chart
                    .reading(person.instant, &asked)
                    .map(|read| read.value)
                    .map_err(|why| why.under(&format!("people.people[{at}]")))
            })
            .collect::<Result<_, _>>()?;
        let hits = chart
            .hits_many(&natals, &HitRequest::between(sunrise, next_sunrise))?
            .value;
        let sky = DaySky {
            nakshatra,
            moon_sign,
            sunrise,
            next_sunrise,
            at_sunrise: GocharRequest::at(sunrise),
            depth,
            system: request.dasha,
        };
        let people = request
            .people
            .iter()
            .zip(&natals)
            .zip(hits)
            .map(|((person, natal), hits)| self.person_day(person, natal, hits, &sky))
            .collect::<Result<Vec<_>, _>>()?;
        provenance.input_hash = content_hash(&serde_json::json!({
            "day": request.day.to_string(),
            "place": request.place,
            "utcOffsetSeconds": request.offset.seconds(),
            "dasha": request.dasha,
            "depth": request.depth,
            "people": request.people.iter().map(|person| serde_json::json!({
                "name": person.name,
                "instant": person.instant,
                "place": person.place,
                "utcOffsetSeconds": person.offset.seconds(),
            })).collect::<Vec<_>>(),
        }));
        Ok(Envelope::sealing(
            PeopleDigest {
                day: request.day.clone(),
                sunrise,
                next_sunrise,
                nakshatra,
                moon_sign,
                dasha: request.dasha,
                people,
            },
            provenance,
        ))
    }

    /// One person's day against the day's sky.
    fn person_day(
        self,
        person: &Person,
        natal: &Document,
        hits: Vec<Hit>,
        sky: &DaySky,
    ) -> Result<PersonDay, Error> {
        let DaySky {
            nakshatra,
            moon_sign,
            sunrise,
            next_sunrise,
            ref at_sunrise,
            depth,
            system,
        } = *sky;
        let chart = self.context.chart();
        let moon = natal
            .foundation
            .graha(Graha::Moon)
            .ok_or_else(|| Error::internal("a founded chart places the Moon"))?;
        let birth_star = Nas::try_from_degrees(moon.longitude_deg)
            .map_err(Error::from)?
            .nakshatra();
        let natal_moon = Rashi::of_longitude(moon.longitude_deg);
        let lagna = Rashi::of_longitude(natal.foundation.lagna_deg);
        let cursor = chart
            .dasha(natal, system)
            .map_err(|why| why.under("people.dasha"))?;
        let running = periods(cursor.at(sunrise, depth).iter());
        let ending = periods(cursor.at(next_sunrise, depth).iter());
        let transits = chart
            .gochar(natal, at_sunrise)?
            .value
            .into_iter()
            .next()
            .map(|reading| reading.grahas.to_vec())
            .unwrap_or_default();
        let house = chandra_house(natal_moon, moon_sign);
        let saturn = transits
            .iter()
            .find(|transit| transit.graha == Graha::Saturn)
            .map(|transit| transit.house);
        Ok(PersonDay {
            name: person.name.clone(),
            birth_star,
            moon_sign: natal_moon,
            lagna,
            dasha_opens: opening(&ending, sunrise.get(), next_sunrise.get()),
            dasha: running,
            transits,
            hits,
            tara: tara(birth_star, nakshatra),
            chandra_house: house,
            chandrashtama: house == 8,
            sade_sati: matches!(saturn, Some(12 | 1 | 2)),
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use teistro_core::interval::Interval;

    use super::*;

    fn span(member: Nakshatra, from: f64, to: f64) -> Span<Nakshatra> {
        let whole = Interval::literal(from, to);
        Span {
            member,
            whole,
            inside: whole,
        }
    }

    #[test]
    fn the_member_at_an_instant_is_the_span_holding_it_else_the_first() {
        let spans = [
            span(Nakshatra::Ashwini, 10.0, 11.0),
            span(Nakshatra::Bharani, 11.0, 12.0),
        ];
        assert_eq!(member_at(&spans, 11.5), Some(Nakshatra::Bharani));
        assert_eq!(member_at(&spans, 9.0), Some(Nakshatra::Ashwini));
        assert_eq!(member_at::<Nakshatra>(&[], 9.0), None);
    }

    #[test]
    fn a_period_opens_on_the_day_only_from_its_sunrise_on() {
        let period = |from: f64| RunningPeriod {
            level: 3,
            lord: Graha::Venus,
            sign: None,
            from: JulianDay::literal(from),
            to: JulianDay::literal(from + 10.0),
        };
        let then = [period(5.0), period(10.2), period(11.0)];
        let opened = opening(&then, 10.0, 11.0);
        assert_eq!(opened.len(), 1);
        assert_eq!(opened[0].from.get(), 10.2);
    }

    #[test]
    fn a_request_is_refused_by_its_place() {
        let head = r#""day": {"year": 2026, "month": 11, "day": 1}, "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20700"#;
        let person = r#"{"name": "A", "instant": 2447000.25, "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20700}"#;
        let field = |extra: &str, list: &str| {
            PeopleRequest::from_json(&format!(r#"{{{head}{extra}, "people": [{list}]}}"#))
                .unwrap_err()
                .field()
                .map(str::to_owned)
        };
        assert_eq!(
            field("", &format!("{person}, {person}")).as_deref(),
            Some("people.people[1].name")
        );
        assert_eq!(
            field(r#", "depth": 0"#, person).as_deref(),
            Some("people.depth")
        );
        let far = person.replace("27.7", "97.7");
        assert_eq!(
            field("", &far).as_deref(),
            Some("people.people[0].latitudeDeg")
        );
        let many = vec![person.replace("\"A\"", "\"{}\""); MOST_PEOPLE + 1]
            .iter()
            .enumerate()
            .map(|(n, one)| one.replace("{}", &n.to_string()))
            .collect::<Vec<_>>()
            .join(", ");
        let refused =
            PeopleRequest::from_json(&format!(r#"{{{head}, "people": [{many}]}}"#)).unwrap_err();
        assert_eq!(refused.status, teistro_core::error::Status::Limit);
        let read = PeopleRequest::from_json(&format!(
            r#"{{{head}, "dasha": "YOGINI", "depth": 2, "people": [{person}]}}"#
        ))
        .unwrap();
        assert_eq!((read.dasha, read.depth), (DashaSystem::Yogini, 2));
    }
}

//! Mundane charts (`03-design/mundane-charts.md`): a moment found for a
//! named kind, an ingress, the lunation before one, an eclipse or a great
//! conjunction, or taken from an event on record, and the chart cast for
//! it at a supplied place.
//!
//! Every moment is found by the solvers the rest of the SDK searches
//! with: an ingress is the Sun's longitude crossing its degree in the
//! context's own zodiac, or in the tropical one, through the crossing
//! search; a lunation is the elongation crossing 0° or 180° through the
//! same search; an eclipse is the eclipse search's next greatest; a great
//! conjunction is Jupiter less Saturn crossing 0°. Nothing here solves on
//! its own.

use serde::{Deserialize, Serialize};
use teistro_astro::DeltaTModel;
use teistro_astro::completion::Completion;
use teistro_astro::eclipse::{Eclipses, LunarEclipseKind, ShadowRule, SolarEclipseKind};
use teistro_astro::events::{Lattice, Quantity, Search};
use teistro_astro::precession::PrecessionModel;
use teistro_astro::sidereal::{Sidereal, Zodiac as SearchZodiac};
use teistro_calendar::{CalendarSystem, Gregorian};
use teistro_chart::foundation::ChartFoundation;
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::catalogue::{ChartKind, MundaneKind};
use teistro_core::envelope::{Envelope, Provenance, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Place, Tt, Ut1, Utc};
use teistro_core::settings::Settings;
use teistro_core::time::{LocalClock as _, UtcOffset};
use teistro_port_ephemeris::{
    Astronomy, Body, Centre, EphemerisProvider, Frame, Zodiac as FrameZodiac,
};
use teistro_time::ZoneSpec;

use crate::DayClock;
use crate::area::ChartArea;
use crate::asked::{instant, place_of};
use crate::ephemeris::no_ephemeris;
use crate::reading::ReadClock;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const MUNDANE: &str = "mundane";

/// How far after its start an ingress is looked for, days: a year and a
/// few days, so the year's own ingress is always reached.
const INGRESS_WINDOW_DAYS: f64 = 370.0;

/// How far before an ingress its lunation is looked for, days: more than
/// a synodic month.
const LUNATION_REACH_DAYS: f64 = 32.0;

/// How far after its start a great conjunction is looked for, days:
/// twenty years and a quarter, longer than any gap between two of them.
const CONJUNCTION_WINDOW_DAYS: f64 = 7_400.0;

/// The step a great conjunction is sampled at, days. Jupiter less Saturn
/// moves under half a degree a day, and the three passes of a triple
/// conjunction are months apart (C1153), so no pass falls between two
/// samples; the search still refines each to its own tolerance. The
/// search's own step, a day, made a twenty-year window cost over a second.
/// Held against the daily scan over the triple conjunction of 1940–41
/// (`tests/mundane.rs`).
pub const CONJUNCTION_STEP_DAYS: f64 = 10.0;

/// An event on record, cast as given (`EVENT`).
///
/// ```
/// use teistro::MundaneRequest;
///
/// let asked = MundaneRequest::from_json(r#"{"kind": "EVENT", "event": {"name": "A treaty", "instant": 2433000.25, "source": "a gazette"}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324}"#)?;
/// assert_eq!(asked.event.as_ref().map(|event| event.source.as_str()), Some("a gazette"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MundaneEvent {
    /// What happened.
    pub name: String,
    /// When, a Julian day (UTC).
    pub instant: f64,
    /// Where the record of it comes from, carried on the answer.
    pub source: String,
}

/// A mundane chart as a binding asks for it: a kind, when to look from
/// (a `year` or an instant `from`), or the `event` an `EVENT` casts, and
/// the place the chart is cast for.
///
/// ```
/// use teistro::MundaneRequest;
/// use teistro::catalogue::MundaneKind;
///
/// let asked = MundaneRequest::from_json(r#"{"kind": "MESHA_SANKRANTI", "year": 2026, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}"#)?;
/// assert_eq!(asked.kind, MundaneKind::MeshaSankranti);
/// // A lunation reads the ingress it precedes; an ingress does not.
/// let refused = MundaneRequest::from_json(r#"{"kind": "ARIES_INGRESS", "year": 2026, "ingress": "ARIES_INGRESS", "latitudeDeg": 0, "longitudeDeg": 0}"#).unwrap_err();
/// assert_eq!(refused.field(), Some("mundane.ingress"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MundaneRequest {
    /// Which chart.
    pub kind: MundaneKind,
    /// The year (Gregorian) whose ingress, or whose first eclipse or
    /// conjunction after 1 January 0h UTC, is asked; or name `from`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    /// The instant the search starts at, a Julian day (UTC): the first
    /// ingress, eclipse or conjunction at or after it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<f64>,
    /// For a lunation, the ingress it precedes; `MESHA_SANKRANTI` when
    /// left out, the new moon that opens Chaitra (C1151).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ingress: Option<MundaneKind>,
    /// For an `EVENT`, the event on record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<MundaneEvent>,
    /// The place's latitude, degrees, north positive (C1152).
    pub latitude_deg: f64,
    /// The place's longitude, degrees, east positive.
    pub longitude_deg: f64,
    /// The place's altitude, metres.
    #[serde(default)]
    pub altitude_m: f64,
    /// The local clock's offset from UTC, seconds, east positive; or
    /// `zone`, not both; UTC when neither is given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utc_offset_seconds: Option<i32>,
    /// The zone the moment is read on, at the offset it had then
    /// (`03-design/time-and-timezone.md` §3.4); or `utcOffsetSeconds`, not
    /// both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<ZoneSpec>,
}

/// The ingress a lunation was found before.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MundaneIngress {
    /// Which ingress.
    pub kind: MundaneKind,
    /// When, a Julian day (UTC).
    pub instant: JulianDay<Utc>,
}

/// The moment a mundane chart is cast for, and where.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MundaneMoment {
    /// Which chart.
    pub kind: MundaneKind,
    /// The moment, a Julian day (UTC).
    pub instant: JulianDay<Utc>,
    /// The place's latitude, degrees.
    pub latitude_deg: f64,
    /// The place's longitude, degrees.
    pub longitude_deg: f64,
    /// The place's altitude, metres.
    pub altitude_m: f64,
    /// The local clock's offset from UTC at the moment, seconds: a zone's
    /// offset then.
    pub utc_offset_seconds: i32,
    /// For a lunation, the ingress it precedes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingress: Option<MundaneIngress>,
    /// For a solar eclipse, its kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solar_eclipse: Option<SolarEclipseKind>,
    /// For a lunar eclipse, its kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lunar_eclipse: Option<LunarEclipseKind>,
    /// For an event, its name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// For an event, the source the record came from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// A mundane chart: its moment and the chart cast for it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MundaneChart {
    /// The moment and the place.
    pub moment: MundaneMoment,
    /// The chart, `chart_kind.MUNDANE` (`chart_kind.EVENT` for an event).
    pub chart: ChartFoundation,
}

/// The six ingresses, each with the Sun's degree and whether it is read
/// in the context's own zodiac.
fn ingress_of(kind: MundaneKind) -> Option<(f64, bool)> {
    match kind {
        MundaneKind::MeshaSankranti => Some((0.0, true)),
        MundaneKind::MakaraSankranti => Some((270.0, true)),
        MundaneKind::AriesIngress => Some((0.0, false)),
        MundaneKind::CancerIngress => Some((90.0, false)),
        MundaneKind::LibraIngress => Some((180.0, false)),
        MundaneKind::CapricornIngress => Some((270.0, false)),
        _ => None,
    }
}

/// The lattice of elongations a lunation kind crosses.
fn lunation_of(kind: MundaneKind) -> Option<Lattice> {
    match kind {
        MundaneKind::NewMoonBeforeIngress => Some(Lattice::single(0.0)),
        MundaneKind::FullMoonBeforeIngress => Some(Lattice::single(180.0)),
        MundaneKind::SyzygyBeforeIngress => Some(Lattice {
            origin_deg: 0.0,
            step_deg: 180.0,
        }),
        _ => None,
    }
}

impl MundaneRequest {
    /// The request a binding writes as `mundane`, read and checked; a
    /// refusal names the field under `mundane`.
    ///
    /// ```
    /// use teistro::MundaneRequest;
    ///
    /// let both = MundaneRequest::from_json(r#"{"kind": "SOLAR_ECLIPSE", "year": 2024, "from": 2460000.5, "latitudeDeg": 0, "longitudeDeg": 0}"#).unwrap_err();
    /// assert_eq!(both.field(), Some("mundane.from"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or a request [`MundaneRequest::check`] refuses.
    pub fn from_json(text: &str) -> Result<MundaneRequest, Error> {
        let asked: MundaneRequest = teistro_core::strict::read(text, MUNDANE)?;
        asked.check().map_err(|why| why.under(MUNDANE))?;
        Ok(asked)
    }

    /// The clock the chart is cast on: the offset stated, the zone, or UTC
    /// when neither is given.
    ///
    /// ```
    /// use teistro::{DayClock, MundaneRequest, UtcOffset, ZoneSpec};
    ///
    /// let utc = MundaneRequest::from_json(r#"{"kind": "MESHA_SANKRANTI", "year": 2026, "latitudeDeg": 27.7172, "longitudeDeg": 85.324}"#)?;
    /// assert_eq!(utc.clock()?, DayClock::Offset(UtcOffset::UTC));
    /// let zoned = MundaneRequest::from_json(r#"{"kind": "MESHA_SANKRANTI", "year": 2026, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}}"#)?;
    /// assert_eq!(zoned.clock()?, DayClock::Zone(ZoneSpec::iana("Asia/Kathmandu")));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `zone` for both, on `utcOffsetSeconds` for an
    /// offset out of range.
    pub fn clock(&self) -> Result<DayClock, Error> {
        match (self.utc_offset_seconds, &self.zone) {
            (None, None) => Ok(DayClock::Offset(UtcOffset::UTC)),
            (seconds, zone) => DayClock::asked(seconds, zone.clone()),
        }
    }

    /// The request as its input hash seals it: an offset left out is the
    /// UTC it stands for, as the hash has always read it.
    fn sealed(&self) -> MundaneRequest {
        let mut sealed = self.clone();
        if sealed.zone.is_none() {
            sealed.utc_offset_seconds.get_or_insert(0);
        }
        sealed
    }

    /// Whether the request can be answered, before any sky is read.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `event` missing from an `EVENT` or sent with
    /// another kind; `year` and `from` both or neither named for a kind
    /// that searches, or either named for an `EVENT`; `ingress` for a kind
    /// that is not a lunation, or naming a kind that is not an ingress; a
    /// place or an offset out of range; `zone` beside an offset.
    pub fn check(&self) -> Result<(), Error> {
        place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?;
        self.clock()?;
        let event = self.kind == MundaneKind::Event;
        match (&self.event, event) {
            (None, true) => {
                return Err(
                    Error::invalid_arg("an EVENT chart casts the event on record")
                        .with_field("event")
                        .with_hint("send {\"name\", \"instant\", \"source\"}"),
                );
            }
            (Some(_), false) => {
                return Err(Error::invalid_arg(format!(
                    "only an EVENT chart reads an event, not {}",
                    self.kind.key()
                ))
                .with_field("event"));
            }
            (Some(record), true) => {
                instant(record.instant, "event.instant")?;
            }
            (None, false) => {}
        }
        match (self.year, self.from, event) {
            (Some(_), _, true) => {
                return Err(
                    Error::invalid_arg("an EVENT chart is cast at its own instant")
                        .with_field("year"),
                );
            }
            (_, Some(_), true) => {
                return Err(
                    Error::invalid_arg("an EVENT chart is cast at its own instant")
                        .with_field("from"),
                );
            }
            (Some(_), Some(_), false) => {
                return Err(Error::invalid_arg(
                    "name a year or an instant to search from, not both",
                )
                .with_field("from"));
            }
            (None, None, false) => {
                return Err(
                    Error::invalid_arg("name the year, or the instant to search from")
                        .with_field("year"),
                );
            }
            (None, Some(from), false) => {
                instant(from, "from")?;
            }
            _ => {}
        }
        if let Some(ingress) = self.ingress {
            if lunation_of(self.kind).is_none() {
                return Err(Error::invalid_arg(format!(
                    "only a lunation reads the ingress it precedes, not {}",
                    self.kind.key()
                ))
                .with_field("ingress"));
            }
            if ingress_of(ingress).is_none() {
                return Err(Error::invalid_arg(format!("{} is not an ingress", ingress.key()))
                    .with_field("ingress")
                    .with_hint("an ingress is one of the two sankrantis or the four tropical ingresses"));
            }
        }
        Ok(())
    }

    /// The instant a search starts at: `from`, or 1 January 0h UTC of
    /// `year`.
    fn start(&self) -> Result<JulianDay<Utc>, Error> {
        if let Some(from) = self.from {
            return instant(from, "from");
        }
        let year = self.year.unwrap_or_default();
        Gregorian
            .to_fixed_ymd(year, 1, 1)
            .and_then(|day| day.jd_at_midnight().map_err(Error::from))
            .map_err(|why| why.with_field("year"))
    }
}

/// UT1 as the searches read it from a UTC Julian day, the convention a
/// chart's own instant is founded under.
fn ut1(at: JulianDay<Utc>) -> JulianDay<Ut1> {
    JulianDay::<Ut1>::literal(at.get())
}

/// A searched UT1 instant as a chart is founded at.
fn utc(at: JulianDay<Ut1>) -> JulianDay<Utc> {
    JulianDay::<Utc>::literal(at.get())
}

impl ChartArea<'_> {
    /// A **mundane chart** (`03-design/mundane-charts.md`): the moment its
    /// kind names, found by the SDK's own searches, and the chart cast for
    /// it at the request's place, `chart_kind.MUNDANE` (`chart_kind.EVENT`
    /// for an event on record). Sealed by the chart's provenance with the
    /// request as its input.
    ///
    /// The sankrantis are read in the context's own zodiac; the four
    /// tropical ingresses in the tropical one; a lunation is the last
    /// before its ingress (C1151); an eclipse is the next after the
    /// start, at its greatest (C1154); a great conjunction is the next
    /// time Jupiter and Saturn share a geocentric longitude (C1153).
    ///
    /// ```
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::{Context, Ephemeris, MundaneRequest};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let asked = MundaneRequest::from_json(r#"{"kind": "ARIES_INGRESS", "year": 2024, "latitudeDeg": 51.5, "longitudeDeg": -0.12}"#)?;
    /// let cast = sdk.chart().mundane(&asked)?.value;
    /// // USNO: the March equinox of 2024 fell on 20 March at 03:06 UTC.
    /// assert!((cast.moment.instant.get() - 2_460_389.629_2).abs() < 2.0 / 1440.0);
    /// assert_eq!(cast.chart.kind, teistro::catalogue::ChartKind::Mundane);
    /// # }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// What [`MundaneRequest::check`] refuses; a context with no
    /// ephemeris; an eclipse asked of a classical astronomy, whose eclipse
    /// is its own method (`UNSUPPORTED`, naming `kind`); a search that
    /// finds nothing inside its window or the provider's range.
    pub fn mundane(self, asked: &MundaneRequest) -> Result<Envelope<MundaneChart>, Error> {
        let (moment, chart, provenance) = self.cast(asked)?;
        Ok(Envelope::sealing(
            MundaneChart { moment, chart },
            provenance,
        ))
    }

    /// The moment a mundane chart is cast for, and where, without the
    /// chart: what `ts_chart_mundane` and the `chart.mundane` record
    /// answer, a binding founding the chart from it with its own chart
    /// reader. Sealed as [`ChartArea::mundane`] is.
    ///
    /// ```
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::{Context, Ephemeris, MundaneRequest};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let asked = MundaneRequest::from_json(r#"{"kind": "EVENT", "event": {"name": "A coronation", "instant": 2433000.25, "source": "a gazette"}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324}"#)?;
    /// let moment = sdk.chart().mundane_moment(&asked)?.value;
    /// assert_eq!(moment.source.as_deref(), Some("a gazette"));
    /// # }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::mundane`].
    pub fn mundane_moment(self, asked: &MundaneRequest) -> Result<Envelope<MundaneMoment>, Error> {
        let (moment, _, provenance) = self.cast(asked)?;
        Ok(Envelope::sealing(moment, provenance))
    }

    /// The moment, the chart and the provenance to seal either with.
    fn cast(
        self,
        asked: &MundaneRequest,
    ) -> Result<(MundaneMoment, ChartFoundation, Provenance), Error> {
        asked.check()?;
        let place = place_of(asked.latitude_deg, asked.longitude_deg, asked.altitude_m)?;
        let clock = asked.clock()?;
        let read = ReadClock::of(&clock, self.context())?;
        let mut moment = MundaneMoment {
            kind: asked.kind,
            instant: JulianDay::<Utc>::literal(0.0),
            latitude_deg: asked.latitude_deg,
            longitude_deg: asked.longitude_deg,
            altitude_m: asked.altitude_m,
            utc_offset_seconds: 0,
            ingress: None,
            solar_eclipse: None,
            lunar_eclipse: None,
            name: None,
            source: None,
        };
        let chart_kind = if let Some(event) = &asked.event {
            moment.instant = instant(event.instant, "event.instant")?;
            moment.name = Some(event.name.clone());
            moment.source = Some(event.source.clone());
            ChartKind::Event
        } else {
            self.search(asked, &place, &mut moment)?;
            ChartKind::Mundane
        };
        moment.utc_offset_seconds = read.offset_at(moment.instant).seconds();
        let Envelope {
            value: chart,
            mut provenance,
        } = self.found(moment.instant, &place, clock, chart_kind)?;
        provenance.input_hash = content_hash(&asked.sealed());
        Ok((moment, chart, provenance))
    }

    /// Finds the moment a searched kind names, writing it into `moment`.
    fn search(
        self,
        asked: &MundaneRequest,
        place: &Place,
        moment: &mut MundaneMoment,
    ) -> Result<(), Error> {
        let context = self.context();
        let provider = context.ephemeris().ok_or_else(no_ephemeris)?;
        let sky = Sky {
            completion: Completion::new(
                provider,
                context.settings().provider.overrides,
                context.delta_t(),
            ),
            settings: context.settings(),
            delta_t: context.delta_t(),
            place: *place,
        };
        let start = asked.start()?;
        let kind = asked.kind;
        if ingress_of(kind).is_some() {
            moment.instant = sky.ingress(kind, start)?;
        } else if let Some(lattice) = lunation_of(kind) {
            let before = asked.ingress.unwrap_or(MundaneKind::MeshaSankranti);
            let at = sky.ingress(before, start)?;
            moment.instant = sky.lunation_before(lattice, at)?;
            moment.ingress = Some(MundaneIngress {
                kind: before,
                instant: at,
            });
        } else if kind == MundaneKind::GreatConjunction {
            moment.instant = sky.great_conjunction(start)?;
        } else {
            sky.eclipse(kind == MundaneKind::SolarEclipse, start, moment)?;
        }
        Ok(())
    }
}

/// What every search reads: the context's sky, its settings and the place.
struct Sky<'a> {
    completion: Completion<'a, dyn EphemerisProvider + 'a>,
    settings: &'a Settings,
    delta_t: DeltaTModel,
    place: Place,
}

impl Sky<'_> {
    /// The first ingress of `kind` at or after `from`: the Sun's longitude
    /// crossing its degree in the context's own zodiac (a sankranti) or the
    /// tropical one.
    fn ingress(&self, kind: MundaneKind, from: JulianDay<Utc>) -> Result<JulianDay<Utc>, Error> {
        let (degree, own) = ingress_of(kind)
            .ok_or_else(|| Error::internal(format!("{} is an ingress", kind.key())))?;
        let searched = ChartZodiac::searched(
            &self.completion,
            self.settings,
            JulianDay::<Tt>::literal(from.get()),
            (PrecessionModel::default(), self.delta_t),
        )?;
        let (frame, zodiac) = if own {
            (searched.frame, searched.zodiac)
        } else {
            (
                Frame {
                    zodiac: FrameZodiac::Tropical,
                    ..searched.frame
                },
                SearchZodiac {
                    ayanamsha: None,
                    ..searched.zodiac
                },
            )
        };
        let mut longitudes = self.completion.longitudes(frame);
        if frame.centre == Centre::Topocentric {
            longitudes = longitudes.with_observer(self.place);
        }
        let read = Sidereal::over(&longitudes, zodiac);
        let found = Search::new(
            &read,
            Quantity::Longitude(Body::Sun),
            Lattice::single(degree),
        )
        .next_within(ut1(from), INGRESS_WINDOW_DAYS)?
        .ok_or_else(|| {
            Error::new(
                Status::NotConverged,
                format!("no {} within a year of {}", kind.key(), from.get()),
            )
            .with_field("year")
        })?;
        Ok(utc(found.instant))
    }

    /// The last lunation on `lattice` before the ingress at `at` (C1151).
    fn lunation_before(
        &self,
        lattice: Lattice,
        at: JulianDay<Utc>,
    ) -> Result<JulianDay<Utc>, Error> {
        let longitudes = self.completion.longitudes(Frame::CANONICAL);
        let reach = ut1(at).plus_days(-LUNATION_REACH_DAYS)?;
        let found = Search::new(&longitudes, Quantity::ELONGATION, lattice)
            .between(reach, ut1(at))?
            .pop()
            .ok_or_else(|| Error::internal("a synodic month holds a new and a full moon"))?;
        Ok(utc(found.instant))
    }

    /// The next time Jupiter and Saturn share a geocentric longitude
    /// (C1153).
    fn great_conjunction(&self, start: JulianDay<Utc>) -> Result<JulianDay<Utc>, Error> {
        let longitudes = self.completion.longitudes(Frame::CANONICAL);
        let apart = Quantity::Composite {
            a: 1.0,
            first: Body::Jupiter,
            b: -1.0,
            second: Body::Saturn,
        };
        let found = Search::new(&longitudes, apart, Lattice::single(0.0))
            .with_step_days(CONJUNCTION_STEP_DAYS)
            .next_within(ut1(start), CONJUNCTION_WINDOW_DAYS)?
            .ok_or_else(|| {
                Error::new(
                    Status::NotConverged,
                    "no great conjunction within twenty years",
                )
                .with_field("year")
            })?;
        Ok(utc(found.instant))
    }

    /// The next solar or lunar eclipse after `start`, at its greatest
    /// (C1154), written into `moment` with its kind.
    fn eclipse(
        &self,
        solar: bool,
        start: JulianDay<Utc>,
        moment: &mut MundaneMoment,
    ) -> Result<(), Error> {
        if self.completion.capabilities().astronomy == Astronomy::Classical {
            return Err(Error::unsupported(
                "a classical astronomy's eclipse is its own method, which a mundane chart does not read",
            )
            .with_field("kind"));
        }
        let eclipses = Eclipses::new(&self.completion, self.delta_t).with_shadow(
            ShadowRule::of_setting(self.settings.panchanga.eclipse_shadow)?,
        );
        if solar {
            let found = eclipses.next_solar(ut1(start))?;
            moment.instant = utc(found.greatest);
            moment.solar_eclipse = Some(found.kind);
        } else {
            let found = eclipses.next_lunar(ut1(start))?;
            moment.instant = utc(found.greatest);
            moment.lunar_eclipse = Some(found.kind);
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "a test fails by panicking, and a start is an exact day"
    )]

    use super::*;

    const PLACE: &str = r#""latitudeDeg": 27.7172, "longitudeDeg": 85.324"#;

    fn refused(body: &str) -> Option<String> {
        MundaneRequest::from_json(&format!("{{{body}, {PLACE}}}"))
            .unwrap_err()
            .field()
            .map(str::to_owned)
    }

    #[test]
    fn every_misread_request_is_refused_by_its_field() {
        for (body, field) in [
            (r#""kind": "EVENT""#, "mundane.event"),
            (
                r#""kind": "ARIES_INGRESS", "year": 2024, "event": {"name": "x", "instant": 2460000.5, "source": "y"}"#,
                "mundane.event",
            ),
            (
                r#""kind": "EVENT", "year": 2024, "event": {"name": "x", "instant": 2460000.5, "source": "y"}"#,
                "mundane.year",
            ),
            (
                r#""kind": "EVENT", "from": 2460000.5, "event": {"name": "x", "instant": 2460000.5, "source": "y"}"#,
                "mundane.from",
            ),
            (r#""kind": "SOLAR_ECLIPSE""#, "mundane.year"),
            (
                r#""kind": "NEW_MOON_BEFORE_INGRESS", "year": 2024, "ingress": "SOLAR_ECLIPSE""#,
                "mundane.ingress",
            ),
            (
                r#""kind": "GREAT_CONJUNCTION", "year": 2024, "ingress": "ARIES_INGRESS""#,
                "mundane.ingress",
            ),
        ] {
            assert_eq!(refused(body).as_deref(), Some(field), "{body}");
        }
        let place = MundaneRequest::from_json(
            r#"{"kind": "ARIES_INGRESS", "year": 2024, "latitudeDeg": 95, "longitudeDeg": 0}"#,
        )
        .unwrap_err();
        assert_eq!(place.field(), Some("mundane.latitudeDeg"));
        let offset = MundaneRequest::from_json(
            r#"{"kind": "ARIES_INGRESS", "year": 2024, "latitudeDeg": 0, "longitudeDeg": 0, "utcOffsetSeconds": 999999}"#,
        )
        .unwrap_err();
        assert_eq!(offset.field(), Some("mundane.utcOffsetSeconds"));
    }

    #[test]
    fn a_year_starts_its_search_at_its_first_midnight() {
        let asked = MundaneRequest::from_json(&format!(
            r#"{{"kind": "LUNAR_ECLIPSE", "year": 2000, {PLACE}}}"#
        ))
        .unwrap();
        assert_eq!(asked.start().unwrap().get(), 2_451_544.5);
        let from = MundaneRequest::from_json(&format!(
            r#"{{"kind": "LUNAR_ECLIPSE", "from": 2451600.25, {PLACE}}}"#
        ))
        .unwrap();
        assert_eq!(from.start().unwrap().get(), 2_451_600.25);
        // Every kind is an ingress, a lunation or neither, never both.
        for kind in MundaneKind::ALL {
            assert!(
                !(ingress_of(kind).is_some() && lunation_of(kind).is_some()),
                "{kind:?}"
            );
        }
    }
}

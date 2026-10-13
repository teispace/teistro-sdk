//! Progressions and directions through the façade
//! (`03-design/western-progressions.md`).
//!
//! `teistro-western` holds the measures; this founds the chart at the
//! instant of sky a progression answers, turns the angles by the asked
//! method, and moves a birth's points by a direction's arc.

use std::borrow::Cow;

use teistro_astro::houses::{Input, houses};
use teistro_astro::sky::{Spherical, ecliptic_to_equatorial};
use teistro_chart::OuterPlanets;
use teistro_chart::foundation::{ChartAngles, ChartFoundation};
use teistro_core::angle::declination_deg;
use teistro_core::catalogue::{Graha, HouseSystem};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_gochar::hits::{HitEvent, Motion, NatalPoint};
use teistro_serial::Document;
use teistro_western::{
    AngleMethod, ArcMeasure, Meridian, ParallelRequest, Progression, SpanSearch, SunAt,
    YearMeasure, parallel_spans, progressed_armc,
};

use crate::area::ChartArea;
use crate::declinations::Declinations;
use crate::hit_request::{HitKind, HitRequest};
use crate::reading::ChartRequest;

/// What a progressed chart is asked with: the measure, and how the angles
/// move.
///
/// ```
/// use teistro::{AngleMethod, ProgressionRequest};
/// use teistro::western::{Rate, YearMeasure};
///
/// let leo = ProgressionRequest::default()
///     .with_year(YearMeasure::NoonSiderealTime)
///     .with_angles(AngleMethod::NaibodRightAscension);
/// assert_eq!(leo.progression.rate, Rate::SECONDARY);
/// let minor = ProgressionRequest::default().with_rate(Rate::MINOR);
/// assert_eq!(minor.angles, AngleMethod::NaibodRightAscension);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressionRequest {
    /// The rate and the year: a day for a tropical year unless asked.
    pub progression: Progression,
    /// How the angles move: Leo's mean Sun in right ascension unless asked
    /// (C237).
    pub angles: AngleMethod,
}

impl ProgressionRequest {
    /// The same request at another rate.
    #[must_use]
    pub const fn with_rate(self, rate: teistro_western::Rate) -> ProgressionRequest {
        ProgressionRequest {
            progression: self.progression.with_rate(rate),
            ..self
        }
    }

    /// The same request under another year (C236).
    #[must_use]
    pub const fn with_year(self, year: YearMeasure) -> ProgressionRequest {
        ProgressionRequest {
            progression: self.progression.with_year(year),
            ..self
        }
    }

    /// The same request with the angles moved another way (C237).
    #[must_use]
    pub const fn with_angles(self, angles: AngleMethod) -> ProgressionRequest {
        ProgressionRequest { angles, ..self }
    }
}

/// A progressed chart: the instant of sky that measures an instant of
/// life, the chart founded there, and its angles turned by the asked method.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Progressed {
    /// The instant of life asked for.
    pub life: JulianDay<Utc>,
    /// The instant of sky that measures it.
    pub sky: JulianDay<Utc>,
    /// The chart at `sky`, at the request's place. Its planets are the
    /// progressed planets; its own angles are the quotidian ones.
    pub chart: Envelope<Document>,
    /// The progressed angles by [`ProgressionRequest::angles`], in the
    /// chart's zodiac.
    pub angles: ChartAngles,
    /// The progressed meridian's right ascension, degrees.
    pub armc_deg: f64,
    /// What it was asked with: the request's own, which its JSON does not
    /// repeat.
    #[serde(skip)]
    pub asked: ProgressionRequest,
}

/// The request a later chart of a birth is founded with: the outer
/// planets too where the birth carries them, so a progressed chart
/// places every body its birth placed
/// (`03-design/western-outer-planets.md`).
fn carrying<'a>(birth: &Document, request: &'a ChartRequest) -> Cow<'a, ChartRequest> {
    if birth.foundation.outer.is_empty() || request.outer_planets() == OuterPlanets::Placed {
        Cow::Borrowed(request)
    } else {
        Cow::Owned(request.clone().with_outer_planets())
    }
}

/// The arc a direction moves every point by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DirectionArc {
    /// The true Sun's motion over the span of sky a progression matches to
    /// the life: the solar arc.
    Solar(Progression),
    /// An arc measure's degrees for the years of life: Naibod's or
    /// Ptolemy's.
    Measure(ArcMeasure),
}

impl Default for DirectionArc {
    fn default() -> DirectionArc {
        DirectionArc::Solar(Progression::SECONDARY)
    }
}

/// One planet of the birth moved by the arc.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DirectedPlanet {
    /// Which planet.
    pub graha: Graha,
    /// Its directed longitude in the birth chart's zodiac, degrees.
    pub longitude_deg: f64,
}

/// A birth's points directed: every planet and both angles moved forward
/// along the ecliptic by one arc.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Directed {
    /// The instant of life asked for.
    pub life: JulianDay<Utc>,
    /// The arc, degrees.
    pub arc_deg: f64,
    /// Every planet the birth chart places, in its order.
    pub planets: Vec<DirectedPlanet>,
    /// The directed ascendant, degrees.
    pub ascendant_deg: f64,
    /// The directed midheaven, degrees.
    pub midheaven_deg: f64,
    /// What it was asked with: the request's own, which its JSON does not
    /// repeat.
    #[serde(skip)]
    pub arc: DirectionArc,
}

/// The seven planets, the Sun to Saturn: the progressed bodies and the
/// radical points a contact search reads unless asked.
const SEVEN: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// What a progressed chart's contacts are searched for: the measure, the
/// progressed planets, the radical points and the aspects.
///
/// ```
/// use teistro::catalogue::Graha;
/// use teistro::gochar::hits::NatalPoint;
/// use teistro::ContactRequest;
///
/// // The progressed Moon's sesquiquadrates and quincunxes to the radical Sun.
/// let asked = ContactRequest::default()
///     .with_grahas([Graha::Moon])
///     .with_points([NatalPoint::Graha { graha: Graha::Sun }])
///     .with_aspects([135, 150]);
/// assert_eq!(asked.aspects, [135, 150]);
/// // Leo's table of aspects unless asked.
/// assert_eq!(ContactRequest::default().aspects, ContactRequest::LEO_ASPECTS);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ContactRequest {
    /// The rate and the year: a day for a tropical year unless asked.
    pub progression: Progression,
    /// The progressed planets: the seven unless asked.
    pub grahas: Vec<Graha>,
    /// The radical points: the seven, the ascendant and the midheaven
    /// unless asked.
    pub points: Vec<NatalPoint>,
    /// The aspects' angles, whole degrees from 0 to 180: Leo's table
    /// unless asked.
    pub aspects: Vec<u16>,
    /// The parallels of declination, read by
    /// [`ChartArea::progressed_parallels`] under this orb; none unless
    /// asked, and that reading takes Leo's 1° then.
    pub parallels: Option<ParallelRequest>,
}

impl ContactRequest {
    /// The aspects of Leo's table (p. 48): the conjunction, and the
    /// columns 30°, 45°, 60°, 90°, 120°, 135°, 150° and 180°.
    pub const LEO_ASPECTS: [u16; 9] = [0, 30, 45, 60, 90, 120, 135, 150, 180];

    /// The same request under another progression.
    #[must_use]
    pub fn with_progression(self, progression: Progression) -> ContactRequest {
        ContactRequest {
            progression,
            ..self
        }
    }

    /// The same request for these progressed planets.
    #[must_use]
    pub fn with_grahas(self, grahas: impl IntoIterator<Item = Graha>) -> ContactRequest {
        ContactRequest {
            grahas: grahas.into_iter().collect(),
            ..self
        }
    }

    /// The same request to these radical points.
    #[must_use]
    pub fn with_points(self, points: impl IntoIterator<Item = NatalPoint>) -> ContactRequest {
        ContactRequest {
            points: points.into_iter().collect(),
            ..self
        }
    }

    /// The same request at these aspects' angles.
    #[must_use]
    pub fn with_aspects(self, angles: impl IntoIterator<Item = u16>) -> ContactRequest {
        ContactRequest {
            aspects: angles.into_iter().collect(),
            ..self
        }
    }

    /// The same request asking the parallels of declination too, under
    /// this orb.
    #[must_use]
    pub fn with_parallels(self, parallels: ParallelRequest) -> ContactRequest {
        ContactRequest {
            parallels: Some(parallels),
            ..self
        }
    }
}

impl ContactRequest {
    /// The hit list that finds these contacts over a window of sky: its
    /// aspects alone, by these planets to these points.
    pub(crate) fn hits_between(&self, from: JulianDay<Utc>, to: JulianDay<Utc>) -> HitRequest {
        HitRequest::between(from, to)
            .with_kinds([HitKind::Aspect])
            .with_grahas(self.grahas.iter().copied())
            .with_points(self.points.iter().copied())
            .with_aspects(self.aspects.iter().copied())
    }
}

impl Default for ContactRequest {
    fn default() -> ContactRequest {
        ContactRequest {
            progression: Progression::SECONDARY,
            grahas: SEVEN.to_vec(),
            points: SEVEN
                .iter()
                .map(|graha| NatalPoint::Graha { graha: *graha })
                .chain([NatalPoint::Lagna, NatalPoint::Midheaven])
                .collect(),
            aspects: ContactRequest::LEO_ASPECTS.to_vec(),
            parallels: None,
        }
    }
}

/// One contact: a progressed planet exactly in aspect to a radical point,
/// and the instant of life it falls due.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProgressedContact {
    /// The instant of life the contact measures to.
    pub life: JulianDay<Utc>,
    /// The instant of sky the aspect is exact at.
    pub sky: JulianDay<Utc>,
    /// The progressed planet.
    pub graha: Graha,
    /// The radical point.
    pub to: NatalPoint,
    /// The aspect's angle, 0 to 180 degrees.
    pub angle: u16,
    /// Which way the progressed planet was moving.
    pub motion: Motion,
}

/// One parallel of declination: a progressed planet within the orb of a
/// radical point's distance from the equator, on either side of it, over a
/// stretch of life. Leo keeps parallels "as a class quite apart from
/// aspects": they "remain in force for a period of several years" and
/// count only "within one degree of completion" (*The Progressed
/// Horoscope*, p. 34).
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProgressedParallel {
    /// The progressed planet.
    pub graha: Graha,
    /// The radical point.
    pub to: NatalPoint,
    /// The instant of life it came within the orb; none when it already
    /// stood within it at the window's start.
    pub enters: Option<JulianDay<Utc>>,
    /// The instant of life it left the orb; none when it still stood
    /// within it at the window's end.
    pub leaves: Option<JulianDay<Utc>>,
    /// The first instant of life the two distances stood closest: the
    /// exact parallel, when the span reaches it.
    pub closest: JulianDay<Utc>,
    /// How far apart the two distances stood then, degrees.
    pub closest_deg: f64,
    /// Whether the two stood on opposite sides of the equator then: the
    /// contra-parallel, which Leo counts a parallel ("it is unnecessary to
    /// add N. or S.", p. 34).
    pub contrary: bool,
}

impl ChartArea<'_> {
    /// The **progressed chart** of a birth at an instant of its life
    /// (`03-design/western-progressions.md`): the chart founded at the
    /// instant of sky the progression matches to `life`, at the request's
    /// place, with the angles turned by the asked method.
    ///
    /// The progressed planets are the chart's own. Its angles are the
    /// quotidian ones, turning a full circle for each day of sky, so the
    /// progressed angles are answered beside it; [`AngleMethod::Quotidian`]
    /// answers the chart's own.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Document, Ephemeris, ProgressionRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (birth, request): (Document, ChartRequest) = todo!();
    /// let at_forty = JulianDay::<Utc>::try_new(birth.foundation.instant.get() + 40.0 * 365.25)?;
    /// let progressed = sdk.chart().progressed(&birth, at_forty, &ProgressionRequest::default(), &request)?;
    /// let midheaven = progressed.angles.midheaven_deg;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A progression [`Progression::check`] refuses, under `progression`;
    /// whatever founding the chart at the instant of sky refuses; or a
    /// birthplace the polar policy refuses the angles at.
    pub fn progressed(
        self,
        birth: &Document,
        life: JulianDay<Utc>,
        asked: &ProgressionRequest,
        request: &ChartRequest,
    ) -> Result<Progressed, Error> {
        self.progressed_of(birth, life, asked, request)
            .map_err(|error| error.under("progression"))
    }

    fn progressed_of(
        self,
        birth: &Document,
        life: JulianDay<Utc>,
        asked: &ProgressionRequest,
        request: &ChartRequest,
    ) -> Result<Progressed, Error> {
        let born = birth.foundation.instant;
        let sky = asked.progression.sky_at(born, life)?;
        let chart = self.reading(sky, &carrying(birth, request))?;
        let natal = self.angles(birth)?;
        let own = self.angles(&chart.value)?;
        let zodiac = &chart.value.foundation.zodiac;
        let meridian = Meridian {
            armc_deg: right_ascension(
                birth.foundation.zodiac.to_tropical(natal.midheaven_deg),
                natal.obliquity_deg,
            ),
            obliquity_deg: natal.obliquity_deg,
        };
        let turned = progressed_armc(
            asked.angles,
            meridian,
            sky.get() - born.get(),
            sun_at(birth, natal)?,
            sun_at(&chart.value, own)?,
        )?;
        let (angles, armc_deg) = match turned {
            None => (
                own,
                right_ascension(zodiac.to_tropical(own.midheaven_deg), own.obliquity_deg),
            ),
            Some(armc_deg) => {
                let built = houses(
                    HouseSystem::WholeSign,
                    &Input {
                        armc_deg,
                        latitude_deg: chart.value.foundation.place.latitude.get(),
                        obliquity_deg: own.obliquity_deg,
                        sun_declination_deg: None,
                        sidereal_offset_deg: zodiac.offset_deg,
                    },
                    self.context().settings().houses.polar_policy,
                )?;
                let angles = ChartAngles {
                    ascendant_deg: zodiac.of_tropical(built.angles.ascendant_deg),
                    midheaven_deg: zodiac.of_tropical(built.angles.midheaven_deg),
                    obliquity_deg: own.obliquity_deg,
                };
                (angles, armc_deg)
            }
        };
        Ok(Progressed {
            life,
            sky,
            chart,
            angles,
            armc_deg,
            asked: *asked,
        })
    }

    /// A birth's points **directed** to an instant of its life: every
    /// planet and both angles moved forward along the ecliptic by one arc,
    /// the solar arc unless asked (`03-design/western-progressions.md`).
    ///
    /// A solar arc founds the chart at the instant of sky its progression
    /// matches to `life`, at the request's place, to read the Sun there; an
    /// arc measure needs no ephemeris.
    ///
    /// # Errors
    ///
    /// A progression or arc measure that is refused, under `direction`, or
    /// whatever founding the chart at the instant of sky refuses.
    pub fn directed(
        self,
        birth: &Document,
        life: JulianDay<Utc>,
        arc: &DirectionArc,
        request: &ChartRequest,
    ) -> Result<Directed, Error> {
        self.directed_of(birth, life, arc, request)
            .map_err(|error| error.under("direction"))
    }

    fn directed_of(
        self,
        birth: &Document,
        life: JulianDay<Utc>,
        arc: &DirectionArc,
        request: &ChartRequest,
    ) -> Result<Directed, Error> {
        let born = birth.foundation.instant;
        let arc_deg = match arc {
            DirectionArc::Solar(progression) => {
                let sky = progression.sky_at(born, life)?;
                let later = self.reading(sky, &carrying(birth, request))?.value;
                // Signed, so a converse arc is negative and an arc across
                // 0° Aries is not a circle short.
                (sun_of(&later)? - sun_of(birth)? + 180.0).rem_euclid(360.0) - 180.0
            }
            DirectionArc::Measure(measure) => {
                measure.degrees((life.get() - born.get()) / teistro_western::TROPICAL_YEAR_DAYS)?
            }
        };
        let moved = |deg: f64| (deg + arc_deg).rem_euclid(360.0);
        let natal = self.angles(birth)?;
        Ok(Directed {
            life,
            arc_deg,
            planets: birth
                .foundation
                .grahas
                .iter()
                .chain(&birth.foundation.outer)
                .map(|at| DirectedPlanet {
                    graha: at.graha,
                    longitude_deg: moved(at.longitude_deg),
                })
                .collect(),
            ascendant_deg: moved(natal.ascendant_deg),
            midheaven_deg: moved(natal.midheaven_deg),
            arc: *arc,
        })
    }

    /// A birth's **progressed contacts** between two instants of its life
    /// (`03-design/western-progressions.md`, Leo's Appendix V): each
    /// exact aspect a progressed planet makes to a radical point, with the
    /// instant of life it falls due, in the order they fall due.
    ///
    /// The search is the transit hit list's over the sky the progression
    /// matches to the window, so its aspects, points and refusals are the
    /// hit list's.
    ///
    /// ```no_run
    /// # use teistro::{ContactRequest, Context, Document, Ephemeris};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (birth, from, to): (Document, JulianDay<Utc>, JulianDay<Utc>) = todo!();
    /// for contact in sdk.chart().progressed_contacts(&birth, from, to, &ContactRequest::default())? {
    ///     println!("{:?} {}° {:?} at {}", contact.graha, contact.angle, contact.to, contact.life.get());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A window that does not run forward, a progression
    /// [`Progression::check`] refuses, and whatever the hit list refuses,
    /// each named under `contacts`.
    pub fn progressed_contacts(
        self,
        birth: &Document,
        from: JulianDay<Utc>,
        to: JulianDay<Utc>,
        asked: &ContactRequest,
    ) -> Result<Vec<ProgressedContact>, Error> {
        self.contacts_of(birth, from, to, asked)
            .map_err(|error| error.under("contacts"))
    }

    fn contacts_of(
        self,
        birth: &Document,
        from: JulianDay<Utc>,
        to: JulianDay<Utc>,
        asked: &ContactRequest,
    ) -> Result<Vec<ProgressedContact>, Error> {
        forward(from, to)?;
        let born = birth.foundation.instant;
        let progression = asked.progression;
        // Leo's rule reaches the last minutes before each noon of sky twice,
        // so an instant of life may be measured from two instants of sky, a
        // day of life apart at most. Nothing of sky before the instant a
        // day earlier in life measures into the window: the search starts
        // there and the window's own bounds keep what falls inside.
        let window = asked.hits_between(
            progression.sky_at(born, from.plus_days(-1.0)?)?,
            progression.sky_at(born, to)?,
        );
        let mut contacts = Vec::new();
        for hit in self.hits(birth, &window)?.value {
            let HitEvent::Aspect {
                to: point,
                angle,
                motion,
                ..
            } = hit.event
            else {
                continue;
            };
            let life = progression.life_at(born, hit.instant)?;
            if (from.get()..=to.get()).contains(&life.get()) {
                contacts.push(ProgressedContact {
                    life,
                    sky: hit.instant,
                    graha: hit.graha,
                    to: point,
                    angle,
                    motion,
                });
            }
        }
        // Stable, so contacts due together keep the hit list's order.
        contacts.sort_by(|a, b| a.life.get().total_cmp(&b.life.get()));
        Ok(contacts)
    }

    /// The **parallels of declination** a birth's progressed planets make
    /// to its radical points over a window of life (`03-design/
    /// western-progressions.md`): each the stretch of life a progressed
    /// planet stands within the orb of a radical point's distance from the
    /// equator, on either side of it, with the instant they stand closest.
    /// The planets, the points and the progression are the request's, and
    /// the orb its `parallels`' (Leo's 1° when it names none); its aspects
    /// play no part. Ordered by the closest instant.
    ///
    /// The radical points' declinations are the birth's own
    /// ([`ChartArea::declinations`]); the progressed planets' are read
    /// where the progression puts each instant of life, by the same rule.
    ///
    /// ```no_run
    /// # use teistro::{ContactRequest, Context, Document, Ephemeris, ParallelRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (birth, from, to): (Document, JulianDay<Utc>, JulianDay<Utc>) = todo!();
    /// let asked = ContactRequest::default().with_parallels(ParallelRequest::default());
    /// for row in sdk.chart().progressed_parallels(&birth, from, to, &asked)? {
    ///     println!("{:?} to {:?}: closest {:.2}° at {}", row.graha, row.to, row.closest_deg, row.closest.get());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A window that does not run forward, what the hit list refuses of
    /// the planets and points, what [`ParallelRequest::check`] refuses of
    /// the orb, and an instant the ephemeris cannot place, named under
    /// `contacts`.
    pub fn progressed_parallels(
        self,
        birth: &Document,
        from: JulianDay<Utc>,
        to: JulianDay<Utc>,
        asked: &ContactRequest,
    ) -> Result<Vec<ProgressedParallel>, Error> {
        self.parallels_of(birth, from, to, asked)
            .map_err(|error| error.under("contacts"))
    }

    fn parallels_of(
        self,
        birth: &Document,
        from: JulianDay<Utc>,
        to: JulianDay<Utc>,
        asked: &ContactRequest,
    ) -> Result<Vec<ProgressedParallel>, Error> {
        forward(from, to)?;
        asked.hits_between(from, to).check()?;
        let orb = asked.parallels.unwrap_or_default();
        orb.check().map_err(|why| why.under("parallels"))?;
        let foundation = &birth.foundation;
        let born = foundation.instant;
        let progression = asked.progression;
        let radical = self.declinations(birth)?;
        let fixed: Vec<(NatalPoint, f64)> = asked
            .points
            .iter()
            .map(|&point| radical_declination(&radical, foundation, point).map(|d| (point, d)))
            .collect::<Result<_, Error>>()?;
        // Fine enough that no body's declination moves half the orb
        // between two samples: no span inside the orb is missed, and one
        // through it between two is found by its crossing.
        let sky = |life: JulianDay<Utc>| progression.sky_at(born, life);
        let sky_days = sky(to)?.get() - sky(from)?.get();
        let life_days = to.get() - from.get();
        let step_sky =
            (orb.orb_deg / (2.0 * FASTEST_DECLINATION_DEG_PER_DAY)).max(MIN_SKY_STEP_DAYS);
        let steps = sample_count(sky_days.abs() / step_sky)?;
        let lives: Vec<JulianDay<Utc>> = (0..=steps)
            .map(|k| from.plus_days(life_days * f64::from(k) / f64::from(steps)))
            .collect::<Result<_, _>>()?;
        let skies: Vec<JulianDay<Utc>> = lives
            .iter()
            .map(|&life| sky(life))
            .collect::<Result<_, _>>()?;
        let mut rows = self.founding(UtcOffset::UTC, |founder| {
            let read = |instants: &[JulianDay<Utc>], grahas: &[Graha]| {
                founder.declinations_at(instants, &foundation.place, &foundation.zodiac, grahas)
            };
            let grid = read(&skies, &asked.grahas)?;
            let mut rows = Vec::new();
            for (column, &graha) in asked.grahas.iter().enumerate() {
                let samples: Vec<(f64, f64)> = lives
                    .iter()
                    .zip(&grid)
                    .map(|(life, row)| (life.get(), row.get(column).copied().unwrap_or(f64::NAN)))
                    .collect();
                for &(point, fixed_deg) in &fixed {
                    let search = SpanSearch {
                        fixed_deg,
                        orb_deg: orb.orb_deg,
                        tolerance: LIFE_TOLERANCE_DAYS,
                    };
                    let spans = parallel_spans(&samples, search, |life| {
                        let sky = sky(JulianDay::try_new(life)?)?;
                        read(&[sky], &[graha])?
                            .first()
                            .and_then(|row| row.first().copied())
                            .ok_or_else(|| Error::internal("a declination was asked and none read"))
                    })?;
                    for span in spans {
                        rows.push(ProgressedParallel {
                            graha,
                            to: point,
                            enters: span.enters.map(JulianDay::try_new).transpose()?,
                            leaves: span.leaves.map(JulianDay::try_new).transpose()?,
                            closest: JulianDay::try_new(span.closest)?,
                            closest_deg: span.closest_deg,
                            contrary: span.contrary,
                        });
                    }
                }
            }
            Ok(rows)
        })?;
        // Stable, so spans closest together keep the planets' order.
        rows.sort_by(|a, b| a.closest.get().total_cmp(&b.closest.get()));
        Ok(rows)
    }
}

/// The fastest any planet's declination moves, degrees a day of sky: the
/// Moon's, at most 15.4° a day of longitude turned by the obliquity and
/// her latitude, with room to spare.
const FASTEST_DECLINATION_DEG_PER_DAY: f64 = 8.0;

/// The finest a parallels' search samples the sky, days: a narrower orb
/// is found by its crossings between samples.
const MIN_SKY_STEP_DAYS: f64 = 1e-3;

/// How close in life a parallel's edges and closest instant are found,
/// days: about nine seconds.
const LIFE_TOLERANCE_DAYS: f64 = 1e-4;

/// The most samples a parallels' search reads.
const MAX_SAMPLES: u32 = 1_000_000;

/// How many steps cover `steps` (a fraction), at least one, refused
/// past [`MAX_SAMPLES`].
fn sample_count(steps: f64) -> Result<u32, Error> {
    let whole = steps.ceil().max(1.0);
    if whole.is_finite() && whole <= f64::from(MAX_SAMPLES) {
        // Whole, positive and at most `MAX_SAMPLES`, so it converts.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "checked above"
        )]
        return Ok(whole as u32);
    }
    Err(Error::invalid_arg(format!(
        "a parallels' window would read {whole} instants of sky, more than {MAX_SAMPLES}"
    ))
    .with_field("to")
    .with_hint("ask a shorter window of life"))
}

/// Refuses a window of life that does not run forward.
fn forward(from: JulianDay<Utc>, to: JulianDay<Utc>) -> Result<(), Error> {
    if to.get() > from.get() {
        return Ok(());
    }
    Err(Error::invalid_arg(format!(
        "a contact window must run forward, and {} is not after {}",
        to.get(),
        from.get()
    ))
    .with_field("to"))
}

/// A radical point's declination: an angle's as the birth's declinations
/// read it, a graha's from its own place, the nodes' included.
fn radical_declination(
    radical: &Declinations,
    foundation: &ChartFoundation,
    point: NatalPoint,
) -> Result<f64, Error> {
    match point {
        NatalPoint::Lagna => Ok(radical.lagna_deg),
        NatalPoint::Midheaven => Ok(radical.midheaven_deg),
        NatalPoint::Graha { graha } => foundation
            .grahas
            .iter()
            .chain(&foundation.outer)
            .find(|at| at.graha == graha)
            .map(|at| declination_deg(at.tropical_deg, at.latitude_deg, radical.obliquity_deg))
            .ok_or_else(|| {
                Error::invalid_arg(format!("the birth chart places no {}", graha.key()))
                    .with_field("points")
                    .with_hint("found the birth with the outer planets placed")
            }),
    }
}

/// The right ascension of an ecliptic point, degrees.
pub(crate) fn right_ascension(longitude_deg: f64, obliquity_deg: f64) -> f64 {
    ecliptic_to_equatorial(
        Spherical {
            lon_deg: longitude_deg,
            lat_deg: 0.0,
        },
        obliquity_deg,
    )
    .lon_deg
}

/// The Sun's tropical longitude in a chart.
fn sun_of(chart: &Document) -> Result<f64, Error> {
    chart
        .foundation
        .grahas
        .iter()
        .find(|at| at.graha == Graha::Sun)
        .map(|at| at.tropical_deg)
        .ok_or_else(|| Error::invalid_arg("the chart does not place the Sun").with_field("chart"))
}

/// The Sun in a chart, with the obliquity of its angles.
fn sun_at(chart: &Document, angles: ChartAngles) -> Result<SunAt, Error> {
    Ok(SunAt {
        longitude_deg: sun_of(chart)?,
        obliquity_deg: angles.obliquity_deg,
    })
}

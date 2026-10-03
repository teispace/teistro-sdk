//! Progressions and directions through the façade
//! (`03-design/western-progressions.md`).
//!
//! `teistro-western` holds the measures; this founds the chart at the
//! instant of sky a progression answers, turns the angles by the asked
//! method, and moves a birth's points by a direction's arc.

use teistro_astro::houses::{Input, houses};
use teistro_astro::sky::{Spherical, ecliptic_to_equatorial};
use teistro_chart::foundation::ChartAngles;
use teistro_core::catalogue::{Graha, HouseSystem};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_serial::Document;
use teistro_western::{
    AngleMethod, ArcMeasure, Meridian, Progression, SunAt, YearMeasure, progressed_armc,
};

use crate::area::ChartArea;
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
#[derive(Clone, Debug, PartialEq)]
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
    /// What it was asked with.
    pub asked: ProgressionRequest,
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectedPlanet {
    /// Which planet.
    pub graha: Graha,
    /// Its directed longitude in the birth chart's zodiac, degrees.
    pub longitude_deg: f64,
}

/// A birth's points directed: every planet and both angles moved forward
/// along the ecliptic by one arc.
#[derive(Clone, Debug, PartialEq)]
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
    /// What it was asked with.
    pub arc: DirectionArc,
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
        let chart = self.reading(sky, request)?;
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
                let later = self.reading(sky, request)?.value;
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
}

/// The right ascension of an ecliptic point, degrees.
fn right_ascension(longitude_deg: f64, obliquity_deg: f64) -> f64 {
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

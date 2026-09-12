//! The foundation itself: one moment at one place, with everything above
//! it needs and nothing it can compute for itself.
//!
//! The three modules beneath this one each answer a question that is easy
//! to get wrong — which day the moment belongs to ([`crate::day`]), which
//! zodiac it is measured in ([`crate::zodiac`]), and where a graha falls
//! between the bhavas ([`crate::bhava`]). This one asks each of them
//! once, asks the provider once, and hands the answers on stamped.
//!
//! The rule it is built on: **the foundation holds what is needed to
//! compute, never what is computed.** A field belongs here when more than
//! one module above needs it and no module above can produce it. That is
//! why the vargas, the arudhas, the upagrahas and the dashas' bhayat and
//! bhabhoga are not here even though the recording engine put them in its
//! own foundation — each depends on a module that depends on this one.

use serde::{Deserialize, Serialize};
use teistro_astro::completion::{Completed, Completion};
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::houses::{ChartFrame, houses_at};
use teistro_astro::precession::PrecessionModel;
use teistro_astro::scale::tt_of;
use teistro_calendar::CalendarSystem;
use teistro_calendar::solar::SolarModel;
use teistro_core::catalogue::{ChartKind, Graha, HouseSystem};
use teistro_core::envelope::Version;
use teistro_core::envelope::{CALCULATION_VERSION, Envelope, Provenance, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Tt, Ut1, Utc};
use teistro_core::settings::{GhatiReckoning, HoraReckoning, Node, Resolved, Settings};
use teistro_core::time::LocalClock;
use teistro_port_ephemeris::columns::CellStatus;
use teistro_port_ephemeris::{Body, EphemerisProvider, PositionRequest, TimeScale};
use teistro_time::ghati::{self, GhatiPala};
use teistro_time::hora::{self, Hora};

/// What the birth timing is, taken from the day's arc and the profile's
/// reckonings.
///
/// Bhayat and bhabhoga are **not** here, and the recording engine's own
/// foundation is misleading about them: they are the duration of the
/// Moon's traversal of its nakshatra and the elapsed part of it, which
/// the corpus settles to within 0.39 minutes over all 55 charts. They
/// belong to `dasha`, the only module that needs them.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BirthTiming {
    /// The ishtakaal: how far into the day the moment is, in ghati, pala
    /// and vipala, measured from the sunrise that opened **the chart's
    /// day** — which for a birth before dawn is the previous morning's.
    pub ishtakaal: GhatiPala,
    /// How that count was reckoned.
    pub ghati_reckoning: GhatiReckoning,
    /// The planetary hour holding the moment, with its lord.
    pub hora: Hora,
    /// How the horas were reckoned.
    pub hora_reckoning: HoraReckoning,
}

use crate::bhava::{Bhavas, Chalit, Placement};
use crate::day::{ChartDay, chart_day};
use crate::zodiac::ChartZodiac;

/// Where one graha is, in the chart's own zodiac.
///
/// Both readings are kept and neither is recomputed: a module that wanted
/// the other one and converted it itself would use a different ayanamsha
/// than the chart did, the day someone changed the setting.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GrahaPosition {
    /// Which graha.
    pub graha: Graha,
    /// Longitude in the chart's zodiac, degrees.
    pub longitude_deg: f64,
    /// The same longitude in the tropical zodiac of date, degrees.
    pub tropical_deg: f64,
    /// Ecliptic latitude, degrees.
    pub latitude_deg: f64,
    /// Distance, astronomical units; zero for a point that has none.
    pub distance_au: f64,
    /// Motion in longitude, degrees a day; negative when retrograde.
    pub speed_deg_per_day: f64,
    /// Where it falls under the chart's chalit.
    pub placement: Placement,
    /// Where it falls under the chart's placement system, which is the
    /// one that answers "in the seventh" for most of the tradition.
    pub house: Placement,
}

impl GrahaPosition {
    /// Whether the graha is moving backwards through the zodiac.
    #[must_use]
    pub fn is_retrograde(&self) -> bool {
        self.speed_deg_per_day < 0.0
    }

    /// The sign it occupies, 0 for Aries.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a normalised longitude over thirty is 0 to 11"
    )]
    pub fn sign_index(&self) -> u8 {
        (self.longitude_deg.rem_euclid(360.0) / 30.0) as u8
    }
}

/// One moment at one place, founded.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChartFoundation {
    /// The instant.
    pub instant: JulianDay<Utc>,
    /// The place.
    pub place: Place,
    /// What kind of chart this is.
    pub kind: ChartKind,
    /// The day it belongs to, which is not always its civil date.
    pub day: ChartDay,
    /// The zodiac it is measured in.
    pub zodiac: ChartZodiac,
    /// The lagna at the instant, in the chart's zodiac, degrees.
    pub lagna_deg: f64,
    /// The lagna at the sunrise that opened the day: what the arudhas,
    /// the birth timing's proportions and the Lagna dashas measure from,
    /// and for a birth before dawn the previous morning's.
    pub day_lagna_deg: f64,
    /// The bhavas a graha is placed in for "which house is it in".
    pub houses: Bhavas,
    /// The bhavas of the chart's chalit, which is a different question
    /// and often a different answer.
    pub chalit: Bhavas,
    /// When in its day the moment falls, and which planetary hour holds
    /// it.
    pub timing: BirthTiming,
    /// The grahas, in the catalogue's order.
    pub grahas: Vec<GrahaPosition>,
    /// What the provider and the completion did, for the stamp.
    pub steps: Vec<String>,
}

impl ChartFoundation {
    /// One graha, by name.
    #[must_use]
    pub fn graha(&self, graha: Graha) -> Option<&GrahaPosition> {
        self.grahas.iter().find(|position| position.graha == graha)
    }

    /// The lagna's sign, 0 for Aries.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a normalised longitude over thirty is 0 to 11"
    )]
    pub fn lagna_sign_index(&self) -> u8 {
        (self.lagna_deg.rem_euclid(360.0) / 30.0) as u8
    }
}

/// What a foundation was asked for, hashed into its stamp so that two
/// results of the same question can be told apart from two of different
/// ones.
#[derive(serde::Serialize)]
struct Input {
    jd_utc: f64,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
    kind: &'static str,
}

/// The same, for a batch.
#[derive(serde::Serialize)]
struct BatchInput {
    jds_utc: Vec<f64>,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
    kind: &'static str,
}

/// The bodies a chart asks the provider for, and the graha each answers
/// for.
///
/// Rahu is whichever node `frame.node` names; Ketu is not asked for,
/// because no body is Ketu — it is Rahu's opposite point in the same
/// frame (`05-testing/01-golden-vectors.md`, entry 6).
#[must_use]
pub fn bodies_of(settings: &Settings) -> Vec<Body> {
    vec![
        Body::Sun,
        Body::Moon,
        Body::Mars,
        Body::Mercury,
        Body::Jupiter,
        Body::Venus,
        Body::Saturn,
        if settings.frame.node == Node::True {
            Body::TrueNode
        } else {
            Body::MeanNode
        },
    ]
}

/// Founds charts: one construction, many moments.
///
/// Batch is the primary shape (principle 5), so the service is built once
/// with the provider and the settings and then asked for as many
/// foundations as are wanted; [`Founder::found_one`] is the convenience
/// over it and not the other way round.
pub struct Founder<'a, P: EphemerisProvider + ?Sized> {
    provider: &'a P,
    resolved: &'a Resolved,
    model: &'a dyn SolarModel,
    calendar: &'a dyn CalendarSystem,
    clock: &'a dyn LocalClock,
    precession: PrecessionModel,
    delta_t: DeltaTModel,
}

impl<P: EphemerisProvider + ?Sized> core::fmt::Debug for Founder<'_, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Founder")
            .field("model", &self.model.describe())
            .field("precession", &self.precession)
            .finish_non_exhaustive()
    }
}

impl<'a, P: EphemerisProvider + ?Sized> Founder<'a, P> {
    /// A founder over a provider, a solar model for the day's arcs, a
    /// calendar and a clock.
    pub const fn new(
        provider: &'a P,
        resolved: &'a Resolved,
        model: &'a dyn SolarModel,
        calendar: &'a dyn CalendarSystem,
        clock: &'a dyn LocalClock,
        precession: PrecessionModel,
        delta_t: DeltaTModel,
    ) -> Founder<'a, P> {
        Founder {
            provider,
            resolved,
            model,
            calendar,
            clock,
            precession,
            delta_t,
        }
    }

    /// The resolved settings every part reads.
    const fn settings(&self) -> &Settings {
        &self.resolved.settings
    }

    /// Founds one chart.
    ///
    /// # Errors
    ///
    /// The day's own refusals (a polar day under `UNDEFINED`, a date the
    /// calendar lacks), an ayanamsha the catalogue cannot evaluate at the
    /// instant, a provider that cannot answer for the instant or a body,
    /// or a house system that cannot be built and has no substitute under
    /// the profile's polar policy.
    pub fn found_one(
        &self,
        instant: JulianDay<Utc>,
        place: &Place,
        kind: ChartKind,
    ) -> Result<Envelope<ChartFoundation>, Error> {
        let chart = self.value(instant, place, kind)?;
        let provenance = self.provenance(&chart);
        Ok(Envelope::sealing(chart, provenance))
    }

    /// The foundation without its stamp, which the batch shares.
    fn value(
        &self,
        instant: JulianDay<Utc>,
        place: &Place,
        kind: ChartKind,
    ) -> Result<ChartFoundation, Error> {
        let ut1 = JulianDay::<Ut1>::literal(instant.get());
        let (tt, _) = tt_of(ut1, self.delta_t)?;
        let zodiac = ChartZodiac::of(self.settings(), tt, self.precession, self.delta_t)?;
        let day = chart_day(
            self.model,
            self.calendar,
            self.clock,
            place,
            instant,
            self.settings().day.polar_day_policy,
        )?;

        // Two divisions, each read the way its own system is read. The
        // chalit is built from the cusps of the system it *reads*, which
        // for Sripati is Porphyry's.
        let placement = self.settings().houses.placement_system;
        let (cusps, built) = self.cusps(placement, ut1, tt, place, &zodiac)?;
        let houses = Bhavas::of(Chalit::of(built), &cusps);
        let wanted = Chalit::of(self.settings().houses.chalit_system);
        let (cusps, built) = self.cusps(wanted.source, ut1, tt, place, &zodiac)?;
        let chalit = Bhavas::of(
            Chalit {
                // A substituted division is still reported as itself.
                method: if built == wanted.source {
                    wanted.method
                } else {
                    built
                },
                source: built,
                reading: if built == wanted.source {
                    wanted.reading
                } else {
                    Chalit::of(built).reading
                },
            },
            &cusps,
        );

        let lagna_deg = self.lagna(ut1, tt, place, &zodiac)?;
        let day_ut1 = JulianDay::<Ut1>::literal(day.lagna_sunrise().get());
        let (day_tt, _) = tt_of(day_ut1, self.delta_t)?;
        let day_lagna_deg = self.lagna(day_ut1, day_tt, place, &zodiac)?;

        let timing = self.timing(&day, instant)?;
        let (grahas, steps) = self.grahas(ut1, place, &zodiac, &houses, &chalit)?;
        Ok(ChartFoundation {
            instant,
            place: *place,
            kind,
            day,
            zodiac,
            lagna_deg,
            day_lagna_deg,
            houses,
            chalit,
            timing,
            grahas,
            steps,
        })
    }

    /// Where in its day the moment falls, and which hora holds it.
    ///
    /// Both are measured over the arc of the day the chart belongs to,
    /// so a birth before dawn is counted from the previous morning's
    /// sunrise, which is what the corpus records.
    fn timing(&self, day: &ChartDay, instant: JulianDay<Utc>) -> Result<BirthTiming, Error> {
        let ghati_reckoning = self.settings().day.ghati_reckoning;
        let hora_reckoning = self.settings().day.hora_reckoning;
        Ok(BirthTiming {
            ishtakaal: ghati::ghati_pala(&day.day, instant, ghati_reckoning.try_into()?)?,
            ghati_reckoning,
            hora: hora::hora_at(&day.day, instant, hora_reckoning.try_into()?)?,
            hora_reckoning,
        })
    }

    /// What produced a foundation, stamped as every result of the SDK is.
    ///
    /// The provider's own stamp comes from the completion that answered,
    /// so a foundation says which ephemeris placed its grahas, in which
    /// frame, and which steps the SDK completed itself.
    fn provenance(&self, chart: &ChartFoundation) -> Provenance {
        let mut provenance = Provenance::new(
            Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version::new(0, 0, 0)),
            CALCULATION_VERSION,
            teistro_core::catalogue::SCHEMA_VERSION,
            self.resolved.profile.as_str(),
            self.settings().hash(),
            content_hash(&Input {
                jd_utc: chart.instant.get(),
                latitude_deg: chart.place.latitude.get(),
                longitude_deg: chart.place.longitude.get(),
                altitude_m: chart.place.altitude.get(),
                kind: chart.kind.key(),
            }),
        );
        provenance.provider = self
            .provider
            .capabilities()
            .identity
            .stamp(chart.zodiac.request, chart.steps.clone());
        provenance.time.delta_t_model = self.delta_t.key().to_string();
        provenance.time.leap_table = teistro_time::leap::version().to_string();
        provenance
    }

    /// Founds many charts at one place, sharing the settings and the
    /// solar model.
    ///
    /// # Errors
    ///
    /// As [`Founder::found_one`], on the first instant that refuses.
    pub fn found(
        &self,
        instants: &[JulianDay<Utc>],
        place: &Place,
        kind: ChartKind,
    ) -> Result<Envelope<Vec<ChartFoundation>>, Error> {
        let charts: Vec<ChartFoundation> = instants
            .iter()
            .map(|instant| self.value(*instant, place, kind))
            .collect::<Result<_, _>>()?;
        // One stamp over the batch: the settings, the provider and the
        // steps are the same for every chart in it, and the input hash is
        // of the whole request rather than of one instant.
        let mut provenance = charts.first().map_or_else(
            || self.empty_provenance(place, kind),
            |first| self.provenance(first),
        );
        provenance.input_hash = content_hash(&BatchInput {
            jds_utc: instants.iter().map(|jd| jd.get()).collect(),
            latitude_deg: place.latitude.get(),
            longitude_deg: place.longitude.get(),
            altitude_m: place.altitude.get(),
            kind: kind.key(),
        });
        Ok(Envelope::sealing(charts, provenance))
    }

    /// The stamp of a batch that founded nothing, which still says under
    /// which settings nothing was founded.
    fn empty_provenance(&self, place: &Place, kind: ChartKind) -> Provenance {
        Provenance::new(
            Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version::new(0, 0, 0)),
            CALCULATION_VERSION,
            teistro_core::catalogue::SCHEMA_VERSION,
            self.resolved.profile.as_str(),
            self.settings().hash(),
            content_hash(&BatchInput {
                jds_utc: Vec::new(),
                latitude_deg: place.latitude.get(),
                longitude_deg: place.longitude.get(),
                altitude_m: place.altitude.get(),
                kind: kind.key(),
            }),
        )
    }

    /// A house system's cusps in the chart's zodiac, and the system that
    /// actually produced them — which inside the polar circle may be the
    /// substitute the profile's policy names.
    fn cusps(
        &self,
        system: HouseSystem,
        ut1: JulianDay<Ut1>,
        tt: JulianDay<Tt>,
        place: &Place,
        zodiac: &ChartZodiac,
    ) -> Result<([f64; 12], HouseSystem), Error> {
        let frame = ChartFrame {
            sidereal_offset_deg: zodiac.offset_deg,
            sun_declination_deg: None,
        };
        let built = houses_at(
            system,
            ut1,
            tt,
            place,
            &frame,
            self.settings().houses.polar_policy,
        )?;
        let mut cusps = [0.0_f64; 12];
        for (out, cusp) in cusps.iter_mut().zip(built.cusps.iter()) {
            *out = zodiac.of_tropical(*cusp);
        }
        Ok((cusps, built.system))
    }

    /// The ascendant at an instant, in the chart's zodiac.
    fn lagna(
        &self,
        ut1: JulianDay<Ut1>,
        tt: JulianDay<Tt>,
        place: &Place,
        zodiac: &ChartZodiac,
    ) -> Result<f64, Error> {
        let frame = ChartFrame {
            sidereal_offset_deg: zodiac.offset_deg,
            sun_declination_deg: None,
        };
        let built = houses_at(
            HouseSystem::WholeSign,
            ut1,
            tt,
            place,
            &frame,
            self.settings().houses.polar_policy,
        )?;
        Ok(zodiac.of_tropical(built.angles.ascendant_deg))
    }

    /// The grahas, placed.
    fn grahas(
        &self,
        ut1: JulianDay<Ut1>,
        place: &Place,
        zodiac: &ChartZodiac,
        houses: &Bhavas,
        chalit: &Bhavas,
    ) -> Result<(Vec<GrahaPosition>, Vec<String>), Error> {
        let bodies = bodies_of(self.settings());
        let jds = [ut1.get()];
        let mut request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, zodiac.request);
        if zodiac.needs_observer() {
            request.observer = Some(*place);
        }
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        )
        .with_precession(self.precession);
        let completed: Completed = completion.positions(&request)?;

        let mut grahas = Vec::with_capacity(bodies.len() + 1);
        for (index, body) in bodies.iter().enumerate() {
            let cell = completed.columns.at(0, index).ok_or_else(|| {
                Error::internal(format!("the grid has no cell for {}", body.key()))
            })?;
            if cell.status != CellStatus::Ok {
                return Err(Error::unsupported(format!(
                    "the provider could not place {}: {:?}",
                    body.key(),
                    cell.status
                ))
                .with_field("bodies"));
            }
            let graha = body
                .graha()
                .ok_or_else(|| Error::internal(format!("{} is not a graha", body.key())))?;
            grahas.push(Self::position(
                graha,
                cell.lon,
                cell.lat,
                cell.dist,
                cell.lon_speed,
                zodiac,
                houses,
                chalit,
            ));
        }

        // Ketu: Rahu's opposite point in the same frame, with the same
        // speed and no distance of its own (entry 6).
        if let Some(rahu) = grahas.iter().find(|g| g.graha == Graha::Rahu).copied() {
            grahas.push(Self::position(
                Graha::Ketu,
                rahu.tropical_deg + 180.0,
                -rahu.latitude_deg,
                0.0,
                rahu.speed_deg_per_day,
                zodiac,
                houses,
                chalit,
            ));
        }
        Ok((grahas, completed.step_keys()))
    }

    /// One graha's position, placed in both divisions.
    #[expect(
        clippy::too_many_arguments,
        reason = "one value assembled from its parts"
    )]
    fn position(
        graha: Graha,
        tropical_deg: f64,
        latitude_deg: f64,
        distance_au: f64,
        speed_deg_per_day: f64,
        zodiac: &ChartZodiac,
        houses: &Bhavas,
        chalit: &Bhavas,
    ) -> GrahaPosition {
        let tropical_deg = tropical_deg.rem_euclid(360.0);
        let longitude_deg = zodiac.of_tropical(tropical_deg);
        GrahaPosition {
            graha,
            longitude_deg,
            tropical_deg,
            latitude_deg,
            distance_au,
            speed_deg_per_day,
            placement: chalit.place(longitude_deg),
            house: houses.place(longitude_deg),
        }
    }
}

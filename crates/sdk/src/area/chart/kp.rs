//! KP's reading of a chart (`03-design/kp.md`): the cusps and their lords,
//! the significators and the ruling planets.

use teistro_chart::foundation::{cusps_of, cusps_raising};
use teistro_core::angle::Nas;
use teistro_core::catalogue::HouseSystem;
use teistro_core::error::Error;
use teistro_core::settings::DayLordDay;
use teistro_houses::system::override_of;
use teistro_kp::{
    KpChart, KpNumber, KpReading, Position, RulingPlanets, RulingRules, Significators,
};
use teistro_serial::Document;

use super::{ChartArea, weekday_lord};
use crate::kp_request::KpRequest;

impl ChartArea<'_> {
    /// A chart read as KP (`03-design/kp.md`): every cusp and every graha
    /// the chart carries with its sign, star, sub and sub-sub lords, and
    /// each graha in the house whose cusp it follows.
    ///
    /// The cusps are the house system `houses.module_overrides.kp` names,
    /// Placidus's under `kp-default` and when nothing is named, as the
    /// Readers take them; inside the polar circle the polar policy decides,
    /// and [`KpChart::system`] says which system built them. It needs **no
    /// ephemeris**: the cusps are rebuilt from the chart's instant, place
    /// and zodiac, so a stored chart answers it.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, KpRequest, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// # use teistro::catalogue::HouseSystem;
    /// let sdk = Context::builder()
    ///     .ephemeris([Ephemeris::Builtin])
    ///     .profile("kp-default")
    ///     .build()?;
    /// let chennai = Place::new(Latitude::try_new(13.08)?, Longitude::try_new(80.27)?, Altitude::try_new(6.0)?);
    /// let chart = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_451_545.0), &ChartRequest::at(chennai, UtcOffset::literal(5, 30, 0)))?
    ///     .value;
    /// let kp = sdk.chart().kp(&chart, &KpRequest::new())?;
    /// assert_eq!(kp.system, HouseSystem::Placidus);
    /// let lagna = kp.cusp(1).expect("twelve cusps");
    /// println!("lagna sub lord {:?}", lagna.lords.sub.lord);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A chart founded under an ayanamsha [`KpRequest`] does not take,
    /// named `frame.ayanamsha` (C157); a chart whose angles were its
    /// provider's; and a house system the polar policy refuses at the
    /// chart's latitude.
    pub fn kp(self, chart: &Document, request: &KpRequest) -> Result<KpChart, Error> {
        self.kp_cusps(chart, request, request.number())
    }

    /// A KP chart with the chart's own cusps, or those raising a horary
    /// number's start.
    fn kp_cusps(
        self,
        chart: &Document,
        request: &KpRequest,
        number: Option<KpNumber>,
    ) -> Result<KpChart, Error> {
        let foundation = &chart.foundation;
        request.check(&foundation.zodiac)?;
        let settings = self.context.settings();
        let system = override_of(settings, teistro_kp::MODULE).unwrap_or(HouseSystem::Placidus);
        let (delta_t, policy) = (self.context.delta_t(), settings.houses.polar_policy);
        let (cusps, built) = match number {
            None => cusps_of(foundation, system, delta_t, policy)?,
            Some(number) => {
                let lagna = number.start().to_degrees();
                cusps_raising(foundation, system, lagna, delta_t, policy)?
            }
        };
        let mut longitudes = [Nas::ZERO; 12];
        for (longitude, cusp) in longitudes.iter_mut().zip(cusps) {
            *longitude = Nas::try_from_degrees(cusp)?;
        }
        // The number's start is the lagna exactly, not the search's
        // nanodegree from it.
        if let (Some(number), Some(lagna)) = (number, longitudes.first_mut()) {
            *lagna = number.start();
        }
        let planets = foundation
            .grahas
            .iter()
            .map(|graha| {
                Ok(
                    Position::new(graha.graha, Nas::try_from_degrees(graha.longitude_deg)?)
                        .retrograde(graha.is_retrograde()),
                )
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(KpChart::new(built, longitudes, planets))
    }

    /// A **KP horary** chart from the querent's number (KP Reader I; crux
    /// C156): the lagna at the start of the number's arc, the other cusps
    /// those that ascendant has at the place, from the meridian that raises
    /// it at the moment's obliquity and ayanamsha, and the planets where
    /// `moment` placed them — the chart founded for the moment of judgement
    /// at the place the question is judged.
    ///
    /// The cusps are the system [`ChartArea::kp`] reads; the ayanamsha is
    /// checked as there.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, KpNumber, KpRequest, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).profile("kp-default").build()?;
    /// let place = Place::new(Latitude::try_new(13.08)?, Longitude::try_new(80.27)?, Altitude::try_new(6.0)?);
    /// let now = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_461_000.25), &ChartRequest::at(place, UtcOffset::literal(5, 30, 0)))?
    ///     .value;
    /// // The querent says 74: the lagna is Cancer 14°53′20″.
    /// let horary = sdk.chart().kp_horary(&now, KpNumber::new(74)?, &KpRequest::new())?;
    /// assert_eq!(horary.cusp(1).map(|cusp| cusp.longitude), Some(KpNumber::new(74)?.start()));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::kp`]; and a latitude where no meridian raises the
    /// number's start, named `place.latitude`.
    pub fn kp_horary(
        self,
        moment: &Document,
        number: KpNumber,
        request: &KpRequest,
    ) -> Result<KpChart, Error> {
        self.kp(moment, &request.for_number(number))
    }

    /// A KP chart's **significators** (`03-design/kp.md` §1): each house's
    /// four levels in the Reader's order, the planets conjoined with or
    /// aspected by them, its intercepted signs, and whose results each node
    /// gives, a node's own aspects read from `aspect.node_aspects`.
    ///
    /// Needs nothing but the chart, from [`ChartArea::kp`].
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, KpRequest, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).profile("kp-default").build()?;
    /// let place = Place::new(Latitude::try_new(13.08)?, Longitude::try_new(80.27)?, Altitude::try_new(6.0)?);
    /// let chart = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_451_545.0), &ChartRequest::at(place, UtcOffset::literal(5, 30, 0)))?
    ///     .value;
    /// let kp = sdk.chart().kp(&chart, &KpRequest::new())?;
    /// let significators = sdk.chart().kp_significators(&kp);
    /// // The 7th house's significators, strongest first.
    /// println!("{:?}", significators.house(7).map(|house| house.in_order()));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn kp_significators(self, kp: &KpChart) -> Significators {
        Significators::of(kp, self.context.settings().aspect.node_aspects)
    }

    /// The KP **ruling planets** at the moment a chart was cast for, the
    /// moment of judgement (KP Reader VI): the lagna's and the Moon's star
    /// and sign lords and the day's lord, the nodes standing for them, and
    /// the rulers a retrograde planet rejects, under the settings' `kp`
    /// group (cruxes C150 to C153).
    ///
    /// The day is the chart's own, sunrise to sunrise, unless
    /// `kp.day_lord_day` is `CIVIL`, which takes the weekday of the civil
    /// date on the clock the request names ([`KpRequest::on_clock`]).
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, KpRequest, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).profile("kp-default").build()?;
    /// let place = Place::new(Latitude::try_new(13.08)?, Longitude::try_new(80.27)?, Altitude::try_new(6.0)?);
    /// let now = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_461_000.25), &ChartRequest::at(place, UtcOffset::literal(5, 30, 0)))?
    ///     .value;
    /// let ruling = sdk.chart().kp_ruling(&now, &KpRequest::new())?;
    /// println!("ruling planets {:?}", ruling.accepted());
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::kp`]; and under `CIVIL` a request without a clock,
    /// named `kp.day_lord_day`.
    pub fn kp_ruling(self, chart: &Document, request: &KpRequest) -> Result<RulingPlanets, Error> {
        let moment = self.kp_cusps(chart, request, None)?;
        self.ruling_of(chart, &moment, request)
    }

    /// A chart read as KP whole (`03-design/kp.md` §5): the chart — the
    /// horary one when the request names a number — its significators,
    /// and the ruling planets at its moment, which are the moment's own
    /// whatever the number.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, KpNumber, KpRequest, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).profile("kp-default").build()?;
    /// let place = Place::new(Latitude::try_new(13.08)?, Longitude::try_new(80.27)?, Altitude::try_new(6.0)?);
    /// let now = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_461_000.25), &ChartRequest::at(place, UtcOffset::literal(5, 30, 0)))?
    ///     .value;
    /// let reading = sdk.chart().kp_reading(&now, &KpRequest::new().for_number(KpNumber::new(74)?))?;
    /// println!("{:?} rule", reading.ruling.accepted());
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::kp`] and [`ChartArea::kp_ruling`].
    pub fn kp_reading(self, chart: &Document, request: &KpRequest) -> Result<KpReading, Error> {
        let moment = self.kp_cusps(chart, request, None)?;
        let ruling = self.ruling_of(chart, &moment, request)?;
        let read = match request.number() {
            None => moment,
            Some(number) => self.kp_cusps(chart, request, Some(number))?,
        };
        let significators = self.kp_significators(&read);
        Ok(KpReading::new(read, significators, ruling))
    }

    /// The ruling planets of a moment already read as KP.
    fn ruling_of(
        self,
        chart: &Document,
        moment: &KpChart,
        request: &KpRequest,
    ) -> Result<RulingPlanets, Error> {
        let settings = self.context.settings();
        let foundation = &chart.foundation;
        let day_lord = match settings.kp.day_lord_day {
            DayLordDay::Civil => {
                let clock = request.clock().ok_or_else(|| {
                    Error::invalid_arg(
                        "the civil day lord is the weekday on a clock, and the request names none",
                    )
                    .with_field("kp.day_lord_day")
                    .with_hint(
                        "ask KpRequest::new().on_clock(offset), or read the day from sunrise",
                    )
                })?;
                weekday_lord(foundation.instant.get() + clock.days())?
            }
            DayLordDay::Sunrise | _ => foundation.day.day.vara.attributes().lord,
        };
        Ok(RulingPlanets::of(
            moment,
            day_lord,
            RulingRules::of(settings),
        ))
    }
}

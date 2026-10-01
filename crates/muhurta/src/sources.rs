//! The search's [`Sources`] over an ephemeris provider: the almanac for
//! the days, the chart founder for the lagna, the frame completion for the
//! grahas, and the visibility reckoner for asta.
//!
//! One chart zodiac serves the whole search, taken at the instant the
//! caller names. The ayanamsha moves about 0.14″ a day, so at the far end
//! of a three-month search the lagna is read about 13″ off, which moves a
//! window's edge by about a second of time and changes no clause inside
//! it. A caller who wants the zodiac at each day searches a day at a
//! time.

use teistro_astro::Completion;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::events::{Longitudes, Search};
use teistro_astro::precession::PrecessionModel;
use teistro_astro::scale::tt_of;
use teistro_astro::visibility::Heliacal;
use teistro_calendar::{CalendarDate, CalendarSystem};
use teistro_chart::foundation::{Founder, bodies_of};
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Place, Ut1, Utc};
use teistro_core::settings::Settings;
use teistro_core::time::LocalClock;
use teistro_panchanga::limb::{Sidereal, Zodiac, signs_within};
use teistro_panchanga::{Almanac, Panchanga};
use teistro_port_ephemeris::{Body, Centre, EphemerisProvider, Frame, Lattice, Quantity};
use teistro_time::local_day::local_midnight;

use crate::instant::Sky;
use crate::search::{Sources, same_day};
use crate::season::{Blackout, BlackoutKind, asta_over, blackouts};

/// The slow grahas whose ingresses a day may hold, beside the Sun's and
/// the Moon's, which the almanac day already has: Mars to Saturn, and the
/// node (Ketu crosses when Rahu does).
const SLOW: [usize; 6] = [2, 3, 4, 5, 6, 7];

/// A navamsa, degrees: the Moon's quarter of a star.
const NAVAMSA_DEG: f64 = 30.0 / 9.0;

/// The search's sources over a provider.
pub struct ProviderSources<'a, P: EphemerisProvider + ?Sized> {
    almanac: &'a Almanac<'a, P>,
    founder: &'a Founder<'a, P>,
    completion: &'a Completion<'a, P>,
    heliacal: &'a Heliacal<'a, P>,
    calendar: &'a dyn CalendarSystem,
    clock: &'a dyn LocalClock,
    place: Place,
    bodies: Vec<Body>,
    chart: ChartZodiac,
    /// The frame the grahas are asked for: the chart's tropical request,
    /// or the provider's own sidereal frame where it defines the zodiac
    /// ([`ChartZodiac::searched`]).
    frame: Frame,
    zodiac: Zodiac,
    days: &'a [Panchanga],
}

impl<P: EphemerisProvider + ?Sized> core::fmt::Debug for ProviderSources<'_, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ProviderSources")
            .field("place", &self.place)
            .finish_non_exhaustive()
    }
}

/// The collaborators a [`ProviderSources`] is built over.
pub struct Over<'a, P: EphemerisProvider + ?Sized> {
    /// The almanac of the days.
    pub almanac: &'a Almanac<'a, P>,
    /// The founder the lagna is asked of.
    pub founder: &'a Founder<'a, P>,
    /// The frame completion the grahas are read from.
    pub completion: &'a Completion<'a, P>,
    /// The visibility reckoner asta is read from.
    pub heliacal: &'a Heliacal<'a, P>,
    /// The calendar the dates are in.
    pub calendar: &'a dyn CalendarSystem,
    /// The clock a civil day is local to.
    pub clock: &'a dyn LocalClock,
    /// The settings the almanac and the founder were built with.
    pub settings: &'a Settings,
    /// The precession model.
    pub precession: PrecessionModel,
    /// The Delta T model.
    pub delta_t: DeltaTModel,
}

impl<P: EphemerisProvider + ?Sized> core::fmt::Debug for Over<'_, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Over")
            .field("almanac", self.almanac)
            .field("founder", self.founder)
            .finish_non_exhaustive()
    }
}

impl<'a, P: EphemerisProvider + ?Sized> ProviderSources<'a, P> {
    /// Sources at a place, with the chart zodiac taken at `reference`.
    ///
    /// # Errors
    ///
    /// An instant outside the Delta T model, or an ayanamsha the catalogue
    /// cannot evaluate there.
    pub fn new(
        over: &Over<'a, P>,
        place: Place,
        reference: JulianDay<Utc>,
    ) -> Result<ProviderSources<'a, P>, Error> {
        let (tt, _) = tt_of(JulianDay::<Ut1>::literal(reference.get()), over.delta_t)?;
        let searched = ChartZodiac::searched(
            over.completion,
            over.settings,
            tt,
            (over.precession, over.delta_t),
        )?;
        Ok(ProviderSources {
            almanac: over.almanac,
            founder: over.founder,
            completion: over.completion,
            heliacal: over.heliacal,
            calendar: over.calendar,
            clock: over.clock,
            place,
            bodies: bodies_of(over.settings),
            chart: searched.chart,
            frame: searched.frame,
            zodiac: searched.zodiac,
            days: &[],
        })
    }

    /// The same sources, answering a day from `days` when it is there —
    /// consecutive days, as `Almanac::between` founds a range — and from
    /// the almanac otherwise: so a caller that founded the range for its
    /// own use does not have the search found each day a second time.
    ///
    /// The days must be founded with the same almanac at the same place;
    /// a day of `days` is taken as the one the almanac would give.
    #[must_use]
    pub const fn with_days(mut self, days: &'a [Panchanga]) -> ProviderSources<'a, P> {
        self.days = days;
        self
    }

    /// The day of `days` on `date`, when `days` holds it.
    fn founded(&self, date: &CalendarDate) -> Result<Option<&Panchanga>, Error> {
        let Some(first) = self.days.first() else {
            return Ok(None);
        };
        let offset = self
            .calendar
            .fixed_of(&first.day.date)?
            .days_until(self.calendar.fixed_of(date)?);
        Ok(usize::try_from(offset)
            .ok()
            .and_then(|index| self.days.get(index))
            .filter(|day| same_day(&day.day.date, date)))
    }

    /// The grahas' source: the chart's own frame, at the place.
    fn chart_sky(&self) -> impl Longitudes + '_ {
        self.completion
            .longitudes(self.frame)
            .with_observer(self.place)
    }
}

impl<P: EphemerisProvider + ?Sized> Sources for ProviderSources<'_, P> {
    fn civil_day(&self, date: &CalendarDate) -> Result<Interval, Error> {
        let fixed = self.calendar.fixed_of(date)?;
        Interval::new(
            local_midnight(self.clock, fixed)?,
            local_midnight(self.clock, fixed.plus_days(1))?,
        )
    }

    fn next_date(&self, date: &CalendarDate) -> Result<CalendarDate, Error> {
        self.calendar
            .date_of(self.calendar.fixed_of(date)?.plus_days(1))
    }

    fn day(&self, date: &CalendarDate) -> Result<Panchanga, Error> {
        match self.founded(date)? {
            Some(day) => Ok(day.clone()),
            None => Ok(self.almanac.day(date, &self.place)?.value),
        }
    }

    fn season(&self, range: Interval, kinds: &[BlackoutKind]) -> Result<Vec<Blackout>, Error> {
        // An almanac's season is geocentric wherever it is read.
        let geocentric = self.completion.longitudes(Frame {
            centre: Centre::Geocentric,
            ..self.frame
        });
        let mut found: Vec<Blackout> = blackouts(&geocentric, self.zodiac, range)?
            .into_iter()
            .filter(|b| kinds.contains(&b.kind))
            .collect();
        for (kind, body) in [
            (BlackoutKind::GuruAsta, Body::Jupiter),
            (BlackoutKind::ShukraAsta, Body::Venus),
        ] {
            if kinds.contains(&kind) {
                found.extend(asta_over(self.heliacal, body, range)?);
            }
        }
        found.sort_by(|a, b| a.at.from.get().total_cmp(&b.at.from.get()));
        Ok(found)
    }

    fn lagna_at(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        self.founder.ascendant_at(at, &self.place, &self.chart)
    }

    fn sky_at(&self, at: JulianDay<Utc>) -> Result<Sky, Error> {
        let source = self.chart_sky();
        let sidereal = Sidereal::over(&source, self.zodiac);
        let t = JulianDay::<Ut1>::literal(at.get());
        let mut grahas = [0.0; 9];
        let mut speeds = [0.0; 9];
        for ((slot, speed), body) in grahas.iter_mut().zip(&mut speeds).zip(&self.bodies) {
            (*slot, *speed) = sidereal.longitude_and_speed(*body, t)?;
        }
        // Ketu is Rahu's opposite, and moves with it.
        grahas[8] = (grahas[7] + 180.0).rem_euclid(360.0);
        speeds[8] = speeds[7];
        Ok(Sky {
            lagna_deg: self.lagna_at(at)?,
            grahas,
            speeds,
        })
    }

    fn ingresses(&self, within: Interval) -> Result<Vec<JulianDay<Utc>>, Error> {
        let source = self.chart_sky();
        let mut cuts = Vec::new();
        for body in SLOW.iter().filter_map(|i| self.bodies.get(*i)) {
            for span in signs_within(&source, *body, within, self.zodiac, Some(1.0))? {
                cuts.extend([span.whole.from, span.whole.to]);
            }
        }
        let sidereal = Sidereal::over(&source, self.zodiac);
        let lattice = Lattice {
            origin_deg: 0.0,
            step_deg: NAVAMSA_DEG,
        };
        let moon = Search::new(&sidereal, Quantity::Longitude(Body::Moon), lattice).between(
            JulianDay::<Ut1>::literal(within.from.get()),
            JulianDay::<Ut1>::literal(within.to.get()),
        )?;
        cuts.extend(moon.iter().map(|e| JulianDay::literal(e.instant.get())));
        Ok(cuts)
    }
}

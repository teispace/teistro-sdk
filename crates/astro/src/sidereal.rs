//! Reading a provider's tropical longitudes in a sidereal zodiac.
//!
//! The provider is asked for tropical positions and shifted here, because
//! a chart holds **one** ayanamsha and the provider's may not be it
//! (`03-design/chart-foundation.md` §4) — and because the basis matters:
//! a chart applies the nutated ayanamsha where a frame's own
//! `Zodiac::Sidereal` applies the mean one, 18.46 arcseconds apart.
//!
//! This began as the panchanga limb search's private adapter. It is here
//! because the annual chart needs the same thing for the same reason: a
//! solar return read through the mean ayanamsha lands 18 arcseconds from
//! where the chart would read it, which is seven minutes of the Sun's
//! time and about two degrees of lagna
//! (`03-design/annual-chart-measured.md`). A second copy of that shift
//! would be a second thing to get wrong.

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_core::settings::{AyanamshaBasis, AyanamshaChoice};
use teistro_port_ephemeris::Body;

use crate::ayanamsha::Basis;
use crate::delta_t::DeltaTModel;
use crate::events::Longitudes;
use crate::precession::PrecessionModel;
use crate::scale::tt_of;

/// A source of longitudes in the chart's own zodiac.
///
/// The provider is asked for tropical positions and shifted here, because
/// a chart holds one ayanamsha and the provider's may not be it
/// (`03-design/chart-foundation.md` §4). The shift is evaluated at every
/// instant rather than once for the window: the ayanamsha moves about
/// 0.00014 degrees a day, which over a four-day search is two seconds of
/// the Moon's time — wider than the tolerance the corpus declares.
#[derive(Debug)]
pub struct Sidereal<'a, S: Longitudes + ?Sized> {
    /// The source the provider answers in.
    pub tropical: &'a S,
    /// The ayanamsha to subtract, or `None` for a tropical reading.
    pub ayanamsha: Option<AyanamshaChoice>,
    /// Whether that ayanamsha carries the nutation.
    pub basis: Basis,
    /// The precession model behind it.
    pub precession: PrecessionModel,
    /// The Delta T model.
    pub delta_t: DeltaTModel,
}

impl<S: Longitudes + ?Sized> Sidereal<'_, S> {
    /// The ayanamsha at an instant, degrees, and its rate, degrees a day.
    fn offset(&self, ut1: JulianDay<Ut1>) -> Result<(f64, f64), Error> {
        let Some(choice) = self.ayanamsha else {
            return Ok((0.0, 0.0));
        };
        let (tt, _) = tt_of(ut1, self.delta_t)?;
        let value =
            crate::ayanamsha::value_deg(&choice, tt, self.basis, self.precession, self.delta_t)?;
        // A day on either side gives the rate by a central difference;
        // the ayanamsha is a smooth function of the instant and the rate
        // only corrects a speed nothing classifies with.
        let step = 1.0;
        let ahead = crate::ayanamsha::value_deg(
            &choice,
            JulianDay::literal(tt.get() + step),
            self.basis,
            self.precession,
            self.delta_t,
        )?;
        let behind = crate::ayanamsha::value_deg(
            &choice,
            JulianDay::literal(tt.get() - step),
            self.basis,
            self.precession,
            self.delta_t,
        )?;
        Ok((value, (ahead - behind) / (2.0 * step)))
    }
}

impl<S: Longitudes + ?Sized> Sidereal<'_, S> {
    /// Shifts a grid of tropical readings into the zodiac, instant by
    /// instant.
    ///
    /// The ayanamsha is read **at each instant** rather than once for
    /// the window (the module's third rule), so the shift is a walk
    /// however the readings arrived; what the grid saved is the
    /// ephemeris calls under them, which is the expensive half.
    fn shift<T: Copy>(
        &self,
        ut1: &[JulianDay<Ut1>],
        out: &mut [T],
        apply: impl Fn(T, f64, f64) -> T,
    ) -> Result<(), Error> {
        for (value, at) in out.iter_mut().zip(ut1) {
            let (offset, rate) = self.offset(*at)?;
            *value = apply(*value, offset, rate);
        }
        Ok(())
    }
}

impl<S: Longitudes + ?Sized> Longitudes for Sidereal<'_, S> {
    fn longitude_and_speed(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<(f64, f64), Error> {
        let (longitude, speed) = self.tropical.longitude_and_speed(body, ut1)?;
        let (offset, rate) = self.offset(ut1)?;
        Ok(((longitude - offset).rem_euclid(360.0), speed - rate))
    }

    fn longitude_and_speed_pair(
        &self,
        bodies: [Body; 2],
        ut1: JulianDay<Ut1>,
    ) -> Result<[(f64, f64); 2], Error> {
        let pair = self.tropical.longitude_and_speed_pair(bodies, ut1)?;
        let (offset, rate) = self.offset(ut1)?;
        Ok(pair.map(|(longitude, speed)| ((longitude - offset).rem_euclid(360.0), speed - rate)))
    }

    fn longitudes_and_speeds(
        &self,
        body: Body,
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<(f64, f64)>,
    ) -> Result<(), Error> {
        self.tropical.longitudes_and_speeds(body, ut1, out)?;
        self.shift(ut1, out, |value, offset, rate| {
            ((value.0 - offset).rem_euclid(360.0), value.1 - rate)
        })
    }

    fn longitudes_and_speeds_pair(
        &self,
        bodies: [Body; 2],
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<[(f64, f64); 2]>,
    ) -> Result<(), Error> {
        self.tropical.longitudes_and_speeds_pair(bodies, ut1, out)?;
        self.shift(ut1, out, |pair, offset, rate| {
            pair.map(|(longitude, speed)| ((longitude - offset).rem_euclid(360.0), speed - rate))
        })
    }

    fn describe(&self) -> String {
        match self.ayanamsha {
            Some(choice) => format!("{} shifted by {choice:?}", self.tropical.describe()),
            None => self.tropical.describe(),
        }
    }
}

/// A zodiac to read longitudes in: which ayanamsha, on which basis, under
/// which models.
#[derive(Clone, Copy, Debug)]
pub struct Zodiac {
    /// The ayanamsha the day's limbs are measured from, or `None` for a
    /// tropical reading.
    pub ayanamsha: Option<AyanamshaChoice>,
    /// Whether that ayanamsha carries the nutation.
    pub basis: Basis,
    /// The precession model behind it.
    pub precession: PrecessionModel,
    /// The Delta T model.
    pub delta_t: DeltaTModel,
}

impl Zodiac {
    /// The zodiac a chart's own positions are read in: its ayanamsha under
    /// the profile's basis.
    ///
    /// The basis is the whole point of this type. A chart applies the
    /// **nutated** ayanamsha where a plain sidereal frame applies the mean
    /// one, and the two are 18.46 arcseconds apart — which for the Sun is
    /// about seven minutes of time, and for a solar return about two
    /// degrees of lagna (`03-design/annual-chart-measured.md`).
    #[must_use]
    pub const fn of(
        ayanamsha: Option<AyanamshaChoice>,
        basis: AyanamshaBasis,
        precession: PrecessionModel,
        delta_t: DeltaTModel,
    ) -> Zodiac {
        Zodiac {
            ayanamsha,
            basis: match basis {
                AyanamshaBasis::True => Basis::True,
                _ => Basis::Mean,
            },
            precession,
            delta_t,
        }
    }
}

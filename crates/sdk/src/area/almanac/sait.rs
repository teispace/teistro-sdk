//! Each observance's sait (`03-design/festival-sait.md`): its day founded
//! whole, and the lagna's navamsas found over each window the rule tries.

use teistro_astro::Completion;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::scale::tt_of;
use teistro_chart::foundation::Founder;
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::angle::Nas;
use teistro_core::catalogue::Calendar;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::time::UtcOffset;
use teistro_panchanga::almanac::Almanac;
use teistro_panchanga::festival::{Navamsas, Observances, PublishedStatus};
use teistro_panchanga::rising::{lagna_cuts, navamsa_of};
use teistro_port_ephemeris::EphemerisProvider;

use super::AlmanacArea;
use crate::area::{drik_sun, system_of};
use crate::ephemeris::no_ephemeris;
use crate::festival_request::FestivalRequest;

impl AlmanacArea<'_> {
    /// Each observance of a rule `request` carries a sait for gains it,
    /// elected on its day at `place`; a day a published set withdrew
    /// gains none, as no rite is kept on it.
    pub(super) fn with_saits(
        self,
        found: &mut Observances,
        request: &FestivalRequest,
        calendar: Calendar,
        (place, offset): (&Place, UtcOffset),
    ) -> Result<(), Error> {
        if request.saits().is_empty() {
            return Ok(());
        }
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let resolved = self.context.resolved();
        let (precession, delta_t) = (PrecessionModel::default(), self.context.delta_t());
        let system = system_of(calendar)?;
        let model = drik_sun(self.context, provider);
        let almanac = Almanac::new(
            provider, resolved, &model, system, &offset, precession, delta_t,
        );
        let settings = &resolved.settings;
        let completion = Completion::new(provider, settings.provider.overrides, delta_t);
        let founder = Founder::new(
            provider, resolved, &model, system, &offset, precession, delta_t,
        );
        for observance in &mut found.observances {
            let Some(rule) = request
                .saits()
                .iter()
                .find(|sait| sait.rule == observance.rule)
            else {
                continue;
            };
            if observance
                .published
                .as_ref()
                .is_some_and(|p| p.status == PublishedStatus::Withdrawn)
            {
                continue;
            }
            let day = almanac.day(&observance.day, place)?.value;
            // The grahas stand where the day's chart places them.
            let (tt, _) = tt_of(JulianDay::<Ut1>::literal(day.day.sunrise.get()), delta_t)?;
            let zodiac =
                ChartZodiac::searched(&completion, settings, tt, (precession, delta_t))?.chart;
            // The day's lagna table's own ascendant, in the zodiac at its
            // sunrise: a day's drift of the ayanamsha moves a navamsa's
            // edge by well under a second.
            let lagna_at = founder.lagna_table_ascendant(day.day.sunrise, place)?;
            let free_of = rule.rising.as_ref().map_or(&[][..], |r| &r.free_of[..]);
            let sait = rule.elect(&day, observance.tithi, |within| {
                Ok(Navamsas {
                    first: navamsa_of(lagna_at(within.from)?),
                    entered: lagna_cuts(within, &lagna_at)?
                        .into_iter()
                        .map(|cut| (cut.at, cut.navamsa))
                        .collect(),
                    held: held_in(&founder, within, place, &zodiac, free_of)?,
                })
            })?;
            observance.sait = Some(sait);
        }
        Ok(())
    }
}

/// The signs `grahas` stand in at the window's middle: none move a sign
/// in a few hours but the Moon, which a lagna's purity does not name.
fn held_in<P: EphemerisProvider + ?Sized>(
    founder: &Founder<'_, P>,
    within: Interval,
    place: &Place,
    zodiac: &ChartZodiac,
    grahas: &[Graha],
) -> Result<Vec<Rashi>, Error> {
    if grahas.is_empty() {
        return Ok(Vec::new());
    }
    let middle = JulianDay::literal(f64::midpoint(within.from.get(), within.to.get()));
    founder
        .longitudes_in(middle, place, zodiac, grahas)?
        .into_iter()
        .map(|longitude| Ok(Nas::try_from_degrees(longitude.rem_euclid(360.0))?.sign()))
        .collect()
}

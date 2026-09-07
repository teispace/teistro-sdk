//! The zodiac a chart is computed in, and the frame it asks the provider
//! for.
//!
//! A chart has **one** ayanamsha value, and the grahas and the cusps are
//! both measured from it. That is not a convenience: a placement compares
//! a graha's longitude with a bhava's boundary, so if the provider
//! computed the graha sidereally with *its* ayanamsha while the SDK
//! computed the cusps with *its own*, the comparison would be between two
//! different zodiacs and every placement near a boundary would be a coin
//! toss.
//!
//! So the chart never asks a provider for a sidereal frame. It asks for
//! the tropical one, computes the ayanamsha once through
//! `astro::ayanamsha` — which handles a custom ayanamsha as well as a
//! catalogued one, where the port's `Zodiac` cannot — and shifts. The
//! corpus says the recording engine does the same: over its 55 charts and
//! 550 bodies, `sidereal = tropical - ayanamsha` to the last bit, with one
//! value per chart.

use teistro_astro::ayanamsha::{self, Basis};
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Tt};
use teistro_core::settings::{
    AyanamshaBasis, AyanamshaChoice, Centre, Positions, Settings, Zodiac as ZodiacKnob,
};
use teistro_port_ephemeris::{Centre as FrameCentre, Corrections, Frame, Zodiac};

/// The port's centre a setting names.
///
/// The port has four — it can be asked for a heliocentric or barycentric
/// position — and the settings have the two a chart is ever computed
/// from. A knob core adds before this crate learns it is geocentric,
/// which is the root's own value.
const fn centre_of(centre: Centre) -> FrameCentre {
    match centre {
        Centre::Topocentric => FrameCentre::Topocentric,
        _ => FrameCentre::Geocentric,
    }
}

/// The zodiac of one chart: what to ask the provider for, and how far to
/// shift what comes back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartZodiac {
    /// The frame to ask the provider for. Always tropical, so that the
    /// grahas and the cusps are shifted by the same value.
    pub request: Frame,
    /// Degrees to subtract from a tropical longitude to reach the
    /// chart's zodiac. Zero for a tropical chart.
    pub offset_deg: f64,
    /// The ayanamsha the offset came from, for the stamp; `None` for a
    /// tropical chart.
    pub ayanamsha: Option<AyanamshaChoice>,
}

impl ChartZodiac {
    /// The zodiac a profile's frame settings describe at an instant.
    ///
    /// # Errors
    ///
    /// A sidereal zodiac whose ayanamsha the catalogue cannot evaluate at
    /// this instant, which `astro::ayanamsha` reports with the member and
    /// the span it holds.
    pub fn of(
        settings: &Settings,
        at: JulianDay<Tt>,
        precession: PrecessionModel,
        delta_t: DeltaTModel,
    ) -> Result<ChartZodiac, Error> {
        let frame = &settings.frame;
        let request = Frame {
            centre: centre_of(frame.centre),
            // The tropical zodiac of date, whatever the chart's own is.
            zodiac: Zodiac::Tropical,
            corrections: match frame.positions {
                Positions::Apparent => Corrections::APPARENT,
                // A true position is the geometric one with the equinox
                // still of date; the nutation stays because the equinox
                // does.
                _ => Corrections {
                    light_time: false,
                    aberration: false,
                    deflection: false,
                    nutation: true,
                },
            },
            ..Frame::CANONICAL
        };
        let (offset_deg, ayanamsha) = if frame.zodiac == ZodiacKnob::Tropical {
            (0.0, None)
        } else {
            // A basis core adds before this crate learns it is the mean
            // value, which is what the root sets.
            let basis = if frame.ayanamsha_basis == AyanamshaBasis::True {
                Basis::True
            } else {
                Basis::Mean
            };
            let value = ayanamsha::value_deg(&frame.ayanamsha, at, basis, precession, delta_t)?;
            (value, Some(frame.ayanamsha))
        };
        Ok(ChartZodiac {
            request,
            offset_deg,
            ayanamsha,
        })
    }

    /// A tropical longitude in the chart's zodiac.
    ///
    /// ```
    /// # use teistro_chart::zodiac::ChartZodiac;
    /// # use teistro_port_ephemeris::Frame;
    /// let sidereal = ChartZodiac {
    ///     request: Frame::CANONICAL,
    ///     offset_deg: 23.7,
    ///     ayanamsha: None,
    /// };
    /// assert!((sidereal.of_tropical(23.7) - 0.0).abs() < 1e-12);
    /// // And past the start of the circle rather than into a negative.
    /// assert!((sidereal.of_tropical(10.0) - 346.3).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn of_tropical(&self, tropical_deg: f64) -> f64 {
        (tropical_deg - self.offset_deg).rem_euclid(360.0)
    }

    /// A longitude of the chart's zodiac back in the tropical one.
    #[must_use]
    pub fn to_tropical(&self, chart_deg: f64) -> f64 {
        (chart_deg + self.offset_deg).rem_euclid(360.0)
    }

    /// Whether the chart is sidereal, which is what a stamp reports.
    #[must_use]
    pub const fn is_sidereal(&self) -> bool {
        self.ayanamsha.is_some()
    }

    /// Whether the provider is being asked for a topocentric position,
    /// which it needs an observer for.
    #[must_use]
    pub fn needs_observer(&self) -> bool {
        self.request.centre == FrameCentre::Topocentric
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::ChartZodiac;
    use teistro_astro::delta_t::DeltaTModel;
    use teistro_astro::precession::PrecessionModel;
    use teistro_core::catalogue::Ayanamsha;
    use teistro_core::quantity::{JulianDay, Tt};
    use teistro_core::settings::{
        AyanamshaChoice, Centre, Positions, Profile, SettingsPatch, Zodiac as ZodiacKnob, root,
    };
    use teistro_port_ephemeris::{Centre as FrameCentre, Corrections, Zodiac};

    fn at() -> JulianDay<Tt> {
        JulianDay::<Tt>::literal(2_451_545.0)
    }

    fn of(settings: &teistro_core::settings::Settings) -> ChartZodiac {
        ChartZodiac::of(
            settings,
            at(),
            PrecessionModel::Iau2006,
            DeltaTModel::TableThenModel,
        )
        .unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn a_sidereal_chart_asks_for_tropical_and_shifts_it_itself() {
        // The default profile is sidereal, Lahiri.
        let settings = root();
        let zodiac = of(&settings);
        assert_eq!(
            zodiac.request.zodiac,
            Zodiac::Tropical,
            "the provider is never asked for a sidereal frame"
        );
        assert!(zodiac.is_sidereal());
        // Lahiri at J2000 is a little under 24 degrees.
        assert!(
            (23.0..24.5).contains(&zodiac.offset_deg),
            "{}",
            zodiac.offset_deg
        );
        // And the shift is the whole of the difference.
        let tropical = 100.0;
        assert!((zodiac.of_tropical(tropical) - (tropical - zodiac.offset_deg)).abs() < 1e-12);
        assert!((zodiac.to_tropical(zodiac.of_tropical(tropical)) - tropical).abs() < 1e-12);
    }

    #[test]
    fn a_tropical_chart_shifts_by_nothing() {
        let mut patch = SettingsPatch::default();
        patch.frame.zodiac = Some(ZodiacKnob::Tropical);
        let settings = root().patched(&patch);
        let zodiac = of(&settings);
        assert!(!zodiac.is_sidereal());
        assert!(zodiac.offset_deg.abs() < f64::EPSILON);
        assert!((zodiac.of_tropical(123.456) - 123.456).abs() < 1e-12);
    }

    #[test]
    fn a_custom_ayanamsha_is_evaluated_where_the_ports_frame_could_not_carry_it() {
        // The port's `Zodiac::Sidereal` takes a catalogued member and has
        // nowhere to put a custom epoch and value. Computing the offset
        // here is what lets a custom ayanamsha work at all.
        let mut patch = SettingsPatch::default();
        patch.frame.ayanamsha = Some(AyanamshaChoice::Custom {
            epoch_jd_tt: 2_451_545.0,
            value_deg: 24.0,
            rate_deg_per_year: 0.0,
        });
        let settings = root().patched(&patch);
        let zodiac = of(&settings);
        assert!(zodiac.is_sidereal());
        // At its own epoch a custom ayanamsha is its own value.
        assert!(
            (zodiac.offset_deg - 24.0).abs() < 1e-9,
            "{}",
            zodiac.offset_deg
        );
    }

    #[test]
    fn the_frame_carries_the_centre_and_the_corrections_the_profile_asks_for() {
        let mut patch = SettingsPatch::default();
        patch.frame.centre = Some(Centre::Topocentric);
        let settings = root().patched(&patch);
        let zodiac = of(&settings);
        assert_eq!(zodiac.request.centre, FrameCentre::Topocentric);
        assert!(zodiac.needs_observer(), "a topocentric frame needs a place");
        assert_eq!(zodiac.request.corrections, Corrections::APPARENT);

        let mut patch = SettingsPatch::default();
        patch.frame.positions = Some(Positions::True);
        let settings = root().patched(&patch);
        let zodiac = of(&settings);
        assert!(
            !zodiac.request.corrections.aberration,
            "a true position has none"
        );
        assert!(
            zodiac.request.corrections.nutation,
            "the equinox is still of date"
        );
        assert!(!zodiac.needs_observer());
    }

    #[test]
    fn the_default_profile_is_sidereal_lahiri_and_geocentric() {
        // ADR-0024: the default is the texts as read, which is geocentric.
        let profile = Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
            .unwrap_or_else(|| panic!("the default profile"));
        let resolved = profile
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        let zodiac = of(&resolved.settings);
        assert_eq!(zodiac.request.centre, FrameCentre::Geocentric);
        assert!(zodiac.is_sidereal());
        assert_eq!(
            resolved.settings.frame.ayanamsha,
            AyanamshaChoice::Catalogued {
                id: Ayanamsha::Lahiri
            }
        );
    }
}

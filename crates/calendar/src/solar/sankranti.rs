//! The sankranti finder: when the Sun next enters a sign, from a model,
//! through the shared solver.

use teistro_astro::solve::{Caps, SolveError, next_crossing};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Utc};

use crate::solar::SolarModel;

/// The tolerance a sankranti is found to, in days: a tenth of a second,
/// far inside any civil-day decision and a hundredth of the classical
/// model's own precision.
pub const TOLERANCE_DAYS: f64 = 1e-6;

/// The Sun's mean sidereal rate, degrees per day, the solver's step guide.
pub const MEAN_SOLAR_RATE_DEG_PER_DAY: f64 = 0.985_6;

/// A found sankranti.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sankranti {
    /// The sign entered, 0 for Mesha to 11 for Meena.
    pub sign: u8,
    /// The instant.
    pub instant: JulianDay<Utc>,
    /// How many times the model was asked.
    pub evaluations: u32,
}

/// The first entry of the Sun into a sign at or after an instant.
///
/// # Errors
///
/// A sign outside 0 to 11, a model that cannot answer, or a search that
/// does not converge (`NOT_CONVERGED`, naming the sign and the instant).
pub fn find_sankranti(
    model: &dyn SolarModel,
    sign: u8,
    from: JulianDay<Utc>,
) -> Result<Sankranti, Error> {
    if sign > 11 {
        return Err(
            Error::invalid_arg(format!("a sign index is 0 to 11, not {sign}")).with_field("sign"),
        );
    }
    let target = f64::from(sign) * 30.0;
    let crossing = next_crossing(
        |jd| model.sidereal_sun_deg(jd),
        target,
        from.get(),
        MEAN_SOLAR_RATE_DEG_PER_DAY,
        TOLERANCE_DAYS,
        Caps::DEFAULT,
    )
    .map_err(|error| match error {
        SolveError::Evaluation(inner) => inner,
        other => Error::new(
            Status::NotConverged,
            format!("the sankranti of sign {sign} after {from} was not found: {other}"),
        ),
    })?;
    Ok(Sankranti {
        sign,
        instant: JulianDay::try_new(crossing.instant)?,
        evaluations: crossing.evaluations,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_siddhanta::SuryaSiddhanta;

    use super::*;
    use crate::CalendarSystem;
    use crate::fixed::FixedDay;
    use crate::gregorian::{Gregorian, gregorian_from_fixed};

    #[test]
    fn the_mesha_sankranti_of_2024_falls_on_13_april() {
        let text = SuryaSiddhanta::text();
        let from = Gregorian
            .to_fixed_ymd(2024, 3, 1)
            .unwrap()
            .jd_at_midnight()
            .unwrap();
        let mesha = find_sankranti(&text, 0, from).unwrap();
        let (day, fraction) = FixedDay::from_jd(mesha.instant);
        assert_eq!(gregorian_from_fixed(day), (2024, 4, 13), "{fraction}");
        assert!(mesha.evaluations < 60, "{}", mesha.evaluations);
        let sun = text.sidereal_sun_deg(mesha.instant.get()).unwrap();
        assert!(sun < 1e-4 || sun > 360.0 - 1e-4, "{sun}");
        // The next sign comes about a month later.
        let vrishabha = find_sankranti(&text, 1, mesha.instant.plus_days(25.0).unwrap()).unwrap();
        let gap = vrishabha.instant.get() - mesha.instant.get();
        assert!(gap > 30.0 && gap < 32.0, "{gap}");
        assert!(find_sankranti(&text, 12, from).is_err());
    }
}

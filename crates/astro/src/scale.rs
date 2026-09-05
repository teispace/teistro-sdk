//! Terrestrial Time and Universal Time: the conversion Delta T makes, as
//! explicit functions that return what they applied so the envelope can
//! stamp it (`docs/03-design/time-and-timezone.md`, §3.1). The civil side
//! (UTC, the leap-second table) lives in the time layer, which builds on
//! these.

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Tt, Ut1};

use crate::delta_t::{DeltaT, DeltaTModel, delta_t};

/// Seconds in a day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;

/// TT from UT1 by adding Delta T.
///
/// ```
/// use teistro_astro::delta_t::{DeltaTModel, delta_t};
/// use teistro_astro::scale::tt_from_ut1;
/// use teistro_core::quantity::{JulianDay, Ut1};
///
/// let ut1 = JulianDay::<Ut1>::literal(2_451_544.5);
/// let dt = delta_t(ut1, DeltaTModel::TableThenModel).expect("the table");
/// let tt = tt_from_ut1(ut1, &dt);
/// assert!((tt.get() - ut1.get()) * 86_400.0 > 63.0);
/// ```
#[must_use]
pub fn tt_from_ut1(instant: JulianDay<Ut1>, delta_t: &DeltaT) -> JulianDay<Tt> {
    JulianDay::try_new(instant.get() + delta_t.days()).unwrap_or(instant.relabel())
}

/// UT1 from TT by subtracting Delta T, evaluated at the UT1 instant it
/// converges on (one refinement, which is exact to a microsecond because
/// Delta T changes by under a second a year).
///
/// # Errors
///
/// A model that cannot answer (`delta_t`).
pub fn ut1_from_tt(
    instant: JulianDay<Tt>,
    model: DeltaTModel,
) -> Result<(JulianDay<Ut1>, DeltaT), Error> {
    let first = delta_t(instant.relabel(), model)?;
    let guess = JulianDay::<Ut1>::try_new(instant.get() - first.days())?;
    let refined = delta_t(guess, model)?;
    let ut1 = JulianDay::<Ut1>::try_new(instant.get() - refined.days())?;
    Ok((ut1, refined))
}

/// TT from a UT1 instant under a model, with the Delta T applied.
///
/// # Errors
///
/// A model that cannot answer.
pub fn tt_of(
    instant: JulianDay<Ut1>,
    model: DeltaTModel,
) -> Result<(JulianDay<Tt>, DeltaT), Error> {
    let applied = delta_t(instant, model)?;
    Ok((tt_from_ut1(instant, &applied), applied))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn tt_and_ut1_round_trip_through_delta_t() {
        let ut1 = JulianDay::<Ut1>::try_new(2_451_544.5).unwrap();
        let dt = delta_t(ut1, DeltaTModel::TableThenModel).unwrap();
        let tt = tt_from_ut1(ut1, &dt);
        assert!((tt.get() - ut1.get() - dt.days()).abs() < 1e-9);
        let (back, applied) = ut1_from_tt(tt, DeltaTModel::TableThenModel).unwrap();
        assert!(
            (back.get() - ut1.get()).abs() < 1e-9,
            "{}",
            (back.get() - ut1.get()) * 86_400.0
        );
        assert!((applied.seconds - dt.seconds).abs() < 1e-6);
        let (again, same) = tt_of(ut1, DeltaTModel::TableThenModel).unwrap();
        assert_eq!(again, tt);
        assert_eq!(same, dt);
    }
}

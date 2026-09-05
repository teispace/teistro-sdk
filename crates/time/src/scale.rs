//! The time scales and their conversions: explicit functions, never
//! `From`, each returning what it applied so the envelope can stamp it
//! (`docs/03-design/time-and-timezone.md`, §3.1).
//!
//! - TT from UT1 adds Delta T; UT1 from TT subtracts it, iterating once.
//! - UTC and UT1 differ by DUT1, under 0.9 s by definition; the SDK
//!   applies zero and says so. Before 1972 UTC had no whole-second
//!   relation to TAI and is treated as UT1, stamped proleptic.
//! - TT from UTC after 1972 goes through the leap-second table exactly:
//!   TT = UTC + (TAI − UTC) + 32.184 s.

use teistro_core::envelope::TimeStamp;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Tt, Ut1, Utc};

use crate::delta_t::{DeltaT, DeltaTModel, DeltaTSource, delta_t};
use crate::leap;

/// TT less TAI, seconds, by definition.
pub const TT_MINUS_TAI_SECONDS: f64 = 32.184;
/// Seconds in a day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;

/// What a UTC conversion assumed.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimeBasis {
    /// The instant precedes 1972, so UTC was read as UT1.
    pub proleptic_utc: bool,
    /// The UT1 less UTC applied, seconds: zero until a provider supplies
    /// the IERS bulletins.
    pub dut1_applied_seconds: f64,
}

impl TimeBasis {
    /// The note the envelope's `time_basis_applied` carries, if any.
    #[must_use]
    pub fn describe(&self) -> Option<String> {
        self.proleptic_utc
            .then(|| String::from("proleptic UTC read as UT1 (before 1972)"))
    }
}

/// A conversion to TT with what it applied.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TtConversion {
    /// The instant on TT.
    pub tt: JulianDay<Tt>,
    /// The Delta T applied.
    pub delta_t: DeltaT,
    /// What the UTC side assumed.
    pub basis: TimeBasis,
}

/// TT from UT1 by adding Delta T.
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

/// UT1 from UTC with DUT1 taken as zero.
#[must_use]
pub fn ut1_from_utc(instant: JulianDay<Utc>) -> (JulianDay<Ut1>, TimeBasis) {
    (
        instant.relabel(),
        TimeBasis {
            proleptic_utc: leap::tai_minus_utc(instant).is_none(),
            dut1_applied_seconds: 0.0,
        },
    )
}

/// UTC from UT1 with DUT1 taken as zero.
#[must_use]
pub fn utc_from_ut1(instant: JulianDay<Ut1>) -> (JulianDay<Utc>, TimeBasis) {
    let utc: JulianDay<Utc> = instant.relabel();
    (
        utc,
        TimeBasis {
            proleptic_utc: leap::tai_minus_utc(utc).is_none(),
            dut1_applied_seconds: 0.0,
        },
    )
}

/// TT from UTC: through the leap-second table from 1972, exactly; before
/// it, UTC read as UT1 with the model's Delta T.
///
/// # Errors
///
/// A model that cannot answer, for an instant before 1972.
pub fn tt_from_utc(instant: JulianDay<Utc>, model: DeltaTModel) -> Result<TtConversion, Error> {
    if let Some(tai_minus_utc) = leap::tai_minus_utc(instant) {
        let seconds = f64::from(tai_minus_utc) + TT_MINUS_TAI_SECONDS;
        let applied = DeltaT {
            seconds,
            model,
            source: DeltaTSource::LeapSeconds,
            uncertainty_seconds: None,
        };
        Ok(TtConversion {
            tt: JulianDay::try_new(instant.get() + seconds / SECONDS_PER_DAY)?,
            delta_t: applied,
            basis: TimeBasis::default(),
        })
    } else {
        let (ut1, basis) = ut1_from_utc(instant);
        let applied = delta_t(ut1, model)?;
        Ok(TtConversion {
            tt: tt_from_ut1(ut1, &applied),
            delta_t: applied,
            basis,
        })
    }
}

/// UTC from TT: the inverse of [`tt_from_utc`], through the table where
/// it applies.
///
/// # Errors
///
/// A model that cannot answer, for an instant before 1972.
pub fn utc_from_tt(
    instant: JulianDay<Tt>,
    model: DeltaTModel,
) -> Result<(JulianDay<Utc>, TtConversion), Error> {
    // A guess a minute early is inside the same leap row unless the
    // instant is within a minute of a row's start; a second pass settles
    // it.
    let mut utc: JulianDay<Utc> = JulianDay::try_new(instant.get() - 70.0 / SECONDS_PER_DAY)?;
    for _ in 0..2 {
        let conversion = tt_from_utc(utc, model)?;
        utc = JulianDay::try_new(instant.get() - conversion.delta_t.days())?;
    }
    let conversion = tt_from_utc(utc, model)?;
    Ok((utc, conversion))
}

/// The envelope's time stamp from a Delta T, a UTC basis and the zone
/// database's version.
#[must_use]
pub fn stamp(delta_t: &DeltaT, basis: &TimeBasis, tzdb_version: &str) -> TimeStamp {
    TimeStamp {
        delta_t_model: delta_t.model.key().to_string(),
        delta_t_seconds: delta_t.seconds,
        leap_table: leap::version().to_string(),
        tzdb_version: tzdb_version.to_string(),
        time_basis_applied: basis.describe(),
        uncertainty_seconds: delta_t.uncertainty_seconds,
    }
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
    }

    #[test]
    fn utc_after_1972_goes_through_the_leap_table_exactly() {
        let utc = JulianDay::<Utc>::try_new(2_451_544.5).unwrap(); // 2000-01-01
        let conversion = tt_from_utc(utc, DeltaTModel::TableThenModel).unwrap();
        assert_eq!(conversion.delta_t.source, DeltaTSource::LeapSeconds);
        assert!((conversion.delta_t.seconds - 64.184).abs() < 1e-12);
        assert!((conversion.tt.get() - (utc.get() + 64.184 / 86_400.0)).abs() < 1e-12);
        assert!(!conversion.basis.proleptic_utc);
        assert_eq!(conversion.basis.describe(), None);
        let (back, again) = utc_from_tt(conversion.tt, DeltaTModel::TableThenModel).unwrap();
        assert!((back.get() - utc.get()).abs() < 1e-9);
        assert!((again.delta_t.seconds - conversion.delta_t.seconds).abs() < 1e-12);
        // Across a leap row: an instant a second after 2017 began.
        let just_after = JulianDay::<Utc>::try_new(2_457_754.5 + 1.0 / 86_400.0).unwrap();
        let c = tt_from_utc(just_after, DeltaTModel::TableThenModel).unwrap();
        assert!((c.delta_t.seconds - 69.184).abs() < 1e-12);
        let (b, _) = utc_from_tt(c.tt, DeltaTModel::TableThenModel).unwrap();
        assert!((b.get() - just_after.get()).abs() < 1e-9);
    }

    #[test]
    fn utc_before_1972_is_proleptic_and_uses_the_model() {
        let utc = JulianDay::<Utc>::try_new(2_415_020.5).unwrap(); // 1900-01-01
        let conversion = tt_from_utc(utc, DeltaTModel::TableThenModel).unwrap();
        assert!(conversion.basis.proleptic_utc);
        assert_eq!(conversion.delta_t.source, DeltaTSource::Model);
        assert!(
            (conversion.delta_t.seconds + 2.7).abs() < 1.5,
            "{}",
            conversion.delta_t
        );
        assert!(conversion.basis.describe().unwrap().contains("proleptic"));
        let (ut1, basis) = ut1_from_utc(utc);
        assert!(basis.proleptic_utc && (ut1.get() - utc.get()).abs() < 1e-12);
        let (utc_again, basis) = utc_from_ut1(ut1);
        assert!(basis.proleptic_utc && (utc_again.get() - utc.get()).abs() < 1e-12);
        let (back, _) = utc_from_tt(conversion.tt, DeltaTModel::TableThenModel).unwrap();
        assert!((back.get() - utc.get()).abs() < 1e-9);
        let stamp = stamp(&conversion.delta_t, &conversion.basis, "2026a");
        assert_eq!(stamp.delta_t_model, "TABLE_THEN_MODEL");
        assert_eq!(stamp.tzdb_version, "2026a");
        assert!(stamp.time_basis_applied.is_some() && stamp.uncertainty_seconds.is_some());
    }
}

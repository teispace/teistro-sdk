//! `sdk.time`: civil times to instants and back, the time scales, and
//! ΔT.

use teistro_astro::delta_t::{DeltaT, delta_t};
use teistro_core::catalogue::Calendar;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Tt, Ut1, Utc};
use teistro_time::scale::{
    tt_from_utc, tt_of, ut1_from_tt, ut1_from_utc, utc_from_tt, utc_from_ut1,
};
use teistro_time::{
    CivilDateTime, EmbeddedTzdb, Policy, Resolved, ZoneResolution, ZoneSpec, civil_of_with, resolve,
};

use crate::area::system_of;
use crate::context::Context;
use crate::scale::{Conversion, Scale};

/// `sdk.time`: a civil time to an instant with the zone metadata a
/// stored chart keeps, an instant back to a civil time, a conversion
/// between scales, and ΔT.
#[derive(Clone, Copy, Debug)]
pub struct TimeArea<'a> {
    context: &'a Context,
}

impl<'a> TimeArea<'a> {
    pub(crate) fn of(context: &'a Context) -> TimeArea<'a> {
        TimeArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(&self) -> &'a Context {
        self.context
    }

    /// A civil date and time in a zone, as an instant — with the offset,
    /// the database's version, the abbreviation and every warning the
    /// resolution carries, which is what a stored chart keeps.
    ///
    /// The zone policy is the context's: `time.polar_day_policy`,
    /// `time.dst_policy` and their like are settings and not arguments,
    /// so two contexts with the same hash resolve the same instant.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship, a civil time that does not
    /// exist in the zone under a policy that refuses it, or a zone the
    /// database does not have.
    pub fn resolve(&self, civil: &CivilDateTime, zone: &ZoneSpec) -> Result<Resolved, Error> {
        let policy = Policy::of(self.context.settings());
        resolve(civil, zone, &policy, EmbeddedTzdb::shared())
    }

    /// An instant as the civil clock in a zone reads it, in a calendar.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship, an instant outside its range,
    /// or a zone the database does not have.
    pub fn civil_of(
        &self,
        instant: JulianDay<Utc>,
        zone: &ZoneSpec,
        calendar: Calendar,
    ) -> Result<(CivilDateTime, ZoneResolution), Error> {
        civil_of_with(system_of(calendar)?, instant, zone, EmbeddedTzdb::shared())
    }

    /// An instant from one scale into another, **and what was applied**.
    ///
    /// The one operation on this surface that is dynamic — a caller
    /// names the scales at run time — which is why `Scale` and
    /// [`Conversion`] are the surface's own types rather than a crate's;
    /// `03-design/rust-consumer-surface.md` says why.
    ///
    /// # Errors
    ///
    /// An instant outside the range of the scale it is in, or a ΔT model
    /// that cannot answer for it.
    pub fn convert(&self, jd: f64, from: Scale, to: Scale) -> Result<Conversion, Error> {
        let model = self.context.delta_t();
        let given =
            |field| JulianDay::<Ut1>::try_new(jd).map_err(|e| Error::from(e).with_field(field));
        Ok(match (from, to) {
            (Scale::Ut1, Scale::Tt) => {
                let (tt, applied) = tt_of(given("jd")?, model)?;
                Conversion {
                    jd: tt.get(),
                    delta_t: Some(applied),
                    ..Conversion::NONE
                }
            }
            (Scale::Tt, Scale::Ut1) => {
                let (ut1, applied) = ut1_from_tt(given("jd")?.relabel::<Tt>(), model)?;
                Conversion {
                    jd: ut1.get(),
                    delta_t: Some(applied),
                    ..Conversion::NONE
                }
            }
            (Scale::Utc, Scale::Tt) => {
                let converted = tt_from_utc(given("jd")?.relabel::<Utc>(), model)?;
                Conversion {
                    jd: converted.tt.get(),
                    delta_t: Some(converted.delta_t),
                    proleptic_utc: converted.basis.proleptic_utc,
                    dut1_seconds: converted.basis.dut1_applied_seconds,
                }
            }
            (Scale::Tt, Scale::Utc) => {
                let (utc, converted) = utc_from_tt(given("jd")?.relabel::<Tt>(), model)?;
                Conversion {
                    jd: utc.get(),
                    delta_t: Some(converted.delta_t),
                    proleptic_utc: converted.basis.proleptic_utc,
                    dut1_seconds: converted.basis.dut1_applied_seconds,
                }
            }
            (Scale::Utc, Scale::Ut1) => {
                let (ut1, basis) = ut1_from_utc(given("jd")?.relabel::<Utc>());
                Conversion {
                    jd: ut1.get(),
                    proleptic_utc: basis.proleptic_utc,
                    dut1_seconds: basis.dut1_applied_seconds,
                    delta_t: None,
                }
            }
            (Scale::Ut1, Scale::Utc) => {
                let (utc, basis) = utc_from_ut1(given("jd")?);
                Conversion {
                    jd: utc.get(),
                    proleptic_utc: basis.proleptic_utc,
                    dut1_seconds: basis.dut1_applied_seconds,
                    delta_t: None,
                }
            }
            // A scale to itself applies nothing, and says so rather than
            // refusing: a caller converting by a run-time value should
            // not have to special-case the identity.
            (Scale::Ut1, Scale::Ut1) | (Scale::Tt, Scale::Tt) | (Scale::Utc, Scale::Utc) => {
                Conversion {
                    jd: given("jd")?.get(),
                    ..Conversion::NONE
                }
            }
        })
    }

    /// ΔT at an instant, under the model the settings chose, with its
    /// source and its uncertainty where the model has one.
    ///
    /// # Errors
    ///
    /// An instant outside UT1's range, or a model that cannot answer.
    pub fn delta_t(&self, instant: JulianDay<Ut1>) -> Result<DeltaT, Error> {
        delta_t(instant, self.context.delta_t())
    }
}

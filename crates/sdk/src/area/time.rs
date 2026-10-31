//! `sdk.time`: civil times to instants and back, the time scales, and
//! ΔT.

use serde::Serialize;
use teistro_astro::delta_t::{DeltaT, delta_t};
use teistro_core::catalogue::Calendar;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Tt, Ut1, Utc};
#[cfg(feature = "signed-packs")]
use teistro_time::ZonePack;
use teistro_time::scale::{
    tt_from_utc, tt_of, ut1_from_tt, ut1_from_utc, utc_from_tt, utc_from_ut1,
};
use teistro_time::{
    CivilDateTime, Policy, Resolved, ZoneResolution, ZoneSpec, civil_of_with, resolve,
};

use crate::area::system_of;
use crate::context::Context;
use crate::scale::{Conversion, Scale};

/// Where a context's zones come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZoneDatabaseSource {
    /// The database compiled into the SDK.
    Embedded,
    /// A pack loaded into this context.
    Pack,
}

/// The zone database a context reads (`03-design/zone-packs.md` §4).
///
/// ```
/// use teistro::{Context, ZoneDatabaseSource};
///
/// let sdk = Context::builder().build()?;
/// let database = sdk.time().zone_database();
/// assert_eq!(database.source, ZoneDatabaseSource::Embedded);
/// assert_eq!(database.version, database.bundled);
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ZoneDatabase {
    /// Where it comes from.
    pub source: ZoneDatabaseSource,
    /// Its release (`2026f`), as every answer's provenance names it.
    pub version: String,
    /// How many names it holds, links included.
    pub zones: usize,
    /// The release compiled into this SDK: the fallback, and the floor an
    /// app that never wants to go below its build passes as `at_least`.
    pub bundled: String,
}

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
    pub fn context(self) -> &'a Context {
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
    pub fn resolve(self, civil: &CivilDateTime, zone: &ZoneSpec) -> Result<Resolved, Error> {
        let policy = Policy::of(self.context.settings());
        resolve(civil, zone, &policy, &*self.context.zones())
    }

    /// An instant as the civil clock in a zone reads it, in a calendar.
    ///
    /// # Errors
    ///
    /// A calendar the SDK does not ship, an instant outside its range,
    /// or a zone the database does not have.
    pub fn civil_of(
        self,
        instant: JulianDay<Utc>,
        zone: &ZoneSpec,
        calendar: Calendar,
    ) -> Result<(CivilDateTime, ZoneResolution), Error> {
        civil_of_with(system_of(calendar)?, instant, zone, &*self.context.zones())
    }

    /// The zone database this context reads.
    ///
    /// ```
    /// use teistro::Context;
    ///
    /// let sdk = Context::builder().build()?;
    /// assert!(sdk.time().zone_database().zones > 300);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn zone_database(self) -> ZoneDatabase {
        self.context.zone_database()
    }

    /// Replaces this context's zone database with a signed pack's, as
    /// [`ZonePack::from_signed`] reads it (`03-design/zone-packs.md`): the
    /// caller fixes the publisher's key at its own build, and `at_least` is
    /// the newest release it has applied, `""` for any. A refused pack
    /// leaves the context as it was; a stored chart replays from its
    /// resolution whatever is loaded.
    ///
    /// ```
    /// use teistro::Context;
    ///
    /// let sdk = Context::builder().build()?;
    /// // A key whose y-coordinate, 2, has no curve point is refused before
    /// // any signature is checked.
    /// let mut key = [0_u8; 32];
    /// key[0] = 2;
    /// let refused = sdk.time().load_zones(b"{}", &[0; 64], &key, "").unwrap_err();
    /// assert_eq!(refused.field(), Some("key"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ZonePack::from_signed`].
    #[cfg(feature = "signed-packs")]
    pub fn load_zones(
        self,
        pack: &[u8],
        signature: &[u8],
        key: &[u8],
        at_least: &str,
    ) -> Result<ZoneDatabase, Error> {
        self.context
            .install_zones(ZonePack::from_signed(pack, signature, key, at_least)?);
        Ok(self.context.zone_database())
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
    pub fn convert(self, jd: f64, from: Scale, to: Scale) -> Result<Conversion, Error> {
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
    pub fn delta_t(self, instant: JulianDay<Ut1>) -> Result<DeltaT, Error> {
        delta_t(instant, self.context.delta_t())
    }
}

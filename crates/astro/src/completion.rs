//! Frame completion: from the frame a provider returns to the frame the
//! caller asked for, with the override policy deciding whether a declared
//! native override or the SDK's own routine does each step, and every step
//! stamped on the result (`docs/03-design/ephemeris-port-and-adapters.md`,
//! §5; ADR-0013).
//!
//! Two differences are completed today: coordinates (equatorial to
//! ecliptic and back, through the obliquity) and the zodiac (tropical to
//! sidereal, through an ayanamsha). Any other difference between the
//! provider's native frame and the request (centre, equinox, corrections)
//! is refused with the step named; light time, aberration, deflection,
//! nutation, precession and topocentric parallax arrive in Phase 2.

use core::fmt;

use serde::Serialize;
use teistro_core::angle::{difference_deg, normalise_deg};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Tt, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{
    Body, Capabilities, Cell, Coordinates, EphemerisProvider, Obliquity, Overrides,
    PositionColumns, PositionRequest, ProviderError, TimeScale, Zodiac,
};

use crate::ayanamsha;
use crate::delta_t::DeltaTModel;
use crate::precession::PrecessionModel;
use crate::scale::tt_of;
use crate::sky::{self, Apparent, ApparentPositions, Spherical};

/// Who computed a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Implementation {
    /// The provider's declared override.
    Native,
    /// The SDK's own routine.
    Sdk,
    /// Nothing to do: the provider returned the requested frame.
    PassThrough,
}

/// One completion step and who did it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Step {
    /// The step's name.
    pub name: &'static str,
    /// Who computed it.
    pub implementation: Implementation,
}

/// A completed response: the columns in the requested frame and the
/// steps that produced them, in order.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Completed {
    /// The columns, in the requested frame.
    pub columns: PositionColumns,
    /// The steps applied.
    pub steps: Vec<Step>,
}

impl Completed {
    /// The steps as `name:IMPLEMENTATION` for a stamp.
    #[must_use]
    pub fn step_keys(&self) -> Vec<String> {
        self.steps
            .iter()
            .map(|s| format!("{}:{:?}", s.name, s.implementation))
            .collect()
    }
}

/// Why completion could not be done.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompletionError {
    /// The provider failed.
    Provider {
        /// Its error.
        error: ProviderError,
    },
    /// A step the completion cannot perform yet.
    Unsupported {
        /// The step.
        step: &'static str,
    },
    /// The policy forbids the only implementation available.
    PolicyRefused {
        /// The step.
        step: &'static str,
        /// The policy.
        policy: OverridePolicy,
    },
    /// The SDK's own routine failed (a Delta T model that cannot answer).
    Sdk {
        /// The error.
        error: Error,
    },
}

impl From<ProviderError> for CompletionError {
    fn from(error: ProviderError) -> CompletionError {
        CompletionError::Provider { error }
    }
}

impl From<Error> for CompletionError {
    fn from(error: Error) -> CompletionError {
        CompletionError::Sdk { error }
    }
}

impl fmt::Display for CompletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompletionError::Provider { error } => write!(f, "{error}"),
            CompletionError::Unsupported { step } => {
                write!(f, "completion step `{step}` is not implemented")
            }
            CompletionError::PolicyRefused { step, policy } => write!(
                f,
                "the {} override policy refuses the available implementation of `{step}`",
                policy.key()
            ),
            CompletionError::Sdk { error } => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CompletionError {}

impl From<CompletionError> for Error {
    fn from(error: CompletionError) -> Error {
        match error {
            CompletionError::Provider { error } => error.into(),
            CompletionError::Sdk { error } => error,
            CompletionError::Unsupported { step } => {
                Error::unsupported(format!("frame completion step `{step}` is not implemented"))
                    .with_hint("ask the provider for a frame it returns natively")
            }
            CompletionError::PolicyRefused { step, policy } => Error::new(
                teistro_core::error::Status::Capability,
                format!(
                    "the {} override policy refuses the available implementation of `{step}`",
                    policy.key()
                ),
            )
            .with_field("provider.overrides"),
        }
    }
}

/// The completion engine over one provider, one policy and one Delta T
/// model (for the SDK's own obliquity, which is a function of TT).
///
/// ```
/// use teistro_astro::{Completion, DeltaTModel, Implementation};
/// use teistro_core::settings::OverridePolicy;
/// use teistro_port_ephemeris::{Body, Coordinates, Frame, PositionRequest, TestProvider, TimeScale};
///
/// let provider = TestProvider::new();
/// let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// let jds = [2_451_545.0];
/// let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL.with_coordinates(Coordinates::Equatorial));
/// let done = completion.positions(&request).expect("completed by the SDK");
/// assert!(done.steps.iter().any(|s| s.name == "obliquity" && s.implementation == Implementation::Sdk));
/// assert_eq!(done.columns.frame.coordinates, Coordinates::Equatorial);
/// ```
#[derive(Debug)]
pub struct Completion<'p, P: EphemerisProvider + ?Sized> {
    provider: &'p P,
    capabilities: Capabilities,
    policy: OverridePolicy,
    delta_t: DeltaTModel,
    precession: PrecessionModel,
}

impl<'p, P: EphemerisProvider + ?Sized> Completion<'p, P> {
    /// Binds a provider under a policy; the capabilities are read once.
    /// The SDK's ayanamshas are carried by the default precession model
    /// until [`Completion::with_precession`] says otherwise.
    pub fn new(provider: &'p P, policy: OverridePolicy, delta_t: DeltaTModel) -> Completion<'p, P> {
        Completion {
            provider,
            capabilities: provider.capabilities(),
            policy,
            delta_t,
            precession: PrecessionModel::default(),
        }
    }

    /// The precession model the SDK's ayanamshas are carried by.
    #[must_use]
    pub const fn with_precession(mut self, model: PrecessionModel) -> Self {
        self.precession = model;
        self
    }

    /// The policy in force.
    #[must_use]
    pub const fn policy(&self) -> OverridePolicy {
        self.policy
    }

    /// The precession model in force for the SDK's ayanamshas.
    #[must_use]
    pub const fn precession(&self) -> PrecessionModel {
        self.precession
    }

    /// The TT instant of a request's instant, converting from UT1 through
    /// the Delta T model and stamping the step.
    fn tt_at(
        &self,
        jd: f64,
        scale: TimeScale,
        steps: &mut Vec<Step>,
    ) -> Result<JulianDay<Tt>, CompletionError> {
        match scale {
            TimeScale::Tt => Ok(JulianDay::try_new(jd).map_err(Error::from)?),
            TimeScale::Ut1 => {
                let (tt, _) = tt_of(JulianDay::try_new(jd).map_err(Error::from)?, self.delta_t)?;
                push_once(
                    steps,
                    Step {
                        name: "delta-t",
                        implementation: Implementation::Sdk,
                    },
                );
                Ok(tt)
            }
        }
    }

    /// The provider's capabilities, as read at construction.
    #[must_use]
    pub const fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    /// The provider.
    #[must_use]
    pub const fn provider(&self) -> &'p P {
        self.provider
    }

    /// Decides who implements a step the provider may override.
    pub(crate) fn choose(
        &self,
        declared: Overrides,
        step: &'static str,
    ) -> Result<Implementation, CompletionError> {
        let native = self.capabilities.has(declared);
        match (self.policy, native) {
            (OverridePolicy::PreferNative | OverridePolicy::NativeOnly, true) => {
                Ok(Implementation::Native)
            }
            (OverridePolicy::PreferNative | OverridePolicy::SdkOnly, false)
            | (OverridePolicy::SdkOnly, true) => Ok(Implementation::Sdk),
            (OverridePolicy::NativeOnly, false) => Err(CompletionError::PolicyRefused {
                step,
                policy: self.policy,
            }),
            // A policy core adds before this crate learns it.
            (other, _) => Err(CompletionError::PolicyRefused {
                step,
                policy: other,
            }),
        }
    }

    /// The obliquity at an instant, by the policy.
    fn obliquity(
        &self,
        jd: f64,
        scale: TimeScale,
        steps: &mut Vec<Step>,
    ) -> Result<Obliquity, CompletionError> {
        let implementation = self.choose(Overrides::OBLIQUITY, "obliquity")?;
        let value = if implementation == Implementation::Native {
            self.provider.obliquity(jd, scale)?
        } else {
            let tt = self.tt_at(jd, scale, steps)?;
            sky::obliquity(tt)
        };
        push_once(
            steps,
            Step {
                name: "obliquity",
                implementation,
            },
        );
        Ok(value)
    }

    /// Positions in the requested frame, completed from the provider's
    /// native frame where they differ.
    ///
    /// # Errors
    ///
    /// The provider's error, an unsupported step, a policy refusal, or a
    /// Delta T model that cannot answer.
    pub fn positions(&self, request: &PositionRequest<'_>) -> Result<Completed, CompletionError> {
        let native = self.capabilities.native_frame;
        let wanted = request.frame;
        if wanted == native {
            let columns = self.provider.positions(request)?;
            return Ok(Completed {
                columns,
                steps: vec![Step {
                    name: "positions",
                    implementation: Implementation::PassThrough,
                }],
            });
        }
        // A provider that can produce the frame natively answers itself;
        // one that refuses with `Unsupported` is asked for its native frame
        // and completed.
        match self.provider.positions(request) {
            Ok(columns) => {
                return Ok(Completed {
                    columns,
                    steps: vec![Step {
                        name: "positions",
                        implementation: Implementation::Native,
                    }],
                });
            }
            Err(ProviderError::Unsupported { .. }) => {}
            Err(error) => return Err(error.into()),
        }
        if wanted.centre != native.centre {
            return Err(CompletionError::Unsupported { step: "centre" });
        }
        if wanted.equinox != native.equinox {
            return Err(CompletionError::Unsupported { step: "equinox" });
        }
        if wanted.corrections != native.corrections {
            return Err(CompletionError::Unsupported {
                step: "corrections",
            });
        }
        let mut steps = vec![Step {
            name: "positions",
            implementation: Implementation::Native,
        }];
        let native_request = request.in_frame(native);
        let mut columns = self.provider.positions(&native_request)?;
        // The zodiac is a shift of ecliptic longitude, so it is applied
        // while the columns are ecliptic: before a rotation out of the
        // ecliptic, after a rotation into it. A rotation to the equator
        // takes the tropical longitude, so a sidereal native frame is
        // shifted first.
        let shift = wanted.zodiac != native.zodiac;
        let native_ecliptic = native.coordinates == Coordinates::Ecliptic;
        if shift && native_ecliptic {
            self.shift_zodiac(
                &mut columns,
                request,
                (native.zodiac, wanted.zodiac),
                native.coordinates,
                &mut steps,
            )?;
        }
        if wanted.coordinates != native.coordinates {
            self.rotate(&mut columns, request, wanted.coordinates, &mut steps)?;
        }
        if shift && !native_ecliptic {
            self.shift_zodiac(
                &mut columns,
                request,
                (native.zodiac, wanted.zodiac),
                wanted.coordinates,
                &mut steps,
            )?;
        }
        columns.frame = wanted;
        Ok(Completed { columns, steps })
    }

    /// Rotates every cell between the ecliptic and the equator with the
    /// true obliquity (the frames here carry nutation) or the mean one.
    fn rotate(
        &self,
        columns: &mut PositionColumns,
        request: &PositionRequest<'_>,
        to: Coordinates,
        steps: &mut Vec<Step>,
    ) -> Result<(), CompletionError> {
        let name = match to {
            Coordinates::Ecliptic => "rotate-equatorial-to-ecliptic",
            Coordinates::Equatorial => "rotate-ecliptic-to-equatorial",
        };
        for (jd_index, jd) in request.jds.iter().enumerate() {
            let obliquity = self.obliquity(*jd, request.scale, steps)?;
            let eps = if request.frame.corrections.nutation {
                obliquity.true_deg
            } else {
                obliquity.mean_deg
            };
            for body_index in 0..columns.body_count {
                let Some(cell) = columns.at(jd_index, body_index) else {
                    continue;
                };
                if cell.is_ok() {
                    columns.set_at(
                        jd_index,
                        body_index,
                        rotate_cell(cell, to, eps, request.speeds),
                    );
                }
            }
        }
        push_once(
            steps,
            Step {
                name,
                implementation: Implementation::Sdk,
            },
        );
        Ok(())
    }

    /// Moves longitudes between the tropical and a sidereal zodiac; the
    /// columns hold `coordinates`, which must be ecliptic.
    fn shift_zodiac(
        &self,
        columns: &mut PositionColumns,
        request: &PositionRequest<'_>,
        (from, to): (Zodiac, Zodiac),
        coordinates: Coordinates,
        steps: &mut Vec<Step>,
    ) -> Result<(), CompletionError> {
        if coordinates != Coordinates::Ecliptic {
            return Err(CompletionError::Unsupported {
                step: "sidereal-equatorial",
            });
        }
        // The provider's override when the policy allows and it declares
        // one; otherwise the SDK's catalogue, the mean value carried by the
        // precession model in force, which every epoch-defined ayanamsha has.
        let implementation = self.choose(Overrides::AYANAMSHA, "ayanamsha")?;
        let mut ayanamsha_steps = Vec::new();
        let mut value = |zodiac: Zodiac, jd: f64| -> Result<f64, CompletionError> {
            match zodiac {
                Zodiac::Tropical => Ok(0.0),
                Zodiac::Sidereal { ayanamsha } => match implementation {
                    Implementation::Native => {
                        Ok(self.provider.ayanamsha_deg(jd, request.scale, ayanamsha)?)
                    }
                    Implementation::Sdk | Implementation::PassThrough => {
                        let tt = self.tt_at(jd, request.scale, &mut ayanamsha_steps)?;
                        Ok(ayanamsha::mean_deg(
                            &ayanamsha.into(),
                            tt,
                            self.precession,
                            self.delta_t,
                        )?)
                    }
                },
            }
        };
        for (jd_index, jd) in request.jds.iter().enumerate() {
            let shift = value(from, *jd)? - value(to, *jd)?;
            for body_index in 0..columns.body_count {
                let Some(cell) = columns.at(jd_index, body_index) else {
                    continue;
                };
                if cell.is_ok() {
                    columns.set_at(
                        jd_index,
                        body_index,
                        Cell {
                            lon: normalise_deg(cell.lon + shift),
                            ..cell
                        },
                    );
                }
            }
        }
        for step in ayanamsha_steps {
            push_once(steps, step);
        }
        push_once(
            steps,
            Step {
                name: "ayanamsha",
                implementation,
            },
        );
        push_once(
            steps,
            Step {
                name: "zodiac-shift",
                implementation: Implementation::Sdk,
            },
        );
        Ok(())
    }
}

impl<P: EphemerisProvider + ?Sized> ApparentPositions for Completion<'_, P> {
    fn apparent(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<Apparent, Error> {
        let jds = [ut1.get()];
        let bodies = [body];
        // Equatorial coordinates in the tropical zodiac, with the
        // provider's own centre, equinox and corrections: an ephemeris
        // answers in the apparent frame, a classical text in its own,
        // and "apparent" to an observer is what each provides.
        let frame = self
            .capabilities
            .native_frame
            .with_coordinates(Coordinates::Equatorial)
            .with_zodiac(Zodiac::Tropical);
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, frame).without_speeds();
        let done = self.positions(&request)?;
        let cell = done
            .columns
            .at(0, 0)
            .ok_or_else(|| Error::internal("a one-cell grid has a cell"))?;
        if !cell.is_ok() {
            return Err(Error::new(
                teistro_core::error::Status::Provider,
                format!(
                    "{} at {ut1}: the provider answered {:?}",
                    body.key(),
                    cell.status
                ),
            )
            .with_field("jd"));
        }
        Ok(Apparent {
            ra_deg: cell.lon,
            dec_deg: cell.lat,
            distance_au: cell.dist,
        })
    }

    fn describe(&self) -> String {
        format!(
            "{} through the ephemeris port ({} overrides)",
            self.capabilities.identity,
            self.policy.key()
        )
    }
}

/// Rotates one cell's coordinates; speeds are rotated by a central
/// difference over a short step, so the obliquity's own rate is neglected
/// (it is under 0.5 arcsecond per year).
fn rotate_cell(cell: Cell, to: Coordinates, eps_deg: f64, speeds: bool) -> Cell {
    let rotate = |p: Spherical| match to {
        Coordinates::Ecliptic => sky::equatorial_to_ecliptic(p, eps_deg),
        Coordinates::Equatorial => sky::ecliptic_to_equatorial(p, eps_deg),
    };
    let here = rotate(Spherical {
        lon_deg: cell.lon,
        lat_deg: cell.lat,
    });
    let (lon_speed, lat_speed) = if speeds {
        let h = 1e-3;
        let ahead = rotate(Spherical {
            lon_deg: cell.lon + cell.lon_speed * h,
            lat_deg: cell.lat + cell.lat_speed * h,
        });
        let behind = rotate(Spherical {
            lon_deg: cell.lon - cell.lon_speed * h,
            lat_deg: cell.lat - cell.lat_speed * h,
        });
        (
            difference_deg(ahead.lon_deg, behind.lon_deg) / (2.0 * h),
            (ahead.lat_deg - behind.lat_deg) / (2.0 * h),
        )
    } else {
        (0.0, 0.0)
    };
    Cell {
        lon: here.lon_deg,
        lat: here.lat_deg,
        lon_speed,
        lat_speed,
        ..cell
    }
}

fn push_once(steps: &mut Vec<Step>, step: Step) {
    if !steps.contains(&step) {
        steps.push(step);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::catalogue::Ayanamsha;
    use teistro_port_ephemeris::{Equinox, Frame, TestProvider};

    use super::*;

    fn completion(policy: OverridePolicy) -> Completion<'static, TestProvider> {
        static PROVIDER: TestProvider = TestProvider;
        Completion::new(&PROVIDER, policy, DeltaTModel::TableThenModel)
    }

    #[test]
    fn a_matching_frame_passes_through() {
        let completion = completion(OverridePolicy::PreferNative);
        let jds = [2_460_000.5];
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
        let done = completion.positions(&request).unwrap();
        assert_eq!(
            done.steps.first().map(|s| s.implementation),
            Some(Implementation::PassThrough)
        );
        assert_eq!(done.step_keys(), vec!["positions:PassThrough"]);
        assert!(completion.describe().starts_with("test-provider"));
        assert_eq!(completion.policy(), OverridePolicy::PreferNative);
        assert_eq!(completion.capabilities().bodies.len(), 8);
    }

    #[test]
    fn rotation_to_equatorial_and_back_is_the_identity() {
        let completion = completion(OverridePolicy::SdkOnly);
        let jds = [2_460_000.5, 2_451_545.0];
        let bodies = [Body::Sun, Body::Moon, Body::Mars];
        let canonical = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
        let equatorial =
            canonical.in_frame(Frame::CANONICAL.with_coordinates(Coordinates::Equatorial));
        let native = completion.provider().positions(&canonical).unwrap();
        let done = completion.positions(&equatorial).unwrap();
        assert!(
            done.steps
                .iter()
                .any(|s| s.name == "rotate-ecliptic-to-equatorial")
        );
        assert!(
            done.steps
                .iter()
                .any(|s| s.name == "obliquity" && s.implementation == Implementation::Sdk)
        );
        assert!(done.steps.iter().any(|s| s.name == "delta-t"));
        for (index, cell) in done.columns.cells().enumerate() {
            let jd = jds.get(index / bodies.len()).copied().unwrap();
            let (tt, _) =
                tt_of(JulianDay::try_new(jd).unwrap(), DeltaTModel::TableThenModel).unwrap();
            let eps = sky::obliquity(tt).true_deg;
            let back = sky::equatorial_to_ecliptic(
                Spherical {
                    lon_deg: cell.lon,
                    lat_deg: cell.lat,
                },
                eps,
            );
            let original = native.cell(index).unwrap();
            assert!(difference_deg(back.lon_deg, original.lon).abs() < 1e-10);
            assert!((back.lat_deg - original.lat).abs() < 1e-10);
        }
        // The apparent position of the rise and set solver reads the same cells.
        let apparent = completion
            .apparent(Body::Sun, JulianDay::literal(2_460_000.5))
            .unwrap();
        assert!((apparent.ra_deg - done.columns.at(0, 0).unwrap().lon).abs() < 1e-12);
        assert!((apparent.distance_au - 1.0).abs() < 1e-12);
    }

    #[test]
    fn unsupported_differences_and_refusals_are_named() {
        let completion = completion(OverridePolicy::PreferNative);
        let jds = [2_460_000.5];
        let request = PositionRequest::new(
            &jds,
            TimeScale::Ut1,
            &[Body::Sun],
            Frame {
                equinox: Equinox::J2000,
                ..Frame::CANONICAL
            },
        );
        assert_eq!(
            completion.positions(&request).err(),
            Some(CompletionError::Unsupported { step: "equinox" })
        );
        // A sidereal request over a provider without the ayanamsha override
        // is completed by the SDK's own catalogue: the longitude moves by
        // Lahiri's value at the instant and the step says who did it.
        let sidereal =
            request.in_frame(Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::Lahiri)));
        let done = completion.positions(&sidereal).unwrap();
        let tropical = completion
            .positions(&request.in_frame(Frame::CANONICAL))
            .unwrap();
        let shift = teistro_core::angle::difference_deg(
            tropical.columns.at(0, 0).unwrap().lon,
            done.columns.at(0, 0).unwrap().lon,
        );
        assert!((shift - 24.2).abs() < 0.05, "{shift}");
        assert!(
            done.steps
                .iter()
                .any(|step| step.name == "ayanamsha" && step.implementation == Implementation::Sdk)
        );
        // A star-anchored ayanamsha reads the star table: Spica held at 180°.
        let anchored =
            request.in_frame(Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::TrueChitra)));
        let chitra = completion.positions(&anchored).unwrap();
        let chitra_shift = difference_deg(
            tropical.columns.at(0, 0).unwrap().lon,
            chitra.columns.at(0, 0).unwrap().lon,
        );
        assert!((chitra_shift - 24.2).abs() < 0.1, "{chitra_shift}");
        let native_only = self::completion(OverridePolicy::NativeOnly);
        let equatorial =
            request.in_frame(Frame::CANONICAL.with_coordinates(Coordinates::Equatorial));
        let error = native_only.positions(&equatorial).unwrap_err();
        assert!(matches!(
            error,
            CompletionError::PolicyRefused {
                step: "obliquity",
                ..
            }
        ));
        assert!(error.to_string().contains("NATIVE_ONLY"));
        let sdk: Error = error.into();
        assert_eq!(sdk.field(), Some("provider.overrides"));
        let sdk: Error = CompletionError::Unsupported { step: "equinox" }.into();
        assert_eq!(sdk.status, teistro_core::error::Status::Unsupported);
        let outside = [TestProvider::JD_RANGE.0 - 10.0];
        let out = PositionRequest::new(&outside, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
        let apparent = completion.apparent(Body::Sun, JulianDay::literal(outside[0]));
        assert!(apparent.is_err() && completion.positions(&out).is_ok());
    }
}

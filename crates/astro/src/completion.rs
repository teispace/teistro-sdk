//! Frame completion: from the frame a provider returns to the frame the
//! caller asked for, with the override policy deciding whether a declared
//! native override or the SDK's own routine does each step, and every step
//! stamped on the result (`docs/03-design/ephemeris-port-and-adapters.md`,
//! §5; ADR-0013).
//!
//! Three differences are completed today: the centre (geocentric to
//! topocentric, through [`crate::topocentric`]), coordinates (equatorial
//! to ecliptic and back, through the obliquity) and the zodiac (tropical
//! to sidereal, through an ayanamsha). The differences that remain — a
//! heliocentric or barycentric centre, the equinox, and the corrections
//! that turn a geometric position into an apparent one — are refused
//! with the step named, and arrive with the built-in ephemeris, which is
//! the first provider that returns a geometric J2000 frame.

use core::fmt;

use serde::Serialize;
use teistro_core::angle::{difference_deg, normalise_deg};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Tt, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{
    Body, Capabilities, Cell, Centre, Coordinates, EphemerisProvider, Equinox, Frame, Obliquity,
    Overrides, PositionColumns, PositionRequest, ProviderError, TimeScale, Zodiac,
};

use crate::ayanamsha;
use crate::delta_t::DeltaTModel;
use crate::precession::{self, PrecessionModel};
use crate::scale::{tt_of, ut1_from_tt};
use crate::sky::{self, Apparent, ApparentPositions, Spherical};
use crate::topocentric::Station;

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

    /// The UT1 and the TT instant of a request's instant, whichever it
    /// was given in: the station's place needs the Earth's rotation,
    /// which is UT1, and the sky around it needs TT.
    fn both_scales(
        &self,
        jd: f64,
        scale: TimeScale,
        steps: &mut Vec<Step>,
    ) -> Result<(JulianDay<Ut1>, JulianDay<Tt>), CompletionError> {
        let (ut1, tt) = match scale {
            TimeScale::Ut1 => {
                let ut1 = JulianDay::try_new(jd).map_err(Error::from)?;
                let (tt, _) = tt_of(ut1, self.delta_t)?;
                (ut1, tt)
            }
            TimeScale::Tt => {
                let tt = JulianDay::try_new(jd).map_err(Error::from)?;
                let (ut1, _) = ut1_from_tt(tt, self.delta_t)?;
                (ut1, tt)
            }
        };
        push_once(
            steps,
            Step {
                name: "delta-t",
                implementation: Implementation::Sdk,
            },
        );
        Ok((ut1, tt))
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
        // A property of the request rather than of any provider, so it is
        // answered here and answered the same way under every policy.
        if wanted.centre == Centre::Topocentric && request.observer.is_none() {
            return Err(CompletionError::Sdk {
                error: Error::new(
                    teistro_core::error::Status::InvalidArg,
                    "a topocentric frame needs the place the observer stands at",
                )
                .with_field("observer")
                .with_hint("build the request with `PositionRequest::from_place`"),
            });
        }
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
        // and completed. Under `sdk-only` it is not asked at all: a
        // provider that answers a whole frame has done several of the
        // steps itself, and the policy is that the SDK's own routines do
        // them (ADR-0013). Until the centre step there was nothing a
        // shipped adapter would have answered here, so this changes what
        // `sdk-only` means for the first time.
        if self.policy != OverridePolicy::SdkOnly {
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
        // The equinox before the centre, and before everything else.
        //
        // Every other step is a rotation or a shift within one epoch, and
        // carries a displacement as it carries anything. The equinox is
        // not: it changes which epoch the axes belong to, and the
        // observer's station is built in the equator **of date**. A
        // topocentric correction applied to J2000 columns would subtract
        // an of-date vector from a J2000 one, which is a wrong answer
        // rather than a refused one.
        if wanted.equinox != native.equinox {
            self.precess(&mut columns, request, native, &mut steps)?;
        }
        if wanted.centre != native.centre {
            self.recentre(&mut columns, request, native, &mut steps)?;
        }
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

    /// The columns carried from the equinox the provider answered in to
    /// the one the request asks for.
    ///
    /// Only J2000 to the equinox of date is done, which is the direction
    /// every consumer needs: a theory is stated at a fixed epoch and a
    /// chart is cast in the equinox of its own moment. The other
    /// direction is refused by the same name rather than guessed at.
    ///
    /// Precession is defined on the equator, so an ecliptic frame is
    /// turned to the equator of J2000, precessed, and turned back to the
    /// ecliptic **of date** — with each end's own mean obliquity, taken
    /// from the precession model itself so the frame and the rotation
    /// that reaches it cannot come from two different theories.
    fn precess(
        &self,
        columns: &mut PositionColumns,
        request: &PositionRequest<'_>,
        native: Frame,
        steps: &mut Vec<Step>,
    ) -> Result<(), CompletionError> {
        if native.equinox != Equinox::J2000 || request.frame.equinox != Equinox::OfDate {
            return Err(CompletionError::Unsupported { step: "equinox" });
        }
        if self.policy == OverridePolicy::NativeOnly {
            return Err(CompletionError::PolicyRefused {
                step: "equinox",
                policy: self.policy,
            });
        }
        let model = self.precession;
        let epoch = JulianDay::<Tt>::literal(2_451_545.0);
        let from_obliquity = precession::mean_obliquity_deg(model, epoch);
        for (jd_index, jd) in request.jds.iter().enumerate() {
            let tt = self.tt_at(*jd, request.scale, steps)?;
            let to_obliquity = precession::mean_obliquity_deg(model, tt);
            for body_index in 0..columns.body_count {
                let Some(cell) = columns.at(jd_index, body_index) else {
                    continue;
                };
                if cell.is_ok() {
                    columns.set_at(
                        jd_index,
                        body_index,
                        precess_cell(
                            cell,
                            model,
                            tt,
                            native.coordinates,
                            (from_obliquity, to_obliquity),
                            request.speeds,
                        ),
                    );
                }
            }
        }
        push_once(
            steps,
            Step {
                name: "equinox",
                implementation: Implementation::Sdk,
            },
        );
        Ok(())
    }

    /// The columns as seen from the request's place rather than from the
    /// centre of the Earth, which is what a chart cast for a place asks
    /// for and what every recorded chart in the conformance corpus is
    /// (`docs/03-design/topocentric-measured.md`).
    ///
    /// Only geocentric to topocentric is done. A heliocentric or
    /// barycentric centre is a different question — it needs the Earth's
    /// own place in the solar system rather than the observer's on the
    /// Earth — and is refused by the same name, as is a sidereal native
    /// frame, whose longitudes are not the ecliptic's.
    fn recentre(
        &self,
        columns: &mut PositionColumns,
        request: &PositionRequest<'_>,
        native: Frame,
        steps: &mut Vec<Step>,
    ) -> Result<(), CompletionError> {
        if native.centre != Centre::Geocentric || request.frame.centre != Centre::Topocentric {
            return Err(CompletionError::Unsupported { step: "centre" });
        }
        if native.zodiac != Zodiac::Tropical {
            return Err(CompletionError::Unsupported {
                step: "sidereal-topocentric",
            });
        }
        // The provider was asked for the topocentric frame above and could
        // not give it — or was not asked, because the policy is
        // `sdk-only` — so the SDK's own routine is the only implementation
        // left, and `native-only` refuses it.
        if self.policy == OverridePolicy::NativeOnly {
            return Err(CompletionError::PolicyRefused {
                step: "centre",
                policy: self.policy,
            });
        }
        // `positions` refuses a topocentric request with no place before
        // any of this, so a request that reaches here carries one.
        let Some(place) = request.observer else {
            return Err(CompletionError::Unsupported { step: "centre" });
        };
        for (jd_index, jd) in request.jds.iter().enumerate() {
            let (ut1, tt) = self.both_scales(*jd, request.scale, steps)?;
            let mut station = Station::at(place, ut1, tt);
            if native.coordinates == Coordinates::Ecliptic {
                // The station is built in the equator of date; the columns
                // are ecliptic, so it is turned once rather than every
                // cell being turned twice.
                let obliquity = self.obliquity(*jd, request.scale, steps)?;
                station = station.in_ecliptic(if native.corrections.nutation {
                    obliquity.true_deg
                } else {
                    obliquity.mean_deg
                });
            }
            for (body_index, body) in request.bodies.iter().enumerate() {
                let Some(cell) = columns.at(jd_index, body_index) else {
                    continue;
                };
                columns.set_at(
                    jd_index,
                    body_index,
                    station.seen(cell, *body, native.corrections, request.speeds),
                );
            }
        }
        push_once(
            steps,
            Step {
                name: "centre",
                implementation: Implementation::Sdk,
            },
        );
        Ok(())
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
        apparent_cell(&done, 0, body, ut1)
    }

    fn apparent_many(
        &self,
        body: Body,
        ut1: &[JulianDay<Ut1>],
        out: &mut Vec<Apparent>,
    ) -> Result<(), Error> {
        out.clear();
        if ut1.is_empty() {
            return Ok(());
        }
        let jds: Vec<f64> = ut1.iter().map(|at| at.get()).collect();
        let bodies = [body];
        let frame = self
            .capabilities
            .native_frame
            .with_coordinates(Coordinates::Equatorial)
            .with_zodiac(Zodiac::Tropical);
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, frame).without_speeds();
        let done = self.positions(&request)?;
        out.reserve(ut1.len());
        for (row, at) in ut1.iter().enumerate() {
            out.push(apparent_cell(&done, row, body, *at)?);
        }
        Ok(())
    }

    fn describe(&self) -> String {
        format!(
            "{} through the ephemeris port ({} overrides)",
            self.capabilities.identity,
            self.policy.key()
        )
    }
}

/// One row of a completed grid as an apparent position, or the
/// provider's refusal for that cell.
fn apparent_cell(
    done: &Completed,
    row: usize,
    body: Body,
    ut1: JulianDay<Ut1>,
) -> Result<Apparent, Error> {
    let cell = done
        .columns
        .at(row, 0)
        .ok_or_else(|| Error::internal("a grid has a cell for every instant"))?;
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

/// A direction as a unit vector, and back.
fn to_vector(p: Spherical) -> [f64; 3] {
    let lon = p.lon_deg.to_radians();
    let lat = p.lat_deg.to_radians();
    let (sin_lon, cos_lon) = lon.sin_cos();
    let (sin_lat, cos_lat) = lat.sin_cos();
    [cos_lat * cos_lon, cos_lat * sin_lon, sin_lat]
}

fn to_spherical(v: [f64; 3]) -> Spherical {
    let flat = v[0].hypot(v[1]);
    Spherical {
        lon_deg: v[1].atan2(v[0]).to_degrees().rem_euclid(360.0),
        lat_deg: v[2].atan2(flat).to_degrees(),
    }
}

/// Precesses one cell.
///
/// The speeds are carried the way [`rotate_cell`] carries them — the
/// transform applied a short step either side — so the precession
/// matrix's own rate is neglected. It turns by about fifty arcseconds a
/// year, which over the step used here is under a microarcsecond.
fn precess_cell(
    cell: Cell,
    model: PrecessionModel,
    tt: JulianDay<Tt>,
    coordinates: Coordinates,
    obliquities: (f64, f64),
    speeds: bool,
) -> Cell {
    let (from_obliquity, to_obliquity) = obliquities;
    let carry = |p: Spherical| -> Spherical {
        let equatorial = match coordinates {
            Coordinates::Ecliptic => sky::ecliptic_to_equatorial(p, from_obliquity),
            Coordinates::Equatorial => p,
        };
        let moved = precession::to_date(model, tt, to_vector(equatorial));
        let of_date = to_spherical(moved);
        match coordinates {
            Coordinates::Ecliptic => sky::equatorial_to_ecliptic(of_date, to_obliquity),
            Coordinates::Equatorial => of_date,
        }
    };
    let here = carry(Spherical {
        lon_deg: cell.lon,
        lat_deg: cell.lat,
    });
    let (lon_speed, lat_speed) = if speeds {
        let h = 1e-3;
        let ahead = carry(Spherical {
            lon_deg: cell.lon + cell.lon_speed * h,
            lat_deg: cell.lat + cell.lat_speed * h,
        });
        let behind = carry(Spherical {
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

    /// A provider that answers whatever frame it is asked for, by
    /// stamping the frame on the canonical cells. Nothing shipped does
    /// this — the two adapters answer their native frame and refuse the
    /// rest — but it is the case the `sdk-only` policy is about.
    #[derive(Debug)]
    struct Obliging;

    impl EphemerisProvider for Obliging {
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                overrides: Overrides::TOPOCENTRIC,
                ..TestProvider.capabilities()
            }
        }

        fn positions(
            &self,
            request: &PositionRequest<'_>,
        ) -> Result<PositionColumns, ProviderError> {
            let mut columns = TestProvider.positions(&request.in_frame(Frame::CANONICAL))?;
            columns.frame = request.frame;
            Ok(columns)
        }
    }

    fn kathmandu() -> teistro_core::quantity::Place {
        teistro_core::quantity::Place::try_from_degrees(27.7172, 85.324, 1_400.0).unwrap()
    }

    #[test]
    fn the_centre_step_displaces_a_body_and_leaves_a_direction_alone() {
        let completion = completion(OverridePolicy::PreferNative);
        let jds = [2_460_000.5];
        let bodies = [Body::Moon, Body::MeanNode];
        let from_centre = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
        let from_place = from_centre.from_place(kathmandu());
        let centre = completion.positions(&from_centre).unwrap();
        let place = completion.positions(&from_place).unwrap();
        assert!(
            place
                .steps
                .iter()
                .any(|s| s.name == "centre" && s.implementation == Implementation::Sdk),
            "{:?}",
            place.step_keys()
        );
        assert_eq!(place.columns.frame.centre, Centre::Topocentric);
        // The test provider stands every body one astronomical unit off,
        // so what moves is the solar parallax and the station's own
        // aberration together.
        let moved = difference_deg(
            place.columns.at(0, 0).unwrap().lon,
            centre.columns.at(0, 0).unwrap().lon,
        )
        .abs()
            * 3_600.0;
        assert!((0.5..30.0).contains(&moved), "the body moved {moved}″");
        assert_eq!(
            place.columns.at(0, 1),
            centre.columns.at(0, 1),
            "a direction was displaced"
        );
        // The step composes with the others: the same request in
        // equatorial coordinates and a sidereal zodiac still names it.
        let turned = from_place.in_frame(
            Frame::CANONICAL
                .with_centre(Centre::Topocentric)
                .with_zodiac(Zodiac::sidereal(Ayanamsha::Lahiri)),
        );
        let done = completion.positions(&turned).unwrap();
        assert!(done.step_keys().contains(&String::from("centre:Sdk")));
        assert!(done.step_keys().contains(&String::from("zodiac-shift:Sdk")));
    }

    #[test]
    fn the_centre_step_refuses_what_it_cannot_do_and_says_which() {
        let completion = completion(OverridePolicy::PreferNative);
        let jds = [2_460_000.5];
        let bodies = [Body::Sun];
        let placeless = PositionRequest::new(
            &jds,
            TimeScale::Ut1,
            &bodies,
            Frame::CANONICAL.with_centre(Centre::Topocentric),
        );
        let refusal = completion.positions(&placeless).unwrap_err();
        let error: Error = refusal.into();
        assert_eq!(error.field(), Some("observer"));
        assert!(error.to_string().contains("observer stands"));
        // A centre this step is not about is refused by the same name.
        for centre in [Centre::Heliocentric, Centre::Barycentric] {
            let elsewhere = placeless.in_frame(Frame::CANONICAL.with_centre(centre));
            assert_eq!(
                completion.positions(&elsewhere).err(),
                Some(CompletionError::Unsupported { step: "centre" }),
                "{centre:?}"
            );
        }
        // `native-only` refuses the SDK's own routine, which is the only
        // one left once the provider has said no.
        let native_only = self::completion(OverridePolicy::NativeOnly);
        let placed = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL)
            .from_place(kathmandu());
        assert!(matches!(
            native_only.positions(&placed).unwrap_err(),
            CompletionError::PolicyRefused { step: "centre", .. }
        ));
    }

    #[test]
    fn sdk_only_does_not_let_a_provider_answer_a_frame_it_declares() {
        static OBLIGING: Obliging = Obliging;
        let jds = [2_460_000.5];
        let bodies = [Body::Moon];
        let placed = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL)
            .from_place(kathmandu());
        // Under `prefer-native` the provider answers and nothing is
        // completed; under `sdk-only` it is not even asked.
        let native = Completion::new(
            &OBLIGING,
            OverridePolicy::PreferNative,
            DeltaTModel::TableThenModel,
        );
        let done = native.positions(&placed).unwrap();
        assert_eq!(done.step_keys(), vec!["positions:Native"]);
        let sdk = Completion::new(
            &OBLIGING,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let done = sdk.positions(&placed).unwrap();
        assert!(done.step_keys().contains(&String::from("centre:Sdk")));
        // A frame that needs nothing still passes through under either.
        let plain = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
        assert_eq!(
            sdk.positions(&plain).unwrap().step_keys(),
            vec!["positions:PassThrough"]
        );
    }

    /// A provider whose native frame is J2000, which is what an analytic
    /// theory is stated in. Until the built-in ephemeris existed, every
    /// provider of this port answered of-date natively and the equinox
    /// step had nothing to exercise it.
    #[derive(Debug)]
    struct AtJ2000;

    impl EphemerisProvider for AtJ2000 {
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                native_frame: Frame {
                    equinox: Equinox::J2000,
                    ..Frame::CANONICAL
                },
                ..TestProvider.capabilities()
            }
        }

        fn positions(
            &self,
            request: &PositionRequest<'_>,
        ) -> Result<PositionColumns, ProviderError> {
            let frame = self.capabilities().native_frame;
            if request.frame != frame {
                return Err(ProviderError::unsupported("only its own frame"));
            }
            let mut columns = PositionColumns::new(request.jds.len(), request.bodies.len(), frame);
            // One body, fixed on the sky, so what moves is the equinox.
            for jd_index in 0..request.jds.len() {
                for body_index in 0..request.bodies.len() {
                    columns.set_at(
                        jd_index,
                        body_index,
                        Cell {
                            lon: 100.0,
                            lat: 2.0,
                            dist: 1.0,
                            lon_speed: 0.0,
                            lat_speed: 0.0,
                            dist_speed: 0.0,
                            status: teistro_port_ephemeris::CellStatus::Ok,
                            source: teistro_port_ephemeris::Source::UNKNOWN,
                        },
                    );
                }
            }
            Ok(columns)
        }
    }

    /// The equinox step carries a J2000 frame to the equinox of date.
    ///
    /// A fixed direction's ecliptic longitude grows with general
    /// precession, about 50.29 arcseconds a year, so a century is 1.396
    /// degrees.
    ///
    /// The latitude moves too, and by a specific amount rather than
    /// nothing: the frame asked for is the mean ecliptic **of date**, and
    /// the ecliptic plane itself turns by about 47 arcseconds a century
    /// under the planets. A test that demanded an unmoving latitude
    /// would be demanding a rotation about a fixed ecliptic pole, which
    /// is not what precession is — and would have failed against correct
    /// code, which is how this bound came to be written from the physics
    /// instead of from an expectation.
    #[test]
    fn the_equinox_step_precesses_a_j2000_frame_to_date() {
        static PROVIDER: AtJ2000 = AtJ2000;
        let completion = Completion::new(
            &PROVIDER,
            OverridePolicy::PreferNative,
            DeltaTModel::TableThenModel,
        );
        // J2000 itself, and a century after it.
        let jds = [2_451_545.0, 2_488_070.0];
        let request = PositionRequest::new(&jds, TimeScale::Tt, &[Body::Sun], Frame::CANONICAL);
        let done = completion.positions(&request).unwrap();
        assert!(
            done.step_keys().iter().any(|key| key == "equinox:Sdk"),
            "the step must say who did it: {:?}",
            done.step_keys()
        );

        let at_epoch = done.columns.at(0, 0).unwrap();
        assert!(
            (at_epoch.lon - 100.0).abs() < 1e-6,
            "at J2000 the two equinoxes are the same, so nothing moves: {}",
            at_epoch.lon
        );
        assert!((at_epoch.lat - 2.0).abs() < 1e-6);

        let after = done.columns.at(1, 0).unwrap();
        let moved = after.lon - at_epoch.lon;
        assert!(
            (moved - 1.3964).abs() < 0.002,
            "a century of general precession is about 1.3964 degrees, not {moved}"
        );
        let latitude_arcsec = (after.lat - 2.0).abs() * 3_600.0;
        assert!(
            (5.0..60.0).contains(&latitude_arcsec),
            "the moving ecliptic should shift the latitude by tens of arcseconds \
             in a century, not {latitude_arcsec}"
        );
    }

    /// The other direction is still refused, by name. A theory stated at
    /// J2000 is what the SDK has to carry forward; carrying a chart back
    /// is a different question and is not guessed at.
    #[test]
    fn of_date_to_j2000_is_still_refused() {
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

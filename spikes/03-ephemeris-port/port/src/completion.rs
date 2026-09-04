//! Frame completion: from the frame a provider returns to the frame the
//! caller asked for, with the override policy deciding whether a declared
//! native override or the SDK's own routine does each step, and every step
//! stamped on the result.
//!
//! The spike completes two differences: coordinates (equatorial to
//! ecliptic and back, through the obliquity) and the zodiac (tropical to
//! sidereal, through an ayanamsha). Any other difference between the
//! provider's native frame and the request (centre, equinox, corrections)
//! is refused with the step named, which is what the SDK's `astro` layer
//! adds in Phase 2 (light time, aberration, deflection, nutation,
//! precession, topocentric parallax).

use serde::Serialize;

use crate::astro::{self, Spherical};
use crate::model::{
    Capabilities, Cell, CellStatus, Coordinates, Obliquity, OverridePolicy, Overrides,
    PositionColumns, PositionRequest, ProviderError, TimeScale, Zodiac,
};
use crate::provider::EphemerisProvider;

/// Who computed a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
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

/// Why completion could not be done.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CompletionError {
    /// The provider failed.
    Provider {
        /// Its error.
        error: ProviderError,
    },
    /// A step the spike cannot perform.
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
}

impl From<ProviderError> for CompletionError {
    fn from(error: ProviderError) -> Self {
        CompletionError::Provider { error }
    }
}

impl core::fmt::Display for CompletionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CompletionError::Provider { error } => write!(f, "{error}"),
            CompletionError::Unsupported { step } => {
                write!(f, "completion step `{step}` is not implemented")
            }
            CompletionError::PolicyRefused { step, policy } => {
                write!(
                    f,
                    "policy {policy:?} refuses the available implementation of `{step}`"
                )
            }
        }
    }
}

impl std::error::Error for CompletionError {}

/// The completion engine over one provider and one policy.
#[derive(Debug)]
pub struct Completion<'p, P: EphemerisProvider + ?Sized> {
    provider: &'p P,
    capabilities: Capabilities,
    policy: OverridePolicy,
}

impl<'p, P: EphemerisProvider + ?Sized> Completion<'p, P> {
    /// Binds a provider under a policy; the capabilities are read once.
    pub fn new(provider: &'p P, policy: OverridePolicy) -> Completion<'p, P> {
        Completion {
            provider,
            capabilities: provider.capabilities(),
            policy,
        }
    }

    /// The policy in force.
    #[must_use]
    pub const fn policy(&self) -> OverridePolicy {
        self.policy
    }

    /// Decides who implements a step the provider may override.
    fn choose(
        &self,
        declared: Overrides,
        step: &'static str,
    ) -> Result<Implementation, CompletionError> {
        let native = self.capabilities.overrides.contains(declared);
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
        }
    }

    /// TT for an instant of the request's scale: the provider's Delta T
    /// override or the SDK's model.
    fn tt(&self, jd: f64, scale: TimeScale, steps: &mut Vec<Step>) -> Result<f64, CompletionError> {
        if scale == TimeScale::Tt {
            return Ok(jd);
        }
        let implementation = self.choose(Overrides::DELTA_T, "delta-t")?;
        let delta_t = match implementation {
            Implementation::Native => self.provider.delta_t_seconds(jd)?,
            _ => astro::delta_t_seconds_approx(jd),
        };
        push_once(
            steps,
            Step {
                name: "delta-t",
                implementation,
            },
        );
        Ok(astro::tt_from_ut1(jd, delta_t))
    }

    /// The obliquity at an instant, by the policy.
    fn obliquity(
        &self,
        jd: f64,
        scale: TimeScale,
        steps: &mut Vec<Step>,
    ) -> Result<Obliquity, CompletionError> {
        let implementation = self.choose(Overrides::OBLIQUITY, "obliquity")?;
        let value = match implementation {
            Implementation::Native => self.provider.obliquity(jd, scale)?,
            _ => astro::obliquity(self.tt(jd, scale, steps)?),
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
    /// The provider's error, an unsupported step, or a policy refusal.
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
        if wanted.center != native.center {
            return Err(CompletionError::Unsupported { step: "center" });
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
        let native_request = PositionRequest {
            frame: native,
            ..*request
        };
        let mut columns = self.provider.positions(&native_request)?;
        if wanted.coordinates != native.coordinates {
            self.rotate(&mut columns, request, wanted.coordinates, &mut steps)?;
        }
        if wanted.zodiac != native.zodiac {
            self.shift_zodiac(
                &mut columns,
                request,
                native.zodiac,
                wanted.zodiac,
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
                let Some(index) = columns.index(jd_index, body_index) else {
                    continue;
                };
                let Some(cell) = columns.cell(index) else {
                    continue;
                };
                if cell.status != CellStatus::Ok {
                    continue;
                }
                columns.set(index, rotate_cell(cell, to, eps, request.speeds));
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

    /// Moves longitudes between the tropical and a sidereal zodiac.
    fn shift_zodiac(
        &self,
        columns: &mut PositionColumns,
        request: &PositionRequest<'_>,
        from: Zodiac,
        to: Zodiac,
        steps: &mut Vec<Step>,
    ) -> Result<(), CompletionError> {
        if request.frame.coordinates != Coordinates::Ecliptic {
            return Err(CompletionError::Unsupported {
                step: "sidereal-equatorial",
            });
        }
        let value = |zodiac: Zodiac, jd: f64| -> Result<f64, CompletionError> {
            match zodiac {
                Zodiac::Tropical => Ok(0.0),
                Zodiac::Sidereal(id) => match self.choose(Overrides::AYANAMSHA, "ayanamsha")? {
                    Implementation::Native => {
                        Ok(self.provider.ayanamsha_deg(jd, request.scale, id)?)
                    }
                    _ => Err(CompletionError::Unsupported {
                        step: "ayanamsha-sdk",
                    }),
                },
            }
        };
        for (jd_index, jd) in request.jds.iter().enumerate() {
            let shift = value(from, *jd)? - value(to, *jd)?;
            for body_index in 0..columns.body_count {
                let Some(index) = columns.index(jd_index, body_index) else {
                    continue;
                };
                let Some(cell) = columns.cell(index) else {
                    continue;
                };
                if cell.status != CellStatus::Ok {
                    continue;
                }
                columns.set(
                    index,
                    Cell {
                        lon: astro::normalise_deg(cell.lon + shift),
                        ..cell
                    },
                );
            }
        }
        push_once(
            steps,
            Step {
                name: "ayanamsha",
                implementation: Implementation::Native,
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

/// Rotates one cell's coordinates; speeds are rotated by a central
/// difference over a short step, so the obliquity's own rate is neglected
/// (it is under 0.5 arcsecond per year).
fn rotate_cell(cell: Cell, to: Coordinates, eps_deg: f64, speeds: bool) -> Cell {
    let rotate = |p: Spherical| match to {
        Coordinates::Ecliptic => astro::equatorial_to_ecliptic(p, eps_deg),
        Coordinates::Equatorial => astro::ecliptic_to_equatorial(p, eps_deg),
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
            astro::angle_difference_deg(ahead.lon_deg, behind.lon_deg) / (2.0 * h),
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
    #![allow(clippy::panic, reason = "a test fails by panicking")]

    use super::*;
    use crate::model::{Body, Frame};
    use crate::test_provider::SliceTestProvider;

    #[test]
    fn a_matching_frame_passes_through() {
        let provider = SliceTestProvider::new();
        let completion = Completion::new(&provider, OverridePolicy::PreferNative);
        let jds = [2_460_000.5];
        let request = PositionRequest {
            jds: &jds,
            scale: TimeScale::Ut1,
            bodies: &[Body::Sun],
            frame: Frame::CANONICAL,
            observer: None,
            speeds: true,
        };
        let done = completion
            .positions(&request)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            done.steps.first().map(|s| s.implementation),
            Some(Implementation::PassThrough)
        );
    }

    #[test]
    fn rotation_to_equatorial_and_back_is_the_identity() {
        let provider = SliceTestProvider::new();
        let completion = Completion::new(&provider, OverridePolicy::SdkOnly);
        let jds = [2_460_000.5, 2_451_545.0];
        let bodies = [Body::Sun, Body::Moon, Body::Mars];
        let canonical = PositionRequest {
            jds: &jds,
            scale: TimeScale::Ut1,
            bodies: &bodies,
            frame: Frame::CANONICAL,
            observer: None,
            speeds: true,
        };
        let equatorial = PositionRequest {
            frame: Frame::CANONICAL.with_coordinates(Coordinates::Equatorial),
            ..canonical
        };
        let native = provider
            .positions(&canonical)
            .unwrap_or_else(|e| panic!("{e}"));
        let done = completion
            .positions(&equatorial)
            .unwrap_or_else(|e| panic!("{e}"));
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
        // Rotate back by hand with the same obliquity and compare.
        for (index, cell) in done.columns.cells().enumerate() {
            let jd = jds.get(index / bodies.len()).copied().unwrap_or_default();
            let eps = astro::obliquity(astro::tt_from_ut1(jd, astro::delta_t_seconds_approx(jd)))
                .true_deg;
            let back = astro::equatorial_to_ecliptic(
                Spherical {
                    lon_deg: cell.lon,
                    lat_deg: cell.lat,
                },
                eps,
            );
            let original = native.cell(index).unwrap_or_else(|| panic!("cell {index}"));
            assert!(astro::angle_difference_deg(back.lon_deg, original.lon).abs() < 1e-10);
            assert!((back.lat_deg - original.lat).abs() < 1e-10);
        }
    }

    #[test]
    fn unsupported_differences_are_named() {
        let provider = SliceTestProvider::new();
        let completion = Completion::new(&provider, OverridePolicy::PreferNative);
        let jds = [2_460_000.5];
        let request = PositionRequest {
            jds: &jds,
            scale: TimeScale::Ut1,
            bodies: &[Body::Sun],
            frame: Frame {
                equinox: crate::model::Equinox::J2000,
                ..Frame::CANONICAL
            },
            observer: None,
            speeds: false,
        };
        assert_eq!(
            completion.positions(&request).err(),
            Some(CompletionError::Unsupported { step: "equinox" })
        );
        let sidereal = PositionRequest {
            frame: Frame::CANONICAL
                .with_zodiac(Zodiac::Sidereal(crate::model::AyanamshaId::LAHIRI)),
            ..request
        };
        assert_eq!(
            completion.positions(&sidereal).err(),
            Some(CompletionError::Unsupported {
                step: "ayanamsha-sdk"
            })
        );
    }
}

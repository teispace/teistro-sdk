//! A provider written in a binding's own language, as the port sees it
//! (`docs/02-architecture/07-binding-architecture.md`, "Ports across the
//! boundary").
//!
//! Each binding wraps its own callback mechanism — napi's function
//! reference in Node, a `js_sys::Function` in wasm — and nothing else
//! differs: what the provider declares becomes [`Capabilities`] with the
//! same defaults, what it answers is read into [`PositionColumns`] with
//! the same checks, and what it threw is kept for the layer above with the
//! same wording. That is this module, so a binding supplies a
//! [`HostCallback`] and the port's rules hold for every binding by
//! construction rather than by copying.
//!
//! ```
//! use teistro_port_ephemeris::host::{Answer, Bound, Declared, HostCallback, Unanswered};
//! use teistro_port_ephemeris::{EphemerisProvider, Frame, PositionRequest};
//!
//! /// A host that answers every cell at 0°.
//! struct Zero;
//!
//! impl HostCallback for Zero {
//!     fn positions(&self, request: &PositionRequest<'_>) -> Result<Option<Answer>, Unanswered> {
//!         let cells = request.cell_count();
//!         Ok(Some(Answer {
//!             frame_bits: request.frame.to_bits(),
//!             lon: vec![0.0; cells],
//!             lat: vec![0.0; cells],
//!             dist: vec![1.0; cells],
//!             status: vec![0; cells],
//!             ..Answer::default()
//!         }))
//!     }
//! }
//!
//! let declared = Declared::named("zero", vec!["graha.SUN".into()]);
//! let bound = Bound::new(declared, Zero).unwrap();
//! assert_eq!(bound.provider().capabilities().identity.name, "zero");
//! ```

use std::ffi::c_void;
use std::sync::Mutex;

use teistro_core::catalogue::Ayanamsha;

use crate::{
    Astronomy, Body, Capabilities, Cell, CellStatus, DistanceUnit, EphemerisKind,
    EphemerisProvider, Exported, Frame, Identity, Overrides, PositionColumns, PositionRequest,
    ProviderError, ProviderVtable, Source, SpeedModel, validate,
};

/// The first Julian day a provider covers when it does not say: year 0.
pub const DEFAULT_JD_MIN: f64 = 1_721_057.5;

/// The last Julian day a provider covers when it does not say: year 3000.
pub const DEFAULT_JD_MAX: f64 = 2_816_787.5;

/// What a host provider says about itself. Everything but the name and
/// the bodies has a default, because a provider that answers the canonical
/// frame with apparent geocentric positions is the common case and should
/// not have to say so.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Declared {
    /// What the provider is, stamped in every result's provenance.
    pub name: String,
    /// The bodies it answers, by their catalogue keys, full (`graha.SUN`)
    /// or bare (`SUN`).
    pub bodies: Vec<String>,
    /// Its version; empty by default.
    pub version: Option<String>,
    /// What identifies its data; empty by default.
    pub data_version: Option<String>,
    /// The first Julian day it covers; [`DEFAULT_JD_MIN`] by default.
    pub jd_min: Option<f64>,
    /// The last Julian day it covers; [`DEFAULT_JD_MAX`] by default.
    pub jd_max: Option<f64>,
    /// The frame it returns natively, packed; [`Frame::CANONICAL`] by
    /// default.
    pub native_frame_bits: Option<u32>,
    /// Whether it computes speeds; `true` by default.
    pub speeds: Option<bool>,
    /// Whether identical requests give identical bits; `true` by default,
    /// and a provider that is not deterministic must say so, because the
    /// conformance contract rests on it (ADR-0022).
    pub deterministic: Option<bool>,
}

impl Declared {
    /// A provider's name and bodies, every other field at its default.
    #[must_use]
    pub fn named(name: impl Into<String>, bodies: Vec<String>) -> Declared {
        Declared {
            name: name.into(),
            bodies,
            ..Declared::default()
        }
    }

    /// The capabilities the SDK drives the provider by.
    ///
    /// # Errors
    ///
    /// A body key the catalogue does not have, no body at all, or frame
    /// bits no frame sets, each as the sentence a binding throws.
    pub fn capabilities(self) -> Result<Capabilities, String> {
        let bodies = self
            .bodies
            .iter()
            .map(|key| {
                Body::from_key(key.rsplit('.').next().unwrap_or(key))
                    .ok_or_else(|| format!("`{key}` is not a body the port knows"))
            })
            .collect::<Result<Vec<Body>, String>>()?;
        if bodies.is_empty() {
            return Err(String::from("a provider must answer at least one body"));
        }
        let native_frame = match self.native_frame_bits {
            Some(bits) => Frame::try_from_bits(bits).map_err(|error| error.to_string())?,
            None => Frame::CANONICAL,
        };
        Ok(Capabilities {
            identity: Identity {
                name: self.name,
                version: self.version.unwrap_or_default(),
                data_version: self.data_version.unwrap_or_default(),
                tier: None,
                data_hashes: Vec::new(),
            },
            jd_range: (
                self.jd_min.unwrap_or(DEFAULT_JD_MIN),
                self.jd_max.unwrap_or(DEFAULT_JD_MAX),
            ),
            bodies,
            native_frame,
            astronomy: Astronomy::Modern,
            speeds: self.speeds.unwrap_or(true),
            speed_model: SpeedModel::Derivative,
            distance_unit: DistanceUnit::AstronomicalUnits,
            overrides: Overrides::NONE,
            ayanamshas: Vec::<Ayanamsha>::new(),
            deterministic: self.deterministic.unwrap_or(true),
            // A host provider describes no engine of its own yet; the host
            // surface for it arrives with B2.
            native: false,
        })
    }
}

/// The columns a host provider answered with, one value per cell in the
/// request's order. A column left empty reads as zero, except `status`,
/// which reads as `Ok`, and `source`, which reads as unknown; the four a
/// cell cannot do without (`lon`, `lat`, `dist`, `status`) must be full.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Answer {
    /// `Frame::to_bits` of the values.
    pub frame_bits: u32,
    /// Longitudes, in degrees.
    pub lon: Vec<f64>,
    /// Latitudes, in degrees.
    pub lat: Vec<f64>,
    /// Distances.
    pub dist: Vec<f64>,
    /// Longitude speeds, in degrees a day.
    pub lon_speed: Vec<f64>,
    /// Latitude speeds, in degrees a day.
    pub lat_speed: Vec<f64>,
    /// Distance speeds.
    pub dist_speed: Vec<f64>,
    /// Per-cell status codes (`CellStatus::code`).
    pub status: Vec<i32>,
    /// Per-cell sources (`Source::to_bits`).
    pub source: Vec<u32>,
}

impl Answer {
    /// The answer read into the port's own shape. A column of the wrong
    /// length is refused by name rather than read past its end.
    ///
    /// # Errors
    ///
    /// Frame bits no frame sets, or a required column whose length is not
    /// the request's cell count.
    pub fn columns(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, String> {
        let cells = request.cell_count();
        let frame = Frame::try_from_bits(self.frame_bits)
            .map_err(|error| format!("answered in a frame that is not one: {error}"))?;
        for (name, len) in [
            ("lon", self.lon.len()),
            ("lat", self.lat.len()),
            ("dist", self.dist.len()),
            ("status", self.status.len()),
        ] {
            if len != cells {
                return Err(format!(
                    "returned {len} values in `{name}` for {cells} cells"
                ));
            }
        }
        let mut columns = PositionColumns::new(request.jds.len(), request.bodies.len(), frame);
        let value = |column: &[f64], index: usize| column.get(index).copied().unwrap_or(0.0);
        for index in 0..cells {
            let status = self
                .status
                .get(index)
                .map_or(CellStatus::Ok, |code| CellStatus::from_code(*code));
            let source = self.source.get(index).map_or(
                Source {
                    kind: EphemerisKind::Unknown,
                    tier: None,
                },
                |bits| Source::from_bits(*bits),
            );
            columns.set(
                index,
                Cell {
                    lon: value(&self.lon, index),
                    lat: value(&self.lat, index),
                    dist: value(&self.dist, index),
                    lon_speed: value(&self.lon_speed, index),
                    lat_speed: value(&self.lat_speed, index),
                    dist_speed: value(&self.dist_speed, index),
                    status,
                    source,
                },
            );
        }
        Ok(columns)
    }
}

/// Why a host callback gave no answer.
#[derive(Debug, Clone, PartialEq)]
pub enum Unanswered {
    /// The callback threw; the sentence is what it threw, and it is kept
    /// for the layer above to rethrow.
    Threw(String),
    /// The binding refused before calling: the port's own error, passed
    /// through as it is.
    Refused(ProviderError),
}

/// A binding's way of asking the host for positions.
pub trait HostCallback {
    /// The positions the host gives for a request the port has already
    /// validated against the declared capabilities.
    ///
    /// `Ok(None)` is a provider saying it cannot produce the frame asked
    /// for; the SDK then asks for its native frame and completes the rest
    /// itself.
    ///
    /// # Errors
    ///
    /// What the callback threw, or the binding's own refusal.
    fn positions(&self, request: &PositionRequest<'_>) -> Result<Option<Answer>, Unanswered>;
}

/// A host provider: its capabilities, its callback, and what the callback
/// last threw.
#[derive(Debug)]
pub struct HostProvider<C> {
    capabilities: Capabilities,
    callback: C,
    /// Only a code crosses the C boundary, and a provider written in a
    /// host language has more to say than a code, so the sentence is kept
    /// here for the layer above.
    thrown: Mutex<Option<String>>,
}

impl<C> HostProvider<C> {
    /// The binding's callback.
    #[must_use]
    pub fn callback(&self) -> &C {
        &self.callback
    }

    /// Forgets what an earlier call threw, before a new call starts.
    pub fn forget_thrown(&self) {
        self.set_thrown(None);
    }

    /// What the callback threw during the call that just ended, taken.
    #[must_use]
    pub fn take_thrown(&self) -> Option<String> {
        self.thrown.lock().map_or(None, |mut thrown| thrown.take())
    }

    fn set_thrown(&self, sentence: Option<String>) {
        if let Ok(mut thrown) = self.thrown.lock() {
            *thrown = sentence;
        }
    }

    /// Records what went wrong and refuses the call.
    fn refused(&self, sentence: String) -> ProviderError {
        self.set_thrown(Some(sentence.clone()));
        ProviderError::Refused { detail: sentence }
    }
}

impl<C: HostCallback + Send + Sync> EphemerisProvider for HostProvider<C> {
    fn capabilities(&self) -> Capabilities {
        self.capabilities.clone()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        // The port's own check runs first, so a provider is never asked
        // for a body, an instant or a frame it did not declare, and the
        // refusal names what is missing rather than leaving a callback to
        // discover it.
        validate(&self.capabilities, request)?;
        match self.callback.positions(request) {
            Ok(None) => Err(ProviderError::unsupported(format!(
                "the {} frame",
                request.frame.key()
            ))),
            Ok(Some(answer)) => answer
                .columns(request)
                .map_err(|sentence| self.refused(sentence)),
            Err(Unanswered::Threw(sentence)) => Err(self.refused(format!("threw: {sentence}"))),
            Err(Unanswered::Refused(error)) => Err(error),
        }
    }
}

/// A host provider bound into the port's vtable: the boxed provider the
/// vtable points at, kept alive by the context that owns it.
pub struct Bound<C: HostCallback + Send + Sync> {
    exported: Box<Exported<HostProvider<C>>>,
}

impl<C: HostCallback + Send + Sync> core::fmt::Debug for Bound<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Bound").finish_non_exhaustive()
    }
}

impl<C: HostCallback + Send + Sync> Bound<C> {
    /// Binds a provider's declaration and its callback.
    ///
    /// # Errors
    ///
    /// As [`Declared::capabilities`].
    pub fn new(declared: Declared, callback: C) -> Result<Bound<C>, String> {
        Ok(Bound {
            exported: Exported::new(HostProvider {
                capabilities: declared.capabilities()?,
                callback,
                thrown: Mutex::new(None),
            }),
        })
    }

    /// The provider inside.
    #[must_use]
    pub fn provider(&self) -> &HostProvider<C> {
        self.exported.provider()
    }

    /// The vtable the SDK drives the provider through.
    #[must_use]
    pub fn vtable() -> ProviderVtable {
        Exported::<HostProvider<C>>::vtable()
    }

    /// The pointer the SDK hands back to every callback.
    #[must_use]
    pub fn user_data(&self) -> *mut c_void {
        self.exported.user_data()
    }

    /// Takes what the callback threw during the call that just ended, as
    /// the sentence a binding throws.
    ///
    /// # Errors
    ///
    /// The provider's failure, so a failure inside a provider reaches the
    /// caller as its own error rather than as a status code.
    pub fn leave(&self) -> Result<(), String> {
        match self.provider().take_thrown() {
            Some(sentence) => Err(format!("the ephemeris provider {sentence}")),
            None => Ok(()),
        }
    }
}

/// The vtable and the user data of an optional host, as a generated
/// constructor passes them: none, and a null pointer, when there is no
/// host.
#[must_use]
pub fn parts<C: HostCallback + Send + Sync>(
    host: Option<&Bound<C>>,
) -> (Option<ProviderVtable>, *mut c_void) {
    match host {
        Some(host) => (Some(Bound::<C>::vtable()), host.user_data()),
        None => (None, core::ptr::null_mut()),
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "a test fails by panicking"
    )]

    use super::*;
    use crate::{TimeScale, VtableProvider};

    /// A host that answers with what it was built with.
    struct Scripted(Result<Option<Answer>, Unanswered>);

    impl HostCallback for Scripted {
        fn positions(&self, _: &PositionRequest<'_>) -> Result<Option<Answer>, Unanswered> {
            self.0.clone()
        }
    }

    fn request<'a>(jds: &'a [f64], bodies: &'a [Body]) -> PositionRequest<'a> {
        PositionRequest::new(jds, TimeScale::Tt, bodies, Frame::CANONICAL)
    }

    fn full(cells: usize) -> Answer {
        Answer {
            frame_bits: Frame::CANONICAL.to_bits(),
            lon: vec![12.5; cells],
            lat: vec![0.25; cells],
            dist: vec![1.0; cells],
            status: vec![0; cells],
            ..Answer::default()
        }
    }

    fn provider(answer: Result<Option<Answer>, Unanswered>) -> Bound<Scripted> {
        Bound::new(
            Declared::named("host", vec!["SUN".into()]),
            Scripted(answer),
        )
        .unwrap()
    }

    #[test]
    fn a_declaration_takes_its_defaults() {
        let capabilities = Declared::named("host", vec!["graha.SUN".into(), "MOON".into()])
            .capabilities()
            .unwrap();
        assert_eq!(capabilities.bodies, [Body::Sun, Body::Moon]);
        assert_eq!(capabilities.jd_range, (DEFAULT_JD_MIN, DEFAULT_JD_MAX));
        assert_eq!(capabilities.native_frame, Frame::CANONICAL);
        assert!(capabilities.speeds && capabilities.deterministic);
    }

    #[test]
    fn a_declaration_is_refused_in_the_words_a_binding_throws() {
        let refused = |declared: Declared| declared.capabilities().unwrap_err();
        assert_eq!(
            refused(Declared::named("host", vec!["graha.NOPE".into()])),
            "`graha.NOPE` is not a body the port knows"
        );
        assert_eq!(
            refused(Declared::named("host", Vec::new())),
            "a provider must answer at least one body"
        );
        let bad_frame = Declared {
            native_frame_bits: Some(u32::MAX),
            ..Declared::named("host", vec!["SUN".into()])
        };
        assert!(!refused(bad_frame).is_empty());
    }

    #[test]
    fn an_answer_reaches_the_port_and_a_short_column_is_refused_by_name() {
        let jds = [2_451_545.0, 2_451_546.0];
        let bodies = [Body::Sun];
        let bound = provider(Ok(Some(full(2))));
        let columns = bound.provider().positions(&request(&jds, &bodies)).unwrap();
        assert_eq!(columns.cell(1).map(|cell| cell.lon), Some(12.5));
        assert_eq!(bound.leave(), Ok(()));

        let mut short = full(2);
        short.lat.pop();
        let bound = provider(Ok(Some(short)));
        assert!(bound.provider().positions(&request(&jds, &bodies)).is_err());
        assert_eq!(
            bound.leave(),
            Err(String::from(
                "the ephemeris provider returned 1 values in `lat` for 2 cells"
            ))
        );
    }

    #[test]
    fn what_a_callback_threw_is_kept_once_and_a_refusal_passes_through() {
        let jds = [2_451_545.0];
        let bodies = [Body::Sun];
        let bound = provider(Err(Unanswered::Threw(String::from("boom"))));
        assert!(bound.provider().positions(&request(&jds, &bodies)).is_err());
        assert_eq!(
            bound.leave(),
            Err(String::from("the ephemeris provider threw: boom"))
        );
        assert_eq!(bound.leave(), Ok(()), "taken, not kept");

        let bound = provider(Err(Unanswered::Refused(ProviderError::unsupported("x"))));
        assert_eq!(
            bound.provider().positions(&request(&jds, &bodies)),
            Err(ProviderError::unsupported("x"))
        );
        assert_eq!(bound.leave(), Ok(()), "a refusal is not a throw");

        let bound = provider(Ok(None));
        assert!(matches!(
            bound.provider().positions(&request(&jds, &bodies)),
            Err(ProviderError::Unsupported { .. })
        ));
    }

    /// The vtable drives the same provider: what the C side reads is what
    /// the host answered.
    #[test]
    #[allow(unsafe_code, reason = "binding the vtable is the thing tested")]
    fn the_bound_provider_answers_through_its_vtable() {
        let bound = provider(Ok(Some(full(1))));
        let (vtable, user_data) = parts(Some(&bound));
        // SAFETY: the vtable is this provider type's and `user_data` is the
        // box `bound` owns, alive for the whole test.
        let driven =
            unsafe { VtableProvider::bind(vtable.unwrap(), user_data) }.expect("a valid vtable");
        let jds = [2_451_545.0];
        let bodies = [Body::Sun];
        let columns = driven.positions(&request(&jds, &bodies)).unwrap();
        assert_eq!(columns.cell(0).map(|cell| cell.lat), Some(0.25));
        assert!(parts::<Scripted>(None).0.is_none());
    }
}

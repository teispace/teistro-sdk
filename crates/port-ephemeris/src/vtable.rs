//! The port as a C vtable, so a native engine, a licensed library's shim
//! and a binding's host object are one shape to the SDK
//! (`docs/03-design/ephemeris-port-and-adapters.md`, §6). Two directions:
//! [`VtableProvider`] drives any vtable through the Rust trait, and
//! [`Exported`] presents any Rust provider as a vtable. The crate's only
//! unsafe code lives here, one SAFETY comment per block.
//!
//! Every struct that crosses carries `struct_size` so that either side can
//! be older; columns are caller-allocated, so a provider writes into the
//! SDK's vectors and never allocates for it.

#![allow(unsafe_code, reason = "the C boundary of the port")]

use core::ffi::{CStr, c_char, c_void};
use core::ptr;
use std::ffi::CString;

use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Place, Ut1};

use crate::body::{Body, TimeScale};
use crate::capabilities::{
    Astronomy, Capabilities, DataHash, DistanceUnit, Identity, Obliquity, Overrides, SpeedModel,
};
use crate::columns::{CellStatus, PositionColumns, Source, tier_bits, tier_from_bits};
use crate::crossing::{CrossingRequest, Direction, Event, Lattice, Quantity};
use crate::error::ProviderError;
use crate::frame::Frame;
use crate::horizon::{DiscPoint, Horizon, HorizonEventKind, HorizonRequest, Refraction};
use crate::provider::{EphemerisProvider, PositionRequest};

/// The ABI version of the vtable layout.
pub const VTABLE_ABI_VERSION: u32 = 2;

/// A C observer: degrees and metres, validated into a [`Place`] on the
/// way in.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ObserverC {
    /// Degrees, east positive.
    pub longitude_deg: f64,
    /// Degrees, north positive.
    pub latitude_deg: f64,
    /// Metres above sea level.
    pub altitude_m: f64,
}

impl ObserverC {
    fn of(place: Place) -> ObserverC {
        ObserverC {
            longitude_deg: place.longitude.get(),
            latitude_deg: place.latitude.get(),
            altitude_m: place.altitude.get(),
        }
    }

    fn place(self) -> Result<Place, ProviderError> {
        Place::try_from_degrees(self.latitude_deg, self.longitude_deg, self.altitude_m)
            .map_err(|error| ProviderError::invalid(error.to_string()))
    }
}

/// A C position request over a grid.
#[repr(C)]
#[derive(Debug)]
pub struct PositionRequestC {
    /// `sizeof(PositionRequestC)` as the caller compiled it.
    pub struct_size: u32,
    /// [`TimeScale::id`].
    pub scale: u32,
    /// [`Frame::to_bits`].
    pub frame_bits: u32,
    /// Non-zero when speeds are wanted.
    pub speeds: u8,
    /// Non-zero when `observer` is set.
    pub has_observer: u8,
    /// Reserved, zero.
    pub reserved: [u8; 2],
    /// The observer, read when `has_observer` is non-zero.
    pub observer: ObserverC,
    /// The instants.
    pub jds: *const f64,
    /// How many instants.
    pub jd_count: usize,
    /// The bodies as [`Body::id`].
    pub bodies: *const u16,
    /// How many bodies.
    pub body_count: usize,
}

/// C columns the provider fills: caller-owned arrays of `capacity` cells,
/// instants outermost.
#[repr(C)]
#[derive(Debug)]
pub struct PositionColumnsC {
    /// `sizeof(PositionColumnsC)` as the caller compiled it.
    pub struct_size: u32,
    /// Written by the provider: [`Frame::to_bits`] of the values.
    pub frame_bits: u32,
    /// Longitudes.
    pub lon: *mut f64,
    /// Latitudes.
    pub lat: *mut f64,
    /// Distances.
    pub dist: *mut f64,
    /// Longitude speeds.
    pub lon_speed: *mut f64,
    /// Latitude speeds.
    pub lat_speed: *mut f64,
    /// Distance speeds.
    pub dist_speed: *mut f64,
    /// Per-cell status codes ([`CellStatus::code`]).
    pub status: *mut i32,
    /// Per-cell sources ([`Source::to_bits`]).
    pub source: *mut u32,
    /// The number of cells each array holds.
    pub capacity: usize,
}

/// A C obliquity.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ObliquityC {
    /// Mean obliquity, degrees.
    pub mean_deg: f64,
    /// True obliquity, degrees.
    pub true_deg: f64,
    /// Nutation in longitude, degrees.
    pub nutation_lon_deg: f64,
    /// Nutation in obliquity, degrees.
    pub nutation_obl_deg: f64,
}

/// A C horizon-event request.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HorizonRequestC {
    /// `sizeof(HorizonRequestC)` as the caller compiled it.
    pub struct_size: u32,
    /// [`Body::id`].
    pub body: u16,
    /// [`HorizonEventKind::id`].
    pub kind: u8,
    /// [`DiscPoint::id`].
    pub disc: u8,
    /// [`Refraction::id`].
    pub refraction: u8,
    /// Reserved, zero.
    pub reserved: [u8; 7],
    /// The place.
    pub observer: ObserverC,
    /// The search begins here, UT1.
    pub from_jd_ut1: f64,
    /// The search ends this many days later.
    pub window_days: f64,
    /// The altitude of the disc point at the event, degrees.
    pub altitude_deg: f64,
}

/// A C crossings request.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CrossingRequestC {
    /// `sizeof(CrossingRequestC)` as the caller compiled it.
    pub struct_size: u32,
    /// [`Frame::to_bits`].
    pub frame_bits: u32,
    /// [`Quantity::kind_id`]: 0 a longitude, 1 a speed, 2 a composite.
    pub quantity_kind: u8,
    /// Whether `observer` is set.
    pub has_observer: u8,
    /// Reserved, zero.
    pub reserved: [u8; 2],
    /// [`Body::id`] of the first body.
    pub first_body: u16,
    /// [`Body::id`] of the second body of a composite; else zero.
    pub second_body: u16,
    /// The first body's coefficient in a composite.
    pub coefficient_a: f64,
    /// The second body's coefficient in a composite.
    pub coefficient_b: f64,
    /// The lattice's first line, degrees.
    pub origin_deg: f64,
    /// The lattice's spacing, degrees; zero for a single target.
    pub step_deg: f64,
    /// The window's start, UT1.
    pub from_jd_ut1: f64,
    /// The window's end, UT1.
    pub to_jd_ut1: f64,
    /// How closely each instant is placed, days.
    pub tolerance_days: f64,
    /// The observer, read when `has_observer` is set.
    pub observer: ObserverC,
}

/// A C crossing event.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CrossingEventC {
    /// The instant, UT1.
    pub jd_ut1: f64,
    /// The boundary reached, degrees.
    pub boundary_deg: f64,
    /// [`Direction::id`].
    pub direction: u8,
    /// Reserved, zero.
    pub reserved: [u8; 7],
}

impl CrossingEventC {
    fn of(event: &Event) -> CrossingEventC {
        CrossingEventC {
            jd_ut1: event.instant.get(),
            boundary_deg: event.boundary_deg,
            direction: event.direction.id(),
            reserved: [0; 7],
        }
    }

    fn event(self) -> Result<Event, ProviderError> {
        let direction = Direction::from_id(self.direction)
            .ok_or_else(|| ProviderError::invalid(format!("direction {}", self.direction)))?;
        if !self.boundary_deg.is_finite() {
            return Err(ProviderError::invalid(
                "a crossing's boundary is not finite",
            ));
        }
        Ok(Event {
            instant: JulianDay::try_new(self.jd_ut1)
                .map_err(|error| ProviderError::invalid(error.to_string()))?,
            boundary_deg: self.boundary_deg,
            direction,
            evaluations: 0,
        })
    }
}

/// A C data hash.
#[repr(C)]
#[derive(Debug)]
pub struct DataHashC {
    /// The file name, NUL-terminated.
    pub file: *const c_char,
    /// SHA-256 as hex, NUL-terminated.
    pub sha256: *const c_char,
    /// The size in bytes.
    pub bytes: u64,
}

/// C capabilities; every pointer is owned by the provider and valid for
/// its lifetime.
#[repr(C)]
#[derive(Debug)]
pub struct CapabilitiesC {
    /// `sizeof(CapabilitiesC)` as the caller compiled it.
    pub struct_size: u32,
    /// Non-zero when speeds are returned.
    pub speeds: u8,
    /// Non-zero when identical requests give identical bits.
    pub deterministic: u8,
    /// The tier plus one, zero for none ([`Source::to_bits`] encoding).
    pub tier: u8,
    /// [`DistanceUnit::id`].
    pub distance_unit: u8,
    /// [`SpeedModel::id`].
    pub speed_model: u8,
    /// [`Astronomy::id`].
    pub astronomy: u8,
    /// Reserved, zero.
    pub reserved: [u8; 2],
    /// The name, NUL-terminated.
    pub name: *const c_char,
    /// The version, NUL-terminated.
    pub version: *const c_char,
    /// The data version, NUL-terminated.
    pub data_version: *const c_char,
    /// Coverage start, UT1.
    pub jd_min: f64,
    /// Coverage end, UT1.
    pub jd_max: f64,
    /// The bodies as ids.
    pub bodies: *const u16,
    /// How many.
    pub body_count: usize,
    /// [`Frame::to_bits`] of the native frame.
    pub native_frame_bits: u32,
    /// [`Overrides::bits`].
    pub overrides: u32,
    /// The ayanamshas the override knows, as catalogue ids.
    pub ayanamshas: *const u16,
    /// How many.
    pub ayanamsha_count: usize,
    /// The data hashes.
    pub hashes: *const DataHashC,
    /// How many.
    pub hash_count: usize,
}

/// The vtable: `user_data` is whatever the provider registered and is
/// passed back to every function; a null function is an undeclared
/// operation.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ProviderVtable {
    /// `sizeof(ProviderVtable)` as the caller compiled it.
    pub struct_size: u32,
    /// [`VTABLE_ABI_VERSION`].
    pub abi_version: u32,
    /// Fills the capabilities; returns `0` or a [`ProviderError::code`].
    pub capabilities:
        Option<unsafe extern "C" fn(user_data: *mut c_void, out: *mut CapabilitiesC) -> i32>,
    /// Fills the columns; returns `0` or a [`ProviderError::code`].
    pub positions: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            request: *const PositionRequestC,
            out: *mut PositionColumnsC,
        ) -> i32,
    >,
    /// The obliquity override.
    pub obliquity: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            jd: f64,
            scale: u32,
            out: *mut ObliquityC,
        ) -> i32,
    >,
    /// The Delta T override, seconds.
    pub delta_t: Option<
        unsafe extern "C" fn(user_data: *mut c_void, jd_ut1: f64, out_seconds: *mut f64) -> i32,
    >,
    /// The ayanamsha override, degrees, by catalogue id.
    pub ayanamsha: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            jd: f64,
            scale: u32,
            ayanamsha: u16,
            out_deg: *mut f64,
        ) -> i32,
    >,
    /// The DUT1 override, seconds.
    pub dut1: Option<
        unsafe extern "C" fn(user_data: *mut c_void, jd_utc: f64, out_seconds: *mut f64) -> i32,
    >,
    /// The rise and set override: writes the instant and whether the
    /// event was found inside the window.
    pub horizon_event: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            request: *const HorizonRequestC,
            out_jd_ut1: *mut f64,
            out_found: *mut u8,
        ) -> i32,
    >,
    /// The crossings override: writes up to `capacity` events into the
    /// caller's buffer and the number found into `out_count`, which may
    /// exceed the capacity, in which case the caller calls again with a
    /// larger buffer.
    pub crossings: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            request: *const CrossingRequestC,
            out_events: *mut CrossingEventC,
            capacity: u32,
            out_count: *mut u32,
        ) -> i32,
    >,
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "guarded by the comparison above it"
)]
const fn size_of_u32<T>() -> u32 {
    let size = core::mem::size_of::<T>();
    if size > u32::MAX as usize {
        u32::MAX
    } else {
        size as u32
    }
}

fn status(result: Result<(), ProviderError>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(error) => error.code(),
    }
}

fn check(code: i32, context: &str) -> Result<(), ProviderError> {
    if code == 0 {
        Ok(())
    } else {
        Err(ProviderError::from_code(code, context))
    }
}

/// The code a trampoline answers a malformed call with.
fn invalid_call() -> i32 {
    ProviderError::invalid("").code()
}

// ── Driving a vtable through the trait ─────────────────────────────────────

/// Any vtable as an [`EphemerisProvider`]. The capabilities are read once
/// at construction and owned in Rust; every call after that goes through
/// the function pointers.
pub struct VtableProvider {
    vtable: ProviderVtable,
    user_data: *mut c_void,
    capabilities: Capabilities,
}

// SAFETY: the vtable's functions are required by `bind`'s contract to be
// callable from any thread with `user_data`, as a C provider that serves
// a threaded host must be; the pointer is opaque and never dereferenced
// here.
unsafe impl Send for VtableProvider {}
// SAFETY: as above; the provider serialises its own state.
unsafe impl Sync for VtableProvider {}

impl core::fmt::Debug for VtableProvider {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VtableProvider")
            .field("abi_version", &self.vtable.abi_version)
            .field("identity", &self.capabilities.identity.name)
            .finish_non_exhaustive()
    }
}

impl VtableProvider {
    /// Binds a vtable.
    ///
    /// # Safety
    ///
    /// `vtable` and `user_data` must come from a provider that stays alive
    /// and valid for as long as the returned value is used, whose functions
    /// honour the layout and ownership rules of the C structs above and
    /// may be called from any thread.
    ///
    /// # Errors
    ///
    /// A wrong ABI version or size, a missing `positions` or
    /// `capabilities` function, or a failing capabilities call.
    pub unsafe fn bind(
        vtable: ProviderVtable,
        user_data: *mut c_void,
    ) -> Result<VtableProvider, ProviderError> {
        if vtable.struct_size != size_of_u32::<ProviderVtable>()
            || vtable.abi_version != VTABLE_ABI_VERSION
        {
            return Err(ProviderError::invalid(format!(
                "vtable size {} version {}; this port is version {VTABLE_ABI_VERSION}",
                vtable.struct_size, vtable.abi_version
            )));
        }
        let (Some(capabilities_fn), Some(_)) = (vtable.capabilities, vtable.positions) else {
            return Err(ProviderError::invalid(
                "a vtable needs capabilities and positions",
            ));
        };
        let mut raw = CapabilitiesC {
            struct_size: size_of_u32::<CapabilitiesC>(),
            speeds: 0,
            deterministic: 0,
            tier: 0,
            distance_unit: 0,
            speed_model: 0,
            astronomy: 0,
            reserved: [0; 2],
            name: ptr::null(),
            version: ptr::null(),
            data_version: ptr::null(),
            jd_min: 0.0,
            jd_max: 0.0,
            bodies: ptr::null(),
            body_count: 0,
            native_frame_bits: 0,
            overrides: 0,
            ayanamshas: ptr::null(),
            ayanamsha_count: 0,
            hashes: ptr::null(),
            hash_count: 0,
        };
        // SAFETY: the caller guarantees the vtable and user_data; `raw` is a
        // valid, writable struct with its size set.
        check(
            unsafe { capabilities_fn(user_data, &raw mut raw) },
            "capabilities",
        )?;
        // SAFETY: the provider promises every pointer it wrote is owned by
        // it, NUL-terminated where a string, and sized as it says.
        let capabilities = unsafe { capabilities_from_c(&raw) }?;
        Ok(VtableProvider {
            vtable,
            user_data,
            capabilities,
        })
    }
}

/// # Safety
///
/// Every pointer in `raw` must be valid as the struct documents.
unsafe fn capabilities_from_c(raw: &CapabilitiesC) -> Result<Capabilities, ProviderError> {
    let text = |p: *const c_char| {
        if p.is_null() {
            String::new()
        } else {
            // SAFETY: the provider promises a NUL-terminated string.
            unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
        }
    };
    // SAFETY: the provider promises `body_count` readable ids.
    let bodies = unsafe { slice_or_empty(raw.bodies, raw.body_count) }
        .iter()
        .filter_map(|id| Body::from_id(*id))
        .collect();
    // SAFETY: the provider promises `ayanamsha_count` readable ids.
    let ayanamshas = unsafe { slice_or_empty(raw.ayanamshas, raw.ayanamsha_count) }
        .iter()
        .filter_map(|id| Ayanamsha::from_id(*id))
        .collect();
    // SAFETY: the provider promises `hash_count` readable entries.
    let data_hashes = unsafe { slice_or_empty(raw.hashes, raw.hash_count) }
        .iter()
        .map(|h| DataHash {
            file: text(h.file),
            sha256: text(h.sha256),
            bytes: h.bytes,
        })
        .collect();
    let distance_unit = DistanceUnit::from_id(raw.distance_unit)
        .ok_or_else(|| ProviderError::invalid(format!("distance unit id {}", raw.distance_unit)))?;
    let speed_model = SpeedModel::from_id(raw.speed_model)
        .ok_or_else(|| ProviderError::invalid(format!("speed model id {}", raw.speed_model)))?;
    let astronomy = Astronomy::from_id(raw.astronomy)
        .ok_or_else(|| ProviderError::invalid(format!("astronomy id {}", raw.astronomy)))?;
    Ok(Capabilities {
        identity: Identity {
            name: text(raw.name),
            version: text(raw.version),
            data_version: text(raw.data_version),
            tier: tier_from_bits(u32::from(raw.tier)),
            data_hashes,
        },
        jd_range: (raw.jd_min, raw.jd_max),
        bodies,
        native_frame: Frame::try_from_bits(raw.native_frame_bits)?,
        astronomy,
        speeds: raw.speeds != 0,
        speed_model,
        distance_unit,
        overrides: Overrides::from_bits(raw.overrides),
        ayanamshas,
        deterministic: raw.deterministic != 0,
    })
}

impl EphemerisProvider for VtableProvider {
    fn capabilities(&self) -> Capabilities {
        self.capabilities.clone()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let Some(positions_fn) = self.vtable.positions else {
            return Err(ProviderError::unsupported("positions"));
        };
        let ids: Vec<u16> = request.bodies.iter().map(|b| b.id()).collect();
        let raw_request = PositionRequestC {
            struct_size: size_of_u32::<PositionRequestC>(),
            scale: request.scale.id(),
            frame_bits: request.frame.to_bits(),
            speeds: u8::from(request.speeds),
            has_observer: u8::from(request.observer.is_some()),
            reserved: [0; 2],
            observer: request.observer.map_or(ObserverC::default(), ObserverC::of),
            jds: request.jds.as_ptr(),
            jd_count: request.jds.len(),
            bodies: ids.as_ptr(),
            body_count: ids.len(),
        };
        let mut columns =
            PositionColumns::new(request.jds.len(), request.bodies.len(), request.frame);
        let mut status_codes = vec![CellStatus::NotComputed.code(); columns.len()];
        let mut sources = vec![0u32; columns.len()];
        let mut out = PositionColumnsC {
            struct_size: size_of_u32::<PositionColumnsC>(),
            frame_bits: request.frame.to_bits(),
            lon: columns.lon.as_mut_ptr(),
            lat: columns.lat.as_mut_ptr(),
            dist: columns.dist.as_mut_ptr(),
            lon_speed: columns.lon_speed.as_mut_ptr(),
            lat_speed: columns.lat_speed.as_mut_ptr(),
            dist_speed: columns.dist_speed.as_mut_ptr(),
            status: status_codes.as_mut_ptr(),
            source: sources.as_mut_ptr(),
            capacity: columns.len(),
        };
        // SAFETY: the request's slices outlive the call; every out array
        // holds exactly `capacity` cells and outlives the call.
        check(
            unsafe { positions_fn(self.user_data, &raw const raw_request, &raw mut out) },
            "positions",
        )?;
        columns.frame = Frame::try_from_bits(out.frame_bits)?;
        for (slot, code) in columns.status.iter_mut().zip(&status_codes) {
            *slot = CellStatus::from_code(*code);
        }
        for (slot, bits) in columns.source.iter_mut().zip(&sources) {
            *slot = Source::from_bits(*bits);
        }
        Ok(columns)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        let Some(f) = self.vtable.obliquity else {
            return Err(ProviderError::unsupported("obliquity"));
        };
        let mut out = ObliquityC::default();
        // SAFETY: `out` is a valid, writable struct for the call.
        check(
            unsafe { f(self.user_data, jd, scale.id(), &raw mut out) },
            "obliquity",
        )?;
        Ok(Obliquity {
            mean_deg: out.mean_deg,
            true_deg: out.true_deg,
            nutation_lon_deg: out.nutation_lon_deg,
            nutation_obl_deg: out.nutation_obl_deg,
        })
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        let Some(f) = self.vtable.delta_t else {
            return Err(ProviderError::unsupported("delta_t"));
        };
        let mut out = 0.0f64;
        // SAFETY: `out` is a valid, writable f64 for the call.
        check(
            unsafe { f(self.user_data, jd_ut1, &raw mut out) },
            "delta_t",
        )?;
        Ok(out)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        let Some(f) = self.vtable.ayanamsha else {
            return Err(ProviderError::unsupported("ayanamsha"));
        };
        let mut out = 0.0f64;
        // SAFETY: `out` is a valid, writable f64 for the call.
        check(
            unsafe { f(self.user_data, jd, scale.id(), ayanamsha.id(), &raw mut out) },
            "ayanamsha",
        )?;
        Ok(out)
    }

    fn dut1_seconds(&self, jd_utc: f64) -> Result<f64, ProviderError> {
        let Some(f) = self.vtable.dut1 else {
            return Err(ProviderError::unsupported("dut1"));
        };
        let mut out = 0.0f64;
        // SAFETY: `out` is a valid, writable f64 for the call.
        check(unsafe { f(self.user_data, jd_utc, &raw mut out) }, "dut1")?;
        Ok(out)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        let Some(f) = self.vtable.horizon_event else {
            return Err(ProviderError::unsupported("rise_set"));
        };
        let raw = HorizonRequestC {
            struct_size: size_of_u32::<HorizonRequestC>(),
            body: request.body.id(),
            kind: request.kind.id(),
            disc: request.horizon.disc.id(),
            refraction: request.horizon.refraction.id(),
            reserved: [0; 7],
            observer: ObserverC::of(request.place),
            from_jd_ut1: request.from.get(),
            window_days: request.window_days,
            altitude_deg: request.horizon.altitude_deg,
        };
        let mut jd = 0.0f64;
        let mut found = 0u8;
        // SAFETY: `raw` outlives the call; the two outs are valid and
        // writable.
        check(
            unsafe { f(self.user_data, &raw const raw, &raw mut jd, &raw mut found) },
            "rise_set",
        )?;
        if found == 0 {
            return Ok(None);
        }
        JulianDay::try_new(jd)
            .map(Some)
            .map_err(|error| ProviderError::invalid(error.to_string()))
    }

    fn crossings(&self, request: &CrossingRequest) -> Result<Vec<Event>, ProviderError> {
        let Some(f) = self.vtable.crossings else {
            return Err(ProviderError::unsupported("crossings"));
        };
        let (first, second) = request.quantity.bodies();
        let (coefficient_a, coefficient_b) = match request.quantity {
            Quantity::Composite { a, b, .. } => (a, b),
            Quantity::Longitude(_) | Quantity::Speed(_) => (0.0, 0.0),
        };
        let raw = CrossingRequestC {
            struct_size: size_of_u32::<CrossingRequestC>(),
            frame_bits: request.frame.to_bits(),
            quantity_kind: request.quantity.kind_id(),
            has_observer: u8::from(request.observer.is_some()),
            reserved: [0; 2],
            first_body: first.id(),
            second_body: second.map_or(0, Body::id),
            coefficient_a,
            coefficient_b,
            origin_deg: request.lattice.origin_deg,
            step_deg: request.lattice.step_deg,
            from_jd_ut1: request.from.get(),
            to_jd_ut1: request.to.get(),
            tolerance_days: request.tolerance_days,
            observer: request
                .observer
                .map_or_else(ObserverC::default, ObserverC::of),
        };
        // A buffer that grows to what the provider reports: two rounds at
        // most for any window.
        let mut capacity = 64u32;
        loop {
            let mut buffer = vec![CrossingEventC::default(); capacity as usize];
            let mut count = 0u32;
            // SAFETY: `raw` outlives the call; the buffer holds `capacity`
            // writable events and `count` is a valid out.
            check(
                unsafe {
                    f(
                        self.user_data,
                        &raw const raw,
                        buffer.as_mut_ptr(),
                        capacity,
                        &raw mut count,
                    )
                },
                "crossings",
            )?;
            if count <= capacity {
                buffer.truncate(count as usize);
                return buffer.into_iter().map(CrossingEventC::event).collect();
            }
            capacity = count;
        }
    }
}

// ── Presenting a Rust provider as a vtable ─────────────────────────────────

/// A Rust provider behind a vtable: owns the provider and the C-side
/// copies of its capabilities, so a binding or a native caller can drive
/// it through [`ProviderVtable`].
pub struct Exported<P: EphemerisProvider> {
    provider: P,
    name: CString,
    version: CString,
    data_version: CString,
    bodies: Vec<u16>,
    ayanamshas: Vec<u16>,
    hash_strings: Vec<(CString, CString)>,
    hashes: Vec<DataHashC>,
    capabilities: Capabilities,
}

// SAFETY: the raw pointers inside point into the box's own vectors and
// strings, which never move while the box lives; the provider is `Sync`.
unsafe impl<P: EphemerisProvider> Send for Exported<P> {}
// SAFETY: as above.
unsafe impl<P: EphemerisProvider> Sync for Exported<P> {}

impl<P: EphemerisProvider> core::fmt::Debug for Exported<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Exported")
            .field("identity", &self.capabilities.identity.name)
            .finish_non_exhaustive()
    }
}

impl<P: EphemerisProvider> Exported<P> {
    /// Wraps a provider; the box keeps its address stable for `user_data`.
    #[must_use]
    pub fn new(provider: P) -> Box<Exported<P>> {
        let capabilities = provider.capabilities();
        let c = |s: &str| CString::new(s.replace('\0', " ")).unwrap_or_default();
        let hash_strings: Vec<(CString, CString)> = capabilities
            .identity
            .data_hashes
            .iter()
            .map(|h| (c(&h.file), c(&h.sha256)))
            .collect();
        let mut exported = Box::new(Exported {
            name: c(&capabilities.identity.name),
            version: c(&capabilities.identity.version),
            data_version: c(&capabilities.identity.data_version),
            bodies: capabilities.bodies.iter().map(|b| b.id()).collect(),
            ayanamshas: capabilities.ayanamshas.iter().map(|a| a.id()).collect(),
            hash_strings,
            hashes: Vec::new(),
            capabilities,
            provider,
        });
        exported.hashes = exported
            .hash_strings
            .iter()
            .zip(&exported.capabilities.identity.data_hashes)
            .map(|((file, sha), h)| DataHashC {
                file: file.as_ptr(),
                sha256: sha.as_ptr(),
                bytes: h.bytes,
            })
            .collect();
        exported
    }

    /// The vtable for this provider type.
    #[must_use]
    pub fn vtable() -> ProviderVtable {
        ProviderVtable {
            struct_size: size_of_u32::<ProviderVtable>(),
            abi_version: VTABLE_ABI_VERSION,
            capabilities: Some(capabilities_trampoline::<P>),
            positions: Some(positions_trampoline::<P>),
            obliquity: Some(obliquity_trampoline::<P>),
            delta_t: Some(delta_t_trampoline::<P>),
            ayanamsha: Some(ayanamsha_trampoline::<P>),
            dut1: Some(dut1_trampoline::<P>),
            horizon_event: Some(horizon_event_trampoline::<P>),
            crossings: Some(crossings_trampoline::<P>),
        }
    }

    /// The `user_data` to pass with [`Exported::vtable`]: this box.
    #[must_use]
    pub fn user_data(&self) -> *mut c_void {
        ptr::from_ref::<Exported<P>>(self).cast_mut().cast()
    }

    /// This provider driven through its own vtable, borrowed for as long
    /// as the box lives: the safe way to measure or test the C path.
    ///
    /// # Errors
    ///
    /// When the vtable does not bind, which cannot happen for a vtable this
    /// type produced.
    pub fn bound(&self) -> Result<ExportedVtable<'_>, ProviderError> {
        // SAFETY: the vtable is this type's own and `user_data` is this box,
        // which the returned lifetime keeps alive.
        let provider = unsafe { VtableProvider::bind(Exported::<P>::vtable(), self.user_data()) }?;
        Ok(ExportedVtable {
            provider,
            owner: core::marker::PhantomData,
        })
    }

    /// The provider inside.
    pub fn provider(&self) -> &P {
        &self.provider
    }
}

unsafe extern "C" fn capabilities_trampoline<P: EphemerisProvider>(
    user_data: *mut c_void,
    out: *mut CapabilitiesC,
) -> i32 {
    if user_data.is_null() || out.is_null() {
        return invalid_call();
    }
    // SAFETY: `user_data` is the `Exported` box registered with this vtable.
    let this = unsafe { &*user_data.cast::<Exported<P>>() };
    // SAFETY: the caller promises a writable struct with its size set.
    if unsafe { (*out).struct_size } != size_of_u32::<CapabilitiesC>() {
        return invalid_call();
    }
    let caps = &this.capabilities;
    let filled = CapabilitiesC {
        struct_size: size_of_u32::<CapabilitiesC>(),
        speeds: u8::from(caps.speeds),
        deterministic: u8::from(caps.deterministic),
        tier: u8::try_from(tier_bits(caps.identity.tier)).unwrap_or(0),
        distance_unit: caps.distance_unit.id(),
        speed_model: caps.speed_model.id(),
        astronomy: caps.astronomy.id(),
        reserved: [0; 2],
        name: this.name.as_ptr(),
        version: this.version.as_ptr(),
        data_version: this.data_version.as_ptr(),
        jd_min: caps.jd_range.0,
        jd_max: caps.jd_range.1,
        bodies: this.bodies.as_ptr(),
        body_count: this.bodies.len(),
        native_frame_bits: caps.native_frame.to_bits(),
        overrides: caps.overrides.bits(),
        ayanamshas: this.ayanamshas.as_ptr(),
        ayanamsha_count: this.ayanamshas.len(),
        hashes: this.hashes.as_ptr(),
        hash_count: this.hashes.len(),
    };
    // SAFETY: as above.
    unsafe { out.write(filled) };
    0
}

unsafe extern "C" fn positions_trampoline<P: EphemerisProvider>(
    user_data: *mut c_void,
    request: *const PositionRequestC,
    out: *mut PositionColumnsC,
) -> i32 {
    if user_data.is_null() || request.is_null() || out.is_null() {
        return invalid_call();
    }
    // SAFETY: `user_data` is the `Exported` box registered with this vtable;
    // the caller promises readable request and writable columns structs.
    let (this, req, columns) = unsafe { (&*user_data.cast::<Exported<P>>(), &*request, &mut *out) };
    if req.struct_size != size_of_u32::<PositionRequestC>()
        || columns.struct_size != size_of_u32::<PositionColumnsC>()
    {
        return invalid_call();
    }
    let Some(scale) = TimeScale::from_id(req.scale) else {
        return invalid_call();
    };
    let frame = match Frame::try_from_bits(req.frame_bits) {
        Ok(frame) => frame,
        Err(error) => return error.code(),
    };
    let observer = if req.has_observer == 0 {
        None
    } else {
        match req.observer.place() {
            Ok(place) => Some(place),
            Err(error) => return error.code(),
        }
    };
    // SAFETY: the caller promises `jd_count` instants and `body_count` ids.
    let (jds, ids) = unsafe {
        (
            slice_or_empty(req.jds, req.jd_count),
            slice_or_empty(req.bodies, req.body_count),
        )
    };
    let bodies: Vec<Body> = ids.iter().filter_map(|id| Body::from_id(*id)).collect();
    if bodies.len() != ids.len() {
        return ProviderError::unsupported("body id").code();
    }
    let needed = jds.len().saturating_mul(bodies.len());
    if columns.capacity < needed {
        return invalid_call();
    }
    let request = PositionRequest {
        jds,
        scale,
        bodies: &bodies,
        frame,
        observer,
        speeds: req.speeds != 0,
    };
    let result = match this.provider.positions(&request) {
        Ok(result) => result,
        Err(error) => return error.code(),
    };
    columns.frame_bits = result.frame.to_bits();
    // SAFETY: the caller promises every out array holds `capacity` cells,
    // and `needed <= capacity` was checked.
    unsafe {
        copy_column(columns.lon, &result.lon);
        copy_column(columns.lat, &result.lat);
        copy_column(columns.dist, &result.dist);
        copy_column(columns.lon_speed, &result.lon_speed);
        copy_column(columns.lat_speed, &result.lat_speed);
        copy_column(columns.dist_speed, &result.dist_speed);
        for (i, cell) in result.cells().enumerate() {
            columns.status.add(i).write(cell.status.code());
            columns.source.add(i).write(cell.source.to_bits());
        }
    }
    0
}

/// # Safety
///
/// `ptr` must be null or valid for `len` reads.
unsafe fn slice_or_empty<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        // SAFETY: as documented.
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }
}

/// # Safety
///
/// `dst` must be valid for `src.len()` writes.
unsafe fn copy_column(dst: *mut f64, src: &[f64]) {
    if !dst.is_null() {
        // SAFETY: as documented.
        unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len()) };
    }
}

unsafe extern "C" fn obliquity_trampoline<P: EphemerisProvider>(
    user_data: *mut c_void,
    jd: f64,
    scale: u32,
    out: *mut ObliquityC,
) -> i32 {
    if user_data.is_null() || out.is_null() {
        return invalid_call();
    }
    let Some(scale) = TimeScale::from_id(scale) else {
        return invalid_call();
    };
    // SAFETY: `user_data` is the `Exported` box registered with this vtable.
    let this = unsafe { &*user_data.cast::<Exported<P>>() };
    status(this.provider.obliquity(jd, scale).map(|o| {
        // SAFETY: the caller promises a writable struct.
        unsafe {
            out.write(ObliquityC {
                mean_deg: o.mean_deg,
                true_deg: o.true_deg,
                nutation_lon_deg: o.nutation_lon_deg,
                nutation_obl_deg: o.nutation_obl_deg,
            });
        }
    }))
}

unsafe extern "C" fn delta_t_trampoline<P: EphemerisProvider>(
    user_data: *mut c_void,
    jd_ut1: f64,
    out: *mut f64,
) -> i32 {
    if user_data.is_null() || out.is_null() {
        return invalid_call();
    }
    // SAFETY: `user_data` is the `Exported` box registered with this vtable.
    let this = unsafe { &*user_data.cast::<Exported<P>>() };
    // SAFETY: the caller promises a writable f64.
    status(
        this.provider
            .delta_t_seconds(jd_ut1)
            .map(|v| unsafe { out.write(v) }),
    )
}

unsafe extern "C" fn dut1_trampoline<P: EphemerisProvider>(
    user_data: *mut c_void,
    jd_utc: f64,
    out: *mut f64,
) -> i32 {
    if user_data.is_null() || out.is_null() {
        return invalid_call();
    }
    // SAFETY: `user_data` is the `Exported` box registered with this vtable.
    let this = unsafe { &*user_data.cast::<Exported<P>>() };
    // SAFETY: the caller promises a writable f64.
    status(
        this.provider
            .dut1_seconds(jd_utc)
            .map(|v| unsafe { out.write(v) }),
    )
}

unsafe extern "C" fn ayanamsha_trampoline<P: EphemerisProvider>(
    user_data: *mut c_void,
    jd: f64,
    scale: u32,
    ayanamsha: u16,
    out: *mut f64,
) -> i32 {
    if user_data.is_null() || out.is_null() {
        return invalid_call();
    }
    let (Some(scale), Some(ayanamsha)) = (TimeScale::from_id(scale), Ayanamsha::from_id(ayanamsha))
    else {
        return invalid_call();
    };
    // SAFETY: `user_data` is the `Exported` box registered with this vtable.
    let this = unsafe { &*user_data.cast::<Exported<P>>() };
    // SAFETY: the caller promises a writable f64.
    status(
        this.provider
            .ayanamsha_deg(jd, scale, ayanamsha)
            .map(|v| unsafe { out.write(v) }),
    )
}

unsafe extern "C" fn horizon_event_trampoline<P: EphemerisProvider>(
    user_data: *mut c_void,
    request: *const HorizonRequestC,
    out_jd_ut1: *mut f64,
    out_found: *mut u8,
) -> i32 {
    if user_data.is_null() || request.is_null() || out_jd_ut1.is_null() || out_found.is_null() {
        return invalid_call();
    }
    // SAFETY: `user_data` is the `Exported` box registered with this vtable;
    // the caller promises a readable request.
    let (this, raw) = unsafe { (&*user_data.cast::<Exported<P>>(), &*request) };
    if raw.struct_size != size_of_u32::<HorizonRequestC>() {
        return invalid_call();
    }
    let (Some(body), Some(kind), Some(disc), Some(refraction)) = (
        Body::from_id(raw.body),
        HorizonEventKind::from_id(raw.kind),
        DiscPoint::from_id(raw.disc),
        Refraction::from_id(raw.refraction),
    ) else {
        return invalid_call();
    };
    let (Ok(place), Ok(from)) = (raw.observer.place(), JulianDay::try_new(raw.from_jd_ut1)) else {
        return invalid_call();
    };
    let request = HorizonRequest {
        body,
        kind,
        place,
        from,
        window_days: raw.window_days,
        horizon: Horizon {
            disc,
            refraction,
            altitude_deg: raw.altitude_deg,
        },
    };
    status(this.provider.horizon_event(&request).map(|found| {
        // SAFETY: the caller promises two writable outs.
        unsafe {
            out_jd_ut1.write(found.map_or(0.0, JulianDay::get));
            out_found.write(u8::from(found.is_some()));
        }
    }))
}

unsafe extern "C" fn crossings_trampoline<P: EphemerisProvider>(
    user_data: *mut c_void,
    request: *const CrossingRequestC,
    out_events: *mut CrossingEventC,
    capacity: u32,
    out_count: *mut u32,
) -> i32 {
    if user_data.is_null()
        || request.is_null()
        || out_count.is_null()
        || (out_events.is_null() && capacity > 0)
    {
        return invalid_call();
    }
    // SAFETY: `user_data` is the `Exported` box registered with this vtable;
    // the caller promises a readable request.
    let (this, raw) = unsafe { (&*user_data.cast::<Exported<P>>(), &*request) };
    if raw.struct_size != size_of_u32::<CrossingRequestC>() {
        return invalid_call();
    }
    let Some(first) = Body::from_id(raw.first_body) else {
        return invalid_call();
    };
    let second = (raw.quantity_kind == 2).then(|| Body::from_id(raw.second_body));
    let second = match second {
        Some(None) => return invalid_call(),
        Some(Some(body)) => Some(body),
        None => None,
    };
    let Some(quantity) = Quantity::from_parts(
        raw.quantity_kind,
        first,
        second,
        raw.coefficient_a,
        raw.coefficient_b,
    ) else {
        return invalid_call();
    };
    let (Ok(frame), Ok(from), Ok(to)) = (
        Frame::try_from_bits(raw.frame_bits),
        JulianDay::try_new(raw.from_jd_ut1),
        JulianDay::try_new(raw.to_jd_ut1),
    ) else {
        return invalid_call();
    };
    let observer = if raw.has_observer == 0 {
        None
    } else {
        match raw.observer.place() {
            Ok(place) => Some(place),
            Err(_) => return invalid_call(),
        }
    };
    let request = CrossingRequest {
        quantity,
        lattice: Lattice {
            origin_deg: raw.origin_deg,
            step_deg: raw.step_deg,
        },
        from,
        to,
        tolerance_days: raw.tolerance_days,
        frame,
        observer,
    };
    status(this.provider.crossings(&request).map(|events| {
        // SAFETY: the caller promises a writable count and a buffer of
        // `capacity` events.
        unsafe {
            out_count.write(u32::try_from(events.len()).unwrap_or(u32::MAX));
            for (index, event) in events.iter().take(capacity as usize).enumerate() {
                out_events.add(index).write(CrossingEventC::of(event));
            }
        }
    }))
}

/// An [`Exported`] provider seen through its vtable, tied to the box's
/// lifetime so the pointers inside stay valid.
pub struct ExportedVtable<'a> {
    provider: VtableProvider,
    owner: core::marker::PhantomData<&'a ()>,
}

impl core::fmt::Debug for ExportedVtable<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.provider.fmt(f)
    }
}

impl EphemerisProvider for ExportedVtable<'_> {
    fn capabilities(&self) -> Capabilities {
        self.provider.capabilities()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        self.provider.positions(request)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        self.provider.obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        self.provider.delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        self.provider.ayanamsha_deg(jd, scale, ayanamsha)
    }

    fn dut1_seconds(&self, jd_utc: f64) -> Result<f64, ProviderError> {
        self.provider.dut1_seconds(jd_utc)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        self.provider.horizon_event(request)
    }

    fn crossings(&self, request: &CrossingRequest) -> Result<Vec<Event>, ProviderError> {
        self.provider.crossings(request)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::{Altitude, Latitude, Longitude};

    use super::*;
    use crate::frame::Zodiac;
    use crate::test_provider::TestProvider;

    #[test]
    fn a_rust_provider_round_trips_through_its_vtable() {
        let exported = Exported::new(TestProvider::new());
        // SAFETY: the box outlives the bound provider in this test.
        let bound = unsafe {
            VtableProvider::bind(Exported::<TestProvider>::vtable(), exported.user_data())
        }
        .unwrap();
        let direct = exported.provider().capabilities();
        assert_eq!(bound.capabilities(), direct);
        let jds = [2_460_000.5, 2_460_100.5];
        let request = PositionRequest::new(
            &jds,
            TimeScale::Ut1,
            &[Body::Sun, Body::Moon],
            Frame::CANONICAL,
        );
        let via_vtable = bound.positions(&request).unwrap();
        let via_trait = exported.provider().positions(&request).unwrap();
        assert!(via_vtable.bit_identical(&via_trait));
        assert_eq!(via_vtable.frame, via_trait.frame);
        assert!(bound.obliquity(2_460_000.5, TimeScale::Tt).is_err());
        assert!(bound.delta_t_seconds(2_460_000.5).is_err());
        assert!(bound.dut1_seconds(2_460_000.5).is_err());
        assert!(
            bound
                .ayanamsha_deg(2_460_000.5, TimeScale::Tt, Ayanamsha::Lahiri)
                .is_err()
        );
        let place = Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        );
        let horizon = HorizonRequest {
            body: Body::Sun,
            kind: HorizonEventKind::Rise,
            place,
            from: JulianDay::literal(2_460_000.5),
            window_days: 1.0,
            horizon: Horizon::CENTRE_NO_REFRACTION,
        };
        assert!(matches!(
            bound.horizon_event(&horizon),
            Err(ProviderError::Unsupported { .. })
        ));
        let crossings = CrossingRequest {
            quantity: Quantity::Longitude(Body::Sun),
            lattice: Lattice::SIGNS,
            from: JulianDay::literal(2_460_000.5),
            to: JulianDay::literal(2_460_030.5),
            tolerance_days: 1e-7,
            frame: Frame::CANONICAL,
            observer: None,
        };
        assert!(matches!(
            bound.crossings(&crossings),
            Err(ProviderError::Unsupported { .. })
        ));
        let through_box = exported.bound().unwrap();
        assert!(
            through_box
                .positions(&request)
                .unwrap()
                .bit_identical(&via_trait)
        );
        assert!(format!("{through_box:?}").contains("test-provider"));
    }

    /// The test provider with a crossings override that answers one event
    /// a day, so a window longer than the first buffer makes the caller
    /// come back for more.
    struct DailyCrossings(TestProvider);

    impl EphemerisProvider for DailyCrossings {
        fn capabilities(&self) -> Capabilities {
            let mut capabilities = self.0.capabilities();
            capabilities.overrides = capabilities.overrides.with(Overrides::CROSSINGS);
            capabilities
        }

        fn positions(
            &self,
            request: &PositionRequest<'_>,
        ) -> Result<PositionColumns, ProviderError> {
            self.0.positions(request)
        }

        fn crossings(&self, request: &CrossingRequest) -> Result<Vec<Event>, ProviderError> {
            if request.observer.is_some() {
                return Err(ProviderError::unsupported("crossings: topocentric"));
            }
            let mut events = Vec::new();
            let mut day = request.from.get().ceil();
            let mut k = 0i64;
            while day < request.to.get() {
                events.push(Event {
                    instant: JulianDay::literal(day),
                    boundary_deg: request.lattice.line(k),
                    direction: if k % 2 == 0 {
                        Direction::Rising
                    } else {
                        Direction::Falling
                    },
                    evaluations: 0,
                });
                day += 1.0;
                k += 1;
            }
            Ok(events)
        }
    }

    #[test]
    fn a_crossings_override_round_trips_through_its_vtable_whatever_the_count() {
        let exported = Exported::new(DailyCrossings(TestProvider::new()));
        // SAFETY: the box outlives the bound provider in this test.
        let bound = unsafe {
            VtableProvider::bind(Exported::<DailyCrossings>::vtable(), exported.user_data())
        }
        .unwrap();
        assert!(bound.capabilities().has(Overrides::CROSSINGS));
        for days in [10u32, 64, 200] {
            let request = CrossingRequest {
                quantity: Quantity::ELONGATION,
                lattice: Lattice::TITHIS,
                from: JulianDay::literal(2_460_000.5),
                to: JulianDay::literal(2_460_000.5 + f64::from(days)),
                tolerance_days: 1e-7,
                frame: Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::Lahiri)),
                observer: None,
            };
            let via_vtable = bound.crossings(&request).unwrap();
            let direct = exported.provider().crossings(&request).unwrap();
            assert_eq!(via_vtable, direct);
            assert_eq!(via_vtable.len(), usize::try_from(days).unwrap());
        }
        let topocentric = CrossingRequest {
            quantity: Quantity::Longitude(Body::Moon),
            lattice: Lattice::NAKSHATRAS,
            from: JulianDay::literal(2_460_000.5),
            to: JulianDay::literal(2_460_001.5),
            tolerance_days: 1e-7,
            frame: Frame::CANONICAL,
            observer: Some(Place::new(
                Latitude::literal(27.7172),
                Longitude::literal(85.324),
                Altitude::literal(1400.0),
            )),
        };
        assert!(matches!(
            bound.crossings(&topocentric),
            Err(ProviderError::Unsupported { .. })
        ));
    }

    #[test]
    fn a_wrong_abi_or_a_bad_observer_is_refused() {
        let exported = Exported::new(TestProvider::new());
        let mut vtable = Exported::<TestProvider>::vtable();
        vtable.abi_version += 1;
        // SAFETY: the box outlives the call.
        let error = unsafe { VtableProvider::bind(vtable, exported.user_data()) }.unwrap_err();
        assert!(matches!(error, ProviderError::Invalid { .. }));
        let mut without = Exported::<TestProvider>::vtable();
        without.positions = None;
        // SAFETY: as above.
        assert!(unsafe { VtableProvider::bind(without, exported.user_data()) }.is_err());
        assert!(
            ObserverC {
                longitude_deg: 200.0,
                latitude_deg: 0.0,
                altitude_m: 0.0
            }
            .place()
            .is_err()
        );
    }
}

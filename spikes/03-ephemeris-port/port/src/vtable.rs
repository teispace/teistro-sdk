//! The port as a C vtable, so a native engine (Teimeris), a Swiss shim and
//! a binding's host object are one shape to the SDK. Two directions:
//! [`VtableProvider`] drives any vtable through the Rust trait, and
//! [`Exported`] presents any Rust provider as a vtable. The crate's only
//! unsafe code lives here, one SAFETY comment per block.

#![allow(unsafe_code, reason = "the C boundary of the port")]

use core::ffi::{CStr, c_char, c_void};
use core::ptr;
use std::ffi::CString;

use crate::model::{
    AyanamshaId, Body, Capabilities, CellStatus, DataHash, Frame, Identity, Obliquity, Observer,
    Overrides, PositionColumns, PositionRequest, ProviderError, Source, TimeScale,
};
use crate::provider::EphemerisProvider;

/// The ABI version of the vtable layout.
pub const VTABLE_ABI_VERSION: u32 = 1;

/// A C observer.
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
    /// Reserved, zero.
    pub reserved: u8,
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
    /// The ayanamsha ids the override knows.
    pub ayanamshas: *const u8,
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
    /// The ayanamsha override, degrees.
    pub ayanamsha: Option<
        unsafe extern "C" fn(
            user_data: *mut c_void,
            jd: f64,
            scale: u32,
            ayanamsha: u8,
            out_deg: *mut f64,
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

// ── Driving a vtable through the trait ─────────────────────────────────────

/// Any vtable as an [`EphemerisProvider`]. The capabilities are read once
/// at construction and owned in Rust; every call after that goes through
/// the function pointers.
pub struct VtableProvider {
    vtable: ProviderVtable,
    user_data: *mut c_void,
    capabilities: Capabilities,
}

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
    /// and valid for as long as the returned value is used, and the
    /// vtable's functions must honour the layout and ownership rules of the
    /// C structs above.
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
            return Err(ProviderError::Invalid {
                detail: format!(
                    "vtable size {} version {}",
                    vtable.struct_size, vtable.abi_version
                ),
            });
        }
        let (Some(capabilities_fn), Some(_)) = (vtable.capabilities, vtable.positions) else {
            return Err(ProviderError::Invalid {
                detail: String::from("a vtable needs capabilities and positions"),
            });
        };
        let mut raw = CapabilitiesC {
            struct_size: size_of_u32::<CapabilitiesC>(),
            speeds: 0,
            deterministic: 0,
            tier: 0,
            reserved: 0,
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
        let capabilities = unsafe { capabilities_from_c(&raw) };
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
unsafe fn capabilities_from_c(raw: &CapabilitiesC) -> Capabilities {
    let text = |p: *const c_char| {
        if p.is_null() {
            String::new()
        } else {
            // SAFETY: the provider promises a NUL-terminated string.
            unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
        }
    };
    let bodies = if raw.bodies.is_null() {
        Vec::new()
    } else {
        // SAFETY: the provider promises `body_count` readable ids.
        unsafe { core::slice::from_raw_parts(raw.bodies, raw.body_count) }
            .iter()
            .filter_map(|id| Body::from_id(*id))
            .collect()
    };
    let ayanamshas = if raw.ayanamshas.is_null() {
        Vec::new()
    } else {
        // SAFETY: the provider promises `ayanamsha_count` readable ids.
        unsafe { core::slice::from_raw_parts(raw.ayanamshas, raw.ayanamsha_count) }
            .iter()
            .map(|id| AyanamshaId(*id))
            .collect()
    };
    let data_hashes = if raw.hashes.is_null() {
        Vec::new()
    } else {
        // SAFETY: the provider promises `hash_count` readable entries.
        unsafe { core::slice::from_raw_parts(raw.hashes, raw.hash_count) }
            .iter()
            .map(|h| DataHash {
                file: text(h.file),
                sha256: text(h.sha256),
                bytes: h.bytes,
            })
            .collect()
    };
    Capabilities {
        identity: Identity {
            name: text(raw.name),
            version: text(raw.version),
            data_version: text(raw.data_version),
            tier: Source::from_bits(u32::from(raw.tier) << 8).tier,
            data_hashes,
        },
        jd_range: (raw.jd_min, raw.jd_max),
        bodies,
        native_frame: Frame::from_bits(raw.native_frame_bits),
        speeds: raw.speeds != 0,
        overrides: Overrides::from_bits(raw.overrides),
        ayanamshas,
        deterministic: raw.deterministic != 0,
    }
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
            observer: request
                .observer
                .map_or(ObserverC::default(), |o| ObserverC {
                    longitude_deg: o.longitude_deg,
                    latitude_deg: o.latitude_deg,
                    altitude_m: o.altitude_m,
                }),
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
        columns.frame = Frame::from_bits(out.frame_bits);
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
        id: AyanamshaId,
    ) -> Result<f64, ProviderError> {
        let Some(f) = self.vtable.ayanamsha else {
            return Err(ProviderError::unsupported("ayanamsha"));
        };
        let mut out = 0.0f64;
        // SAFETY: `out` is a valid, writable f64 for the call.
        check(
            unsafe { f(self.user_data, jd, scale.id(), id.0, &raw mut out) },
            "ayanamsha",
        )?;
        Ok(out)
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
    ayanamshas: Vec<u8>,
    hash_strings: Vec<(CString, CString)>,
    hashes: Vec<DataHashC>,
    capabilities: Capabilities,
}

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
            ayanamshas: capabilities.ayanamshas.iter().map(|a| a.0).collect(),
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
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
    }
    // SAFETY: `user_data` is the `Exported` box registered with this vtable.
    let this = unsafe { &*user_data.cast::<Exported<P>>() };
    // SAFETY: the caller promises a writable struct with its size set.
    if unsafe { (*out).struct_size } != size_of_u32::<CapabilitiesC>() {
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
    }
    let caps = &this.capabilities;
    let tier = Source {
        kind: crate::model::EphemerisKind::Unknown,
        tier: caps.identity.tier,
    }
    .to_bits()
        >> 8;
    let filled = CapabilitiesC {
        struct_size: size_of_u32::<CapabilitiesC>(),
        speeds: u8::from(caps.speeds),
        deterministic: u8::from(caps.deterministic),
        tier: u8::try_from(tier).unwrap_or(0),
        reserved: 0,
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
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
    }
    // SAFETY: `user_data` is the `Exported` box registered with this vtable;
    // the caller promises readable request and writable columns structs.
    let (this, req, columns) = unsafe { (&*user_data.cast::<Exported<P>>(), &*request, &mut *out) };
    if req.struct_size != size_of_u32::<PositionRequestC>()
        || columns.struct_size != size_of_u32::<PositionColumnsC>()
    {
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
    }
    let Some(scale) = TimeScale::from_id(req.scale) else {
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
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
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
    }
    let request = PositionRequest {
        jds,
        scale,
        bodies: &bodies,
        frame: Frame::from_bits(req.frame_bits),
        observer: (req.has_observer != 0).then_some(Observer {
            longitude_deg: req.observer.longitude_deg,
            latitude_deg: req.observer.latitude_deg,
            altitude_m: req.observer.altitude_m,
        }),
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
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
    }
    let Some(scale) = TimeScale::from_id(scale) else {
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
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
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
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

unsafe extern "C" fn ayanamsha_trampoline<P: EphemerisProvider>(
    user_data: *mut c_void,
    jd: f64,
    scale: u32,
    ayanamsha: u8,
    out: *mut f64,
) -> i32 {
    if user_data.is_null() || out.is_null() {
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
    }
    let Some(scale) = TimeScale::from_id(scale) else {
        return ProviderError::Invalid {
            detail: String::new(),
        }
        .code();
    };
    // SAFETY: `user_data` is the `Exported` box registered with this vtable.
    let this = unsafe { &*user_data.cast::<Exported<P>>() };
    // SAFETY: the caller promises a writable f64.
    status(
        this.provider
            .ayanamsha_deg(jd, scale, AyanamshaId(ayanamsha))
            .map(|v| unsafe { out.write(v) }),
    )
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
        id: AyanamshaId,
    ) -> Result<f64, ProviderError> {
        self.provider.ayanamsha_deg(jd, scale, id)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "a test fails by panicking")]

    use super::*;
    use crate::test_provider::SliceTestProvider;

    #[test]
    fn a_rust_provider_round_trips_through_its_vtable() {
        let exported = Exported::new(SliceTestProvider::new());
        // SAFETY: the box outlives the bound provider in this test.
        let bound = unsafe {
            VtableProvider::bind(
                Exported::<SliceTestProvider>::vtable(),
                exported.user_data(),
            )
        }
        .unwrap_or_else(|e| panic!("{e}"));
        let direct = exported.provider().capabilities();
        assert_eq!(bound.capabilities(), direct);
        let jds = [2_460_000.5, 2_460_100.5];
        let request = PositionRequest {
            jds: &jds,
            scale: TimeScale::Ut1,
            bodies: &[Body::Sun, Body::Moon],
            frame: Frame::CANONICAL,
            observer: None,
            speeds: true,
        };
        let via_vtable = bound.positions(&request).unwrap_or_else(|e| panic!("{e}"));
        let via_trait = exported
            .provider()
            .positions(&request)
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(via_vtable.bit_identical(&via_trait));
        assert_eq!(via_vtable.frame, via_trait.frame);
        assert!(bound.obliquity(2_460_000.5, TimeScale::Tt).is_err());
    }
}

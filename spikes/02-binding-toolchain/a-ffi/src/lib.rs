//! Spike 2, option A: the designed C ABI over the slice.
//!
//! The boundary follows the conventions the binding architecture wants
//! (`02-architecture/07-binding-architecture.md`): opaque handles, structs
//! with a `struct_size` handshake, unit-suffixed field names, a status code
//! on every call, a provider as a vtable plus `user_data`, and the tree
//! result as a length-prefixed blob with a table of contents that every
//! binding's decoder reads the same way. The API description of option A
//! is extracted from this file by `a-gen`; the `api:` lines in doc
//! comments carry units, ranges, examples and enum links (ADR-0023).
//!
//! Every `unsafe` block names what the caller guarantees; the C contract is
//! the doc comment of each function.

use core::cell::Cell;
use core::ffi::{c_char, c_void};
use core::ptr;

use teistro_spike_slice::{
    Ayanamsha, Body, BodyPosition, Chart, Context, DashaRow, EphemerisPort, Error, NodeKind,
    Position, Settings, TestProvider,
};

/// The ABI version the `tsp_abi_version` handshake returns.
pub const TSP_ABI_VERSION: u32 = 1;

/// The status of every call. `0` is success; everything else names the
/// failure, with the same numbers in every binding.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TspStatus {
    /// Success.
    Ok = 0,
    /// A null pointer, a wrong `struct_size`, or an enum index outside its
    /// range.
    InvalidArgument = -1,
    /// `dasha_depth` outside `1..=5`.
    DepthOutOfRange = -2,
    /// The Julian Day is NaN or infinite.
    JulianDayNotFinite = -3,
    /// The provider failed; `tsp_context_last_provider_code` has its code.
    Provider = -4,
    /// The provider returned a non-finite longitude.
    PositionNotFinite = -5,
}

impl TspStatus {
    const fn from_error(error: Error) -> TspStatus {
        match error {
            Error::DepthOutOfRange { .. } => TspStatus::DepthOutOfRange,
            Error::JulianDayNotFinite => TspStatus::JulianDayNotFinite,
            Error::Provider { .. } => TspStatus::Provider,
            Error::PositionNotFinite { .. } => TspStatus::PositionNotFinite,
        }
    }
}

/// The settings of a context.
///
/// `api: handshake=struct_size`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TspSettings {
    /// `sizeof(TspSettings)` as the caller compiled it; the library refuses
    /// a size it does not know.
    /// `api: role=struct_size`
    pub struct_size: u32,
    /// The ayanamsha: `0` Lahiri, `1` Raman, `2` Krishnamurti.
    /// `api: enum=TspAyanamsha example=0`
    pub ayanamsha: u8,
    /// The lunar node: `0` mean, `1` true.
    /// `api: enum=TspNodeKind example=0`
    pub node: u8,
    /// Dasha levels to build, `1` to `5`; the tree has `9^depth` leaves.
    /// `api: range=[1,5] example=3`
    pub dasha_depth: u8,
    /// Reserved, zero.
    pub reserved: u8,
}

/// The ayanamsha catalogue.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TspAyanamsha {
    /// Lahiri (Chitrapaksha), the default.
    Lahiri = 0,
    /// B. V. Raman.
    Raman = 1,
    /// K. S. Krishnamurti.
    Krishnamurti = 2,
}

/// Which lunar node the provider is asked for.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TspNodeKind {
    /// The mean node.
    Mean = 0,
    /// The true node.
    True = 1,
}

/// The nine bodies, the index every position column uses.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TspBody {
    /// The Sun.
    Sun = 0,
    /// The Moon.
    Moon = 1,
    /// Mars.
    Mars = 2,
    /// Mercury.
    Mercury = 3,
    /// Jupiter.
    Jupiter = 4,
    /// Venus.
    Venus = 5,
    /// Saturn.
    Saturn = 6,
    /// The ascending node.
    Rahu = 7,
    /// The descending node.
    Ketu = 8,
}

/// A tropical position, the provider's answer.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TspPosition {
    /// Tropical ecliptic longitude in degrees; any real number.
    /// `api: unit=deg example=280.46`
    pub longitude_deg: f64,
    /// Ecliptic latitude in degrees.
    /// `api: unit=deg range=[-90,90] example=0.0`
    pub latitude_deg: f64,
    /// Longitude speed in degrees per day; negative is retrograde.
    /// `api: unit=deg/day example=0.9856`
    pub speed_deg_per_day: f64,
}

/// The provider's one entry point: fill `out_position` for `body` at
/// `jd_ut` and return `0`, or return a non-zero code of the provider's own.
///
/// `api: callback=position`
pub type TspPositionFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    jd_ut: f64,
    body: u32,
    out_position: *mut TspPosition,
) -> i32;

/// A host-implemented ephemeris provider.
///
/// `api: handshake=struct_size`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TspProviderVtable {
    /// `sizeof(TspProviderVtable)` as the caller compiled it.
    /// `api: role=struct_size`
    pub struct_size: u32,
    /// The position callback; must not be null.
    pub position: Option<TspPositionFn>,
}

/// A computed chart, owned by the library until `tsp_blob_free`.
///
/// The bytes follow the `TSPB` layout: a 32-byte header (magic
/// `0x4250_5354`, version `1`, section count, total length), a table of
/// `{id, offset, length, count}` entries, then the sections. Section `1`
/// is the chart header (`jd_ut`, `ayanamsha_deg`, depth, counts), section
/// `2` the nine positions as columns, section `3` the dasha rows as
/// columns; each column section starts with a directory of column offsets
/// relative to the section start. Every multi-byte column is 8-aligned.
///
/// `api: blob=TSPB`
#[repr(C)]
#[derive(Debug)]
pub struct TspBlob {
    /// The bytes; null when the blob is empty.
    pub data: *mut u8,
    /// The number of bytes.
    pub len: usize,
    /// The allocation's capacity, needed to free it.
    pub cap: usize,
}

/// An opaque context: settings plus a provider. Not thread-safe; one per
/// thread, as the architecture requires.
pub struct TspContext {
    inner: Context,
    last_provider_code: Cell<i32>,
}

impl core::fmt::Debug for TspContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TspContext")
            .field("inner", &self.inner)
            .field("last_provider_code", &self.last_provider_code.get())
            .finish()
    }
}

/// The host's vtable seen from the slice as an [`EphemerisPort`].
struct HostProvider {
    position: TspPositionFn,
    user_data: *mut c_void,
}

impl EphemerisPort for HostProvider {
    #[allow(
        unsafe_code,
        reason = "the call into the host is the boundary this crate exists for"
    )]
    fn position(&self, jd_ut: f64, body: Body) -> Result<Position, i32> {
        let mut out = TspPosition {
            longitude_deg: 0.0,
            latitude_deg: 0.0,
            speed_deg_per_day: 0.0,
        };
        // SAFETY: `position` is the host's function pointer, checked non-null
        // in `tsp_context_new`; `user_data` is whatever the host registered
        // and is passed back untouched; `out` is a valid, writable position
        // for the duration of the call.
        let code = unsafe {
            (self.position)(self.user_data, jd_ut, u32::from(body.index()), &raw mut out)
        };
        if code == 0 {
            Ok(Position {
                longitude_deg: out.longitude_deg,
                latitude_deg: out.latitude_deg,
                speed_deg_per_day: out.speed_deg_per_day,
            })
        } else {
            Err(code)
        }
    }
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

/// The ABI version this library implements; a binding refuses to load a
/// library whose version it was not generated for.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "the exported-symbol attribute is unsafe in edition 2024"
)]
pub extern "C" fn tsp_abi_version() -> u32 {
    TSP_ABI_VERSION
}

/// Writes the default settings (Lahiri, the mean node, three levels) into
/// `out_settings`, whose `struct_size` must already be set by the caller.
///
/// Returns `InvalidArgument` for a null pointer or an unknown size.
///
/// # Safety
///
/// Every pointer must be null or valid for the access its documentation
/// describes, for the duration of the call; a handle must come from this
/// library and must not be used after `tsp_context_free`.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "reads and writes through caller pointers at the boundary"
)]
pub unsafe extern "C" fn tsp_settings_default(out_settings: *mut TspSettings) -> TspStatus {
    if out_settings.is_null() {
        return TspStatus::InvalidArgument;
    }
    // SAFETY: non-null, and the caller promises it points at a TspSettings
    // whose struct_size field is initialised.
    let size = unsafe { (*out_settings).struct_size };
    if size != size_of_u32::<TspSettings>() {
        return TspStatus::InvalidArgument;
    }
    let defaults = Settings::DEFAULT;
    // SAFETY: as above; the whole struct is written.
    unsafe {
        out_settings.write(TspSettings {
            struct_size: size,
            ayanamsha: defaults.ayanamsha.index(),
            node: defaults.node.index(),
            dasha_depth: defaults.dasha_depth,
            reserved: 0,
        });
    }
    TspStatus::Ok
}

/// Creates a context. `provider` may be null, which selects the built-in
/// analytic test provider; otherwise its `position` must be non-null and
/// `user_data` stays valid until `tsp_context_free`. On success
/// `*out_context` owns the context.
///
/// # Safety
///
/// Every pointer must be null or valid for the access its documentation
/// describes, for the duration of the call; a handle must come from this
/// library and must not be used after `tsp_context_free`.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "reads caller structs and writes the handle at the boundary"
)]
pub unsafe extern "C" fn tsp_context_new(
    settings: *const TspSettings,
    provider: *const TspProviderVtable,
    user_data: *mut c_void,
    out_context: *mut *mut TspContext,
) -> TspStatus {
    if settings.is_null() || out_context.is_null() {
        return TspStatus::InvalidArgument;
    }
    // SAFETY: non-null; the caller promises a readable TspSettings.
    let raw = unsafe { *settings };
    if raw.struct_size != size_of_u32::<TspSettings>() {
        return TspStatus::InvalidArgument;
    }
    let (Some(ayanamsha), Some(node)) = (
        Ayanamsha::from_index(raw.ayanamsha),
        NodeKind::from_index(raw.node),
    ) else {
        return TspStatus::InvalidArgument;
    };
    let settings = Settings {
        ayanamsha,
        node,
        dasha_depth: raw.dasha_depth,
    };
    let port: Box<dyn EphemerisPort> = if provider.is_null() {
        Box::new(TestProvider)
    } else {
        // SAFETY: non-null; the caller promises a readable vtable.
        let vtable = unsafe { *provider };
        if vtable.struct_size != size_of_u32::<TspProviderVtable>() {
            return TspStatus::InvalidArgument;
        }
        let Some(position) = vtable.position else {
            return TspStatus::InvalidArgument;
        };
        Box::new(HostProvider {
            position,
            user_data,
        })
    };
    match Context::new(settings, port) {
        Ok(inner) => {
            let context = Box::new(TspContext {
                inner,
                last_provider_code: Cell::new(0),
            });
            // SAFETY: non-null; the caller promises a writable slot.
            unsafe { out_context.write(Box::into_raw(context)) };
            TspStatus::Ok
        }
        Err(error) => TspStatus::from_error(error),
    }
}

/// Frees a context created by `tsp_context_new`; null is ignored.
///
/// # Safety
///
/// Every pointer must be null or valid for the access its documentation
/// describes, for the duration of the call; a handle must come from this
/// library and must not be used after `tsp_context_free`.
#[unsafe(no_mangle)]
#[allow(unsafe_code, reason = "reclaims a Box handed out by tsp_context_new")]
pub unsafe extern "C" fn tsp_context_free(context: *mut TspContext) {
    if context.is_null() {
        return;
    }
    // SAFETY: the pointer came from Box::into_raw in tsp_context_new and
    // the caller promises not to use it again.
    drop(unsafe { Box::from_raw(context) });
}

/// Copies the context's settings into `out_settings` (its `struct_size`
/// set by the caller).
///
/// # Safety
///
/// Every pointer must be null or valid for the access its documentation
/// describes, for the duration of the call; a handle must come from this
/// library and must not be used after `tsp_context_free`.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "reads the handle and writes the caller struct at the boundary"
)]
pub unsafe extern "C" fn tsp_context_settings(
    context: *const TspContext,
    out_settings: *mut TspSettings,
) -> TspStatus {
    if context.is_null() || out_settings.is_null() {
        return TspStatus::InvalidArgument;
    }
    // SAFETY: non-null; the caller promises a live context handle and a
    // TspSettings with its struct_size initialised.
    let (settings, size) = unsafe { ((*context).inner.settings(), (*out_settings).struct_size) };
    if size != size_of_u32::<TspSettings>() {
        return TspStatus::InvalidArgument;
    }
    // SAFETY: as above.
    unsafe {
        out_settings.write(TspSettings {
            struct_size: size,
            ayanamsha: settings.ayanamsha.index(),
            node: settings.node.index(),
            dasha_depth: settings.dasha_depth,
            reserved: 0,
        });
    }
    TspStatus::Ok
}

/// The code the provider returned on the last `Provider` failure, `0` when
/// there was none.
///
/// # Safety
///
/// Every pointer must be null or valid for the access its documentation
/// describes, for the duration of the call; a handle must come from this
/// library and must not be used after `tsp_context_free`.
#[unsafe(no_mangle)]
#[allow(unsafe_code, reason = "reads the handle at the boundary")]
pub unsafe extern "C" fn tsp_context_last_provider_code(context: *const TspContext) -> i32 {
    if context.is_null() {
        return 0;
    }
    // SAFETY: non-null; the caller promises a live context handle.
    unsafe { (*context).last_provider_code.get() }
}

/// The one batch call: computes the chart at `jd_ut` and hands back the
/// result blob in `out_blob`, which the caller frees with `tsp_blob_free`.
///
/// # Safety
///
/// Every pointer must be null or valid for the access its documentation
/// describes, for the duration of the call; a handle must come from this
/// library and must not be used after `tsp_context_free`.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "reads the handle and writes the blob descriptor at the boundary"
)]
pub unsafe extern "C" fn tsp_chart_compute(
    context: *const TspContext,
    jd_ut: f64,
    out_blob: *mut TspBlob,
) -> TspStatus {
    if context.is_null() || out_blob.is_null() {
        return TspStatus::InvalidArgument;
    }
    // SAFETY: non-null; the caller promises a live context handle.
    let ctx = unsafe { &*context };
    match ctx.inner.compute_chart(jd_ut) {
        Ok(chart) => {
            let bytes = encode_chart(&chart);
            let mut bytes = core::mem::ManuallyDrop::new(bytes);
            let blob = TspBlob {
                data: bytes.as_mut_ptr(),
                len: bytes.len(),
                cap: bytes.capacity(),
            };
            // SAFETY: non-null; the caller promises a writable blob slot.
            unsafe { out_blob.write(blob) };
            TspStatus::Ok
        }
        Err(error) => {
            if let Error::Provider { code, .. } = error {
                ctx.last_provider_code.set(code);
            }
            TspStatus::from_error(error)
        }
    }
}

/// Frees a blob returned by `tsp_chart_compute` and zeroes the descriptor;
/// null or an empty blob is ignored.
///
/// # Safety
///
/// Every pointer must be null or valid for the access its documentation
/// describes, for the duration of the call; a handle must come from this
/// library and must not be used after `tsp_context_free`.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "reclaims the Vec handed out by tsp_chart_compute"
)]
pub unsafe extern "C" fn tsp_blob_free(blob: *mut TspBlob) {
    if blob.is_null() {
        return;
    }
    // SAFETY: non-null; the caller promises a descriptor written by
    // tsp_chart_compute (or zeroed) that is not used again.
    let descriptor = unsafe {
        ptr::replace(
            blob,
            TspBlob {
                data: ptr::null_mut(),
                len: 0,
                cap: 0,
            },
        )
    };
    if descriptor.data.is_null() {
        return;
    }
    // SAFETY: the three fields are exactly those of the Vec that
    // tsp_chart_compute leaked with ManuallyDrop.
    drop(unsafe { Vec::from_raw_parts(descriptor.data, descriptor.len, descriptor.cap) });
}

/// The number of tree nodes a chart of `depth` levels holds, so a caller
/// can size its own structures before decoding.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "the exported-symbol attribute is unsafe in edition 2024"
)]
pub extern "C" fn tsp_chart_node_count(depth: u8) -> u32 {
    u32::try_from(Chart::node_count_for_depth(depth)).unwrap_or(u32::MAX)
}

/// A static, NUL-terminated English message for a status.
#[unsafe(no_mangle)]
#[allow(
    unsafe_code,
    reason = "the exported-symbol attribute is unsafe in edition 2024"
)]
pub extern "C" fn tsp_status_message(status: TspStatus) -> *const c_char {
    let message: &'static [u8] = match status {
        TspStatus::Ok => b"ok\0",
        TspStatus::InvalidArgument => b"invalid argument\0",
        TspStatus::DepthOutOfRange => b"dasha depth outside 1..=5\0",
        TspStatus::JulianDayNotFinite => b"the Julian Day is not finite\0",
        TspStatus::Provider => b"the provider failed; see the last provider code\0",
        TspStatus::PositionNotFinite => b"the provider returned a non-finite longitude\0",
    };
    message.as_ptr().cast::<c_char>()
}

// ── The result blob ────────────────────────────────────────────────────────

/// The magic at the start of every blob: the bytes `TSPB`.
pub const TSPB_MAGIC: u32 = 0x4250_5354;
/// The blob layout version.
pub const TSPB_VERSION: u32 = 1;
/// Section ids.
pub const TSPB_SECTION_CHART: u32 = 1;
/// The positions section: nine columns.
pub const TSPB_SECTION_POSITIONS: u32 = 2;
/// The dasha section: five columns.
pub const TSPB_SECTION_DASHA: u32 = 3;

/// The fixed fields of the chart header section, in order.
pub const CHART_HEADER_FIELDS: [&str; 6] = [
    "jd_ut:f64",
    "ayanamsha_deg:f64",
    "depth:u32",
    "position_count:u32",
    "node_count:u32",
    "reserved:u32",
];

/// The columns of the positions section, in directory order.
pub const POSITION_COLUMNS: [&str; 9] = [
    "body:u8",
    "longitude_nas:i64",
    "longitude_deg:f64",
    "latitude_deg:f64",
    "speed_deg_per_day:f64",
    "sign:u8",
    "nakshatra:u8",
    "pada:u8",
    "retrograde:u8",
];

/// The columns of the dasha section, in directory order.
pub const DASHA_COLUMNS: [&str; 5] = [
    "level:u8",
    "lord:u8",
    "parent:i32",
    "start_jd:f64",
    "end_jd:f64",
];

struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    fn align8(&mut self) {
        while self.buf.len() % 8 != 0 {
            self.buf.push(0);
        }
    }

    fn u8(&mut self, value: u8) {
        self.buf.push(value);
    }

    fn i32(&mut self, value: i32) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn i64(&mut self, value: i64) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn f64(&mut self, value: f64) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn patch_u32(&mut self, at: usize, value: u32) {
        if let Some(slot) = self.buf.get_mut(at..at + 4) {
            slot.copy_from_slice(&value.to_le_bytes());
        }
    }

    fn len_u32(&self) -> u32 {
        u32::try_from(self.buf.len()).unwrap_or(u32::MAX)
    }
}

/// Encodes a chart into the `TSPB` layout.
#[must_use]
pub fn encode_chart(chart: &Chart) -> Vec<u8> {
    let rows = chart.dasha_rows();
    let mut w = Writer {
        buf: Vec::with_capacity(256 + rows.len() * 24),
    };
    w.u32(TSPB_MAGIC);
    w.u32(TSPB_VERSION);
    w.u32(3);
    let total_at = w.buf.len();
    for _ in 0..5 {
        w.u32(0);
    }
    let table_at = w.buf.len();
    for _ in 0..12 {
        w.u32(0);
    }

    w.align8();
    let chart_at = w.len_u32();
    w.f64(chart.jd_ut);
    w.f64(chart.ayanamsha_deg);
    w.u32(u32::from(chart.depth()));
    w.u32(len_u32(chart.positions.len()));
    w.u32(len_u32(rows.len()));
    w.u32(0);
    let length = w.len_u32() - chart_at;
    write_entry(&mut w, table_at, 0, TSPB_SECTION_CHART, chart_at, length, 1);

    write_positions(&mut w, table_at, &chart.positions);
    write_dasha(&mut w, table_at, &rows);

    let total = w.len_u32();
    w.patch_u32(total_at, total);
    w.buf
}

fn len_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

/// Starts a column section: 8-aligned, with a zeroed directory of `count`
/// offsets that [`column`] fills in. Returns the section start and the
/// directory's byte position.
fn begin_columns(w: &mut Writer, count: usize) -> (u32, usize) {
    w.align8();
    let at = w.len_u32();
    let dir_at = w.buf.len();
    for _ in 0..count {
        w.u32(0);
    }
    w.align8();
    (at, dir_at)
}

/// Writes one 8-aligned column and records its offset, relative to the
/// section start, in the directory.
fn column(
    w: &mut Writer,
    section_at: u32,
    dir_at: usize,
    index: usize,
    write: impl FnOnce(&mut Writer),
) {
    w.align8();
    let offset = w.len_u32() - section_at;
    w.patch_u32(dir_at + index * 4, offset);
    write(w);
}

fn write_entry(
    w: &mut Writer,
    table_at: usize,
    index: usize,
    id: u32,
    offset: u32,
    length: u32,
    count: u32,
) {
    let at = table_at + index * 16;
    w.patch_u32(at, id);
    w.patch_u32(at + 4, offset);
    w.patch_u32(at + 8, length);
    w.patch_u32(at + 12, count);
}

fn write_positions(w: &mut Writer, table_at: usize, positions: &[BodyPosition]) {
    let (at, dir) = begin_columns(w, POSITION_COLUMNS.len());
    column(w, at, dir, 0, |w| {
        for p in positions {
            w.u8(p.body.index());
        }
    });
    column(w, at, dir, 1, |w| {
        for p in positions {
            w.i64(p.longitude_nas);
        }
    });
    column(w, at, dir, 2, |w| {
        for p in positions {
            w.f64(p.longitude_deg);
        }
    });
    column(w, at, dir, 3, |w| {
        for p in positions {
            w.f64(p.latitude_deg);
        }
    });
    column(w, at, dir, 4, |w| {
        for p in positions {
            w.f64(p.speed_deg_per_day);
        }
    });
    column(w, at, dir, 5, |w| {
        for p in positions {
            w.u8(p.sign);
        }
    });
    column(w, at, dir, 6, |w| {
        for p in positions {
            w.u8(p.nakshatra);
        }
    });
    column(w, at, dir, 7, |w| {
        for p in positions {
            w.u8(p.pada);
        }
    });
    column(w, at, dir, 8, |w| {
        for p in positions {
            w.u8(u8::from(p.retrograde));
        }
    });
    w.align8();
    let length = w.len_u32() - at;
    write_entry(
        w,
        table_at,
        1,
        TSPB_SECTION_POSITIONS,
        at,
        length,
        len_u32(positions.len()),
    );
}

fn write_dasha(w: &mut Writer, table_at: usize, rows: &[DashaRow]) {
    let (at, dir) = begin_columns(w, DASHA_COLUMNS.len());
    column(w, at, dir, 0, |w| {
        for r in rows {
            w.u8(r.level);
        }
    });
    column(w, at, dir, 1, |w| {
        for r in rows {
            w.u8(r.lord.index());
        }
    });
    column(w, at, dir, 2, |w| {
        for r in rows {
            w.i32(r.parent);
        }
    });
    column(w, at, dir, 3, |w| {
        for r in rows {
            w.f64(r.start_jd);
        }
    });
    column(w, at, dir, 4, |w| {
        for r in rows {
            w.f64(r.end_jd);
        }
    });
    w.align8();
    let length = w.len_u32() - at;
    write_entry(
        w,
        table_at,
        2,
        TSPB_SECTION_DASHA,
        at,
        length,
        len_u32(rows.len()),
    );
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "a test fails by panicking")]

    use super::*;

    fn read_u32(bytes: &[u8], at: usize) -> u32 {
        let mut b = [0u8; 4];
        b.copy_from_slice(bytes.get(at..at + 4).unwrap_or(&[0; 4]));
        u32::from_le_bytes(b)
    }

    fn read_f64(bytes: &[u8], at: usize) -> f64 {
        let mut b = [0u8; 8];
        b.copy_from_slice(bytes.get(at..at + 8).unwrap_or(&[0; 8]));
        f64::from_le_bytes(b)
    }

    #[test]
    fn blob_has_the_documented_layout() {
        let ctx = Context::new(Settings::DEFAULT, Box::new(TestProvider))
            .unwrap_or_else(|err| panic!("{err}"));
        let chart = ctx
            .compute_chart(2_460_000.5)
            .unwrap_or_else(|err| panic!("{err}"));
        let bytes = encode_chart(&chart);
        assert_eq!(read_u32(&bytes, 0), TSPB_MAGIC);
        assert_eq!(read_u32(&bytes, 4), TSPB_VERSION);
        assert_eq!(read_u32(&bytes, 8), 3);
        assert_eq!(read_u32(&bytes, 12) as usize, bytes.len());
        // Table entries.
        let entry = |i: usize| {
            let at = 32 + i * 16;
            (
                read_u32(&bytes, at),
                read_u32(&bytes, at + 4),
                read_u32(&bytes, at + 8),
                read_u32(&bytes, at + 12),
            )
        };
        let (id, off, _, count) = entry(0);
        assert_eq!((id, count), (TSPB_SECTION_CHART, 1));
        assert_eq!(
            read_f64(&bytes, off as usize).to_bits(),
            2_460_000.5f64.to_bits()
        );
        let (id, off, _, count) = entry(2);
        assert_eq!((id, count), (TSPB_SECTION_DASHA, 819));
        let start_col = read_u32(&bytes, off as usize + 12) as usize;
        assert_eq!(
            read_f64(&bytes, off as usize + start_col).to_bits(),
            2_460_000.5f64.to_bits()
        );
        assert_eq!((off as usize + start_col) % 8, 0);
    }

    #[test]
    #[allow(unsafe_code, reason = "the test exercises the C boundary")]
    fn the_c_boundary_round_trips_with_a_host_provider() {
        unsafe extern "C" fn host_position(
            user_data: *mut c_void,
            jd_ut: f64,
            body: u32,
            out: *mut TspPosition,
        ) -> i32 {
            // SAFETY: the test registers a counter as user_data and passes a
            // valid out pointer through the library.
            unsafe {
                *user_data.cast::<u32>() += 1;
                out.write(TspPosition {
                    longitude_deg: jd_ut.rem_euclid(360.0) + f64::from(body) * 10.0,
                    latitude_deg: 0.0,
                    speed_deg_per_day: 1.0,
                });
            }
            0
        }
        let mut calls = 0u32;
        let vtable = TspProviderVtable {
            struct_size: size_of_u32::<TspProviderVtable>(),
            position: Some(host_position),
        };
        let mut settings = TspSettings {
            struct_size: size_of_u32::<TspSettings>(),
            ayanamsha: 0,
            node: 0,
            dasha_depth: 0,
            reserved: 0,
        };
        // SAFETY: valid pointers throughout; the context is freed below.
        unsafe {
            assert_eq!(tsp_settings_default(&raw mut settings), TspStatus::Ok);
            assert_eq!(settings.dasha_depth, 3);
            let mut context: *mut TspContext = ptr::null_mut();
            let status = tsp_context_new(
                &raw const settings,
                &raw const vtable,
                (&raw mut calls).cast(),
                &raw mut context,
            );
            assert_eq!(status, TspStatus::Ok);
            let mut blob = TspBlob {
                data: ptr::null_mut(),
                len: 0,
                cap: 0,
            };
            assert_eq!(
                tsp_chart_compute(context, 2_460_000.5, &raw mut blob),
                TspStatus::Ok
            );
            assert_eq!(calls, 9);
            assert!(blob.len > 32);
            tsp_blob_free(&raw mut blob);
            assert!(blob.data.is_null());
            assert_eq!(
                tsp_chart_compute(context, f64::NAN, &raw mut blob),
                TspStatus::JulianDayNotFinite
            );
            tsp_context_free(context);
        }
        assert_eq!(tsp_chart_node_count(3), 819);
    }
}

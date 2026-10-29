//! The C boundary from both sides, as a misbehaving peer meets it: a
//! foreign table whose answers are malformed, which the port refuses as
//! it reads them, and malformed calls into an exported Rust provider,
//! which each trampoline refuses before the provider is asked.

#![allow(
    unsafe_code,
    clippy::unwrap_used,
    reason = "tests drive a C table by hand, and fail by panicking"
)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use teistro_core::quantity::{JulianDay, Place};
use teistro_port_ephemeris::vtable::{
    CapabilitiesC, CrossingEventC, CrossingRequestC, HorizonRequestC, ObliquityC, ObserverC,
    PositionColumnsC,
};
use teistro_port_ephemeris::{
    Astronomy, Body, CrossingRequest, DistanceUnit, EphemerisProvider, Exported, Frame, Horizon,
    HorizonEventKind, HorizonRequest, Lattice, Overrides, PositionRequest, PositionRequestC,
    ProviderCode, ProviderError, ProviderVtable, Quantity, SpeedModel, TestProvider, TimeScale,
    VTABLE_ABI_VERSION, VtableProvider,
};

const INVALID: i32 = ProviderCode::Invalid as i32;

fn size<T>() -> u32 {
    u32::try_from(size_of::<T>()).unwrap()
}

// ── A foreign table that answers what its script says ─────────────────────

/// What the scripted table answers; `user_data` points at it.
struct Script {
    name: CString,
    bodies: Vec<u16>,
    distance_unit: u8,
    speed_model: u8,
    astronomy: u8,
    native_frame_bits: u32,
    overrides: u32,
    /// The code every call but `capabilities` answers.
    code: i32,
    /// The code `capabilities` answers.
    capabilities_code: i32,
    /// The frame `positions` says its columns are in.
    frame_bits: u32,
    /// The instant a horizon event is found at.
    jd: f64,
    /// Whether a horizon event is found at all.
    found: bool,
    /// How many crossings there are, and what each is.
    count: u32,
    event: CrossingEventC,
    /// The size a text call reports when asked, and then when filled.
    asked: usize,
    filled: usize,
    text: Vec<u8>,
}

impl Script {
    fn new() -> Script {
        Script {
            name: CString::new("scripted").unwrap(),
            bodies: vec![Body::Sun.id(), Body::Moon.id()],
            distance_unit: DistanceUnit::AstronomicalUnits.id(),
            speed_model: SpeedModel::Derivative.id(),
            astronomy: Astronomy::Modern.id(),
            native_frame_bits: Frame::CANONICAL.to_bits(),
            overrides: 0,
            code: 0,
            capabilities_code: 0,
            frame_bits: Frame::CANONICAL.to_bits(),
            jd: 2_451_545.25,
            found: true,
            count: 1,
            event: CrossingEventC {
                jd_ut1: 2_451_545.5,
                boundary_deg: 30.0,
                direction: 0,
                reserved: [0; 7],
            },
            asked: 2,
            filled: 2,
            text: b"{}".to_vec(),
        }
    }

    fn user_data(&self) -> *mut c_void {
        ptr::from_ref(self).cast_mut().cast()
    }

    /// The script bound through `vtable`.
    fn bind(&self, vtable: ProviderVtable) -> Result<VtableProvider, ProviderError> {
        // SAFETY: every function in the table reads `user_data` as this
        // script, which outlives the provider in each test.
        unsafe { VtableProvider::bind(vtable, self.user_data()) }
    }
}

/// # Safety
///
/// `user_data` is a live [`Script`].
unsafe fn script<'a>(user_data: *mut c_void) -> &'a Script {
    // SAFETY: as documented.
    unsafe { &*user_data.cast::<Script>() }
}

unsafe extern "C" fn capabilities(user_data: *mut c_void, out: *mut CapabilitiesC) -> i32 {
    // SAFETY: the table's contract.
    let script = unsafe { script(user_data) };
    if script.capabilities_code != 0 {
        return script.capabilities_code;
    }
    // SAFETY: the port hands a writable struct.
    unsafe {
        out.write(CapabilitiesC {
            struct_size: size::<CapabilitiesC>(),
            speeds: 1,
            deterministic: 1,
            tier: 0,
            distance_unit: script.distance_unit,
            speed_model: script.speed_model,
            astronomy: script.astronomy,
            native: 1,
            reserved: [0; 1],
            name: script.name.as_ptr(),
            version: ptr::null(),
            data_version: ptr::null(),
            jd_min: 2_000_000.0,
            jd_max: 3_000_000.0,
            bodies: script.bodies.as_ptr(),
            body_count: script.bodies.len(),
            native_frame_bits: script.native_frame_bits,
            overrides: script.overrides,
            ayanamshas: ptr::null(),
            ayanamsha_count: 0,
            hashes: ptr::null(),
            hash_count: 0,
        });
    }
    0
}

unsafe extern "C" fn positions(
    user_data: *mut c_void,
    _: *const PositionRequestC,
    out: *mut PositionColumnsC,
) -> i32 {
    // SAFETY: the table's contract.
    let script = unsafe { script(user_data) };
    // SAFETY: the port hands writable columns.
    unsafe { (*out).frame_bits = script.frame_bits };
    script.code
}

unsafe extern "C" fn obliquity(user_data: *mut c_void, _: f64, _: u32, _: *mut ObliquityC) -> i32 {
    // SAFETY: the table's contract.
    unsafe { script(user_data) }.code
}

unsafe extern "C" fn seconds(user_data: *mut c_void, _: f64, _: *mut f64) -> i32 {
    // SAFETY: the table's contract.
    unsafe { script(user_data) }.code
}

unsafe extern "C" fn ayanamsha(user_data: *mut c_void, _: f64, _: u32, _: u16, _: *mut f64) -> i32 {
    // SAFETY: the table's contract.
    unsafe { script(user_data) }.code
}

unsafe extern "C" fn horizon(
    user_data: *mut c_void,
    _: *const HorizonRequestC,
    jd: *mut f64,
    found: *mut u8,
) -> i32 {
    // SAFETY: the table's contract; both outs are writable.
    unsafe {
        let script = script(user_data);
        jd.write(script.jd);
        found.write(u8::from(script.found));
        script.code
    }
}

unsafe extern "C" fn crossings(
    user_data: *mut c_void,
    _: *const CrossingRequestC,
    events: *mut CrossingEventC,
    capacity: u32,
    count: *mut u32,
) -> i32 {
    // SAFETY: the table's contract; the buffer holds `capacity` events.
    unsafe {
        let script = script(user_data);
        count.write(script.count);
        for index in 0..script.count.min(capacity) as usize {
            events.add(index).write(script.event);
        }
        script.code
    }
}

/// # Safety
///
/// `user_data` is a live [`Script`]; `out` null or valid for `capacity`
/// bytes; `len` writable.
unsafe fn text(user_data: *mut c_void, out: *mut c_char, capacity: usize, len: *mut usize) -> i32 {
    // SAFETY: as documented.
    unsafe {
        let script = script(user_data);
        if out.is_null() {
            len.write(script.asked);
        } else {
            len.write(script.filled);
            let fits = script.text.len().min(capacity);
            ptr::copy_nonoverlapping(script.text.as_ptr().cast::<c_char>(), out, fits);
        }
        script.code
    }
}

unsafe extern "C" fn manifest(
    user_data: *mut c_void,
    out: *mut c_char,
    capacity: usize,
    len: *mut usize,
) -> i32 {
    // SAFETY: the table's contract.
    unsafe { text(user_data, out, capacity, len) }
}

unsafe extern "C" fn call(
    user_data: *mut c_void,
    _: *const c_char,
    _: *const c_char,
    out: *mut c_char,
    capacity: usize,
    len: *mut usize,
) -> i32 {
    // SAFETY: the table's contract.
    unsafe { text(user_data, out, capacity, len) }
}

/// The two functions a table must have, and nothing else.
fn bare() -> ProviderVtable {
    ProviderVtable {
        struct_size: size::<ProviderVtable>(),
        abi_version: VTABLE_ABI_VERSION,
        capabilities: Some(capabilities),
        positions: Some(positions),
        ..ProviderVtable::EMPTY
    }
}

/// Every function filled from the script.
fn full() -> ProviderVtable {
    ProviderVtable {
        obliquity: Some(obliquity),
        delta_t: Some(seconds),
        ayanamsha: Some(ayanamsha),
        dut1: Some(seconds),
        horizon_event: Some(horizon),
        crossings: Some(crossings),
        native_manifest: Some(manifest),
        native_call: Some(call),
        ..bare()
    }
}

fn place() -> Place {
    Place::try_from_degrees(27.7, 85.3, 1_300.0).unwrap()
}

fn horizon_request() -> HorizonRequest {
    HorizonRequest {
        body: Body::Sun,
        kind: HorizonEventKind::Rise,
        place: place(),
        from: JulianDay::try_new(2_451_545.0).unwrap(),
        window_days: 1.0,
        horizon: Horizon::UPPER_LIMB_REFRACTION,
    }
}

fn crossing_request() -> CrossingRequest {
    CrossingRequest {
        quantity: Quantity::Longitude(Body::Sun),
        lattice: Lattice {
            origin_deg: 0.0,
            step_deg: 30.0,
        },
        from: JulianDay::try_new(2_451_545.0).unwrap(),
        to: JulianDay::try_new(2_451_575.0).unwrap(),
        tolerance_days: 1e-6,
        frame: Frame::CANONICAL,
        observer: Some(place()),
    }
}

fn is_invalid<T>(result: Result<T, ProviderError>, naming: &str) -> bool {
    match result {
        Err(ProviderError::Invalid { detail }) => detail.contains(naming),
        _ => false,
    }
}

#[test]
fn a_table_is_bound_only_with_capabilities_it_can_read() {
    let script = Script::new();
    let bound = script.bind(full()).unwrap();
    assert_eq!(bound.capabilities().identity.name, "scripted");
    assert_eq!(
        bound.capabilities().identity.version,
        "",
        "a null string is empty"
    );
    assert_eq!(bound.capabilities().bodies, [Body::Sun, Body::Moon]);

    let resized = ProviderVtable {
        struct_size: 8,
        ..full()
    };
    assert!(is_invalid(script.bind(resized), "vtable size 8"));
    let missing = ProviderVtable {
        capabilities: None,
        ..full()
    };
    assert!(is_invalid(
        script.bind(missing),
        "needs capabilities and positions"
    ));

    let refusing = Script {
        capabilities_code: ProviderCode::Refused as i32,
        ..Script::new()
    };
    assert!(matches!(
        refusing.bind(full()),
        Err(ProviderError::Refused { detail }) if detail == "capabilities"
    ));

    for (wrong, naming) in [
        (
            Script {
                distance_unit: 99,
                ..Script::new()
            },
            "distance unit id 99",
        ),
        (
            Script {
                speed_model: 99,
                ..Script::new()
            },
            "speed model id 99",
        ),
        (
            Script {
                astronomy: 99,
                ..Script::new()
            },
            "astronomy id 99",
        ),
        (
            Script {
                overrides: Overrides::ANGLES.bits(),
                ..Script::new()
            },
            "angles override",
        ),
        (
            Script {
                native_frame_bits: 1 << 9,
                ..Script::new()
            },
            "reserved bit",
        ),
    ] {
        assert!(is_invalid(wrong.bind(full()), naming), "{naming}");
    }
}

#[test]
fn a_function_the_table_leaves_out_is_unsupported_by_name() {
    let script = Script::new();
    let bound = script.bind(bare()).unwrap();
    let unsupported = |result: Result<(), ProviderError>, what: &str| {
        assert!(
            matches!(&result, Err(ProviderError::Unsupported { what: named }) if named == what),
            "{what}: {result:?}"
        );
    };
    unsupported(
        bound.obliquity(2_451_545.0, TimeScale::Tt).map(drop),
        "obliquity",
    );
    unsupported(bound.delta_t_seconds(2_451_545.0).map(drop), "delta_t");
    unsupported(
        bound
            .ayanamsha_deg(
                2_451_545.0,
                TimeScale::Tt,
                teistro_core::catalogue::Ayanamsha::Lahiri,
            )
            .map(drop),
        "ayanamsha",
    );
    unsupported(bound.dut1_seconds(2_451_545.0).map(drop), "dut1");
    unsupported(
        bound.horizon_event(&horizon_request()).map(drop),
        "rise_set",
    );
    unsupported(bound.crossings(&crossing_request()).map(drop), "crossings");
    unsupported(bound.native_manifest().map(drop), "native_manifest");
    unsupported(bound.native_call("f", "{}").map(drop), "native_call");
}

#[test]
fn each_code_a_table_answers_is_the_error_it_names() {
    let jds = [2_451_545.0];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    for (code, want) in [
        (ProviderCode::OutOfRange as i32, "OutOfRange"),
        (ProviderCode::DataMissing as i32, "DataMissing"),
        (ProviderCode::Unsupported as i32, "Unsupported"),
        (17, "Provider"),
    ] {
        let script = Script {
            code,
            ..Script::new()
        };
        let bound = script.bind(full()).unwrap();
        let kind = |error: ProviderError| format!("{error:?}");
        for error in [
            bound.positions(&request).map(drop).unwrap_err(),
            bound
                .obliquity(2_451_545.0, TimeScale::Tt)
                .map(drop)
                .unwrap_err(),
            bound.delta_t_seconds(2_451_545.0).map(drop).unwrap_err(),
            bound
                .ayanamsha_deg(
                    2_451_545.0,
                    TimeScale::Tt,
                    teistro_core::catalogue::Ayanamsha::Raman,
                )
                .map(drop)
                .unwrap_err(),
            bound.dut1_seconds(2_451_545.0).map(drop).unwrap_err(),
            bound
                .horizon_event(&horizon_request())
                .map(drop)
                .unwrap_err(),
            bound.crossings(&crossing_request()).map(drop).unwrap_err(),
            bound.native_manifest().map(drop).unwrap_err(),
        ] {
            assert!(kind(error.clone()).starts_with(want), "{code}: {error:?}");
        }
    }
}

#[test]
fn an_answer_the_port_cannot_read_is_refused_as_it_is_read() {
    let jds = [2_451_545.0];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    // Columns said to be in a frame no bits name.
    let script = Script {
        frame_bits: 1 << 9,
        ..Script::new()
    };
    assert!(is_invalid(
        script.bind(full()).unwrap().positions(&request),
        "reserved bit"
    ));

    // A rise at an instant that is no instant.
    let script = Script {
        jd: f64::NAN,
        ..Script::new()
    };
    assert!(matches!(
        script
            .bind(full())
            .unwrap()
            .horizon_event(&horizon_request()),
        Err(ProviderError::Invalid { .. })
    ));
    let none = Script {
        found: false,
        jd: f64::NAN,
        ..Script::new()
    };
    let unfound = none.bind(full()).unwrap().horizon_event(&horizon_request());
    assert_eq!(unfound, Ok(None), "no event, whatever the instant says");
    let script = Script::new();
    let found = script
        .bind(full())
        .unwrap()
        .horizon_event(&horizon_request())
        .unwrap();
    assert_eq!(found.map(JulianDay::get), Some(2_451_545.25));

    // A crossing with no direction, no boundary or no instant.
    for (event, naming) in [
        (
            CrossingEventC {
                direction: 99,
                ..Script::new().event
            },
            "direction 99",
        ),
        (
            CrossingEventC {
                boundary_deg: f64::INFINITY,
                ..Script::new().event
            },
            "boundary is not finite",
        ),
        (
            CrossingEventC {
                jd_ut1: f64::NAN,
                ..Script::new().event
            },
            "",
        ),
    ] {
        let script = Script {
            event,
            ..Script::new()
        };
        let found = script.bind(full()).unwrap().crossings(&crossing_request());
        assert!(is_invalid(found, naming), "{naming}");
    }
}

#[test]
fn more_crossings_than_the_first_buffer_are_asked_for_again() {
    let script = Script {
        count: 100,
        ..Script::new()
    };
    let events = script
        .bind(full())
        .unwrap()
        .crossings(&crossing_request())
        .unwrap();
    assert_eq!(events.len(), 100);
    assert!(
        events
            .iter()
            .all(|event| event.boundary_deg.to_bits() == 30.0_f64.to_bits())
    );
    let none = Script {
        count: 0,
        ..Script::new()
    };
    let found = none.bind(full()).unwrap().crossings(&crossing_request());
    assert_eq!(found.map(|events| events.len()), Ok(0));
}

#[test]
fn a_text_is_read_by_size_then_fill_and_refused_when_either_lies() {
    let empty = Script {
        asked: 0,
        ..Script::new()
    };
    assert_eq!(empty.bind(full()).unwrap().native_manifest().unwrap(), "");

    let fine = Script::new();
    let bound = fine.bind(full()).unwrap();
    assert_eq!(bound.native_manifest().unwrap(), "{}");
    assert_eq!(bound.native_call("f", "{}").unwrap(), "{}");
    assert!(is_invalid(
        bound.native_call("f\0g", "{}"),
        "cannot contain a NUL"
    ));
    assert!(is_invalid(
        bound.native_call("f", "{\0}"),
        "cannot contain a NUL"
    ));

    let grew = Script {
        filled: 5,
        ..Script::new()
    };
    assert!(matches!(
        grew.bind(full()).unwrap().native_manifest(),
        Err(ProviderError::Refused { detail }) if detail.contains("between the size and the fill")
    ));

    let garbled = Script {
        text: vec![0xff, 0xfe],
        ..Script::new()
    };
    assert!(matches!(
        garbled.bind(full()).unwrap().native_call("f", "{}"),
        Err(ProviderError::Refused { detail }) if detail.contains("not UTF-8")
    ));
}

// ── Malformed calls into an exported Rust provider ────────────────────────

fn exported() -> (Box<Exported<TestProvider>>, ProviderVtable) {
    (
        Exported::new(TestProvider::new()),
        Exported::<TestProvider>::vtable(),
    )
}

fn request_c(jds: &[f64], bodies: &[u16]) -> PositionRequestC {
    PositionRequestC {
        struct_size: size::<PositionRequestC>(),
        scale: TimeScale::Ut1.id(),
        frame_bits: Frame::CANONICAL.to_bits(),
        speeds: 1,
        has_observer: 0,
        reserved: [0; 2],
        observer: ObserverC::default(),
        jds: jds.as_ptr(),
        jd_count: jds.len(),
        bodies: bodies.as_ptr(),
        body_count: bodies.len(),
    }
}

/// A bad observer: a longitude past the antimeridian.
const NOWHERE: ObserverC = ObserverC {
    longitude_deg: 200.0,
    latitude_deg: 0.0,
    altitude_m: 0.0,
};

#[test]
fn a_request_struct_is_decoded_or_refused_for_what_it_gets_wrong() {
    let jds = [2_451_545.0];
    let bodies = [Body::Sun.id()];
    // SAFETY: each request's arrays are the live locals above.
    unsafe {
        let fine = request_c(&jds, &bodies);
        let fine = fine.decode().unwrap();
        assert_eq!(fine.bodies, [Body::Sun]);
        let scale = PositionRequestC {
            scale: 99,
            ..request_c(&jds, &bodies)
        };
        assert!(is_invalid(scale.decode().map(drop), "time scale id 99"));
        let frame = PositionRequestC {
            frame_bits: 1 << 9,
            ..request_c(&jds, &bodies)
        };
        assert!(is_invalid(frame.decode().map(drop), "reserved bit"));
        let unknown = [u16::MAX];
        let body = PositionRequestC {
            body_count: 1,
            bodies: unknown.as_ptr(),
            ..request_c(&jds, &bodies)
        };
        assert!(matches!(
            body.decode(),
            Err(ProviderError::Unsupported { .. })
        ));
        let observer = PositionRequestC {
            has_observer: 1,
            observer: NOWHERE,
            ..request_c(&jds, &bodies)
        };
        assert!(matches!(
            observer.decode(),
            Err(ProviderError::Invalid { .. })
        ));
        let empty = PositionRequestC {
            jds: ptr::null(),
            bodies: ptr::null(),
            ..request_c(&jds, &bodies)
        };
        assert!(
            empty.decode().unwrap().jds.is_empty(),
            "a null array is empty"
        );
    }
}

#[test]
fn each_trampoline_refuses_a_malformed_call_before_the_provider_is_asked() {
    let (provider, vtable) = exported();
    let user_data = provider.user_data();
    let null = ptr::null_mut::<c_void>();
    let mut seconds = 0.0_f64;
    let mut obliquity = ObliquityC::default();
    // SAFETY: every pointer passed is null, a live local or the box's own
    // user data; the trampolines refuse each null before reading it.
    unsafe {
        let capabilities = vtable.capabilities.unwrap();
        assert_eq!(capabilities(null, ptr::null_mut()), INVALID);
        assert_eq!(capabilities(user_data, ptr::null_mut()), INVALID);
        let mut caps: CapabilitiesC = core::mem::zeroed();
        caps.struct_size = 8;
        assert_eq!(
            capabilities(user_data, &raw mut caps),
            INVALID,
            "another header's size"
        );

        let obliquity_fn = vtable.obliquity.unwrap();
        assert_eq!(obliquity_fn(null, 0.0, 0, &raw mut obliquity), INVALID);
        assert_eq!(
            obliquity_fn(user_data, 2_451_545.0, 99, &raw mut obliquity),
            INVALID
        );
        assert_eq!(
            obliquity_fn(
                user_data,
                2_451_545.0,
                TimeScale::Tt.id(),
                &raw mut obliquity
            ),
            ProviderCode::Unsupported as i32,
            "a well-formed call reaches the provider, which answers none"
        );
        for f in [vtable.delta_t.unwrap(), vtable.dut1.unwrap()] {
            assert_eq!(f(null, 0.0, &raw mut seconds), INVALID);
            assert_eq!(f(user_data, 0.0, ptr::null_mut()), INVALID);
            assert_eq!(
                f(user_data, 2_451_545.0, &raw mut seconds),
                ProviderCode::Unsupported as i32
            );
        }
        let ayanamsha = vtable.ayanamsha.unwrap();
        assert_eq!(ayanamsha(null, 0.0, 0, 0, &raw mut seconds), INVALID);
        assert_eq!(ayanamsha(user_data, 0.0, 0, 0, ptr::null_mut()), INVALID);
        assert_eq!(obliquity_fn(user_data, 0.0, 0, ptr::null_mut()), INVALID);
        assert_eq!(
            ayanamsha(
                user_data,
                0.0,
                TimeScale::Tt.id(),
                u16::MAX,
                &raw mut seconds
            ),
            INVALID
        );
        assert_eq!(ayanamsha(user_data, 0.0, 99, 0, &raw mut seconds), INVALID);
    }
}

#[test]
fn the_native_trampolines_refuse_a_call_they_cannot_read() {
    let (provider, vtable) = exported();
    let user_data = provider.user_data();
    let null = ptr::null_mut::<c_void>();
    // SAFETY: every pointer passed is null, a live local or the box's own
    // user data; the trampolines refuse each null before reading it.
    unsafe {
        let manifest = vtable.native_manifest.unwrap();
        let mut len = 0_usize;
        assert_eq!(manifest(null, ptr::null_mut(), 0, &raw mut len), INVALID);
        // The length is reported, and a buffer too small for it is left
        // untouched rather than given half the JSON.
        assert_eq!(manifest(user_data, ptr::null_mut(), 0, &raw mut len), 0);
        let mut short = [7 as c_char; 4];
        assert_eq!(
            manifest(user_data, short.as_mut_ptr(), short.len(), &raw mut len),
            0
        );
        assert_eq!(short, [7; 4]);
        let mut whole = vec![0 as c_char; len];
        assert_eq!(
            manifest(user_data, whole.as_mut_ptr(), len, ptr::null_mut()),
            0,
            "no length wanted"
        );
        assert_eq!(whole.first().map(|c| c.to_ne_bytes()), Some(*b"{"));
        let call = vtable.native_call.unwrap();
        let name = CString::new("tp_echo").unwrap();
        assert_eq!(
            call(
                null,
                name.as_ptr(),
                ptr::null(),
                ptr::null_mut(),
                0,
                &raw mut len
            ),
            INVALID
        );
        assert_eq!(
            call(
                user_data,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                0,
                &raw mut len
            ),
            INVALID
        );
        let garbled = [0xff_u8, 0];
        assert_eq!(
            call(
                user_data,
                garbled.as_ptr().cast(),
                ptr::null(),
                ptr::null_mut(),
                0,
                &raw mut len
            ),
            INVALID,
            "a name that is not UTF-8"
        );
        // A size query answers the length alone; null arguments are read
        // as an empty object, which the echo refuses for its value.
        let three = CString::new(r#"{"value": 3}"#).unwrap();
        assert_eq!(
            call(
                user_data,
                name.as_ptr(),
                three.as_ptr(),
                ptr::null_mut(),
                0,
                &raw mut len
            ),
            0
        );
        assert_eq!(len, r#"{"value":3.0}"#.len());
        assert_eq!(
            call(
                user_data,
                name.as_ptr(),
                ptr::null(),
                ptr::null_mut(),
                0,
                &raw mut len
            ),
            INVALID
        );
        let unknown = CString::new("no_such_function").unwrap();
        assert_ne!(
            call(
                user_data,
                unknown.as_ptr(),
                ptr::null(),
                ptr::null_mut(),
                0,
                &raw mut len
            ),
            0,
            "the provider's own refusal crosses as its code"
        );
    }
}

#[test]
fn the_positions_trampoline_refuses_columns_it_cannot_fill() {
    let (provider, vtable) = exported();
    let user_data = provider.user_data();
    let positions = vtable.positions.unwrap();
    let jds = [2_451_545.0, 2_451_546.0];
    let bodies = [Body::Sun.id(), Body::Moon.id()];
    let mut lon = [0.0_f64; 4];
    let mut status = [0_i32; 4];
    let mut source = [0_u32; 4];
    let columns =
        |capacity: usize, lon: *mut f64, status: *mut i32, source: *mut u32| PositionColumnsC {
            struct_size: size::<PositionColumnsC>(),
            frame_bits: 0,
            lon,
            lat: ptr::null_mut(),
            dist: ptr::null_mut(),
            lon_speed: ptr::null_mut(),
            lat_speed: ptr::null_mut(),
            dist_speed: ptr::null_mut(),
            status,
            source,
            capacity,
        };
    let request = request_c(&jds, &bodies);
    // SAFETY: every array is a live local holding `capacity` cells, and a
    // null column is skipped by the trampoline.
    unsafe {
        let mut out = columns(
            4,
            lon.as_mut_ptr(),
            status.as_mut_ptr(),
            source.as_mut_ptr(),
        );
        assert_eq!(
            positions(ptr::null_mut(), &raw const request, &raw mut out),
            INVALID
        );
        assert_eq!(positions(user_data, ptr::null(), &raw mut out), INVALID);
        assert_eq!(
            positions(user_data, &raw const request, ptr::null_mut()),
            INVALID
        );
        let mut short = columns(
            3,
            lon.as_mut_ptr(),
            status.as_mut_ptr(),
            source.as_mut_ptr(),
        );
        assert_eq!(
            positions(user_data, &raw const request, &raw mut short),
            INVALID,
            "too few cells"
        );
        let mut other = columns(
            4,
            lon.as_mut_ptr(),
            status.as_mut_ptr(),
            source.as_mut_ptr(),
        );
        other.struct_size = 8;
        assert_eq!(
            positions(user_data, &raw const request, &raw mut other),
            INVALID
        );
        let older = PositionRequestC {
            struct_size: 8,
            ..request_c(&jds, &bodies)
        };
        assert_eq!(
            positions(user_data, &raw const older, &raw mut out),
            INVALID
        );
        let node = [Body::TrueNode.id()];
        let unanswered = request_c(&jds[..1], &node);
        assert_ne!(
            positions(user_data, &raw const unanswered, &raw mut out),
            0,
            "a body the provider does not answer is its refusal"
        );
        // The well-formed call fills the columns it was given, and skips
        // the null ones.
        assert_eq!(positions(user_data, &raw const request, &raw mut out), 0);
        assert_eq!(out.frame_bits, Frame::CANONICAL.to_bits());
    }
    assert!(lon.iter().all(|value| *value != 0.0));
    assert!(
        status
            .iter()
            .all(|code| *code == teistro_port_ephemeris::CellStatus::Ok.code())
    );
}

#[test]
fn the_horizon_trampoline_refuses_a_request_it_cannot_read() {
    let (provider, vtable) = exported();
    let user_data = provider.user_data();
    let horizon = vtable.horizon_event.unwrap();
    let good = HorizonRequestC::of(&horizon_request());
    let (mut jd, mut found) = (0.0_f64, 0_u8);
    let wrong = [
        HorizonRequestC {
            struct_size: 8,
            ..good
        },
        HorizonRequestC {
            body: u16::MAX,
            ..good
        },
        HorizonRequestC { kind: 99, ..good },
        HorizonRequestC { disc: 99, ..good },
        HorizonRequestC {
            refraction: 99,
            ..good
        },
        HorizonRequestC {
            observer: NOWHERE,
            ..good
        },
        HorizonRequestC {
            from_jd_ut1: f64::NAN,
            ..good
        },
    ];
    // SAFETY: every pointer is null or a live local.
    unsafe {
        assert_eq!(
            horizon(
                ptr::null_mut(),
                &raw const good,
                &raw mut jd,
                &raw mut found
            ),
            INVALID
        );
        assert_eq!(
            horizon(user_data, ptr::null(), &raw mut jd, &raw mut found),
            INVALID
        );
        assert_eq!(
            horizon(user_data, &raw const good, ptr::null_mut(), &raw mut found),
            INVALID
        );
        assert_eq!(
            horizon(user_data, &raw const good, &raw mut jd, ptr::null_mut()),
            INVALID
        );
        for (index, raw) in wrong.iter().enumerate() {
            assert_eq!(
                horizon(user_data, raw, &raw mut jd, &raw mut found),
                INVALID,
                "case {index}"
            );
        }
        assert_eq!(
            horizon(user_data, &raw const good, &raw mut jd, &raw mut found),
            ProviderCode::Unsupported as i32,
            "a well-formed request reaches the provider"
        );
    }
}

/// A composite crossing request, seen from Kathmandu.
fn crossing_c() -> CrossingRequestC {
    CrossingRequestC {
        struct_size: size::<CrossingRequestC>(),
        frame_bits: Frame::CANONICAL.to_bits(),
        quantity_kind: 2,
        has_observer: 1,
        reserved: [0; 2],
        first_body: Body::Moon.id(),
        second_body: Body::Sun.id(),
        coefficient_a: 1.0,
        coefficient_b: -1.0,
        origin_deg: 0.0,
        step_deg: 12.0,
        from_jd_ut1: 2_451_545.0,
        to_jd_ut1: 2_451_575.0,
        tolerance_days: 1e-6,
        observer: ObserverC {
            longitude_deg: 85.3,
            latitude_deg: 27.7,
            altitude_m: 1_300.0,
        },
    }
}

#[test]
fn the_crossings_trampoline_refuses_a_request_it_cannot_read() {
    let (provider, vtable) = exported();
    let user_data = provider.user_data();
    let crossings = vtable.crossings.unwrap();
    let good = crossing_c();
    let wrong = [
        CrossingRequestC {
            struct_size: 8,
            ..good
        },
        CrossingRequestC {
            first_body: u16::MAX,
            ..good
        },
        CrossingRequestC {
            second_body: u16::MAX,
            ..good
        },
        CrossingRequestC {
            quantity_kind: 7,
            ..good
        },
        CrossingRequestC {
            frame_bits: 1 << 9,
            ..good
        },
        CrossingRequestC {
            from_jd_ut1: f64::NAN,
            ..good
        },
        CrossingRequestC {
            to_jd_ut1: f64::INFINITY,
            ..good
        },
        CrossingRequestC {
            observer: NOWHERE,
            ..good
        },
    ];
    let mut events = [CrossingEventC::default(); 4];
    let mut count = 0_u32;
    // SAFETY: every pointer is null or a live local; `events` holds four.
    unsafe {
        let out = events.as_mut_ptr();
        assert_eq!(
            crossings(ptr::null_mut(), &raw const good, out, 4, &raw mut count),
            INVALID
        );
        assert_eq!(
            crossings(user_data, ptr::null(), out, 4, &raw mut count),
            INVALID
        );
        assert_eq!(
            crossings(user_data, &raw const good, out, 4, ptr::null_mut()),
            INVALID
        );
        assert_eq!(
            crossings(
                user_data,
                &raw const good,
                ptr::null_mut(),
                4,
                &raw mut count
            ),
            INVALID,
            "no buffer for a capacity"
        );
        for (index, raw) in wrong.iter().enumerate() {
            assert_eq!(
                crossings(user_data, raw, out, 4, &raw mut count),
                INVALID,
                "case {index}"
            );
        }
        // A longitude ignores the second body, and a geocentric request
        // its observer; both reach the provider.
        let longitude = CrossingRequestC {
            quantity_kind: 0,
            second_body: u16::MAX,
            has_observer: 0,
            observer: NOWHERE,
            ..good
        };
        assert_eq!(
            crossings(
                user_data,
                &raw const longitude,
                ptr::null_mut(),
                0,
                &raw mut count
            ),
            ProviderCode::Unsupported as i32
        );
        assert_eq!(
            crossings(user_data, &raw const good, out, 4, &raw mut count),
            ProviderCode::Unsupported as i32
        );
    }
}

#[test]
fn an_error_message_is_cut_to_the_buffer_or_not_written_at_all() {
    use teistro_port_ephemeris::plugin::write_error;
    let mut one = [7 as c_char; 1];
    let mut empty = [7 as c_char; 1];
    // SAFETY: each buffer is writable for the capacity given, and a null
    // buffer is refused before any write.
    unsafe {
        write_error("refused", ptr::null_mut(), 8);
        write_error("refused", empty.as_mut_ptr(), 0);
        write_error("refused", one.as_mut_ptr(), 1);
    }
    assert_eq!(empty, [7], "no room, no write");
    assert_eq!(one, [0], "room for the terminator alone");
}

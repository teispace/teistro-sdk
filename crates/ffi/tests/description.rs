//! The description agrees with the library it describes: every boundary
//! struct's C layout is the Rust layout, the ABI version is the constant,
//! the status enum is the core's, and every entry point of this crate is
//! in the description.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests fail by panicking"
)]

use std::path::Path;

use teistro_core::Status;
use teistro_ffi::{SDK_VERSION, TS_ABI_VERSION, schemas};
use teistro_idl::layout::{Target, struct_layout};
use teistro_idl::model::Api;
use teistro_idl::sdk::describe;

fn api() -> Api {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    describe(&root, schemas::schemas(), SDK_VERSION).unwrap_or_else(|e| panic!("{e}"))
}

/// The Rust sizes and alignments of every boundary struct, by name.
macro_rules! rust_layouts {
    ($($name:ident => $ty:ty),* $(,)?) => {
        [$((stringify!($name), size_of::<$ty>(), align_of::<$ty>())),*]
    };
}

#[test]
fn every_struct_has_the_layout_the_description_computes() {
    use teistro_ffi::calendar::TsCalendarDate;
    use teistro_ffi::context::{TsContextOptions, TsError};
    use teistro_ffi::frame::TsFrame;
    use teistro_ffi::intl::TsIntlLoaded;
    use teistro_ffi::strings::{TsHash, TsStr, TsString};
    use teistro_ffi::time::{
        TsCivilDateTime, TsCivilTime, TsDeltaT, TsTimeConversion, TsZoneResolution, TsZoneSpec,
    };
    use teistro_port_ephemeris::vtable::{
        CapabilitiesC, CrossingEventC, CrossingRequestC, DataHashC, HorizonRequestC, ObliquityC,
        ObserverC, PositionColumnsC, PositionRequestC, ProviderVtable,
    };

    let api = api();
    let expected = rust_layouts!(
        ObserverC => ObserverC,
        PositionRequestC => PositionRequestC,
        PositionColumnsC => PositionColumnsC,
        ObliquityC => ObliquityC,
        HorizonRequestC => HorizonRequestC,
        CrossingRequestC => CrossingRequestC,
        CrossingEventC => CrossingEventC,
        DataHashC => DataHashC,
        CapabilitiesC => CapabilitiesC,
        ProviderVtable => ProviderVtable,
        TsString => TsString,
        TsStr => TsStr,
        TsHash => TsHash,
        TsBlob => teistro_ffi::blob::TsBlob,
        TsContextOptions => TsContextOptions,
        TsError => TsError,
        TsFrame => TsFrame,
        TsCalendarDate => TsCalendarDate,
        TsCivilTime => TsCivilTime,
        TsCivilDateTime => TsCivilDateTime,
        TsZoneSpec => TsZoneSpec,
        TsZoneResolution => TsZoneResolution,
        TsTimeConversion => TsTimeConversion,
        TsDeltaT => TsDeltaT,
        TsIntlLoaded => TsIntlLoaded,
    );
    assert_eq!(
        api.structs.len(),
        expected.len(),
        "structs in the description: {:?}",
        api.structs
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
    );
    for (name, size, align) in expected {
        let def = api
            .struct_named(name)
            .unwrap_or_else(|| panic!("{name} is not described"));
        let layout =
            struct_layout(&api, def, Target::host()).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!((layout.size, layout.align), (size, align), "{name}");
    }
}

#[test]
fn the_versions_the_status_and_the_entry_points_are_described() {
    let api = api();
    assert_eq!(api.abi_version, TS_ABI_VERSION);
    assert_eq!(api.sdk_version, SDK_VERSION);
    let status = api.enum_named("Status").expect("the status enum");
    for value in &status.values {
        let code = i32::try_from(value.value).unwrap();
        assert_eq!(
            Status::from_code(code).map(|s| s.name().replace('_', "")),
            Some(value.name.to_ascii_uppercase()),
            "{}",
            value.name
        );
    }
    assert_eq!(api.blobs.len(), schemas::schemas().len());
    let kinds: Vec<&str> = api.enums.iter().filter_map(|e| e.kind.as_deref()).collect();
    assert!(
        kinds.contains(&"graha") && kinds.contains(&"calendar"),
        "{kinds:?}"
    );
    let symbols: Vec<&str> = api.functions.iter().map(|f| f.name.as_str()).collect();
    for expected in [
        "ts_abi_version",
        "ts_frame_canonical",
        "ts_frame_pack",
        "ts_frame_unpack",
        "ts_context_new",
        "ts_context_free",
        "ts_context_last_error",
        "ts_positions",
        "ts_intl_render",
        "ts_time_resolve",
        "ts_calendar_convert",
        "ts_key_parse",
        "ts_blob_free",
        "ts_string_free",
    ] {
        assert!(
            symbols.contains(&expected),
            "{expected} is not described; the description has {symbols:?}"
        );
    }
    let positions = api.function_named("ts_positions").unwrap();
    assert_eq!(positions.meta.blob.as_deref(), Some("positions"));
    let constants: Vec<&str> = api.constants.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        constants,
        [
            "VTABLE_ABI_VERSION",
            "TS_ABI_VERSION",
            "TS_CONTEXT_TEST_PROVIDER"
        ]
    );
}

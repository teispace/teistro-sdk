//! An adapter opened from a real shared library: the in-repo test
//! adapter, so the loader runs on every push rather than only where a
//! maintainer built an engine (ADR-0029).

#![allow(
    unsafe_code,
    clippy::unwrap_used,
    reason = "tests load a library they built, and fail by panicking"
)]

use teistro_core::error::Status;
use teistro_port_ephemeris::load::Adapter;
use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, PositionRequest, TimeScale};

fn adapter(config: Option<&str>) -> Result<Adapter, teistro_core::error::Error> {
    let path = teistro_test_adapter::library().unwrap();
    // SAFETY: the library is this workspace's own test adapter.
    unsafe { Adapter::load(&path.to_string_lossy(), config) }
}

#[test]
fn a_loaded_adapter_computes_and_outlives_its_handle() {
    let adapter = adapter(None).unwrap();
    let first = adapter.provider().unwrap();
    let second = adapter.provider().unwrap();
    // The handle goes first: each provider keeps the library loaded.
    drop(adapter);
    let jds = [2_451_545.0];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    let sun = first.positions(&request).unwrap();
    assert!(sun.at(0, 0).unwrap().is_ok());
    drop(first);
    assert!(second.capabilities().native, "the engine's own operations");
    let manifest = second.native_manifest().unwrap();
    assert!(manifest.contains("tp_sum"), "{manifest}");
    let summed = second
        .native_call("tp_sum", r#"{"values": [1, 2, 3.5]}"#)
        .unwrap();
    let summed: serde_json::Value = serde_json::from_str(&summed).unwrap();
    assert_eq!(summed.get("total"), Some(&serde_json::json!(6.5)));
}

#[test]
fn a_refusal_is_the_adapters_own_and_names_the_path() {
    let refused = adapter(Some(r#"{"data": "/nowhere"}"#)).unwrap_err();
    assert_eq!(refused.status, Status::InvalidArg, "{refused}");
    assert_eq!(refused.field(), Some("path"));
    assert!(refused.to_string().contains("/nowhere"), "{refused}");

    // SAFETY: a path that does not exist opens nothing.
    let missing = unsafe { Adapter::load("/nowhere/libnothing.so", None) }.unwrap_err();
    assert_eq!(missing.status, Status::Unsupported, "{missing}");
}

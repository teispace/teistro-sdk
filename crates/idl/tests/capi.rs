//! The C API's binding emitters over the boundary's description with one
//! shape changed that the checked-in description does not have: an opaque
//! type with no constructor and no destructor, no status enum, a fixed
//! section laid out as a shape two blobs could share, a 64-bit signed
//! column. Each emitter is held to what it writes, or leaves out, there.

#![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

use std::path::Path;

use serde_json::{Value, json};
use teistro_idl::emit::node::{self, Backend};
use teistro_idl::emit::{c, dart, java, mdx, python, ts};
use teistro_idl::model::Api;

fn description() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../idl/api.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn api(change: impl FnOnce(&mut Value)) -> Api {
    let mut value = description();
    change(&mut value);
    serde_json::from_value(value).unwrap()
}

/// The text from `start` to the first `end` after it.
fn block<'t>(text: &'t str, start: &str, end: &str) -> &'t str {
    let from = text.find(start).unwrap();
    let rest = &text[from..];
    let to = rest.find(end).map_or(rest.len(), |at| at + end.len());
    &rest[..to]
}

fn java_text(api: &Api) -> String {
    java::render(api)
        .into_iter()
        .map(|file| file.text)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn an_opaque_type_without_a_constructor_or_destructor_frees_nothing() {
    let lonely = api(|value| {
        value["opaques"].as_array_mut().unwrap().push(json!({
            "name": "TsLonely",
            "doc": "A handle nothing makes or frees.",
            "source": "crates/ffi/src/lonely.rs"
        }));
    });
    let dart = dart::declarations(&lonely);
    let class = block(
        &dart,
        "final class TeistroLonely implements ffi.Finalizable {",
        "\n}\n",
    );
    assert!(
        class.contains("final ffi.Pointer<Lonely> _handle;"),
        "{class}"
    );
    for absent in ["_freeAddress", "void dispose()", "_alive()"] {
        assert!(!class.contains(absent), "{absent}: {class}");
    }
    // The context, which has both, keeps them.
    let context = block(
        &dart,
        "final class TeistroContext implements ffi.Finalizable {",
        "\n}\n",
    );
    assert!(context.contains("void dispose()") && context.contains("_freeAddress"));

    let python = python::declarations(&lonely);
    let class = block(&python, "class TeistroLonely:", "\n\n\n");
    assert!(
        class.contains("weakref.finalize(self, _free_lonely, lib, handle)"),
        "{class}"
    );
    assert!(
        !python.contains("def _free_lonely("),
        "no finaliser is written for it"
    );
    assert!(python.contains("def _free_context("));

    let napi = node::render(&lonely, Backend::Napi);
    assert!(napi.contains("pub struct Lonely {"));
    assert!(!napi.contains("impl Drop for Lonely"));
    assert!(napi.contains("impl Drop for Context"));
    let methods = block(&napi, "impl Lonely {", "\n}\n");
    assert!(!methods.contains("pub fn dispose("), "{methods}");
    assert!(!methods.contains("pub fn new("), "{methods}");
    // The wasm glue binds only the handles some function it builds takes.
    let wasm = node::render(&lonely, Backend::WasmBindgen);
    assert!(!wasm.contains("pub struct Lonely {"));
    assert!(wasm.contains("impl Drop for Context"));
}

#[test]
fn a_description_without_a_status_enum_declares_no_exception() {
    let before = api(|_| {});
    let statusless = api(|value| {
        for e in value["enums"].as_array_mut().unwrap() {
            for v in e["values"].as_array_mut().unwrap() {
                if v["name"] == "Ok" {
                    v["name"] = json!("Fine");
                }
            }
        }
    });
    let exception = "final class TeistroException implements Exception {";
    assert!(dart::declarations(&before).contains(exception));
    assert!(!dart::declarations(&statusless).contains(exception));
    for backend in [Backend::Napi, Backend::WasmBindgen] {
        assert!(node::render(&before, backend).contains("pub struct LastError {"));
        let rust = node::render(&statusless, backend);
        assert!(!rust.contains("pub struct LastError {"), "{backend:?}");
        assert!(!rust.contains("fn refused("), "{backend:?}");
    }
}

#[test]
fn a_description_without_an_error_freer_refuses_no_handle_by_record() {
    let before = api(|_| {});
    let freer = teistro_idl::rules::error_free(&before)
        .unwrap()
        .name
        .clone();
    let freeless = api(|value| {
        value["functions"]
            .as_array_mut()
            .unwrap()
            .retain(|f| f["name"] != json!(freer));
    });
    for backend in [Backend::Napi, Backend::WasmBindgen] {
        assert!(node::render(&before, backend).contains("fn refused("));
        let rust = node::render(&freeless, backend);
        assert!(rust.contains("pub struct LastError {"), "{backend:?}");
        assert!(!rust.contains("fn refused("), "{backend:?}");
    }
}

#[test]
fn a_fixed_section_with_a_shape_is_one_type_in_every_binding() {
    let shaped = api(|value| {
        let naam = value["blobs"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|b| b["name"] == "naam")
            .unwrap();
        let section = naam["sections"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|s| s["name"] == "naam_varga")
            .unwrap();
        section["shape"] = json!("varga_standing");
        let fields = section["fields"].as_array_mut().unwrap();
        fields.push(json!({"name": "reserved", "scalar": "u8", "doc": "Kept for later."}));
        fields.push(json!({"name": "weight", "scalar": "u32", "doc": "A count."}));
    });
    let dart = dart::decoders(&shaped);
    let class = block(&dart, "final class VargaStanding {", "\n}\n");
    assert!(!class.contains("reserved"), "{class}");
    assert!(dart.contains("  final VargaStanding naamVarga;"), "{dart}");
    assert!(dart.contains("    naamVarga: VargaStanding(\n"), "{dart}");
    assert!(
        dart.contains("blob.data.getUint8(atNaamVarga.offset + 0),")
            && dart.contains("blob.data.getUint32(atNaamVarga.offset + 16, Endian.little),"),
        "a one-byte read takes no endianness, a wider one does, and the reserved slot is skipped: {dart}"
    );
    let python = python::decoders(&shaped);
    assert!(python.contains("class VargaStanding"), "{python}");
    let types = ts::blob_declarations(&shaped);
    assert!(types.contains("VargaStanding"), "{types}");
    let decoders = ts::decoders(&shaped);
    assert!(decoders.contains("naamVarga"), "{decoders}");
    let java = java_text(&shaped);
    assert!(java.contains("VargaStanding"));
}

#[test]
fn a_signed_64_bit_column_is_read_as_one() {
    let before = api(|_| {});
    let wide = api(|value| {
        let naam = value["blobs"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|b| b["name"] == "naam")
            .unwrap();
        let section = naam["sections"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|s| s["name"] == "naam_names")
            .unwrap();
        section["fields"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "serial", "scalar": "i64", "doc": "A signed serial."}));
    });
    assert!(!dart::decoders(&before).contains("Int64List"));
    assert!(dart::decoders(&wide).contains("Int64List"));
    let python = python::decoders(&wide);
    assert!(python.contains("serial"), "{python}");
    assert!(ts::decoders(&wide).contains("serial"));
    assert!(java_text(&wide).contains("serial"));
}

#[test]
fn a_deprecated_value_a_wide_constant_and_a_control_character() {
    let before = api(|_| {});
    let changed = api(|value| {
        let e = value["enums"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["name"] == "Status")
            .unwrap();
        let values = e["values"].as_array_mut().unwrap();
        values.get_mut(1).unwrap()["deprecated"] = json!(true);
        values.get_mut(2).unwrap()["key"] = json!("BELL\u{7}KEY");
        let constants = value["constants"].as_array_mut().unwrap();
        constants.push(
            json!({"name": "TS_PAST_AN_INT", "doc": "Past an int.", "ty": "u32",
            "value": 5_000_000_000_u64, "source": "crates/ffi/src/lib.rs"}),
        );
        constants.push(
            json!({"name": "TS_WIDE", "doc": "Of a wide type.", "ty": "u64",
            "value": 7, "source": "crates/ffi/src/lib.rs"}),
        );
    });
    // The header says so under the value.
    let deprecated = |text: &str| text.matches("Deprecated.").count();
    assert_eq!(
        deprecated(&c::render(&changed)),
        deprecated(&c::render(&before)) + 1
    );
    // Java tags it for Javadoc and the compiler both, writes a control
    // character as an escape, and a constant past an int as a long.
    let java = java_text(&changed);
    assert!(java.contains("@deprecated Kept so an old id still reads."));
    assert!(java.contains("    @Deprecated\n"));
    assert!(
        java.contains("\"BELL\\u0007KEY\""),
        "a control character is escaped"
    );
    assert!(java.contains(" = 5000000000L;"));
    assert!(java.contains("public static final long TS_WIDE = 7L;") || java.contains(" = 7L;"));
    assert!(!java_text(&before).contains("@Deprecated\n"));
}

#[test]
fn a_reference_page_keeps_the_lines_of_a_structured_doc_and_notes_each_parameter() {
    let doc = "A paragraph\nwrapped over\ntwo lines.\n\n- a list\n- of two\n\n* a star\n\n# A heading\n\n> A quotation\n\n```\nfenced\ncode\n```\n\n| a | table |\n\n1. numbered\n2. steps";
    let changed = api(|value| {
        let f = value["functions"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|f| f["name"] == "ts_positions")
            .unwrap();
        f["doc"] = json!(doc);
        let meta = &mut f["params"].as_array_mut().unwrap().get_mut(1).unwrap()["meta"];
        meta["range"] = json!("[0,1]");
        meta["brand"] = json!("latitude");
        meta["len"] = json!("count");
        meta["nullable"] = json!(true);
    });
    let page = mdx::render(&changed, &mdx::Reach::new())
        .into_iter()
        .find(|page| page.path.ends_with("ts_positions.mdx"))
        .unwrap();
    // A paragraph is joined onto one line; every structured block keeps
    // its own.
    for kept in [
        "A paragraph wrapped over two lines.",
        "- a list\n- of two",
        "* a star",
        "> A quotation",
        "fenced\ncode",
        "1. numbered\n2. steps",
    ] {
        assert!(page.text.contains(kept), "{kept:?}: {}", page.text);
    }
    for note in [
        "Range `[0,1]`.",
        "Carries a latitude, which the typed bindings refuse to swap.",
        "Its length is `count`.",
        "May be null.",
    ] {
        assert!(page.text.contains(note), "{note}: {}", page.text);
    }
}

//! The API description and everything rendered from it, generated from
//! the boundary crates: `gen ffi` extracts `idl/api.json` and renders the
//! C header, the Node binding's TypeScript surface, catalogue tables and
//! blob decoders, the Dart binding's layer, the Python binding's `ctypes`
//! layer, and the documentation site's reference; `check-ffi` regenerates
//! them all in memory and fails on any difference, so a new entry point, a
//! changed field or a reworded doc comment can never leave a binding — or
//! the reference — behind.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use teistro_idl::emit::{c, dart, mdx, node, python, records, ts};
use teistro_idl::sdk::describe;

use crate::generated::{Output, check, prune, strays, write};

const API_JSON: &str = "idl/api.json";
const C_HEADER: &str = "bindings/c/include/teistro.h";
const TS_CATALOGUE: &str = "bindings/node/lib/catalogue.d.ts";
const TS_TABLES: &str = "bindings/node/lib/catalogue.js";
const TS_TYPES: &str = "bindings/node/lib/types.d.ts";
const TS_BLOB_TYPES: &str = "bindings/node/lib/blob.d.ts";
const TS_DECODERS: &str = "bindings/node/lib/blob.js";
const NAPI_GLUE: &str = "bindings/node/native/src/generated.rs";
const DART_CATALOGUE: &str = "bindings/dart/lib/src/catalogue.dart";
const DART_FFI: &str = "bindings/dart/lib/src/ffi.dart";
const DART_BLOB: &str = "bindings/dart/lib/src/blob.dart";
const PYTHON_CATALOGUE: &str = "bindings/python/teistro/catalogue.py";
const PYTHON_FFI: &str = "bindings/python/teistro/_ffi.py";
const PYTHON_BLOB: &str = "bindings/python/teistro/_blob.py";
const TS_RECORDS: &str = "bindings/node/lib/records.d.ts";
const JS_RECORDS: &str = "bindings/node/lib/records.js";
const PYTHON_RECORDS: &str = "bindings/python/teistro/_records.py";
const DART_RECORDS: &str = "bindings/dart/lib/src/records.dart";
/// Where the site's generated reference lives. Everything under it is
/// written by this task, and anything else there is a stray.
const REFERENCE: &str = "site/content/docs/reference";

/// Rust text as `rustfmt` writes it, so the generated file passes the
/// format gate. Text rustfmt cannot parse comes back unchanged, and the
/// gate then says so.
fn rustfmt(text: &str) -> String {
    let child = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut child) = child else {
        eprintln!("no `rustfmt` on this machine; the generated glue is written unformatted");
        return text.to_string();
    };
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(text.as_bytes());
    }
    match child.wait_with_output() {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout).unwrap_or_else(|_| text.to_string())
        }
        _ => {
            eprintln!("`rustfmt` refused the generated glue; it is written unformatted");
            text.to_string()
        }
    }
}

/// The records that cross as JSON text inside a blob, named by their
/// schema: a result's provenance and a positions result's steps. Every
/// record they reach comes with them.
const RECORDS: [&str; 2] = ["Provenance", "Step"];

/// serde's own schema of the [`RECORDS`], as schemars writes it: the one
/// description of their JSON, which the document schema also reads for
/// the provenance a stored chart carries.
fn records_schema() -> serde_json::Value {
    let mut generator = schemars::generate::SchemaSettings::draft2020_12().into_generator();
    generator.subschema_for::<teistro_core::envelope::Provenance>();
    generator.subschema_for::<teistro_astro::completion::Step>();
    serde_json::json!({ "$defs": generator.definitions() })
}

/// The boundary's description with the JSON records it carries: what every
/// generated file and every measurement of the surfaces reads.
pub(crate) fn api(root: &Path) -> teistro_idl::model::Api {
    let mut api = describe(
        root,
        teistro_ffi::schemas::schemas(),
        teistro_ffi::SDK_VERSION,
    )
    .unwrap_or_else(|e| panic!("the boundary does not describe: {e}"));
    api.records = teistro_idl::records::from_schema(&records_schema(), &RECORDS)
        .unwrap_or_else(|e| panic!("the records do not describe: {e}"));
    let shadowing: Vec<&str> = api
        .records
        .iter()
        .map(|record| record.name.as_str())
        .filter(|name| teistro_idl::emit::reserved::PYTHON_BUILTINS.contains(name))
        .collect();
    assert!(
        shadowing.is_empty(),
        "these records would shadow a Python builtin: {shadowing:?}; name each apart with \
         #[cfg_attr(feature = \"schema\", schemars(rename = \"...\"))]"
    );
    api
}

fn outputs(root: &Path) -> Vec<Output> {
    let api = api(root);
    // A shape is one type in every binding, so two sections that name it
    // must agree. They cannot be checked at the emitters, which render a
    // shape once and would silently use whichever came first.
    if let Err(complaint) = teistro_idl::model::check_shapes(&api.blobs) {
        panic!("the blob schemas disagree: {complaint}");
    }
    eprintln!(
        "described ABI {}: {} constants, {} enums, {} opaques, {} callbacks, {} structs, {} functions, {} blob schemas",
        api.abi_version,
        api.constants.len(),
        api.enums.len(),
        api.opaques.len(),
        api.callbacks.len(),
        api.structs.len(),
        api.functions.len(),
        api.blobs.len()
    );
    let json = serde_json::to_string_pretty(&api).expect("the description serialises");
    let mut outputs = vec![
        Output::new(API_JSON, format!("{json}\n")),
        Output::new(C_HEADER, c::render(&api)),
        Output::new(TS_CATALOGUE, ts::catalogue_declarations(&api)),
        Output::new(TS_TABLES, ts::tables(&api)),
        Output::new(TS_TYPES, ts::type_declarations(&api)),
        Output::new(TS_BLOB_TYPES, ts::blob_declarations(&api)),
        Output::new(TS_DECODERS, ts::decoders(&api)),
        Output::new(DART_CATALOGUE, dart::catalogue(&api)),
        Output::new(DART_FFI, dart::declarations(&api)),
        Output::new(DART_BLOB, dart::decoders(&api)),
        Output::new(PYTHON_CATALOGUE, python::catalogue(&api)),
        Output::new(PYTHON_FFI, python::declarations(&api)),
        Output::new(PYTHON_BLOB, python::decoders(&api)),
        Output::new(TS_RECORDS, records::typescript_declarations(&api)),
        Output::new(JS_RECORDS, records::javascript_decoders(&api)),
        Output::new(PYTHON_RECORDS, records::python_records(&api)),
        Output::new(DART_RECORDS, records::dart_records(&api)),
        Output::new(
            NAPI_GLUE,
            // Formatted here rather than by `cargo fmt`, because napi's
            // derive macro reads the source file and a `rustfmt::skip` on
            // the module stops it finding the class before its `impl`.
            rustfmt(&node::render(&api)),
        ),
    ];
    outputs.extend(
        mdx::render(&api)
            .into_iter()
            .map(|page| Output::new(format!("{REFERENCE}/{}", page.path), page.text)),
    );
    outputs
}

pub(crate) fn generate(root: &Path) -> i32 {
    let outputs = outputs(root);
    let code = write(root, &outputs);
    prune(root, REFERENCE, &outputs);
    code
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    let outputs = outputs(root);
    let mut failures = check(root, &outputs, "cargo xtask gen ffi");
    for stray in strays(root, REFERENCE, &outputs) {
        println!("FAIL  {stray} is not generated by `cargo xtask gen ffi`; it is left over");
        failures += 1;
    }
    i32::from(failures != 0)
}

//! The API description and everything rendered from it, generated from
//! the boundary crates: `gen ffi` extracts `idl/api.json` and renders the
//! C header, the Node addon's and the wasm module's glue, the Node
//! binding's TypeScript surface, catalogue tables and blob decoders, the Dart binding's layer, the Python binding's `ctypes`
//! layer, and the documentation site's reference; `check-ffi` regenerates
//! them all in memory and fails on any difference, so a new entry point, a
//! changed field or a reworded doc comment can never leave a binding — or
//! the reference — behind.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use teistro_idl::emit::{c, dart, java, mdx, node, python, records, ts};
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
const WASM_GLUE: &str = "bindings/wasm/native/src/generated.rs";
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
/// The Java binding's generated source root: every file under it is
/// generated, so a file nothing generates is a stray and is pruned. The
/// hand-written layer is a source root of its own beside it.
const JAVA_GENERATED: &str = "bindings/java/generated";

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

/// The record entry points whose request stays a JSON value in every
/// binding: an engine's own operations, whose arguments only the engine
/// knows.
const UNTYPED_REQUESTS: [&str; 2] = ["engine.call", "engine.manifest"];

/// The parts of a request that are a JSON value and nothing narrower: the
/// rule language's untagged references (a subject, a sign, a body), a
/// record read in a second step by its own reader (a rule set, research's
/// test and control), and a theme's open style and content.
const RAW_REQUEST_PARTS: [&str; 11] = [
    "BodyRef",
    "BodySubject",
    "FestivalRequest.rules",
    "MuhurtaRequest.rules",
    "ResearchRequest.control",
    "ResearchRequest.rules",
    "ResearchRequest.test",
    "SignRef",
    "Subject",
    "ThemePatch.content",
    "ThemePatch.style",
];

/// Every record entry point's request, as the façade's own schemas state
/// it, under one generator so a name is one type across them all. A
/// request written inline is named after its record (`chart.found` is
/// `ChartFoundRequest`).
fn requests() -> teistro_idl::records::Requests {
    let mut generator = schemars::generate::SchemaSettings::draft2020_12().into_generator();
    let mut roots = Vec::new();
    let mut inline = serde_json::Map::new();
    let records = teistro::records::records();
    for untyped in UNTYPED_REQUESTS {
        assert!(
            records.iter().any(|record| record.name == untyped),
            "{untyped} is listed as an untyped request and is no record; take it off the list"
        );
    }
    for record in records {
        if UNTYPED_REQUESTS.contains(&record.name) {
            continue;
        }
        let schema = record
            .request_schema(&mut generator)
            .unwrap_or_else(|| panic!("{} states no request schema", record.name))
            .to_value();
        let name = if let Some(reference) = schema.get("$ref").and_then(serde_json::Value::as_str) {
            reference.trim_start_matches("#/$defs/").to_owned()
        } else {
            let name = format!(
                "{}Request",
                teistro_idl::names::pascal(&record.name.replace('.', "_"))
            );
            inline.insert(name.clone(), schema);
            name
        };
        roots.push(teistro_idl::records::RequestRoot {
            record: record.name.to_owned(),
            name,
        });
    }
    let mut defs = generator.take_definitions(true);
    for (name, schema) in inline {
        assert!(
            defs.insert(name.clone(), schema).is_none(),
            "{name} is a request written inline and a definition both"
        );
    }
    let names: Vec<&str> = roots.iter().map(|root| root.name.as_str()).collect();
    let records = teistro_idl::records::read(
        &serde_json::json!({ "$defs": defs }),
        &names,
        teistro_idl::records::Reading {
            raw: &RAW_REQUEST_PARTS,
        },
    )
    .unwrap_or_else(|e| panic!("the requests do not describe: {e}"));
    teistro_idl::records::Requests { roots, records }
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
    api.requests = requests();
    // The answer records' emitters render the subset the answers use; the
    // request side's constructs reach Java alone, through its own
    // emitter, so an answer carrying one is refused here rather than at
    // an emitter's `unreachable!`.
    let request_only: Vec<&str> = api
        .records
        .iter()
        .filter(|record| record_is_request_only(record))
        .map(|record| record.name.as_str())
        .collect();
    assert!(
        request_only.is_empty(),
        "these answer records use a construct only the request emitter renders: {request_only:?}"
    );
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

/// Whether a record uses a construct the answer emitters do not render.
fn record_is_request_only(record: &teistro_idl::records::RecordDef) -> bool {
    use teistro_idl::records::{RecordShape, RecordType};
    fn request_only(ty: &RecordType) -> bool {
        match ty {
            RecordType::Member { .. }
            | RecordType::Keyed { .. }
            | RecordType::Map { .. }
            | RecordType::Json => true,
            RecordType::List { of } => request_only(of),
            RecordType::Pair { first, second } => request_only(first) || request_only(second),
            RecordType::Bool
            | RecordType::Integer { .. }
            | RecordType::Number
            | RecordType::Text
            | RecordType::Record { .. } => false,
        }
    }
    match &record.shape {
        RecordShape::External { .. } | RecordShape::OrNamed { .. } => true,
        RecordShape::Keys { .. } => false,
        RecordShape::Object { fields } => fields.iter().any(|field| request_only(&field.ty)),
        RecordShape::Tagged { variants, .. } => variants
            .iter()
            .flat_map(|variant| &variant.fields)
            .any(|field| request_only(&field.ty)),
    }
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
            rustfmt(&node::render(&api, node::Backend::Napi)),
        ),
        Output::new(
            WASM_GLUE,
            rustfmt(&node::render(&api, node::Backend::WasmBindgen)),
        ),
    ];
    outputs.extend(
        java::render(&api)
            .into_iter()
            .map(|file| Output::new(format!("{JAVA_GENERATED}/{}", file.path), file.text)),
    );
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
    for dir in [REFERENCE, JAVA_GENERATED] {
        prune(root, dir, &outputs);
    }
    code
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    let outputs = outputs(root);
    let mut failures = check(root, &outputs, "cargo xtask gen ffi");
    for dir in [REFERENCE, JAVA_GENERATED] {
        for stray in strays(root, dir, &outputs) {
            println!("FAIL  {stray} is not generated by `cargo xtask gen ffi`; it is left over");
            failures += 1;
        }
    }
    i32::from(failures != 0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    /// **Every record entry point but the engine's has its request read**,
    /// each by a name of its own, and the request records name one type
    /// a name.
    #[test]
    fn every_typed_request_is_read_once() {
        let requests = requests();
        let records = teistro::records::records();
        assert_eq!(
            requests.roots.len(),
            records.len() - UNTYPED_REQUESTS.len(),
            "a record's request was left unread"
        );
        let mut names: Vec<&str> = requests
            .records
            .iter()
            .map(|record| record.name.as_str())
            .collect();
        let all = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), all, "two request records share a name");
        for root in &requests.roots {
            assert!(
                names.binary_search(&root.name.as_str()).is_ok(),
                "{} names {}, which was not read",
                root.record,
                root.name
            );
        }
    }
}

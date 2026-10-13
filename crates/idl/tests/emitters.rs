//! Every emitter over the boundary's checked-in description.
//!
//! `cargo xtask check-generated` is what holds the generated files to the
//! emitters, but xtask is not a measured crate, so without this test the
//! emitters would count as unexercised. Here each one renders the real
//! description, and each file it renders from the description alone must
//! equal the one checked in. The glue rustfmt formats and the reference,
//! which also reads the façades' spellings, are held to what they must
//! contain instead. The Java request types are not rendered here: their
//! description is read from the SDK's schemas by xtask and is not in
//! `idl/api.json`.

#![allow(
    clippy::expect_used,
    reason = "a test fails on the first thing it cannot read"
)]

use std::path::{Path, PathBuf};

use teistro_idl::emit::{c, dart, java, mdx, node, python, records, ts};
use teistro_idl::model::Api;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn api() -> Api {
    let text = std::fs::read_to_string(repo().join("idl/api.json")).expect("idl/api.json");
    serde_json::from_str(&text).expect("idl/api.json describes the boundary")
}

#[test]
fn every_file_rendered_from_the_description_is_the_one_checked_in() {
    let api = api();
    let rendered: [(&str, String); 16] = [
        ("bindings/c/include/teistro.h", c::render(&api)),
        (
            "bindings/node/lib/catalogue.d.ts",
            ts::catalogue_declarations(&api),
        ),
        ("bindings/node/lib/catalogue.js", ts::tables(&api)),
        ("bindings/node/lib/types.d.ts", ts::type_declarations(&api)),
        ("bindings/node/lib/blob.d.ts", ts::blob_declarations(&api)),
        ("bindings/node/lib/blob.js", ts::decoders(&api)),
        (
            "bindings/dart/lib/src/catalogue.dart",
            dart::catalogue(&api),
        ),
        ("bindings/dart/lib/src/ffi.dart", dart::declarations(&api)),
        ("bindings/dart/lib/src/blob.dart", dart::decoders(&api)),
        (
            "bindings/python/teistro/catalogue.py",
            python::catalogue(&api),
        ),
        (
            "bindings/python/teistro/_ffi.py",
            python::declarations(&api),
        ),
        ("bindings/python/teistro/_blob.py", python::decoders(&api)),
        (
            "bindings/node/lib/records.d.ts",
            records::typescript_declarations(&api),
        ),
        (
            "bindings/node/lib/records.js",
            records::javascript_decoders(&api),
        ),
        (
            "bindings/python/teistro/_records.py",
            records::python_records(&api),
        ),
        (
            "bindings/dart/lib/src/records.dart",
            records::dart_records(&api),
        ),
    ];
    let differ: Vec<&str> = rendered
        .iter()
        .filter(|(path, text)| {
            std::fs::read_to_string(repo().join(path)).ok().as_ref() != Some(text)
        })
        .map(|(path, _)| *path)
        .collect();
    assert!(
        differ.is_empty(),
        "rendered differently from the checked-in file: {differ:?}"
    );
}

#[test]
fn the_java_files_are_the_ones_checked_in() {
    let files = java::render(&api());
    let differ: Vec<&str> = files
        .iter()
        .filter(|file| {
            std::fs::read_to_string(repo().join("bindings/java/generated").join(&file.path))
                .ok()
                .as_ref()
                != Some(&file.text)
        })
        .map(|file| file.path.as_str())
        .collect();
    assert_ne!(files.len(), 0);
    assert!(
        differ.is_empty(),
        "rendered differently from the checked-in file: {differ:?}"
    );
}

/// Every page under a directory, its `meta.json` included, relative to it.
fn pages(dir: &Path, under: &Path, found: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("the reference directory") {
        let path = entry.expect("a reference entry").path();
        if path.is_dir() {
            pages(&path, under, found);
        } else if path
            .extension()
            .is_some_and(|ext| ext == "mdx" || ext == "json")
        {
            let relative = path.strip_prefix(under).expect("under the reference");
            found.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}

#[test]
fn the_reference_renders_every_page_checked_in() {
    let mut rendered: Vec<String> = mdx::render(&api(), &mdx::Reach::new())
        .into_iter()
        .map(|page| page.path)
        .collect();
    let under = repo().join("site/content/docs/reference");
    let mut checked_in = Vec::new();
    pages(&under, &under, &mut checked_in);
    rendered.sort();
    checked_in.sort();
    assert_eq!(rendered, checked_in);
}

/// The wasm module has no plugin loader, so it leaves out what the
/// description marks native-only (ADR-0029); Node calls everything.
#[test]
fn both_glues_call_every_function_their_target_builds() {
    let api = api();
    for backend in [node::Backend::Napi, node::Backend::WasmBindgen] {
        let glue = node::render(&api, backend);
        let missing: Vec<&str> = api
            .functions
            .iter()
            .filter(|function| backend == node::Backend::Napi || !function.native_only)
            .map(|function| function.name.as_str())
            .filter(|name| !glue.contains(name))
            .collect();
        assert!(
            missing.is_empty(),
            "{backend:?} glue never calls {missing:?}"
        );
    }
    let wasm = node::render(&api, node::Backend::WasmBindgen);
    let native: Vec<&str> = api
        .functions
        .iter()
        .filter(|function| function.native_only)
        .map(|function| function.name.as_str())
        .collect();
    assert_ne!(native.len(), 0);
    assert!(
        native.iter().all(|name| !wasm.contains(name)),
        "the wasm glue reaches a native-only function: {native:?}"
    );
}

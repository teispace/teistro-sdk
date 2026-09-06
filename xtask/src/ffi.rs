//! The API description and everything rendered from it, generated from
//! the boundary crates: `gen ffi` extracts `idl/api.json` and renders the
//! C header and the Node binding's TypeScript surface, catalogue tables
//! and blob decoders; `check-ffi` regenerates them all in memory and
//! fails on any difference, so a new entry point, a changed field or a
//! reworded doc comment can never leave a binding behind.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use teistro_idl::emit::{c, node, ts};
use teistro_idl::sdk::describe;

use crate::generated::{Output, check, write};

const API_JSON: &str = "idl/api.json";
const C_HEADER: &str = "bindings/c/include/teistro.h";
const TS_CATALOGUE: &str = "bindings/node/lib/catalogue.d.ts";
const TS_TABLES: &str = "bindings/node/lib/catalogue.js";
const TS_TYPES: &str = "bindings/node/lib/types.d.ts";
const TS_BLOB_TYPES: &str = "bindings/node/lib/blob.d.ts";
const TS_DECODERS: &str = "bindings/node/lib/blob.js";
const NAPI_GLUE: &str = "bindings/node/native/src/generated.rs";

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

fn outputs(root: &Path) -> Vec<Output> {
    let api = describe(
        root,
        teistro_ffi::schemas::schemas(),
        teistro_ffi::SDK_VERSION,
    )
    .unwrap_or_else(|e| panic!("the boundary does not describe: {e}"));
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
    vec![
        Output {
            path: API_JSON,
            text: format!("{json}\n"),
        },
        Output {
            path: C_HEADER,
            text: c::render(&api),
        },
        Output {
            path: TS_CATALOGUE,
            text: ts::catalogue_declarations(&api),
        },
        Output {
            path: TS_TABLES,
            text: ts::tables(&api),
        },
        Output {
            path: TS_TYPES,
            text: ts::type_declarations(&api),
        },
        Output {
            path: TS_BLOB_TYPES,
            text: ts::blob_declarations(&api),
        },
        Output {
            path: TS_DECODERS,
            text: ts::decoders(&api),
        },
        Output {
            path: NAPI_GLUE,
            // Formatted here rather than by `cargo fmt`, because napi's
            // derive macro reads the source file and a `rustfmt::skip` on
            // the module stops it finding the class before its `impl`.
            text: rustfmt(&node::render(&api)),
        },
    ]
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &outputs(root))
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    let failures = check(root, &outputs(root), "cargo xtask gen ffi");
    i32::from(failures != 0)
}

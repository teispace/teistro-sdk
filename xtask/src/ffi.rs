//! The API description and the C header, generated from the boundary
//! crates: `gen ffi` extracts `idl/api.json` and renders
//! `bindings/c/include/teistro.h`; `check-ffi` regenerates both in memory
//! and fails on any difference, so a new entry point, a changed field or a
//! reworded doc comment can never leave the header or a binding behind.

use std::path::Path;

use teistro_idl::emit::c;
use teistro_idl::sdk::describe;

use crate::generated::{Output, check, write};

const API_JSON: &str = "idl/api.json";
const C_HEADER: &str = "bindings/c/include/teistro.h";

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
    ]
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &outputs(root))
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    let failures = check(root, &outputs(root), "cargo xtask gen ffi");
    i32::from(failures != 0)
}

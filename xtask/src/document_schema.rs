//! `document-schema` and `check-document-schema`: the JSON Schemas of a
//! sealed chart document and of its gochar sidecar, generated from the
//! types serde writes them with (`docs/03-design/document-schema.md`,
//! `docs/03-design/gochar-sidecar.md` §7).
//!
//! The schema is checked in beside the crate that embeds it, so a change
//! to the document's shape shows in the diff of the pull request that made
//! it, and the constant a consumer reads can never be older than the types.

use std::path::Path;

use crate::generated::{Output, check, write};

/// Where the schemas live: inside the crate that embeds them, because a
/// published crate cannot include a file from outside itself.
const FILE: &str = "crates/serial/schema/document.schema.json";
/// The gochar sidecar's schema, beside the document's.
const SIDECAR_FILE: &str = "crates/serial/schema/gochar-sidecar.schema.json";

fn outputs() -> Result<[Output; 2], String> {
    Ok([
        Output::new(FILE, teistro_serial::schema::generate()?),
        Output::new(SIDECAR_FILE, teistro_serial::schema::generate_sidecar()?),
    ])
}

pub(crate) fn generate(root: &Path) -> i32 {
    match outputs() {
        Ok(outputs) => write(root, &outputs),
        Err(err) => refused(&err),
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs() {
        Ok(outputs) => i32::from(check(root, &outputs, "cargo xtask document-schema") != 0),
        Err(err) => refused(&err),
    }
}

fn refused(err: &str) -> i32 {
    println!("FAIL  {err}");
    1
}

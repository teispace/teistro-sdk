//! `document-schema` and `check-document-schema`: the JSON Schema of a
//! sealed chart document, generated from the types serde writes it with
//! (`docs/03-design/document-schema.md`).
//!
//! The schema is checked in beside the crate that embeds it, so a change
//! to the document's shape shows in the diff of the pull request that made
//! it, and the constant a consumer reads can never be older than the types.

use std::path::Path;

use crate::generated::{Output, check, write};

/// Where the schema lives: inside the crate that embeds it, because a
/// published crate cannot include a file from outside itself.
const FILE: &str = "crates/serial/schema/document.schema.json";

fn outputs() -> Result<[Output; 1], String> {
    Ok([Output::new(FILE, teistro_serial::schema::generate()?)])
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

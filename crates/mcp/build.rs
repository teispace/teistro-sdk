//! Carries the boundary's documentation of each record into the server,
//! so a tool's description is the sentence `idl/api.json` already holds
//! for the record it reads (`03-design/mcp-server.md` D3) and not a second
//! copy kept equal by hand; and the catalogue, a kind at a time, so the
//! resource `teistro://catalogue/{kind}` answers text written once at
//! build time (§7, P2).

use std::fmt::Write as _;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../idl/api.json");
    println!("cargo::rerun-if-changed={}", api.display());
    let api: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&api)?)?;
    let functions = api
        .get("functions")
        .and_then(serde_json::Value::as_array)
        .ok_or("idl/api.json has no `functions` array")?;
    let mut out = String::from(
        "/// Every boundary function's documentation, by name.\nconst BOUNDARY_DOCS: &[(&str, &str)] = &[\n",
    );
    for function in functions {
        let (Some(name), Some(doc)) = (
            function.get("name").and_then(serde_json::Value::as_str),
            function.get("doc").and_then(serde_json::Value::as_str),
        ) else {
            continue;
        };
        writeln!(out, "    ({name:?}, {doc:?}),")?;
    }
    out.push_str("];\n");
    let target = Path::new(&std::env::var("OUT_DIR")?).join("boundary_docs.rs");
    std::fs::write(target, out)?;
    catalogue()
}

/// `catalogue/catalogue.json` as one row a kind: its name, number, first
/// line of documentation, whether consumers register its members, and
/// the kind's own JSON.
fn catalogue() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalogue/catalogue.json");
    println!("cargo::rerun-if-changed={}", path.display());
    let catalogue: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
    let schema = catalogue
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .ok_or("catalogue/catalogue.json names no `schema`")?;
    let mut out = format!(
        "/// The catalogue's format, as the file names it.\nconst CATALOGUE_SCHEMA: &str = {schema:?};\n\n\
         /// Every kind: name, number, documentation, open, JSON.\n\
         const CATALOGUE_KINDS: &[(&str, u64, &str, bool, &str)] = &[\n"
    );
    for (key, open) in [("kinds", false), ("open_kinds", true)] {
        let kinds = catalogue
            .get(key)
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("catalogue/catalogue.json has no `{key}` array"))?;
        for kind in kinds {
            let (Some(name), Some(number), Some(doc)) = (
                kind.get("kind").and_then(serde_json::Value::as_str),
                kind.get("number").and_then(serde_json::Value::as_u64),
                kind.get("doc").and_then(serde_json::Value::as_str),
            ) else {
                return Err(format!("a kind under `{key}` lacks its name, number or doc").into());
            };
            let json = serde_json::to_string(kind)?;
            writeln!(out, "    ({name:?}, {number}, {doc:?}, {open}, {json:?}),")?;
        }
    }
    out.push_str("];\n");
    let target = Path::new(&std::env::var("OUT_DIR")?).join("catalogue_kinds.rs");
    std::fs::write(target, out)?;
    Ok(())
}

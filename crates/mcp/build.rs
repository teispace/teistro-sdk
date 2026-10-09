//! Carries the boundary's documentation of each record into the server,
//! so a tool's description is the sentence `idl/api.json` already holds
//! for the record it reads (`03-design/mcp-server.md` D3) and not a second
//! copy kept equal by hand.

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
    Ok(())
}

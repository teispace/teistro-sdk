//! `check-abi [--against REF]`: the C ABI breaks nothing a release
//! promised unless its version says so (`05-testing/01-quality-bar.md`,
//! "public API stability").
//!
//! The boundary's description, `idl/api.json`, is compared with the one at
//! a baseline: `REF`, or by default the latest `v*` tag before the commit
//! checked out, so a release is compared with the one before it. What a binding
//! generated from the baseline relies on must still be there and mean the
//! same:
//!
//! - a function, callback, struct, opaque or constant is removed or
//!   changes in any part a binding reads (its parameters, return, fields
//!   in order, value);
//! - an enum loses a member or renumbers one;
//! - a blob loses a section or changes one's columns, or changes its id;
//! - a record loses a field or changes one's type.
//!
//! Adding is never a break: a new function, member, section or field is
//! unknown to an old binding, which reads past it. A break passes only
//! when `abi_version` rose since the baseline, which is what makes a
//! binding generated for the old ABI refuse the new library rather than
//! misread it. Documentation, sources and metadata never break anything.
//! With no `v*` tag and no `--against`, nothing has been released and
//! there is nothing to compare with.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

/// The description, relative to the repository.
const DESCRIPTION: &str = "idl/api.json";

/// Keys that describe an item to a reader and never change what a binding
/// does with it.
const PROSE: [&str; 3] = ["doc", "source", "meta"];

pub(crate) fn check(root: &Path, args: &[String]) -> i32 {
    let against = args
        .iter()
        .position(|arg| arg == "--against")
        .and_then(|at| args.get(at + 1))
        .cloned()
        .or_else(|| latest_tag(root));
    let Some(against) = against else {
        println!("ok    no `v*` tag: nothing has been released, so the ABI has nothing to keep");
        return 0;
    };
    match compare(root, &against) {
        Ok(code) => code,
        Err(why) => {
            println!("FAIL  {why}");
            1
        }
    }
}

fn latest_tag(root: &Path) -> Option<String> {
    let out = Command::new("git")
        // From the parent, so a release's own tag is never its baseline.
        .args(["describe", "--tags", "--abbrev=0", "--match", "v*", "HEAD^"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|out| out.status.success())?;
    let tag = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (!tag.is_empty()).then_some(tag)
}

fn compare(root: &Path, against: &str) -> Result<i32, String> {
    let out = Command::new("git")
        .args(["show", &format!("{against}:{DESCRIPTION}")])
        .current_dir(root)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git cannot read {DESCRIPTION} at `{against}`: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let base: Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("{DESCRIPTION} at `{against}`: {e}"))?;
    let text = std::fs::read_to_string(root.join(DESCRIPTION))
        .map_err(|e| format!("{DESCRIPTION}: {e}"))?;
    let now: Value = serde_json::from_str(&text).map_err(|e| format!("{DESCRIPTION}: {e}"))?;
    let found = breaks(&base, &now);
    let version = |v: &Value| v.get("abi_version").and_then(Value::as_u64).unwrap_or(0);
    let (was, is) = (version(&base), version(&now));
    if found.is_empty() {
        println!("ok    the ABI keeps everything `{against}` described (abi_version {was} → {is})");
        return Ok(0);
    }
    let listed = found.join("\n      ");
    if is > was {
        println!(
            "ok    {} break(s) since `{against}`, and abi_version rose from {was} to {is}:\n      \
             {listed}",
            found.len()
        );
        Ok(0)
    } else {
        println!(
            "FAIL  {} break(s) since `{against}` with abi_version still {is}; restore what a \
             binding relies on, or raise TS_ABI_VERSION in crates/ffi/src/lib.rs:\n      {listed}",
            found.len()
        );
        Ok(1)
    }
}

/// An item without the keys that only describe it.
fn bare(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| !PROSE.contains(&key.as_str()))
                .map(|(key, value)| (key.clone(), bare(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(bare).collect()),
        other => other.clone(),
    }
}

/// The items of a list, by the value under `key`.
fn by<'a>(list: Option<&'a Value>, key: &str) -> BTreeMap<String, &'a Value> {
    list.and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let id = item.get(key)?;
            Some((
                id.as_str().map_or_else(|| id.to_string(), str::to_owned),
                item,
            ))
        })
        .collect()
}

/// Every way `now` breaks what `base` described.
fn breaks(base: &Value, now: &Value) -> Vec<String> {
    let mut found = Vec::new();
    // Whole items: any change a binding reads is a break.
    for (list, what) in [
        ("functions", "function"),
        ("callbacks", "callback"),
        ("structs", "struct"),
        ("opaques", "opaque"),
        ("constants", "constant"),
    ] {
        let after = by(now.get(list), "name");
        for (name, before) in by(base.get(list), "name") {
            match after.get(&name) {
                None => found.push(format!("{what} `{name}` is gone")),
                Some(item) if bare(item) != bare(before) => {
                    found.push(format!("{what} `{name}` changed"));
                }
                Some(_) => {}
            }
        }
    }
    // Enums: members may be added, never lost or renumbered.
    let after = by(now.get("enums"), "name");
    for (name, before) in by(base.get("enums"), "name") {
        let Some(item) = after.get(&name) else {
            found.push(format!("enum `{name}` is gone"));
            continue;
        };
        if item.get("repr") != before.get("repr") {
            found.push(format!("enum `{name}` changed its representation"));
        }
        let members = by(item.get("values"), "name");
        for (member, was) in by(before.get("values"), "name") {
            match members.get(&member) {
                None => found.push(format!("enum `{name}` lost `{member}`")),
                Some(is) if is.get("value") != was.get("value") => {
                    found.push(format!("enum `{name}` renumbered `{member}`"));
                }
                Some(_) => {}
            }
        }
    }
    // Blobs: sections may be added, never lost or changed.
    let after = by(now.get("blobs"), "name");
    for (name, before) in by(base.get("blobs"), "name") {
        let Some(item) = after.get(&name) else {
            found.push(format!("blob `{name}` is gone"));
            continue;
        };
        if item.get("id") != before.get("id") {
            found.push(format!("blob `{name}` changed its id"));
        }
        let sections = by(item.get("sections"), "id");
        for (id, was) in by(before.get("sections"), "id") {
            match sections.get(&id) {
                None => found.push(format!("blob `{name}` lost section {id}")),
                Some(is) if bare(is) != bare(was) => {
                    found.push(format!("blob `{name}` changed section {id}"));
                }
                Some(_) => {}
            }
        }
    }
    // Records: fields may be added, never lost or retyped.
    let after = by(now.get("records"), "name");
    for (name, before) in by(base.get("records"), "name") {
        let Some(item) = after.get(&name) else {
            found.push(format!("record `{name}` is gone"));
            continue;
        };
        let fields = by(item.pointer("/shape/fields"), "name");
        for (field, was) in by(before.pointer("/shape/fields"), "name") {
            match fields.get(&field) {
                None => found.push(format!("record `{name}` lost `{field}`")),
                Some(is) if bare(is) != bare(was) => {
                    found.push(format!("record `{name}` changed `{field}`"));
                }
                Some(_) => {}
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn description() -> Value {
        json!({
            "abi_version": 1,
            "functions": [{"name": "ts_f", "doc": "F.", "params": [{"name": "x", "ty": "u32"}], "returns": "i32"}],
            "structs": [{"name": "S", "fields": [{"name": "a", "ty": "f64", "doc": "A."}]}],
            "enums": [{"name": "E", "repr": "u8", "values": [{"name": "One", "value": 1}]}],
            "blobs": [{"name": "b", "id": 1, "sections": [{"id": 1, "name": "s", "fields": [{"name": "c", "scalar": "u32"}]}]}],
            "records": [{"name": "R", "shape": {"fields": [{"name": "f", "ty": {"kind": "STRING"}}]}}],
        })
    }

    /// Additions and prose are never breaks.
    #[test]
    fn adding_and_rewording_break_nothing() {
        let base = description();
        let mut now = description();
        now["functions"][0]["doc"] = json!("Reworded.");
        now["functions"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "ts_g", "params": [], "returns": "i32"}));
        now["enums"][0]["values"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "Two", "value": 2}));
        now["blobs"][0]["sections"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id": 2, "name": "t", "fields": []}));
        now["records"][0]["shape"]["fields"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "g", "ty": {"kind": "STRING"}}));
        assert_eq!(breaks(&base, &now), Vec::<String>::new());
    }

    /// Each kind of loss or change is named.
    #[test]
    fn a_loss_or_a_change_is_a_break() {
        let base = description();
        let mut now = description();
        now["functions"][0]["params"][0]["ty"] = json!("u64");
        now["structs"][0]["fields"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "b", "ty": "f64"}));
        now["enums"][0]["values"][0]["value"] = json!(3);
        now["blobs"][0]["sections"][0]["fields"][0]["scalar"] = json!("u16");
        now["records"][0]["shape"]["fields"] = json!([]);
        assert_eq!(
            breaks(&base, &now),
            [
                "function `ts_f` changed",
                "struct `S` changed",
                "enum `E` renumbered `One`",
                "blob `b` changed section 1",
                "record `R` lost `f`",
            ]
        );
    }
}

//! `cargo xtask wasm-crates [MODULE]`: a wasm module's code by Rust crate.
//!
//! Reads the module's name section, which the cargo build keeps and the
//! staged package drops, and counts each function body's bytes to a crate
//! (`03-design/wasm-profiles.md`). Rust's v0 mangling names every crate in
//! a function's path, so a generic instantiated for a type of the SDK's
//! names both; the function is counted to the first SDK crate it names,
//! since it is in the module because that crate is, else to the first
//! crate outside the standard library, else to its own.
//!
//! A report, not a gate: the module is a release build, which no fast
//! gate can afford.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// The module the cargo build writes before staging strips its names.
const MODULE: &str = "target/wasm32-unknown-unknown/wasm/teistro_wasm.wasm";

/// The standard library's crates, which a function is counted to only
/// when it names nothing else.
const STANDARD: [&str; 5] = ["core", "alloc", "std", "compiler_builtins", "hashbrown"];

/// A LEB128 unsigned integer at `at`, and where the next field starts.
fn leb(bytes: &[u8], mut at: usize) -> Option<(u64, usize)> {
    let (mut value, mut shift) = (0_u64, 0_u32);
    loop {
        let byte = *bytes.get(at)?;
        at += 1;
        value |= u64::from(byte & 0x7f).checked_shl(shift)?;
        if byte < 0x80 {
            return Some((value, at));
        }
        shift += 7;
    }
}

/// A length-prefixed name at `at`, and where the next field starts.
fn name(bytes: &[u8], at: usize) -> Option<(&[u8], usize)> {
    let (length, at) = leb(bytes, at)?;
    let end = at.checked_add(usize::try_from(length).ok()?)?;
    Some((bytes.get(at..end)?, end))
}

/// What the report needs of a module: how many functions it imports,
/// each defined function's body size, and the names the name section
/// gives by function index.
struct Module {
    imported: usize,
    bodies: Vec<usize>,
    names: BTreeMap<usize, String>,
}

/// The imports section's function count, skipping every other import.
fn imported_functions(bytes: &[u8], mut at: usize) -> Option<usize> {
    let (count, next) = leb(bytes, at)?;
    at = next;
    let mut functions = 0;
    for _ in 0..count {
        at = name(bytes, at)?.1;
        at = name(bytes, at)?.1;
        let kind = *bytes.get(at)?;
        at += 1;
        // A limits field: a flag, a minimum, and a maximum when flagged.
        let limits = |at: usize| -> Option<usize> {
            let (flags, at) = leb(bytes, at)?;
            let at = leb(bytes, at)?.1;
            if flags & 1 == 1 {
                Some(leb(bytes, at)?.1)
            } else {
                Some(at)
            }
        };
        at = match kind {
            0 => {
                functions += 1;
                leb(bytes, at)?.1
            }
            1 => limits(at + 1)?,
            2 => limits(at)?,
            3 => at + 2,
            _ => return None,
        };
    }
    Some(functions)
}

fn parse(bytes: &[u8]) -> Option<Module> {
    if bytes.get(..4)? != b"\0asm" {
        return None;
    }
    let mut module = Module {
        imported: 0,
        bodies: Vec::new(),
        names: BTreeMap::new(),
    };
    let mut at = 8;
    while at < bytes.len() {
        let id = *bytes.get(at)?;
        let (size, start) = leb(bytes, at + 1)?;
        let end = start.checked_add(usize::try_from(size).ok()?)?;
        match id {
            2 => module.imported = imported_functions(bytes, start)?,
            10 => {
                let (count, mut body) = leb(bytes, start)?;
                for _ in 0..count {
                    let (length, code) = leb(bytes, body)?;
                    let next = code.checked_add(usize::try_from(length).ok()?)?;
                    module.bodies.push(next - body);
                    body = next;
                }
            }
            0 => {
                let (section, mut sub) = name(bytes, start)?;
                if section == b"name" {
                    while sub < end {
                        let kind = *bytes.get(sub)?;
                        let (length, content) = leb(bytes, sub + 1)?;
                        let next = content.checked_add(usize::try_from(length).ok()?)?;
                        if kind == 1 {
                            let (count, mut entry) = leb(bytes, content)?;
                            for _ in 0..count {
                                let (index, at) = leb(bytes, entry)?;
                                let (text, at) = name(bytes, at)?;
                                module.names.insert(
                                    usize::try_from(index).ok()?,
                                    String::from_utf8_lossy(text).into_owned(),
                                );
                                entry = at;
                            }
                        }
                        sub = next;
                    }
                }
            }
            _ => {}
        }
        at = end;
    }
    Some(module)
}

/// Every crate a v0-mangled name carries, in order: each crate root is
/// `C`, an optional `s<base62>_` disambiguator, then a decimal length, an
/// optional `_`, and the identifier.
fn crates_in(symbol: &str) -> Vec<&str> {
    let bytes = symbol.as_bytes();
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(offset) = symbol.get(at..).and_then(|rest| rest.find('C')) {
        let mut cursor = at + offset + 1;
        if bytes.get(cursor) == Some(&b's') {
            while bytes.get(cursor).is_some_and(|b| *b != b'_') {
                cursor += 1;
            }
            cursor += 1;
        }
        let digits = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        let length = symbol
            .get(digits..cursor)
            .and_then(|d| d.parse::<usize>().ok());
        if bytes.get(cursor) == Some(&b'_') {
            cursor += 1;
        }
        if let Some(ident) = length.and_then(|n| symbol.get(cursor..cursor + n)) {
            let well_formed = ident.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
                && ident
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            if well_formed {
                found.push(ident);
            }
        }
        at = at + offset + 1;
    }
    found
}

/// The crate a function is in the module for.
fn crate_of(symbol: &str) -> &str {
    let found = crates_in(symbol);
    found
        .iter()
        .find(|c| c.starts_with("teistro"))
        .or_else(|| found.iter().find(|c| !STANDARD.contains(c)))
        .or_else(|| found.first())
        .copied()
        .unwrap_or("(unnamed)")
}

pub(crate) fn report(root: &Path, module: Option<&str>) -> i32 {
    let path = module.map_or_else(|| root.join(MODULE), |given| root.join(given));
    let Ok(bytes) = fs::read(&path) else {
        println!(
            "FAIL  {} could not be read; build it with `cargo xtask package wasm` first",
            path.display()
        );
        return 1;
    };
    let Some(parsed) = parse(&bytes) else {
        println!(
            "FAIL  {} is not a wasm module this reader follows",
            path.display()
        );
        return 1;
    };
    if parsed.names.is_empty() {
        println!(
            "FAIL  {} has no function names; read the cargo build's module, not the staged one",
            path.display()
        );
        return 1;
    }
    let mut by: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, size) in parsed.bodies.iter().enumerate() {
        let symbol = parsed
            .names
            .get(&(parsed.imported + index))
            .map_or("", String::as_str);
        *by.entry(crate_of(symbol)).or_default() += size;
    }
    let total: usize = by.values().sum();
    let mut rows: Vec<(&str, usize)> = by.into_iter().collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!(
        "{} bytes of code in {} functions, {}",
        total,
        parsed.bodies.len(),
        path.display()
    );
    for (name, size) in rows {
        #[allow(clippy::cast_precision_loss)]
        let share = 100.0 * size as f64 / total.max(1) as f64;
        println!("{size:>10} {share:>5.1}%  {name}");
    }
    0
}

#[cfg(test)]
mod tests {
    use super::{crate_of, crates_in};

    #[test]
    fn a_v0_name_carries_its_crates() {
        let sort = "_RINvNtNtNtNtCs8hZryZq3oeI_4core5slice4sort6stable5drift10create_runNtNtCs5rtjQqJ2XCi_22teistro_port_ephemeris8crossing5EventE";
        assert_eq!(crates_in(sort), ["core", "teistro_port_ephemeris"]);
        assert_eq!(crate_of(sort), "teistro_port_ephemeris");
        let glue = "_RNvCs3WfNrDpPat_12wasm_bindgen26___wbindgen_object_drop_ref";
        assert_eq!(crate_of(glue), "wasm_bindgen");
        assert_eq!(crate_of("memcpy"), "(unnamed)");
    }
}

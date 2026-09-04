//! Spike 2, option A: the extractor and the generators.
//!
//! `extract <lib.rs> <api.json>` reads the C ABI crate's source and writes
//! the API description; `gen <api.json> <spike dir>` writes the C header,
//! the Node glue, the TypeScript surface, the blob decoder and the Dart
//! layer from it, each into its binding's directory.

// A tooling binary: it reports through stdout and stderr and stops on a
// bad input, so the library lints against printing, panicking and indexing
// are allowed here (as in xtask); generators are long template functions.
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::too_many_lines,
    reason = "a tooling binary made of templates"
)]

mod common;
mod extract;
mod gen_c;
mod gen_dart;
mod gen_node;
mod gen_ts;
mod model;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let code = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["extract", source, out] => run_extract(source, out),
        ["gen", api, dir] => run_gen(api, dir),
        _ => {
            eprintln!(
                "usage: teistro-spike-a-gen extract <lib.rs> <api.json> | gen <api.json> <spike dir>"
            );
            2
        }
    };
    process::exit(code);
}

fn run_extract(source: &str, out: &str) -> i32 {
    let text = match fs::read_to_string(source) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("cannot read {source}: {err}");
            return 1;
        }
    };
    match extract::extract(source, &text) {
        Ok(api) => {
            let json = serde_json::to_string_pretty(&api).expect("the description serialises");
            if let Err(err) = fs::write(out, format!("{json}\n")) {
                eprintln!("cannot write {out}: {err}");
                return 1;
            }
            println!(
                "{}: {} enums, {} structs, {} callbacks, {} opaques, {} functions, {} blob sections",
                out,
                api.enums.len(),
                api.structs.len(),
                api.callbacks.len(),
                api.opaques.len(),
                api.functions.len(),
                api.blob.sections.len()
            );
            0
        }
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn run_gen(api_path: &str, dir: &str) -> i32 {
    let text = match fs::read_to_string(api_path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("cannot read {api_path}: {err}");
            return 1;
        }
    };
    let api: model::Api = match serde_json::from_str(&text) {
        Ok(api) => api,
        Err(err) => {
            eprintln!("{api_path}: {err}");
            return 1;
        }
    };
    let dir = Path::new(dir);
    let outputs = [
        (dir.join("a-c/teistro_spike.h"), gen_c::render(&api)),
        (
            dir.join("a-node/native/src/generated.rs"),
            gen_node::render(&api),
        ),
        (
            dir.join("a-node/lib/generated.d.ts"),
            gen_ts::render_types(&api),
        ),
        (dir.join("a-node/lib/blob.js"), gen_ts::render_decoder(&api)),
        (
            dir.join("a-dart/lib/src/generated.dart"),
            gen_dart::render(&api),
        ),
    ];
    let mut lines = 0usize;
    for (path, content) in &outputs {
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                eprintln!("cannot create {}: {err}", parent.display());
                return 1;
            }
        }
        if let Err(err) = fs::write(path, content) {
            eprintln!("cannot write {}: {err}", path.display());
            return 1;
        }
        let count = content.lines().count();
        lines += count;
        println!("{:>6} lines  {}", count, path.display());
    }
    println!("{lines:>6} lines generated");
    0
}

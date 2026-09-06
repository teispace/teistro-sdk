//! Runs one section of the fixed scenario, once, and reports how many
//! values it computed.
//!
//! It exists to be measured: `cargo xtask bench` runs it under callgrind,
//! once per section and once doing nothing, and reports the difference as
//! the instructions that section costs. Everything a benchmark must not
//! have is absent — no arguments but the section's name, no input, no
//! clock, no allocation before the work — so that two runs of the same
//! build count the same instructions.
//!
//! ```sh
//! teistro-scenario nothing     # what the process costs before any work
//! teistro-scenario astro       # that, plus the astronomy section
//! ```

// A measurement binary: it reports through stdout and exits on a name it
// does not know, so the library lints against printing are allowed here
// and in no library crate.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::hint::black_box;

use teistro_scenario::{SECTIONS, section};

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(name) = args.next() else {
        eprintln!(
            "usage: teistro-scenario <nothing | {}>",
            SECTIONS.join(" | ")
        );
        std::process::exit(2);
    };
    // The empty run: everything this process does except the work, which
    // is what `cargo xtask bench` subtracts from every other run.
    if name == "nothing" {
        println!("nothing 0");
        return;
    }
    let Some(section) = section(&name) else {
        eprintln!(
            "no section named `{name}`; the scenario has {}",
            SECTIONS.join(", ")
        );
        std::process::exit(2);
    };
    // Printed as well as black-boxed: a value the program never observes
    // is a value the optimiser may decide not to compute.
    let section = black_box(section);
    println!("{} {}", section.name, section.values.len());
}

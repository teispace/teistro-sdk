//! Compiles the Swiss Ephemeris library from the sources named by
//! `SWEPH_SRC_DIR` and links it statically. The sources are not vendored
//! here and never will be (ADR-0019): the adapter is built only where a
//! copy of the library and its licence terms are already present.

#![allow(clippy::panic, reason = "a build script reports by failing")]

use std::path::PathBuf;

/// The library's translation units, as its own makefile lists them.
const UNITS: [&str; 9] = [
    "swecl.c",
    "swedate.c",
    "swehel.c",
    "swehouse.c",
    "swejpl.c",
    "swemmoon.c",
    "swemplan.c",
    "sweph.c",
    "swephlib.c",
];

fn main() {
    println!("cargo:rerun-if-env-changed=SWEPH_SRC_DIR");
    let Some(dir) = std::env::var_os("SWEPH_SRC_DIR").map(PathBuf::from) else {
        panic!(
            "SWEPH_SRC_DIR must name a directory holding the Swiss Ephemeris C sources; \
             the adapter does not vendor them (ADR-0019)"
        );
    };
    let mut build = cc::Build::new();
    for unit in UNITS {
        let path = dir.join(unit);
        assert!(
            path.is_file(),
            "{} is not a file; SWEPH_SRC_DIR must hold the library's sources",
            path.display()
        );
        println!("cargo:rerun-if-changed={}", path.display());
        build.file(path);
    }
    build
        .include(&dir)
        .warnings(false)
        .opt_level(2)
        .compile("swe");
}

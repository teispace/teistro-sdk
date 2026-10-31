//! Compiles the digest's C reader for the `digest` target, instrumented
//! when `cargo fuzz` is building, so libFuzzer sees inside it.

fn main() {
    const SOURCE: &str = "../bindings/c/digest/teistro_digest.c";
    println!("cargo:rerun-if-changed={SOURCE}");
    println!("cargo:rerun-if-changed=../bindings/c/digest/teistro_digest.h");
    let mut build = cc::Build::new();
    build.file(SOURCE).include("../bindings/c/digest");
    if std::env::var_os("CARGO_CFG_FUZZING").is_some() {
        // Apple's clang writes an ASan its own runtime alone answers, and
        // rustc links LLVM's: on a Mac the C is covered and not sanitized,
        // which `check-c`'s sanitizer run makes good.
        let macos = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "macos");
        build.flag(if macos {
            "-fsanitize=fuzzer-no-link"
        } else {
            "-fsanitize=fuzzer-no-link,address"
        });
    }
    build.compile("teistro_digest");
}

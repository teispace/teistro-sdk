//! Builds the SDK's locale bundles from `i18n/` into `OUT_DIR`, so the C
//! library carries every shipped locale and a consumer needs no files to
//! render the SDK's own messages (ADR-0010). `bundles.rs` lists them for
//! `include_bytes!`.

// A build script stops the build by panicking and speaks to cargo through
// stdout, so the library lints against both are allowed here.
#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    clippy::panic,
    reason = "a build script"
)]

use std::fmt::Write as _;
use std::path::PathBuf;

use teistro_intl::pack::build_bundle;
use teistro_intl::source::Tree;

fn main() {
    let root = teistro_intl::sdk_root();
    println!("cargo:rerun-if-changed={}", root.display());
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let tree = Tree::load(&root).unwrap_or_else(|e| panic!("the i18n/ sources load: {e}"));
    let mut entries = String::new();
    for (tag, locale) in &tree.locales {
        let bytes = build_bundle(locale).unwrap_or_else(|e| panic!("{tag}: {e}"));
        let path = out.join(format!("{tag}.tbundle"));
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let _ = writeln!(
            entries,
            "    ({tag:?}, include_bytes!({:?})),",
            path.display()
        );
    }
    let listing = format!(
        "/// The SDK's locales as bundles, built from `i18n/` when the crate is compiled.\npub(crate) static BUNDLES: &[(&str, &[u8])] = &[\n{entries}];\n"
    );
    let path = out.join("bundles.rs");
    std::fs::write(&path, listing).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

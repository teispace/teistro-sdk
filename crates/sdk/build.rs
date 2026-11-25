//! Builds the SDK's locale bundles from `i18n/` into `OUT_DIR`, so a Rust
//! consumer needs no files to render the SDK's own messages (ADR-0010).
//! `bundles.rs` lists them for `include_bytes!`.
//!
//! The sources are `teistro-intl`'s own files (`crates/intl/i18n`), as
//! that crate embedded them when it was compiled
//! ([`teistro_intl::source::Tree::sdk`]), so this script reads no
//! directory at all: not one outside the packages a consumer compiles
//! (`cargo xtask check-packages` builds this crate from its `.crate`
//! alone), and not one a build reused across checkouts sharing a target
//! directory remembers from a checkout since deleted (SR-135). The
//! bundles change only when `teistro-intl` does, and cargo runs a build
//! script again when one of its build dependencies is rebuilt.
//!
//! **The same thirty lines as `crates/ffi/build.rs`, knowingly**, and for
//! as long as the order of work in `03-design/rust-consumer-surface.md`
//! says: a build script cannot use the crate it builds, so `teistro-intl`
//! cannot compile its own bundles, and the two crates that need them
//! compile them apiece until the boundary reads this crate's. What is
//! *not* duplicated is the rest of that script — the commit, the target
//! and the profile a binding checks a library against describe the C
//! library and nothing here wants them.

// A build script stops the build by panicking and speaks to cargo
// through stdout, so the library lints against both are allowed here.
#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    clippy::panic,
    reason = "a build script"
)]

use std::fmt::Write as _;
use std::path::PathBuf;

use teistro_intl::pack::{TEXT_SWITCHES, TextSwitch, build_bundle, without};
use teistro_intl::source::Tree;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let tree = Tree::sdk().unwrap_or_else(|e| panic!("the i18n/ sources load: {e}"));
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("the package's own manifest");
    // The texts this build cannot say, which no bundle carries (core#241).
    let silent: Vec<TextSwitch> = TEXT_SWITCHES
        .into_iter()
        .filter(|switch| !said(switch, &manifest))
        .collect();
    let mut entries = String::new();
    let mut left_out = String::new();
    for (tag, locale) in &tree.locales {
        if !embedded(tag, &manifest) {
            let _ = write!(left_out, "{tag:?}, ");
            continue;
        }
        let bytes =
            build_bundle(&without(locale, &silent)).unwrap_or_else(|e| panic!("{tag}: {e}"));
        let path = out.join(format!("{tag}.tbundle"));
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let _ = writeln!(
            entries,
            "    ({tag:?}, include_bytes!({:?})),",
            path.display()
        );
    }
    let listing = format!(
        "/// The SDK's locales as bundles, built from `i18n/` when the crate is compiled.\npub(crate) static BUNDLES: &[(&str, &[u8])] = &[\n{entries}];\n\n/// The SDK's locales this build leaves out, by their `locale-` features.\npub(crate) static LEFT_OUT: &[&str] = &[{left_out}];\n"
    );
    let path = out.join("bundles.rs");
    std::fs::write(&path, listing).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

/// The locale every other falls back on, embedded in every build.
const ROOT: &str = "en-Latn";

/// The feature that embeds `tag`: `locale-` and the tag in lower case.
fn feature(tag: &str) -> String {
    format!("locale-{}", tag.to_ascii_lowercase())
}

/// Whether a feature of this package is on. One the manifest does not
/// declare stops the build, so a switch cannot be named and not wired.
fn on(feature: &str, manifest: &str) -> bool {
    assert!(
        manifest.contains(&format!("\n{feature} = [")),
        "no feature `{feature}` in crates/sdk/Cargo.toml"
    );
    let var = format!(
        "CARGO_FEATURE_{}",
        feature.to_ascii_uppercase().replace('-', "_")
    );
    println!("cargo:rerun-if-env-changed={var}");
    std::env::var_os(var).is_some()
}

/// Whether this build says a text: its switch on, and its family's too.
fn said(switch: &TextSwitch, manifest: &str) -> bool {
    on(switch.feature, manifest) && switch.family.is_none_or(|family| on(family, manifest))
}

/// Whether this build embeds `tag`: the root always, any other when its
/// feature is on. A locale with no feature declared in the manifest stops
/// the build, so every locale `teistro-intl` carries can be left out.
fn embedded(tag: &str, manifest: &str) -> bool {
    if tag == ROOT {
        return true;
    }
    let feature = feature(tag);
    assert!(
        manifest.contains(&format!("\n{feature} = [")),
        "{tag} has no feature `{feature}` in crates/sdk/Cargo.toml: declare it and add it to `locales`"
    );
    on(&feature, manifest)
}

//! The `teistro-intl` commands over the SDK's own `i18n/` tree, each run
//! as the command line parses it, writing under the test's own directory
//! and never into the tree. What a command writes from the tree must be
//! what the tree already holds: the derived `sa-Latn` and an XLIFF round
//! trip of `hi-Deva-IN`.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use std::path::{Path, PathBuf};

use clap::Parser as _;
use teistro_intl::cli::{Cli, run};
use teistro_intl::sdk_root;

/// A fresh directory of the test's own.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("intl-cli")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Runs `teistro-intl` with `args`, and whether the command passed.
fn intl(args: &[&str]) -> bool {
    let cli = Cli::try_parse_from(std::iter::once("teistro-intl").chain(args.iter().copied()))
        .unwrap_or_else(|e| panic!("{args:?}: {e}"));
    run(cli).unwrap_or_else(|e| panic!("{args:?}: {e}"))
}

fn text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn path(dir: &Path) -> &str {
    dir.to_str().unwrap()
}

#[test]
fn the_tree_validates_builds_generates_and_reports() {
    assert!(intl(&["validate"]));
    let out = scratch("build");
    assert!(intl(&["build", "--out", path(&out)]));
    assert_ne!(
        std::fs::metadata(out.join("en-Latn.sdk.entity.tpack"))
            .unwrap()
            .len(),
        0
    );
    let bundles = scratch("bundles");
    assert!(intl(&[
        "build",
        "--bundle",
        "--locales",
        "en-Latn",
        "--out",
        path(&bundles)
    ]));
    let generated = scratch("gen");
    assert!(intl(&[
        "gen",
        "--target",
        "ts,dart,rs",
        "--out",
        path(&generated)
    ]));
    for file in ["ts/sdk.ts", "dart/lib/sdk.dart", "rs/messages.rs"] {
        assert!(!text(&generated.join(file)).is_empty(), "{file}");
    }
    let report = scratch("report");
    assert!(intl(&["report", "--out", path(&report)]));
    assert!(text(&report.join("intl.json")).contains("en-Latn"));
}

#[test]
fn a_key_renders_and_text_transliterates() {
    let key = "sdk.condition.vargottama";
    assert!(intl(&[
        "render",
        "--locale",
        "en-Latn",
        key,
        "--param",
        "graha=@graha.SUN"
    ]));
    // A parameter left out is a warning, which fails the command.
    assert!(!intl(&["render", "--locale", "en-Latn", key]));
    assert!(intl(&["translit", "--from", "deva", "--to", "iast", "सूर्य"]));
    let refused =
        run(Cli::try_parse_from(["teistro-intl", "translit", "--from", "cyrl", "x"]).unwrap());
    assert!(refused.is_err());
}

#[test]
fn derivation_writes_the_derived_locale_the_tree_holds() {
    let out = scratch("derive");
    assert!(intl(&[
        "derive",
        "--from",
        "sa-Deva",
        "--to",
        "sa-Latn",
        "--out",
        path(&out)
    ]));
    let shipped = sdk_root().join("sa-Latn/sdk.entity.json");
    assert_eq!(text(&out.join("sa-Latn/sdk.entity.json")), text(&shipped));
}

#[test]
fn an_exported_locale_imports_back_unchanged() {
    let dir = scratch("xliff");
    let file = dir.join("hi.xlf");
    assert!(intl(&[
        "export",
        "xliff",
        "--locale",
        "hi-Deva-IN",
        "--out",
        path(&file)
    ]));
    assert!(intl(&["import", "xliff", path(&file), "--dry-run"]));
    assert!(intl(&["import", "xliff", path(&file), "--out", path(&dir)]));
    let shipped = sdk_root().join("hi-Deva-IN");
    let mut compared = 0;
    for entry in dir.join("hi-Deva-IN").read_dir().unwrap() {
        let written = entry.unwrap().path();
        let name = written.file_name().unwrap();
        assert_eq!(text(&written), text(&shipped.join(name)), "{name:?}");
        compared += 1;
    }
    assert_ne!(compared, 0);
}

#[test]
fn extraction_scaffolds_a_locale_from_the_base() {
    let out = scratch("extract");
    assert!(intl(&[
        "extract",
        "--locale",
        "ta-Taml-IN",
        "--namespaces",
        "sdk.entity",
        "--out",
        path(&out)
    ]));
    assert!(text(&out.join("ta-Taml-IN/sdk.entity.json")).contains("Sun"));
}
